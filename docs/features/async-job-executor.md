# Async Job Executor — Patrón Asincrónico de Operaciones Costosas

**Carpeta:** `./features/async-job-executor/`  
**Estado:** Lista para implementar  
**Última actualización:** 2026-04-08  
**Decisión Arquitectónica Asociada:** ADR-0011 (Operaciones Asincrónicas)

---

## ¿Qué es?

Async Job Executor implementa un patrón de tres fases para ejecutar operaciones computacionalmente costosas (backtesting, generación, optimización) sin bloquear la interfaz del usuario.

**Problema:** Backtesting masivo puede tardar minutos/horas. Si ejecuta sincrónico, usuario está bloqueado, no puede hacer nada más.

**Solución:** Operación se envía a un job queue. Sistema la procesa en background. Usuario obtiene UUID y puede consultar progreso en cualquier momento. Cuando termina, usuario obtiene resultados.

**Resultado observable:** Usuario dispara job costoso, interfaz sigue responsiva, puede monitorear progreso, obtiene resultados cuando estén listos.

---

## Comportamientos Observables

- [ ] Usuario solicita backtest de estrategia (operación costosa):
  ```
  POST /api/backtest/submit
  { "strategy_id": 123, "date_range": "2024-01-01:2024-12-31" }
  ```
  → Sistema crea registro en tabla `jobs` (`state=QUEUED`, `uuid=abc-123`)
  → Responde inmediatamente: `{ "job_uuid": "abc-123", "state": "QUEUED" }`
  → Usuario no está bloqueado

- [ ] Usuario consulta estado del job mientras se ejecuta:
  ```
  GET /api/jobs/abc-123/status
  ```
  → Retorna: `{ "state": "RUNNING", "progress": 45, "estimated_time_remaining": "2 minutes" }`
  → Usuario puede consultar múltiples veces sin bloqueos

- [ ] Job completa exitosamente:
  - Estado cambia a `COMPLETED`
  - Resultados se guardan en tabla `job_results` (append-only, nunca se modifican)
  - Usuario obtiene: `{ "state": "COMPLETED", "result_uuid": "xyz-456" }`

- [ ] Usuario obtiene resultados de job completado:
  ```
  GET /api/jobs/abc-123/result
  ```
  → Retorna: `{ "metrics": {...}, "cagr": 0.25, "sharpe": 2.1, ... }`
  → Resultado es inmutable (snapshot de cuando completó)

- [ ] Job falla (ej: error de datos):
  - Estado cambia a `FAILED`
  - Mensaje de error se guarda: "Invalid date range: start > end"
  - Usuario obtiene: `{ "state": "FAILED", "error": "Invalid date range: ..." }`

- [ ] Usuario cancela un job en progreso:
  ```
  POST /api/jobs/abc-123/cancel
  ```
  → Sistema marca job como `CANCELLED`
  → Worker actual detiene ejecución gracefully
  → Estado retorna: `{ "state": "CANCELLED" }`

- [ ] Sistema reinicia inesperadamente (crash):
  - Jobs en estado `QUEUED`/`RUNNING` se recuperan de tabla SQLite
  - En startup, worker re-encolam jobs recuperados
  - Auditoría registra: "SYSTEM_RESTART: recovered 3 jobs"

- [ ] Múltiples jobs se disparan en paralelo:
  ```
  Job 1: backtest estrategia A
  Job 2: generar 100 candidatas
  Job 3: optimizar portafolio
  ```
  → Los 3 se encolan
  → Se ejecutan concurrentemente (máximo `max_concurrent_jobs`)
  → Usuario monitorea todos sin esperar

---

## Restricciones

- **NUNCA un job se modifica después de completar.** Resultado es inmutable.
- **NUNCA se pierden job results.** Append-only en tabla SQLite.
- **NUNCA se ejecutan más de `max_concurrent_jobs` jobs simultáneamente.** Hard limit de recursos.
- **NUNCA un job QUEUED permanece indefinidamente.** Timeout configurable (default: sin límite, pero recomendado 1 hora).
- **NUNCA se olvidan jobs después de crash.** Recuperación automática en startup.
- **NUNCA un job retorna resultado parcial.** Es TODO exitoso o TODO fallido (atómico).

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | CONFIG/FIJO |
|-----------|---------|-------|----------|------------|
| `max_concurrent_jobs` | 3 | 1-16 | Máximo de jobs ejecutándose simultáneamente | CONFIG |
| `job_timeout` | 3600 | 60-86400 | Timeout en segundos para un job (0 = sin límite) | CONFIG |
| `job_queue_size` | 1000 | 10-10000 | Máximo de jobs encolados antes de rechazar nuevos | CONFIG |
| `progress_interval` | 5 | 1-60 | Segundos entre actualizaciones de progreso | CONFIG |
| `result_retention_days` | 365 | 1-3650 | Cuántos días guardar job results antes de archivar | CONFIG |
| `persist_to_disk` | true | true/false | Si true, jobs se guardan en SQLite para recuperación en crash | FIJO |

---

## Ciclo de Vida de la Feature

### Entrada
- **Quién llama:** Módulos costosos (generate, validate, manage, incubate, feedback)
- **Qué recibe:** Request de job (tipo de operación, parámetros, usuario_id)

### Proceso

#### Fase 1: Disparo (Submit)
1. Sistema crea registro en tabla `jobs`: uuid, user_id, job_type, parameters, state=QUEUED, created_at
2. Job entra a `tokio.Queue` (in-memory)
3. Sistema retorna UUID inmediatamente

#### Fase 2: Monitoreo (Poll)
1. Worker thread toma job de queue cuando hay capacity
2. Worker cambia estado a RUNNING, inicializa progreso=0
3. Usuario consulta endpoint `/api/jobs/{uuid}/status` periódicamente
4. Retorna: estado, progreso (0-100%), estimación de tiempo restante
5. Worker actualiza progreso cada `progress_interval` segundos

#### Fase 3: Recuperación (Fetch)
1. Job completa (exitoso o con error)
2. Estado cambia a COMPLETED o FAILED
3. Resultado se guarda en tabla `job_results` (append-only)
4. Usuario consulta endpoint `/api/jobs/{uuid}/result`
5. Retorna resultado inmutable + timestamp de finalización

### Salida
- **Produce:** UUID de job, estado, progreso, resultado eventual

### Contextos de Uso
- **Generate:** Generar N candidatas con algoritmo evolutivo (minutos a horas)
- **Validate:** Backtesting robusto WFA+MC (minutos)
- **Manage:** Optimización de portafolio (minutos)
- **Incubate:** Forward testing histórico (minutos)
- **Feedback:** Análisis diario de anomalías (segundos a minutos)

---

## Tareas (TTRs)

### **TTR-ASYNC-EXECUTOR-001: Implementar Job Queue con AsyncIO + SQLite**

**Qué hace:** Crear JobExecutor que maneja jobs en memoria (rápido) + SQLite (durabilidad).

**Entrada:**
- Request de job: `JobRequest(job_type, parameters, user_id)`

**Salida:**
- UUID único del job
- Job guardado en tabla `jobs`
- Job encolado en `tokio.Queue`

**Reglas de Negocio:**
- UUID debe ser único e irrepetible (usar uuid.uuid4())
- Job se guarda ANTES de retornar UUID (garantía de persistencia)
- Queue en memoria para velocidad, disco para durabilidad

**Precondiciones:**
- JobExecutor inicializado
- Tabla `jobs` creada en SQLite

**Postcondiciones:**
- UUID retornado
- Job en cola esperando ejecución

---

### **TTR-ASYNC-EXECUTOR-002: Implementar Worker Pool (Ejecutores Paralelos)**

**Qué hace:** N workers que toman jobs de la queue y los ejecutan.

**Entrada:**
- Job de la queue
- Funciones callback que ejecutar (función que hace el trabajo real)

**Salida:**
- Job completado o con error
- Estado actualizado a COMPLETED/FAILED

**Reglas de Negocio:**
- Máximo `max_concurrent_jobs` workers corriendo simultáneamente
- Si queue está vacía, workers esperan sin consumir CPU
- Si job tira excepción, se captura y guarda como FAILED

**Precondiciones:**
- JobExecutor inicializado
- `max_concurrent_jobs` configurado

**Postcondiciones:**
- Job ejecutado o fallido
- Estado reflejado en tabla `jobs`

---

### **TTR-ASYNC-EXECUTOR-003: Implementar Persistencia en SQLite**

**Qué hace:** Tabla `jobs` y tabla `job_results` que guardan jobs y resultados.

**Entrada:**
- Job object (uuid, user_id, type, parameters, state, progress, timestamps)
- Result object (job_uuid, result_data, error_message, completed_at)

**Salida:**
- Rows en SQLite con datos persistidos

**Reglas de Negocio:**
- Tabla `jobs`: cada fila es un job, se actualiza conforme progresa (state, progress)
- Tabla `job_results`: append-only, nunca se modifica después de insertar
- Ambas tablas tienen timestamp para auditoría

**Precondiciones:**
- SQLite inicializado
- Migraciones ejecutadas (SQLx Migrator)

**Postcondiciones:**
- Datos durables en disco
- Recuperables ante crash

---

### **TTR-ASYNC-EXECUTOR-004: Implementar Recuperación en Startup**

**Qué hace:** Cuando sistema arranca, recuperar jobs incompletos de SQLite.

**Entrada:**
- Tabla `jobs` con registros en estado QUEUED o RUNNING

**Salida:**
- Jobs reencolados en `tokio.Queue`
- Estado actualizado a QUEUED (no RUNNING, porque no sabemos si completó)

**Reglas de Negocio:**
- QUEUED jobs se recuperan directamente
- RUNNING jobs: cambiar a QUEUED (asumir que no completó, porque si completó estaría en COMPLETED)
- Registrar en audit: "JOB_RECOVERED_AT_STARTUP: job_uuid=..., previous_state=..."

**Precondiciones:**
- SQLite disponible
- Sistema arrancando

**Postcondiciones:**
- Jobs recuperados y esperando ejecución
- Auditoría registrada

---

### **TTR-ASYNC-EXECUTOR-005: Implementar Progreso y Estimación de Tiempo**

**Qué hace:** Permitir que job report progreso, sistema estima tiempo restante.

**Entrada:**
- Job en ejecución
- Callback de progreso: `on_progress(percent: float, elapsed_seconds: float)`

**Salida:**
- Estado actualizado: `{ progress: 45, estimated_remaining: 120 }`

**Reglas de Negocio:**
- Progreso es 0-100%
- Estimación = (elapsed_time / progress) × (100 - progress)
- Se actualiza cada `progress_interval` segundos

**Precondiciones:**
- Job ejecutándose

**Postcondiciones:**
- Usuario puede consultar progreso en cualquier momento

---

### **TTR-ASYNC-EXECUTOR-006: Implementar Cancellación de Jobs**

**Qué hace:** Usuario puede cancelar un job en QUEUED o RUNNING.

**Entrada:**
- Job UUID
- Señal de cancelación

**Salida:**
- Job marcado como CANCELLED
- Worker detiene ejecución
- Resultado parcial se descarta

**Reglas de Negocio:**
- QUEUED jobs se cancelan inmediatamente (no se ejecutan)
- RUNNING jobs se cancelan con cancel token (worker verifica periódicamente)
- Una vez CANCELLED, no se puede reanudar

**Precondiciones:**
- Job en estado QUEUED o RUNNING

**Postcondiciones:**
- Job en estado CANCELLED
- No hay resultado (o resultado parcial se ignora)

---

### **TTR-ASYNC-EXECUTOR-007: Integrar con Módulos Costosos (generate, validate, manage, etc.)**

**Qué hace:** Cada módulo costoso usa JobExecutor en lugar de ejecutar sincrónico.

**Entrada:**
- Módulo que necesita ejecutar operación costosa

**Salida:**
- Job UUID retornado al usuario

**Reglas de Negocio:**
- Módulo no ejecuta lógica directamente, la delega a job
- Módulo recibe callback que reporta progreso
- Módulo puede ser cancelado por usuario

**Precondiciones:**
- JobExecutor inicializado
- Módulo refactorizado para usar async pattern

**Postcondiciones:**
- Usuario puede monitorear progreso
- Interfaz responsiva

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. La cola de trabajos y sus resultados se gestionan íntegramente en el hardware local.
- **Inundación de Fundaciones (ADR-0020 V2):** 
    - Toda tarea y resultado de job registra el set completo de **25 campos mandatorios** (ver ADR-0020 V2 V2).
    - Metadatos de concurrencia e integridad: `process_id` (Worker ID), `session_id`, `node_id` (Hardware Fingerprint), `audit_chain_hash`, `logic_hash` (Executor version), `event_sequence_id`.
    - Soberanía: `owner_id`, `access_token_id`.


---

## Dependencias y Bloqueantes

- **Requiere:** ADR-0011 (patrón definido)
- **Requiere:** SQLite con WAL (persistencia)
- **Requiere:** SQLx Migrator (migraciones para tablas jobs/results)
- **Requiere:** AsyncIO (Rust)
- **Habilita:** Operaciones costosas sin bloquear UI
- **Habilita:** ADR-0012 (multi-pipeline si se usa job executor para pipelines)

---

## Referencias

- `ADR.md` → ADR-0011: Operaciones Asincrónicas
- `ADR.md` → ADR-0012: Arquitectura Multi-Pipeline (usa job executor)
- `features/infrastructure-setup/` → SQLite + SQLx Migrator
