# Databank Manager — Catálogo de Alpha Soberano

**Carpeta:** `./features/databank-manager/`
**Estado:** Especificación / Prioritaria (P1)

> **Corrección por pruebas múltiples (ADR-0151, punto #2):** el filtro Top-X% sobre el databank **es** un punto de decisión — se corrige (PBO/CSCV si el criterio ≠ Sharpe, DSR si es Sharpe). El databank es también la fuente del pool de Sharpe de los ensayos que la política de N consume. Impacto progresivo (ADR-0137).
> 🔶 **Extendido por ADR-0153 (2026-07-12):** esta feature gana (1) el **Grid View** — superficie de tabla masiva escrolleable con columnas de métricas configurables, filtro y acciones en bloque, complementaria al Canvas nodal (que no escala visualmente a miles de filas); y (2) la **Selección** — primitivo efímero y mutable (conjunto de `version_hash` sin peso ni versionado git-like), el escalón intermedio entre "una estrategia suelta" y un Portafolio formal.
**Última actualización:** 2026-07-12

---

## ¿Qué es?

El Databank Manager es el almacén centralizado de Alpha de Drasus Engine. Utiliza **Parquet** para almacenamiento de alto rendimiento y **DuckDB** para consultas SQL ultrarrápidas. Su misión es la **Prevención de Overfitting** mediante el etiquetado de toxicidad y la clusterización por verosimilitud (UMAP/PCA).

---

## Comportamientos Observables

- [ ] **Almacenamiento Masivo:** Soporta 100K+ estrategias comprimidas en Parquet local.
- [ ] **Toxicity Score:** Calcula qué estrategias están demasiado correlacionadas con el azar (HRP based).
- [ ] **Clustering (UMAP/PCA):** Agrupa estrategias por perfil de riesgo-retorno para identificar duplicados.
- [ ] **DuckDB Partition Pruning:** Consulta 1M+ backtests en < 500ms filtrando por Sharpe o DD.

---

## Tareas (TTRs)

### **TTR-001: Motor de Almacenamiento Parquet + DuckDB Indexing**
*   **Descripción:** Implementa el esquema `strategies.parquet` con particionamiento por `symbol` y `version`.
*   **Eficiencia:** El índice DuckDB debe permitir recuperar el top 100 de Sharpe en subseguindos.
*   **Entrada:** `Strategy_Metadata_Batch`.

### **TTR-002: Cálculo de Toxicity Score (SQX Mod 3/21)**
*   **Descripción:** Mide la correlación entre las curvas de equidad de las estrategias. 
*   **Regla:** Si una estrategia tiene correlación > 0.95 con otra ya existente, es marcada como `DUPLICATE` o `TOXIC` (Redundant Alpha).

### **TTR-003: Visualización de Población (UMAP Cluster)**
*   **Descripción:** Proyecta el espacio de parámetros de 10K+ estrategias en 2D (ADR-0028).
*   **Propósito:** Identificar "islas de robustez" versus "zonas de fragilidad".

### **TTR-004: Grid View — Tabla Masiva con Columnas Configurables y Acciones en Bloque (ADR-0153)**
*   **¿Cuál es el problema?** El Canvas nodal (ADR-0136) es óptimo para navegar jerarquía y relaciones, pero no escala visualmente a miles de filas con selección y acciones masivas — el caso de uso central de operar un banco con decenas de miles de estrategias.
*   **¿Qué tiene que pasar?** Superficie de tabla escrolleable sobre el índice DuckDB (TTR-001): columnas de métricas configurables por el usuario (Sharpe, Profit Factor, número de trades, ratio Return/Drawdown, y cualquier campo de `MetricsDict`), orden y filtro por columna, selección individual y multi-selección con checkbox, acciones en bloque (crear Selección, promover a Portafolio, conectar a un puerto de nodo del Canvas, descartar — ver `canvas-navigation`).
*   **¿Cómo sé que está hecho?**
    - [ ] El usuario filtra `Sharpe > 1.5 AND MaxDD < 15%` sobre 100K+ filas y la consulta responde en el presupuesto de latencia ya fijado por TTR-001 (subsegundos).
    - [ ] El usuario selecciona N filas con checkbox y dispara una acción en bloque sin que el costo perciba diferencia entre N=1 y N=100.000 (la acción opera sobre referencias, nunca copia filas).
*   **¿Qué no puede pasar?**
    - No se materializa una copia de las filas seleccionadas en ninguna tabla nueva — la selección es una lista de referencias (ver TTR-005).

### **TTR-005: Selección — Conjunto Efímero y Mutable de Referencias (ADR-0153)**
*   **¿Cuál es el problema?** Agrupar N estrategias para un test masivo (ej. "5.000 candidatas para retestear con nuevo slippage") no debería forzar la semántica pesada de un Portafolio formal (pesos, reglas de riesgo, versionado git-like) cuando el usuario solo quiere un lote de trabajo desechable.
*   **¿Qué tiene que pasar?** Se persiste una Selección: fila mutable con lista de `version_hash` miembros, sin peso, sin reglas, sin nodo en ningún DAG. Se arma desde el Grid View (filtro o checkbox manual). Se conecta directo al puerto de entrada (`0..N` de `ExecutableContainer`, ADR-0137) de cualquier nodo del Canvas — el nodo opera sobre las referencias, nada se copia ni se mueve del banco global. Botón "Promover a Portafolio" congela la Selección como primer nodo del DAG de un Portafolio versionado (`ADR-0077`), a partir de ahí con pesos asignables.
*   **¿Cómo sé que está hecho?**
    - [ ] El usuario arma una Selección de 5.000 estrategias, la conecta a un nodo Retester, y el Retester opera sobre las 5.000 sin que ninguna desaparezca del banco global.
    - [ ] El usuario agrega/quita miembros de una Selección existente sin crear una versión nueva (mutación in-place).
    - [ ] "Promover a Portafolio" crea el nodo raíz de un DAG de Portafolio con los mismos miembros; la Selección original puede descartarse sin afectar al Portafolio ya promovido.
*   **¿Qué no puede pasar?**
    - Una Selección nunca aparece en `expedition_lineage` como `artifact_kind` — no es una entidad versionada, es conveniencia de trabajo (ADR-0153).
    - Una Selección nunca se usa como referencia permanente desde otra feature — cualquier consumo que necesite persistir una referencia estable debe promoverla a Portafolio primero.

---

## Persistencia (Inundación de Fundamentos — ADR-0020)

Cada inserción en el Databank registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la estrategia/portafolio |
| | `created_at` | Timestamp de indexación |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash del cuerpo Parquet persistido |
| | `audit_chain_hash` | Hash de la integridad del catálogo |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **II. Soberanía** | `owner_id` | Propietario del Alpha |
| | `manifest_id` | ID del contrato de diseño legal |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash de la configuración de clustering |
| | `indicator_state_hash` | Snapshot de métricas (Sharpe/DD) |
| | `version_node_id` | Versión del genoma en el catálogo |
| **IV. Hardware** | `node_id` | ID del hardware físico del databank |
| | `process_id` | PID del worker de DuckDB |

**Tabla `strategy_selections` (Perfil D — Ops/Auditoría, ADR-0153):** a diferencia del databank principal (Perfil B), la Selección es mutable y liviana — no lleva el aparato de linaje completo.

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la Selección |
| | `created_at` | Timestamp de creación |
| | `updated_at` | Timestamp de última mutación (agregar/quitar miembros) |
| **II. Soberanía** | `owner_id` | Dueño de la Selección |
| — | `member_version_hashes` | Lista JSON de `version_hash` miembros (mutable in-place) |

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `strategies_out` | `ExecutableContainer` | Output | `0..N` | Estrategias/portafolios del banco filtrados/consultados por el Grid View |
| `selection_out` | `StrategySelection` (tipo nuevo — cableado de Canvas diferido, ADR-0137 enmienda 2026-07-12) | Output | `1` | Conjunto efímero armado desde el Grid View; conectable directo a cualquier puerto `0..N` de `ExecutableContainer` |
| `family_labels_in` | `MetricsDict` | Input | `0..N` | `family_label`/`toxicity_score` de `pca-toxicity-analyzer`, mostrados como columnas del Grid View |

## Cáscara Visual (Thin Shell)

> Pendiente de la Etapa 0.5 (UI-Designer). Superficie prevista: Grid View (tabla masiva, columnas configurables, multi-selección, acciones en bloque) como superficie propia de esta feature, complementaria al nodo Canvas. El Architect NO rellena esta sección.

---

## Dependencias
- [`strategy-versioning`](../features/strategy-versioning.md) — para la integridad de los hashes.
- [`institutional-metrics`](../features/institutional-metrics.md) — para los parámetros de búsqueda.
- [`pca-toxicity-analyzer`](../features/pca-toxicity-analyzer.md) — provee `family_label`/`toxicity_score` como columnas del Grid View.

**Consumido por:**
- Cualquier nodo del Canvas con puerto `0..N` de `ExecutableContainer` (Retester/Optimizer/etc.) — vía Selección o Portafolio promovido.
- [`portfolio-optimizer`](../features/portfolio-optimizer.md) — al promover una Selección a Portafolio.
