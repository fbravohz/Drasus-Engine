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

El dominio candidato de Ejecución y Enrutamiento (genes de latencia de bróker, profundidad de libro L2, ratio cancelación/fill) fue evaluado y **excluido** del Registro activo por falta de datos de microestructura consistentes para el operador retail/solopreneur (mismo principio que ADR-0100); queda archivado en [`genoma-ejecucion-enrutamiento`](../moonshots/genoma-ejecucion-enrutamiento.md).

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
- [`robustness-score-aggregator`](../features/robustness-score-aggregator.md) — Motor de consolidación de los 5 scores individuales en el score ponderado final.
- [`robustness-verdict-engine`](../features/robustness-verdict-engine.md) — Motor de veredictos por plantilla determinista (ADR-0115); LLM local opcional, sin Ollama.

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

