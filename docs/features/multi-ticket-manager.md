# Multi-Ticket Manager — Gestor de Múltiples Posiciones por Estrategia

**Carpeta:** `./features/multi-ticket-manager/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-11
**Decisión Arquitectónica Asociada:** ADR-0078 (Autopilot Execution & Stealth Network Infrastructure), ADR-0108, ADR-0109

---

## ¿Qué es?

El gestor de múltiples posiciones por estrategia es un componente que rompe la limitación tradicional de SQX ("una sola operación a la vez"). Permite gestionar y rastrear múltiples tickets individuales concurrentes para la misma estrategia.

**Problema:** En muchos entornos de trading, una estrategia puede recibir señales de re-entrada o de escalamiento válidas mientras ya tiene una posición abierta. El `multi-ticket-manager` permite que nuevas señales generen tickets individuales orientados a objetos (OOP Tickets) sin interferir con las posiciones ya activas, diferenciándolas estrictamente por `signal_hash` y `timestamp`.

**Primitiva de Acción de Morfología de Salida — Split_Position(N_Fases) (ADR-0108/ADR-0109):** cuando el Genoma de Riesgo y Gestión de Posición está activo y resuelve `Split_Position(N_Fases)` para una señal, esta feature materializa cada fase como un OOP Ticket independiente derivado de la misma señal, diferenciado por `signal_hash` más un nuevo identificador de fase (`phase_id`). Cada ticket de fase puede recibir su propio nivel de SL/TP resultante de `Move_SL_To_Target`, sin interferir con los demás tickets de la misma posición lógica.

---

## Comportamientos Observables

- [ ] La estrategia ya tiene una posición activa y se cumple una nueva condición en una barra posterior
  → Se calcula el hash de la señal (`signal_hash`) y el timestamp exacto de disparo.
  → El sistema valida que NO sea la misma señal que detonó la posición previa (evita doble operación accidental).
  → Abre una nueva posición individual (nuevo ticket) con su propio rastro de auditoría.

- [ ] Cuando el Genoma de Riesgo y Gestión de Posición (ADR-0109) resuelve `Split_Position(N_Fases)` para una señal de entrada, el sistema abre N tickets independientes en la misma barra, todos derivados del mismo `signal_hash`, diferenciados por `phase_id` (0..N-1).

---

## Restricciones

- **NUNCA se abre una segunda posición si el `signal_hash` es idéntico al de una posición ya activa en la misma barra.**
- **NUNCA se abren posiciones que violen las restricciones de gestión de riesgo definidas en el `manage` y en los tests de optimización.**
- **EXCEPCIÓN (ADR-0109):** cuando el Genoma de Riesgo y Gestión de Posición resuelve `Split_Position(N_Fases)` para la señal activa, el sistema SÍ abre múltiples tickets con el mismo `signal_hash` en la misma barra — uno por cada fase declarada por el genoma — siempre que cada ticket porte un `phase_id` distinto. Estos tickets no son posiciones duplicadas: son tramos de una misma posición lógica.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| ALLOW_CONCURRENT_TRADES | true | true/false | Habilita múltiples posiciones por estrategia | CONFIG |

---

## Ciclo de Vida de la Feature

### Entrada
- Señal de entrada válida emitida por la estrategia
- Lista de posiciones activas actualmente en la estrategia

### Proceso
- Evalúa si el `signal_hash` y el `timestamp` corresponden a una nueva condición de entrada.
- Aplica las reglas de optimización y de riesgo del portafolio.
- Dispara un nuevo ticket individual si la señal es válida.

### Salida
- Nuevo ticket en estado OPERATING.
- Registro del DAG actualizado con la nueva rama del ticket.

---

## Tareas (TTRs)

### **TTR-001: Validación de Señales y Disparo de Nuevos Tickets**
*   **¿Cuál es el problema?** El sistema debe evitar abrir dos posiciones para la misma señal mientras permite múltiples operaciones en señales diferentes.
*   **¿Qué tiene que pasar?** El gestor compara el `signal_hash` de la señal propuesta con el de las posiciones activas en la misma barra.
*   **¿Cómo sé que está hecho?**
    - [ ] Una segunda señal con el mismo hash es bloqueada.
    - [ ] Una señal en una barra posterior con diferente hash genera un nuevo ticket individual.

### **TTR-002: Seguimiento Independiente de Tickets (OOP Tickets)**
*   **¿Cuál es el problema?** Cada posición debe poder tener su propio nivel de SL, TP o reglas de salida independientes.
*   **¿Qué tiene que pasar?** El sistema trata cada ticket como un objeto único con su propia máquina de estados FSM.
*   **¿Cómo sé que está hecho?**
    - [ ] Puedo cerrar un ticket individual por SL/TP sin alterar la exposición ni los niveles de los demás tickets activos.

### **TTR-003: Materialización de Fases de Salida (Split_Position, ADR-0109)**
*   **¿Cuál es el problema?** El Dominio de Riesgo y Gestión de Posición necesita poder fragmentar una posición lógica en N tramos independientes (`Split_Position(N_Fases)`), cada uno con su propio recorrido de SL/TP, sin que la restricción de unicidad por `signal_hash` (TTR-001) bloquee la apertura de los tickets adicionales.
*   **¿Qué tiene que pasar?** Cuando el genoma activo resuelve `Split_Position(N_Fases)` para una señal, el gestor abre N OOP Tickets en la misma barra, todos con el `signal_hash` de la señal original, cada uno con un `phase_id` único (0..N-1) y su propio SL/TP inicial.
*   **¿Cómo sé que está hecho?**
    - [ ] Una señal con `Split_Position(3)` resuelto genera exactamente 3 tickets en la misma barra, con `phase_id` 0, 1 y 2.
    - [ ] Cada ticket de fase puede cerrarse independientemente (por SL, TP o `Move_SL_To_Target`) sin afectar el estado de los demás tickets de la misma señal.
*   **¿Qué no puede pasar?** Sin `Split_Position(N_Fases)` resuelto para la señal activa, la restricción de TTR-001 (unicidad de `signal_hash` por barra) se mantiene sin excepción.

---

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Esta feature aplica el perfil de **Ops / Hot-Path**:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la orden/fill |
| | `created_at` | Timestamp de origen (nanosegundos) |
| | `audit_hash` | Hash de la transacción (Firma digital) |
| | `audit_chain_hash` | Hash de la secuencia de fills de la sesión |
| | `phase_id` | Índice de fase de salida dentro de `Split_Position(N_Fases)` (ADR-0109); `null` si la posición no está fragmentada |
| **II. Soberanía** | `owner_id` | Usuario responsable del capital real |
| | `compliance_status_id` | Veredicto del Pre-Trade Validator |
| **III. Hardware** | `node_id` | ID del hardware físico ejecutor |
| | `process_id` | PID del motor de ejecución real |
| | `execution_latency_ms` | Latencia señal-a-broker (Máximo 1ms) |

## Gobernanza y Estándares (Fijos)

- **Genomas Modulares por Dominio (ADR-0108/ADR-0109):** Esta feature es Primitiva de Acción de Morfología de Salida (`Split_Position(N_Fases)`) del Dominio de Riesgo y Gestión de Posición. Ver Registro de Dominios Genómicos en [`SAD.md`](../SAD.md) §2.3.

---

## Dependencias
**Depende de:**
- [`order-fsm`](../features/order-fsm.md) — para la máquina de estados.
- [`precision-sizing-models`](../features/precision-sizing-models.md) — para el cálculo de lotaje individual.

**Consumido por:**
- [`execute`](../modules/execute.md) — para la orquestación y escalamiento.
