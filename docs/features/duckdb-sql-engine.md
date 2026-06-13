# DuckDB Analytical Engine — Motor SQL Out-of-Core

**Carpeta:** `./features/duckdb-sql-engine/`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0030 (Persistencia Soberana)

---

## ¿Qué es esta feature?

Es el motor analítico central de Drasus Engine para **Procesamiento Analítico en Línea (OLAP)**. Proporciona una interfaz para ejecutar consultas SQL vectorizadas y agregaciones masivas (ej. velas de 7 minutos vía SIMD) directamente sobre archivos **Parquet** sin necesidad de cargar todo el dataset en la memoria RAM.

**Problema que resuelve:** Analizar 50GB de datos históricos en una laptop de 16GB de RAM es imposible con herramientas tradicionales. DuckDB permite tratar los archivos Parquet como si fueran tablas de una base de datos de alta velocidad en disco.

## Comportamientos Observables

- [ ] El usuario solicita el promedio de volumen de los últimos 5 años de un activo y el sistema responde en menos de un segundo, incluso si el archivo pesa varios Gigabytes.
- [ ] Generación de temporalidades personalizadas (ej. velas de N-minutos) mediante agregación SIMD bajo demanda.
- [ ] El sistema permite unir (JOIN) datos de diferentes archivos Parquet (ej: Precios + Sentimiento) usando sintaxis SQL estándar.
- [ ] La consulta no bloquea el sistema — utiliza procesos multihilo nativos de DuckDB.

## Restricciones

- **NUNCA** intervenir en la ruta de ejecución crítica (*hot path*) del backtesting para evitar latencias prohibitivas.
- **FIJO:** El motor debe configurarse en modo "Read-Only" para análisis para evitar corrupciones accidentales de los archivos fuente de mercado.
- **Límite Técnico:** El throughput está limitado por la velocidad de lectura del disco (SSD recomendado).

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| DUCKDB_THREADS | AUTO | 1 - N | Hilos asignados al motor analítico | CONFIG |
| MEMORY_LIMIT | 4GB | 1GB - 80% RAM | Límite de memoria para caché de DuckDB | CONFIG |
| TEMP_DIRECTORY | ./tmp | N/A | Directorio para archivos temporales de queries pesadas | CONFIG |

---

## Ciclo de Vida de la Feature — DuckDB Analytical Engine

### Entrada
- Sentencia SQL (Select, Group By, Join).
- Rutas de archivos Parquet/CSV (registrados como tablas virtuales).

### Proceso
- Análisis y optimización de la consulta.
- Ejecución vectorial directamente sobre el almacenamiento (Out-of-Core).
- Conversión del resultado a formato Arrow.

### Salida
- Resultado de la consulta como Arrow Table o Polars DataFrame.
- Metadatos de ejecución (tiempo, memoria usada).

### Contextos de Uso

**Contexto 1: Generación de Alfas (MOD-02)**
- Se usa para buscar patrones históricos (ej: "Días donde el precio subió > 5% con volumen bajo").

**Contexto 2: Validación Walk-Forward (MOD-03)**
- Se usa para segmentar datasets históricos por fechas dinámicas para pruebas de robustez.

---

## Tareas (TTRs)

### **TTR-001: Gestor de Vistas Virtuales (Ruta OLAP)**
* **¿Cuál es el problema?** Registrar manualmente cada archivo Parquet como tabla en DuckDB es propenso a errores y lento para el usuario.
* **¿Qué tiene que pasar?** El motor debe abstraer la ruta física del archivo y registrar "vistas virtuales" de forma dinámica al recibir una consulta SQL, optimizando el esquema en disco.
* **¿Cómo sé que está hecho?**
    - [ ] Puedo ejecutar un `SELECT` sobre un archivo `.parquet` nuevo pasando solo su nombre lógico.
    - [ ] El sistema mantiene un caché de esquemas para acelerar consultas recurrentes.
* **¿Qué no puede pasar?** PROHIBIDO el acceso a DuckDB durante el loop de eventos del backtesting (Core hot-path).

### **TTR-002: Adaptador Zero-Copy DuckDB ↔ Polars**
* **¿Cuál es el problema?** El motor analítico devuelve tablas de DuckDB, pero la lógica de AI prefiere DataFrames de Polars. Copiar los datos entre formatos anularía la ventaja de velocidad.
* **¿Qué tiene que pasar?** Implementar un adaptador que utilice el protocolo de memoria Apache Arrow para transferir los resultados de la query DuckDB a Polars en tiempo constante (sin copia).
* **¿Cómo sé que está hecho?**
    - [ ] El tiempo de transferencia entre motores es < 1ms independientemente del volumen del resultado.
    - [ ] No se observa duplicación en el consumo de RAM durante la transferencia.
* **¿Qué no puede pasar?** NUNCA usar serialización intermedia (CSV/JSON/Parquet) para esta transferencia.

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. Los datos residen en la carpeta de usuario.
- **Inundación de Fundaciones (ADR-0020 V2): Perfil A (Datos / Ingest)**.
    - **I. Identidad & Integridad (Grupo I completo):** `id`, `created_at`, `updated_at`, `audit_hash` (del archivo consultado), `audit_chain_hash`, `event_sequence_id`.
    - **III. Linaje:** `data_snapshot_id` (del mercado, para asegurar que el análisis se hizo sobre datos inalterados).
    - **IV. Hardware:** `node_id`, `process_id`.
    - **Hooks Forenses:** Registra el `data_snapshot_id` del mercado para asegurar que el análisis se hizo sobre datos inalterados.
- **Contrato de Persistencia:** Interactúa con archivos Parquet (Historical Bars / Fact Tables).

## Dependencias y Bloqueantes
**Depende de:** `ingest` (para que existan los archivos Parquet).
**Bloquea:** `factor-decomposition`, `walk-forward-analyzer`.
