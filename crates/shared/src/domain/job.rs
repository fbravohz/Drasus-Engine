//! [CORE] Pure state machine for async jobs (`docs/features/async-job-executor.md`
//! TTR-ASYNC-EXECUTOR-001..006, ADR-0004, ADR-0011).
//!
//! No I/O, no system clock, no unseeded randomness (ADR-0002/0004). `id`
//! (UUID) and timestamps are injected by the shell (persistence layer /
//! orchestrator) — the same pattern as [`super::audit_log::chain_event`] —
//! so that, given the same inputs, every function here always produces the
//! same output, bit for bit.
//!
//! ## States (ADR-0004 FSM)
//!
//! - [`JobState::Queued`]: waiting for a worker.
//! - [`JobState::Running`]: a worker has picked it up.
//! - [`JobState::Completed`]: finished successfully (terminal).
//! - [`JobState::Failed`]: finished with an error (terminal).
//! - [`JobState::Cancelled`]: cancelled by the user (terminal).
//!
//! ## Valid transitions (async-job-executor.md TTRs 002/004/006)
//!
//! | From | To | When |
//! |---|---|---|
//! | `QUEUED` | `RUNNING` | A worker picks up the job (TTR-002) |
//! | `RUNNING` | `COMPLETED` | The job finishes successfully (TTR-002/003) |
//! | `RUNNING` | `FAILED` | The job's callback returns/throws an error (TTR-002) |
//! | `QUEUED` | `CANCELLED` | User cancels a not-yet-started job (TTR-006) |
//! | `RUNNING` | `CANCELLED` | User cancels a running job; worker observes the cancel token (TTR-006) |
//! | `RUNNING` | `QUEUED` | Startup recovery: a job that was `RUNNING` when the process died is re-queued, because completion is unknown (TTR-004) |
//!
//! Every other `(from, to)` pair — including any transition out of a
//! terminal state — is rejected by [`validate_transition`].

use std::fmt;

/// The five states a job can be in (ADR-0004: states represented as a fixed,
/// finite set; here as a Rust enum rather than raw integers, since this
/// table is not on the hot trading path).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JobState {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl JobState {
    /// The exact string persisted in the `jobs.state` column
    /// (migration `0003_jobs.sql`).
    pub fn as_str(&self) -> &'static str {
        match self {
            JobState::Queued => "QUEUED",
            JobState::Running => "RUNNING",
            JobState::Completed => "COMPLETED",
            JobState::Failed => "FAILED",
            JobState::Cancelled => "CANCELLED",
        }
    }

    /// Parses a `jobs.state` column value back into a [`JobState`].
    ///
    /// Returns `None` for any value other than the five canonical strings —
    /// the persistence layer treats that as a data-integrity error (a row
    /// written outside this state machine), not a silent default.
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "QUEUED" => Some(JobState::Queued),
            "RUNNING" => Some(JobState::Running),
            "COMPLETED" => Some(JobState::Completed),
            "FAILED" => Some(JobState::Failed),
            "CANCELLED" => Some(JobState::Cancelled),
            _ => None,
        }
    }

    /// A terminal state is never the source of a valid transition
    /// (async-job-executor.md "Restricciones": "Una vez CANCELLED, no se
    /// puede reanudar"; the same applies to COMPLETED/FAILED).
    pub fn is_terminal(&self) -> bool {
        matches!(self, JobState::Completed | JobState::Failed | JobState::Cancelled)
    }
}

impl fmt::Display for JobState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A transition `(from, to)` that [`validate_transition`] rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTransition {
    pub from: JobState,
    pub to: JobState,
}

impl fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid job state transition: {} -> {}", self.from, self.to)
    }
}

impl std::error::Error for InvalidTransition {}

/// Validates a proposed `(from, to)` state transition against the table in
/// this module's doc comment.
///
/// Returns `Ok(to)` if the transition is allowed, or
/// [`InvalidTransition`] otherwise. Pure: no I/O, deterministic.
pub fn validate_transition(from: JobState, to: JobState) -> Result<JobState, InvalidTransition> {
    let allowed = matches!(
        (from, to),
        (JobState::Queued, JobState::Running)
            | (JobState::Running, JobState::Completed)
            | (JobState::Running, JobState::Failed)
            | (JobState::Queued, JobState::Cancelled)
            | (JobState::Running, JobState::Cancelled)
            | (JobState::Running, JobState::Queued)
    );

    if allowed {
        Ok(to)
    } else {
        Err(InvalidTransition { from, to })
    }
}

/// Progress percentage, clamped to the `0..=100` range mandated by
/// async-job-executor.md TTR-005 ("Progreso es 0-100%").
///
/// Construction always succeeds: out-of-range inputs are clamped rather than
/// rejected, since a worker reporting `progress = 104` due to a rounding
/// quirk should not abort the job — it should be recorded as `100`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Progress(u8);

impl Progress {
    /// Clamps `percent` into `0..=100`.
    pub fn new(percent: u8) -> Self {
        Progress(percent.min(100))
    }

    pub fn value(&self) -> u8 {
        self.0
    }

    /// `0%` — the value a job starts at when it transitions to `RUNNING`
    /// (TTR-002 "Worker cambia estado a RUNNING, inicializa progreso=0").
    pub fn zero() -> Self {
        Progress(0)
    }

    /// `100%` — the value a job reaches when it transitions to `COMPLETED`.
    pub fn complete() -> Self {
        Progress(100)
    }
}

impl fmt::Display for Progress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Estimates the remaining time for a running job (TTR-005 "Reglas de
/// Negocio": `estimación = (elapsed_time / progress) * (100 - progress)`).
///
/// Returns `None` when the estimate cannot be computed:
/// - `progress == 0`: no work has been observed yet, so the elapsed/progress
///   ratio is undefined (division by zero).
///
/// Returns `Some(0)` when `progress >= 100` (nothing left to do — TTR-005's
/// formula gives `(elapsed/100) * 0 = 0`, returned directly to sidestep any
/// floating-point rounding at the boundary).
///
/// `elapsed_seconds` and the result are both expressed in whole seconds
/// (TTR-005 example: `"estimated_time_remaining": "2 minutes"`).
pub fn estimate_remaining_seconds(progress: Progress, elapsed_seconds: u64) -> Option<u64> {
    let percent = progress.value();

    if percent == 0 {
        return None;
    }

    if percent >= 100 {
        return Some(0);
    }

    // remaining = elapsed * (100 - percent) / percent
    let percent = percent as u128;
    let remaining = (elapsed_seconds as u128) * (100 - percent) / percent;

    Some(remaining as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- validate_transition: allowed transitions ---------------------

    #[test]
    fn queued_to_running_is_allowed() {
        assert_eq!(
            validate_transition(JobState::Queued, JobState::Running),
            Ok(JobState::Running)
        );
    }

    #[test]
    fn running_to_completed_is_allowed() {
        assert_eq!(
            validate_transition(JobState::Running, JobState::Completed),
            Ok(JobState::Completed)
        );
    }

    #[test]
    fn running_to_failed_is_allowed() {
        assert_eq!(
            validate_transition(JobState::Running, JobState::Failed),
            Ok(JobState::Failed)
        );
    }

    #[test]
    fn queued_to_cancelled_is_allowed() {
        assert_eq!(
            validate_transition(JobState::Queued, JobState::Cancelled),
            Ok(JobState::Cancelled)
        );
    }

    #[test]
    fn running_to_cancelled_is_allowed() {
        assert_eq!(
            validate_transition(JobState::Running, JobState::Cancelled),
            Ok(JobState::Cancelled)
        );
    }

    /// TTR-004: startup recovery re-queues a job that was `RUNNING` when the
    /// process died, because completion is unknown.
    #[test]
    fn running_to_queued_is_allowed_for_recovery() {
        assert_eq!(
            validate_transition(JobState::Running, JobState::Queued),
            Ok(JobState::Queued)
        );
    }

    // --- validate_transition: rejected transitions ---------------------

    /// Terminal states never transition anywhere
    /// (async-job-executor.md: "Una vez CANCELLED, no se puede reanudar").
    #[test]
    fn terminal_states_reject_every_transition() {
        for terminal in [JobState::Completed, JobState::Failed, JobState::Cancelled] {
            for target in [
                JobState::Queued,
                JobState::Running,
                JobState::Completed,
                JobState::Failed,
                JobState::Cancelled,
            ] {
                let result = validate_transition(terminal, target);
                assert!(
                    result.is_err(),
                    "expected {terminal} -> {target} to be rejected, got {result:?}"
                );
            }
        }
    }

    #[test]
    fn queued_to_completed_is_rejected() {
        let result = validate_transition(JobState::Queued, JobState::Completed);
        assert_eq!(
            result,
            Err(InvalidTransition {
                from: JobState::Queued,
                to: JobState::Completed
            })
        );
    }

    #[test]
    fn queued_to_failed_is_rejected() {
        assert!(validate_transition(JobState::Queued, JobState::Failed).is_err());
    }

    #[test]
    fn queued_to_queued_is_rejected() {
        assert!(validate_transition(JobState::Queued, JobState::Queued).is_err());
    }

    #[test]
    fn running_to_running_is_rejected() {
        assert!(validate_transition(JobState::Running, JobState::Running).is_err());
    }

    #[test]
    fn invalid_transition_display_is_human_readable() {
        let err = InvalidTransition {
            from: JobState::Queued,
            to: JobState::Completed,
        };
        assert_eq!(err.to_string(), "invalid job state transition: QUEUED -> COMPLETED");
    }

    // --- JobState string round-trip -------------------------------------

    #[test]
    fn job_state_round_trips_through_its_string_representation() {
        for state in [
            JobState::Queued,
            JobState::Running,
            JobState::Completed,
            JobState::Failed,
            JobState::Cancelled,
        ] {
            let s = state.as_str();
            assert_eq!(JobState::from_str_value(s), Some(state));
        }
    }

    #[test]
    fn from_str_value_rejects_unknown_strings() {
        assert_eq!(JobState::from_str_value("BOGUS"), None);
        assert_eq!(JobState::from_str_value(""), None);
        assert_eq!(JobState::from_str_value("queued"), None); // case-sensitive
    }

    #[test]
    fn is_terminal_matches_the_three_terminal_states() {
        assert!(!JobState::Queued.is_terminal());
        assert!(!JobState::Running.is_terminal());
        assert!(JobState::Completed.is_terminal());
        assert!(JobState::Failed.is_terminal());
        assert!(JobState::Cancelled.is_terminal());
    }

    // --- Progress ---------------------------------------------------------

    #[test]
    fn progress_clamps_values_above_100() {
        assert_eq!(Progress::new(150).value(), 100);
        assert_eq!(Progress::new(255).value(), 100);
    }

    #[test]
    fn progress_accepts_values_within_range() {
        assert_eq!(Progress::new(0).value(), 0);
        assert_eq!(Progress::new(45).value(), 45);
        assert_eq!(Progress::new(100).value(), 100);
    }

    #[test]
    fn progress_zero_and_complete_constants() {
        assert_eq!(Progress::zero().value(), 0);
        assert_eq!(Progress::complete().value(), 100);
    }

    // --- estimate_remaining_seconds (TTR-005) ----------------------------

    /// TTR-005 worked example: 45% done after some elapsed time should
    /// yield a remaining estimate proportional to `(100 - 45) / 45`.
    #[test]
    fn estimate_matches_ttr_005_formula() {
        // elapsed=90s at 45% => remaining = 90 * (100-45)/45 = 90 * 55/45 = 110
        let remaining = estimate_remaining_seconds(Progress::new(45), 90);
        assert_eq!(remaining, Some(110));
    }

    #[test]
    fn estimate_is_none_when_progress_is_zero() {
        assert_eq!(estimate_remaining_seconds(Progress::zero(), 100), None);
    }

    #[test]
    fn estimate_is_zero_when_progress_is_complete() {
        assert_eq!(estimate_remaining_seconds(Progress::complete(), 1_000), Some(0));
    }

    #[test]
    fn estimate_at_50_percent_is_equal_to_elapsed() {
        // 50% done => remaining == elapsed (symmetric midpoint).
        let remaining = estimate_remaining_seconds(Progress::new(50), 60);
        assert_eq!(remaining, Some(60));
    }

    #[test]
    fn estimate_with_zero_elapsed_is_zero() {
        let remaining = estimate_remaining_seconds(Progress::new(10), 0);
        assert_eq!(remaining, Some(0));
    }
}
