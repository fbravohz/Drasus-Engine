//! `generate`: Strategy/candidate generation module (ADR-0003, FCIS).
//!
//! Pipeline stage: **Generate** — generate candidates and evaluate fitness.
//!
//! Fixed module layout:
//! - `domain`: pure logic (genetic evolution, symbolic regression). No I/O.
//! - `orchestrator`: thin shell (evolutionary loop, signal combination).
//! - `persistence`: thin shell (strategy persistence, factor analysis).
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
