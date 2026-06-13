# Broker Connector — Abstracción de Comunicación con Brokers

**Carpeta:** `./features/broker-connector/`
**Estado:** Lista para implementar
**Última actualización:** 2026-04-08

---

## ¿Qué es?

Abstrae la comunicación con brokers externos (Binance, IBKR, Oanda). Se apalanca en los **Adaptadores Nativos de NautilusTrader** para garantizar latencia mínima y estabilidad institucional.

---

## Comportamientos Observables

- [ ] Execute envía orden a Broker Connector
  → El conector sabe internamente que usar es IBKR
  → Convierte orden genérica a formato IBKR
  → Envía, recibe respuesta
  → Convierte respuesta de vuelta a formato genérico
  → Devuelve broker_order_id

- [ ] Usuario cambia de broker (IBKR → Binance)
  → Solo cambia el adaptador inyectado
  → El código de Execute NO cambia
  → Los tests NO necesitan cambiar

---

## Restricciones

- **NUNCA Broker Connector expone detalles específicos de broker.**
- **NUNCA se envía orden sin heartbeat exitoso primero.**

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace |
|---|---|---|---|
| BROKER | "paper" | cadena | Qué broker conectar (ej: "ibkr", "binance", "paper") |
| HEARTBEAT_INTERVAL | 30 seg | 5-300 seg | Cada cuántos segundos verificar conexión |

---

## Ciclo de Vida
*   **Entrada:** `execute` (órdenes live), `incubate` (paper trading), `manage` (rebalanceo).
*   **Proceso:** Abstracción de protocolos (FIX, REST, gRPC/WebSocket) → Normalización a contratos internos.
*   **Salida:** Eventos de ejecución, fills normalizados, rastro de auditoría.

---

## Tareas (TTRs)

### **TTR-001: Establecer y mantener conexión con broker**
*   **Descripción:** Conecta con el broker y verifica que conexión está viva (heartbeat).
*   **Reglas de Negocio:**
    * NUNCA operar sin heartbeat exitoso (< 5s latencia).
    * Reintento automático exponencial ante pérdida de socket.
*   **Entrada:** `connection_id` (UUID).
*   **Salida:** `bool` (status de conexión).
*   **Precondición:** Credenciales autorizadas en `broker_connections`.
*   **Postcondición:** Canal de eventos activo y suscrito a tópicos de ejecución.

### **TTR-002: Ejecución de Órdenes y Fills**
*   **Descripción:** Enviar órdenes genéricas al broker y normalizar los fills recibidos.
*   **Reglas de Negocio:**
    * Cada orden devuelta DEBE incluir el `broker_order_id` inmutable.
    * Timestamps de fill deben convertirse a escala de nanosegundos (ADR-0020 V2).
*   **Entrada:** Objeto `Order` (genérico).
*   **Salida:** `FillEvent` (normalizado).
*   **Precondición:** Sesión de trading OPERATING.
*   **Postcondición:** Rastro de evidencia emitido para `feedback`.

---

## Gobernanza y Estándares (Fijos)
## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Toda interacción con el conector registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la interacción |
| | `created_at` | Timestamp de envío/recepción |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de integridad del mensaje normalizado |
| | `audit_chain_hash` | Hash de la sesión del conector |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **II. Soberanía** | `owner_id` | Dueño de la cuenta |
| | `access_token_id` | Token de autorización (API Key Ref) |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del adaptador Nautilus |
| | `indicator_state_hash` | Signal ID que detonó la transacción |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del conector de bajo nivel |
| | `execution_latency_ms` | Latencia de red (Wire time) |

- **Decisión Arquitectónica Asociada:**
    - ADR-0004: FSM para transiciones de órdenes.
    - ADR-0013: Stack NautilusTrader para conectividad nativa.
    - ADR-0020 V2: Inundación de Fundaciones.

---

## Dependencias
**Depende de:**
- [`clock`](../features/clock.md) — para timestamps deterministas.
- [`audit-log`](../features/audit-log.md) — para rastro inmutable.

**Consumido por:**
- [`execute`](../modules/execute.md) — para ejecución real.
- [`incubate`](../modules/incubate.md) — para paper trading.
