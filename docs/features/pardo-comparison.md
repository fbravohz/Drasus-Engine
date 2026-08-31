# Prueba de Consistencia Pardo (Pardo Comparison)

**Carpeta:** `./features/pardo-comparison/`
**Estado:** Especificación / Prioritario
**Última actualización:** 2026-04-12

---

---

## ¿Qué es?

Componente estadístico que valida la consistencia entre dos series de resultados (ej: Backtest Histórico vs Paper Trading Vivo). Su misión es ser el **Juez de la Realidad** basado en el rastro de eventos determinista de **NautilusTrader** (ADR-0013): detecta desviaciones (Drift).

---

## Narrativa de Producto

**Capacidad:** `CAP-05` — Cuarentena antes del dinero real

**Historia de usuario:** Como investigador, quiero un juicio estadístico objetivo —no una opinión— que me diga si mi estrategia en vivo sigue siendo la misma que aprobé en el backtest, para no promoverla, ni dejarla operar, por intuición.

**El problema (técnico):** una curva de equidad que cae un poco en vivo puede ser ruido normal o el primer síntoma de que la estrategia ya dejó de ser la que fue validada. Decidir a ojo cuál de las dos cosas está pasando es exactamente el tipo de error humano que un proceso de validación disciplinado existe para eliminar.

**La solución (técnica):** contrasta la serie de resultados del backtest original contra la serie de resultados de la incubación o de la operación en vivo, calcula la caída de Sharpe y su significancia estadística, y emite un veredicto binario — consistente o degradado. Ese veredicto, no la impresión de nadie, decide si la estrategia pasa a operar con dinero real o si se retira.

**Explicado sin tecnicismos:** es un árbitro, no un comentarista. Un comentarista opina si el partido va bien o mal según cómo se ve. Un árbitro aplica una regla fija y dice "falta" o "no falta", siempre con la misma vara, se vea como se vea el partido.

Aquí la vara es una prueba matemática: compara lo que la estrategia prometió con lo que de verdad hizo, y solo puede decir dos cosas — "sigue siendo la misma" o "algo cambió" — sin dejarse impresionar por si el día se vio espectacular o mediocre.

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
*   **Postcondición:** Registro de la "Análisis de Auditoría Pardo" en el rastro de evidencia causal (ADR-0015).

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
| **I. Identidad** | `id` | Identificador único del análisis de causa raíz |
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
