//! `validate`: módulo de validación de estrategias (ADR-0003, FCIS).
//!
//! Etapa del pipeline: **Validate** — validar estrategia, correr suites de prueba.
//!
//! Layout fijo del módulo:
//! - `domain`: lógica pura (análisis walk-forward, Monte Carlo, pruebas
//!   de coherencia). Sin I/O.
//! - `orchestrator`: cáscara delgada (orquestación de backtests, cálculo de métricas).
//! - `persistence`: cáscara delgada (resultados del motor de pruebas, métricas de validación).
//! - `public_interface`: el único puerto que pueden llamar otros módulos.
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
