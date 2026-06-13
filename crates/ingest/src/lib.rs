//! `ingest`: Market data acquisition module (ADR-0003, FCIS).
//!
//! Pipeline stage: **Ingest** — fetch market bars and detect regime.
//!
//! Fixed module layout:
//! - `domain`: pure logic (price parsing, anomaly detection). No I/O.
//! - `orchestrator`: thin shell (gRPC/WebSocket handling, normalization).
//! - `persistence`: thin shell (bar storage, regime history).
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
