//! `shared`: Reusable components for Drasus Engine (ADR-0003).
//!
//! Houses cross-cutting building blocks (telemetry, common types, utilities)
//! consumed by every pipeline module through their public interfaces.
//!
//! Follows the same FCIS layout as pipeline modules:
//! - `domain`: pure logic, no I/O, no system clock, no unseeded randomness.
//!   Includes the `Clock` port and `DeterministicClock` (W3, "clock"
//!   feature, TTR-001/TTR-002).
//! - `orchestrator`: thin shell coordinating domain logic. Includes
//!   `SystemClock`, the only piece of `shared` that reads the real system
//!   clock (W3, TTR-001 production implementation).
//! - `clock_audit`: thin shell emitting the Clock's audit trail
//!   (`CLOCK_NTP_SYNC`, `CLOCK_MODE_TRANSITION`, `CLOCK_SESSION_CLOSE`) to
//!   the existing Audit Log (`docs/features/clock.md` "Gobernanza y
//!   Estándares"). I/O (writes via `AuditLogRepository::append`), so it
//!   lives outside `domain::clock`.
//! - `persistence`: centralized SQLite pool factory + embedded SQLx
//!   migrations (ADR-0006). Migration files live in `/migrations` at the
//!   workspace root.
//! - `public_interface`: the only surface other crates may depend on.
//! - `schemas`: shared data contracts (Serde-validated at boundaries).

pub mod clock_audit;
pub mod domain;
pub mod orchestrator;
pub mod persistence;
pub mod public_interface;
pub mod schemas;

#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles_and_links() {
        assert_eq!(2 + 2, 4);
    }
}
