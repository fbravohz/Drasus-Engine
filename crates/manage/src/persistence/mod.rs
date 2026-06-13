//! [SHELL] Persistence for `manage`.
//!
//! Owns this module's tables exclusively (ADR-0003): portfolios,
//! weights and rules. Other modules must go through
//! `public_interface`.

pub mod models;
pub mod repository;
