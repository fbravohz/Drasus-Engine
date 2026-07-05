# Precision Sizing Models (Modelos de Dimensionamiento de Precisión)

**Carpeta:** `./features/precision-sizing-models/`
**Estado:** En Diseño
**Última actualización:** 2026-06-11
**Decisión Arquitectónica Asociada:** ADR-0044 (Sizing Transversal), ADR-0108, ADR-0109

---

## ¿Qué es esta feature?

Proporciona un framework unificado y determinista para el cálculo del tamaño de las posiciones. Este motor es consumido por los módulos de **Investigación (Backtest)**, **Gestión (Portafolio)** y **Ejecución (Live)** para garantizar la consistencia absoluta de los resultados.

### Modelos Soportados:
1.  **Ratio Fijo (Ryan Jones):** Incrementa el tamaño de la posición basándose en unidades de beneficio acumulado (*Delta*), permitiendo un crecimiento geométrico controlado.
2.  **Ajuste por ATR (Average True Range):** Normaliza el riesgo de la posición según la volatilidad actual del activo, asignando tamaños menores en entornos volátiles y mayores en entornos calmos.
3.  **Risk Percent Sizing:** Calcula el lote basándose en un porcentaje fijo del capital de la cuenta (ej. 1% de riesgo por trade) y la distancia al Stop Loss.
4.  **Volatility Targeting Engine:** Ajusta dinámicamente la exposición basándose en la volatilidad histórica del activo (ATR). Si el ATR del mercado se duplica, el motor divide el lotaje a la mitad automáticamente para mantener el riesgo monetario en dólares ($R) constante. Fórmula: $Size = TargetRisk / ATR$.

**Primitivas de Acción de Mutación de Sizing del Genoma de Riesgo y Gestión de Posición (ADR-0108/ADR-0109):** los valores de `SIZING_MODE` son, en conjunto, las Primitivas de Acción de Mutación de Sizing del Dominio de Riesgo y Gestión de Posición: `risk_pct` materializa `Risk_Percent_Equated(%)`, `vol_target` con `TARGET_RISK_USD` materializa `Fixed_Monetary_Risk($R)`, y el Sizing Kelly Dinámico (TTR-005) materializa `Kelly_Sizing_Capped(Max_Risk)`. `fixed_ratio` actúa como `Multiplier(Factor)` cuando su progresión es dirigida por los Genes de Condición de Estado del genoma (p. ej. `Equity_DD`, `Balance_Streak_Losses`) en lugar de únicamente por el beneficio acumulado. Cuando ese genoma no está activo, `SIZING_MODE` y sus parámetros operan exactamente como hoy (comportamiento por defecto, sin cambios).

---

## Comportamientos Observables

- [ ] **Paridad Bit-a-Bit:** El cálculo realizado en un backtest sobre 2 años de datos debe ser idéntico al cálculo realizado por el bot en vivo ante las mismas condiciones de equidad e indicadores.
- [ ] **Ajuste Dinámico por Volatilidad:** Al recibir un incremento en el ATR, el motor debe reducir automáticamente el tamaño sugerido para la siguiente operación.
- [ ] **Crecimiento Geométrico:** En modo *Fixed Ratio*, el sistema acelera el dimensionamiento tras alcanzar hitos de beneficio definidos por el usuario.
- [ ] **Kelly Dinámico por Convicción:** El motor escala el tamaño de la posición según un "Conviction Score" (0-100) por señal. El usuario configura tramos (ej. arriesgar 0.5% si el score < 60, escalar a 3% si el score > 90), de modo que el apalancamiento se adapta a la probabilidad predictiva de éxito de cada operación, no a una regla plana.
- [ ] Cuando el Genoma de Riesgo y Gestión de Posición (ADR-0109) está activo, el motor evolutivo puede cambiar `SIZING_MODE` o sus parámetros asociados (p. ej. `RISK_PER_TRADE`) por operación según sus Genes de Condición de Estado — por ejemplo, reducir `RISK_PER_TRADE` de 1% a 0.2% tras `Balance_Streak_Losses >= 3`.

---

## Restricciones

- **NUNCA** permitir un tamaño de posición que exceda el margen disponible de la cuenta.
- **NUNCA** redondear el tamaño hacia arriba si eso viola el límite de riesgo máximo configurado.
- **NUNCA** operar sin una lectura válida del ATR si el modelo seleccionado es `ATR-Adjusted`.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| SIZING_MODE | risk_pct | fixed_ratio / atr_adj / risk_pct / vol_target | Modelo de cálculo activo | CONFIG |
| RISK_PER_TRADE | 0.01 | 0.001 - 0.05 | % de capital a arriesgar por trade | CONFIG |
| FIXED_RATIO_DELTA | 1000 | 100 - 100000 | Beneficio requerido para añadir 1 lote extra | CONFIG |
| ATR_PERIOD | 14 | 5 - 50 | Período de cálculo de volatilidad para el sizing | CONFIG |
| TARGET_RISK_USD | 100.0 | 1.0 - 10000.0 | Riesgo objetivo en dólares ($R) para el motor de Volatility Targeting | CONFIG |

---

## Ciclo de Vida de la Feature

### Entrada
- **Account Data:** Equity actual, Balance, Margen disponible.
- **Market Data:** Valor del ATR (si aplica), precio actual.
- **Trade Setup:** Distancia al Stop Loss (en ticks o precio).

### Proceso
1. **Validación de Datos:** Verifica la presencia de todas las variables requeridas para el modelo activo.
2. **Cálculo Base:** Aplica la fórmula matemática del modelo (ej. `(Equity * Risk) / (SL_Distance * TickValue)`).
3. **Filtro de Seguridad:** Cruza el resultado con los límites de margen y exposición máxima del portafolio.

### Salida
- `PositionSize` (float/int): El número exacto de contratos o acciones a operar.
- `SizingMetadata`: Rastro de la fórmula aplicada y variables usadas para auditoría.

---

## Tareas (TTRs)

### TTR-001: Implementación de Ratio Fijo (Ryan Jones)
- **Qué tiene que pasar:** Programar la lógica que escala la posición según la serie de beneficios acumulados y el parámetro de Delta.
- **Criterio de éxito:** Si el Delta es $500 y tengo $1000 de beneficio, la posición debe subir de 1 a 2 unidades (Nivel 2).

### TTR-002: Implementación de Sizing por Volatilidad (ATR)
- **Qué tiene que pasar:** Crear el adaptador que normaliza el tamaño de lote basándose en la volatilidad histórica reciente.

### TTR-004: Implementación de Volatility Targeting Engine
- **Qué tiene que pasar:** Programar la lógica del motor de volatilidad que ajusta dinámicamente el lotaje de forma inversa al ATR del activo para mantener constante el riesgo en dólares.
- **Criterio de éxito:** Si el ATR se duplica en el feed, el tamaño se divide a la mitad automáticamente.
- **Qué no puede pasar:** El motor no debe aplicar el ajuste si el ATR es nulo o menor o igual a cero (degrada a sizing base).

### TTR-003: Orquestación Transversal (Backtest & Live)
- **Qué tiene que pasar:** Asegurar que tanto el `backtest-engine` como el `broker-connector` utilicen este componente como fuente única de verdad para el cálculo de lotaje.

### TTR-005: Sizing Kelly Dinámico por Conviction Score
- **¿Cuál es el problema?** No todas las señales tienen la misma probabilidad de éxito; arriesgar un % fijo siempre desperdicia ventaja en las señales fuertes y sobre-expone en las débiles.
- **¿Qué tiene que pasar?** El modelo recibe un Conviction Score (0-100) por señal y aplica un Kelly fraccionado por tramos configurables, ajustando el tamaño de posición de forma agresiva o conservadora según el score.
- **¿Cómo sé que está hecho?**
  - [ ] Una señal con score > 90 produce un lotaje mayor (hasta el tope configurado) que una señal con score < 60.
  - [ ] Los tramos y los porcentajes de riesgo son configurables.
- **¿Qué no puede pasar?**
  - El tamaño NUNCA supera el tope máximo de riesgo configurado, sin importar el score.
  - El cálculo conserva la Paridad Bit-a-Bit entre backtest y live.
- **Nota de alcance:** El cálculo del Conviction Score (confluencia de liquidez/volatilidad/correlación vía ML) es R&D y vive como moonshot; esta feature solo consume el score como entrada y lo traduce a lotaje.

### TTR-006: Exposición de Modelos de Sizing como Primitivas de Acción del Genoma de Riesgo y Gestión (ADR-0108/ADR-0109)
- **¿Cuál es el problema?** El Dominio de Riesgo y Gestión de Posición necesita poder seleccionar y parametrizar dinámicamente el modelo de sizing activo (`SIZING_MODE` y sus parámetros) en función de sus Genes de Condición de Estado, sin romper la Paridad Bit-a-Bit ni los modelos por defecto.
- **¿Qué tiene que pasar?** `SIZING_MODE` y los parámetros de la tabla de configuración (`RISK_PER_TRADE`, `TARGET_RISK_USD`, `FIXED_RATIO_DELTA`) deben ser direccionables como nodos `wildcard_group` del dominio de Riesgo y Gestión de Posición (ADR-0108), mapeados a las Primitivas de Acción de Mutación de Sizing (`Multiplier`, `Risk_Percent_Equated`, `Kelly_Sizing_Capped`, `Fixed_Monetary_Risk`).
- **¿Cómo sé que está hecho?**
    - [ ] Un Manifest sin Genoma de Riesgo y Gestión activo calcula el sizing exactamente igual que antes de ADR-0109 (sin regresión).
    - [ ] Un Manifest con Genoma de Riesgo y Gestión activo puede resolver `RISK_PER_TRADE` a un valor distinto por Gen de Condición de Estado, conservando la Paridad Bit-a-Bit entre backtest y live para cada valor resuelto.
- **¿Qué no puede pasar?** El cambio de `SIZING_MODE` o de sus parámetros en LIVE nunca ocurre fuera del proceso de resolución de `wildcard_group` y re-compilación del Manifest (ADR-0043).

## Persistencia (Inundación de Fundamentos — ADR-0020)

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador del cálculo de sizing |
| | `created_at` | Timestamp del cálculo de sizing |
| | `updated_at` | Timestamp de última actualización del registro |
| | `audit_hash` | Hash de la fórmula y variables de entrada |
| | `audit_chain_hash` | Hash de enlace con el cálculo de sizing anterior |
| | `event_sequence_id` | Secuencia ordinal del evento de cálculo |
| **IV. Hardware** | `node_id` | ID del hardware de ejecución |
| **V. Forense** | `process_id` | PID del servicio de riesgo |
| | `logic_hash` | Hash del modelo matemático activo |

## Preparación para Opciones (Post-MVP — ADR-0140)

> **Estado:** Diferido. No implementar hasta que los cinco prerrequisitos de ADR-0140 se cumplan.

Los 4 modelos de sizing actuales (Fixed Ratio, ATR-Adjusted, Risk Percent, Volatility Targeting) calculan tamaño de posición en contratos/unidades. En opciones, el sizing es fundamentalmente diferente:

- El "tamaño" de una opción no es solo cantidad de contratos: es **exposición delta-equivalente**.
- Un contrato de opción sobre SPY puede tener delta 0.30 o delta 0.95 dependiendo del strike y la volatilidad implícita.
- El sizing correcto requiere calcular el **delta-adjusted notional**, no solo `Equity * Risk% / SL_Distance`.

**Refactorización necesaria:** añadir un quinto modelo de sizing (`delta_equivalent`) que calcule la exposición en unidades equivalentes del subyacente, consumiendo las griegas del [`greeks-monitor`](../moonshots/greeks-monitor.md).

**Moonshots asociados:** [`greeks-monitor`](../moonshots/greeks-monitor.md), [`option-pricing-engine`](../moonshots/option-pricing-engine.md).

---

## Gobernanza y Estándares (Fijos)

- **Genomas Modulares por Dominio (ADR-0108/ADR-0109):** `SIZING_MODE` y sus parámetros son Primitivas de Acción de Mutación de Sizing del Dominio de Riesgo y Gestión de Posición. Ver Registro de Dominios Genómicos en [`SAD.md`](../SAD.md) §2.3.
