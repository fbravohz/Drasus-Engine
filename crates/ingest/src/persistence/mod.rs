//! [SHELL] Persistence for `ingest`.
//!
//! Owns this module's tables exclusively (ADR-0003): bars and
//! regime history. Other modules must go through `public_interface`.

pub mod models;
pub mod repository;
