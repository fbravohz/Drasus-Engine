//! Módulo raíz de la API pública del Bridge.
//!
//! Cada archivo aquí corresponde a un observable del Panel Operativo o a una
//! feature con superficie FFI:
//! - `clock`        → timestamp del reloj determinista de Drasus.
//! - `jobs`         → trabajos encolados/ejecutados/completados.
//! - `audit`        → bitácora de eventos de auditoría con hash de cadena.
//! - `data_fetcher` → Sovereign Data Fetcher: envío y consulta de descargas
//!   de históricos de mercado (STORY-024, EPIC-1).
//! - `hybrid_data_transformer` → Hybrid Data Transformer: envío de
//!   transformaciones (`TradeTick` → `Bars`) y consulta de datasets de
//!   origen / registros de transformación (STORY-051, EPIC-1).
//! - `data_import_wizard` → Asistente de importación, vía de mapeo manual:
//!   seleccionar archivo, columnas detectadas, vista previa transformada y
//!   confirmación de la declaración (STORY-073, EPIC-1).
//! - `verification` → Banco de Pruebas: despacho genérico hacia cualquier
//!   `verify_<feature>` del backend (Canal #2, ADR-0142).

pub mod audit;
pub mod clock;
pub mod data_fetcher;
pub mod data_import_wizard;
pub mod hybrid_data_transformer;
pub mod jobs;
pub mod verification;
