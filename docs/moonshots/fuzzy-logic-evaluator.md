# Fuzzy Logic Evaluator — Evaluación Probabilística de Señales

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 4 - Experimental)
**Última actualización:** 2026-04-13

---

## ¿Qué es?

Extraído de la Visión "New-Era" de Drasus Engine. Este módulo permite pasar de una lógica binaria (Si/No) a una evaluación de **Lógica Difusa (Fuzzy Logic)**. En lugar de disparar una señal al cruzar un umbral fijo, el sistema evalúa la "fuerza" de la señal (0.0 a 1.0) y utiliza actualización Bayesiana para ajustar la confianza.

---

## Comportamientos Observables

- [ ] **Fuzzy Membership Functions:** Define conjuntos difusos (ej: Volatilidad = {Baja, Media, Alta}) con transiciones suaves.
- [ ] **Probabilistic Signal Weighting:** Las señales de entrada (RSI, MA, etc.) devuelven un grado de veracidad.
- [ ] **Bayesian Inference Update:** Actualiza la probabilidad a posteriori de éxito de la estrategia tras cada nueva observación de mercado.

---

## Tareas (TTRs)

### **TTR-001: Motor de Inferencia Difusa (Mamdani/Sugeno)**
*   **Descripción:** Implementación del motor de reglas difusas para combinar señales no binarias.

### **TTR-002: Actualizador Bayesiano de Confianza**
*   **Descripción:** Algoritmo que ajusta el peso del "voto" de cada indicador basándose en su precisión histórica reciente.

---

## Gobernanza y Estándares (ADR-0020)
- Registro del **Grupo I (universal) + Perfil IA/R&D** (ADR-0020) para cada inferencia difusa y actualización de peso.
