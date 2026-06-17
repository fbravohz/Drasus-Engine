//! [SHELL] Persistencia para `execute`.
//!
//! Es dueño exclusivo de las tablas de este módulo (ADR-0003): orders,
//! executions y supervision events. Otros módulos deben pasar por
//! `public_interface`.

pub mod models;
pub mod repository;
