//! `validate`: Strategy validation module (ADR-0003, FCIS).
//!
//! Pipeline stage: **Validate** — validate strategy, run test suites.
//!
//! Fixed module layout:
//! - `domain`: pure logic (walk-forward analysis, Monte Carlo, coherence
//!   tests). No I/O.
//! - `orchestrator`: thin shell (backtest orchestration, metric computation).
//! - `persistence`: thin shell (test engine results, validation metrics).
//! - `public_interface`: the only port other modules may call.
//! - `schemas`: input/output contracts for this module.
//!
//! Empty skeleton for F0 (W1): no business logic implemented yet.

pub mod domain;
pub mod orchestrator;
pub mod persistence;
pub mod public_interface;
pub mod schemas;

#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles_and_links() {
        assert_eq!(2 + 2, 4);
    }
}
