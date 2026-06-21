//! Módulo raíz de la API pública del Bridge.
//!
//! Cada archivo aquí corresponde a un observable del Panel Operativo:
//! - `clock`    → timestamp del reloj determinista de Drasus.
//! - `jobs`     → trabajos encolados/ejecutados/completados.
//! - `audit`    → bitácora de eventos de auditoría con hash de cadena.

pub mod audit;
pub mod clock;
pub mod jobs;
