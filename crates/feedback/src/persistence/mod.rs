//! [SHELL] Persistencia de `feedback`.
//!
//! Posee en exclusiva las tablas de este módulo (ADR-0003): anomalías,
//! sugerencias y veredictos. Otros módulos deben pasar por
//! `public_interface`.

pub mod models;
pub mod repository;
