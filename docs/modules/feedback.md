# Retroalimentar (Guardián de Pardo)

**Carpeta:** `./modules/feedback/`
**Estado:** Orquestador (Imperative Shell / Guardián de Pardo)
**Última actualización:** 2026-04-12

---

## ¿Qué es?

El módulo de retroalimentación es el **Cerebro del Aprendizaje y Guardián del Pipeline**. Su función es orquestar la reconciliación de la operativa real, detectar anomalías y emitir el **Veredicto de Continuidad**. 

Es el módulo encargado de decidir cuándo una estrategia ya no es apta para operar y debe ser enviada al módulo de retiro (MOD-08).

---

## Épica 0: Esqueleto Fundacional

### Estructura de Archivos (FCIS — ADR-0003)

```
crates/feedback/
├── public_interface.rs   # Frontera pública: único punto de entrada para otros módulos
├── logic.rs              # Lógica pura: reconciliación, detección de anomalías, veredicto Pardo (sin DB, sin I/O)
├── orchestrator.rs       # Coordinación: invoca Trade Reconciler, Anomaly Detector, Pardo Comparison
├── persistence.rs        # Acceso a SQLite WAL y Parquet (lectura/escritura)
├── schemas.rs            # Definición de tablas: performance_drifts, anomalies, continuity_verdicts
└── types.rs              # Tipos de entrada/salida: ReconciliationReport, AnomalyReport, ContinuityVerdict
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
| | `logic_hash` | Hash del motor de feedback |
| | `data_snapshot_id` | Snapshot PIT del stream de ejecución |
| | `transformation_id` | ID del paso/tipo de transformación aplicado |
| **IV. Infraestructura y Ops** | `process_id` | PID del servicio monitor |
| | `session_id` | Agrupación de runtime |
| | `node_id` | ID del hardware físico supervisor |
| **V. Forense y Ejecución** | `portfolio_container_id` | Contenedor de portafolio |
| | `compliance_status_id` | Veredicto de riesgo |
| | `risk_audit_id` | Ticket detallado de riesgo |
| | `indicator_state_hash` | Snapshot de la anomalía detectada |
| | `execution_latency_ms` | Latencia de análisis |
| | `source_signal_id` | Link a señal origen |
| | `signature_hash` | HMAC de señales |

### TTRs Etiquetados por Fase

| TTR | Fase | Descripción corta |
|---|---|---|
| TTR-001 | **EPIC-7** | Reconciliación de operativa (Trade Reconciler) |
| TTR-002 | **EPIC-7** | Veredicto de continuidad (Pardo Comparison) |
| TTR-003 | **EPIC-7** | Autopsia de anomalías (Anomaly Detector) |
| TTR-004 | **EPIC-7** | Auditoría masiva (Queryable Audit Shell) |
| TTR-005 | **EPIC-7** | Atribución (Factor Decomposition) |
| TTR-006 | **EPIC-7** | Rastreo (Equity Curve Tracker) |
| TTR-007 | **EPIC-7** | Métricas institucionales |
| TTR-008 | **EPIC-7** | Correlación (Signal Correlation Analyzer) |
| TTR-009 | **EPIC-7** | Avisos (Notification) |
| TTR-010 | **EPIC-7** | Forense temporal (Time Warp Debugger) |
| TTR-011 | **EPIC-7** | Sellado forense (Audit Log) |
| TTR-012 | **EPIC-7** | Reporte robusto (Robust Reporting) |
| TTR-013 | **EPIC-7** | Monitoreo cinético |
| TTR-014 | **EPIC-7** | Métricas Autopilot |
| TTR-015 | **EPIC-7** | Reporte de auto-auditoría (Cost Reconciler) |
| TTR-016 | **EPIC-7** | Reconstrucción táctil de fricción (Interactive Stress Lab) |
| TTR-017 | **EPIC-7** | Diagnóstico de pureza de Alpha (Alpha Purity Analyzer) |

---

## Comportamientos Observables (Orquestación)

- [ ] **Reconciliación de Realidad:** Coordina el balance de la sesión llamando a [trade-reconciler](../features/trade-reconciler.md).
- [ ] **Extracción de Lecciones:** Invoca a [anomaly-detector](../features/anomaly-detector.md) para traducir fallos en restricciones (Insights).
- [ ] **Validación de Salud:** Ejecuta los veredictos de drift estadístico a través de [pardo-comparison](../features/pardo-comparison.md).

---

## Restricciones

- El módulo no puede modificar datos históricos ya guardados (solo lee y analiza)
- Las sugerencias son recomendaciones, no acciones automáticas — el sistema de generación las recibe pero decide si usarlas
- La reconciliación diaria no puede bloquear la ejecución de órdenes

---

## Parámetros Configurables

| Parámetro | Default | Qué hace |
|---|---|---|
| RECONCILIATION_TIME | configurable | A qué hora se hace la reconciliación diaria (ej: cierre de mercado) |
| SPREAD_TOLERANCE | configurable | Diferencia máxima aceptable entre spread esperado y real |
| CORRELATION_BREAK_THRESHOLD | configurable | Cuánto debe cambiar la correlación para considerarse una ruptura |

---

## Features Consumidas (Reutilizables)

- **[`trade-reconciler`](../features/trade-reconciler.md)** — Reconciliación de fills reales vs esperados.
- **[`pardo-comparison`](../features/pardo-comparison.md)** — Validación de drift estadístico y consistencia.
- **[`anomaly-detector`](../features/anomaly-detector.md)** — Detección de patrones anómalos de ejecución.
- **[`factor-decomposition`](../features/factor-decomposition.md)** — Análisis de atribución (Alpha vs Beta) y origen de anomalías.
- **[`alpha-purity-analyzer`](../features/alpha-purity-analyzer.md)** — Diagnóstico de significancia del Alpha (¿murió el Alpha o el Beta?).
- **[`equity-curve-tracker`](../features/equity-curve-tracker.md)** — Monitoreo de degradación de capital consolidado.
- **[`institutional-metrics`](../features/institutional-metrics.md)** — Cálculo de KPIs para análisis de supervivencia.
- **[`signal-correlation-analyzer`](../features/signal-correlation-analyzer.md)** — Detección de ruptura de diversificación.
- **[`notification`](../features/notification.md)** — Emisión de recomendaciones y reportes EOD.
- **[`time-warp-debugger`](../features/time-warp-debugger.md)** — Reproducción forense de eventos pasados para análisis de fallos.
- **[`duckdb-sql-engine`](../features/duckdb-sql-engine.md)** — Reconciliación masiva Out-of-Core de historiales operativos.
- **[`audit-log`](../features/audit-log.md)** — Registro inmutable de autopsias de sesión.
- **[`robust-reporting`](../features/robust-reporting.md)** — Generación de reportes detallados JSON/HTML enriquecidos.
- **[`kinetic-micro-management`](../features/kinetic-micro-management.md)** — Módulo defensivo hostil de scale out y z-score trailing.
- **[`autopilot-metrics-provider`](../features/autopilot-metrics-provider.md)** — Métricas dinámicas en tiempo real del Autopilot.
- **[`auto-auditoria-portafolios-vivos`](../features/auto-auditoria-portafolios-vivos.md)** — Monitoreo dinámico de costes reales de ejecución y recalculador de R Expectancy.
- **[`interactive-stress-lab`](../features/interactive-stress-lab.md)** — Reconstrucción táctil del nivel de fricción que explica una degradación observada en vivo.

---

---

## Tareas (TTRs) — Protocolo de Orquestación (§8.1)

### **TTR-001: Orquestación de Reconciliación de Operativa (Trade Reconciler)**
*   **Descripción:** Invoca a [`trade-reconciler`](../features/trade-reconciler.md) para auditar fills reales vs teóricos.
*   **Reglas de Orquestación:**
    * El proceso debe ejecutarse al cierre de cada sesión (EOD) o en cada fill (Real-Time).
    * Toda discrepancia de slippage debe vincularse al `process_id` de la ejecución original (ADR-0020 V2).
*   **Entrada:** `actual_fills`, `expected_simulation_fills`.
*   **Salida:** `reconciliation_report`.
*   **Precondición:** Fills persistidos en el módulo `execute`.
*   **Postcondición:** Registro de la eficiencia de ejecución en `performance_drifts`.

### **TTR-002: Orquestación de Veredicto de Continuidad (Pardo Comparison)**
*   **Descripción:** Invoca periódicamente a [`pardo-comparison`](../features/pardo-comparison.md) para certificar la supervivencia de la estrategia.
*   **Reglas de Orquestación:**
    * Si el veredicto Pardo es negativo, emitir señal de `AUTO_WITHDRAW` al módulo `withdraw`.
    * El veredicto debe incluir el `audit_hash` consolidado del historial de la estrategia (ADR-0020 V2).
*   **Entrada:** `live_vs_paper_metrics`.
*   **Salida:** `continuity_verdict` (approved | deprecated).
*   **Precondición:** Suficiente historial operativo (> 100 trades o > 30 días).
*   **Postcondición:** Marcado del nodo de versión como `DEPRECATED` en el DAG si falla.

### **TTR-003: Orquestación de Autopsia de Anomalías (Anomaly Detector)**
*   **Descripción:** Invoca a [`anomaly-detector`](../features/anomaly-detector.md) para identificar fallos estructurales.
*   **Reglas de Orquestación:**
    * Traducir anomalías detectadas en `negative_constraints` para el módulo `generate`.
    * Inyectar el `version_node_id` del modelo detector en el reporte de anomalía (ADR-0020 V2).
*   **Entrada:** `execution_logs`, `market_volatility_context`.
*   **Salida:** `anomaly_report`, `generation_constraints`.
*   **Precondición:** Reporte de reconciliación (TTR-001) finalizado.
*   **Postcondición:** Retroalimentación inyectada en la base de conocimientos de `generate`.

### **TTR-004: Orquestación de Auditoría Masiva (Queryable Audit Shell)**
*   **Descripción:** Invoca a [`duckdb-sql-engine`](../features/duckdb-sql-engine.md) para realizar auditorías cruzadas sobre meses de ejecución.
*   **Reglas de Orquestación:**
    * Permite buscar correlaciones entre fallos de ejecución y regímenes de mercado históricos.
    * El resultado debe certificar la integridad de los datos consultados via `audit_hash`.
*   **Entrada:** `sql_audit_query`.
*   **Salida:** `mass_audit_report`.
*   **Precondición:** Cierre de periodo (Weekly/Monthly) o disparo por anomalía grave.
*   **Postcondición:** Insights para ajuste de gestión de riesgo a nivel Portafolio.

### **TTR-005: Orquestación de Atribución (Factor Decomposition)**
*   **Descripción:** Invoca a [`factor-decomposition`](../features/factor-decomposition.md) para extraer el Alpha.
*   **Reglas de Orquestación:**
    *   Si Alpha < 0, se audita por degradación estructural.
*   **Entrada:** `live_returns`, `market_beta`.
*   **Salida:** `alpha_attribution_report`.
*   **Precondición:** Cierre de mes.
*   **Postcondición:** Reporte de calidad del retorno.

### **TTR-006: Orquestación de Rastreo (Equity Curve Tracker)**
*   **Descripción:** Invoca a [`equity-curve-tracker`](../features/equity-curve-tracker.md) para monitorear el capital consolidado de la estrategia.
*   **Reglas de Orquestación:**
    *   Si el PnL acumulado toca el Stop de Portafolio, alerta a Manage.
*   **Entrada:** `daily_pnl`.
*   **Salida:** `equity_curve_update`.
*   **Precondición:** EOD (End of Day).
*   **Postcondición:** Curva actualizada.

### **TTR-007: Orquestación de Métricas Institucionales**
*   **Descripción:** Invoca a [`institutional-metrics`](../features/institutional-metrics.md) para calcular Calmar y Sortino.
*   **Reglas de Orquestación:**
    *   Inyección de los KPIs en el `audit_hash` del reporte.
*   **Entrada:** `trade_history`.
*   **Salida:** `kpi_report`.
*   **Precondición:** Reconciliación completada.
*   **Postcondición:** Métricas disponibles para visualización.

### **TTR-008: Orquestación de Correlación (Signal Correlation Analyzer)**
*   **Descripción:** Invoca a [`signal-correlation-analyzer`](../features/signal-correlation-analyzer.md) para detectar sobre-exposición de señales.
*   **Reglas de Orquestación:**
    *   Rechaza correlación si pasa > 0.8 en la ventana viva.
*   **Entrada:** `live_signals_vector`.
*   **Salida:** `correlation_matrix`.
*   **Precondición:** Múltiples estrategias activas.
*   **Postcondición:** Alerta de concentración.

### **TTR-009: Orquestación de Avisos (Notification)**
*   **Descripción:** Invoca a [`notification`](../features/notification.md) al completar la autopsia.
*   **Reglas de Orquestación:**
    *   Envía resumen de KPIs y veredicto de continuidad.
*   **Entrada:** `daily_report`.
*   **Salida:** `slack_discord_message`.
*   **Precondición:** Todos los TTRs de feedback cerrados.
*   **Postcondición:** Usuario informado.

### **TTR-010: Orquestación Forense Temporal (Time Warp Debugger)**
*   **Descripción:** Invoca a [`time-warp-debugger`](../features/time-warp-debugger.md) ante una anomalía grave.
*   **Reglas de Orquestación:**
    *   Recrea el evento tick-a-tick aislando el `version_node_id`.
*   **Entrada:** `audit_log_trace`, `anomaly_id`.
*   **Salida:** `debug_session_replay`.
*   **Precondición:** Anomalía crítica detectada.
*   **Postcondición:** Diagnóstico de causa raíz.

### **TTR-011: Orquestación de Sellado Forense (Audit Log)**
*   **Descripción:** Invoca a [`audit-log`](../features/audit-log.md) para firmar el veredicto Pardo.
*   **Reglas de Orquestación:**
    *   Inmutabilidad absoluta del veredicto de supervivencia.
*   **Entrada:** `continuity_verdict`.
*   **Salida:** `audit_hash`.
*   **Precondición:** Veredicto emitido.
*   **Postcondición:** Conformidad regulatoria.

### **TTR-012: Orquestación de Reporte Robusto (Robust Reporting)**
*   **Descripción:** Invoca a [`robust-reporting`](../features/robust-reporting.md) para generar manifiestos estáticos del veredicto final o autopsias de sesión.
*   **Reglas de Orquestación:**
    *   El reporte incluye el `audit_hash` y se comprime como evidencia histórica soberana.
*   **Entrada:** `continuity_verdict`, `anomaly_report`, `trade_history`.
*   **Salida:** `static_html_report`, `json_metadata_export`.
*   **Precondición:** Autopsia completada.
*   **Postcondición:** Reporte estático persistido en disco y vinculado al `version_node_id`.

### **TTR-013: Orquestación de Monitoreo Cinético**
*   **Descripción:** Invoca a [`kinetic-micro-management`](../features/kinetic-micro-management.md) para auditar la efectividad de las defensas hostiles aplicadas en la sesión.
*   **Reglas de Orquestación:**
    - Evalúa el ahorro de drawdown logrado gracias a las intervenciones de Scale Out y Z-Score Trailing.
*   **Entrada:** `kinetic_adjustment_commands`, `execution_logs`.
*   **Salida:** `kinetic_efficiency_report`.
*   **Precondición:** Cierre de la sesión de trading.
*   **Postcondición:** Registro del factor de reducción de riesgo.

### **TTR-014: Orquestación de Métricas Autopilot**
*   **Descripción:** Invoca a [`autopilot-metrics-provider`](../features/autopilot-metrics-provider.md) para extraer las métricas históricas de la sesión.
*   **Reglas de Orquestación:**
    - Consolida las métricas diarias del Autopilot en el reporte final de Feedback.
*   **Entrada:** `autopilot_metrics_dict`.
*   **Salida:** `session_metrics_snapshot`.
*   **Precondición:** Reconciliación (TTR-001) aprobada.
*   **Postcondición:** Snapshot almacenado inmutablemente.

### **TTR-015: Orquestación de Reporte de Auto-Auditoría (Cost Reconciler)**
*   **Descripción:** Invoca a [`auto-auditoria-portafolios-vivos`](../features/auto-auditoria-portafolios-vivos.md) para reconciliar la deriva de costos e integrarla en el informe de fin de sesión.
*   **Reglas de Orquestación:**
    - Evalúa la diferencia entre los costos de transacción promedio modelados en backtest y los costos reales registrados por la auto-auditoría.
    - Exporta los datos de desviación de spread para alimentar los veredictos de salud de Pardo.
*   **Entrada:** `expected_costs_profile`, `live_audit_costs`.
*   **Salida:** `cost_deviation_metrics`.
*   **Precondición:** Cierre de la sesión de trading.
*   **Postcondición:** Métricas de desviación añadidas al veredicto de continuidad.

### **TTR-016: Orquestación de Reconstrucción Táctil de Fricción (Interactive Stress Lab)**
*   **Descripción:** Invoca a [`interactive-stress-lab`](../features/interactive-stress-lab.md) para reconstruir qué nivel de fricción (slippage/spread) explica una degradación detectada en la operativa real.
*   **Reglas de Orquestación:**
    *   Carga la curva esperada del backtest y permite mover los deslizadores hasta empatar la curva real degradada.
    *   El vector de deslizadores que reproduce la degradación se registra como evidencia de la autopsia.
*   **Entrada:** `expected_equity_curve`, `live_degraded_curve`.
*   **Salida:** `friction_explaining_vector`, `autopsy_snapshot`.
*   **Precondición:** Degradación detectada (TTR-002/TTR-003).
*   **Postcondición:** Evidencia táctil de fricción adjunta a la autopsia de sesión.

### **TTR-017: Orquestación de Diagnóstico de Pureza (Alpha Purity Analyzer)**
*   **Descripción:** Invoca a [`alpha-purity-analyzer`](../features/alpha-purity-analyzer.md) para diagnosticar la causa de una degradación detectada en vivo.
*   **Reglas de Orquestación:**
    *   Distingue si la caída se debe a muerte del Alpha (la lógica perdió eficacia) o al apagado del Beta (el mercado/régimen dejó de empujar).
    *   El diagnóstico alimenta el veredicto de continuidad/retiro (TTR-002).
*   **Entrada:** `live_returns`, `benchmark_series`.
*   **Salida:** `alpha_decay_diagnosis`.
*   **Precondición:** Degradación detectada (TTR-002/TTR-003).
*   **Postcondición:** Causa raíz (Alpha vs Beta) registrada en la autopsia de sesión.

### **TTR-018: Orquestación de Acceso Agéntico vía MCP (Cabina Dual)**
*   **Descripción:** Invoca a [`agentic-mcp-gateway`](../features/agentic-mcp-gateway.md) para evaluar el permiso antes de aceptar una llamada proveniente del canal MCP sobre la `public_interface` de este módulo.
*   **Reglas de Orquestación:**
    * `feedback` pertenece al grupo de pipelines abiertos por defecto (ADR-0123): un agente conectado vía MCP tiene permiso total sin gate adicional.
    * Toda llamada concedida queda auditada con su procedencia agente (`agent_session_id`).
*   **Entrada:** Llamada MCP entrante con pipeline `feedback`.
*   **Salida:** Resultado de la operación enrutado al agente + registro de auditoría de procedencia.

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundamentos (ADR-0020 V2):** El catálogo de los 25 campos maestros está en la sección "Épica 0: Esqueleto Fundacional" de este documento (referencia, no esquema). Toda entidad persistida por este módulo incluye el Grupo I de forma universal; los Grupos II–V se aplican solo en los campos que el Perfil Técnico de cada feature exige (Filtro de Relevancia, ADR-0020 V2) — nunca el catálogo completo.

- **Decisión Arquitectónica Asociada:**
    - ADR-0005: Versionamiento (Feedback Loop).
    - ADR-0011: Observabilidad y Logs (Audit Trail).
    - ADR-0020 V2: Inundación de Fundaciones.

---

## Dependencias
**Depende de:**
- [`execute`](../modules/execute.md) — para la obtención de datos de ejecución real.
- [`incubate`](../modules/incubate.md) — para la comparación contra paper trading.

**Consumido por:**
- [`withdraw`](../modules/withdraw.md) — para la ejecución física del veredicto de retiro.
- [`generate`](../modules/generate.md) — para la evolución del genoma estratégico basado en fallos.
