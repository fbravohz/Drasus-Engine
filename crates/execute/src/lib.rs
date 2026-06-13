//! `execute`: Order execution module (ADR-0003, FCIS).
//!
//! Pipeline stage: **Execute** — place order, cancel order, veto.
//!
//! Fixed module layout:
//! - `domain`: pure logic (order state machine, 64-bit FSM). No I/O.
//! - `orchestrator`: thin shell (broker connection, 10 pre-trade
//!   validations per ADR-0025).
//! - `persistence`: thin shell (orders, positions persistence).
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
