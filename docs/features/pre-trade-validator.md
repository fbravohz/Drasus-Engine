# Pre-Trade Validator

**Carpeta:** `./features/pre-trade-validator/`
**Estado:** Especificación / Crítica
**Última actualización:** 2026-04-12

---

---

## ¿Qué es?

Componente de alta velocidad encargado de validar cada orden contra **11 filtros de seguridad** críticos antes de permitir su salida al mercado real (Pipeline de 10 pasos de ADR-0025 más el Robustness Verdict Check de ADR-0095). Su misión es la prevención de errores catastróficos ("Fat Finger") y la protección de la integridad del capital en milisegundos.

---

## Comportamientos Observables

- [ ] Valida 11 condiciones críticas en secuencia antes de la ejecución.
- [ ] Bloquea órdenes en < 1ms si el spread es excesivo o el margen insuficiente.
- [ ] Emite códigos de error específicos para cada fallo de validación.

---

## Ciclo de Vida de la Feature — Pre-Trade Validator

### Entrada
- Objeto `Order` con dirección, tamaño y precio.
- Estado actual de cuenta (Margen/Balance).
- Parámetros de umbral activos.

### Proceso
- Ejecuta el **Pipeline de 11 Checks Secuenciales**:
  1. **Liquidity & Spread Gap Check:** Mide volumen y spread. Bloquea si detecta caída de volumen >60% o spread excesivo.
  2. **Slippage Check:** Valida el precio de señal vs precio actual.
  3. **Position Size Check:** Valida si excede el lotaje máximo permitido para el símbolo.
  4. **Portfolio Exposure Check:** Valida si excede el % de capital global asignado por símbolo o sector.
  5. **Correlation Check:** Valida si genera exposición correlacionada excesiva con posiciones abiertas.
  6. **Drawdown Breaker:** Valida si el DD actual > máximo permitido por el Design Manifest.
  7. **Daily Loss Limit Check:** Valida si la pérdida diaria > límite histórico o de la Prop Firm.
  8. **Order Frequency Check:** Limitador de ráfagas de órdenes (Anti-bug/Anti-HFT accidental).
  9. **Margin Check:** Valida si hay suficiente margen según las reglas del bróker.
  10. **Robustness Verdict Check (Veto Monte Carlo - ADR-0095):** Bloquea si la estrategia carece de veredicto o si está catalogada como `PROP_FIRM_FRAGILE` o `TOXIC` bajo severidad `HARD_VETO`.
  11. **Final Operational Approval:** Veredicto final del orquestador de ejecución.


### Salida
- **Veredicto:** VALIDADA / RECHAZADA.
- **Trace ID:** Vínculo al log de auditoría del veredicto.

### Contextos de Uso
**Contexto Único: Módulo Execute**
- Es el componente más crítico en términos de rendimiento dentro del flujo de trading en vivo.

---

---

## Tareas (TTRs)

### **TTR-001: Pipeline de los 11 Checks Secuenciales (ADR-0025)**
*   **Descripción:** Ejecuta la suite de validación (Capital → Size → Exposure → DD → Corr → Liquidity → Slippage → Robustness Verdict → Approval).
*   **Reglas de Negocio:**
    * Si CUALQUIER check falla, la orden se marca como `REJECTED` y no sale al broker.
    * El veredicto de robustez se valida contra el estado inmutable registrado para el hash de versión de la estrategia.
    * El veredicto debe incluir el `audit_hash` del estado actual (ADR-0020 V2).
*   **Entrada:** `Order`, `AccountState`, `MarketState`.
*   **Salida:** `bool` (is_safe), `error_code`, `failed_check_id`.
*   **Precondición:** Memoria compartida con estados de cuenta actualizada.
*   **Postcondición:** Registro del intento de trade en `pre_trade_logs` con `process_id`.

### **TTR-002: Optimización de Hot Path (SLA < 1ms)**
*   **Descripción:** Garantiza que los checks se ejecuten en tiempo récord usando lógica vectorizada o Rust SIMD/Rayon.
*   **Reglas de Negocio:**
    * NUNCA realizar I/O de disco o red dentro del pipeline de validación.
    * Se debe registrar la `latency_ns` de cada check para auditoría institucional (ADR-0020 V2).
*   **Entrada:** `PerformanceProfiler`.
*   **Salida:** `MetricsSummary`.
*   **Precondición:** Motor de validación pre-compilado en el arranque del sistema.
*   **Postcondición:** Alarma disparada vía `notification` si la latencia promedio excede los 0.5ms.

---

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Toda validación pre-milio registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la validación |
| | `created_at` | Timestamp de ejecución (nanosegundos) |
| | `audit_hash` | Hash del estado de cuenta evaluado |
| | `audit_chain_hash` | Hash del rastro de cumplimiento |
| **II. Soberanía** | `owner_id` | Dueño responsable |
| | `manifest_id` | ID del contrato de diseño legal |
| **III. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del validador |
| **IV. Latencia** | `execution_latency_ms` | Latencia de validación (< 1ms target) |
| | `latency_ns` | Latencia en nanosegundos para alta resolución |

- **Decisión Arquitectónica Asociada:**
    - ADR-0004: FSM para registro de rechazos atómicos.
    - ADR-0010: Hard Limits (Checks de riesgo).
    - ADR-0020 V2: Inundación de Fundaciones.

---

## Dependencias
**Depende de:**
- [`portfolio-rules`](../features/portfolio-rules.md) — para la jerarquía de límites.
- [`data-validator`](../features/data-validator.md) — para la sanidad de los precios de referencia.

**Consumido por:**
- [`execute`](../modules/execute.md) — para la validación crítica pre-mercado.
