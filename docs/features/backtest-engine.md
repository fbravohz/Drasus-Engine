# Backtest Engine — Motor de Simulación Histórica

**Carpeta:** `./features/backtest-engine/`
**Estado:** Lista para implementar
**Última actualización:** 2026-04-08

---

## ¿Qué es?

El Backtest Engine simula cómo se habría comportado una estrategia en el pasado, usando datos históricos reales de barras. Devuelve métricas de desempeño (PnL, Sharpe, drawdown, etc.) que el usuario puede usar para evaluar si la estrategia merece ser probada en producción.

**Problema:** Evaluar estrategias sin simulación es adivinar. Necesitas saber: ¿hubiera ganado dinero en 2020? ¿Y en 2021? ¿Sobrevivió a la crisis de March 2020?

**Solución:** Simular la estrategia contra datos históricos con un **motor dual** (ADR-0114):
- **Ruta Express (exploración):** enfoque híbrido — pre-cálculo **vectorizado** columnar (Polars/SIMD) de la lógica sin estado (indicadores, señales) + **mini-loop secuencial plano** en Rust para la lógica con estado (sizing, stops dinámicos, equity). Rápida; sesgo pesimista por diseño.
- **Ruta Event-Driven (fidelidad):** crates nativos de NautilusTrader v2 (ADR-0107), ticks reales o 4-ticks, paridad simulación/vivo.

**La ruta la elige el usuario** mediante el parámetro `ENGINE_MODE` del contrato; el sistema no fuerza la promoción de una a otra.

---

## Comportamientos Observables

- [ ] Usuario presiona "Backtest" en una estrategia con 2 años de datos históricos
  → Motor elige modo de fidelidad:
    1. **Tick-by-Tick Vectorizado:** Precisión milimétrica sobre cada trade individual.
    2. **Every Tick (4-ticks/1M):** Alta fidelidad para SL/TP.
    3. **Real Ticks:** Uso de historial real de ticks del exchange.
    4. **1 Minute OHLC:** Reconstitución de temporalidades mayores.
    5. **Open Prices Only:** Máxima velocidad para optimización.
  → Aplica fricción institucional: spread variable, comisiones, triple swap.
  → **Bar-Open Alignment:** Garantiza que las señales se generen y ejecuten al abrirse la vela (Bar Open).
  → **Warm-up & Gap Handling:** Calentamiento automático de indicadores y manejo de datos faltantes (`FillFlat`/`Ignore`).
  → Ejecuta ruteo inteligente via [volume-profile-router](./volume-profile-router.md).
  → Devuelve: Sharpe, drawdown máximo, win rate, número de trades, duración en ms.

- [ ] Usuario compara dos backtests
  → Primer backtest: modo ticks simulados, resultado Sharpe=1.2
  → Segundo backtest: modo apertura sólo, resultado Sharpe=1.5
  → Mayor fidelidad (ticks) → resultado más conservador (lower Sharpe)

- [ ] Backtest genera DSR score (Probabilidad de Sobreajuste Deflated)
  → Compara PnL in-sample vs out-of-sample
  → Si DSR baja (ej: 0.3) → estrategia probablemente overfitted
  → Si DSR alta (ej: 0.8) → estrategia más robusta

---

## Restricciones

- **NUNCA se ejecuta un backtest sin barras validadas.** Barras inválidas corrompen los resultados.
- **NUNCA se omite slippage en la simulación.** Siempre simular de forma realista (no "prix exacto").
- **NUNCA se borra un BacktestResult.** Historial completo para auditoría.
- **NUNCA el motor modifica el estado de la estrategia durante simulación.** Backtest es read-only.
- **Contrato de Consistencia Conservadora (ADR-0114, FIJO):** la Ruta Express NUNCA puede ser más optimista que la Event-Driven. Se garantiza por: **(a) Bar-Open Alignment** —señal calculada al cierre de la barra N se ejecuta en la apertura de N+1, sin precios intermedios— y **(b) Regla Intrabar Pesimista** —si SL y TP se tocan en la misma vela, se asume SIEMPRE que el Stop Loss saltó primero—.
- **Frontera Sin-Estado / Con-Estado (ADR-0114, FIJO):** ningún cálculo dependiente del estado de cuenta/posición corre en la sub-fase vectorizada; la lógica con estado (sizing, stops, Dominio de Riesgo y Gestión ADR-0109) corre en el mini-loop secuencial.
- **Agnosticismo de Temporalidad (ADR-0130, FIJO):** el motor simula scalping, intradía, swing, posición y **ticks** como ciudadanos de primera clase, sin sesgo estructural hacia ninguna temporalidad. Bajar la temporalidad debe aumentar el número de oportunidades evaluadas, nunca reducirlo artificialmente.
- **Posiciones Concurrentes (ADR-0129, FIJO):** el motor modela N posiciones concurrentes por estrategia/activo (no una sola), con contabilidad de margen agregada y P&L por ticket, para que el conteo de trades refleje la semántica no bloqueante. `MAX_CONCURRENT_POSITIONS = 1` reproduce el comportamiento clásico.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace |
|---|---|---|---|
| ENGINE_MODE | Por contexto (anulable) | Express \| EventDriven | Ruta de motor elegida por el llamador/usuario (ADR-0114). Express = híbrido vectorizado+secuencial; EventDriven = NT v2 alta fidelidad. |
| FIDELITY_MODE | SIMULATED_TICKS | Ver opciones | 4 ticks/barra, reales, 1m reconstituidas, apertura sólo |
| SLIPPAGE_PERCENT | 0.0001 | 0.0-0.01 | Costo de slippage en % del precio |
| COMMISSION_PERCENT | 0.0005 | 0.0-0.01 | Comisión por trade en % |
| INITIAL_CAPITAL | 100000 | > 0 | Capital inicial de simulación |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Motor de matching determinista, algoritmos de cálculo de métricas (Sharpe, DD, DSR) y modelos de fricción en `engine_core.rs`. Sin acceso a DB/IO.
- **Shell (Infraestructura):** Orquestador de NautilusTrader, cargadores de datos Arrow y persistencia de resultados. `engine_shell.rs`.
- **Frontera Pública:** Contrato `run_backtest(strategy, data, config)`.

## Ciclo de Vida de la Feature — Backtest Engine

### Entrada
- **Sujeto:** `ExecutableContainer` (Versioned Strategy/Portfolio).
- **Datos:** Dataset OHLCV/Ticks validado ([data-sanitizer-pipeline](./data-sanitizer-pipeline.md)).
- **Configuración:** Modo de fidelidad, parámetros de capital, modelos de slippage/comisiones.

### Proceso
- **Initial Setup:** Carga de datos OHLCV en memoria compartida (Arrow).
- **Dry-Run:** Calentamiento de indicadores (Warm-up) para alcanzar estado estable.
- **Matching Loop:** NautilusTrader recorre el dataset emulando la microestructura del mercado.
- **Metrics Aggregation:** Cálculo vectorial de métricas de rendimiento y robustez (DSR/PBO).

### Salida
- **BacktestResult:** Objeto inmutable con metadatos de desempeño, rastro de trades y equity curve.
- **Purity Check:** Verificación de integridad bit-a-bit del resultado.

### Contextos de Uso

**Contexto 1: Optimización de Genes (Módulo Generate)**
- Entrada: Candidatos rápidos + Datos reducidos.
- Objetivo: Evaluar el fitness score para la siguiente generación.

**Contexto 2: Validación de Robustez (Módulo Validate)**
- Entrada: Candidatos aprobados + Datos OOS (Out-of-Sample).
- Objetivo: Producir el veredicto definitivo antes de incubación.

**Contexto 3: Validación de Portafolio (Módulo Manage)**
- Entrada: Combinación de pesos + Datos históricos agregados.
- Objetivo: Probar la correlación y el drawdown del portafolio completo.

---

## Tareas (TTRs)

### **TTR-001: Ejecución Simétrica bit-a-bit (Nautilus-Ready)**
*   **Descripción:** Orquesta el motor de **NautilusTrader** usando reconstrucción de **4-ticks por barra OHLC** (ADR-0017).
*   **Reglas de Negocio:**
    * El resultado DEBE ser reproducible al 100% (mismo input → mismo output).
    * Obligatorio aplicar `Triple Swap` y `Slippage` dinámico.
*   **Entrada:** `strategy_dna`, `ohlcv_data` (Arrow), `slippage_model`.
*   **Salida:** `TradeLog`, `EquityCurve`.
*   **Precondición:** Datos OHLCV con integridad verificada por hash.
*   **Postcondición:** Emisión de `audit_hash` del resultado final (ADR-0020 V2).

### **TTR-002: Cálculo de Métricas y Probabilidad de Overfitting**
*   **Descripción:** Calcula Sharpe, MaxDD, DSR y PBO (Probability of Overfitting).
*   **Reglas de Negocio:**
    * Penalizar métricas según el número de parámetros optimizados (DSR).
    * Resultados persistidos con referencia al `process_id` del job.
*   **Entrada:** `TradeLog`.
*   **Salida:** `MetricsDict` (Sharpe, Drawdown, DSR, PBO).
*   **Precondición:** Trades cerrados reconciliados.
*   **Postcondición:** Persistencia con `version_node_id` vinculado a la estrategia.

### **TTR-003: Simulador Tick-by-Tick Vectorizado**
*   **Descripción:** Implementa un motor de ejecución paralela que procesa cada movimiento individual (tick) para determinar con precisión qué orden se llenó primero en condiciones de alta volatilidad.
*   **Reglas de Negocio:**
    * Debe detectar "Flash Crashes" y gaps de liquidez usando el `volume-profile-router`.
    * Aceleración nativa via Rust SIMD/Rayon; criterio de rendimiento relativo (más rápido que MT5/SQX/QuantConnect en igual hardware, ADR-0114; sin KPI absoluto).
*   **Entrada:** Tick data stream, Active Orders list.
*   **Salida:** Precision Execution Logs.

### **TTR-004: Simulación de Posiciones Concurrentes y Agnosticismo de Temporalidad (ADR-0129/0130)**
*   **Descripción:** El motor simula N posiciones concurrentes por estrategia/activo (entradas no bloqueantes por defecto + de-duplicación de señal) sobre cualquier temporalidad o ticks, con margen agregado y P&L atribuido por ticket.
*   **Reglas de Negocio:**
    * Respeta `MAX_CONCURRENT_POSITIONS` y `SIGNAL_DEDUP_BARS` (ADR-0129); `=1` reproduce "una posición a la vez".
    * El margen/exposición se validan sobre el agregado de posiciones concurrentes (HARD, ADR-0010/0025).
    * Ninguna temporalidad recibe trato preferente; bajar el timeframe aumenta las oportunidades evaluadas (ADR-0130).
*   **Entrada:** `strategy_dna`, dataset (OHLCV/ticks) de cualquier temporalidad, set de posiciones abiertas.
*   **Salida:** `TradeLog` con trades concurrentes y conteo coherente con la temporalidad.
*   **Precondición:** TTR-001/TTR-003 disponibles.
*   **Postcondición:** Métricas (incl. número de trades) reflejan la semántica no bloqueante y la temporalidad real.

---

## Gobernanza y Estándares (Fijos)
## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Toda ejecución de backtest y resultado registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del backtest |
| | `created_at` | Timestamp de ejecución |
| | `updated_at` | Última actualización del resultado |
| | `audit_hash` | Hash del PnL reportado |
| | `audit_chain_hash` | Hash del stream de eventos matching |
| | `event_sequence_id` | Secuencia del evento de backtest |
| **II. Soberanía** | `owner_id` | Usuario responsable del capital simulado |
| | `manifest_id` | ID del diseño de la estrategia |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del motor (Nautilus/Polars version) |
| | `data_snapshot_id` | Ref al dataset histórico utilizado |
| | `indicator_state_hash` | Snapshot de métricas (Sharpe/DD) |
| | `version_node_id` | Versión de la estrategia en el DAG |
| **IV. Hardware** | `node_id` | ID del hardware físico ejecutor |
| | `process_id` | PID del motor de simulación |

- **Decisión Arquitectónica Asociada:**
    - ADR-0013: Stack Tecnológico (NautilusTrader).
    - ADR-0017: Simulación de Alta Fidelidad.
    - ADR-0020 V2: Inundación de Fundaciones.
    - ADR-0107: Integración nativa NT v2 (ruta Event-Driven).
    - ADR-0114: Motor dual, ruta Express híbrida, modo elegido por el usuario y contrato de consistencia conservadora.
    - ADR-0129: Entradas Concurrentes No Bloqueantes + De-duplicación de Señal (N posiciones simuladas).
    - ADR-0130: Frecuencia/Horizonte de Operación + Agnosticismo de Temporalidad.

---

## Preparación para Opciones (Post-MVP — ADR-0140)

> **Estado:** Diferido. No implementar hasta que los cinco prerrequisitos de ADR-0140 se cumplan.

El motor dual (ADR-0114) está diseñado para instrumentos lineales. Las opciones introducen desafíos específicos:

- **Ruta Express (vectorizada):** no aplica directamente a opciones. Las opciones no tienen un solo precio — tienen una cadena de strikes × vencimientos, cada uno con su propio bid/ask. La vectorización columnar sobre OHLCV no modela payoffs no-lineales.
- **Ruta Event-Driven (NautilusTrader):** NT sí soporta opciones nativamente (ADR-0107). El puente anticorrupción (`nautilus-integration`) debe extenderse para mapear `OptionContract` y `OptionChain` a tipos Drasus Engine.
- **Contrato de Consistencia Conservadora:** la "Regla Intrabar Pesimista" (SL antes que TP) no tiene análogo claro en opciones, donde el payoff es no-lineal y depende del precio al vencimiento, no de precios intrabar.

**Refactorización necesaria:** extender la Ruta Event-Driven para opciones vía NT; repensar la Ruta Express para estrategias de opciones (posiblemente como simulación Monte Carlo en lugar de vectorización columnar).

**Moonshots asociados:** [`option-pricing-engine`](../moonshots/option-pricing-engine.md), [`exercise-assignment-handler`](../moonshots/exercise-assignment-handler.md).

---

## Dependencias
**Depende de:**
- [`clock`](../features/clock.md) — para timestamps deterministas.
- [`slippage-models`](../features/slippage-models.md) — para realismo de ejecución.

**Consumido por:**
- [`validate`](../modules/validate.md) — para Validate.
- [`generate`](../modules/generate.md) — para optimización genética.
- [`portfolio-optimizer`](../features/portfolio-optimizer.md) — para backtesting de portafolio agregado.
