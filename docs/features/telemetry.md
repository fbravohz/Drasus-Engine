# Telemetría — Trazabilidad Técnica y Diagnóstico

**Carpeta:** `./features/telemetry/`
**Estado:** En Diseño
**Última actualización:** 2026-04-11
**Decisión Arquitectónica Asociada:** ADR-0015 (Causalidad)

## ¿Qué es?

Es el componente encargado de capturar y persistir métricas de **performance técnica** y estado de salud del sistema en tiempo real. A diferencia del `audit-log` (que registra eventos de negocio), la telemetría se enfoca en el "cómo" está operando el software.

Es la fuente primaria de evidencia para que el módulo de `feedback` detecte anomalías de infraestructura (ej: picos de latencia) que podrían estar afectando el rendimiento de las estrategias.

## Comportamientos Observables

- [ ] Captura latencias de hot-paths (ej: tiempo de ejecución desde señal hasta orden en el puerto).
- [ ] Registra uso de memoria y CPU por proceso/módulo.
- [ ] Proporciona una "Señal de Vida" (heartbeat) de todos los daemons y servicios asíncronos.
- [ ] Permite consultar series temporales de performance técnica para correlacionar con eventos de P&L.
- [ ] **Builder Telemetry & ETA Prediction:** Monitoreo operativo en tiempo real vía gRPC/WebSocket.
  - **Throughput:** `strategies_generated`, `time_per_strategy_ms`, `accepted_pct`.
  - **Hardware Monitor:** Carga CPU por núcleo, VRAM usage (GPU opcional vía `candle`, ADR-0112), Disk I/O (DuckDB).
  - **Predictive ETA:** Tiempo restante dinámico basado en velocidad actual.
  - **State Probability:** Inferencia estadística ligera sobre viabilidad de sobrevivientes.
- [ ] **Heap Memory Monitor & Cleanup:** Monitoreo activo de RAM (`psutil.virtual_memory()`) con alertas, y endpoint (`/api/system/gc`) para forzar la recolección de basura de Rust.
- [ ] **Best Strategy Tracker:** Emisión del evento `best_strategy_update` con minigráfico de equity curve para visualización en tiempo real.

## Restricciones

- **ALTA EFICIENCIA:** El registro de telemetría no debe añadir más de 50µs de latencia al proceso emisor.
- **PODA AUTOMÁTICA:** Los datos de telemetría técnicos (no críticos) se purgan tras un período configurable (ej: 7 días) para evitar el bloat de la DB.
- **DETERMINISMO NO AFECTADO:** La captura de telemetría no debe alterar el resultado de los cálculos del Functional Core.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace |
|---|---|---|---|
| SAMPLING_RATE | 1.0 | 0.1-1.0 | Porcentaje de eventos capturados (1.0 = todos) |
| RETENTION_DAYS | 7 | 1-30 | Dias de persistencia antes de la poda |
| LATENCY_THRESHOLD_MS | 10 | 1-100 | Umbral para alertar anomalía técnica inmediata |

## Tareas (TTRs)

### TTR-001: Implementar Buffer de Alta Velocidad
Crear un sistema de recolección no bloqueante (probablemente usando un Queue asíncrono) para que los módulos emitan telemetría sin esperar a la escritura en disco.

### TTR-002: Diseñador de Vistas de Correlación
Crear la lógica que permita al módulo de `feedback` preguntar: "¿Cuál era la latencia del puerto de ejecución cuando ocurrió este fill con alto slippage?"

---

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Toda serie de telemetría porta el set de relevancia técnica:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador de la muestra métrica |
| | `created_at` | Timestamp de captura |
| | `audit_chain_hash` | Hash de integridad del stream técnico |
| **II. Soberanía** | `institutional_tag` | Tag de firma/entorno operativo |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del emisor de telemetría |
| | `session_id` | Sesión global vinculada |
| **IV. Hardware** | `node_id` | ID del host físico monitorizado |
| | `process_id` | PID del proceso muestreado |
| | `execution_latency_ms` | Latencia intrínseca de captura |


---

## Dependencias
- **Consumido por:** Todos los módulos (especialmente `execute` y `feedback`).
- **Almacenado en:** `infrastructure-setup` (SQLite/WAL).
