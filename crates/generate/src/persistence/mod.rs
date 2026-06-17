//! [SHELL] Persistencia de `generate`.
//!
//! Dueño exclusivo de las tablas de este módulo (ADR-0003): planos de
//! estrategias y candidatos. Otros módulos deben pasar por
//! `public_interface`.

pub mod models;
pub mod repository;
