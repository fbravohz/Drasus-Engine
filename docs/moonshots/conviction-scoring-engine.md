# Conviction Scoring Engine (Evolución SQX Money Management)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-06-06
**Origen:** Propuesta CPO "Asignación Dinámica por Convicción Neuronal". El consumo del score (Kelly dinámico) ya es viable en `precision-sizing-models`; el cálculo del score es R&D → incubación.

---

## ¿Qué es?

Motor que calcula un **Conviction Score (0-100)** en tiempo real para cada señal de entrada, basado en la confluencia de variables: liquidez actual, volatilidad macroeconómica y correlación entre múltiples marcos temporales. El score alimenta el sizing Kelly dinámico ya soportado por `precision-sizing-models` (que arriesga más en señales de alta convicción y menos en las débiles).

**Por qué es moonshot:** El modelo de scoring por confluencia (ML) requiere investigación y validación; la feature de sizing solo consume el score como entrada.

---

## Comportamientos Observables

- [ ] Cada señal recibe un score 0-100 antes de dimensionar la posición.
- [ ] El score se descompone en sus factores (liquidez, volatilidad, correlación multi-timeframe) para auditoría humana.

---

## Tareas (TTRs)

### **TTR-001: Modelo de Confluencia Multi-Factor**
*   **¿Cuál es el problema?** Tratar todas las señales como igualmente probables desperdicia ventaja y sobre-expone en señales débiles.
*   **¿Qué tiene que pasar?** Producir un score calibrado por señal a partir de la confluencia de factores, auditable (no caja negra opaca).
*   **¿Cómo sé que está hecho?**
    - [ ] El score se entrega con el desglose de los factores que lo componen.
    - [ ] El score está calibrado (un score alto se asocia históricamente a mayor tasa de acierto).
*   **¿Qué no puede pasar?** NUNCA entregar un score sin trazabilidad de los factores que lo originan (coherencia Glass-Box).

---

## Gobernanza y Estándares (ADR-0020 V2)
- Perfil IA / R&D: Identidad + Soberanía + Pesos/Arquitectura + Hardware. Registro del modelo y de la calibración por cada score emitido.
- **Consumido por (cuando madure):** `precision-sizing-models` (TTR-005, Kelly Dinámico por Conviction Score).
