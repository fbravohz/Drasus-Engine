# Gestionar

**Carpeta:** `./modules/manage/`
**Estado:** Orquestador (Imperative Shell)
**Última actualización:** 2026-06-11

---

## ¿Qué es?

El módulo de gestión es el que combina múltiples estrategias en un portafolio y decide cuánto capital asignar a cada una. No opera una sola estrategia — opera el conjunto, buscando que la combinación sea mejor que cualquier estrategia individual.

También define las reglas del portafolio: límites de riesgo, reglas de rebalanceo, condiciones de pausa. Estas reglas tienen prioridad sobre cualquier regla individual de estrategia.

---

## Épica 0: Esqueleto Fundacional

### Estructura de Archivos (FCIS — ADR-0003)

```
crates/manage/
├── public_interface.rs   # Frontera pública: único punto de entrada para otros módulos
├── logic.rs              # Lógica pura: optimización HRP/Markowitz, cálculo de pesos (sin DB, sin I/O)
├── orchestrator.rs       # Coordinación: invoca Portfolio Optimizer, Rules, Rebalancing Daemon
├── persistence.rs        # Acceso a SQLite WAL y Parquet (lectura/escritura)
├── schemas.rs            # Definición de tablas: portfolios, weights, rebalance_history
└── types.rs              # Tipos de entrada/salida: PortfolioState, WeightsVector, RebalanceTrigger
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
| | `logic_hash` | Hash del optimizador (HRP/Markowitz) |
| | `data_snapshot_id` | Snapshot PIT de track-records analizados |
| | `transformation_id` | ID del paso/tipo de transformación aplicado |
| **IV. Infraestructura y Ops** | `process_id` | PID del servicio de gestión |
| | `session_id` | Agrupación de runtime |
| | `node_id` | ID del hardware físico |
| **V. Forense y Ejecución** | `portfolio_container_id` | Contenedor de portafolio |
| | `compliance_status_id` | Veredicto de riesgo |
| | `risk_audit_id` | Ticket detallado de riesgo |
| | `indicator_state_hash` | Snapshot de correlaciones agregadas |
| | `execution_latency_ms` | Latencia de cálculo de pesos |
| | `source_signal_id` | Link a señal origen |
| | `signature_hash` | HMAC de señales |

### TTRs Etiquetados por Fase

| TTR | Fase | Descripción corta |
|---|---|---|
| TTR-002 | **EPIC-5** | Vigilancia de riesgo y reglas (Rules Wrapper & Challenge Mode) |
| TTR-019 | **EPIC-5** | Promoción directa (Bypass MOD-04) |
| TTR-024 | **EPIC-5** | Advanced Trade Management (ATM) |
| TTR-001 | **EPIC-6** | Optimización de pesos (Portfolio Optimizer) |
| TTR-003 | **EPIC-6** | Adaptación de régimen (HMM Detection) |
| TTR-004 | **EPIC-6** | Persistencia de portafolio (Databank Manager) |
| TTR-005 | **EPIC-6** | Daemon de rebalanceo automático |
| TTR-006 | **EPIC-6** | Backtesting multiestratégico |
| TTR-010 | **EPIC-6** | Reparto de capital (Portfolio Sizing) |
| TTR-012 | **EPIC-6** | Vigilancia sistémica de Beta (Alpha Decoupling) |
| TTR-013 | **EPIC-6** | Diversificación (Signal Correlation Analyzer) |
| TTR-014 | **EPIC-6** | Atribución (Factor Decomposition) |
| TTR-015 | **EPIC-6** | Equidad global (Equity Curve Tracker) |
| TTR-016 | **EPIC-6** | KPIs macro (Institutional Metrics) |
| TTR-017 | **EPIC-6** | Versionado (Strategy Versioning) |
| TTR-018 | **EPIC-6** | Auditoría de gestión (Audit Log) |
| TTR-021 | **EPIC-6** | Métricas de riesgo avanzadas |
| TTR-022 | **EPIC-6** | Clustering K-Means y hrp_rank |
| TTR-023 | **EPIC-6** | Versionado de portafolio Git-Like |
| TTR-025 | **EPIC-6** | Hedging cointegrativo |
| TTR-026 | **EPIC-6** | Router de liquidez |
| TTR-027 | **EPIC-6** | Búsqueda genética de portafolios |
| TTR-028 | **EPIC-6** | Análisis de solapamiento temporal real |
| TTR-029 | **EPIC-6** | Rescalado de pesos y ledger de simulación |
| TTR-030 | **EPIC-6** | Portafolios federados (Federated Portfolio) |
| TTR-031 | **EPIC-6** | Simulación de portafolio real (Portfolio Backtest) |
| TTR-033 | **EPIC-6** | Fitness contextual de portafolio (Contextual Fitness Scorer) |
| TTR-034 | **EPIC-6** | Genoma de portafolio y correlación (ADR-0108/ADR-0111) |
| TTR-035 | **EPIC-6** | Acceso agéntico MCP (Cabina Dual — permiso condicionado) |
| TTR-036 | **EPIC-6** | Aporte fundamental en ponderación de portafolio (Exposure Map + Indicator Projector) |
| TTR-011 | EPIC-8 | Auditoría narrativa (Self-Explanation) |
| TTR-020 | EPIC-8 | Validación visual (Dendrogram / Heatmap) |
| TTR-032 | EPIC-9+ | Protocolo de acceso remoto (RPAP) |

---

## Comportamientos Observables (Orquestación)

- [ ] **Asignación Soberana:** Invoca a [portfolio-optimizer](../features/portfolio-optimizer.md) para repartir el capital.
- [ ] **Vigilancia de Límites:** Aplica las reglas globales de [portfolio-rules](../features/portfolio-rules.md) sobre todo el sistema.
- [ ] **Reequilibrio Automático:** Reacciona a la entrada/salida de estrategias coordinando nuevos cálculos de pesaje.

---

## Restricciones

- Las reglas del portafolio siempre tienen prioridad sobre las reglas de estrategia individual
- Los pesos de las estrategias siempre deben sumar 100% (no puede haber capital sin asignar ni sobre-asignado)
- Una estrategia no puede recibir más capital del que su propia regla de riesgo permite

---

## Parámetros Configurables

| Parámetro | Default | Qué hace |
|---|---|---|
| OPTIMIZATION_METHOD | configurable | Método de optimización: HRP (jerárquico) o Markowitz (clásico) |
| MAX_WEIGHT_PER_STRATEGY | configurable | Máximo porcentaje de capital en una sola estrategia |
| REBALANCE_FREQUENCY | configurable | Con qué frecuencia se rebalancea el portafolio |
| HARD_LIMITS | configurable | Límites duros que nunca se cruzan (ej: máximo DrawDown del portafolio) |
| SOFT_ALERTS | configurable | Umbrales que generan alerta pero no acción automática |

---

## Features Consumidas (Reutilizables)

> *(ADR-0137)* Este módulo es la **composición preset canónica** de estas features — define el cableado por defecto. En el Canvas [Forge/Reactor], las features pueden conectarse directamente sin que este módulo sea intermediario obligatorio en runtime.

- **[`portfolio-optimizer`](../features/portfolio-optimizer.md)** — Cálculo de pesos y optimización de capital.
- **[`signal-correlation-analyzer`](../features/signal-correlation-analyzer.md)** — Diversificación de señales del portafolio.
- **[`factor-decomposition`](../features/factor-decomposition.md)** — Análisis de riesgo y atribución del portafolio.
- **[`equity-curve-tracker`](../features/equity-curve-tracker.md)** — Tracking de capital agregado.
- **[`institutional-metrics`](../features/institutional-metrics.md)** — Métricas consolidadas del portafolio.
- **[`hmm-regime-detection`](../features/hmm-regime-detection.md)** — Pesos adaptativos por estado de mercado.
- **[`portfolio-rules`](../features/portfolio-rules.md)** — Reglas de riesgo globales.
- **[`federated-portfolio`](../features/federated-portfolio.md)** — Aislamiento lógico de reglas y gobernanza autónoma de múltiples contenedores de portafolios.
- **[`strategy-versioning`](../features/strategy-versioning.md)** — Referencias locked a versiones del portafolio.
- **[`databank-manager`](../features/databank-manager.md)** — Persistencia de snapshots de portafolio y composiciones históricas.
- **[`audit-log`](../features/audit-log.md)** — Registro inmutable de eventos de gestión.
- **[`precision-sizing-models`](../features/precision-sizing-models.md)** — Reparto de capital y dimensionamiento estratégico.
- **[`strategy-self-explanation`](../features/strategy-self-explanation.md)** — Auditoría narrativa determinista.
- **[`alpha-decoupling`](../features/alpha-decoupling.md)** — Evaluación sistemática de exposición Beta del portafolio.
- **[`advanced-trade-management`](../features/advanced-trade-management.md)** — Gestión operativa multicapa, Grid Trading y Hedging.
- **[`remote-portfolio-access-protocol`](../features/remote-portfolio-access-protocol.md)** — Acceso remoto colaborativo y exposición controlada de datos transaccionales.
- **[`contextual-fitness-scorer`](../features/contextual-fitness-scorer.md)** — Score multidimensional por régimen para seleccionar estrategias que cubran los regímenes débiles del portafolio.

---

---

## Tareas (TTRs) — Protocolo de Orquestación (§8.1)

### **TTR-001: Orquestación de Optimización de Pesos (Portfolio Optimizer)**
*   **Descripción:** Invoca a [`portfolio-optimizer`](../features/portfolio-optimizer.md) para asignar capital basado en HRP, Markowitz, Black-Litterman o el Ensamblador Singular D-Score con desacoplo de ATR macro.
*   **Reglas de Orquestación:**
    * La suma de pesos DEBE ser exactamente 1.0 (100%).
    * La inyección de pesos para D-Score aplica normalizaciones basadas en la volatilidad macro para modular el riesgo de cola.
    * Cada cálculo de rebalanceo debe registrar el `process_id` del job y el `audit_hash` correspondiente en la base de datos (ADR-0020).
*   **Entrada:** `active_strategies_list`, `optimization_method`, `atr_multiplier`.
*   **Salida:** `optimal_weights_vector`.
*   **Precondición:** Estrategias promovidas desde el módulo `incubate` en estado `OPERATING`.
*   **Postcondición:** Registro de snapshot de pesos inmutable en la base de datos.

### **TTR-002: Orquestación de Vigilancia de Riesgo y Envolvente de Reglas (Rules Wrapper & Challenge Mode)**
*   **Descripción:** Invoca a [`portfolio-rules`](../features/portfolio-rules.md) para imponer límites soberanos al portafolio y validar las restricciones del challenge o gestión de capital global.
*   **Reglas de Orquestación:**
    * Los límites del portafolio (Hard Limits, Drawdown Diario Regla de Medianoche, Trailing Drawdown, News Blackouts) invalidan cualquier límite de estrategia individual.
    * Todo bloqueo de órdenes debe vincularse al `institutional_tag` de cumplimiento (ADR-0020).
*   **Entrada:** `proposed_portfolio_state`, `hard_limits_config`, `challenge_profile`.
*   **Salida:** `compliance_verdict` (ALLOW | BLOCK).
*   **Precondición:** Pesos optimizados (TTR-001).
*   **Postcondición:** El estado del portafolio se marca como `COMPLIANT` en el DAG de versiones.


### **TTR-003: Orquestación de Adaptación de Régimen (HMM Detection)**
*   **Descripción:** Invoca a [`hmm-regime-detection`](../features/hmm-regime-detection.md) para ajustar el perfil de riesgo según el mercado.
*   **Reglas de Orquestación:**
    * En regímenes de `VOL_EXPANSION`, se deben aplicar factores de reducción de capital (De-risking).
    * El ajuste de riesgo se registra con el `version_node_id` del modelo HMM (ADR-0020).
*   **Entrada:** `current_market_regime`.
*   **Salida:** `risk_multiplier_adjustment`.
*   **Precondición:** Datos de mercado en tiempo real sincronizados.
*   **Postcondición:** Portafolio ajustado para el entorno actual.

### **TTR-004: Orquestación de Persistencia de Portafolio (Databank Manager)**
*   **Descripción:** Utiliza [`databank-manager`](../features/databank-manager.md) para salvar la composición exacta del portafolio.
*   **Reglas de Orquestación:**
    * Cada snapshot de portafolio debe ser content-addressed (hash) para evitar duplicados.
    * Se debe inyectar el `owner_id` y `institutional_tag` en la persistencia del databank.
*   **Entrada:** `optimal_weights_vector`, `active_strategies`.
*   **Salida:** `portfolio_snapshot_id`.
*   **Precondición:** TTR-001 y TTR-002 exitosos.
*   **Postcondición:** Configuración del portafolio recuperable para futuras autopsias.

### **TTR-005: Orquestación de Daemon de Rebalanceo Automático (Portfolio Optimizer)**
*   **Descripción:** Invoca a [`portfolio-optimizer`](../features/portfolio-optimizer.md) para coordinar el Auto-Rebalancing Daemon gobernado por triggers.
*   **Reglas de Orquestación:**
    *   **Gestión de Disparadores:** Escucha cuatro tipos de disparadores:
        *   *Calendario:* Frecuencia optimizada en simulación (semanal, mensual o trimestral).
        *   *Régimen Dinámico HMM:* Dispara si cambia la clasificación del régimen y la confianza estadística supera el umbral optimizado (60/70/80%).
        *   *Desviación (Threshold):* Reequilibrio si la desviación de los pesos actuales respecto al objetivo supera el límite configurable.
        *   *Riesgo (Risk-Trigger):* Activación inmediata si el VaR o CVaR excede los límites de riesgo del portafolio.
    *   **Mitigación Operativa (Circuit Breaker):** Restringe atómicamente a máximo 1 rebalanceo automático por día para evitar sobrecarga por comisiones y deslizamientos.
    *   **Portfolio Variance Check:** Cancela o pospone la ejecución si la varianza del portafolio diario es superior a las dos desviaciones estándar (2σ) (caos de mercado).
    *   **Degradación Elegante:** Si el módulo de detección HMM falla, utiliza el último régimen de mercado conocido como fallback de salvaguarda.
    *   **Persistencia y Auditoría:** Almacena el historial inmutable en la tabla de base de datos dedicada.
    *   **Monitoreo y Alertas:** Inyecta datos al tablero en tiempo real (pesos actuales, régimen detectado, próximo rebalanceo programado) y dispara una alerta de sistema si el deslizamiento realizado supera el doble del deslizamiento esperado.
*   **Entrada:** `rebalance_triggers_config`, `portfolio_variance`, `current_date`, `hmm_confidence`, `var_cvar_state`.
*   **Salida:** `optimal_rebalancing_orders_payload`.
*   **Precondición:** Pesos óptimos HRP/D-Score iniciales calculados y daemon de rebalanceo activo.
*   **Postcondición:** Envío de órdenes de ajuste a la cola de comandos interna de NautilusTrader sin brokers externos.

### **TTR-006: Orquestación de Backtesting Multiestratégico (Portfolio Optimizer)**
*   **Descripción:** Invoca a [`portfolio-optimizer`](../features/portfolio-optimizer.md) para correr backtesting de portafolio agregado.
*   **Reglas de Orquestación:**
    * Corre N estrategias en paralelo con reloj sincronizado y deducción de fricción realista (spreads, comisiones agregadas).
*   **Entrada:** `active_strategies_list`, `frictions_config`.
*   **Salida:** `aggregated_equity_curve`.
*   **Precondición:** Estrategias válidas con series históricas en el databank.
*   **Postcondición:** Curva de equidad agregada disponible para análisis de riesgo.

> **TTR-007 / TTR-008 / TTR-009:** Retirados — numeración reservada, alcance consolidado en TTRs adyacentes durante el refinamiento del backlog del módulo.

### **TTR-010: Orquestación de Reparto de Capital (Portfolio Sizing)**
*   **Descripción:** Utiliza [`precision-sizing-models`](../features/precision-sizing-models.md) para calcular el tamaño de capital asignado a cada "viga" (estrategia) del portafolio.
*   **Reglas de Orquestación:**
    - Traduce los pesos óptimos del optimizador en cantidades nominales basadas en el `Total Equity`.
    - Aplica límites de `Fixed Ratio` o `Risk %` sobre la equidad agregada.
*   **Entrada:** `optimal_weights_vector`, `current_equity_snapshot`.
*   **Salida:** `nominal_capital_allocation`.
*   **Precondición:** TTR-001 (Optimización) finalizado.
*   **Postcondición:** El sistema conoce el límite de exposición exacto por estrategia.

### **TTR-011: Orquestación de Auditoría Narrativa (Self-Explanation)**
*   **Descripción:** Invoca a [`strategy-self-explanation`](../features/strategy-self-explanation.md) para generar un reporte forense humano del AST.
*   **Reglas de Orquestación:**
    *   Prohibido usar esta salida para alterar la ejecución técnica. Su fin es exclusivamente documental.
*   **Entrada:** `validated_strategy_ast`.
*   **Salida:** `human_readable_audit_report`.
*   **Precondición:** Inspección manual de estrategia aprobada.
*   **Postcondición:** Auditoría generada y visualizada para el inspector.

### **TTR-012: Vigilancia Sistémica de Beta (Alpha Decoupling)**
*   **Descripción:** Invoca a [`alpha-decoupling`](../features/alpha-decoupling.md) a nivel macro de portafolio.
*   **Reglas de Orquestación:**
    *   Detecta si, a pesar de estar descorrelacionados entre sí, la suma del portafolio tiene un Beta altísimo al mercado (riesgo sistemático oculto).
*   **Entrada:** `portfolio_equity_curve`, `benchmark_series`.
*   **Salida:** `portfolio_systematic_exposure`.
*   **Precondición:** TTR-001 y TTR-004 finalizados.
*   **Postcondición:** Alerta preventiva en caso de sobreexposición direccional.


### **TTR-013: Orquestación de Diversificación (Signal Correlation Analyzer)**
*   **Descripción:** Invoca a [`signal-correlation-analyzer`](../features/signal-correlation-analyzer.md) para balanceo.
*   **Reglas de Orquestación:**
    *   Si añade una estrategia altamente correlacionada, penaliza el peso de ambas (De-weighting).
*   **Entrada:** `portfolio_signals`.
*   **Salida:** `correlation_penalty_matrix`.
*   **Precondición:** Evaluación de candidatos al portafolio.
*   **Postcondición:** Matrices de covarianza ajustadas.

### **TTR-014: Orquestación de Atribución (Factor Decomposition)**
*   **Descripción:** Invoca a [`factor-decomposition`](../features/factor-decomposition.md) para analizar el portafolio total.
*   **Reglas de Orquestación:**
    *   Separa el rendimiento del portafolio en Factores (Momentum, Value, Volatilidad).
*   **Entrada:** `portfolio_equity_curve`.
*   **Salida:** `factor_exposure_report`.
*   **Precondición:** Rebalanceo mensual.
*   **Postcondición:** Comprensión de fuentes de riesgo.

### **TTR-015: Orquestación de Equidad Global (Equity Curve Tracker)**
*   **Descripción:** Invoca a [`equity-curve-tracker`](../features/equity-curve-tracker.md) nivel portafolio.
*   **Reglas de Orquestación:**
    *   Si el Portafolio Global toca Hard Limit (Drawdown > 20%), fuerza Pausa Sistémica.
*   **Entrada:** `aggregated_daily_pnl`.
*   **Salida:** `portfolio_equity_state`.
*   **Precondición:** Cierre diario.
*   **Postcondición:** Vigilancia general de capital.

### **TTR-016: Orquestación de KPIs Macro (Institutional Metrics)**
*   **Descripción:** Invoca a [`institutional-metrics`](../features/institutional-metrics.md) para calificar la gestión.
*   **Reglas de Orquestación:**
    *   Calcula el Sharpe Combinado y el Omega Ratio de toda la cuenta.
*   **Entrada:** `portfolio_equity_state`.
*   **Salida:** `macro_kpis`.
*   **Precondición:** Fin de semana/mes.
*   **Postcondición:** Reporte para inversores/stakeholders.

### **TTR-017: Orquestación de Versionado (Strategy Versioning)**
*   **Descripción:** Invoca a [`strategy-versioning`](../features/strategy-versioning.md) para el Macro-DAG.
*   **Reglas de Orquestación:**
    *   Cada combinación de pesos genera un `version_node_id` de Nivel Portafolio.
*   **Entrada:** `optimal_weights_vector`.
*   **Salida:** `portfolio_dag_node`.
*   **Precondición:** Pesos optimizados.
*   **Postcondición:** Historial inmutable de composiciones.

### **TTR-018: Orquestación de Auditoría de Gestión (Audit Log)**
*   **Descripción:** Invoca a [`audit-log`](../features/audit-log.md) para firmar cambios manuales.
*   **Reglas de Orquestación:**
    *   Cualquier intervención humana sobre los pesos requiere un hash criptográfico.
*   **Entrada:** `manual_weight_override`.
*   **Salida:** `audit_hash`.
*   **Precondición:** Usuario fuerza rebalanceo.
*   **Postcondición:** Intervención registrada permanentemente.

### **TTR-019: Orquestación de Promoción Directa (Bypass MOD-04)**
*   **Descripción:** Permite promover estrategias directamente al módulo de gestión.
*   **Reglas de Orquestación:**
    * Exige que el `audit_hash` de la serie de retornos esté presente y validado.
*   **Entrada:** `external_strategy_results`, `validation_metadata`.
*   **Salida:** `promoted_strategy_id`.
*   **Postcondición:** Estrategia añadida al databank para optimización HRP.

### **TTR-020: Orquestación de Validación Visual (Dendrogram / Heatmap)**
*   **Descripción:** Genera y exporta la matriz de distancias y el dendrograma de las estrategias del portafolio.
*   **Reglas de Orquestación:**
    * Utiliza los datos del `portfolio-data-preparation` para visualización frontend.
*   **Entrada:** `optimal_weights_vector`, `correlation_matrix`.
*   **Salida:** `visual_dendrogram_json`.
*   **Postcondición:** Visualización interactiva disponible en la UI.

### **TTR-021: Orquestación de Métricas de Riesgo Avanzadas (Portfolio Optimizer)**
*   **Descripción:** Calcula métricas avanzadas de concentración y riesgo de cola del portafolio.
*   **Reglas de Orquestación:**
    * Calcula el Índice Herfindahl de concentración, CVaR y descomposición estacional.
*   **Entrada:** `portfolio_returns_series`.
*   **Salida:** `concentration_and_tail_risk_metrics`.

### **TTR-022: Orquestación de Clustering K-Means y hrp_rank (Portfolio Optimizer)**
*   **Descripción:** Agrupa estrategias vía KMeans y asigna rango según su importancia en la diversificación.
*   **Reglas de Orquestación:**
    * Aplica KMeans sobre la matriz de correlación para etiquetar familias de estrategias y calcular `hrp_rank`.
*   **Entrada:** `correlation_matrix`.
*   **Salida:** `hrp_rank_and_clusters`.

### **TTR-023: Orquestación de Versionado de Portafolio Git-Like (Strategy Versioning)**
*   **Descripción:** Persiste y ramifica composiciones inmutables de portafolio en `portfolios.parquet`.
*   **Reglas de Orquestación:**
    * Registra el snapshot de configuración con `portfolio_id`, `version_hash`, `parent_hash`, `branch_name`.
*   **Entrada:** `portfolio_composition_data`.
*   **Salida:** `new_portfolio_version_hash`.

### **TTR-024: Orquestación del Advanced Trade Management (ATM)**
*   **Descripción:** Invoca a [`advanced-trade-management`](../features/advanced-trade-management.md) para definir la configuración base de las órdenes de Grid, Hedging y Trailing Stop del Portafolio.
*   **Reglas de Orquestación:**
    - Los parámetros de Grid y Hedging calculados por el portafolio se inyectan en las órdenes del Autopilot.
*   **Entrada:** `portfolio_allocation_data`.
*   **Salida:** `atm_order_parameters`.
*   **Precondición:** Pesos óptimos calculados (TTR-001).
*   **Postcondición:** Parámetros listos para la ejecución real.

### **TTR-025: Orquestación de Hedging Cointegrativo (Portfolio Optimizer)**
*   **Descripción:** Invoca a [`portfolio-optimizer`](../features/portfolio-optimizer.md) para ejecutar la intercepción y bloqueo de órdenes redundantes por cointegración de alta frecuencia.
*   **Reglas de Orquestación:**
    * Analiza nano-solapamientos destructivos (+0.85) intra-segundo entre pares operados.
    * Bloquea de forma inmediata el margen asignado reduciendo el volumen de las órdenes conflictivas a cero con latencia < 2ms.
*   **Entrada:** `live_order_event_stream`, `cointegration_threshold`.
*   **Salida:** `hedged_order_command`.
*   **Precondición:** Canales FFI en caliente operando con el backend.
*   **Postcondición:** Protección de la cuenta master contra sobreexposiciones correlacionadas.

### **TTR-026: Orquestación de Router de Liquidez (Portfolio Optimizer)**
*   **Descripción:** Invoca a [`portfolio-optimizer`](../features/portfolio-optimizer.md) para rotar la asignación de capital desde mercados planos hacia activos tendenciales.
*   **Reglas de Orquestación:**
    * Evalúa la predecibilidad del mercado. Si un par entra en lateralidad sin alfa mayor a 72 horas, conmuta el balance disponible.
    * Rota el capital vía llamadas de API seguras a los exchanges autorizados hacia los vectores exóticos o commodities configurados.
*   **Entrada:** `market_predictibility_index`, `laterality_timeout_config`.
*   **Salida:** `capital_reallocation_orders`.
*   **Precondición:** Cierre de sesiones abiertas inactivas y margen libre validado.
*   **Postcondición:** Aprovechamiento óptimo de capital líquido trans-activo.

### **TTR-027: Orquestación de Búsqueda Genética de Portafolios (Portfolio Optimizer)**
*   **Descripción:** Invoca a [`portfolio-optimizer`](../features/portfolio-optimizer.md) para ejecutar la búsqueda evolutiva de combinaciones óptimas de estrategias del Databank.
*   **Reglas de Orquestación:**
    *   Somete la población a generaciones genéticas usando como filtros el límite de cantidad de estrategias por portafolio y el multiplicador de Retorno/DD objetivo.
    *   Califica el fitness global aplicando la misma Weighted Fitness Formula configurable.
*   **Entrada:** `databank_strategies_list`, `genetic_search_config`.
*   **Salida:** `candidate_portfolios_report`.
*   **Precondición:** Databank con estrategias con histórico persistido.
*   **Postcondición:** Reportes de portafolios guardados en formato Parquet.

### **TTR-028: Orquestación de Análisis de Solapamiento Temporal Real (Portfolio Optimizer)**
*   **Descripción:** Invoca a [`portfolio-optimizer`](../features/portfolio-optimizer.md) para evaluar la superposición temporal real de tickets abiertos.
*   **Reglas de Orquestación:**
    *   Ejecuta consultas DuckDB vectorizadas para medir solapamientos, duración, y riesgo máximo.
    *   Aplica filtros de rechazo si la superposición supera los límites permitidos.
*   **Entrada:** `portfolio_trades_history`, `overlap_config`.
*   **Salida:** `overlap_matrix_report`.
*   **Postcondición:** Reporte de solapamiento adjunto a la configuración del portafolio.

### **TTR-029: Orquestación de Rescalado de Pesos y Ledger de Simulación (Portfolio Optimizer)**
*   **Descripción:** Invoca a [`portfolio-optimizer`](../features/portfolio-optimizer.md) para transformar pesos teóricos en lotes operativos exactos.
*   **Reglas de Orquestación:**
    *   Construye el Ledger de simulación continuo por hora integrando balances y márgenes cruzados.
*   **Entrada:** `weights_vector`, `account_margin_rules`.
*   **Salida:** `simulation_ledger_report`.
*   **Postcondición:** Ledger persistido para auditorías de margen.

### **TTR-030: Orquestación de Portafolios Federados (Federated Portfolio)**
*   **Descripción:** Integra la gestión y control del clúster de portafolios federados consumiendo [`federated-portfolio`](../features/federated-portfolio.md) y [`portfolio-rules`](../features/portfolio-rules.md).
*   **Reglas de Orquestación:**
    *   Registra, inicializa y actualiza los contenedores lógicos independientes en la tabla relacional SQLite `portfolio_containers`.
    *   Orquesta los triggers locales del daemon de rebalanceo y coordina la recolección de telemetría y consolidación analítica inter-portafolio.
    *   Gestiona el ruteo del Kill Switch Global cancelando y liquidando posiciones en paralelo.
*   **Entrada:** `federated_config_json`, `cluster_telemetry_stream`.
*   **Salida:** `federated_containers_status`, `aggregated_cluster_metrics`.
*   **Postcondición:** Estado de contenedores inmutable en persistencia y telemetría de clúster expuesta a la UI.

### **TTR-031: Orquestación de Simulación de Portafolio Real (Portfolio Backtest)**
*   **Descripción:** Orquesta la validación histórica multiestrategia consumiendo [`portfolio-backtest`](../features/portfolio-backtest.md) y [`backtest-engine`](../features/backtest-engine.md).
*   **Reglas de Orquestación:**
    - Carga las estrategias del portafolio y los feeds de mercado históricos alineados temporalmente.
    - Configura el pool de capital y el método de compounding dinámico.
    - Ejecuta el backtest en NautilusTrader evaluando las restricciones de margen y las sesiones de mercado.
*   **Entrada:** `portfolio_strategies_config`, `backtest_capital_rules`.
*   **Salida:** `real_portfolio_backtest_report`.
*   **Postcondición:** Reporte de rendimiento consolidado del portafolio analizado y disponible para validación pesada.

### **TTR-032: Orquestación de Protocolo de Acceso Remoto (RPAP)**
*   **Descripción:** Invoca a [`remote-portfolio-access-protocol`](../features/remote-portfolio-access-protocol.md) para levantar y gestionar el endpoint seguro de consultas P2P.
*   **Reglas de Orquestación:**
    *   Gestiona la validación del JWT e impone el rate limiting configurado para conexiones entrantes de empleados o copiers.
    *   Filtra las consultas a la base de datos de manera proactiva (Field Masking) y delega a DuckDB la computación analítica sin cargar memoria persistente.
*   **Entrada:** `jwt_token`, `remote_analytics_query`.
*   **Salida:** `masked_analytics_response`.
*   **Postcondición:** Datos operativos exportados según scope permitido, transacción auditada inmutablemente en el `rpap_access_log`.

### **TTR-033: Orquestación del Fitness Contextual de Portafolio (Contextual Fitness Scorer)**
*   **Descripción:** Invoca a [`contextual-fitness-scorer`](../features/contextual-fitness-scorer.md) para evaluar qué regímenes quedan débiles en el portafolio actual y priorizar candidatas que los cubran.
*   **Reglas de Orquestación:**
    *   Disecciona la equidad consolidada del portafolio por régimen y proyecta el radar multidimensional.
    *   Señala el régimen de mayor vulnerabilidad del portafolio como criterio de admisión de nuevas estrategias.
*   **Entrada:** `portfolio_equity_curve`, `regime_classification`, `regime_priority_map`.
*   **Salida:** `portfolio_multidimensional_score`, `weakest_regime`.
*   **Precondición:** TTR-001 (Optimización de pesos) finalizado.
*   **Postcondición:** Vulnerabilidad por régimen registrada y disponible para la decisión de composición.

### **TTR-034: Orquestación del Genoma de Portafolio y Correlación (ADR-0108/ADR-0111)**
*   **Descripción:** Cuando `ACTIVE_GENOME_DOMAINS` incluye Portafolio y Correlación (co-evolución de cartera), conecta los Genes de Condición de Estado del dominio con sus Genes de Acción operativos sobre la cartera en gestión.
*   **Reglas de Orquestación:**
    *   Lee la correlación de curva de equidad rodante por miembro desde [`fit-to-portfolio-search`](../features/fit-to-portfolio-search.md) (TTR-002) y el `RuleVerdict`/`RiskStatusEnum` agregado de cartera desde [`portfolio-rules`](../features/portfolio-rules.md) (TTR-004) como Genes de Condición de Estado.
    *   Aplica los Genes de Acción resueltos por la co-evolución (activación/desactivación de miembro, rotación de pesos, cobertura sintética) invocando [`portfolio-optimizer`](../features/portfolio-optimizer.md) (TTR-010, TTR-011).
    *   Cuando ese genoma no está activo, este TTR es un no-op: TTR-001/TTR-002/TTR-005 operan sobre sus disparadores FIJOS sin cambios.
*   **Entrada:** `portfolio_correlation_state`, `RuleVerdict`, `RiskStatusEnum`, `ACTIVE_GENOME_DOMAINS`.
*   **Salida:** `portfolio_genome_action_orders`.
*   **Precondición:** TTR-001 (pesos óptimos) y TTR-002 (veredicto de reglas) finalizados.
*   **Postcondición:** Órdenes de activación/desactivación, rotación de pesos y/o cobertura sintética encoladas para `execute`.

### **TTR-035: Orquestación de Acceso Agéntico vía MCP (Cabina Dual — Permiso Condicionado)**
*   **Descripción:** Invoca a [`agentic-mcp-gateway`](../features/agentic-mcp-gateway.md) para evaluar el permiso antes de aceptar una llamada proveniente del canal MCP sobre la `public_interface` de este módulo.
*   **Reglas de Orquestación:**
    * `manage` no separa una porción de producción en su lógica (los mismos TTR de pesos/reglas operan igual para cualquier portafolio); el permiso se condiciona al `institutional_tag` del portafolio objetivo de la llamada (ADR-0020, ADR-0123).
    * Si el objetivo tiene `institutional_tag = Demo`, la llamada se concede sin gate adicional.
    * Si el objetivo tiene `institutional_tag = Live`, la llamada se rechaza salvo que `PRODUCTION_OVERRIDE` esté activo en ese momento.
    * Toda llamada (concedida o rechazada) queda auditada con su procedencia agente (`agent_session_id`) y el `institutional_tag` evaluado.
*   **Entrada:** Llamada MCP entrante con pipeline `manage`, `institutional_tag` del portafolio objetivo, estado de `PRODUCTION_OVERRIDE`.
*   **Salida:** Resultado de la operación enrutado al agente, o rechazo con motivo, + registro de auditoría de procedencia.

### **TTR-036: Integración del Aporte Fundamental en Ponderación de Portafolio (Exposure Map + Indicator Projector)**
*   **Descripción:** TTR de Integración (ADR-0118): consume el [`asset-exposure-map`](../features/asset-exposure-map.md) y el [`fundamental-indicator-projector`](../features/fundamental-indicator-projector.md) vía sus `public_interface` para ponderar riesgo/exposición del portafolio según la concentración de eventos fundamentales relevantes sobre sus activos.
*   **Reglas de Orquestación:**
    * No reconstruye las features: las consume por sus puertos (Soberanía de Datos).
    * La relevancia evento→activo se usa para detectar concentración de exposición fundamental simultánea entre estrategias del portafolio (ADR-0128).
    * El ajuste de pesos respeta los límites duros/blandos del portafolio (ADR-0010).
*   **Entrada:** `event_asset_relevance` y `fundamental_indicator_series` de los activos del portafolio.
*   **Salida:** Pesos/exposición ajustados por concentración de aporte fundamental.
*   **Precondición:** Indicador y relevancia disponibles vía `public_interface` de `generate`.
*   **Postcondición:** Portafolio con riesgo sensible a eventos fundamentales concentrados, auditable.

---


## Gobernanza y Estándares (Fijos)

- **Inundación de Fundamentos (ADR-0020):** El catálogo de los 25 campos maestros está en la sección "Épica 0: Esqueleto Fundacional" de este documento (referencia, no esquema). Toda entidad persistida por este módulo incluye el Grupo I de forma universal; los Grupos II–V se aplican solo en los campos que el Perfil Técnico de cada feature exige (Filtro de Relevancia, ADR-0020) — nunca el catálogo completo.

- **Decisión Arquitectónica Asociada:**
    - ADR-0005: Versionamiento Reproducible (Snapshots de pesos).
    - ADR-0010: Hard vs Soft Limits (Soberanía de reglas).
    - ADR-0020: Inundación de Fundaciones.
    - ADR-0108 / ADR-0111: Genoma de Portafolio y Correlación (TTR-034).

---

## Dependencias
**Depende de:**
- [`incubate`](../modules/incubate.md) — para la recepción de estrategias probadas.

**Consumido por:**
- [`execute`](../modules/execute.md) — para la aplicación de pesos y límites en ejecución real.
- [`withdraw`](../modules/withdraw.md) — para la gestión de salida de capital del portafolio.
