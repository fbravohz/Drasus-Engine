# GANs para Universos Sintéticos (SQX Mod 2)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-06-06

---

## ¿Qué es?

Generative Adversarial Networks (GANs) entrenadas para generar microestructura de mercado sintética hiperrealista. En lugar de perturbar datos históricos (Monte Carlo clásico), el GAN genera "universos paralelos" completos con Order Flow, volatilidad regímenes y correlaciones implícitas aprendidas.

---

## Comportamientos Observables

- [ ] **Generación de Microestructura Sintética:** Producción de Order Flow sintético y matrices de liquidez que respeten correlaciones cruzadas realistas.
- [ ] **Cluster Multi-GPU Training:** Entrenamiento distribuido de 100K epochs sobre clusters de GPUs NVIDIA A100.
- [ ] **Simulación de Regímenes:** Configuración del generador para simular aumentos drásticos de volatilidad o contracción de liquidez sintética.

---

## Tareas (TTRs)

### **TTR-001: Modelado de Redes Generativas para Series Temporales**
*   **¿Cuál es el problema?** El remuestreo o perturbación clásica (Monte Carlo) no captura la dependencia temporal ni la microestructura de mercado compleja en escala fina.
*   **¿Qué tiene que pasar?** Diseñar e implementar redes generativas adversarias especializadas en series de tiempo para simular flujos de órdenes sintéticos con propiedades estadísticas equivalentes a datos reales.
*   **¿Cómo sé que está hecho?**
    - [ ] El modelo genera series temporales sintéticas donde las pruebas de distribución (Kolmogorov-Smirnov) muestran coincidencia estadística con datos reales.
*   **¿Qué no puede pasar?**
    - Las series sintéticas no deben tener precios negativos ni discontinuidades lógicamente imposibles.

---

## Gobernanza y Estándares (ADR-0020)
- Registro del **Grupo I (universal) + Perfil IA/R&D** (ADR-0020) para cada ciclo de entrenamiento e inferencia sintética.
