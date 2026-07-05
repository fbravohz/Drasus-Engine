# Predictive Quant Oracles (Evolución SQX Analysis Metrics)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental, Alpha no validado)
**Última actualización:** 2026-06-06
**Origen:** Propuesta CPO "De Métricas Retrospectivas a Oráculos Predictivos". Alpha incierto (predecir ruptura futura) → incubación R&D bajo escrutinio.

---

## ¿Qué es?

Motor de inferencia bayesiana que intenta **proyectar el futuro** de una estrategia en lugar de solo medir su pasado. Dos oráculos: el **Live Fragility Index** (probabilidad de que la estrategia se rompa en los próximos N días según VIX y liquidez actual) y el **Predictive Stagnation** (probabilidad de entrar en estancamiento, con sugerencia de reducir exposición).

**Por qué es moonshot:** Predecir rupturas a futuro tiene alto riesgo de vanidad estadística y alucinación; debe validarse out-of-sample con rigor antes de cualquier uso operativo. Las métricas retrospectivas reales viven en `institutional-metrics` / `statistical-inference-ebta`.

---

## Comportamientos Observables

- [ ] El sistema estima una probabilidad de ruptura a horizonte configurable basada en el régimen actual.
- [ ] Ante alta probabilidad de estancamiento, sugiere al humano reducir la exposición de capital; el humano aprueba con un clic.

---

## Tareas (TTRs)

### **TTR-001: Motor de Inferencia Bayesiana de Fragilidad**
*   **¿Cuál es el problema?** La métrica de estabilidad clásica solo mira atrás; no anticipa el deterioro bajo el régimen vigente.
*   **¿Qué tiene que pasar?** Estimar una distribución de probabilidad de ruptura condicionada a variables de régimen actuales, con bandas de incertidumbre explícitas.
*   **¿Cómo sé que está hecho?**
    - [ ] La proyección se acompaña SIEMPRE de su intervalo de confianza y se valida out-of-sample antes de exponerse.
*   **¿Qué no puede pasar?** NUNCA presentar una predicción puntual sin su incertidumbre; NUNCA ejecutar reducción de capital sin aprobación humana.

---

## Gobernanza y Estándares (ADR-0020)
- Perfil IA / R&D: Identidad + Soberanía + Pesos/Arquitectura + Hardware. Registro del modelo bayesiano y de su validación out-of-sample por cada predicción emitida.
