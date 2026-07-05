# Hive Partition Manager

**Carpeta:** `./features/hive-partition-manager/`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0035 (Persistencia en Particionado Hive-Style)

## ¿Qué es?

Es el componente encargado de organizar físicamente los archivos Parquet en el disco del usuario. Utiliza la estructura de directorios **Hive-Style** para permitir que las herramientas de consulta (DuckDB, Polars) realicen **Partition Pruning**, ignorando carpetas irrelevantes y acelerando el acceso a los datos.

**Problema:** Un solo archivo Parquet con 10 años de datos es inmanejable y lento de leer si solo quieres un mes.
**Solución:** Fragmentar los datos en carpetas `year=YYYY/month=MM/` para acceso instantáneo.

## Comportamientos Observables

- [ ] Los datos se guardan en rutas predecibles: `{data_root}/market_data/exchange=binance/symbol=BTCUSDT/timeframe=1m/year=2024/...`
- [ ] El sistema utiliza configuración tipada validada en Rust (Serde) para localizar la raíz de datos; el usuario puede cambiar `DRASUS_DATA_ROOT` y todo el sistema se reubica automáticamente.
- [ ] Al realizar un backtest de un rango de fechas, los logs muestran que solo se cargan los archivos de los meses involucrados.

## Restricciones

- NUNCA se permiten rutas "hardcoded" fuera de la configuración central.
- NUNCA se mezclan datos de diferentes exchanges o símbolos en la misma partición terminal.
- El sistema debe validar que los nombres de las carpetas sigan estrictamente el formato `key=value`.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| DATA_ROOT | ~/drasus_data | - | Carpeta base para toda la persistencia | CONFIG |
| PARTITION_KEYS | [year, month] | - | Niveles de particionamiento temporal | [FIJO] |
| FILE_SIZE_THRESHOLD | 100MB | 10-500MB | Tamaño objetivo para cada fragmento Parquet | CONFIG |

## Ciclo de Vida de la Feature — Hive Partition Manager

### Entrada
- DataFrame normalizado y sanitizado.
- Metadatos del activo (Exchange, Symbol, Timeframe).

### Proceso
- Genera la ruta de destino basada en la configuración.
- Crea las carpetas necesarias si no existen.
- Ejecuta la escritura particionada de Polars (`collect().write_parquet(..., partition_by=[...])`).

### Salida
- Estructura física de directorios en el disco.
- Registro del job de escritura en la DB de auditoría.

### Contextos de Uso

**Contexto 1: Escritura Ingesta (ETL)**
- Guardado final de datos descargados masivamente.

**Contexto 2: Organización de Resultados (Research)**
- Almacenamiento de métricas de backtest también particionadas para comparativas rápidas.

## Tareas (TTRs)

### **TTR-001: Generador Dinámico de Rutas Serde**
- Implementa la clase `DataPathsConfig` y la lógica de resolución de paths inyectables en todo el sistema.

### **TTR-002: Lógica de Escritura Particionada (Hive-Pruning Ready)**
- Desarrolla el orquestador que toma un DataFrame masivo y lo fragmenta en el disco según los metadatos temporales.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local.
## Persistencia (Inundación de Fundamentos — ADR-0020)

Cada escritura particionada registra el set de relevancia técnica:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del bloque/archivo |
| | `created_at` | Timestamp de escritura |
| | `updated_at` | Timestamp de última reescritura/compactación del bloque |
| | `audit_hash` | Hash de integridad del archivo Parquet |
| | `audit_chain_hash` | Hash de la integridad del set completo |
| | `event_sequence_id` | Secuencia ordinal de escrituras del job de particionado |
| **III. Linaje** | `data_snapshot_id` | Ref al dataset origen |
| | `transformation_id` | ID del esquema Hive aplicado |
| | `logic_hash` | Hash del motor de particionado |
| **IV. Hardware** | `node_id` | ID del hardware físico (Storage Node) |
| | `process_id` | PID del proceso de escritura |
