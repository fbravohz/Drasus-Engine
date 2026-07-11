# Incubar

**Carpeta:** `./modules/incubate/`
**Estado:** Orquestador (Imperative Shell)

> **Corrección por pruebas múltiples (ADR-0151):** la **promoción** es un punto de decisión — la selección entre candidatos que compiten por promover se corrige (PBO/CSCV o DSR según criterio), con N desde el linaje; el toque se registra en `expedition_lineage` con naturaleza `PROMOTED`. Nota de impacto progresivo (ADR-0137).
**Última actualización:** 2026-04-12

---

## ¿Qué es?

El módulo de incubación es donde una estrategia aprobada pasa por un período de prueba en vivo pero sin dinero real. Opera exactamente como si fuera live (recibe señales reales del mercado, simula órdenes con spreads reales), pero el dinero es virtual.

El objetivo es detectar si la estrategia funciona igual en condiciones reales que en el backtesting histórico. Si hay una diferencia grande entre los dos, algo anda mal (puede ser sobreajuste, o que el mercado cambió).

Una estrategia que pasa esta fase es promovida al portafolio real.

---

## Épica 0: Esqueleto Fundacional

### Estructura de Archivos (FCIS — ADR-0003)

```
crates/incubate/
├── public_interface.rs   # Frontera pública: único punto de entrada para otros módulos
├── logic.rs              # Lógica pura: comparación Pardo, métricas de drift (sin DB, sin I/O)
├── orchestrator.rs       # Coordinación: invoca Paper Trader, Pardo Comparison; maneja ciclo de vida
├── persistence.rs        # Acceso a SQLite WAL (lectura/escritura)
├── schemas.rs            # Definición de tablas: incubation_sessions, virtual_trades, drift_metrics
└── types.rs              # Tipos de entrada/salida: IncubationSession, VirtualFill, DriftVerdict
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
| | `logic_hash` | Hash del motor paper (FSM/Nautilus version) |
| | `data_snapshot_id` | Snapshot PIT del stream de datos en vivo |
| | `transformation_id` | ID del paso/tipo de transformación aplicado |
| **IV. Infraestructura y Ops** | `process_id` | PID del worker de incubación |
| | `session_id` | Agrupación de runtime |
| | `node_id` | ID del hardware físico |
| **V. Forense y Ejecución** | `portfolio_container_id` | Contenedor de portafolio |
| | `compliance_status_id` | Veredicto de riesgo |
| | `risk_audit_id` | Ticket detallado de riesgo |
| | `indicator_state_hash` | Snapshot del drift medido (Pardo Profile) |
| | `execution_latency_ms` | Latencia de procesamiento |
| | `source_signal_id` | Link a señal origen |
| | `signature_hash` | HMAC de señales |

### TTRs Etiquetados por Fase

| TTR | Fase | Descripción corta |
|---|---|---|
| TTR-001 | **EPIC-5** | Despliegue de Paper Trading |
| TTR-002 | **EPIC-5** | Comparación de consistencia (Pardo) |
| TTR-003 | **EPIC-5** | Gestión de vida de orden (Order FSM) |
| TTR-004 | **EPIC-5** | Gestión de sesión (Incubation Manager) |
| TTR-005 | **EPIC-5** | Aislamiento virtual (Executable Container) |
| TTR-007 | **EPIC-5** | Conexión live (Broker Connector) |
| TTR-008 | **EPIC-5** | Deslizamiento (Slippage Models) |
| TTR-009 | **EPIC-5** | Rastreo paper (Equity Curve Tracker) |
| TTR-010 | **EPIC-5** | KPIs (Institutional Metrics) |
| TTR-011 | **EPIC-5** | Linaje (Strategy Versioning) |
| TTR-012 | **EPIC-5** | Auditoría virtual (Audit Log) |
| TTR-013 | **EPIC-5** | Temporal (Clock) |
| TTR-014 | **EPIC-5** | Monitoreo de cuarentena (Efficiency Dashboard) |
| TTR-015 | **EPIC-5** | Acceso agéntico MCP (Cabina Dual) |
| TTR-999 | **EPIC-5** | Protocolo Fail-Fast Safe (ADR-0066) |
| TTR-006 | EPIC-8 | Retroactiva (Time Warp Debugger) |

---

## Comportamientos Observables (Orquestación)

- [ ] **Despliegue Virtual:** Inicia la operación en vivo (sin capital real) llamando a [paper-trader](../features/paper-trader.md).
- [ ] **Monitoreo de Consistencia:** Realiza comparativas periódicas invocando a [pardo-comparison](../features/pardo-comparison.md).
- [ ] **Decisión de Promoción:** Si se completa el período y [pardo-comparison](../features/pardo-comparison.md) es aprobada, coordina la promoción a OPERATING.

---

## Restricciones

- El baseline de comparación (rendimiento esperado del backtesting) es fijo desde el momento en que la estrategia entra a incubación — no se recalcula
- La duración mínima de incubación es configurable y no puede saltarse
- Una estrategia no puede promoverse automáticamente si no ha completado el período mínimo de incubación

---

## Parámetros Configurables

| Parámetro | Default | Qué hace |
|---|---|---|
| INCUBATION_MONTHS | configurable | Cuántos meses dura el paper trading (ej: 3-6 meses) |
| MAX_SHARPE_DRIFT | configurable | Máxima caída aceptable del Sharpe vs baseline (ej: -20%) |
| AUTO_PROMOTE | configurable | Si true, promueve automáticamente al pasar la prueba; si false, espera confirmación del usuario |

---

## Features Consumidas (Reutilizables)

> *(ADR-0137)* Este módulo es la **composición preset canónica** de estas features — define el cableado por defecto. En el Canvas [Forge/Reactor], las features pueden conectarse directamente sin que este módulo sea intermediario obligatorio en runtime.

- **[`paper-trader`](../features/paper-trader.md)** — Simulación de trading sin capital real.
- **[`executable-container`](../features/executable-container.md)** — Entorno aislado de ejecución para la estrategia (Runner).
- **[`incubation-manager`](../features/incubation-manager.md)** — Gestión del ciclo de vida de la sesión (Start/Pause/Stop).
- **[`time-warp-debugger`](../features/time-warp-debugger.md)** — Depuración "paso a paso" de eventos pasados en la sesión.
- **[`pardo-comparison`](../features/pardo-comparison.md)** — Juez de consistencia estadística (Robert Pardo).
- **[`order-fsm`](../features/order-fsm.md)** — Máquina de estados de órdenes virtuales.
- **[`broker-connector`](../features/broker-connector.md)** — Conector para datos en vivo y ejecución simulada.
- **[`slippage-models`](../features/slippage-models.md)** — Modelado realista de spreads en paper.
- **[`equity-curve-tracker`](../features/equity-curve-tracker.md)** — Tracking de capital virtual.
- **[`institutional-metrics`](../features/institutional-metrics.md)** — Cálculo de KPIs para veredicto.
- **[`strategy-versioning`](../features/strategy-versioning.md)** — Referencia a versión exacta en el DAG.
- **[`audit-log`](../features/audit-log.md)** — Registro inmutable de la sesión de incubación.
- **[`clock`](../features/clock.md)** — Timestamps deterministas para eventos en vivo.

---

---

## Tareas (TTRs) — Protocolo de Orquestación (§8.1)

### **TTR-001: Orquestación de Despliegue de Paper Trading (Paper Trader)**
*   **Descripción:** Inicia la sesión de trading virtual invocando a [`paper-trader`](../features/paper-trader.md).
*   **Reglas de Orquestación:**
    * El motor debe utilizar la precisión de ejecución espejo (Slippage + Latencia simulada).
    * Se debe inyectar el `process_id` del worker de incubación en cada orden virtual (ADR-0020).
*   **Entrada:** `approved_strategy`, `live_data_stream`.
*   **Salida:** `virtual_trade_fills`.
*   **Precondición:** Estrategia en estado `INCUBATING`.
*   **Postcondición:** Registro de actividad en `virtual_trades` marcado con `institutional_tag`.

### **TTR-002: Orquestación de Comparación de Consistencia (Pardo)**
*   **Descripción:** Invoca periódicamente a [`pardo-comparison`](../features/pardo-comparison.md) para medir el Drift vs Backtest.
*   **Reglas de Orquestación:**
    * Si el Drift excede `MAX_SHARPE_DRIFT`, disparar alerta de `PAUSE_INCUBATION`.
    * El veredicto de consistencia debe incluir el `audit_hash` del baseline histórico (ADR-0020).
*   **Entrada:** `virtual_performance`, `backtest_baseline`.
*   **Salida:** `consistency_verdict` (STABLE | DRIFTED).
*   **Precondición:** TTR-001 acumulando datos vivos (> 30 días).
*   **Postcondición:** Actualización del score de confianza en `incubation_sessions`.

### **TTR-003: Orquestación de Gestión de Vida de Orden (Order FSM)**
*   **Descripción:** Delega el control de transiciones a [`order-fsm`](../features/order-fsm.md).
*   **Reglas de Orquestación:**
    * Toda transición de estado virtual debe ser auditable y coincidir con la lógica live.
    * Las órdenes deben persistir en el DAG con `version_node_id` de la sesión (ADR-0020).
*   **Entrada:** `virtual_order_event`.
*   **Salida:** `fsm_state_transition`.
*   **Precondición:** TTR-001 procesando eventos de mercado.
*   **Postcondición:** Integridad del flujo de órdenes garantizada por la máquina de estados.

### **TTR-004: Orquestación de Gestión de Sesión (Incubation Manager)**
*   **Descripción:** Utiliza [`incubation-manager`](../features/incubation-manager.md) para controlar la persistencia, ciclo de vida dual (Legacy vs Quarantine Sandbox), cono de silencio y interruptores automáticos.
*   **Reglas de Orquestación:**
    * El orquestador evalúa barra a barra el desvío MAE en el Sandbox de Cuarentena (7 días). Dispara eutanasia predictiva si excede el umbral (+15% MAE flotante).
    * Dibuja y proyecta las bandas estadísticas (1, 2, 3 sigmas) basadas en Monte Carlo en caliente.
    * Si la equidad cruza el límite inferior (-1 sigma), marca de forma inmediata la estrategia con la Broken Strategy Flag, cerrando posiciones virtuales/reales vía [`order-fsm`](../features/order-fsm.md) en <1ms.
    * Asegura que los datos de la sesión sobrevivan a reinicios del sistema y emite eventos inmutables de cambio de estado atados a un identificador único (ADR-0020).
*   **Entrada:** `session_control_command`, `live_data_stream`, `monte_carlo_distribution`.
*   **Salida:** `updated_session_status`, `consistency_metrics` (Return/Drawdown Efficiency), `kill_switch_trigger`.
*   **Precondición:** Módulo `incubate` activo y estrategia en estado `INCUBATING`.
*   **Postcondición:** Estado de la sesión persistido con métricas de drift y veredicto definitivo.



### **TTR-005: Orquestación de Aislamiento Virtual (Executable Container)**
*   **Descripción:** Delega ejecución a [`executable-container`](../features/executable-container.md).
*   **Reglas de Orquestación:**
    *   Ejecuta el AST en sandbox sin acceso a capital.
*   **Entrada:** `strategy_ast`.
*   **Salida:** `container_process`.
*   **Precondición:** Sesión iniciada.
*   **Postcondición:** Entorno seguro operativo.

### **TTR-006: Orquestación Retroactiva (Time Warp Debugger)**
*   **Descripción:** Invoca a [`time-warp-debugger`](../features/time-warp-debugger.md) para revisión de paper trades.
*   **Reglas de Orquestación:**
    *   Permite al usuario pausar y retroceder el paper trading.
*   **Entrada:** `session_id`, `target_timestamp`.
*   **Salida:** `replayed_market_state`.
*   **Precondición:** Incidencias de llenado.
*   **Postcondición:** Inspección forense activa.

### **TTR-007: Orquestación de Conexión Live (Broker Connector)**
*   **Descripción:** Invoca a [`broker-connector`](../features/broker-connector.md) en modo solo lectura (WebSockets).
*   **Reglas de Orquestación:**
    *   Mapea los ticks reales al motor de paper trading.
*   **Entrada:** `symbol_subscription`.
*   **Salida:** `live_data_stream`.
*   **Precondición:** Token de API válido.
*   **Postcondición:** Feed de precios inyectado.

### **TTR-008: Orquestación de Deslizamiento (Slippage Models)**
*   **Descripción:** Invoca a [`slippage-models`](../features/slippage-models.md) para simular penalización real.
*   **Reglas de Orquestación:**
    *   Obligatorio para evitar sobre-optimismo en paper.
*   **Entrada:** `virtual_order`.
*   **Salida:** `penalized_fill_price`.
*   **Precondición:** TTR-001 generó fill virtual.
*   **Postcondición:** Precio realista asignado.

### **TTR-009: Orquestación de Rastreo Paper (Equity Curve Tracker)**
*   **Descripción:** Invoca a [`equity-curve-tracker`](../features/equity-curve-tracker.md) para medir capital simulado.
*   **Reglas de Orquestación:**
    *   Actualiza el balance virtual en cada tick.
*   **Entrada:** `penalized_fill_price`.
*   **Salida:** `virtual_equity`.
*   **Precondición:** Orden llenada.
*   **Postcondición:** Gráfico UI actualizado.

### **TTR-010: Orquestación de KPIs (Institutional Metrics)**
*   **Descripción:** Invoca a [`institutional-metrics`](../features/institutional-metrics.md) para paper stats.
*   **Reglas de Orquestación:**
    *   Emite Sharpe de incubación para comparar con backtest.
*   **Entrada:** `virtual_equity`.
*   **Salida:** `paper_kpis`.
*   **Precondición:** Ventana de tiempo cumplida.
*   **Postcondición:** Base para el Veredicto Pardo.

### **TTR-011: Orquestación de Linaje (Strategy Versioning)**
*   **Descripción:** Invoca a [`strategy-versioning`](../features/strategy-versioning.md) para marcar la incubación en el DAG.
*   **Reglas de Orquestación:**
    *   Crea un nodo hijo atado a la simulación paper.
*   **Entrada:** `session_id`, `parent_strategy`.
*   **Salida:** `dag_node`.
*   **Precondición:** Inicio de sesión.
*   **Postcondición:** Trazabilidad asegurada.

### **TTR-012: Orquestación de Auditoría Virtual (Audit Log)**
*   **Descripción:** Invoca a [`audit-log`](../features/audit-log.md) para firmar la sesión completa.
*   **Reglas de Orquestación:**
    *   Evita alteraciones de paper trading post-facto.
*   **Entrada:** `session_results`.
*   **Salida:** `audit_hash`.
*   **Precondición:** Cierre de incubación.
*   **Postcondición:** Reporte certificado.

### **TTR-013: Orquestación Temporal (Clock)**
*   **Descripción:** Invoca a [`clock`](../features/clock.md) para marcación inmutable de eventos.
*   **Reglas de Orquestación:**
    *   Evita desfases entre el reloj local y el exchange.
*   **Entrada:** `tick`.
*   **Salida:** `synced_timestamp`.
*   **Precondición:** Llegada de datos.
*   **Postcondición:** Ticks ordenados y limpios.### **TTR-014: Orquestación de Monitoreo de Cuarentena (Efficiency & Incubation Dashboard)**
*   **Descripción:** Invoca a [`efficiency-incubation-dashboard`](../features/efficiency-incubation-dashboard.md) para reflejar las bandas de confianza de Monte Carlo en vivo.
*   **Reglas de Orquestación:**
    - Superpone la equidad real de paper trading en vivo sobre las bandas estadísticas.
    - Emite una alerta de desviación si la curva sale del Cono de Silencio.
*   **Entrada:** `live_vs_historical_equity_metrics`.
*   **Salida:** `cone_status_updates`.
*   **Precondición:** TTR-009 y TTR-010 iniciados.
*   **Postcondición:** Telemetría enviada a la UI.

### **TTR-015: Orquestación de Acceso Agéntico vía MCP (Cabina Dual)**
*   **Descripción:** Invoca a [`agentic-mcp-gateway`](../features/agentic-mcp-gateway.md) para evaluar el permiso antes de aceptar una llamada proveniente del canal MCP sobre la `public_interface` de este módulo.
*   **Reglas de Orquestación:**
    * `incubate` pertenece al grupo de pipelines abiertos por defecto (ADR-0123): un agente conectado vía MCP tiene permiso total sin gate adicional.
    * Toda llamada concedida queda auditada con su procedencia agente (`agent_session_id`).
*   **Entrada:** Llamada MCP entrante con pipeline `incubate`.
*   **Salida:** Resultado de la operación enrutado al agente + registro de auditoría de procedencia.

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

- **Inundación de Fundamentos (ADR-0020):** El catálogo de los 25 campos maestros está en la sección "Épica 0: Esqueleto Fundacional" de este documento (referencia, no esquema). Toda entidad persistida por este módulo incluye el Grupo I de forma universal; los Grupos II–V se aplican solo en los campos que el Perfil Técnico de cada feature exige (Filtro de Relevancia, ADR-0020) — nunca el catálogo completo.

- **Decisión Arquitectónica Asociada:**
    - ADR-0017: Simulación de Alta Fidelidad (Paper Trading).
    - ADR-0020: Inundación de Fundaciones.
    - ADR-0010: Hard Limits (aplicados a paper trading).

---

## Dependencias
**Depende de:**
- [`validate`](../modules/validate.md) — para la recepción de estrategias certificadas.
- [`broker-connector`](../features/broker-connector.md) — para el feed de datos en vivo.

**Consumido por:**
- [`manage`](../modules/manage.md) — para la promoción a capital real (OPERATING).
