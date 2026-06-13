//! `withdraw`: Strategy/portfolio withdrawal module (ADR-0003, FCIS).
//!
//! Pipeline stage: **Withdraw** — detect degradation, withdraw strategy.
//!
//! Fixed module layout:
//! - `domain`: pure logic (performance profile comparison). No I/O.
//! - `orchestrator`: thin shell (controlled withdrawal flow, veto
//!   management).
//! - `persistence`: thin shell (archived strategies persistence).
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
