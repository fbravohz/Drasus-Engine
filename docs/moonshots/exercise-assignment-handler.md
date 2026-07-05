# Exercise & Assignment Handler — Gestor de Ejercicio y Asignación de Opciones

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Post-MVP — Diferido por ADR-0140)
**Última actualización:** 2026-06-27
**Decisión Arquitectónica Asociada:** ADR-0140 (Opciones Financieras — Diferimiento al Post-MVP con Puerta Abierta)

---

## ¿Qué es?

Gestor del ciclo de vida terminal de los contratos de opciones: ejercicio voluntario (el tenedor decide ejercer), asignación (la contraparte ejerce y el sistema es asignado), y expiración (con o sin valor). Coordina la generación de la orden sobre el subyacente que se produce cuando una opción se ejerce o se asigna, y gestiona los estados terminales del Order FSM extendido.

**Por qué es moonshot:** El ejercicio y la asignación son eventos que generan órdenes sobre el subyacente de forma automática (compra/venta de acciones, futuros, etc.). Esto implica una **FSM de dos capas**: la FSM de la opción (que transita a EXERCISED, ASSIGNED o EXPIRED) y la FSM del subyacente (que recibe una nueva orden generada). Además, la asignación es un evento externo no disparado por el usuario — el sistema debe detectarlo y reaccionar sin intervención humana.

**Condición de activación (ADR-0140):** los cinco prerrequisitos del ADR-0140 deben cumplirse antes de implementar.

---

## Comportamientos Observables

- [ ] Una opción call larga ITM al vencimiento se ejerce automáticamente (o el usuario la ejerce antes del vencimiento si es americana), generando una orden de compra del subyacente.
- [ ] Una opción put corta ITM al vencimiento es asignada (la contraparte ejerce), generando una orden de compra del subyacente al precio de strike.
- [ ] Una opción OTM al vencimiento expira sin valor (EXPIRED_WORTHLESS), sin generar orden sobre el subyacente.
- [ ] El sistema detecta opciones próximas a vencer (N días antes) y alerta al usuario para que decida si cerrar, rollar o dejar vencer.
- [ ] El sistema gestiona el riesgo de asignación en opciones cortas: alerta cuando una opción corta está ITM y es probable que sea asignada.

---

## Tareas (TTRs)

### **TTR-001: FSM Extendida para Opciones**
*   **¿Cuál es el problema?** El Order FSM actual tiene 6 estados para instrumentos lineales. Las opciones requieren 4+ estados adicionales (EXERCISED, ASSIGNED, EXPIRED_WORTHLESS, EXPIRED_ITM) y una FSM de dos capas.
*   **¿Qué tiene que pasar?** Extender el FSM con los estados de opciones, definiendo las transiciones válidas y la generación de órdenes sobre el subyacente cuando la opción transita a EXERCISED o ASSIGNED. (Refactorización de [`order-fsm`](../features/order-fsm.md) documentada en ADR-0140.)
*   **¿Cómo sé que está hecho?**
    - [ ] Una opción que transita a EXERCISED genera automáticamente una orden sobre el subyacente que entra en la FSM estándar (SENT → APPROVED → FILLED).

### **TTR-002: Detección y Alerta de Vencimiento Próximo**
*   **¿Cuál es el problema?** Las opciones tienen un vencimiento fijo. Si el usuario no actúa antes del vencimiento, la opción se ejerce/asigna/expira automáticamente según las reglas del exchange.
*   **¿Qué tiene que pasar?** Detectar opciones que vencen en N días (configurable) y alertar al usuario con el estado de moneyness (ITM/OTM), el valor intrínseco y las opciones de acción (cerrar, rollar, dejar vencer).
*   **¿Cómo sé que está hecho?**
    - [ ] Una opción que vence en 3 días genera una alerta con su estado ITM/OTM y las acciones disponibles.

### **TTR-003: Gestión de Asignación (Evento Externo)**
*   **¿Cuál es el problema?** La asignación es un evento externo: la contraparte ejerce su opción y el sistema es asignado sin que el usuario lo dispare. El sistema debe detectar la asignación y generar la orden correspondiente sobre el subyacente.
*   **¿Qué tiene que pasar?** Monitorear las notificaciones de asignación del broker/exchange y generar la orden sobre el subyacente, transitando la opción a ASSIGNED en el FSM.
*   **¿Cómo sé que está hecho?**
    - [ ] Una asignación recibida del broker genera la orden sobre el subyacente y transita la opción a ASSIGNED sin intervención del usuario.

### **TTR-004: Roll de Opciones (Cerrar + Abrir Nuevo Vencimiento)**
*   **¿Cuál es el problema?** El usuario frecuentemente quiere "rollar" una posición de opción: cerrar el vencimiento actual y abrir uno nuevo (más lejano) en el mismo o diferente strike.
*   **¿Qué tiene que pasar?** Proporcionar una operación atómica de roll: cerrar la posición actual + abrir una nueva en el vencimiento seleccionado, como una unidad.
*   **¿Cómo sé que está hecho?**
    - [ ] Un roll de un call vendido de vencimiento mensual a vencimiento semanal se ejecuta como dos órdenes atómicas (cerrar + abrir).

---

## Gobernanza y Estándares (ADR-0020)
- Perfil C (Ops / Hot-Path): Identidad + Soberanía + Hardware + Latencia. El ejercicio/asignación genera órdenes en el hot path; la detección de vencimiento próximo es una operación de auditoría.

---

## Dependencias

**Depende de:**
- [`order-fsm`](../features/order-fsm.md) — para la FSM extendida (requiere refactorización post-MVP, ADR-0140).
- [`option-chain-manager`](./option-chain-manager.md) — para identificar vencimientos y strikes disponibles (para rolls).
- [`option-pricing-engine`](./option-pricing-engine.md) — para determinar moneyness y valor intrínseco.
- [`broker-connector`](../features/broker-connector.md) — para notificaciones de asignación y envío de órdenes de ejercicio.

**Consumido por:**
- [`execute`](../modules/execute.md) — para orquestación de ejercicio/asignación en vivo.
