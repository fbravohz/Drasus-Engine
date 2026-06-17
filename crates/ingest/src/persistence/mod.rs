//! [SHELL] Persistencia de `ingest`.
//!
//! Es dueño exclusivo de las tablas de este módulo (ADR-0003): barras e
//! historial de régimen. Otros módulos deben pasar por `public_interface`.

pub mod models;
pub mod repository;
