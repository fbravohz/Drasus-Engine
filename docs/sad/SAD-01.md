## 1. Introducción y Objetivos

**Drasus Engine** es una infraestructura privada de trading algorítmico diseñada para el descubrimiento, validación y ejecución autónoma de estrategias de alto rendimiento. No es un bot convencional; es un **motor matemático determinista** (el sistema da siempre el mismo resultado numérico) acoplado a una **capa de interacción** que maneja las tareas del mundo real (lectura/escritura, manejo de errores) que interactúa con mercados reales.

### 1.1 Visión Estratégica (PRD §2)

#### El Problema (Causalidad del Proyecto)
La generación de Alpha es un problema **combinatorialmente explosivo** y plagado de sesgos:
* **Overfitting pervasivo:** Confusión entre ruido y rentabilidad genuina.
* **Ilusión estadística:** Backtests que ganan pero pierden en mercados reales.
* **Fricción invisible:** Spreads, comisiones y límites de penetración (Pardo) no modelados.
* **Regímenes cambiantes:** Estrategias robustas que se rompen ante cambios de volatilidad.

#### Propuesta de Valor
Drasus Engine es **infraestructura soberana** para el descubrimiento automático de Alpha:
1. **Descubrimiento:** Sin hipótesis humanas (NSGA-II + regresión simbólica nativa sobre el AST, ADR-0113).
2. **Validación:** Rigor institucional (WFA, Monte Carlo, CPCV).
3. **Ejecución:** Protección multinivel (10 risk steps, kill switch < 5s).
4. **Aprendizaje:** Cierre de ciclo causal (Regime Aware + Feedback).

### 1.2 Métricas de Éxito (KPIs Instrumental)

| Métrica | Target MVP | Razón Técnica |
|---|---|---|
| **Throughput Backtest** | Medible y demostrablemente más rápido que MT5, SQX y QuantConnect en la misma máquina (ADR-0114; sin KPI absoluto) | Posibilita exploración masiva. |
| **Live Order Latency** | ≤100ms (end-to-end) | Ejecución competitiva. |
| **PBO (Backtest Overfitting)** | ≤0.10 (Configurable) | Garantiza que el Alpha no es suerte. |
| **Reproducibilidad** | 100% bit-a-bit | Auditoría forense y científica. |

---

