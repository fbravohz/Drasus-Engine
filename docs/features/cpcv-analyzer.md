# CPCV Analyzer (Combinatorial Purged Cross-Validation)

**Carpeta:** `./features/cpcv-analyzer/`
**Estado:** Lista para implementar
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0063 (Protocolo CPCV y Validación PBO)

## ¿Qué es esta feature?

El **CPCV Analyzer** es el motor de validación cruzada de grado institucional del sistema. Su función es particionar los datos históricos en miles de combinaciones de "caminos" no lineales para evaluar si el rendimiento de una estrategia es consistente o producto de la suerte estadística.

A diferencia de la validación cruzada tradicional, esta feature aplica técnicas de **Purging** (limpieza de trades solapados) y **Embargo** (eliminación de correlación serial) para garantizar que los datos de entrenamiento y prueba sean verdaderamente independientes. Finalmente, calcula el **PBO (Probability of Backtest Overfitting)** para determinar si la estrategia ha sido sobreajustada.

## Comportamientos Observables

- [ ] El sistema particiona la historia en $N$ bloques y genera $\binom{N}{k}$ combinaciones, reensamblando fragmentos OOS en **Caminos (Paths)** completos para un cálculo de métricas consistente.
- [ ] **Purga Dinámica:** El sistema detecta automáticamente la duración máxima de los trades de la estrategia y ajusta la purga para cubrir el 110% de ese lapso, eliminando el *Data Leakage*.
- [ ] El sistema añade un margen de seguridad temporal (Embargo) tras cada set de prueba para neutralizar la correlación serial.
- [ ] **Voz Cantante (Rank Degradation):** El reporte de PBO se centra en la estabilidad del ranking; si la "Estrategia Ganadora" en entrenamiento colapsa en los caminos OOS, el sistema emite una alerta roja.
- [ ] El usuario recibe un reporte con el **PBO**, donde un valor de 0.05 indica alta confianza y 0.50 indica una "estrategia de casino" (sobreajustada).
- [ ] Si el PBO supera el umbral configurado, la estrategia se marca automáticamente como **RECHAZADA por Sobreajuste**.

## Restricciones

- **NUNCA** realizar CPCV sin aplicar Purging si hay trades abiertos que cruzan la frontera de los bloques.
- **PROHIBIDO** usar el set de prueba para ajustar parámetros (debe ser estrictamente "fuera de la muestra").
- **Límite de Cómputo:** El número de combinaciones debe estar acotado para evitar bloqueos del sistema (paralelización mandatoria).

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| CPCV_BLOCKS | 10 | 5 - 20 | Número de particiones de la historia | CONFIG |
| CPCV_TEST_GROUPS | 2 | 1 - 5 | Cuántos bloques se usan para prueba en cada combinación | CONFIG |
| DYNAMIC_PURGE_MARGIN | 1.1 | 1.0 - 2.0 | Multiplicador sobre el Max_Trade_Duration para la purga | [FIJO] |
| EMBARGO_PERCENT | 0.01 | 0.0 - 0.05 | Porcentaje de la serie a eliminar tras el test (correlación serial) | CONFIG |
| PBO_THRESHOLD | 0.10 | 0.05 - 0.50 | Límite de probabilidad de sobreajuste permitido | CONFIG |

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Algoritmo combinatorial de particionamiento, lógica de Purging/Embargo basada en índices, cálculo de PBO (logit-based distribution).
- **Shell (Infraestructura):** Orquestador de tareas paralelas (Rust Rayon/ProcessPool), persistencia de los miles de resultados de caminos en archivos Parquet efímeros.
- **Frontera Pública:** Recibe una Estrategia + Datos Históricos; produce el Score de PBO y el Veredicto de Robustez.

## Ciclo de Vida de la Feature

### Entrada
- Datos históricos validados (OHLCV).
- Estrategia con parámetros fijos.
- Configuración de bloques y umbrales.

### Proceso
1. Genera todas las combinaciones de bloques para entrenamiento y prueba.
2. Aplica Purging y Embargo para aislar los sets.
3. Ejecuta el backtest en cada camino generado.
4. Calcula la distribución de rangos de rendimiento entre caminos.

### Salida
- Score PBO (0.0 - 1.0) basado en Rank Degradation.
- **Varianza Inter-Camino:** Dispersión (StdDev) de métricas críticas (Sharpe, MDD, Ulcer Index) entre los paths sintéticos.
- Matriz de resultados por camino (para visualización de "Spaghetti Chart" y detección de fragilidad).
- Veredicto: APROBADA / RECHAZADA.

### Contextos de Uso
**Contexto: Módulo Validate**
- Uso principal para otorgar el "Sello de Robustez Institucional" a una estrategia candidata.

## Tareas (TTRs)

### **TTR-001: Generación de Caminos y Reensamblaje OOS**
El sistema debe calcular las combinaciones de bloques y **reensamblar los fragmentos OOS en un Stream de Retornos Contiguo (Virtual Time)**. El cálculo de métricas de riesgo (Maximum Drawdown, Ulcer Index) se realizará sobre la Curva de Equidad Sintética resultante del reensamblaje, garantizando que el "salto temporal" entre bloques no genere anomalías de cálculo gracias al aislamiento estricto por Purga y Embargo.

### **TTR-002: Implementación de Purga Dinámica y Embargo**
Lógica para "ciegar" barras de datos basándose en la duración real de los trades de la estrategia evaluada. Debe garantizar que ningún trade del set de entrenamiento tenga acceso a precios del set de prueba.

### **TTR-003: Cálculo de PBO via Rank Degradation**
Implementación del algoritmo de Lopez de Prado:
1. Recibe la matriz de rendimientos de todas las variaciones de la estrategia (Trial Matrix).
2. Identifica la "Mejor Variante" en el set In-Sample (Entrenamiento).
3. Mide su posición relativa (Ranking) en los Caminos OOS (Prueba) correspondientes.
4. Calcula el PBO como la frecuencia con la que la mejor variante IS cae por debajo de la mediana en el OOS.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. No se envían datos de optimización ni resultados a nubes externas.
- **Inundación de Fundaciones (ADR-0020 V2):** 
Esta feature registra el set de relevancia técnica para **AI / R&D**:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | ID único del job de análisis CPCV |
| | `created_at` | Timestamp de ejecución |
| | `audit_hash` | Hash de integridad del reporte de PBO |
| | `audit_chain_hash` | Hash vinculado al dataset de optimización original |
| **II. Soberanía** | `owner_id` | Identificador del analista/estación responsable |
| | `logic_hash` | Hash de la versión del algoritmo CPCV |
| | `manifest_id` | Ref al contrato de diseño de la estrategia |
| **III. Pesos/IA** | `pbo_score` | Probabilidad de sobreajuste calculada |
| | `paths_count` | Número total de caminos (Paths) generados |
| | `rank_stability_score` | Métrica de degradación de ranking IS/OOS |
| | `block_partition_id` | Identificador de la semilla de particionamiento |
| **IV. Hardware** | `node_id` | ID del hardware físico (CPU/GPU) |
| | `process_id` | PID del worker de Rust Rayon |
| | `execution_time_ms` | Latencia total de la validación |

- **Rastro de Evidencia:** Emite el PBO, la **Varianza Inter-Camino** (indicador de fragilidad estructural) y la matriz de equidad sintética al módulo de `feedback` y al `robustness-verdict-engine`.
