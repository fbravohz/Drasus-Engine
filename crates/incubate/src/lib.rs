//! `incubate`: módulo de incubación con paper-trading (ADR-0003, FCIS).
//!
//! Etapa del pipeline: **Incubate** — corre paper trading y compara contra el backtest.
//!
//! Estructura fija del módulo:
//! - `domain`: lógica pura (validación de Pardo). Sin I/O.
//! - `orchestrator`: cáscara delgada (simulación de ejecución, detección de cambios).
//! - `persistence`: cáscara delgada (persistencia del paper trading).
//! - `public_interface`: el único puerto que otros módulos pueden invocar.
//! - `schemas`: contratos de entrada/salida de este módulo.
//!
//! Esqueleto vacío en F0 (W1): todavía sin lógica de negocio implementada.

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
