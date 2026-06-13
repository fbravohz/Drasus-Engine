//! [SHELL] Persistence for `incubate`.
//!
//! Owns this module's tables exclusively (ADR-0003): paper trading
//! sessions and comparison results. Other modules must go through
//! `public_interface`.

pub mod models;
pub mod repository;
