# STORY-051 · Transformador de datos de mercado (motor Polars)

| Campo | Valor |
|---|---|
| **ID** | STORY-051 |
| **Título** | Transformador de datos de mercado — de archivo crudo a tabla canónica |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-1 — Soberanía de Datos (`ingest`) |
| **Sprint** | 1 |
| **Estado** | ✅ **CERRADA (2026-07-18)** — QA APTO · Quant APTO-CON-CONDICIONES (condición registrada como DEBT-033) |
| **Responsable** | Rust-Engineer (Sonnet, Modo Docente) · auditan Tech-Lead + QA-Engineer + Quant-Engineer |
| **Creada** | 2026-07-18 |
| **Completada** | 2026-07-18 |

> 🟢 **DESCONGELADA (2026-07-18).** El congelamiento cumplió su función y se levanta. Historia, para que la decisión quede entendible y no se repita el error que evitó:
>
> **Por qué se congeló.** El backend original agregaba asumiendo **un solo precio**. ADR-0155 canonizó después el esquema real —agregación exclusivamente desde operaciones ejecutadas, compra y venta separadas, spread declarado por la fuente, volumen en cuatro magnitudes independientes— y ADR-0155 + STORY-057 añadieron los seis campos de identidad de serie. Despachar el puente antes habría congelado el contrato equivocado en el puente y en la pantalla, donde sacarlo cuesta un orden de magnitud más.
>
> **Qué se hizo durante el congelamiento.** Tres retrofits encadenados sobre el backend (esquema de ADR-0155 · retiro del precio medio ponderado por ser un indicador, DEBT-028 · identidad de serie, STORY-057), cada uno auditado por el Tech-Lead antes de continuar.
>
> **Estado actual.** Las siete etapas entregadas y auditadas: Quant (viabilidad), UI-Designer, Rust, Bridge, Flutter, QA. Gate de mutación pasado sobre el crate completo. Queda solo la validación cuantitativa post-código (Etapa 6) antes del cierre.
>
> **La lección que deja:** "entregado" no es "cerrado". Congelar a tiempo costó tres retrofits de backend; no haberlo hecho habría costado esos tres más el puente y la pantalla rehechos.

## 0. Resumen ejecutivo

**Qué problema resuelve:** los datos que descarga el motor soberano llegan como archivos crudos, con el formato y la precisión de cada proveedor. Nada aguas abajo — ni el sanitizador, ni el validador anti look-ahead, ni el motor de backtest — puede trabajar sobre eso.

**Qué se va a construir:**
- Un motor de transformación en Polars que convierte millones de filas crudas en una tabla con esquema único: precio y volumen como enteros escalados, tiempo en nanosegundos UTC.
- Agregación de operaciones individuales a barras de cualquier temporalidad, con convención temporal explícita que impide mirar el futuro.
- Un registro auditable de cada transformación ejecutada, reproducible entre corridas.
- Su sección en el Banco de Pruebas y su comando de verificación por terminal.

**Por qué ahora:** es la precondición dura del resto de la cadena de ingesta. Sin él, sanitizador, validador y particionador no tienen sobre qué operar.

---

## 1. Especificación de origen

- **Feature:** [`hybrid-data-transformer`](../features/hybrid-data-transformer.md) (saneada por el Tech-Lead el 2026-07-17; ver §8)
- **TTRs:** TTR-007 del módulo, que implementa TTR-001 y TTR-002 de la feature
- **Módulo:** [`ingest`](../modules/ingest.md)
- **ADRs:** ADR-0105 (Polars nativo), ADR-0141 (escala entera ×10⁸, timestamps en nanosegundos, STRICT, UUIDv7), ADR-0112 (escalera de cómputo numérico), ADR-0137 (puertos tipados), ADR-0020 (contrato de persistencia, Perfil A), ADR-0117 / ADR-0136 / ADR-0152 (cáscara delgada, canvas, Banco de Pruebas), ADR-0142 (verificación por terminal), ADR-0133 (pirámide de pruebas + mutación), ADR-0012 (paralelismo), ADR-0037 (frontera con el sanitizador), ADR-0017 (fidelidad), ADR-0121 (comentarios en español)

## 2. Objetivo

Que el sistema convierta datos crudos de mercado en una tabla limpia, tipada y auditable, sin perder un céntimo por redondeo ni permitir que una barra sepa del futuro.

## 3. Agentes y Modo de Acompañamiento (ADR-0120)

| Agente | Etapa del pipeline | Depende de | Modo |
|---|---|---|---|
| Quant-Engineer | Etapa 0.4 — viabilidad experta | ninguno | Autónomo · ✅ **APTO-CON-CONDICIONES** (2026-07-17) |
| UI-Designer | Etapa 0.5 — cáscara visual | ninguno | Autónomo · ✅ entregada (2026-07-17) |
| **Rust-Engineer** | **Etapa 2 — implementación del núcleo** | Quant | **Docente** |
| Bridge-Engineer | Etapa 3 — contrato del puente | Rust-Engineer | Autónomo |
| Flutter-Engineer | Etapa 4 — cáscara delgada | Bridge-Engineer | Autónomo |
| **QA-Engineer** | **Etapa 5 — gate obligatorio** | Rust / Bridge / Flutter | **Autónomo** |
| Quant-Engineer | Etapa 6 — validación post-código | QA | Autónomo |

## 4. Instrucciones de despacho por agente

### 4.1 Rust-Engineer (Modo Docente)

Eres el Rust-Engineer de Drasus Engine. Lee en este orden: (1) `CLAUDE.md`, (2) `.agents/knowledge/base.md`, (3) `.agents/skills/rust-engineer/SKILL.md`, (4) esta Orden completa, (5) `docs/features/hybrid-data-transformer.md`, (6) `docs/adr/ADR-0141.md` y `docs/adr/ADR-0105.md`. Declara qué leíste.

Implementas TTR-007 del módulo `ingest`: la feature `hybrid-data-transformer` como **crate hexagonal de dominio** en `crates/features/data/hybrid-data-transformer/`, siguiendo el patrón ya establecido por `crates/features/data/sovereign-data-fetcher/` (léelo antes de escribir: misma estructura `domain.rs` / `persistence.rs` / `orchestrator.rs` / `public_interface.rs` / `schemas.rs`, depende solo de `shared`).

**Modo Docente:** implementas el bloque completo por tu cuenta y además escribes la lección en `docs/lessons/rust/STORY-051-hybrid-data-transformer.md`, explicando cada decisión con profundidad cero-conocimiento y anclada al código real que produjiste (ADR-0122 / ADR-0124). Enlace bidireccional con esta Orden.

**Invariantes que NO puedes violar (los cazó el Quant en la Etapa 0.4):**

1. **Aritmética entera.** Precio y volumen son `i64` escalados ×10⁸. Todo producto de dos magnitudes escaladas se acumula en `i128`, nunca en `i64`. Toda suma y producto usa `checked_*`. Toda división que reintroduce la escala usa **redondeo half-to-even**, con una función nombrada y probada. Cero `f64` en el Núcleo.
2. **Convención temporal.** `bar_timestamp_ns` = inicio del intervalo, por `floor(event_timestamp_ns / interval_ns) * interval_ns`, intervalo semiabierto. Al agregar barra→barra mayor, el timestamp se deriva del calendario, nunca copiando el de la primera sub-barra.
3. **Determinismo bajo paralelismo.** El plan termina siempre con un orden explícito por clave canónica antes de materializar. Los ticks se ordenan por `(event_timestamp_ns, clave de desempate de origen)` antes de agrupar. `audit_hash` se calcula sobre la representación ya ordenada.
4. **No decides calidad de dato.** Prohibido eliminar duplicados o rellenar huecos: eso es del sanitizador (ADR-0037) y del validador PIT. Tu conformado es de esquema. El conteo de filas solo cambia por la agregación declarada en el plan.
5. **Frontera con coma flotante.** La conversión a `ndarray::Array2<f64>` solo es exacta bajo 2⁵³; por encima se rechaza explícitamente en vez de pasar en silencio.
6. **FCIS.** El Núcleo (plan, `logic_hash`, reglas de agregación y redondeo, decisión de umbral) es lógica pura sin I/O. La Cáscara hace lectura/escritura, conversión de escala y persistencia.

**Persistencia (ADR-0020 Perfil A + ADR-0141):** tabla `transformation_records`, append-only, `STRICT`, PK `TEXT` con UUIDv7, `event_sequence_id UNIQUE` con triggers anti UPDATE/DELETE, `audit_chain_hash` NULL en la fila génesis, escritura envuelta en `BEGIN IMMEDIATE` + reintento acotado + error tipado de contención (patrón obligatorio de todo ledger, `persistence/data_portability.rs` es la referencia). Campos exactos en la sección de Persistencia de la feature. **En el mismo cambio** creas la ficha `docs/data-model/transformation_records.md` (plantilla `docs/templates/DATA-MODEL.md`) y su fila en el índice `docs/DATA-MODEL.md` — sin eso la Story no está completa.

**Puertos y verificación:** expón el puerto en `public_interface` siguiendo ADR-0137 (ver la sección Puertos de la feature) y el subcomando `verify hybrid-data-transformer` en `crates/app/src/main.rs`, con el mismo contrato que las features ya enchufadas.

**Datos de prueba:** las pruebas **no descargan nada de la red**. Usas datos fabricados para los casos borde y una muestra real pequeña de cripto (Binance) congelada en el repositorio como recurso de prueba. La muestra de Forex/CFD llega en una segunda tanda cuando exista la fuente — deuda **DEBT-025**, no la inventes ni la simules.

**Retrofit ADR-0155 (2026-07-18, actualiza esta sección tras el congelamiento):** la agregación parte EXCLUSIVAMENTE de `TradeTick` (ejecuciones) — nunca de `QuoteTick` (cotizaciones); son tipos físicos distintos que no se mezclan. El volumen de `Bar` se descompone en cuatro magnitudes independientes (`volume_traded`, `trade_count`, `volume_buy`, `volume_ticks`) — ninguna se rellena a partir de otra ni se asume por defecto cuando la fuente no la distingue. `Bar` también gana los campos esquema-listo de Futuros/Opciones de ADR-0155 (`open_interest`, `settlement_price`, griegas `*_reported`, etc.), todos `None` en esta feature (no derivables de un `TradeTick`). Los invariantes numéricos de la lista de arriba (acumulador `i128`, redondeo half-to-even, convención temporal, orden estable, frontera `f64`) NO cambiaron — se conservan y sus pruebas también.

**Corrección DEBT-028 (2026-07-18, ajusta el párrafo anterior tras el fallo del Tech-Lead):** en este retrofit, "sus pruebas también" se leyó como "las pruebas de VWAP también se conservan" — el retrofit incluso pedía explícitamente no tocar ese invariante y así calculaba `vwap` dentro de `Bar` con la maquinaria ancha. El Tech-Lead revirtió esa lectura: `Bar` **NO lleva campo `vwap`** — es un indicador (como el ATR o el RSI), no un primitivo de ingesta, y pertenece al motor de indicadores (EPIC-2/EPIC-3), nunca a esta feature (ver §8 punto 1 y `docs/debt/DEBT-028.md`). Lo que SÍ se conserva, intacto, es la aritmética ancha en sí (`checked_scaled_product` + `round_half_to_even_div_i128`, con sus pruebas) — correcta y reutilizable por el motor de indicadores — ahora ejercitada directamente en `domain.rs` en vez de a través de `Bar.vwap`.

**Disciplina de pruebas (ADR-0133, ciclo rojo→verde):** cada criterio de aceptación de §6 se expresa como una prueba nombrada y **discriminante** — debe fallar sin la implementación y solo pasar con ella. Demuéstralo. Una prueba que pasa aunque el comportamiento esté ausente no cuenta. Entregas ya en verde, con el mapeo criterio→prueba.

### 4.2 Bridge-Engineer (Autónomo)

Despachado cuando el contrato de tipos de Rust esté congelado. Genera los bindings de `flutter_rust_bridge` para el puerto de la feature, con el patrón `await` de `data_fetcher.dart` (no sondeo). Incluye la función de listado de datasets de origen que el panel necesita para poblar su selector — el UI-Designer la marcó como requisito y hoy no existe.

### 4.3 Flutter-Engineer (Autónomo)

Despachado cuando los bindings compilen. Implementa la `## Cáscara Visual` de la feature: la sección de esta feature enchufada al **harness genérico** del Banco de Pruebas (prohibido reimplementar el harness), el panel inspector y el widget de tablero en solo lectura. Consume exclusivamente `custom_ui.*`; prohibido reimplementar componentes en línea o escribir colores literales.

### 4.4 QA-Engineer (Autónomo)

Gate obligatorio. Además de la pirámide de pruebas y las Pruebas de Guerra, verifica el **gate de mutación** sobre los archivos de la Story (`cargo-mutants`, estándar 0 survivors, con `--output` dedicado) y las tres pruebas companion del patrón append-only.

## 5. Comandos de validación (para el propietario)

```bash
cargo test -p hybrid-data-transformer
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p app -- verify hybrid-data-transformer --input '<json de ejemplo, lo fija el ingeniero>' | jq .
cargo build --release -p bridge && cd ui && flutter build linux
```

## 6. Criterio de aceptación (cada uno, una prueba que primero falla)

1. **Agregación exacta.** La agregación OHLCV sobre precios y volúmenes escalados produce resultados idénticos a un oráculo de aritmética entera de referencia, sin ningún `f64` en el camino.
2. **Sin desbordamiento silencioso.** Con precio y volumen en el borde superior de magnitud realista, el cálculo del precio ponderado por volumen falla explícitamente si el acumulador es `i64` y produce el valor correcto con `i128`.
3. **Redondeo sin sesgo.** Al menos tres casos de empate exacto documentados producen el resultado half-to-even esperado.
4. **Determinismo bajo paralelismo.** Con dos operaciones de idéntico nanosegundo y distinto origen, dos corridas del mismo plan con distinto número de hilos producen el mismo primer y último precio por barra.
5. **Auditoría reproducible.** Dos corridas del mismo plan sobre el mismo dataset producen el mismo `audit_hash`, no solo el mismo `logic_hash`.
6. **Anti look-ahead.** Toda barra expone su timestamp de inicio, y ninguna operación posterior al cierre del intervalo contribuye a esa barra. Prueba explícita, no revisión de código.
7. **Frontera de coma flotante honesta.** Un volumen agregado por encima de 2⁵³ se rechaza al convertir a matriz densa en lugar de pasar como exacto.
8. **Sin descarte oculto.** Filas con timestamp repetido y contenido distinto se conservan; el conteo de entrada frente a salida solo cambia por la agregación declarada.
9. **Ledger atómico.** El registro de transformaciones sobrevive a 16 escritores concurrentes sin perder ni duplicar secuencia, y falla de forma tipada bajo contención sostenida.
10. **Verificación end-to-end.** El comando de terminal devuelve el JSON correcto y la sección del Banco de Pruebas ejecuta contra el backend real por el puente, mostrando esquema, conteo de filas, hash de lógica y duración; el observable persiste tras recargar.
11. **Modelo de datos sincronizado.** La ficha de la tabla nueva existe y coincide columna por columna con la migración real.
12. **Lección escrita** (Modo Docente), enlazada con esta Orden.

**Retrofit ADR-0155 (2026-07-18) — criterios añadidos:**

13. **Separación física TradeTick/QuoteTick.** Ninguna cotización puede llegar jamás a la agregación de barras: el tipo de entrada (`TradeTickInput`) no tiene forma de representar `bid`/`ask`, y un JSON con esa forma falla al deserializar antes de alcanzar el Núcleo. Prueba explícita en el borde real del CLI/harness, no solo revisión de tipos.
14. **Volumen en cuatro magnitudes, nunca colapsado ni rellenado por defecto.** Cuando la fuente distingue el lado comprador para todas las ejecuciones del intervalo, `volume_buy` refleja solo esa porción — numéricamente distinta de `volume_traded`. Cuando la fuente no lo distingue en absoluto, `volume_buy` queda `None` (nunca `0`, nunca copia de `volume_traded`). `volume_ticks`/campos de Futuros-Opciones quedan `None` siempre en esta feature (esquema-listo, sin fuente que los derive desde `TradeTick`).

## 7. Registro de ejecución

| Fecha | Agente / Modelo | Resultado | Evidencia |
|---|---|---|---|
| 2026-07-17 | Quant-Engineer (Sonnet) | ✅ APTO-CON-CONDICIONES | 6 hallazgos; los críticos (desbordamiento en producto escalado, ausencia de convención temporal de barra) incorporados como invariantes y criterios 2 y 6 |
| 2026-07-17 | UI-Designer (Sonnet) | ✅ Cáscara Visual entregada | Panel inspector como contexto de superficie; componentes verificados uno por uno como funcionales; sin bloqueos |
| 2026-07-18 | Rust-Engineer (Sonnet, Docente) | ✅ Entregado | Crate `crates/features/data/hybrid-data-transformer/`; migración `migrations/0021_hybrid_data_transformer.sql`; 36/36 tests verdes (21 unitarios `domain.rs` + 15 integración, incl. 16-escritores-concurrentes, muestra real Binance congelada, round-trip Parquet, CLI end-to-end); `cargo clippy -p hybrid-data-transformer --all-targets -- -D warnings` limpio; lección [`STORY-051-hybrid-data-transformer.md`](../lessons/rust/STORY-051-hybrid-data-transformer.md) |
| 2026-07-18 | Rust-Engineer (Sonnet, Docente) | ✅ Retrofit ADR-0155 entregado, sigue 🧊 CONGELADA | `RawTick`→`RawTrade`/`RawTickInput`→`TradeTickInput` (sin campos `bid`/`ask` — garantía de tipo, no runtime check); `Bar.volume` único → `volume_traded`/`trade_count`/`volume_buy` (`Option`, regla "todas conocidas o `None`")/`volume_ticks` (`Option`, siempre `None` en esta feature) + 9 campos Futuros/Opciones esquema-listo (`Option`, siempre `None`); `SCHEMA_VERSION` "1.0"→"2.0"; invariantes numéricos previos (acumulador `i128`, half-to-even, convención temporal, orden estable, frontera `f64`) intactos con sus pruebas originales sin modificar la aserción. **45/45 tests verdes** (28 unitarios + 17 integración, +9 nuevas: 5 discriminantes de las 4 magnitudes de volumen en `domain.rs`, 1 de separación física TradeTick/QuoteTick en `domain.rs`, 2 en `schemas.rs`, 2 end-to-end en `integration_tests.rs` incl. rechazo de JSON con forma de `QuoteTick` en el borde real del CLI); `cargo test --workspace` (684+ tests) y `cargo clippy --workspace --all-targets -- -D warnings` verdes; `cargo run -p app -- verify hybrid-data-transformer` ejercitado con y sin `aggressor_side` y con payload con forma de cotización (falla como se espera). Sin cambios a `migrations/0021` ni a `docs/data-model/transformation_records.md` (el ledger de ejecuciones del transformador es ortogonal al esquema de `Bar`, ver corrección del Architect en ADR-0155). Puntos abiertos para el Tech-Lead: ver §8. |
| 2026-07-18 | Rust-Engineer (Sonnet, Autónomo) | ✅ Corrección DEBT-028 entregada, sigue 🧊 CONGELADA | Falló el Tech-Lead §8 punto 1 (opción a): `vwap` retirado de `domain::Bar`, `schemas::BarOutput`, `aggregate_trades_to_bars`, `reaggregate_bars` y del esquema de `VerifyOutput`. `checked_scaled_product`/`round_half_to_even_div_i128` conservadas (con sus pruebas), reorientadas para ejercitarse directamente en vez de a través de `Bar.vwap`. Pruebas reescritas: `ohlcv_aggregation_matches_integer_oracle` (criterio 1, sin la aserción de VWAP) y `vwap_computation_succeeds_at_realistic_upper_bound_magnitude` → renombrada `wide_accumulator_division_succeeds_at_realistic_upper_bound_magnitude` (criterio 2, ejercita la aritmética ancha directamente). `compute_bars_audit_hash` corregida para incluir `bar_close_timestamp_ns`, `spread_reported` y los 9 campos de Futuros/Opciones (hallazgo de la auditoría: el doc-comment afirmaba proteger el contenido completo, pero los omitía); prueba nueva `audit_hash_differs_when_only_a_futures_options_field_differs`. Guarda `verify_exact_f64_representable` corregida (comparación en `u64` sin volver a `i64`, evita el envolvimiento de `i64::MIN`); prueba nueva `i64_min_is_rejected_without_wrapping_the_guard`. `SCHEMA_VERSION` "2.0"→"2.1" (el campo denota la forma de salida de `Bar`, que cambió de forma incompatible). **47/47 tests verdes** (30 unitarios + 17 integración, netos +2 tests nuevos frente a los 45 previos: 1 de auditoría de Futuros/Opciones, 1 de `i64::MIN`); `cargo test --workspace` (751 tests) y `cargo clippy --workspace --all-targets -- -D warnings` verdes; `cargo run -p app -- verify hybrid-data-transformer` ejercitado, `ok: true`, esquema sin `vwap`. Sin cambios a `migrations/0021`, `docs/data-model/transformation_records.md`, `crates/shared` ni `sovereign-data-fetcher` (fuera de alcance). Lección ampliada en `docs/lessons/rust/STORY-051-hybrid-data-transformer.md`. |

| 2026-07-18 | Rust-Engineer (Sonnet, Autónomo) | ✅ STORY-057 entregada — identidad de serie en la barra | `Bar`/`BarOutput` ganan los 6 campos de identidad obligatorios de ADR-0155 (`instrument_id`, `venue`, `source_provider`, `bar_type` como enum nuevo, `timeframe_value`, `data_snapshot_id`), materializados por fila; `TransformRequest` los exige y `data_snapshot_id` pasa de opcional a obligatorio (cambio incompatible). Los 6 entran en `compute_bars_audit_hash`: dos series de brókers distintos con OHLCV byte a byte idéntico ya producen sellos distintos — prueba central `audit_hash_differs_when_only_venue_differs_despite_identical_ohlcv`, verificada como discriminante por el Tech-Lead. Tripleta de linaje probada en sus dos casos borde (símbolo con guion propio, serie de ticks con token reservado). `SCHEMA_VERSION` "2.1"→"3.0". **56/56 tests verdes** (37 unitarios + 19 integración); clippy limpio; cobertura 90.83% (`domain.rs` 98.06%). Efecto colateral declarado: `Bar`/`BarOutput` pierden `Copy` al llevar `String`. Deuda hallada por la auditoría del Tech-Lead → **DEBT-030** (la reagregación hereda `timeframe_value` de la sub-barra en vez de derivarlo del intervalo destino; latente, sin llamador hoy). |
| 2026-07-18 | Bridge-Engineer (Sonnet, Autónomo) | ✅ Etapa 3 entregada — puente FFI | `crates/bridge/src/api/hybrid_data_transformer.rs`: `submit_hybrid_data_transformation` (async, patrón `await`, devuelve el puerto `bars_out` tipado, no un JSON opaco), `list_source_datasets` (la función de listado que el UI-Designer marcó como requisito y no existía) y `list_transformation_records` para el widget de tablero. Feature registrada en el despachador genérico del Banco de Pruebas (15→16). Bindings generados y `ui/lib/src/rust/api/hybrid_data_transformer.dart` verificado: las magnitudes que pueden faltar cruzan la frontera como **nulables**, no como cero — la distinción "la fuente no lo reporta" vs. "vale cero" sobrevive al puente, probada con 6 pruebas de ida y vuelta real (Polars + SQLite). **15/15 tests verdes** en `bridge`, clippy limpio, `dart analyze` de los bindings sin incidencias. **No verificado y declarado como tal:** `flutter build linux` completo (etapa del Flutter-Engineer). |
| 2026-07-18 | Tech-Lead | ✅ Duplicación de vocabulario eliminada | El Bridge-Engineer se vio obligado a duplicar la tabla de temporalidades (`1m`…`1d`) porque la función de la feature era privada, y lo reportó en vez de tocar el crate ajeno (correcto). El Tech-Lead hizo `pub` `parse_target_aggregation` en la feature —dueña del vocabulario— y el puente ahora delega. Motivo: una tabla de vocabulario duplicada se desincroniza en cuanto alguien añade una temporalidad en un solo lado, y el síntoma sería una temporalidad válida rechazada solo desde la interfaz gráfica. Verificado tras el cambio: `bridge` 15/15, feature 56/56, clippy limpio. |

| 2026-07-18 | Flutter-Engineer (Sonnet, Autónomo) | ✅ Etapa 4 entregada — cáscara visual | Sección enchufada al harness genérico (sin reimplementarlo), panel inspector y widget de tablero en solo lectura. Consume solo `custom_ui.*`. `OptionalMagnitudeText` distingue visualmente magnitud ausente de cero real (texto, color y estilo distintos) — 4 pruebas de widget. `flutter build linux` verde. **Gap reportado, no ocultado:** `SourceDatasetDto` no transporta los campos de identidad, así que se pidieron al usuario como entrada manual → elevado por el Tech-Lead a **DEBT-031** (identidad tecleada a mano queda sellada en el hash de auditoría). |
| 2026-07-18 | QA-Engineer (Sonnet, Autónomo) | ❌ **NO APTO** — Gate de QA, primera pasada | Los 14 criterios reproducidos de forma independiente. Tres bloqueos: (1) **14 supervivientes de mutación**, el más grave `compute_logic_hash` mutable a cadena vacía sin que nada falle — su única prueba comparaba la función consigo misma (tautología), verificado por el Tech-Lead; (2) criterio 10, la **persistencia tras recargar no estaba comprobada** en ningún nivel de la cáscara visual (confirmó la sospecha del Tech-Lead); (3) `schemas.rs`/`orchestrator.rs`/`public_interface.rs` **sin gate de mutación corrido**. Observación: migración `0021` sin triggers anti UPDATE/DELETE pese a que §4.1 los exigía. Identificó correctamente que la prueba no discriminante del clasificador es un **hueco heredado del patrón de referencia**, no un defecto de esta Story → **DEBT-032**. |
| 2026-07-18 | Tech-Lead | Disposiciones sobre el NO APTO | (a) **Triggers anti UPDATE/DELETE se añaden** a `migrations/0021`, editada in-situ por fase GREENFIELD (ADR-0006): un ledger cuya inmutabilidad depende de que nadie escriba el `UPDATE` equivocado es una convención, no un invariante — la base de datos debe rechazarlo. (b) Los defaults de `VerifyInput` (`SMOKE`, `SMOKE_VENUE`…) se declaran **correctos por diseño**: son centinelas explícitos del arranque de humo, no fabricación silenciosa de identidad; solo se les exige prueba de valores dorados. (c) Los huecos sistémicos del patrón de ledger se registran como **DEBT-032** en vez de parchearse solo aquí — lo que cierra esa deuda es corregir el molde, no los dos sitios que ya lo copiaron. |
| 2026-07-18 | Rust-Engineer (Sonnet, Autónomo) ×2 tandas + Flutter-Engineer | ✅ Bloqueos cerrados | 1ª tanda: `compute_logic_hash` ahora se prueba contra planes distintos y construcciones independientes (fin de la tautología); `is_transient_write_conflict` ejercitada directamente con sus tres casos; triggers añadidos con sus pruebas; referencia colgante de `docs/data-model/transformation_records.md` corregida. 2ª tanda (32 supervivientes hallados por el Tech-Lead al correr el gate completo): valores dorados de la tabla de temporalidades — nadie verificaba los nanosegundos que produce cada token, y esa tabla decide el ancho de la barra y el puente delega en ella—, brazos `BUY`/`SELL`/`UNKNOWN` del lado agresor, signo con divisor negativo, y centinelas de humo. Flutter: `hybrid_data_transformer_dashboard_persistence_test.dart`, discriminación **demostrada en vivo** reintroduciendo una caché en memoria, viéndola fallar y revirtiendo. |
| 2026-07-18 | Tech-Lead | Gate de mutación corrido sobre el crate completo | 177 mutantes, los cinco archivos (el QA solo había alcanzado dos). Nota de entorno: `cargo-mutants` **revienta usando `/tmp`** (es tmpfs sobre RAM; el enlazado de Polars la agota y el síntoma es un `Bus error` del enlazador que aparenta fallo de compilación) — correr siempre con `TMPDIR` en disco real y `--jobs` acotado. Resultado tras las dos tandas: **137 caught, 37 unviable, 1 timeout, 2 missed**, ambos `missed` equivalentes genuinos demostrados por traza aritmética. |
| 2026-07-18 | QA-Engineer (Sonnet, Autónomo) | ✅ **APTO** — Gate de QA, segunda pasada | Reejecutó el gate de mutación **desde cero**, sin reutilizar la corrida del Tech-Lead: números idénticos (177 / 2 missed / 137 caught / 37 unviable / 1 timeout). Verificó la equivalencia de los 2 `missed` trazando la aritmética por su cuenta y **concordó**; dictaminó el timeout como captura por ADR-0133 (el mutante congela el contador y cuelga la prueba de contención). Confirmó triggers, ficha 17/17 columnas, centinelas de humo y el diseño de la prueba de recarga (almacenes distintos por montaje; una caché en memoria la rompería). **Búsqueda activa de "pruebas escritas para matar el mutante"** entre las ~25 nuevas: ninguna — todas con valor esperado calculado a mano y razonamiento explícito. Sin bloqueantes. |

| 2026-07-18 | Quant-Engineer (Sonnet, Autónomo) | ⚠️ **APTO-CON-CONDICIONES** — Etapa 6, validación post-código | Verificó en el comportamiento (no en el nombre de las pruebas) que sus dos condiciones críticas de la Etapa 0.4 quedaron satisfechas. Juicios de dominio sellados con las 7 comprobaciones de `critical-domain-reasoning.md`: la convención temporal **sí** impide mirar el futuro y es agnóstica al calendario por operar en nanosegundos UTC absolutos —sin deriva por horario de verano, sin ambigüedad en fin de semana de Forex, sin barras vacías fabricadas—; la regla de volumen "todas conocidas o ninguna" es **cuantitativamente correcta** para una capa de ingesta, porque la alternativa introduciría sesgo sistemático a la baja en cualquier métrica de flujo de órdenes. **Hallazgo nuevo (condición):** la escala ×10⁸ de ADR-0141 se eligió por precisión decimal y nunca se re-derivó contra **techo de magnitud** — `i64` solo admite ~92 200 millones de unidades, y una barra horaria de cripto de oferta extrema (SHIB/PEPE) a volumen NORMAL lo desborda 1,13× (4,52× en día activo). Aritmética verificada de forma independiente por el Tech-Lead. Falla seguro (rechazo explícito, sin corrupción) pero rechaza sistemáticamente → **DEBT-033**, escalado al Architect. Segundo hallazgo, sin código que lo resuelva aún → **TASK-059** (fijar como contrato que el consumidor use el instante de CIERRE como momento de conocibilidad, no la etiqueta de inicio). |

## 8. Deudas, decisiones y constancia del Gate de Coherencia

**Correcciones del Tech-Lead a la feature (autoridad propia, ejecutadas 2026-07-17):**
- Precio, volumen y tiempo pasaban por coma flotante (`float64`, `datetime64[ns]`) — la spec es de abril, anterior a ADR-0141. Corregido a entero escalado ×10⁸ y nanosegundos UTC, en el DataFrame y en el Parquet.
- `linfa` figuraba como dependencia implícita, saltándose la escalera de ADR-0112. Acotado: `ndarray` + Rayon como primera línea, sin ascender sin benchmark.
- Faltaban por completo tres secciones obligatorias: puertos de integración (ADR-0137), separación núcleo/cáscara, y contrato de integración de interfaz (ADR-0117). Escritas por el Tech-Lead. La feature se clasificó como **Superficie propia**: produce un tipo de dominio del catálogo y recibe configuración del usuario.
- El contrato de persistencia declaraba `execution_latency_ms`, campo de Grupo V no autorizado en el Perfil A. Reemplazado por un campo propio declarado como tal, con la excepción documentada de ADR-0020. Añadidos `parent_id` y `session_id`, autorizados por el perfil.
- Fidelidad declaraba "precisión float64"; corregido a precisión exacta por escala entera.

**Barrido de ADRs no citados por la spec** (índice completo revisado, candidatos abiertos bajo demanda): ADR-0141, ADR-0112, ADR-0137, ADR-0117, ADR-0136, ADR-0152, ADR-0142, ADR-0133, ADR-0020, ADR-0037, ADR-0035, ADR-0012, ADR-0017. Todos incorporados como restricciones o criterios. Ninguno reveló una decisión de diseño nueva que exigiera escalamiento.

**Ambigüedad de orquestación resuelta por el Tech-Lead:** el Quant señaló que `docs/modules/ingest.md` no declara si el sanitizador corre antes o después del transformador. La tabla de secuencia del módulo sí lo fija — descarga → transformación → normalización → sanitizador → validador → PIT — y ese orden es correcto: la sanitización de seis capas opera sobre datos ya columnares. Lo que estaba mal era que la spec del transformador se atribuyera decisiones de limpieza. Corregido en la feature; no es escalamiento.

**Impacto en el SAD:** sin impacto. La Story no altera arquitectura de datos global, contratos entre módulos, dependencias ni gobernanza — implementa una feature ya prevista con el esquema canónico ya decidido en ADR-0141.

**Deuda registrada:**
- **DEBT-025** (🟡 Baja) — la muestra real de prueba cierra solo con cripto; la tanda de Forex/CFD entra cuando exista la fuente. Disparador: resolución de TASK-052.
- **Fuente Forex/CFD inexistente** — escalado al Architect en [TASK-052](./TASK-052-architect-escalation-market-data-sources.md). No bloquea esta Story.
- **Nodo de canvas** — se entrega si la infraestructura está disponible al implementar; si no, se registra como deuda de integración visual.

**Puntos abiertos del retrofit ADR-0155 (2026-07-18) — el Rust-Engineer se detuvo a reportar en vez de resolver por su cuenta:**

1. **Tensión `Bar.vwap` vs. "guarda el hecho, calcula la interpretación".** ADR-0155 lista explícitamente "vwap calculado por Drasus" en "Qué NO se persiste" — solo autoriza el `vwap` que la propia fuente reporta con su convención. Esta feature SÍ calcula `vwap` desde `TradeTick` (acumulador `i128` + half-to-even) y lo expone en `Bar`/`BarOutput`, porque la Orden de retrofit exige explícitamente conservar "acumulador ancho... redondeo al par... y sus pruebas" — y dos pruebas existentes (`ohlcv_aggregation_matches_integer_oracle`, `vwap_computation_succeeds_at_realistic_upper_bound_magnitude`) verifican `vwap` precisamente a través de esa maquinaria. Hoy no hay violación viva: `Bar` todavía no se escribe a Parquet en ningún punto del pipeline (no existe ese escritor en esta Story), así que no hay "persistencia" real que infrinja la regla. **Pendiente de fallo del Tech-Lead antes de que exista un escritor de `Bars` a Parquet:** (a) anular `vwap` en el camino `TradeTick`→`Bar` (rompe las dos pruebas citadas, requiere reescribirlas para no depender de `bar.vwap`), o (b) documentar como excepción admitida mientras `Bar` no cruce a persistencia columnar. No se decidió unilateralmente porque el ADR es de un día y la Orden pedía explícitamente no tocar ese invariante.

   **Resolución (2026-07-18) — el Tech-Lead falla la opción (a): se anula el campo.** Motivo canónico, doctrina del propietario registrada en `docs/debt/DEBT-028.md`: los primitivos de mercado (apertura, máximo, mínimo, cierre, compra, venta, volumen) son materia prima de la INGESTA; cualquier cosa calculada a partir de ellos es un INDICADOR y pertenece al motor (EPIC-2/EPIC-3), igual que el ATR o el RSI — es una frontera de fase, no un problema de mutabilidad. La auditoría del Tech-Lead encontró además dos defectos reales en la implementación que refuerzan el fallo (no se "arreglaron": desaparecen al retirar el campo): un `None` por fallo aritmético (desbordamiento/divisor cero) era indistinguible de un `None` por "la fuente no lo distingue" (fallo de cálculo silencioso); y `reaggregate_bars` excluía del denominador del VWAP compuesto las sub-barras sin `vwap` y presentaba el resultado parcial como si fuera el de la barra completa — el mismo error que la regla "todas conocidas o `None`" de `volume_buy` existe para impedir. `Bar`/`BarOutput` ya NO tienen campo `vwap`; `checked_scaled_product`/`round_half_to_even_div_i128` se conservan intactas (con sus pruebas, reorientadas para no depender de `Bar`) porque el motor de indicadores las necesitará. `compute_bars_audit_hash` de paso se corrigió para incluir `bar_close_timestamp_ns`, `spread_reported` y los nueve campos de Futuros/Opciones, que el hallazgo de la auditoría encontró ausentes pese al doc-comment. Ver Registro de ejecución §7 (fila 2026-07-18, corrección DEBT-028) y `docs/lessons/rust/STORY-051-hybrid-data-transformer.md`.
2. **Catálogo de puertos `shared::types` desactualizado frente a la propia enmienda de ADR-0137 (2026-07-18).** El código en `crates/shared/src/types/mod.rs` todavía declara solo `Tick` (marcador vacío) — no `QuoteTick`/`TradeTick`/`DepthLevel`/`OrderEvent`, aunque `docs/adr/ADR-0137.md` ya documenta esos cuatro tipos como vigentes y dice textualmente "no queda un tipo `Tick` genérico en el catálogo". `sovereign-data-fetcher::public_interface::OutputPorts.ticks_out: Tick` (línea 110 de ese crate) sigue usando el marcador viejo. **No toqué `crates/shared` ni `sovereign-data-fetcher`** — están fuera del alcance de esta Story (otro feature-crate, otra auditoría cerrada) y el retrofit de `hybrid-data-transformer` no lo necesitaba para compilar (el contrato físico de `TradeTickInput` vive en este crate, no en el marcador de canvas). Recomiendo una Story/Task corta dedicada a: añadir `QuoteTick`/`TradeTick`/`DepthLevel`/`OrderEvent` a `shared::types` (aditivo, sin riesgo) y migrar `sovereign-data-fetcher` de `Tick` a `QuoteTick`+`TradeTick` (ese sí rompe compilación si se retira `Tick` sin ese paso).
3. **Campos de identidad de instrumento/venue ausentes en `Bar`.** ADR-0155 declara `instrument_id`/`venue`/`source_provider`/`bar_type`/`timeframe_value` como "campos comunes obligatorios" de `Bars`. Ni el código pre-retrofit ni este retrofit los incluyen en `Bar` — quedan implícitos a nivel de `TransformRequest`/`data_snapshot_id` (un dataset por transformación). Ninguno de los 12+2 criterios de aceptación de esta Story los exige. Lo dejo señalado, no lo agregué por mi cuenta (habría expandido el alcance más allá de lo pedido en el retrofit: TradeTick/QuoteTick, volumen en 4 magnitudes, Futuros/Opciones esquema-listo, guarda-el-hecho).

## 9. Cierre ejecutivo

**Qué quedó funcionando.** El motor que convierte archivos crudos de mercado en una tabla canónica: precio y volumen como enteros exactos, tiempo en nanosegundos UTC, barras agregadas de cualquier temporalidad con una convención que impide que una barra sepa del futuro, y un registro auditable e inmutable de cada transformación. Con su comando de terminal, su sección en el Banco de Pruebas y su widget de tablero.

**Qué costó de más, y por qué valió la pena.** La Story se congeló a mitad de camino, con el backend entregado y el puente sin despachar. El motivo: el esquema canónico de datos de mercado se selló *después* de que el backend estuviera escrito, y despachar el puente antes habría congelado el contrato equivocado en la interfaz. El congelamiento costó tres retrofits de backend; no haberlo hecho habría costado esos tres más el puente y la pantalla rehechos.

**Lo que el gate de calidad encontró, y por qué importa.** El QA dictaminó NO APTO en primera pasada. Ninguno de los bloqueos era un defecto de comportamiento — el código hacía lo correcto. Eran **huecos de detección**: pruebas que no probaban. La peor, la del sello del plan de transformación, comparaba la función consigo misma y habría pasado igual si la función devolviera siempre una cadena vacía. El gate de mutación destapó 46 casos así en total, todos cerrados.

**Lo que sabemos ahora y no sabíamos.** El transformador no puede procesar cripto de oferta extrema: una barra horaria de SHIB a volumen normal desborda el techo de la escala entera. Falla seguro, sin corromper nada, pero falla siempre. Es una laguna de ADR-0141, que eligió la escala por precisión decimal sin comprobar el rango — no un defecto de esta feature.

**Estado final.** QA: **APTO**. Quant: **APTO-CON-CONDICIONES**, con la condición registrada como DEBT-033 y escalada al Architect. Ninguna condición bloquea el cierre del código.

**Deuda que deja:** DEBT-025 (muestra de Forex/CFD), DEBT-029 (contratos de entrada laxos), DEBT-030 (reagregación con temporalidad heredada), DEBT-031 (identidad tecleada a mano), DEBT-032 (huecos del patrón de ledger, transversal), DEBT-033 (techo de magnitud, arquitectónico). Más TASK-059 (contrato del momento de conocibilidad). Todas con disparador explícito; ninguna mordiendo hoy.
