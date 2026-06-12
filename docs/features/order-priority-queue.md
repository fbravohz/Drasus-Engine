# Order-Priority Queue (Anti-Throttling)

**Carpeta:** `./features/order-priority-queue/`
**Estado:** En Diseño
**Decisiones Arquitectónicas Asociadas:** ADR-0080, ADR-0020 V2

---

## ¿Qué es? (Explicado Simple)

Es una **cola inteligente de órdenes** diseñada para mitigar los límites de tasa (*rate limits*) impuestos por los exchanges. Durante episodios de alta volatilidad o congestión de la red, los brokers rechazan órdenes por exceso de peticiones (*throttling*). 

Este componente clasifica las órdenes según su urgencia:
- Las de supervivencia (**Stop Loss**) van al frente de la cola con prioridad absoluta.
- Las de toma de ganancias (**Take Profit**) van después.
- Las entradas a mercado y órdenes límite se encolan con menor prioridad y pueden ser descartadas o diferidas si persiste el estrangulamiento de la conexión.

---

## Comportamientos Observables

- **Clasificación Automática de Prioridad:** Toda orden emitida por las estrategias es evaluada en memoria y se le asigna un tier de urgencia (P0 a P3).
- **Controlador de Tráfico (Rate Limiter):** El despachador mide continuamente la tasa de peticiones enviadas al bróker por segundo.
- **Retardo Adaptativo (Backoff Exponencial):** Si el bróker retorna un error de límite de tasa, el despachador pausa el envío de órdenes de menor prioridad y reintenta las órdenes críticas aumentando progresivamente los intervalos de tiempo entre intentos.

---

## Parámetros Configurables (Configuración Tipada Serde)

Se gestiona a través del objeto `OrderPriorityConfig`:

| Parámetro | Valor por Defecto | Qué Mide |
|---|---|---|
| `max_requests_per_second` | 10 | Límite de peticiones permitidas por el exchange |
| `p0_retry_backoff_ms` | 100 | Tiempo inicial de espera antes de reintentar un Stop Loss |
| `p3_discard_on_congestion` | True | Indica si las órdenes límite deben ser descartadas ante saturación |

---

## Tareas (TTRs)

### TTR-001: Implementación de la Cola Concurrente
- **Descripción:** Diseñar una estructura de datos en memoria (Heap de Prioridad Concurrente) para encolar órdenes por nivel de prioridad. Las órdenes P0 (`Stop Loss`) deben ignorar cualquier limitación de tamaño de la cola e insertarse en el primer puesto de ejecución disponible.
- **Criterio de Éxito:** Al encolar 100 órdenes simultáneamente bajo simulación de estrangulamiento de red, el 100% de las órdenes P0 se despachan primero.

### TTR-002: Despachador con Backoff Exponencial
- **Descripción:** Implementar el loop despachador que escucha la cola de prioridad y transmite las órdenes al adaptador del broker. Si el adaptador devuelve un error de rate limit, el despachador pausa el envío de órdenes de menor nivel (P1, P2, P3) y reintenta las P0 aplicando el factor de retardo exponencial.
- **Criterio de Éxito:** Las órdenes Stop Loss se reintentan sucesivamente a los 100ms, 200ms, 400ms hasta lograr confirmación.

---

## Gobernanza y Estándares (Fijos)
- **Inundación de Fundaciones (ADR-0020 V2):**
    - Obligatorio incluir en cada guardado: `execution_latency_ms` (latencia total desde emisión hasta confirmación), `compliance_status_id` (veredicto de envío exitoso).
- **Dependencias:** Utilizado primordialmente en `execute`.
