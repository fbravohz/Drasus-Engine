//! `withdraw`: módulo de retiro de estrategia/portafolio (ADR-0003, FCIS).
//!
//! Etapa del pipeline: **Withdraw** — detectar degradación, retirar estrategia.
//!
//! Estructura fija del módulo:
//! - `domain`: lógica pura (comparación de perfiles de rendimiento). Sin I/O.
//! - `orchestrator`: cáscara delgada (flujo de retiro controlado, gestión
//!   de vetos).
//! - `persistence`: cáscara delgada (persistencia de estrategias archivadas).
//! - `public_interface`: el único puerto que otros módulos pueden llamar.
//! - `schemas`: contratos de entrada/salida de este módulo.
//!
//! Esqueleto vacío para F0 (W1): todavía no hay lógica de negocio implementada.

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
