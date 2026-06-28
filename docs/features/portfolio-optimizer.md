# Portfolio Optimizer (Optimizador de Portafolio & Rebalanceador)

**Carpeta:** `./features/portfolio-optimizer/`
**Estado:** Especificación (Fase 2)
**Última actualización:** 2026-06-11
**Decisiones Arquitectónicas Asociadas:** ADR-0089 (Motores de Optimización de Portfolio Clásicos & Ensamblador Singular D-Score), ADR-0108, ADR-0111

---

## ¿Qué es esta feature?

El **Portfolio Optimizer** es el motor matemático central de gestión y rebalanceo de capital (Asset Allocation). Combina múltiples estrategias individuales para modelar un portafolio adaptativo de grado institucional, maximizando la diversificación real y defendiendo la curva de equidad agregada.

El componente opera bajo dos pilares:
1. **Motores de Pesaje Clásicos (Portfolio Optimization Engine):**
   - Resuelve la asignación clásica mediante el arsenal estadístico de: **Markowitz** (Mean-Variance estándar), **Black-Litterman** (ajustes basados en opiniones/views), **Equal Weighting** (línea base), **HRP (Hierarchical Risk Parity)** (agrupación jerárquica por distancia de correlación para evitar la sensibilidad de matrices de covarianza inestables), **Minimum Variance**, **Volatility Stabilization** y **Cluster Risk Convergence**.
2. **Ensamblador Singular D-Score (Risk Parity Dinámico & Alpha Decay):**
   - **Risk-Parity Normalizado (Desacoplo de ATR):** Ajusta pesos castigando volatilidades extremas del ATR macro, aplanando el ratio de retornos diarios para mitigar drawdowns.
   - **Hedging Tick-by-Tick (Cointegrative Voiding):** Monitor de nano-solapamientos cruzados de alta frecuencia. Si Agente A (ej. USDJPY Largo) y Agente B (ej. SP500 Corto) entran en solapamientos destructivos cointegrados (+0.85) intra-segundo, bloquea y desasiste márgenes de forma recíproca con cero rezago.
   - **Router Viviente (Liquidez Radárica):** Rastrea predecibilidad de activos. Si un activo entra en lateralidad agónica sin alfa (>72h), rota capital vía API hacia vectores eficientes emergentes exóticos (Materias primas o Criptoactivos).

Adicionalmente, incorpora el **Auto-Rebalancing Daemon** (scheduler nativo Tokio) que ejecuta el reequilibrio dinámico del portafolio bajo disparadores inteligentes y con mitigadores de riesgo de sobre-operación.

**Genes de Acción del Genoma de Portafolio y Correlación (ADR-0108/ADR-0111):** cuando ese genoma está activo (co-evolución de cartera), tres mecanismos existentes de este motor se exponen como sus Genes de Acción evolutivos: la composición de la **Búsqueda Genética de Portafolios** (TTR-005) materializa la activación/desactivación de miembro; el **Auto-Rebalancing Daemon** (TTR-004) junto con el **Router Viviente** (TTR-003) y el **Anclaje de Pesos** (TTR-008) materializan la rotación dinámica de pesos; y una nueva capacidad de **Inyección de Cobertura Sintética** (TTR-011) extiende el **Hedging Tick-by-Tick** (TTR-002) para abrir posiciones de cobertura activas, no solo bloquear solapamientos. El solapamiento direccional simultáneo medido por el **Análisis de Solapamiento Temporal Real** (TTR-006, `OVERLAP_THRESHOLD_PERCENT`) es el tercer Gen de Condición de Estado del dominio.

---

## Comportamientos Observables

- [ ] **Selección de Motor de Asignación:** Permite configurar y alternar entre los métodos de pesaje clásicos y el Ensamblador Singular D-Score.
- [ ] **Rastreo de Cointegración en Vivo:** Realiza análisis de correlación y cointegración en el feed intra-segundo; bloquea el margen asignado de forma inmediata ante nano-solapamientos destructivos (+0.85).
- [ ] **Rotación Activa de Capital:** El Router de Liquidez detecta estancamientos en el feed lateral (>72h) y conmuta la asignación del saldo disponible a otros mercados a través de comandos de la API.
- [ ] **Auto-Rebalancing Multitrayecto:** El daemon de rebalanceo escucha eventos de calendario, HMM (cambio de régimen con alta confianza) o variaciones de pesos, calculando y enviando comandos a la cola de NautilusTrader.
- [ ] **Circuit Breaker de Rebalanceo:** Restringe a máximo una ejecución de reequilibrio de pesos por día.
- [ ] **Variance Safety Gate:** Bloquea y suspende rebalanceos si la varianza diaria agregada del portafolio excede las 2 desviaciones estándar (2σ).
- [ ] **What-If de Pesos con Anclaje Manual:** El usuario desbloquea los pesos sugeridos por el motor, fija manualmente algunos (ej. "ninguna estrategia supera 20%"), "ancla" esos valores con un candado visual y ordena recalcular el resto respetando la restricción manual.
- [ ] **Cestas de Riesgo (Risk Buckets) Drag-and-Drop:** El usuario arrastra estrategias a cestas "Defensiva", "Agresiva" o "Core"; el motor respeta la categorización humana y recalcula los lotajes para que la cesta Defensiva cubra el Drawdown potencial de la Agresiva.
- [ ] Cuando el Genoma de Portafolio y Correlación (ADR-0111) está activo, el motor evolutivo puede activar o desactivar miembros de la cartera entre generaciones como Gen de Acción, además de — o en lugar de — la Búsqueda Genética de Portafolios offline (TTR-005).
- [ ] Cuando ese genoma está activo, la rotación de pesos del Auto-Rebalancing Daemon puede ser disparada directamente por los Genes de Condición del genoma de portafolio (correlación de cartera, DD/volatilidad agregada, solapamiento direccional), no solo por sus disparadores FIJOS (calendario, régimen, desviación, riesgo).
- [ ] Cuando ese genoma está activo, el motor puede inyectar una posición de cobertura sintética en un instrumento correlacionado como Gen de Acción, registrada con su propio `audit_hash` y vinculada al `manifest_id` de la cartera.

---

## Restricciones

- **Suma Atómica de Pesos:** El vector de asignación de capital del portafolio debe sumar de forma matemática e inquebrantable el 100% (1.0).
- **Límite de Latencia de Cointegración:** El cálculo y detección de cointegraciones intra-segundo (Voiding Gate) debe completarse en `<2ms` para evitar retrasar el hot path de órdenes.
- **Inviolabilidad de Circuit Breaker:** El Circuit Breaker diario es una invariante del daemon de rebalanceo; no puede omitirse a menos que el usuario emita un Comando firmado.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| OPTIMIZATION_METHOD | hrp | Markowitz / HRP / D-Score | Selecciona el solver matemático de distribución de capital. | CONFIG |
| REBALANCE_TRIGGERS | regime,calendar | regime, calendar, threshold, risk | Lista de disparadores activos para el daemon de rebalanceo. | CONFIG |
| COINTEGRATION_THRESHOLD | 0.85 | 0.70 - 0.99 | Coeficiente mínimo de cointegración destructiva para bloquear margen. | CONFIG |
| CORRELATION_NEUTRALIZER_THRESHOLD | 0.80 | 0.50 - 0.95 | Umbral en vivo para el Correlation Neutralizer. | CONFIG |
| LATERALITY_TIMEOUT_HOURS | 72 | 12 - 168 horas | Tiempo de estancamiento lateral antes de rotar capital. | CONFIG |
| CIRCUIT_BREAKER_LIMIT | 1 | 1 - 5 | Cantidad máxima de rebalanceos automáticos permitidos por día. | [FIJO] |
| VARIANCE_SAFETY_SIGMAS | 2.0 | 1.0 - 3.0 | Desviación estándar de varianza máxima para permitir reequilibrios. | CONFIG |
| GENETIC_POPULATION_SIZE | 100 | 20 - 500 | Tamaño de población para la búsqueda genética de portafolios. | CONFIG |
| GENETIC_GENERATIONS | 50 | 10 - 200 | Número de generaciones en la búsqueda genética de portafolios. | CONFIG |
| PORTFOLIO_SIZE_LIMITS | [2, 10] | [1, 50] | Límites mínimos y máximos de estrategias permitidas por portafolio. | CONFIG |
| TARGET_RETURN_DD_MULTIPLIER | 1.5 | 0.5 - 5.0 | Multiplicador de Retorno/DD objetivo para la selección de portafolios. | CONFIG |
| OVERLAP_THRESHOLD_PERCENT | 0.20 | 0.05 - 0.80 | Límite máximo de solapamiento temporal real de trades permitido. | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Algoritmos de clustering HRP, solver de optimización Markowitz, cálculo de D-Score y coeficientes de cointegración en `solvers.rs`.
- **Shell (Infraestructura):** Daemon de rebalanceo, listeners del bus de eventos de mercado, gestor de colas de órdenes internas y persistencia de reportes.
- **Frontera Pública:** Contrato de asignación `optimize_portfolio_weights()` y puerto de ejecución del daemon `start_rebalancing_daemon()`.

---

## Ciclo de Vida de la Feature — Portfolio Optimizer

### Entrada
- Lista de estrategias en vivo con su historial de retornos, balances y métricas en caliente.
- Feed de datos de mercado consolidado (ticks y cotizaciones multi-activo).
- Configuración de la política de rebalanceo (triggers y mitigación).

### Proceso
- Alinea los feeds temporales y calcula la matriz de covarianza en tiempo real.
- Resuelve la asignación óptima aplicando pesos dinámicos o la reducción de volatilidad.
- Evalúa nano-solapamientos en el feed de órdenes y aplica filtros de cointegración.
- Monitorea lateralidades y evalúa los disparadores del daemon de rebalanceo.

### Salida
- Vector inmutable de pesos {Estrategia: Peso}.
- Comandos de órdenes de ajuste enviados al core de NautilusTrader.
- Historial registrado en SQLite (`portfolio_rebalancing_history`).
- Atribución de fitness del Genoma de Portafolio y Correlación por candidato de cartera (ADR-0108/ADR-0111), cuando ese genoma está activo.

### Contextos de Uso

**Contexto 1: Gestión de Capital (Módulo Manage)**
- Calcula el tamaño nominal de las posiciones agregadas para el hot path de trading en vivo.

**Contexto 2: Daemon de Rebalanceo Automático (Hot Path / Ops)**
- Procesa triggers en tiempo real y ejecuta rebalanceos seguros sin intervención humana.

---

## Tareas (TTRs)

### **TTR-001: Implementación de Motores de Pesaje (HRP & Ensamblador D-Score)**
*   **¿Cuál es el problema?**
    La optimización clásica de media-varianza sufre inestabilidad matemática severa ante pequeños cambios en los retornos. Se requiere un pesaje robusto HRP y un modelo D-Score que normalice el ATR macro.
*   **¿Qué tiene que pasar?**
    El sistema calcula pesos óptimos normalizados (0.0 a 1.0) mediante HRP (clustering jerárquico) y D-Score (reducción de lotes por ATR macro), garantizando que las estrategias compartan el riesgo de forma eficiente y sumen exactamente 1.0.
*   **¿Cómo sé que está hecho?**
    - [ ] El dendrograma en la UI muestra la jerarquía y el vector final de pesos coincide con el solver HRP.
    - [ ] En pánicos de volatilidad simulada, los lotes se reducen de forma automática por la normalización de ATR.
*   **¿Qué no puede pasar?**
    - La suma de pesos no puede ser diferente a 1.0.

### **TTR-002: Hedging Tick-by-Tick (Cointegrative Voiding Gate, Cópulas & Correlation Neutralizer)**
*   **¿Cuál es el problema?**
    Estrategias con lógicas distintas a veces abren posiciones que colisionan en nano-segundos o en dependencias de colas extremas, acumulando un riesgo cruzado destructivo antes de que el monitor de portafolio actúe. La correlación lineal clásica (Pearson) es ineficiente en pánicos macroeconómicos donde las correlaciones convergen a 1.
*   **¿Qué tiene que pasar?**
    *   **Mapas de Cointegración y Cópulas:** El sistema utiliza cópulas dinámicas asimétricas para modelar y comprender la dependencia de colas pesadas (*Tail Dependence*) entre activos, forzando al portafolio a mantener coberturas de riesgo extremo fijas (ej. exposición al VIX).
    *   **Correlation Neutralizer:** Monitorea y neutraliza exposiciones en tiempo real. Si dos activos superan el umbral de cointegración en vivo configurado, el motor limita automáticamente el lotaje combinado o pausa orgánicamente la ejecución del agente con menor *Efficiency Ratio*.
    *   **Voiding Gate:** Rastreará y cancelará de forma recíproca volúmenes de órdenes en nano-segundos si detecta solapamientos destructivos con latencia menor a 2ms.
*   **¿Cómo sé que está hecho?**
    - [ ] Al simular señales simultáneas cointegradas en USDJPY Largo y SP500 Corto, las órdenes se bloquean y el margen asignado se mantiene intacto.
    - [ ] La latencia de cálculo se loguea consistentemente por debajo de 2ms.
    - [ ] El neutralizador interviene activamente limitando o pausando estrategias de menor rendimiento al superar el umbral de cointegración.
*   **¿Qué no puede pasar?**
    - El bloqueo no debe retrasar la ejecución de órdenes normales no correlacionadas.
    - No se debe permitir la omisión de las coberturas de riesgo extremo fijas si el modo de protección de cópulas está activo.

### **TTR-003: Router de Liquidez Vectorizada**
*   **¿Cuál es el problema?**
    El capital queda atrapado durante días en activos estancados en lateralidad sin generar Alpha, perdiendo coste de oportunidad.
*   **¿Qué tiene que pasar?**
    El orquestador mide el comportamiento de los activos. Si detecta lateralidad o ausencia de alfa en un par durante más de 72 horas, desplaza la asignación de capital a otros vectores de liquidez eficientes predefinidos mediante la API.
*   **¿Cómo sé que está hecho?**
    - [ ] Al detectar un mercado plano por 72 horas en el simulador, el capital disponible se transfiere a la estrategia de materias primas/criptoactivos activa.
*   **¿Qué no puede pasar?**
    - No se rota capital de estrategias con posiciones abiertas perdedoras sin antes gestionar la salida controlada.

### **TTR-004: Daemon de Rebalanceo Dinámico (Auto-Rebalancing Daemon)**
*   **¿Cuál es el problema?**
    El reequilibrio manual de pesos introduce demoras operativas e ineficiencias de mercado, mientras que el rebalanceo automático descontrolado (thrashing) destruye el balance por comisiones y deslizamientos.
*   **¿Qué tiene que pasar?**
    El demonio de rebalanceo asíncrono carga y ejecuta la política óptima descubierta durante la fase de simulación (frecuencia, ventana temporal, umbral y composición). Se activa mediante cuatro disparadores específicos:
    *   **Calendario:** Ejecución periódica parametrizada (semanal, mensual o trimestral).
    *   **Régimen Dinámico:** Disparo ante cambios en la clasificación del régimen del modelo oculto de Markov (HMM) si la confianza estadística supera el umbral optimizado.
    *   **Desviación:** Disparo si la desviación de los pesos actuales respecto al objetivo supera el límite establecido.
    *   **Riesgo:** Activación defensiva cuando el valor en riesgo (VaR) o el valor en riesgo condicional (CVaR) del portafolio excede los límites permitidos.
    En cada disparo, el motor recalcula la asignación óptima utilizando la ventana rodante y genera órdenes de ajuste comparando el estado actual con el nuevo objetivo. Las órdenes se envían directamente a la cola de comandos interna de NautilusTrader sin intermediarios externos. Se registra cada evento en la persistencia del historial con el sello temporal, régimen, pesos anteriores/posteriores y el deslizamiento realizado.
*   **¿Cómo sé que está hecho?**
    - [ ] El cambio de régimen con suficiente confianza detectado por HMM inicia la optimización y genera el vector de órdenes en la cola de NautilusTrader.
    - [ ] Se verifica en base de datos el registro del historial con el formato inmutable conteniendo todos los metadatos requeridos.
    - [ ] Alertas en el tablero en vivo reportan si el deslizamiento realizado duplica el valor esperado.
    - [ ] El tablero en tiempo real muestra consistentemente el régimen del mercado, pesos actuales y cronograma del próximo rebalanceo.
*   **¿Qué no puede pasar?**
    - No se permite más de un rebalanceo automático por día (Circuit Breaker).
    - Se prohíben rebalanceos si la varianza del portafolio excede las dos desviaciones estándar (2σ), protegiendo la cuenta en caos de mercado.
    - Si falla la detección de régimen, el sistema no se detiene; degrada elegantemente utilizando el último estado de régimen conocido.

### **TTR-005: Búsqueda Genética de Portafolios (Genetic Portfolio Search)**
*   **¿Cuál es el problema?**
    Optimizar únicamente el peso de un conjunto de estrategias fijas limita la diversificación real. Se requiere una herramienta automatizada que busque qué estrategias del Databank combinar óptimamente.
*   **¿Qué tiene que pasar?**
    Un motor genético asíncrono offline busca combinaciones óptimas de estrategias aplicando restricciones de población, generaciones, límites de cantidad mínima/máxima de estrategias por portafolio, y un multiplicador de Retorno/DD objetivo. El motor califica el score global usando la misma Weighted Fitness Formula configurable del minero de estrategias.
*   **¿Cómo sé que está hecho?**
    - [ ] El optimizador genético evoluciona combinaciones de portafolios y genera reportes en formato Parquet filtrando candidatos por debajo del multiplicador objetivo.
*   **¿Qué no puede pasar?**
    - El motor no debe sugerir combinaciones que violen los límites mínimos o máximos de cantidad de estrategias por portafolio.

### **TTR-006: Análisis de Solapamiento Temporal Real (Overlapping Trades Analysis)**
*   **¿Cuál es el problema?**
    Estrategias descorrelacionadas linealmente pueden tener trades abiertos simultáneamente, elevando el riesgo acumulado y la carga de margen de forma destructiva.
*   **¿Qué tiene que pasar?**
    Mediante consultas vectorizadas de DuckDB, el sistema calcula la superposición exacta barra a barra de los tickets abiertos entre cada par de estrategias. Reporta el número de solapamientos, la duración total de la superposición, y el riesgo acumulado simultáneo máximo. Aplica un filtro punitivo para alertar o rechazar combinaciones que superen el límite permitido.
*   **¿Cómo sé que está hecho?**
    - [ ] Al simular un portafolio, el análisis de DuckDB genera la matriz de solapamientos temporales reales con los tres indicadores solicitados.
*   **¿Qué no puede pasar?**
    - No se permiten combinaciones de estrategias en el portafolio candidato si su porcentaje de solapamiento temporal supera el umbral punitivo configurado.

### **TTR-007: Redimensionamiento de Pesos y Ledger de Simulación (Portfolio Weights Rescaler)**
*   **¿Cuál es el problema?**
    Los pesos configurados de forma simplificada por los humanos (ej. 25% plano) no corresponden a la viabilidad de margen y lotes computacionales en caliente.
*   **¿Qué tiene que pasar?**
    El optimizador transforma la distribución de pesos estática en lotajes precisos asimétricos. Genera un Ledger de simulación continuo que unifica por hora el balance general, margen libre y margen ocupado multi-ticket en base a las posiciones abiertas del portafolio.
*   **¿Cómo sé que está hecho?**
    - [ ] El Ledger de simulación reconstruye con fidelidad de hora los requerimientos de margen de múltiples transacciones cruzadas.
*   **¿Qué no puede pasar?**
    - La conversión de pesos no debe inducir a errores por redondeo que dejen capital sub-asignado o sobre-asignado.

### **TTR-008: What-If de Pesos con Anclaje Manual (Weight Locking)**
*   **¿Cuál es el problema?**
    La matemática (frontera eficiente / HRP) puede sugerir un 48% a una sola estrategia, pero la política del fondo del usuario prohíbe que cualquier estrategia supere cierto tope. El humano necesita imponer su restricción sin perder la optimización del resto.
*   **¿Qué tiene que pasar?**
    El usuario fija manualmente uno o más pesos y los "ancla" con un candado. El motor recalcula los pesos restantes respetando los anclados, manteniendo la suma total coherente.
*   **¿Cómo sé que está hecho?**
    - [ ] Anclo "Oro = 20%" y el motor redistribuye el resto sin tocar ese 20%.
    - [ ] Si los anclados ya suman el total, el motor no asigna nada al resto y lo advierte.
*   **¿Qué no puede pasar?**
    - El recálculo NUNCA modifica un peso anclado por el humano.
    - La suma de pesos NUNCA queda inconsistente (sobre/sub-asignación).

### **TTR-009: Cestas de Riesgo Drag-and-Drop (Risk Buckets)**
*   **¿Cuál es el problema?**
    Una tarta de porcentajes inamovible no refleja la intención táctica del Quant. El humano quiere agrupar estrategias por rol de riesgo y que el sistema dimensione en consecuencia.
*   **¿Qué tiene que pasar?**
    El usuario arrastra estrategias a cestas categóricas (Defensiva / Agresiva / Core). El motor dimensiona los lotajes de modo que la cesta Defensiva cubra matemáticamente el Drawdown potencial de la cesta Agresiva, usando los clústeres de HRP existentes.
*   **¿Cómo sé que está hecho?**
    - [ ] Muevo una estrategia a "Defensiva" y su lotaje se recalcula respecto a la cesta Agresiva.
    - [ ] El número y nombre de cestas es configurable.
*   **¿Qué no puede pasar?**
    - El motor NUNCA ignora la categorización humana de cesta.
*   **Slice Visual (Flutter/Impeller/FFI):** Lienzo de cestas con arrastre nativo; candados de anclaje; recálculo en el Core Rust vía FFI.

### **TTR-010: Genes de Acción de Activación de Miembro y Rotación de Pesos del Genoma de Portafolio y Correlación (ADR-0108/ADR-0111)**
*   **¿Cuál es el problema?** El Dominio de Portafolio y Correlación necesita que la composición de la cartera (qué Manifests están activos) y la distribución de pesos entre ellos sean Genes de Acción evolutivos, condicionados por sus Genes de Condición de Estado (correlación de cartera, DD/volatilidad agregada, solapamiento direccional).
*   **¿Qué tiene que pasar?** Cuando `ACTIVE_GENOME_DOMAINS` incluye Portafolio y Correlación, cada candidato de la población de carteras (TTR-007 de [`nsga2-optimizer`](./nsga2-optimizer.md)) declara qué miembros de `PORTFOLIO_COEVOLUTION_SIZE` están activos y el vector de pesos resultante; el Auto-Rebalancing Daemon (TTR-004) ejecuta ese vector como su objetivo de rebalanceo.
*   **¿Cómo sé que está hecho?**
    - [ ] Una cartera candidata con 5 miembros puede evolucionar hacia 4 miembros activos y 1 inactivo sin violar `PORTFOLIO_SIZE_LIMITS` ni `MIN_MEMBERS_PER_PORTFOLIO_PARAM`.
    - [ ] El vector de pesos resultante de la evolución respeta la Suma Atómica de Pesos (1.0) y el Circuit Breaker diario.
*   **¿Qué no puede pasar?** El motor evolutivo no puede activar más miembros de los permitidos por `PORTFOLIO_SIZE_LIMITS`, ni desactivar miembros por debajo de `MIN_MEMBERS_PER_PORTFOLIO_PARAM` (ADR-0111, [`complexity-penalization`](./complexity-penalization.md)).

### **TTR-011: Inyección de Cobertura Sintética como Gen de Acción (ADR-0108/ADR-0111)**
*   **¿Cuál es el problema?** El Hedging Tick-by-Tick actual (TTR-002) solo bloquea o cancela solapamientos destructivos detectados; el Dominio de Portafolio y Correlación necesita un Gen de Acción que **abra proactivamente** una posición de cobertura sintética en un instrumento correlacionado cuando sus Genes de Condición lo determinen.
*   **¿Qué tiene que pasar?** Cuando ese genoma está activo, el motor evolutivo puede resolver un nodo `wildcard_group` que especifica un instrumento de cobertura (de un universo configurado) y una proporción de exposición a cubrir; el Portfolio Optimizer envía la orden de cobertura a la cola de NautilusTrader igual que cualquier otra orden de ajuste de pesos.
*   **¿Cómo sé que está hecho?**
    - [ ] Una cartera con alta correlación direccional simultánea (Gen de Condición de Solapamiento, TTR-006) puede resolver una cobertura sintética en un instrumento correlacionado, registrada en `portfolio_rebalancing_history` con su propio `audit_hash`.
    - [ ] Sin el Genoma de Portafolio y Correlación activo, no se generan órdenes de cobertura sintética (comportamiento actual sin cambios).
*   **¿Qué no puede pasar?** La cobertura sintética nunca se abre si excede el margen disponible del portafolio (Restricción de Suma Atómica de Pesos sigue aplicando incluyendo la posición de cobertura).

---

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Cada reporte de rebalanceo y sesión de optimización registra el set de relevancia técnica. **Perfil B (IA/R&D), híbrido B+latencia** (lleva linaje III + latencia V <2ms del Voiding Gate):

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del evento de rebalanceo/snapshot |
| | `created_at` | Timestamp del cálculo de pesos (nanosegundos) |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de la matriz de covarianza y pesos óptimos |
| | `audit_chain_hash` | Hash acumulativo del historial de rebalanceos |
| | `event_sequence_id` | Secuencia de recuperación del rebalanceo |
| **II. Soberanía** | `owner_id` | Responsable legal de la cuenta agregada |
| | `institutional_tag` | Tag de cumplimiento operativo (LIVE / AUDIT) |
| **III. Linaje** | `logic_hash` | Hash del solver de optimización (HRP/Markowitz) |
| | `version_node_id` | Identificador del portafolio en el DAG de versiones |
| **IV. Hardware** | `node_id` | ID único del hardware físico ejecutor |
| | `process_id` | PID del daemon de rebalanceo |
| **V. Forense & Ejecución (latencia, híbrido)** | `execution_latency_ms` | Latencia de cálculo del rebalanceo / cointegración (<2ms) |
| | `indicator_state_hash` | Snapshot del vector de pesos resultantes (Grupo V) |
| | `portfolio_container_id` | Agrupador del portafolio optimizado (Gobernanza) |

---

## Gobernanza y Estándares (Fijos)

- **Genomas Modulares por Dominio (ADR-0108/ADR-0111):** este motor materializa los Genes de Acción (activación/desactivación de miembro, rotación de pesos, cobertura sintética) y el Gen de Condición de solapamiento direccional del Dominio de Portafolio y Correlación. Ver Registro de Dominios Genómicos en [`SAD.md`](../SAD.md) §2.3.

---

## Preparación para Opciones (Post-MVP — ADR-0140)

> **Estado:** Diferido. No implementar hasta que los cinco prerrequisitos de ADR-0140 se cumplan.

Los motores de pesaje (HRP, Markowitz, D-Score) optimizan retornos lineales. Las opciones introducen payoffs no-lineales que invalidan la matriz de covarianza clásica:

- Un portafolio con opciones necesita **optimización por escenarios (Monte Carlo)**, no por covarianza histórica.
- Las métricas de riesgo deben ser específicas: delta-neutral, vega-weighted exposure.
- El "Hedging Tick-by-Tick" (TTR-002) asume instrumentos lineales correlacionados; las opciones como cobertura no-lineal requieren un modelo distinto.

**Refactorización necesaria:** añadir un modo de optimización por escenarios para portafolios que incluyan opciones, con métricas de riesgo no-lineal (delta-adjusted exposure, vega-weighted VaR).

**Moonshots asociados:** [`greeks-monitor`](../moonshots/greeks-monitor.md), [`option-pricing-engine`](../moonshots/option-pricing-engine.md), [`option-strategy-builder`](../moonshots/option-strategy-builder.md).

---

## Dependencias
- [`portfolio-rules`](../features/portfolio-rules.md) — para la validación de límites globales.
- [`hmm-regime-detection`](../features/hmm-regime-detection.md) — para el trigger de cambio de régimen.
- [`backtest-engine`](../features/backtest-engine.md) — para el cálculo de retornos históricos del portafolio.
