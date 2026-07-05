# Toxicity Purifier — Purga de Clústeres Tóxicos

**Carpeta:** `./features/toxicity-purifier/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0098 (Gobernanza de Purgas y Snapshots de Databank)

---

## ¿Qué es esta feature?

El Toxicity Purifier es el componente visual e interactivo encargado de la curación y purga masiva de clústeres de estrategias tóxicas identificadas por el backend de análisis de componentes principales PCA.

Provee un panel con un scatter plot PCA 3D nativo y una tabla analítica que detalla las métricas de toxicidad acumulada por clúster (`avg_toxicity`, `size`). El operador puede previsualizar el impacto de eliminar un clúster en los KPIs de la cartera y, tras una confirmación multi-paso, ejecutar una purga atómica (soft-delete). Antes de realizar cambios, el sistema genera automáticamente un snapshot en la base de datos para posibilitar rollbacks rápidos (deshacer).

---

## Comportamientos Observables

- [ ] La UI presenta un scatter plot PCA 3D nativo donde los puntos se agrupan y colorean según la etiqueta del clúster identificada por el backend.
- [ ] La tabla de clústeres colorea en rojo los clústeres marcados como "peligrosos" o altamente tóxicos (`avg_toxicity >= 0.6`) y en verde los clústeres seguros.
- [ ] Al presionar el botón "Purge" en un clúster, se abre un modal de confirmación multi-paso que muestra el impacto simulado (ej: "Se eliminarán 412 estrategias, Reducción de Sharpe de cartera en +0.15").
- [ ] El usuario confirma la operación y el sistema realiza la purga atómica en el backend Rust. Los puntos correspondientes al clúster purgado desaparecen inmediatamente de la vista.
- [ ] Un panel de historial de purgas en la UI lista las operaciones completadas con su `snapshot_id` y un botón de "Deshacer / Rollback".
- [ ] Al presionar "Rollback", las estrategias purgadas se rehidratan y reaparecen en el scatter plot sin pérdida de datos.

---

## Restricciones

- **NUNCA** permitir la destrucción física de registros sin una confirmación explícita del usuario; todas las purgas se marcan con soft-delete lógico (`is_purged=true`).
- **NUNCA** aplicar la purga en SQLite/Parquet sin haber generado previamente un snapshot de restauración inmutable.
- **Límite Técnico:** El cálculo previo del impacto del clúster y la generación del snapshot deben ejecutarse en menos de 10 segundos.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| TOXICITY_THRESHOLD | 0.6 | 0.3 - 0.9 | Umbral de toxicidad promedio para marcar un clúster en estado de alerta | CONFIG |
| MAX_SNAPSHOTS_RETAINED | 10 | 3 - 50 | Número de snapshots de purga almacenados antes de ser reciclados | CONFIG |
| ROLLBACK_WINDOW_SECS | 5.0 | 1.0 - 20.0 | Límite máximo de tiempo aceptado para procesar un rollback atómico | [FIJO] |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Gestor de estados de confirmación, simulación del impacto en los KPIs agregados del portafolio al omitir clústeres, y mapeo de snapshots.
- **Shell (Infraestructura):** API Rust para soft-delete de registros en base de datos (`purge_cluster`), orquestación de snapshots locales SQLite en formato WAL y persistencia de auditoría.
- **Frontera Pública:** Interfaz para solicitar el análisis de toxicidad (`get_toxicity_analysis`), ejecutar la purga (`purge_cluster_data`) y solicitar la reversión (`rollback_purge_data`).

---

## Ciclo de Vida de la Feature

### Entrada
- Ruta del databank local seleccionado.
- Tabla de estrategias indexada en Parquet con puntuaciones de toxicidad y etiquetas de clúster.
- Selección de clúster a purgar y confirmación de seguridad del operador.

### Proceso
- Consulta en DuckDB el tamaño y la toxicidad promedio por clúster.
- Simula la exclusión de las estrategias del clúster para proyectar el impacto en Sharpe/PnL de la cartera.
- Genera un snapshot físico del catálogo de estrategias.
- Actualiza el estado lógico (`is_purged=true`) en SQLite/Parquet.
- Firma la acción en el audit log local.

### Salida
- Estrategias eliminadas de la memoria R&D y ocultas de las consultas.
- Snapshot de restauración registrado con capacidad de rollback de un clic.

---

## Tareas (TTRs)

### **TTR-001: API de Análisis de Toxicidad y Purga (Rust Shell)**
*   **¿Cuál es el problema?** El operador necesita saber el impacto exacto en la cartera y el tamaño de un clúster de forma ágil antes de purgarlo.
*   **¿Qué tiene que pasar?** Programar en Rust un puerto analítico que use DuckDB para agrupar las estrategias por clúster, calcular la media de la métrica de toxicidad e implementar el marcado lógico `is_purged=true` por lote.
*   **¿Cómo sé que está hecho?**
    - [ ] El análisis del databank con 10,000 estrategias retorna las agrupaciones por clúster en menos de 5 segundos.

### **TTR-002: Gestor de Snapshots y Rollbacks atómicos (Rust persistence)**
*   **¿Cuál es el problema?** Si el operador purga un clúster incorrecto, la única forma de recuperarse es regenerar o volver a entrenar las estrategias (pérdida de tiempo masiva).
*   **¿Qué tiene que pasar?** Desarrollar la lógica en Rust que cree un snapshot (copia ligera o registro de transacciones delta) del catálogo en la base transaccional antes de la purga, y un método de reversión que restaure el estado anterior en menos de 5 segundos.
*   **¿Cómo sé que está hecho?**
    - [ ] Al ejecutar una purga y luego presionar deshacer, todas las estrategias del clúster vuelven a estar marcadas como activas en SQLite y visibles en las consultas.

### **TTR-003: Confirmación Multi-paso y Panel de Historial (Dart UI)**
*   **¿Cuál es el problema?** Los clics accidentales en "Purge" pueden destruir la configuración operativa del R&D.
*   **¿Qué tiene que pasar?** Diseñar un modal de confirmación interactivo en Flutter que presente al usuario los KPIs antes/después del descarte, y un panel flotante de historial que muestre los snapshots vigentes con su botón de deshacer (rollback).
*   **¿Cómo sé que está hecho?**
    - [ ] El modal de impacto previsualiza la caída/mejora de Sharpe y requiere confirmación secundaria antes de disparar la acción de FFI en Rust.

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. Los snapshots e historiales se graban de manera privada en la base SQLite.
- **Inundación de Fundaciones (ADR-0020):** Perfil Ops / Auditoría. Registra `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`, `process_id`, `node_id`.
- **Rastro de Evidencia:** Emite el rastro de la purga e impacto para su análisis en `feedback` y actualización del ledger.

---

## Dependencias

**Depende de:**
- [`pca-toxicity-analyzer`](../features/pca-toxicity-analyzer.md) — para la detección inicial de clústeres.
- [`audit-log`](../features/audit-log.md) — para el registro forense de la purga.

**Bloquea:**
- [`validate`](../modules/validate.md) — las herramientas de purga de clústeres de validación de base de datos dependen de esta feature.
