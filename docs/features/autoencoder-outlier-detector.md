# Autoencoder Outlier Detector

**Carpeta:** `./features/autoencoder-outlier-detector/`
**Estado:** En Diseño
**Última actualización:** 2026-04-30

---

## ¿Qué es?

Detector de anomalías multidimensionales en el flujo de transacciones de una estrategia mediante un modelo de Autoencoder neuronal. Evalúa si el rendimiento de una estrategia es producto de su lógica o si está distorsionado por trades extremadamente afortunados (outliers).

El modelo neural se implementa con una arquitectura de perceptrón multicapa (MLP) simple con estructura de capas `64-32-64` (64 entradas de características de trades normalizadas, 32 en el espacio latente de cuello de botella, y 64 neuronas en la capa de salida para reconstrucción). El sistema calcula el error de reconstrucción (MSE) por transacción para calificar anomalías e invalidar o penalizar estrategias sobreajustadas.

---

## Comportamientos Observables

- [ ] El sistema extrae características clave por cada transacción de la estrategia (PnL, MAE, MFE, duración, hora, día).
- [ ] El modelo neuronal comprime estas características en un espacio latente de baja dimensión (32 neuronas en cuello de botella) y luego las reconstruye (estructura 64-32-64).
- [ ] Se calcula el error de reconstrucción por transacción mediante la función de pérdida MSE.
- [ ] Si el error supera el percentil configurable, la transacción se marca como outlier.
- [ ] Se re-calculan las métricas estadísticas originales excluyendo los trades marcados como outliers.

---

## Restricciones

- **NUNCA omitir el cálculo del error de reconstrucción** para ninguna transacción del dataset.
- **NUNCA procesar datasets de transacciones con tamaño inferior** al mínimo requerido para entrenar el modelo.
- **El percentil de corte es configurable** pero debe estar en el rango de 80 a 99.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| OUTLIER_PERCENTILE | 95.0 | 80.0-99.0 | Percentil para declarar un trade como outlier | CONFIG |
| MIN_TRADES_REQUIRED | 50 | 20-500 | Mínimo de trades para entrenar el modelo | CONFIG |
| HIDDEN_DIMENSION | 3 | 2-5 | Dimensión del espacio latente de compresión | CONFIG |
| OUTLIER_FITNESS_PENALTY | 0.3 | 0.1-1.0 | Factor de penalización si el impacto de outliers excede el umbral | CONFIG |

---

## Ciclo de Vida de la Feature — Autoencoder Outlier Detector

### Entrada
- Dataset de transacciones históricas de la estrategia (PnL, MAE, MFE, duración, hora, día).
- Parámetros de umbrales configurables.

### Proceso
- Normaliza las características de las transacciones.
- Entrena el Autoencoder para aprender el perfil de trades "normales".
- Evalúa el error de reconstrucción de cada trade.
- Aplica el percentil de corte para marcar anomalías.
- Re-calcula métricas sin outliers.

### Salida
- Dataset de transacciones actualizado con el error de reconstrucción y el indicador de anomalía por cada trade.
- Reporte analítico con la comparación de las métricas originales frente a las ajustadas sin outliers.

### Contextos de Uso

**Contexto Único: Validación de Robustez (Módulo Validate)**
- Actúa dentro de la cascada de intensidad para filtrar candidatos genéticos espurios que dependan de trades de suerte.

---

## Tareas (TTRs)

### TTR-001: Modelado de Reconstrucción Neuronal de Trades (Autoencoder MLP 64-32-64)

**Qué hace:** Procesa el conjunto de transacciones, extrae las dimensiones requeridas (normalizando características como PnL, MAE, MFE, duración, hora y día a un vector de 64 elementos), las comprime en el espacio latente (32 dimensiones) y reconstruye la salida de vuelta a 64 dimensiones utilizando un modelo MLP autoencoder en Rust (vía bindings nativos o librerías de tensores).

**Entrada:**
- Serie de trades con características: PnL, MAE, MFE, duración, hora y día.

**Salida:**
- Pesos del autoencoder entrenado e inyecciones de error de reconstrucción (MSE).

**Restricciones:**
- Todas las dimensiones de entrada deben ser numéricas, estar normalizadas (MinMax o Standard scaling) y libres de valores nulos.
- Estructura de capas fija en `64-32-64`.

---

### TTR-002: Scoring y Clasificación de Anomalías por Percentil (Outlier Scorer & Reconstruction Evaluator)

**Qué hace:** Aplica el cálculo del error de reconstrucción cuadrático medio (MSE) para cada trade frente a su reconstrucción (Outlier Scorer). Luego, el Evaluador de Reconstrucción ordena las transacciones por error y aplica el percentil de corte (por ejemplo, percentil 95) para marcar los trades anómalos (outliers).

**Entrada:**
- Errores de reconstrucción (MSE) por trade.
- Percentil de corte configurable (`OUTLIER_PERCENTILE`).

**Salida:**
- Indicador booleano por trade indicando si es o no outlier.

**Restricciones:**
- El percentil de corte debe validarse antes de su aplicación.
- El cálculo de error debe estar desacoplado y optimizado para ejecutarse en la cascada de validación.

---

### TTR-003: Ajuste de Métricas y Penalización de Fitness

**Qué hace:** Re-calcula el score de fitness y métricas como el Sharpe Ratio de la estrategia tras excluir los trades outliers.

**Entrada:**
- Dataset original.
- Indicador de transacciones marcadas como outliers.

**Salida:**
- Sharpe Ratio y métricas ajustadas.
- Score de fitness penalizado si el impacto de los outliers es >50% del PnL total.

**Restricciones:**
- Si la penalización se aplica, el score original debe multiplicarse por el factor de penalización configurado.

---

## Dependencias

**Depende de:**
- `institutional-metrics` (para re-calcular el Sharpe Ratio).

**Depende de ella:**
- `validate` (ejecuta la validación durante la cascada de intensidad).

---

## Gobernanza y Estándares

- **Inundación de Fundaciones (ADR-0020):**

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del modelo y análisis |
| | `created_at` | Timestamp de ejecución |
| | `updated_at` | Última actualización del análisis |
| | `audit_hash` | Hash del reporte de outliers generado |
| | `audit_chain_hash` | Hash de la integridad de los trades |
| | `event_sequence_id` | Secuencia del evento de análisis |
| **II. Soberanía** | `owner_id` | Dueño de la IP evaluada |
| | `manifest_id` | ID del contrato de diseño legal |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del modelo de Autoencoder |
| | `data_snapshot_id` | Contexto de mercado del evento |
| | `indicator_state_hash` | Score de toxicidad/outliers |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del validador de anomalías |
