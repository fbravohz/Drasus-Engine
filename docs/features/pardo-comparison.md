# Prueba de Consistencia Pardo (Pardo Comparison)

**Carpeta:** `./features/pardo-comparison/`
**Estado:** Especificación / Prioritario
**Última actualización:** 2026-04-12

---

---

## ¿Qué es?

Componente estadístico que valida la consistencia entre dos series de resultados (ej: Backtest Histórico vs Paper Trading Vivo). Su misión es ser el **Juez de la Realidad** basado en el rastro de eventos determinista de **NautilusTrader** (ADR-0013): detecta desviaciones (Drift).

---

## Comportamientos Observables

- [ ] Compara métricas (Sharpe, DD, Profit Factor) de la sesión actual contra el baseline inmutable.
- [ ] Calcula el **Sharpe Drift** y el **Intervalo de Confianza (95%)**.
- [ ] Emite un veredicto binario: **CONSISTENTE** (dentro de límites) o **DEGRADADO** (fuera de límites).

---

## Ciclo de Vida de la Feature — Pardo Comparison

### Entrada
- Serie de trades del Baseline (Backtest Original).
- Serie de trades de la Comparativa (Incubación/Live).
- Umbrales de tolerancia configurables.

### Proceso
- Normaliza ambas series para que sean comparables en el tiempo.
- Realiza un test de hipótesis estadística (ej: t-test o permutación) sobre la diferencia de medias de retorno.
- Determina si la caída del Sharpe cae fuera del intervalo de confianza del 95%.

### Salida
- **Drift Score:** Caída porcentual de calidad.
- **P-Value de Consistencia:** Probabilidad de que el resultado sea por suerte.
- **Veredicto:** POSITIVO / NEGATIVO.

### Contextos de Uso
**Contexto 1: Promoción (Módulo Incubate)**
- Decide si una estrategia virtual pasa a operar con dinero real.
**Contexto 2: Auditoría (Módulo Feedback)**
- Analiza periódicamente si las estrategias en Live siguen siendo fieles a su diseño original.

---

---

## Tareas (TTRs)

### **TTR-001: Cálculo de Sharpe Drift y Límites de Confianza**
*   **Descripción:** Motor estadístico que mide la delta entre performance esperada (IS) y real (OOS/Live).
*   **Reglas de Negocio:**
    * Si el P-Value es < 0.05, el veredicto DEBE ser `DEGRADADO` (Drift estructural detectado).
    * El cálculo debe ignorar los primeros N días de "warm-up" de la incubación.
*   **Entrada:** `baseline_trades`, `live_trades`, `confidence_level` (default 0.95).
*   **Salida:** `drift_score` (float), `is_consistent` (bool), `p_value`.
*   **Precondición:** Ambas series de trades reconciliadas y con `audit_hash` válido.
*   **Postcondición:** Registro de la "Análisis Forense Pardo" en el rastro de evidencia causal (ADR-0015).

### **TTR-002: Reporte de Veredicto de Consistencia (Rastro del Juez)**
*   **Descripción:** Genera el veredicto inmutable para la promoción o retiro de la estrategia.
*   **Reglas de Negocio:**
    * El veredicto DEBE incluir el `version_node_id` de la estrategia evaluada (ADR-0020).
    * Si el veredicto es `DEGRADADO`, se debe sugerir un "Learning Constraint" para el módulo de `feedback`.
*   **Entrada:** `drift_score`, `p_value`.
*   **Salida:** `PardoVerdict` (APROBADO | REVISAR | RECHAZADO).
*   **Precondición:** TTR-001 finalizado exitosamente.
*   **Postcondición:** Firma digital del veredicto guardada en `strategy_versioning`.

---

## Gobernanza y Estándares (Fijos)
## Persistencia (Inundación de Fundamentos — ADR-0020)

Toda comparación Pardo registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la autopsia |
| | `created_at` | Timestamp del veredicto |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash del veredicto de consistencia |
| | `audit_chain_hash` | Hash del rastro de evidencia causal |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **II. Soberanía** | `owner_id` | Autor que solicita la validación |
| | `manifest_id` | ID del contrato de diseño legal |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash de la fórmula de comparación |
| | `data_snapshot_id` | Ref al par IS/OOS comparado |
| | `indicator_state_hash` | Drift Score actual |
| | `version_node_id` | ID de la versión evaluada en el DAG |
| **IV. Hardware** | `node_id` | ID del hardware físico (Juez) |
| | `process_id` | PID del evaluador estadístico |

- **Decisión Arquitectónica Asociada:**
    - ADR-0015: Arquitectura de Causalidad y Aprendizaje.
    - ADR-0017: Simulación de Alta Fidelidad.
    - ADR-0020: Inundación de Fundaciones.

---

## Dependencias
**Depende de:**
- [`institutional-metrics`](../features/institutional-metrics.md) — para cálculo de Sharpe y DD.

**Consumido por:**
- [`incubate`](../modules/incubate.md) — para decisión de promoción a Live.
- [`feedback`](../modules/feedback.md) — para detección de drift operativo.
