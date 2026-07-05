# Throttling Metrics Dashboard

**Carpeta:** `./features/throttling-metrics-dashboard/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0106 (Paradigma de Interfaz de Usuario y Dashboards Visuales de Alta Precisión)

---

## ¿Qué es esta feature?

El `Throttling Metrics Dashboard` provee visualización en tiempo real y diagnósticos de latencia en la capa de conectividad con los brokers. Monitorea los tiempos de tránsito de ida y vuelta (RTT) de las órdenes, la latencia de recepción del feed de datos de mercado y la ocupación de la cola de prioridades de órdenes (Anti-Throttling Queue), alertando ante degradaciones físicas de la red o del broker.

---

## Comportamientos Observables

- [ ] El usuario ve un panel con la latencia actual medida en milisegundos hacia el servidor del broker (RTT de órdenes).
- [ ] La UI dibuja un histograma continuo de latencias de ticks entrantes a través del WebSocket de datos.
- [ ] El dashboard muestra alertas visuales de color cuando la latencia supera el umbral configurado o se detectan descardes por throttling del broker.
- [ ] Indica el estado de llenado de la cola de prioridad de órdenes (`Order-Priority Queue`).

---

## Restricciones

- **NUNCA** permitir que el dashboard consuma más del 1% de CPU en su renderizado para no afectar los procesos concurrentes de ejecución en vivo.
- **NUNCA** realizar consultas bloqueantes de red en el hilo de UI; la telemetría se despacha de forma asíncrona mediante streams de gRPC o FFI.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| METRIC_FLUSH_INTERVAL_MS | 500ms | 100ms - 5000ms | Frecuencia de actualización de la telemetría en la UI | CONFIG |
| LATENCY_WARN_THRESHOLD_MS | 200 | 50 - 1000 | Umbral de latencia RTT de órdenes para emitir una alerta visual | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Agregación estadística de latencias (media, mediana, percentil 99).
- **Shell (Infraestructura):** Hilos de medición de latencia de red y streams FFI hacia Flutter.
- **Frontera Pública:** Interfaz de consulta de salud de conectores y métricas de latencia de trading en vivo.

---

## Ciclo de Vida de la Feature — Throttling Metrics Dashboard

### Entrada
- Timestamps de envío de orden y de recepción de respuesta del broker.
- Eventos del WebSocket de datos con timestamp del servidor vs timestamp local.

### Proceso
- Calcula la diferencia de tiempo (latencia) y la acumula en ventanas rodantes.
- Compara con los umbrales de advertencia.

### Salida
- Feed de telemetría de latencia en tiempo real para visualización.
- Alertas de degradación de red: `SALUDABLE` / `DEGRADADO` / `CRÍTICO`.

---

## Tareas (TTRs)

### **TTR-001: Visualizador de Latencia en Vivo (Broker RTT)**
*   **¿Cuál es el problema?** El operador desconoce si el retraso en el llenado de órdenes se debe a problemas de red física o al motor del broker.
*   **¿Qué tiene que pasar?** Dibuja en tiempo real un gráfico de líneas suavizado del RTT de ejecución de órdenes.
*   **¿Cómo sé que está hecho?**
    - [ ] El gráfico muestra la latencia de órdenes actualizada en cada llenado/respuesta del broker.

### **TTR-002: Monitor de Cola Anti-Throttling**
*   **¿Cuál es el problema?** Si la cola de órdenes se llena debido a límites de tasa (rate limits), el operador debe saberlo antes de enviar más transacciones.
*   **¿Qué tiene que pasar?** Renderizar una barra de estado de llenado de la cola de prioridad de órdenes.
*   **¿Cómo sé que está hecho?**
    - [ ] La barra refleja dinámicamente la ocupación de la cola en vivo.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020):** Perfil Ops / Hot-Path. Registra `process_id`, `node_id`, `execution_latency_ms`. Latencia de telemetría máxima de 1ms.
- **Rastro de Evidencia:** Emite métricas agregadas de latencia al módulo de `feedback`.

---

## Dependencias
- **Depende de:** `/features/order-priority-queue.md`, `/features/broker-connector.md`
- **Bloquea:** `/modules/execute.md`
