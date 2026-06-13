//! [SHELL] Persistence for `feedback`.
//!
//! Owns this module's tables exclusively (ADR-0003): anomalies,
//! suggestions and verdicts. Other modules must go through
//! `public_interface`.

pub mod models;
pub mod repository;
