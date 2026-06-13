# Quantum Portfolio Solver — Optimización Cuántica (VQE/QAOA)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-04-13

---

## ¿Qué es?

Explora el uso de algoritmos de **Computación Cuántica** para resolver problemas de optimización combinatoria complejos, como la selección de activos y la asignación de capital en universos masivos donde los solvers clásicos convergen lentamente.

---

## Comportamientos Observables

- [ ] **QUBO Mapping:** Mapeo de problemas de optimización de portafolio a problemas cuadráticos binarios sin restricciones (QUBO).
- [ ] **Quantum Annealing / VQE:** Ejecución de optimizaciones en simuladores cuánticos o hardware real (ej: AWS Braket, IBM Quantum).
- [ ] **Hybrid Optimization:** Uso de algoritmos híbridos cuántico-clásicos para refinar pesos de portafolio.

---

## Tareas (TTRs)

### **TTR-001: Modelado de Portafolio Cuántico (QUBO Formulation)**
*   **Descripción:** Formular la función objetivo de riesgo-retorno como un Hamiltoniano para optimización cuántica.

### **TTR-002: Ejecución en Simuladores Cuánticos**
*   **Descripción:** Pruebas de concepto usando Qiskit / PennyLane sobre entornos simulados.

---

## Gobernanza y Estándares (ADR-0020 V2)
- Registro del **Grupo I (universal) + Perfil IA/R&D** (ADR-0020 V2) para cada corrida cuántica.
