# Drasus Engine — Sistema Documental

Este es el punto de entrada central a la arquitectura y especificación de Drasus Engine. El sistema está diseñado como un **Monolito Modular** siguiendo el patrón **FCIS** (Functional Core / Imperative Shell).

---

## 🏗️ Módulos (Pipeline de Trading)

Los módulos son **Orquestadores Puros** (Imperative Shell) que definen las etapas secuenciales del ciclo de vida de una estrategia. Cada módulo consume múltiples features para realizar su tarea.

| Módulo | Descripción | Estado |
|--------|-------------|--------|
| [**ingest**](./modules/ingest.md) | Ingesta de datos, validación y detección de régimen | Especificación |
| [**generate**](./modules/generate.md) | Generación de estrategias candidatas | Especificación |
| [**validate**](./modules/validate.md) | Validación estadística y backtesting robusto | Especificación |
| [**incubate**](./modules/incubate.md) | Paper trading forward y prueba Pardo | Especificación |
| [**manage**](./modules/manage.md) | Optimización de portafolio y asignación de capital | Especificación |
| [**execute**](./modules/execute.md) | Ejecución real, checks de seguridad y watchdog | Especificación |
| [**feedback**](./modules/feedback.md) | Análisis de delta, veredicto de continuidad y cierre de bucle | Especificación |
| [**withdraw**](./modules/withdraw.md) | Retiro emérito y archivo estratégico | Especificación |

**Pipeline Secuencial:**
`ingest → generate → validate → incubate → manage → execute → feedback → withdraw`

---

## 🧩 Features (Componentes Reutilizables)

Las features son las piezas de **Lógica Pura** (Functional Core) o drivers de infraestructura que son agnósticas a los módulos y pueden ser reutilizadas en diferentes contextos.

| Feature | Descripción | Consumido por |
|---------|-------------|---------------|
| [**adaptive-logic-er**](./features/adaptive-logic-er.md) | El Adaptive Logic basado en el Efficiency Ratio (ER) de Kaufman es un filtro de calidad de la señal. Su objetivo es asegurar que el Alpha detectado... | validate |
| [**adaptive-volume-indicators**](./features/adaptive-volume-indicators.md) | Esta suite de indicadores avanzados se aleja de los promedios estáticos para adaptarse a la volatilidad y liquidez del mercado. Incluye indicadores... | ingest, generate |
| [**advanced-trade-management**](./features/advanced-trade-management.md) | Es el **gestor operativo base de transacciones**. Implementa reglas tradicionales de control de órdenes que permiten estructurar operaciones... | execute, manage |
| [**adversarial-noise-agent**](./features/adversarial-noise-agent.md) | El "Adversarial Noise Agent" o Red Team AI es el gran villano del auditor de robustez. Es un componente que ejecuta **Data Perturbation**... | validate |
| [**algorithmic-bars**](./features/algorithmic-bars.md) | Es un procesador de datos de mercado que transforma el flujo de ticks en barras basadas en eventos de precio o volumen, en lugar de intervalos de... | ingest |
| [**alpha-decoupling**](./features/alpha-decoupling.md) | Aisla el rendimiento puro de la estrategia (Alpha) eliminando el efecto inercial del mercado general (Beta). Resuelve el problema de falsos... | manage, validate |
| [**alpha-harvesting-gateway**](./features/alpha-harvesting-gateway.md) | Es un portal de ingesta (Gateway) que permite recibir, desencriptar y refinar estrategias anonimizadas provenientes de la mente colectiva (o peers)... | generate |
| [**anomaly-detector**](./features/anomaly-detector.md) | Componente encargado de detectar comportamientos atípicos y fallos de modelo. Su misión es el **Aprendizaje de Fallas**: transforma anomalías... | feedback |
| [**ast-compiler**](./features/ast-compiler.md) | El **Compilador de Árbol de Sintaxis Abstracta (AST)** es el primer filtro del protocolo **Zero-Trust Validation**. Traduce el diseño visual de la... | generate |
| [**async-job-executor**](./features/async-job-executor.md) | Async Job Executor implementa un patrón de tres fases para ejecutar operaciones computacionalmente costosas (backtesting, generación, optimización)... | Todos |
| [**audit-event-store**](./features/audit-event-store.md) | El Audit Event Store es el historial inmutable de vida de todas las decisiones, señales y órdenes del sistema. Implementa el patrón **Event Sourcing... | execute |
| [**audit-log**](./features/audit-log.md) | El Audit Log es el registro histórico inmutable de todos los eventos significativos del sistema. Cada cambio de estado, cada decisión de trading,... | Todos |
| [**auto-auditoria-portafolios-vivos**](./features/auto-auditoria-portafolios-vivos.md) | La auto-auditoría de portafolios vivos es un sistema de monitoreo en tiempo real que protege el capital operativo de la degradación de la... | execute, feedback |
| [**autoencoder-outlier-detector**](./features/autoencoder-outlier-detector.md) | Detector de anomalías multidimensionales en el flujo de transacciones de una estrategia mediante un modelo de Autoencoder neuronal. Evalúa si el... | validate |
| [**autopilot-metrics-provider**](./features/autopilot-metrics-provider.md) | Es el **proveedor dinámico de métricas del Autopilot (Módulo Execute)**. Expone métricas en tiempo real al Dashboard para que el usuario pueda... | execute, feedback |
| [**background-download-manager**](./features/background-download-manager.md) | Es el orquestador visual y técnico que permite gestionar las descargas de datos históricos en segundo plano sin bloquear la interfaz de usuario.... | ingest |
| [**backtest-engine**](./features/backtest-engine.md) | El Backtest Engine simula cómo se habría comportado una estrategia en el pasado, usando datos históricos reales de barras. Devuelve métricas de... | generate, validate |
| [**bayesian-optimizer**](./features/bayesian-optimizer.md) | El **Optimizador Bayesiano** es un motor de búsqueda inteligente de parámetros que utiliza modelos probabilísticos (Procesos Gaussianos) para... | generate |
| [**binary-arrow-transport**](./features/binary-arrow-transport.md) | El **Transporte Binario Arrow** permite la transmisión de alta velocidad de grandes conjuntos de datos (series temporales, curvas de equidad) entre... | Todos |
| [**broker-connector**](./features/broker-connector.md) | Abstrae la comunicación con brokers externos (Binance, IBKR, Oanda). Se apalanca en los **Adaptadores Nativos de NautilusTrader** para garantizar... | execute |
| [**clock**](./features/clock.md) | El Clock es un puerto inyectado que proporciona el tiempo actual a cualquier módulo que lo necesite. En producción devuelve el Unix timestamp real.... | Todos |
| [**complexity-penalization**](./features/complexity-penalization.md) | La penalización por complejidad es la aplicación directa de la Navaja de Ockham ("en igualdad de condiciones, la explicación más sencilla suele ser... | validate |
| [**component-isolation**](./features/component-isolation.md) | El "Monkey Test" (Aislamiento de Componentes) es una auditoría de sentido común estadístico. Su objetivo es evitar la falsa confianza en una... | validate |
| [**contextual-fitness-scorer**](./features/contextual-fitness-scorer.md) | Motor de **fitness contextual multi-régimen**. En lugar de un único número estático de calidad (como el "Weighted Fitness" de SQX, donde fijas un... | validate, manage |
| [**copy-trading-engine**](./features/copy-trading-engine.md) | El Motor de Copy-Trading permite a los traders maestros (Masters) distribuir la ejecución de sus estrategias en tiempo real a múltiples clientes... | execute |
| [**cpcv-analyzer**](./features/cpcv-analyzer.md) | El **CPCV Analyzer** es el motor de validación cruzada de grado institucional del sistema. Su función es particionar los datos históricos en miles... | validate |
| [**crash-recovery**](./features/crash-recovery.md) | Módulo de contingencia ante fallos físicos de infraestructura (cortes de luz, desconexión de red, reinicios inesperados del sistema operativo). Su... | execute, feedback |
| [**cross-filtering-visualizer**](./features/cross-filtering-visualizer.md) | El **Visualizador de Vistas Coordinadas** (Cross-Filtering) es un componente de análisis que presenta múltiples histogramas interactivos... | validate |
| [**cross-market-validation**](./features/cross-market-validation.md) | Una prueba de robustez fundamental que somete la estrategia a la iteración en mercados hermanos (correlacionados) sin reoptimizar parámetros.... | validate |
| [**data-bus-pubsub**](./features/data-bus-pubsub.md) | En sistemas con múltiples agentes operando simultáneamente, la redundancia de datos de mercado es un cuello de botella crítico. Si 50 estrategias... | Todos |
| [**data-import-wizard**](./features/data-import-wizard.md) | Es el componente que permite al usuario incorporar datos externos (EJ: CSV de MetaTrader, TXT de NinjaTrader) al sistema Drasus Engine. Automatiza... | ingest |
| [**data-normalization-layer**](./features/data-normalization-layer.md) | Es la capa encargada de unificar el caos de diferentes formatos de exchanges y brokers en un estándar interno único. Resuelve el problema de que... | ingest |
| [**data-sanitizer-pipeline**](./features/data-sanitizer-pipeline.md) | Es el guardian de la calidad de datos de Drasus Engine y el cerebro del Módulo Ingest. Implementa un protocolo institucional de 6 capas de limpieza... | ingest |
| [**data-validator**](./features/data-validator.md) | Es el componente encargado de la integridad estructural de los datos de mercado. Su misión es detectar anomalías técnicas (precios negativos, saltos... | ingest |
| [**databank-lake**](./features/databank-lake.md) | El Databank masivo ultra-rápido soluciona la degradación de rendimiento extremo observada al guardar el estado completo en la búsqueda evolutiva de... | ingest |
| [**databank-manager**](./features/databank-manager.md) | El Databank Manager es el almacén centralizado de Alpha de Drasus Engine. Utiliza **Parquet** para almacenamiento de alto rendimiento y **DuckDB**... | ingest |
| [**design-manifest**](./features/design-manifest.md) | El Design Manifest es el "Vigilante de la Puerta" (Filtro de Calidad) final antes de que una estrategia sea promovida a incubación o ejecución en... | generate, manage |
| [**dsr-tracking-engine**](./features/dsr-tracking-engine.md) | El DSR Tracking Engine es el encargado de registrar el volumen de intentos y la varianza de los resultados durante la fase de minería genética.... | generate |
| [**dtw-adaptive-window**](./features/dtw-adaptive-window.md) | Es el motor de **segmentación temporal adaptativa** de Drasus Engine. Reemplaza el corte de historia en bloques rígidos de tamaño fijo (el defecto... | validate, manage |
| [**duckdb-resampler**](./features/duckdb-resampler.md) | Es el motor analítico que permite crear temporalidades personalizadas (ej. 7m, 21m, 1h 34m) a partir de datos base de alta frecuencia (1m o Ticks)... | generate |
| [**duckdb-sql-engine**](./features/duckdb-sql-engine.md) | Es el motor analítico central de Drasus Engine para **Procesamiento Analítico en Línea (OLAP)**. Proporciona una interfaz para ejecutar consultas... | Todos |
| [**efficiency-incubation-dashboard**](./features/efficiency-incubation-dashboard.md) | El `Efficiency & Incubation Dashboard` es la interfaz de visualización y control del período de incubación (cuarentena) de las estrategias de... | incubate |
| [**equity-curve-tracker**](./features/equity-curve-tracker.md) | Mantiene un registro barra-por-barra (o tick-por-tick) del capital, beneficio/pérdida, y drawdown máximo consumiendo los eventos de `PositionClosed`... | validate, execute |
| [**event-driven-pipeline-triggers**](./features/event-driven-pipeline-triggers.md) | El sistema de disparadores de pipelines basado en eventos permite automatizar la ejecución de flujos de descubrimiento y validación de estrategias... | generate, validate |
| [**executable-container**](./features/executable-container.md) | El Executable Container es un contrato técnico (Interface o Abstract Base Class) que estandariza cómo viajan los datos de una Estrategia o un... | incubate, execute |
| [**factor-decomposition**](./features/factor-decomposition.md) | Es el motor analítico encargado de realizar la **"Análisis Forense del Retorno"**. Descompone el rendimiento de cualquier estrategia en sus... | validate, feedback |
| [**feature-router**](./features/feature-router.md) | Feature Router implementa un mecanismo para activar/desactivar features dinámicamente en tiempo de startup, sin hardcodear qué features están... | generate |
| [**federated-portfolio**](./features/federated-portfolio.md) | El **Federated Portfolio** es una arquitectura avanzada que permite la coexistencia y coordinación de múltiples portafolios independientes y... | manage |
| [**fit-to-portfolio-search**](./features/fit-to-portfolio-search.md) | Inyección del estado del portafolio vivo como una presión restrictiva en el motor evolutivo (NSGA-II). Resuelve el problema de generar cientos de... | manage |
| [**flutter-packaging-manager**](./features/flutter-packaging-manager.md) | El **Manejador de Empaquetado de Flutter FFI** orquesta el ciclo de vida del binario congelado de Rust (backend + assets frontend) utilizando... | execute |
| [**fractional-differencer**](./features/fractional-differencer.md) | El **Fractional Differencer** es una herramienta de procesamiento de series temporales que permite transformar una serie no-estacionaria (como los... | ingest, generate |
| [**fragility-gradient-auditor**](./features/fragility-gradient-auditor.md) | La Auditoría de Gradiente de Fragilidad Descendente es la evolución (New Era) del clásico análisis de varianza/mediana. | validate, feedback |
| [**glass-box-ai-translator**](./features/glass-box-ai-translator.md) | Es el sistema puente que elimina el "código espagueti" y las "cajas negras" propias de las redes neuronales usadas en trading. Transforma la... | generate, validate |
| [**hierarchical-parameter-optimization**](./features/hierarchical-parameter-optimization.md) | La Optimización Jerárquica de Parámetros es un proceso controlado de mapeo secuencial. En lugar de optimizar todas las variables a la vez ("fuerza... | generate |
| [**hive-partition-manager**](./features/hive-partition-manager.md) | Es el componente encargado de organizar físicamente los archivos Parquet en el disco del usuario. Utiliza la estructura de directorios... | ingest |
| [**hmm-regime-detection**](./features/hmm-regime-detection.md) | El motor de detección de regímenes utiliza **Modelos Ocultos de Markov (HMM)** y modelos **ARIMA** para clasificar el entorno macro y... | ingest, generate |
| [**hybrid-data-transformer**](./features/hybrid-data-transformer.md) | Es el motor de transformación que aplica la regla **80/20**: utiliza el máximo rendimiento de **Polars** para el procesamiento masivo (ETL,... | ingest, validate |
| [**incremental-test-engine**](./features/incremental-test-engine.md) | El **Incremental Test Engine** es un motor transversal de optimización computacional que permite al sistema de validación (The Torture Chamber)... | Todos |
| [**incubation-manager**](./features/incubation-manager.md) | El Incubation Manager es el componente transversal responsable de orquestar el periodo de prueba final en tiempo real (Paper Trading) antes de... | incubate |
| [**infrastructure-setup**](./features/infrastructure-setup.md) | Antes de escribir cualquier módulo del sistema, necesitamos preparar el terreno: la estructura de carpetas, la base de datos, el sistema de logs, y... | Todos |
| [**institutional-friction-modeling**](./features/institutional-friction-modeling.md) | El motor de **Institutional Friction Modeling** inyecta realismo probabilístico en la ejecución de órdenes Límite. Modela el fenómeno de **Adverse... | validate, backtest-engine |
| [**institutional-metrics**](./features/institutional-metrics.md) | Es la "Calculadora Maestra" del sistema. Mide qué tan buena, mala o riesgosa es una estrategia. En lugar de calcular todo al mismo tiempo y trabar... | Todos |
| [**interactive-stress-lab**](./features/interactive-stress-lab.md) | Panel de control **táctil y reactivo** que permite al analista deformar la curva de capital de una estrategia **en tiempo real** moviendo... | validate, feedback |
| [**kinetic-micro-management**](./features/kinetic-micro-management.md) | Es el **módulo defensivo hostil** de la nueva escuela. Provee protecciones reactivas agresivas y de alta velocidad diseñadas para contrarrestar la... | execute, manage |
| [**licensing-system**](./features/licensing-system.md) | El sistema de licenciamiento regula los niveles de acceso del usuario al ecosistema Drasus Engine sin comprometer la privacidad o el rendimiento... | Todos |
| [**manual-regime-tagger**](./features/manual-regime-tagger.md) | Herramienta de **etiquetado visual manual de regímenes históricos**. Complementa la detección automática (`hmm-regime-detection`, `regime-guard`,... | ingest, validate |
| [**monte-carlo-simulator**](./features/monte-carlo-simulator.md) | Es un analizador estadístico de permutación y remuestreo que opera en **dos modos independientes** dentro del guantelete de robustez. El Scoring... | validate, feedback |
| [**monthly-performance-heatmap**](./features/monthly-performance-heatmap.md) | El `Monthly Performance Heatmap` es un componente visual analítico que muestra el rendimiento porcentual mensual de una estrategia o portafolio en... | validate, feedback |
| [**multi-ticket-manager**](./features/multi-ticket-manager.md) | El gestor de múltiples posiciones por estrategia es un componente que rompe la limitación tradicional de SQX ("una sola operación a la vez").... | execute |
| [**multiplatform-execution-bridge**](./features/multiplatform-execution-bridge.md) | El puente de ejecución multiplataforma es un desacoplador de órdenes y capa de abstracción diseñado para comunicar nuestro entorno de ejecución en... | execute |
| [**nautilus-integration**](./features/nautilus-integration.md) | Es la capa de adaptación que permite a Drasus Engine usar NautilusTrader como motor de ejecución y backtesting sin quedar acoplados permanentemente... | execute, validate |
| [**node-preview**](./features/node-preview.md) | El Micro-Backtest Node Preview provee al operador de Drasus Engine una retroalimentación visual instantánea de la curva de balance y métricas de... | generate |
| [**notification**](./features/notification.md) | Abstrae canales de notificación (email, webhook, Slack, SMS). El Core dispara eventos sin saber por qué canal se enviarán. --- | execute, manage |
| [**nsga2-optimizer**](./features/nsga2-optimizer.md) | NSGA-II es un algoritmo de optimización que busca configuraciones de estrategia que sean buenas en múltiples objetivos simultáneamente (Sharpe,... | generate, manage |
| [**operational-safety-monitor**](./features/operational-safety-monitor.md) | El **Operational Safety Monitor** es el guardián de la integridad del capital en tiempo real. Combina el monitoreo de deriva estadística (**Pardo... | execute, feedback |
| [**order-flow-microstructure**](./features/order-flow-microstructure.md) | Esta feature provee las métricas de alta frecuencia necesarias para detectar la presión institucional y la absorción de liquidez. A diferencia de... | ingest, execute |
| [**order-fsm**](./features/order-fsm.md) | El Order FSM define los 6 estados posibles de una orden y las transiciones válidas entre ellos. Una orden es un contrato para comprar o vender un... | execute, manage |
| [**order-priority-queue**](./features/order-priority-queue.md) | Es una **cola inteligente de órdenes** diseñada para mitigar los límites de tasa (*rate limits*) impuestos por los exchanges. Durante episodios de... | execute |
| [**paper-trader**](./features/paper-trader.md) | Es el componente encargado de ejecutar una estrategia en tiempo real sin riesgo de capital. Su misión es la **Simulación de Alta Fidelidad**: operar... | incubate |
| [**parallel-coordinates-visualizer**](./features/parallel-coordinates-visualizer.md) | El **Visualizador de Coordenadas Paralelas** es un componente de análisis visual de alta densidad que permite proyectar optimizaciones de más de 20... | validate, generate |
| [**parameter-optimization**](./features/parameter-optimization.md) | Busca los parámetros óptimos de una estrategia usando Grid Search (exhaustivo) o Bayesian Search (inteligente). --- | generate |
| [**pardo-comparison**](./features/pardo-comparison.md) | Componente estadístico que valida la consistencia entre dos series de resultados (ej: Backtest Histórico vs Paper Trading Vivo). Su misión es ser el... | incubate, feedback |
| [**pca-toxicity-analyzer**](./features/pca-toxicity-analyzer.md) | Es un módulo de validación avanzada que aplica técnicas de aprendizaje no supervisado para agrupar y purgar familias de estrategias que demuestran... | validate, feedback |
| [**pdf-charts-rendering**](./features/pdf-charts-rendering.md) | El `PDF Charts Rendering` es el componente de backend (server-side/headless) encargado de generar y renderizar gráficos vectoriales estáticos de... | validate, feedback |
| [**perfect-profit-benchmark**](./features/perfect-profit-benchmark.md) | El Perfect Profit Benchmark es un filtro de eficiencia del modelo (ME). Su misión es medir qué porcentaje del beneficio teórico máximo captura la... | validate, feedback |
| [**performance-monitor**](./features/performance-monitor.md) | Componente de vigilancia encargado de detectar la degradación del rendimiento en vivo. Su misión es la **Prevención de Quiebra (Bankruptcy... | Todos |
| [**persistent-daemons**](./features/persistent-daemons.md) | En un entorno de trading institucional, la latencia y la estabilidad son críticas. Mientras que los procesos de Investigación y Desarrollo (R&D)... | manage, execute |
| [**pit-data-validator**](./features/pit-data-validator.md) | Valida que los datos históricos son "Point-In-Time" (PIT-real): información que realmente estaba disponible en ese momento específico, sin... | ingest, validate |
| [**plateau-copilot**](./features/plateau-copilot.md) | Asistente visual de **auditoría topológica manual** de parámetros. El motor de fuerza bruta (no LLM) ya existe (`parameter-optimization`,... | validate, generate |
| [**portfolio-backtest**](./features/portfolio-backtest.md) | El **Portfolio Backtest** es el componente de simulación avanzada encargado de evaluar el comportamiento histórico conjunto de múltiples estrategias... | validate |
| [**portfolio-data-preparation**](./features/portfolio-data-preparation.md) | Es el sistema encargado de preparar las fundaciones matemáticas y de datos para el análisis de portafolio multi-estrategia. Esta preparación ocurre... | generate, manage |
| [**portfolio-optimizer**](./features/portfolio-optimizer.md) | El **Portfolio Optimizer** es el motor matemático central de gestión y rebalanceo de capital (Asset Allocation). Combina múltiples estrategias... | manage |
| [**portfolio-rules**](./features/portfolio-rules.md) | Componente de gobernanza encargado de imponer los límites de seguridad globales del portafolio. Actúa como el **Filtro de Invariantes** y una **Capa... | manage, execute |
| [**pre-trade-validator**](./features/pre-trade-validator.md) | Componente de alta velocidad encargado de validar cada orden contra **11 filtros de seguridad** críticos antes de permitir su salida al mercado real... | execute |
| [**precision-sizing-models**](./features/precision-sizing-models.md) | Proporciona un framework unificado y determinista para el cálculo del tamaño de las posiciones. Este motor es consumido por los módulos de... | Todos |
| [**production-optimization**](./features/production-optimization.md) | Después de que todos los módulos están implementados y benchmarked **individualmente**, esta fase se ejecuta **solo si hay cuellos de botella... | execute, manage |
| [**prop-firm-grader**](./features/prop-firm-grader.md) | Es un **verdugo implacable**. Las firmas de fondeo modernas (como FTMO o TopStep) tienen reglas clarísimas y muy estrictas: si pierdes más de un 5%... | validate, execute |
| [**quality-heatmap-generator**](./features/quality-heatmap-generator.md) | Es el componente responsable de auditar la integridad de los datos históricos y generar una representación visual (Heatmap) de su calidad. Permite... | validate |
| [**quantops-daemon**](./features/quantops-daemon.md) | QuantOps Daemon es la evolución de la automatización manual de flujos de trabajo hacia una infraestructura de "Continuous Integration / Continuous... | Todos |
| [**regime-guard**](./features/regime-guard.md) | Componente de coherencia de mercado encargado de imponer la compatibilidad entre el modelo y el entorno. Su misión es la **Prevención de Mismatch**:... | validate |
| [**remote-portfolio-access-protocol**](./features/remote-portfolio-access-protocol.md) | Protocolo de acceso remoto autenticado con seguridad a nivel de campo (Field-Level Security). Expone una interfaz analítica de solo lectura... | execute, manage |
| [**robust-reporting**](./features/robust-reporting.md) | Genera reportes estáticos (JSON/HTML) hiper-detallados de una estrategia o portafolio, incluyendo curvas de equity hiper-resolución, distribuciones... | validate, feedback |
| [**robustness-score-aggregator**](./features/robustness-score-aggregator.md) | El agregador de score de robustez es el motor de consolidación que reemplaza el viejo enfoque binario de "Muerte Súbita". Toma los 5 resultados... | validate, execute, feedback |
| [**robustness-verdict-engine**](./features/robustness-verdict-engine.md) | El Robustness Verdict Engine es un motor de interpretación semántica que convierte los resultados crudos del guantelete de robustez en un veredicto... | validate, feedback |
| [**rule-ablation**](./features/rule-ablation.md) | La Ablación de Reglas es una técnica de simplificación y validación de robustez que consiste en desactivar sistemáticamente componentes lógicos de... | validate |
| [**secure-updater**](./features/secure-updater.md) | El actualizador seguro gestiona el ciclo de vida de las actualizaciones de software del núcleo binario de Rust y la interfaz gráfica de Flutter,... | Todos |
| [**signal-correlation-analyzer**](./features/signal-correlation-analyzer.md) | Calcula matrices de correlación entre señales (señal vs señal) y entre señales y factores de mercado. Proporciona auditoría visual de... | feedback |
| [**slippage-models**](./features/slippage-models.md) | Es el componente que inyecta "realismo institucional" a las ejecuciones. Se apalanca en los modelos de impacto de **NautilusTrader** (ADR-0013) para... | validate, execute |
| [**sovereign-data-fetcher**](./features/sovereign-data-fetcher.md) | Es el componente encargado de saturar el ancho de banda para la obtención masiva de históricos. Resuelve el problema de la lentitud de las APIs REST... | ingest |
| [**sovereign-security**](./features/sovereign-security.md) | Sovereign Security establece el marco de ciberseguridad local e integridad de datos para Drasus Engine. Protege las credenciales sensibles del... | execute, feedback |
| [**statistical-inference-ebta**](./features/statistical-inference-ebta.md) | La capa EBTA (Evidence-Based Technical Analysis) es el filtro de rigor estadístico final del módulo `validate`. Su objetivo es cuantificar la... | validate |
| [**strategy-ast-copilot**](./features/strategy-ast-copilot.md) | Asistente manejado por LLM que traduce lenguaje natural en la estructura de árbol determinista (AST) que gobierna las reglas de una estrategia.... | generate |
| [**strategy-config-diff**](./features/strategy-config-diff.md) | El `Strategy Config Diff` es una herramienta visual en la interfaz de usuario que permite comparar la configuración de parámetros activa de un... | manage |
| [**strategy-ensemble**](./features/strategy-ensemble.md) | Orquesta canales (NSGA-II, Simbólico nativo, HMM) en estrategias híbridas mediante **Fusión de Pareto** y **Mayoría Ponderada**, gestionando la **Asimetría... | generate |
| [**strategy-self-explanation**](./features/strategy-self-explanation.md) | Módulo de auditoría que traduce un Árbol de Sintaxis Abstracta (AST) críptico (típicamente vomitado por el motor evolutivo genético) a un párrafo de... | generate |
| [**strategy-versioning**](./features/strategy-versioning.md) | El Strategy Versioning implementa un sistema de historial completo similar a Git para estrategias y portafolios. Cada modificación a la... | manage, feedback |
| [**system-watchdog**](./features/system-watchdog.md) | El protector de última instancia del sistema. Su misión es la **Prevención de Ruina**: monitorea continuamente la salud técnica (latencia, conexión)... | execute, feedback |
| [**telemetry**](./features/telemetry.md) | Es el componente encargado de capturar y persistir métricas de **performance técnica** y estado de salud del sistema en tiempo real. A diferencia... | execute, feedback |
| [**temporal-aggregator**](./features/temporal-aggregator.md) | Es un motor de procesamiento de series temporales encargado de agrupar ticks o barras de alta frecuencia en intervalos de tiempo arbitrarios y no... | ingest, validate |
| [**throttling-metrics-dashboard**](./features/throttling-metrics-dashboard.md) | El `Throttling Metrics Dashboard` provee visualización en tiempo real y diagnósticos de latencia en la capa de conectividad con los brokers.... | execute |
| [**time-warp-debugger**](./features/time-warp-debugger.md) | El Time-Warp Debugger es el motor de reproducción forense y depuración de Drasus Engine. Provee una línea de tiempo interactiva en la UI que permite... | validate |
| [**topological-plateau-finder**](./features/topological-plateau-finder.md) | El "Buscador Topológico de Mesetas" es un analizador del "vecindario" hiperespacial de parámetros (Optimization Profile). Evalúa automáticamente la... | validate, generate |
| [**toxicity-purifier**](./features/toxicity-purifier.md) | El Toxicity Purifier es el componente visual e interactivo encargado de la curación y purga masiva de clústeres de estrategias tóxicas identificadas... | validate |
| [**trade-analysis-bi-suite**](./features/trade-analysis-bi-suite.md) | El `Trade Analysis BI Suite` es una colección integrada de gráficos estadísticos avanzados y cuadros de control analíticos pre-calculados en el... | feedback |
| [**trade-reconciler**](./features/trade-reconciler.md) | Componente encargado de la **Feedback de ejecución de la Operativa**. Su misión es la reconciliación diaria: compara la realidad cruda del broker... | feedback |
| [**umap-scatter-visualizer**](./features/umap-scatter-visualizer.md) | El UMAP Scatter Visualizer es una herramienta del lienzo Meso/Micro que permite al operador explorar visualmente el espacio de robustez de miles de... | validate |
| [**universal-basket-backtester**](./features/universal-basket-backtester.md) | Es un motor de orquestación de simulaciones diseñado para evaluar una estrategia (o un conjunto de ellas) sobre múltiples activos y temporalidades... | generate, validate |
| [**vector-time-pruning**](./features/vector-time-pruning.md) | Imagina un robot trader que gana dinero consistentemente casi toda la semana, pero por alguna razón, **siempre** pierde dinero los viernes a las... | validate, manage |
| [**visual-dag-editor**](./features/visual-dag-editor.md) | El Visual DAG Editor es la herramienta de diseño gráfico y configuración de alto nivel de Drasus Engine. Utiliza un lienzo interactivo renderizado... | generate |
| [**visual-downsampling-service**](./features/visual-downsampling-service.md) | Es un servicio de procesamiento en el backend que reduce el número de puntos de datos de una serie temporal masiva (ej: un millón de ticks) a una... | validate, feedback |
| [**visual-stockpicker-configurator**](./features/visual-stockpicker-configurator.md) | El `Visual StockPicker Configurator` es la interfaz gráfica que permite al operador configurar los criterios de selección y rotación del universo de... | generate, manage |
| [**volatility-stabilization**](./features/volatility-stabilization.md) | El motor de **Volatility Stabilization** garantiza que las estrategias operen bajo un perfil de riesgo constante (Target Vol) y sean certificadas... | validate, execute |
| [**volume-profile-router**](./features/volume-profile-router.md) | El **Ruteador por Perfil de Volumen** es una capa de seguridad en la ejecución que suspende automáticamente las órdenes ante caídas de liquidez... | validate, execute |
| [**walk-forward-analyzer**](./features/walk-forward-analyzer.md) | Es el motor de validación dinámica de Drasus Engine. Utiliza una **Matriz WFA** y el método **CPCV (Cross-Validation Combinatorial)** con técnicas... | validate, manage |
| [**worker-isolation-orchestrator**](./features/worker-isolation-orchestrator.md) | El **Orquestador de Aislamiento de Trabajadores** gestiona la ejecución de tareas pesadas (simulaciones, entrenamientos de IA) en procesos... | Todos |
| [**zero-crossing-filter**](./features/zero-crossing-filter.md) | Filtra señales de trading para detectar aquellas que son ortogonales (independientes) respecto a factores de mercado conocidos. Detecta los momentos... | generate |
| [**zui-navigation**](./features/zui-navigation.md) | La **Zoomable User Interface (ZUI)** es el paradigma de navegación espacial y contextual de Drasus Engine. En lugar de pantallas y menús aislados... | validate, generate |

---

---

## 🚀 Moonshots (Experimental)

Proyectos de investigación avanzada y alta complejidad.

| Moonshot | Descripción |
|----------|-------------|
| [**advanced-equities-engine**](./moonshots/advanced-equities-engine.md) | Motor avanzado especializado en negociación de acciones, incluyendo análisis fundamental, screening de valores y optimización de cartera de renta va... |
| [**ai-dimensionality-suite**](./moonshots/ai-dimensionality-suite.md) | Compresión UMAP y detección outliers Autoencoder |
| [**alternative-data-fabric**](./moonshots/alternative-data-fabric.md) | Orquestador visual de datos alternativos (sentimiento, satélite) con alineación PIT |
| [**auto-hedger**](./moonshots/auto-hedger.md) | Cirugía de curva de capital: generación dirigida de cobertura inversa (Targeted DD Patching) |
| [**causal-inference-discovery**](./moonshots/causal-inference-discovery.md) | Descubrimiento de Relaciones Causales (DoWhy/PC) |
| [**cellular-automata-logic-growth**](./moonshots/cellular-automata-logic-growth.md) | Crecimiento procedural de lógica vía autómatas celulares (no validado) |
| [**collective-intelligence**](./moonshots/collective-intelligence.md) | Es el framework de inteligencia colectiva distribuida que feddera múltiples estrategias independientes para colaborar sin compartir datos sensibles.... |
| [**compliance-dashboard**](./moonshots/compliance-dashboard.md) | Es el panel de cumplimiento regulatorio que monitorea en vivo todas las posiciones y operaciones contra restricciones de cumplimiento (CME, FSA, S... |
| [**conviction-scoring-engine**](./moonshots/conviction-scoring-engine.md) | Conviction Score 0-100 multi-factor para sizing Kelly dinámico |
| [**deep-learning-suite**](./moonshots/deep-learning-suite.md) | Es la suite de aprendizaje profundo que integra redes neuronales LSTM, Transformer, GRU para predicción de series de tiempo y clasificación de reg... |
| [**drl-parameter-tuning**](./moonshots/drl-parameter-tuning.md) | Ajuste de parámetros mediante Reinforcement Learning |
| [**drl-portfolio-optimization**](./moonshots/drl-portfolio-optimization.md) | Es la optimización de portafolios usando DRL (Deep Reinforcement Learning). Entrena un agente que toma decisiones de asignación de capital dinámicas... |
| [**figma-style-canvas**](./moonshots/figma-style-canvas.md) | Es el lienzo de diseño visual tipo Figma que permite construir estrategias arrastrando nodos, conectando, scalando, anidando. Implementa un edito... |
| [**fix-api-execution**](./moonshots/fix-api-execution.md) | Ejecución institucional FIX API, Edge Computing y simulación de impacto de mercado (SOR) |
| [**fuzzy-logic-evaluator**](./moonshots/fuzzy-logic-evaluator.md) | Es el evaluador de lógica borrosa (fuzzy logic) que permite reglas suaves ("casi buy", "moderadamente bullish") en lugar de binarias. Implementa... |
| [**gans-universos-sinteticos**](./moonshots/gans-universos-sinteticos.md) | Generación de microestructura de mercado sintética hiperrealista |
| [**genoma-ejecucion-enrutamiento**](./moonshots/genoma-ejecucion-enrutamiento.md) | Quinto dominio genómico candidato (Ejecución y Enrutamiento) evaluado y excluido del Registro de Dominios Genómicos (ADR-0108) por falta de datos... |
| [**gnn-contagio-macro**](./moonshots/gnn-contagio-macro.md) | Modelado de contagio macro y propagación de shocks financieros |
| [**god-mode-edge**](./moonshots/god-mode-edge.md) | Es el modo "dios" (debugging avanzado) que permite al desarrollador hacer stepping temporal, modificar estado histórico, y re-simular portafolios. ... |
| [**hybrid-prompting-ui**](./moonshots/hybrid-prompting-ui.md) | Es la interfaz de prompting híbrida que combina entrada visual (drag-drop) con lenguaje natural. Permite a usuarios no-técnicos describir estrategi... |
| [**institutional-plugin-system**](./moonshots/institutional-plugin-system.md) | Es el sistema de plugins institucionales que permite que fondos construyan componentes propietarios (custom risk models, execution algorithms) que... |
| [**interactive-chat-loop**](./moonshots/interactive-chat-loop.md) | Es el bucle interactivo de chat que permite conversación iterativa con un asistente IA para refinar estrategias. "Hazme una estrategia alcista"... |
| [**knowledge-graphs-galaxias**](./moonshots/knowledge-graphs-galaxias.md) | Grafo evolutivo e histórico de linaje con explorador espacial 3D |
| [**la-colmena**](./moonshots/la-colmena.md) | Minería descentralizada de estrategias mediante nodos distribuidos |
| [**marketplace-cajas-negras**](./moonshots/marketplace-cajas-negras.md) | Permite a creadores de estrategias empaquetar, encriptar y monetizar subgrafos complejos de lógica visual como un solo nodo cerrado "Caja Negra" de... |
| [**meta-learning-hub**](./moonshots/meta-learning-hub.md) | El Meta-Learning Hub implementa el concepto de "Aprender a Aprender". En lugar de optimizar una estrategia aislada, el sistema analiza el éxito y f... |
| [**microestructura-l3**](./moonshots/microestructura-l3.md) | Explora la simulación y análisis de estrategias cuantitativas basadas en datos de Nivel 3 L3 - Market-by-Order MBO. A diferencia de L1 Bid/Ask y L2... |
| [**monetization-stripe**](./moonshots/monetization-stripe.md) | El sistema de monetización conecta el ecosistema de facturación externa Stripe con la estructura de control de accesos del SaaS, regulando qué cara... |
| [**neuro-symbolic-fusion**](./moonshots/neuro-symbolic-fusion.md) | Fusión neuro-simbólica de estrategias maestras (El Colisionador) |
| [**predictive-quant-oracles**](./moonshots/predictive-quant-oracles.md) | Inferencia bayesiana de fragilidad y estancamiento futuro (Alpha no validado) |
| [**pysr-signal-discovery**](./moonshots/pysr-signal-discovery.md) | Descubrimiento simbólico de señales alpha |
| [**quantum-portfolio-solver**](./moonshots/quantum-portfolio-solver.md) | Explora el uso de algoritmos de Computación Cuántica para resolver problemas de optimización combinatoria complejos, como la selección de activos y... |
| [**saas-cloud-engine**](./moonshots/saas-cloud-engine.md) | Arquitectura y orquestación masiva para despliegue Cloud/VPS de alta densidad |
| [**saas-gateway**](./moonshots/saas-gateway.md) | El Gateway central de acceso regula los flujos de comunicación externa en la nube entre los Thin Clients Flutter local y el clúster de ejecución or... |
| [**shared-capital-pool**](./moonshots/shared-capital-pool.md) | El Shared Capital Pool es un módulo de investigación y desarrollo diseñado para permitir que múltiples portafolios federados ej. Portafolio A y Por... |
| [**shield-netting-translator**](./moonshots/shield-netting-translator.md) | El traductor de compensación es una capa intermedia que actúa como envoltorio de set algorítmicos para compactar operaciones subyacentes de cobertu... |
| [**simulador-adversarial**](./moonshots/simulador-adversarial.md) | Es un motor de simulación alternativo que, en lugar de evaluar una estrategia contra datos históricos estáticos del mercado, crea y modela un Libro... |
| [**sovereign-execution-engine**](./moonshots/sovereign-execution-engine.md) | Motor de ejecución propio multi-activo: contingencia de salida del ADR-0107 si el upstream de NautilusTrader se abandona (acciones, forex, futuros... |
| [**tda-phase-space-isolation**](./moonshots/tda-phase-space-isolation.md) | Aislamiento de co-colapso de cola vía Análisis de Datos Topológicos |
| [**topografia-3d-liquidez**](./moonshots/topografia-3d-liquidez.md) | Es un modo de visualización avanzado que renderiza el historial del Order Book y las zonas de liquidez acumuladas como un modelo tridimensional 3D ... |
| [**universal-strategy-transpiler**](./moonshots/universal-strategy-transpiler.md) | Permite exportar y traducir de forma nativa la lógica del Grafo de Lógica visual Strategy AST de Drasus Engine a múltiples lenguajes de programación y... |

---

## 📋 Registro de Decisiones de Arquitectura (ADRs)

Registro ordenado de las decisiones de diseño clave que gobiernan la arquitectura y evolución de Drasus Engine.

| ADR | Decisión / Título |
|-----|-------------------|
| [**ADR-0001**](./ADR.md#adr-0001-monolito-modular-fcis) | Monolito Modular + FCIS |
| [**ADR-0002**](./ADR.md#adr-0002-desacoplamiento-de-persistencia) | Desacoplamiento de Persistencia |
| [**ADR-0003**](./ADR.md#adr-0003-organizacin-de-mdulos-fcis-features-reutilizables) | Organización de Módulos (FCIS) + Features Reutilizables |
| [**ADR-0004**](./ADR.md#adr-0004-mquina-de-estados-fsm) | Máquina de Estados (FSM) |
| [**ADR-0005**](./ADR.md#adr-0005-strategy-portfolio-git-like-versioning-con-dag) | Strategy-Portfolio Git-Like Versioning con DAG |
| [**ADR-0006**](./ADR.md#adr-0006-migraciones-centralizadas-con-sqlx-migrator) | Migraciones Centralizadas con SQLx Migrator |
| [**ADR-0007**](./ADR.md#adr-0007-inyeccin-dinmica-de-comportamiento-feature-router) | Inyección Dinámica de Comportamiento (Feature Router) |
| [**ADR-0008**](./ADR.md#adr-0008-configurabilidad-universal-todo-es-parmetro-excepto-invariantes) | Configurabilidad Universal (TODO es Parámetro, Excepto Invariantes) |
| [**ADR-0009**](./ADR.md#adr-0009-interfaz-unificada-strategy-portfolio-executablecontainer) | Interfaz Unificada Strategy-Portfolio (ExecutableContainer) |
| [**ADR-0010**](./ADR.md#adr-0010-reglas-dinmicas-hard-limits-vs-soft-alerts) | Reglas Dinámicas (Hard Limits vs Soft Alerts) |
| [**ADR-0011**](./ADR.md#adr-0011-operaciones-asincrnicas-async-job-pattern) | Operaciones Asincrónicas (Async Job Pattern) |
| [**ADR-0012**](./ADR.md#adr-0012-arquitectura-multi-pipeline-paralela-single-machine-architecture) | Arquitectura Multi-Pipeline Paralela (Single Machine Architecture) |
| [**ADR-0013**](./ADR.md#adr-0013-seleccin-de-stack-tecnolgico-high-performance-core) | Selección de Stack Tecnológico (High-Performance Core) |
| [**ADR-0014**](./ADR.md#adr-0014-evolucin-incremental-de-contratos) | Evolución Incremental de Contratos |
| [**ADR-0015**](./ADR.md#adr-0015-arquitectura-de-causalidad-y-aprendizaje-cerrado) | Arquitectura de Causalidad y Aprendizaje Cerrado |
| [**ADR-0016**](./ADR.md#adr-0016-local-first-processing-external-overlays) | Local-First Processing & External Overlays |
| [**ADR-0017**](./ADR.md#adr-0017-simulacin-de-alta-fidelidad-institutional) | Simulación de Alta Fidelidad Institutional |
| [**ADR-0018**](./ADR.md#adr-0018-taxonoma-y-topologa-del-pipeline-los-8-mdulos) | Taxonomía y Topología del Pipeline (Los 8 Módulos) |
| [**ADR-0019**](./ADR.md#adr-0019-interoperabilidad-frontend-backend-ffigrpc) | Interoperabilidad Frontend-Backend (FFI/gRPC) |
| [**ADR-0020 V2**](./ADR.md#adr-0020-v2-principio-de-inundacin-de-fundaciones-v2-foundation-inundation) | Principio de Inundación de Fundaciones V2 (Foundation Inundation) |
| [**ADR-0021**](./ADR.md#adr-0021-modelo-de-decisin-dual-autopilot-con-veto) | Modelo de Decisión Dual (Autopilot con Veto) |
| [**ADR-0022**](./ADR.md#adr-0022-pipeline-no-lineal-dag-multiflujal) | Pipeline No-Lineal (DAG Multiflujal) |
| [**ADR-0023**](./ADR.md#adr-0023-dashboard-dinmico-vs-arquitectura-de-plugins) | Dashboard Dinámico vs Arquitectura de Plugins |
| [**ADR-0024**](./ADR.md#adr-0024-reglas-dominantes-extracted-constraints) | Reglas Dominantes (Extracted Constraints) |
| [**ADR-0025**](./ADR.md#adr-0025-pre-trade-risk-10-steps-gate) | Pre-Trade Risk 10-Steps Gate |
| [**ADR-0026**](./ADR.md#adr-0026-shadow-watchdog-heartbeat) | Shadow Watchdog & Heartbeat |
| [**ADR-0027**](./ADR.md#adr-0027-event-sourcing-inventory-reconstruction) | Event Sourcing & Inventory Reconstruction |
| [**ADR-0028**](./ADR.md#adr-0028-zui-fractal-navigation-orchestratorstrategy-inspector) | ZUI Fractal Navigation (Orchestrator/Strategy Inspector) |
| [**ADR-0029**](./ADR.md#adr-0029-patrn-todo-en-uno-rust-flutter-ffi) | Patrón Todo en Uno (Rust + Flutter FFI) |
| [**ADR-0030**](./ADR.md#adr-0030-persistencia-soberana-zero-docker) | Persistencia Soberana "Zero-Docker" |
| [**ADR-0031**](./ADR.md#adr-0031-inteligencia-artificial-hbrida-hybrid-genesis-engine) | Inteligencia Artificial Híbrida (Hybrid Genesis Engine) |
| [**ADR-0032**](./ADR.md#adr-0032-estndares-de-hardware-soberano-single-machine-sovereignty) | Estándares de Hardware Soberano (Single Machine Sovereignty) |
| [**ADR-0033**](./ADR.md#adr-0033-arquitectura-de-despliegue-trimodal) | Arquitectura de Despliegue Trimodal |
| [**ADR-0034**](./ADR.md#adr-0034-ingesta-hbrida-soberana-bulk-s3-api-delta) | Ingesta Híbrida Soberana (Bulk S3 + API Delta) |
| [**ADR-0035**](./ADR.md#adr-0035-persistencia-en-particionado-hive-style-parquet) | Persistencia en Particionado Hive-Style (Parquet) |
| [**ADR-0036**](./ADR.md#adr-0036-remuestreo-dinmico-multidimensional-duckdb) | Remuestreo Dinámico Multidimensional (DuckDB) |
| [**ADR-0037**](./ADR.md#adr-0037-protocolo-de-calidad-the-sanitizer) | Protocolo de Calidad "The Sanitizer" |
| [**ADR-0038**](./ADR.md#adr-0038-estndar-de-nomenclatura-institucional-sanitizacin-terminolgica) | Estándar de Nomenclatura Institucional (Sanitización Terminológica) |
| [**ADR-0039**](./ADR.md#adr-0039-infraestructura-de-lgica-causal-hbrida-legacy-sqx-sovereign-qf) | Infraestructura de Lógica Causal Híbrida (Legacy SQX + Sovereign QF) |
| [**ADR-0040**](./ADR.md#adr-0040-disparadores-de-seal-metamrficos-capital-aware) | Disparadores de Señal Metamórficos (Capital-Aware) |
| [**ADR-0041**](./ADR.md#adr-0041-arquitectura-de-hemisferios-de-asimetra-estructural) | Arquitectura de Hemisferios de Asimetría Estructural |
| [**ADR-0042**](./ADR.md#adr-0042-arquitectura-de-fitness-metamrfico-de-estado) | Arquitectura de Fitness Metamórfico de Estado |
| [**ADR-0043**](./ADR.md#adr-0043-protocolo-de-programacin-evolutiva-parcial-wildcards) | Protocolo de Programación Evolutiva Parcial (WildCards) |
| [**ADR-0044**](./ADR.md#adr-0044-framework-de-dimensionamiento-de-riesgo-multimodal) | Framework de Dimensionamiento de Riesgo Multimodal |
| [**ADR-0045**](./ADR.md#adr-0045-prop-firm-compliance-profile-ley-de-cero-hardcoding) | Prop-Firm Compliance Profile (Ley de Cero Hardcoding) |
| [**ADR-0046**](./ADR.md#adr-0046-vector-time-pruning-poda-temporal-autnoma) | Vector-Time Pruning (Poda Temporal Autónoma) |
| [**ADR-0047**](./ADR.md#adr-0047-computacin-asimtrica-de-mtricas-hot-path-vs-rd) | Computación Asimétrica de Métricas (Hot-Path vs R&D) |
| [**ADR-0048**](./ADR.md#adr-0048-neutralizacin-analtica-de-beta-alpha-decoupling) | Neutralización Analítica de Beta (Alpha Decoupling) |
| [**ADR-0049**](./ADR.md#adr-0049-validacin-transversal-de-robustez-cross-market-validation) | Validación Transversal de Robustez (Cross-Market Validation) |
| [**ADR-0050**](./ADR.md#adr-0050-bsqueda-generativa-diversificada-fit-to-portfolio-search) | Búsqueda Generativa Diversificada (Fit-to-Portfolio Search) |
| [**ADR-0051**](./ADR.md#adr-0051-determinismo-asistido-por-llm-sovereign-ai-wizard) | Determinismo Asistido por LLM (Sovereign AI Wizard) |
| [**ADR-0052**](./ADR.md#adr-0052-quantops-daemonized-pipelines-cron-cicd-autnomo) | QuantOps Daemonized Pipelines (Cron CI/CD Autónomo) |
| [**ADR-0053**](./ADR.md#adr-0053-envoltorio-de-despliegue-y-objetivos-smart) | Envoltorio de Despliegue y Objetivos SMART |
| [**ADR-0054**](./ADR.md#adr-0054-encadenamiento-de-proyectos-y-conectores-externos) | Encadenamiento de Proyectos y Conectores Externos |
| [**ADR-0055**](./ADR.md#adr-0055-separacin-databank-rd-vs-produccin-semillas-vs-ast) | Separación Databank R&D vs Producción (Semillas vs AST) |
| [**ADR-0056**](./ADR.md#adr-0056-portfolio-data-preparation-hmm-matriz-pearson) | Portfolio Data Preparation (HMM & Matriz Pearson) |
| [**ADR-0057**](./ADR.md#adr-0057-glass-box-ai-translator-semantic-explainer-y-ast) | Glass-Box AI Translator (Semantic Explainer y AST) |
| [**ADR-0058**](./ADR.md#adr-0058-poltica-de-scoring-ponderado-de-robustez-y-veredicto-en-lenguaje-natural) | Política de Scoring Ponderado de Robustez y Veredicto en Lenguaje Natural |
| [**ADR-0059**](./ADR.md#adr-0059-continuous-rolling-walk-forward-matrix-matriz-microrodante-nocturna) | Continuous Rolling Walk-Forward Matrix (Matriz Microrodante Nocturna) |
| [**ADR-0060**](./ADR.md#adr-0060-tests-incrementales-versionados-herencia-delta) | Tests Incrementales Versionados (Herencia + Delta) |
| [**ADR-0061**](./ADR.md#adr-0061-motor-hpc-monte-carlo-hbrido-y-embudo-txico-de-estrs) | Motor HPC Monte Carlo Híbrido y Embudo Tóxico de Estrés |
| [**ADR-0062**](./ADR.md#adr-0062-motor-de-robustez-decagonal-y-fsica-de-broker-friccin-realista) | Motor de Robustez Decagonal y Física de Broker (Fricción Realista) |
| [**ADR-0063**](./ADR.md#adr-0063-protocolo-cpcv-y-validacin-pbo-lopez-de-prado-standard) | Protocolo CPCV y Validación PBO (Lopez de Prado Standard) |
| [**ADR-0064**](./ADR.md#adr-0064-preservacin-de-memoria-estadstica-via-diferenciacin-fraccional) | Preservación de Memoria Estadística via Diferenciación Fraccional |
| [**ADR-0065**](./ADR.md#adr-0065-protocolo-de-ablacin-de-reglas-simplificacin-estructural) | Protocolo de Ablación de Reglas (Simplificación Estructural) |
| [**ADR-0066**](./ADR.md#adr-0066-orquestacin-en-cascada-por-intensidad-de-cmputo-fail-fast-scalability) | Orquestación en Cascada por Intensidad de Cómputo (Fail-Fast Scalability) |
| [**ADR-0067**](./ADR.md#adr-0067-capa-de-inferencia-estadstica-ebta) | Capa de Inferencia Estadística (EBTA) |
| [**ADR-0068**](./ADR.md#adr-0068-certificacin-de-estabilizacin-de-volatilidad-target-vol) | Certificación de Estabilización de Volatilidad (Target Vol) |
| [**ADR-0069**](./ADR.md#adr-0069-modelado-de-friccin-institucional-adverse-selection) | Modelado de Fricción Institucional (Adverse Selection) |
| [**ADR-0070**](./ADR.md#adr-0070-monitoreo-de-seguridad-operativa-pardo-profile-ssl) | Monitoreo de Seguridad Operativa (Pardo Profile & SSL) |
| [**ADR-0071**](./ADR.md#adr-0071-filtrado-y-proyecciones-multidimensionales-de-optimizaciones) | Filtrado y Proyecciones Multidimensionales de Optimizaciones |
| [**ADR-0072**](./ADR.md#adr-0072-pca-toxicity-clustering) | PCA Toxicity Clustering |
| [**ADR-0073**](./ADR.md#adr-0073-adaptive-walk-forward-analysis-windows) | Adaptive Walk-Forward Analysis Windows |
| [**ADR-0074**](./ADR.md#adr-0074-autoencoder-outlier-detector) | Autoencoder Outlier Detector |
| [**ADR-0075**](./ADR.md#adr-0075-dynamic-portfolio-optimization-walk-forward-rebalancing) | Dynamic Portfolio Optimization & Walk-Forward Rebalancing |
| [**ADR-0076**](./ADR.md#adr-0076-direct-promotion-visual-validation-of-portfolios) | Direct Promotion & Visual Validation of Portfolios |
| [**ADR-0077**](./ADR.md#adr-0077-portfolio-risk-metrics-git-like-portfolio-versioning-with-clusters) | Portfolio Risk Metrics & Git-Like Portfolio Versioning with Clusters |
| [**ADR-0078**](./ADR.md#adr-0078-autopilot-execution-multiplatform-infrastructure) | Autopilot Execution & Multiplatform Infrastructure |
| [**ADR-0079**](./ADR.md#adr-0079-rules-wrappers-for-portfolios-universal-rules-injection-challenge-mode) | Rules Wrappers for Portfolios & Universal Rules Injection (Challenge Mode) |
| [**ADR-0080**](./ADR.md#adr-0080-order-priority-queue-anti-throttling) | Order-Priority Queue (Anti-Throttling) |
| [**ADR-0081**](./ADR.md#adr-0081-advanced-trade-management-atm) | Advanced Trade Management (ATM) |
| [**ADR-0082**](./ADR.md#adr-0082-micro-gestin-cintica-institucional) | Micro-Gestión Cinética Institucional |
| [**ADR-0083**](./ADR.md#adr-0083-autopilot-dynamic-metrics-engine) | Autopilot Dynamic Metrics Engine |
| [**ADR-0084**](./ADR.md#adr-0084-daemons-persistentes-y-aislamiento-de-ncleo-core-pinning) | Daemons Persistentes y Aislamiento de Núcleo (Core Pinning) |
| [**ADR-0085**](./ADR.md#adr-0085-bus-de-datos-pubsub-zero-copy-multiplexacin) | Bus de Datos Pub/Sub Zero-Copy (Multiplexación) |
| [**ADR-0086**](./ADR.md#adr-0086-minera-descentralizada-de-estrategias-la-colmena) | Minería Descentralizada de Estrategias (La Colmena) |
| [**ADR-0087**](./ADR.md#adr-0087-el-guardin-global-execution-router-el-centinela-rust-shadow-watchdog-kill-switch) | El Guardián (Global Execution Router) & El Centinela (Rust Shadow Watchdog & Kill Switch) |
| [**ADR-0088**](./ADR.md#adr-0088-protocolo-de-incubacin-cono-de-silencio-sandbox-de-7-das-proyeccin-de-monte-carlo-y-broken-strategy-flag) | Protocolo de Incubación & Cono de Silencio (Sandbox de 7 Días, Proyección de Monte Carlo y Broken Strategy Flag) |
| [**ADR-0089**](./ADR.md#adr-0089-motores-de-optimizacin-de-portfolio-clsicos-ensamblador-singular-d-score-con-hedging-cointegrativo-router-de-liquidez-y-daemon-de-rebalanceo) | Motores de Optimización de Portfolio Clásicos & Ensamblador Singular D-Score con Hedging Cointegrativo, Router de Liquidez y Daemon de Rebalanceo |
| [**ADR-0090**](./ADR.md#adr-0090-arquitectura-de-portafolios-federados-federated-portfolio-clusters) | Arquitectura de Portafolios Federados (Federated Portfolio Clusters) |
| [**ADR-0091**](./ADR.md#adr-0091-simulacin-de-portafolio-real-real-portfolio-backtesting) | Simulación de Portafolio Real (Real Portfolio Backtesting) |
| [**ADR-0092**](./ADR.md#adr-0092-copy-trading-mediante-rel-ciego-de-seales-e2e) | Copy-Trading mediante Relé Ciego de Señales (E2E) |
| [**ADR-0093**](./ADR.md#adr-0093-arquitectura-de-seguridad-soberana-sovereign-security-architecture) | Arquitectura de Seguridad Soberana (Sovereign Security Architecture) |
| [**ADR-0094**](./ADR.md#adr-0094-delegacin-hbrida-de-cmputo-cooperative-hybrid-compute) | Delegación Híbrida de Cómputo (Cooperative Hybrid Compute) |
| [**ADR-0095**](./ADR.md#adr-0095-veto-operativo-por-degradacin-de-robustez-de-slippage-y-umbrales-monte-carlo) | Veto Operativo por Degradación de Robustez de Slippage y Umbrales Monte Carlo |
| [**ADR-0096**](./ADR.md#adr-0096-cach-de-previews-locales-de-nodo-para-iteracin-rpida) | Caché de Previews Locales de Nodo para Iteración Rápida |
| [**ADR-0097**](./ADR.md#adr-0097-renderizado-grfico-multidimensional-nativo-sin-webviews) | Renderizado Gráfico Multidimensional Nativo sin WebViews |
| [**ADR-0098**](./ADR.md#adr-0098-gobernanza-de-purgas-y-snapshots-de-databank) | Gobernanza de Purgas y Snapshots de Databank |
| [**ADR-0099**](./ADR.md#adr-0099-marketplace-de-cajas-negras-con-zero-knowledge-nodes) | Marketplace de "Cajas Negras" con Zero-Knowledge Nodes |
| [**ADR-0100**](./ADR.md#adr-0100-relegacin-de-microestructura-l3-a-saas-institucional-y-proxies-client-zero) | Relegación de Microestructura L3 a SaaS Institucional y Proxies Client Zero |
| [**ADR-0101**](./ADR.md#adr-0101-transpilacin-basada-en-plantillas-tera-para-modelos-ast) | Transpilación Basada en Plantillas Tera para Modelos AST |
| [**ADR-0102**](./ADR.md#adr-0102-anonimizacin-criptogrfica-local-first-en-collective-intelligence) | Anonimización Criptográfica local-first en Collective Intelligence |
| [**ADR-0103**](./ADR.md#adr-0103-filosofa-dual-y-sandboxing-en-el-sistema-de-plugins-institucionales) | Filosofía Dual y Sandboxing en el Sistema de Plugins Institucionales |
| [**ADR-0104**](./ADR.md#adr-0104-traduccin-de-caractersticas-y-pila-del-roadmap-acelerado-a-rustflutter-core) | Traducción de Características y Pila del Roadmap Acelerado a Rust/Flutter Core |
| [**ADR-0105**](./ADR.md#adr-0105-estrategia-de-datos-100-polars-nativo-en-rust) | Estrategia de Datos (100% Polars Nativo en Rust) |
| [**ADR-0106**](./ADR.md#adr-0106-paradigma-de-interfaz-de-usuario-y-dashboards-visuales-de-alta-precisin) | Paradigma de Interfaz de Usuario y Dashboards Visuales de Alta Precisión |
| [**ADR-0107**](./ADR.md#adr-0107-integracin-nativa-con-nautilustrader-v2-crates-rust-sin-python-sin-fork) | Integración Nativa con NautilusTrader v2 (Crates Rust, Sin Python, Sin Fork) |
| [**ADR-0108**](./ADR.md#adr-0108-arquitectura-de-genomas-modulares-por-dominio-generalizacin-del-patrn-de-genes-condicinaccin) | Arquitectura de Genomas Modulares por Dominio (Generalización del Patrón de Genes Condición→Acción) |
| [**ADR-0109**](./ADR.md#adr-0109-generador-genmico-de-riesgo-y-gestin-de-posicin-fase-a--wildcard-invertido-y-rplica-de-estado-de-riesgo-en-monte-carlo) | Generador Genómico de Riesgo y Gestión de Posición (Fase A) — Wildcard Invertido y Réplica de Estado de Riesgo en Monte Carlo |
| [**ADR-0110**](./ADR.md#adr-0110-generador-genmico-de-rgimen-y-filtro-de-entorno-fase-b--mscaras-de-permisoprohibicin-por-estructura-de-mercado) | Generador Genómico de Régimen y Filtro de Entorno (Fase B) — Máscaras de Permiso/Prohibición por Estructura de Mercado |
| [**ADR-0111**](./ADR.md#adr-0111-generador-genmico-de-portafolio-y-correlacin-fase-c--co-evolucin-de-cartera-y-monte-carlo-de-desfase-temporal) | Generador Genómico de Portafolio y Correlación (Fase C) — Co-evolución de Cartera y Monte Carlo de Desfase Temporal |
| [**ADR-0112**](./ADR.md#adr-0112-veredicto-spike-002--erradicacion-de-tch-rslibtorch-escalera-de-computo-numerico-soberano-ndarrayrayon-candle-burn) | Veredicto SPIKE-002 — Erradicación de `tch-rs`/libtorch; Escalera de Cómputo Numérico Soberano (`ndarray`/Rayon → `candle` → `burn`) |
| [**ADR-0113**](./ADR.md#adr-0113-veredicto-spike-003--erradicacion-de-pysr-regresion-simbolica-como-modo-del-motor-genetico-nativo-y-diferimiento-de-la-mineria-simbolica-libre-a-moonshot-egg) | Veredicto SPIKE-003 — Erradicación de PySR; Regresión Simbólica como Modo del Motor Genético Nativo y Diferimiento de la Minería Simbólica Libre a Moonshot (`egg`) |
| [**ADR-0114**](./ADR.md#adr-0114-veredicto-spike-004--motor-de-backtest-dual-con-ruta-express-hibrida-vectorizada-secuencial-modo-de-motor-elegido-por-el-usuario-y-contrato-de-consistencia-conservadora) | Veredicto SPIKE-004 — Motor de Backtest Dual con Ruta Express Híbrida (Vectorizada + Secuencial), Modo de Motor Elegido por el Usuario y Contrato de Consistencia Conservadora |
| [**ADR-0115**](./ADR.md#adr-0115-veredicto-spike-005--verdict-engine-determinista-sin-llm-erradicacion-de-ollama-como-requisito) | Veredicto SPIKE-005 — Verdict Engine Determinista sin LLM; Erradicación de Ollama como Requisito |
| [**ADR-0116**](./ADR.md#adr-0116-veredicto-spike-006--downsampling-obligatorio-en-backend-como-condicion-de-la-frontera-ffi-zerocopybuffer-solo-para-cargas-masivas) | Veredicto SPIKE-006 — Downsampling Obligatorio en Backend como Condición de la Frontera FFI; `ZeroCopyBuffer` solo para Cargas Masivas |

---
## 📖 Arquitectura y Gobernanza

- [**SAD.md**](./SAD.md) — Documento de Arquitectura de Software (Visión General).
- [**ADR.md**](./ADR.md) — Decisiones Arquitectónicas (Registro de decisiones inmutables).
- [**TEMPLATES.md**](./TEMPLATES.md) — Plantillas para nuevas especificaciones.
