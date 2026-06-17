//! `manage`: módulo de gestión de portafolio (ADR-0003, FCIS).
//!
//! Etapa del pipeline: **Manage** — optimizar portafolio, fijar reglas,
//! correr backtests de portafolio HRP.
//!
//! Estructura fija del módulo:
//! - `domain`: lógica pura (optimización de portafolio HRP, correlaciones,
//!   rebalanceo walk-forward). Sin I/O.
//! - `orchestrator`: cáscara delgada (rebalanceo, cálculo de correlaciones).
//! - `persistence`: cáscara delgada (persistencia de portafolio y estrategia).
//! - `public_interface`: el único puerto que otros módulos pueden llamar.
//! - `schemas`: contratos de entrada/salida de este módulo.
//!
//! Esqueleto vacío para F0 (W1): todavía sin lógica de negocio implementada.

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
