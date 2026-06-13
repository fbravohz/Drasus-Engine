//! [CORE] Pure clock primitives — the `Clock` port and the deterministic
//! (backtest-ready) clock implementation.
//!
//! No I/O, no system clock access, no unseeded randomness (ADR-0002/0004).
//! `docs/features/clock.md`:
//! - TTR-001: nanosecond-precision timestamp port (`Clock::timestamp_ns`).
//! - TTR-002: deterministic clock for reproducible simulations
//!   (`DeterministicClock`), advanced only via explicit `advance(ns)` calls.

use std::sync::atomic::{AtomicI64, Ordering};

/// The Clock port (TTR-001).
///
/// Any module needing the current time calls this port instead of the
/// system clock directly (`docs/features/clock.md`: "NUNCA un módulo llama
/// a `datetime.now()` o equivalente directo. Siempre a través de Clock.").
///
/// Implementations:
/// - [`super::super::orchestrator::SystemClock`] (shell): wraps the real
///   system clock for production (`request_type = REAL`).
/// - [`DeterministicClock`] (core, this module): a fully controlled,
///   reproducible clock for backtests and tests (`request_type = FAKE`).
///
/// Invariant (clock.md "Restricciones"): a `Clock` implementation NEVER
/// returns a `timestamp_ns` smaller than a previously returned value
/// within the same instance — time is monotonically non-decreasing.
pub trait Clock: Send + Sync {
    /// Returns the current timestamp in nanoseconds since the Unix epoch
    /// (TTR-001: "Expone el Unix timestamp actual con precisión de
    /// nanosegundos").
    fn timestamp_ns(&self) -> i64;
}

/// Deterministic, backtest-ready clock (TTR-002).
///
/// The clock starts at `initial_timestamp_ns` and only moves forward via
/// explicit calls to [`DeterministicClock::advance`]. Repeated calls to
/// [`Clock::timestamp_ns`] between `advance` calls return the exact same
/// value — this is what makes a backtest reproducible bit-for-bit
/// (clock.md: "Todas las llamadas a Clock dentro de la barra devuelven el
/// mismo timestamp").
///
/// `step_ns` is the configured default step (clock.md `ADVANCE_PER_STEP`,
/// in nanoseconds) used by [`DeterministicClock::tick`] to advance by one
/// simulation step (e.g. one bar) without the caller having to repeat the
/// step size on every call.
pub struct DeterministicClock {
    /// Current virtual timestamp, in nanoseconds since the Unix epoch.
    virtual_timestamp_ns: AtomicI64,
    /// Default per-step advance, in nanoseconds (clock.md `ADVANCE_PER_STEP`).
    step_ns: i64,
}

impl DeterministicClock {
    /// Creates a deterministic clock starting at `initial_timestamp_ns`
    /// (clock.md `INITIAL_TIMESTAMP` / TTR-002 entrada) with a default
    /// per-step advance of `step_ns` (clock.md `ADVANCE_PER_STEP`).
    ///
    /// `step_ns` must be `>= 0` (clock.md: "NUNCA Clock devuelve un valor
    /// menor al anterior" — a negative default step would make `tick`
    /// move time backwards).
    pub fn new(initial_timestamp_ns: i64, step_ns: i64) -> Self {
        assert!(
            step_ns >= 0,
            "DeterministicClock::new: step_ns must be >= 0 (got {step_ns}); \
             a negative step would violate the monotonic-time invariant"
        );

        Self {
            virtual_timestamp_ns: AtomicI64::new(initial_timestamp_ns),
            step_ns,
        }
    }

    /// Advances the virtual clock by exactly `delta_ns` nanoseconds
    /// (TTR-002: "El reloj solo avanza mediante llamadas explícitas
    /// `advance(ns)`."). Returns the new `virtual_timestamp_ns`.
    ///
    /// `delta_ns` must be `>= 0` (clock.md: "NUNCA Clock devuelve un valor
    /// menor al anterior. El tiempo es monótono creciente dentro de una
    /// sesión.").
    pub fn advance(&self, delta_ns: i64) -> i64 {
        assert!(
            delta_ns >= 0,
            "DeterministicClock::advance: delta_ns must be >= 0 (got \
             {delta_ns}); time must be monotonically non-decreasing"
        );

        self.virtual_timestamp_ns
            .fetch_add(delta_ns, Ordering::SeqCst)
            + delta_ns
    }

    /// Advances the virtual clock by the configured `step_ns` (clock.md
    /// `ADVANCE_PER_STEP`), e.g. once per simulated bar. Returns the new
    /// `virtual_timestamp_ns`.
    pub fn tick(&self) -> i64 {
        self.advance(self.step_ns)
    }

    /// The configured per-step advance, in nanoseconds (clock.md
    /// `ADVANCE_PER_STEP`).
    pub fn step_ns(&self) -> i64 {
        self.step_ns
    }
}

impl Clock for DeterministicClock {
    fn timestamp_ns(&self) -> i64 {
        self.virtual_timestamp_ns.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TTR-002 closing criterion: bit-for-bit determinism. Two independent
    /// `DeterministicClock` instances, built with the same seed
    /// (`initial_timestamp_ns`, `step_ns`) and driven through the same
    /// sequence of `advance`/`tick` calls, must produce the EXACT same
    /// sequence of `timestamp_ns()` values, every run.
    #[test]
    fn deterministic_clock_same_seed_produces_identical_sequence() {
        const INITIAL_TIMESTAMP_NS: i64 = 1_577_869_800_000_000_000; // 2020-01-01 09:30:00 UTC
        const STEP_NS: i64 = 60_000_000_000; // 60 seconds, in nanoseconds (1m timeframe)

        let run = || {
            let clock = DeterministicClock::new(INITIAL_TIMESTAMP_NS, STEP_NS);
            let mut sequence = Vec::new();

            // Initial read, before any advance.
            sequence.push(clock.timestamp_ns());

            // Simulate 5 bars: each `tick()` advances by exactly `step_ns`,
            // and repeated reads within a "bar" return the same value.
            for _ in 0..5 {
                clock.tick();
                sequence.push(clock.timestamp_ns());
                sequence.push(clock.timestamp_ns()); // same bar, same timestamp
            }

            // An explicit, non-uniform advance (e.g. a custom gap).
            clock.advance(3_600_000_000_000); // +1 hour
            sequence.push(clock.timestamp_ns());

            sequence
        };

        let sequence_a = run();
        let sequence_b = run();

        assert_eq!(
            sequence_a, sequence_b,
            "same seed (initial_timestamp_ns, step_ns) and same call \
             sequence must produce an identical timestamp sequence, bit \
             for bit"
        );

        // Sanity check on the expected values to guard against a
        // vacuously-true comparison (e.g. both sequences being empty).
        assert_eq!(sequence_a.len(), 1 + 5 * 2 + 1);
        assert_eq!(sequence_a[0], INITIAL_TIMESTAMP_NS);
        assert_eq!(
            sequence_a[1],
            INITIAL_TIMESTAMP_NS + STEP_NS,
            "first tick must advance by exactly step_ns"
        );
        assert_eq!(
            sequence_a[1], sequence_a[2],
            "repeated reads within the same simulated bar must be identical"
        );
        let last_tick_value = INITIAL_TIMESTAMP_NS + STEP_NS * 5;
        assert_eq!(
            sequence_a[11],
            last_tick_value + 3_600_000_000_000,
            "final explicit advance must apply on top of the last tick"
        );
    }

    #[test]
    fn timestamp_ns_does_not_change_without_advance() {
        let clock = DeterministicClock::new(1_000, 100);

        let first = clock.timestamp_ns();
        let second = clock.timestamp_ns();
        let third = clock.timestamp_ns();

        assert_eq!(first, 1_000);
        assert_eq!(first, second);
        assert_eq!(second, third);
    }

    #[test]
    fn advance_is_monotonically_non_decreasing() {
        let clock = DeterministicClock::new(0, 0);

        let t1 = clock.advance(500);
        let t2 = clock.advance(0); // zero-delta advance is allowed
        let t3 = clock.advance(250);

        assert_eq!(t1, 500);
        assert_eq!(t2, 500);
        assert_eq!(t3, 750);
        assert!(t2 >= t1);
        assert!(t3 >= t2);
    }

    #[test]
    #[should_panic(expected = "delta_ns must be >= 0")]
    fn advance_rejects_negative_delta() {
        let clock = DeterministicClock::new(0, 0);
        clock.advance(-1);
    }

    #[test]
    #[should_panic(expected = "step_ns must be >= 0")]
    fn new_rejects_negative_step() {
        let _ = DeterministicClock::new(0, -1);
    }

    #[test]
    fn tick_advances_by_configured_step() {
        let clock = DeterministicClock::new(1_000, 60);

        assert_eq!(clock.tick(), 1_060);
        assert_eq!(clock.tick(), 1_120);
        assert_eq!(clock.step_ns(), 60);
    }
}
