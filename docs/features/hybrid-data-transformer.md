# Hybrid Data Transformer

**Carpeta:** `./features/hybrid-data-transformer/`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0105 (Estrategia de Datos Híbrida Polars/ndarray), ADR-0013 (Stack 100% Rust Nativo)

## ¿Qué es?

Es el motor de transformación que aplica la regla **80/20**: utiliza el máximo rendimiento de **Polars** para el procesamiento masivo (ETL, limpieza, agregaciones) y reserva **`ndarray`/`linfa`** exclusivamente para el consumo estadístico y de modelos de ML nativos en Rust.

**Problema:** Los intérpretes externos (Python/Pandas) introducen latencia de serialización, GIL y doble ecosistema de dependencias. Polars es rápido pero el consumo estadístico avanzado (regresiones, modelos lineales) requiere matrices densas en memoria contigua.
**Solución:** Orquestar transformaciones en Polars y convertir a `ndarray` solo en la frontera de consumo estadístico mediante Zero-Copy vía Apache Arrow.

## Comportamientos Observables

- [ ] El sistema procesa 5GB de datos en memoria sin colapsar la RAM del usuario gracias al procesamiento columnar de Polars.
- [ ] Las transformaciones (normalización, limpieza al vuelo) se ejecutan utilizando todos los núcleos del procesador central (CPU).
- [ ] Cuando se requiere un modelo estadístico (ej: OLS vía `linfa`), el sistema convierte el bloque de datos resultante (pequeño) a `ndarray` instantáneamente.

## Restricciones

- NUNCA se usa un intérprete externo para la fase de Ingesta ETL masiva.
- NUNCA se materializan DataFrames intermedios en Polars si se puede usar *Lazy Evaluation* (`scan_parquet`).
- Toda conversión a `ndarray` debe usar Apache Arrow como puente para evitar duplicación de memoria (Zero-Copy).

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
- Ejecuta limpieza estructural (duplicados, nulos, tipado datetime).
- Aplica agregaciones columnares (EJ: OHLCV a barras de 1h).
- Si el consumidor requiere matrices densas: Ejecuta `collect()` y conversión Zero-Copy a `ndarray` vía Arrow.

### Salida
- `Polars.DataFrame` (para persistencia Parquet).
- `ndarray::Array2<f64>` (para consumo estadístico y modelos `linfa`).
- Esquema unificado de tipos (float64 para precios, datetime64[ns] para tiempo).

### Contextos de Uso

**Contexto 1: Ingesta Central (Módulo Ingest)**
- Transforma millones de filas crudas en archivos Parquet optimizados.

**Contexto 2: Investigación R&D (Módulo Generate/Validate)**
- Prepara los datos para modelos de IA o pruebas de robustez estadísticas.

## Tareas (TTRs)

### **TTR-001: Pipeline de Transformación Lazy (Polars)**
- Implementa la lógica de "Clean-on-the-fly" usando Polars para saturar la CPU en transformaciones columnares masivas.

### **TTR-002: Puente de Interoperabilidad Zero-Copy**
- Implementa la conversión optimizada Polars → `ndarray` vía Apache Arrow asegurando que el tipado estricto se mantenga consistente.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local (procesamiento In-Memory / Out-of-Core).
- **Fidelidad (ADR-0017):** Alta (precisión float64).
## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Cada ejecución de transformación registra el set de relevancia técnica para Datos:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la transformación |
| | `created_at` | Timestamp de ejecución |
| | `updated_at` | Timestamp de última actualización del registro de transformación |
| | `audit_hash` | Hash del DataFrame Polars resultante |
| | `audit_chain_hash` | Hash de la integridad del pipeline ETL |
| | `event_sequence_id` | Secuencia ordinal del paso dentro del pipeline ETL |
| **II. Linaje** | `source_id` | Ref al dataset crudo (input) |
| | `transformation_id` | ID del paso de transformación (80/20) |
| | `logic_hash` | Hash de la lógica Polars/ndarray |
| **III. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del motor de transformación |
| | `execution_latency_ms" | Tiempo de procesamiento multihilo |
