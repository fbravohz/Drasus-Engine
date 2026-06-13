# Order FSM — Máquina de Estados de Órdenes

**Carpeta:** `./features/order-fsm/`
**Estado:** Lista para implementar
**Última actualización:** 2026-04-09
**Decisión Arquitectónica Asociada:** ADR-0004 (Máquina de Estados FSM)

---

## ¿Qué es?

El Order FSM define los 6 estados posibles de una orden y las transiciones válidas entre ellos. Una orden es un contrato para comprar o vender un número de contratos de un símbolo en el futuro (o inmediatamente).

**Problema:** Si una orden puede transitar de cualquier estado a cualquier otro, el sistema es impredecible e inseguro. Ej: no puedes "ejecutar" una orden que fue rechazada.

**Solución:** Definir una máquina de estados finita: qué transiciones son válidas, cuáles no son. Cualquier intento de transición inválida es rechazado antes de persistir. De forma adicional, las órdenes también tienen asociadas posiciones (número de contratos abiertos), y las posiciones tienen sus propios invariantes de margen.

**Resultado observable:** Las órdenes se mueven de forma predecible y controlada. Imposible alcanzar estados inconsistentes.

---

## Comportamientos Observables

- [ ] Execute envía una nueva orden al broker
  → Order comienza en estado ENVIADA (SENT)
  → El broker responde: "Aprobada" o "Rechazada"
  → Si Aprobada: transita a APROBADA (APPROVED)
  → Si Rechazada: transita a RECHAZADA (REJECTED) — es terminal, no hay más transiciones

- [ ] Un usuario intenta cambiar manualmente el estado de una orden de ENVIADA a COMPLETADA
  → Sistema rechaza: "Transición no válida. ENVIADA solo puede → APROBADA o RECHAZADA"
  → El cambio nunca se persiste

- [ ] Una orden alcanza estado COMPLETADA
  → Se registra en Audit Log: "ORDER_FILLED: order_id=123, timestamp=..., quantity_filled=..."
  → La orden puede transitar a CANCELADA (si el usuario la cancela manualmente)
  → O permanece en COMPLETADA indefinidamente

- [ ] Un trade se ejecuta (ej: se compran 10 contratos)
  → Se crea una Posición abierta: symbol, quantity=10, avg_entry_price, unrealized_pnl, available_margin
  → Se validan dos invariantes sobre la Posición:
    1. Margen disponible >= 0 (HARD: si violaría, se rechaza antes de ejecutar la orden)
    2. Cantidad de contratos > 0 (para posiciones abiertas)
  → Si ambas se cumplen, la posición se persiste

- [ ] El usuario consulta "¿Qué órdenes enviadas hace más de 1 hora todavía están en ENVIADA?"
  → Sistema busca en el historial de órdenes
  → Devuelve la lista (todas están bloqueadas: en estado ENVIADA esperando respuesta del broker, algo puede estar mal)

---

## Restricciones

- **NUNCA una orden transita entre estados que no estén en la FSM válida.** Cualquier intento es rechazado.
- **NUNCA una Posición abierta tiene margen negativo.** Es una HARD constraint — si el trade la violaría, se rechaza la orden antes de enviarla.
- **NUNCA una Posición abierta tiene cantidad de contratos <= 0.** Una posición cerrada se elimina (no persiste).
- **NUNCA se persiste una orden sin timestamp.** Cada transición de estado incluye un timestamp para auditoría.
- **NUNCA se permite una orden con precio fuera del rango válido del símbolo.** (Ej: precio negativo, precio 1000x el máximo histórico)

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace |
|---|---|---|---|
| POSITION_MARGIN_HARD_LIMIT | true | true / false | Si true (FIJO NO CONFIGURABLE), margen negativo causa rechazo inmediato. Si false, permite margen negativo (NUNCA hacer esto en producción) |
| ORDER_TIMEOUT | infinity | 1-86400 segundos | Si una orden está en ENVIADA > N segundos, se marca como timeout (opcional) |
| SLIPPAGE_FACTOR | 0.0001 | 0.0-0.01 | Margen de precio simulado en backtests (multiplicado al precio) |

---

## Ciclo de Vida de la Feature

### Entrada
- **Quién llama:** Execute (orders de producción y paper trading), Incubate (órdenes de paper trading), Validate (órdenes simuladas en backtests)
- **Qué recibe:** Una orden candidata (símbolo, cantidad, precio, dirección LONG/SHORT, tipo de orden)

### Proceso
1. **Validación de invariantes de orden:** Se valida cantidad > 0, precio en rango válido del símbolo
2. **Cálculo de margen:** Se calcula el margen requerido para abrir la posición (configurable por leverage)
3. **Validación de Posición resultante:** Se proyecta la posición resultante (si la orden se ejecutara) y se valida que margen disponible >= 0 (HARD)
4. **Transición de estado:** Se ejecuta la transición ENVIADA → APROBADA/RECHAZADA según respuesta del broker
5. **Registro en Audit Log:** Se loguea cada cambio de estado

### Salida
- **Qué produce:** 
  - Si todas las validaciones pasan: Orden persiste en ENVIADA, esperando respuesta del broker
  - Si alguna validación falla: Rechazo inmediato, nunca se persiste la orden, Audit Log registra la violación

### Contextos de Uso
- **Execute:** Usa FSM para validar órdenes antes de enviarlas al broker real
- **Incubate (Paper Trading):** Usa FSM para simular ciclo de vida de órdenes sin broker real
- **Validate (Backtests):** Usa FSM para generar respuestas simuladas del broker (transiciones de estado)

---

## Tareas (TTRs)

### **TTR-001: Validar y ejecutar transición de estado de una orden (FSM)**
*   **Descripción:** Valida y ejecuta la transición entre estados de la orden (SENT → APPROVED → FILLED).
*   **Reglas de Negocio:**
    * Toda transición DEBE registrar el `audit_hash` del estado anterior para garantizar inmutabilidad.
    * Las transiciones hacia estados terminales (REJECTED, CANCELLED, RETIRED) son irreversibles.
*   **Entrada:** `order_id`, `target_state`, `reason`, `process_id` (ADR-0020 V2).
*   **Salida:** `bool` (success), `transition_metadata`.
*   **Precondición:** Orden cargada en memoria con `audit_hash` verificado.
*   **Postcondición:** Registro en `audit-log` con precisión de nanosegundos (ADR-0020 V2).

### **TTR-002: Invariantes de Posición e Inundación Institucional**
*   **Descripción:** Valida `available_margin >= 0` y `quantity > 0` antes de la persistencia atómica.
*   **Reglas de Negocio:**
    * Toda posición persistida DEBE incluir `institutional_tag` y `version_node_id` (ADR-0020 V2).
    * Una violación de `available_margin` dispara una `CIRCUIT_BREAKER` alert (ADR-0010).
*   **Entrada:** `Position` object (Pure Entity).
*   **Salida:** `ValidationResult`.
*   **Precondición:** Cálculo de margen finalizado.
*   **Postcondición:** Persistencia con `audit_hash` proyectado.

### **TTR-003: ATM Canvas (Lienzo FSM de Gestión de Órdenes)**
*   **¿Cuál es el problema?** Definir el ciclo de vida complejo de un trade (ej: Pending -> Partial Fill -> Trailing Active) mediante código es complejo y poco intuitivo.
*   **¿Qué tiene que pasar?** Proveer un lienzo visual dedicado en Flutter CustomPainter que permita al usuario diseñar y enlazar las transiciones de estados de órdenes de forma gráfica.
*   **¿Cómo sé que está hecho?**
    - [ ] Puedo diseñar una máquina de estados de orden personalizada, exportar su configuración e inyectarla como la FSM de ejecución.

---

## Gobernanza y Estándares (Fijos)
## Persistencia (Inundación de Fundamentos — ADR-0020 V2 · Perfil C Hot-Path, híbrido C+III)

Híbrido: Perfil C (Ops/Hot-Path = I + II + IV + V latencia) + linaje III legítimo (resultado forense-reproducible de cada transición de orden). El linaje se mantiene a propósito.

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único (atómico) |
| | `created_at` | Timestamp de transición (nanosegundos) |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de integridad del estado actual |
| | `audit_chain_hash` | Hash del historial de la sesión |
| | `event_sequence_id` | Secuencia de recuperación de la FSM |
| **II. Soberanía** | `owner_id` | Dueño responsable |
| | `manifest_id` | ID del contrato de diseño legal |
| **III. Linaje (híbrido)** | `logic_hash` | Hash del motor de ejecución (FSM) |
| | `version_node_id` | Versión de la estrategia en el DAG |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del ejecutor/worker |
| **V. Forense & Ejecución** | `execution_latency_ms` | Latencia de transición interna (hot-path) |
| | `source_signal_id` | Señal de origen que disparó la transición |
| | `indicator_state_hash` | Snapshot técnico T-0 de la ejecución (Margen/Precio) — Grupo V, recategorizado desde III |

- **Decisión Arquitectónica Asociada:**
    - ADR-0004: Máquina de Estados FSM (int64).
    - ADR-0010: Hard Limits (Cierre automático ante margen insuficiente).
    - ADR-0020 V2: Inundación de Fundaciones.

---

## Dependencias
**Depende de:**
- [`clock`](../features/clock.md) — para timestamps deterministas.
- [`audit-log`](../features/audit-log.md) — para registro institucional.

**Consumido por:**
- [`execute`](../modules/execute.md) — para orquestación de órdenes reales.
- [`incubate`](../modules/incubate.md) — para ciclo de vida en paper trading.
- [`portfolio-rules`](../features/portfolio-rules.md) — para evaluación de márgenes.
