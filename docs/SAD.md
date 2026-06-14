# Drasus Engine - Software Architecture Document (SAD)

## 1. Introducción y Objetivos

**Drasus Engine** es una infraestructura privada de trading algorítmico diseñada para el descubrimiento, validación y ejecución autónoma de estrategias de alto rendimiento. No es un bot convencional; es un **motor matemático determinista** (el sistema da siempre el mismo resultado numérico) acoplado a una **capa de interacción** que maneja las tareas del mundo real (lectura/escritura, manejo de errores) que interactúa con mercados reales.

### 1.1 Visión Estratégica (PRD §2)

#### El Problema (Causalidad del Proyecto)
La generación de Alpha es un problema **combinatorialmente explosivo** y plagado de sesgos:
* **Overfitting pervasivo:** Confusión entre ruido y rentabilidad genuina.
* **Ilusión estadística:** Backtests que ganan pero pierden en mercados reales.
* **Fricción invisible:** Spreads, comisiones y límites de penetración (Pardo) no modelados.
* **Regímenes cambiantes:** Estrategias robustas que se rompen ante cambios de volatilidad.

#### Propuesta de Valor
Drasus Engine es **infraestructura soberana** para el descubrimiento automático de Alpha:
1. **Descubrimiento:** Sin hipótesis humanas (NSGA-II + regresión simbólica nativa sobre el AST, ADR-0113).
2. **Validación:** Rigor institucional (WFA, Monte Carlo, CPCV).
3. **Ejecución:** Protección multinivel (10 risk steps, kill switch < 5s).
4. **Aprendizaje:** Cierre de ciclo causal (Regime Aware + Feedback).

### 1.2 Métricas de Éxito (KPIs Instrumental)

| Métrica | Target MVP | Razón Técnica |
|---|---|---|
| **Throughput Backtest** | Medible y demostrablemente más rápido que MT5, SQX y QuantConnect en la misma máquina (ADR-0114; sin KPI absoluto) | Posibilita exploración masiva. |
| **Live Order Latency** | ≤100ms (end-to-end) | Ejecución competitiva. |
| **PBO (Backtest Overfitting)** | ≤0.10 (Configurable) | Garantiza que el Alpha no es suerte. |
| **Reproducibilidad** | 100% bit-a-bit | Auditoría forense y científica. |

---

## 2. Arquitectura Técnica (Monolito Rust de Alto Rendimiento)

Drasus Engine elimina la complejidad de microservicios externos y dependencias de red pesadas (Docker, ClickHouse, Redis). Todo el ecosistema reside en un monorepo gestionado por `cargo`, utilizando **Rust** compilado a nivel nativo para alcanzar latencias críticas sin sobrecarga de recolección de basura.

### 2.1 El Stack Principal

| Componente | Tecnología | Descripción |
| :--- | :--- | :--- |
| **Interfaz de Usuario (UI)** | Flutter (Dart) | Renderizado fluido nativo en GPU (Impeller) para gráficos financieros masivos y una Interfaz de Usuario de ultra rendimiento. |
| **Integración UI-Core** | flutter_rust_bridge (FFI) | Comunicación C-ABI de memoria compartida con latencia cero entre la interfaz gráfica y el motor numérico en Rust. |
| **Backend Orquestador** | Rust | Core nativo compilado, empaquetado conjuntamente con la UI. |
| **Motor de Ejecución** | NautilusTrader (núcleo v2, crates Rust) | Motor institucional consumido como dependencias Cargo nativas (sin intérprete Python), con paridad absoluta entre simulación y operativa real (ADR-0107). |
| **Aceleración Matemática** | Rust Native (SIMD) | Optimización de algoritmos críticos a código máquina utilizando múltiples hilos de Procesador (CPU) o núcleos vectoriales. |
| **Laboratorio de IA** | `ndarray` + Rayon (default) → `candle` (Rust puro) si se justifica (ADR-0112) | Monte Carlo masivo CPU-first; redes pequeñas (autoencoders) en Rust puro. `tch-rs`/libtorch erradicados. |
| **Procesamiento de Datos** | Polars (Rust Native) | Ingesta y transformación de datos masivos mediante evaluación perezosa y procesamiento multihilo nativo. |
| **Motor Analítico** | DuckDB (Embebido) | Consultas SQL vectorizadas directamente sobre archivos Parquet sin saturar la memoria RAM (*Out-of-Core*). |
| **Persistencia (OLTP)** | SQLite (Modo WAL) | Gestión de configuraciones, libros de estrategias, almacén de eventos y registros de operaciones cifrados. |
| **Series Temporales (OLAP)** | Archivos Parquet | Almacenamiento en columnas particionado estilo Hive para datos históricos de mercado y resultados de investigación. |
| **Transporte UI ↔ Backend** | FFI / gRPC | Intercambio binario de alta velocidad en memoria compartida (Local) o mediante flujos gRPC (Ejecución VPS Headless). |
| **Interfaz de Comandos (CLI)** | Clap (Rust) | Interfaz de consola que interactúa con el motor central puenteando el API para ejecuciones locales directas. |

### 2.2 El Músculo: Simulación y Ejecución (NautilusTrader + Rust SIMD)

*   **Mecanismo de Integración (ADR-0107):** NautilusTrader se integra consumiendo los crates Rust nativos de su núcleo v2 (backtesting de eventos y ejecución live) como dependencias Cargo con versiones fijadas y vendorizadas. No existe fork, ni sidecar, ni capa Python: el motor corre dentro del mismo proceso Rust del Core, detrás de la capa anticorrupción contratada en `nautilus-integration`. Las clases de activo de primera clase son acciones, forex, futuros, ETFs y CFDs; las opciones financieras se difieren a la última fase del roadmap.

El motor de simulación implementa modelos de fidelidad progresiva para garantizar precisión operativa:

*   **Modos de Simulación de Históricos:**
    *   **Every Tick (Simulated Ticks):** Simulación de alta fidelidad basada en barras de 1M. Descompone la barra en 4 movimientos (O, H, L, C) para validar SL/TP contra extremos reales.
    *   **Every Tick based on Real Ticks (Real Ticks):** El nivel más alto de precisión. La formación de la vela y ejecución ocurren sobre ticks reales (requiere datos históricos de ticks).
    *   **1 Minute OHLC:** Reconstrucción de temporalidades superiores procesando barras de 1 minuto en su precio de apertura.
    *   **Open Prices Only:** Modo de máxima velocidad para optimización masiva de parámetros iniciales.

*   **Aceleración y Precisión:**
    *   **Simulador Tick-by-Tick Vectorizado:** Procesa cada movimiento individual con precisión milimétrica para determinar el orden exacto de ejecución, eliminando falsos positivos.
    *   **Rust Safety:** Ejecución de fórmulas de señal a velocidades nativas con seguridad de memoria garantizada.
    *   **Fricción Institucional Mandatoria:** Simulación realista de spreads variables, comisiones por niveles, lógica de triple swap y limitación de penetración (Pardo).

*   **Capacidades Avanzadas:**
    *   **Universal Basket Backtesting:** Ejecución multi-activo y multi-timeframe simultánea para generar una Curva de Equidad Global agregada (combate el curve-fitting).
    *   **Barras Algorítmicas:** Soporte nativo para gráficos no temporales (Renko, Range, Volume, Tick) construidos dinámicamente.
    *   **Bar-Open Alignment & Execution:** Garantía de paridad 1:1 entre Backtest y Real. La lógica de decisión se ejecuta estrictamente al abrirse una nueva vela (Bar Open), eliminando el sesgo de mirar datos intra-vela no disponibles en tiempo real.
    *   **Fricción Realista Institucional (ADR-0017):** Configuración de Triple Swap (reloj de simulación), limitación de penetración (Pardo) exigiendo que el precio atraviese el límite por $X$ ticks, y diferenciación entre Settlement e Histórico (Davey).
    *   **Ruteo por Perfil de Volumen:** Suspensión automática ante caídas de liquidez detectadas en el pre-volumen para evitar deslizamientos excesivos.

*   **Épica 3: Ejecución Institucional (Execute):**
    *   **Live Execution Engine con Paridad Real:** Ejecución determinista acoplada a NautilusTrader para mantener paridad inquebrantable Out-Of-Sample.
    *   **Multiplatform Execution Bridge:** Comunicación nativa vía FFI/gRPC o canales binarios hacia terminales externas, enviando comandos genéricos sin exportación de lógica local.
    *   **Multi-Ticket Manager:** Gestión concurrente de múltiples tickets individuales por estrategia identificados vía signal hash y timestamp para romper limitaciones de una operación simultánea.

### 2.3 El Cerebro: Laboratorio de Inteligencia Artificial (CPU-First — `ndarray`/Rayon, `candle` opcional)

El sistema está optimizado para hardware de alto rendimiento doméstico (Sovereign Infrastructure). **El cómputo es CPU-first por defecto (ADR-0112);** una GPU es un acelerador opcional vía `candle`, nunca un requisito, y `tch-rs`/libtorch quedan erradicados (preservan el binario único, ADR-0029):

| Operación | Motor por defecto | Acelerador opcional (si se justifica) | Nota |
| :--- | :--- | :--- | :--- |
| **HPC Monte Carlo** (10K iter) | CPU `ndarray` + Rayon/SIMD | `candle` (GPU dinámica) | Es permutación matricial, no deep learning (ADR-0061) |
| **UMAP** (100K puntos) | CPU Rust `ndarray` | `candle` | Reducción dimensional, EPIC-8 |
| **Autoencoder** (10K trades) | CPU Rust puro (`candle`) | `candle` (GPU dinámica) | Red pequeña; no requiere libtorch |
*Regla de Carga:* la ausencia de GPU jamás impide la ejecución; toda carga corre en CPU preservando el determinismo (ADR-0107).

*   **Laboratorio de IA Soberano (Hybrid Genesis Engine):**
    *   **HPC Monte Carlo Híbrido (CPU `ndarray`/Rayon):** Motor de simulación que ejecuta las permutaciones matriciales y la lógica de mutación dinámica en CPU multihilo; GPU opcional vía `candle` solo si un benchmark lo justifica (ADR-0061/0112).
    *   **Sinergia GA-DRL:** El motor utiliza agentes de Aprendizaje por Refuerzo Profundo (PPO/DQN) para descubrir "Regímenes de Recompensa" y proponer una "Tesis de Alpha" (Macro-lógica). El Genetic Builder (NSGA-II) realiza el ajuste fino (Tuning) de umbrales y topologías de grafos para garantizar paridad operativa.
    *   **Sovereign AI Wizard (Copilot):** Asistente **opcional** de LLM local soberano (vía `candle` embebido, nunca Ollama como requisito — ADR-0115) para la construcción guiada del Grafo de Lógica (Strategy AST); nunca para veredictos ni datos operativos. Actúa como fontanero determinista (ej. *"Crea filtro de spread"*) eliminando alucinaciones, y ofrece **Strategy Self-Explanation** para auditar en lenguaje humano los modelos complejos.
    *   **Descubrimiento No-Template:** Descubrimiento de ineficiencias matemáticas sin hipótesis humanas previas, ensamblando bloques funcionales o aprendiendo directamente de la serie temporal.
    *   **Compilador AST de Lógica Procedural:** Conversión automática de grafos lógicos en Árboles de Sintaxis Abstracta (AST) optimizados para ejecución vectorizada en CPU (Rust SIMD); aceleración GPU opcional vía `candle` (ADR-0112), nunca requisito.
    *   **Minería Simbólica (Rust nativo, ADR-0113):** Descubrimiento de ecuaciones matemáticas Alpha ($Alpha = f(x)$) como **modo del motor genético NSGA-II sobre el AST** (no PySR), priorizando la transparencia sobre "cajas negras". La minería simbólica de forma libre se difiere a moonshot (`egg`).
    *   **Fit-to-Portfolio Search:** Búsqueda proactiva que inyecta presión evolutiva para castigar correlaciones > 0.3 con el portafolio actual durante la fase generativa, forzando diversificación estructural temprana.
    *   **NSGA-II (Rust Native):** Descubrimiento multiobjetivo (Sharpe vs DD) para optimización de Pareto.
    *   **Motor de Lógica Difusa:** Sustitución de reglas binarias por evaluación probabilística continua (SignalConfidence 0.0-1.0) para mitigar el ruido (Whipsaws).
    *   **Optimización Bayesiana:** Búsqueda inteligente de parámetros reduciendo drásticamente el tiempo de optimización vs Grid Search.

*   **Arquitecturas de Red (Deep Learning):**
    *   Modelos LSTM y Transformadores para predicción de secuencias.
    *   Agentes DRL (PPO/DQN) para extracción de características no lineales y descubrimiento de regímenes.
    *   Clasificación de Regímenes vía Modelos Ocultos de Markov (HMM) en 4 estados: Tendencia, Rango, Volátil y Calmo.

*   **Motores Adaptativos y Procesamiento de Series (Fractional Differencing):** 
    *   **Diferenciación Fraccional (Preservación de Memoria):** Implementación de pesos de ventana fija para lograr estacionariedad sin destruir la memoria estadística (señal predictiva) de la serie temporal.
    *   **Indicadores Dinámicos:** Bloques nativos de indicadores adaptativos (ER, KAMA, VIDYA) y métricas de flujo de dinero (Open Interest, Herrick Payoff).

*   **Lógica Causal y Motores de Confianza:**
    *   **[OLD-SCHOOL] Lógica Binaria y Difusa:** Integración de reglas booleanas tradicionales y motores de Lógica Difusa (`FuzzyGroup`) para evaluar el `Grado de Confianza` de la señal y mitigar el ruido.
    *   **[NEW-ERA] Motores de Confianza Bayesiana:** Modelos estocásticos continuos que emiten un `Grado de Certidumbre Predictiva`, permitiendo umbrales de disparo dinámicos (Disparador Metamórfico) condicionados al estado del capital (se relaja con colchón, se exige >95% en niveles de riesgo crítico).
    *   **[NEW-ERA] Motores de Asimetría Estructural:** Evaluación desacoplada de sesgos direccionales; los modelos para pánicos (Cortos) operan con arquitecturas independientes a las de distribución (Largos).

*   **Clasificación de Regímenes y Análisis de Estancamiento:**
    *   **[OLD-SCHOOL] Detector de Regímenes (HMM):** Clasificación estática en 4 estados (Tendencia, Rango, Volátil y Calmo) para el enrutamiento de flujos hacia Momentum o Mean Reversion.
    *   **[NEW-ERA] Filtro Dinámico Invisible HMM:** Decodificación dimensional de transacciones crudas L1 (Bid-Ask puro) para validar la similitud microscópica con entornos de éxito in-sample sin usar indicadores retrasados (Cero SMAs).
    *   **[NEW-ERA] Predicción de Estancamiento ARIMA:** Diagnóstico de anomalías de volatilidad futura. Si se detecta escasez de liquidez o rango asintótico (>85%), se desautorizan disparos para proteger el capital de comisiones y spreads ineficientes.

*   **Lógica Genética Avanzada y Metamorfismo de Fitness:**
    *   **Configuración Evolutiva NSGA-II:** Implementación de decimación (generación cero extendida), renovación sanguínea periódica y detección de convergencia para evitar el estancamiento genético.
    *   **[NEW-ERA] Fitness Metamórfico de Estado:** El objetivo de optimización muta dinámicamente según el estado de la cuenta (Fase Challenge: Maximización de Profit; Fase Funded: Maximización de Estabilidad y Defensa de Capital). 

*   **Dimensionamiento de Precisión Transversal:**
    *   Protocolo unificado de cálculo de riesgo compartido por simulación y ejecución.
    *   Implementación de modelos **Ratio Fijo**, **Ajuste por ATR** y **Porcentaje de Capital** (Risk Percent Sizing) con paridad bit-a-bit entre backtest y real.

*   **Programación Evolutiva Colaborativa (WildCards):**
    *   Soporte para ASTs parciales (Árboles de Sintaxis Abstracta) donde el humano fija nodos de control y el motor genético resuelve los "Comodines" (WildCards) mediante búsqueda exhaustiva. Este mecanismo es la instancia fundacional ("Dominio de Señal") del **Registro de Dominios Genómicos** descrito a continuación (ADR-0108).

*   **Microestructura y Orden Flow (NautilusTrader L2):**
    *   Cálculo de **CVD (Delta de Volumen Acumulado)**, **VWAP** con bandas de desviación y **OFI (Desequilibrio de Flujo de Órdenes)**.
    *   Integración con datos de Nivel 2 (DOM) para detectar presión institucional y absorción de liquidez en tiempo real.

#### Arquitectura de Genomas Modulares por Dominio (ADR-0108)

El patrón de WildCards (genes evolutivos que resuelven nodos `wildcard_group` dentro de un AST parcial) se generaliza en una **Gramática de Genes Condición→Acción** aplicable a múltiples dominios del sistema, no solo a la lógica de entrada/salida. Cada dominio del **Registro de Dominios Genómicos** define:

*   **Genes de Condición:** predicados que observan el estado de un dominio específico (mercado, posición, régimen estructural, portafolio) y devuelven un veredicto booleano/categórico en cada barra o evento.
*   **Genes de Acción:** primitivas paramétricas — ya expuestas como comportamientos configurables por las Features existentes — que el motor activa o reconfigura cuando su Gen de Condición asociado se satisface.
*   **Wildcard Invertido Generalizado:** `ACTIVE_GENOME_DOMAINS` (CONFIG, `ast-compiler`) declara cualquier subconjunto no vacío de los 4 dominios; los genomas **fuera** de ese subconjunto quedan congelados. Cuando el subconjunto tiene más de un dominio, todos evolucionan juntos como un único genoma compuesto — patrón inverso al WildCards original (donde el humano fija salidas/filtros y el motor descubre entradas; aquí el motor descubre el/los genoma(s) activo(s) sobre lo que queda fijo). La co-evolución de cartera (Fase C) es un eje ortogonal: opera sobre un conjunto de Manifests, cada uno con su propio `ACTIVE_GENOME_DOMAINS`.
*   **Regla Genómica:** unidad de evolución — 1..`MAX_CONDITIONS_PER_RULE` Genes de Condición (AND/OR, de cualquier dominio activo) → 1..`MAX_ACTIONS_PER_RULE` Genes de Acción simultáneos (de cualquier dominio activo). Generaliza las reglas de entrada/salida multi-condición del Dominio de Señal (ADR-0043) a los 4 dominios.
*   **Compuerta de Robustez Propia:** cada dominio extiende el pipeline backtest → WFA → Monte Carlo con su propio modo de validación bloqueante antes de avanzar en el Lifecycle (§12).

| Dominio | Genes de Condición (ejemplos) | Genes de Acción (ejemplos) | Compuerta de Robustez | ADR |
| :--- | :--- | :--- | :--- | :--- |
| **1. Señal** (línea base) | Indicadores técnicos, estructura de precio | Entradas/salidas, filtros de sesión | Monte Carlo Modo 1/2, WFA estándar | ADR-0043 |
| **2. Riesgo y Gestión de Posición** (Fase A) | Drawdown de equity, racha de pérdidas/ganancias, duración de operación, múltiplo-R no realizado | Mutación de tamaño (multiplicador, % riesgo, Kelly acotado, riesgo monetario fijo), morfología de salida (split, scale-out, trailing de SL, decaimiento temporal) | Réplica de Estado de Riesgo (Monte Carlo, bloqueante) | ADR-0109 |
| **3. Régimen y Filtro de Entorno** (Fase B) | Exponente de Hurst, entropía de Shannon, pendientes Hull multinivel, estado HMM | Máscara binaria Permitido/Prohibido sobre el Genoma de Señal | WFA Segmentado por Régimen (bloqueante) | ADR-0110 |
| **4. Portafolio y Correlación** (Fase C) | Correlación móvil entre curvas de equidad, drawdown agregado, solapamiento direccional | Activación/desactivación de miembro, rotación de peso, cobertura sintética | Monte Carlo de Desfase Temporal (nuevo, bloqueante) | ADR-0111 |

Cuando `ACTIVE_GENOME_DOMAINS` activa más de un dominio, las Reglas Genómicas combinan Genes de Condición y de Acción de columnas distintas de la tabla anterior en un mismo individuo evolutivo (p.ej. condición de Régimen + acción de Riesgo y Gestión).

El dominio candidato de Ejecución y Enrutamiento (genes de latencia de bróker, profundidad de libro L2, ratio cancelación/fill) fue evaluado y **excluido** del Registro activo por falta de datos de microestructura consistentes para el operador retail/solopreneur (mismo principio que ADR-0100); queda archivado en [`genoma-ejecucion-enrutamiento`](./moonshots/genoma-ejecucion-enrutamiento.md).

### 2.4 Evaluador Institucional y Poda Temporal (Métricas orientadas al Alpha)
*Evaluación dual: Métricas estadísticas de alta velocidad y filtros estrictos orientados a proteger el capital.*

*   **Prop-Firm Grader (Filtro de Fondeo e Inundación Tóxica):**
    *   **Embudo Tóxico de Estrés (Prop-Firm MC):** Simulación condicional severa que destruye mutaciones que violen límites diarios absolutos (Drawdown > 4.5% Intradiario). Ignora drawdowns relativos temporales para enfocarse en la supervivencia de la cuenta maestra.
    *   **Configuración Dinámica:** Los límites (FTMO, TopStep, Darwinex) no están fijos en el código; se configuran mediante esquemas validables tipados en Rust (Serde).

*   **Robustez Decagonal (10 Perturbaciones Mandatorias):**
    *   **Motor de Perturbación:** Aplica 10 transformaciones (Trade Reordering, Data Perturbation, Slippage Stress, Shock Injection 3.5x ATR, etc.) para aislar el mérito de la lógica de entrada y la resiliencia ante outliers.
    *   **Broker Physics (Física de Broker):** Aleatorización de saltos (Min Distance), spreads y deslizamientos (Slippage) en tiempo real para simular la falta de volumen y fricciones de ejecución.

*   **Poda Temporal Vectorial (Vector-Time Pruning):**
    *   **Aislamiento de Pérdidas:** Si el sistema detecta que una estrategia pierde dinero consistentemente en momentos específicos (ej. viernes a las 14:00 por noticias macroeconómicas), corta y bloquea esa franja horaria automáticamente, salvando el resto de la operativa que sí funciona.

*   **Alpha Decoupling & Cross-Validation:**
    *   **Alpha Decoupling Module:** Neutralización de Beta y Hedging para aislar el rendimiento puro del algoritmo frente al sesgo inercial del mercado.
    *   **Cross-Market Validation:** Prueba automática de robustez en mercados correlacionados para descartar estrategias sobreajustadas a un solo activo.
    *   **Rule Ablation (Ablación de Reglas):** Desactivación sistemática de reglas para eliminar el ruido estadístico y simplificar la lógica.

#### **Protocolo de Validación "Fail-Fast" por Cascada de Intensidad (ADR-0066)**
Para garantizar escalabilidad y eficiencia extrema, el módulo `validate` no utiliza una secuencia fija, sino una **Orquestación por Cascada** basada en metadatos de intensidad de cómputo:

1.  **Capa de Autodescubrimiento:** El orquestador escanea todas las features activas y las clasifica dinámicamente en 3 cubetas: `LIGHT`, `MEDIUM` y `HEAVY`.
2.  **Ejecución en Cascada (Short-Circuit):**
    - **Bloque LIGHT (Análisis de Metadatos):** Se ejecutan filtros instantáneos (Navaja de Ockham, Sharpe Mínimo, WinRate). Si alguno falla, el pipeline se aborta (Fail-Fast).
    - **Bloque MEDIUM (Optimización Local):** Se ejecutan tests de impacto moderado (Ablación, Sensibilidad).
    - **Bloque HEAVY (Estrés de Hardware):** Solo si la estrategia sobrevive a los bloques anteriores, se asignan recursos de CPU/GPU para tests masivos (CPCV, Monte Carlo 10K, Cross-Market).
3.  **Gobernanza de Cómputo:** Este modelo permite añadir decenas de nuevas features al guantelete sin reconfigurar el pipeline; cada feature "conoce" su peso y se posiciona automáticamente en la cascada.

*   **CPCV (Combinatorial Purged Cross-Validation):**
    *   **Particionado No-Lineal:** División de la historia en miles de combinaciones de caminos para probar la robustez en escenarios nunca antes vistos.
    *   **Purging & Embargo:** Eliminación obligatoria de solapamientos temporales y correlaciones seriales post-trade para evitar el *Data Leakage*.
    *   **PBO (Probability of Backtest Overfitting):** Cálculo de la probabilidad de que los resultados sean producto del sobreajuste estadístico (si PBO > Threshold, estrategia rechazada).

*   **Institutional Metrics Suite (Muestrario Base):**
    *   **Performance Base:** Net Profit, Gross Profit/Loss, Win Rate, Profit Factor, Expectancy, Average Trade, Payoff Ratio.
    *   **Riesgo y Retorno:** Sharpe, Sortino, Calmar, VaR (95/99%), CVaR y Tail Ratio.
    *   **Drawdown y Estancamiento:** Max Drawdown ($ & %), Average Drawdown, Stagnation (periodo sin nuevo máximo), Time Under Water y Recovery Factor.
    *   **Calidad y Robustez:** Davey's Linearity Score (R²), Ulcer Index, Martin Ratio y PROM (Pessimistic Return on Margin).
    *   **Eficiencia de Ejecución:** Análisis MAE/MFE para optimización de Stops y TPs.
    *   **Microestructura y Desviación:** Exposure (% tiempo mercado), Z-Score de rachas, Trades Symmetry y SQN (System Quality Number) categorizado.
    *   **Implementación Dual (El secreto de la velocidad):** Las métricas en caliente (Hot-Path) las calcula directamente `NautilusTrader` en Rust. Las métricas masivas (durante la evolución genética) utilizan `Polars` y `SIMD` para procesar miles de estrategias por segundo sin sobrecarga de intérpretes.

*   **Statistical Inference Layer (EBTA — Robustness Filter):**
    *   **Deflated Sharpe Ratio (DSR):** Ajuste matemático del Sharpe Ratio basado en el número de intentos ($N$) y la varianza de los resultados en la fase de minería genética, combatiendo el *Selection Bias*.
    *   **White's Reality Check (WRC) & Romano-Wolf:** Pruebas de significancia estadística ajustadas para múltiples hipótesis (Family-Wise Error Rate). Romano-Wolf actúa como el estándar de oro mediante bootstrap CPU-first (`ndarray`/Rayon); aceleración GPU opcional vía `candle` (ADR-0112).
    *   **Market Detrender:** Eliminación de la componente Beta (tendencia base) para aislar el Alpha puro. Las estrategias que no sobreviven al mercado "aplanado" son rechazadas.
    *   **Logic Inversion:** Verificación de robustez estructural invirtiendo las señales de entrada para validar que la lógica no es producto del ruido.

*   **Capa de Fricción y Estabilización Institucional:**
    *   **Volatility Stabilization (Target Vol):** Certificación obligatoria de estabilidad bajo diferentes regímenes de volatilidad antes de la aprobación (ADR-0068).
    *   **Adverse Selection Modeling:** Modelado de *Limit Order Drop-Out* y peor escenario de *Fill Rate* (60%) para eliminar el sesgo de backtest optimista (ADR-0069).

*   **Vigilancia Operativa Pardo (Operational Safety):**
    *   **Pardo Profile Monitor:** Vigilancia continua de drift en métricas clave (Win%, Avg Win/Loss). Alerta o bloqueo si la desviación es >50% vs Perfil Histórico (ADR-0070).
    *   **Strategy Stop-Loss (SSL):** Desconexión mandatoria (`Hard Limit`) si el `Live DD > HistMaxDD × Safety Factor` (ADR-0070).

#### Modelo de Scoring Ponderado de Robustez (Anti-Parálisis) y Robustness Verdict Engine

El sistema reemplaza el enfoque binario de "Muerte Súbita" (descartar estrategias por fallar un solo test) por un **Scoring Ponderado (0-100)** que evita la parálisis por análisis. El score determina también el dimensionamiento de posición inicial.

**Matriz de Tests y Pesos:**

| Test | Peso | Propósito |
|---|---|---|
| **WFA (Walk-Forward)** | 30% | Capacidad de adaptación a datos nunca vistos. |
| **Monte Carlo (Trades)** | 25% | Resiliencia ante variaciones en el orden de ejecución. |
| **Monte Carlo (Tóxico)** | 20% | Supervivencia ante eventos diarios letales de Prop Firms. |
| **CPCV / PBO** | 15% | Probabilidad de sobreajuste estadístico. |
| **Ockham (Complexity Penalization)** | 10% | Castigo al exceso de parámetros y eliminación de reglas redundantes. |

**Regla de Aprobación:** Estrategia con Score > 75 se considera "Aprobable". El score determina el dimensionamiento de posición inicial: a mayor score, mayor lotaje.

**Robustness Verdict Engine (Veredictos en Lenguaje Natural):**
- **Veredicto en Lenguaje Humano:** Un motor de **plantillas deterministas** (ADR-0115) compone el resumen del guantelete de robustez; un LLM local soberano (`candle`) es realce opcional, nunca requisito ni dependencia de Ollama. Ejemplo: *"La estrategia sobrevive en el 98% de las mutaciones. El parámetro más sensible es el Trailing Stop. Fijado en el centro del rango estable (45 pips). Listo para revisión"*.
- **Identificación de Puntos de Ruptura:** Alerta sobre condiciones específicas donde el sistema colapsa. Ejemplo: *"Falla críticamente si el spread promedio supera los 2.5 pips"*.
- **Score Explicable:** Justificación semántica de por qué una estrategia obtuvo un score determinado.
- **Arquitectura:** El Verdict Engine consume los 5 resultados individuales de los tests y el score ponderado, y genera por **plantilla determinista** (ADR-0115) un veredicto estructurado con hallazgos y recomendaciones. Sin dependencia de LLM ni Ollama; un LLM local soberano (`candle`) es realce opcional.

**Componentes asociados:**
- [`robustness-score-aggregator`](./features/robustness-score-aggregator.md) — Motor de consolidación de los 5 scores individuales en el score ponderado final.
- [`robustness-verdict-engine`](./features/robustness-verdict-engine.md) — Motor de veredictos por plantilla determinista (ADR-0115); LLM local opcional, sin Ollama.

**Traducción a Position Sizing:** El módulo de ejecución recibe el score de robustez como parámetro de entrada para los modelos de dimensionamiento de posición. Mayor score implica mayor fracción de riesgo asignable.

**Veto Operativo Pre-Trade por Robustez (Veto Monte Carlo - ADR-0095):**
El motor de ejecución interactúa con el veredicto de robustez mediante un veto proactivo en el `Pre-Trade Validator`:
- **Evaluación en Vivo:** La cuenta o portfolio define la severidad del bloqueo (`HARD_VETO`, `WARNING_ALERT`, `DISABLED`) en el `Design Manifest`.
- **Rastro de Veredicto:** Si una estrategia es catalogada como `PROP_FIRM_FRAGILE` o `TOXIC` por el guantelete Monte Carlo o carece de veredicto, el pre-trade validator bloquea el envío de órdenes si la política es `HARD_VETO`.


### 2.5 Persistencia Soberana "Zero-Docker" (SQLite + Parquet + DuckDB)

Infraestructura de datos embebida y de alto rendimiento que elimina la necesidad de contenedores externos y reduce drásticamente el uso de Memoria de Acceso Aleatorio (RAM).

#### El Databank — Data Lake de R&D (Alta Eficiencia)
El sistema divide radicalmente el entorno de Investigación y Desarrollo (R&D) del entorno de Producción:
*   **Entorno R&D (Efímero):** Las estrategias mutadas (millones) no se guardan como JSON AST masivos. Viven exclusivamente en RAM durante los milisegundos del backtest. Una vez finalizado, se guardan en archivos Parquet (Catálogo y Trades) únicamente las métricas y la "Semilla de ADN" paramétrica de la estrategia.
*   **Entorno Producción (Hall de la Fama):** SQLite se reserva exclusivamente para almacenar estrategias promovidas, que son rehidratadas bajo demanda desde su Semilla de ADN hacia el JSON AST completo mediante el Orquestador Rust. Esto incluye la columna `node_preview_cache` que persiste un JSON blob con las curvas de equidad reducidas y métricas resumidas (Sharpe, Retornos) de las estrategias/nodos del editor visual para un acceso sub-milisegundo.
*   **Embudos con Memoria (Derived Databanks):** El pipeline evolutivo crea catálogos derivados en Parquet sin destruir los anteriores (`Databank_A` → `Databank_B`), posibilitando la trazabilidad masiva vía DuckDB y WebSockets. En consultas analíticas temporales, DuckDB aprovecha la indexación Hive-style del data lake en archivos Parquet particionados para realizar poda de particiones (partition pruning) al vuelo, cargando selectivamente los trades de periodos específicos.

#### Versionado Git-Like de Estrategias (Immutable Strategy IDs)
Para garantizar la reproducibilidad 100% (Bit-to-Bit), el sistema trata las estrategias como repositorios de código inmutables:
*   **Inmutabilidad Post-Creación:** Cada estrategia es inmutable. Cualquier cambio genera una nueva versión con un `version_hash` único.
*   **Encadenamiento (Hash Chain):** Cada versión apunta a su antecesora via `parent_hash`, permitiendo reconstruir el linaje completo.
*   **Ramificación (Branches):** Soporte nativo para ramas (ej: `main`, `risk-opt`) permitiendo experimentación paralela sin duplicación masiva de datos.
*   **Herencia de Resultados (Cumulative Results):** Si una versión nueva no modifica parámetros evaluados en la anterior, los resultados de tests (WFA, Monte Carlo) se heredan automáticamente, ahorrando hasta un 80% en tiempo de validación.
*   **Almacenamiento:** Uso de `strategies.parquet` para el catálogo de versiones y SQLite para el estado operativo activo.

#### Jerarquía de Responsabilidades (Separación de Preocupaciones)
Para garantizar la máxima velocidad en la ruta de ejecución crítica, el sistema divide las tareas de datos de forma estricta:

*   **Configuración Tipada (Serde):** Desvinculación de rutas físicas; toda la estructura de directorios se gestiona desde una configuración central validada en Rust (`DataPathsConfig`).
*   **DuckDB:** Procesamiento Analítico en Línea (OLAP) exclusivo para consultas bajo demanda y remuestreo SQL (Partition Pruning). No interviene en la ruta de ejecución crítica del backtesting.
*   **The Sanitizer:** Pipeline de calidad obligatorio: `Raw Data → Delisted Filter → Corporate Events Adjuster → PIT Validator → Clean Data`.
*   **SQLite WAL:** Gestiona la persistencia de eventos reactivos, el estado actual de la operativa en vivo y el registro del historial de purgas y metadatos de snapshots.
*   **Polars:** Motor encargado de la ingesta y transformación de datos masivos, cargando los resultados directamente en memoria mediante tablas de Apache Arrow.
*   **Gobernanza de Purga (Snapshots):** La eliminación masiva de clústeres tóxicos de estrategias (PCA) se realiza mediante marcado lógico (soft-delete). El backend Rust genera snapshots automáticos del catálogo antes de aplicar cambios estructurales en Parquet/SQLite, garantizando la recuperación atómica (rollback) ante errores del operador.
*   **SQLite:** Gestión de Procesamiento de Transacciones en Línea (OLTP) para el almacenamiento de configuraciones, eventos y el libro mayor de operaciones.
*   **NautilusTrader:** Motor de ejecución que consume datos precargados en memoria; no realiza consultas a bases de datos en tiempo real durante la simulación o el trading en vivo.

#### Flujos de Datos por Caso de Uso

1.  **Ingesta de Datos de Mercado (Pipeline ETL Soberano):**
    *   **Extracción Híbrida:** Priorización de descarga **Bulk** (S3) + **API Delta** (HTTP Nativo). 
    *   **The Sanitizer (Quality Protocol):** Flujo secuencial: `Raw Data → Delisted Filter → Corporate Events Adjuster → PIT Validator → Clean Data`. Incluye detección de gaps e integridad OHLC.
    *   **Transformación y Carga:** Procesamiento **Polars** nativo y escritura en **Hive-Style Parquet**.
    *   **Remuestreo Dinámico:** **DuckDB** agrupa barras crudas (1m/Ticks) en temporalidades personalizadas (ej. 7 min, 21 min) al vuelo via SQL.
    *   **Feedback:** Telemetría en tiempo real hacia la UI mediante Flutter FFI Events.

2.  **Simulación Histórica (Ruta de Rendimiento Crítico):**
    *   Escaneo de archivos Parquet con Polars.
    *   Conversión a formato de memoria **Apache Arrow**.
    *   Reproducción inmediata en el motor de **NautilusTrader** (Replay en memoria).

3.  **Analítica y Remuestreo Bajo Demanda:**
    *   Ejecución de consultas SQL vectorizadas mediante **DuckDB** sobre archivos locales.
    *   Generación de temporalidades personalizadas (ej. velas de 7 minutos) mediante agregación SIMD (Instrucción Única, Múltiples Datos).
    *   Envío de resultados hacia la Interfaz de Usuario.

4.  **Registro Transaccional (Propiedades ACID):**
    *   Inserción de operaciones ejecutadas en **SQLite** utilizando el modo de Escritura Adelantada (WAL).
    *   Mantenimiento de un almacén de eventos inmutable para auditoría.

#### Especificaciones de Almacenamiento y Transporte

*   **Datos de Mercado (Series Temporales):** El histórico de movimientos de precio (Ticks y Velas) se comprime en archivos Parquet estructurados por activo, temporalidad y periodo cronológico.
*   **Estado y Ledger:** Base de datos relacional local gestionada mediante **SQLx** para configuraciones, estrategias promovidas y registros de seguridad cifrados.
*   **Transporte UI ↔ Backend:** Uso de **Apache Arrow** para la transmisión binaria de alta velocidad a través de FFI/gRPC. Se aplica un proceso de reducción de resolución (*downsampling*) en el servidor para permitir la visualización fluida de más de 10,000 puntos de datos en el navegador.

### 2.6 El Orquestador Nativo (Arquitectura Multimodal y FFI)

Provee la infraestructura de comunicación entre la UI en Flutter y los motores de ejecución en Rust. Para garantizar la soberanía, rendimiento y flexibilidad, Drasus Engine implementa una **Arquitectura Multimodal** con las siguientes modalidades configurables:

1. **LocalPowerUser (FFI):** Ejecución nativa por defecto. Flutter UI y Rust Core operan en la misma máquina local compartiendo memoria mediante FFI (`flutter_rust_bridge`). Latencia FFI/gRPC eliminada.
2. **VpsMonolithic:** Ejecución local en un VPS de bajos recursos gráficos vía Escritorio Remoto (RDP). Se activa una variable global para renderizar Flutter en modo Software (sin shaders complejos) preservando los recursos del CPU del VPS.
3. **SaaSCloudEngine (Headless CLI):** Todo el backend de Rust (los 8 módulos: Ingest a Withdraw) se compila como un Daemon CLI que se ejecuta 24/7 en un servidor de alta densidad (VPS o Contenedor Bare-Metal). La UI local en Flutter se conecta a este demonio vía **gRPC / WebSockets**, permitiendo control total y visualización acelerada en GPU desde la laptop del usuario sin interrumpir las operaciones del servidor.
4. **HybridComputeCooperative (Local FFI + Remote Workers):** El backend Rust local corre acoplado al Frontend local vía FFI 100% del tiempo. Mantiene la base de datos transaccional SQLite en local. Sin embargo, delega de forma asíncrona tareas analíticas intensivas (backtesting masivo, optimización) o procesos de ejecución persistentes 24/7 (Execute remoto) a una o varias instancias daemon remotas en VPS vía gRPC/WebSockets, permitiendo la desconexión segura del cliente local sin interrumpir la operación del VPS.


#### Asignación de Puertos (Configuración de Red)
La infraestructura de red se gestiona mediante variables de entorno validadas, permitiendo una configuración flexible y segura del ecosistema local:

| Servicio | Puerto Predeterminado | Protocolo | Variable de Entorno |
| :--- | :--- | :--- | :--- |
| **FFI Bridge** | N/A | Memoria FFI/gRPC | `CORE_CHANNEL` |
| **Flutter FFI Events (Progreso)** | N/A | FFI/gRPC | - |
| **gRPC / WebSockets** | 50051 | TCP | `LIVE_EVENT_CHANNEL` |
| **Shadow Watchdog** | N/A | Daemon Nativo | `WATCHDOG_DAEMON` |

#### Flujo de Responsabilidades y Ejecución
Para garantizar que la interfaz de usuario mantenga una fluidez constante, el sistema aplica una separación estricta entre las tareas de Entrada/Salida (I/O) y las de procesamiento intensivo (CPU):

*   **Validación Zero-Trust:** El Árbol de Sintaxis Abstracta (AST) enviado desde el Frontend en formato JSON es validado mediante esquemas Serde antes de generar un identificador de ejecución único (`run_id`).
*   **Despacho de Tareas:** El Core en Rust actúa exclusivamente como orquestador. Las simulaciones y entrenamientos se delegan a **Workers** nativos (hilos Tokio o Rayon) mediante memoria compartida, eliminando el costo de serialización de grandes volúmenes de datos.
*   **Ciclo de Vida del Trabajador (Worker):**
    1.  **Hidratación:** El trabajador reconstruye el objeto de la estrategia y sus parámetros a partir del JSON validado.
    2.  **Compilación Nativa:** Los algoritmos se ejecutan como código nativo Rust para su ejecución ultra-rápida.
    3.  **Ejecución (Institutional Protocol):** Inicio del motor de simulación de NautilusTrader siguiendo el ciclo síncrono: `Ingesta → Warm-up → Indicadores → Sincronización (Bar Open) → Señal → Risk Check → Order Server → Matching Engine → Account/Ledger Update`.
    4.  **Persistencia:** Almacenamiento de resultados en archivos Parquet particionados.
    5.  **Emisión:** Comunicación de señales de progreso hacia Flutter vía FFI o flujos gRPC.
*   **Optimización de Ancho de Banda (Throttling):** Los trabajadores agrupan las actualizaciones de estado y emiten señales de progreso cada 100 milisegundos, evitando la saturación del tráfico de datos y garantizando la estabilidad de la interfaz gráfica.

### 2.7 El Patrón Todo en Uno (Rust + Flutter FFI)

Drasus Engine se distribuye como una aplicación nativa empaquetada. Al ser compilado en Rust/Dart, no requiere runtimes adicionales (Node, Python).

*   **Empaquetado Completo:** El código fuente (Rust) y el motor UI (Flutter) se compilan en un único ejecutable.
*   **Resolución de Dependencias Invisible:**
    * **Windows:** Creación de un `.exe` con instalador (*Inno Setup* o *NSIS*).
    * **macOS:** Creación de archivo `.app` o `.dmg` firmado para Apple Developer Program.
    * **Linux:** Creación de un `AppImage` que se ejecuta en cualquier distribución sin instalar nada.
*   **Interfaz Premium Nativa (Impeller):**
    * **Frameless:** Sin bordes y con barra superior personalizada.
    * **Modo Oscuro Nativo:** Sincronización del tema del dashboard con el del sistema operativo.
    * **Local API Port Selection:** Búsqueda dinámica de un puerto local libre para evitar colisiones.
*   **Detección Dinámica de Hardware:** Durante el arranque, el sistema identifica la disponibilidad de tarjetas gráficas NVIDIA. 
    *   **Con GPU:** Habilita rutas opcionales aceleradas vía `candle` (CUDA/Metal dinámico, ADR-0112); nunca obligatorio.
    *   **Sin GPU:** Realiza un respaldo automático (*Fallback*) hacia la ejecución multi-hilo en el procesador central (CPU).

### 2.8 El Compilador de Nodos (Puente Interfaz → Motor)

Este componente traduce la lógica visual del usuario en instrucciones de ejecución de alto rendimiento.

1.  **Exportación de Estrategia:** El lienzo de diseño en Flutter convierte el flujo visual en un **Árbol de Sintaxis Abstracta** (Abstract Syntax Tree) en formato JSON/Protobuf.
2.  **Validación Institucional:** El Orquestador valida la estructura y tipos del payload mediante esquemas estrictos de **Serde** (Rust).
3.  **Patrón Factory (Fábrica):** El sistema orquesta "Bloques Pre-Compilados". La lógica central reside en código estático de NautilusTrader o indicadores compilados previamente (AOT - *Ahead-Of-Time*). El Grafo Dirigido Acíclico (DAG) visual solo define las conexiones entre estos bloques para evitar la fragilidad de compilar todo el sistema en tiempo real.
4.  **Nodos de Inyección Directa (Escape-Hatch):** Para lógica personalizada que no existe en los bloques base, el sistema permite integrar código Rust nativo directamente en el bucle de eventos del motor.

### 2.9 Infraestructura de Datos y Comunicación entre Procesos (FFI/gRPC)

#### Capa de Datos (Local-First "Zero-Docker")
| Capa | Tecnología | Función |
| :--- | :--- | :--- |
| **Transaccional (OLTP)** | SQLite (Modo WAL) | Almacena el estado de la aplicación, configuraciones, bitácora de órdenes y el **Live Ledger de Trades** (ACID). |
| **Analítica (OLAP)** | Archivos Parquet | Series de tiempo comprimidas y particionadas bajo el estándar **Hive-Style** (`year=Y/month=M/`). Almacena históricos y resultados de investigación. |
| **Consulta/Resample** | DuckDB | Ejecuta consultas SQL analíticas y **remuestreo dinámico** (ej. 7m, 21m) sobre Parquet sin redundancia de archivos. |
| **Procesamiento ETL**| Polars (Rust Native) | Motor principal para transformación masiva y el pipeline **The Sanitizer** (Quality Protocol). |
| **Transporte Binario** | Apache Arrow | Serialización ultrarrápida de datos hacia el frontend vía FFI/gRPC (Arrow JS). |

#### Estrategia de Comunicación (FFI/gRPC)
*   **Rust ↔ Workers:** Las tareas pesadas de simulación se gestionan mediante un thread pool (Tokio/Rayon). Se utiliza **memoria compartida** (`shared_memory`) para evitar la sobrecarga de serializar grandes conjuntos de datos (*DataFrames*).
*   **Rust ↔ Flutter:** Comunicación nativa vía FFI (Foreign Function Interface) para cero latencia. Para despliegues Headless (Modo VPS), se activa el canal gRPC/WebSockets para telemetría.
*   **Escalabilidad Futura:** El diseño es agnóstico al gestor de tareas; la arquitectura permite migrar hacia infraestructuras de nube distribuidas (Orquestación Rust) simplemente modificando la capa de despacho.

### 2.10 Protocolo de Acceso Remoto de Portafolio (RPAP)

Drasus Engine habilita el trabajo colaborativo multi-tenant sin comprometer la Propiedad Intelectual mediante una arquitectura P2P de consulta descentralizada.

*   **Seguridad a Nivel Campo (Field Masking):** El clúster maestro expone un servidor API (gRPC) que permite a instancias de empleados realizar consultas analíticas de manera segura. El motor de masking proyecta exclusivamente campos operacionales (precios, pnl, timestamps) y omite permanentemente los Árboles de Sintaxis Abstracta (AST) y parámetros internos.
*   **Evaluación Computacional Descentralizada:** El cálculo pesado de validación pre-absorción se transfiere a la máquina del empleado. El empleado ejecuta su simulación local, consume datos del maestro a través del RPAP y evalúa el impacto cruzado (Drawdown Alignment, Correlación) antes de someter su estrategia a la revisión del maestro.
*   **Soberanía Administrativa:** Todo acceso remoto se rige por la persistencia soberana; regulado mediante tokens JWT con expiración definida, asignación estricta de scopes operacionales, y un registro inmutable en el log de auditoría local (SQLite) del clúster maestro.

### 2.11 Motor de Copy-Trading (Signal Relay y Risk-Scaled Execution)

Drasus Engine habilita la distribución segura y de baja latencia de ejecución a múltiples terminales clientes mediante una topología basada en un relevo intermedio de señales cifradas:

*   **Arquitectura de Relevo Ciego (Zero-Knowledge Relay):** El motor descarta conexiones entrantes directas al Master. El Master abre un flujo saliente único (WebSocket o gRPC con TLS) hacia un servidor intermedio (Signal Relay). Dicho servidor redistribuye los paquetes de datos a los clientes (Copiers) autorizados sin capacidad de descifrar la carga útil (Zero-Knowledge), protegiendo la propiedad intelectual del Master.
*   **Seguridad y Cifrado Extremo a Extremo (E2E):** Cada señal emitida por el Master es comprimida y cifrada en origen mediante el estándar AES-256-GCM y firmada mediante HMAC-SHA256 con claves simétricas de sesión. La verificación y descifrado ocurren de forma exclusiva y local en la máquina de cada Copier.
*   **Algoritmo de Escalado de Riesgo Local:** Cada Copier procesa localmente el escalado de tamaño de orden basándose en el capital relativo (Capital Copier / Capital Master), la volatilidad realizada del instrumento (ATR) y límites dinámicos de riesgo (Drawdown máximo y porcentaje de riesgo por operación), reduciendo el tamaño si el riesgo en dólares excede el límite del Copier.

---

## 3. Decisiones Técnicas Clave (ADRs)

> **Nota:** Esta tabla es un resumen curado de los ADRs más representativos para la comprensión arquitectónica general; no es exhaustiva. El registro completo y vigente de todas las Decisiones de Arquitectura (incluyendo ADR-0014 a ADR-0106) es [`ADR.md`](./ADR.md).

| ID | Decisión | Propósito |
|---|---|---|
| **ADR-0001** | Un solo binario con módulos independientes + separación lógica pura vs. interacción | Evitar latencia de red de microservicios; mantener testabilidad de funciones sin efectos secundarios. |
| **ADR-0002** | Estructuras de datos puras + librerías de procesamiento vectorial | Desacoplar lógica de base de datos; permitir acceso a memoria compartida sin copias (zero-copy) y cálculos en paralelo en CPU con compilación JIT. |
| **ADR-0003** | Estructura de Carpetas (separación clara lógica pura / interacción) | Forzar aislamiento físico entre lógica pura e infraestructura; escalabilidad infinita sin archivos gigantes. |
| **ADR-0004** | Máquina de estados con números enteros (int64) | Garantizar que dos ejecuciones del mismo cálculo den idéntico resultado numérico; permitir optimización automática en cambios de estado. |
| **ADR-0005** | Versionado reproducible con historial completo en grafo | Reproducibilidad 100%, auditoría completa, pruebas A/B en vivo. |
| **ADR-0006** | Control de cambios de esquema centralizado | Una única fuente de verdad para cambios en tablas; reversión atómica de cambios; auditoría de quién cambió qué. |
| **ADR-0007** | Inyección dinámica de comportamiento (Feature Router) | Permitir coexistencia de variantes de features sin hardcoding; activar/desactivar por configuración. |
| **ADR-0008** | Todo es configurable excepto las reglas invariables | Cada usuario/equipo puede ajustar parámetros; solo las restricciones arquitectónicas son fijas. |
| **ADR-0009** | Interfaz Unificada Strategy-Portfolio (ExecutableContainer) | Strategy y Portfolio comparten contrato idéntico; módulos operan con lógica única, sin duplicación. |
| **ADR-0010** | Reglas Dinámicas (Hard Limits vs Soft Alerts) | Autonomía operativa: hard limits ejecutan automáticamente, soft alerts notifican. Usuario es autoridad final, no bloqueador. |
| **ADR-0011** | Operaciones Asincrónicas (Async Job Pattern) | Operaciones costosas se ejecutan en background con patrón Disparo→Monitoreo→Recuperación. Interfaz no se bloquea. |
| **ADR-0012** | Arquitectura Multi-Pipeline Paralela | N pipelines simultáneamente con recursos reservados para live trading. Usuario configura % CPU/RAM para exploración. |
| **ADR-0013** | Selección de Stack Tecnológico | Rust, SQLite+WAL, Polars/DuckDB, Tokio, Flutter (UI), flutter_rust_bridge (FFI). |
| **ADR-0029** | Patrón Todo en Uno (Rust + Flutter FFI) | Aplicación compilada nativamente como un único ejecutable, eliminando latencia FFI/gRPC/DOM. |
| **ADR-0086** | Minería Descentralizada (La Colmena) | Arquitectura cliente-servidor para crowdsourcing de cómputo GPU/CPU de exploración de estrategias. |
| **ADR-0087** | El Guardián & El Centinela | Validador de riesgo pre-trade global (<1ms) y Shadow Watchdog e interruptor automático de emergencia en Rust. |
| **ADR-0088** | Protocolo de Incubación & Cono de Silencio | Cuarentena acelerada de 7 días con Eutanasia Predictiva (MAE), bandas de confianza Monte Carlo y Broken Strategy Flag. |
| **ADR-0089** | Optimización & Rebalanceo de Portfolio | Asignación adaptativa HRP/Markowitz, Hedging Cointegrativo Tick-by-Tick (+0.85), Router de Liquidez y Auto-Rebalancing Daemon. |


### 3.1 Patrones Descubiertos (Insights de Arquitectura)

1. **Hexagonal > Performance:** Si una optimización de velocidad (hot path) rompe la modularidad o el desacoplamiento, se rechaza. La modularidad es la base de la longevidad del sistema.
2. **Clarificación Previa a Abstracción:** Antes de proponer una nueva interfaz (ABC/Protocol), se debe clarificar el problema de negocio. La mayoría de las "necesidades" de auditoría se resuelven con `test_results` (inmutable) + `live_results` (runtime), sin módulos extra.
3. **Versión Inmutable + Rama:** Las estrategias/portafolios se tratan como repositorios Git. Todo cambio es una nueva versión; los resultados de pruebas son acumulativos y heredables.
4. **Soberanía "Zero-Docker":** El sistema debe ser funcional localmente sin dependencias de red pesadas. SQLite WAL + Parquet es el estándar de oro para el Client Zero.
5. **IDs Mandatorios para Trazabilidad:** Todo (Módulo, Tarea, Feature) tiene un ID único (MOD-X, TASK-X, FEAT-X) para evitar redundancia y asegurar coherencia en auditorías forenses.

---

## 4. Vistas del Sistema (Modelo C4)

### 4.1 Nivel 1: Contexto
```
    ┌───────────────────────────┐
    │       Flutter UI          │
    │ (Dart + Impeller Engine)  │
    └────────┬──────────────────┘
             │ (Local: FFI / Remoto: gRPC)
    ┌────────▼──────────────────┐      ┌─────────────────────────┐
    │   Drasus Engine Backend   │◄────►│       Brokers           │
    │        (Rust Core)        │ API/ │  (Binance, Interactive  │
    │   [broker-connector]      │  WS  │   Brokers, etc.)        │
    └────────┬──────────────────┘      └─────────────────────────┘
             │
    ┌────────▼──────────────────┐
    │      SQLite Local         │
    │   (Historial, States)     │
    └───────────────────────────┘
```

### 4.2 Nivel 2: Contenedores (8 Módulos de Pipeline + Features Reutilizables)

El sistema sigue un pipeline claro: **Ingestar → Generar → Validar → Incubar → Gestionar → Ejecutar → Retroalimentar → Retirar**.

**Estructura de Carpetas:**
```
Directorio raíz del proyecto
├── Archivo principal (Orquestación de módulos)
├── Carpeta shared (Features reutilizables: telemetría, tipos, utilidades)
│   ├── telemetría/ (Registro estructurado, métricas)
│   ├── tipos/ (Estados máquina de 64 bits, Enumeraciones, tipos base)
│   └── utilidades/ (Conversión de datos, serialización, ayudas de tiempo)
├── Carpeta modules (8 módulos con separación clara)
│   ├── ingest/ (Separación clara: API pública, lógica pura, orquestación, acceso datos, modelos DB, esquemas)
│   ├── generate/
│   ├── validate/
│   ├── incubate/
│   ├── manage/
│   ├── execute/
│   ├── withdraw/
│   └── feedback/
└── Carpeta infrastructure
    ├── Configuración base de datos (Mapeo de objetos a SQL + SQLite)
    └── Bus de eventos (Colas asincrónicas para comunicación entre módulos)

Carpeta migraciones (Control de cambios de esquema centralizado)
Carpeta tests (Pruebas unitarias, integración, simulación histórica)
```

**Arquitectura de Módulos (Cada módulo contiene):**
```
Carpeta módulo/
├── mod.rs
├── public_interface.rs  <-- [SHELL] Única entrada que otros módulos ven (API Interna)
├── domain/              <-- [CORE] Lógica pura (Business Logic), sin efectos secundarios
│   └── logic.rs
├── orchestrator.rs      <-- [SHELL] Manejo de flujo, estados, ruteo de eventos
├── persistence/         <-- [SHELL] Acceso a datos (solo tablas del módulo)
│   ├── models.rs        <-- Esquema de base de datos relacional (tablas locales)
│   └── repository.rs    <-- Consultas y conversión Core <-> DB
└── schemas.rs           <-- Modelos de datos (Estructuras / Contratos)
```

**Árbol Visual del Sistema C4 Nivel 2:**
```
┌────────────────────────────────────────────────────────────────────────────┐
│                    Archivo principal: Orquestación                          │
└────────────────────────────────────────────────────────────────────────────┘
       │
       ├─► Carpeta shared (Features Reutilizables)
       │    ├── Telemetría/ (Registro, métricas)
       │    ├── Tipos/ (Máquina de estados 64-bit, Enumeraciones)
       │    └── Utilidades/ (Conversión de datos, serialización)
       │
       ├─► Carpeta módulo-ingest
       │    ├── API pública: Ingesta de barras, obtener régimen de mercado
       │    ├── Lógica pura: Parsing de precios, detección de anomalías
       │    ├── Orquestación: Manejo gRPC/WebSocket, normalización
       │    ├── Acceso datos: Persistencia de barras, detección de régimen
       │    └── Modelos DB: Tablas barras, histórico régimen
       │
       ├─► Carpeta módulo-generate
       │    ├── API pública: Generar candidatos, evaluar aptitud
       │    ├── Lógica pura: Evolución genética, regresión simbólica
       │    ├── Orquestación: Bucle evolutivo, combinación de señales
       │    ├── Acceso datos: Persistencia estrategias, análisis de factores
       │    └── Modelos DB: Tablas planos estrategia, candidatos
       │
       ├─► Carpeta módulo-validate
       │    ├── API pública: Validar estrategia, suite de pruebas
       │    ├── Lógica pura: Análisis walk-forward, Monte Carlo, pruebas de coherencia
       │    ├── Orquestación: Orquestación backtesting, cálculo métricas
       │    ├── Acceso datos: Motor pruebas, resultados validación
       │    └── Modelos DB: Tablas resultados pruebas, métricas
       │
       ├─► Carpeta módulo-incubate
       │    ├── API pública: Ejecución paper trading, comparación con backtest
       │    ├── Lógica pura: Validación Pardo
       │    ├── Orquestación: Simulación de ejecuciones, detección cambios
       │    ├── Acceso datos: Persistencia paper trading
       │    └── Modelos DB: Tablas sesiones, resultados comparación
       │
       ├─► Carpeta módulo-manage
       │    ├── API pública: Optimizar portafolio, establecer reglas, backtesting de portafolio HRP
       │    ├── Lógica pura: Optimización portafolio (HRP), correlaciones, rebalanceo Walk-Forward
       │    ├── Orquestación: Rebalanceo, cálculo correlaciones
       │    ├── Acceso datos: Persistencia portafolio, estrategias
       │    └── Modelos DB: Tablas portafolios, pesos, reglas
       │
       ├─► Carpeta módulo-execute
       │    ├── API pública: Colocar orden, cancelar orden, veto
       │    ├── Lógica pura: Cambios de estado orden (máquina 64-bit)
       │    ├── Orquestación: Conexión broker, 10 validaciones pre-comercio (ADR-0025)
       │    ├── Acceso datos: Persistencia órdenes, posiciones
       │    └── Modelos DB: Tablas órdenes, ejecuciones, eventos supervisión
       │
       ├─► Carpeta módulo-feedback
       │    ├── API pública: Control de Calidad Estadístico (Pardo), Veredicto de salud
       │    ├── Lógica pura: Detección de Drift (Real vs Esperado)
       │    ├── Orquestación: Cierre de ciclo de vida (Veredicto de retiro)
       │    ├── Acceso datos: Historial de veredictos, constraints de aprendizaje
       │    └── Modelos DB: Tablas anomalías, sugerencias, veredictos
       │
       ├─► Carpeta módulo-withdraw
       │    ├── API pública: Detectar degradación, retiro estrategia
       │    ├── Lógica pura: Comparación de perfiles de rendimiento
       │    ├── Orquestación: Flujo retiro controlado, gestión de veto
       │    ├── Acceso datos: Persistencia de estrategias archivadas
       │    └── Modelos DB: Tablas registro retiro, estrategias archivadas
       │
       └─► Carpeta infrastructure
            ├── Configuración base de datos: Mapeo de objetos + SQLite
            └── Bus de eventos: Colas asincrónicas inter-módulos
```

---

## 5. Requisitos No-Funcionales (8 Leyes de Drasus Engine)

Drasus Engine adhiere a 8 leyes fundamentales que garantizan su rigor científico y operativo:

1. **Event-Driven:** Operación sobre flujos de eventos tipados (NautilusTrader).
2. **Deterministic Replay:** Backtests reproducibles bit-a-bit (Seeds PRNG documentados).
3. **Fail-Safe by Default:** Circuit breakers, kill switches y límites de riesgo integrados en el core.
4. **High-Performance FFI/gRPC:** Uso de Apache Arrow y Polars para zero-copy entre módulos.
5. **CPU-Centric Efficiency:** Rust Native y Polars lazy evaluation; GPU para cómputo masivo.
6. **Zero-Trust Validation:** Esquemas Serde estrictos en todas las fronteras de módulos.
7. **Absolute Parameterization:** Cero hardcoding; 100% configurable dinámicamente.
8. **Data Sovereignty:** Arquitectura Local-First; soberanía total de datos y capital.

### 5.1 KPIs de Rendimiento y Escalabilidad

| Métrica | Target | Justificación |
|---------|--------|---|
| **Backtest Throughput** | Más rápido que MT5/SQX/QuantConnect en igual hardware (ADR-0114; sin KPI absoluto) | Exploración masiva de Alpha. |
| **Live Order Latency** | ≤100ms (end-to-end) | Ejecución competitiva institucional. |
| **Monte Carlo (CPU)** | 10K iteraciones en tiempo acotado vía `ndarray`/Rayon | CPU-first; GPU `candle` opcional (ADR-0061/0112). |
| **Watchdog Kill** | ≤5s detección | Supervivencia ante fallos sistémicos. |
| **Data Size Support** | 100GB+ | Gestión eficiente mediante DuckDB/Parquet. |

---

## 6. Experiencia de Usuario y Flujos (ZUI Fractal)

El sistema se organiza en una interfaz zoomable (ZUI) de 3 niveles de profundidad:

1. **Nivel 1: Fleet Command (Visión Ejecutiva - Macro):** Monitoreo de portafolios activos en un lienzo infinito. Integra agregación vectorial de curvas de balance en tiempo real, matriz de correlación dinámica Pearson calculada mediante DuckDB (alerta ámbar si Pearson > 0.85), e inspección contextual macro (Max Drawdown global y distribución de margen).
2. **Nivel 2: Orchestrator (Visual Editor DAG - Meso):** Editor visual de nodos conectables en Flutter CustomPainter. Implementa layout automático con algoritmos jerárquicos (Dagre), validación estricta de aciclicidad (DAG) en backend Rust mediante `petgraph`, y bus de eventos Pub/Sub visual con pulsos de luz en cables y nodos de suscripción inalámbricos.
3. **Nivel 3: Strategy Inspector (Micro):** Inspección de estrategias individuales con gráficos interactivos nativos Flutter CustomPainter/Impeller de alta frecuencia (downsampling LTTB), visualización de genoma (AST), cono de confianza Monte Carlo y editor de código embebido nativo Flutter para inyección de código (Escape Hatch 90/10) evaluado en motor nativo de scripting en Rust (Rhai).

### 6.1 El "Happy Path" (Máxima Confianza)
`Fleet Status` (Detección oportunidad) → `Orquestador` (Ejecución Generate/Validate) → `Strategy Inspector` (Inspección robustez) → `Deploy` (Incubación; perfil configurable: Cuarentena 7 días / Extendido 21 días / Legacy 3-6 meses — ADR-0088) → `Live Trading`.

---

### 6.2 Puertos Públicos de Módulos

| Puerto | Módulo | Propósito | Operaciones |
|---|---|---|---|
| **DataPort** | ingest | Ingesta y consulta de datos de mercado | Descargar barras históricas, consultar régimen actual, suscribirse a datos en tiempo real |
| **StrategyPort** | generate | Generación y catálogo de estrategias | Generar candidatos vía NSGA-II, crear ramas de experimentación, listar estrategias |
| **ValidationPort** | validate | Pruebas estadísticas rigurosas | Ejecutar backtest, suite de validación (WFA/MC/CPCV), obtener veredicto final |
| **PaperTradingPort** | incubate | Ejecución forward sin dinero real | Iniciar sesión de paper trading, comparar con backtest (Pardo), obtener métricas |
| **PortfolioPort** | manage | Gestión de portafolios versionados | Crear portafolio, optimizar pesos (HRP), definir reglas, rebalanceo Walk-Forward, backtesting de portafolio |
| **ExecutionPort** | execute | Ejecución real y Veto Power | Colocar órdenes, cancelar órdenes, vetar decisiones, activar kill switch |
| **RetirementPort** | withdraw | Monitoreo de degradación | Evaluar salud de estrategia, retirar controladamente, reactivar |
| **FeedbackPort** | feedback | Control de Calidad y Cierre de Bucle | Detectar drift (Pardo), vetar estrategias degradadas, disparar refinamiento |

**Características Clave de los Puertos:**
- **Asincronía Transparente:** Las operaciones costosas (generación, backtests) retornan un `job_id` inmediatamente. El cliente monitorea el progreso consultando el puerto repetidamente o escuchando eventos de finalización.
- **Determinismo Garantizado:** Dentro de un Puerto, mismo input → mismo output, siempre. (Excepto operaciones que explícitamente muestrean aleatoriedad, donde la semilla es configurable.)
- **Sin Efectos Secundarios en Core:** Los Puertos pueden loguear, persistir, notificar. El Core no.
- **Inyectables para Testing:** Los Puertos pueden reemplazarse con implementaciones fake en tests. Ej: `BacktestEngine` puede ser un `FakeBacktestEngine` que devuelve resultados precalculados.

**Pipeline de Ejemplo: Flujo Completo a través de Puertos**
```
Cliente (FFI/gRPC)
    │
    ├─► DataPort.ingest(symbol="BTC", start=2020-01-01) 
    │      → "job_id=12345" (asincrónico)
    │
    ├─► StrategyPort.generate(method=NSGA2, pop_size=100)
    │      → "job_id=12346" (asincrónico)
    │
    ├─► ValidationPort.run_validation(strategy_id="S1", tests=[WFA, MC, CPCV])
    │      → "job_id=12347" (asincrónico)
    │      → Cuando finaliza: ValidationPort.get_verdict(strategy_id="S1") → APROBADA
    │
    ├─► PaperTradingPort.start_session(strategy_id="S1", profile=EXTENDED)   # perfil de incubación configurable (ADR-0088)
    │      → "session_id=sess_001" (sesión iniciada)
    │
    ├─► ExecutionPort.start_execution(portfolio_id="P1", broker="ibkr")
    │      → "execution_id=exec_001" (trading en vivo iniciado)
    │
    ├─► ExecutionPort.get_pending_decisions(execution_id="exec_001")
    │      → Lista de órdenes pendientes de veto del usuario
    │
    └─► RetirementPort.retire_strategy(strategy_id="S1", reason="drawdown > -40%")
           → Estrategia marcada RETIRADA en DAG de versiones
```
### 6.3 El Motor de Producción: Nautilus LiveNode

#### 6.3.1 Procesos Persistentes (Daemons)
En R&D se usan Workers efímeros que nacen, calculan y mueren. En Producción se necesitan **Procesos Persistentes (Daemons)**. El Core en Rust orquesta un hilo en segundo plano (Tokio task) dedicado exclusivamente a la ejecución en vivo y paper trading, inicializando el componente **LiveNode** de NautilusTrader.

- **Aislamiento de Entorno:** El proceso en vivo corre en su propio núcleo lógico mediante afinidad de CPU (*Core Pinning*), totalmente aislado de los Workers de R&D. Si se lanza una optimización genética masiva que consume el 99% del hardware, el sistema operativo garantiza que el núcleo reservado para el LiveNode mantenga latencia de microsegundos para ejecutar órdenes reales.
- **Componentes del LiveNode:** Conectividad nativa con brokers (Binance, IBKR, Oanda), loop de eventos determinista (Local-First) y gestión de órdenes mediante el FSM operativo descrito en la sección 12.
- **Reconstrucción de Inventario:** El Event Store (persistencia local en modo WAL) registra cada evento de ejecución, permitiendo reconstruir el estado del inventario tras un reinicio o caída del proceso.

#### 6.3.2 Multiplexación de Datos (El Bus Pub/Sub)
Si se abren 50 conexiones individuales con un mercado para 50 agentes en el mismo símbolo, la IP se banea instantáneamente. La solución:

1. **Conexión Única (Single Data Client):** El LiveNode levanta un solo cliente de datos hacia el mercado por símbolo.
2. **El Bus de Mensajes (Message Bus):** Los ticks y actualizaciones del Order Book llegan al Message Bus central de NautilusTrader, de altísimo rendimiento en memoria RAM (Cero-Copias).
3. **Suscripción de Agentes:** Cuando se "promueve" una estrategia a Producción, el motor lee su configuración inmutable (AST) y suscribe el agente pasivamente al bus.
4. **Distribución (Fan-out):** Un solo evento del mercado se distribuye por referencia en memoria a los 50 agentes en nanosegundos. Cero duplicación de red, cero clonación innecesaria de objetos.


**Pipeline de Ejecución (Happy Path):**
```
Datos del Mercado
    ↓
Módulo ingest: Ingesta barras ──► Guardar precios y régimen de mercado
    ↓
Módulo generate: Generar candidatos ──► Crear planos de estrategia (descubrimiento)
    ↓
Módulo validate: Validar estrategia ──► Suite completa de pruebas → APROBADA
    ↓
Módulo incubate: Ejecución paper trading ──► Test forward (perfil configurable: 7/21 días o 3-6 meses, ADR-0088) → PROMOVIDA
    ↓
Módulo manage: Optimizar portafolio ──► Combinar estrategias, establecer reglas
    ↓
Módulo execute: Colocar orden ──► Ejecutar en mercado vivo [validación <1ms; orden end-to-end ≤100ms]
    ↓
Módulo feedback: Veredicto Pardo ──► Control de calidad estadístico, decisión de retiro
    ↓
Módulo withdraw: Retiro controlado ──► Archivo definitivo, fin de ciclo ──► [Volver a Generar via Feedback]
```

### 6.4 El Frontend: Paradigma de Interfaz de Usuario (UI/UX) - Dashboards y Visualización

La interfaz gráfica de Drasus Engine se diseña sobre un paradigma responsivo y de ultra-bajo retardo visual:

*   **Visualización High Precision (Impeller nativo):** Renderizado en GPU de alto rendimiento para interactuar con cientos de miles de puntos de datos sin congelamientos de UI. El lienzo se reserva para la representación de la topología del grafo.
*   **Micro-Backtest Node Preview:** Visualizador integrado en los nodos del Strategy Inspector del Nivel 3. Permite la visualización de curvas de equidad reducidas y métricas clave precargadas desde SQLite local, con invalidación visual y regeneración asíncrona manual ante la edición de parámetros.
*   **Time-Warp UI:** Selector de rango temporal y slider interactivo para navegación forense rápida. Realiza consultas optimizadas con DuckDB mediante partition pruning sobre el data lake en Parquet para minimizar la latencia de carga (<200ms) y la carga de memoria RAM en el frontend.
*   **UMAP Scatter Visualizer:** Scatter plot 2D/3D interactivo en Flutter nativo (CustomPainter/GPU) para identificar clústeres de robustez mediante reducción de dimensionalidad UMAP, con soporte de brushing (lasso) para drill-down directo de estrategias.
*   **Toxicity Purifier UI:** Panel interactivo para la purga masiva de clústeres de estrategias tóxicas detectadas por PCA. Permite previsualizar el impacto, realizar soft-delete por lote con firma en el log de auditoría y rollbacks mediante snapshots automatizados.
*   **Efficiency & Incubation Dashboard:** Monitoreo y control del Cono de Silencio y métricas clave de incubación.
*   **Data Manager UI:** Interfaz que incorpora el asistente de importación (Import Wizard) y el mapa cromático de calidad de datos.
*   **Throttling Metrics Dashboard:** Monitoreo en tiempo real de la latencia inducida por colas de brokers y conectores.
*   **Gráficos en PDF:** Motor de renderizado en el backend (headless) para la generación de reportes de auditoría y análisis inmutables.
*   **Monthly Performance Heatmap:** Matriz visual interactiva de Años × Meses segmentable por dirección (Long/Short) y tipo de muestra (IS/OOS).
*   **Trade Analysis BI Suite:** Dashboard de análisis de transacciones históricas (cascada P/L, Wins/Losses semanales y correlación duración vs rentabilidad).
*   **Strategy Config Diff:** Visor de diferencias entre la configuración del último test válido y la actual.
*   **AI Experience:** Incorporación de asistentes contextuales (Interactive Chat Loop, Hybrid Prompting UI, Natural Language Explanation) y auditorías regulatorias (Compliance Dashboard).
*   **Workflow Configurators:** Diseñador visual de pipelines automatizados (Visual Workflow Builder) y selectores dinámicos de universos accionarios (Visual StockPicker Configurator).

---

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

## 8. Arquitectura de Datos

### Flujo de Datos con Features Reutilizables (Happy Path Completo - 8 Módulos)

```
FLUJO DE EJECUCIÓN DURANTE EL DÍA:

1. ingest: Ingesta de barras de mercado
   ├─► Features consumidas: [data-validator], [pit-data-validator], [hmm-regime-detection], [audit-log]
   ├─► Lógica pura: Normalizar precios
   ├─► Validación: PIT-real (sin look-ahead bias)
   ├─► Acceso datos: Guardar barra en base de datos
   └─► Detección régimen: Identificar estado del mercado (TRENDING, CHOPPY, etc.)

2. generate: Generar candidatos [Proceso batch offline]
   ├─► Features consumidas: [nsga2-optimizer], [hmm-regime-detection], [zero-crossing-filter], [strategy-ensemble], [audit-log]
   ├─► Lógica pura: Evolución multi-objetivo NSGA-II
   ├─► Lógica pura: Descubrimiento de ecuaciones por regresión simbólica nativa (modo NSGA-II, ADR-0113)
   ├─► Lógica pura: Filtrado ortogonal de señales (independent de factores)
   ├─► Síntesis: Ensemble (NSGA + simbólica nativa + HMM) → Estrategias híbridas
   └─► Acceso datos: Guardar candidatos → estado = Pendiente

3. validate: Validar estrategia [Proceso offline, reproducible]
   ├─► Features consumidas: [pit-data-validator], [backtest-engine], [walk-forward-analyzer], [factor-decomposition], [alpha-purity-analyzer], [zero-crossing-filter], [signal-correlation-analyzer], [equity-curve-tracker], [slippage-models], [institutional-metrics], [audit-log], [pca-toxicity-analyzer], [autoencoder-outlier-detector]

   ├─► Validación PIT: Asegurar datos sin look-ahead
   ├─► Backtesting: Ejecución histórica con slippage realista
   ├─► Lógica pura: Análisis walk-forward (robustez en ventanas)
   ├─► Análisis alpha: Descomposición FF5 (habilidad vs factor luck)
   ├─► Análisis señales: Correlaciones, ortogonalidad, diversificación
   ├─► Tracking: Equity curve para Sharpe, Max DD, ratios
   └─► Acceso datos: Guardar análisis → resultado = APROBADO/REVISAR/RECHAZADO

4. incubate: Ejecución paper trading [Quarantine 7 Días, Extended 21 Días o Legacy 3-6 meses — ADR-0088]
   ├─► Features consumidas: [paper-trader], [incubation-manager], [backtest-engine], [slippage-models], [equity-curve-tracker], [institutional-metrics], [trade-reconciler], [order-fsm], [audit-log]
   ├─► Simulación forward: Trading simulado con spreads reales (Paper Trading tradicional o Cuarentena Acelerada de 7 Días con Eutanasia Predictiva por MAE flotante).
   ├─► Cono de Silencio: Proyección de bandas de confianza Monte Carlo (1, 2 y 3 sigmas) para auditar la equidad en caliente.
   ├─► Broken Strategy Flag: Kill Switch automático (pausa, liquidación y reasignación) si la equidad cruza el límite inferior de -1 sigma.
   ├─► Tracking & Eficiencias: Medición de Return Efficiency y Drawdown Efficiency vs backtest.
   └─► Acceso datos: Decidir promoción → promovido a vivo = sí


5. manage: Optimizar y Rebalancear Portafolio [Asignación Adaptativa y Daemon de Rebalanceo]
   ├─► Features consumidas: [portfolio-optimizer], [portfolio-rules], [federated-portfolio], [portfolio-backtest], [signal-correlation-analyzer], [factor-decomposition], [equity-curve-tracker], [institutional-metrics], [hmm-regime-detection], [audit-log]
   ├─► Arquitectura de Contenedores Federados: Aislamiento lógico de reglas y gobernanza autónoma individual de múltiples subportafolios dentro del ecosistema unificado.
   ├─► Simulación de Portafolio Real (Real Portfolio Backtest): Simulación concurrente de múltiples estrategias compartiendo capital de margen, compounding configurable y sincronización de sesiones de mercado.
   ├─► Motores de Pesaje: Asignación clásica (Markowitz, HRP, Equal Weighting, Minimum Variance) y Ensamblador D-Score.
   ├─► Volatility Targeting Engine: Ajuste dinámico de exposición de forma inversa al ATR para mantener constante el riesgo en dólares ($R) de forma parametrizable.
   ├─► Risk-Parity Normalizado: Desacoplamiento de ATR macro para mitigar drawdowns durante pánicos.
   ├─► Mapas de Cointegración, Cópulas y Correlation Neutralizer: Modelado de dependencias de colas pesadas, coberturas extremas fijas, y neutralización/capado de lotaje activo ante cointegraciones > 0.8 en vivo.
   ├─► Router Viviente: Rotación de capital desde lateralidades estancadas (>72h) hacia vectores de liquidez eficientes.
   ├─► Auto-Rebalancing Daemon: Disparador automático por triggers (HMM régimen, Calendario semanal/mensual, Threshold de desviación o alertas VaR/CVaR).
   ├─► Búsqueda Genética de Portafolios: Motor genético offline que busca qué estrategias del Databank combinar óptimamente usando la Weighted Fitness Formula.
   ├─► Análisis de Solapamiento Temporal Real: Consultas vectoriales DuckDB de colisiones de tickets abiertos barra a barra con cálculo de riesgo acumulado simultáneo máximo.
   ├─► Portfolio Weights Rescaler & Ledger: Conversión de pesos en lotes exactos y simulación continua por hora de balance y margen integrado.
   ├─► Mitigación de Riesgo: Restricción a máximo 1 rebalanceo por día (Circuit Breaker) y suspensión si la varianza del portafolio es mayor a 2σ.
   └─► Acceso datos: Historial inmutable en SQLite `portfolio_rebalancing_history` (pesos, régimen, slippage).


6. execute: Colocar orden [EJECUCIÓN VIVA - validación <1ms; orden end-to-end ≤100ms]
   ├─► Features consumidas: [broker-connector], [order-fsm], [slippage-models], [equity-curve-tracker], [institutional-metrics], [hmm-regime-detection], [audit-log]
   ├─► [10 validaciones (ADR-0025): liquidez/spread, slippage, tamaño de posición, exposición de portafolio, correlación, drawdown, pérdida diaria, frecuencia de órdenes, margen, aprobación final]
   ├─► Lógica pura: Transición de estado orden (máquina 64-bit, atómica)
   ├─► Orquestación: Enviar a broker
   ├─► Acceso datos: Guardar ejecución en transacción
   └─► Supervisión: Latido de vida + botón de emergencia + detección anomalías

7. feedback: Veredicto de salud y cierre de bucle [Continuo + cierre batch] — ANALISTA
   ├─► Features consumidas: [pardo-comparison], [trade-reconciler], [anomaly-detector], [factor-decomposition], [alpha-purity-analyzer], [equity-curve-tracker], [audit-log]
   ├─► Lógica pura: Reconciliar real vs esperado (spreads, paper vs vivo)
   ├─► Lógica pura: Diagnosticar causa de degradación (¿murió el Alpha o solo el Beta/régimen?)
   ├─► Detectar anomalías: Cambios ejecución, brechas datos, correlaciones rotas
   ├─► Orquestación: Emitir veredicto de continuidad/retiro (señal AUTO_WITHDRAW si procede)
   └─► Acceso datos: Sugerir constraints a generate (cierre de bucle causal, ADR-0015)

8. withdraw: Salida controlada [Actuador del veredicto de feedback]
   ├─► Lógica pura: Recibir veredicto de retiro (no monitorea; el monitoreo es de feedback)
   ├─► Orquestación: Transición FSM Ejecutando → En Pausa (ventana de veto) → Retirado/Archivo
   └─► Acceso datos: Archivar métricas terminales y notificar a manage → rebalanceo
```

### Persistencia
* **SQLite:** Local, modo **WAL** (Write-Ahead Logging) forzado para lectura/escritura concurrente.
* **Auto-Recovery:** Al arrancar, el orquestador consulta la tabla de `jobs` y reanuda automáticamente tareas interrumpidas (`RUNNING`).
* **Invariante de Trazabilidad:** Todo cambio de estado atómico genera un registro en el `audit-log`.
* **Zero-Copy Performance:** Uso de Polars/Arrow para mover grandes volúmenes de datos OHLCV sin serialización costosa.

### Condiciones de Transición entre Módulos

| Transición | Condición Conceptual | Detalles |
|---|---|---|
| ingest → generate | Datos validados + régimen clasificado | Barras listas para exploración |
| generate → validate | Candidatas generadas | Suite de validación (WFA, Monte Carlo, coherencia) |
| validate → incubate | Validación aprobada + robustez mínima configurable | Forward testing en vivo |
| incubate → manage | Pardo test pasado + drift aceptable (configurable) | Promoción a portafolio candidato |
| execute → feedback | Orden ejecutada / anomalía detectada | Delta real vs esperado disponible |
| feedback → withdraw | Veredicto de retiro (Drift > umbral) | Estrategia enviada a Retiro Emérito (Archivo Institucional) |
| withdraw → generate | Archivo completado + Insights | Nuevo ciclo con restricciones del fallo |
por usuario (ver ADR-0008: Configurabilidad Universal)*

---

## 9. La Fontanería

### Seguridad
* **Validación Desconfiada:** Validación de toda entrada externa (FFI/gRPC, WebSockets externos, base de datos).
* **Máquina de Estados:** Estados definidos exhaustivamente; transiciones imposibles no pueden ocurrir.
* **Auditoría:** Cada cambio de estado guardado con marca de tiempo.
* **Soberanía y Criptografía (ADR-0093):**
    * **Protección de Llaves:** Encriptación AES-256-GCM de claves de API en `broker_connections` con Master Key en variable de entorno.
    * **Auditoría Inmutable:** Registro secuencial encadenado mediante hash de transacciones en SQLite.
    * **Privacidad Soberana:** Cero telemetría externa para proteger IP de estrategias y datos operativos del usuario.

### Ejecución Automática con Auditoría

Ver **ADR-0010: Reglas Dinámicas (Hard Limits vs Soft Alerts)** para el mecanismo completo de ejecución automática, auditoría y veto.

### Concurrencia y Parallelismo
* **Monitoreo de degradación en paralelo con ejecución:** Monitoreo continuo de cambios mientras el portafolio ejecuta. Sin bloqueos; detección de cambios PnL/drawdown/régimen activable en cualquier momento.
* **Retroalimentación (Bucle Pardo) en paralelo y al cierre:** Control de calidad estadístico continuo. Compara paper/vivo vs backtest. Sugerencias se retroalimentan a generar para nueva evolución.
* **Arquitectura:** Tareas independientes asincrónicas; eventos entre módulos via bus de eventos.

### Operaciones Asincrónicas

Para operaciones costosas (backtesting masivo, generación de candidatas, optimización de portafolio), ver **ADR-0011: Operaciones Asincrónicas (Async Job Pattern)**.

### Multi-Pipeline Paralela

Para ejecución de N pipelines simultáneamente con reserva de recursos para live trading (SLA de reserva), ver **ADR-0012: Arquitectura Multi-Pipeline Paralela**. El sistema garantiza que al menos 2 núcleos de CPU y 4GB de RAM estén siempre disponibles exclusivamente para el pipeline de ejecución en vivo.

### Contratos de Intercambio (Signal Contracts)
Para asegurar que el pipeline sea reproducible y agnóstico, se definen los siguientes objetos de intercambio inmutables:
* **SignalEquation:** Representación simbólica o lógica de una señal de entrada.
* **FitnessVector:** Vector multi-objetivo (Sharpe, MaxDD, WinRate) usado para la selección natural de estrategias.

### Observabilidad

- **Propiedades:** Latencia p99 < 100ms (NautilusTrader), Throughput competitivo (más rápido que MT5/SQX/QuantConnect en igual hardware vía Polars Native, sin KPI absoluto — ADR-0114), disponibilidad local 100%.
- **Foundation Inundation (ADR-0020 V2):** Inyección de hooks tempranos para evitar refactorizaciones futuras. El esquema de base de datos es **Distribuidor y Basado en Requisitos**: cada Feature define su propio contrato de persistencia en su archivo `.md`, pero todas deben obedecer el **Contrato Global** definido en [ADR-0020 V2](./ADR.md#adr-0020-principio-de-inundación-de-fundaciones-v2-foundation-inundation).
- **Registro de eventos Estructurado:** Definido en [`telemetry.md`](./features/telemetry.md) (en JSON, rastreable).
- **Métricas:** Latencia de ingesta, rendimiento de señales, órdenes por segundo, cambio de rendimiento, drawdown actual.
* **Métricas Desacopladas por Módulo:** Cada módulo expone sus propias métricas de forma independiente; la feature `telemetry` las recolecta dinámicamente. 
* **Consumidor Maestro:** El módulo de `feedback` consume todos estos puntos de evidencia (causalidad) para generar los veredictos de salud de Pardo. **Ver ADR-0015**.
* **Supervisión Activa:** Latido de vida cada N segundos desde ejecución; botón de emergencia automático si drawdown excede límite crítico.

### Despliegue
* **Un solo binario ejecutable:** Binario nativo Rust compilado.
* **CI/CD:** Automatización (tests + validación de cambios de esquema).
* **Reversión:** Reversión de base de datos (control de cambios), reversión de código (historial de versiones).

---

## 10. Propiedades del Sistema (SLAs, Limitaciones)

| Métrica | Objetivo | Cómo se logra |
|---|---|---|
| **Barras Algorítmicas** | Soporte Nativo | Motor `algorithmic-bars` (Renko, Range, Volume). |
| **Latencia Ingesta** | < 10ms (dato → base datos) | Entrada/salida asincrónica, parsing con compilación automática. |
| **Latencia Señal** | < 50ms (precio → orden propuesta) | Lógica pura, sin consultas a base de datos. |
| **Rendimiento** | Más rápido que MT5/SQX/QuantConnect en igual hardware (ADR-0114; sin KPI absoluto) | Cálculos en paralelo en CPU, hilos nativos Rust. |
| **Reproducibilidad** | 100% (simulación = vivo) | Lógica pura, semilla fija, estados numéricos exactos. |
| **Disponibilidad** | 99.5% (mercado cripto) | Reintentos automáticos, fallback manual. |
| **Velocidad Pruebas** | < 1ms por prueba unitaria | Sin base de datos, lógica pura. |

---

## 11. Restricciones de Negocio (Invariantes del Sistema)

### Validación de Datos (ingest)
* **Regla:** Ningún dato sin validación puede entrar en un módulo.
* **Por qué:** Datos malos contaminan toda simulación histórica → estrategias falsas → pérdida de dinero.
* **Implementación:** Antes de cualquier lectura externa (gRPC/WebSocket, archivo), validar; rechazar si no pasa.
* **Consecuencia:** Anomalía registrada en observabilidad; no procesar ese dato.

### Regímenes de Mercado Incompletos (ingest)
* **Regla:** "Régimen desconocido" es válido pero explícito; módulos posteriores saben que no hay clasificación.
* **Por qué:** Evitar que generar/validar asuman régimen cuando no hay suficiente historial de volatilidad.
* **Implementación:** Precio con régimen desconocido se guarda; generar puede usarla pero debe registrar advertencia.

### Inmutabilidad de Veredictos de Validación (validar)
* **Regla:** Una vez que se genera un análisis, es inmutable. Nuevas pruebas se agregan, pero el veredicto original no cambia.
* **Por qué:** Auditoría regulatoria; reproducibilidad histórica. Si el veredicto cambiara, se pierden registros.
* **Implementación:** Marcar análisis como bloqueado después de primera generación; rechazar recomputaciones.
* **Consecuencia:** Historial completo rastreable + reproducibilidad total.

### Herencia de Resultados (validar - Optimización del Historial)
* **Regla:** Si la prueba es idéntica a una versión anterior, heredar resultado sin re-ejecutar.
* **Por qué:** Pruebas A/B sin costo extra; evitar recalcular lo ya validado (ahorro >80% en iteraciones rápidas).
* **Implementación:** El **[`incremental-test-engine`](./features/incremental-test-engine.md)** gestiona el hashing de parámetros y la búsqueda de evidencia previa.
* **Beneficio:** Pruebas transversales (WFA, MC, Stress) más rápidas y consistentes.


### Baseline Congelado en Comparativas (incubar)
* **Regla:** La comparativa entre ejecución simulada y viva usa el baseline original, no un recálculo nuevo.
* **Por qué:** Si el baseline cambia, la comparativa pierde validez estadística → alertas falsas de degradación.
* **Implementación:** Guardar baseline cuando se aprueba la estrategia; usarlo siempre igual.

### Portafolio tiene prioridad sobre Estrategia Individual (gestionar / ejecutar)
* **Regla:** Si hay conflicto entre regla de portafolio y regla de estrategia individual, portafolio gana.
* **Por qué:** El portafolio gestiona riesgo global; una estrategia no puede violar límites del conjunto.
* **Implementación:** Al ejecutar, validar contra reglas de portafolio ANTES que reglas de estrategia.

### Decisiones Automáticas Críticas Revertibles (ejecutar)
* **Regla:** Toda decisión automática crítica (cierre de posición, reducción de peso) puede deshacerse en un plazo configurable.
* **Por qué:** Control del usuario: el sistema actúa pero el dueño mantiene poder de decisión final.
* **Implementación:** Marcar decisión como reversible, registrar cuándo ocurrió, permitir ventana de tiempo (ej: 5 minutos). Usuario puede deshacer.

### Retiro con Período de Espera (retirar)
* **Regla:** Entre ejecutando y retirado siempre hay pausa con período configurable (ej: 1 día) donde se puede revertir.
* **Por qué:** Evitar retiros accidentales por anomalías temporales; poder cambiar de opinión.
* **Implementación:** Máquina de estados: Ejecutando → En Pausa → Retirado. En pausa, usuario puede reactivar.

### Precios en Lógica Pura son Números Exactos (Transversal)
* **Regla:** En la lógica pura, precios siempre son números exactos (centavos/ticks), no decimales.
* **Por qué:** Evitar errores acumulados de decimales en operaciones financieras.
* **Implementación:** Conversión de decimal a exacto ocurre solo en acceso datos y capas externas. Lógica pura siempre usa exactos.
* **Beneficio:** Reproducibilidad absoluta; cálculos de ganancias/pérdidas sin errores.

### Sin Sorpresas de Tiempo en Lógica Pura (Transversal)
* **Regla:** Nunca obtener la hora actual dentro de la lógica pura. Recibir el tiempo como parámetro de entrada.
* **Por qué:** Reproducibilidad y testeabilidad. Una prueba puede decir "es 2024-01-01 09:30:00" y forzar ese tiempo.
* **Implementación:** El tiempo es un parámetro que se pasa (inyección de dependencia).
* **Beneficio:** Simulaciones históricas reproducibles; debugging sin sorpresas.

---

## 12. Lifecycle de Estrategia (FSM Completo)

```
                    ┌─────────────────────────────────────┐
                    │  EN PRUEBA                          │
                    │  (generación crea candidatos)       │
                    └──────────┬──────────────────────────┘
                               │ validación: aprobado
                               ▼
                    ┌─────────────────────────────────────┐
                    │  EN INCUBACIÓN                      │
                    │  (simulada; perfil config. ADR-0088)│
                    └──────────┬──────────────────────────┘
                               │ incubación: pasa validación
                               ▼
                    ┌─────────────────────────────────────┐
                    │  EJECUTANDO                         │
                    │  (ejecución viva en portafolio)     │
                    └──────────┬──────────────────────────┘
                               │ retiro detecta degradación
                               ▼
                    ┌─────────────────────────────────────┐
                    │  EN PAUSA                           │
                    │  (período para reconsiderar: 1 día) │
                    └─┬──────────────────────────────────┬┘
       usuario reactiva│                                 │ usuario decide
              (vuelve a EJECUTANDO)                       │ retiro permanente
                                                          ▼
                                              ┌─────────────────────────────────────┐
                                              │  RETIRADO                           │
                                              │  (archivado; no se reactiva solo)   │
                                              └─────────────────────────────────────┘
```

**Reglas de Transición:**
- EN PRUEBA → EN INCUBACIÓN: Después de validar (aprobado).
- EN INCUBACIÓN → EJECUTANDO: Después de incubar (pasa validación).
- EJECUTANDO → EN PAUSA: Retiro detecta degradación (rendimiento cae >30% OR pérdidas máximas >150% de lo esperado).
- EN PAUSA → EJECUTANDO: Usuario decide reactivar dentro del período.
- EN PAUSA → RETIRADO: Usuario decide retiro permanente tras el período.
- EJECUTANDO → RETIRADO: Usuario fuerza retiro inmediato (sin pasar por EN PAUSA).
- RETIRADO → X: Sin retorno automático (requiere acción manual).

### 12.1 El Ciclo de Refinamiento (Refine Cycle)

El sistema no es un pipeline lineal unidireccional. Permite bucles de realimentación (loops) donde una estrategia o portafolio puede retroceder a fases anteriores para ajustes sin perder su identidad ni su historial en el DAG de versiones:

1.  **Operación → Validación:** Si el módulo de ejecución detecta una debilidad o cambio de comportamiento leve, la estrategia puede ser enviada de vuelta a validación robusta para re-evaluar sus métricas bajo el nuevo régimen de mercado.
2.  **Gestión → Validación:** Si el optimizador de portafolio detecta que una combinación de estrategias es subóptima, puede forzar una re-validación de las piezas individuales antes de rebalancear.
3.  **Generación Continua:** El módulo de Feedback detecta anomalías persistentes y dispara un nuevo ciclo en Generación, inyectando restricciones (constraints) que eviten repetir errores pasados.

---

## 13. Estándares de Implementación (Gobernanza)

Para mantener la integridad del monolito modular, se aplican los siguientes estándares obligatorios:

*   **Contratos Inyectables:** Todo acceso a infraestructura (DB, tiempo, red) se realiza a través de interfaces (Ports).
*   **Evolución Incremental:** Las nuevas funcionalidades no crean tareas paralelas, sino que refinan los contratos y TTRs existentes. Ver **ADR-0014**.
*   **Causalidad Obligatoria:** Todo módulo debe emitir evidencia para el módulo de Feedback (Consumidor Maestro). Ver **ADR-0015**.
*   **Local-First Processing:** El cómputo pesado reside en la infraestructura del usuario; el cloud es solo un overlay de soporte (Auth, Flags, P2P). Ver **ADR-0016**.
*   **Fidelidad Extrema:** La simulación debe replicar la fricción institucional (4-ticks, triple swap, límite de Pardo). Ver **ADR-0017**.
*   **Cero Lógica en el Shell (Soberanía de Features):** Los módulos son orquestadores puros (Thin Shell); toda lógica algorítmica reside en Features reutilizables.

---

## 14. Glosario Técnico

* **Functional Core:** La lógica pura y determinista del sistema. Sin side-effects, sin I/O. Compatible con optimización vectorial SIMD y compilación Ahead-Of-Time. 
  * *Sinónimos prohibidos:* "Business Logic" (demasiado vago), "Service Layer" (eso es el Shell).

* **Imperative Shell:** Todo lo que no es Core: Controllers, Services de orquestación, Repositories, I/O.
  * *Sinónimos prohibidos:* "Glue Code" (suena despectivo), "Infrastructure" (eso es solo una parte del Shell).

* **Entidad Pura:** Objeto de datos (estructuras de Rust) que representa un agregado del dominio. Nunca un modelo de base de datos ORM físico.
  * *Sinónimos prohibidos:* "DTO" (confunde con patrón diferente).

* **Invariante:** Una regla del dominio que NUNCA puede violarse. Ejemplo: "margen no puede ser negativo".
  * *Sinónimos prohibidos:* "Constraint" (SQL, demasiado físico).

* **Transacción ACID:** Cambio de estado garantizado atómico en persistencia. La capa Service/Repository es responsable.
  * *Sinónimos prohibidos:* "Cambio de estado" a secas (ambiguo si es atomic o no).

* **Máquina de Estados:** Conjunto de situaciones definidas exhaustivamente; cambios entre ellas sólo ocurren cuando se deben.

* **Compilación Automática:** Compilador que convierte código de alto nivel a código máquina (velocidad similar a C).

* **Acceso a Memoria Compartida:** Compartir datos entre partes sin copiar (copiar es costoso).

* **Control de Cambios de Esquema:** Herramienta que versionea cambios de estructura de tablas.

* **Un Binario, Muchos Módulos:** Una sola aplicación ejecutable, múltiples partes independientes, sin latencia de red.

* **Interfaz Pública:** El punto de entrada de cada módulo; única forma que otros módulos lo usan.

---

## 15. Riesgos y Mitigaciones

| Riesgo | Impacto | Mitigación |
|---|---|---|
| Dos cambios simultáneos de estado | Estado confuso, pérdida de dinero | Transacción atómica en base de datos. |
| Lógica pura toca base de datos | Rompe compilación automática, lento | Pruebas + verificación automática que prohibe acceso a datos en lógica pura. |
| Módulo A consulta tabla de módulo B | Acoplamiento (cambios rompen todo) | Regla de diseño; revisión de código. |
| Control de cambios confunde tablas | Inconsistencia de base de datos | Una sola fuente de verdad para cambios (archivo centralizado). |
| Librerías vectoriales se quedan sin memoria | Crash en producción | Procesar en lotes pequeños; pruebas de volumen. |

---

## 16. Grafo de Dependencias Técnicas Entre Módulos

El sistema está estructurado en capas de dependencia que definen el orden lógico requerido:

### Capa Base: Ingesta de Datos
**MOD-01 (ingest)** — Fuente de datos inmutable de verdad.
* Sin dependencias de otros módulos.
* Todos los módulos posteriores dependen directa o indirectamente de su salida.

### Capas de Descubrimiento y Validación
**MOD-02 (generate)** depende de MOD-01.
* Lee datos históricos de ingest.
* Genera candidatas de estrategias.

**MOD-03 (validate)** depende de MOD-02.
* Valida candidatas generadas.
* Produce veredictos sobre robustez.

### Capa de Forward Testing
**MOD-04 (incubate)** depende de MOD-03.
* Requiere estrategias aprobadas en validación.
* Prueba en tiempo forward (perfil de incubación configurable: 7/21 días o 3-6 meses — ADR-0088).
* Filtra overfitting y cambios de régimen.

### Capa de Orquestación
**MOD-05 (manage)** depende de MOD-04.
* Requiere estrategias promovidas desde incubación.
* Ensambla portafolios optimizados.
* Define reglas de portafolio que guían ejecución.

### Capas Vivas (Ejecución y Monitoreo — Paralelo Continuo)
Estas capas operan en paralelo mientras el portafolio está activo. No tienen relación secuencial entre sí, pero todas dependen de MOD-05:

**MOD-06 (execute)** depende de MOD-05.
* Recibe portafolio optimizado y reglas.
* Ejecuta en tiempo real.
* Registra todas las decisiones automáticas en audit trail.

**MOD-07 (feedback)** — Analista. Observa TODOS los módulos en paralelo.
* Recolecta datos de ejecución (MOD-06), validación histórica (MOD-03) y P&L del portafolio.
* Detecta degradación y anomalías transversales; diagnostica la causa (Alpha muerto vs Beta/régimen).
* Emite el veredicto de continuidad/retiro y retroalimenta constraints a MOD-02 (cierre de bucle, ADR-0015).

**MOD-08 (withdraw)** — Actuador. Ejecuta el veredicto de retiro emitido por MOD-07 (no monitorea).
* Gestiona la transición del FSM: Ejecutando → En Pausa (ventana de veto) → Retirado/Archivo.
* Archiva métricas terminales y notifica a MOD-05 (manage) para rebalanceo.

---

## 17. Gobernanza Operacional (Protocolos de Salud)

Para garantizar la integridad a largo plazo del monolito modular, se establecen los siguientes protocolos de mantenimiento obligatorio:

### 17.1 Protocolo de Propagación de Contratos (Interface Drift)
Ante cualquier cambio en una **Frontera Pública** o **Schema** de un módulo:
1.  Identificar todos los módulos consumidores (Clientes).
2.  Actualizar los TTRs de integración en los Clientes para reflejar el nuevo contrato.
3.  Validar que el "Evidence Trail" (ADR-0015) no se haya roto por el cambio de esquema.

### 17.2 Protocolo de Cierre de Bucle (Feedback Harvest)
Al añadir o modificar cualquier métrica técnica o de negocio en una Feature:
1.  Evaluar utilidad para el Control de Calidad Estadístico (MOD-07).
2.  Definir el punto de emisión en el **Rastro de Evidencia** de la feature.
3.  Actualizar la especificación de `feedback.md` para integrar la nueva fuente de aprendizaje.

### 17.3 Protocolo de Soberanía de Datos (Cross-Module Shield)
Si un módulo requiere datos persistidos por otro módulo:
1.  **PROHIBIDO** el acceso a la DB ajena.
2.  Crear un Puerto de consulta en la `public_interface.rs` del módulo dueño.
3.  El módulo consultor debe tratar el dato como inmutable y conforme al Contrato Global (ADR-0020 V2).

### 17.4 Protocolo de Neutralización del Masterplan (Legacy Extraction)
Al extraer requisitos de documentos legacy (como el Masterplan):
1.  Mapear a la Feature correspondiente o crear una nueva si no existe.
2.  Integrar los TTRs bajo la regla de **Evolución Incremental (ADR-0014)**.
3.  Ejecutar **Inundación de Fundaciones (ADR-0020 V2)** sobre el nuevo requerimiento de inmediato.

### 17.5 Protocolo de Preservación de Performance (SLA Guard)
Toda feature que impacte en el "Hot Path" (Ingest, Generate, Validate) debe ser auditada contra el criterio competitivo relativo (más rápido que MT5/SQX/QuantConnect en igual hardware, ADR-0114; sin KPI absoluto):
1.  **Vectorización Obligatoria:** Uso de Polars/Arrow para manipulación masiva de datos.
2.  **Native Compliance:** Cualquier loop secuencial debe ser implementado en Rust nativo optimizado.
3.  **IO Inundation:** Los campos inyectados por ADR-0020 V2 deben escribirse mediante transacciones batch en SQLite WAL.

### 17.6 Protocolo de Madurez (Transition Audit)
Al mover una Feature de estado `Especificación` a `Implementación`:
1.  Auditar cumplimiento de Gobernanza (ADR-0016, 0017, ADR-0020 V2).
2.  Verificar que los TTRs no sean ambiguos y tengan criterios de éxito técnicos.
3.  Asegurar que el orquestador del módulo posee los Puertos necesarios para la nueva lógica.

### 17.7 Protocolo de Auto-Evolución del Skill (Meta-Governance)
El conocimiento arquitectónico generado en sesiones de alta densidad debe ser "decriptado" en instrucciones técnicas para el agente:
1.  **Sync-Trigger:** Todo nuevo patrón aprobado en SAD/ADR debe evaluarse para su inclusión en el `.agent/workflows/quant-architect.md`.
2.  **Cierre de Brecha Cognitiva:** Si el agente requiere aclaración sobre un estándar >2 veces, se debe formalizar una sección en el workflow para evitar la recurrencia.
3.  **Refactorización de Skill:** Las instrucciones del agente se consideran "Código Vivo" y deben ser refactorizadas para eliminar ambigüedad tras cada hito arquitectónico.

### 17.8 Protocolo de Integridad Cruzada (Cross-Document Integrity - CODI)
Ningún documento es una isla. Todo cambio técnico significativo conlleva una revisión de impacto transversal:
1.  **Análisis de Sprint:** Ante un cambio en una Feature, auditar: SAD (Topología), ADR (Decisiones), TEMPLATES (Estándares) y Workflow (Operación).
2.  **Sincronización Atómica:** El cambio no se considera "commiteado" hasta que todos los mirrors y referencias cruzadas han sido actualizados.
3.  **Trazabilidad de Impacto:** Mantener las dependencias explícitas para facilitar la identificación del radio de acción de cada cambio.

### 17.9 Protocolo de Inundación Institucional (Audit Readiness)
Al crear o refactorizar cualquier entidad de persistencia (Tabla, Archivo Parquet, Evento):
1.  **Inundación Obligatoria (selectiva por perfil):** Inyectar el **grupo I (Identidad & Integridad)** de forma universal en toda entidad, y el resto de los **25 campos del contrato lógico** de forma **selectiva según el Perfil Técnico** (A. Datos/Ingest, B. IA/R&D, C. Ops/Hot-Path, D. Ops/Auditoría), conforme a la tabla canónica de Filtro de Relevancia definida en [ADR-0020 V2](./ADR.md#adr-0020-v2). El contrato es un vocabulario lógico obligatorio, no 25 columnas calcadas en cada tabla.

    > **Ejemplo concreto (dos capas que NO deben confundirse):** la tabla `foundation_master_fields` (migración 0001) es el **catálogo de referencia** con las 25 columnas — existe UNA sola vez en todo el sistema, no se replica. Las tablas propias de cada módulo/feature (ADR-0003: cada módulo es dueño de sus tablas) NUNCA tienen esas 25 columnas; tienen sus columnas de dominio + el Grupo I completo (6 columnas, universal) + solo los campos concretos de su Perfil Técnico. Ej: la tabla de `adaptive-volume-indicators` (Perfil B / IA-R&D) lleva sus valores de indicador + Grupo I + (`owner_id`, `institutional_tag`, `manifest_id` de II) + (`logic_hash`, `data_snapshot_id`, `indicator_state_hash`, `version_node_id` de III) + (`node_id`, `process_id`, `execution_latency_ms` de IV) — nada de Grupo V, porque su perfil no lo cubre.

2.  **Hooks Forenses:** Definir el rastro de evidencia específico (latencias, estados internos) para alimentar el módulo de `feedback`.
3.  **Soberanía Multi-tenant:** Asegurar que `institutional_tag` y `owner_id` están correctamente mapeados en la capa de interacción (Shell).

### Resumen Visual
```
MOD-01 (ingest)
    ↓
MOD-02 (generate)
    ↓
MOD-03 (validate)
    ↓
MOD-04 (incubate)
    ↓
MOD-05 (manage)
    ↓
MOD-06 (execute)
    ↓
MOD-07 (feedback)
    ↓
MOD-08 (withdraw)
    │
    └─► [Aprendizaje para MOD-02]
```

---

## 18. Plan de Lanzamiento (Rollout Strategy v2.0)

El desarrollo se organiza en sprints incrementales para mitigar riesgos técnicos bajo la arquitectura unificada Rust (Core) + Flutter (Frontend via FFI):

* **Sprint 0.X — Fundación:** Configuración del entorno Cargo + Cargo workspaces, estructura de módulos en Rust, persistencia local con SQLite (SQLx) y SQLx compile-time embedded migrations. Paridad básica de NautilusTrader backtest y smoke test de cómputo numérico CPU-first (`ndarray`/Rayon, ADR-0112). Implementación de interfaz base trait/protocol `IDrasusNode` en Rust. Endpoint de telemetría local de hardware y notificaciones locales SQLite.
* **Sprint 1.X — Generación (Ingest & Generate):** Ingesta local de datos históricos Parquet optimizada con Polars. Pipeline de limpieza y alineación temporal (Data Sanitizer). DuckDB embebido para remuestreo dinámico. Ejecución de NautilusTrader con simulación multicanal y alineación Bar-Open. Minero Genético NSGA-II nativo en Rust, compilación a Strategy AST v3.0, cálculo de métricas vectorizadas y entrenamiento local de modelos HMM para clasificación de regímenes.
* **Sprint 2.X — Robustez (Validate):** Walk-Forward Analysis (WFA) Matrix local y tests de simulación de estrés Monte Carlo CPU-first (`ndarray`/Rayon; GPU `candle` opcional, ADR-0112). Métricas de inferencia estadística e inyección de ruido. PCA Toxicity Clustering y Autoencoder Outlier Detector local en CPU Rust puro (`candle`). Proyección dimensional UMAP y score de robustez ponderado. Portafolio multiactivo y optimización de pesos HRP.
* **Sprint 3.X — Ejecución (Execute):** Daemons persistentes de NautilusTrader (LiveNode) y adaptadores para brokers reales. Pre-Trade Risk Validator (<1ms) y Shadow Watchdog. Daemon de auto-rebalanceo de portafolios asíncrono con HRP y HMM dinámico.
* **Sprint 4.X — Interfaz Gráfica (Glass Box):** Frontend en Flutter con Impeller Engine para renderizado a 120 FPS. Lienzo nodal visual (DAG Editor) en CustomPainter y previsualizaciones locales mediante Micro-Backtests en hilos de Rust (FFI). Heatmaps interactivos de performance y suites de visualización.


---

## 19. Glosario y Apéndices

### 19.1 Stack Técnico Detallado
* **Backend:** Rust, Tokio, NautilusTrader nativo (crates v2), Polars, DuckDB, `ndarray`/Rayon (cómputo numérico CPU-first; `candle` opcional, ADR-0112).
* **Frontend:** Flutter, Flutter FFI, Flutter CustomPainter (Impeller GPU rendering).
* **Persistencia:** SQLite (WAL), Parquet (Hive), Apache Arrow.

### 19.2 Nomenclatura de Tareas (Traceability)
Para asegurar el 100% de trazabilidad entre el PRD y la implementación, se utiliza el formato `FASE.MODULO.TAREA`:
* **MOD-X.Y:** Módulo técnico (ej: MOD-01 Ingest).
* **TASK-X.Y.Z:** Requisito funcional específico (ej: TASK-1.1.1 NSGA-II).
* **FEAT-X:** Feature arquitectónica compartida (ADR-0003).

---

## 20. Gobernanza y Soberanía de Datos

Drasus Engine es un sistema **Local-First (ADR-0016)**. La persistencia se realiza en el sistema de archivos del usuario mediante SQLite para estados y Parquet para datos históricos. El usuario retiene el control total de su IP (estrategias) y capital, sin dependencia obligatoria de servicios en la nube. Toda entidad de datos obedece el **Contrato Global (ADR-0020 V2)**: el grupo I de Identidad & Integridad es universal y el resto del contrato lógico se inyecta de forma selectiva por perfil, asegurando auditabilidad institucional sin replicar 25 columnas en cada tabla.

---

**Documento versión 4.2** | Última actualización: 2026-06-14
