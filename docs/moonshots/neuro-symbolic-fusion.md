# Fusión Neuro-Simbólica de Estrategias (El Colisionador) — SQX Mod 26

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-06-06

---

## ¿Qué es?

Hibridación de dos o más estrategias maestras mediante **extracción neuro-simbólica de características**: redes neuronales extraen la "intención" matemática de cada estrategia, mientras se preserva intacta su arquitectura lógica simbólica. En lugar de mutar a ciegas (el método "Improve Existing Strategy" de StrategyQuant X, que a menudo destruye la premisa original), el motor busca los huecos de eficiencia entre varias estrategias y los fusiona en una aleación coherente.

**Problema que resuelve:** Mejorar una estrategia metiéndola de nuevo al algoritmo genético y mutando sus partes destruye la premisa que el quant diseñó a propósito. La fusión neuro-simbólica combina la fortaleza de cada estrategia (ej. una buena en mercado lateral + otra buena en mercado direccional) sin destruir la lógica de ninguna.

---

## Comportamientos Observables

- [ ] **Extracción de Intención:** El motor descompone cada estrategia en su componente lógico simbólico y su intención matemática.
- [ ] **Fusión Controlada (El Colisionador, UI espacial):** El usuario arrastra dos estrategias hacia un nodo central, regula la "Energía de Fusión" con un deslizador y dispara la colisión. El sistema imprime una estrategia híbrida nueva.
- [ ] **Auditoría del Híbrido:** El usuario revisa el árbol de nodos resultante y su simetría antes de aprobar el nacimiento de la estrategia fusionada (Human-in-the-loop, ninguna fusión se aprueba sola).

---

## Tareas (TTRs)

### **TTR-001: Extracción neuro-simbólica y motor de fusión**
*   **¿Cuál es el problema?** La mejora por mutación ciega rompe la premisa original de las estrategias maestras.
*   **¿Qué tiene que pasar?** Extraer la intención matemática de varias estrategias preservando su lógica simbólica, y combinarlas buscando los huecos de eficiencia entre ellas.
*   **¿Cómo sé que está hecho?**
    - [ ] El híbrido resultante conserva premisas reconocibles de las estrategias de origen (no es ruido mutado).
    - [ ] El híbrido se valida contra el guantelete de robustez antes de promoverse.
*   **¿Qué no puede pasar?**
    - Ninguna estrategia fusionada se promueve a incubación sin aprobación humana explícita y sin pasar la validación de robustez existente.

---

## Gobernanza y Estándares (ADR-0020)
- Registro del **Grupo I (universal) + Perfil IA/R&D** (ADR-0020) por cada evento de fusión, incluyendo el linaje de las estrategias de origen.
- **Relegación:** R&D experimental. Se relaciona con la síntesis multi-canal existente (`strategy-ensemble`) y el hub de meta-aprendizaje (`meta-learning-hub`); permanece en `/moonshots/` hasta demostrar alpha medible de la fusión.
