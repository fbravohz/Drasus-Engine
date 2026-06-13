//! [SHELL] Persistence for `withdraw`.
//!
//! Owns this module's tables exclusively (ADR-0003): withdrawal
//! log and archived strategies. Other modules must go through
//! `public_interface`.

pub mod models;
pub mod repository;
