# TASK-061 · Revisar el techo de magnitud de la escala entera (ADR-0141)

| Campo | Valor |
|---|---|
| **ID** | TASK-061 |
| **Tipo** | Task (decisión de arquitectura) |
| **Responsable** | Architect |
| **Estado** | 🟡 Abierta — pendiente de despacho |
| **Creada** | 2026-07-19 |
| **Origen** | Quant-Engineer, Etapa 6 de STORY-051; aritmética verificada por el Tech-Lead |
| **Resuelve** | DEBT-033 |

## Directiva del propietario (vinculante, 2026-07-19)

> *"Mi intención no es descartar esos activos. Mientras más activos podamos operar, mejor. 100%."*

**Queda descartada de entrada la opción de declarar la clase de activo fuera de alcance.** La decisión es *cómo* soportarla, no *si* se soporta.

## El problema

ADR-0141 fija la escala entera ×10⁸ y la justifica por **precisión decimal**: "cubre crypto (8 dec.) y forex (5 dec.) sin overflow en `i64`". La justificación es correcta para el problema que resolvía — pero responde a la pregunta de **cuántos decimales**, no a la de **cuánta magnitud**. Son preguntas distintas y solo se contestó una.

Un `i64` con escala ×10⁸ solo representa **~92 200 millones de unidades**. Verificación aritmética (Tech-Lead, 2026-07-18):

| Escenario | Unidades en una barra de 1h | Contra el techo |
|---|---|---|
| SHIB a ~$0.00002, 50 M USD/día (volumen conservador) | ~104 200 millones | **1,13× — desborda** |
| SHIB en día activo, 200 M USD/día | ~416 700 millones | **4,52× — desborda** |
| PEPE a ~$0.000012, 30 M USD/día | ~104 200 millones | **1,13× — desborda** |

No es un extremo teórico: **el volumen de un día cualquiera desborda en una sola barra horaria**, y estos activos se negocian en la misma fuente que usamos como muestra real.

Hoy el sistema falla seguro (`checked_add` → error explícito, sin corrupción), pero rechaza sistemáticamente.

## Qué debe decidir el Architect

### 1. Re-derivar el supuesto contra rango, no solo contra precisión

Declarar explícitamente qué combinaciones de precio unitario y oferta circulante debe soportar el sistema. Es la comprobación que ADR-0141 nunca hizo.

### 2. Elegir la salida, con su coste real

Las opciones **no** son equivalentes y el Architect debe pronunciarse con argumentos, no elegir la más fácil:

- **Volumen en `i128`, precio en `i64`.** El precio no tiene problema de magnitud (ningún activo cuesta 92 000 millones); el volumen sí. Separar los dos anchos ataca exactamente el problema. Coste: SQLite no tiene entero de 128 bits nativo — hay que decidir la representación en persistencia (dos columnas, texto, o `BLOB`), y eso toca el modelo de datos, Parquet y el puente FFI.
- **Escala variable por instrumento.** Precisión adaptada a cada activo. Coste alto y peligroso: la escala pasa a ser un dato por fila, y comparar dos series exige normalizar. Contradice el espíritu de "escala fija para todo el sistema" de ADR-0141.
- **Escala menor para volumen que para precio.** El volumen rara vez necesita 8 decimales. Bajarlo a ×10² o ×10⁰ para volumen multiplica el techo por 10⁶ u 10⁸. Coste bajo, pero hay que verificar que ningún activo negocie fracciones más finas que la escala elegida.

**Recomendación del Tech-Lead, no vinculante:** la tercera opción parece la de mejor relación coste/beneficio y merece evaluarse primero — pero es el Architect quien decide, tras comprobar si existe algún activo real que negocie fracciones de unidad por debajo de la escala que proponga.

### 3. Alcance de la corrección

Qué se toca: ADR-0141, el esquema canónico de ADR-0155, las migraciones afectadas (fase GREENFIELD: se editan in-situ), el crate del transformador, el puente y la interfaz. Estimar el radio antes de despachar implementación.

### 4. Si alguna combinación queda fuera pese a todo

Decirlo **donde el usuario lo vea**. La doctrina es que el usuario trae sus propios datos (ADR-0154); si trae un activo que no podemos procesar, merece un mensaje que lo explique, no un error genérico de desbordamiento.

## Disparador

Antes de habilitar ingesta real de cripto de oferta extrema, y antes de cualquier promesa pública sobre cobertura de mercados.

---

## Análisis del Quant-Engineer (2026-07-19, primer despacho — insumo para el Architect, no reemplaza su decisión)

### 1. Re-derivación independiente de la aritmética

Confirmada, sin corrección material. `i64::MAX / 10⁸ = 92 233 720 368,547...` → techo real **92 233 720 368** unidades (piso entero; el redondeo a `...369` de DEBT-033/TASK-061 es un error de un dígito sin efecto en la conclusión).

- SHIB conservador: `50 000 000 / 0,00002 = 2,5 × 10¹² unidades/día`; `/24 = 104 166 666 667`/hora → **1,1294×** el techo. Confirma "1,13×".
- SHIB activo: `200 000 000 / 0,00002 = 10¹³/día`; `/24 = 416 666 666 667`/hora → **4,518×**. Confirma "4,52×".
- PEPE: `30 000 000 / 0,000012 = 2,5 × 10¹²/día`; `/24 = 104 166 666 667`/hora → **1,1294×**. Idéntico a SHIB conservador (misma proporción USD/precio).

**Evidencia empírica externa (Binance `exchangeInfo`, verificado en vivo):** `SHIBUSDT` LOT_SIZE `maxQty = 46 116 860 414`, `stepSize = 1`; `PEPEUSDT` LOT_SIZE `maxQty = 92 233 720 368` — **exactamente el mismo techo** que `i64::MAX/10⁸`. Binance, con muchos más recursos de ingeniería, tropieza con el mismo límite de 64 bits escalado y lo resuelve **acotando la cantidad de UNA orden**, no ensanchando el ancho — porque su problema es por-orden, no por-barra agregada. `stepSize = 1` en ambos confirma que estos dos activos concretos no negocian fracciones de unidad (ver punto 3).

### 2. Evaluación cuantitativa de las opciones

- **`i128` para volumen agregado.** Techo en unidades naturales ≈ `1,70 × 10³⁰` — ~1,8 × 10¹⁹ veces el escenario SHIB extremo. Sobrevive la prueba de límite degenerado (`critical-domain-reasoning.md` comprobación 1): empujar oferta circulante → cualquier valor real conocido (SHIB ≈ 5,89 × 10¹⁴ unidades totales) queda 15 órdenes de magnitud por debajo del nuevo techo. Precisión exacta preservada (aritmética entera, mismo patrón que `checked_scaled_product`/`round_half_to_even_div_i128` ya existentes en `domain.rs`, que **ya** operan en `i128` para productos intermedios). Costo real, verificado: `flutter_rust_bridge 2.12` **sí soporta `i128`/`u128`**, mapeados a `BigInt` en Dart — mismo mecanismo que `u64` hoy (no es bloqueante). El costo genuino está en persistencia: SQLite no tiene entero nativo de 128 bits (opciones: `TEXT` decimal — pierde aritmética SQL nativa pero es exacta; dos columnas `INTEGER` hi/lo — exacta pero obliga a todo consumidor a recombinar; `BLOB` de 16 bytes — opaco). DuckDB sí tiene `HUGEINT` nativo (128 bits) sin costo. Parquet/Polars: `Decimal128`/`Int128` existen pero deben verificarse contra la versión de Polars en uso antes de comprometerse (pregunta abierta para Rust-Engineer/Bridge).
- **Escala variable por instrumento.** Rechazada por rigor cuantitativo, no solo por costo: convierte la escala en metadato por fila, obliga a normalizar en cada comparación/agregación/ratio entre series, y es exactamente la clase de ambigüedad silenciosa que ADR-0155 prohíbe para campos derivados ("guarda el hecho, calcula la interpretación") aplicada aquí a la codificación misma del hecho. Confirmo el rechazo del Tech-Lead.
- **Escala menor para volumen que para precio (hipótesis del TL).** Ver punto 3.

### 3. Veredicto sobre "escala menor para volumen que para precio"

**Direccionalmente correcta, pero no sobrevive la prueba de límite degenerado y depende de un supuesto no verificado universalmente.**

- Es una constante GLOBAL fija (no por instrumento), así que no reintroduce el peligro de la opción rechazada — eso es correcto.
- El fundamento económico es real y lo confirma la evidencia de Binance: `SHIBUSDT` y `PEPEUSDT` tienen `stepSize = 1` (unidades enteras, sin fracciones) — precisamente porque un activo de precio ínfimo hace que una fracción de unidad no tenga valor negociable. Pero esto es una **regularidad empírica del diseño de los exchanges (filtros anti-polvo), no una ley matemática**: la búsqueda confirma que Binance permite `stepSize` fraccionario en otros símbolos (`0,001`, etc.), así que no se puede asumir sin verificar. Adoptar una escala reducida sin auditar el `stepSize` real de **todo** el universo de activos en alcance repite, en sentido inverso, el error exacto de ADR-0141: revisó precisión sin revisar magnitud; una escala reducida sin auditoría revisaría magnitud (para SHIB/PEPE) sin re-verificar precisión (para el resto del universo, presente y futuro).
- Empujando el parámetro al extremo (comprobación 1 de `critical-domain-reasoning.md`): cualquier escala fija más pequeña sigue teniendo un techo finito; existe, por construcción, un activo hipotético con oferta aún mayor o precio aún más ínfimo que lo desborda igual — el problema se pospone, no se cierra. `i128` sí sobrevive ese mismo empuje con margen de ~19 órdenes de magnitud sobre el peor caso real conocido.
- **Recomendación cuantitativa:** `i128` es la solución que cierra el problema de forma permanente; una escala reducida, si se adopta, debe tratarse como optimización complementaria tras auditar `stepSize` real del universo de activos — nunca como sustituto que evite el ensanche de ancho.

### 4. Riesgo de frontera `f64`/2⁵³ al cruzar el puente o la UI

Hallazgo no solicitado explícitamente en la aritmética original pero crítico para cualquier opción elegida: el techo de precisión exacta de `f64` es `2⁵³ ≈ 9,007 × 10¹⁵`.

- El valor **desescalado** (unidades naturales, ej. 416 666 666 667 SHIB/hora) cabe cómodo en `f64` exacto — no hay riesgo en mostrarlo en UI ya desescalado.
- El riesgo real es cualquier ruta que convierta el entero **ya escalado** (×10⁸) directamente a `f64` sin desescalar primero: `4,17 × 10¹¹ × 10⁸ = 4,17 × 10¹⁹`, muy por encima de `2⁵³`. Esto ya es cierto hoy con `i64` para un volumen agregado cercano al techo, y se vuelve más probable de tropezar cuanto más ancho el acumulador (`i128`).
- Riesgo secundario, menos obvio: sumas acumuladas de volumen (ej. volumen total de un backtest de un año, o de un portafolio) cruzando a `DOUBLE` en DuckDB o a `f64` en un modelo de impacto de mercado, pueden acercarse a `2⁵³` incluso hoy sin que ningún campo individual desborde.
- **Regla a fijar (cualquiera sea la opción elegida):** ninguna ruta de código convierte el entero **escalado** a `f64`; toda conversión a punto flotante ocurre después de desescalar (dividir por 10⁸ con aritmética entera), y toda suma acumulada de alto volumen se mantiene en dominio entero (`i128`/`HUGEINT`) hasta el último punto de presentación.

### 5. Asimetría precio/volumen — qué NO rompe y qué sí exige

- No rompe comparabilidad ni agregación **dentro** de un mismo campo (sumar volumen con volumen, a la misma escala, es homogéneo).
- Si se adopta escala distinta para precio y volumen, toda fórmula que **mezcle** ambos (nocional = precio × volumen, barras `DOLLAR`, VWAP, estimadores de impacto de mercado) deja de tener un solo factor de reescalado universal — pasa a depender de los dos exponentes. Esto es manejable pero exige **un único conversor canónico** (mismo patrón que `checked_scaled_product`/`round_half_to_even_div_i128`), nunca aritmética de reescalado in-line en cada sitio — de lo contrario es una clase de bug silenciosa (error de escala, no de crash) sobre datos financieros.
- Conecta directamente con TASK-062: el tipo de barra `DOLLAR` (si sobrevive, ver ese análisis) es exactamente el caso que más sufre esta asimetría, porque acumula precio×volumen a lo largo de toda la sesión.
- **Hallazgo con valor propio:** `quote_asset_volume` (volumen en moneda de cotización, ej. USDT) **no comparte el problema de magnitud** — está denominado en una moneda de precio "normal", nunca necesita 92 200 millones de unidades. El ensanche a `i128` aplica solo a los campos denominados en el activo base (`volume_traded`, `volume_buy`); `trade_volume_scaled` de `TradeTick` (una sola ejecución, no una suma) tampoco lo necesita — el propio `maxQty` de Binance para `PEPEUSDT` (`92 233 720 368`, el techo exacto de `i64×10⁸`) confirma que una orden individual nunca se acerca al problema; el riesgo es estrictamente de la **suma** (`Bars.volume_traded`/`volume_buy`), no del dato crudo por ejecución.

### Preguntas abiertas para el Architect

1. ¿Se acepta el costo de infraestructura de `i128` (representación SQLite — `TEXT`/dos columnas/`BLOB` — y verificación de soporte `Decimal128`/`Int128` en la versión de Polars en uso) a cambio del cierre permanente y robusto del problema?
2. Si se combina `i128` con una escala reducida como optimización adicional, ¿quién audita el `stepSize`/LOT_SIZE real de cada activo del universo en alcance antes de fijar el exponente, para no repetir en sentido inverso el error de ADR-0141?
3. ¿El ensanche aplica solo a `Bars.volume_traded`/`volume_buy` (agregados) o también a otros campos de `ADR-0155` que acumulan sobre una ventana (`quote_asset_volume` se descarta por el punto 5; `open_interest`/`settlement_price` no acumulan, son snapshots — probablemente sin riesgo, confirmar)?
4. Alcance de archivos a tocar (ADR-0141, ADR-0155, migración `0021_hybrid_data_transformer.sql`, `domain.rs`/`schemas.rs`/`persistence.rs` de `hybrid-data-transformer`, el puente `crates/bridge/src/api/hybrid_data_transformer.rs`, `ui/lib/src/rust/api/hybrid_data_transformer.dart`) — corresponde al Architect estimar el radio real y decidir si es Story o Task de esquema.

---

## Resolución del Architect (2026-07-19)

**Decisión sellada:** `Bars.volume_traded`/`volume_buy` ensanchan `i64`→`i128`; escala ×10⁸ sin cambio; nada más se toca. Costo de infraestructura verificado (no solo estimado): SQLite no interviene (el dato vive en Parquet, no en la única tabla SQLite del dominio); Polars 0.54 soporta `Int128` tras activar el feature `dtype-i128` (no bloqueante); DuckDB tiene `HUGEINT` nativo; `flutter_rust_bridge` 2.12 soporta `i128`→`BigInt` nativamente (verificado en código fuente de ambos crates). Alcance real: ADR-0141 (enmendado), ADR-0155 (tabla de esquema `Bars` enmendada), `Cargo.toml`+`domain.rs`+`schemas.rs`+`persistence.rs` de `hybrid-data-transformer`, `BarDto` del Bridge + regeneración mecánica de FFI. **Ninguna migración SQL cambia.** Es Story de esquema (pendiente de que el Tech-Lead la abra con el siguiente número global disponible), no una decisión abierta. Documentos canonizados y las 7 comprobaciones de dominio: `.agents/plans/architect-2026-07-19-escala-barras-ingesta.md`.
