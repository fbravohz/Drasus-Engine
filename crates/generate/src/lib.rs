//! `generate`: módulo de generación de estrategias/candidatos (ADR-0003, FCIS).
//!
//! Etapa del pipeline: **Generate** — generar candidatos y evaluar su fitness.
//!
//! Estructura fija del módulo:
//! - `domain`: lógica pura (evolución genética, regresión simbólica). Sin I/O.
//! - `orchestrator`: cáscara delgada (bucle evolutivo, combinación de señales).
//! - `persistence`: cáscara delgada (persistencia de estrategias, análisis de factores).
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
