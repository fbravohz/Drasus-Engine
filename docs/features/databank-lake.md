# El Databank — Data Lake de R&D

**Carpeta:** `./features/databank-lake/`
**Estado:** Lista para implementar
**Última actualización:** 2026-04-28
**Decisión Arquitectónica Asociada:** ADR-0053 (Separación Databank R&D vs Producción)

## 1. ¿Qué es esta feature?
El Databank masivo ultra-rápido soluciona la degradación de rendimiento extremo observada al guardar el estado completo en la búsqueda evolutiva de estrategias (ej. miles de archivos masivos). Divide el universo de Investigación (R&D efímero) del universo de Producción (Persistente). 

En lugar de almacenar objetos de estrategia gigantes (JSON AST) en cada iteración, almacena únicamente "Semillas de ADN" efímeras (pares clave-valor de parámetros y métricas) en tablas columnares Parquet de altísima velocidad. Cuando el usuario decide utilizar una estrategia y la promueve, el sistema la "rehidrata" desde la semilla construyendo su forma AST completa para producción y la inyecta en la BD relacional.

## 2. Comportamientos Observables
- [ ] Finalizada una generación, el motor guarda resultados y semillas paramétricas en `strategies.parquet` sin tocar SQLite.
- [ ] Flutter renderiza gráficas complejas (coordenadas paralelas, nube UMAP) de decenas de miles de estrategias conectándose localmente a DuckDB sin latencia (Arrow Data).
- [ ] Al promover una estrategia con doble clic en la UI, el backend lee la Semilla y crea un "Snapshot de Perfil" (Pardo Profile) guardándolo en SQLite como un Agente JSON listo para operar.

## 3. Restricciones
- NUNCA se escribe el JSON AST completo en Parquet (solo la semilla paramétrica ligera).
- NUNCA se escriben resultados intermedios de R&D a la base relacional SQLite.
- Los historiales de operaciones (trades) de las pruebas masivas de R&D se particionan rígidamente por generación para preservar el "embudo de memoria".

## 4. Parámetros Configurables
| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| DATABANK_PATH | /drasus_data/workspaces | N/A | Ruta base de los Databanks efímeros | CONFIG |
| MAX_SURVIVORS | 10000 | 100-50000 | Tamaño máximo de retención por etapa de embudo | CONFIG |

## 5. Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** Empaquetado del ADN en formato de Semilla y el motor de Rehidratación que reconstruye AST a partir de semillas.
- **Shell (Infraestructura):** Lectura/escritura veloz en `strategies.parquet` y control del particionamiento en disco (Data Lake).
- **Frontera Pública:** Endpoints para consultas masivas (compilación Arrow para DuckDB) y señal de promoción para almacenamiento definitivo.

## 6. Ciclo de Vida de la Feature — Databank R&D Lake

### Entrada
- Matrices de parámetros iterados en memoria (provenientes de NSGA-II/Walk-Forward) y sus resultados estadísticos crudos calculados en RAM.

### Proceso
- Empaqueta los parámetros de la estrategia en una Semilla (JSON muy ligero).
- Agrupa métricas de calidad y Semillas en bloques columnares.
- Escribe las Semillas y las Métricas agregadas en particiones Hive-Style Parquet por cada embudo y generación.

### Salida
- Particiones inmutables de R&D (`strategies.parquet`, `trades.parquet`).
- Al ejecutarse la promoción: Estrategia rehidratada en AST dentro de SQLite.

### Contextos de Uso
**Contexto 1: Búsqueda Masiva en generate**
- Entrada: Resultados crudos de la evaluación de cientos de miles de genomas.
- Impacto: Registro analítico ultra-rápido sin sufrir de sobrecarga I/O.

**Contexto 2: Promoción a manage / incubate**
- Entrada: Clic interactivo del usuario sobre un punto de datos excelente.
- Impacto: Esa semilla singular se convierte en un portafolio de producción institucional listo para backtests confirmatorios y operación viva.

## 7. Tareas (TTRs)

### **TTR-001: Persistencia Efímera Columnar (Parquet Lake)**
* **¿Cuál es el problema?** Guardar el estado JSON de millones de estrategias revienta la memoria y el almacenamiento (caso StrategyQuant X).
* **¿Qué tiene que pasar?** Las candidatas generadas se exportan a archivos particionados usando solo métricas vitales y Semilla, divididas jerárquicamente por pipeline_id y generación.
* **¿Cómo sé que está hecho?** 
  - [ ] Herramientas como DuckDB pueden leer `/workspaces/pipeline_1/generation=1/strategies.parquet` instantáneamente.
  - [ ] No ocurre ninguna escritura en la base relacional SQLite durante el proceso masivo.
* **¿Qué no puede pasar?** No escribir JSON AST en estas etapas. No saturar la RAM persistiendo objetos complejos.

### **TTR-002: Rehidratación de Semillas a Demanda**
* **¿Cuál es el problema?** El motor en caliente requiere un Grafo AST completo para operar; la Semilla es solo para almacenamiento R&D.
* **¿Qué tiene que pasar?** Al promover, Orquestador Rust toma la Semilla inyectando sus valores numéricos en una plantilla base inmutable, reconstruyendo el árbol de dependencias (AST) y guardándolo como Agente definitivo.
* **¿Cómo sé que está hecho?**
  - [ ] El clic en la interfaz envía una señal de promoción y se observa un nuevo registro en SQLite con su Snapshot de Perfil histórico (Pardo Profile) atado.
* **¿Qué no puede pasar?** Modificar los parámetros base durante la rehidratación o perder el historial `parent_id` (link genético padre).

## 8. Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local (Soberanía Parquet + DuckDB).
- **Inundación de Fundaciones (ADR-0020 V2):** 
    - **Perfil B (IA / R&D):** data lake R&D efímero (semillas genéticas), no ruta crítica de ejecución.
    - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
    - **II. Soberanía & Propiedad:** `owner_id`, `manifest_id`.
    - **III. Pesos/Arquitectura (subset):** `logic_hash`, `data_snapshot_id`, `version_node_id`, `parent_id` (link genético padre, canónico).
    - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
- **Contrato de Persistencia (Semillas Parquet):** Grupo I completo + Perfil B arriba (`parent_id` = link genético padre), más los campos propios de negocio: `fitness_score`, `sharpe_ratio`, `max_drawdown_pct`, `dna_payload` (JSON).

## 9. Decisión Arquitectónica Asociada
- ADR-0053: Separación Databank R&D vs Producción (Semillas vs AST).

## 10. Regla de Soberanía Técnica
Los Módulos no leen Parquet directamente. Orquestan las consultas requiriendo al componente el volcado en formato Arrow para el frontend.
