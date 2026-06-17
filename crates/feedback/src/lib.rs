//! `feedback`: módulo de ciclo de retroalimentación estadística (ADR-0003, FCIS).
//!
//! Etapa del pipeline: **Feedback** — control de calidad estadístico (Pardo),
//! veredicto de salud.
//!
//! Layout fijo del módulo:
//! - `domain`: lógica pura (detección de drift, real vs. esperado). Sin I/O.
//! - `orchestrator`: cáscara delgada (cierre de ciclo de vida, veredicto de retiro).
//! - `persistence`: cáscara delgada (historial de veredictos, restricciones de aprendizaje).
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
