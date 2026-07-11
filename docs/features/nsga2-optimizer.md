# NSGA-II Optimizer — Optimización Multiobjetivo

**Carpeta:** `./features/nsga2-optimizer/`
**Estado:** Lista para implementar

> **Corrección por pruebas múltiples (ADR-0151, punto #1 de la matriz):** la minería NSGA-II **NO** es un punto de publicación; **solo REGISTRA** el insumo (N + σ² + sketch del vector de Sharpe) en la Expedition activa (`expedition-ledger`). No deflacta aquí: la corrección se aplica río abajo, por punto de decisión. Impacto progresivo (ADR-0137).
**Última actualización:** 2026-06-11
**Decisiones Arquitectónicas Asociadas:** ADR-0042, ADR-0044, ADR-0108, ADR-0109, ADR-0110, ADR-0111, ADR-0130

---

## ¿Qué es?

NSGA-II es un algoritmo de optimización que busca configuraciones de estrategia que sean buenas en múltiples objetivos simultáneamente (Sharpe, drawdown mínimo, win rate) sin sacrificar uno por otro. Devuelve la Frontera Pareto: todas las configuraciones no-dominadas.

**Contexto de Negocio:** El módulo "generate" automatiza el descubrimiento de estrategias. Sin NSGA-II, la optimización sería mono-objetivo y propensa al overfitting. En el **Hybrid Genesis Engine**, el NSGA-II actúa como el motor de *fine-tuning* (Micro-ajuste), refinando los umbrales y topologías de las "Tesis de Alpha" generadas por el componente de Deep Reinforcement Learning (DRL).

**Problema:** Si optimizas solo para Sharpe máximo, obtienes estrategias de riesgo gigantesco. Si optimizas para drawdown mínimo, pierdes rentabilidad. Necesitas equilibrio.

**Solución:** NSGA-II evalúa todos los candidatos en los 3 objetivos simultáneamente y devuelve todas las soluciones "buenas" (no-dominadas). Incorpora **Fitness Metamórfico de Estado (ADR-0042)**, permitiendo que la función objetivo mute automáticamente según el estado de la cuenta (Challenge vs Funded).

**User Story:** Como usuario, quiero generar automáticamente estrategias pareto-óptimas sin intervención manual. El sistema debe explorar miles de combinaciones de parámetros y devolverme las mejores, dejándome elegir cuál balance de riesgo-rentabilidad prefiero.

**Impacto:** Bloqueador MVP. Sin NSGA-II, el descubrimiento automático no existe y todo es manual.

**Genoma Agnóstico al Dominio (ADR-0108):** El motor evolutivo es agnóstico al dominio del genoma que recibe. Según el dominio activo declarado en `ACTIVE_GENOME_DOMAINS` (ver [`ast-compiler.md`](./ast-compiler.md)), la misma maquinaria de evolución (Parallel Islands, Blood Renewal, Convergence Detector, Fitness Metamórfico) opera sobre el Genoma de Señal (línea base, ADR-0043), el Genoma de Riesgo y Gestión de Posición (ADR-0109), el Genoma de Régimen y Filtro de Entorno (ADR-0110), o — en su modalidad de co-evolución de cartera — el Genoma de Portafolio y Correlación (ADR-0111). Los genomas de los dominios no activos viajan como entradas fijas a la evaluación de fitness ("Wildcard Invertido").

---

## Comportamientos Observables

- [ ] Usuario presiona "Generar candidatos" con parámetros de NSGA-II
  → Sistema crea 100 candidatos iniciales (aleatoriamente)
  → Evalúa cada uno: Sharpe=X, Drawdown=Y%, WinRate=Z%
  → Los 20 mejores (no-dominados) forman la Frontera Pareto inicial
  → **Evolución Multicanal (Parallel Islands):** Divide la población en 4 islas aisladas con migración selectiva cada 5 generaciones.
  → Evoluciona N generaciones: aplica cruce, mutación (Wildcard Mutations 5-10%) y poda.
  → **Blood Renewal:** Si el fitness se estancando, descarta el 50% más débil y renueva el 10% con genomas frescos.
  → **Convergence Detector:** Detiene la búsqueda si la mejora es < 0.1% durante 10 generaciones seguidas.
  → Devuelve Frontera Pareto final rehidratable desde AST v3.0.

- [ ] Usuario compara dos candidatos en Frontera Pareto
  → Candidato A: Sharpe=1.2, Drawdown=-15%, WinRate=52%
  → Candidato B: Sharpe=0.9, Drawdown=-8%, WinRate=48%
  → A es "mejor" en Sharpe, B es "mejor" en Drawdown — ambos están en Pareto

- [ ] Usuario lanza un ciclo de evolución sobre el Genoma de Riesgo y Gestión de Posición (ADR-0109) de una estrategia con Genoma de Señal ya validado
  → El motor evoluciona únicamente los nodos `wildcard_group` etiquetados para ese dominio
  → Cada candidato de la Frontera Pareto resultante reporta, además de Sharpe/Drawdown/WinRate, la atribución de fitness específica del Genoma de Riesgo y Gestión

- [ ] Usuario lanza un ciclo de co-evolución de cartera (ADR-0111) sobre un conjunto de Manifests ya validados individualmente
  → El motor opera sobre una población de configuraciones de cartera (población de poblaciones)
  → Cada individuo evaluado simula el conjunto completo de miembros bajo su Genoma de Portafolio y Correlación candidato
  → La Frontera Pareto resultante son configuraciones de cartera, no estrategias individuales

---

## Restricciones

- **NUNCA fitness objetivo devuelve NaN o Infinity.** Siempre números válidos.
- **NUNCA la Frontera Pareto contiene candidatos duplicados.** Cada candidato es único.
- **NUNCA se pierde la mejor solución entre generaciones.** Elitismo garantizado.
- **NUNCA se evoluciona más de un dominio no-Señal en la misma corrida (ADR-0108),** salvo la co-evolución de cartera del Dominio de Portafolio y Correlación (ADR-0111), cuya unidad evolutiva es el conjunto de Manifests, no un Manifest individual.
- **OBLIGATORIO:** cuando `ACTIVE_GENOME_DOMAINS` apunta a un dominio distinto de Señal, la evaluación de fitness de cada candidato debe incluir la atribución de la contribución de ese genoma al score final (ADR-0108), visible en el reporte de la Frontera Pareto.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace |
|---|---|---|---|
| POPULATION_SIZE | 100 | 50-500 | Candidatos totales por generación |
| GENERATIONS | 50 | 10-500 | Número máximo de generaciones |
| MUTATION_RATE | 0.2 | 0.05-0.5 | Probabilidad de mutación estándar |
| DECIMATION_COEFFICIENT | 2.0 | 1.0-5.0 | Ratio de sobre-generación en Gen 0 (ADR-0044) |
| RENEWAL_INTERVAL | 10 | 1-50 | Cada cuántas generaciones aplicar Blood Renewal |
| RENEWAL_PCT | 0.1 | 0.0-0.5 | % de población a renovar (Blood Renewal) |
| STAGNATION_TRIGGER | 20 | 5-100 | Reset total si no hay mejora en N generaciones |
| FITNESS_MODE | metamorphic | static/metamorphic | Modo de fitness (ADR-0042) |
| INDICATOR_WEIGHTS | {} | dict | Guided Search: Sesgo hacia indicadores específicos |
| CONVERGENCE_THRESHOLD | 0.001 | — | % de mejora mínima para continuar |
| LONG_SHORT_MODE | symmetric | sym/asymmetric | Reglas independientes por dirección (ADR-0041) |
| ACTIVE_GENOME_DOMAINS | [Señal] | Señal / Riesgo y Gestión / Régimen y Filtro / Portafolio y Correlación | Dominio(s) genómico(s) que esta corrida evoluciona; los demás dominios del Manifest viajan congelados (ADR-0108) |
| PORTFOLIO_COEVOLUTION_SIZE | 5 | 3-20 | Número de Manifests miembros co-evolucionados simultáneamente como una cartera (solo Fase C, ADR-0111) |
| OPERATION_HORIZON | any | any / scalping / intraday / swing / position / tick | Perfil de operación declarado; `any` no impone sesgo de frecuencia (agnóstico, ADR-0130). Ajusta el fitness y el objetivo de frecuencia |
| MIN_TRADES | configurable | 0-N | Mínimo de operaciones para validez estadística; estrategias por debajo se penalizan/rechazan (ADR-0130). Se combina con backtest multiactivo para alcanzar el N |
| FREQUENCY_OBJECTIVE | off | off / target / maximize | Si la frecuencia entra como objetivo del Pareto: encajar una frecuencia objetivo del horizonte, o maximizarla sin sacrificar robustez (ADR-0130) |

---

## Ciclo de Vida

### Entrada
- **Quién:** Generate (evolucionador de estrategias)
- **Recibe:** Población inicial, barras para evaluación, dominio(s) genómico(s) activo(s) (`ACTIVE_GENOME_DOMAINS`) y los genomas congelados de los demás dominios del Manifest (o del conjunto de Manifests, en co-evolución de cartera)

### Proceso
1. Evaluar fitness de cada candidato (Sharpe, drawdown, win rate), combinando el genoma del dominio activo con los genomas congelados de los demás dominios
2. Identificar Frontera Pareto (no-dominados)
3. Cruzar y mutar para nueva generación, restringido a los nodos `wildcard_group` del dominio activo
4. Repetir N generaciones
5. Atribuir, por candidato, la contribución de fitness del genoma del dominio activo (ADR-0108)

### Salida
- **Produce:** ParetoFront final con candidatos no-dominados, hypervolume y atribución de fitness por dominio genómico activo

---

## Tareas (TTRs)

### TTR-001: Evolución con Gestión de Diversidad (Herencia Generate)
*   **Descripción:** Ciclo completo de NSGA-II con **Parallel Islands**. Inicializa, evalúa y evoluciona. Si la calidad se estanca, aplica **Blood Renewal** (descarta `DECIMATION_RATIO`, renueva 10%) para evitar convergencia prematura.
*   **Entrada:** `DNA_pool`, `Market_Data`.
*   **Salida:** `Pareto_Front` (población no-dominada).
*   **Criterio de Éxito:** La lista final contiene al menos 50 candidatos con indicadores distintos (diversidad > 80%) y mejora de hypervolume respecto a Gen 0.

### TTR-002: Límite de Complejidad Granular (Anti-Sobreajuste / §3.3.10)
*   **Descripción:** Impone restricciones estructurales definidas en esquemas Serde para evitar el Curve Fitting.
*   **Campos de Validación:**
    - `min_conditions` / `max_conditions` (nodos de decisión).
    - `min_indicator_period` / `max_indicator_period`.
    - `max_lookback_shift` (desplazamiento temporal del indicador).
    - `max_exit_types` (Stop Loss, Take Profit, Trailing, TimeExits).
*   **Regla:** Penalización asintótica del fitness si cualquier límite es superado.
*   **Criterio de Éxito:** Ninguna estrategia en la Frontera Pareto viola las restricciones Serde.

### TTR-004: Implementación de Fitness Metamórfico y Ponderación (Fase 1 vs Fase 2)
*   **Descripción:** Programar la lógica que reescribe los pesos de la fórmula de aptitud basada en el `API Account Status`.
*   **Configuraciones (§3.3.10):**
    - **[OLD-SCHOOL] Weighted:** Diccionario estático normalizado a 1.0 (ej. `sharpe*0.4 - drawdown*0.3`).
    - **[NEW-ERA] Metamorphic:** Transmuta entre Fase 1 (Agresivo: Profit Factor/Retornos) y Fase 2 (Fondeada: Stability=1.0, penaliza Exposure y MAE).
*   **Guided Search:** Prioriza combinaciones que utilicen los indicadores definidos en `INDICATOR_WEIGHTS`.
*   **Criterio de Éxito:** El algoritmo responde dinámicamente al estado de la cuenta y a los sesgos de usuario sin re-programación.

### TTR-005: Coeficiente de Decimación (Purificación de Gen 0)
*   **Descripción:** Genera una población inicial expandida y elimina a los más débiles antes de la primera iteración evolutiva real.
*   **Criterio de Éxito:** La Gen 1 inicia con un fitness promedio superior a la media aleatoria.

### TTR-006: Evolución Multi-Dominio Simultánea y Atribución de Fitness por Genoma (ADR-0108)
*   **Descripción:** Generaliza el ciclo evolutivo para operar sobre el conjunto de dominios declarados en `ACTIVE_GENOME_DOMAINS` (cualquier subconjunto no vacío de Señal, Riesgo y Gestión de Posición — ADR-0109, Régimen y Filtro de Entorno — ADR-0110, Portafolio y Correlación — ADR-0111). Cada individuo de la población es un genoma compuesto de Reglas Genómicas (1..`MAX_CONDITIONS_PER_RULE` Genes de Condición AND/OR → 1..`MAX_ACTIONS_PER_RULE` Genes de Acción) que pueden mezclar genes de cualquiera de los dominios activos. Los genomas de dominios fuera de `ACTIVE_GENOME_DOMAINS` se reciben como entradas congeladas a la evaluación de fitness.
*   **Entrada:** `DNA_pool` compuesto de los dominios activos, genomas congelados de los dominios inactivos del Manifest, `Market_Data`.
*   **Salida:** `Pareto_Front` con atribución de fitness desglosada por dominio genómico de origen de cada gen.
*   **Criterio de Éxito:** La Frontera Pareto resultante reporta, para cada candidato, qué proporción de la mejora/deterioro de cada objetivo proviene de cada dominio activo frente a los genomas congelados, incluso cuando una Regla Genómica individual combina genes de más de un dominio activo.

### TTR-007: Co-evolución de Población de Carteras (ADR-0111)
*   **Descripción:** Modo de operación donde la unidad evolutiva es una configuración de cartera: un conjunto de `PORTFOLIO_COEVOLUTION_SIZE` Manifests miembros (ya validados individualmente) más un Genoma de Portafolio y Correlación candidato (activación/desactivación de miembros, rotación de pesos, cobertura sintética). Cada evaluación de fitness simula el conjunto completo de miembros bajo el genoma candidato. Este modo es ortogonal a TTR-006: cada Manifest miembro puede tener su propio `ACTIVE_GENOME_DOMAINS` (incluida la combinación multi-dominio de TTR-006) evolucionando simultáneamente con el Genoma de Portafolio y Correlación de la cartera.
*   **Entrada:** Conjunto de Manifests miembros validados, `DNA_pool` de configuraciones de cartera, `Market_Data` por miembro.
*   **Salida:** `Pareto_Front` de configuraciones de cartera, con métricas agregadas (Sharpe/drawdown/correlación de cartera) y desglose por miembro.
*   **Criterio de Éxito:** Ninguna configuración de cartera en la Frontera Pareto opera con menos de `PORTFOLIO_COEVOLUTION_SIZE` miembros activos simultáneamente sin que el Genoma de Portafolio y Correlación lo determine explícitamente.

### TTR-008: Objetivo de Frecuencia/Horizonte y Restricción de Mínimo de Operaciones (ADR-0130)
*   **¿Cuál es el problema?** Sin tratar la frecuencia como objetivo, el optimizador deriva a baja frecuencia (curvas más limpias con menos trades) y entrega estrategias de ~10 operaciones al mes en temporalidad de scalping — el sesgo de StrategyQuant que se quiere evitar.
*   **¿Qué tiene que pasar?** El usuario declara un perfil de operación (`OPERATION_HORIZON`) y, opcionalmente, activa la frecuencia como objetivo del Pareto (`FREQUENCY_OBJECTIVE`). Las estrategias con menos de `MIN_TRADES` se penalizan/rechazan; el N se alcanza combinando el backtest multiactivo ([`universal-basket-backtester`](./universal-basket-backtester.md)).
*   **¿Cómo sé que está hecho?**
    - [ ] Con `OPERATION_HORIZON = scalping` y `FREQUENCY_OBJECTIVE = target`, la Frontera Pareto contiene estrategias de alta frecuencia, no swing de 10 trades/mes.
    - [ ] Una estrategia bajo `MIN_TRADES` queda penalizada/rechazada.
    - [ ] Con `OPERATION_HORIZON = any` el motor no impone sesgo de frecuencia (agnosticismo, ADR-0130).
*   **¿Qué no puede pasar?** Maximizar trades a ciegas sacrificando robustez; ni sesgar estructuralmente el descubrimiento hacia una temporalidad (la agnosticidad de horizonte es invariante).

---

## Persistencia (Inundación de Fundamentos — ADR-0020)

Cada Job de optimización evolutiva y generación de población registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del Job de optimización |
| | `created_at` | Timestamp de inicio de la evolución |
| | `updated_at` | Timestamp de última actualización del Job |
| | `audit_hash` | Hash de la Frontera Pareto final |
| | `audit_chain_hash` | Hash de la secuencia de semillas genéticas |
| | `event_sequence_id` | Índice secuencial de generaciones evolutivas |
| **II. Soberanía** | `owner_id` | Usuario que lanzó el descubrimiento |
| | `institutional_tag` | Etiqueta de entorno (ADR-0020) |
| | `manifest_id` | ID del diseño de búsqueda legal |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del motor evolutivo (NSGA-II) |
| | `data_snapshot_id` | Puntero a los datos de entrenamiento |
| | `indicator_state_hash` | Snapshot del hypervolume/fitness promedio |
| | `version_node_id` | Versión en la base de conocimiento (DAG) |
| | `active_genome_domain` | Dominio genómico evolucionado por este Job (Señal / Riesgo y Gestión / Régimen y Filtro / Portafolio y Correlación) |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del worker de optimización |

## Gobernanza y Estándares (Fijos)

- **Registro de Dominios Genómicos (ADR-0108):** este motor es la única implementación evolutiva del sistema; toda nueva instancia de dominio (ADR-0109/ADR-0110/ADR-0111 y futuros dominios admitidos) lo reutiliza parametrizado por `ACTIVE_GENOME_DOMAINS`, sin forks ni motores paralelos.

---

## Dependencias

**Depende de:**
- `backtest-engine` (para evaluar fitness)

**Depende de ella:**
- `generate` (usa NSGA-II para evolucionar candidatos)
