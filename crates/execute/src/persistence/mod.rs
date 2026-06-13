//! [SHELL] Persistence for `execute`.
//!
//! Owns this module's tables exclusively (ADR-0003): orders,
//! executions and supervision events. Other modules must go
//! through `public_interface`.

pub mod models;
pub mod repository;
