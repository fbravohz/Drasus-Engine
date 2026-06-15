## 7. Features Reutilizables (Componentes Transversales)

Las siguientes características son componentes independientes que se organizan en `/features/` o `/moonshots/`, accesibles desde cualquier módulo a través de su interfaz pública. Cada feature es **agnóstica al consumidor específico** (ver ADR-0003, ADR-0001):

### Infraestructura Base
| Feature | Localización | Descripción | Consumido por |
|---|---|---|---|
| **Infraestructura centralizada** | `/infrastructure/` | Configuración BD, bus de eventos, pools de conexión | Todos |
| **Auditoría y Telemetría** | `/features/audit-log.md` | Registro de eventos, métricas, trazabilidad | Todos |

### Análisis y Validación de Datos
| Feature | Localización | Descripción | Consumido por |
|---|---|---|---|
| **PIT Data Validator** | `/features/pit-data-validator.md` | Validación de datos sin look-ahead bias (Point-In-Time real) | `ingest`, `validate`, `feedback` |
| **Data Validator** | `/features/data-validator.md` | Validación general de OHLCV (sanidad lógica) | `ingest` |
| **Walk-Forward Analyzer** | `/features/walk-forward-analyzer.md` | Validación robusta de estrategias en ventanas rolling, WFA Matrix y Matriz Microrodante Nocturna (23:59h). | `validate`, `execute` |
| **DTW Adaptive Window** | `/features/dtw-adaptive-window.md` | Segmentación temporal adaptativa (Matriz Orgánica): ventanas que respiran según régimen de volatilidad vía Dynamic Time Warping. | `validate`, `manage` |
| **Cross-Market Validation** | `/features/cross-market-validation.md` | Validación de robustez en cestas de mercados correlacionados | `validate`, `incubate` |
| **Rule Ablation** | `/features/rule-ablation.md` | Desactivación sistemática de reglas para eliminar ruido y redundancia | `validate`, `generate` |
| **Robustness Score Aggregator** | `/features/robustness-score-aggregator.md` | Scoring Ponderado 0-100 (ADR-0058). Consolida los 5 tests del guantelete. | `validate`, `execute`, `feedback` |
| **Robustness Verdict Engine** | `/features/robustness-verdict-engine.md` | Veredictos en lenguaje natural por **plantilla determinista** (ADR-0058/0115; LLM opcional, sin Ollama requerido). Identifica puntos de ruptura. | `validate`, `feedback` |
| **Volatility Stabilization** | `/features/volatility-stabilization.md` | Certificación de Target Vol y estabilidad bajo regímenes. | `validate`, `execute` |
| **Institutional Friction** | `/features/institutional-friction-modeling.md` | Modelado de Adverse Selection y Probabilistic Fills. | `validate`, `backtest-engine` |
| **Operational Safety Monitor** | `/features/operational-safety-monitor.md` | Pardo Profile Monitor y Strategy Stop-Loss (SSL). | `execute`, `feedback` |
| **Parallel Coordinates** | `/features/parallel-coordinates-visualizer.md` | Visualización y brushing interactivo de optimizaciones de alta dimensión. | `validate` |
| **Cross-Filtering** | `/features/cross-filtering-visualizer.md` | Histogramas interactivos coordinados de múltiples parámetros. | `validate` |
| **PCA Toxicity Analyzer** | `/features/pca-toxicity-analyzer.md` | Análisis no supervisado PCA y K-Means para purga de clústeres tóxicos | `validate`, `feedback` |
| **Autoencoder Outlier Detector** | `/features/autoencoder-outlier-detector.md` | Detección de trades anómalos mediante reconstrucción neuronal profunda | `validate`, `feedback` |
| **Design Manifest** | `/features/design-manifest.md` | Contratos de metas SMART y validación de calidad final | `validate`, `generate` |
| **Interactive Stress Lab** | `/features/interactive-stress-lab.md` | Deformación táctil de la curva de capital en tiempo real (sliders de fricción y shock macro) sobre los motores MC/slippage existentes. | `validate`, `feedback` |
| **Plateau Co-Pilot** | `/features/plateau-copilot.md` | Mapa de calor 2D con sugerencia de meseta (IA) y selección manual del parámetro de producción (delega geometría a Topological Plateau Finder). | `validate`, `generate` |
| **Manual Regime Tagger** | `/features/manual-regime-tagger.md` | Etiquetado visual de crisis (Drag & Tag) y reglas duras de desempeño por zona crítica. | `ingest`, `validate` |
| **Contextual Fitness Scorer** | `/features/contextual-fitness-scorer.md` | Fitness multidimensional diseccionado por régimen con gráfico de radar (sustituye el fitness plano estático). | `validate`, `manage` |
| **Incremental Test Engine** | `/features/incremental-test-engine.md` | Herencia de resultados de validación por hashing de parámetros (ahorra >80% de recomputación). | `validate`, `generate` |



### Generación de Estrategias
| Feature | Localización | Descripción | Consumido por |
|---|---|---|---|
| **NSGA-II Optimizer** | `/features/nsga2-optimizer.md` | Optimización multi-objetivo (Sharpe↑, DD↓, WR↑) | `generate` |
| **Symbolic Signal Discovery** | `/moonshots/symbolic-signal-discovery.md` | Descubrimiento automático de ecuaciones de alpha (regresión simbólica libre, moonshot con `egg`; no PySR — ADR-0113) | `generate` |
| **Fit-to-Portfolio Search** | `/features/fit-to-portfolio-search.md` | Búsqueda generativa con presión evolutiva de correlación < 0.3 | `generate`, `manage` |
| **Strategy AST Copilot** | `/features/strategy-ast-copilot.md` | Asistente determinista LLM para topología estructural | `generate` |
| **Glass-Box AI Translator** | `/features/glass-box-ai-translator.md` | Traducción de pesos DRL a AST visual y premisas en lenguaje natural (Semantic Explainer) | `generate`, `manage`, `withdraw` |
| **HMM Regime Detection** | `/features/hmm-regime-detection.md` | Detección de regímenes de mercado (trending, choppy, etc.) | `generate`, `manage`, `execute` |
| **Strategy Ensemble** | `/features/strategy-ensemble.md` | Síntesis multi-canal (NSGA + regresión simbólica nativa + HMM) | `generate` |
| **Alpha Harvesting Gateway** | `/features/alpha-harvesting-gateway.md` | Ingesta y refinamiento privado de estrategias anonimizadas | `generate` |
| **La Colmena** | `/moonshots/la-colmena.md` | Minería descentralizada de estrategias mediante nodos distribuidos | `generate` |

### Análisis de Pureza y Riesgo
| Feature | Localización | Descripción | Consumido por |
|---|---|---|---|
| **Factor Decomposition (FF5)** | `/features/factor-decomposition.md` | Descomposición de retornos en factores Fama-French 5 | `validate`, `manage`, `feedback` |
| **Alpha Purity Analyzer** | `/features/alpha-purity-analyzer.md` | Cálculo de pureza de alpha (habilidad vs factor luck) | `validate`, `feedback` |
| **Alpha Decoupling Module** | `/features/alpha-decoupling.md` | Neutralización de Beta para aislar ventaja de la estrategia | `validate`, `manage`, `feedback` |
| **Zero-Crossing Filter** | `/features/zero-crossing-filter.md` | Filtrado de señales ortogonales (independencia de factores) | `generate`, `validate` |
| **Signal Correlation Analyzer** | `/features/signal-correlation-analyzer.md` | Matriz de correlaciones (diversificación de señales) | `validate`, `manage`, `feedback` |
| **Portfolio Data Preparation** | `/features/portfolio-data-preparation.md` | Fundaciones de datos: Matriz Pearson, curvas normalizadas y labeling HMM | `ingest`, `generate`, `manage` |

### Backtesting y Ejecución
| Feature | Localización | Descripción | Consumido por |
|---|---|---|---|
| **Backtest Engine** | `/features/backtest-engine.md` | Motor de backtesting rápido y determinista | `validate`, `incubate` |
| **Portfolio Backtest** | `/features/portfolio-backtest.md` | Motor de simulación multiestrategia concurrente compartiendo capital y horarios | `manage`, `validate` |
| **Portfolio Optimizer** | `/features/portfolio-optimizer.md` | Motores de pesaje (Markowitz, HRP, Risk-Parity, Min-Variance) y rebalanceo Walk-Forward | `manage` |
| **Paper Trader** | `/features/paper-trader.md` | Simulación de trading sin capital real en tiempo real con alta fidelidad. | `incubate` |
| **Incubation Manager** | `/features/incubation-manager.md` | Orquestador de perfiles de incubación (Quarantine 7 días, Extended 21 días, Legacy 3-6 meses) con cono de confianza Monte Carlo (ADR-0088). | `incubate` |
| **Slippage Models** | `/features/slippage-models.md` | Modelado de slippage (spread, market impact) | `backtest-engine`, `execute` |
| **Equity Curve Tracker** | `/features/equity-curve-tracker.md` | Tracking barra-a-barra de capital, PnL, drawdown | `backtest-engine`, `manage`, `execute`, `feedback` |
| **Order FSM** | `/features/order-fsm.md` | Máquina de estados de órdenes (pendiente, enviada, ejecutada, etc.) | `execute`, `manage` |
| **Pre-Trade Validator** | `/features/pre-trade-validator.md` | Validación secuencial de riesgo en 10 pasos (ADR-0025) | `execute`, `manage` |
| **Advanced Trade Management** | `/features/advanced-trade-management.md` | Lógicas transaccionales base: Grid Trading, Hedging y Trailing Stop | `execute`, `manage` |
| **Kinetic Micro-Management** | `/features/kinetic-micro-management.md` | Módulo defensivo hostil: Scale Out mandatorio, Z-Score Trailing y Tapering | `execute`, `feedback` |


### Integración y Comunicación
| Feature | Localización | Descripción | Consumido por |
|---|---|---|---|
| **Broker Connector** | `/features/broker-connector.md` | Integración con brokers (API, gRPC/WebSocket) | `execute`, `ingest` |
| **Trade Reconciler** | `/features/trade-reconciler.md` | Reconciliación de órdenes reales vs esperadas | `manage`, `feedback` |
| **Institutional Metrics** | `/features/institutional-metrics.md` | Motor hiper-rápido de estadísticas clásicas (Sharpe, Drawdown, MAE/MFE) | Todos los módulos |
| **Prop-Firm Grader** | `/features/prop-firm-grader.md` | Filtro implacable contra límites de cuentas de fondeo (Drawdown diario, Profit Factor) | `validate`, `execute`, `withdraw` |
| **Portfolio Rules** | `/features/portfolio-rules.md` | Capa de envolvente de reglas / Challenge Mode y limites globales de portafolio | `manage`, `execute` |
| **Federated Portfolio** | `/features/federated-portfolio.md` | Aislamiento lógico de reglas y gobernanza autónoma de múltiples contenedores de portafolios | `manage`, `execute` |
| **Vector-Time Pruning** | `/features/vector-time-pruning.md` | Poda quirúrgica de ventanas horarias con pérdidas recurrentes (ej. noticias macro) | `validate`, `manage` |
| **QuantOps Daemon** | `/features/quantops-daemon.md` | Orquestador de CI/CD asíncrono para automatizar pipelines 24/7 sin intervención | `generate`, `validate`, `incubate` |
| **Robust Reporting** | `/features/robust-reporting.md` | Generación de reportes detallados estáticos (JSON/HTML) hiper-enriquecidos | `feedback`, `validate` |
| **Order-Priority Queue** | `/features/order-priority-queue.md` | Cola inteligente anti-throttling con backoff exponencial y prioridades de orden | `execute` |
| **Autopilot Metrics Provider** | `/features/autopilot-metrics-provider.md` | Exposición de métricas en vivo (PNL, DD, cumplimiento) al Dashboard | `execute`, `feedback` |
| **Event-Driven Pipeline Triggers** | `/features/event-driven-pipeline-triggers.md` | Automatización de flujos de descubrimiento y reoptimización reactivos basados en eventos | `generate`, `validate` |
| **Auto-Auditoría de Portafolios Vivos** | `/features/auto-auditoria-portafolios-vivos.md` | Monitoreo dinámico de costes reales de ejecución y recalculador de R Expectancy | `execute`, `feedback` |

### Herramientas Experimentales (Moonshots)
| Feature | Localización | Descripción | Consumido por |
|---|---|---|---|
| **Marketplace de Cajas Negras** | `/moonshots/marketplace-cajas-negras.md` | Distribución segura y suscripción de subgrafos lógicos encriptados sin revelar IP | `generate`, `execute` |
| **Simulador Adversarial** | `/moonshots/simulador-adversarial.md` | Motor de orden book dinámico simulado con 50 agentes concurrentes competidores | `validate` |
| **Topografía 3D de Liquidez** | `/moonshots/topografia-3d-liquidez.md` | Visualización en tres dimensiones del histórico del Order Book y zonas de liquidez | `validate`, `execute` |
| **God Mode (Edge Deployment)** | `/moonshots/god-mode-edge.md` | Pipeline automático CI/CD para empaquetar AST en contenedor Docker Headless en AWS ECS/Fargate | `execute` |
| **Microestructura L3 (MBO)** | `/moonshots/microestructura-l3.md` | Simulación y analítica a nivel de orden individual (Market-by-Order) para SaaS institucional | `validate`, `backtest-engine` |
| **Deep Learning Suite** | `/moonshots/deep-learning-suite.md` | Suite de aprendizaje profundo (LSTM, Transformers, DRL PPO, DARTS, Cloud LLM Gateway) | `generate`, `validate` |
| **Advanced Equities Engine** | `/moonshots/advanced-equities-engine.md` | StockPicker dinámico, splits/dividendos PIT y mitigación de sesgo de supervivencia | `validate`, `ingest` |
| **Universal Strategy Transpiler** | `/moonshots/universal-strategy-transpiler.md` | Exportación de AST visual a lenguajes MQL4/MQL5, NinjaScript, EasyLanguage, Python | `validate` |
| **Collective Intelligence** | `/moonshots/collective-intelligence.md` | Extracción de patrones y Meta-Learning local-first con firmas anonimizadas P2P | `generate`, `validate` |
| **Institutional Plugin System** | `/moonshots/institutional-plugin-system.md` | SDK de Python, sandboxing WebAssembly (Wasmer) y encriptación E2EE de extensiones | Todos los módulos |
| **GANs para Universos Sintéticos** | `/moonshots/gans-universos-sinteticos.md` | Generación de microestructura de mercado sintética realista mediante modelos adversarios | `generate`, `validate` |
| **TDA Phase-Space Isolation** | `/moonshots/tda-phase-space-isolation.md` | Aislamiento de co-colapso de cola del portafolio vía Análisis de Datos Topológicos (no lineal) | `manage`, `validate` |
| **Cellular Automata Logic Growth** | `/moonshots/cellular-automata-logic-growth.md` | Crecimiento procedural de ramas lógicas vía autómatas celulares (hipótesis R&D no validada) | `generate` |
| **Neuro-Symbolic Fusion** | `/moonshots/neuro-symbolic-fusion.md` | Fusión de estrategias maestras preservando lógica simbólica (El Colisionador, human-in-the-loop) | `generate` |
| **Graph Neural Networks para Contagio Macro** | `/moonshots/gnn-contagio-macro.md` | Modelado de contagio y propagación de shocks financieros usando grafos dinámicos | `generate`, `validate` |
| **Knowledge Graphs Vectoriales** | `/moonshots/knowledge-graphs-galaxias.md` | Grafo evolutivo e histórico de linaje de estrategias con explorador espacial 3D | `generate` |

**Regla de Oro (ADR-0003):** 
- Cada feature es una unidad desacoplable y testeable independientemente
- Los módulos acceden SOLO a través de interfaz pública (nunca a internals)
- Una feature puede ser reutilizada en 1 módulo (ej: broker-connector solo en execute) o en 8 (ej: audit-log en todos)
- Los módulos SON orquestadores puros (Thin Shell § ADR-0001): NO implementan lógica propia, solo orquestan features

---

