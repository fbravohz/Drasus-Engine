# Ejecutar

**Carpeta:** `./modules/execute/`
**Estado:** Orquestador (Imperative Shell)
**Última actualización:** 2026-06-11

---

## ¿Qué es?

El módulo de ejecución es donde el dinero real entra en juego. Recibe señales de las estrategias activas, pasa 8 verificaciones previas a cada orden, y la envía al broker. Se apalanca en el motor institucional de **NautilusTrader** (ADR-0013) para la conectividad de baja latencia y el loop de eventos determinista. Todo esto en menos de 5 milisegundos.

También tiene un sistema de vigilancia (watchdog) que monitorea continuamente que todo esté bien. Si detecta que el DrawDown superó un límite crítico, cierra todas las posiciones automáticamente sin esperar instrucción del usuario.

El usuario siempre tiene la capacidad de vetar cualquier decisión automática del sistema dentro de una ventana de tiempo.

---

## Épica 0: Esqueleto Fundacional

### Estructura de Archivos (FCIS — ADR-0003)

```
crates/execute/
├── public_interface.rs   # Frontera pública: único punto de entrada para otros módulos
├── logic.rs              # Lógica pura: FSM de órdenes, pre-trade checks (sin DB, sin I/O)
├── orchestrator.rs       # Coordinación: invoca Pre-Trade Validator, Broker Connector, Watchdog
├── persistence.rs        # Acceso a SQLite WAL (lectura/escritura)
├── schemas.rs            # Definición de tablas: orders, fills, pre_trade_logs, event_store
└── types.rs              # Tipos de entrada/salida: Order, Fill, TacticalClearance, KillSwitchSignal
```

### Vocabulario de Persistencia — Catálogo de 25 Campos (ADR-0020)

Esta tabla es el **catálogo de referencia completo** del Contrato Global de ADR-0020 (vocabulario lógico, no esquema literal). La migración 0001 crea la tabla ancla `foundation_master_fields` con estas 25 columnas como referencia ÚNICA del sistema — este módulo NO la replica.

Las tablas propias de este módulo (una por feature/TTR, en sus propias migraciones) llevan: el **Grupo I (Identidad & Integridad, 6 primeras filas) de forma universal y obligatoria**, más solo los campos concretos de los Grupos II–V que correspondan al **Perfil Técnico** de cada feature (Filtro de Relevancia, tabla canónica en ADR-0020) — nunca el catálogo completo. Cada feature documenta su selección en su propia sección "Contrato de Persistencia" (`features/*.md`).

| Categoría | Campo | Descripción |
|---|---|---|
| **I. Identidad e Integridad** | `id` | UUID del registro |
| | `created_at` | Timestamp de creación (nanosegundos) |
| | `updated_at` | Timestamp de última modificación |
| | `audit_hash` | SHA-256 del contenido del registro |
| | `audit_chain_hash` | Hash encadenado al registro anterior |
| | `event_sequence_id` | Secuencia de recuperación post-crash |
| **II. Soberanía y Propiedad** | `owner_id` | Dueño del capital/IP |
| | `institutional_tag` | Etiqueta de entorno (PROD/PAPER/CHALLENGE) |
| | `manifest_id` | Contrato de diseño vinculado |
| | `access_token_id` | Token de autenticación usado |
| **III. Linaje Alpha y Datos** | `version_node_id` | Nodo en el DAG de versiones |
| | `parent_id` | Puntero al registro padre |
| | `logic_hash` | Hash del motor de ejecución |
| | `data_snapshot_id` | Snapshot PIT del mercado |
| | `transformation_id` | ID del paso/tipo de transformación aplicado |
| **IV. Infraestructura y Ops** | `process_id` | PID del motor de ejecución real |
| | `session_id` | Agrupación de runtime |
| | `node_id` | ID del hardware físico |
| **V. Forense y Ejecución** | `portfolio_container_id` | Contenedor de portafolio |
| | `compliance_status_id` | Veredicto del Pre-Trade Validator |
| | `risk_audit_id` | Ticket detallado de riesgo |
| | `indicator_state_hash` | Snapshot técnico |
| | `execution_latency_ms` | Latencia señal-a-broker (Hot-Path) |
| | `source_signal_id` | Link a señal origen |
| | `signature_hash` | HMAC de señales |

### TTRs Etiquetados por Fase

| TTR | Fase | Descripción corta |
|---|---|---|
| TTR-001 | **EPIC-5** | Validación táctica (Pre-Trade Validator & 10 Pasos) |
| TTR-002 | **EPIC-5** | Despliegue de órdenes (Broker Connector) |
| TTR-003 | **EPIC-5** | Vigilancia de emergencia (System Watchdog) |
| TTR-004 | **EPIC-5** | Telemetría institutional |
| TTR-005 | **EPIC-5** | Persistencia crítica e inmunidad (Event Store) |
| TTR-014 | **EPIC-5** | Estados atómicos (Order FSM) |
| TTR-015 | **EPIC-5** | Deslizamiento (Slippage Models) |
| TTR-016 | **EPIC-5** | Rastreo (Equity Curve Tracker) |
| TTR-019 | **EPIC-5** | Alertas (Notification) |
| TTR-020 | **EPIC-5** | Asíncrona (Async Job Executor) |
| TTR-021 | **EPIC-5** | Afinidad (Production Optimization) |
| TTR-022 | **EPIC-5** | Forense (Audit Log) |
| TTR-023 | **EPIC-5** | Temporal (Clock) |
| TTR-024 | **EPIC-5** | Vigilancia Pardo y SSL (Operational Safety) |
| TTR-026 | **EPIC-5** | Bridge multiplataforma (Multiplatform Execution Bridge) |
| TTR-027 | **EPIC-5** | Gestor multi-ticket (Multi-Ticket Manager) |
| TTR-028 | **EPIC-5** | Cola anti-throttling (Order-Priority Queue) |
| TTR-032 | **EPIC-5** | Daemons persistentes (LiveNode Aisle) |
| TTR-033 | **EPIC-5** | Multiplexación (Data Bus) |
| TTR-034 | **EPIC-5** | Protocolo de recuperación (Crash Recovery) |
| TTR-038 | **EPIC-5** | Seguridad soberana (Sovereign Security) |
| TTR-040 | **EPIC-5** | Auto-auditoría de portafolios vivos |
| TTR-042 | **EPIC-5** | Acceso agéntico MCP (Cabina Dual — bloqueado por defecto) |
| TTR-043 | **EPIC-5** | Integración indicador fundamental en ejecución (Fundamental Indicator Projector) |
| TTR-044 | **EPIC-5** | Entradas concurrentes no bloqueantes (ADR-0129) |
| TTR-006 | **EPIC-6** | Bridge de ejecución (Nautilus Integration) |
| TTR-011 | **EPIC-6** | Guardia de microestructura (Pre-Trade Order Flow) |
| TTR-013 | **EPIC-6** | Dimensionamiento táctico (Live Sizing con Robustness Score) |
| TTR-017 | **EPIC-6** | Adaptación (HMM Regime) |
| TTR-018 | **EPIC-6** | Aislamiento (Executable Container) |
| TTR-025 | **EPIC-6** | Escalado de volatilidad (Target Vol) |
| TTR-029 | **EPIC-6** | Advanced Trade Management (ATM) |
| TTR-030 | **EPIC-6** | Micro-gestión cinética |
| TTR-031 | **EPIC-6** | Autopilot Metrics Provider |
| TTR-035 | **EPIC-7** | Auditor de rendimiento (Real-Time Auditor) |
| TTR-036 | **EPIC-6** | Despacho y aislamiento federado (Federated Execution) |
| TTR-037 | EPIC-9+ | Copy-Trading (Signal Relay & Risk Scaling) |
| TTR-039 | **EPIC-6** | Monitor de latencia de broker (Throttling Metrics Dashboard) |
| TTR-041 | **EPIC-6** | Genes de acción del genoma de riesgo y gestión de posición (ADR-0108/ADR-0109) |

---

## Comportamientos Observables (Orquestación)

- [ ] **Validación Táctica:** Antes de enviar, coordina el paso de la orden por [pre-trade-validator](../features/pre-trade-validator.md).
- [ ] **Despliegue de Órdenes:** Envía órdenes aprobadas a través de [broker-connector](../features/broker-connector.md).
- [ ] **Supervisión de Emergencia:** Mantiene activo el [system-watchdog](../features/system-watchdog.md) para cierres automáticos de protección.
- [ ] **Protocolo de Veto:** Gestiona la ventana de intervención del usuario en las transiciones de [order-fsm](../features/order-fsm.md).

---

## Restricciones

- Latencia máxima de ejecución (señal → orden enviada): configurable, objetivo < 5ms en hot path
- El watchdog no puede bloquear la ejecución normal (corre en paralelo)
- **FIJO — NO CONFIGURABLE:** Cada cambio de estado de una orden es atómico y registrado — no puede haber estados intermedios o perdidos

---

## Parámetros Configurables

| Parámetro | Default | Qué hace |
|---|---|---|
| MAX_DD_MULTIPLIER | configurable | DrawDown máximo como múltiplo del baseline (ej: 1.5x) para activar kill switch |
| WATCHDOG_INTERVAL | configurable | Cada cuántos segundos el watchdog verifica el estado |
| VETO_WINDOW | configurable | Cuánto tiempo tiene el usuario para revertir una decisión automática |
| PRE_TRADE_CHECKS | configurable | Cuáles de las 8 verificaciones están activas |

---

## Features Consumidas (Reutilizables)

> *(ADR-0137)* Este módulo es la **composición preset canónica** de estas features — define el cableado por defecto. En el Canvas [Forge/Reactor], las features pueden conectarse directamente sin que este módulo sea intermediario obligatorio en runtime.

- **[`order-fsm`](../features/order-fsm.md)** — Máquina de estados de órdenes (atómica e inmutable).
- **[`broker-connector`](../features/broker-connector.md)** — Adaptador de comunicación de baja latencia con brokers.
- **[`system-watchdog`](../features/system-watchdog.md)** — Guardián de seguridad y kill switch de emergencia.
- **[`slippage-models`](../features/slippage-models.md)** — Modelado de spread y deslizamiento en vivo.
- **[`equity-curve-tracker`](../features/equity-curve-tracker.md)** — Tracking de capital y PnL en tiempo real.
- **[`hmm-regime-detection`](../features/hmm-regime-detection.md)** — Filtro de ejecución según régimen de mercado.
- **[`pre-trade-validator`](../features/pre-trade-validator.md)** — 8 validaciones tácticas previas a la orden.
- **[`executable-container`](../features/executable-container.md)** — Contenedor de ejecución institucional (Runner) para dinero real.
- **[`notification`](../features/notification.md)** — Sistema de alertas y avisos institucionales.
- **[`telemetry`](../features/telemetry.md)** — Emisión de métricas de rendimiento del sistema y salud de hilos.
- **[`async-job-executor`](../features/async-job-executor.md)** — Orquestación de reportes y cierres en segundo plano.
- **[`production-optimization`](../features/production-optimization.md)** — Ajustes de latencia y afinidad de CPU para el hot-path.
- **[`audit-log`](../features/audit-log.md)** — Registro inmutable de transacciones ( nanosegundos).
- **[`audit-event-store`](../features/audit-event-store.md)** — Almacén inmutable de eventos para reconstrucción y auditoría de ejecución (SQX Mod 5.3.5).
- **[`clock`](../features/clock.md)** — Timestamps deterministas para sincronización de órdenes.
- **[`nautilus-integration`](../features/nautilus-integration.md)** — Bridge institucional para ejecución live determinista.
- **[`order-flow-microstructure`](../features/order-flow-microstructure.md)** — Validación de liquidez y pánicos pre-trade.
- **[`volatility-stabilization`](../features/volatility-stabilization.md)** — Escalado dinámico por Target Vol y veto por régimen (ADR-0068).
- **[`operational-safety-monitor`](../features/operational-safety-monitor.md)** — Vigilancia de Pardo Profile y Strategy Stop-Loss (ADR-0070).
- **[`federated-portfolio`](../features/federated-portfolio.md)** — Aislamiento lógico de reglas y gobernanza autónoma de múltiples contenedores de portafolios.
- **[`multiplatform-execution-bridge`](../features/multiplatform-execution-bridge.md)** — Comunicación monda vía WebSockets/gRPC hacia múltiples terminales externas (ADR-0078).
- **[`multi-ticket-manager`](../features/multi-ticket-manager.md)** — Gestión concurrente de múltiples tickets individuales identificados por signal hash (ADR-0078).

- **[`precision-sizing-models`](../features/precision-sizing-models.md)** — Cálculo de lotaje táctico (Fixed Ratio, ATR, % Riesgo).
- **[`robustness-score-aggregator`](../features/robustness-score-aggregator.md)** — Score de robustez como parámetro de entrada para dimensionamiento de posición (ADR-0058).
- **[`advanced-trade-management`](../features/advanced-trade-management.md)** — Gestión operativa multicapa, Grid Trading y Hedging.
- **[`kinetic-micro-management`](../features/kinetic-micro-management.md)** — Módulo defensivo hostil de scale out y z-score trailing.
- **[`order-priority-queue`](../features/order-priority-queue.md)** — Cola inteligente anti-throttling con backoff exponencial.
- **[`autopilot-metrics-provider`](../features/autopilot-metrics-provider.md)** — Métricas dinámicas en tiempo real del Autopilot.
- **[`persistent-daemons`](../features/persistent-daemons.md)** — Aislamiento de núcleo y procesos de larga duración para el LiveNode (ADR-0084).
- **[`data-bus-pubsub`](../features/data-bus-pubsub.md)** — Multiplexación de datos de mercado zero-copy para múltiples agentes (ADR-0085).
- **[`crash-recovery`](../features/crash-recovery.md)** — Protocolo de recuperación de estado (Crash Recovery) y reconciliación atómica.
- **[`copy-trading-engine`](../features/copy-trading-engine.md)** — Relevo y escalado de órdenes copy-trading en tiempo real (ADR-0092).
- **[`sovereign-security`](../features/sovereign-security.md)** — Encriptación de llaves, registro inmutable de auditoría y cero telemetría (ADR-0093).
- **[`auto-auditoria-portafolios-vivos`](../features/auto-auditoria-portafolios-vivos.md)** — Monitoreo dinámico de costes reales de ejecución y recalculador de R Expectancy.

---


## Ciclo de Vida

1. **Inicialización:** Carga de inventario desde `audit-event-store` y sincronización con broker.
2. **Monitoreo:** Activación de `system-watchdog` y `telemetry`.
3. **Ejecución:** Recepción de señal → Validación Táctica → Guardia de Microestructura → Detonación Metamórfica → **Dimensionamiento de Precisión** → Envío al Broker.
4. **Cierre:** Liquidación de posiciones y persistencia final de logs.

---

## Tareas (TTRs) — Protocolo de Orquestación (§8.1)

### **TTR-001: Orquestación de Validación Táctica (Pre-Trade Validator & 10 Pasos)**
*   **Descripción:** Invoca a [`pre-trade-validator`](../features/pre-trade-validator.md) antes de cada envío de orden para validar la secuencia de 10 pasos (Liquidity, Slippage, Position Size, Portfolio Exposure, Correlation, Drawdown Breaker, Daily Loss Limit, Order Frequency, Margin Check, Final Approval).
*   **Reglas de Orquestación:**
    * El "Hot-Path" de validación debe completarse en < 1ms.
    * Si cualquier check falla, la orden es abortada y se emite un log a Flutter explicando qué regla bloqueó al agente.
    * Cualquier rechazo debe vincularse al `audit_hash` del estado de cuenta actual (ADR-0020).
*   **Entrada:** `proposed_order`.
*   **Salida:** `tactical_clearance` (APPROVED | REJECTED).
*   **Precondición:** Estrategia autorizada por el portafolio (Módulo `manage`).
*   **Postcondición:** Registro del intento de trade en `pre_trade_logs`.



### **TTR-002: Orquestación de Despliegue de Órdenes (Broker Connector)**
*   **Descripción:** Envía órdenes autorizadas a través de [`broker-connector`](../features/broker-connector.md) usando NautilusTrader.
*   **Reglas de Orquestación:**
    * Utiliza marcas de tiempo de nanosegundos para la sincronización de fills (ADR-0013).
    * La ejecución ocurre estrictamente bajo el protocolo **Bar-Open Alignment** para paridad con backtesting.
    * Cada orden enviada debe heredar el `process_id` de la señal generadora (ADR-0020).
*   **Entrada:** `authorized_order`.
*   **Salida:** `broker_acknowledgment`, `fill_event`.
*   **Precondición:** TTR-001 aprobado.
*   **Postcondición:** Transición de estado en `order-fsm` registrada inmutablemente.

### **TTR-003: Orquestación de Vigilancia de Emergencia (System Watchdog)**
*   **Descripción:** Mantiene el [`system-watchdog`](../features/system-watchdog.md) activo en un loop o daemon paralelo de alta prioridad escrito en **Rust** (Tokio).
*   **Reglas de Orquestación:**
    * El "Kill-Switch" tiene soberanía absoluta para ejecutar `FlattenAll()` cerrando todas las posiciones e ignorando el FSM de estrategia.
    * Orquesta la ejecución paralela inerte (Shadow Mode) sin volumen expuesto para auditoría de drift.
    * Enlaza la señal de pánico externa recibida de la **Emergency PWA** para ejecutar un barrido manual remoto seguro.
    * Toda acción de emergencia debe inyectar la huella digital del hardware (`node_id`) en el log forense (ADR-0020).
*   **Entrada:** `system_health_metrics`, `pwa_panic_signal`.
*   **Salida:** `kill_switch_signal` (opcional).
*   **Precondición:** Sesión de trading en vivo o shadow mode iniciada.
*   **Postcondición:** Bloqueo físico de la terminal ante eventos catastróficos.

### **TTR-004: Orquestación de Telemetría Institutional**
*   **Descripción:** Invoca a [`telemetry`](../features/telemetry.md) para registrar la salud del motor de ejecución.
*   **Reglas de Orquestación:**
    * Debe emitir latencias de red y uso de CPU por cada fill recibido.
    * Los datos de telemetría deben vincularse a la `session_id` activa (ADR-0020).
*   **Entrada:** `system_status`, `fill_latencies`.
*   **Salida:** `telemetry_stream`.
*   **Precondición:** Motor de ejecución activo.
*   **Postcondición:** Datos disponibles para el Dashboard de monitoreo en tiempo real.

### **TTR-005: Orquestación de Persistencia Crítica e Inmunidad (Event Store)**
*   **Descripción:** Coordina la persistencia de cada evento de ejecución en el [`audit-event-store`](../features/audit-event-store.md).
*   **Reglas de Orquestación:**
    * Cada cambio de estado atómico de la FSM debe disparar la escritura inmutable en SQLite WAL.
    * En el arranque de sesión, orquesta la reconstrucción del inventario leyendo el histórico de eventos del store.
*   **Entrada:** `FSM_State_Change`, `Initialization_Trigger`.
*   **Salida:** `Inmutable_Event_Record`, `Mem_Inventary_Reconstructed`.
*   **Precondición:** SQLite WAL habilitado y audit_chain_hash validado.
*   **Postcondición:** Inventario sincronizado 1:1 con el estado real del broker.

### **TTR-006: Orquestación del Bridge de Ejecución (Nautilus Integration)**
*   **Descripción:** Utiliza [`nautilus-integration`](../features/nautilus-integration.md) para conectar el motor de ejecución con la infraestructura de baja latencia.
*   **Reglas de Orquestación:**
    * El bridge debe garantizar que el estado interno coincida 1:1 con el ledger del broker.
    * Todas las latencias de tránsito se registran en el campo `execution_latency_ms` (ADR-0020).
*   **Entrada:** `execution_engine_commands`.
*   **Salida:** `low_latency_execution_status`.
*   **Precondición:** TTR-002 y TTR-005 finalizados.
*   **Postcondición:** Canal de ejecución ultra-rápido verificado y operativo.

> **TTR-007 / TTR-008 / TTR-009 / TTR-010:** Retirados — numeración reservada, alcance consolidado en TTRs adyacentes durante el refinamiento del backlog del módulo.

### **TTR-011: Orquestación de Guardia de Microestructura (Pre-Trade Order Flow)**
*   **Descripción:** Invoca a [`order-flow-microstructure`](../features/order-flow-microstructure.md) (parte en vivo OFI/DOM L2 del split ADR-0118) segundos antes del disparo táctico.
*   **Reglas de Orquestación:**
    * Veta la orden si la absorción institucional (CVD) contradice la dirección de la señal.
    * Registra el `event_sequence_id` del último tick procesado (ADR-0020).
*   **Entrada:** `proposed_trade`, `market_depth_snapshot`.
*   **Salida:** `microstructure_clearance` (OK | REJECT).
*   **Precondición:** TTR-001 (Tactical Check) en curso.
*   **Postcondición:** Transición a despacho de orden al broker.

> **TTR-012:** Retirado — numeración reservada, alcance consolidado en TTRs adyacentes durante el refinamiento del backlog del módulo.

### **TTR-013: Orquestación de Dimensionamiento Táctico (Live Sizing con Robustness Score)**
*   **Descripción:** Invoca a [`precision-sizing-models`](../features/precision-sizing-models.md) inmediatamente después del disparo de señal, utilizando el score de robustez de [`robustness-score-aggregator`](../features/robustness-score-aggregator.md) como parámetro multiplicador de riesgo (ADR-0058).
*   **Reglas de Orquestación:**
    - Calcula el número de contratos/acciones basándose en la equidad actual y el riesgo configurado.
    - Aplica el score de robustez como multiplicador: mayor score → mayor fracción de riesgo asignable.
    - Asegura paridad bit-a-bit con el modo Backtest.
*   **Entrada:** `metamorphic_fire_event`, `live_account_snapshot`, `final_robustness_score`.
*   **Salida:** `computed_order_quantity`.
*   **Precondición:** TTR-012 (Detonación) aprobado.
*   **Postcondición:** Inyección del tamaño dimensionado por score de robustez en la orden final del broker.

### **TTR-014: Orquestación de Estados Atómicos (Order FSM)**
*   **Descripción:** Invoca a [`order-fsm`](../features/order-fsm.md) para garantizar la inmutabilidad de la máquina de estados.
*   **Reglas de Orquestación:**
    *   Ningún estado puede saltarse; requiere firma de transición.
*   **Entrada:** `raw_order_event`.
*   **Salida:** `fsm_locked_state`.
*   **Precondición:** Recepción de señal válida.
*   **Postcondición:** Transición registrada.

### **TTR-015: Orquestación de Deslizamiento (Slippage Models)**
*   **Descripción:** Invoca a [`slippage-models`](../features/slippage-models.md) para modelar spread asimétrico en la orden.
*   **Reglas de Orquestación:**
    *   Calcula coste de impacto antes del disparo metamórfico.
*   **Entrada:** `order_size`, `order_book_state`.
*   **Salida:** `expected_slippage_cost`.
*   **Precondición:** TTR-011 finalizado.
*   **Postcondición:** Spread inyectado en el coste de transacción.

### **TTR-016: Orquestación de Rastreo (Equity Curve Tracker)**
*   **Descripción:** Invoca a [`equity-curve-tracker`](../features/equity-curve-tracker.md) para medir PnL en caliente.
*   **Reglas de Orquestación:**
    *   Sincroniza el balance con el ledger del broker cada 1s.
*   **Entrada:** `broker_balance_feed`.
*   **Salida:** `live_equity_tick`.
*   **Precondición:** Conexión a broker viva.
*   **Postcondición:** Capital reportado al módulo Manage.

### **TTR-017: Orquestación de Adaptación (HMM Regime)**
*   **Descripción:** Invoca a [`hmm-regime-detection`](../features/hmm-regime-detection.md) para vetar operativa si el régimen invierte.
*   **Reglas de Orquestación:**
    *   Filtro en frío: Si mercado choca con la estrategia, aborta.
*   **Entrada:** `live_bars`.
*   **Salida:** `regime_clearance`.
*   **Precondición:** Flujo de datos live activo.
*   **Postcondición:** Validación HMM aprobada.

### **TTR-018: Orquestación de Aislamiento (Executable Container)**
*   **Descripción:** Delega ejecución a [`executable-container`](../features/executable-container.md).
*   **Reglas de Orquestación:**
    *   Mantiene la estrategia corriendo en un hilo aislado (Rust/C++).
*   **Entrada:** `strategy_ast`.
*   **Salida:** `container_health_status`.
*   **Precondición:** Estrategia fondeada.
*   **Postcondición:** Runner protegido contra fallos de otros módulos.

### **TTR-019: Orquestación de Alertas (Notification)**
*   **Descripción:** Invoca a [`notification`](../features/notification.md) en eventos críticos (fills, pánicos).
*   **Reglas de Orquestación:**
    *   Debe despachar via WebSockets y Webhooks.
*   **Entrada:** `critical_event`.
*   **Salida:** `dispatched_alert`.
*   **Precondición:** FSM emite transición.
*   **Postcondición:** Cliente notificado.

### **TTR-020: Orquestación Asíncrona (Async Job Executor)**
*   **Descripción:** Invoca a [`async-job-executor`](../features/async-job-executor.md) para IO no bloqueante.
*   **Reglas de Orquestación:**
    *   Persistencia de logs y métricas fuera del hilo principal.
*   **Entrada:** `telemetry_payload`.
*   **Salida:** `background_job_id`.
*   **Precondición:** Telemetría generada.
*   **Postcondición:** I/O liberado del hot-path.

### **TTR-021: Orquestación de Afinidad (Production Optimization)**
*   **Descripción:** Invoca a [`production-optimization`](../features/production-optimization.md) para CPU Pinning.
*   **Reglas de Orquestación:**
    *   Aísla el hilo de ejecución en núcleos exclusivos del CPU.
*   **Entrada:** `process_id`.
*   **Salida:** `cpu_affinity_status`.
*   **Precondición:** Arranque del módulo.
*   **Postcondición:** Cero context-switching en Hot-Path.

### **TTR-022: Orquestación Forense (Audit Log)**
*   **Descripción:** Invoca a [`audit-log`](../features/audit-log.md) para firmar transacciones con SHA-256.
*   **Reglas de Orquestación:**
    *   Cadena de bloques local para la secuencia de órdenes.
*   **Entrada:** `trade_fill`.
*   **Salida:** `audit_hash`.
*   **Precondición:** TTR-002 completo.
*   **Postcondición:** Integridad criptográfica.

### **TTR-023: Orquestación Temporal (Clock)**
*   **Descripción:** Invoca a [`clock`](../features/clock.md) para sellado de tiempo de alta fidelidad.
*   **Reglas de Orquestación:**
    *   Uso de nanosegundos; prohíbe `datetime.now()`.
*   **Entrada:** `system_event`.
*   **Salida:** `deterministic_timestamp`.
*   **Precondición:** Evento generado.
*   **Postcondición:** Tiempo sincronizado.

### **TTR-024: Orquestación de Vigilancia Pardo y SSL (Operational Safety)**
*   **Descripción:** Invoca a [`operational-safety-monitor`](../features/operational-safety-monitor.md) para monitorear el drift métrico y límites de DD.
*   **Reglas de Orquestación:**
    *   Compara cada fill recibido con el perfil histórico (Pardo Profile).
    *   Si detecta drift > 50% o DD > `SSL_Limit`, emite una señal de liquidación inmediata (ADR-0070).
*   **Entrada:** `live_trade_fills`, `historical_profile_data`.
*   **Salida:** `safety_verdict`, `kill_signal` (opcional).
*   **Precondición:** Estrategia en ejecución activa.
*   **Postcondición:** Protección del capital ante fallo de modelo o mercado.

### **TTR-025: Orquestación de Escalado de Volatilidad (Target Vol)**
*   **Descripción:** Invoca a [`volatility-stabilization`](../features/volatility-stabilization.md) para ajustar el lotaje según la volatilidad realizada.
*   **Reglas de Orquestación:**
    *   Recalcula el lotaje dinámicamente antes de cada orden para mantener el Target Vol (ADR-0068).
    *   Veta la apertura de nuevas posiciones si el régimen supera el `VOL_MAX_LIMIT`.
*   **Entrada:** `proposed_order`, `realized_vol_stream`.
*   **Salida:** `vol_adjusted_order_size`.
*   **Precondición:** TTR-001 (Pre-Trade Validator) en curso.
*   **Postcondición:** Exposición de riesgo normalizada al régimen de mercado.

### **TTR-026: Orquestación de Bridge Multiplataforma (Multiplatform Execution Bridge)**
*   **Descripción:** Invoca a [`multiplatform-execution-bridge`](../features/multiplatform-execution-bridge.md) para enviar comandos JSON de compra/venta vía WebSockets/gRPC API nativos hacia terminales externas sin exportación de lógica local.
*   **Reglas de Orquestación:**
    *   La transmisión debe completarse en < 1ms para mantener la velocidad en el hot path.
*   **Entrada:** `proposed_order`.
*   **Salida:** `broker_fill_event`.
*   **Precondición:** TTR-001 (Pre-Trade Validator) aprobado.
*   **Postcondición:** Registro de la orden enviada a la terminal externa.

### **TTR-027: Orquestación del Gestor Multi-Ticket (Multi-Ticket Manager)**
*   **Descripción:** Invoca a [`multi-ticket-manager`](../features/multi-ticket-manager.md) para gestionar múltiples tickets individuales por estrategia concurrentes.
*   **Reglas de Orquestación:**
    *   Verifica la unicidad de señales mediante el `signal_hash` + `timestamp` antes de disparar un nuevo ticket.
*   **Entrada:** `new_signal`.
*   **Salida:** `ticket_execution_status`.
*   **Precondición:** TTR-026 (Execution Bridge) aprobado.
*   **Postcondición:** Ticket individual en estado OPERATING vinculado al genoma.

### **TTR-028: Orquestación de Cola Anti-Throttling (Order-Priority Queue)**
*   **Descripción:** Invoca a [`order-priority-queue`](../features/order-priority-queue.md) para gestionar el despacho de órdenes en condiciones de estrangulamiento de tasa por parte del broker.
*   **Reglas de Orquestación:**
    - Las órdenes Stop Loss P0 omiten la cola y se despachan de forma inmediata.
    - Se registran la latencia y los reintentos mediante `execution_latency_ms`.
*   **Entrada:** `proposed_orders`.
*   **Salida:** `order_transmission_clearance`.
*   **Precondición:** TTR-001 (Pre-Trade Validator) aprobado.
*   **Postcondición:** Transmisión exitosa al broker.

### **TTR-029: Orquestación de Advanced Trade Management (ATM)**
*   **Descripción:** Invoca a [`advanced-trade-management`](../features/advanced-trade-management.md) para ejecutar reglas avanzadas de Grid, Hedging y Trailing Stop Mecánico.
*   **Reglas de Orquestación:**
    - Modifica dinámicamente los niveles de Stop Loss barra-a-barra según el movimiento del precio a favor.
*   **Entrada:** `active_positions`, `current_bars`.
*   **Salida:** `order_modifications`.
*   **Precondición:** Posición en estado OPERATING.
*   **Postcondición:** Actualización de los parámetros de salida de la orden.

### **TTR-030: Orquestación de Micro-Gestión Cinética**
*   **Descripción:** Invoca a [`kinetic-micro-management`](../features/kinetic-micro-management.md) para aplicar las lógicas defensivas de Scale Out mandatorio, Z-Score Trailing y Tapering Logarítmico.
*   **Reglas de Orquestación:**
    - Monitorea el PnL intradiario en tiempo real.
    - Cierra el 50% de la posición al alcanzar +1.0R y mueve el stop a BreakEven.
*   **Entrada:** `live_pnl_stream`, `active_positions`.
*   **Salida:** `kinetic_adjustment_commands`.
*   **Precondición:** Posición abierta.
*   **Postcondición:** Cierre parcial o total de la posición según reglas estadísticas.

### **TTR-031: Orquestación del Autopilot Metrics Provider**
*   **Descripción:** Invoca a [`autopilot-metrics-provider`](../features/autopilot-metrics-provider.md) para exponer los KPIs del Autopilot hacia la interfaz de usuario en tiempo real.
*   **Reglas de Orquestación:**
    - Genera las métricas en un diccionario plano con los 8 campos requeridos.
*   **Entrada:** `internal_execution_state`.
*   **Salida:** `autopilot_metrics_dict`.
*   **Precondición:** Autopilot activo.
*   **Postcondición:** Dashboard sincronizado.

### **TTR-032: Orquestación de Daemons Persistentes (LiveNode Aisle)**
*   **Descripción:** Orquesta el ciclo de vida del hilo de ejecución persistente mediante [`persistent-daemons`](../features/persistent-daemons.md).
*   **Reglas de Orquestación:**
    - Asegura la afinidad de CPU (*Core Pinning*) antes de inicializar NautilusTrader.
    - Monitorea el latido (heartbeat) del daemon y coordina reinicios automáticos si se pierde la persistencia.
*   **Entrada:** `core_id_config`.
*   **Salida:** `live_daemon_status`.
*   **Precondición:** Configuración de hardware validada.
*   **Postcondición:** LiveNode operando en núcleo aislado.

### **TTR-033: Orquestación de Multiplexación (Data Bus)**
*   **Descripción:** Gestiona la suscripción masiva de agentes al bus de datos mediante [`data-bus-pubsub`](../features/data-bus-pubsub.md).
*   **Reglas de Orquestación:**
    - Garantiza que solo exista un DataClient físico por símbolo.
    - Distribuye los eventos de mercado a los agentes por referencia (Zero-Copy) para minimizar la latencia.
*   **Entrada:** `symbol_subscription_request`.
*   **Salida:** `data_bus_receiver`.
*   **Precondición:** Conectividad de red activa.
*   **Postcondición:** Agente recibiendo datos de mercado multiplexados.

### **TTR-034: Orquestación del Protocolo de Recuperación (Crash Recovery)**
*   **Descripción:** Invoca a [`crash-recovery`](../features/crash-recovery.md) en el arranque del módulo para reconciliar la base de datos local SQLite con las posiciones reales del broker.
*   **Reglas de Orquestación:**
    - Bloquea la emisión de señales operativas en caliente si el estado anterior no finalizó limpiamente.
    - Sincroniza y re-alinea parámetros cinéticos (Trailing Stops) en el broker antes de volver al estado `ONLINE`.
    - Garantiza un tiempo de recovery total `<= 10 segundos`.
*   **Entrada:** `session_initialization_trigger`.
*   **Salida:** `recovery_clearance` (ONLINE | EMERGENCY_LOCK).
*   **Precondición:** SQLite e hilos persistentes de la base de datos activos.
*   **Postcondición:** Inventario e indicadores sincronizados al 100% con el broker.

### **TTR-035: Orquestación del Auditor de Rendimiento (Real-Time Auditor)**
*   **Descripción:** Coordina la ejecución asíncrona de los tests de deriva estadística (WRC / KS Test) de [`operational-safety-monitor`](../features/operational-safety-monitor.md).
*   **Reglas de Orquestación:**
    - Delega el cómputo de WRC y KS Test a hilos o workers paralelos asíncronos para proteger el hot-path.
    - Emite alertas visuales y telemetría de drift si las métricas en vivo divergen del perfil histórico.
*   **Entrada:** `live_performance_feed`, `historical_profile`.
*   **Salida:** `drift_warning_level`.
*   **Precondición:** Suficientes operaciones ejecutadas en la sesión.
*   **Postcondición:** Monitoreo estadístico activo sin impacto en latencia de despacho.

### **TTR-036: Orquestación de Despacho y Aislamiento Federado (Federated Execution)**
*   **Descripción:** Orquesta la validación táctica y el enrutamiento lógico aislado de órdenes federadas consumiendo [`federated-portfolio`](../features/federated-portfolio.md) y [`pre-trade-validator`](../features/pre-trade-validator.md).
*   **Reglas de Orquestación:**
    - Intercepta las intenciones de orden del `LiveNode` e inyecta la etiqueta e identidad del contenedor de origen con latencia < 1ms.
    - Valida de forma aislada que el subportafolio posea la asignación de capital y margen suficiente antes de transferir la orden al broker.
*   **Entrada:** `proposed_order`, `portfolio_containers_state`.
*   **Salida:** `labeled_order`, `execution_clearance` (APPROVED | BLOCKED).
*   **Precondición:** TTR-001 (Pre-Trade Validator) and TTR-030 (Federated Portfolios orquestado en `manage`) inicializados.
*   **Postcondición:** Orden ruteada con metadatos de linaje inmutables registrados en el almacén de eventos.

### **TTR-037: Orquestación de Copy-Trading (Signal Relay & Risk Scaling)**
*   **Descripción:** Orquesta el flujo de copia de señales y la replicación local de órdenes consumiendo [`copy-trading-engine`](../features/copy-trading-engine.md) y [`pre-trade-validator`](../features/pre-trade-validator.md).
*   **Reglas de Orquestación:**
    - Para la instancia Master: Tras la confirmación de llenado de orden (`fill_event`), serializa, cifra y firma la señal para enviarla al Signal Relay en menos de 1ms.
    - Para la instancia Copier: Tras recibir una señal cifrada, valida la integridad y el timestamp, ejecuta el cálculo de Risk Scaling local, y somete la orden resultante al `pre-trade-validator` antes del envío final.
    - Si la latencia total medida es superior a 500ms o el broker del copier pierde conexión, suspende inmediatamente el proceso de copia.
*   **Entrada:** `fill_event` (para Master), `encrypted_signal` (para Copier).
*   **Salida:** `dispatched_signal` (para Master), `scaled_local_order` (para Copier).
*   **Precondición:** TTR-002 (para Master) o conexión de red activa hacia el Signal Relay (para Copier).
*   **Postcondición:** Emisión exitosa de la señal o ejecución local atómica registrada en el ledger de auditoría local.

### **TTR-038: Orquestación de Seguridad Soberana (Sovereign Security)**
*   **Descripción:** Integra la validación criptográfica de credenciales del bróker, el registro inmutable encadenado por hash y el bloqueo de telemetría consumiendo [`sovereign-security`](../features/sovereign-security.md).
*   **Reglas de Orquestación:**
    - Antes de inicializar la conexión con el bróker, lee y desencripta en memoria las credenciales guardadas usando `DRASUS_MASTER_KEY`.
    - Cada acción de ejecución completada debe registrarse en la tabla de auditoría calculando el hash secuencial `audit_chain_hash`.
    - El orquestador del módulo debe asegurar que no se carguen módulos o dependencias de red externas de telemetría en tiempo de ejecución.
*   **Entrada:** `DRASUS_MASTER_KEY` (variable de entorno), `broker_credentials`, `execution_records`.
*   **Salida:** `decrypted_credentials` (en memoria), `secured_audit_records`.
*   **Precondición:** Variable de entorno inyectada.
*   **Postcondición:** Credenciales y registros de auditoría protegidos bajo arquitectura Local-First.

### **TTR-039: Orquestación del Monitor de Latencia de Broker (Throttling Metrics Dashboard)**
*   **Descripción:** Invoca a [`throttling-metrics-dashboard`](../features/throttling-metrics-dashboard.md) para monitorear el RTT de red y datos de mercado.
*   **Reglas de Orquestación:**
    - Recolecta la latencia y ocupación de la cola de prioridades en caliente.
    - Despacha streams de telemetría a la interfaz gráfica cada 100ms.
*   **Entrada:** `latency_measurements_raw`.
*   **Salida:** `throttling_metrics_stream`.
*   **Precondición:** Conector de broker y cola de prioridad activos.
*   **Postcondición:** Visualización en tiempo real en la UI.

### **TTR-040: Orquestación de Auto-Auditoría de Portafolios Vivos**
*   **Descripción:** Integra el monitoreo activo llamando a [`auto-auditoria-portafolios-vivos`](../features/auto-auditoria-portafolios-vivos.md) para recalcular la expectativa matemática en vivo basada en spreads, comisiones y swaps del broker.
*   **Reglas de Orquestación:**
    - Se invoca periódicamente según la configuración temporal.
    - Si la expectativa cae por debajo del umbral mínimo, el orquestador aplica una pausa operativa preventiva (Hard Limit) y registra el veredicto en la base de datos de auditoría.
*   **Entrada:** `live_account_snapshot`, `broker_cost_feed`.
*   **Salida:** `expectancy_status` (NORMAL | DEGRADED).
*   **Precondición:** Conector al bróker inicializado y transmitiendo cotizaciones.
*   **Postcondición:** Alerta generada o pausa de órdenes del portafolio aplicada.

### **TTR-041: Orquestación de Genes de Acción del Genoma de Riesgo y Gestión de Posición (ADR-0108/ADR-0109)**
*   **Descripción:** Cuando `ACTIVE_GENOME_DOMAINS` incluye Riesgo y Gestión de Posición, conecta los Genes de Acción resueltos por la evolución (mutación de sizing, morfología de salida) con su materialización en el hot-path de ejecución.
*   **Reglas de Orquestación:**
    *   La Primitiva de Acción de Mutación de Sizing (`Risk_Percent_Equated`, `Fixed_Monetary_Risk`, `Kelly_Sizing_Capped`, `Multiplier`) resuelta por el genoma sustituye al `SIZING_MODE` estático en TTR-013, vía [`precision-sizing-models`](../features/precision-sizing-models.md) (TTR-006).
    *   La Primitiva de Acción de Morfología de Salida `Split_Position(N_Fases)` se materializa abriendo N tickets con `phase_id` (0..N-1) vía [`multi-ticket-manager`](../features/multi-ticket-manager.md) (TTR-003), cada uno gestionado independientemente por TTR-027.
    *   Las Primitivas `Scale_Out_Trigger`, `Move_SL_To_Target` y `Time_Decay_Exit` se ejecutan vía [`advanced-trade-management`](../features/advanced-trade-management.md) (TTR-003) y [`kinetic-micro-management`](../features/kinetic-micro-management.md) (TTR-003), reemplazando dinámicamente sus disparadores FIJOS cuando el genoma está activo.
    *   Cuando el genoma no está activo, TTR-013/TTR-027/TTR-029/TTR-030 operan exactamente como hoy (sin regresión).
*   **Entrada:** `risk_genome_action_genes`, `ACTIVE_GENOME_DOMAINS`.
*   **Salida:** `mutated_sizing_directive`, `split_position_tickets`, `exit_morphology_commands`.
*   **Precondición:** Señal de entrada validada (TTR-001) y Genoma de Riesgo y Gestión de Posición resuelto por `generate` (TTR-040 de `generate`).
*   **Postcondición:** Posición abierta y gestionada bajo los Genes de Acción del genoma activo, con `audit_hash` por fase/mutación.

### **TTR-042: Orquestación de Acceso Agéntico vía MCP (Cabina Dual — Bloqueado por Defecto)**
*   **Descripción:** Invoca a [`agentic-mcp-gateway`](../features/agentic-mcp-gateway.md) para evaluar el permiso antes de aceptar una llamada proveniente del canal MCP sobre la `public_interface` de este módulo.
*   **Reglas de Orquestación:**
    * `execute` opera exclusivamente con capital y broker reales: ninguna llamada del canal MCP nace con permiso, sin excepción (ADR-0123).
    * Toda llamada se rechaza salvo que `PRODUCTION_OVERRIDE` esté activo en ese momento; el rechazo y la concesión quedan ambos auditados con su procedencia agente (`agent_session_id`).
*   **Entrada:** Llamada MCP entrante con pipeline `execute`, estado de `PRODUCTION_OVERRIDE`.
*   **Salida:** Resultado de la operación enrutado al agente, o rechazo, + registro de auditoría de procedencia.

### **TTR-043: Integración del Indicador Fundamental en Tiempo Real (Fundamental Indicator Projector)**
*   **Descripción:** TTR de Integración (ADR-0118): lee en vivo el indicador fundamental ya construido ([`fundamental-indicator-projector`](../features/fundamental-indicator-projector.md)) vía su `public_interface`, para confirmar o ponderar la decisión de ejecución, sin ejecutar lógica fundamental en el hot-path.
*   **Reglas de Orquestación:**
    * El hot-path solo lee una serie numérica precalculada; cero lógica fundamental en la ruta de validación ≤1ms (ADR-0125/0128).
    * El valor leído respeta el contrato estándar de indicador; se usa la serie normalizada del activo operado (ADR-0128).
    * La lectura no puede bloquear la cascada de validación pre-trade.
*   **Entrada:** `fundamental_indicator_series` del activo operado.
*   **Salida:** Confirmación/ponderación fundamental aplicada a la decisión viva.
*   **Precondición:** Indicador disponible vía `public_interface` de `generate` (TTR-044).
*   **Postcondición:** Decisión de ejecución enriquecida con el aporte fundamental, sin penalizar latencia.

### **TTR-044: Orquestación de Entradas Concurrentes No Bloqueantes (ADR-0129)**
*   **Descripción:** Invoca a [`order-fsm`](../features/order-fsm.md) y [`advanced-trade-management`](../features/advanced-trade-management.md) para abrir una posición concurrente independiente cuando llega una señal de entrada válida y ya existe una posición abierta de la misma estrategia, aplicando la de-duplicación de señal y el gate de riesgo.
*   **Reglas de Orquestación:**
    * Default no bloqueante (`MAX_CONCURRENT_POSITIONS`); `=1` reproduce el bloqueo clásico (ADR-0129).
    * `SIGNAL_DEDUP_BARS` descarta una entrada en la misma vela de disparo.
    * Cada apertura concurrente pasa completa por el Pre-Trade Risk Gate de 10 pasos (ADR-0025), validando margen/exposición sobre el **agregado** de posiciones (HARD, ADR-0010).
    * Toda apertura o descarte queda auditado con su `source_signal_id`.
*   **Entrada:** Nueva señal de entrada, conjunto de posiciones abiertas de la estrategia.
*   **Salida:** Orden de apertura concurrente enviada al broker, o descarte con motivo.
*   **Precondición:** Gate de riesgo pre-trade disponible.
*   **Postcondición:** Posición concurrente registrada en el FSM con margen agregado y P&L por ticket.

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundamentos (ADR-0020):** El catálogo de los 25 campos maestros está en la sección "Épica 0: Esqueleto Fundacional" de este documento (referencia, no esquema). Toda entidad persistida por este módulo incluye el Grupo I de forma universal; los Grupos II–V se aplican solo en los campos que el Perfil Técnico de cada feature exige (Filtro de Relevancia, ADR-0020) — nunca el catálogo completo.

- **Decisión Arquitectónica Asociada:**
    - ADR-0004: FSM para atomicidad de estados.
    - ADR-0013: Stack Tecnológico (NautilusTrader).
    - ADR-0020: Inundación de Fundaciones.
    - ADR-0108 / ADR-0109: Genoma de Riesgo y Gestión de Posición (TTR-041).

---

## Dependencias
**Depende de:**
- [`manage`](../modules/manage.md) — para la recepción de límites y pesos soberanos.
- [`pre-trade-validator`](../features/pre-trade-validator.md) — para la seguridad táctica.

**Consumido por:**
- [`withdraw`](../modules/withdraw.md) — para la monitorización de degradación en tiempo real.
- [`feedback`](../modules/feedback.md) — para la autopsia de ejecuciones y latencias.
---

## Operación Paralela

Este módulo corre en paralelo con **withdraw**:
- Execute: maneja órdenes, ejecuciones, fills reales.
- Withdraw: monitorea continuamente si hay degradación.

**Kill Switch Activado:**
- Watchdog detecta DD crítico
- Cierra todas las posiciones en < 100ms
- Cancela órdenes pendientes
- Genera alerta y notificación
- Persiste evento de Kill Switch en el `audit-event-store`
