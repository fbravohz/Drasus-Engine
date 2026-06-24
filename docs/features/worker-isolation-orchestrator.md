> ✅ **Implementado** 2026-06-20 · Orden de trabajo [STORY-008](../execution/STORY-008-worker-isolation-orchestrator.md)

# Worker Isolation Orchestrator

**Carpeta:** `./features/worker-isolation-orchestrator.md`
**Estado:** Implementado
**Última actualización:** 2026-06-20
**Decisión Arquitectónica Asociada:** ADR-0013 (Stack Tecnológico — Rust puro; Python rechazado permanentemente)

## ¿Qué es esta feature?

El **Orquestador de Aislamiento de Trabajadores** gestiona la ejecución de tareas pesadas (simulaciones, entrenamientos de IA) en procesos independientes del sistema operativo. Su objetivo es evitar que el bucle de eventos del API (Orquestador Rust) se bloquee y permitir el paralelismo masivo utilizando todos los núcleos del CPU/GPU disponibles en la máquina host.

## Comportamientos Observables

- [ ] Lanzamiento de un job de generación con 1024 individuos → el orquestador reparte la carga en 8 workers paralelos.
- [ ] Monitoreo de salud: si un worker falla (crash), el orquestador lo reinicia y reasigna la tarea.
- [ ] Uso de **Memoria Compartida**: los datos de mercado se cargan una sola vez en RAM y todos los workers acceden a ellos sin duplicar el consumo de memoria.
- [ ] **Throttling de Progreso:** Emisión de señales de estado agrupadas cada 100 milisegundos hacia la UI.
- [ ] Cancelación en caliente: el usuario detiene un backtest y el proceso asociado se termina inmediatamente (`SIGTERM`).

## Restricciones

- **NUNCA** ejecutar tareas pesadas en el proceso principal de Orquestador Rust.
- **OBLIGATORIO:** Limitar el número de workers según los recursos del sistema para evitar el colapso de la UI (reservar al menos 2 hilos para el OS/Flutter FFI/Watchdog).
- La comunicación entre el orquestador y los workers locales usa memoria compartida (`memmap2`, buffer Arrow Read-Only) para datos de mercado y señales OS (`SIGTERM`/`SIGKILL` vía `nix`) para control de ciclo de vida. Sin ZeroMQ ni sockets TCP locales.
- **Extensión Híbrida Remota (HybridComputeCooperative):** En esta modalidad, el orquestador local extiende su pool de trabajadores enviando tareas serializadas a través de gRPC/WebSockets a daemons ejecutores remotos (VPS/bare-metal), recopilando los resultados de forma asíncrona en la persistencia local.


## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| MAX_CONCURRENT_WORKERS | Auto | 1 - Total Cores | Límite de procesos paralelos | CONFIG |
| PROGRESS_EMISSION_RATE | 100ms | 50ms - 1000ms | Frecuencia de actualización de la UI | CONFIG |
| SHARED_MEMORY_SIZE | 4GB | 1GB - 32GB | Tamaño del segmento de memoria compartida | CONFIG |

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Algoritmo de despacho de tareas (Round Robin o Prioridad). Tipos propios + traits de abstracción. Cero imports de OS, `tokio`, `memmap2` o `nix`.
- **Shell (Infraestructura):** Spawn de procesos con `std::process::Command`/`Child`; buffer Arrow compartido con `memmap2` (PROT\_READ en el worker); supervisión async con `tokio::time`; señales OS con `nix` (`SIGTERM` → espera 2s → `SIGKILL`). Integra con `JobRepository` (STORY-005) para persistir estado y recuperar jobs RUNNING→QUEUED al reiniciar.

## Ciclo de Vida de la Feature — Worker Orchestrator

### Entrada
- Definición de tarea (Task Payload - JSON Validado).
- Prioridad del job.

### Proceso
1.  **Hidratación:** El trabajador reconstruye el objeto de la estrategia y sus parámetros a partir del JSON.
2.  **Compilación AOT:** Los cuellos de botella algorítmicos se compilan mediante Rust SIMD/Rayon.
3.  **Ejecución:** Inicio del motor de simulación de NautilusTrader.
4.  **Persistencia:** Almacenamiento de resultados en archivos Parquet particionados.
5.  **Emisión:** Comunicación de señales de progreso agrupadas (Throttling 100ms).

### Salida
- `JobResult` (referencia al archivo de salida o valor de retorno).
- Estado final en el Event Store de SQLite.

## Tareas (TTRs)

### **TTR-001: Implementación de Bridge de Memoria Compartida**
* **¿Cuál es el problema?** Copiar DataFrames de mercado masivos (OHLCV) a cada proceso worker es extremadamente lento y satura la RAM del host.
* **¿Qué tiene que pasar?** El sistema debe utilizar `shared_memory` de Rust para que todos los procesos visualicen el mismo buffer binario de Apache Arrow sin duplicación.
* **¿Cómo sé que está hecho?**
    - [ ] Tiempo de acceso a datos en el worker < 1ms tras el disparo inicial.
    - [ ] El consumo de RAM no aumenta linealmente con el número de workers activos.
* **¿Qué no puede pasar?** NUNCA permitir la escritura en el segmento de memoria compartida desde los workers (mapping abierto con `PROT_READ` únicamente; un intento de escritura resulta en error del OS).

### **TTR-002: Watchdog de Procesos y Graceful Shutdown**
* **¿Cuál es el problema?** Los procesos worker pueden quedar huérfanos (zombis) si el proceso principal falla, consumiendo recursos infinitamente.
* **¿Qué tiene que pasar?** Implementar un monitor supervisor que termine inmediatamente todos los workers si se detecta que el proceso padre (Orquestador Rust) ha desaparecido o si el job es cancelado.
* **¿Cómo sé que está hecho?**
    - [ ] Limpieza total de procesos en el sistema operativo en < 2s tras una cancelación manual.
    - [ ] El watchdog registra el cierre de cada worker en el log de auditoría (`AuditLogRepository`) con evento `WORKER_TERMINATED`.
* **¿Qué no puede pasar?** PROHIBIDO dejar procesos de NautilusTrader activos tras el cierre del orquestador.

## Puertos de Integración (ADR-0137)

> Obligatorio en toda feature. Define los tipos de dato que la feature acepta (inputs) y produce (outputs).
> Los IDs de tipo deben pertenecer al catálogo de ADR-0137. Un puerto sin tipo declarado es inválido en el Canvas [Forge/Reactor].

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `job_in` | `Job` | Input | 1..N | Job a despachar a un worker aislado. El orquestador consume el Job del repositorio durable (STORY-005), lo asigna a un worker según capacidad y supervisa su ciclo de vida. |
| `job_result_out` | `(interno)` | Output | 1 | Resultado que el worker emite al terminar: `job_id`, `status` (DONE/FAILED), `output_path` (ruta al artefacto generado), `duration_ms`. No es un tipo de canvas; es la firma Rust de la respuesta del worker (Enmienda 2026-06-24 de ADR-0137). |

> **Cardinalidad:** `1` = exactamente uno · `0..1` = opcional · `0..N` = múltiple · `1..N` = al menos uno.
> **Puerto interno (ADR-0137 Enmienda 2026-06-24):** `job_result_out` es un puerto interno — plomería `textLabel`, payload específico de esta feature, nunca cableable en el canvas. No requiere tipo de catálogo.

## Persistencia (Inundación de Fundamentos — ADR-0020 V2 · Perfil D Ops/Auditoría de infraestructura)

Orquestación/aislamiento de procesos worker (infra), no hot-path de mercado → **Perfil D** (I + II + IV), con linaje de job padre-hijo documentado como híbrido legítimo (orquestador→worker):

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del Job de ejecución |
| | `created_at` | Timestamp de inicio de la tarea |
| | `updated_at` | Última actualización de latido (heartbeat) |
| | `audit_hash` | Hash de integridad del estado del Job |
| | `audit_chain_hash` | Hash encadenado del historial de heartbeats |
| | `event_sequence_id` | Secuencia de recuperación del Job |
| **II. Soberanía** | `owner_id` | Usuario que lanzó el Job |
| | `institutional_tag` | Etiqueta de cumplimiento organizacional |
| | `manifest_id` | ID del diseño evaluado |
| | `access_token_id` | Token de seguridad del worker |
| **III. Linaje (híbrido)** | `parent_id` | ID del proceso padre (Orquestador Rust) — linaje job padre-hijo |
| **IV. Hardware** | `process_id` | OS PID del proceso worker |
| | `session_id` | Sesión de orquestación global |
| | `node_id` | ID del hardware de asignación |

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local (Headless support).
- **Inundación de Fundaciones (ADR-0020 V2):** Perfil D + linaje híbrido. Cada proceso worker registra su `process_id` y su `parent_id` (orquestador) en el log de auditoría.
