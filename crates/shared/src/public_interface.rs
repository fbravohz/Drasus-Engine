//! [SHELL] Public interface (port) of `shared`.
//!
//! This is the only surface pipeline modules (`ingest`, `generate`,
//! `validate`, `incubate`, `manage`, `execute`, `feedback`, `withdraw`)
//! may depend on when reusing common components (ADR-0003).
//!
//! ## Clock (W3, `docs/features/clock.md`)
//!
//! Every module that needs the current time depends on the [`Clock`] port
//! instead of calling the system clock directly:
//!
//! - [`SystemClock`]: production implementation (TTR-001, `request_type =
//!   REAL`), nanosecond-precision and monotonically non-decreasing.
//! - [`DeterministicClock`]: backtest/test implementation (TTR-002,
//!   `request_type = FAKE`), advances only via explicit `advance(ns)` /
//!   `tick()` calls — same seed (`initial_timestamp_ns`, `step_ns`) and
//!   same call sequence produce an identical timestamp sequence, bit for
//!   bit.
//!
//! ## Audit Log (`docs/features/audit-log.md` TTR-001)
//!
//! Every module fires audit events through [`AuditLogRepository::append`]
//! instead of writing logs directly (audit-log.md: "El Core nunca escribe
//! logs. En su lugar, dispara eventos al puerto de auditoría injected.").
//!
//! - [`AuditEventContent`]: the event payload (`action_type`,
//!   `entity_type`, `entity_id`, `details_json`, plus the ADR-0020 V2
//!   "Ops / Auditoría" profile fields — `process_id` and
//!   `institutional_tag` are mandatory).
//! - [`AuditEvent`]: a persisted, hash-chained event (`audit_hash`,
//!   `audit_chain_hash`, `event_sequence_id`).
//! - [`AuditLogRepository`]: append-only repository (`append`,
//!   `load_chain`, `events_for_entity`) — no update/delete surface exists.
//! - [`verify_chain`] / [`ChainVerificationResult`]: pure hash-chain
//!   verification, detects tampering with historical events.
//! - [`AuditLogError`]: error type for repository operations.
//!
//! ## Clock Audit Trail (`docs/features/clock.md` "Gobernanza y Estándares")
//!
//! The Clock has no persistence of its own — its three auditable events are
//! emitted through [`AuditLogRepository::append`] via
//! [`ClockAuditContext`] and the three `emit_*` functions below. Granularity
//! is fixed at exactly these three events; `timestamp_ns()`, `advance(ns)`
//! and `tick()` never emit audit events.
//!
//! - [`ClockAuditContext`]: caller-supplied identity (`session_id`,
//!   `institutional_tag`, `process_id`) shared by all three events.
//! - [`ClockMode`]: `REAL` / `SIMULATION`, used by [`emit_mode_transition`].
//! - [`emit_ntp_sync`]: `CLOCK_NTP_SYNC` (TTR-001, once at startup).
//! - [`emit_mode_transition`]: `CLOCK_MODE_TRANSITION` (on `REAL` <->
//!   `SIMULATION` transitions).
//! - [`emit_session_close`]: `CLOCK_SESSION_CLOSE` (TTR-002, once when a
//!   simulation session closes).
//!
//! ## Async Job Executor (`docs/features/async-job-executor.md`)
//!
//! Three-phase async job pattern (ADR-0011): submit a job, poll its status
//! and progress, fetch its immutable result once terminal.
//!
//! - [`JobState`]: the job state machine's five states + pure
//!   [`validate_transition`] (TTR-002/004/006).
//! - [`Progress`] / [`estimate_remaining_seconds`]: 0-100 progress and
//!   time-remaining estimation (TTR-005).
//! - [`Job`] / [`JobResult`] / [`NewJob`] / [`NewJobResult`] /
//!   [`RecoveredJob`]: persistence-layer types (`jobs`/`job_results`,
//!   migration `0003_jobs.sql`).
//! - [`JobRepository`] / [`JobRepositoryError`]: the `jobs`/`job_results`
//!   repository (TTR-001/003/004).
//! - [`JobExecutor`] / [`JobExecutorConfig`] / [`ExecutorIdentity`] /
//!   [`JobExecutorError`]: the executor shell — submit, recover at startup,
//!   spawn the worker pool, poll status/result, cancel (TTR-001/002/004/006).
//! - [`JobHandler`] / [`JobOutcome`] / [`ProgressReporter`] /
//!   [`CancellationToken`]: the pluggable per-`job_type` callback contract
//!   (TTR-002/005/006). TTR-ASYNC-EXECUTOR-007 (wiring real handlers from
//!   `generate`/`validate`/`manage`/`incubate`/`feedback`) is out of scope
//!   for this story.

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
pub use crate::orchestrator::SystemClock;
pub use crate::persistence::audit_log::{AuditLogError, AuditLogRepository};
pub use crate::persistence::job::{Job, JobRepository, JobRepositoryError, JobResult, NewJob, NewJobResult, RecoveredJob};
