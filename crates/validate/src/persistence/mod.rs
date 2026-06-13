//! [SHELL] Persistence for `validate`.
//!
//! Owns this module's tables exclusively (ADR-0003): test results
//! and metrics. Other modules must go through `public_interface`.

pub mod models;
pub mod repository;
