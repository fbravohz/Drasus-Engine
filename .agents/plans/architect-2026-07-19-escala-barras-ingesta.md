# Plan del Architect — Escala entera, catálogo de barras y formato de ingesta (TASK-061/062/063)

**Fecha:** 2026-07-19 · **Autor:** Architect · **Insumo:** Quant-Engineer (Etapa 0.3, análisis íntegro citado en cada TASK) + General Counsel (LDR-0008)

**Conocimiento consultado:** `.agents/knowledge/base.md` §7.4 (Gate Doc-First) — aplicado antes de cada decisión; `.agents/knowledge/critical-domain-reasoning.md` — las 7 comprobaciones se corrieron para TASK-061 (ya en el análisis del Quant, ratificadas) y para el ADR-0156 nuevo (sección propia); `.agents/knowledge/WRITING-EFFICIENCY.MD` — aplicado a este documento y a las ediciones de `docs/`.

---

## 1. Resumen ejecutivo

Las tres decisiones quedan selladas y canonizadas en `docs/`. Ninguna requirió pausar y escalar al propietario — las directivas vinculantes ya existían (no descartar activos, D1–D4, veredicto legal); el trabajo del Architect fue re-derivar el rango real, verificar el costo de infraestructura que el Quant no podía evaluar, cerrar el catálogo de barras con criterio explícito, y ensamblar D1–D4 con el veredicto legal en un contrato coherente.

- **TASK-061:** `Bars.volume_traded`/`volume_buy` ensanchan de `i64` a `i128`. La escala ×10⁸ no cambia. Verificado (no solo estimado): SQLite no interviene, Polars lo soporta tras un feature flag, DuckDB y el puente FFI ya lo soportan sin costo.
- **TASK-062:** catálogo cerrado en seis tipos (`TIME`/`TICK_COUNT`/`VOLUME`/`RENKO`/`RANGE`/`DOLLAR`), con criterio de admisión explícito. Heikin Ashi confirmado como indicador. Kagi/Point & Figure/Line Break quedan fuera, con disparador de reapertura escrito.
- **TASK-063:** rechazo por defecto + transformación con vista previa como acción explícita del usuario (no son alternativas, son las dos caras de la misma política). Nuevo ADR-0156 ensambla D1/D2/D4 con las tres condiciones técnicas del veredicto legal. D3 (doctrina compra/venta) se ancla donde ya vivía la doctrina de ejecución: ADR-0114.

**Ningún bloqueo duro de producto/negocio.** Una contradicción documental real se encontró y se corrigió en el mismo movimiento (`data-import-wizard.md` contradecía D2 directamente — ver §4).

---

## 2. TASK-061 — Techo de magnitud de la escala entera

### 2.1 Decisión sellada

**El ancho del acumulador ensancha; la escala no.** `Bars.volume_traded` y `Bars.volume_buy` (las dos magnitudes que SUMAN ejecuciones dentro de una barra) pasan de `i64` a `i128`, con la misma escala ×10⁸ de siempre. Todo lo demás del sistema —precios, `QuoteTick`/`TradeTick` completos, `quote_asset_volume`, contadores, snapshots de Futuros/Opciones— permanece `i64` sin cambio, porque ninguno de esos campos suma ejecuciones dentro de una ventana: una ejecución individual nunca se acerca al techo (el propio `maxQty` de Binance lo confirma empíricamente), y `quote_asset_volume` está denominado en moneda de cotización, no en unidades del activo base.

**Fundamento canonizado en:** `docs/adr/ADR-0141.md` §"Ensanche de ancho para volumen acumulado" (enmienda 2026-07-19).

### 2.2 Por qué esta opción y no las otras dos

- **Escala reducida para volumen (rechazada):** no sobrevive la prueba de límite degenerado (`critical-domain-reasoning.md` comprobación 1) — cualquier escala fija más pequeña sigue teniendo un techo finito, y exige auditar `stepSize`/LOT_SIZE de todo el universo de activos presente y futuro antes de fijar el exponente, repitiendo en sentido inverso el error original de ADR-0141 (revisar magnitud sin re-verificar precisión).
- **Escala variable por instrumento (rechazada):** convierte la escala en metadato por fila, obliga a normalizar en cada comparación entre series, contradice "escala fija para todo el sistema" (ADR-0141, sin cambio).
- **`i128` (elegida):** techo ≈ 1,70 × 10³⁰ unidades — ~19 órdenes de magnitud sobre el peor caso real conocido (SHIB en día activo). Sobrevive la prueba de límite degenerado sin reservas. Reutiliza un patrón que el código YA tiene (`checked_scaled_product`/`round_half_to_even_div_i128` en `domain.rs` operan en `i128` para productos intermedios) — no es una técnica nueva, es extender el mismo acumulador ancho a la persistencia.

### 2.3 El costo de infraestructura — lo que el Quant no podía evaluar, verificado, no estimado

| Capa | Hallazgo | Evidencia |
|---|---|---|
| **SQLite** | **No interviene.** `Bars`/`QuoteTick`/`TradeTick` viven en Parquet, no en SQLite (ADR-0155: "Grupo I no aplica por fila"). La única tabla SQLite del dominio (`transformation_records`, migración `0021`) guarda `rows_in`/`rows_out` — conteos, nunca la magnitud del volumen. | Leído `migrations/0021_hybrid_data_transformer.sql` completo: cero columnas de volumen. |
| **`sqlx-sqlite` 0.8.6** | Sin tipo `i128` nativo, confirmando que si algún día una tabla SQLite lo necesitara, la representación canónica es `TEXT` decimal (exacto, auditable), no `BLOB` ni split hi/lo. | `grep -rln "i128" .../sqlx-sqlite-0.8.6/src` → sin resultados de soporte. |
| **Polars 0.54** | Tipo `Int128`/`Decimal128` **existe**, pero detrás del feature flag `dtype-i128`, **no activo hoy** en el `Cargo.toml` del crate (`features = ["lazy", "parquet", "ndarray"]`). Costo: añadir el flag, recompilar. No bloqueante. | Leído `polars-core-0.54.4/src/datatypes/dtype.rs` y `polars-0.54.4/Cargo.toml` — `dtype-i128`/`dtype-decimal` confirmados como features opt-in. |
| **DuckDB** | `HUGEINT` nativo (128 bits), sin costo. | Hecho conocido de la biblioteca, no requiere verificación local — no participa en la ruta de escritura de este crate hoy. |
| **`flutter_rust_bridge` 2.12.0** | Soporta `i128` nativamente (`impl_into_into_dart_by_self!(i128)`), mapeado a `BigInt` en Dart — mismo mecanismo que `u64` ya usa hoy. **No bloqueante**, contra lo que TASK-061 dejaba como pregunta abierta. | `grep -n "i128" .../flutter_rust_bridge-2.12.0/src/misc/into_into_dart.rs` → confirmado. |

### 2.4 Riesgo de frontera `f64`/2⁵³ (hallazgo del Quant, ratificado)

Se ratifica sin corrección la regla que el Quant propuso: ninguna ruta de código convierte el entero **escalado** (×10⁸) directamente a `f64`; toda conversión a punto flotante ocurre después de desescalar con aritmética entera, y toda suma acumulada de alto volumen permanece en dominio entero (`i128`/`HUGEINT`) hasta el último punto de presentación. Queda como criterio de aceptación de la Story de implementación, no como código de este ADR.

### 2.5 Alcance de la corrección (más pequeño de lo previsto, porque SQLite no interviene)

`docs/adr/ADR-0141.md` (enmendado, ya aplicado) · `docs/adr/ADR-0155.md` (tabla de esquema `Bars`, ya aplicado) · `Cargo.toml` + `domain.rs` + `schemas.rs` + `persistence.rs` de `crates/features/data/hybrid-data-transformer` · `crates/bridge/src/api/hybrid_data_transformer.rs` (`BarDto`) · regeneración mecánica de `frb_generated.*`/`.dart`. **Cero migraciones SQL.** Clasificación: **Story de esquema** (código real que tocar), no Task de decisión — esa ya cerró aquí.

### 2.6 Respuesta a las 4 preguntas del Quant

1. **¿Se acepta el costo de infraestructura de `i128`?** Sí, y es menor de lo que la pregunta anticipaba: SQLite no interviene, Polars requiere solo un feature flag, DuckDB y el puente ya lo soportan. Aceptado.
2. **¿Quién audita `stepSize`/LOT_SIZE si se combina con escala reducida?** No aplica — se descartó la escala reducida (§2.2). No hay auditoría de universo de activos que hacer para esta decisión.
3. **¿El ensanche aplica solo a `volume_traded`/`volume_buy` o también a otros campos?** Solo a esos dos, confirmado. `quote_asset_volume` no comparte el problema de magnitud (denominado en moneda de cotización); `open_interest`/`settlement_price` son snapshots, no acumulan — confirmado sin riesgo, sin cambio.
4. **Alcance de archivos a tocar, Story o Task de esquema.** Resuelto en §2.5: es Story de esquema, radio menor al anticipado porque SQLite queda fuera.

---

## 3. TASK-062 — Catálogo de tipos de barra

### 3.1 Catálogo cerrado (único sitio canónico: `docs/features/algorithmic-bars.md`)

| `bar_type` | Naturaleza | Estado |
|---|---|---|
| `TIME` | Regla de cierre — intervalo fijo | Construido |
| `TICK_COUNT` | Regla de cierre — N transacciones | Especificado |
| `VOLUME` | Regla de cierre — umbral de volumen | Especificado |
| `RENKO` | Regla de cierre — tamaño de ladrillo | Especificado |
| `RANGE` | Regla de cierre — rango High-Low fijo | Especificado (faltaba en el `CHECK` de ADR-0155 pese a estar ya en `algorithmic-bars.md` — corregido) |
| `DOLLAR` | Regla de cierre — umbral de valor nocional (precio×volumen) | Especificado, implementación diferida hasta TASK-061 (asimetría precio/volumen) |

`ADR-0155.bar_type` `CHECK` corregido para coincidir exactamente con esta tabla.

### 3.2 Criterio de admisión (para no volver a discutir esto por cada tipo nuevo)

Un tipo entra al catálogo si cumple las tres: (a) es una regla de **cierre** computable causalmente desde el flujo de ticks, sin información posterior al cierre de la propia barra; (b) su salida encaja en el esquema `Bars` (OHLC + metadatos) sin inventar forma nueva; (c) respaldo verificable (literatura cuantitativa reconocida o adopción de plataforma mayor confirmada por fuente).

### 3.3 Heikin Ashi — confirmado indicador, no tipo de barra

Ratificada la tesis del Tech-Lead/Quant sin corrección: fórmula recursiva sobre OHLC de barras **ya cerradas** por otra regla — no decide cuándo cierra una barra, repinta valores de una que ya cerró. Falla el criterio (a). Misma doctrina que sacó el precio medio ponderado de la ingesta (DEBT-028). Pertenece al motor de indicadores (EPIC-2/EPIC-3, que no existe todavía) — no se crea una feature nueva para esto ahora; se documenta la exclusión y la advertencia de riesgo (precio Heikin Ashi ≠ precio negociable) en `algorithmic-bars.md`, igual que DEBT-028 dejó sus puntos 2–3 anotados sin construir feature.

### 3.4 Kagi / Point & Figure / Line Break — NO admitidos, decisión diferida (no bloqueo)

Confirmado que NinjaTrader 8 los ofrece. Fallan el criterio (b): su salida nativa es líneas/cajas sin eje de tiempo uniforme, no encajan en `Bars`. **Esto no es una contradicción documental ni un bloqueo de producto — es una decisión de arquitectura que el Architect puede resolver, y la resuelve diferir:** admitirlos exigiría diseñar un tipo de dato nuevo sin que exista hoy un consumidor concreto que lo necesite (ni el propietario lo pidió como requisito, solo preguntó si estaban contemplados). La Inundación de Fundaciones (ADR-0020 §7.3) aplica a campos dentro de un perfil ya decidido, no a arquitecturas de dato completas sin caso de uso. Disparador de reapertura escrito en `algorithmic-bars.md` y `ROADMAP.md`: necesidad concreta del propietario, o una Story de EPIC-8 que los requiera para paridad de UX.

"Order Flow Volumetric/Delta Bars" de NT8 **no es un tipo nuevo** — ya cubierto por `DepthLevel`/`OrderEvent` (ADR-0155), sin acción.

### 3.5 Barras de desequilibrio/run (López de Prado) — candidatas de ROADMAP

Anotadas en `algorithmic-bars.md` y `ROADMAP.md` como candidatas de mayor rigor estadístico, sin EPIC asignado — no las nombró el propietario, no se construyen ahora, pero quedan escritas para no perderse.

### 3.6 Advertencia de sesgo estadístico por tipo de barra

Trasladada íntegra del análisis del Quant a `algorithmic-bars.md` (Restricciones), para que llegue a cualquier feature futura de EPIC-4 (WFA/CPCV/Monte Carlo/DSR) que permita elegir la fuente de barras, sin depender de que alguien la redacte de memoria en ese momento.

### 3.7 Respuesta a las 4 preguntas del Quant

1. **¿Se acepta `DOLLAR` como quinto `bar_type`?** Sí — sexto, junto con `RANGE` que también faltaba. Implementación diferida a TASK-061, confirmado.
2. **¿Se admite Kagi/P&F/Line Break?** No, por ahora — ver §3.4. No es que se rechace para siempre: se difiere con disparador explícito.
3. **¿Se añaden imbalance/run bars al ROADMAP?** Sí, como candidatas sin EPIC asignado — ver §3.5.
4. **¿Quién redacta la advertencia de sesgo estadístico?** El Architect la traslada ahora a `algorithmic-bars.md` (§3.6) para que no se pierda; cuando exista la feature de validación que la consuma, el Quant-Engineer y el UI-Designer la adaptan a esa superficie concreta.

---

## 4. TASK-063 — Formato estándar de ingesta y doctrina compra/venta

### 4.1 Decisión 1 — ¿Rechazo puro o transformación con vista previa?

**No son alternativas: son las dos caras de una misma política, ya implícitas en D1+D2 del propietario.** Rechazo puro es el comportamiento **por defecto** ante un archivo sin plantilla reconocida y sin mapeo declarado. Transformación con vista previa existe como **acción explícita que el usuario elige**, nunca automática ni silenciosa — y solo protege legalmente a Drasus si cumple las cuatro condiciones del veredicto legal (vista previa de dato real, ledger inmutable, guardarraíles propios de Drasus corriendo antes de aceptar, plantillas con su propio control de calidad). Canonizado en **ADR-0156** (nuevo).

### 4.2 Hallazgo de coherencia — `data-import-wizard.md` contradecía D2 directamente

Antes de esta sesión, `docs/features/data-import-wizard.md` declaraba `AUTO_DETECT_COLUMNS = True` por defecto y "el sistema propone un mapeo automático" sin distinguir borrador de aceptación — exactamente el caso "adivinanza silenciosa" que el veredicto legal marca como de **mayor exposición**, y que D2 ya prohibía ("nunca mapeo automático por adivinanza de nombres de columna"). Es el tipo de hallazgo que la Etapa 0.4 existe para producir: un documento de feature nunca corregido tras una nueva directiva. **Corregido en el mismo movimiento** — la sugerencia automática se conserva como valor de UX (borrador que acelera al usuario) pero deja de ser aceptación; ver `docs/features/data-import-wizard.md` (Comportamientos Observables, Restricciones, Parámetros Configurables, Persistencia — todos actualizados).

### 4.3 Decisión 2 — Mecánica de trazabilidad (requisito, no preferencia — LDR-0008)

**Por referencia estable, nunca por duplicación.** El ledger de declaración de mapeo tiene su propio identificador; la partición Parquet resultante (vía `data_snapshot_id`, ya obligatorio por fila en ADR-0155) y la fila de `transformation_records` de la corrida apuntan a ese identificador. Cadena de cuatro saltos: `backtest → dataset (data_snapshot_id) → corrida (transformation_records) → declaración de mapeo (ledger nuevo)`. Ningún dato de la declaración se copia dentro de cada `Bar`/`QuoteTick`/`TradeTick` — sería el mismo error que la Restricción 8 de ADR-0155 ya prohíbe para Grupo I. Canonizado en ADR-0156 restricción 8.

### 4.4 Decisión 3 — Doctrina compra/venta (D3): enmienda a ADR-0114, no ADR nuevo

D3 fija qué precio lee el motor de ejecución simulada — eso es exactamente el dominio del **Contrato de Consistencia Conservadora** de ADR-0114 (ya tiene dos reglas FIJAS sobre qué es "optimismo no permitido": Bar-Open Alignment y Regla Intrabar Pesimista). D3 se añade como regla **(c) Bid/Ask Exclusivo**: el motor lee siempre compra o venta según la dirección, nunca un precio genérico; si `bid == ask`, se opera sin spread pero sigue siendo un campo con significado declarado, no un promedio. El dato de operación (`TradeTick`) se conserva como materia prima para tipos de barra — la regla prohíbe usarlo para simular un fill, no prohíbe persistirlo. No pasó por ADR nuevo porque crear uno habría duplicado la autoridad sobre "qué es optimismo no permitido en la ejecución", que ya vive en un solo sitio.

### 4.5 Decisión 4 — TASK-063 no absorbe DEBT-029; son capas distintas

**DEBT-029 punto 1 (decisión de política) queda resuelto** por D3/ADR-0156 restricción 5: un payload que mezcla cotización y ejecución sin mecanismo de plataforma conocido se rechaza con error de formato explícito — es exactamente el caso que DEBT-029 describe (`TradeTickInput` deserializando un JSON con `bid`/`ask` de sobra). **DEBT-029 puntos 2–3 (aplicación técnica y barrido del repositorio) siguen abiertos** — son implementación (`#[serde(deny_unknown_fields)]` o equivalente + pruebas discriminantes por struct), no decisión. La ficha `docs/debt/DEBT-029.md` se actualizó con esta resolución parcial, sin tocar su `Estado` (dominio del Tech-Lead cerrarla cuando el código aterrice).

### 4.6 Nuevo documento canónico: ADR-0156

`docs/adr/ADR-0156.md` — extiende ADR-0154/ADR-0155, relacionado con ADR-0114 (D3 vive ahí, no aquí). Contiene: rechazo por defecto, las dos vías con su responsable legal distinto (tabla de LDR-0008), las 8 restricciones operativas (incluidas las tres condiciones técnicas del veredicto legal sobre el ledger), la mecánica de trazabilidad por referencia, y las 5 comprobaciones de razonamiento crítico de dominio corridas explícitamente.

### 4.7 Lo que NO se resuelve aquí (correctamente fuera de alcance)

- **Catálogo inicial de plantillas por plataforma** (cuáles se soportan al arrancar): depende de TASK-060 (muestras reales), **bloqueada en el propietario** — no es una decisión que el Architect pueda tomar sin esos archivos. Queda anotado como dependencia, no como pregunta abierta que alguien deba responder ahora.
- **Jurisdicción / exigibilidad de clickwrap:** LDR-0008 ya lo remite a TASK-053 sin duplicar análisis — este plan tampoco lo duplica.

---

## 5. Lo que no se pudo resolver aquí, separado por tipo

**Contradicción entre documentos vigentes (reportada al Tech-Lead, no tapada):** ninguna sobrevivió sin corregirse en esta sesión. La única encontrada (`data-import-wizard.md` vs. D2, §4.2) se corrigió en el mismo movimiento porque estaba dentro del mandato del Architect (editar `docs/`) y no requería una decisión de producto nueva — solo alinear un documento con una directiva ya vinculante.

**Bloqueo duro de producto/negocio (para el propietario):** ninguno. Las tres Tasks traían directivas vinculantes suficientes para decidir sin volver a preguntar. El único punto que depende del propietario (catálogo inicial de plantillas, TASK-060) ya estaba correctamente marcado como bloqueado antes de esta sesión y sigue así — no es nuevo, no se generó aquí.

---

## 6. Plan de secuenciación

No se abren Stories de código en este plan — ese es el paso siguiente del Tech-Lead. Orden recomendado, con dependencias explícitas:

1. **Story de esquema — ensanche `i128` (TASK-061).** Sin dependencias de las otras dos. Toca `hybrid-data-transformer` + Bridge. Prerrequisito de la implementación de `DOLLAR` (barra de dólar acumula precio×volumen, es la que más sufre el techo actual).
2. **Story de `algorithmic-bars` — `TICK_COUNT`/`VOLUME`/`RENKO`/`RANGE` (TASK-062, catálogo ya cerrado).** Sin dependencia dura de (1) — estos cuatro no acumulan precio×volumen. Puede construirse en paralelo a (1).
3. **Story de `algorithmic-bars` — `DOLLAR`.** Depende de (1) completada (el ensanche `i128` debe existir antes de que `DOLLAR` tenga sentido acumular precio×volumen sin desbordar).
4. **Story de `data-import-wizard` — mecánica D1/D2/D4 + ledger de mapeo (TASK-063, ADR-0156).** Sin dependencia dura de (1)/(2)/(3) a nivel de código, pero comparte el esquema canónico (ADR-0155) al que el mapeo apunta — construir después de que el catálogo de `bar_type` esté estable evita que el mapeo declare contra un esquema que cambia debajo. Bloqueada en la práctica por TASK-060 (muestras reales) para el catálogo de plantillas, no para la mecánica de rechazo/ledger, que no depende de tener plantillas ya construidas.
5. **Story de `hybrid-data-transformer` — aplicación de DEBT-029 puntos 2–3** (rechazo de payload mixto a nivel de struct). Natural continuación de (4): mismo evento de dominio (payload híbrido), capas distintas (wizard = archivo completo; DEBT-029 = struct de deserialización). Puede ir en paralelo a (4) si el Tech-Lead prefiere separar equipos, pero comparten doctrina (D3/ADR-0156 restricción 5) — implementarlas sin coordinación arriesga dos rechazos con mensajes inconsistentes.
6. **Enmienda a ADR-0114 (regla Bid/Ask Exclusivo) — verificación en `backtest-engine`.** Cuando el motor de ejecución (EPIC-2) se construya, su Story de implementación hereda la regla (c) ya escrita; no requiere Story propia ahora — es una restricción de diseño para cuando ese motor exista, no deuda retroactiva (el motor no existe en código todavía).

Los números de Story/Task exactos los asigna el Tech-Lead al despachar, siguiendo la regla de contador global (`docs/execution/`, máximo actual 063 → siguiente libre 064 a la fecha de este plan; puede haber avanzado si otra Orden se creó entre tanto — verificar antes de asignar).

---

## 7. Documentos canonizados — qué cambió en cada uno

| Documento | Cambio |
|---|---|
| `docs/adr/ADR-0141.md` | Enmienda: sección "Ensanche de ancho para volumen acumulado" (re-derivación de rango, decisión `i128`, costo de infraestructura verificado, alcance); fila nueva en la tabla de mapeo de tipos; trazabilidad actualizada. |
| `docs/adr/ADR-0155.md` | Banner de enmienda; `CHECK` de `bar_type` con `RANGE` añadido; tipo de `volume_traded`/`volume_buy` anotado como `i128` en Rust/Polars; trazabilidad ampliada con ADR-0156. |
| `docs/features/algorithmic-bars.md` | Sección nueva "Catálogo cerrado de `bar_type`" (tabla, criterio de admisión, exclusión de Heikin Ashi, diferimiento de Kagi/P&F/Line Break, candidatos de imbalance/run, advertencia de sesgo estadístico); `DOLLAR` añadido a comportamientos observables y parámetros configurables. |
| `docs/adr/ADR-0114.md` | Banner de enmienda; regla (c) "Bid/Ask Exclusivo" añadida al Contrato de Consistencia Conservadora; trazabilidad ampliada. |
| `docs/adr/ADR-0156.md` | **Nuevo.** Formato estándar de ingesta: rechazo por defecto, plantillas vs. mapeo declarado, ledger de evidencia inmutable, mecánica de trazabilidad por referencia, 5 comprobaciones de dominio. |
| `docs/features/data-import-wizard.md` | Reescritura de Comportamientos Observables/Restricciones/Parámetros Configurables/Persistencia para eliminar la contradicción con D2 (mapeo automático adivinado) y reflejar ADR-0156. |
| `docs/debt/DEBT-033.md` | Sección "Resolución de arquitectura" — decisión sellada, estado sin cambio (Tech-Lead cierra al implementar). |
| `docs/debt/DEBT-029.md` | Sección "Resolución parcial de arquitectura" — punto 1 resuelto por D3/ADR-0156, puntos 2–3 siguen abiertos. |
| `docs/ROADMAP.md` | Nota en EPIC-1 sobre el catálogo cerrado de `algorithmic-bars`, dependencias y candidatos diferidos. |
| `docs/ADR.md` / `docs/README.md` | Fila nueva para ADR-0156; calificadores de enmienda 2026-07-19 en las filas de ADR-0114/0141/0155. |
| `docs/execution/TASK-061/062/063-*.md` | Sección "Resolución del Architect" añadida al final de cada una, con referencia a este plan. |
