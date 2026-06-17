//! [SHELL] Persistencia para `manage`.
//!
//! Dueño exclusivo de las tablas de este módulo (ADR-0003): portafolios,
//! pesos y reglas. Otros módulos deben pasar por `public_interface`.

pub mod models;
pub mod repository;
