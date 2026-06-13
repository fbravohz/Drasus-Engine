//! [SHELL] Orchestration for `shared`.
//!
//! Coordinates `domain` logic for reusable components (FCIS, ADR-0003).
//!
//! `SystemClock` is the only piece of `shared` that touches real I/O (the
//! operating system clock). It implements the `Clock` port (TTR-001,
//! `docs/features/clock.md`) for production use (`request_type = REAL`).
//!
//! - `job_executor`: the Async Job Executor's shell -- Tokio worker pool,
//!   in-memory queue, UUID generation, [`Clock`] reads, and startup
//!   recovery (`docs/features/async-job-executor.md`
//!   TTR-ASYNC-EXECUTOR-001/002/004/005/006, ADR-0011).

pub mod job_executor;

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::clock::Clock;

/// Production implementation of the [`Clock`] port (TTR-001).
///
/// Wraps `SystemTime::now()` and converts it to nanoseconds since the Unix
/// epoch (TTR-001: "En producción, utiliza `time.time_ns()` para evitar
/// errores de precisión de punto flotante.").
///
/// `SystemTime` itself is NOT guaranteed monotonic across calls (an NTP
/// step can move the wall clock backwards). To uphold the Clock port's
/// invariant — "NUNCA Clock devuelve un valor menor al anterior" — this
/// implementation remembers the last timestamp it returned and clamps any
/// new reading to be strictly greater than it.
pub struct SystemClock {
    last_timestamp_ns: AtomicI64,
}

impl SystemClock {
    /// Creates a new `SystemClock`. The first call to
    /// [`Clock::timestamp_ns`] returns the current wall-clock time.
    pub fn new() -> Self {
        Self {
            last_timestamp_ns: AtomicI64::new(i64::MIN),
        }
    }

    /// Reads the current wall-clock time as nanoseconds since the Unix
    /// epoch. Panics only if the system clock is set before the Unix
    /// epoch (1970-01-01), which is not a supported deployment.
    fn read_system_time_ns() -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is set before the Unix epoch");

        now.as_nanos()
            .try_into()
            .expect("system time in nanoseconds overflows i64")
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn timestamp_ns(&self) -> i64 {
        let observed_ns = Self::read_system_time_ns();

        // Enforce monotonic non-decreasing time even if the OS clock steps
        // backwards (e.g. NTP correction): never return less than (or
        // equal to, across consecutive calls) the previous value.
        let mut previous = self.last_timestamp_ns.load(Ordering::SeqCst);
        loop {
            // Strictly greater than the previous value, but otherwise the
            // real observed time (no artificial drift when the OS clock
            // is already ahead).
            let next = observed_ns.max(previous + 1);

            match self.last_timestamp_ns.compare_exchange(
                previous,
                next,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return next,
                Err(actual) => previous = actual,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_ns_is_monotonically_increasing() {
        let clock = SystemClock::new();

        let first = clock.timestamp_ns();
        let second = clock.timestamp_ns();
        let third = clock.timestamp_ns();

        assert!(second >= first);
        assert!(third >= second);
    }

    #[test]
    fn timestamp_ns_is_positive_and_plausible() {
        let clock = SystemClock::new();
        let ts = clock.timestamp_ns();

        // Sanity bound: any timestamp after 2020-01-01 (in nanoseconds).
        assert!(ts > 1_577_836_800_000_000_000);
    }
}
