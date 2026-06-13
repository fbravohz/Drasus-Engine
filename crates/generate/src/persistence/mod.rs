//! [SHELL] Persistence for `generate`.
//!
//! Owns this module's tables exclusively (ADR-0003): strategy
//! blueprints and candidates. Other modules must go through
//! `public_interface`.

pub mod models;
pub mod repository;
