//! `shared`: componentes reutilizables de Drasus Engine (ADR-0003).
//!
//! Alberga building blocks transversales (telemetría, tipos comunes,
//! utilidades) que cada módulo del pipeline consume a través de su
//! interfaz pública.
//!
//! Sigue el mismo layout FCIS que los módulos del pipeline:
//! - `domain`: lógica pura, sin I/O, sin reloj de sistema, sin azar sin
//!   semilla. Incluye el puerto `Clock` y `DeterministicClock` (W3,
//!   feature "clock", TTR-001/TTR-002).
//! - `orchestrator`: cáscara delgada que coordina la lógica de `domain`.
//!   Incluye `SystemClock`, la única pieza de `shared` que lee el reloj
//!   real del sistema (W3, implementación de producción de TTR-001).
//! - `clock_audit`: cáscara delgada que emite el rastro de auditoría del
//!   Clock (`CLOCK_NTP_SYNC`, `CLOCK_MODE_TRANSITION`,
//!   `CLOCK_SESSION_CLOSE`) hacia el Audit Log existente
//!   (`docs/features/clock.md` "Gobernanza y Estándares"). Hace I/O
//!   (escribe vía `AuditLogRepository::append`), por eso vive fuera de
//!   `domain::clock`.
//! - `persistence`: fábrica centralizada del pool de SQLite + migraciones
//!   de SQLx embebidas (ADR-0006). Los archivos de migración viven en
//!   `/migrations` en la raíz del workspace.
//! - `public_interface`: la única superficie de la que otros crates
//!   pueden depender.
//! - `schemas`: contratos de datos compartidos (validados con Serde en
//!   las fronteras).

pub mod clock_audit;
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
