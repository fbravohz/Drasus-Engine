# Equity Curve Tracker — Tracking Continuo de Capital y PnL

**Carpeta:** `./features/equity-curve-tracker/`
**Estado:** Crítico / Fundacional
**Última actualización:** 2026-04-12

---

## ¿Qué es?

Mantiene un registro barra-por-barra (o tick-por-tick) del capital, beneficio/pérdida, y drawdown máximo consumiendo los eventos de `PositionClosed` y `OrderFilled` de **NautilusTrader** (ADR-0013) durante backtesting o trading live.

**Problema:** Si no sabes cuánto capital tienes en cada momento, no puedes calcular Sharpe, no puedes detectar cuando estás en drawdown, no puedes tomar decisiones. Equity Curve Tracker es la columna vertebral de la auditoría de performance.

**User Story:** Como usuario (o como desarrollador de otras features), necesito saber en cada barra: ¿cuánto capital tengo?, ¿cuánto he ganado/perdido hoy?, ¿cuál es mi peor pérdida desde el peak? El sistema debe reportar esto continuamente sin lagunas.

---

## Comportamientos Observables

- [ ] Estrategia inicia con 100,000 de capital
  → Barra 1: +2,000 → Equity = 102,000, PnL diario = +2,000
  → Barra 2: -1,000 → Equity = 101,000, PnL diario = -1,000 acumulado = +1,000
  → Sistema reporta: Capital=101k, DailyPnL=-1k, CumulativePnL=+1k, Drawdown=1%

- [ ] Estrategia alcanza peak de 110,000
  → Barra siguiente: -3,000 → Equity = 107,000
  → RunningMaxDD = (Peak - Current) / Peak = (110k - 107k) / 110k = 2.7%
  → Sistema reporte: MaxDDFromPeak = 2.7%

- [ ] Fin de día
  → Sistema cierra todas las posiciones abiertas (si aplica)
  → Calcula PnL final del día = Close Equity - Open Equity
  → Registra en Daily Summary

---

## Restricciones

- **NUNCA capital negativo.** Si es < 0, error → estrategia inviable.
- **NUNCA drawdown negativo.** Drawdown siempre >= 0 (es pérdida desde peak).
- **NUNCA inconsistencia:** Equity[t] debe ser = Equity[t-1] + PnL[t] (exacto, sin errores de redondeo).
- **NUNCA saltos no-explicados en capital.** Cada cambio debe trazarse a una orden ejecutada.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| INITIAL_CAPITAL | 100000 | 1000-10000000 | Capital inicial para backtest/live | CONFIG |
| TRACKING_FREQUENCY | bar | bar / tick / minute | Frecuencia de tracking (barra, tick, minuto) | CONFIG |
| COMPOUNDING | true | true/false | Si true, ganancias se reinvierten (compounding) | CONFIG |
| DRAWDOWN_BASIS | peak | peak / initial | Si "peak": DD desde máximo histórico. Si "initial": desde capital inicial | CONFIG |
| REBALANCE_FREQUENCY | daily | daily / weekly / never | Cuando rebalancear si hay múltiples estrategias | CONFIG |

---

## Ciclo de Vida de la Feature — Equity Curve Tracker

### Entrada
- Capital inicial
- Serie de órdenes ejecutadas (entrada, salida, pesos, tamaños)
- Precios de ejecución (slippage incluido si aplica)
- Fechas/timestamps de cada orden

### Proceso
1. **Inicializa:** Capital = INITIAL_CAPITAL, Peak = Capital, Equity = Capital
2. **Por cada orden:**
   - Calcula PnL = (Exit Price - Entry Price) × Quantity (con slippage)
   - Actualiza: Equity[t] = Equity[t-1] + PnL[t]
   - Si Equity[t] > Peak → actualiza Peak
   - Calcula: DrawDown[t] = (Peak - Equity[t]) / Peak
3. **Resumen periódico** (fin de barra/día):
   - Totaliza PnL del período
   - Registra estadísticas (equity, DD, retorno %)

### Salida
- **EquityCurve:** Vector de capital por barra (timestamps vs equity)
- **DailyPnL:** Resumen de ganancia/pérdida por día
- **DrawdownCurve:** Vector de drawdown máximo por barra
- **EquityStats:** Peak capital, Min capital, Final capital, Max Drawdown, Total Return %

### Contextos de Uso

**Contexto 1: Backtesting (Módulo Generate/Validate)**
- Entrada: Órdenes generadas por estrategia candidata en período histórico
- Pregunta: ¿Cuál es el capital final y el drawdown máximo?
- Impacto: Alimenta cálculo de Sharpe, Max DD, otros KPIs. Veredicto de estrategia depende de estos números.

**Contexto 2: Validación de Robustez (Módulo Validate)**
- Entrada: Mismo equity curve pero en período OOS (validación)
- Pregunta: ¿El drawdown y retorno se mantienen similares fuera de la muestra?
- Impacto: Detecta overfitting (equity curve OOS mucho peor que in-sample)

**Contexto 3: Trading LIVE (Módulo Execute)**
- Entrada: Órdenes reales ejecutadas contra broker
- Pregunta: ¿Cuál es mi capital actual? ¿Cuán profundo estoy en drawdown?
- Impacto: Decision gate: si MaxDD > límite configurable, pausa estrategia automáticamente

**Contexto 4: Auditoría (Módulo Feedback)**
- Entrada: Histórico de equity curves LIVE + Backtest esperado
- Pregunta: ¿Hay degradación sistemática? ¿La performance LIVE baja vs backtest?
- Impacto: Detecta si estrategia debe retirarse o si cambios de mercado la invalidaron

---

## Tareas (TTRs)

### TTR-001: Inicializar y Mantener Equity

**Qué hace:** Setup inicial de capital, y actualización en cada orden.

**Entrada:**
- Capital inicial
- Orden (entry/exit) con precio y cantidad

**Salida:**
- Equity actualizado
- PnL de la orden

**Restricciones:**
- Capital nunca < 0
- Equity = Capital anterior + PnL orden

---

### TTR-002: Calcular Drawdown en Vivo

**Qué hace:** Mantiene peak histórico y calcula drawdown actual (pérdida desde peak).

**Entrada:**
- Equity actual
- Peak anterior

**Salida:**
- Peak actualizado (si equity > peak)
- Drawdown = (Peak - Equity) / Peak ∈ [0, 1]

**Restricciones:**
- Peak nunca baja (es máximo histórico)
- Drawdown >= 0

---

### TTR-003: Resumen Periódico (Diario/Semanal)

**Qué hace:** Cierra período y calcula PnL acumulado, retorno %, cambios.

**Entrada:**
- Equity al inicio del período
- Equity al final del período
- Trades ejecutados en período

**Salida:**
- DailySummary: PnL total, retorno %, trades count, win rate del período
- Metadata: timestamp período, capital start, capital end, max DD del período

---

### TTR-004: Detectar Degradación Live vs Backtest

**Qué hace:** Compara equity curve LIVE vs backtest esperado, detecta drift.

**Entrada:**
- Backtest equity curve esperada
- LIVE equity curve actual (últimos N días)

**Salida:**
- Degradation score: cuánto peor es LIVE vs backtest (%)
- Alerta: si degradation > umbral, marca para revisión

---

## Dependencias

**Depende de:**
- `trade-reconciler` (obtiene órdenes ejecutadas reales)
- `institutional-metrics` (Sharpe, Max DD y otros se calculan BASÁNDOSE en equity curve)
- `slippage-models` (precios de ejecución incluyen slippage)

**Depende de ella:**
- `institutional-metrics` (usa equity curve para KPIs)
- `validate` (usa backtest equity para evaluar candidatos)
- `execute` (tracking LIVE de capital)
- `feedback` (monitoreo de degradación)

---

## Gobernanza y Estándares

- **Inundación de Fundaciones (ADR-0020): Perfil B (IA / R&D)** — tracking de equity con auditoría + linaje.

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del snapshot de equity |
| | `created_at` | Timestamp de la barra/tick |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de integridad del balance actual |
| | `audit_chain_hash` | Hash del historial de performance |
| | `event_sequence_id` | Secuencia de recuperación del snapshot |
| **II. Soberanía** | `owner_id` | Dueño del portafolio/capital |
| | `manifest_id` | ID del contrato de diseño |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del tracker PnL (Nautilus core) |
| | `data_snapshot_id` | Ref al rastro de ejecución (Fills) |
| | `indicator_state_hash` | Snapshot de capital y Drawdown |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del motor de performance |


---

## Nota Crítica

**Si el equity curve tracking es incorrecto, TODA la validación de estrategias colapsa.** Este feature es foundational—debe ser 100% correcto.
