//! [SHELL] Persistencia para `validate`.
//!
//! Dueño exclusivo de las tablas de este módulo (ADR-0003): resultados
//! de pruebas y métricas. Otros módulos deben pasar por `public_interface`.

pub mod models;
pub mod repository;
