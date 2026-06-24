//! `shared`: componentes transversales de Drasus Engine (ADR-0003).
//!
//! Alberga los tipos del catálogo de puertos (ADR-0137), la plomería
//! cross-cutting (Clock, AuditLog, Telemetry, JobExecutor, MCP Gateway)
//! y las utilidades compartidas. Es la ÚNICA dependencia permitida para
//! cualquier feature crate.
//!
//! ## Estructura interna
//!
//! - `types`: catálogo canónico de 109 tipos de puerto (ADR-0137).
//!   Cada tipo implementa el trait `TypedPort` con id, color canvas y
//!   cardinalidad.
//! - `domain`: lógica pura, sin I/O (Clock determinista, estados de Job).
//! - `orchestrator`: cáscara delgada — implementaciones de producción
//!   (SystemClock, JobExecutor, TelemetryBuffer, MCP server).
//! - `clock_audit`: rastro de auditoría del Clock hacia el AuditLog.
//! - `persistence`: fábrica del pool SQLite + migraciones SQLx embebidas
//!   (ADR-0006).
//! - `public_interface`: superficie pública — lo único que otros crates
//!   pueden importar.
//! - `schemas`: contratos de datos compartidos (Serde).

pub mod clock_audit;
pub mod domain;
pub mod orchestrator;
pub mod persistence;
pub mod public_interface;
pub mod schemas;
pub mod types;

#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles_and_links() {
        assert_eq!(2 + 2, 4);
    }
}
