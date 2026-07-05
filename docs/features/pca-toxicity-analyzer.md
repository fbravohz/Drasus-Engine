# PCA Toxicity Analyzer

**Carpeta:** `./features/pca-toxicity-analyzer/`
**Estado:** Lista para implementar
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0072 (PCA Toxicity Clustering)

## ¿Qué es?

Es un módulo de validación avanzada que aplica técnicas de aprendizaje no supervisado para agrupar y purgar familias de estrategias que demuestran comportamientos tóxicos ocultos (bajo número de operaciones, dependencia excesiva de valores atípicos o sobreajustadas). El sistema reduce las métricas de riesgo relevantes a un número menor de componentes y agrupa las estrategias mediante clústeres.

## Comportamientos Observables

- [ ] El usuario selecciona un conjunto de estrategias en la interfaz (Databank).
- [ ] La interfaz muestra una visualización interactiva de clústeres en tres dimensiones con colores que indican el nivel de toxicidad.
- [ ] El usuario filtra o selecciona un clúster específico y presiona el botón "Purgar".
- [ ] El sistema elimina las estrategias pertenecientes al clúster seleccionado del databank y actualiza las métricas del portafolio.

## Restricciones

- **NUNCA** permitir la purga de estrategias sin una confirmación manual explícita en la interfaz, a menos que pertenezcan a clústeres declarados automáticamente tóxicos por umbrales configurados.
- El tiempo de cálculo total de reducción de dimensiones y agrupamiento para 10,000 estrategias no debe exceder los 15 segundos.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| N_CLUSTERS | 7 | 3-15 | Número de clústeres para el agrupamiento | CONFIG |
| PCA_COMPONENTS | 3 | 2-4 | Dimensiones de salida de la reducción | [FIJO] |
| TOXICITY_THRESHOLD | 0.6 | 0.1-0.9 | Puntuación de toxicidad mínima para purga automática | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Algoritmo de reducción dimensional y agrupamiento.
- **Shell (Infraestructura):** Consultas DuckDB/Parquet sobre el Databank.
- **Frontera Pública:** Contrato `analyze_toxicity(databank_path, n_clusters)`.

## Ciclo de Vida de la Feature

### Entrada
- Archivo o datos del databank (`strategies.parquet`).
- Selección de métricas de riesgo (probabilidad de ruina, puntuación z, total de operaciones).

### Proceso
- Lee y normaliza el subconjunto de métricas.
- Aplica reducción dimensional (PCA) y clústeres (K-Means).
- Calcula la puntuación de toxicidad por estrategia y clúster.

### Salida
- El databank enriquecido con las columnas inmutables `cluster_label` y `toxicity_score`.

### Contextos de Uso

#### Contexto 1: Validación de Estrategias
- Se utiliza como filtro ex-ante para descartar familias completas antes de evaluar su portafolio.

---

## Tareas (TTRs)

### **TTR-001: Reducción Dimensional PCA y Clústeres K-Means**
* **¿Cuál es el problema?**
  Las métricas de riesgo de las estrategias están altamente correlacionadas, lo que dificulta el análisis mediante umbrales independientes.
* **¿Qué tiene que pasar?**
  El sistema extrae las métricas del databank, las normaliza y calcula los clústeres junto con las puntuaciones de toxicidad por estrategia.
* **¿Cómo sé que está hecho?**
  - [ ] El proceso retorna el databank con las nuevas etiquetas y puntuaciones asignadas correctamente.
* **¿Qué no puede pasar?**
  - No debe alterar las métricas originales del databank.

### **TTR-002: Dashboard de Purga Visual (Scatter 3D + Tabla)**
* **¿Cuál es el problema?**
  El usuario necesita comprender la distribución espacial de los clústeres para tomar decisiones de eliminación informadas.
* **¿Qué tiene que pasar?**
  La interfaz gráfica renderiza una visualización de puntos tridimensionales que el usuario puede rotar y filtrar mediante controles visuales.
* **¿Cómo sé que está hecho?**
  - [ ] El usuario visualiza la tabla y el gráfico tridimensional y ejecuta la purga de un clúster seleccionado con un solo clic.

---

## Gobernanza y Estándares (Fijos)

### Inundación de Fundaciones (ADR-0020)
- **Perfil AI / R&D:**
  - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
  - **II. Soberanía IP & Propiedad:** `owner_id`.
  - **III. Linaje Alpha & Datos:** `logic_hash`, `data_snapshot_id`.
  - **IV. Infraestructura & Ops:** `node_id`.

### Rastro de Evidencia (Causalidad)
- Emite un registro de eventos con los IDs de las estrategias purgadas hacia el módulo de `feedback`.
