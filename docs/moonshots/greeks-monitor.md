# Greeks Monitor — Monitoreo de Griegas en Tiempo Real

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Post-MVP — Diferido por ADR-0140)
**Última actualización:** 2026-06-27
**Decisión Arquitectónica Asociada:** ADR-0140 (Opciones Financieras — Diferimiento al Post-MVP con Puerta Abierta)

---

## ¿Qué es?

Motor de cálculo y monitoreo en tiempo real de las griegas de primer orden (Delta, Gamma, Theta, Vega, Rho) y segundo orden (Charm, Vanna, Volga) agregadas por posición, estrategia y portafolio. Proporciona la visibilidad necesaria para gestionar el riesgo de opciones: Delta mide la sensibilidad al precio del subyacente, Gamma la aceleración de Delta, Theta la erosión temporal, Vega la sensibilidad a la volatilidad implícita y Rho la sensibilidad a las tasas de interés. Las griegas de segundo orden capturan interacciones cruzadas que las de primer orden ignoran.

**Por qué es moonshot:** Las griegas son aproximaciones locales (derivadas parciales). En mercados volátiles, Gamma puede hacer que Delta cambie más rápido de lo que el sistema reaccione. Las griegas de segundo orden (Charm, Vanna, Volga) son necesarias para cobertura predictiva y gestión precisa del riesgo en portafolios grandes, pero su cálculo y renderizado en tiempo real para miles de contratos exige optimización matemática significativa. El monitoreo en tiempo real requiere recalcular en cada tick del subyacente.

**Condición de activación (ADR-0140):** los cinco prerrequisitos del ADR-0140 deben cumplirse antes de implementar.

---

## Comportamientos Observables

- [ ] El sistema muestra las griegas agregadas de todas las posiciones de opciones abiertas: Delta total, Gamma total, Theta diario, Vega total, Rho total.
- [ ] El usuario puede ver las griegas desglosadas por estrategia, por subyacente o por contrato individual.
- [ ] El sistema alerta cuando una griega agregada supera un umbral configurable (ej. Delta > 0.50 equivalente a 50 acciones del subyacente).
- [ ] El Theta diario muestra la erosión de valor esperada por el paso del tiempo (decay).
- [ ] El sistema recalcula las griegas en cada actualización del precio del subyacente o de la volatilidad implícita.
- [ ] El sistema calcula las griegas de segundo orden (Charm, Vanna, Volga) cuando el modo avanzado está activo, mostrando la Charm Surface y los flujos de cobertura predictiva.
- [ ] El Charm agregado del portafolio muestra el decaimiento pasivo de Delta esperado durante períodos sin trading (noches, fines de semana), permitiendo cobertura predictiva antes del cierre.

---

## Tareas (TTRs)

### **TTR-001: Cálculo de Griegas por Posición y Agregadas**
*   **¿Cuál es el problema?** Cada contrato de opción tiene sus propias griegas, pero el riesgo real se gestiona a nivel agregado (portafolio completo).
*   **¿Qué tiene que pasar?** Calcular las cinco griegas para cada posición de opción y agregarlas por estrategia, subyacente y portafolio. El Delta se expresa en unidades equivalentes del subyacente (delta-adjusted notional).
*   **¿Cómo sé que está hecho?**
    - [ ] Un portafolio con 3 posiciones de opciones sobre SPY muestra el Delta agregado en unidades equivalentes de SPY.

### **TTR-002: Alertas de Griegas (Umbrales Configurables)**
*   **¿Cuál es el problema?** Un Delta agregado alto equivale a una exposición direccional significativa al subyacente; un Vega alto expone a cambios de volatilidad implícita.
*   **¿Qué tiene que pasar?** Configurar umbrales de alerta por griega y emitir notificaciones cuando se superen.
*   **¿Cómo sé que está hecho?**
    - [ ] Una alerta se dispara cuando el Delta agregado del portafolio supera el umbral configurado.

### **TTR-003: Dashboard de Griegas en Tiempo Real**
*   **¿Cuál es el problema?** El usuario necesita una vista consolidada del riesgo de opciones sin tener que calcular manualmente.
*   **¿Qué tiene que pasar?** Renderizar un panel con las griegas agregadas, el perfil de Delta por subyacente y la erosión temporal (Theta) proyectada.
*   **¿Cómo sé que está hecho?**
    - [ ] El panel muestra las griegas actualizadas con cada tick del subyacente.

### **TTR-004: Griegas de Segundo Orden (Charm, Vanna, Volga) y Charm Surface**
*   **¿Cuál es el problema?** Las griegas de primer orden son aproximaciones locales que ignoran interacciones cruzadas. En portafolios grandes o durante períodos cercanos al vencimiento, el decaimiento pasivo de Delta (Charm) y la sensibilidad de Delta a cambios de volatilidad (Vanna) generan pérdidas silenciosas que las griegas principales no detectan.
*   **¿Qué tiene que pasar?**
    *   **Charm (∂Δ/∂t):** mide cómo el paso del tiempo altera el Delta sin que el subyacente se mueva. Para opciones OTM, el Delta decae hacia 0; para ITM, converge hacia ±1. El Charm agregado del portafolio predice cuántas unidades del subyacente el portafolio "perderá" o "ganará" durante la noche o el fin de semana, permitiendo cobertura predictiva antes del cierre (los flujos de "weekend effect" institucional).
    *   **Vanna (∂Δ/∂σ):** mide cómo cambia Delta ante variaciones de la volatilidad implícita. Un portafolio con Vanna alto amplifica las pérdidas cuando un crash dispara la IV y simultáneamente mueve el subyacente.
    *   **Volga (∂²V/∂σ²):** mide la convexidad del precio respecto a la volatilidad. Un portafolio largo de Volga se beneficia de movimientos grandes de IV en cualquier dirección.
    *   **Charm Surface:** representación tridimensional de Charm cruzando Strike/Moneyness × Tiempo al Vencimiento. El relieve se vuelve violento cerca de expiración (short-dated options) y prácticamente plano para LEAPs. El algoritmo lee la superficie para optimizar el rebalanceo de cobertura antes de que el decaimiento pasivo genere desajustes.
*   **¿Cómo sé que está hecho?**
    - [ ] El Charm agregado predice correctamente el cambio de Delta del portafolio durante un período de 2 días sin movimiento del subyacente (validación contra el Delta real post-período).
    - [ ] La Charm Surface se renderiza en 3D con ejes Strike × DTE × Charm, mostrando los picos en opciones OTM/ITM cercanas a expiración.
    - [ ] Una alerta de Vanna se dispara cuando la exposición cruzada Delta-Volatilidad del portafolio supera el umbral configurado.
*   **¿Qué no puede pasar?** El cálculo de griegas de segundo orden no puede degradar la latencia del cálculo de primer orden por debajo del SLA de 5ms. Si el modo avanzado está desactivado, las griegas de segundo orden no se calculan (cero overhead).

---

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| DELTA_ALERT_THRESHOLD | 0.50 | 0.01 - 10.0 | Delta agregado que dispara alerta (en unidades equivalentes del subyacente) | CONFIG |
| VEGA_ALERT_THRESHOLD | 1000 | 100 - 100000 | Vega agregada que dispara alerta (en $ por punto de IV) | CONFIG |
| THETA_DAILY_ALERT | 500 | 50 - 50000 | Theta diario que dispara alerta (en $ de erosión diaria) | CONFIG |
| GREEKS_REFRESH_MODE | real_time | real_time / on_close | Cuándo recalcular: cada tick o al cierre de barra | CONFIG |
| SECOND_ORDER_GREEKS | false | true / false | Activa cálculo de Charm, Vanna, Volga y Charm Surface. Desactivado por defecto para evitar overhead en portafolios pequeños | CONFIG |
| VANNA_ALERT_THRESHOLD | 500 | 50 - 50000 | Exposición cruzada Delta-Volatilidad que dispara alerta | CONFIG |
| CHARM_PREDICTION_DAYS | 2 | 1 - 5 | Días de decaimiento pasivo que el sistema predice (para cobertura pre-cierre y weekend effect) | CONFIG |

---

## Gobernanza y Estándares (ADR-0020 V2)
- Perfil C (Ops / Hot-Path): Identidad + Soberanía + Hardware + Latencia. El cálculo de griegas en tiempo real opera en el hot path; latencia objetivo <5ms por recalculo.

---

## Dependencias

**Depende de:**
- [`option-pricing-engine`](./option-pricing-engine.md) — para el cálculo base de griegas por contrato.
- [`option-chain-manager`](./option-chain-manager.md) — para mapear contratos a posiciones abiertas.

**Consumido por:**
- [`pre-trade-validator`](../features/pre-trade-validator.md) — para checks de griegas agregadas (requiere refactorización post-MVP, ADR-0140).
- [`portfolio-optimizer`](../features/portfolio-optimizer.md) — para optimización con métricas de riesgo no-lineal (requiere refactorización post-MVP, ADR-0140).
