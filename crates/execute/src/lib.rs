//! `execute`: módulo de ejecución de órdenes (ADR-0003, FCIS).
//!
//! Etapa del pipeline: **Execute** — colocar orden, cancelar orden, veto.
//!
//! Estructura fija del módulo:
//! - `domain`: lógica pura (máquina de estados de órdenes, FSM de 64 bits). Sin I/O.
//! - `orchestrator`: cáscara delgada (conexión al broker, las 10
//!   validaciones pre-trade de ADR-0025).
//! - `persistence`: cáscara delgada (persistencia de órdenes y posiciones).
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
