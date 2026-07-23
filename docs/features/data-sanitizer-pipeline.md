# Data Sanitizer Pipeline (ETL Soberana — 6 Capas de Limpieza)

**Carpeta:** `./features/data-sanitizer-pipeline/`
**Estado:** En Diseño
**Última actualización:** 2026-07-23 (TASK-064 — Política de Integridad de Spread)
**Decisión Arquitectónica Asociada:** ADR-0037 (Protocolo de Calidad "The Sanitizer") · ADR-0155 (esquema canónico — `spread_reported` habilita la Capa 5) · ADR-0114 (sub-cláusula de regla (c), 2026-07-23 — esta feature es la dueña de la Política de Integridad de Spread que esa sub-cláusula exige) · ADR-0162 (`instrument-catalog`, fuente del piso de spread declarado por instrumento)

## ¿Qué es?

Es el guardian de la calidad de datos de Drasus Engine y el cerebro del Módulo Ingest. Implementa un protocolo institucional de 6 capas de limpieza profunda para transformar feeds de datos brutos (especialmente de proveedores como Tiingo, FirstRateData o Binance Vision) en datos "institucionales" antes de que lleguen al motor de backtest. Asegura la soberanía técnica eliminando outliers, rellenando huecos, validando la integridad del balance de precios y evitando que errores estructurales (gaps, precios anómalos) contaminen las estrategias.

**Problema:** Los datos crudos de brokers suelen tener huecos temporales o errores de OHLC que arruinan la fidelidad de las pruebas.
**Solución:** Un pipeline secuencial estricto de 6 capas que valida y repara la data bajo estándares profesionales.

## Comportamientos Observables (Pipeline de 6 Capas)

- [ ] **Capa 1: Ingesta Soberana.** Carga Parquet/CSV local con esquema estricto validado mediante schemas tipados nativos en Rust (Serde).
- [ ] **Capa 2: Gap Infusion (Detección y Relleno de Huecos).** Identifica huecos temporales en la serie (fines de semana, lag de red). Si el hueco es menor al umbral configurable (`MAX_GAP_FILL_LIMIT`), aplica interpolación lineal automática (Gap Auto-fill / Forward Fill); si lo excede, marca el tramo como `INVALID` para backtesting.
  > **Política de Integridad de Spread — caso NULO (ADR-0114 sub-cláusula, TASK-064 §1.1).** El campo `spread_reported` **no** se interpola linealmente como el OHLC — un spread es una cotización discreta que puede saltar por una noticia, no un precio continuo entre dos puntos conocidos. En su lugar se aplica **arrastre acotado** (`carry-forward`, LOCF): se propaga el último `spread_reported` realmente observado, hasta un umbral propio (`SPREAD_GAP_FILL_LIMIT`) — más allá del umbral, el campo se marca `INVALID` para ese tramo, mientras el OHLC de la misma barra puede seguir siendo válido si su propio gap no excede `MAX_GAP_FILL_LIMIT` (dos relojes de vencimiento independientes sobre la misma fila).
- [ ] **Capa 3: Outlier Scrubbing.** Elimina picos de precios imposibles (fat fingers) usando filtros de desviación estándar.
- [ ] **Capa 4: OHLC Sanity (Integrity Check).** Valida que `High >= (Open, Close, Low)` y `Low <= (Open, Close, High)`. Si una vela falla esta validación, la fila es descartada e incluida en el `error_log`, o reparada si es posible.
- [ ] **Capa 5: Volume & Spread Corroboration.** Filtra ticks con volumen cero o inconsistente (distinguiendo `volume_traded`/`volume_ticks`/`trade_count`/`volume_buy` — ADR-0155, nunca una columna `volume` genérica). Compara dos señales de spread: el derivado (`ask − bid` de `QuoteTick`) contra `spread_reported` cuando la fuente lo entrega como columna propia (ADR-0155 §"segunda observación independiente"); si la desviación entre ambos supera `SPREAD_SIGMA_THRESHOLD` (default 3σ), genera una alerta automática de anomalía — el defecto concreto que este esquema existe para atrapar es un bróker que reporta un spread que no coincide con su propia cotización.
  > **Política de Integridad de Spread — caso CERO (ADR-0114 sub-cláusula, TASK-064 §1.2).** Un `spread_reported = 0` explícito **no se acepta a ciegas** — un CSV M1 de 25 años exportado de un terminal minorista es exactamente el tipo de fuente con historial de reportar `0` por defecto de exportador en tramos antiguos, no por mercado real (aunque el `0` legítimo también existe, ej. solapes de sesión de máxima liquidez en brókers ECN Tier-1). Se aplica un **piso calibrado por instrumento**, en este orden: (1) `default_spread` declarado en el catálogo (`instrument-catalog`, puerto `resolved_terms_out` de ADR-0162) — el valor que el propietario declaró al configurar ese instrumento/bróker, nunca adivinado; (2) si no hay `default_spread` declarado, la distribución de los spreads no-cero del propio archivo (percentil bajo robusto) como calibración autocontenida. **Nunca** un piso fijo idéntico para todo instrumento, y **nunca** calibrado contra un dataset externo — ambos serían el mismo tipo de alucinación de cifra que esta política existe para evitar, solo que con signo distinto (uno optimista, uno pesimista arbitrario).
- [ ] **Capa 6: Point-in-Time Injection & Parquet Final Inundation.** Reconstruye el mercado ignorando eventos que ocurrieron "en el futuro" respecto a la fecha de simulación, y escribe el rastro auditable (ADR-0020) en el archivo de salida.

## Restricciones

- NUNCA un dato sin sanitizar puede guardarse como "Golden Source".
- NUNCA se realiza interpolación en gaps mayores a `MAX_GAP_FILL_LIMIT` (configurable).
- El pipeline debe ser determinista: misma entrada cruda siempre produce misma salida limpia.
- Si la Capa 4 (OHLC Sanity) falla y `CLEAN_FLOW_STRICT` está activo, el job completo falla.
- **(ADR-0114/TASK-064) NUNCA se interpola linealmente un spread nulo.** Se arrastra el último valor observado (`carry-forward`), acotado por `SPREAD_GAP_FILL_LIMIT`.
- **(ADR-0114/TASK-064) NUNCA un `spread_reported = 0` se acepta como hecho sin pasar por el piso calibrado por instrumento.** El piso nunca es una constante global ni se calibra contra un dataset externo.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| GAP_AUTOFILL_ENABLED | True | True/False | Activa la interpolación de micro-gaps | CONFIG |
| MAX_GAP_FILL_LIMIT | 5 | 1 - 60 (bins) | Máximo de barras consecutivas a interpolar | CONFIG |
| SPREAD_SIGMA_THRESHOLD | 3.0 | 1.0 - 10.0 | Umbral de desviación para alertas de spread | CONFIG |
| CLEAN_FLOW_STRICT | True | True/False | Si true, falla el job ante cualquier error OHLC | [FIJO] |
| SPREAD_GAP_FILL_LIMIT | 5 | 1 - 60 (bins) | Máximo de barras consecutivas para arrastrar (`carry-forward`) un `spread_reported` nulo antes de marcar el tramo `INVALID` — parámetro propio, no reutiliza `MAX_GAP_FILL_LIMIT` porque spread y OHLC pueden divergir en tolerancia (ADR-0114/TASK-064) | CONFIG |
| SPREAD_ZERO_FLOOR_PERCENTILE | percentil bajo (a fijar en Story de implementación) | 0.0 - 1.0 | Percentil de la distribución de spreads no-cero del propio archivo usado como piso cuando no hay `default_spread` declarado en el catálogo (ADR-0114/TASK-064 §1.2) | CONFIG |

## Ciclo de Vida de la Feature — Data Sanitizer Pipeline

### Entrada
- `Raw Data` (DataFrame de Polars).
- Configuración de filtros (Delisted, Corporate Events).

### Proceso
1. **Filtro de Delisting:** Elimina activos que ya no existían en el momento de la captura.
2. **Ajuste de Eventos Corporativos:** Aplica splits/dividendos para series históricas consistentes.
3. **Validador PIT:** Certifica que no hay look-ahead bias.
4. **Sanitización Física (6 Capas):** Ejecuta secuencialmente las Capas 1-6 descritas arriba para corregir OHLC y rellenar micro-gaps.

### Salida
- `Clean Data` (`Sanitized_Dataframe`, Parquet) lista para persistencia.
- Reporte detallado de anomalías detectadas (`error_log`, estadísticas de salud del dataset).

### Contextos de Uso

**Contexto 1: Ingesta ETL (Ingest)**
- Limpieza masiva de históricos tras descarga Bulk.

**Contexto 2: Feed Profesional (Live)**
- Validación en tiempo real de datos entrantes de proveedores institucionales.

## Tareas (TTRs)

### **TTR-001: Implementación de Pipeline Secuencial**
- Construye la lógica de flujo: `Raw → Delisted → Adjuster → PIT → Clean` ejecutando las 6 capas de limpieza en Polars para máxima velocidad.

### **TTR-002: Algoritmo de Gap Detection e Interpolación**
- Desarrolla la detección de discontinuidades temporales (Gap Infusion) y el motor de relleno lineal (Forward Fill).

### **TTR-003: Validador de Sanidad (Sanity Audit)**
- **Descripción:** Genera estadísticas de salud del dataset (ej: "% de outliers eliminados", "% de gaps rellenados").
- **Postcondición:** Persistencia de metadatos en `data_inventory` con hash de integridad.

### **TTR-004: Política de Integridad de Spread (ADR-0114/TASK-064)**
- **Descripción:** Extiende la Capa 2 (arrastre acotado para `spread_reported` nulo) y la Capa 5 (piso calibrado por instrumento para `spread_reported = 0`), consumiendo el `default_spread` resuelto de `instrument-catalog` como primera fuente de piso.
- **Reglas de Negocio:**
    * Nulo → `carry-forward` acotado por `SPREAD_GAP_FILL_LIMIT`, nunca interpolación lineal.
    * Cero → piso desde `default_spread` declarado (catálogo) o, en su ausencia, percentil bajo de la distribución de spreads no-cero del propio archivo — nunca constante global, nunca calibrado contra un dataset externo.
- **Entrada:** `spread_reported` crudo de la barra/tick, `ResolvedInstrumentTerms` (de `instrument-catalog`, opcional — su ausencia degrada al percentil del propio archivo).
- **Salida:** `spread_reported` saneado + marca `INVALID` en los tramos que excedieron el umbral de arrastre.
- **Precondición:** Capa 1 (ingesta) completada.
- **Postcondición:** El piso aplicado y su fuente (declarado vs. percentil del archivo) quedan registrados en el reporte de anomalías de la Capa 5, para auditoría.

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `raw_data_in` | `Bars` | Input | 1 | Dataset crudo a sanitizar (Capas 1-6) |
| `resolved_terms_in` | `ResolvedInstrumentTerms` | Input | 0..1 | Términos resueltos del instrumento (de `instrument-catalog`), fuente primaria del piso de spread (Capa 5); su ausencia degrada al percentil del propio archivo |
| `sanitized_data_out` | `SanitizedDataframe` | Output | 1 | Dataset post-6 capas, incluida la Política de Integridad de Spread |

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local.

## Persistencia (Inundación de Fundamentos — ADR-0020)

Toda sanitización registra el set de relevancia técnica para Datos:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la sanitización |
| | `created_at` | Timestamp de inicio del proceso |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash del dato limpio resultante |
| | `audit_chain_hash` | Hash de la secuencia de 6 capas de calidad |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **III. Linaje** | `data_snapshot_id` | Ref al dataset crudo original (input) |
| | `transformation_id` | ID del paso de limpieza (Capa 1-6) |
| | `logic_hash` | Hash de la lógica del motor sanitizador |
| | `indicator_state_hash` | Estadísticas del dataset (outliers/gaps detectados) |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del worker de limpieza |
| | `execution_latency_ms` | Tiempo total de sanitización |

## Dependencias
- [`pit-data-validator`](./pit-data-validator.md) — para la validación final post-limpieza (Validador PIT).
- [`instrument-catalog`](./instrument-catalog.md) — fuente del `default_spread` declarado que alimenta el piso de la Política de Integridad de Spread (ADR-0114/TASK-064); opcional, degrada al percentil del propio archivo si no está disponible.
