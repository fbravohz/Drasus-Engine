//! [SHELL] Async Job Executor (`docs/features/async-job-executor.md`
//! TTR-ASYNC-EXECUTOR-001/002/004/005/006, ADR-0011, ADR-0016).
//!
//! Coordinates the pure state machine in [`crate::domain::job`] with:
//! - [`crate::persistence::job::JobRepository`] (durability — SQLite,
//!   `jobs`/`job_results`).
//! - [`crate::persistence::audit_log::AuditLogRepository`] (the
//!   `JOB_RECOVERED_AT_STARTUP` audit event, TTR-004).
//! - A Tokio in-memory queue + a bounded worker pool (TTR-002,
//!   `max_concurrent_jobs`).
//!
//! ## Three-phase pattern (ADR-0011)
//!
//! 1. **Disparo (submit)**: [`JobExecutor::submit`] persists the job via
//!    [`JobRepository::submit`] — durable on disk — and only then enqueues
//!    its id in memory and returns the UUID (persist-before-ack, TTR-001).
//! 2. **Monitoreo (poll)**: [`JobExecutor::status`] /
//!    [`JobExecutor::cancel`] let the caller inspect or cancel a job.
//! 3. **Recuperación (fetch)**: [`JobExecutor::result`] returns the
//!    immutable [`JobResult`] once a job reaches a terminal state.
//!
//! ## Startup recovery (TTR-004)
//!
//! [`JobExecutor::recover_at_startup`] MUST be called once, after
//! construction and before [`JobExecutor::spawn_workers`]. It scans `jobs`
//! for rows in `QUEUED` or `RUNNING`:
//! - `QUEUED` rows are re-enqueued as-is.
//! - `RUNNING` rows are transitioned to `QUEUED` (completion is unknown —
//!   ADR-0011 "Auto-Recovery") and re-enqueued.
//!
//! For every recovered job, a `JOB_RECOVERED_AT_STARTUP` audit event is
//! appended via [`AuditLogRepository::append`] (the existing audit log —
//! `docs/features/async-job-executor.md` TTR-004: "Registrar en audit:
//! ... NO inventes otra forma de auditar"), with `job_uuid` and
//! `previous_state` in `details_json`.
//!
//! ## Worker pool (TTR-002)
//!
//! [`JobExecutor::spawn_workers`] starts a dispatcher task that pulls job
//! ids from the in-memory queue and, for each, acquires one of
//! `max_concurrent_jobs` [`tokio::sync::Semaphore`] permits before spawning
//! the job's execution task — enforcing the hard concurrency limit
//! (`docs/features/async-job-executor.md` "Restricciones": "NUNCA se
//! ejecutan más de `max_concurrent_jobs` jobs simultáneamente").
//!
//! ## Handlers (TTR-002 "Entrada": "Funciones callback que ejecutar")
//!
//! [`JobHandler`] is the pluggable callback a worker invokes to do the
//! actual (costly) work. TTR-ASYNC-EXECUTOR-007 — wiring real handlers from
//! `generate`/`validate`/`manage`/`incubate`/`feedback` — is explicitly OUT
//! OF SCOPE for this story; those modules do not exist yet. A job whose
//! `job_type` has no registered handler fails immediately with a
//! descriptive [`JobOutcome::Failure`] — this is generic executor behavior,
//! not business logic.
//!
//! ## Cancellation (TTR-006)
//!
//! - A `QUEUED` job is cancelled immediately:
//!   [`JobExecutor::cancel`] transitions it straight to `CANCELLED` in
//!   SQLite. The dispatcher skips ids whose stored state is no longer
//!   `QUEUED` when it is their turn to run.
//! - A `RUNNING` job is cancelled cooperatively: [`JobExecutor::cancel`]
//!   flips a per-job [`CancellationToken`] that the running
//!   [`JobHandler::execute`] is expected to poll
//!   ([`CancellationToken::is_cancelled`]). Once the handler returns, the
//!   worker transitions the job to `CANCELLED` and records no result.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde_json::json;
use sqlx::SqlitePool;
use tokio::sync::{mpsc, Mutex, Semaphore};

use crate::domain::audit_log::AuditEventContent;
use crate::domain::clock::Clock;
use crate::domain::job::{JobState, Progress};
use crate::persistence::audit_log::{AuditLogError, AuditLogRepository};
use crate::persistence::job::{Job, JobRepository, JobRepositoryError, NewJob, NewJobResult, RecoveredJob};

/// `action_type` for the startup-recovery audit event (TTR-004: "Registrar
/// en audit: 'JOB_RECOVERED_AT_STARTUP: job_uuid=..., previous_state=...'").
pub const JOB_RECOVERED_AT_STARTUP: &str = "JOB_RECOVERED_AT_STARTUP";

/// Configuration for a [`JobExecutor`] (`docs/features/async-job-executor.md`
/// "Parámetros Configurables"). Only the parameters this story implements
/// are included; `job_timeout`, `job_queue_size`, `result_retention_days`
/// are deferred to later stories.
#[derive(Debug, Clone)]
pub struct JobExecutorConfig {
    /// Hard limit on simultaneously running jobs (default 3, range 1-16).
    pub max_concurrent_jobs: usize,
    /// Seconds between progress updates a worker is expected to emit
    /// (TTR-005 `progress_interval`, default 5). This is a contract for
    /// handler implementations; the executor itself does not enforce a
    /// timer.
    pub progress_interval_seconds: u64,
}

impl Default for JobExecutorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_jobs: 3,
            progress_interval_seconds: 5,
        }
    }
}

/// ADR-0020 V2 metadata identifying the executor instance/process
/// (`docs/features/async-job-executor.md` "Gobernanza y Estándares":
/// `process_id`, `session_id`, `node_id`, `logic_hash`, plus the audit log's
/// mandatory `institutional_tag`). Injected by the caller — never invented
/// inside this shell.
#[derive(Debug, Clone)]
pub struct ExecutorIdentity {
    /// Worker ID / Job Anchor for jobs this executor instance picks up, and
    /// `process_id` on the `JOB_RECOVERED_AT_STARTUP` audit event.
    pub process_id: String,
    /// Runtime Grouping, stamped on every job this executor submits.
    pub session_id: Option<String>,
    /// Hardware Fingerprint, stamped on every job this executor submits.
    pub node_id: Option<String>,
    /// Executor version (commit/build hash), stamped on every job this
    /// executor submits.
    pub logic_hash: Option<String>,
    /// Mandatory on every audit event (audit-log.md TTR-001).
    pub institutional_tag: String,
}

/// Cooperative cancellation signal for a running job (TTR-006).
///
/// Cloning shares the same underlying flag — the executor holds one clone,
/// the spawned job task holds another, and (if the handler propagates it)
/// the handler holds a third.
#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Signals cancellation. Idempotent.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Returns `true` once [`Self::cancel`] has been called.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Reports progress for a running job (TTR-005).
///
/// Wraps [`JobRepository::update_progress`] plus the [`Clock`] reading
/// needed for [`crate::domain::job::estimate_remaining_seconds`]. Handed to
/// [`JobHandler::execute`] so handlers never touch the pool/clock directly.
pub struct ProgressReporter<'a> {
    repo: &'a JobRepository<'a>,
    job: Job,
}

impl<'a> ProgressReporter<'a> {
    /// Persists `progress` (0-100, clamped) for the wrapped job and updates
    /// the reporter's internal snapshot so subsequent calls chain correctly
    /// (`event_sequence_id`/`audit_chain_hash`).
    pub async fn report(&mut self, progress: u8) -> Result<(), JobRepositoryError> {
        let updated = self.repo.update_progress(&self.job, Progress::new(progress)).await?;
        self.job = updated;
        Ok(())
    }

    /// The job's current `created_at`, in nanoseconds since the Unix epoch
    /// — handlers use this with the executor's [`Clock`] to compute elapsed
    /// time for [`crate::domain::job::estimate_remaining_seconds`].
    pub fn started_at_ns(&self) -> i64 {
        self.job.created_at_ns
    }
}

/// Outcome of [`JobHandler::execute`] (TTR-002/003).
#[derive(Debug, Clone)]
pub enum JobOutcome {
    /// The job finished successfully. `result_data` is the JSON-encoded
    /// payload persisted to `job_results.result_data`.
    Success { result_data: String },
    /// The job failed. `error_message` is persisted to
    /// `job_results.error_message` (TTR-002 "Si job tira excepción, se
    /// captura y guarda como FAILED").
    Failure { error_message: String },
}

/// The pluggable callback a worker invokes to perform a job's actual work
/// (TTR-002 "Entrada": "Funciones callback que ejecutar").
///
/// TTR-ASYNC-EXECUTOR-007 (wiring `generate`/`validate`/`manage`/`incubate`/
/// `feedback` as real handlers) is explicitly out of scope for this story.
#[async_trait::async_trait]
pub trait JobHandler: Send + Sync {
    /// Executes `job`. Implementations should periodically call
    /// `progress.report(...)` (TTR-005) and check `cancel.is_cancelled()`
    /// (TTR-006), returning as soon as cancellation is observed.
    async fn execute(&self, job: &Job, progress: &mut ProgressReporter<'_>, cancel: &CancellationToken) -> JobOutcome;
}

/// Errors returned by [`JobExecutor`] operations.
#[derive(Debug)]
pub enum JobExecutorError {
    Repository(JobRepositoryError),
    Audit(AuditLogError),
    /// [`JobExecutor::cancel`] was called for a job id that does not exist,
    /// or that is already in a terminal state.
    JobNotCancellable(String),
}

impl std::fmt::Display for JobExecutorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobExecutorError::Repository(err) => write!(f, "job executor repository error: {err}"),
            JobExecutorError::Audit(err) => write!(f, "job executor audit error: {err}"),
            JobExecutorError::JobNotCancellable(id) => {
                write!(f, "job '{id}' cannot be cancelled (not found or already terminal)")
            }
        }
    }
}

impl std::error::Error for JobExecutorError {}

impl From<JobRepositoryError> for JobExecutorError {
    fn from(err: JobRepositoryError) -> Self {
        JobExecutorError::Repository(err)
    }
}

impl From<AuditLogError> for JobExecutorError {
    fn from(err: AuditLogError) -> Self {
        JobExecutorError::Audit(err)
    }
}

/// Shared state behind [`JobExecutor`]'s cheap-to-clone handle.
struct Shared {
    pool: SqlitePool,
    clock: Arc<dyn Clock>,
    identity: ExecutorIdentity,
    config: JobExecutorConfig,
    handlers: HashMap<String, Arc<dyn JobHandler>>,
    semaphore: Arc<Semaphore>,
    cancel_tokens: Mutex<HashMap<String, CancellationToken>>,
}

/// The Async Job Executor (`docs/features/async-job-executor.md`).
///
/// Cheap to clone (an `Arc` handle): clones share the same in-memory queue,
/// semaphore and cancellation tokens, all backed by the same SQLite pool.
#[derive(Clone)]
pub struct JobExecutor {
    shared: Arc<Shared>,
    queue_tx: mpsc::UnboundedSender<String>,
    queue_rx: Arc<Mutex<mpsc::UnboundedReceiver<String>>>,
}

impl JobExecutor {
    /// Creates a new executor bound to `pool` (already migrated — see
    /// [`crate::persistence::pool::connect`] +
    /// [`crate::persistence::pool::migrate`]) and `clock`.
    ///
    /// `handlers` maps `job_type` -> [`JobHandler`]. A job whose `job_type`
    /// has no entry fails immediately when picked up (see module docs).
    ///
    /// Does NOT start workers and does NOT recover jobs from a previous run
    /// — call [`Self::recover_at_startup`] then [`Self::spawn_workers`]
    /// explicitly, in that order.
    pub fn new(
        pool: SqlitePool,
        clock: Arc<dyn Clock>,
        identity: ExecutorIdentity,
        config: JobExecutorConfig,
        handlers: HashMap<String, Arc<dyn JobHandler>>,
    ) -> Self {
        let (queue_tx, queue_rx) = mpsc::unbounded_channel();

        Self {
            shared: Arc::new(Shared {
                pool,
                clock,
                identity,
                semaphore: Arc::new(Semaphore::new(config.max_concurrent_jobs.max(1))),
                config,
                handlers,
                cancel_tokens: Mutex::new(HashMap::new()),
            }),
            queue_tx,
            queue_rx: Arc::new(Mutex::new(queue_rx)),
        }
    }

    fn repo(&self) -> JobRepository<'_> {
        JobRepository::new(&self.shared.pool, self.shared.clock.as_ref())
    }

    fn audit_repo(&self) -> AuditLogRepository<'_> {
        AuditLogRepository::new(&self.shared.pool, self.shared.clock.as_ref())
    }

    /// **Disparo (TTR-001)**: persists `request` (durable, SQLite —
    /// persist-before-ack) and enqueues its id for a worker to pick up.
    /// Returns the new job's UUID.
    pub async fn submit(&self, request: NewJob) -> Result<String, JobExecutorError> {
        // Stamp the executor's identity metadata when the caller did not
        // supply its own (ADR-0020 V2 "Gobernanza y Estándares").
        let request = NewJob {
            session_id: request.session_id.or_else(|| self.shared.identity.session_id.clone()),
            node_id: request.node_id.or_else(|| self.shared.identity.node_id.clone()),
            logic_hash: request.logic_hash.or_else(|| self.shared.identity.logic_hash.clone()),
            ..request
        };

        let job = self.repo().submit(request).await?;
        self.enqueue(&job.id);

        Ok(job.id)
    }

    /// Pushes `job_id` onto the in-memory queue. Internal helper shared by
    /// [`Self::submit`] and [`Self::recover_at_startup`].
    fn enqueue(&self, job_id: &str) {
        // An unbounded channel send only fails if the receiver was dropped,
        // which only happens if this `JobExecutor` (and all its clones) are
        // gone — at that point there is nothing useful to do with the
        // error, and the job remains durably QUEUED in SQLite for the next
        // startup's recovery pass.
        let _ = self.queue_tx.send(job_id.to_string());
    }

    /// **Recuperación en startup (TTR-004)**.
    ///
    /// MUST be called once, after [`Self::new`] and before
    /// [`Self::spawn_workers`].
    ///
    /// Scans `jobs` for rows in `QUEUED` or `RUNNING`:
    /// - `QUEUED` rows are re-enqueued as-is.
    /// - `RUNNING` rows are transitioned to `QUEUED` (ADR-0011
    ///   "Auto-Recovery": completion is unknown after a crash) and then
    ///   re-enqueued.
    ///
    /// For every recovered job (both `QUEUED` and former-`RUNNING`), appends
    /// a `JOB_RECOVERED_AT_STARTUP` audit event via
    /// [`AuditLogRepository::append`] with `job_uuid` and `previous_state`
    /// in `details_json`.
    ///
    /// Returns the list of [`RecoveredJob`]s (job after recovery +
    /// `previous_state`), ordered by `created_at` — useful for tests and
    /// startup logging. An empty result means there was nothing to recover.
    pub async fn recover_at_startup(&self) -> Result<Vec<RecoveredJob>, JobExecutorError> {
        let repo = self.repo();
        let audit = self.audit_repo();

        let mut queued = repo.jobs_in_state(JobState::Queued).await?;
        let running = repo.jobs_in_state(JobState::Running).await?;

        let mut recovered = Vec::with_capacity(queued.len() + running.len());

        // RUNNING -> QUEUED first, so the merged list below reflects the
        // post-recovery state for every job.
        for job in running {
            let previous_state = job.state;
            let requeued = repo.transition(&job, JobState::Queued, None).await?;

            self.append_recovery_audit_event(&audit, &requeued.id, previous_state).await?;

            recovered.push(RecoveredJob {
                job: requeued,
                previous_state,
            });
        }

        for job in queued.drain(..) {
            let previous_state = job.state; // == JobState::Queued
            self.append_recovery_audit_event(&audit, &job.id, previous_state).await?;

            recovered.push(RecoveredJob { job, previous_state });
        }

        // Stable order across both groups (chronological by created_at),
        // matching jobs_in_state's own ordering.
        recovered.sort_by_key(|r| r.job.created_at_ns);

        for recovered_job in &recovered {
            self.enqueue(&recovered_job.job.id);
        }

        Ok(recovered)
    }

    async fn append_recovery_audit_event(
        &self,
        audit: &AuditLogRepository<'_>,
        job_uuid: &str,
        previous_state: JobState,
    ) -> Result<(), JobExecutorError> {
        let details_json = json!({
            "job_uuid": job_uuid,
            "previous_state": previous_state.as_str(),
        })
        .to_string();

        audit
            .append(AuditEventContent {
                action_type: JOB_RECOVERED_AT_STARTUP.to_string(),
                entity_type: "JOB".to_string(),
                entity_id: job_uuid.to_string(),
                details_json,
                owner_id: None,
                institutional_tag: self.shared.identity.institutional_tag.clone(),
                manifest_id: None,
                access_token_id: None,
                process_id: self.shared.identity.process_id.clone(),
                session_id: self.shared.identity.session_id.clone(),
                node_id: self.shared.identity.node_id.clone(),
            })
            .await?;

        Ok(())
    }

    /// **Monitoreo (poll, TTR-002/005)**: returns the current [`Job`] row,
    /// or `None` if `job_id` does not exist.
    pub async fn status(&self, job_id: &str) -> Result<Option<Job>, JobExecutorError> {
        Ok(self.repo().find(job_id).await?)
    }

    /// **Recuperación (fetch, TTR-003)**: returns the job's
    /// [`crate::persistence::job::JobResult`], or `None` if the job has not
    /// reached a terminal state with a recorded result yet.
    pub async fn result(&self, job_id: &str) -> Result<Option<crate::persistence::job::JobResult>, JobExecutorError> {
        Ok(self.repo().result_for_job(job_id).await?)
    }

    /// **Cancelación (TTR-006)**.
    ///
    /// - If `job_id` is `QUEUED`, transitions it directly to `CANCELLED`.
    ///   The dispatcher skips it when it reaches the front of the queue
    ///   (its persisted state is checked before execution starts).
    /// - If `job_id` is `RUNNING`, flips its [`CancellationToken`]; the
    ///   running [`JobHandler`] is expected to observe it and return. The
    ///   worker then transitions the job to `CANCELLED`.
    /// - Any other state (already terminal, or unknown id) returns
    ///   [`JobExecutorError::JobNotCancellable`].
    pub async fn cancel(&self, job_id: &str) -> Result<(), JobExecutorError> {
        let repo = self.repo();
        let job = repo
            .find(job_id)
            .await?
            .ok_or_else(|| JobExecutorError::JobNotCancellable(job_id.to_string()))?;

        match job.state {
            JobState::Queued => {
                repo.transition(&job, JobState::Cancelled, None).await?;
                Ok(())
            }
            JobState::Running => {
                let tokens = self.shared.cancel_tokens.lock().await;
                match tokens.get(job_id) {
                    Some(token) => {
                        token.cancel();
                        Ok(())
                    }
                    // RUNNING in the database but no token registered means
                    // no live worker owns it in this process (e.g. it was
                    // RUNNING before a crash and hasn't been picked up by
                    // the dispatcher yet after recovery). Recovery already
                    // demoted it to QUEUED before re-enqueueing, so by the
                    // time a worker would run it, `find` above would have
                    // returned QUEUED -- this branch is defensive.
                    None => Err(JobExecutorError::JobNotCancellable(job_id.to_string())),
                }
            }
            JobState::Completed | JobState::Failed | JobState::Cancelled => {
                Err(JobExecutorError::JobNotCancellable(job_id.to_string()))
            }
        }
    }

    /// **Worker pool (TTR-002)**: spawns a dispatcher task that pulls job
    /// ids from the in-memory queue and, for each, acquires one of
    /// `max_concurrent_jobs` semaphore permits before spawning the job's
    /// execution task. Returns immediately; the dispatcher runs until every
    /// clone of this [`JobExecutor`] (and thus the queue sender) is dropped.
    ///
    /// Must be called on a Tokio runtime (`#[tokio::main]` /
    /// `#[tokio::test]`).
    pub fn spawn_workers(&self) -> tokio::task::JoinHandle<()> {
        let executor = self.clone();

        tokio::spawn(async move {
            loop {
                let job_id = {
                    let mut rx = executor.queue_rx.lock().await;
                    match rx.recv().await {
                        Some(id) => id,
                        None => break, // all senders dropped -> shut down
                    }
                };

                let permit = match executor.shared.semaphore.clone().acquire_owned().await {
                    Ok(permit) => permit,
                    Err(_) => break, // semaphore closed -> shutting down
                };

                let worker = executor.clone();
                tokio::spawn(async move {
                    worker.run_job(job_id).await;
                    drop(permit);
                });
            }
        })
    }

    /// Executes a single job end to end: QUEUED -> RUNNING -> terminal,
    /// invoking the registered [`JobHandler`] (if any) and recording the
    /// result. Skips jobs that were cancelled while queued (TTR-006).
    async fn run_job(&self, job_id: String) {
        let repo = self.repo();

        let job = match repo.find(&job_id).await {
            Ok(Some(job)) => job,
            Ok(None) | Err(_) => return, // job vanished or DB error: nothing to run
        };

        // Skip jobs cancelled while still QUEUED (TTR-006).
        if job.state != JobState::Queued {
            return;
        }

        let running = match repo.transition(&job, JobState::Running, Some(&self.shared.identity.process_id)).await {
            Ok(running) => running,
            Err(_) => return,
        };

        let cancel_token = CancellationToken::new();
        {
            let mut tokens = self.shared.cancel_tokens.lock().await;
            tokens.insert(job_id.clone(), cancel_token.clone());
        }

        let handler = self.shared.handlers.get(&running.job_type).cloned();

        let outcome = match handler {
            Some(handler) => {
                let mut progress = ProgressReporter {
                    repo: &repo,
                    job: running.clone(),
                };
                handler.execute(&running, &mut progress, &cancel_token).await
            }
            None => JobOutcome::Failure {
                error_message: format!("no handler registered for job_type '{}'", running.job_type),
            },
        };

        {
            let mut tokens = self.shared.cancel_tokens.lock().await;
            tokens.remove(&job_id);
        }

        // Re-read current state: the handler may have observed cancellation
        // (TTR-006), or another path may have already moved the job.
        let current = match repo.find(&job_id).await {
            Ok(Some(job)) => job,
            _ => return,
        };

        if cancel_token.is_cancelled() && current.state == JobState::Running {
            let _ = repo.transition(&current, JobState::Cancelled, None).await;
            return;
        }

        if current.state != JobState::Running {
            // Already moved out of RUNNING by another path (e.g. cancel
            // raced with completion) -- do not double-record a result.
            return;
        }

        match outcome {
            JobOutcome::Success { result_data } => {
                if let Ok(completed) = repo.transition(&current, JobState::Completed, None).await {
                    let _ = repo
                        .record_result(NewJobResult {
                            job_uuid: completed.id,
                            result_data: Some(result_data),
                            error_message: None,
                        })
                        .await;
                }
            }
            JobOutcome::Failure { error_message } => {
                if let Ok(failed) = repo.transition(&current, JobState::Failed, None).await {
                    let _ = repo
                        .record_result(NewJobResult {
                            job_uuid: failed.id,
                            result_data: None,
                            error_message: Some(error_message),
                        })
                        .await;
                }
            }
        }
    }

    /// Configuration this executor was constructed with.
    pub fn config(&self) -> &JobExecutorConfig {
        &self.shared.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::persistence::pool::{connect, migrate};

    /// ADR-0020 V2 "Gobernanza y Estándares": minimal identity for tests —
    /// no business meaning, just stable strings to stamp on jobs/audit
    /// events.
    fn test_identity() -> ExecutorIdentity {
        ExecutorIdentity {
            process_id: "test-process".to_string(),
            session_id: Some("test-session".to_string()),
            node_id: Some("test-node".to_string()),
            logic_hash: Some("executor-v1".to_string()),
            institutional_tag: "DRASUS_TEST".to_string(),
        }
    }

    fn sample_new_job(job_type: &str) -> NewJob {
        NewJob {
            user_id: "user-1".to_string(),
            job_type: job_type.to_string(),
            parameters: "{\"strategy_id\":123}".to_string(),
            owner_id: Some("owner-1".to_string()),
            access_token_id: None,
            session_id: None,
            node_id: None,
            logic_hash: None,
        }
    }

    /// **EPIC-0 closing gate** (`docs/execution/STORY-005-async-job-executor.md`
    /// §4, TTR-ASYNC-EXECUTOR-004): jobs survive a simulated `kill -9` and
    /// are recovered on restart.
    ///
    /// Uses a SQLite database in a **temporary file** (not `sqlite::memory:`)
    /// because an in-memory database does not survive closing and reopening
    /// the pool — it cannot demonstrate durability or crash recovery.
    ///
    /// Steps:
    /// 1. Open a file-backed pool, migrate, submit three jobs and drive one
    ///    to `RUNNING` without ever completing it (simulates the moment a
    ///    crash interrupts an in-flight job).
    /// 2. Close that pool entirely — the on-disk state is now the only
    ///    source of truth, exactly as after `kill -9`.
    /// 3. Open a brand-new pool over the SAME database file, build a new
    ///    [`JobExecutor`] on top of it, and call
    ///    [`JobExecutor::recover_at_startup`].
    /// 4. Assert the gate: (a) the QUEUED job is recovered and re-enqueued;
    ///    (b) the RUNNING job is now QUEUED (not lost, not stuck RUNNING);
    ///    (c) no job is lost (total count unchanged); (d) a
    ///    `JOB_RECOVERED_AT_STARTUP` audit event with `previous_state` was
    ///    recorded for each recovered job.
    #[tokio::test]
    async fn jobs_survive_simulated_crash_and_are_recovered_on_restart() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("job_executor_crash.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());

        let (queued_job_id, running_job_id, completed_job_id) = {
            // --- Pre-crash process -------------------------------------------------
            let pool = connect(&database_url).await.expect("connect (pre-crash)");
            migrate(&pool).await.expect("migrate (pre-crash)");

            let clock: Arc<dyn Clock> = Arc::new(DeterministicClock::new(1_000, 100));
            let executor = JobExecutor::new(
                pool.clone(),
                clock,
                test_identity(),
                JobExecutorConfig::default(),
                HashMap::new(),
            );

            // No prior run to recover from -- the gate test focuses on the
            // restart pass below, but calling this here exercises the
            // documented "no-op on empty jobs table" path too.
            let pre_recovery = executor.recover_at_startup().await.expect("pre-crash recover (no-op)");
            assert!(pre_recovery.is_empty(), "fresh database has nothing to recover");

            // Job A stays QUEUED (never picked up before the "crash").
            let queued_job_id = executor.submit(sample_new_job("BACKTEST")).await.expect("submit queued job");

            // Job B is driven to RUNNING and left there -- this is the job
            // an in-flight worker would have been executing at the moment
            // of the crash.
            let running_job_id = executor.submit(sample_new_job("BACKTEST")).await.expect("submit running job");
            {
                let repo = executor.repo();
                let job = repo.find(&running_job_id).await.expect("find running job").expect("job exists");
                repo.transition(&job, JobState::Running, Some("worker-pre-crash"))
                    .await
                    .expect("transition to running");
            }

            // Job C completes normally before the crash -- recovery must
            // leave it untouched (it is not QUEUED or RUNNING).
            let completed_job_id = executor.submit(sample_new_job("BACKTEST")).await.expect("submit completed job");
            {
                let repo = executor.repo();
                let job = repo
                    .find(&completed_job_id)
                    .await
                    .expect("find completed job")
                    .expect("job exists");
                let running = repo
                    .transition(&job, JobState::Running, Some("worker-pre-crash"))
                    .await
                    .expect("transition completed job to running");
                repo.transition(&running, JobState::Completed, None)
                    .await
                    .expect("transition completed job to completed");
            }

            // Simulate `kill -9`: drop the executor and close the pool
            // without ever finishing job B. The on-disk state (QUEUED,
            // RUNNING, COMPLETED) is the only surviving truth.
            drop(executor);
            pool.close().await;

            (queued_job_id, running_job_id, completed_job_id)
        };

        // --- Restart: brand-new pool over the SAME file -----------------------
        let pool = connect(&database_url).await.expect("connect (restart)");
        migrate(&pool).await.expect("migrate (restart) must be idempotent");

        let clock: Arc<dyn Clock> = Arc::new(DeterministicClock::new(2_000, 100));
        let executor = JobExecutor::new(pool.clone(), clock, test_identity(), JobExecutorConfig::default(), HashMap::new());

        let recovered = executor.recover_at_startup().await.expect("recover at startup after crash");

        // (c) No job is lost: exactly the two non-terminal jobs (A and B)
        // are recovered. The completed job C is untouched.
        assert_eq!(recovered.len(), 2, "exactly the QUEUED and RUNNING jobs must be recovered");

        let recovered_ids: HashMap<&str, &RecoveredJob> = recovered.iter().map(|r| (r.job.id.as_str(), r)).collect();

        // (a) The QUEUED job is recovered with its state unchanged.
        let recovered_queued = recovered_ids
            .get(queued_job_id.as_str())
            .expect("queued job is in the recovered list");
        assert_eq!(recovered_queued.previous_state, JobState::Queued);
        assert_eq!(recovered_queued.job.state, JobState::Queued);

        // (b) The RUNNING job is demoted to QUEUED -- not lost, not stuck
        // RUNNING.
        let recovered_running = recovered_ids
            .get(running_job_id.as_str())
            .expect("running job is in the recovered list");
        assert_eq!(recovered_running.previous_state, JobState::Running);
        assert_eq!(recovered_running.job.state, JobState::Queued);

        // (c) Total job count is unchanged -- nothing was lost or
        // duplicated -- and the completed job is untouched.
        let repo = executor.repo();
        let all_queued = repo.jobs_in_state(JobState::Queued).await.expect("query queued after recovery");
        let all_running = repo.jobs_in_state(JobState::Running).await.expect("query running after recovery");
        let all_completed = repo.jobs_in_state(JobState::Completed).await.expect("query completed after recovery");

        assert_eq!(all_running.len(), 0, "no job must remain stuck in RUNNING after recovery");
        assert_eq!(all_queued.len(), 2, "both recovered jobs must now be QUEUED");
        assert_eq!(all_completed.len(), 1, "the already-completed job must be untouched by recovery");
        assert_eq!(all_completed[0].id, completed_job_id);

        let queued_ids: std::collections::HashSet<&str> = all_queued.iter().map(|j| j.id.as_str()).collect();
        assert!(queued_ids.contains(queued_job_id.as_str()));
        assert!(queued_ids.contains(running_job_id.as_str()));

        // (d) A JOB_RECOVERED_AT_STARTUP audit event with `previous_state`
        // was recorded for each recovered job (reusing the existing audit
        // log -- TTR-004: "Registrar en audit ... NO inventes otra forma de
        // auditar").
        let audit = executor.audit_repo();

        let queued_audit_events = audit
            .events_for_entity("JOB", &queued_job_id)
            .await
            .expect("load audit events for queued job");
        assert_eq!(queued_audit_events.len(), 1);
        assert_eq!(queued_audit_events[0].content.action_type, JOB_RECOVERED_AT_STARTUP);
        let queued_details: serde_json::Value =
            serde_json::from_str(&queued_audit_events[0].content.details_json).expect("parse details_json");
        assert_eq!(queued_details["job_uuid"], queued_job_id);
        assert_eq!(queued_details["previous_state"], "QUEUED");

        let running_audit_events = audit
            .events_for_entity("JOB", &running_job_id)
            .await
            .expect("load audit events for running job");
        assert_eq!(running_audit_events.len(), 1);
        assert_eq!(running_audit_events[0].content.action_type, JOB_RECOVERED_AT_STARTUP);
        let running_details: serde_json::Value =
            serde_json::from_str(&running_audit_events[0].content.details_json).expect("parse details_json");
        assert_eq!(running_details["job_uuid"], running_job_id);
        assert_eq!(running_details["previous_state"], "RUNNING");

        // The completed job must NOT have a recovery audit event -- it was
        // never QUEUED or RUNNING at restart time.
        let completed_audit_events = audit
            .events_for_entity("JOB", &completed_job_id)
            .await
            .expect("load audit events for completed job");
        assert!(
            completed_audit_events.is_empty(),
            "a job that was already COMPLETED before the crash must not get a recovery event"
        );

        pool.close().await;
    }

    /// Calling [`JobExecutor::recover_at_startup`] on a freshly migrated,
    /// empty database is a documented no-op: nothing to recover, no audit
    /// events written. Complements the crash-recovery gate test above by
    /// covering the "nothing to do" branch explicitly.
    #[tokio::test]
    async fn recover_at_startup_on_empty_database_is_a_noop() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("job_executor_empty.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());

        let pool = connect(&database_url).await.expect("connect");
        migrate(&pool).await.expect("migrate");

        let clock: Arc<dyn Clock> = Arc::new(DeterministicClock::new(1_000, 100));
        let executor = JobExecutor::new(pool.clone(), clock, test_identity(), JobExecutorConfig::default(), HashMap::new());

        let recovered = executor.recover_at_startup().await.expect("recover on empty db");
        assert!(recovered.is_empty());

        let audit = executor.audit_repo();
        let chain = audit.load_chain().await.expect("load chain");
        assert!(chain.is_empty(), "no audit events on an empty database");

        pool.close().await;
    }
}
