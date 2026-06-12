# Graph Neural Networks para Contagio Macro (SQX Mod 18)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-06-06

---

## ¿Qué es?

Modelado de mercados financieros como grafos dinámicos utilizando Graph Neural Networks (GNNs). Los nodos representan activos o sectores y las aristas cuantifican sus correlaciones y dependencias dinámicas. Permite predecir la propagación de shocks macroeconómicos a través de la red global de activos.

---

## Comportamientos Observables

- [ ] **Mapeo de Grafo de Correlaciones:** Representación visual y matemática de la red de activos financieros y sus fuerzas de interconexión.
- [ ] **Simulación de Propagación de Shock:** Introducción de perturbaciones en nodos macro (ej. tasa de interés de la Fed) y cálculo del vector de contagio resultante en el resto de los activos del grafo.
- [ ] **Cálculo de Centralidad Dinámica:** Identificación de activos clave que actúan como emisores primarios de riesgo o receptores de volatilidad.

---

## Tareas (TTRs)

### **TTR-001: Modelado de Redes en Framework DL Nativo Rust (Candle/Burn)**
*   **¿Cuál es el problema?** Las matrices tradicionales de correlación no capturan la naturaleza no lineal ni la dirección de la propagación del riesgo en redes de activos grandes.
*   **¿Qué tiene que pasar?** Diseñar un pipeline basado en un framework DL nativo Rust (candle/burn, según Gate G2 del ROADMAP) con soporte para Graph Neural Networks que estructure los activos como nodos y las relaciones como aristas dirigidas ponderadas para entrenar modelos de propagación de shocks.
*   **¿Cómo sé que está hecho?**
    - [ ] El modelo genera predicciones de contagio de volatilidad con menor error cuadrático medio que los modelos multivariados tradicionales VAR.
*   **¿Qué no puede pasar?**
    - La inferencia no debe tardar más de 100ms una vez actualizado el grafo base.

---

## Gobernanza y Estándares (ADR-0020 V2)
- Registro de **25 campos mandatorios** (Perfil IA / R&D) para cada inferencia sobre la estructura del grafo macro.
