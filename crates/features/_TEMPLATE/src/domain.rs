//! Functional Core — lógica pura, sin I/O.
//!
//! PROHIBIDO: imports de `shared::persistence`, `shared::orchestrator`,
//! `tokio`, `sqlx`, o cualquier crate que toque el sistema.
//! PERMITIDO: `shared::types`, `shared::domain` (si aplica).
