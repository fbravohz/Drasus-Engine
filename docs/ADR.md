# Registro de Decisiones de Arquitectura (ADR) — Índice

Cada ADR vive en su propio archivo bajo [`adr/`](./adr/). El contenido es **idéntico** al monolito original; se partió para lectura por demanda (ver `CLAUDE.md` §3). Para leer un ADR concreto, abre su archivo; no hace falta cargar todos.

| ADR | Decisión / Título |
|-----|-------------------|
| [ADR-0001](./adr/ADR-0001.md) | Monolito Modular + FCIS |
| [ADR-0002](./adr/ADR-0002.md) | Desacoplamiento de Persistencia |
| [ADR-0003](./adr/ADR-0003.md) | Organización de Módulos (FCIS) + Features Reutilizables |
| [ADR-0004](./adr/ADR-0004.md) | Máquina de Estados (FSM) |
| [ADR-0005](./adr/ADR-0005.md) | Strategy-Portfolio Git-Like Versioning con DAG |
| [ADR-0006](./adr/ADR-0006.md) | Migraciones Centralizadas con SQLx Migrator |
| [ADR-0007](./adr/ADR-0007.md) | Inyección Dinámica de Comportamiento (Feature Router) |
| [ADR-0008](./adr/ADR-0008.md) | Configurabilidad Universal (TODO es Parámetro, Excepto Invariantes) |
| [ADR-0009](./adr/ADR-0009.md) | Interfaz Unificada Strategy-Portfolio (ExecutableContainer) |
| [ADR-0010](./adr/ADR-0010.md) | Reglas Dinámicas (Hard Limits vs Soft Alerts) |
| [ADR-0011](./adr/ADR-0011.md) | Operaciones Asincrónicas (Async Job Pattern) |
| [ADR-0012](./adr/ADR-0012.md) | Arquitectura Multi-Pipeline Paralela (Single Machine Architecture) |
| [ADR-0013](./adr/ADR-0013.md) | Selección de Stack Tecnológico (High-Performance Core) |
| [ADR-0014](./adr/ADR-0014.md) | Evolución Incremental de Contratos |
| [ADR-0015](./adr/ADR-0015.md) | Arquitectura de Causalidad y Aprendizaje Cerrado |
| [ADR-0016](./adr/ADR-0016.md) | Local-First Processing & External Overlays |
| [ADR-0017](./adr/ADR-0017.md) | Simulación de Alta Fidelidad Institutional |
| [ADR-0018](./adr/ADR-0018.md) | Taxonomía y Topología del Pipeline (Los 8 Módulos) |
| [ADR-0019](./adr/ADR-0019.md) | ~~Interoperabilidad Frontend-Backend (FFI/gRPC)~~ ⚠️ Superado por ADR-0029 |
| [ADR-0020 V2](./adr/ADR-0020.md) | Principio de Inundación de Fundaciones V2 (Foundation Inundation) |
| [ADR-0021](./adr/ADR-0021.md) | ~~Modelo de Decisión Dual (Autopilot con Veto)~~ ⚠️ Superado por ADR-0010 |
| [ADR-0022](./adr/ADR-0022.md) | Pipeline No-Lineal (DAG Multiflujal) |
| [ADR-0023](./adr/ADR-0023.md) | Dashboard Dinámico vs Arquitectura de Plugins |
| [ADR-0024](./adr/ADR-0024.md) | Reglas Dominantes (Extracted Constraints) |
| [ADR-0025](./adr/ADR-0025.md) | Pre-Trade Risk 10-Steps Gate |
| [ADR-0026](./adr/ADR-0026.md) | Shadow Watchdog & Heartbeat |
| [ADR-0027](./adr/ADR-0027.md) | Event Sourcing & Inventory Reconstruction |
| [ADR-0028](./adr/ADR-0028.md) | ZUI Fractal Navigation (Orchestrator/Strategy Inspector) |
| [ADR-0029](./adr/ADR-0029.md) | Patrón Todo en Uno (Rust + Flutter FFI) |
| [ADR-0030](./adr/ADR-0030.md) | Persistencia Soberana "Zero-Docker" |
| [ADR-0031](./adr/ADR-0031.md) | Inteligencia Artificial Híbrida (Hybrid Genesis Engine) |
| [ADR-0032](./adr/ADR-0032.md) | Estándares de Hardware Soberano (Single Machine Sovereignty) |
| [ADR-0033](./adr/ADR-0033.md) | Arquitectura de Despliegue Trimodal |
| [ADR-0034](./adr/ADR-0034.md) | Ingesta Híbrida Soberana (Bulk S3 + API Delta) |
| [ADR-0035](./adr/ADR-0035.md) | Persistencia en Particionado Hive-Style (Parquet) |
| [ADR-0036](./adr/ADR-0036.md) | Remuestreo Dinámico Multidimensional (DuckDB) |
| [ADR-0037](./adr/ADR-0037.md) | Protocolo de Calidad "The Sanitizer" |
| [ADR-0038](./adr/ADR-0038.md) | Estándar de Nomenclatura Institucional (Sanitización Terminológica) |
| [ADR-0039](./adr/ADR-0039.md) | Infraestructura de Lógica Causal Híbrida (Legacy SQX + Sovereign QF) |
| [ADR-0040](./adr/ADR-0040.md) | Disparadores de Señal Metamórficos (Capital-Aware) |
| [ADR-0041](./adr/ADR-0041.md) | Arquitectura de Hemisferios de Asimetría Estructural |
| [ADR-0042](./adr/ADR-0042.md) | Arquitectura de Fitness Metamórfico de Estado |
| [ADR-0043](./adr/ADR-0043.md) | Protocolo de Programación Evolutiva Parcial (WildCards) |
| [ADR-0044](./adr/ADR-0044.md) | Framework de Dimensionamiento de Riesgo Multimodal |
| [ADR-0045](./adr/ADR-0045.md) | Prop-Firm Compliance Profile (Ley de Cero Hardcoding) |
| [ADR-0046](./adr/ADR-0046.md) | Vector-Time Pruning (Poda Temporal Autónoma) |
| [ADR-0047](./adr/ADR-0047.md) | Computación Asimétrica de Métricas (Hot-Path vs R&D) |
| [ADR-0048](./adr/ADR-0048.md) | Neutralización Analítica de Beta (Alpha Decoupling) |
| [ADR-0049](./adr/ADR-0049.md) | Validación Transversal de Robustez (Cross-Market Validation) |
| [ADR-0050](./adr/ADR-0050.md) | Búsqueda Generativa Diversificada (Fit-to-Portfolio Search) |
| [ADR-0051](./adr/ADR-0051.md) | Determinismo Asistido por LLM (Sovereign AI Wizard) |
| [ADR-0052](./adr/ADR-0052.md) | QuantOps Daemonized Pipelines (Cron CI/CD Autónomo) |
| [ADR-0053](./adr/ADR-0053.md) | Envoltorio de Despliegue y Objetivos SMART |
| [ADR-0054](./adr/ADR-0054.md) | Encadenamiento de Proyectos y Conectores Externos |
| [ADR-0055](./adr/ADR-0055.md) | Separación Databank R&D vs Producción (Semillas vs AST) |
| [ADR-0056](./adr/ADR-0056.md) | Portfolio Data Preparation (HMM & Matriz Pearson) |
| [ADR-0057](./adr/ADR-0057.md) | Glass-Box AI Translator (Semantic Explainer y AST) |
| [ADR-0058](./adr/ADR-0058.md) | Política de Scoring Ponderado de Robustez y Veredicto en Lenguaje Natural |
| [ADR-0059](./adr/ADR-0059.md) | Continuous Rolling Walk-Forward Matrix (Matriz Microrodante Nocturna) |
| [ADR-0060](./adr/ADR-0060.md) | Tests Incrementales Versionados (Herencia + Delta) |
| [ADR-0061](./adr/ADR-0061.md) | Motor HPC Monte Carlo Híbrido y Embudo Tóxico de Estrés |
| [ADR-0062](./adr/ADR-0062.md) | Motor de Robustez Decagonal y Física de Broker (Fricción Realista) |
| [ADR-0063](./adr/ADR-0063.md) | Protocolo CPCV y Validación PBO (Lopez de Prado Standard) |
| [ADR-0064](./adr/ADR-0064.md) | Preservación de Memoria Estadística via Diferenciación Fraccional |
| [ADR-0065](./adr/ADR-0065.md) | Protocolo de Ablación de Reglas (Simplificación Estructural) |
| [ADR-0066](./adr/ADR-0066.md) | Orquestación en Cascada por Intensidad de Cómputo (Fail-Fast Scalability) |
| [ADR-0067](./adr/ADR-0067.md) | Capa de Inferencia Estadística (EBTA) |
| [ADR-0068](./adr/ADR-0068.md) | Certificación de Estabilización de Volatilidad (Target Vol) |
| [ADR-0069](./adr/ADR-0069.md) | Modelado de Fricción Institucional (Adverse Selection) |
| [ADR-0070](./adr/ADR-0070.md) | Monitoreo de Seguridad Operativa (Pardo Profile & SSL) |
| [ADR-0071](./adr/ADR-0071.md) | Filtrado y Proyecciones Multidimensionales de Optimizaciones |
| [ADR-0072](./adr/ADR-0072.md) | PCA Toxicity Clustering |
| [ADR-0073](./adr/ADR-0073.md) | Adaptive Walk-Forward Analysis Windows |
| [ADR-0074](./adr/ADR-0074.md) | Autoencoder Outlier Detector |
| [ADR-0075](./adr/ADR-0075.md) | Dynamic Portfolio Optimization & Walk-Forward Rebalancing |
| [ADR-0076](./adr/ADR-0076.md) | Direct Promotion & Visual Validation of Portfolios |
| [ADR-0077](./adr/ADR-0077.md) | Portfolio Risk Metrics & Git-Like Portfolio Versioning with Clusters |
| [ADR-0078](./adr/ADR-0078.md) | Autopilot Execution & Multiplatform Infrastructure |
| [ADR-0079](./adr/ADR-0079.md) | Rules Wrappers for Portfolios & Universal Rules Injection (Challenge Mode) |
| [ADR-0080](./adr/ADR-0080.md) | Order-Priority Queue (Anti-Throttling) |
| [ADR-0081](./adr/ADR-0081.md) | Advanced Trade Management (ATM) |
| [ADR-0082](./adr/ADR-0082.md) | Micro-Gestión Cinética Institucional |
| [ADR-0083](./adr/ADR-0083.md) | Autopilot Dynamic Metrics Engine |
| [ADR-0084](./adr/ADR-0084.md) | Daemons Persistentes y Aislamiento de Núcleo (Core Pinning) |
| [ADR-0085](./adr/ADR-0085.md) | Bus de Datos Pub/Sub Zero-Copy (Multiplexación) |
| [ADR-0086](./adr/ADR-0086.md) | Minería Descentralizada de Estrategias (La Colmena) |
| [ADR-0087](./adr/ADR-0087.md) | El Guardián (Global Execution Router) & El Centinela (Rust Shadow Watchdog & Kill Switch) |
| [ADR-0088](./adr/ADR-0088.md) | Protocolo de Incubación & Cono de Silencio (Sandbox de 7 Días, Proyección de Monte Carlo y Broken Strategy Flag) |
| [ADR-0089](./adr/ADR-0089.md) | Motores de Optimización de Portfolio Clásicos & Ensamblador Singular D-Score con Hedging Cointegrativo, Router de Liquidez y Daemon de Rebalanceo |
| [ADR-0090](./adr/ADR-0090.md) | Arquitectura de Portafolios Federados (Federated Portfolio Clusters) |
| [ADR-0091](./adr/ADR-0091.md) | Simulación de Portafolio Real (Real Portfolio Backtesting) |
| [ADR-0092](./adr/ADR-0092.md) | Copy-Trading mediante Relé Ciego de Señales (E2E) |
| [ADR-0093](./adr/ADR-0093.md) | Arquitectura de Seguridad Soberana (Sovereign Security Architecture) |
| [ADR-0094](./adr/ADR-0094.md) | Delegación Híbrida de Cómputo (Cooperative Hybrid Compute) |
| [ADR-0095](./adr/ADR-0095.md) | Veto Operativo por Degradación de Robustez de Slippage y Umbrales Monte Carlo |
| [ADR-0096](./adr/ADR-0096.md) | Caché de Previews Locales de Nodo para Iteración Rápida |
| [ADR-0097](./adr/ADR-0097.md) | Renderizado Gráfico Multidimensional Nativo sin WebViews |
| [ADR-0098](./adr/ADR-0098.md) | Gobernanza de Purgas y Snapshots de Databank |
| [ADR-0099](./adr/ADR-0099.md) | Marketplace de "Cajas Negras" con Zero-Knowledge Nodes |
| [ADR-0100](./adr/ADR-0100.md) | Relegación de Microestructura L3 a SaaS Institucional y Proxies Client Zero |
| [ADR-0101](./adr/ADR-0101.md) | Transpilación Basada en Plantillas Tera para Modelos AST |
| [ADR-0102](./adr/ADR-0102.md) | Anonimización Criptográfica local-first en Collective Intelligence |
| [ADR-0103](./adr/ADR-0103.md) | Filosofía Dual y Sandboxing en el Sistema de Plugins Institucionales |
| [ADR-0104](./adr/ADR-0104.md) | Traducción de Características y Pila del Roadmap Acelerado a Rust/Flutter Core |
| [ADR-0105](./adr/ADR-0105.md) | Estrategia de Datos (100% Polars Nativo en Rust) |
| [ADR-0106](./adr/ADR-0106.md) | Paradigma de Interfaz de Usuario y Dashboards Visuales de Alta Precisión |
| [ADR-0107](./adr/ADR-0107.md) | Integración Nativa con NautilusTrader v2 (Crates Rust, Sin Python, Sin Fork) |
| [ADR-0108](./adr/ADR-0108.md) | Arquitectura de Genomas Modulares por Dominio (Generalización del Patrón de Genes Condición→Acción) |
| [ADR-0109](./adr/ADR-0109.md) | Generador Genómico de Riesgo y Gestión de Posición (Fase A) — Wildcard Invertido y Réplica de Estado de Riesgo en Monte Carlo |
| [ADR-0110](./adr/ADR-0110.md) | Generador Genómico de Régimen y Filtro de Entorno (Fase B) — Máscaras de Permiso/Prohibición por Estructura de Mercado |
| [ADR-0111](./adr/ADR-0111.md) | Generador Genómico de Portafolio y Correlación (Fase C) — Co-evolución de Cartera y Monte Carlo de Desfase Temporal |
| [ADR-0112](./adr/ADR-0112.md) | Veredicto SPIKE-002 — Erradicación de `tch-rs`/libtorch; Escalera de Cómputo Numérico Soberano (`ndarray`/Rayon → `candle` → `burn`) |
| [ADR-0113](./adr/ADR-0113.md) | Veredicto SPIKE-003 — Erradicación de PySR; Regresión Simbólica como Modo del Motor Genético Nativo y Diferimiento de la Minería Simbólica Libre a Moonshot (`egg`) |
| [ADR-0114](./adr/ADR-0114.md) | Veredicto SPIKE-004 — Motor de Backtest Dual con Ruta Express Híbrida (Vectorizada + Secuencial), Modo de Motor Elegido por el Usuario y Contrato de Consistencia Conservadora |
| [ADR-0115](./adr/ADR-0115.md) | Veredicto SPIKE-005 — Verdict Engine Determinista sin LLM; Erradicación de Ollama como Requisito |
| [ADR-0116](./adr/ADR-0116.md) | Veredicto SPIKE-006 — Downsampling Obligatorio en Backend como Condición de la Frontera FFI; `ZeroCopyBuffer` solo para Cargas Masivas |
| [ADR-0117](./adr/ADR-0117.md) | Entrega Progresiva de Cáscara Delgada por Feature — Techo Fijo, Ventana de Verificación y Redefinición de EPIC-8 como Unificación ZUI |
| [ADR-0118](./adr/ADR-0118.md) | Unidad de Entrega = Módulo Completo; Construcción en el Primer Consumidor; ROADMAP como Guía sin Bitácora |
| [ADR-0119](./adr/ADR-0119.md) | Separación Plano de Control / Plano de Ejecución para Operación Distribuida (Edge Execution / Central Control) |
| [ADR-0120](./adr/ADR-0120.md) | Modos de Acompañamiento de Implementación (Autónomo / Mentor / Revisión) — Selección por el Usuario, Persistida en la Orden de Trabajo |
| [ADR-0121](./adr/ADR-0121.md) | Comentarios de Código en Español — Identificadores en Inglés se Mantienen |
| [ADR-0122](./adr/ADR-0122.md) | Cuarto Modo de Acompañamiento "Docente" + Protocolo de Lecciones Compartido (`docs/lessons/`) |
| [ADR-0123](./adr/ADR-0123.md) | Cabina Dual — Acceso Agéntico vía MCP con Permisos Graduados por Riesgo de Pipeline |
| [ADR-0124](./adr/ADR-0124.md) | Lecciones Organizadas por Story/Task, no por Tema — Corrige esa Regla de ADR-0122 |
| [ADR-0125](./adr/ADR-0125.md) | Frontera Determinista de Datos Fundamentales — Event Study + Surprise como Métodos Canónicos; Extracción NLP Fuera del Núcleo |
| [ADR-0126](./adr/ADR-0126.md) | Sourcing y Soberanía de Datos Fundamentales — Hecho Crudo Externo, Scoring 100% Propio |
| [ADR-0127](./adr/ADR-0127.md) | Point-In-Time de Eventos Fundamentales — Instante de Publicación + Versionado Vintage/As-Of (First-Print vs Revisiones) |
| [ADR-0128](./adr/ADR-0128.md) | Relevancia Evento→Activo y Normalización por Instrumento — Mapa de Exposición Determinista |
| [ADR-0129](./adr/ADR-0129.md) | Entradas Concurrentes No Bloqueantes por Defecto + De-duplicación de Señal |
| [ADR-0130](./adr/ADR-0130.md) | Frecuencia/Horizonte de Operación como Objetivo de Generación + Agnosticismo de Temporalidad |
| [ADR-0131](./adr/ADR-0131.md) | Flutter como Framework de Frontend — Rechazo Razonado de Qt, iced, slint y egui |
| [ADR-0132](./adr/ADR-0132.md) | Rust como Lenguaje del Núcleo — Rechazo Razonado de C++/Qt Total |
| [ADR-0133](./adr/ADR-0133.md) | Pirámide de Pruebas Canónica + Fuzzing Obligatorio en Fronteras Externas |
| [ADR-0134](./adr/ADR-0134.md) | Matriz de Plataformas de Despliegue (Desktop Nativo Windows/Linux/macOS + Mobile/Web Cliente Delgado) y Detección de Muerte del Padre Portátil |
| [ADR-0135](./adr/ADR-0135.md) | Sección `## Cáscara Visual` Obligatoria en Features con Superficie — Skill UI-Designer como Etapa 0.5 del Tech Lead |
| [ADR-0136](./adr/ADR-0136.md) | Arquitectura Visual Unificada — Dashboard + Canvas [Forge/Reactor — TBD] · supersede ADR-0028 (ZUI 3 niveles nominados) |
| [ADR-0137](./adr/ADR-0137.md) | Feature como Unidad Hexagonal Autónoma con Puertos Tipados — Módulos como Composiciones Preset · enmienda ADR-0002 + ADR-0118 · enmienda 2026-06-23: infra crosscutting vive en `crates/shared` (excepción acotada) |
