# AI Dimensionality Suite (UMAP & Autoencoder)

**Carpeta:** `./features/ai-dimensionality-suite.md`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0031 (IA Híbrida), ADR-0032 (Hardware Soberano)

## ¿Qué es esta feature?

La **Suite de Dimensionalidad AI** agrupa algoritmos avanzados para la compresión, visualización y detección de anomalías en grandes conjuntos de datos de trading (trades, señales, perfiles de indicadores). Utiliza **UMAP** (Uniform Manifold Approximation and Projection) para proyectar datos de alta dimensión en espacios 2D/3D (ZUI Visualizer) y **Autoencoders** para filtrar el ruido y detectar estrategias o ejecuciones "tóxicas" (Outliers).

## Comportamientos Observables

- [ ] Proyección de 100,000 estrategias candidatas en el lienzo 3D mediante UMAP/t-SNE para aplastar el hiperespacio en un mapa 2D o 3D.
- [ ] Visualización de los backtests como una galaxia de puntos agrupados físicamente según similitud matemática de sus parámetros.
- [ ] Coloreado de puntos por rentabilidad (ej. verde = profit, rojo = pérdida) para discriminar visualmente clústeres robustos de puntos sobreoptimizados aislados.
- [ ] Brushing interactivo en el componente de frontend utilizando selección Plotly (`select2d`/`lasso2d`) para aislar clústeres.
- [ ] Detección automática de "clusters" de estrategias similares para evitar la redundancia en el portafolio.
- [ ] Un Autoencoder analiza cada trade ejecutado; si el "Error de Reconstrucción" es alto, la estrategia se marca como anómala (Posible cambio de régimen no detectado).
- [ ] Visualización de la "huella digital" (embedding) de una estrategia en el Strategy Inspector.

## Restricciones

- **OBLIGATORIO:** Respetar los límites de VRAM definidos en ADR-0032.
- **NUNCA** bloquear el hilo principal durante el entrenamiento de UMAP (proceso asíncrono en Job Executor).
- **FIJO:** Los embeddings calculados se persisten directamente en el schema Parquet de la estrategia (`embedding_x`, `embedding_y`) para que la caché evite cualquier recálculo redundante.
- El error de reconstrucción del Autoencoder debe estar normalizado para permitir comparativas entre diferentes activos.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| UMAP_COMPONENTS | 3 | 2 - 3 | Dimensiones finales de proyección | [FIJO] |
| AE_LATENT_DIM | 8 | 2 - 32 | Tamaño del cuello de botella del Autoencoder | CONFIG |
| ANOMALY_THRESHOLD | 2.5 | 1.0 - 5.0 | Desviaciones estándar para marcar outlier | CONFIG |

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Implementación de la arquitectura del Autoencoder en `candle` (Rust puro, CPU-first; sin libtorch — ADR-0112) y parámetros de proyección UMAP.
- **Shell (Infraestructura):** Ejecución CPU `ndarray`/Rayon por defecto; aceleración CUDA/Metal opcional vía `candle`. Sin Scikit-Learn ni dependencias Python.

## Ciclo de Vida de la Feature — AI Dimensionality

### Entrada
- Matriz de métricas/trades (Polars DataFrame).
- Labels de estrategias (ID).

### Proceso
1. **Normalization:** Escalamiento robusto de los datos.
2. **Dimension Reduction:** Ejecuta UMAP para reducir a 2D/3D.
3. **Anomaly Check:** (Autoencoder) Comprime y reconstruye para medir el error.

### Salida
- Coordenadas `(x, y, z)` para el visor visual.
- Nuevas columnas `embedding_x`, `embedding_y` persistidas en el Parquet de la estrategia.
- `AE_Error` y flag `is_outlier`.

## Tareas (TTRs)

### TTR-001: Pipeline UMAP de Alta Densidad
- **Problema:** Procesar 100K puntos puede saturar la RAM.
- **Qué tiene que pasar:** Implementar UMAP en Rust puro (`ndarray`/`candle`) con multithreading Rayon; sin Scikit-Learn ni Python (ADR-0112).
- **Límites de Hardware:** CPU-first por defecto; GPU opcional vía `candle` (~1-2 GB VRAM para 100K puntos) solo si acelera.
- **Criterio de éxito:** Proyección completada en < 30s para 100K registros.

### TTR-002: Autoencoder de Pureza Estructural
- **Problema:** El overfitting genera estrategias que "parecen" buenas pero tienen lógica ruidosa.
- **Qué tiene que pasar:** Entrenar un Autoencoder sobre los trades de estrategias aprobadas. Las nuevas candidatas pasan por el AE; si el error es alto, se sospecha de overfitting masivo.
- **Criterio de éxito:** Identificación de outliers con una precisión del 90% en datasets de prueba con ruido inyectado.

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Cada entrenamiento y proyección registra los campos necesarios para auditoría de modelos e integridad de datos:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del Job AI |
| | `created_at` | Timestamp de inicio |
| | `audit_hash` | Hash del modelo/proyección final |
| | `audit_chain_hash` | Hash de integridad de los pesos entrenados |
| **II. Soberanía** | `owner_id` | Usuario responsable del modelo |
| | `institutional_tag` | Etiqueta de cumplimiento |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash de la arquitectura (`candle`/UMAP) |
| | `data_snapshot_id` | Puntero al dataset original |
| | `indicator_state_hash` | Hash de los hiperparámetros |
| | `version_node_id` | Versión en la base de conocimiento |
| **IV. Hardware** | `node_id` | Hardware ID (Cuda Device) |
| | `process_id` | PID del worker GPU/CPU |
| | `execution_latency_ms` | Tiempo total de cómputo |

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Hardware Soberano (ADR-0032):** Respeto estricto del límite de 6GB VRAM.
