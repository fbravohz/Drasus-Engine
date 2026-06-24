# Retiro

**Carpeta:** `./modules/withdraw/`
**Estado:** Orquestador (Imperative Shell)
**Última actualización:** 2026-04-12

---

## ¿Qué es?

El módulo de retiro es el portal de **Retiro y Archivo Institucional** de Drasus Engine. Su función es gestionar la transición digna de las estrategias que el Guardián (módulo de feedback) o el usuario han decidido retirar tras culminar su servicio.

El retiro no es un acto de monitoreo (eso lo hace Feedback), sino un acto de **Gestión de Transición** y preservación de datos. Asegura que la estrategia deje de consumir recursos, cierra sus hilos en el FSM y archiva sus métricas para la posteridad.

---

## Épica 0: Esqueleto Fundacional

### Estructura de Archivos (FCIS — ADR-0003)

```
crates/withdraw/
├── public_interface.rs   # Frontera pública: único punto de entrada para otros módulos
├── logic.rs              # Lógica pura: detección de degradación, veredicto de retiro (sin DB, sin I/O)
├── orchestrator.rs       # Coordinación: invoca Performance Monitor, Regime Guard, Order FSM
├── persistence.rs        # Acceso a SQLite WAL (lectura/escritura)
├── schemas.rs            # Definición de tablas: retirement_records, terminal_snapshots
└── types.rs              # Tipos de entrada/salida: DegradationAlert, RetirementReason, ArchivalConfirmation
```

### Vocabulario de Persistencia — Catálogo de 25 Campos (ADR-0020 V2)

Esta tabla es el **catálogo de referencia completo** del Contrato Global de ADR-0020 V2 (vocabulario lógico, no esquema literal). La migración 0001 crea la tabla ancla `foundation_master_fields` con estas 25 columnas como referencia ÚNICA del sistema — este módulo NO la replica.

Las tablas propias de este módulo (una por feature/TTR, en sus propias migraciones) llevan: el **Grupo I (Identidad & Integridad, 6 primeras filas) de forma universal y obligatoria**, más solo los campos concretos de los Grupos II–V que correspondan al **Perfil Técnico** de cada feature (Filtro de Relevancia, tabla canónica en ADR-0020 V2) — nunca el catálogo completo. Cada feature documenta su selección en su propia sección "Contrato de Persistencia" (`features/*.md`).

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
| | `logic_hash` | Hash del motor de retiro |
| | `data_snapshot_id` | Snapshot PIT del mercado |
| | `transformation_id` | ID del paso/tipo de transformación aplicado |
| **IV. Infraestructura y Ops** | `process_id` | PID del servicio de auditoría |
| | `session_id` | Agrupación de runtime |
| | `node_id` | ID del hardware físico |
| **V. Forense y Ejecución** | `portfolio_container_id` | Contenedor de portafolio |
| | `compliance_status_id` | Veredicto de supervivencia (ReasonCode) |
| | `risk_audit_id` | Ticket detallado de riesgo |
| | `indicator_state_hash` | Snapshot técnico |
| | `execution_latency_ms` | Latencia de procesamiento |
| | `source_signal_id` | Link a señal origen |
| | `signature_hash` | HMAC de señales |

### TTRs Etiquetados por Fase

| TTR | Fase | Descripción corta |
|---|---|---|
| TTR-001 | **EPIC-7** | Monitoreo de supervivencia (Performance Monitor) |
| TTR-002 | **EPIC-7** | Cierre de ciclo de vida (Order FSM) |
| TTR-003 | **EPIC-7** | Vigilancia de coherencia (Regime Guard) |
| TTR-004 | **EPIC-7** | Rastreo final (Equity Curve Tracker) |
| TTR-005 | **EPIC-7** | KPIs de decadencia (Institutional Metrics) |
| TTR-006 | **EPIC-7** | Detección macro (HMM Regime Detection) |
| TTR-007 | **EPIC-7** | Límites soberanos (Portfolio Rules) |
| TTR-008 | **EPIC-7** | Alerta terminal (Notification) |
| TTR-009 | **EPIC-7** | Archivo inmutable (Audit Log) |
| TTR-010 | **EPIC-7** | Acceso agéntico MCP (Cabina Dual — bloqueado por defecto) |
| TTR-999 | **EPIC-7** | Protocolo Fail-Fast Safe (ADR-0066) |

---

## Comportamientos Observables (Orquestación)

- [ ] **Vigilancia de Perfil:** Coordina las alertas de [performance-monitor](../features/performance-monitor.md) para detectar drift.
- [ ] **Vigilancia de Mercado:** Invoca a [regime-guard](../features/regime-guard.md) para asegurar coherencia estratégica.
- [ ] **Gestión de Retiro:** Gestiona la ventana de veto y la transición final de la estrategia en [order-fsm](../features/order-fsm.md) desde OPERANDO a RETIRADA.

---

## Restricciones

- NUNCA una estrategia puede pasar directamente de OPERANDO a RETIRADA sin pasar por PAUSADA primero
- La ventana de veto tiene un mínimo configurable (no puede ser instantánea) y un máximo configurable (no puede ser infinita)
- El monitoreo no puede interferir con la ejecución de órdenes (corre en paralelo sin bloquear)

---

## Parámetros Configurables

| Parámetro | Default | Qué hace |
|---|---|---|
| MAX_SHARPE_DRIFT | configurable | Cuánto puede caer el Sharpe vs baseline antes de considerar degradación (ej: -30%) |
| MAX_DD_MULTIPLIER | configurable | Cuánto puede crecer el DrawDown vs baseline (ej: 1.5x) |
| VETO_WINDOW | configurable | Cuánto tiempo tiene el usuario para decidir (ej: 1 día) |
| MONITORING_INTERVAL | configurable | Cada cuánto tiempo se verifican las métricas |

---

## Features Consumidas (Reutilizables)

> *(ADR-0137)* Este módulo es la **composición preset canónica** de estas features — define el cableado por defecto. En el Canvas [Forge/Reactor], las features pueden conectarse directamente sin que este módulo sea intermediario obligatorio en runtime.

- **[`performance-monitor`](../features/performance-monitor.md)** — Detección proactiva de degradación estadística.
- **[`regime-guard`](../features/regime-guard.md)** — Guardian de coherencia entre estrategia y mercado.
- **[`equity-curve-tracker`](../features/equity-curve-tracker.md)** — Monitoreo de capital y PnL en background.
- **[`institutional-metrics`](../features/institutional-metrics.md)** — Cálculo de drift y metrics de supervivencia.
- **[`hmm-regime-detection`](../features/hmm-regime-detection.md)** — Detección de cambios estructurales de mercado.
- **[`portfolio-rules`](../features/portfolio-rules.md)** — Reglas soberanas para pausa y retiro.
- **[`order-fsm`](../features/order-fsm.md)** — Transiciones de estado bloqueadas (PAUSED).
- **[`notification`](../features/notification.md)** — Alertas de degración y ventana de veto.
- **[`audit-log`](../features/audit-log.md)** — Registro inmutable de la causa de retiro.

---

---

## Tareas (TTRs) — Protocolo de Orquestación (§8.1)

### **TTR-001: Orquestación de Monitoreo de Supervivencia (Performance Monitor)**
*   **Descripción:** Invoca a [`performance-monitor`](../features/performance-monitor.md) para detectar anomalías estadísticas en tiempo real.
*   **Reglas de Orquestación:**
    * Si el Sharpe real cae un 30% bajo el baseline, la estrategia se marca automáticamente como `NEEDS_REVIEW`.
    * Cada alerta generada debe incluir el `process_id` del monitor de riesgo (ADR-0020 V2).
*   **Entrada:** `live_performance_feed`, `strategy_baseline`.
*   **Salida:** `degradation_alert`.
*   **Precondición:** Estrategia en estado `OPERATING`.
*   **Postcondición:** Registro de la alerta en `retirement_records` con `institutional_tag`.

### **TTR-002: Orquestación de Cierre de Ciclo de Vida (Order FSM)**
*   **Descripción:** Gestiona la transición final de la estrategia en [`order-fsm`](../features/order-fsm.md) de `OPERATING` a `RETIRED`.
*   **Reglas de Orquestación:**
    * No se permite el retiro sin un `ReasonCode` válido (Performance | Regime | User).
    * El registro de retiro debe persistir el `audit_hash` final del PnL acumulado (ADR-0020 V2).
*   **Entrada:** `decommission_order`, `retirement_reason`.
*   **Salida:** `archival_confirmation`.
*   **Precondición:** Todas las posiciones de la estrategia cerradas en el broker.
*   **Postcondición:** Marcado inmutable del `version_node_id` como `RETIRED` en el DAG.

### **TTR-003: Orquestación de Vigilancia de Coherencia (Regime Guard)**
*   **Descripción:** Invoca a [`regime-guard`](../features/regime-guard.md) para verificar si el entorno sigue siendo apto.
*   **Reglas de Orquestación:**
    * Si el régimen HMM cambia a un estado no apto para la estrategia, se fuerza la transición a `PAUSED`.
    * El evento de desajuste se vincula al `version_node_id` del modelo de mercado actual (ADR-0020 V2).
*   **Entrada:** `current_market_regime`, `strategy_regime_constraints`.
*   **Salida:** `alignment_status` (ALIGNED | MISMATCH).
*   **Precondición:** Feed de régimen de mercado actualizado (Módulo `ingest`).
*   **Postcondición:** Bloqueo preventivo de ejecución si hay desalineación.

### **TTR-004: Orquestación de Rastreo Final (Equity Curve Tracker)**
*   **Descripción:** Invoca a [`equity-curve-tracker`](../features/equity-curve-tracker.md) para consolidar el PnL definitivo.
*   **Reglas de Orquestación:**
    *   Realiza el snapshot final del capital que generó la estrategia en toda su vida útil.
*   **Entrada:** `historical_fills`.
*   **Salida:** `terminal_equity_snapshot`.
*   **Precondición:** Posiciones cerradas.
*   **Postcondición:** Legado financiero registrado.

### **TTR-005: Orquestación de KPIs de Decadencia (Institutional Metrics)**
*   **Descripción:** Invoca a [`institutional-metrics`](../features/institutional-metrics.md) para documentar el fallo.
*   **Reglas de Orquestación:**
    *   Calcula la métrica exacta de cuánto cayó el Sharpe que motivó el retiro.
*   **Entrada:** `terminal_equity_snapshot`.
*   **Salida:** `degradation_kpis`.
*   **Precondición:** Equidad final calculada.
*   **Postcondición:** Base de datos de "fallos" alimentada.

### **TTR-006: Orquestación de Detección Macro (HMM Regime Detection)**
*   **Descripción:** Invoca a [`hmm-regime-detection`](../features/hmm-regime-detection.md) en el análisis post-mortem.
*   **Reglas de Orquestación:**
    *   Si el retiro fue forzado por el régimen, registra cuál fue el régimen que "mató" la estrategia.
*   **Entrada:** `current_market_regime`.
*   **Salida:** `fatal_regime_tag`.
*   **Precondición:** Alerta de Regime Guard.
*   **Postcondición:** Contexto estructural archivado.

### **TTR-007: Orquestación de Límites Soberanos (Portfolio Rules)**
*   **Descripción:** Invoca a [`portfolio-rules`](../features/portfolio-rules.md) para forzar salidas Sistémicas.
*   **Reglas de Orquestación:**
    *   Si el portafolio entero colapsa, este componente retira masivamente las estrategias perdedoras.
*   **Entrada:** `portfolio_hard_limit_breach`.
*   **Salida:** `mass_withdraw_signal`.
*   **Precondición:** Alerta del Manage.
*   **Postcondición:** Detención sangría global.

### **TTR-008: Orquestación de Alerta Terminal (Notification)**
*   **Descripción:** Invoca a [`notification`](../features/notification.md) informando el funeral de la estrategia.
*   **Reglas de Orquestación:**
    *   Envia reporte del PnL total generado y razón de muerte.
*   **Entrada:** `retirement_reason`, `terminal_equity_snapshot`.
*   **Salida:** `discord_funeral_alert`.
*   **Precondición:** Retiro confirmado.
*   **Postcondición:** Cierre del ciclo comunicacional.

### **TTR-009: Orquestación de Archivo Inmutable (Audit Log)**
*   **Descripción:** Invoca a [`audit-log`](../features/audit-log.md) para sellar la tumba.
*   **Reglas de Orquestación:**
    *   Firma digitalmente el historial de trades para que nunca sea modificado.
*   **Entrada:** `complete_strategy_history`.
*   **Salida:** `terminal_audit_hash`.
*   **Precondición:** Todos los procesos finalizados.
*   **Postcondición:** Registro auditable permanentemente.

### **TTR-010: Orquestación de Acceso Agéntico vía MCP (Cabina Dual — Bloqueado por Defecto)**
*   **Descripción:** Invoca a [`agentic-mcp-gateway`](../features/agentic-mcp-gateway.md) para evaluar el permiso antes de aceptar una llamada proveniente del canal MCP sobre la `public_interface` de este módulo.
*   **Reglas de Orquestación:**
    * `withdraw` opera exclusivamente sobre el cierre de posiciones con capital real: ninguna llamada del canal MCP nace con permiso, sin excepción (ADR-0123).
    * Toda llamada se rechaza salvo que `PRODUCTION_OVERRIDE` esté activo en ese momento; el rechazo y la concesión quedan ambos auditados con su procedencia agente (`agent_session_id`).
*   **Entrada:** Llamada MCP entrante con pipeline `withdraw`, estado de `PRODUCTION_OVERRIDE`.
*   **Salida:** Resultado de la operación enrutado al agente, o rechazo, + registro de auditoría de procedencia.

### **TTR-999: Implementación del Protocolo Fail-Fast Safe (ADR-0066)**
*   **Descripción:** Garantizar que cualquier invocación a componentes de validación o procesamiento intensivo esté gobernada por la cascada de intensidad.
*   **Reglas de Orquestación:**
    *   **Short-Circuit Mandatorio:** El módulo debe validar el éxito de los filtros `LIGHT` antes de solicitar recursos para tareas `MEDIUM` o `HEAVY`.
    *   **Telemetry:** Registrar el ahorro de ciclos de CPU/GPU cuando se produzca un descarte temprano.
*   **Entrada:** `ComputeIntensityMetadata`.
*   **Salida:** `fail_fast_execution_status`.
*   **Postcondición:** Optimización del consumo de hardware bajo el principio de Soberanía Local (ADR-0032).

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundamentos (ADR-0020 V2):** El catálogo de los 25 campos maestros está en la sección "Épica 0: Esqueleto Fundacional" de este documento (referencia, no esquema). Toda entidad persistida por este módulo incluye el Grupo I de forma universal; los Grupos II–V se aplican solo en los campos que el Perfil Técnico de cada feature exige (Filtro de Relevancia, ADR-0020 V2) — nunca el catálogo completo.

- **Decisión Arquitectónica Asociada:**
    - ADR-0005: Versionamiento (Cierre de linaje).
    - ADR-0010: Hard Limits (Pausa por riesgo).
    - ADR-0020 V2: Inundación de Fundaciones.

---

## Dependencias
**Depende de:**
- [`execute`](../modules/execute.md) — para confirmar el cierre de posiciones vivas.
- [`feedback`](../modules/feedback.md) — para la recepción de señales de alerta de largo plazo.

**Consumido por:**
- [`manage`](../modules/manage.md) — para el rebalanceo de capital tras el retiro.
- [`generate`](../modules/generate.md) — para la exclusión del genoma retirado en futuros ciclos.

---

## Lifecycle Completo de Estrategia

Cuando una estrategia llega a este módulo está en estado **OPERATING**:

1. **OPERATING** → (degradación detectada) → **PAUSED**
   - Ventana de veto configurable (ej: 1 día)
   - Usuario puede reactivar dentro de esa ventana
   
2. **PAUSED** → (usuario reactiva) → **OPERATING**
   - Estrategia vuelve a ejecutarse
   - Portafolio se rebalancea de nuevo
   
3. **PAUSED** → (ventana veto expira sin acción) → **RETIRED**
   - Estrategia se archiva permanentemente
   - No puede reactivarse sin override manual explícito
   
4. **OPERATING** → (usuario fuerza retiro manual) → **RETIRED**
   - Bypasea PAUSED directamente
   - Efecto inmediato

---

## Operación Final
Este módulo no corre continuamente vigilando el mercado, sino que responde a eventos de retiro:
- **Evento Feedback:** El guardián detecta anomalía crítica → Iniciar Retiro.
- **Evento Usuario:** El humano aprieta el botón de pánico → Iniciar Retiro.
- **Evento Lifecycle:** Una estrategia programada para N tiempo expira → Iniciar Retiro.

---

## Flujos de Degradación

**Detección por Métrica (Sharpe/DD):**
- Comparar performance actual vs baseline histórico
- Si caída > umbral configurable → PAUSAR
- Notificar usuario con números exactos

**Detección por Cambio de Régimen:**
- Si régimen de mercado cambió significativamente
- Estrategia diseñada para "trending" pero mercado es "mean-revert"
- Pausa automática si confianza en cambio es alta
