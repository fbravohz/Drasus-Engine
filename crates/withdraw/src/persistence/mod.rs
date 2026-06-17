//! [SHELL] Persistencia de `withdraw`.
//!
//! Dueño exclusivo de las tablas de este módulo (ADR-0003): log de
//! retiros y estrategias archivadas. Otros módulos deben pasar por
//! `public_interface`.

pub mod models;
pub mod repository;
