# Autómatas Celulares para Crecimiento Lógico Procedural (SQX Mod 19)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental / No validado)
**Última actualización:** 2026-06-06

---

## ¿Qué es?

Generación de ramificaciones lógicas de estrategia mediante la matemática de los **Autómatas Celulares** (familia del Juego de la Vida de Conway). A partir de una regla base sólida, el motor "cultiva" proceduralmente estructuras lógicas anexas que se adaptan a la microestructura del mercado, en lugar de insertar indicadores al azar en huecos vacíos (el método "Random Groups" de StrategyQuant X).

**Problema que resuelve (declarado):** El crecimiento aleatorio de plantillas en SQX es ensayo y error ciego. La hipótesis es que el crecimiento procedural por autómatas genera estructuras complejas de forma orgánica y no aleatoria.

**Advertencia arquitectónica (rigor):** El valor alpha de aplicar autómatas celulares a la generación de lógica de trading es **no probado y especulativo**. No hay evidencia de que supere a los algoritmos evolutivos multi-objetivo ya implementados. La capacidad útil y concreta de este módulo — **podar** ramas lógicas que el operador rechaza por intuición macro — ya existe en producción vía `rule-ablation` y `vector-time-pruning`. Este moonshot conserva la idea para R&D sin descartarla, pero NO se promueve a feature hasta demostrar alpha medible contra el optimizador NSGA-II existente.

---

## Comportamientos Observables

- [ ] **Crecimiento Procedural:** A partir de una regla base, el motor genera ramificaciones lógicas anexas siguiendo reglas de autómata celular (no aleatorias).
- [ ] **Poda Orgánica (UI espacial):** El usuario ve la lógica crecer como un sistema de raíces; toma una herramienta de poda y corta físicamente una rama que considera inútil; el sistema re-estabiliza la estrategia base.
- [ ] **Comparativa contra baseline:** Toda rama cultivada se mide contra el resultado del optimizador evolutivo existente antes de considerarse válida.

---

## Tareas (TTRs)

### **TTR-001: Prototipo de crecimiento por autómata celular y validación de alpha**
*   **¿Cuál es el problema?** No está demostrado que el crecimiento procedural genere edge real frente a la generación evolutiva ya existente.
*   **¿Qué tiene que pasar?** Construir un prototipo que cultive ramas lógicas desde una regla base y medir si las estrategias resultantes superan a las del optimizador NSGA-II en robustez Out-of-Sample.
*   **¿Cómo sé que está hecho?**
    - [ ] Existe un experimento reproducible que compara estrategias "cultivadas" vs "evolucionadas" sobre el mismo historial.
    - [ ] El veredicto del experimento (supera / no supera al baseline) queda registrado.
*   **¿Qué no puede pasar?**
    - Promover este método a feature sin haber superado al baseline evolutivo (evita complejidad sin alpha).

---

## Gobernanza y Estándares (ADR-0020 V2)
- Registro del **Grupo I (universal) + Perfil IA/R&D** (ADR-0020 V2) por cada experimento de crecimiento.
- **Relegación:** Conservado como hipótesis de R&D no descartada. La función de poda operativa ya está cubierta por features de producción (`rule-ablation`, `vector-time-pruning`).
