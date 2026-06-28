# Option Strategy Builder — Constructor de Estrategias Multi-Pata de Opciones

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Post-MVP — Diferido por ADR-0140)
**Última actualización:** 2026-06-27
**Decisión Arquitectónica Asociada:** ADR-0140 (Opciones Financieras — Diferimiento al Post-MVP con Puerta Abierta)

---

## ¿Qué es?

Constructor visual y programático de estrategias de opciones multi-pata (spreads, straddles, strangles, iron condors, butterflies, calendar spreads, etc.). Permite al usuario componer 2-6 patas (legs) como una unidad atómica, calcular el payoff diagram, el costo neto, el margen requerido y las griegas agregadas antes de ejecutar.

**Por qué es moonshot:** Las estrategias multi-pata requieren ejecución atómica: si una pata no se llena, toda la estrategia se deshace (all-or-nothing). Esto exige soporte de combo orders en el broker (IBKR lo soporta nativamente, otros no) y un modelo de ejecución que gestione fills parciales de patas. Además, el P&L es no-lineal y depende de la interacción entre strikes, vencimientos y el precio del subyacente al vencimiento.

**Condición de activación (ADR-0140):** los cinco prerrequisitos del ADR-0140 deben cumplirse antes de implementar.

---

## Comportamientos Observables

- [ ] El usuario compone una estrategia de 4 patas (Iron Condor: sell put OTM + buy put más OTM + sell call OTM + buy call más OTM) y el sistema calcula el payoff diagram, el crédito neto, el margen requerido y las griegas agregadas.
- [ ] El usuario ejecuta la estrategia y el sistema envía las 4 patas como una unidad atómica: si una pata falla, las demás se cancelan.
- [ ] El sistema muestra el perfil de riesgo/recompensa visual (payoff al vencimiento) antes de ejecutar.
- [ ] El usuario puede cerrar una pata individual o toda la estrategia como unidad.
- [ ] El sistema calcula el punto de equilibrio (breakeven) al vencimiento para cualquier combinación de patas.

---

## Tareas (TTRs)

### **TTR-001: Catálogo de Estrategias de Opciones**
*   **¿Cuál es el problema?** Existen decenas de estrategias de opciones estandarizadas (vertical spreads, iron condors, butterflies, etc.), cada una con su propia estructura de patas y perfil de riesgo.
*   **¿Qué tiene que pasar?** Definir un catálogo de estrategias con su estructura de patas (dirección, tipo, distancia al strike relativo al precio del subyacente) y sus propiedades (riesgo máximo, beneficio máximo, breakeven).
*   **¿Cómo sé que está hecho?**
    - [ ] El catálogo incluye al menos las 12 estrategias más comunes y el usuario puede seleccionar una para instanciarla sobre un subyacente concreto.

### **TTR-002: Ejecución Atómica de Multi-Pata (Combo Orders)**
*   **¿Cuál es el problema?** Si las patas se ejecutan como órdenes independientes, el riesgo de fills parciales (una pata se llena, otra no) expone al usuario a un riesgo direccional no deseado.
*   **¿Qué tiene que pasar?** Enviar las patas como una combo order si el broker lo soporta (IBKR, CBOE), o como órdenes individuales con cancelación atómica (si una falla, todas se cancelan) si el broker no soporta combos.
*   **¿Cómo sé que está hecho?**
    - [ ] Una estrategia de 4 patas se ejecuta o se cancela en su totalidad; nunca quedan patas sueltas sin cancelar.

### **TTR-003: Payoff Diagram y Métricas de Riesgo**
*   **¿Cuál es el problema?** El usuario necesita ver el perfil de riesgo/recompensa antes de ejecutar, incluyendo el payoff al vencimiento, las griegas agregadas y el margen requerido.
*   **¿Qué tiene que pasar?** Calcular y renderizar el payoff diagram para cualquier combinación de patas, con las métricas de riesgo (max loss, max gain, breakeven, delta/gamma/theta/vega agregados).
*   **¿Cómo sé que está hecho?**
    - [ ] El payoff diagram de un Iron Condor muestra correctamente las zonas de beneficio (entre los strikes vendidos) y las zonas de pérdida (más allá de los strikes comprados).

---

## Gobernanza y Estándares (ADR-0020 V2)
- Perfil C (Ops / Hot-Path): Identidad + Soberanía + Hardware + Latencia. La ejecución atómica de multi-pata opera en el hot path de ejecución.

---

## Dependencias

**Depende de:**
- [`option-pricing-engine`](./option-pricing-engine.md) — para pricing y griegas de cada pata.
- [`option-chain-manager`](./option-chain-manager.md) — para seleccionar contratos de la cadena.
- [`broker-connector`](../features/broker-connector.md) — para envío de combo orders (requiere refactorización post-MVP, ADR-0140).
- [`order-fsm`](../features/order-fsm.md) — para gestión de estados de órdenes multi-pata (requiere refactorización post-MVP, ADR-0140).

**Bloquea:**
- [`exercise-assignment-handler`](./exercise-assignment-handler.md) — necesita saber qué estrategias multi-pata están abiertas.
