# Simulador Monte Carlo (Robustez — Modo Dual)

**Carpeta:** `./features/monte-carlo-simulator/`
**Estado:** Especificación (Actualizado para Scoring Ponderado)
**Última actualización:** 2026-06-11
**Decisión Arquitectónica Asociada:** ADR-0058 (Scoring Ponderado de Robustez), ADR-0061 (Motor MC), ADR-0109, ADR-0111, ADR-0112 (cómputo CPU-first, `tch-rs` erradicado)

---

## ¿Qué es?

Es un analizador estadístico de permutación y remuestreo que opera en **tres modos independientes** dentro del guantelete de robustez. El Scoring Ponderado (ADR-0058) distingue entre la robustez ante variaciones en el orden de ejecución y la supervivencia ante eventos diarios letales de Prop Firms mediante el **Embudo Tóxico de Estrés** (ADR-0061). A partir de ADR-0108, este simulador es además la **compuerta de robustez bloqueante** de los Genomas de los Dominios de Riesgo y Gestión de Posición (ADR-0109) y de Portafolio y Correlación (ADR-0111).

### Modos de Operación

**Modo 1 — HPC Monte Carlo (Institucional):** Evalúa si el éxito de la estrategia depende del orden secuencial de los trades (suerte) mediante permutación masiva vectorizada. Peso en el scoring: 25%. Cuando el Manifest tiene activo un Genoma de Riesgo y Gestión de Posición (ADR-0109), este modo incorpora la **Réplica de Estado de Riesgo** (ver Proceso, paso 11).

**Modo 2 — Embudo Tóxico de Estrés (Risk-Prop FirmMC):** Modo condicional severísimo que simula entornos dictatoriales (FTMO/Darwinex). Destruye cohortes que atraviesen límites diarios letales absolutos (Drawdown > 4.5% Intradiario). Peso en el scoring: 20%.

**Modo 3 — Monte Carlo de Desfase Temporal (Cartera, ADR-0111):** Modo aplicable exclusivamente a conjuntos de Manifests con un Genoma de Portafolio y Correlación activo. Remuestrea, para cada miembro de la cartera, un desfase temporal independiente sobre su propia secuencia de operaciones, y recombina las curvas de equidad desfasadas para evaluar si la correlación, el drawdown agregado y el Sharpe conjunto observados en el backtest de cartera son un artefacto de la alineación temporal del histórico o si persisten bajo desalineaciones plausibles. Produce un veredicto de robustez de cartera independiente, no compite por el presupuesto del scoring ponderado de la estrategia individual (ADR-0058).

---

## Ciclo de Vida de la Feature — Monte Carlo

### Entrada
- Lista de trades resultantes de un backtest (Profit/Loss, Duración, Timestamp).
- Número de iteraciones (ej: 10,000).
- Modo de operación (TRADES o TÓXICO).

### Proceso
- **MODO 1: HPC Monte Carlo — Robustez Decagonal (Master Suite):**
  Genera miles de variantes de la curva de equidad original aplicando una cascada de perturbaciones granulares y estratégicas:

  1. **Trade Reordering / Reshuffle:** Baraja el orden aleatoriamente para destruir la falsa seguridad de curvas lineales.
  2. **Micro-Tick Reshuffling:** Permuta micro-movimientos intra-vela para validar sensibilidad al "path dependency" del precio.
  3. **Data Perturbation (Métricas):** Altera sutilmente métricas extraídas (ej. de trade ganador a break-even) en lugar de inyectar ruido a velas crudas, preservando la integridad del linaje.
  4. **Slippage & Spread Variation (Stress):** Duplica el slippage y comisiones en tiempo real. Simula deslizamientos agresivos (hasta 3σ).
  5. **Volatility Shocks (OHLC-ATR Noise):** Inyecta ruido en precios basado en ATR real (3.5x ATR en inyecciones de choque) y duplica spreads en momentos aleatorios.
  6. **Equity Noise / Reshuffling:** Agrega varianza estocástica y permutación a la secuencia de PnL para evaluar la dependencia del orden temporal.
  7. **Outlier Removal & Pruning:** Recalcula eliminando el top 5% de windfall profits o el 1% de trades extremos para medir dependencia de eventos afortunados únicos.
  8. **Input Removal / Randomize Inputs:** Aleatoriza las señales de entrada manteniendo la gestión de posición para aislar el mérito de la lógica de entrada. Evalúa si sobrevive sin su mejor trade.
  9. **Event Skipping / Account Breaks:** Omite eventos aleatorios o simula desconexiones/gaps de ejecución para asegurar que la rentabilidad no dependa de un solo trade.
  10. **Dynamic MC Position Sizing:** Recálculo obligatorio del lotaje en cada shuffle según el capital variante (+/- 20% varianza) para exponer margin calls ocultos.
  11. **Réplica de Estado de Riesgo (ADR-0109, condicional):** Cuando el Manifest tiene un Genoma de Riesgo y Gestión de Posición activo, este paso generaliza el #10: en cada iteración de remuestreo, la máquina de estados de los Genes de Condición de ese genoma (drawdown de equity, racha de pérdidas/ganancias, duración del drawdown, duración de operación, múltiplo-R no realizado) se **re-simula desde cero** sobre la secuencia reordenada/perturbada, re-evaluando en cada paso qué Genes de Acción de mutación de tamaño y de morfología de salida se hubieran disparado bajo esa secuencia alternativa. El sizing y la estructura de salida de cada trade simulado dejan de heredarse del backtest histórico y pasan a ser una salida de esta re-simulación.

- **MODO 2: Embudo Tóxico de Estrés (Risk-Prop FirmMC):**
  Simulación condicional severísima orientada a supervivencia institucional (FTMO/Darwinex):
  - **Aislamiento por Cohortes Diarias:** Aísla días individuales de la curva de equidad original.
  - **Inyección de Eventos Letales:** Sintetiza gaps de 10x ATR, spreads duplicados instantáneamente y latencias de ejecución > 5 segundos.
  - **Muerte Súbita (Compliance):** Destruye la mutación completa si atraviesa límites diarios absolutos (Drawdown > 4.5% Intradiario). Ignora drawdowns relativos temporales macro.
  - **Tasa de Supervivencia:** Calcula en qué porcentaje de días/mutaciones la estrategia violó los límites, vinculado al `PropFirmComplianceConfig` (ADR-0045).

- **Capa de Fricción: Física de Broker (Broker Physics):**
  - **Randomize Min Distance:** Aleatorización del salto para órdenes limit y pending stop.
  - **Randomize Slippage/Spread Range:** Rango configurable (ej. 0.0-4.0 pips) simulando falta de volumen.

- **Visualización Dual (Visual Mode Dashboard):**
  - **OPCIÓN A: Spaghetti Literal (Exploración Manual):** Dibuja TODAS las N iteraciones (ej. 1,000 curvas) con opacidad baja (alpha=0.05). Colormap: Verde (Profit) → Rojo (Pérdida). Identifica visualmente "densidad" y Cisnes Negros.
  - **OPCIÓN B: Confidence Cone (Decisión Cuantitativa):** Proyecta bandas de percentiles configurables (P5, P10, P25, P50, P75, P90, P95, P99). Define el área sombreada para el scoring automático.

- **Análisis de Distribución:** Calcula percentiles por cada punto temporal para scoring multi-criterio.

- **MODO 3: Monte Carlo de Desfase Temporal (Cartera, ADR-0111):**
  Aplicable únicamente a un conjunto de Manifests con un Genoma de Portafolio y Correlación activo:
  1. **Desfase Independiente por Miembro:** Para cada Manifest miembro de la cartera, aplica un desplazamiento temporal aleatorio (configurable) sobre su propia secuencia de operaciones, manteniendo intacto el orden interno de sus trades.
  2. **Recombinación de Curvas Desfasadas:** Reconstruye la curva de equidad agregada de la cartera a partir de las curvas individuales desfasadas.
  3. **Recálculo de Genes de Condición Cruzada:** Vuelve a evaluar, sobre la cartera recombinada, la correlación móvil entre miembros, el drawdown agregado, la volatilidad de cartera y el solapamiento direccional simultáneo.
  4. **Distribución de Resultados:** Repite el proceso N veces, generando una distribución de correlación, drawdown agregado y Sharpe conjunto bajo desalineaciones temporales plausibles.

### Salida
- **Modo TRADES:** Confidence Interval Metrics (Sharpe, DD y Retorno esperados con 95% de confianza), Probabilidad de Ruina.
- **Modo TÓXICO:** Daily Survival Rate (% de días sin violación de límites), Worst Day Impact (pérdida del peor día simulado), Toxic Compliance Status (COMPLIANT / VIOLATION_RISK / TOXIC).
- **Réplica de Estado de Riesgo (ADR-0109):** Distribución, por cada Gen de Acción del Genoma de Riesgo y Gestión, de cuántas veces y bajo qué secuencias se habría disparado, junto con el drawdown resultante de cada trayectoria de mutación.
- **Modo Desfase Temporal (ADR-0111):** Distribución de correlación agregada, drawdown de cartera y Sharpe conjunto bajo desalineaciones temporales, junto con un veredicto de robustez de cartera (`portfolio_robustness_verdict`: COMPLIANT / CORRELATION_FRAGILE / RECHAZADA).
- **Consolidado:** `robustness_verdict` (COMPLIANT / PROP_FIRM_FRAGILE / TOXIC / RECHAZADA) exportado e inundado en los fundamentos de persistencia (ADR-0020) para consumo del `Pre-Trade Validator` (ADR-0095).

### Contextos de Uso

**Contexto 1: Stress Test de Robustez (Módulo Validate — Modo TRADES)**
- Determina si el éxito de la estrategia depende del orden secuencial de los trades (suerte) o si es robusta ante variaciones.
- Contribuye con 25% al score ponderado final.

**Contexto 2: Supervivencia ante Eventos Letales (Módulo Validate — Modo TÓXICO)**
- Evalúa si la estrategia sobrevive a los límites diarios de Prop Firms bajo condiciones adversas simuladas.
- Contribuye con 20% al score ponderado final.
- Si la tasa de supervivencia diaria es < 80%, la estrategia es marcada como `PROP_FIRM_FRAGILE`.

**Contexto 3: Validación del Genoma de Riesgo y Gestión de Posición (Módulo Validate — ADR-0109)**
- Compuerta bloqueante: ningún Manifest con Genoma de Riesgo y Gestión activo avanza a "En Incubación" (SAD §12) sin pasar por la Réplica de Estado de Riesgo.
- No reemplaza al Modo TRADES; lo extiende cuando el genoma de riesgo está presente.

**Contexto 4: Validación del Genoma de Portafolio y Correlación (Módulo Validate — ADR-0111)**
- Compuerta bloqueante: ningún conjunto de Manifests con Genoma de Portafolio y Correlación activo avanza a "En Incubación" (SAD §12) sin pasar por el Modo de Desfase Temporal.
- Se ejecuta sobre el conjunto de la cartera, no sobre estrategias individuales.

---

## Tareas (TTRs)

### TTR-001: Motor de Resampling (Bootstrapping Vectorizado CPU-First)
*   **Descripción:** Generador masivo de curvas de equidad sintéticas. **CPU-first** vía `ndarray` + **Rayon/Rust SIMD** (ADR-0112): la permutación de trades es barajado de matrices, no deep learning, por lo que no requiere GPU ni libtorch. `tch-rs` queda erradicado.
*   **Acelerador Opcional (ADR-0112):**
    * Una GPU vía `candle` (Rust puro, CUDA/Metal dinámico) solo se considera si un benchmark demuestra que la CPU no alcanza el tiempo objetivo.
    * La ausencia de GPU jamás impide la ejecución; toda carga corre en CPU.
    * El determinismo bit-a-bit (semilla fija, ADR-0107) se preserva en cualquier ruta.
*   **Criterio de Éxito:** 10K iteraciones en tiempo acotado y configurable en CPU; rendimiento competitivo sin depender de hardware gráfico.

### TTR-002: Filtro de Calidad Automático (Quality Gate Multi-Criterio)
*   **Descripción:** Motor de decisión que evalúa el cumplimiento de umbrales configurables por el usuario sobre los percentiles Monte Carlo.
*   **Regla:** Requisito configurable (ej: P25 > Breakeven AND P5 > -15% Drawdown). Si no se cumple, el veredicto es `RECHAZADA`.

### TTR-003: Generación de Cono de Confianza
*   **Descripción:** Proyecta la probabilidad de resultados futuros basados en la varianza de las simulaciones.
*   **Salida:** Array de percentiles para visualización en el Dashboard (Fase 4).

### TTR-004: Réplica de Estado de Riesgo (ADR-0109)
*   **Descripción:** Extiende el Modo 1 para Manifests con Genoma de Riesgo y Gestión de Posición activo. En cada iteración de remuestreo, mantiene y re-evalúa la máquina de estados completa de los Genes de Condición de ese genoma (rachas, drawdown, duraciones, múltiplo-R) sobre la secuencia perturbada, re-disparando los Genes de Acción de sizing y morfología de salida correspondientes en lugar de reutilizar los valores históricos.
*   **Criterio de Éxito:** Para un mismo Manifest y semilla, dos ejecuciones producen trayectorias de mutación de riesgo idénticas (determinismo bit-a-bit, ADR-0107); cambiar `ACTIVE_GENOME_DOMAINS` a un genoma de riesgo distinto produce trayectorias diferentes sobre las mismas secuencias remuestreadas.
*   **Bloqueante:** Ningún Manifest con Genoma de Riesgo y Gestión activo puede avanzar a "En Incubación" (SAD §12) sin que este TTR se haya ejecutado y reportado.

### TTR-005: Monte Carlo de Desfase Temporal de Cartera (ADR-0111)
*   **Descripción:** Implementa el Modo 3 sobre un conjunto de Manifests co-evolucionados bajo un Genoma de Portafolio y Correlación. Aplica desfases temporales independientes por miembro, recombina las curvas de equidad resultantes y recalcula los Genes de Condición Cruzada (correlación móvil, drawdown agregado, volatilidad de cartera, solapamiento direccional) sobre cada recombinación.
*   **Criterio de Éxito:** El reporte distingue cuánto de la correlación/drawdown observados en el backtest conjunto persiste bajo desalineaciones plausibles frente a cuánto es artefacto de la alineación temporal específica del histórico.
*   **Bloqueante:** Ningún conjunto de Manifests con Genoma de Portafolio y Correlación activo puede avanzar a "En Incubación" (SAD §12) sin que este TTR se haya ejecutado y reportado.

---

## Persistencia (Inundación de Fundamentos — ADR-0020)

Cada reporte de robustez y simulación Monte Carlo registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del reporte MC |
| | `created_at` | Timestamp de inicio de simulación |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de integridad del reporte final |
| | `audit_chain_hash` | Hash de la secuencia de semillas |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **II. Soberanía** | `owner_id` | Usuario que ejecutó la prueba |
| | `institutional_tag` | Tag de cumplimiento (AUDIT/R&D) |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del motor del simulador |
| | `data_snapshot_id` | Ref a los trades base analizados |
| | `indicator_state_hash` | Estado técnico (Peor Escenario Posible) |
| | `version_node_id` | Versión de la estrategia evaluada |
| **IV. Hardware** | `node_id` | ID del hardware físico (CPU/GPU) |
| | `process_id` | PID del proceso worker |
| | `execution_latency_ms` | Tiempo total de simulación |

## Gobernanza y Estándares (Fijos)

- **Compuertas Bloqueantes de Dominio (ADR-0108):** la Réplica de Estado de Riesgo (ADR-0109) y el Monte Carlo de Desfase Temporal (ADR-0111) son condiciones necesarias —no opcionales— para que un Manifest o conjunto de Manifests con esos genomas activos avance en el Lifecycle (SAD §12). Resultados de Modo 1/Modo 2 sin estas extensiones son insuficientes cuando esos genomas están presentes.
- **Determinismo Bit-a-Bit (ADR-0107):** toda re-simulación de máquina de estados (Réplica de Estado de Riesgo) o recombinación de curvas (Desfase Temporal) debe ser reproducible exactamente dado el mismo `manifest_id`/conjunto de `manifest_id`s y la misma semilla.

---

## Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** Algoritmos de permutación y cálculo de percentiles stats en `simulation.rs`.
- **Shell (Infraestructura):** Cargador de reportes de trade.
- **Frontera Pública:** Contrato `run_monte_carlo(trades_list, iterations)`.

---

## Dependencias
**Consumido por:** `validate`, `feedback`.
**Depende de:** `institutional-metrics`, [`ast-compiler.md`](./ast-compiler.md) (lectura del Manifest para identificar el dominio genómico activo y los genomas congelados, ADR-0108).
