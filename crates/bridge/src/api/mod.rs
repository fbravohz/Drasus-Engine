//! Módulo raíz de la API pública del Bridge.
//!
//! Cada archivo aquí corresponde a un observable del Panel Operativo o a una
//! feature con superficie FFI:
//! - `clock`        → timestamp del reloj determinista de Drasus.
//! - `jobs`         → trabajos encolados/ejecutados/completados.
//! - `audit`        → bitácora de eventos de auditoría con hash de cadena.
//! - `data_fetcher` → Sovereign Data Fetcher: envío y consulta de descargas
//!   de históricos de mercado (STORY-024, EPIC-1).

pub mod audit;
pub mod clock;
pub mod data_fetcher;
pub mod jobs;
