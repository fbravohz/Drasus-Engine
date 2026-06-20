//! [SHELL] Interfaz pública (puerto) de `shared`.
//!
//! Esta es la única superficie de la que pueden depender los módulos del
//! pipeline (`ingest`, `generate`, `validate`, `incubate`, `manage`,
//! `execute`, `feedback`, `withdraw`) al reusar componentes comunes
//! (ADR-0003).
//!
//! ## Clock (W3, `docs/features/clock.md`)
//!
//! Cada módulo que necesita la hora actual depende del puerto [`Clock`]
//! en vez de llamar directo al reloj del sistema:
//!
//! - [`SystemClock`]: implementación de producción (TTR-001,
//!   `request_type = REAL`), con precisión de nanosegundos y monótona no
//!   decreciente.
//! - [`DeterministicClock`]: implementación de backtest/test (TTR-002,
//!   `request_type = FAKE`), que solo avanza vía llamadas explícitas a
//!   `advance(ns)` / `tick()` — la misma semilla
//!   (`initial_timestamp_ns`, `step_ns`) y la misma secuencia de llamadas
//!   producen una secuencia de timestamps idéntica, bit a bit.
//!
//! ## Audit Log (`docs/features/audit-log.md` TTR-001)
//!
//! Cada módulo dispara eventos de auditoría a través de
//! [`AuditLogRepository::append`] en vez de escribir logs directamente
//! (audit-log.md: "El Core nunca escribe logs. En su lugar, dispara
//! eventos al puerto de auditoría injected.").
//!
//! - [`AuditEventContent`]: el payload del evento (`action_type`,
//!   `entity_type`, `entity_id`, `details_json`, más los campos del
//!   perfil "Ops / Auditoría" de ADR-0020 V2 — `process_id` e
//!   `institutional_tag` son obligatorios).
//! - [`AuditEvent`]: un evento persistido y encadenado por hash
//!   (`audit_hash`, `audit_chain_hash`, `event_sequence_id`).
//! - [`AuditLogRepository`]: repositorio de solo-apéndice (`append`,
//!   `load_chain`, `events_for_entity`) — no existe superficie de
//!   update/delete.
//! - [`verify_chain`] / [`ChainVerificationResult`]: verificación pura de
//!   la cadena de hashes, detecta manipulación de eventos históricos.
//! - [`AuditLogError`]: tipo de error para operaciones del repositorio.
//!
//! ## Rastro de Auditoría del Clock (`docs/features/clock.md` "Gobernanza y Estándares")
//!
//! El Clock no tiene persistencia propia — sus tres eventos auditables se
//! emiten vía [`AuditLogRepository::append`] a través de
//! [`ClockAuditContext`] y las tres funciones `emit_*` de abajo. La
//! granularidad está fija en exactamente estos tres eventos;
//! `timestamp_ns()`, `advance(ns)` y `tick()` nunca emiten eventos de
//! auditoría.
//!
//! - [`ClockAuditContext`]: identidad provista por quien llama
//!   (`session_id`, `institutional_tag`, `process_id`) compartida por
//!   los tres eventos.
//! - [`ClockMode`]: `REAL` / `SIMULATION`, usado por
//!   [`emit_mode_transition`].
//! - [`emit_ntp_sync`]: `CLOCK_NTP_SYNC` (TTR-001, una vez al iniciar).
//! - [`emit_mode_transition`]: `CLOCK_MODE_TRANSITION` (en transiciones
//!   `REAL` <-> `SIMULATION`).
//! - [`emit_session_close`]: `CLOCK_SESSION_CLOSE` (TTR-002, una vez
//!   cuando cierra una sesión de simulación).
//!
//! ## Async Job Executor (`docs/features/async-job-executor.md`)
//!
//! Patrón de job asíncrono de tres fases (ADR-0011): enviar un job,
//! sondear su estado y progreso, recuperar su resultado inmutable una
//! vez terminal.
//!
//! - [`JobState`]: los cinco estados de la máquina de estados del job +
//!   [`validate_transition`] puro (TTR-002/004/006).
//! - [`Progress`] / [`estimate_remaining_seconds`]: progreso 0-100 y
//!   estimación de tiempo restante (TTR-005).
//! - [`Job`] / [`JobResult`] / [`NewJob`] / [`NewJobResult`] /
//!   [`RecoveredJob`]: tipos de la capa de persistencia
//!   (`jobs`/`job_results`, migración `0003_jobs.sql`).
//! - [`JobRepository`] / [`JobRepositoryError`]: el repositorio de
//!   `jobs`/`job_results` (TTR-001/003/004).
//! - [`JobExecutor`] / [`JobExecutorConfig`] / [`ExecutorIdentity`] /
//!   [`JobExecutorError`]: la cáscara del executor — enviar, recuperar
//!   en startup, levantar el pool de workers, sondear estado/resultado,
//!   cancelar (TTR-001/002/004/006).
//! - [`JobHandler`] / [`JobOutcome`] / [`ProgressReporter`] /
//!   [`CancellationToken`]: el contrato de callback enchufable por
//!   `job_type` (TTR-002/005/006). TTR-ASYNC-EXECUTOR-007 (conectar
//!   handlers reales desde `generate`/`validate`/`manage`/`incubate`/
//!   `feedback`) está fuera de alcance para esta historia.
//!
//! ## Telemetría (`docs/features/telemetry.md` TTR-001)
//!
//! Buffer de alta velocidad: cualquier módulo registra una muestra de
//! latencia o un heartbeat sin esperar al disco; una tarea de fondo vacía
//! la cola a SQLite por lotes.
//!
//! - [`TelemetrySample`] / [`TelemetrySampleContent`] / [`build_sample`] /
//!   [`expired_sample_ids`]: núcleo puro — construcción de una muestra
//!   encadenada y la decisión de poda por ventana de retención.
//! - [`TelemetryRepository`] / [`TelemetryError`]: repositorio de
//!   `telemetry_samples` (insertar por lote, purgar, consultar por
//!   `metric_name` + rango, migración `0004_telemetry.sql`).
//! - [`TelemetryBuffer`] / [`TelemetryBufferConfig`]: la cáscara — cola en
//!   memoria no bloqueante (`record_latency`/`record_heartbeat`), siembra
//!   de la cadena al iniciar (`bootstrap`), vaciado por lotes en segundo
//!   plano (`spawn_flush_task`) y poda (`purge`). Reusa [`ExecutorIdentity`]
//!   del Async Job Executor — mismo perfil de campos ADR-0020 V2, no se
//!   duplica el tipo.

pub use crate::clock_audit::{
    emit_mode_transition, emit_ntp_sync, emit_session_close, ClockAuditContext, ClockMode,
};
pub use crate::domain::audit_log::{
    AuditEvent, AuditEventContent, ChainVerificationResult, verify_chain,
};
pub use crate::domain::clock::{Clock, DeterministicClock};
pub use crate::domain::job::{estimate_remaining_seconds, validate_transition, InvalidTransition, JobState, Progress};
pub use crate::orchestrator::job_executor::{
    CancellationToken, ExecutorIdentity, JobExecutor, JobExecutorConfig, JobExecutorError, JobHandler, JobOutcome,
    ProgressReporter, JOB_RECOVERED_AT_STARTUP,
};
pub use crate::orchestrator::telemetry::{TelemetryBuffer, TelemetryBufferConfig};
pub use crate::orchestrator::SystemClock;
pub use crate::persistence::audit_log::{AuditLogError, AuditLogRepository};
pub use crate::persistence::job::{Job, JobRepository, JobRepositoryError, JobResult, NewJob, NewJobResult, RecoveredJob};
pub use crate::persistence::telemetry::{TelemetryError, TelemetryRepository};
pub use crate::domain::telemetry::{build_sample, expired_sample_ids, TelemetrySample, TelemetrySampleContent};
pub use crate::domain::worker_orchestrator::{WorkerBackend, WorkerBackendError, WorkerConfig, WorkerOrchestrator};
pub use crate::orchestrator::worker_runner::{graceful_shutdown, is_process_alive, open_readonly, OsWorkerBackend, SharedMemorySegment, ShmError};
