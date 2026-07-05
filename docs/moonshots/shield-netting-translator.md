# Shield Netting Translator — Traductor de Compensación y Netting

**Carpeta:** `./features/shield-netting-translator/`
**Estado:** Lista para implementar
**Última actualización:** 2026-05-02
**Decisión Arquitectónica Asociada:** ADR-0078 (Autopilot Execution & Stealth Network Infrastructure)

---

## ¿Qué es?

El traductor de compensación es una capa intermedia que actúa como envoltorio de set algorítmicos para compactar operaciones subyacentes de cobertura (Hedging) que son transicionales (opciones netas de diferencias).

**Problema:** Muchos brokers (o regulaciones FIFO / NFA) prohíben el hedging directo o limitan el número de posiciones abiertas por cuenta. El Shield Netting Translator resuelve esto mapeando las múltiples posiciones de cobertura de las estrategias individuales hacia una posición consolidada neta en el broker maestro.

---

## Comportamientos Observables

- [ ] Estrategias abren órdenes de compra y venta simultáneas
  → El traductor intercepta las órdenes.
  → Calcula la posición neta resultante.
  → Modifica o cierra la posición neta en el broker real para reflejar exactamente la exposición neta.

- [ ] Una estrategia cierra su posición de cobertura
  → El traductor calcula la diferencia neta de exposición.
  → Envía una orden de ajuste al broker para alcanzar el nuevo nivel neto.

---

## Restricciones

- **NUNCA se violan las reglas FIFO del broker.**
- **NUNCA la posición neta real diverge de la suma de las posiciones virtuales de las estrategias.**

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| NETTING_AGGREGATION_MS | 50 | 1-500 | Ventana de tiempo para agrupar órdenes de cobertura | CONFIG |

---

## Ciclo de Vida de la Feature

### Entrada
- Lista de órdenes virtuales generadas por múltiples estrategias
- Exposición actual neta en el broker real

### Proceso
- Consolida las direcciones y tamaños de las órdenes virtuales
- Calcula la orden de compensación óptima para lograr la exposición deseada
- Envía la orden neta al broker

### Salida
- Orden neta ejecutada en el broker real
- Reconciliación de fills virtuales asignados a cada estrategia localmente

---

## Tareas (TTRs)

### **TTR-001: Mapeo dinámico de órdenes virtuales a neta**
*   **¿Cuál es el problema?** El broker no permite posiciones opuestas en el mismo activo simultáneamente.
*   **¿Qué tiene que pasar?** El traductor suma y resta los tamaños virtuales de las estrategias para emitir una única orden neta consolidada.
*   **¿Cómo sé que está hecho?**
    - [ ] Puedo ver en los logs cómo una orden virtual de compra de 1 lote y una de venta de 0.5 lotes se convierten en una orden de compra neta de 0.5 lotes en el broker.

### **TTR-002: Control de reconciliación FIFO**
*   **¿Cuál es el problema?** Las órdenes de salida deben aplicarse en el orden exacto de entrada (First-In, First-Out) sin violar las restricciones de cierre del broker.
*   **¿Qué tiene que pasar?** El traductor gestiona la cola de entrada de las órdenes virtuales para cerrar siempre la posición más antigua de forma secuencial.
*   **¿Cómo sé que está hecho?**
    - [ ] Los cierres parciales de posiciones virtuales respetan el orden FIFO estricto en la cuenta neta.

---

## Gobernanza y Estándares (Fijos)
## Persistencia (Inundación de Fundamentos — ADR-0020)

Esta feature aplica el perfil de **Ops / Hot-Path**:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la orden/fill |
| | `created_at` | Timestamp de origen (nanosegundos) |
| | `audit_hash` | Hash de la transacción (Firma digital) |
| | `audit_chain_hash` | Hash de la secuencia de fills de la sesión |
| **II. Soberanía** | `owner_id` | Usuario responsable del capital real |
| | `compliance_status_id` | Veredicto del Pre-Trade Validator |
| **III. Hardware** | `node_id` | ID del hardware físico ejecutor |
| | `process_id` | PID del motor de ejecución real |
| | `execution_latency_ms` | Latencia señal-a-broker (Máximo 1ms) |

---

## Dependencias
**Depende de:**
- [`order-fsm`](../features/order-fsm.md) — para la máquina de estados.
- [`audit-log`](../features/audit-log.md) — para rastro inmutable.

**Consumido por:**
- [`execute`](../modules/execute.md) — para la compensación de órdenes de cobertura.
