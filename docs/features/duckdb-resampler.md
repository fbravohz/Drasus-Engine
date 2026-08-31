# DuckDB Resampler

**Carpeta:** `./features/duckdb-resampler/`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0036 (Remuestreo Dinámico Multidimensional)

## ¿Qué es?

Es el motor analítico que permite crear temporalidades personalizadas (ej. 7m, 21m, 1h 34m) a partir de datos base de alta frecuencia (1m o Ticks) sin necesidad de guardar archivos físicos adicionales. Utiliza DuckDB para realizar agregaciones SQL vectorizadas a velocidades extremas.

**Problema:** Guardar archivos separados para cada temporalidad (1m, 5m, 15m, 1h) duplica o triplica el uso de disco.
**Solución:** Guardar solo 1m y generar cualquier temporalidad superior en tiempo real mediante SQL.

---

## Narrativa de Producto

**Capacidad:** `CAP-01` — Datos en los que se puede confiar

**Historia de usuario:** Como investigador de estrategias, quiero probar cualquier temporalidad que se me ocurra sin descargarla ni almacenarla antes, para que explorar una idea nueva no dependa de haber previsto que la iba a necesitar.

**El problema (técnico):** guardar un archivo físico por cada temporalidad multiplica el disco y, peor, **fija de antemano qué se puede investigar**. Si solo guardaste cinco minutos, quince minutos y una hora, esas son las únicas preguntas que puedes hacerte. Explorar una periodicidad no estándar exige un ciclo de descarga y almacenamiento que rompe el ritmo de la investigación — y en la práctica significa que nunca se explora.

**La solución (técnica):** guardar únicamente la granularidad base y **derivar cualquier temporalidad superior en el momento de la consulta**, con agregación vectorizada. El disco no crece, la coherencia está garantizada por construcción —todas las vistas salen de la misma fuente— y el espacio de exploración deja de estar limitado por lo que alguien decidió guardar meses atrás.

**Explicado sin tecnicismos:** puedes guardar en el refrigerador el jugo ya exprimido en cinco tamaños de vaso, o guardar las naranjas y exprimir el vaso que quieras cuando lo quieras.

Guardar los vasos ocupa cinco veces más espacio, y solo puedes tomar los tamaños que alguien decidió preparar. Si un día se te ocurre uno distinto, no lo tienes.

Drasus guarda las naranjas. Guarda el dato más fino, y arma en el momento la temporalidad que le pidas: siete minutos, veintiuno, una hora y treinta y cuatro. Ocupa menos, y sobre todo **no te limita las preguntas que puedes hacerte** a las que alguien previó hace meses.

## Comportamientos Observables

- [ ] El usuario solicita un gráfico de 17 minutos; el sistema lanza una consulta DuckDB sobre el Parquet de 1m y devuelve las velas en milisegundos.
- [ ] Soporta periodicidades arbitrarias, no solo múltiplos estándar de brokers.
- [ ] Cumple la **Regla de Múltiplos**: El sistema rechaza automáticamente solicitudes de menor granularidad de la disponible (ej. pedir ticks desde una fuente de 1m).

## Restricciones

- NUNCA se intenta remuestrear hacia abajo (fuente > target).
- NUNCA se materializan resultados de remuestreo en disco a menos que el usuario lo solicite explícitamente para exportación.
- El timestamp de las velas generadas debe ser consistente con la convención de NautilusTrader.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| DUCKDB_MEMORY_LIMIT | 4GB | 1GB - 32GB | RAM máxima que puede usar DuckDB para agregaciones | CONFIG |
| DEFAULT_SOURCE_TF | 1m | - | Temporalidad base preferida para el remuestreo | [FIJO] |
| CACHE_QUERY_RESULTS | True | True/False | Cachear resultados de remuestreos frecuentes | CONFIG |

## Ciclo de Vida de la Feature — DuckDB Resampler

### Entrada
- Path al archivo Parquet de origen (Fuente 1m/Ticks).
- Temporalidad objetivo (ej. 21m).
- Rango temporal solicitado.

### Proceso
- Construye la consulta SQL dinámica con `date_trunc` y agregaciones OHLCV.
- Ejecuta la consulta directamente sobre el archivo en disco (Outside-of-Memory).
- Convierte el resultado a un stream de Apache Arrow.

### Salida
- Serie temporal de velas OHLCV remuestreadas.
- Metadatos del proceso (filas procesadas, tiempo de ejecución).

### Contextos de Uso

**Contexto 1: Visualización (UI)**
- Generación de gráficos en cualquier temporalidad para exploración de alfas.

**Contexto 2: Backtesting Directo**
- Alimenta al motor de simulación con barras sintéticas creadas al vuelo.

## Tareas (TTRs)

### **TTR-001: Motor SQL de Agregación OHLCV**
- Implementa la generación de consultas dinámicas para DuckDB que aseguren la integridad de Open, High, Low, Close y Volume.

### **TTR-002: Validador de Jerarquía Temporal**
- Desarrolla el controlador que asegura que el origen tiene suficiente granularidad para el destino solicitado.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local.
## Persistencia (Inundación de Fundamentos — ADR-0020)

Cada remuestreo registra el set de relevancia técnica para Datos:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del remuestreo |
| | `created_at` | Timestamp de ejecución SQL |
| | `updated_at` | Timestamp de última actualización del registro |
| | `audit_hash` | Hash del resultado Arrow |
| | `audit_chain_hash` | Hash del linaje de transformación |
| | `event_sequence_id` | Orden secuencial del remuestreo |
| **III. Linaje** | `data_snapshot_id` | Ref al archivo Parquet fuente |
| | `transformation_id` | ID de la temporalidad generada |
| | `logic_hash` | Hash de la consulta SQL/DuckDB |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del motor DuckDB |
| | `execution_latency_ms` | Tiempo de ejecución de la consulta |
