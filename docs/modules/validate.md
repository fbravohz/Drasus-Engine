# Validar

**Carpeta:** `./modules/validate/`
**Estado:** Orquestador de Robustez (Scoring Ponderado)
**Última actualización:** 2026-06-11

---

## ¿Qué es?

El módulo de validación es el entorno de certificación de robustez de las estrategias. Una estrategia candidata llega aquí y se somete a un **guantelete de 5 tests estadísticos** con pesos diferenciados. El objetivo es asegurarse de que la estrategia no solo funcionó bien en el pasado por suerte, sino que tiene robustez real y puede funcionar con datos nuevos.

El enfoque es un **Scoring Ponderado (0-100)** que reemplaza la vieja "Muerte Súbita" binaria. Los 5 tests reciben pesos configurables (WFA 30%, MC Trades 25%, MC Tóxico 20%, CPCV/PBO 15%, Ockham 10%) y se consolidan en un score final. Estrategias con score > 75 son "Aprobables".

Un **LLM local** genera un veredicto en lenguaje natural, identifica puntos de ruptura y explica el score.

El veredicto final — APROBADA, EN REVISIÓN, o RECHAZADA — es permanente. No se puede cambiar retroactivamente. El score determina el dimensionamiento de posición inicial en el módulo de ejecución.

---

## Comportamientos Observables

- [ ] Envío una estrategia candidata → el sistema la somete a todas las pruebas y devuelve un veredicto
- [ ] El veredicto tiene tres posibles valores: APROBADA (sigue al siguiente paso), EN REVISIÓN (el usuario decide), o RECHAZADA (vuelve a generar)
- [ ] Una vez que el veredicto está emitido, no puede cambiarse aunque corra el backtesting de nuevo
- [ ] Si la misma estrategia (con los mismos parámetros exactos) ya fue probada antes, el sistema reutiliza ese resultado sin volver a calcularlo
- [ ] Puedo ver el historial completo de todas las pruebas realizadas a una estrategia

---

## Restricciones

- **FIJO — NO CONFIGURABLE:** El veredicto de una estrategia es inmutable. Una vez emitido, no cambia aunque corras el proceso de nuevo.
- Si el score de robustez está por debajo del mínimo configurable, la estrategia es rechazada automáticamente
- Los resultados heredados (calculados en versiones anteriores con los mismos parámetros) se marcan explícitamente como "heredados"

---

## Parámetros Configurables

| Parámetro | Default | Qué hace |
|---|---|---|
| MIN_ROBUSTNESS_SCORE | configurable | Score mínimo para pasar (ej: 70 sobre 100) |
| MONTE_CARLO_ITERATIONS | configurable | Cuántos escenarios simula el Monte Carlo |
| WFA_WINDOWS | configurable | Cuántas ventanas de tiempo usa el análisis deslizante |
| DSR_THRESHOLD | configurable | Umbral de Deflated Sharpe Ratio para anti-sobreajuste |

---

## Features Consumidas (Reutilizables)

- **[`pit-data-validator`](../features/pit-data-validator.md)** — Validación de datos Point-In-Time.
- **[`backtest-engine`](../features/backtest-engine.md)** — Simulación de alta fidelidad 4-ticks.
- **[`walk-forward-analyzer`](../features/walk-forward-analyzer.md)** — Orquestación de ventanas móviles.
- **[`monte-carlo-simulator`](../features/monte-carlo-simulator.md)** — Análisis de robustez por remuestreo.
- **[`factor-decomposition`](../features/factor-decomposition.md)** — Descomposición Alpha/Beta y Atribución.
- **[`signal-correlation-analyzer`](../features/signal-correlation-analyzer.md)** — Diversificación de señales.
- **[`equity-curve-tracker`](../features/equity-curve-tracker.md)** — Tracking de capital y PnL.
- **[`institutional-metrics`](../features/institutional-metrics.md)** — Motor asimétrico de KPIs estadísticos.
- **[`prop-firm-grader`](../features/prop-firm-grader.md)** — Filtro intransigente de fondeo (Drawdown diario, Profit Factor).
- **[`vector-time-pruning`](../features/vector-time-pruning.md)** — Poda de ventanas temporales tóxicas recurrentes.
- **[`strategy-versioning`](../features/strategy-versioning.md)** — Gestión del DAG y persistencia.
- **[`audit-log`](../features/audit-log.md)** — Registro inmutable de veredictos.
- **[`universal-basket-backtester`](../features/universal-basket-backtester.md)** — Certificación de robustez en canastas multi-activo.
- **[`duckdb-sql-engine`](../features/duckdb-sql-engine.md)** — Cálculo masivo de KPIs estadísticos.
- **[`visual-downsampling-service`](../features/visual-downsampling-service.md)** — Preparación de curvas de equidad para visualización.
- **[`volume-profile-router`](../features/volume-profile-router.md)** — Simulación de vetos por liquidez y deslizamiento.
- **[`adaptive-volume-indicators`](../features/adaptive-volume-indicators.md)** — Validación en regímenes de baja/alta liquidez.
- **[`ast-compiler`](../features/ast-compiler.md)** — Validación Zero-Trust y compilación de estrategias (AOT/JIT).
- **[`nautilus-integration`](../features/nautilus-integration.md)** — Motor institucional para simulación de eventos deterministas.
- **[`precision-sizing-models`](../features/precision-sizing-models.md)** — Simulación de dimensionamiento (Fixed Ratio, ATR, % Riesgo) para pruebas de robustez.
- **[`alpha-decoupling`](../features/alpha-decoupling.md)** — Neutralización analítica de Beta.
- **[`cross-market-validation`](../features/cross-market-validation.md)** — Estrés de robustez en mercados correlacionados.
- **[`perfect-profit-benchmark`](../features/perfect-profit-benchmark.md)** — Eficiencia del modelo (Pardo).
- **[`component-isolation`](../features/component-isolation.md)** — Monkey Test de entradas y salidas.
- **[`complexity-penalization`](../features/complexity-penalization.md)** — Navaja de Ockham paramétrica.
- **[`topological-plateau-finder`](../features/topological-plateau-finder.md)** — Búsqueda geométrica de mesetas robustas.
- **[`hierarchical-parameter-optimization`](../features/hierarchical-parameter-optimization.md)** — Optimización en cascada controlada.
- **[`adversarial-noise-agent`](../features/adversarial-noise-agent.md)** — Red Team AI para slippage y volatilidad.
- **[`fragility-gradient-auditor`](../features/fragility-gradient-auditor.md)** — Auditoría de inestabilidad paramétrica.
- **[`adaptive-logic-er`](../features/adaptive-logic-er.md)** — Filtrado por Efficiency Ratio (Kaufman).
- **[`robustness-score-aggregator`](../features/robustness-score-aggregator.md)** — Motor de Scoring Ponderado 0-100 (ADR-0058).
- **[`robustness-verdict-engine`](../features/robustness-verdict-engine.md)** — Veredictos en lenguaje natural vía LLM local (ADR-0058).
- **[`incremental-test-engine`](../features/incremental-test-engine.md)** — Optimización de ejecución acumulativa y herencia de resultados (ADR-0060).
- **[`cpcv-analyzer`](../features/cpcv-analyzer.md)** — Motor de validación cruzada combinatorial con Purging y Embargo (ADR-0063).
- **[`statistical-inference-ebta`](../features/statistical-inference-ebta.md)** — Capa de inferencia estadística avanzada (DSR, Romano-Wolf, Detrender).
- **[`volatility-stabilization`](../features/volatility-stabilization.md)** — Certificación de Target Vol y estabilidad bajo regímenes (ADR-0068).
- **[`institutional-friction-modeling`](../features/institutional-friction-modeling.md)** — Modelado de Adverse Selection y Probabilistic Fills (ADR-0069).
- **[`operational-safety-monitor`](../features/operational-safety-monitor.md)** — Pardo Profile Monitor y Strategy Stop-Loss (SSL) (ADR-0070).
- **[`parallel-coordinates-visualizer`](../features/parallel-coordinates-visualizer.md)** — Visualización y brushing interactivo de optimizaciones de alta dimensión.
- **[`cross-filtering-visualizer`](../features/cross-filtering-visualizer.md)** — Histogramas interactivos coordinados de múltiples parámetros.
- **[`autoencoder-outlier-detector`](../features/autoencoder-outlier-detector.md)** — Detección de trades anómalos mediante reconstrucción neuronal profunda (ADR-0074).
- **[`design-manifest`](../features/design-manifest.md)** — Contratos de metas SMART y validación de calidad final (ADR-0053).
- **[`time-warp-debugger`](../features/time-warp-debugger.md)** — Navegación temporal ultra-rápida y depuración forense mediante DuckDB Partition Pruning.
- **[`umap-scatter-visualizer`](../features/umap-scatter-visualizer.md)** — Visualizador interactivo GPU de estabilidad en nubes de puntos UMAP.
- **[`toxicity-purifier`](../features/toxicity-purifier.md)** — Panel de control y purga de clústeres tóxicos PCA con snapshots.
- **[`interactive-stress-lab`](../features/interactive-stress-lab.md)** — Deformación táctil de la curva de capital en tiempo real (sliders de fricción y macro).
- **[`plateau-copilot`](../features/plateau-copilot.md)** — Mapa de calor 2D con sugerencia de meseta y selección manual del parámetro de producción.
- **[`manual-regime-tagger`](../features/manual-regime-tagger.md)** — Etiquetado visual de crisis y reglas duras de desempeño por zona.
- **[`contextual-fitness-scorer`](../features/contextual-fitness-scorer.md)** — Fitness multidimensional diseccionado por régimen (radar).

---



---

## Tareas (TTRs) — Protocolo de Orquestación (§8.1)

### **TTR-001: Orquestación de Compilación y Validación Zero-Trust (AST Compiler)**
*   **Descripción:** Invoca al [`ast-compiler`](../features/ast-compiler.md) para validar la integridad del diseño visual antes de cualquier prueba.
*   **Reglas de Orquestación:**
    * Coordina el uso del **Bloque Factory (AOT)** para nodos estándar y el **Escape-Hatch (JIT)** para lógica personalizada.
    * Genera la firma de integridad (`manifest_id`) vinculada al `logic_hash` institucional (ADR-0020 V2).
*   **Entrada:** `Visual_Graph_JSON`.
*   **Salida:** `StrategyExecutableObject`, `manifest_id`.
*   **Precondición:** Recepción de candidato desde MOD-02.
*   **Postcondición:** Estrategia certificada para el "Hot-Path" de simulación.

### **TTR-002: Orquestación de Integridad de Simulación (PIT Validator)**
*   **Descripción:** Invoca a [`pit-data-validator`](../features/pit-data-validator.md) para garantizar que el backtest es "limpio" de look-ahead bias.
*   **Reglas de Orquestación:**
    * Si el dataset falla, se marca la validación como `DATA_CONTAMINATED` y se bloquea el proceso.
    * Toda certificación PIT exitosa se registra con el `audit_hash` del dataset (ADR-0020 V2).
*   **Entrada:** `candidate_strategy`, `history_dataset`.
*   **Salida:** `pit_certification_status`.
*   **Precondición:** Estrategia en estado `TESTING`.
*   **Postcondición:** Dataset habilitado para el motor de backtesting.

### **TTR-003: Orquestación de Inferencia Histórica (Backtest Engine)**
*   **Descripción:** Invoca al [`backtest-engine`](../features/backtest-engine.md) para generar the track-record de fills.
*   **Reglas de Orquestación:**
    * El motor debe utilizar la precisión de 4-ticks reglamentaria y el **Bar-Open Alignment** obligatorio para paridad 1:1.
    * Todo fill generado DEBE incluir el `process_id` de la simulación y diferenciar entre **Settlement e Histórico (Davey)**.
*   **Entrada:** `candidate_strategy`, `certified_history`.
*   **Salida:** `trade_fill_log`.
*   **Precondición:** TTR-001 finalizado.
*   **Postcondición:** trade_fill_log inmutable persistido en `test_results`.

### **TTR-004: Orquestación del Guantelete de Robustez Decagonal (HPC Hybrid MC)**
*   **Descripción:** Invoca secuencialmente a los 5 tests del guantelete de robustez: [`walk-forward-analyzer`](../features/walk-forward-analyzer.md) (Modo WFA), [`monte-carlo-simulator`](../features/monte-carlo-simulator.md) (Modo HPC Hybrid), [`cpcv-analyzer`](../features/cpcv-analyzer.md) (Modo PBO), y [`complexity-penalization`](../features/complexity-penalization.md) (Ockham).
*   **Reglas de Orquestación:**
    * **HPC Execution:** Coordina la permutación masiva y las 10 perturbaciones dinámicas en **CPU (`ndarray` + Rust SIMD/Rayon)** por defecto; GPU vía `candle` solo como acelerador opcional si un benchmark lo justifica (ADR-0061/0112).
    * **CPCV Protocol:** Invoca al [`cpcv-analyzer`](../features/cpcv-analyzer.md) para particionar la historia en miles de caminos no lineales, aplicando **Purging** y **Embargo** (ADR-0063).
    * **Decagonal Protocol:** Asegura la aplicación de las 10 transformaciones obligatorias (Trade Reordering, Shock Injection 3.5x ATR, Outlier Removal, etc.).
    * **Toxic Funnel:** Inyecta las reglas de "Muerte Súbita" intradiaria (Drawdown > 4.5%) durante la corrida de simulación para descartar cohortes inoperables.
    * Cada test produce un score individual normalizado (0-100) que se pasa al [`robustness-score-aggregator`](../features/robustness-score-aggregator.md).
*   **Entrada:** `trade_fill_log`.
*   **Salida:** `individual_test_scores` (5 scores 0-100), `mc_distribution_curves`.
*   **Precondición:** TTR-002 finalizado exitosamente.
*   **Postcondición:** 5 scores individuales disponibles para el motor de scoring ponderado (TTR-032).

### **TTR-005: Orquestación de Reporte Visual (Downsampling & Arrow)**
*   **Descripción:** Invoca a [`visual-downsampling-service`](../features/visual-downsampling-service.md) para generar el dataset de reporte visual.
*   **Reglas de Orquestación:**
    * El backend debe enviar los resultados de la validación profunda via [`binary-arrow-transport`](../features/binary-arrow-transport.md).
    * Solo se envían los picos y valles críticos para permitir una navegación fluida en el nivel de Strategy Inspector.
*   **Entrada:** `test_results` (Equity Curve completa).
*   **Salida:** `downsampled_visual_report`.
*   **Precondición:** TTR-004 finalizado.
*   **Postcondición:** Resultados listos para visualización instantánea en la UI.

### **TTR-006: Auditoría de Generalización (Basket Stress Test)**
*   **Descripción:** Invoca a [`universal-basket-backtester`](../features/universal-basket-backtester.md) como filtro final de aprobación institucional.
*   **Reglas de Orquestación:**
    * Si la correlación de la equidad entre activos de la canasta es > 0.9, se emite una alerta de "Falsa Diversificación".
    * El veredicto de aprobación requiere que el Sharpe sea positivo en al menos el 80% de los activos de la canasta.
*   **Entrada:** `candidate_strategy`, `institutional_basket`.
*   **Salida:** `generalization_audit_report`.
*   **Precondición:** TTR-004 finalizado.
*   **Postcondición:** Inclusión del reporte de canasta en el `audit_hash` final.

### **TTR-007: Orquestación de Auditoría de Liquidez (Volume Profile Stress Test)**
*   **Descripción:** Invoca a [`volume-profile-router`](../features/volume-profile-router.md) para simular el impacto de la falta de liquidez en el track-record histórico.
*   **Reglas de Orquestación:**
    * El sistema debe identificar periodos de "huecos" de volumen y re-evaluar si las órdenes se hubieran llenado con el deslizamiento esperado.
    * Si el retorno ajustado por riesgo de liquidez cae > 20% vs backtest estándar, la estrategia se marca como `LOW_LIQUIDITY_FRAGILE`.
*   **Entrada:** `trade_fill_log`, `historical_volume_profiles`.
*   **Salida:** `liquidity_stress_report`.
*   **Precondición:** TTR-003 finalizado.
*   **Postcondición:** Reporte de liquidez adjunto al veredicto final.
### **TTR-008: Orquestación del Kernel de Simulación (Nautilus Integration)**
*   **Descripción:** Invoca a [`nautilus-integration`](../features/nautilus-integration.md) para ejecutar el loop de eventos determinista.
*   **Reglas de Orquestación:**
    * El kernel debe sincronizar los relojes de todos los componentes antes de iniciar la simulación.
    * Cada tick procesado debe ser auditable vía `event_sequence_id` (ADR-0020 V2).
*   **Entrada:** `simulation_config`, `data_feed`.
*   **Salida:** `event_loop_status`.
*   **Precondición:** Datos certificados (TTR-002) disponibles.
*   **Postcondición:** Simulación ejecutada con paridad institucional.

> **TTR-009 / TTR-010:** Retirados — numeración reservada, alcance consolidado en TTRs adyacentes durante el refinamiento del backlog del módulo.

### **TTR-011: Orquestación de Validación de Sizing (Backtest Calibration)**
*   **Descripción:** Utiliza [`precision-sizing-models`](../features/precision-sizing-models.md) para recalcular el track-record histórico bajo diferentes políticas de riesgo.
*   **Reglas de Orquestación:**
    - Verifica si una estrategia robusta en lotaje fijo (Fixed Lot) colapsa bajo `Fixed Ratio` o `Risk %`.
    - Detecta si el dimensionamiento de posición induce un Drawdown catastrófico debido a la volatilidad del activo (ATR Calibration).
*   **Entrada:** `trade_fill_log`, `sizing_config_matrix`.
*   **Salida:** `robustness_sizing_report`.
*   **Precondición:** TTR-003 (Backtest Engine) finalizado.
*   **Postcondición:** Reporte de sensibilidad de sizing adjunto al veredicto final.

### **TTR-012: Orquestación del Filtro de Fondeo (Prop-Firm Grader)**
*   **Descripción:** Invoca a [`prop-firm-grader`](../features/prop-firm-grader.md) durante la validación para evaluar límites de capital de firmas de fondeo.
*   **Reglas de Orquestación:**
    - Inyecta la configuración `PropFirmComplianceConfig` activa en el motor.
    - Si se viola el Drawdown Diario o Profit Factor, aborta el test inmediatamente (Short-Circuit) y emite veredicto `RECHAZADA`.
*   **Entrada:** `trade_fill_log`, `PropFirmComplianceConfig`.
*   **Salida:** `compliance_status_id`.
*   **Precondición:** TTR-003 finalizado.
*   **Postcondición:** Estrategia validada o descartada tempranamente por violación de regla de fondeo.

### **TTR-013: Orquestación de Poda Temporal (Vector-Time Pruning)**
*   **Descripción:** Invoca a [`vector-time-pruning`](../features/vector-time-pruning.md) post-simulación para detectar anomalías horarias y emitir bloqueos futuros.
*   **Reglas de Orquestación:**
    - Extrae ventanas con Z-Score negativo crónico que superen las ocurrencias mínimas.
    - Anexa la lista de ventanas prohibidas a los metadatos de la estrategia para ser forzadas en los módulos `incubate` y `execute`.
*   **Entrada:** `trade_fill_log`.
*   **Salida:** `pruned_time_vectors_list`.
*   **Precondición:** TTR-003 finalizado.
*   **Postcondición:** Regla de tiempo de ejecución (prohibición) persistida en el DAG de la estrategia.

### **TTR-014: Orquestación de Decoupling Inercial (Alpha Decoupling)**
*   **Descripción:** Invoca a [`alpha-decoupling`](../features/alpha-decoupling.md) para separar el rendimiento propio del sesgo direccional.
*   **Reglas de Orquestación:**
    *   Exige un benchmark configurado para evaluar la estrategia.
    *   Si el Alpha Puro es inferior al `MIN_PURE_ALPHA`, se devalúa el score de robustez.
*   **Entrada:** `test_results`, `benchmark_series`.
*   **Salida:** `pure_alpha_score`, `beta_exposure`.
*   **Precondición:** TTR-004 (Robustez) finalizado.
*   **Postcondición:** Alpha Puro reportado en el veredicto final.

### **TTR-015: Orquestación de Canasta Correlacionada (Cross-Market)**
*   **Descripción:** Invoca a [`cross-market-validation`](../features/cross-market-validation.md) para ejecutar la prueba de fuego transversal.
*   **Reglas de Orquestación:**
    *   Ejecuta el backtest con los mismos parámetros en el activo hermano.
    *   Aborta la validación si la caída de rendimiento excede `MAX_DEGRADATION`.
*   **Entrada:** `candidate_strategy`, `correlated_basket_data`.
*   **Salida:** `robustness_degradation_matrix`.
*   **Precondición:** TTR-003 finalizado.
*   **Postcondición:** Veredicto validado contra curve-fitting.

### **TTR-016: Orquestación de Descomposición (Factor Decomposition)**
*   **Descripción:** Invoca a [`factor-decomposition`](../features/factor-decomposition.md) en test de robustez.
*   **Reglas de Orquestación:**
    *   Filtra estrategias que solo capturan Beta (riesgo de mercado puro).
*   **Entrada:** `simulated_equity_curve`, `market_returns`.
*   **Salida:** `factor_loadings_report`.
*   **Precondición:** Simulación completada.
*   **Postcondición:** Atribución de rendimiento.

### **TTR-017: Orquestación de Correlación (Signal Correlation Analyzer)**
*   **Descripción:** Invoca a [`signal-correlation-analyzer`](../features/signal-correlation-analyzer.md) para control de ortogonalidad.
*   **Reglas de Orquestación:**
    *   Verifica que la candidata no sea un clon de una estrategia existente.
*   **Entrada:** `candidate_signals`, `databank_signals`.
*   **Salida:** `correlation_score`.
*   **Precondición:** Estrategia simulada.
*   **Postcondición:** Aprobación de unicidad.

### **TTR-018: Orquestación de Rastreo WFA (Equity Curve Tracker)**
*   **Descripción:** Invoca a [`equity-curve-tracker`](../features/equity-curve-tracker.md) para cada ventana Walk-Forward.
*   **Reglas de Orquestación:**
    *   Mide la varianza de la equidad en periodos OOS (Out of Sample).
*   **Entrada:** `oos_trade_fills`.
*   **Salida:** `oos_equity_segments`.
*   **Precondición:** Ejecución de ventana WFA.
*   **Postcondición:** Insumo para métricas de eficiencia.

### **TTR-019: Orquestación de KPIs (Institutional Metrics)**
*   **Descripción:** Invoca a [`institutional-metrics`](../features/institutional-metrics.md) para el reporte final.
*   **Reglas de Orquestación:**
    *   Calcula SQN, Sharpe y Max Drawdown en la serie combinada In/Out sample.
*   **Entrada:** `full_equity_curve`.
*   **Salida:** `comprehensive_kpi_report`.
*   **Precondición:** Simulación WFA terminada.
*   **Postcondición:** Respaldo para el Veredicto.

### **TTR-020: Orquestación de Certificación (Strategy Versioning)**
*   **Descripción:** Invoca a [`strategy-versioning`](../features/strategy-versioning.md) para inmutabilidad del test.
*   **Reglas de Orquestación:**
    *   Crea el nodo "Tested" y lo enlaza permanentemente al resultado del WFA.
*   **Entrada:** `candidate_strategy`, `final_verdict`.
*   **Salida:** `certified_dag_node`.
*   **Precondición:** Veredicto definitivo.
*   **Postcondición:** Estrategia sellada en el DAG.

### **TTR-021: Orquestación Forense (Audit Log)**
*   **Descripción:** Invoca a [`audit-log`](../features/audit-log.md) para firmar los tests.
*   **Reglas de Orquestación:**
    *   Genera el `audit_hash` del log completo de validación.
*   **Entrada:** `validation_report`.
*   **Salida:** `signed_audit_hash`.
*   **Precondición:** Todos los tests pasados o abortados.
*   **Postcondición:** Cero manipulación de backtests.

### **TTR-022: Orquestación Analítica Masiva (DuckDB SQL Engine)**
*   **Descripción:** Invoca a [`duckdb-sql-engine`](../features/duckdb-sql-engine.md) para cálculos vectorizados WFA.
*   **Reglas de Orquestación:**
    *   Calcula el Deflated Sharpe Ratio procesando cientos de miles de trades en milisegundos.
*   **Entrada:** `all_trades_parquet`.
*   **Salida:** `dsr_score`.
*   **Precondición:** Archivo de trades exportado.
*   **Postcondición:** Filtro de over-fitting aplicado.

### **TTR-023: Orquestación de Estrés de Volatilidad (Adaptive Volume Indicators)**
*   **Descripción:** Invoca a [`adaptive-volume-indicators`](../features/adaptive-volume-indicators.md) en test de liquidez extrema.
*   **Reglas de Orquestación:**
    *   Somete la estrategia a una ventana sintética de contracción de liquidez (Crash test).
*   **Entrada:** `strategy_ast`, `synthetic_illiquid_data`.
*   **Salida:** `illiquidity_drawdown`.
*   **Precondición:** Simulación normal completada.
*   **Postcondición:** Prueba de fuego de resiliencia estructural.

### **TTR-024: Orquestación de Eficiencia del Modelo (Perfect Profit Benchmark)**
*   **Descripción:** Invoca a [`perfect-profit-benchmark`](../features/perfect-profit-benchmark.md) para filtrar sobreajustes "perezosos".
*   **Reglas de Orquestación:**
    *   Calcula qué porcentaje del Alpha teórico máximo capturó la estrategia. Descarta las ineficientes (< 5%).
*   **Entrada:** `historical_prices`, `candidate_trades`.
*   **Salida:** `model_efficiency_ratio`.
*   **Precondición:** TTR-003 completado.
*   **Postcondición:** Filtro previo a las auditorías complejas.

### **TTR-025: Orquestación del Aislamiento de Componentes (Monkey Test)**
*   **Descripción:** Invoca a [`component-isolation`](../features/component-isolation.md) para verificar mérito intrínseco.
*   **Reglas de Orquestación:**
    *   Aleatoriza salidas (Monkey Exit) y entradas (Monkey Entry) para certificar el edge.
*   **Entrada:** `strategy_ast`, `history_dataset`.
*   **Salida:** `component_merit_score`.
*   **Precondición:** Estrategia rentable.
*   **Postcondición:** Eliminación de lógicas superfluas.

### **TTR-026: Orquestación de la Guillotina de Ockham (Complexity Penalization)**
*   **Descripción:** Invoca a [`complexity-penalization`](../features/complexity-penalization.md) contra el overfitting paramétrico.
*   **Reglas de Orquestación:**
    *   Castiga el Fitness global basándose en el ratio `Trades / Parámetros`.
*   **Entrada:** `ast_parameter_count`, `total_trades`.
*   **Salida:** `complexity_penalty_factor`.
*   **Precondición:** Estrategia validada.
*   **Postcondición:** Score de robustez ajustado a la baja.

### **TTR-027: Orquestación de Búsqueda de Meseta (Topological Plateau Finder)**
*   **Descripción:** Invoca a [`topological-plateau-finder`](../features/topological-plateau-finder.md) para auto-configurar la estabilidad.
*   **Reglas de Orquestación:**
    *   Fuerza los parámetros de la estrategia hacia el centro geométrico del vecindario seguro.
*   **Entrada:** `strategy_parameters`, `simulation_engine`.
*   **Salida:** `centered_parameters`.
*   **Precondición:** Parámetros propuestos inicialmente.
*   **Postcondición:** Genoma estabilizado en llanuras, no en acantilados.

### **TTR-028: Orquestación Secuencial Controlada (Hierarchical Optimization)**
*   **Descripción:** Invoca a [`hierarchical-parameter-optimization`](../features/hierarchical-parameter-optimization.md).
*   **Reglas de Orquestación:**
    *   Bloquea y optimiza etapas secuencialmente en base al tag de jerarquía del parámetro.
*   **Entrada:** `tagged_strategy_ast`.
*   **Salida:** `sequentially_optimized_parameters`.
*   **Precondición:** Parámetros mapeados.
*   **Postcondición:** Prevención de conflictos multi-variable.

### **TTR-029: Orquestación del Red Team (Adversarial Noise Agent)**
*   **Descripción:** Invoca a [`adversarial-noise-agent`](../features/adversarial-noise-agent.md) inyectando terror.
*   **Reglas de Orquestación:**
    *   Aplica ruido gaussiano indexado al ATR histórico y simula slippage agresivo en puntos de baja liquidez.
*   **Entrada:** `trade_fill_log`, `historical_atr`.
*   **Salida:** `adversarial_equity_curve`.
*   **Precondición:** Estrategia aprobada en WFA.
*   **Postcondición:** Prueba inyectada contra falsas esperanzas en stop-losses milimétricos.

### **TTR-030: Orquestación de Auditoría Descendente (Fragility Gradient Auditor)**
*   **Descripción:** Invoca a [`fragility-gradient-auditor`](../features/fragility-gradient-auditor.md) para vetos estructurales.
*   **Reglas de Orquestación:**
    *   Si una alteración de 1% destruye >40% del Alpha (derivada segunda de inestabilidad), condena la matriz entera.
*   **Entrada:** `strategy_parameters`, `pnl_function`.
*   **Salida:** `fragility_condemnation_verdict`.
*   **Precondición:** Todas las simulaciones base pasadas.
*   **Postcondición:** Sentencia inamovible de sobreajuste por fragilidad.

### **TTR-031: Orquestación de Lógica Adaptativa (Efficiency Ratio Filter)**
*   **Descripción:** Invoca a [`adaptive-logic-er`](../features/adaptive-logic-er.md) para filtrar ruido.
*   **Reglas de Orquestación:**
    *   Calcula el ER de Kaufman. Bloquea la señal si el mercado es estocástico (ER < 0.3).
*   **Entrada:** `market_data`, `strategy_signals`.
*   **Salida:** `er_veto_status`.
*   **Precondición:** MOD-01 datos limpios.
*   **Postcondición:** Aseguramiento de ineficiencia real explotada.

### **TTR-032: Orquestación del Scoring Ponderado (Robustness Score Aggregator)**
*   **Descripción:** Invoca a [`robustness-score-aggregator`](../features/robustness-score-aggregator.md) para consolidar los 5 scores individuales en el score ponderado final (0-100).
*   **Reglas de Orquestación:**
    * Recibe los 5 scores individuales del guantelete (TTR-004) y los pasa al agregador con los pesos configurados.
    * El score final se compara contra `APPROVAL_THRESHOLD` (default: 75).
    * El score y su desglose se empaquetan para transmisión al módulo `execute` como parámetro de dimensionamiento de posición.
    * El score es inmutable una vez calculado para la versión actual de la estrategia.
*   **Entrada:** `individual_test_scores` (5 scores 0-100), `scoring_weights_config`.
*   **Salida:** `final_robustness_score` (0-100), `approval_status` (APPROVABLE / BELOW_THRESHOLD), `score_breakdown`.
*   **Precondición:** TTR-004 finalizado (5 scores individuales disponibles).
*   **Postcondición:** Score final calculado y vinculado al `version_node_id` de la estrategia en el DAG.

### **TTR-033: Orquestación del Veredicto en Lenguaje Natural (Robustness Verdict Engine)**
*   **Descripción:** Invoca a [`robustness-verdict-engine`](../features/robustness-verdict-engine.md) para generar el veredicto en lenguaje humano, identificación de puntos de ruptura y justificación semántica del score.
*   **Reglas de Orquestación:**
    * Toma los 5 resultados crudos de tests + el score ponderado final + metadatos de la estrategia.
    * Por defecto genera un reporte estructurado determinista por plantilla, sin LLM (ADR-0115); el realce vía LLM local soberano (`candle`) es opcional. Cero dependencia de Ollama.
    * Los puntos de ruptura identificados se inyectan como Dominant Rules (ADR-0024) en los metadatos de la estrategia para ser aplicados por `incubate` y `execute`.
*   **Entrada:** `individual_test_scores`, `final_robustness_score`, `score_breakdown`, `strategy_metadata`.
*   **Salida:** `verdict_text`, `rupture_points`, `most_sensitive_parameter`, `score_explanation`, `recommendations`.
*   **Precondición:** TTR-032 finalizado (score ponderado disponible).
*   **Postcondición:** Veredicto completo (numérico + lenguaje natural) registrado en el log de auditoría y visible en Strategy Inspector.

### **TTR-034: Orquestación de Matriz Microrodante Nocturna (Continuous Rolling WFM)**
*   **Descripción:** Invoca al [`walk-forward-analyzer`](../features/walk-forward-analyzer.md) en modo microrodante diario (23:59h) mediante el [`quantops-daemon`](../features/quantops-daemon.md).
*   **Reglas de Orquestación:**
    * Ejecuta re-optimización sobre los últimos 7-14 días.
    * Si el veredicto es positivo, transfiere parámetros vía FFI/gRPC al módulo `execute`.
*   **Entrada:** `live_market_data` (últimos 14 días), `active_strategy_ast`.
*   **Salida:** `optimized_parameter_set`, `re_optimization_verdict`.
*   **Precondición:** Ejecución recurrente disparada por Daemon.
*   **Postcondición:** Sincronización de parámetros en caliente con el Bridge de ejecución.

### **TTR-035: Orquestación de Evaluación Geométrica (Cluster Contiguo 3x3)**
*   **Descripción:** Invoca el filtro geométrico del [`walk-forward-analyzer`](../features/walk-forward-analyzer.md) sobre la matriz WFA.
*   **Reglas de Orquestación:**
    * Valida la existencia de bloques de 3x3 celdas estables en la matriz.
    * Si no hay cluster, emite veredicto `FAIL` por inestabilidad de ruido, ignorando celdas verdes aisladas.
*   **Entrada:** `WFA_Matrix_Results`.
*   **Salida:** `cluster_stability_status`.
*   **Precondición:** TTR-004 (WFA Analysis) finalizado.
*   **Postcondición:** Filtrado de ruido en la decisión final de robustez.

### **TTR-036: Orquestación de Tests Incrementales (Inheritance + Delta)**
*   **Descripción:** Invoca al [`incremental-test-engine`](../features/incremental-test-engine.md) para optimizar el guantelete.
*   **Reglas de Orquestación:**
    *   Antes de iniciar cualquier test del guantelete (WFA, MC, etc.), consulta al motor incremental buscando el `params_hash`.
    *   Si hay coincidencia (Herencia), inyecta los resultados previos para evitar recálculo de segmentos históricos o simulaciones redundantes.
*   **Entrada:** `candidate_strategy`, `params_hash`, `test_type`.
*   **Salida:** `test_result` (heredado o calculado).
*   **Precondición:** Recepción de candidato con historial de versiones (MOD-02/MOD-03).
*   **Postcondición:** Registro del nuevo resultado acumulativo en el DAG.

### **TTR-037: Preparación de Datos para Visualización Dual (Spaghetti & Confidence Cone)**
*   **Descripción:** Procesa las N iteraciones Monte Carlo para su visualización en el Strategy Inspector.
*   **Reglas de Orquestación:**
    * **Modo Spaghetti:** Serializa las 1,000 curvas de equidad con opacidad reducida (alpha=0.05) clasificadas por PnL final (Verde/Rojo).
    * **Modo Cono:** Calcula los percentiles configurables (P5, P50, P95, etc.) por cada punto temporal para renderizado de bandas sombreadas.
*   **Entrada:** `mc_distribution_curves`.
*   **Salida:** `visual_mc_payload` (Apache Arrow).
*   **Precondición:** TTR-004 finalizado.
*   **Postcondición:** Payload listo para transporte binario hacia el frontend.

### **TTR-038: Orquestación de Capa de Inferencia Estadística (EBTA)**
*   **Descripción:** Invoca a [`statistical-inference-ebta`](../features/statistical-inference-ebta.md) como el guantelete final contra el Data-Mining Bias.
*   **Reglas de Orquestación:**
    *   **DSR Activation:** Ejecuta la deflación de Sharpe utilizando $N$ y $\sigma^2$ recuperados de `dsr-tracking-engine`.
    *   **Bootstrap Significance:** Coordina el test de Romano-Wolf sobre GPU/Rust SIMD-Rayon.
    *   **Alpha Isolation:** Aplica el Market Detrender para asegurar retornos independientes de la tendencia base.
    *   **Logic Symmetry:** Valida la robustez vía inversión de señales.
*   **Entrada:** `trade_fill_log`, `generation_metadata`, `benchmark_data`.
*   **Salida:** `ebta_report`, `adjusted_p_value`, `detrended_sharpe`.
*   **Precondición:** TTR-004 (Robustez) finalizado.
*   **Postcondición:** Resultados EBTA inyectados en el Robustness Score final.

### **TTR-039: Orquestación de Certificación de Estabilidad de Volatilidad (Target Vol)**
*   **Descripción:** Invoca a [`volatility-stabilization`](../features/volatility-stabilization.md) para certificar que la estrategia es estable bajo diferentes regímenes.
*   **Reglas de Orquestación:**
    *   Somete la estrategia a pruebas en 3 regímenes (Calmo, Normal, Volátil).
    *   Rechaza si la volatilidad realizada se desvía > `VOL_STABILITY_THRESHOLD` (ADR-0068).
*   **Entrada:** `trade_fill_log`, `target_vol_config`.
*   **Salida:** `vol_certification_verdict`.
*   **Precondición:** TTR-003 finalizado.
*   **Postcondición:** Sello de certificación adjunto al reporte de robustez.

### **TTR-040: Orquestación de Modelado de Fricción Institucional (Adverse Selection)**
*   **Descripción:** Invoca a [`institutional-friction-modeling`](../features/institutional-friction-modeling.md) para calibrar el realismo de la ejecución.
*   **Reglas de Orquestación:**
    *   Aplica el test de estrés con `STRESS_FILL_RATE` de 0.60 para evaluar fragilidad (ADR-0069).
    *   Calcula el "Friction Cost" comparativo vs backtest perfecto.
*   **Entrada:** `proposed_trades`, `market_liquidity_data`.
*   **Salida:** `friction_adjusted_metrics`, `liquidity_fragility_score`.
*   **Precondición:** TTR-003 finalizado.
*   **Postcondición:** Métricas ajustadas inyectadas en el Scoring Aggregator.

### **TTR-041: Orquestación del Filtrado Dimensional (Parallel Coordinates)**
*   **Descripción:** Invoca al [`parallel-coordinates-visualizer`](../features/parallel-coordinates-visualizer.md) para proyectar y filtrar optimizaciones de más de 20 dimensiones.
*   **Reglas de Orquestación:**
    - Utiliza el downsampling para limitar a un máximo de 5,000 líneas visibles si el número de backtests supera el umbral.
    - Exporta los conjuntos de datos de backtest filtrados mediante Apache Arrow para un renderizado ágil en la UI.
*   **Entrada:** `optimization_results`, `brushing_selection`.
*   **Salida:** `filtered_parameter_spaces`, `isolated_clusters`.
*   **Precondición:** Resultados de optimización masiva disponibles.
*   **Postcondición:** Visualización y filtrado dimensional fluido en la UI.

### **TTR-042: Orquestación del Filtrado Cruzado (Cross-Filtering)**
*   **Descripción:** Invoca al [`cross-filtering-visualizer`](../features/cross-filtering-visualizer.md) para generar y coordinar histogramas interactivos de múltiples dimensiones.
*   **Reglas de Orquestación:**
    - Mantiene una máscara de bits atómica para rastrear el subconjunto de registros válidos según los filtros aplicados.
    - Recalcula los bins y frecuencias en menos de 50 ms utilizando DuckDB.
*   **Entrada:** `optimization_results`, `active_filters_mask`.
*   **Salida:** `updated_bins_and_frequencies`, `matching_backtests`.
*   **Precondición:** TTR-041 finalizado o disponible.
*   **Postcondición:** Distribuciones de parámetros actualizadas según los filtros cruzados.

### **TTR-043: Orquestación de PCA Toxicity Analyzer (ADR-0072)**
*   **Descripción:** Invoca al [`pca-toxicity-analyzer`](../features/pca-toxicity-analyzer.md) para procesar reducción de dimensiones y agrupamiento de estrategias.
*   **Reglas de Orquestación:**
    - Extrae el dataset del databank y lo envía al subproceso de IA en la CPU.
    - Actualiza el rastro de evidencia en el módulo de feedback si se produce la purga de un clúster.
*   **Entrada:** `strategies_data`.
*   **Salida:** `toxicity_scores`, `cluster_labels`.
*   **Precondición:** Databank disponible.
*   **Postcondición:** Purga de clústeres tóxicos completada.

### **TTR-044: Orquestación del Laboratorio de Estrés Interactivo (Interactive Stress Lab)**
*   **Descripción:** Invoca a [`interactive-stress-lab`](../features/interactive-stress-lab.md) para que el analista deforme la curva de capital de la candidata en tiempo real antes de aprobarla.
*   **Reglas de Orquestación:**
    *   Reutiliza los motores ya orquestados (Monte Carlo TTR-004, Adversarial TTR-029, slippage) sobre copias en memoria; la fuente histórica permanece inmutable.
    *   Provee la curva base y el vector de deslizadores; recibe la curva deformada y las métricas recalculadas dentro del presupuesto de un frame.
*   **Entrada:** `equity_curve_base`, `slider_vector` (fricción + macro).
*   **Salida:** `deformed_equity_curve`, `stress_metrics`, `stress_snapshot`.
*   **Precondición:** TTR-003 (Backtest) finalizado.
*   **Postcondición:** Punto de quiebre táctil registrado y emitido a `feedback`.

### **TTR-045: Orquestación del Co-Piloto de Mesetas (Plateau Co-Pilot)**
*   **Descripción:** Invoca a [`plateau-copilot`](../features/plateau-copilot.md) para presentar el mapa de calor 2D del barrido y capturar la selección manual del parámetro de producción.
*   **Reglas de Orquestación:**
    *   La detección geométrica de la meseta se delega a [`topological-plateau-finder`](../features/topological-plateau-finder.md) (TTR-027); el co-piloto solo visualiza y captura el clic humano.
    *   El parámetro de producción NUNCA se fija sin clic humano; se advierte si la selección es un pico sin meseta.
*   **Entrada:** `parameter_sweep_grid`, `target_metric`.
*   **Salida:** `human_selected_parameters`, `plateau_suggestion`.
*   **Precondición:** Barrido de parámetros disponible.
*   **Postcondición:** Par de parámetros fijado por el humano y vinculado al `version_node_id`.

### **TTR-046: Orquestación del Etiquetado Manual de Regímenes (Manual Regime Tagger)**
*   **Descripción:** Invoca a [`manual-regime-tagger`](../features/manual-regime-tagger.md) para aplicar las reglas duras de desempeño por zona crítica en el embudo de robustez.
*   **Reglas de Orquestación:**
    *   Recorta la curva de la candidata a cada zona etiquetada y evalúa la regla dura configurada.
    *   Una candidata que incumple una regla dura activa en zona crítica es rechazada o degradada según `ZONE_RULE_ENFORCEMENT`.
*   **Entrada:** `equity_curve`, `tagged_zones`.
*   **Salida:** `per_zone_verdict` (CUMPLE/NO CUMPLE).
*   **Precondición:** Zonas etiquetadas disponibles desde `ingest`; TTR-003 finalizado.
*   **Postcondición:** Veredicto por zona emitido a `feedback` (causa de rechazo).

### **TTR-047: Orquestación del Fitness Contextual Multi-Régimen (Contextual Fitness Scorer)**
*   **Descripción:** Invoca a [`contextual-fitness-scorer`](../features/contextual-fitness-scorer.md) para reemplazar el fitness plano por un score multidimensional sensible al régimen.
*   **Reglas de Orquestación:**
    *   Disecciona la curva por régimen (fuente automática `hmm-regime-detection` o manual `manual-regime-tagger`) y pondera las métricas según el mapa de prioridades del humano.
    *   El desglose por régimen NUNCA se colapsa en un único número que oculte la vulnerabilidad.
*   **Entrada:** `equity_curve`, `regime_classification`, `regime_priority_map`.
*   **Salida:** `multidimensional_score`, `radar_payload`, `weakest_regime`.
*   **Precondición:** TTR-003 finalizado; clasificación de régimen disponible.
*   **Postcondición:** Régimen más débil emitido a `feedback`; score inyectado en la decisión de aprobación.

### **TTR-055: Orquestación de Adaptive WFA Windows (ADR-0073)**
*   **Descripción:** Invoca al [`hmm-regime-detection`](../features/hmm-regime-detection.md) para ajustar ventanas WFA basadas en regímenes de mercado.
*   **Reglas de Orquestación:**
    - Lee el `regime_label` del dataset de mercado y configura el tamaño de ventanas IS/OOS dinámicamente.
*   **Entrada:** `market_data_with_regimes`.
*   **Salida:** `adaptive_wfa_windows_config`.
*   **Precondición:** Dataset de mercado con etiquetas de régimen disponible.
*   **Postcondición:** Validación WFA por régimen completada.

### **TTR-056: Orquestación de Autoencoder Outlier Detector (ADR-0074)**
*   **Descripción:** Invoca al [`autoencoder-outlier-detector`](../features/autoencoder-outlier-detector.md) para filtrar trades anómalos.
*   **Reglas de Orquestación:**
    - Entrena el modelo neuronal en características de transacciones del candidato y calcula el error de reconstrucción por trade.
    - Aplica la penalización de fitness si el impacto supera el umbral configurable.
*   **Entrada:** `candidate_trades_history`.
*   **Salida:** `outlier_adjusted_metrics`, `outlier_adjusted_fitness`.
*   **Precondición:** Simulación de backtest (TTR-003) completada.
*   **Postcondición:** Métricas ajustadas y penalización registradas en `test_results`.

### **TTR-057: Orquestación del Design Manifest Quality Gate (ADR-0053)**
*   **Descripción:** Invoca al [`design-manifest`](../features/design-manifest.md) como el filtro final ineludible de aprobación.
*   **Reglas de Orquestación:**
    - Evalúa las métricas de la estrategia contra el esquema de metas SMART del contrato inicial.
    - Bloquea el paso a la Fase 3 de incubación si alguna condición no se cumple milimétricamente o el Robustness Score final está por debajo del umbral del Gatekeeper.
*   **Entrada:** `test_results`, `SmartGoalsConfig`.
*   **Salida:** `design_manifest_verdict` (APROBADO / RECHAZADO).
*   **Precondición:** Guantelete de robustez (TTR-004) y tests secundarios completados.
*   **Postcondición:** Estrategia aprobada o bloqueada para promoción de fase.



### **TTR-058: Orquestación de Simulación de Portafolio Real (Portfolio Backtest)**
*   **Descripción:** Invoca a [`portfolio-backtest`](../features/portfolio-backtest.md) durante la validación pesada para someter el conjunto de estrategias combinadas a pruebas concurrentes de margen y capitalización.
*   **Reglas de Orquestación:**
    - Evalúa el drawdown dinámico consolidado del pool de capital compartido.
    - Aborta la validación si se detecta una llamada de margen (Margin Call) simulada en cualquier punto de la historia.
*   **Entrada:** `group_of_strategies`, `margin_rules_config`.
*   **Salida:** `portfolio_stress_clearance` (PASSED | FAILED).
*   **Precondición:** TTR-003 (Backtest Engine) finalizado para todas las estrategias candidatas del grupo.
*   **Postcondición:** Resultados de estrés consolidado agregados al reporte de veredicto.

### **TTR-048: Orquestación del Mapa de Calor Mensual (Monthly Performance Heatmap)**
*   **Descripción:** Invoca a [`monthly-performance-heatmap`](../features/monthly-performance-heatmap.md) para generar datos agregados de retornos mensuales.
*   **Reglas de Orquestación:**
    - Realiza agregaciones dinámicas por dirección (Long/Short) y muestra (IS/OOS) desde el ledger.
    - El resultado se envía en formato de matriz lista para renderizado en Flutter.
*   **Entrada:** `aggregate_trades_request`.
*   **Salida:** `monthly_heatmap_matrix_json`.
*   **Precondición:** Transacciones históricas del backtest disponibles.
*   **Postcondición:** Matriz mensual lista para presentación en la UI.

### **TTR-049: Orquestación del Visor de Diferencias (Strategy Config Diff)**
*   **Descripción:** Invoca a [`strategy-config-diff`](../features/strategy-config-diff.md) al comparar variantes del espacio de trabajo.
*   **Reglas de Orquestación:**
    - Realiza un diff profundo del JSON AST identificando severidades de cambios paramétricos.
    - Genera la señal de `RETEST_MANDATORIO` si se alteran variables lógicas de indicadores.
*   **Entrada:** `current_vs_historical_strategy_ast`.
*   **Salida:** `structural_diff_report`.
*   **Precondición:** Ambas versiones lógicas del AST existiendo en SQLite.
*   **Postcondición:** Alerta visual de sincronización en la UI.

### **TTR-050: Orquestación de la Suite BI de Análisis (Trade Analysis BI Suite)**
*   **Descripción:** Invoca a [`trade-analysis-bi-suite`](../features/trade-analysis-bi-suite.md) para generar histogramas y scatter plots.
*   **Reglas de Orquestación:**
    - Procesa en paralelo con Polars la duración de los trades y su rentabilidad acumulada.
    - Aplica downsampling estadístico si el número de puntos excede los umbrales configurables.
*   **Entrada:** `trades_performance_raw_list`.
*   **Salida:** `bi_aggregated_payload_arrow`.
*   **Precondición:** Backtest finalizado y transacciones registradas.
*   **Postcondición:** Datos optimizados para CustomPainter de Flutter.

### **TTR-051: Orquestación del Generador de Gráficos PDF (PDF Charts Rendering)**
*   **Descripción:** Invoca a [`pdf-charts-rendering`](../features/pdf-charts-rendering.md) al solicitar un reporte de auditoría.
*   **Reglas de Orquestación:**
    - Coordina la creación del PDF incorporando el heatmap y la curva vectorial de equidad de forma asíncrona.
*   **Entrada:** `compiled_metrics_and_curves`.
*   **Salida:** `pdf_report_disk_path`.
*   **Precondición:** Veredicto final de robustez calculado.
*   **Postcondición:** Archivo PDF guardado físicamente y disponible.

### **TTR-052: Orquestación de Navegación Temporal (Time-Warp UI)**
*   **Descripción:** Orquestar la consulta y carga ultra-rápida de transacciones históricas filtradas por fecha mediante partition pruning sobre el data lake Parquet particionado.
*   **Reglas de Orquestación:**
    *   Invoca a [`time-warp-debugger`](../features/time-warp-debugger.md) al cambiar el rango de fechas en la UI.
    *   Gestiona la poda de directorios Hive-style de forma transparente delegando la consulta a DuckDB en Rust y retornando Arrow tables.
    *   Los metadatos se registran conforme al perfil Ops / Auditoría (ADR-0020 V2).
*   **Entrada:** `pipeline_id`, `generation`, `start_date`, `end_date`.
*   **Salida:** `trades` (Arrow array/pylist), `downsampled_equity_curve` (500 puntos), `query_stats` (partitions_scanned, latency_ms).
*   **Precondición:** Selector de tiempo activo en la UI del módulo `validate`.
*   **Postcondición:** Panel de transacciones y gráfico temporal actualizados en Flutter.

### **TTR-053: Orquestación de Exploración Multidimensional (UMAP Scatter Plot)**
*   **Descripción:** Orquestar la obtención de las coordenadas UMAP tridimensionales y la selección interactiva por lazo de candidatos en el Nivel 2 del lienzo.
*   **Reglas de Orquestación:**
    *   Invoca a [`umap-scatter-visualizer`](../features/umap-scatter-visualizer.md) para recuperar y renderizar nativamente en la UI de Flutter los vectores Arrow provenientes de Rust.
    *   Mapea los IDs seleccionados por colisión de lazo y actualiza reactivamente la tabla de candidatos.
    *   Los metadatos se registran conforme al perfil IA / R&D (ADR-0020 V2).
*   **Entrada:** `pipeline_id`, `generation`, `lasso_selection_polygon`.
*   **Salida:** `umap_points` (Arrow table), `selected_strategy_ids`.
*   **Precondición:** Ejecución evolutiva con resultados de embedding finalizados en backend.
*   **Postcondición:** Clúster de estrategias seleccionado y visible en el panel del inspector.

### **TTR-054: Orquestación de Purga de Clústeres Tóxicos (Toxicity Purifier)**
*   **Descripción:** Orquestar el dashboard de purga atómica de clústeres tóxicos PCA, gestionando la simulación de impacto, soft-delete y snapshots de reversión rápida.
*   **Reglas de Orquestación:**
    *   Invoca a [`toxicity-purifier`](../features/toxicity-purifier.md) para recuperar los KPIs de toxicidad por clúster del databank.
    *   Exige la confirmación multi-paso con previsualización de KPIs y la generación obligatoria de un `snapshot_id` en SQLite antes de despachar el comando de soft-delete (`is_purged=true`).
    *   Provee el canal FFI para mandar el comando de rollback restaurando el catálogo.
    *   Los metadatos se registran conforme al perfil Ops / Auditoría (ADR-0020 V2).
*   **Entrada:** `databank_path`, `cluster_label_to_purge`, `rollback_snapshot_id` (opcional).
*   **Salida:** `toxicity_clusters_list`, `purge_status` (SUCCESS | ROLLBACK_SUCCESS), `new_snapshot_id`.
*   **Precondición:** Módulo `validate` con análisis PCA completado.
*   **Postcondición:** Databank limpio de clústeres tóxicos y logs forenses registrados.

### **TTR-059: Orquestación de Compuertas de Robustez por Dominio Genómico (ADR-0108)**
*   **Descripción:** Generaliza el Guantelete de Robustez (TTR-004) para incorporar las Compuertas de Robustez bloqueantes de cada dominio del Registro de Dominios Genómicos (ADR-0108) presente en `ACTIVE_GENOME_DOMAINS`. Cuando hay más de un dominio distinto de Señal activo, **todas** sus compuertas corren — sin exclusión mutua, todas son bloqueantes de forma independiente.
*   **Reglas de Orquestación:**
    *   **Riesgo y Gestión de Posición (ADR-0109), si está en `ACTIVE_GENOME_DOMAINS`:** invoca a [`monte-carlo-simulator`](../features/monte-carlo-simulator.md) en modo Réplica de Estado de Riesgo (TTR-004 de esa feature) — bloqueante.
    *   **Régimen y Filtro de Entorno (ADR-0110), si está en `ACTIVE_GENOME_DOMAINS`:** invoca a [`walk-forward-analyzer`](../features/walk-forward-analyzer.md) (TTR-008, WFA Segmentado por Régimen) y a [`cross-market-validation`](../features/cross-market-validation.md) (TTR-002) — ambos bloqueantes.
    *   **Portafolio y Correlación (ADR-0111, co-evolución de cartera), si está activo:** invoca a [`monte-carlo-simulator`](../features/monte-carlo-simulator.md) en modo Monte Carlo de Desfase Temporal (TTR-005 de esa feature) — bloqueante para cada configuración de cartera candidata.
    *   Cada score individual de compuerta de dominio activo se incorpora al scoring ponderado (TTR-032) con el mismo tratamiento que los 5 tests del Guantelete Decagonal.
*   **Entrada:** `pareto_front_candidates`, `ACTIVE_GENOME_DOMAINS`.
*   **Salida:** `domain_gate_scores` (0-100 por dominio activo), `domain_gate_verdict` (PASS | FAIL, agregado).
*   **Precondición:** TTR-004 finalizado; `ACTIVE_GENOME_DOMAINS` contiene al menos un dominio distinto de Señal.
*   **Postcondición:** un `domain_gate_verdict = FAIL` en **cualquiera** de las compuertas activas produce veredicto `RECHAZADA` sin excepción, independientemente del score ponderado de los 5 tests del Guantelete y del resultado de las demás compuertas de dominio.

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

- **Inundación de Fundamientos (ADR-0020 V2):** 
Cada validación y test de robustez registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del job de validación |
| | `created_at` | Timestamp de proceso |
| | `audit_hash` | Hash del veredicto final |
| | `audit_chain_hash` | Hash de la secuencia de tests (WFA/MC) |
| **II. Soberanía** | `owner_id` | Usuario responsable de la certificación |
| | `manifest_id` | ID del diseño evaluado |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del motor de validación |
| | `data_snapshot_id" | Ref al snapshot PIT (Point-In-Time) |
| | `indicator_state_hash` | Snapshot del score de robustez consolidado |
| | `version_node_id` | Versión de la estrategia en el DAG |
| **IV. Hardware** | `node_id` | ID del hardware físico ejecutor |
| | `process_id` | PID del worker de validación |

- **Decisión Arquitectónica Asociada:**
    - ADR-0005: Versionamiento Reproducible (DAG).
    - ADR-0013: Stack Tecnológico (NautilusTrader).
    - ADR-0020 V2: Inundación de Fundaciones.
    - ADR-0108 / ADR-0109 / ADR-0110 / ADR-0111: Compuertas de Robustez por Dominio Genómico (TTR-059).

---

## Dependencias
**Depende de:**
- [`generate`](../modules/generate.md) — para la recepción de candidatos.
- [`ingest`](../modules/ingest.md) — para la obtención de datos certificados.

**Consumido por:**
- [`incubate`](../modules/incubate.md) — para la ejecución en entorno de paper trading.
- [`manage`](../modules/manage.md) — para la promoción directa (override) a portafolios.
