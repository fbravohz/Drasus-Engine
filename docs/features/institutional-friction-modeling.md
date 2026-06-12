# Institutional Friction Modeling (Adverse Selection)

**Carpeta:** `./features/institutional-friction-modeling/`
**Estado:** Lista para implementar
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0069 (Modelado de Fricción Institucional)

## ¿Qué es esta feature?

El motor de **Institutional Friction Modeling** inyecta realismo probabilístico en la ejecución de órdenes Límite. Modela el fenómeno de **Adverse Selection** (donde solo te llenan cuando el mercado te va a ir en contra) y el **Limit Order Drop-Out** (el precio toca tu nivel pero no hay suficiente liquidez para tu orden).

**Problema que resuelve:** Los backtests retail asumen que si el precio toca el Bid/Ask, la orden se llena. Esto genera falsos beneficios en estrategias de Mean-Reversion que "viven" del spread. En la realidad, si el mercado toca tu orden y rebota a tu favor, es probable que NO te hayan llenado.

## Comportamientos Observables

- [ ] **Probabilistic Fill Rate:** El simulador aplica una probabilidad de éxito al tocar el precio. Si el mercado no atraviesa tu nivel por $X$ ticks, el trade puede ser descartado.
- [ ] **Friction Inversion:** En backtests de estrés, el sistema asume que solo el 60% de las órdenes ganadoras se ejecutan, mientras que el 100% de las perdedoras (donde el precio te atraviesa) sí se ejecutan.
- [ ] **Limit Order Drop-Out:** El usuario ve en los reportes cuántos trades fueron "ignorados" por falta de profundidad simulada.

## Restricciones

- **NUNCA** asumir un fill rate del 100% para órdenes Límite en estrategias de scalping o microestructura.
- **PROHIBIDO** el uso de modelos de fricción que no tengan paridad con el histórico de L2 (DOM) si este está disponible.
- **FIJO:** El peor escenario de fricción (60% fill) debe ser evaluable en el guantelete de robustez.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| BASE_FILL_PROBABILITY | 0.85 | 0.50 - 0.99 | Probabilidad base de llenado al tocar precio | CONFIG |
| STRESS_FILL_RATE | 0.60 | 0.40 - 0.90 | Fill rate asumido en tests de estrés | CONFIG |
| TICKS_TO_CONFIRM | 1 | 0 - 5 | Ticks que debe atravesar para fill 100% | CONFIG |
| ADVERSE_SELECTION_MODE | enabled | enabled / disabled | Activa descarte de ganadores "al toque" | [FIJO] |

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Modelos probabilísticos (Monte Carlo intra-trade) para decidir el llenado basándose en la penetración de precio y volatilidad.
- **Shell (Infraestructura):** Inyección de la lógica en el `Matching Engine` de NautilusTrader (vía conectores de simulación).
- **Frontera Pública:** Interfaz para configurar los perfiles de fricción del broker simulado.

## Ciclo de Vida de la Feature — Institutional Friction Modeling

### Entrada
- Datos de mercado (ticks/bars).
- Órdenes Límite enviadas por la estrategia.
- Perfil de fricción configurado.

### Proceso
- Evalúa si el precio tocó el nivel de la orden.
- Si tocó pero no atravesó: aplica `BASE_FILL_PROBABILITY`.
- Si atravesó: aplica lógica de prioridad de cola (probabilística).
- Decide si la orden se marca como `FILLED` o `EXPIRED` (Drop-out).

### Salida
- Estado de la orden (Filled / Canceled).
- Registro de "Friction Cost" (delta vs backtest perfecto).

## Tareas (TTRs)

### **TTR-001: Implementación del Probabilistic Fill Engine**
*   **¿Cuál es el problema?** El backtest estándar es determinista y optimista.
*   **¿Qué tiene que pasar?** El motor de simulación debe usar una semilla aleatoria (configurable) para decidir si una orden Límite que "tocó" el precio se llena o se pierde.
*   **¿Cómo sé que está hecho?**
    - [ ] Corriendo el mismo backtest con diferentes semillas de fricción da resultados ligeramente distintos.
    - [ ] El número de trades final es menor que en un backtest "perfecto".

### **TTR-002: Inyección de Stress Fill Rate en Validación**
*   **¿Cuál es el problema?** No sabemos si la estrategia es robusta ante condiciones de baja liquidez.
*   **¿Qué tiene que pasar?** Se añade un test en `validate` que fuerza un `STRESS_FILL_RATE` de 0.60.
*   **¿Cómo sé que está hecho?**
    - [ ] Reporte de validación muestra la métrica "Robustness under Low Liquidity".

## Gobernanza y Estándares

- **Local-First (ADR-0016):** 100% Local.
- **Fidelidad (ADR-0017):** Institucional (Pardo Penetration).
- **Inundación de Fundaciones (ADR-0020 V2):** Perfil Datos / Ingest.
- **Rastro de Evidencia:** Emite `fill_events` y `dropped_orders_count` para `feedback`.
