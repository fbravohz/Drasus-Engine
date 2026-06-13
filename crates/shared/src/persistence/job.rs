//! [SHELL] Persistence repository for the Async Job Executor
//! (`docs/features/async-job-executor.md` TTR-ASYNC-EXECUTOR-001/003/004/006,
//! ADR-0011, ADR-0020 V2).
//!
//! Wraps the `jobs` and `job_results` tables (migration `0003_jobs.sql`).
//! Owns the only I/O for jobs: SQLite reads/writes, UUID generation
//! (unseeded randomness, ADR-0002/0004) and the [`Clock`] port read. The
//! state machine itself ([`JobState`], [`validate_transition`]) is pure
//! core logic in [`crate::domain::job`] — this module only feeds it
//! injected inputs and persists/loads the result, mirroring
//! [`crate::persistence::audit_log::AuditLogRepository`].
//!
//! ## Persist-before-ack (TTR-001)
//!
//! [`JobRepository::submit`] performs the `INSERT INTO jobs` and returns
//! only after it commits. The caller (orchestrator) receives the job's UUID
//! from this call's `Ok` value — there is no path that hands back a UUID
//! before the row exists on disk. A `kill -9` between "submit returned" and
//! "row visible on disk" is therefore impossible: they are the same event.
//!
//! ## Append-only `job_results` (TTR-003)
//!
//! Enforced twice, exactly like `audit_events`:
//! - **Database**: migration `0003_jobs.sql` installs `BEFORE UPDATE` /
//!   `BEFORE DELETE` triggers on `job_results` that `RAISE(ABORT, ...)`.
//! - **Application**: this repository exposes [`JobRepository::record_result`]
//!   and [`JobRepository::result_for_job`] only — no update/delete method
//!   exists.

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::clock::Clock;
use crate::domain::job::{validate_transition, JobState};

/// Errors returned by [`JobRepository`] operations.
#[derive(Debug)]
pub enum JobRepositoryError {
    /// The underlying SQLite operation failed.
    Database(sqlx::Error),
    /// A row in `jobs` had a `state` value outside the five canonical
    /// strings (`JobState::from_str_value` returned `None`) — a
    /// data-integrity error, not a transition error.
    UnknownState(String),
    /// A requested state transition is not allowed
    /// ([`crate::domain::job::validate_transition`]).
    InvalidTransition(crate::domain::job::InvalidTransition),
}

impl std::fmt::Display for JobRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobRepositoryError::Database(err) => write!(f, "job repository database error: {err}"),
            JobRepositoryError::UnknownState(value) => {
                write!(f, "job repository: unknown state value '{value}' in jobs table")
            }
            JobRepositoryError::InvalidTransition(err) => write!(f, "job repository: {err}"),
        }
    }
}

impl std::error::Error for JobRepositoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            JobRepositoryError::Database(err) => Some(err),
            JobRepositoryError::UnknownState(_) => None,
            JobRepositoryError::InvalidTransition(err) => Some(err),
        }
    }
}

impl From<sqlx::Error> for JobRepositoryError {
    fn from(err: sqlx::Error) -> Self {
        JobRepositoryError::Database(err)
    }
}

impl From<crate::domain::job::InvalidTransition> for JobRepositoryError {
    fn from(err: crate::domain::job::InvalidTransition) -> Self {
        JobRepositoryError::InvalidTransition(err)
    }
}

/// A new job to persist (TTR-001 "Entrada": `JobRequest(job_type,
/// parameters, user_id)`), plus the ADR-0020 V2 metadata supplied by the
/// orchestrator at submit time.
#[derive(Debug, Clone)]
pub struct NewJob {
    pub user_id: String,
    pub job_type: String,
    /// JSON-encoded job parameters (opaque to this repository).
    pub parameters: String,
    pub owner_id: Option<String>,
    pub access_token_id: Option<String>,
    pub session_id: Option<String>,
    pub node_id: Option<String>,
    pub logic_hash: Option<String>,
}

/// A persisted job row (`jobs` table).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub event_sequence_id: i64,

    pub process_id: Option<String>,
    pub session_id: Option<String>,
    pub node_id: Option<String>,
    pub logic_hash: Option<String>,

    pub owner_id: Option<String>,
    pub access_token_id: Option<String>,

    pub user_id: String,
    pub job_type: String,
    pub parameters: String,
    pub state: JobState,
    pub progress: u8,
}

/// A new job result to persist (TTR-003 "Entrada": `Result object(job_uuid,
/// result_data, error_message, completed_at)`).
#[derive(Debug, Clone)]
pub struct NewJobResult {
    pub job_uuid: String,
    /// JSON-encoded result payload, `None` on failure.
    pub result_data: Option<String>,
    /// Error description, `None` on success.
    pub error_message: Option<String>,
}

/// A persisted job result row (`job_results` table). Immutable once
/// inserted (TTR-003).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobResult {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub event_sequence_id: i64,

    pub job_uuid: String,
    pub result_data: Option<String>,
    pub error_message: Option<String>,
    pub completed_at_ns: i64,
}

/// A job recovered at startup (TTR-004): its previous (pre-recovery) state
/// and its identity, ready to be re-queued and audited.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveredJob {
    pub job: Job,
    pub previous_state: JobState,
}

/// Repository for `jobs` and `job_results`.
///
/// Construct with a migrated [`SqlitePool`] (see
/// [`crate::persistence::pool::connect`] +
/// [`crate::persistence::pool::migrate`]) and any [`Clock`] implementation
/// (production: [`crate::orchestrator::SystemClock`]; tests/backtests:
/// [`crate::domain::clock::DeterministicClock`]).
pub struct JobRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> JobRepository<'a> {
    /// Creates a repository bound to `pool` and `clock`. Both are borrowed
    /// for the lifetime of the repository — no ownership is taken, so the
    /// same pool/clock can be shared with other repositories.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Persists a new job in state `QUEUED` and returns it (TTR-001:
    /// "Job se guarda ANTES de retornar UUID").
    ///
    /// Generates a fresh UUID v4 (`id`, unseeded randomness — confined to
    /// this shell per ADR-0002/0004) and reads the current [`Clock`]
    /// (`created_at_ns` == `updated_at_ns` for a freshly created row).
    /// `event_sequence_id` starts at `1` and `audit_chain_hash` is `None`
    /// for a new job (this job's own update chain has no predecessor yet).
    ///
    /// This call's `INSERT` is the durability boundary: the caller receives
    /// the job's UUID (via the returned [`Job::id`]) only after this
    /// `INSERT` has completed, never before (persist-before-ack).
    pub async fn submit(&self, request: NewJob) -> Result<Job, JobRepositoryError> {
        let id = Uuid::new_v4().to_string();
        let now_ns = self.clock.timestamp_ns();
        let state = JobState::Queued;
        let progress: u8 = 0;
        let event_sequence_id: i64 = 1;
        let audit_hash = compute_job_audit_hash(
            &id,
            now_ns,
            event_sequence_id,
            None,
            &request.user_id,
            &request.job_type,
            &request.parameters,
            state,
            progress,
        );

        sqlx::query(
            "INSERT INTO jobs (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                process_id, session_id, node_id, logic_hash, \
                owner_id, access_token_id, \
                user_id, job_type, parameters, state, progress\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(Option::<String>::None)
        .bind(event_sequence_id)
        .bind(Option::<String>::None) // process_id: unassigned until a worker picks it up
        .bind(&request.session_id)
        .bind(&request.node_id)
        .bind(&request.logic_hash)
        .bind(&request.owner_id)
        .bind(&request.access_token_id)
        .bind(&request.user_id)
        .bind(&request.job_type)
        .bind(&request.parameters)
        .bind(state.as_str())
        .bind(progress as i64)
        .execute(self.pool)
        .await?;

        Ok(Job {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: None,
            event_sequence_id,
            process_id: None,
            session_id: request.session_id,
            node_id: request.node_id,
            logic_hash: request.logic_hash,
            owner_id: request.owner_id,
            access_token_id: request.access_token_id,
            user_id: request.user_id,
            job_type: request.job_type,
            parameters: request.parameters,
            state,
            progress,
        })
    }

    /// Loads a single job by `id`, or `None` if it does not exist.
    pub async fn find(&self, id: &str) -> Result<Option<Job>, JobRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    process_id, session_id, node_id, logic_hash, \
                    owner_id, access_token_id, \
                    user_id, job_type, parameters, state, progress \
             FROM jobs WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(row_to_job(row)?)),
            None => Ok(None),
        }
    }

    /// Loads every job currently in `state` (TTR-004: startup recovery scans
    /// for `QUEUED` and `RUNNING`).
    pub async fn jobs_in_state(&self, state: JobState) -> Result<Vec<Job>, JobRepositoryError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    process_id, session_id, node_id, logic_hash, \
                    owner_id, access_token_id, \
                    user_id, job_type, parameters, state, progress \
             FROM jobs WHERE state = ? ORDER BY created_at ASC",
        )
        .bind(state.as_str())
        .fetch_all(self.pool)
        .await?;

        rows.into_iter().map(row_to_job).collect()
    }

    /// Transitions `job` to `to`, validating the transition via
    /// [`validate_transition`] before writing anything.
    ///
    /// Bumps `updated_at` (current [`Clock`] reading), `event_sequence_id`
    /// (+1) and sets `audit_chain_hash` to the job's previous `audit_hash`
    /// — the same hash-chain shape as `audit_events`, scoped to this job's
    /// own row history. When transitioning into `RUNNING`, `process_id` is
    /// set to `worker_id` (TTR-001/002 "Worker ID"); for any other
    /// transition `process_id` is left unchanged.
    ///
    /// Returns the updated [`Job`] on success.
    pub async fn transition(
        &self,
        job: &Job,
        to: JobState,
        worker_id: Option<&str>,
    ) -> Result<Job, JobRepositoryError> {
        validate_transition(job.state, to)?;

        let now_ns = self.clock.timestamp_ns();
        let event_sequence_id = job.event_sequence_id + 1;
        let progress = match to {
            JobState::Running => 0,
            JobState::Completed => 100,
            _ => job.progress,
        };
        let process_id = match to {
            JobState::Running => worker_id.map(str::to_string).or_else(|| job.process_id.clone()),
            _ => job.process_id.clone(),
        };

        let audit_hash = compute_job_audit_hash(
            &job.id,
            now_ns,
            event_sequence_id,
            Some(&job.audit_hash),
            &job.user_id,
            &job.job_type,
            &job.parameters,
            to,
            progress,
        );

        sqlx::query(
            "UPDATE jobs SET \
                updated_at = ?, audit_hash = ?, audit_chain_hash = ?, event_sequence_id = ?, \
                process_id = ?, state = ?, progress = ? \
             WHERE id = ?",
        )
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&job.audit_hash)
        .bind(event_sequence_id)
        .bind(&process_id)
        .bind(to.as_str())
        .bind(progress as i64)
        .bind(&job.id)
        .execute(self.pool)
        .await?;

        Ok(Job {
            id: job.id.clone(),
            created_at_ns: job.created_at_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: Some(job.audit_hash.clone()),
            event_sequence_id,
            process_id,
            session_id: job.session_id.clone(),
            node_id: job.node_id.clone(),
            logic_hash: job.logic_hash.clone(),
            owner_id: job.owner_id.clone(),
            access_token_id: job.access_token_id.clone(),
            user_id: job.user_id.clone(),
            job_type: job.job_type.clone(),
            parameters: job.parameters.clone(),
            state: to,
            progress,
        })
    }

    /// Updates `progress` (0-100, clamped by [`crate::domain::job::Progress`])
    /// for a job in `RUNNING` state, without changing its `state`
    /// (TTR-005: "Worker actualiza progreso cada `progress_interval`
    /// segundos").
    ///
    /// Bumps `updated_at`, `event_sequence_id` and `audit_chain_hash` like
    /// [`Self::transition`]. Returns the updated [`Job`].
    pub async fn update_progress(&self, job: &Job, progress: crate::domain::job::Progress) -> Result<Job, JobRepositoryError> {
        let now_ns = self.clock.timestamp_ns();
        let event_sequence_id = job.event_sequence_id + 1;
        let progress_value = progress.value();

        let audit_hash = compute_job_audit_hash(
            &job.id,
            now_ns,
            event_sequence_id,
            Some(&job.audit_hash),
            &job.user_id,
            &job.job_type,
            &job.parameters,
            job.state,
            progress_value,
        );

        sqlx::query(
            "UPDATE jobs SET \
                updated_at = ?, audit_hash = ?, audit_chain_hash = ?, event_sequence_id = ?, progress = ? \
             WHERE id = ?",
        )
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&job.audit_hash)
        .bind(event_sequence_id)
        .bind(progress_value as i64)
        .bind(&job.id)
        .execute(self.pool)
        .await?;

        Ok(Job {
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: Some(job.audit_hash.clone()),
            event_sequence_id,
            progress: progress_value,
            ..job.clone()
        })
    }

    /// Appends a [`JobResult`] for a job that just reached a terminal state
    /// (TTR-003). Append-only: this is the only write path for
    /// `job_results`, and the database additionally rejects `UPDATE`/
    /// `DELETE` via triggers (migration `0003_jobs.sql`).
    ///
    /// `event_sequence_id` is the next value in the global `job_results`
    /// chain (read-then-write, mirroring
    /// [`crate::persistence::audit_log::AuditLogRepository::append`]).
    pub async fn record_result(&self, new_result: NewJobResult) -> Result<JobResult, JobRepositoryError> {
        let previous_hash = self.load_latest_result_hash().await?;

        let id = Uuid::new_v4().to_string();
        let now_ns = self.clock.timestamp_ns();
        let event_sequence_id = self.next_result_sequence_id().await?;

        let audit_hash = compute_result_audit_hash(
            &id,
            now_ns,
            event_sequence_id,
            previous_hash.as_deref(),
            &new_result.job_uuid,
            new_result.result_data.as_deref(),
            new_result.error_message.as_deref(),
        );

        sqlx::query(
            "INSERT INTO job_results (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                job_uuid, result_data, error_message, completed_at\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&previous_hash)
        .bind(event_sequence_id)
        .bind(&new_result.job_uuid)
        .bind(&new_result.result_data)
        .bind(&new_result.error_message)
        .bind(now_ns)
        .execute(self.pool)
        .await?;

        Ok(JobResult {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: previous_hash,
            event_sequence_id,
            job_uuid: new_result.job_uuid,
            result_data: new_result.result_data,
            error_message: new_result.error_message,
            completed_at_ns: now_ns,
        })
    }

    /// Loads the (at most one) result recorded for `job_uuid`, or `None` if
    /// the job has not completed/failed yet.
    pub async fn result_for_job(&self, job_uuid: &str) -> Result<Option<JobResult>, JobRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    job_uuid, result_data, error_message, completed_at \
             FROM job_results WHERE job_uuid = ? ORDER BY event_sequence_id ASC LIMIT 1",
        )
        .bind(job_uuid)
        .fetch_optional(self.pool)
        .await?;

        Ok(row.map(row_to_result))
    }

    /// Reads the `audit_hash` of the most recently inserted `job_results`
    /// row (highest `event_sequence_id`), or `None` if the table is empty
    /// (next [`record_result`](Self::record_result) call creates the
    /// genesis row).
    async fn load_latest_result_hash(&self) -> Result<Option<String>, JobRepositoryError> {
        let row = sqlx::query("SELECT audit_hash FROM job_results ORDER BY event_sequence_id DESC LIMIT 1")
            .fetch_optional(self.pool)
            .await?;

        Ok(row.map(|row| row.get::<String, _>(0)))
    }

    /// Computes the next `event_sequence_id` for `job_results` (1 for the
    /// first row, then monotonically increasing).
    async fn next_result_sequence_id(&self) -> Result<i64, JobRepositoryError> {
        let row = sqlx::query("SELECT COALESCE(MAX(event_sequence_id), 0) FROM job_results")
            .fetch_one(self.pool)
            .await?;

        let max: i64 = row.get(0);
        Ok(max + 1)
    }
}

/// Converts a `jobs` row into the [`Job`] type.
fn row_to_job(row: sqlx::sqlite::SqliteRow) -> Result<Job, JobRepositoryError> {
    let state_value: String = row.get("state");
    let state = JobState::from_str_value(&state_value)
        .ok_or_else(|| JobRepositoryError::UnknownState(state_value.clone()))?;
    let progress: i64 = row.get("progress");

    Ok(Job {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        event_sequence_id: row.get("event_sequence_id"),
        process_id: row.get("process_id"),
        session_id: row.get("session_id"),
        node_id: row.get("node_id"),
        logic_hash: row.get("logic_hash"),
        owner_id: row.get("owner_id"),
        access_token_id: row.get("access_token_id"),
        user_id: row.get("user_id"),
        job_type: row.get("job_type"),
        parameters: row.get("parameters"),
        state,
        progress: progress as u8,
    })
}

/// Converts a `job_results` row into the [`JobResult`] type.
fn row_to_result(row: sqlx::sqlite::SqliteRow) -> JobResult {
    JobResult {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        event_sequence_id: row.get("event_sequence_id"),
        job_uuid: row.get("job_uuid"),
        result_data: row.get("result_data"),
        error_message: row.get("error_message"),
        completed_at_ns: row.get("completed_at"),
    }
}

/// Computes a deterministic SHA-256 snapshot hash for a `jobs` row, chained
/// to the row's previous `audit_hash` (or `None` for a freshly submitted
/// job — same "GENESIS" convention as
/// [`crate::domain::audit_log::GENESIS_PREVIOUS_HASH`]).
#[allow(clippy::too_many_arguments)]
fn compute_job_audit_hash(
    id: &str,
    updated_at_ns: i64,
    event_sequence_id: i64,
    previous_audit_hash: Option<&str>,
    user_id: &str,
    job_type: &str,
    parameters: &str,
    state: JobState,
    progress: u8,
) -> String {
    use sha2::{Digest, Sha256};
    const SEP: char = '\u{1F}';

    let mut buffer = String::new();
    let mut push = |field: &str| {
        buffer.push_str(field);
        buffer.push(SEP);
    };

    push(id);
    push(&updated_at_ns.to_string());
    push(&event_sequence_id.to_string());
    push(previous_audit_hash.unwrap_or(crate::domain::audit_log::GENESIS_PREVIOUS_HASH));
    push(user_id);
    push(job_type);
    push(parameters);
    push(state.as_str());
    push(&progress.to_string());

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    let digest = hasher.finalize();

    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Computes a deterministic SHA-256 snapshot hash for a `job_results` row,
/// chained to the previous result row's `audit_hash` (or `None` for the
/// first result ever recorded).
fn compute_result_audit_hash(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    previous_audit_hash: Option<&str>,
    job_uuid: &str,
    result_data: Option<&str>,
    error_message: Option<&str>,
) -> String {
    use sha2::{Digest, Sha256};
    const SEP: char = '\u{1F}';

    let mut buffer = String::new();
    let mut push = |field: &str| {
        buffer.push_str(field);
        buffer.push(SEP);
    };

    push(id);
    push(&created_at_ns.to_string());
    push(&event_sequence_id.to_string());
    push(previous_audit_hash.unwrap_or(crate::domain::audit_log::GENESIS_PREVIOUS_HASH));
    push(job_uuid);
    push(result_data.unwrap_or(""));
    push(error_message.unwrap_or(""));

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    let digest = hasher.finalize();

    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("connect in-memory db");
        migrate(&pool).await.expect("apply migrations");
        pool
    }

    fn sample_new_job() -> NewJob {
        NewJob {
            user_id: "user-1".to_string(),
            job_type: "BACKTEST".to_string(),
            parameters: "{\"strategy_id\":123}".to_string(),
            owner_id: Some("owner-1".to_string()),
            access_token_id: None,
            session_id: Some("session-1".to_string()),
            node_id: Some("node-1".to_string()),
            logic_hash: Some("executor-v1".to_string()),
        }
    }

    /// TTR-001: submitting a job persists it in `QUEUED` state with
    /// progress 0, and the returned UUID corresponds to a row that already
    /// exists in `jobs` (persist-before-ack).
    #[tokio::test]
    async fn submit_persists_job_in_queued_state() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job = repo.submit(sample_new_job()).await.expect("submit job");

        assert_eq!(job.state, JobState::Queued);
        assert_eq!(job.progress, 0);
        assert_eq!(job.event_sequence_id, 1);
        assert_eq!(job.audit_chain_hash, None);

        let found = repo.find(&job.id).await.expect("find job").expect("job exists");
        assert_eq!(found, job);
    }

    /// TTR-002: a queued job transitions to RUNNING, gets process_id set to
    /// the worker id, and progress resets to 0.
    #[tokio::test]
    async fn transition_to_running_sets_worker_and_resets_progress() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job = repo.submit(sample_new_job()).await.expect("submit job");
        clock.tick();
        let running = repo
            .transition(&job, JobState::Running, Some("worker-7"))
            .await
            .expect("transition to running");

        assert_eq!(running.state, JobState::Running);
        assert_eq!(running.process_id, Some("worker-7".to_string()));
        assert_eq!(running.progress, 0);
        assert_eq!(running.event_sequence_id, 2);
        assert_eq!(running.audit_chain_hash, Some(job.audit_hash.clone()));
        assert_ne!(running.audit_hash, job.audit_hash);
    }

    /// TTR-002/003: RUNNING -> COMPLETED sets progress to 100.
    #[tokio::test]
    async fn transition_to_completed_sets_progress_100() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job = repo.submit(sample_new_job()).await.expect("submit job");
        clock.tick();
        let running = repo
            .transition(&job, JobState::Running, Some("worker-7"))
            .await
            .expect("transition to running");
        clock.tick();
        let completed = repo
            .transition(&running, JobState::Completed, None)
            .await
            .expect("transition to completed");

        assert_eq!(completed.state, JobState::Completed);
        assert_eq!(completed.progress, 100);
    }

    /// An invalid transition (e.g. QUEUED -> COMPLETED) is rejected before
    /// any write happens, and the stored row is untouched.
    #[tokio::test]
    async fn transition_rejects_invalid_state_change() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job = repo.submit(sample_new_job()).await.expect("submit job");
        let result = repo.transition(&job, JobState::Completed, None).await;

        assert!(matches!(result, Err(JobRepositoryError::InvalidTransition(_))));

        let found = repo.find(&job.id).await.expect("find job").expect("job exists");
        assert_eq!(found.state, JobState::Queued);
        assert_eq!(found.event_sequence_id, 1);
    }

    /// TTR-005: `update_progress` updates progress without changing state,
    /// and bumps the chain.
    #[tokio::test]
    async fn update_progress_changes_progress_without_changing_state() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job = repo.submit(sample_new_job()).await.expect("submit job");
        clock.tick();
        let running = repo
            .transition(&job, JobState::Running, Some("worker-7"))
            .await
            .expect("transition to running");
        clock.tick();
        let progressed = repo
            .update_progress(&running, crate::domain::job::Progress::new(45))
            .await
            .expect("update progress");

        assert_eq!(progressed.state, JobState::Running);
        assert_eq!(progressed.progress, 45);
        assert_eq!(progressed.event_sequence_id, 3);
    }

    /// TTR-004: `jobs_in_state` returns only jobs matching the requested
    /// state.
    #[tokio::test]
    async fn jobs_in_state_filters_correctly() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job_a = repo.submit(sample_new_job()).await.expect("submit job a");
        let job_b = repo.submit(sample_new_job()).await.expect("submit job b");
        clock.tick();
        repo.transition(&job_b, JobState::Running, Some("worker-1"))
            .await
            .expect("transition job b to running");

        let queued = repo.jobs_in_state(JobState::Queued).await.expect("query queued");
        assert_eq!(queued.len(), 1);
        assert_eq!(queued[0].id, job_a.id);

        let running = repo.jobs_in_state(JobState::Running).await.expect("query running");
        assert_eq!(running.len(), 1);
        assert_eq!(running[0].id, job_b.id);
    }

    /// TTR-003: recording a result for a completed job persists it and it
    /// is retrievable via `result_for_job`.
    #[tokio::test]
    async fn record_result_persists_and_is_retrievable() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job = repo.submit(sample_new_job()).await.expect("submit job");
        clock.tick();
        let running = repo
            .transition(&job, JobState::Running, Some("worker-1"))
            .await
            .expect("transition to running");
        clock.tick();
        let completed = repo
            .transition(&running, JobState::Completed, None)
            .await
            .expect("transition to completed");

        let result = repo
            .record_result(NewJobResult {
                job_uuid: completed.id.clone(),
                result_data: Some("{\"cagr\":0.25}".to_string()),
                error_message: None,
            })
            .await
            .expect("record result");

        assert_eq!(result.job_uuid, completed.id);
        assert_eq!(result.event_sequence_id, 1);
        assert_eq!(result.audit_chain_hash, None);

        let fetched = repo
            .result_for_job(&completed.id)
            .await
            .expect("query result")
            .expect("result exists");
        assert_eq!(fetched, result);
    }

    /// TTR-003 CLOSING CRITERION: `job_results` is append-only — UPDATE and
    /// DELETE are rejected by the database trigger (migration
    /// 0003_jobs.sql), mirroring `audit_events`.
    #[tokio::test]
    async fn job_results_update_and_delete_are_rejected_by_triggers() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job = repo.submit(sample_new_job()).await.expect("submit job");
        clock.tick();
        let running = repo
            .transition(&job, JobState::Running, Some("worker-1"))
            .await
            .expect("transition to running");
        clock.tick();
        let completed = repo
            .transition(&running, JobState::Completed, None)
            .await
            .expect("transition to completed");

        let result = repo
            .record_result(NewJobResult {
                job_uuid: completed.id.clone(),
                result_data: Some("{\"cagr\":0.25}".to_string()),
                error_message: None,
            })
            .await
            .expect("record result");

        let update_result = sqlx::query("UPDATE job_results SET result_data = ? WHERE id = ?")
            .bind("{\"tampered\":true}")
            .bind(&result.id)
            .execute(&pool)
            .await;
        assert!(update_result.is_err(), "UPDATE on job_results must be rejected");

        let delete_result = sqlx::query("DELETE FROM job_results WHERE id = ?")
            .bind(&result.id)
            .execute(&pool)
            .await;
        assert!(delete_result.is_err(), "DELETE on job_results must be rejected");
    }

    /// A second result chains to the first via `audit_chain_hash`.
    #[tokio::test]
    async fn record_result_chains_sequential_results() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job_a = repo.submit(sample_new_job()).await.expect("submit job a");
        let job_b = repo.submit(sample_new_job()).await.expect("submit job b");

        let result_a = repo
            .record_result(NewJobResult {
                job_uuid: job_a.id.clone(),
                result_data: Some("{\"ok\":true}".to_string()),
                error_message: None,
            })
            .await
            .expect("record result a");

        let result_b = repo
            .record_result(NewJobResult {
                job_uuid: job_b.id.clone(),
                result_data: None,
                error_message: Some("Invalid date range".to_string()),
            })
            .await
            .expect("record result b");

        assert_eq!(result_a.event_sequence_id, 1);
        assert_eq!(result_b.event_sequence_id, 2);
        assert_eq!(result_b.audit_chain_hash, Some(result_a.audit_hash));
    }
}
