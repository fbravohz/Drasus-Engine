# Generar

**Carpeta:** `./modules/generate/`
**Estado:** Orquestador (Imperative Shell)
**Última actualización:** 2026-06-11

---

## ¿Qué es?

El módulo de generación es el **Arquitecto de Candidatas** y el núcleo del **Hybrid Genesis Engine**. Su función es orquestar los motores evolutivos (GA/NSGA-II), simbólicos (regresión simbólica nativa sobre el AST, ADR-0113 — no PySR) y, en fase moonshot, de aprendizaje por refuerzo (DRL) para producir una población de estrategias diversas mediante el descubrimiento autónomo (No-Template).

Coordina la sinergia donde el **DRL Genesis** propone la "Tesis de Alpha" y el **NSGA-II** realiza el micro-ajuste funcional, entregando candidatos validados por el **Compilador AST** listos para validación institucional.

También es posible que el usuario cree una estrategia manualmente y la inyecte aquí para que pase al proceso de validación sin pasar por la generación automática.

---

## Épica 0: Esqueleto Fundacional

### Estructura de Archivos (FCIS — ADR-0003)

```
crates/generate/
├── public_interface.rs   # Frontera pública: único punto de entrada para otros módulos
├── logic.rs              # Lógica pura: fitness, selección Pareto, mutación (sin DB, sin I/O)
├── orchestrator.rs       # Coordinación: invoca NSGA-II, AST Compiler, Databank; maneja errores
├── persistence.rs        # Acceso a SQLite WAL y Parquet (lectura/escritura)
├── schemas.rs            # Definición de tablas: candidates, populations, generation_jobs
└── types.rs              # Tipos de entrada/salida: CandidateGenome, ParetoFront, GenerationConfig
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
| | `parent_id` | Puntero al registro padre (linaje genético) |
| | `logic_hash` | Hash del motor evolutivo (NSGA/Simbólico nativo) |
| | `data_snapshot_id` | Snapshot PIT del dataset de entrenamiento |
| | `transformation_id` | ID del paso/tipo de transformación aplicado |
| **IV. Infraestructura y Ops** | `process_id` | PID del worker de generación |
| | `session_id` | Agrupación de runtime |
| | `node_id` | ID del hardware físico (CPU/GPU) |
| **V. Forense y Ejecución** | `portfolio_container_id` | Contenedor de portafolio |
| | `compliance_status_id` | Veredicto de riesgo |
| | `risk_audit_id` | Ticket detallado de riesgo |
| | `indicator_state_hash` | Snapshot de fitness consolidado |
| | `execution_latency_ms` | Latencia de evaluación |
| | `source_signal_id` | Link a señal origen |
| | `signature_hash` | HMAC de señales |

### TTRs Etiquetados por Fase

| TTR | Fase | Descripción corta |
|---|---|---|
| TTR-008 | **EPIC-2** | Compilación de lógica procedural (AST Compiler) |
| TTR-024 | **EPIC-2** | Contrato de diseño (Design Manifest) |
| TTR-001 | **EPIC-3** | Evolución híbrida (NSGA-II + DRL Genesis) |
| TTR-002 | **EPIC-3** | Minería SQL (Analytic Discovery) |
| TTR-009 | **EPIC-3** | Versionado de linaje (Strategy Versioning) |
| TTR-016 | **EPIC-3** | Resolución de comodines (WildCards) |
| TTR-019 | **EPIC-3** | Persistencia R&D (Databank Lake) |
| TTR-022 | **EPIC-3** | Evaluación evolutiva (Backtest Engine) |
| TTR-023 | **EPIC-3** | Lotaje evolutivo (Precision Sizing) |
| TTR-025 | **EPIC-3** | Diseño manual (Visual DAG Editor) |
| TTR-026 | **EPIC-3** | Ajuste fino (Parameter Optimization) |
| TTR-027 | **EPIC-3** | Desacople (Async Job Executor) |
| TTR-028 | **EPIC-3** | Indicadores dinámicos (Adaptive Volume) |
| TTR-030 | **EPIC-3** | Auditoría forense R&D (Audit Log) |
| TTR-035 | **EPIC-3** | Registro de intentos N para DSR |
| TTR-040 | **EPIC-3** | Registro de dominios genómicos (ADR-0108) |
| TTR-003 | EPIC-4 | Detección de régimen awareness (HMM) |
| TTR-004 | EPIC-4 | Test de canasta (Universal Basket) |
| TTR-005 | EPIC-4 | Optimización bayesiana (Intelligent Fine-Tuning) |
| TTR-010 | EPIC-9+ | Deep Learning (Model-Free Alpha) |
| TTR-011 | **EPIC-3** | Enrutador de señales (Feature Router) |
| TTR-012 | **EPIC-3** | Hemisferios de asimetría (Strategy Ensemble) |
| TTR-014 | EPIC-9+ | Dimensionalidad AI |
| TTR-015 | **EPIC-3** | Fitness metamórfico (NSGA-II Tuning) |
| TTR-017 | EPIC-6 | Presión de descorrelación (Fit-to-Portfolio) |
| TTR-018 | EPIC-8 | Diseño guiado (AST Copilot) |
| TTR-020 | EPIC-9+ | Simbolismo (Symbolic Discovery nativo) |
| TTR-021 | **EPIC-3** | Ortogonalidad (Zero-Crossing Filter) |
| TTR-029 | **EPIC-3** | Aislamiento genético (Worker Isolation) |
| TTR-031 | **EPIC-3** | Builder Telemetry (Monitoreo Operativo) |
| TTR-032 | EPIC-9+ | Recolección de Alpha (Alpha Harvesting Gateway) |
| TTR-033 | EPIC-9+ | Traducción AI (Glass-Box AI Translator) |
| TTR-034 | EPIC-6 | Fundaciones de portafolio (Portfolio Data Preparation) |
| TTR-036 | EPIC-9+ | Minería descentralizada (La Colmena) |
| TTR-037 | EPIC-8 | Vistas previas rápidas (Node Preview) |
| TTR-038 | EPIC-7 | Disparadores reactivos (Event-Driven Pipeline Triggers) |
| TTR-039 | EPIC-8 | Inspección de superficie de parámetros (Plateau Co-Pilot) |
| TTR-999 | **EPIC-3** | Protocolo Fail-Fast Safe (ADR-0066) |

---

## Comportamientos Observables (Orquestación)

- [ ] **Flujo Evolutivo:** Inicia la búsqueda llamando a [nsga2-optimizer](../features/nsga2-optimizer.md).
- [ ] **Descubrimiento Simbólico:** (Opcional) Invoca a [pysr-signal-discovery](../moonshots/pysr-signal-discovery.md) para refinar ecuaciones de señal.
- [ ] **Marcado de Régimen:** Asegura que cada candidata generada sea evaluada en el contexto de [hmm-regime-detection](../features/hmm-regime-detection.md).
- [ ] **Inyección Manual:** Permite al usuario saltar la orquestación y subir una estrategia propia directamente.

---

## Restricciones

- NUNCA una estrategia candidata puede tener más indicadores de los permitidos (límite configurable, anti-sobreajuste)
- NUNCA el score de calidad puede ser negativo (indicaría error de cálculo)
- La evolución no puede cambiar de objetivo a mitad de proceso (configura el objetivo antes de empezar)

---

## Parámetros Configurables

| Parámetro | Default | Qué hace |
|---|---|---|
| POPULATION_SIZE | configurable | Cuántas estrategias hay en cada generación |
| GENERATIONS | configurable | Cuántas generaciones evolucionar |
| STAGNATION_LIMIT | configurable | Cuántas generaciones sin mejora antes de renovar población |
| RENEWAL_PCT | configurable | Qué porcentaje de la población se renueva al estancarse |
| MAX_INDICATORS | configurable | Máximo de indicadores por estrategia (anti-sobreajuste) |
| MAX_LOOKBACK | configurable | Período máximo de lookback para indicadores |
| LONG_SHORT_MODE | configurable | Reglas Long y Short iguales o independientes |
| FITNESS_MODE | configurable | Modo de evaluación: estático (pesos fijos) o dinámico (según fase) |
| INDICATOR_WEIGHTS | configurable | Preferencia por ciertos indicadores (sesgo de búsqueda) |

---

## Features Consumidas (Reutilizables)

- **[`nsga2-optimizer`](../features/nsga2-optimizer.md)** — Optimización multiobjetivo (Sharpe↑, DD↓, WR↑)
- **[`pysr-signal-discovery`](../moonshots/pysr-signal-discovery.md)** — Descubrimiento de ecuaciones simbólicas
- **[`hmm-regime-detection`](../features/hmm-regime-detection.md)** — Detección de régimen (regime-aware generation)
- **[`zero-crossing-filter`](../features/zero-crossing-filter.md)** — Filtrado de señales ortogonales (alpha puro)
- **[`strategy-ensemble`](../features/strategy-ensemble.md)** — Síntesis multi-canal (NSGA+Simbólico nativo+HMM)
- **[`backtest-engine`](../features/backtest-engine.md)** — Evaluación de fitness
- **[`strategy-versioning`](../features/strategy-versioning.md)** — Versionado DAG de candidatos
- **[`precision-sizing-models`](../features/precision-sizing-models.md)** — Cálculo de lotaje determinista para backtesting
- **[`design-manifest`](../features/design-manifest.md)** — Contrato formal del genoma y restricciones de la estrategia.
- **[`visual-dag-editor`](../features/visual-dag-editor.md)** — Interfaz visual para diseño y edición de genomas
- **[`parameter-optimization`](../features/parameter-optimization.md)** — Ajuste fino de parámetros (Grid/Random/Bayesian)
- **[`plateau-copilot`](../features/plateau-copilot.md)** — Mapa de calor 2D para inspección manual de la superficie de parámetros de la candidata.
- **[`async-job-executor`](../features/async-job-executor.md)** — Ejecución de jobs genéticos en background
- **[`universal-basket-backtester`](../features/universal-basket-backtester.md)** — Test de generalización multi-activo
- **[`duckdb-sql-engine`](../features/duckdb-sql-engine.md)** — Consultas SQL vectorizadas para minería de señales

- **[`deep-learning-suite`](../moonshots/deep-learning-suite.md)** — Modelos LSTM/Transformer y Agentes DRL
- **[`bayesian-optimizer`](../features/bayesian-optimizer.md)** — Optimización inteligente de parámetros
- **[`adaptive-volume-indicators`](../features/adaptive-volume-indicators.md)** — Indicadores dinámicos (ER, KAMA, VIDYA)
- **[`ast-compiler`](../features/ast-compiler.md)** — Compilación de diseño visual a genoma validado
- **[`worker-isolation-orchestrator`](../features/worker-isolation-orchestrator.md)** — Ejecución paralela con aislamiento de procesos
- **[`audit-log`](../features/audit-log.md)** — Registro de generación
- **[`ai-dimensionality-suite`](../moonshots/ai-dimensionality-suite.md)** — Reducción de varianza y selección de features AI.
- **[`feature-router`](../features/feature-router.md)** — Orquestación dinámica de fuentes de señales.
- **[`fit-to-portfolio-search`](../features/fit-to-portfolio-search.md)** — Presión evolutiva para descorrelación.
- **[`strategy-ast-copilot`](../features/strategy-ast-copilot.md)** — Traducción asistida por LLM a sintaxis AST.
- **[`databank-lake`](../features/databank-lake.md)** — Almacenamiento columnar efímero para resultados R&D masivos.
- **[`alpha-harvesting-gateway`](../features/alpha-harvesting-gateway.md)** — Ingesta y refinamiento privado de estrategias anonimizadas.
- **[`la-colmena`](../moonshots/la-colmena.md)** — Red descentralizada de minería de estrategias.
- **[`portfolio-data-preparation`](../features/portfolio-data-preparation.md)** — Normalización de curvas y cálculo de matriz Pearson.
- **[`glass-box-ai-translator`](../glass-box-ai-translator.md)** — Traducción de pesos DRL opacos a AST visual y lenguaje natural.
- **[`dsr-tracking-engine`](../features/dsr-tracking-engine.md)** — Registro global de intentos ($N$) para deflación de Sharpe.
- **[`node-preview`](../features/node-preview.md)** — Caché de vistas previas y simulación rápida para edición interactiva de nodos.
- **[`event-driven-pipeline-triggers`](../features/event-driven-pipeline-triggers.md)** — Automatización de flujos de descubrimiento y reoptimización reactivos basados en eventos.

---

---

## Tareas (TTRs) — Protocolo de Orquestación (§8.1)

### **TTR-001: Orquestación de Evolución Híbrida (NSGA-II + DRL Genesis)**
*   **Descripción:** Invoca a [`nsga2-optimizer`](../features/nsga2-optimizer.md) para realizar el *fine-tuning* de las tesis generadas por el motor DRL o descubrir estrategias Pareto-óptimas desde cero.
*   **Reglas de Orquestación:**
    * El fitness debe evaluarse usando el motor de backtesting certificado (ADR-0013).
    * **Simbiósis:** Si existe una tesis del DRL, el GA hereda su topología y optimiza únicamente los parámetros de seguridad/umbrales.
    * Cada candidato generado debe incluir el `process_id` del job genético, su `parent_mutation_id` y el `logic_hash` de la tesis base (ADR-0020 V2).
*   **Entrada:** `search_space_config`, `neural_thesis_payload` (opcional).
*   **Salida:** `pareto_front_candidates`.
*   **Precondición:** Datos de ingesta validados (Módulo `ingest`) disponibles.
*   **Postcondición:** Persistencia de individuos en `candidates` con rastro de linaje.

### **TTR-002: Orquestación de Minería SQL (Analytic Discovery)**
*   **Descripción:** Invoca a [`duckdb-sql-engine`](../features/duckdb-sql-engine.md) para ejecutar búsquedas de patrones estadísticos en el histórico.
*   **Reglas de Orquestación:**
    * Las queries SQL deben lanzarse sobre los archivos Parquet generados por `ingest`.
    * El resultado se inyecta como "Alpha Seeds" en el motor genético.
*   **Entrada:** `sql_query_alpha`.
*   **Salida:** `signal_candidates`.
*   **Precondición:** Módulo `ingest` ha finalizado la persistencia Parquet.
*   **Postcondición:** Semillas listas para evolución NSGA-II.

### **TTR-003: Orquestación de Detección de Régimen Awareness (HMM)**
*   **Descripción:** Invoca a [`hmm-regime-detection`](../features/hmm-regime-detection.md) para filtrar candidatos según el clima de mercado.
*   **Reglas de Orquestación:**
    * Solo candidatos con desempeño balanceado entre regímenes (o etiquetados explícitamente) pasan a validación.
    * El etiquetado de régimen debe vincularse al `version_node_id` del modelo HMM (ADR-0020 V2).
*   **Entrada:** `pareto_front_candidates`, `market_regimes`.
*   **Salida:** `regime_aware_candidates`.
*   **Precondición:** TTR-001 finalizado.
*   **Postcondición:** Atributo `regime_id` persistido por candidato en el DAG.

### **TTR-004: Orquestación de Test de Canasta (Universal Basket)**
*   **Descripción:** Invoca a [`universal-basket-backtester`](../features/universal-basket-backtester.md) para validar la robustez del alfa en múltiples activos.
*   **Reglas de Orquestación:**
    * Solo candidatos con Sharpe agregado > MIN_BASKET_SHARPE (default: 1.0) pasan a validación profunda.
    * No se permiten candidatos que solo funcionen en un activo (especificismo extremo).
*   **Entrada:** `regime_aware_candidates`, `default_basket`.
*   **Salida:** `basket_performance_report`.
*   **Precondición:** TTR-002 finalizado.
*   **Postcondición:** Atributo `basket_score` persistido en el DAG.

### **TTR-005: Orquestación de Optimización Bayesiana (Intelligent Fine-Tuning) — ADR-0032**
*   **Descripción:** Invoca a [`bayesian-optimizer`](../features/bayesian-optimizer.md) para realizar búsquedas inteligentes de parámetros sobre los candidatos del frente de Pareto.
*   **Reglas de Orquestación:**
    * Debe utilizar el `backtest-engine` para evaluar cada iteración sugerida por el motor bayesiano.
    * Los resultados refinados se registran como hijos del nodo original con el `logic_hash` de la configuración de optimización.
*   **Entrada:** `top_tier_candidates`, `search_space`.
*   **Salida:** `bayesian_refined_candidates`.
*   **Precondición:** TTR-001 y TTR-005 finalizados.
*   **Postcondición:** Candidatos listos para certificación en `validate`.

### **TTR-010: Orquestación de Deep Learning (Model-Free Alpha)**
*   **Descripción:** Invoca a [`deep-learning-suite`](../moonshots/deep-learning-suite.md) para el descubrimiento de "Regímenes de Recompensa" (Tesis de Alpha) sin hipótesis humanas.
*   **Reglas de Orquestación:**
    * Se debe respetar el límite de VRAM (ADR-0032) activando el fallback a CPU/Rust SIMD/Rayon si es necesario.
    * **Delegación Táctica:** El motor propone la macro-lógica; delega la simetría de ejecución al motor genético para el ajuste de paridad con el broker.
    * El `model_lineage_id` debe vincularse al `version_node_id` de la estrategia en el DAG (ADR-0020 V2).
*   **Entrada:** `normalized_market_data`, `reward_function_config`.
*   **Salida:** `neural_alpha_thesis`.
*   **Precondición:** Módulo `ingest` con datos normalizados disponibles.
*   **Postcondición:** Pesos y topología de la tesis guardados en el databank de estrategias.

> **TTR-006 / TTR-007:** Retirados — numeración reservada, alcance consolidado en TTRs adyacentes durante el refinamiento del backlog del módulo. Las referencias cruzadas existentes a estos TTRs se preservan como contexto histórico de planificación.

### **TTR-008: Compilación de Lógica Procedural (AST Compiler)**
*   **Descripción:** Invoca a [`ast-compiler`](../features/ast-compiler.md) para transformar el diseño visual o la tesis neuronal en lógica procedural optimizada para hardware.
*   **Reglas de Orquestación:**
    * Traduce el grafo lógico en Árboles de Sintaxis Abstracta (AST) que operan sobre matrices vectorizadas (Hardware Accelerated).
    * Cada estrategia debe pasar el check de tipos estrictos antes de ser admitida en el databank.
    * Genera el `design-manifest` (ADR-0020 V2) que blinda la IP y el contrato de ejecución inmutable.
*   **Entrada:** `raw_strategy_genome`.
*   **Salida:** `validated_ast_manifest`.
*   **Precondición:** TTR-001 o TTR-007 finalizados.
*   **Postcondición:** Estratégia lista para ser ejecutada bit-a-bit por el `backtest-engine`.


### **TTR-009: Orquestación de Versionado de Linaje (Strategy Versioning)**
*   **Descripción:** Gestiona la inserción de candidatos en el grafo de versiones [`strategy-versioning`](../features/strategy-versioning.md).
*   **Reglas de Orquestación:**
    * No se permiten duplicados en el DAG (Content-Addressed Hashing).
    * Toda nueva raíz de linaje debe portar el `institutional_tag` del experimento (ADR-0020 V2).
*   **Entrada:** `regime_aware_candidates`.
*   **Salida:** `version_hashes` (Sha256).
*   **Precondición:** Reducción de duplicados finalizada.
*   **Postcondición:** Población lista para el módulo `validate`.

### **TTR-014: Orquestación de Dimensionalidad AI**
*   **Descripción:** Invoca a [`ai-dimensionality-suite`](../moonshots/ai-dimensionality-suite.md) para simplificar el espacio de búsqueda de alfas.
*   **Reglas de Orquestación:**
    * Utiliza PCA o UMAP para detectar redundancia en los indicadores sugeridos.
    * El proceso de reducción se vincula al `logic_hash` del experimento (ADR-0020 V2).
*   **Entrada:** `raw_indicator_pool`.
*   **Salida:** `selected_features_vector`.
*   **Precondición:** TTR-002 (SQL Discovery) finalizado.
*   **Postcondición:** Espacio de búsqueda optimizado para el motor genético.

### **TTR-011: Orquestación del Enrutador de Señales (Feature Router)**
*   **Descripción:** Utiliza [`feature-router`](../features/feature-router.md) para dirigir el flujo de datos hacia los diferentes motores de generación.
*   **Reglas de Orquestación:**
    * Asegura que cada motor (NSGA/Simbólico nativo) reciba solo los datos necesarios para su transformación.
    * Las rutas activas se registran en el `audit_hash` del job (ADR-0020 V2).
*   **Entrada:** `data_stream`, `engine_requirements`.
*   **Salida:** `routed_data_payloads`.
*   **Precondición:** Datos normalizados en `ingest` disponibles.
*   **Postcondición:** Motores de generación alimentados sin redundancia de memoria.

### **TTR-012: Orquestación de Hemisferios de Asimetría (Strategy Ensemble)**
*   **Descripción:** Invoca a [`strategy-ensemble`](../features/strategy-ensemble.md) para desacoplar modelos direccionales.
*   **Reglas de Orquestación:**
    * Permite que el motor evolutivo asigne parámetros diferentes para Largos y Cortos (ADR-0041).
    * Inyecta el `source_signal_id` en el rastro forense de la candidata.
*   **Entrada:** `causal_verified_strategies`, `asymmetry_config`.
*   **Salida:** `asymmetric_ensemble_candidates`.
*   **Precondición:** TTR-006 finalizado.
*   **Postcondición:** Población lista para validación institucional.

> **TTR-013:** Retirado — numeración reservada, alcance consolidado en TTRs adyacentes durante el refinamiento del backlog del módulo.

### **TTR-015: Orquestación de Fitness Metamórfico (NSGA-II Tuning)**
*   **Descripción:** Configura el motor [`nsga2-optimizer`](../features/nsga2-optimizer.md) en modo `metamorphic` durante la evolución.
*   **Reglas de Orquestación:**
    - Sincroniza los pesos de fitness con el `institutional_tag` del experimento (ADR-0042).
    - Registra el `logic_hash` del modo activo en el rastro forense.
*   **Entrada:** `experiment_config`, `account_status_mock` (para simulación).
*   **Salida:** `context_aware_candidates`.

### **TTR-016: Orquestación de Resolución de Comodines (WildCards)**
*   **Descripción:** Intercepta ASTs parciales del humano y activa la búsqueda genética solo en nodos `wildcard_group`.
*   **Reglas de Orquestación:**
    - Coordina [`ast-compiler`](../features/ast-compiler.md) para marcar nodos y [`nsga2-optimizer`](../features/nsga2-optimizer.md) para resolverlos.
*   **Entrada:** `partial_strategy_ast`.
*   **Salida:** `fully_resolved_candidates`.

### **TTR-017: Orquestación de Presión de Descorrelación (Fit-to-Portfolio)**
*   **Descripción:** Invoca a [`fit-to-portfolio-search`](../features/fit-to-portfolio-search.md) como filtro de fitness en NSGA-II.
*   **Reglas de Orquestación:**
    *   Inyecta la curva del portafolio operativo en la memoria del motor genético.
    *   Aplica penalización evolutiva a genomas con correlación > `PORTFOLIO_CORRELATION_CAP`.
*   **Entrada:** `live_portfolio_curve`, `candidate_genome`.
*   **Salida:** `adjusted_fitness_score`.
*   **Precondición:** Portafolio activo disponible.
*   **Postcondición:** Búsqueda dirigida a estrategias ortogonales.

### **TTR-018: Orquestación de Diseño Guiado (AST Copilot)**
*   **Descripción:** Invoca a [`strategy-ast-copilot`](../features/strategy-ast-copilot.md) en el editor visual.
*   **Reglas de Orquestación:**
    *   Valida el grafo devuelto por el LLM contra el esquema estricto (AST Compiler / Serde).
    *   Instancia los nodos generados en el lienzo listos para la compilación manual (TTR-008).
*   **Entrada:** `user_nl_prompt`.
*   **Salida:** `partial_strategy_ast`.
*   **Precondición:** Recepción de prompt natural del usuario.
*   **Postcondición:** Bloques lógicos inyectados en la sesión de edición.

### **TTR-019: Orquestación de Persistencia R&D (Databank Lake)**
*   **Descripción:** Invoca a [`databank-lake`](../features/databank-lake.md) para persistir efímeramente los resultados masivos de la generación.
*   **Reglas de Orquestación:**
    *   Delega la escritura de las candidatas a formato columnar Parquet.
    *   Garantiza que no se inyecten objetos JSON AST en la base relacional durante la exploración.
*   **Entrada:** `fully_resolved_candidates`, `generation_metrics`.
*   **Salida:** `parquet_file_references`.
*   **Precondición:** Evaluaciones de fitness completadas.
*   **Postcondición:** Candidatos almacenados sin bloqueo de I/O, listos para la capa visual.

### **TTR-020: Orquestación de Simbolismo (Symbolic Discovery nativo)**
*   **Descripción:** Invoca a [`pysr-signal-discovery`](../moonshots/pysr-signal-discovery.md) (motor simbólico nativo, moonshot — ADR-0113) para encontrar ecuaciones puras.
*   **Reglas de Orquestación:**
    *   Alimenta la búsqueda matemática y traduce sus ecuaciones a AST compatibles con el genoma base.
*   **Entrada:** `market_vectors`.
*   **Salida:** `symbolic_equations`.
*   **Precondición:** Datos ingestados.
*   **Postcondición:** Ecuaciones disponibles para ensamble.

### **TTR-021: Orquestación de Ortogonalidad (Zero-Crossing Filter)**
*   **Descripción:** Invoca a [`zero-crossing-filter`](../features/zero-crossing-filter.md) en generación.
*   **Reglas de Orquestación:**
    *   Descarta candidatos cuyas señales dependan de umbrales difusos en lugar de cruces cero estrictos.
*   **Entrada:** `candidate_genome`.
*   **Salida:** `filtered_genome`.
*   **Precondición:** Genoma estructurado.
*   **Postcondición:** Candidatos purificados.

### **TTR-022: Orquestación de Evaluación Evolutiva (Backtest Engine)**
*   **Descripción:** Invoca a [`backtest-engine`](../features/backtest-engine.md) para la función fitness masiva.
*   **Reglas de Orquestación:**
    *   Ejecución vectorizada (No tick-by-tick para velocidad, asumiendo barra 1m).
*   **Entrada:** `candidate_ast`, `train_dataset`.
*   **Salida:** `fast_fitness_metrics`.
*   **Precondición:** Compilación AST exitosa.
*   **Postcondición:** Fitness score inyectado.

### **TTR-023: Orquestación de Lotaje Evolutivo (Precision Sizing)**
*   **Descripción:** Invoca a [`precision-sizing-models`](../features/precision-sizing-models.md) durante la evaluación.
*   **Reglas de Orquestación:**
    *   Usa un sizing fijo nominal ($100k estático) para que el fitness mida el Alpha puro sin componer capital.
*   **Entrada:** `fast_fitness_metrics`.
*   **Salida:** `normalized_returns`.
*   **Precondición:** Fills generados.
*   **Postcondición:** Curva limpia de sesgo de compounding.

### **TTR-024: Orquestación de Contrato (Design Manifest)**
*   **Descripción:** Invoca a [`design-manifest`](../features/design-manifest.md) para inyectar límites SMART.
*   **Reglas de Orquestación:**
    *   El genoma debe ser envuelto por el manifiesto de riesgo antes de guardarlo en Databank.
*   **Entrada:** `candidate_ast`.
*   **Salida:** `wrapped_ast`.
*   **Precondición:** Genoma Pareto-óptimo.
*   **Postcondición:** AST blindado con reglas institucionales.

### **TTR-025: Orquestación de Diseño Manual (Visual DAG Editor)**
*   **Descripción:** Orquesta la interacción con [`visual-dag-editor`](../features/visual-dag-editor.md).
*   **Reglas de Orquestación:**
    *   Traduce los clicks y drags del usuario en un JSON AST y lo envía al compilador.
*   **Entrada:** `ui_events`.
*   **Salida:** `user_generated_ast`.
*   **Precondición:** Intervención humana.
*   **Postcondición:** Inyección manual en el pipeline.

### **TTR-026: Orquestación de Ajuste Fino (Parameter Optimization)**
*   **Descripción:** Invoca a [`parameter-optimization`](../features/parameter-optimization.md) (Grid/Random).
*   **Reglas de Orquestación:**
    *   Aplica búsquedas de fuerza bruta focalizada si el frente de Pareto está poco poblado.
*   **Entrada:** `parameter_ranges`.
*   **Salida:** `fine_tuned_genomes`.
*   **Precondición:** Frente de Pareto inestable.
*   **Postcondición:** Espacio local saturado.

### **TTR-027: Orquestación de Desacople (Async Job Executor)**
*   **Descripción:** Invoca a [`async-job-executor`](../features/async-job-executor.md) para las evoluciones.
*   **Reglas de Orquestación:**
    *   Las generaciones (ej: 1000 gens) no bloquean el API; se reporta progreso via WebSockets.
*   **Entrada:** `evolution_task`.
*   **Salida:** `job_id`.
*   **Precondición:** Petición de generación.
*   **Postcondición:** Procesamiento en background.

### **TTR-028: Orquestación de Indicadores Dinámicos (Adaptive Volume)**
*   **Descripción:** Invoca a [`adaptive-volume-indicators`](../features/adaptive-volume-indicators.md) como bloques genéticos.
*   **Reglas de Orquestación:**
    *   Permite que NSGA-II seleccione indicadores como KAMA o VIDYA en lugar de EMAs rígidas.
*   **Entrada:** `indicator_pool`.
*   **Salida:** `genome_with_adaptive_nodes`.
*   **Precondición:** Inicialización de población.
*   **Postcondición:** Mayor adaptabilidad.

### **TTR-029: Orquestación de Aislamiento Genético (Worker Isolation)**
*   **Descripción:** Invoca a [`worker-isolation-orchestrator`](../features/worker-isolation-orchestrator.md).
*   **Reglas de Orquestación:**
    *   Si una estrategia lanza Out-of-Memory, el worker aislado muere sin colapsar el orquestador principal.
*   **Entrada:** `unsafe_ast_eval`.
*   **Salida:** `safe_evaluation_result`.
*   **Precondición:** AST complejo o recursivo.
*   **Postcondición:** Resiliencia de la granja de servidores.

### **TTR-030: Orquestación Forense R&D (Audit Log)**
*   **Descripción:** Invoca a [`audit-log`](../features/audit-log.md) para certificar el experimento.
*   **Reglas de Orquestación:**
    *   Firma todo el bloque evolutivo (Semillas base y resultados de fitness).
*   **Entrada:** `generation_summary`.
*   **Salida:** `experiment_audit_hash`.
*   **Precondición:** Evolución finalizada.
*   **Postcondición:** Trazabilidad de origen probada.

### **TTR-031: Orquestación de Builder Telemetry (Monitoreo Operativo)**
*   **Descripción:** Invoca a [`telemetry`](../features/telemetry.md) para emitir métricas operativas del Worker genético en tiempo real.
*   **Reglas de Orquestación:**
    *   Emisión asíncrona de métricas (`strategies_generated`, `ETA`, `VRAM usage`, `State Probability`) vía gRPC/WebSocket.
    *   Forzado periódico de recolección de basura (`gc.collect()`) basado en la presión de RAM.
    *   El worker más exitoso emite el evento `best_strategy_update`.
*   **Entrada:** `worker_hardware_metrics`, `genetic_throughput_stats`.
*   **Salida:** `websocket_telemetry_stream`.
*   **Precondición:** Workers paralelos activos (TTR-029).
*   **Postcondición:** Dashboard en vivo actualizado sin bloquear la ejecución evolutiva.

### **TTR-032: Orquestación de Recolección de Alpha (Alpha Harvesting Gateway)**
*   **Descripción:** Invoca a [`alpha-harvesting-gateway`](../features/alpha-harvesting-gateway.md) para inyectar estrategias externas anonimizadas en el pool evolutivo local.
*   **Reglas de Orquestación:**
    *   El AST debe ser rigurosamente sanitizado antes de insertarse en la población.
    *   Los candidatos marcados como importados heredan un tag de linaje externo en su `audit_hash`.
*   **Entrada:** `external_alpha_payload`.
*   **Salida:** `sanitized_seed_genome`.
*   **Precondición:** Carga de archivo externo por parte del usuario.
*   **Postcondición:** Candidato inyectado en la siguiente ronda de evaluación genética.

### **TTR-033: Orquestación de Traducción AI (Glass-Box AI Translator)**
*   **Descripción:** Invoca a [`glass-box-ai-translator`](../features/glass-box-ai-translator.md) para convertir pesos opacos de redes neuronales (DRL) en estructuras lógicas auditables.
*   **Reglas de Orquestación:**
    *   Se ejecuta asincrónicamente tras la obtención de un candidato del motor de Deep Learning.
    *   La narrativa semántica generada por el LLM se vincula permanentemente al `version_node_id` de la estrategia.
*   **Entrada:** `neural_alpha_thesis`.
*   **Salida:** `visual_dag_ast`, `semantic_explanation`.
*   **Precondición:** TTR-010 finalizado.
*   **Postcondición:** Candidato DRL legible, disponible para edición manual (TTR-025).

### **TTR-034: Orquestación de Fundaciones de Portafolio (Portfolio Data Preparation)**
*   **Descripción:** Invoca a [`portfolio-data-preparation`](../features/portfolio-data-preparation.md) para procesar las curvas de rendimiento genéticas antes de delegarlas al gestor de portafolio.
*   **Reglas de Orquestación:**
    *   Calcula de forma masiva (batch) la matriz de correlación de Pearson y escala las curvas de rendimiento.
    *   Debe registrar el `indicator_state_hash` de la matriz en DuckDB.
*   **Entrada:** `pareto_front_candidates`.
*   **Salida:** `normalized_equity_curves`, `pearson_correlation_matrix`.
*   **Precondición:** Finalización de evaluaciones evolutivas (TTR-022).
*   **Postcondición:** Matriz de correlación inmutable persistida y lista para que `manage` aplique optimización HRP.

### **TTR-035: Orquestación de Registro de Intentos ($N$) para DSR**
*   **Descripción:** Invoca a [`dsr-tracking-engine`](../features/dsr-tracking-engine.md) para contabilizar cada intento de backtest realizado por los workers.
*   **Reglas de Orquestación:**
    *   Cada worker debe reportar el conteo de intentos y la varianza de Sharpe observada en su lote de trabajo.
    *   Los metadatos finales se adjuntan a la `SessionID` de generación.
*   **Entrada:** `worker_trial_stats`.
*   **Salida:** `updated_session_metadata`.
*   **Precondición:** Ejecución de backtests en workers activa.
*   **Postcondición:** Atributos $N$ y $\sigma^2$ listos para ser consumidos por el módulo `validate`.

### **TTR-036: Orquestación de Minería Descentralizada (La Colmena)**
*   **Descripción:** Orquestar la distribución de trabajos de exploración hacia la red de nodos mineros de [`la-colmena`](../moonshots/la-colmena.md) y recolectar las candidatas descubiertas.
*   **Reglas de Orquestación:**
    *   Formular trabajos de minería basados en las configuraciones de espacio de búsqueda y distribuirlos asincrónicamente.
    *   Verificar los resultados recibidos mediante el motor de prueba de trabajo probabilístico antes de guardarlos en el Databank.
    *   Registrar la firma digital del minero, el ID de hardware, y los metadatos de validación (ADR-0020 V2) en cada candidato inyectado.
*   **Entrada:** `mining_job_spec`.
*   **Salida:** `verified_mined_candidates`.
*   **Precondición:** Protocolo de verificación probabilística de backtests activo en el servidor.
*   **Postcondición:** Candidatos importados y agregados al pool de linaje del DAG.

### **TTR-037: Orquestación de Vistas Previas Rápidas (Node Preview)**
*   **Descripción:** Orquestar la obtención, almacenamiento y regeneración asíncrona no bloqueante de curvas de equidad reducidas y métricas de rendimiento para los nodos interactivos del Strategy Inspector.
*   **Reglas de Orquestación:**
    *   Invoca a [`node-preview`](../features/node-preview.md) para recuperar el caché local de SQLite o despachar el micro-backtest si se requiere.
    *   Maneja eventos de invalidación reactiva sobre el AST e invalida de forma atómica en cascada los nodos dependientes.
    *   Los metadatos se registran conforme al perfil IA / R&D (ADR-0020 V2).
*   **Entrada:** `node_selection_payload`, `ast_edit_event`.
*   **Salida:** `cached_node_preview` (JSON blob) o `regeneration_job_id`.
*   **Precondición:** Canvas del Strategy Inspector activo en Nivel 3.
*   **Postcondición:** Gráfico de rendimiento renderizado o actualizado en Flutter.

### **TTR-999: Implementación del Protocolo Fail-Fast Safe (ADR-0066)**
*   **Descripción:** Garantizar que cualquier invocación a componentes de validación o procesamiento intensivo esté gobernada por la cascada de intensidad.
*   **Reglas de Orquestación:**
    *   **Short-Circuit Mandatorio:** El módulo debe validar el éxito de los filtros `LIGHT` antes de solicitar recursos para tareas `MEDIUM` o `HEAVY`.
    *   **Telemetry:** Registrar el ahorro de ciclos de CPU/GPU cuando se produzca un descarte temprano.
*   **Entrada:** `ComputeIntensityMetadata`.
*   **Salida:** `fail_fast_execution_status`.
*   **Postcondición:** Optimización del consumo de hardware bajo el principio de Soberanía Local (ADR-0032).

### **TTR-038: Orquestación de Disparadores Reactivos (Event-Driven Pipeline Triggers)**
*   **Descripción:** Integra la automatización reactiva llamando a [`event-driven-pipeline-triggers`](../features/event-driven-pipeline-triggers.md) para lanzar pipelines de generación asíncronamente ante eventos macro.
*   **Reglas de Orquestación:**
    - El daemon se activa post-ingesta exitosa y evalúa las condiciones lógicas de los disparadores.
    - Los jobs de generación disparados de forma automática se registran en el `async-job-executor` con prioridad reducida para no saturar los hilos de live trading.
*   **Entrada:** `market_event_payload`, `pipeline_trigger_rules`.
*   **Salida:** `triggered_job_id`.
*   **Precondición:** Detección de régimen HMM completado.
*   **Postcondición:** Pipeline en cola de ejecución asíncrona.

### **TTR-039: Orquestación de Inspección de Superficie de Parámetros (Plateau Co-Pilot)**
*   **Descripción:** Invoca a [`plateau-copilot`](../features/plateau-copilot.md) para que el analista inspeccione el mapa de calor 2D de la superficie de parámetros de una candidata recién generada antes de promoverla a validación.
*   **Reglas de Orquestación:**
    *   La detección de meseta se delega a [`topological-plateau-finder`](../features/topological-plateau-finder.md); aquí solo se visualiza y se captura la elección humana exploratoria.
    *   El parámetro de producción NUNCA queda fijado sin clic humano explícito.
*   **Entrada:** `candidate_parameter_sweep`, `target_metric`.
*   **Salida:** `human_inspected_parameters`, `plateau_suggestion`.
*   **Precondición:** Candidata generada con barrido de parámetros disponible.
*   **Postcondición:** Selección exploratoria registrada para arrastre a `validate`.

### **TTR-040: Orquestación del Registro de Dominios Genómicos (ADR-0108)**
*   **Descripción:** Generaliza TTR-016 (Resolución de Comodines) para que el motor evolutivo opere sobre el conjunto de dominios del Registro de Dominios Genómicos (ADR-0108) declarado en `ACTIVE_GENOME_DOMAINS` — uno, varios o los 4 simultáneamente —, no solo sobre el Genoma de Señal.
*   **Reglas de Orquestación:**
    *   Lee `ACTIVE_GENOME_DOMAINS` del Manifest vía [`ast-compiler`](../features/ast-compiler.md) y delega la resolución de los nodos `wildcard_group` de los dominios activos a [`nsga2-optimizer`](../features/nsga2-optimizer.md) TTR-006 (genoma compuesto multi-dominio simultáneo); si la corrida es además una co-evolución de cartera, TTR-007 opera en paralelo sobre la población de Manifests (eje ortogonal).
    *   Los genomas de los dominios fuera de `ACTIVE_GENOME_DOMAINS` viajan congelados a la evaluación de fitness ("Wildcard Invertido", ADR-0108/ADR-0109).
    *   Si Riesgo y Gestión de Posición (ADR-0109) está en `ACTIVE_GENOME_DOMAINS`, los Genes de Acción resueltos pueden materializarse vía [`advanced-trade-management`](../features/advanced-trade-management.md), [`precision-sizing-models`](../features/precision-sizing-models.md) y [`multi-ticket-manager`](../features/multi-ticket-manager.md) (Split_Position).
    *   Si Régimen y Filtro de Entorno (ADR-0110) está en `ACTIVE_GENOME_DOMAINS`, los Genes de Condición pueden provenir de [`hmm-regime-detection`](../features/hmm-regime-detection.md) y la máscara resultante generaliza el precedente FIJO de [`vector-time-pruning`](../features/vector-time-pruning.md) y [`regime-guard`](../features/regime-guard.md).
    *   Si Portafolio y Correlación (ADR-0111) está en `ACTIVE_GENOME_DOMAINS`, los Genes de Condición pueden provenir de [`fit-to-portfolio-search`](../features/fit-to-portfolio-search.md) y [`portfolio-rules`](../features/portfolio-rules.md), y los Genes de Acción se materializan vía [`portfolio-optimizer`](../features/portfolio-optimizer.md).
    *   Cuando 2 o más dominios están activos simultáneamente, una misma Regla Genómica puede combinar Genes de Condición y de Acción de dominios distintos (p.ej. condición de Régimen + acción de Riesgo y Gestión).
*   **Entrada:** `partial_strategy_ast`, `ACTIVE_GENOME_DOMAINS`.
*   **Salida:** `fully_resolved_candidates` (con atribución de fitness por dominio, ADR-0108).
*   **Precondición:** TTR-008 (Compilación AST) ha etiquetado los nodos `wildcard_group` por dominio de origen.
*   **Postcondición:** Candidatos listos para `validate`, con las Compuertas de Robustez correspondientes a TODOS los dominios en `ACTIVE_GENOME_DOMAINS` (ADR-0109/ADR-0110/ADR-0111, las que apliquen) pendientes.

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundamentos (ADR-0020 V2):** El catálogo de los 25 campos maestros está en la sección "Épica 0: Esqueleto Fundacional" de este documento (referencia, no esquema). Toda entidad persistida por este módulo incluye el Grupo I de forma universal; los Grupos II–V se aplican solo en los campos que el Perfil Técnico de cada feature exige (Filtro de Relevancia, ADR-0020 V2) — nunca el catálogo completo.

- **Decisión Arquitectónica Asociada:**
    - ADR-0005: Versionamiento Git-like (DAG).
    - ADR-0019: Interoperabilidad Rust (Optimización masiva).
    - ADR-0020 V2: Inundación de Fundaciones.

---

## Dependencias
**Depende de:**
- [`ingest`](../modules/ingest.md) — para la obtención de datos "Golden Source" (barras validadas con régimen asignado).
- [`feedback`](../modules/feedback.md) — para la aplicación de restricciones reactivas.

**Consumido por:**
- [`validate`](../modules/validate.md) — para la certificación de robustez de la población (bloquea hasta tener candidatas listas para someter a pruebas estadísticas).

---

## Decisión Arquitectónica Asociada

- ADR-0008: Configurabilidad Universal (todos los parámetros son ajustables)
- ADR-0019: Interoperabilidad Rust (motor simbólico nativo/Rust SIMD-Rayon)
- ADR-0020 V2: Inundación de Fundaciones
- ADR-0108: Arquitectura de Genomas Modulares por Dominio (TTR-040)

---

## Condición de Transición a Módulo Siguiente

Generate → Validate:
- Hay estrategias candidatas generadas
- Cada candidata tiene un score de calidad (no nulo)
- Candidatas están listas para ser sometidas a pruebas

---

## Flujos Alternativos

**Manual Strategy Injection:**
- Un usuario puede crear una estrategia manualmente (sin generar automáticamente)
- Esa estrategia entra al pipeline como si hubiera sido generada
- Continúa hacia validación con el resto de candidatas

**Feedback Loop Forzado:**
- El módulo de retroalimentación puede enviar constraints/anomalías detectadas
- Genera usa esos constraints como restricciones adicionales en la próxima ejecución
- Ej: "Evitar estrategias que operen en spreads > X"
