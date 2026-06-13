//! `incubate`: Paper-trading incubation module (ADR-0003, FCIS).
//!
//! Pipeline stage: **Incubate** — run paper trading and compare with backtest.
//!
//! Fixed module layout:
//! - `domain`: pure logic (Pardo validation). No I/O.
//! - `orchestrator`: thin shell (execution simulation, change detection).
//! - `persistence`: thin shell (paper trading persistence).
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
