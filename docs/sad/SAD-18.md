## 18. Plan de Lanzamiento (Rollout Strategy v2.0)

El desarrollo se organiza en sprints incrementales para mitigar riesgos técnicos bajo la arquitectura unificada Rust (Core) + Flutter (Frontend via FFI):

**Entrega progresiva de UI (ADR-0117):** la "Interfaz Gráfica" no es exclusiva del Sprint 4.X. Cada sprint que implemente una Feature con superficie UI declarada entrega también, dentro de ese mismo sprint, su Cáscara Delgada (Techo Fijo) en el Panel Operativo Fundacional — no se acumula UI para el final. El Sprint 4.X se redefine como unificación y pulido (ver abajo).

* **Sprint 0.X — Fundación:** Configuración del entorno Cargo + Cargo workspaces, estructura de módulos en Rust, persistencia local con SQLite (SQLx) y SQLx compile-time embedded migrations. Paridad básica de NautilusTrader backtest y smoke test de cómputo numérico CPU-first (`ndarray`/Rayon, ADR-0112). Implementación de interfaz base trait/protocol `IDrasusNode` en Rust. Endpoint de telemetría local de hardware y notificaciones locales SQLite. Entrega el Panel Operativo Fundacional (SPIKE-006/ADR-0117): clock, cola de trabajos y bitácora de auditoría en vivo.
* **Sprint 1.X — Generación (Ingest & Generate):** Ingesta local de datos históricos Parquet optimizada con Polars. Pipeline de limpieza y alineación temporal (Data Sanitizer). DuckDB embebido para remuestreo dinámico. Ejecución de NautilusTrader con simulación multicanal y alineación Bar-Open. Minero Genético NSGA-II nativo en Rust, compilación a Strategy AST v3.0, cálculo de métricas vectorizadas y entrenamiento local de modelos HMM para clasificación de regímenes.
* **Sprint 2.X — Robustez (Validate):** Walk-Forward Analysis (WFA) Matrix local y tests de simulación de estrés Monte Carlo CPU-first (`ndarray`/Rayon; GPU `candle` opcional, ADR-0112). Métricas de inferencia estadística e inyección de ruido. PCA Toxicity Clustering y Autoencoder Outlier Detector local en CPU Rust puro (`candle`). Proyección dimensional UMAP y score de robustez ponderado. Portafolio multiactivo y optimización de pesos HRP.
* **Sprint 3.X — Ejecución (Execute):** Daemons persistentes de NautilusTrader (LiveNode) y adaptadores para brokers reales. Pre-Trade Risk Validator (<1ms) y Shadow Watchdog. Daemon de auto-rebalanceo de portafolios asíncrono con HRP y HMM dinámico.
* **Sprint 4.X — Unificación ZUI y Pulido (antes "Interfaz Gráfica/Glass Box"):** Frontend en Flutter con Impeller Engine para renderizado a 120 FPS. Unificación de las Cáscaras Delgadas entregadas en los sprints anteriores dentro de la navegación fractal ZUI (ADR-0028: Fleet Command → Orchestrator → Strategy Inspector). Lienzo nodal visual (DAG Editor) en CustomPainter y previsualizaciones locales mediante Micro-Backtests en hilos de Rust (FFI). Pulido visual (theming, animaciones, responsive) diferido por el Techo Fijo. Heatmaps interactivos de performance y suites de visualización con datos agregados multi-fase.


---

