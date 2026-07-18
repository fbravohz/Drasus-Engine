# Hybrid Data Transformer

**Carpeta:** `crates/features/data/hybrid-data-transformer/`
**Estado:** En Diseño
**Última actualización:** 2026-07-17
**Decisión Arquitectónica Asociada:** ADR-0105 (Estrategia de Datos 100% Polars Nativo), ADR-0013 (Stack 100% Rust Nativo), ADR-0141 (Modelado Relacional Soberano — escala entera ×10⁸ y timestamps en nanosegundos), ADR-0112 (Escalera de Cómputo Numérico Soberano), ADR-0137 (Feature como Unidad Hexagonal con Puertos Tipados), ADR-0117/ADR-0152 (Cáscara Delgada + Banco de Pruebas), ADR-0012 (Multi-Pipeline Paralela), ADR-0155 (esquema canónico — agregación exclusivamente desde `TradeTick`, `Bars` con volumen en cuatro magnitudes)

## ¿Qué es?

Es el motor de transformación que aplica la regla **80/20**: utiliza el máximo rendimiento de **Polars** para el procesamiento masivo (ETL, limpieza, agregaciones) y reserva **`ndarray` + Rayon** (primer peldaño de la escalera de ADR-0112) exclusivamente para el consumo estadístico nativo en Rust.

**Problema:** Los intérpretes externos (Python/Pandas) introducen latencia de serialización, GIL y doble ecosistema de dependencias. Polars es rápido pero el consumo estadístico avanzado (regresiones, modelos lineales) requiere matrices densas en memoria contigua.
**Solución:** Orquestar transformaciones en Polars y convertir a `ndarray` solo en la frontera de consumo estadístico mediante Zero-Copy vía Apache Arrow.

## Comportamientos Observables

- [ ] El sistema procesa 5GB de datos en memoria sin colapsar la RAM del usuario gracias al procesamiento columnar de Polars.
- [ ] Las transformaciones (normalización, limpieza al vuelo) se ejecutan utilizando todos los núcleos del procesador central (CPU).
- [ ] Cuando un consumidor requiere una matriz densa (regresión, álgebra lineal), el sistema convierte el bloque de datos resultante (pequeño) a `ndarray` sin duplicar memoria.

## Restricciones

- NUNCA se usa un intérprete externo para la fase de Ingesta ETL masiva.
- NUNCA se materializan DataFrames intermedios en Polars si se puede usar *Lazy Evaluation* (`scan_parquet`).
- Toda conversión a `ndarray` debe usar Apache Arrow como puente para evitar duplicación de memoria (Zero-Copy).
- **NUNCA `Float64` para precio, volumen o tiempo** en el DataFrame transformado ni en el Parquet resultante (ADR-0141): escala entera ×10⁸ y nanosegundos UTC. El `f64` solo existe cruzando hacia `ndarray` para consumo estadístico, y la conversión ocurre en la Cáscara, jamás en el Núcleo.
- La escalera de cómputo numérico es `ndarray` + Rayon como primera línea (ADR-0112). NO se adopta `linfa`, `candle` ni `burn` sin un consumidor real y un benchmark que demuestre que el peldaño inferior no alcanza.
- Toda transformación queda registrada bajo un `logic_hash` reproducible: el mismo plan sobre el mismo dataset produce el mismo hash (ADR-0020).
- **Volumen en cuatro magnitudes, nunca colapsado (ADR-0155):** `volume_traded`, `volume_ticks`, `trade_count` y `volume_buy` son columnas independientes. Un campo se puebla solo si la fuente concreta lo distingue como tal; si el dataset de entrada no aclara cuál de las magnitudes trae, la agregación deja el resto en `NULL` — nunca asume `volume_traded` por defecto.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| POLARS_THREADS | Auto | 1 - Max | Hilos que Polars usa para cálculos paralelos | CONFIG |
| LAZY_MODE | True | True/False | Si true, usa Lazy Frames para optimizar filtros | [FIJO] |
| INTEROP_THRESHOLD | 100,000 | - | Tamaño máximo de filas para priorizar conversión a `ndarray` | CONFIG |

## Ciclo de Vida de la Feature — Hybrid Data Transformer

### Entrada
- Archivos Parquet o Stream de datos crudos.
- Plan de transformación (filtros, agregaciones, normalizaciones).

### Proceso
- Construye un `LazyFrame` en Polars.
- Ejecuta conformado **de esquema** (tipado, columnas, escala entera). **NO decide calidad de dato:** eliminar duplicados y decidir qué hueco se rellena es trabajo del sanitizador (ADR-0037) y del validador PIT, que lo hacen con su `error_log` auditable y su umbral configurable. Dos operaciones legítimas pueden compartir el mismo nanosegundo; un deduplicador ciego aquí destruiría volumen real antes de que el sanitizador vea el dato.
- Aplica agregaciones columnares (EJ: OHLCV a barras de 1h).
- Si el consumidor requiere matrices densas: Ejecuta `collect()` y conversión Zero-Copy a `ndarray` vía Arrow.

### Salida
- `DataFrame` de Polars (para persistencia Parquet).
- `ndarray::Array2<f64>` **solo en la frontera de consumo estadístico**, nunca en persistencia.
- Esquema unificado de tipos (ADR-0141): precio y volumen `Int64` escalados ×10⁸; tiempo `Int64` en nanosegundos Unix UTC.

### Contextos de Uso

**Contexto 1: Ingesta Central (Módulo Ingest)**
- Transforma millones de filas crudas en archivos Parquet optimizados.

**Contexto 2: Investigación R&D (Módulo Generate/Validate)**
- Prepara los datos para modelos de IA o pruebas de robustez estadísticas.

## Tareas (TTRs)

### **TTR-001: Pipeline de Transformación Lazy (Polars)**
- Implementa la lógica de "Clean-on-the-fly" usando Polars para saturar la CPU en transformaciones columnares masivas.
- **Postcondición verificable:** dado un dataset crudo y un plan de transformación, el resultado es un DataFrame con el esquema canónico (precio/volumen `Int64` ×10⁸, tiempo `Int64` ns UTC), ordenado por clave canónica estable, con `logic_hash` y `audit_hash` reproducibles entre dos corridas idénticas — incluso con distinto número de hilos. El conteo de filas solo cambia por la agregación declarada en el plan, nunca por un descarte oculto.

### Convención temporal de la barra (anti look-ahead — FIJO)

- `bar_timestamp_ns` = **inicio** del intervalo, calculado como `floor(event_timestamp_ns / interval_ns) * interval_ns`. Intervalo semiabierto `[inicio, inicio + interval_ns)`.
- `bar_close_timestamp_ns = bar_timestamp_ns + interval_ns`. Ningún consumidor puede tratar el contenido OHLCV de una barra como conocido antes de ese instante: el cierre, el máximo y el mínimo dependen de operaciones que ocurren hasta el final del intervalo.
- Al agregar barra→barra mayor, el timestamp resultante se deriva del calendario por `floor`, **nunca** copiando el de la primera sub-barra: si la ventana empieza con un hueco, copiar desalinearía la etiqueta en silencio.

### Aritmética entera (FIJO)

- Todo producto de dos magnitudes ya escaladas ×10⁸ (precio ponderado por volumen, varianza, cualquier estadístico de segundo orden) se acumula en `i128`, nunca en `i64`: un precio de ~10⁵ por un volumen de ~10⁷, ambos escalados, desborda `i64` por muchos órdenes de magnitud.
- Toda suma y producto usa aritmética comprobada (`checked_*`). El desbordamiento se convierte en error explícito, jamás en un número basura persistido como fuente de verdad.
- Toda división que reintroduce la escala ×10⁸ usa **redondeo al par más cercano** (half-to-even). Redondear siempre hacia arriba acumula sesgo positivo sistemático sobre millones de barras; el redondeo al par no acumula dirección.
- **Alcance:** la feature asume fuentes con precisión nativa de 8 decimales o menos. Una fuente de mayor precisión (18 decimales de cadena de bloques) exige aplicar aquí la misma regla de redondeo y se registrará como deuda con su disparador cuando aparezca.

### **TTR-002: Puente de Interoperabilidad Zero-Copy**
- Implementa la conversión optimizada Polars → `ndarray` vía Apache Arrow asegurando que el tipado estricto se mantenga consistente.
- **Postcondición verificable:** un bloque de N filas convertido a `ndarray::Array2<f64>` conserva los valores **exactos mientras el valor escalado se mantenga por debajo de 2⁵³** (límite de representación entera exacta en coma flotante de doble precisión) y no duplica el buffer en memoria. Un valor por encima de ese umbral —alcanzable en volumen agregado de pares líquidos— se rechaza explícitamente en lugar de cruzar en silencio como si fuera exacto. Por encima de `INTEROP_THRESHOLD` la conversión se rechaza en lugar de materializar la matriz.

## Núcleo Funcional / Cáscara Imperativa (FCIS)

**Núcleo (lógica pura, cero I/O):**
- Construcción y validación del plan de transformación (qué filtros, agregaciones y normalizaciones se aplican y en qué orden).
- Cálculo del `logic_hash` sobre el plan canonicalizado.
- Reglas de agregación OHLCV y de re-escala entera (redondeo determinista, sin `f64` intermedio).
- Decisión de conversión a `ndarray` contra `INTEROP_THRESHOLD`.

**Cáscara (efectos del sistema):**
- Lectura de Parquet (`scan_parquet`), ejecución del `LazyFrame` (`collect`), escritura del resultado.
- Conversión float ↔ entero ×10⁸ en la frontera (ADR-0141 R4).
- Persistencia del registro de transformación en SQLite y sellado de la cadena de auditoría.
- Configuración del paralelismo de Polars y medición del tiempo de proceso.

## Puertos de Integración (ADR-0137)

| Puerto | Tipo | Dirección | Cardinalidad | Qué transporta |
|---|---|---|---|---|
| `trade_ticks_in` | `TradeTick` | Input | 1..N | Ejecuciones crudas (ya demultiplexadas por `data-normalization-layer`). **Única fuente válida de agregación a `Bars`** (ADR-0155 §"la agregación de un intervalo de `TradeTick`, nunca de `QuoteTick` — el volumen no existe en una cotización") |
| `bars_in` | `Bars` | Input | 0..N | Barras OHLCV crudas cuando la fuente ya entrega agregado (ej. Binance klines) |
| `bars_out` | `Bars` | Output | 1 | Barras transformadas con esquema canónico (ADR-0155: volumen desagregado en `volume_traded`/`volume_ticks`/`trade_count`/`volume_buy`), listas para normalización y validación |
| `matrix_out` | `ArrowStream` | Output | 0..1 | Bloque columnar Arrow para consumo estadístico (`ndarray`) |

> **`QuoteTick` no es puerto de entrada de esta feature.** Una cotización no tiene volumen que agregar a OHLCV; el uso de `QuoteTick` (spread histórico, mid-price) es responsabilidad de features consumidoras aguas abajo, no de este transformador.

## Contrato de Integración UI (ADR-0117 / ADR-0136 / ADR-0152)

**Superficie propia.** La feature produce tipos de dominio del catálogo (`Bars`) y recibe configuración del usuario (plan de transformación, hilos de Polars, umbral de interoperabilidad), por lo que es un nodo del canvas con panel inspector, no plomería.

- **Banco de Pruebas (SVF, Canal #1):** se enchufa al harness genérico — entra el plan de transformación en JSON sobre un dataset ya descargado, sale el esquema resultante, el conteo de filas, el `logic_hash` y el tiempo de proceso reales por FFI.
- **Canal #2 (CLI):** `drasus verify hybrid-data-transformer`.
- **Widget de Tablero:** última transformación ejecutada (dataset, filas procesadas, duración) en modo solo lectura.
- **Nodo de Canvas:** se entrega si la infraestructura de canvas existe al momento de la Story; si no, queda registrada como deuda.

## Cáscara Visual (Thin Shell)

> Autoridad: ADR-0106 · ADR-0136 · ADR-0117 · `docs/DESIGN.md` · `docs/DESIGN.md §"Catálogo de Componentes"` · entregada por el skill `ui-designer` (Etapa 0.5, ADR-0135) el 2026-07-17.

### Correcciones de violaciones (pre-diseño)

Ninguna. El texto de la feature no menciona WebGL/DOM/HTML/CSS/SVG, no calcula métricas en el frontend (todo el cómputo — Polars, `ndarray`, `logic_hash` — vive en el Núcleo Rust), no usa alias MACRO/MESO/MICRO ni tecnologías rechazadas. La mención a "Python/Pandas" en `## ¿Qué es?` describe el problema que la feature evita, no una dependencia propuesta.

### Contexto de superficie (ADR-0136)

**Inspector Panel**, con entrada por card-node en **Canvas — Vista Relacional**. `hybrid-data-transformer` es un *feature node* (leaf, sin sub-nodos) del pipeline de `ingest`/`generate`/`validate`: por el "Contrato del nodo Canvas por-feature" (ADR-0136 §111-116) todo feature-leaf abre siempre el inspector lateral al hacer clic — nunca pantalla full — y ese inspector **es** esta misma Cáscara Visual, no una segunda UI paralela.

### Superficie y Densidad

- **Card-node (Vista Relacional):** `cardInner #11182E`, radio 10px, hairline `borderPanel #1B2440` — anatomía estándar de ADR-0136/DESIGN.md §"Grafo/DAG", sin spec adicional por-feature (la infraestructura del canvas la gobierna, no esta feature).
- **Inspector Panel (superficie principal de esta Cáscara):** `panel-glass` (`glassFill 0x73141C36` + blur 24) para el chrome del panel lateral; el bloque de resultados interno usa `panel-solid` (`Gx.surfacePanel` dinámico, vía `custom_ui.Surface`) — mismo patrón de superficies mixtas que `verification-bench` (chrome de vidrio, datos en sólido).
- **Densidad:** cómoda (Inspector Panel, per SKILL.md §Paso 2) — máximo aire, no cockpit denso.
- **Lienzo de fondo:** `deepSpace #080A18`, heredado del Canvas. Sin telón cósmico adicional dentro del panel lateral.

### Componentes

| ID de catálogo (DESIGN.md §4–§11) | Rol en esta feature | Widget real / Tokens clave | Estados semánticos |
|---|---|---|---|
| `select / dropdown` `[CORE]` | **Dataset de origen** — elige el `data_snapshot_id` ya ingerido sobre el que corre el plan | `custom_ui.Dropdown<String>({value, items, onChanged, hint: 'Selecciona un dataset'})`; radio `Gx.rInput` | reposo / abierto / seleccionado / deshabilitado mientras `procesando` |
| `select / dropdown` `[CORE]` | **Agregación destino** — timeframe objetivo de la barra OHLCV (`1m`/`5m`/`15m`/`1h`/`4h`/`1d`, mismo vocabulario que `timeframe` en `data_fetcher.dart`) | `custom_ui.Dropdown<String>({value, items, onChanged})` | reposo / abierto / seleccionado / deshabilitado mientras `procesando` |
| `number-input / stepper` `[CORE]` | **Umbral de interoperabilidad** (`INTEROP_THRESHOLD`) — filas máximas antes de rechazar la conversión a `ndarray` | `custom_ui.NumberInput({value, onChanged, min, max, step: 1000})` envuelto en `custom_ui.FormField({label: 'Umbral de interoperabilidad', helperText: 'Filas máx. antes de rechazar conversión a matriz'})` | reposo / foco / deshabilitado mientras `procesando` |
| `switch / toggle` `[CORE]` | **Hilos — modo Automático** (`POLARS_THREADS = Auto` por defecto) | `custom_ui.Switch({value: automatico, onChanged})` | off (manual) / on (automático, por defecto) |
| `number-input / stepper` `[CORE]` | **Hilos — cantidad explícita**, solo activo si el switch de Automático está en OFF | `custom_ui.NumberInput({value, onChanged, min: 1, max: coresLogicos, step: 1})`, deshabilitado (`enabled: false`) mientras `automatico == true` | reposo / foco / deshabilitado (auto ON o `procesando`) |
| `button-primary` `[CORE]` | Botón **"Ejecutar Transformación"** — dispara el plan armado con los 4 controles de arriba | `custom_ui.Button({label: 'Ejecutar Transformación', onPressed, variant: ButtonVariant.primary, enabled, loading})`; `Gx.gradOptima` + `glowStrong` | default / disabled (falta dataset) / loading (`procesando`) |
| `spinner / loader` `[CORE]` | Indicador de trabajo en curso durante la transformación (puede tardar sobre 5GB) | `ScanRingWidget({color: Gx.transitionIndigo, active: procesando})` envolviendo `custom_ui.Chip({label: 'Transformando…', status: ChipStatus.transition, pill: true})` — mismo primitivo ya en producción (`sovereign_data_fetcher_section.dart`, reutilizado en `verification-bench`) | activo solo durante `procesando` |
| `alert / banner / callout` `[CORE]` | Resultado de la corrida: éxito o error real del backend | `custom_ui.Banner({message, type: BannerType})` | `success` = "Transformación completada." (`optimaCyan`) · `error` = "Transformación falló: `<detalle>`" (`criticalCrimson`) |
| `key-value-row` `[CORE]` | **Filas de entrada/salida y duración** tras completar | `custom_ui.KeyValue({label, value, mono: true})` × 3: `rows_in`, `rows_out`, `transform_duration_ns` (formateado a ms/s en Dart — formateo de presentación, no cálculo de negocio) | valor en `dataMono`, color `Gx.textBase` |
| `code-block` (composición) | **Esquema resultante** — lista de columnas/tipos del `Bars` transformado, read-only | `SelectableText(schemaJson, style: Gx.dataMono(fontSize: 12, color: Gx.textBase))` dentro de `custom_ui.Surface` + `SingleChildScrollView` — mismo patrón que `verification-bench` (gap de catálogo compartido, ver abajo) | contenido normal; si la transformación falló, el bloque no se renderiza |
| `key-value-row` `[CORE]` | **Hash de lógica** (`logic_hash`) — truncado con copia al portapapeles | `custom_ui.KeyValue({label: 'logic_hash', value: hashCorto, mono: true})` + `IconButton` de copiar (patrón ya usado para IDs truncados en `sovereign_data_fetcher_section.dart`) | valor en `dataMono` |
| `empty-state` `[CORE]` | Estado inicial del bloque de resultado, antes de la primera ejecución | `custom_ui.Empty({message: 'Sin transformación ejecutada.', subtitle: 'Configura el plan y pulsa Ejecutar Transformación.'})` | idle |

### Estados Semánticos (Espectro de Vitalidad)

| Estado de negocio | Color token | Tratamiento visual completo |
|---|---|---|
| **Idle** (sin plan ejecutado) | `Gx.textBaseMuted` (neutro) | Bloque de resultado en `custom_ui.Empty`; botón habilitado solo si hay dataset seleccionado |
| **Procesando** (`Future` FFI en curso) | `transitionIndigo #9A8CFF` | `Button(loading: true)` + `ScanRingWidget` con `Chip('Transformando…')`; los 4 controles del plan se deshabilitan (`enabled: false`) para que el usuario no edite un plan a mitad de corrida |
| **Completado — éxito** | `optimaCyan #54E8D0` | `Banner(type: success)`: borde izq. 3px `optimaCyan`, `glow(optimaCyan)`. Debajo: `rows_in`/`rows_out`/duración en `KeyValue`, esquema en bloque mono, `logic_hash` truncado |
| **Falló** (rechazo por `INTEROP_THRESHOLD`, error de Polars, dataset corrupto) | `criticalCrimson #F0413F` | `Banner(type: error, message: 'Transformación falló: <detalle>')`: borde izq. 3px `criticalCrimson`, `glow(criticalCrimson)`. Sin bloque de esquema (no hay resultado que mostrar). El `<detalle>` es el error real devuelto por el backend — Techo Fijo ADR-0117: sin manejo elaborado de errores en frontend |
| **Card-node — estado del nodo** (Vista Relacional) | espectro completo | Reposo: borde 1px `borderPanel`. Procesando: `scanRing(transitionIndigo)` en el puerto de salida `bars_out`. Completado reciente: `sonarPulse(optimaCyan)` breve en el puerto de salida. Inválido (dataset roto): borde `criticalCrimson` parpadeante — misma anatomía de ADR-0136, sin spec adicional |

### Layout

**Card-node (Vista Relacional):** header 32–36px (icono + "Hybrid Data Transformer" en `displayGrotesque 13px 500` + chip de tipo "Transform") + hasta 4 key-values del plan activo (dataset corto, agregación destino, hilos, umbral) + puertos laterales (`raw_in`/`bars_in` a la izquierda fusionados visualmente como "datos crudos"; `bars_out` a la derecha siempre; `matrix_out` como puerto secundario solo si el último plan cruzó `INTEROP_THRESHOLD`). Gobernado por la infraestructura genérica del canvas — sin layout adicional aquí.

**Inspector Panel (right-side, ~380–420px de ancho):**
- Header: nombre de la feature + chip de estado semántico actual.
- Sección "Plan de transformación" (`Gx.space16` entre campos, densidad cómoda):
  1. `Dataset de origen` (`Dropdown`, ancho completo)
  2. `Agregación destino` (`Dropdown`, ancho completo)
  3. `Umbral de interoperabilidad` (`FormField` + `NumberInput`)
  4. `Hilos`: fila con `Switch` "Automático" + `NumberInput` condicional a la derecha
  5. `Button` "Ejecutar Transformación" — ancho completo, `Gx.space16` de separación superior
- Divider (`Gx.divider`) entre la sección de plan y la de resultado.
- Sección "Resultado": `Banner` → `Chip` espejo opcional → `KeyValue` × 3 (filas in/out, duración) → bloque de esquema (`SelectableText` mono, scrolleable) → `KeyValue` de `logic_hash` con botón copiar.

### Animaciones Aplicables

- [x] Clic de botón: hundimiento + propagación de luz (~460ms) — ya integrado en `custom_ui.Button`.
- [x] Foco de input: borde 1.5px + glow limpio (~200ms) — ya integrado en `custom_ui.NumberInput`/`Dropdown`.
- [x] Dropdown: `AnimatedSize` + chevron rotado 180° — ya integrado en `custom_ui.Dropdown`.
- [x] Switch: knob deslizante + `glowStrong` al encender — ya integrado en `custom_ui.Switch`.
- [x] Chip parpadeante / `scanRing`: `ScanRingWidget` durante `procesando`.
- [ ] Card-node: `sonarPulse(optimaCyan)` breve en `bars_out` al completar — gobernado por la infraestructura del canvas, no por esta feature; documentado aquí para que el Flutter Engineer lo dispare al recibir el resultado si el canvas ya expone ese hook.
- [ ] Zoom canvas: no aplica a esta sección (lo gestiona la infraestructura genérica del nodo).

### Entrada al Banco de Pruebas (SVF)

JSON de ejemplo precargado (mismo formato que consumiría el despachador genérico de `verification-bench`, sobre un dataset ya descargado por `sovereign-data-fetcher`):

```json
{
  "data_snapshot_id": "b3f1c2a0-...-000000000001",
  "target_aggregation": "1h",
  "interop_threshold": 100000,
  "polars_threads": null
}
```

`polars_threads: null` representa el modo "Automático" (POLARS_THREADS = Auto); un entero explícito lo reemplaza cuando el `Switch` está en OFF.

**Observable real esperado** (las mismas cuatro piezas que ya declara "Contrato de Integración UI" arriba): esquema resultante del `Bars` transformado (columnas + tipos, precio/volumen `Int64` ×10⁸, tiempo `Int64` ns UTC), `rows_in`/`rows_out`, `logic_hash` reproducible, y el tiempo de proceso real (`transform_duration_ns`) — nunca un valor mockeado. Si `interop_threshold` se fija deliberadamente bajo (ej. `100`) sobre un dataset mayor, el observable esperado es el rechazo documentado en Restricciones (la conversión a `ndarray` no se materializa) reflejado como `matrix_out` ausente, no como error genérico.

### Widget de Tablero (solo lectura)

Sigue el patrón real ya en producción de `SovereignDataFetcherDashboardWidget` (`ui/lib/tabs/sovereign_data_fetcher_dashboard_widget.dart`): `StatefulWidget` que carga en `initState()` vía `await` sobre un binding tipo `listTransformationRecords({dbPath})` (a construir por el Bridge Engineer, mismo patrón que `listDownloadRecords`) y muestra el registro más reciente dentro de `panelSurface()`/`custom_ui.BentoCard`.

- **Header:** icono + "Transformación de Datos" (`displayGrotesque 12px 500`, `Gx.textBaseLabel`).
- **Dato héroe:** `rows_out` de la última corrida en `dataMono 22px` con `textGlow(Gx.optimaCyan)` (éxito) o `Gx.criticalCrimson` (si la última corrida falló).
- **Key-values compactos:** `Dataset` (`data_snapshot_id` truncado a 8 chars), `Duración` (`transform_duration_ns` formateado), `logic_hash` (truncado a 8 chars).
- **Estado vacío:** "Sin transformaciones registradas." si la tabla append-only no tiene filas aún.
- Sin callbacks — es read-only, igual que su precedente; clic en el widget (si el Canvas ya está cableado) navega al card-node de esta feature, no abre edición en el propio Dashboard (regla FIJO ADR-0136: "El Dashboard es read-only").

### Notas de implementación para el Flutter Engineer

1. **No existe binding FFI todavía** (`ui/lib/src/rust/api/hybrid_data_transformer.dart` no existe aún — se confirmó por búsqueda directa). Cuando el Bridge Engineer lo genere, seguir el mismo patrón ya validado en `data_fetcher.dart`/`verification.dart`: función `Future<TransformResult> submitTransformation({...})` con `await` directo — **sin** `Timer.periodic` ni polling, igual que `submitDownloadJob`. El botón se deshabilita y pasa a `loading: true` mientras el `Future` no resuelve.
2. **`polars_threads` es `Option<u32>` en Rust** (Auto = `None`): el `Switch` "Automático" en ON debe mapear a `null`/ausente en el DTO enviado, no a un valor mágico como `0`. Verificar el nombre exacto del campo en el binding real antes de asumirlo.
3. **`INTEROP_THRESHOLD` no tiene rango fijo en el contrato de la feature** (tabla de Parámetros Configurables declara rango "-"). Los valores `min`/`max` del `NumberInput` en el mockup de esta Cáscara son placeholder de maquetación — el rango real debe venir del DTO/config que exponga el backend o quedar sin `max` explícito (solo `min: 0`) hasta que Rust-Engineer/Quant lo defina; no es una decisión de este skill.
4. **Listado de datasets de origen requiere una función de lectura nueva** (`list...` análoga a `listDownloadRecords`) que hoy no existe en ningún binding — es un requisito de Bridge Engineer para poblar el `Dropdown` de "Dataset de origen", no un componente de UI faltante.
5. **Formateo de duración/hash es presentación, no cálculo:** convertir `transform_duration_ns` a "1.2s"/"340ms" y truncar `logic_hash` a 8 caracteres son transformaciones de texto en Dart, permitidas bajo ADR-0106 (no son métricas analíticas).
6. **Reutilización:** todos los componentes de la sección "Plan de transformación" y "Resultado" ya existen como widgets funcionales en `ui/lib/components/` (`Dropdown`, `NumberInput`, `Switch`, `Button`, `Banner`, `KeyValue`, `Empty`, `Surface`, `FormField`) — no hay que crear ningún widget nuevo para el layout del panel inspector.

### Gaps de catálogo (para el Tech Lead / próxima iteración de DESIGN.md)

1. **No existe un `custom_ui.CodeBlock`/visor JSON dedicado en la librería funcional** (mismo gap ya documentado en `docs/features/verification-bench.md`, no es nuevo): el catálogo (`DESIGN.md §8`) mapea `code-block [CORE]` a `GlowCode`, que vive en la galería-showcase (render-only, no migrado a `ui/lib/components/`). Esta feature usa el mismo workaround funcional (`SelectableText` + `Gx.dataMono` dentro de `custom_ui.Surface`) para el bloque de esquema resultante. No bloquea la implementación.

Ninguno de los puntos anteriores bloquea la implementación del panel inspector ni del card-node: todos tienen una solución funcional inmediata documentada arriba.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local (procesamiento In-Memory / Out-of-Core).
- **Fidelidad (ADR-0017):** Alta — precisión exacta por escala entera ×10⁸ (ADR-0141), no coma flotante.
## Persistencia (Inundación de Fundamentos — ADR-0020)

**Perfil Técnico: A — Datos / Ingest** (Grupo I universal + Grupo III Linaje + Grupo IV Infraestructura). Tabla **append-only**: cada ejecución de transformación es un evento nuevo, nunca una actualización.

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la transformación (UUIDv7) |
| | `created_at` | Instante de persistencia del registro (ns UTC) |
| | `updated_at` | Igual a `created_at` en tabla append-only |
| | `audit_hash` | Hash del DataFrame resultante |
| | `audit_chain_hash` | Hash de la integridad del pipeline ETL (NULL en la fila génesis) |
| | `event_sequence_id` | Secuencia ordinal global, `UNIQUE` (append-only, ADR-0141 M9) |
| **III. Linaje** | `data_snapshot_id` | Ref al dataset crudo de entrada, formato canónico de ADR-0141 |
| | `transformation_id` | ID del paso de transformación dentro del pipeline |
| | `logic_hash` | Hash reproducible del plan de transformación |
| | `parent_id` | Transformación previa encadenada, cuando el pipeline tiene varios pasos |
| **IV. Infraestructura** | `node_id` | ID del hardware físico |
| | `process_id` | PID del motor de transformación |
| | `session_id` | Sesión de ingesta que originó la transformación |

**Campos propios fuera del catálogo de gobernanza** (excepción documentada de ADR-0020): `transform_duration_ns` (tiempo de proceso multihilo — derivado local de `execution_latency_ms`, que pertenece al Grupo V y no está autorizado en Perfil A), `rows_in`, `rows_out`, `schema_version` de la partición Parquet escrita (ADR-0141 F3).
