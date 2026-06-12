# Causal Inference Discovery — Descubrimiento de Relaciones Causa-Efecto

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-04-13

---

## ¿Qué es?

Este módulo busca superar el análisis de correlación tradicional mediante el descubrimiento de **Relaciones Causales**. Utiliza Grafos Acíclicos Dirigidos (DAGs) y Variables Instrumentales para identificar por qué un activo se mueve, buscando Alphas que sean robustos a cambios estructurales de mercado.

---

## Comportamientos Observables

- [ ] **Structural Discovery:** Algoritmos (ej: PC, GES) para identificar la estructura causal entre múltiples activos y factores macro.
- [ ] **Counterfactual Analysis:** "¿Qué habría pasado con el precio si el interés no hubiera subido?"
- [ ] **Robust Alpha Identification:** Selección de señales basadas en nexos causales probados estadísticamente.

---

## Tareas (TTRs)

### **TTR-001: Discovery de Grafos Causales**
*   **Descripción:** Implementación de algoritmos de descubrimiento de estructura sobre series temporales.
*   **Herramientas:** `causal-learn` / `DoWhy`.

### **TTR-002: Análisis de Intervención (What-If Causal)**
*   **Descripción:** Simulación de intervenciones en variables de entrada para medir el efecto causal en los retornos.

### **TTR-003: Disparador Metamórfico Bayesiano (LEGACY / FRANKENSTEIN)**
*   **⚠️ FLAG DE RIESGO:** Feature INNECESARIA y PELIGROSA. Rompe el principio fundamental de determinismo operativo de Drasus Engine.
*   **Descripción:** Evaluación probabilística que altera el umbral de ejecución de una señal en tiempo real (ej. exige 95% de confianza bayesiana si hay alto Drawdown). 
*   **Motivo de Cuarentena:** Reentrenar modelos predictivos en vivo para ajustar parámetros de ejecución añade una caja negra impredecible que invalida la garantía del backtest original. Se conserva puramente como concepto R&D estocástico a futuro.

---

## Gobernanza y Estándares (ADR-0020 V2)
- Cada grafo descubierto y análisis de intervención registra los **25 campos mandatorios**.
