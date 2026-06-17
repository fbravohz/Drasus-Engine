//! [SHELL] Persistencia de `incubate`.
//!
//! Dueño exclusivo de las tablas de este módulo (ADR-0003): sesiones
//! de paper trading y resultados de comparación. Otros módulos deben
//! pasar por `public_interface`.

pub mod models;
pub mod repository;
