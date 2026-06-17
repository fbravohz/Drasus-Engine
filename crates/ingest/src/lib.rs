//! `ingest`: módulo de adquisición de datos de mercado (ADR-0003, FCIS).
//!
//! Etapa del pipeline: **Ingest** — obtiene barras de mercado y detecta el régimen.
//!
//! Estructura fija del módulo:
//! - `domain`: lógica pura (parseo de precios, detección de anomalías). Sin I/O.
//! - `orchestrator`: cáscara delgada (manejo de gRPC/WebSocket, normalización).
//! - `persistence`: cáscara delgada (almacenamiento de barras, historial de régimen).
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
