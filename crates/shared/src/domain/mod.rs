//! [CORE] Pure business logic for `shared`.
//!
//! No I/O, no system clock, no unseeded randomness (ADR-0002/0004).
//!
//! - `audit_log`: hash-chain construction and verification for the Audit
//!   Log (`docs/features/audit-log.md` TTR-001, ADR-0015, ADR-0020 V2,
//!   ADR-0027).
//! - `clock`: the `Clock` port and the deterministic (backtest-ready) clock
//!   implementation (W3, `docs/features/clock.md` TTR-001/TTR-002).
//! - `job`: the async job state machine -- valid transitions, progress and
//!   time-remaining estimation (`docs/features/async-job-executor.md`
//!   TTR-ASYNC-EXECUTOR-002/004/005/006, ADR-0004, ADR-0011).
//! - `logic`: empty placeholder, structure only (F0/W1).

pub mod audit_log;
pub mod clock;
pub mod job;
pub mod logic;
