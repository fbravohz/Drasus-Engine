# Ingest

**Carpeta:** `./modules/ingest/`
**Estado:** Orquestador (Imperative Shell)
**Última actualización:** 2026-04-12

---

## ¿Qué es?

El módulo de ingesta es la puerta de entrada de todos los datos del mercado. Recibe precios (velas OHLCV: apertura, máximo, mínimo, cierre, volumen), los valida, los normaliza, les asigna un "régimen de mercado" (¿está el mercado en tendencia, lateral, o volátil?), y los guarda para que los demás módulos los usen.

Si los datos que entran están sucios o rotos, todo lo que venga después estará mal. Por eso este módulo es el guardián de la calidad de datos.

---

## Épica 0: Esqueleto Fundacional

### Estructura de Archivos (FCIS — ADR-0003)

```
crates/ingest/
├── public_interface.rs   # Frontera pública: único punto de entrada para otros módulos
├── logic.rs              # Lógica pura: validaciones OHLCV, normalización (sin DB, sin I/O)
├── orchestrator.rs       # Coordinación: invoca features, maneja errores, emite eventos
├── persistence.rs        # Acceso a SQLite WAL y Parquet (lectura/escritura)
├── schemas.rs            # Definición de tablas: market_data, regimes, quality_logs
└── types.rs              # Tipos de entrada/salida: RawBar, ValidatedBar, RegimeLabel
```

### Vocabulario de Persistencia — Catálogo de 25 Campos (ADR-0020 V2)

Esta tabla es el **catálogo de referencia completo** del Contrato Global de ADR-0020 V2 (vocabulario lógico, no esquema literal). La migración 0001 crea la tabla ancla `foundation_master_fields` con estas 25 columnas como referencia ÚNICA del sistema — este módulo NO la replica.

Las tablas propias de este módulo (una por feature/TTR, en sus propias migraciones) llevan: el **Grupo I (Identidad & Integridad, 6 primeras filas) de forma universal y obligatoria**, más solo los campos concretos de los Grupos II–V que correspondan al **Perfil Técnico** de cada feature (Filtro de Relevancia, tabla canónica en ADR-0020 V2) — nunca el catálogo completo. Cada feature documenta su selección en su propia sección "Contrato de Persistencia" (`features/*.md`).

| Categoría | Campo | Descripción |
|---|---|---|
| **I. Identidad e Integridad** | `id` | UUID del registro |
| | `created_at` | Timestamp de creación (nanosegundos) |
| | `updated_at` | Timestamp de última modificación |
| | `audit_hash` | SHA-256 del contenido del registro |
| | `audit_chain_hash` | Hash encadenado al registro anterior |
| | `event_sequence_id` | Secuencia de recuperación post-crash |
| **II. Soberanía y Propiedad** | `owner_id` | Dueño del capital/IP |
| | `institutional_tag` | Etiqueta de entorno (PROD/PAPER/CHALLENGE) |
| | `manifest_id` | Contrato de diseño vinculado |
| | `access_token_id` | Token de autenticación usado |
| **III. Linaje Alpha y Datos** | `version_node_id` | Nodo en el DAG de versiones |
| | `parent_id` | Puntero al registro padre |
| | `logic_hash` | Hash del motor/driver que procesó el dato |
| | `data_snapshot_id` | Snapshot PIT del mercado |
| | `transformation_id` | ID del paso/tipo de transformación aplicado |
| **IV. Infraestructura y Ops** | `process_id` | PID del worker |
| | `session_id` | Agrupación de runtime |
| | `node_id` | ID del hardware físico |
| **V. Forense y Ejecución** | `portfolio_container_id` | Contenedor de portafolio |
| | `compliance_status_id` | Veredicto de riesgo |
| | `risk_audit_id` | Ticket detallado de riesgo |
| | `indicator_state_hash` | Snapshot técnico |
| | `execution_latency_ms` | Latencia de procesamiento |
| | `source_signal_id` | Link a señal origen |
| | `signature_hash` | HMAC de señales |

### TTRs Etiquetados por Fase

| TTR | Fase | Descripción corta |
|---|---|---|
| TTR-004 | **EPIC-1** | Limpieza química (Data Sanitizer) |
| TTR-006 | **EPIC-1** | Descarga híbrida (Sovereign Fetcher) |
| TTR-007 | **EPIC-1** | Transformación de alto rendimiento (Hybrid Transformer) |
| TTR-008 | **EPIC-1** | Normalización multi-broker |
| TTR-001 | **EPIC-1** | Validación estructural (Data Validator) |
| TTR-002 | **EPIC-1** | Causalidad temporal (PIT Validator) |
| TTR-010 | **EPIC-1** | Almacenamiento Hive-Style (Partition Manager) |
| TTR-005 | **EPIC-1** | Persistencia soberana (DuckDB/Parquet) |
| TTR-011 | **EPIC-1** | Remuestreo dinámico (DuckDB Resampler) |
| TTR-012 | **EPIC-1** | Importación manual (Import Wizard) |
| TTR-009 | **EPIC-1** | Monitoreo de progreso (Download Manager) |
| TTR-013 | **EPIC-1** | Auditoría visual de calidad (Heatmap) |
| TTR-003 | EPIC-3 | Contextualización (HMM Regime) |
| TTR-014 | **EPIC-1** | Remuestreo algorítmico (Algorithmic Bars) |
| TTR-017 | **EPIC-1** | Microestructura histórica (Order Flow — CVD) |
| TTR-018 | **EPIC-1** | Marcado temporal (Clock) |
| TTR-019 | **EPIC-1** | Memoria estadística (Fractional Differencer) |
| TTR-020 | EPIC-3 | Selección de universo accionario |
| TTR-021 | EPIC-4 | Etiquetado manual de regímenes |
| TTR-015 | EPIC-8 | Inicialización de entorno local (Flutter FFI) |
| TTR-016 | EPIC-8 | Navegación infinita (ZUI Navigation) |
| TTR-999 | **EPIC-1** | Protocolo Fail-Fast Safe (ADR-0066) |

---

## Comportamientos Observables

- [ ] Recibo datos de precios del broker → el sistema los valida y guarda solo los buenos
- [ ] Si llegan datos rotos (precio negativo, volumen cero, OHLC inconsistente), el sistema los rechaza y lo registra en logs
- [ ] Cada barra de precio guardada tiene su "régimen" asignado: TRENDING (mercado en tendencia), MEAN_REVERTING (mercado lateral), o VOL_EXPANSION (mercado volátil)
- [ ] Si el sistema no tiene suficientes datos para clasificar el régimen, lo marca como UNKNOWN explícitamente (no lo inventa)
- [ ] Genera barras de tiempo personalizadas (ej: 17m) usando el [temporal-aggregator](../features/temporal-aggregator.md).
- [ ] Cualquier otro módulo puede preguntar "dame las últimas N barras validadas de BTC/USDT" y obtener los datos

---

## Restricciones

- NUNCA un dato sin validar puede salir del módulo hacia otros módulos
- NUNCA se guarda un precio negativo, un volumen negativo, o un OHLC inconsistente (ej: High < Low)
- NEVER el módulo inventa un régimen — si no hay datos suficientes, dice UNKNOWN
- Los rechazos siempre se registran en logs (nunca silencio total)

---

## Parámetros Configurables

| Parámetro | Default | Qué hace |
|---|---|---|
| REGIME_LOOKBACK | configurable | Cuántas barras pasadas se usan para detectar régimen |
| OUTLIER_THRESHOLD | configurable | Qué tan "raro" debe ser un precio para considerarse outlier |
| VALIDATION_STRICT | true | Si true, rechaza cualquier anomalía; si false, intenta recuperar |

---

## Ciclo de Vida: Ingest

### Entrada
- Datos OHLCV crudos desde broker (gRPC/WebSocket, API directa, archivo CSV)
- Timestamps, volumen, símbolos

### Proceso
1. **Extracción Híbrida:** Invoca [`sovereign-data-fetcher`](../features/sovereign-data-fetcher.md) para Bulk/Delta.
2. **Transformación:** Invoca [`hybrid-data-transformer`](../features/hybrid-data-transformer.md) (Polars 80/20).
3. **Normalización:** Invoca [`data-normalization-layer`](../features/data-normalization-layer.md).
4. **Validaciones:**
   - **Lógica:** [`data-validator`](../features/data-validator.md).
   - **PIT:** [`pit-data-validator`](../features/pit-data-validator.md).
5. **Detección Régimen:** Invoca [`hmm-regime-detection`](../features/hmm-regime-detection.md) (HMM + ARIMA).
6. **Enriquecimiento del Flujo:** Invoca [`order-flow-microstructure`](../features/order-flow-microstructure.md) (CVD/OFI).
7. **Auditoría:** Registra flujo en [`audit-log`](../features/audit-log.md).

### Salida
- Barras normalizadas y persistidas en **Parquet** (OLAP) y **SQLite** (Estado).
- Veredicto de continuidad de datos (Golden Source).
- Telemetría de progreso en UI via [`background-download-manager`](../features/background-download-manager.md).

---

## Features Consumidas (Reutilizables)

- **[`sovereign-data-fetcher`](../features/sovereign-data-fetcher.md)** — Ingesta híbrida (Bulk S3 + API Delta) saturando ancho de banda.
- **[`hybrid-data-transformer`](../features/hybrid-data-transformer.md)** — Transformación de alto rendimiento (Polars 80/20).
- **[`data-sanitizer-pipeline`](../features/data-sanitizer-pipeline.md)** — Pipeline obligatorio de calidad (Gap filling, OHLC check).
- **[`hive-partition-manager`](../features/hive-partition-manager.md)** — Organización de archivos Parquet con particionado Hive.
- **[`duckdb-resampler`](../features/duckdb-resampler.md)** — Remuestreo dinámico SQL sin redundancia física.
- **[`data-normalization-layer`](../features/data-normalization-layer.md)** — Unificación de símbolos y tipado multi-broker.
- **[`data-import-wizard`](../features/data-import-wizard.md)** — Asistente de importación manual de CSV/TXT.
- **[`quality-heatmap-generator`](../features/quality-heatmap-generator.md)** — Auditoría visual de la integridad de los datos.
- **[`background-download-manager`](../features/background-download-manager.md)** — Gestión de jobs de descarga con telemetría para la UI.
- **[`data-validator`](../features/data-validator.md)** — Validación lógica OHLCV.
- **[`pit-data-validator`](../features/pit-data-validator.md)** — Validación Point-In-Time.
- **[`hmm-regime-detection`](../features/hmm-regime-detection.md)** — Asignación de régimen de mercado.
- **[`duckdb-sql-engine`](../features/duckdb-sql-engine.md)** — Motor para consultas multihilo sobre Parquet.
- **[`clock`](../features/clock.md)** — Timestamps deterministas.
- **[`algorithmic-bars`](../features/algorithmic-bars.md)** — Generación de barras sintéticas por Tick/Volumen/Rango.
- **[`flutter-packaging-manager`](../features/flutter-packaging-manager.md)** — Gestión de binarios y servicios locales.
- **[`zui-navigation`](../features/zui-navigation.md)** — Exploración visual de datasets masivos.
- **[`order-flow-microstructure`](../features/order-flow-microstructure.md)** — Enriquecimiento de barras con CVD, OFI y VWAP.
- **[`fractional-differencer`](../features/fractional-differencer.md)** — Preservación de memoria estadística via diferenciación fraccional (ADR-0064).
- **[`manual-regime-tagger`](../features/manual-regime-tagger.md)** — Etiquetado visual manual de zonas de crisis sobre el activo de referencia.

---

---

## Tareas (TTRs) — Protocolo de Orquestación (§8.1)

### **TTR-001: Orquestación de Limpieza Estructural (Data Validator)**
*   **Descripción:** Invoca a [`data-validator`](../features/data-validator.md) para garantizar la integridad física de las barras recibidas.
*   **Reglas de Orquestación:**
    * Si la validación falla, se aborta la ingesta de la barra y se registra en `data_quality_logs`.
    * Toda barra aceptada debe recibir un `audit_hash` del orquestador (ADR-0020 V2).
*   **Entrada:** `raw_market_data`.
*   **Salida:** `validated_struct_bar`.
*   **Precondición:** Suscripción al stream del broker activa.
*   **Postcondición:** Barra lista para validación de causalidad temporal.

### **TTR-002: Orquestación de Causalidad Temporal (PIT Validator)**
*   **Descripción:** Invoca a [`pit-data-validator`](../features/pit-data-validator.md) para certificar la ausencia de look-ahead bias.
*   **Reglas de Orquestación:**
    * Verifica que el timestamp sea monótonamente creciente respecto al `last_persisted_bar`.
    * Se debe inyectar el `process_id` del job de ingesta en cada registro (ADR-0020 V2).
*   **Entrada:** `validated_struct_bar`.
*   **Salida:** `pit_certified_bar`.
*   **Precondición:** TTR-001 finalizado exitosamente.
*   **Postcondición:** Barra certificada para persistencia histórica.

### **TTR-003: Orquestación de Contextualización (HMM Regime)**
*   **Descripción:** Invoca a [`hmm-regime-detection`](../features/hmm-regime-detection.md) para asignar el contexto de mercado.
*   **Reglas de Orquestación:**
    * Si el modelo HMM no tiene convergencia, asignar `REGIME_UNKNOWN`.
    * El veredicto de régimen se vincula al `version_node_id` del modelo (ADR-0020 V2).
*   **Entrada:** `pit_certified_bar`, `regime_model`.
*   **Salida:** `contextualized_bar` (Bar + RegimeID).
*   **Precondición:** TTR-002 finalizado.
*   **Postcondición:** Persistencia final de la barra "Golden Source".

### **TTR-004: Orquestación de Limpieza Química (Data Sanitizer)**
*   **Descripción:** Invoca a [`data-sanitizer-pipeline`](../features/data-sanitizer-pipeline.md) para eliminar ruido estructural y valores atípicos.
*   **Reglas de Orquestación:**
    * Debe ejecutarse antes de cualquier validación lógica pesada.
    * Los cambios realizados por el sanitizer deben ser auditables vía `transformation_id` (ADR-0020 V2).
*   **Entrada:** `raw_binary_stream`.
*   **Salida:** `sanitized_data`.
*   **Precondición:** Recepción de datos crudos.
*   **Postcondición:** Datos listos para `data-validator`.


### **TTR-005: Orquestación de Persistencia Soberana (DuckDB/Parquet)**
*   **Descripción:** Invoca a [`duckdb-sql-engine`](../features/duckdb-sql-engine.md) para historizar las barras en formato Parquet particionado.
*   **Reglas de Orquestación:**
    * El orquestador garantiza que no se escriban duplicados en el archivo Parquet.
    * Cada bloque de escritura debe certificar el `audit_hash` del archivo resultante.
*   **Entrada:** `contextualized_bar`.
*   **Salida:** `parquet_file_update`.
*   **Precondición:** TTR-003 finalizado.
*   **Postcondición:** Datos disponibles para consulta Out-of-Core por otros módulos.

### **TTR-006: Orquestación de Descarga Híbrida (Sovereign Fetcher)**
*   **Descripción:** Invoca a [`sovereign-data-fetcher`](../features/sovereign-data-fetcher.md) para saturar el ancho de banda en descargas masivas.
*   **Reglas de Orquestación:**
    * Debe priorizar la fuente Bulk si existe para el rango solicitado.
    * El `process_id` del job de descarga debe ser único y persistente (ADR-0020 V2).
*   **Entrada:** `symbol_standard_request`.
*   **Salida:** `raw_segmented_data` (Bulk files + API deltas).
*   **Precondición:** Espacio en disco validado.
*   **Postcondición:** Datos listos para normalización estructural.

### **TTR-007: Orquestación de Transformación de Alto Rendimiento (Hybrid Transformer)**
*   **Descripción:** Invoca a [`hybrid-data-transformer`](../features/hybrid-data-transformer.md) para el procesamiento multihilo de millones de filas.
*   **Reglas de Orquestación:**
    * Monitorea el uso de CPU para no bloquear otros pipelines (ADR-0012).
    * Toda transformación debe quedar registrada bajo un `logic_hash` único (ADR-0020 V2).
*   **Entrada:** `raw_segmented_data`.
*   **Salida:** `transformed_dataframe` (Polars).
*   **Precondición:** TTR-006 finalizado.
*   **Postcondición:** Datos optimizados para validadores de calidad.

### **TTR-008: Orquestación de Normalización Multi-Broker (Normalization Layer)**
*   **Descripción:** Invoca a [`data-normalization-layer`](../features/data-normalization-layer.md) para unificar esquemas de diferentes fuentes.
*   **Reglas de Orquestación:**
    * Valida que el `institutional_tag` sea correcto para el mapeo (ADR-0020 V2).
    * Escala precios a la precisión interna predefinida.
*   **Entrada:** `transformed_dataframe`.
*   **Salida:** `normalized_golden_source`.
*   **Precondición:** TTR-007 finalizado.
*   **Postcondición:** Datos listos para `data-validator` y `pit-validator`.

### **TTR-009: Monitoreo de Progreso en Segundo Fondo (Download Manager)**
*   **Descripción:** Invoca a [`background-download-manager`](../features/background-download-manager.md) para informar estados a la UI.
*   **Reglas de Orquestación:**
    * Emite latidos de progreso cada N ms a través del gRPC/WebSocket de orquestación.
    * Vincula cada sesión de descarga al `owner_id` (ADR-0020 V2).
*   **Entrada:** `download_job_status`.
*   **Salida:** `ui_progress_updates`.
*   **Precondición:** TTR-006 iniciado.
*   **Postcondición:** Visibilidad total del proceso para el usuario final.

### **TTR-010: Orquestación de Almacenamiento Hive-Style (Partition Manager)**
*   **Descripción:** Invoca a [`hive-partition-manager`](../features/hive-partition-manager.md) para organizar los archivos Parquet.
*   **Reglas de Orquestación:**
    * Valida que el `data_root` esté configurado via configuración tipada validada en Rust (Serde).
    * Asegura el particionado por `exchange/symbol/timeframe/year/month`.
*   **Entrada:** `sanitized_golden_source_dataframe`.
*   **Salida:** `hive_directory_structure`.
*   **Precondición:** TTR-008 finalizado.
*   **Postcondición:** Almacenamiento optimizado para Partition Pruning.

### **TTR-011: Orquestación de Remuestreo Dinámico (DuckDB Resampler)**
*   **Descripción:** Invoca a [`duckdb-resampler`](../features/duckdb-resampler.md) para generar temporalidades superiores.
*   **Reglas de Orquestación:**
    * Verifica que la solicitud cumpla la Regla de Múltiplos (Fuente <= Target).
    * El resultado se entrega en formato Apache Arrow para consumo directo en UI o Motor.
*   **Entrada:** `resample_request` (TF: 21m).
*   **Salida:** `arrow_candle_stream`.
*   **Precondición:** Datos 1m o Ticks disponibles en Parquet.
*   **Postcondición:** Visualización o Backtesting en periodicidades personalizadas.

### **TTR-012: Orquestación de Importación Manual (Import Wizard)**
*   **Descripción:** Invoca a [`data-import-wizard`](../features/data-import-wizard.md) para integrar archivos externos.
*   **Reglas de Orquestación:**
    * Todo dato importado manualmente debe pasar obligatoriamente por TTR-008 (Sanitizer).
    * Registra el `logic_hash` del parser utilizado en la auditoría.
*   **Entrada:** `local_csv_file`.
*   **Salida:** `imported_asset_data`.
*   **Precondición:** Previsualización aceptada por el usuario.
*   **Postcondición:** Datos externos asimilados en el módulo Ingest.

### **TTR-013: Auditoría de Calidad Visual (Heatmap Generator)**
*   **Descripción:** Invoca a [`quality-heatmap-generator`](../features/quality-heatmap-generator.md) para generar el mapa de integridad.
*   **Reglas de Orquestación:**
    * El proceso se ejecuta de forma asíncrona tras cada job de ingesta masiva o importación.
    * El `Asset Integrity Score` se vincula al `audit_chain_hash`.
*   **Entrada:** `market_data_partition`.
*   **Salida:** `integrity_heatmap_json`.
*   **Precondición:** Datos persistidos en Parquet.
*   **Postcondición:** Diagnóstico visual de la salud del dataset.
### **TTR-014: Orquestación de Remuestreo Algorítmico (Algorithmic Bars)**
*   **Descripción:** Invoca a [`algorithmic-bars`](../features/algorithmic-bars.md) para generar barras no-temporales (Tick, Renko, Volume).
*   **Reglas de Orquestación:**
    * El motor debe disparar la creación de barras cada vez que se alcance el umbral configurado.
    * Cada barra generada hereda el `audit_hash` del bloque de ticks origen.
*   **Entrada:** `tick_stream`, `bar_threshold`.
*   **Salida:** `algorithmic_bar_stream`.
*   **Precondición:** TTR-006 finalizado.
*   **Postcondición:** Barras disponibles para entrenamiento AI en `generate`.

### **TTR-015: Inicialización de Entorno Local (Flutter FFI)**
*   **Descripción:** Utiliza [`flutter-packaging-manager`](../features/flutter-packaging-manager.md) para levantar los servicios de backend y visor necesarios para la ingesta.
*   **Entradas:** Entorno del sistema operativo, variables de entorno.
*   **Proceso:** Despliega el entorno `Flutter FFI` si es necesario o asegura que los servicios backend estén listos para Ingesta.
*   **Salidas:** Confirmación de entorno listo.
*   **Entrada:** `tauri_config`.
*   **Salida:** `service_health_status`.
*   **Precondición:** Arranque del sistema.
*   **Postcondición:** Backend listo para recibir tráfico de datos.

### **TTR-016: Orquestación de Navegación Infinita (ZUI Navigation)**
*   **Descripción:** Invoca a [`zui-navigation`](../features/zui-navigation.md) para permitir la exploración visual de los datos ingestados.
*   **Reglas de Orquestación:**
    * Coordina el zoom desde visión de meses a visión de ticks usando el resample dinámico.
    * Inyecta marcas de régimen técnico en el eje temporal.
*   **Entrada:** `zui_viewport_coordinates`.
*   **Salida:** `visual_data_tiles`.
*   **Precondición:** TTR-005 (DuckDB) con datos disponibles.
*   **Postcondición:** Fluidez visual garantizada para el usuario.

### **TTR-017: Orquestación de Microestructura Histórica (Order Flow — CVD)**
*   **Descripción:** Invoca a [`order-flow-microstructure`](../features/order-flow-microstructure.md) (parte histórica del split ADR-0118) para calcular el Cumulative Volume Delta sobre el registro de transacciones ya ingestado, en cada bloque de barras.
*   **Reglas de Orquestación:**
    * El valor de CVD se normaliza y se adjunta como metadato de la barra.
    * La parte en vivo (OFI / DOM L2) NO se calcula aquí: es la guardia pre-trade de `execute` (su TTR-011).
*   **Entrada:** `arrow_candle_stream`, registro de transacciones a nivel de operación (ej.: aggTrades).
*   **Salida:** `enriched_golden_source` (Precios + CVD histórico).
*   **Precondición:** TTR-008 (Normalización) finalizado; datos a nivel de operación ingestados.
*   **Postcondición:** Golden source enriquecida con microestructura histórica, disponible para `generate` y `validate`.

### **TTR-018: Orquestación de Marcado Temporal (Clock)**
*   **Descripción:** Invoca a [`clock`](../features/clock.md) para garantizar monotonía determinista.
*   **Reglas de Orquestación:**
    *   Sella criptográficamente el timestamp de cada barra ingresada en la base de datos (Parquet/SQLite).
*   **Entrada:** `raw_bar_timestamp`.
*   **Salida:** `deterministic_epoch`.
*   **Precondición:** Dato crudo recibido.
*   **Postcondición:** Prevención absoluta de look-ahead por desajustes horarios.

### **TTR-019: Orquestación de Memoria Estadística (Fractional Differencer)**
*   **Descripción:** Invoca a [`fractional-differencer`](../features/fractional-differencer.md) para generar series temporales estacionarias con memoria preservada.
*   **Reglas de Orquestación:**
    * Debe ejecutarse como paso opcional de enriquecimiento tras la normalización.
    * Si se solicita `auto_d`, el orquestador coordina la búsqueda iterativa del orden óptimo.
    * La serie resultante se persiste en Parquet con el sufijo `_fracdiff` vinculado al `audit_hash` original (ADR-0020 V2).
*   **Entrada:** `normalized_golden_source`.
*   **Salida:** `stationary_memory_series`.
*   **Precondición:** TTR-008 finalizado.
*   **Postcondición:** Datos estacionarios disponibles para entrenamiento AI y generación de señales.

### **TTR-020: Orquestación de Selección de Universo Accionario (StockPicker Configurator)**
*   **Descripción:** Invoca a [`visual-stockpicker-configurator`](../features/visual-stockpicker-configurator.md) para filtrar dinámicamente el universo de equities.
*   **Reglas de Orquestación:**
    - Traduce los umbrales numéricos de los sliders de fundamentales/ADTV a consultas DuckDB en caliente.
    - Registra el `source_id` del universo de activos seleccionado (ADR-0020 V2).
*   **Entrada:** `universe_filter_parameters`.
*   **Salida:** `filtered_asset_universe_list`.
*   **Precondición:** Metadatos de activos disponibles en SQLite/Parquet.
*   **Postcondición:** Lista de activos elegibles cargada en el pipeline.

### **TTR-021: Orquestación del Etiquetado Manual de Regímenes (Manual Regime Tagger)**
*   **Descripción:** Invoca a [`manual-regime-tagger`](../features/manual-regime-tagger.md) para que el analista sombree y etiquete periodos de crisis sobre el activo de referencia.
*   **Reglas de Orquestación:**
    *   Las zonas etiquetadas se persisten como metadato de intervalo temporal (NUNCA alteran el dato OHLCV original).
    *   Las zonas quedan disponibles para que el embudo de robustez del módulo `validate` aplique reglas duras por zona.
*   **Entrada:** `reference_asset_series`, `user_drawn_ranges`, `zone_labels`.
*   **Salida:** `tagged_zones` (intervalos + etiquetas reutilizables).
*   **Precondición:** TTR-005 (DuckDB) con serie del activo de referencia disponible.
*   **Postcondición:** Zonas etiquetadas persistidas y enlazadas al proyecto/activo.

### **TTR-999: Implementación del Protocolo Fail-Fast Safe (ADR-0066)**
*   **Descripción:** Garantizar que cualquier invocación a componentes de validación o procesamiento intensivo esté gobernada por la cascada de intensidad.
*   **Reglas de Orquestación:**
    *   **Short-Circuit Mandatorio:** El módulo debe validar el éxito de los filtros `LIGHT` antes de solicitar recursos para tareas `MEDIUM` o `HEAVY`.
    *   **Telemetry:** Registrar el ahorro de ciclos de CPU/GPU cuando se produzca un descarte temprano.
*   **Entrada:** `ComputeIntensityMetadata`.
*   **Salida:** `fail_fast_execution_status`.
*   **Postcondición:** Optimización del consumo de hardware bajo el principio de Soberanía Local (ADR-0032).

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundamentos (ADR-0020 V2):** El catálogo de los 25 campos maestros está en la sección "Épica 0: Esqueleto Fundacional" de este documento (referencia, no esquema). Toda entidad persistida por este módulo incluye el Grupo I de forma universal; los Grupos II–V se aplican solo en los campos que el Perfil Técnico de cada feature exige (Filtro de Relevancia, ADR-0020 V2) — nunca el catálogo completo.

- **Decisión Arquitectónica Asociada:**
    - ADR-0002: Functional Core / Imperative Shell.
    - ADR-0013: Stack Tecnológico (NautilusTrader).
    - ADR-0020 V2: Inundación de Fundaciones.

---

## Dependencias
**Depende de:**
- [`infrastructure-setup`](../features/infrastructure-setup.md) — para la cimentación del sistema.

**Consumido por:**
- [`generate`](../modules/generate.md) — para alimentar el motor de descubrimiento de alfas.
- [`validate`](../modules/validate.md) — para proporcionar datos de prueba certificados.
