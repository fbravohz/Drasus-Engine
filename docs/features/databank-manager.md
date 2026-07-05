# Databank Manager — Catálogo de Alpha Soberano

**Carpeta:** `./features/databank-manager/`
**Estado:** Especificación / Prioritaria (P1)
**Última actualización:** 2026-04-13

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

---

## Dependencias
- [`strategy-versioning`](../features/strategy-versioning.md) — para la integridad de los hashes.
- [`institutional-metrics`](../features/institutional-metrics.md) — para los parámetros de búsqueda.
