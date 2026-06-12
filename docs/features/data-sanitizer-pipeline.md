# Data Sanitizer Pipeline (ETL Soberana — 6 Capas de Limpieza)

**Carpeta:** `./features/data-sanitizer-pipeline/`
**Estado:** En Diseño
**Última actualización:** 2026-06-09
**Decisión Arquitectónica Asociada:** ADR-0037 (Protocolo de Calidad "The Sanitizer")

## ¿Qué es?

Es el guardian de la calidad de datos de Drasus Engine y el cerebro del Módulo Ingest. Implementa un protocolo institucional de 6 capas de limpieza profunda para transformar feeds de datos brutos (especialmente de proveedores como Tiingo, FirstRateData o Binance Vision) en datos "institucionales" antes de que lleguen al motor de backtest. Asegura la soberanía técnica eliminando outliers, rellenando huecos, validando la integridad del balance de precios y evitando que errores estructurales (gaps, precios anómalos) contaminen las estrategias.

**Problema:** Los datos crudos de brokers suelen tener huecos temporales o errores de OHLC que arruinan la fidelidad de las pruebas.
**Solución:** Un pipeline secuencial estricto de 6 capas que valida y repara la data bajo estándares profesionales.

## Comportamientos Observables (Pipeline de 6 Capas)

- [ ] **Capa 1: Ingesta Soberana.** Carga Parquet/CSV local con esquema estricto validado mediante schemas tipados nativos en Rust (Serde).
- [ ] **Capa 2: Gap Infusion (Detección y Relleno de Huecos).** Identifica huecos temporales en la serie (fines de semana, lag de red). Si el hueco es menor al umbral configurable (`MAX_GAP_FILL_LIMIT`), aplica interpolación lineal automática (Gap Auto-fill / Forward Fill); si lo excede, marca el tramo como `INVALID` para backtesting.
- [ ] **Capa 3: Outlier Scrubbing.** Elimina picos de precios imposibles (fat fingers) usando filtros de desviación estándar.
- [ ] **Capa 4: OHLC Sanity (Integrity Check).** Valida que `High >= (Open, Close, Low)` y `Low <= (Open, Close, High)`. Si una vela falla esta validación, la fila es descartada e incluida en el `error_log`, o reparada si es posible.
- [ ] **Capa 5: Volume & Spread Corroboration.** Filtra ticks con volumen cero o inconsistente. Si la diferencia entre Bid/Ask supera `SPREAD_SIGMA_THRESHOLD` (default 3σ), genera una alerta automática de anomalía de spread.
- [ ] **Capa 6: Point-in-Time Injection & Parquet Final Inundation.** Reconstruye el mercado ignorando eventos que ocurrieron "en el futuro" respecto a la fecha de simulación, y escribe el rastro auditable (ADR-0020 V2) en el archivo de salida.

## Restricciones

- NUNCA un dato sin sanitizar puede guardarse como "Golden Source".
- NUNCA se realiza interpolación en gaps mayores a `MAX_GAP_FILL_LIMIT` (configurable).
- El pipeline debe ser determinista: misma entrada cruda siempre produce misma salida limpia.
- Si la Capa 4 (OHLC Sanity) falla y `CLEAN_FLOW_STRICT` está activo, el job completo falla.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| GAP_AUTOFILL_ENABLED | True | True/False | Activa la interpolación de micro-gaps | CONFIG |
| MAX_GAP_FILL_LIMIT | 5 | 1 - 60 (bins) | Máximo de barras consecutivas a interpolar | CONFIG |
| SPREAD_SIGMA_THRESHOLD | 3.0 | 1.0 - 10.0 | Umbral de desviación para alertas de spread | CONFIG |
| CLEAN_FLOW_STRICT | True | True/False | Si true, falla el job ante cualquier error OHLC | [FIJO] |

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

### **TTR-003: Validador Forense de Sanidad (Sanity Audit)**
- **Descripción:** Genera estadísticas de salud del dataset (ej: "% de outliers eliminados", "% de gaps rellenados").
- **Postcondición:** Persistencia de metadatos en `data_inventory` con hash de integridad.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local.

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Toda sanitización registra el set de relevancia técnica para Datos:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la sanitización |
| | `created_at` | Timestamp de inicio del proceso |
| | `audit_hash` | Hash del dato limpio resultante |
| | `audit_chain_hash` | Hash de la secuencia de 6 capas de calidad |
| **II. Linaje** | `data_snapshot_id` | Ref al dataset crudo original (input) |
| | `transformation_id` | ID del paso de limpieza (Capa 1-6) |
| | `logic_hash` | Hash de la lógica del motor sanitizador |
| | `indicator_state_hash` | Estadísticas del dataset (outliers/gaps detectados) |
| **III. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del worker de limpieza |
| | `execution_latency_ms` | Tiempo total de sanitización |

## Dependencias
- [`pit-data-validator`](./pit-data-validator.md) — para la validación final post-limpieza (Validador PIT).
