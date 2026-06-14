# Strategy Ensemble — Síntesis de Generadores Múltiples

**Carpeta:** `./features/strategy-ensemble/`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0039 (Lógica Híbrida), ADR-0041 (Hemisferios de Asimetría), ADR-0113 (canal simbólico nativo — no PySR)

---

## ¿Qué es?

Orquesta canales (NSGA-II, Simbólico, HMM) en estrategias híbridas mediante **Fusión de Pareto** y **Mayoría Ponderada**, gestionando la **Asimetría Estructural** con hemisferios desacoplados.

**Problema:** NSGA-II (binario), Simbólico (alpha) y HMM (regímenes) requieren una síntesis automatizada para optimizar su combinación dinámica.

**User Story:** Como usuario, busco la generación automática de estrategias híbridas (NSGA-II, Simbólico, HMM) con hemisferios direccionales independientes.

---

## Comportamientos Observables

- [ ] **Desacoplamiento Direccional:** Permite configurar cruces de medias para Largos y Order Flow para Cortos.
- [ ] **Votación Ponderada:** Cálculo de `Grado de Confianza` independiente por hemisferio.
- [ ] Síntesis coherente de candidatos NSGA, ecuaciones simbólicas nativas y modelos HMM.
- [ ] Modos operativos: LIVE (conservador) y Descubrimiento (agresivo).
  → Sistema crea estrategia híbrida que:
    - Usa lógica NSGA como "condición de entrada base"
    - Ajusta peso de ecuaciones simbólicas nativas según régimen detectado (trending=100% alerta, choppy=50%)
    - Modo LIVE: conservador. Modo Descubrimiento: agresivo
  → Salida: HybridStrategy con 3 señales activas + voting rule

- [ ] En mercado TRENDING
  → HMM indica régimen=TRENDING (confianza 85%)
  → Pesos se adaptan: Simbólico signal trending=80%, autre signal=20%
  → Se ejecuta la entrada/salida base de NSGA + ecuación adaptada

- [ ] En mercado CHOPPY
  → HMM indica régimen=CHOPPY (confianza 70%)
  → Sistema baja confianza overall: requiere 2/3 señales alineadas (vs 1/2 en trending)
  → Reduce tamaño de posición para choppy

---

## Restricciones

- **NUNCA incluir un canal si su fitness es < umbral mínimo.** (Ej: Simbólico eq con correlación < 0.10, NSGA candidato con Sharpe < 0.5)
- **NUNCA forzar que un parámetro de Largo sea igual al de Corto.**
- **NUNCA promediar el fitness de ambos hemisferios si uno de ellos presenta resultados letales (Veto de supervivencia).**
- **Pesos de voting siempre suman 1.0** (si hay 3 canales: weight_nsga + weight_pysr + weight_hmm = 1.0).
- **Régimen HMM debe cambiar de forma explícita** (no oscilaciones cada barra, mínimo N barras de estabilidad).
- **No hay sobreexposición a un solo canal.** Max peso individual = 0.7, min = 0.1.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| VOTING_SCHEME | weighted | majority / weighted / consensus | Cómo combinar señales de los 3 canales | CONFIG |
| MIN_FITNESS_THRESHOLD | 0.5 | 0.0-1.0 | Mínimo fitness para incluir un canal en ensemble | CONFIG |
| ASYMMETRY_MODE | enabled | enabled / mirrored | Habilita o fuerza simetría direccional | [FIJO] |
| REGIME_STABILITY_BARS | 5 | 1-20 | Barras de estabilidad antes de cambiar régimen | CONFIG |
| MAX_WEIGHT_SINGLE_CHANNEL | 0.70 | 0.50-0.90 | Máximo peso que puede tener un solo canal | [FIJO] |
| MIN_WEIGHT_SINGLE_CHANNEL | 0.10 | 0.05-0.30 | Mínimo peso que recibe cada canal | [FIJO] |

---

## Ciclo de Vida de la Feature — Strategy Ensemble

### Entrada
- **Canal NSGA:** Candidato con configuración binaria (indicadores, umbrales, tamaño)
- **Canal Simbólico:** Ecuaciones simbólicas (1 o más) con fitness score
- **Canal HMM:** Modelo entrenado de detección de regímenes (TRENDING, MEAN_REVERTING, CHOPPY, LOW_VOLATILITY)
- **Estado de Hemisferio:** Parámetros y lógica diferenciada para Long/Short.
- **Parámetros de síntesis:** voting scheme, umbrales, pesos

### Proceso
1. **Consolidación Pareto:** Fusiona candidatos top de NSGA-II y Simbólico.
2. **Asignación Asimétrica:** Inyecta lógica y parámetros desacoplados por hemisferio.
3. **Selección NSGA-II:** Filtra candidatos no-dominados en Sharpe y DD.
4. **Inicialización Pesos:** Distribución base equitativa por régimen.
5. **Veredicto Ponderado:** Cálculo dinámico basado en alineación de canales y HMM.

### Salida
- **HybridStrategy:** Estrategia completa con:
  - SignalLogic: cómo se combinan los 3 canales
  - RegimeWeights: matriz de pesos por régimen
  - ActiveChannels: lista de canales incluidos
  - VotingRule: "2/3 aligned" en choppy, "1/3 aligned" en trending
- **EnsembleMetadata:** qué versiones de NSGA/Simbólico/HMM se usaron, fecha de síntesis

### Contextos de Uso

**Contexto 1: Generación de Candidatos (Módulo Generate)**
- Entrada: Frontera Pareto NSGA + Top ecuaciones simbólicas nativas + HMM modelo entrenado
- Pregunta: ¿Puedo sintetizar estos 3 canales en 1 estrategia inteligente?
- Impacto: Genera candidatos múltiples con distintos pesos/voting rules para exploración

**Contexto 2: Backtesting (Módulo Generate→Validate)**
- Entrada: Ensemble strategy candidata + datos históricos
- Pregunta: ¿El ensemble se comporta mejor que sus componentes?
- Impacto: Si ensemble es mejor que NSGA/Simbólico/HMM solos, avanza a validación

**Contexto 3: Validación Robusta (Módulo Validate)**
- Entrada: Ensemble strategy + período de validación OOS (out-of-sample)
- Pregunta: ¿El ensemble mantiene su adaptabilidad en nuevos datos?
- Impacto: Verifica que la síntesis no es overfitting a los 3 canales entrenados

---

## Tareas (TTRs)

### TTR-001: Fusión Multiobjetivo (Pareto Front Merging)
*   **Descripción:** Toma los frentes de Pareto de NSGA-II y las mejores ecuaciones simbólicas nativas para crear un Super-Pool de candidatos.
*   **Regla:** Aplica una nueva ronda de selección NSGA-II sobre el pool combinado.
*   **Criterio de Éxito:** El frente final de ensembles domina estadísticamente a los canales individuales.

---

### TTR-002: Integrar Detección de Régimen (HMM)

**Qué hace:** Toma la estrategia NSGA+Simbólico y adapta sus pesos según régimen HMM detectado.

**Entrada:**
- HybridStrategy (del TTR-001)
- HMM modelo entrenado
- Matriz de pesos por régimen (ej: TRENDING={NSGA: 0.6, Simbólico: 0.4}, CHOPPY={0.5, 0.5})

**Salida:**
- RegimeAdaptiveStrategy con pesos dinámicos
- Tabla de transiciones: cuándo cambia régimen, cómo cambian pesos

---

### TTR-003: Validar Coherencia del Ensemble

**Qué hace:** Asegura que los 3 canales no contradicen entre sí de forma patológica.

**Entrada:**
- HybridStrategy completa
- Período histórico para test (ej: 252 barras)

**Salida:**
- CoherenceReport: ¿Los canales están alineados o constantemente en conflicto?
- Métrica: "Alineación promedio entre canales" (si es < 0.1, muy conflictivo)

---

### TTR-004: Implementación de Hemisferios Independientes
- **Qué tiene que pasar:** Modificar la estructura de la `HybridStrategy` para que contenga dos contenedores de lógica funcional desacoplados (`LongHemisphere`, `ShortHemisphere`).
- **Criterio de éxito:** Poder ejecutar una optimización donde el periodo del RSI sea 14 para Largos y 2 para Cortos sin conflicto.

---

## Dependencias

**Depende de:**
- `nsga2-optimizer` (canales NSGA candidatos)
- `symbolic-signal-discovery` (canal Simbólico Nativo, regresión simbólica libre) — *moonshot feature*
- `hmm-regime-detection` (modelos HMM entrenados)
- `backtest-engine` (para evaluar ensemble en histórico)

**Depende de ella:**
- `validate` (valida robustez del ensemble)
- `generate` (usa ensemble como generador final)

---

## Gobernanza y Estándares

- **Inundación de Fundaciones (ADR-0020 V2):** 

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la síntesis |
| | `created_at` | Timestamp de ensamble |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de la estrategia híbrida |
| | `audit_chain_hash` | Link a la integridad del bus de barras |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **II. Soberanía** | `owner_id` | Dueño de la IP colectiva |
| | `manifest_id` | ID del contrato de diseño del ensemble |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del motor de fusión (Weighted Voting) |
| | `parent_id` | ID del linaje de los 3 canales (NSGA/Simbólico/HMM) |
| | `indicator_state_hash` | Snapshot de los pesos por régimen |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del orquestador de síntesis |
| **V. Forense** | `source_signal_id` | Identificador del hemisferio que disparó la orden |

