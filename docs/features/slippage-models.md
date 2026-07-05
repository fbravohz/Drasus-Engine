# Modelos de Slippage y Fricción

**Carpeta:** `./features/slippage-models/`
**Estado:** Especificación
**Última actualización:** 2026-04-12
**Decisión Arquitectónica Asociada:** ADR-0017 (Simulación de Alta Fidelidad)

---

## ¿Qué es?

Es el componente que inyecta "realismo institucional" a las ejecuciones. Se apalanca en los modelos de impacto de **NautilusTrader** (ADR-0013) para asegurar que la fricción en backtesting replique el comportamiento de los venues reales (Binance, IBKR).

---

## Comportamientos Observables

- [ ] Estima el **Slippage** basado en la volatilidad del momento y el tamaño de la orden (Market Impact).
- [ ] Aplica la **Regla de Pardo (Limitación de Penetración):** Una orden simulada no puede ejecutar más de un % configurado del volumen total de la barra actual.
- [ ] Calcula **Fees y Spreads** dinámicos por asset y broker.
- [ ] Modela el **Triple Swap** en los días de rollover (ej: miércoles en Forex) siguiendo la lógica del reloj de simulación de Nautilus.
- [ ] Implementa la **Penetración de Ticks:** Exige que el precio atraviese el límite por $X$ ticks para considerar una orden como llena.

---

## Restricciones

- **PENETRACIÓN DE TICKS:** En backtest, se exige que el precio de mercado supere el precio límite por una cantidad de ticks configurada antes de ejecutar el "fill".
- **FIDELIDAD DE TICK:** En backtest, se asume un modelo de 4-ticks por barra para el cálculo del slippage intra-barra.

---

## Parámetros Configurables

| Parámetro | Default | Qué hace |
|---|---|---|
| PARDO_VOLUME_LIMIT | 0.10 | Max % de volumen de la barra que podemos "llenar" (10% default) |
| SLIPPAGE_BPS_VOL_MULT | 0.1 | Multiplicador de slippage basado en ATR |
| SWAP_TRIPLE_DAY | WEDNESDAY | Día en que se aplica el triple swap (Forex) |

---

---

## Ciclo de Vida
*   **Entrada:** `backtest-engine` (fricción histórica), `execute` (validación pre-trade).
*   **Proceso:** Análisis de Liquidez (Pardo) → Cálculo de Impacto (Market Impact) → Ajuste de Precio de Ejecución.
*   **Salida:** `execution_price_adjusted`, `fees_calculated`, `slippage_bps`.

---

## Tareas (TTRs)

### **TTR-001: Implementar Motor de Impacto de Mercado**
*   **Descripción:** Desplaza el `execution_price` en función de la relación (Order Qty / Average Volume).
*   **Reglas de Negocio:**
    * El slippage debe ser dinámico: mayor volatilidad (ATR) → mayor castigo por slippage.
    * Los precios finales deben persistirse como `int64` (centavos/ticks) para consistencia (ADR-0002).
*   **Entrada:** `order_qty`, `bar_volume`, `market_volatility` (ATR).
*   **Salida:** `slippage_bps` (puntos básicos), `final_price`.
*   **Precondición:** Datos de volumen de barra disponibles (OHLCV).
*   **Postcondición:** Registro de la delta de slippage para análisis de "Realismo de Backtest".

### **TTR-002: Filtro de Liquidez Pardo**
*   **Descripción:** Rechaza o parcializa órdenes que superen el `PARDO_VOLUME_LIMIT`.
*   **Reglas de Negocio:**
    * NUNCA ejecutar > 10% (configurable) del volumen total de la barra en simulación.
    * Si se supera el límite, el trade se marca como "Parcialmente Llenado" o se difiere a la siguiente barra.
*   **Entrada:** `order_qty`, `bar_volume`.
*   **Salida:** `executable_qty`, `is_rejected`.
*   **Precondición:** Modo de fidelidad `SIMULATED_TICKS` o superior activo.
*   **Postcondición:** Inyección de `audit_hash` en el rastro de simulación (ADR-0020).

---

## Gobernanza y Estándares (Fijos)
## Persistencia (Inundación de Fundamentos — ADR-0020)

Toda estimación de impacto registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del cálculo |
| | `created_at` | Timestamp de estimación |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de integridad del modelo L2 |
| | `audit_chain_hash` | Hash de la sesión de simulación |
| | `event_sequence_id` | Secuencia del evento de cálculo |
| **II. Soberanía** | `owner_id` | Responsable del entorno |
| | `manifest_id` | ID del contrato de diseño legal |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del modelo de impacto (Pardo/Nautilus) |
| | `data_snapshot_id` | L2/Orderbook snapshot de referencia |
| | `indicator_state_hash` | Puntos básicos (bps) estimados |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del simulador de fricción |

- **Decisión Arquitectónica Asociada:**
    - ADR-0002: Arregrística entera para precios (int64).
    - ADR-0017: Simulación de Alta Fidelidad.
    - ADR-0020: Inundación de Fundaciones.

---

## Preparación para Opciones (Post-MVP — ADR-0140)

> **Estado:** Diferido. No implementar hasta que los cinco prerrequisitos de ADR-0140 se cumplan.

El modelo de slippage actual usa ATR y volumen de barra. En opciones:

- El spread bid/ask es mucho más amplio (a menudo 5-50x el spread del subyacente).
- La liquidez varía dramáticamente por strike y vencimiento.
- El modelo de Pardo (penetración de volumen) no aplica directamente: el volumen de opciones se mide en contratos abiertos (open interest), no en volumen de barra.
- El slippage en opciones es el **asesino silencioso** de estrategias retail: spreads amplios destruyen el edge rápidamente.

**Refactorización necesaria:** añadir un modelo de slippage específico para opciones basado en bid/ask spread del contrato, open interest del strike/vencimiento y volumen del contrato (no del subyacente).

**Moonshot asociado:** [`option-data-ingestor`](../moonshots/option-data-ingestor.md) — fuente de datos de open interest y spreads de opciones.

---

## Dependencias
**Depende de:**
- [`clock`](../features/clock.md) — para cálculo de Swaps y Horarios de sesión.

**Consumido por:**
- [`backtest-engine`](../features/backtest-engine.md) — para realismo histórico.
- [`execute`](../modules/execute.md) — para estimaciones de ejecución en vivo.
