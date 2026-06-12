# Autopilot Metrics Provider

**Carpeta:** `./features/autopilot-metrics-provider/`
**Estado:** En Diseño
**Decisiones Arquitectónicas Asociadas:** ADR-0083, ADR-0020 V2

---

## ¿Qué es? (Explicado Simple)

Es el **proveedor dinámico de métricas del Autopilot (Módulo Execute)**. Expone métricas en tiempo real al Dashboard para que el usuario pueda monitorear la salud operativa del sistema y el cumplimiento de las normas de las firmas de fondeo.

El Dashboard consume estas métricas dinámicamente según la selección del usuario (`selected_metrics`). El proveedor expone un diccionario plano con la información interna del Autopilot.

---

## Comportamientos Observables

- **Exposición de Estado en Vivo:** Permite consultar los KPIs críticos de la sesión actual sin bloquear el hilo de ejecución de las órdenes.
- **Sincronización Continua:** Las métricas se actualizan al cierre de cada barra o evento de trade y se transmiten vía WebSockets.
- **Evaluación de Cumplimiento:** Calcula métricas específicas de Prop Firms para alertar proactivamente antes de que se viole una regla.

---

## Métricas Expuestas (Esquema de Salida)

El proveedor expone los siguientes campos en su diccionario:

| Campo | Tipo | Qué Mide |
|---|---|---|
| `live_pnl` | float | PnL acumulado del día en tiempo real |
| `max_dd` | float | Máximo drawdown histórico desde el inicio del Autopilot |
| `volatility` | float | Volatilidad realizada de las últimas 20 barras |
| `position_count` | int | Número de posiciones abiertas actualmente |
| `active_strategies` | str | Cantidad de estrategias activas vs pausadas |
| `prop_firm_compliance` | dict | Status de cumplimiento (% margen, % dd día, etc.) |
| `last_rebalance` | timestamp | Marca de tiempo del último rebalanceo del Portfolio |
| `next_rebalance` | timestamp | Marca de tiempo estimada del próximo rebalanceo |

---

## Tareas (TTRs)

### TTR-001: Implementación de la Interfaz ModuleMetricsProvider
- **Descripción:** Desarrollar la clase `AutopilotMetricsProvider` que implemente la interfaz `ModuleMetricsProvider`. Debe recolectar la información interna del motor de ejecución y retornar el diccionario estructurado de métricas.
- **Criterio de Éxito:** Al invocar el método de consulta, el componente devuelve un diccionario válido con los 8 campos base poblados con los valores del motor en vivo.

### TTR-002: gRPC/WebSocket Throttling de Métricas
- **Descripción:** Implementar el mecanismo de publicación periódica hacia el frontend vía WebSockets. Para evitar saturación del tráfico, las métricas deben agruparse y emitirse en lotes cada 100 milisegundos.
- **Criterio de Éxito:** El Dashboard recibe actualizaciones fluidas de PnL y Drawdown sin experimentar retrasos visuales.

---

## Gobernanza y Estándares (Fijos)
- **Inundación de Fundaciones (ADR-0020 V2):**
    - Obligatorio incluir en cada guardado: `indicator_state_hash` (hash del estado completo de las métricas), `session_id` (identificador del runtime actual).
- **Dependencias:** Utilizado primordialmente en `execute` y `feedback`.
