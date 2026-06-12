# Worker Isolation Orchestrator

**Carpeta:** `./features/worker-isolation-orchestrator.md`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0013 (Stack Tecnológico - Ray/ProcessPool)

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
- La comunicación entre el orquestador y los workers locales debe usar FFI/gRPC eficiente (Shared Memory o ZeroMQ).
- **Extensión Híbrida Remota (HybridComputeCooperative):** En esta modalidad, el orquestador local extiende su pool de trabajadores enviando tareas serializadas a través de gRPC/WebSockets a daemons ejecutores remotos (VPS/bare-metal), recopilando los resultados de forma asíncrona en la persistencia local.


## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| MAX_CONCURRENT_WORKERS | Auto | 1 - Total Cores | Límite de procesos paralelos | CONFIG |
| PROGRESS_EMISSION_RATE | 100ms | 50ms - 1000ms | Frecuencia de actualización de la UI | CONFIG |
| SHARED_MEMORY_SIZE | 4GB | 1GB - 32GB | Tamaño del segmento de memoria compartida | CONFIG |

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Algoritmo de despacho de tareas (Round Robin o Prioridad).
- **Shell (Infraestructura):** Implementación de `multiprocessing.shared_memory` y `Ray` o `concurrent.futures.ProcessPoolExecutor`.

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
* **¿Qué no puede pasar?** NUNCA permitir la escritura en el segmento de memoria compartidadesde los workers (debe ser Read-Only).

### **TTR-002: Watchdog de Procesos y Graceful Shutdown**
* **¿Cuál es el problema?** Los procesos worker pueden quedar huérfanos (zombis) si el proceso principal falla, consumiendo recursos infinitamente.
* **¿Qué tiene que pasar?** Implementar un monitor supervisor que termine inmediatamente todos los workers si se detecta que el proceso padre (Orquestador Rust) ha desaparecido o si el job es cancelado.
* **¿Cómo sé que está hecho?**
    - [ ] Limpieza total de procesos en el sistema operativo en < 2s tras una cancelación manual.
    - [ ] El `Shadow Watchdog` reporta el cierre de workers en el puerto 8002.
* **¿Qué no puede pasar?** PROHIBIDO dejar procesos de NautilusTrader activos tras el cierre del orquestador.

## Persistencia (Filtro de Relevancia AI/R&D — ADR-0020 V2)

Cada Job u operación gestionada por este orquestador debe portar el set filtrado de metadatos para auditoría de infraestructura y hardware:

| Campo | Tipo | Categoría | Descripción |
| :--- | :--- | :--- | :--- |
| `id` | UUID | Identidad | Identificador único del Job de ejecución |
| `created_at` | INT64 | Identidad | Timestamp de inicio de la tarea |
| `updated_at` | INT64 | Identidad | Última actualización de latido (heartbeat) |
| `owner_id` | UUID | Soberanía | Usuario que lanzó el Job |
| `institutional_tag` | VARCHAR | Soberanía | Etiqueta de cumplimiento organizacional |
| `manifest_id` | UUID | Soberanía | ID del diseño evaluado |
| `access_token_id` | UUID | Soberanía | Token de seguridad del worker |
| `logic_hash` | VARCHAR | AI/Arquitectura | Hash del binario/lógica cargada en el worker |
| `indicator_state_hash` | VARCHAR | AI/Arquitectura | Snapshot de indicadores post-ejecución |
| `version_node_id` | UUID | AI/Arquitectura | Versión de la estrategia en el DAG |
| `parent_id` | UUID | AI/Arquitectura | ID del proceso padre (Orquestador Rust) |
| `process_id` | INT32 | Hardware | OS PID del proceso worker |
| `session_id` | UUID | Hardware | Sesión de orquestación global |
| `node_id` | VARCHAR | Hardware | ID del hardware de asignación |

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local (Headless support).
- **Inundación de Fundaciones (ADR-0020 V2):** Cada proceso worker debe registrar su `process_id` y `parent_node_id` en el log de auditoría.
