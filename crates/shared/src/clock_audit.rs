//! [SHELL] Audit trail emitter for the Clock (`docs/features/clock.md`
//! "Gobernanza y Estándares", TTR-001/TTR-002 postconditions).
//!
//! The Clock has no persistence of its own. Its three auditable events are
//! emitted through the existing Audit Log port
//! ([`crate::domain::audit_log::AuditEventContent`] +
//! [`crate::persistence::audit_log::AuditLogRepository::append`]) — no new
//! table is created (ADR-0020 V2 Profile D, "Ops / Auditoría").
//!
//! This module performs I/O (it writes to SQLite through
//! [`AuditLogRepository`]), so it lives in the shell, not in
//! `domain::clock` — `domain::clock`'s bit-for-bit determinism must stay
//! untouched (ADR-0002/0004, FCIS).
//!
//! ## Granularity (critical, clock.md "Granularidad de Auditoría")
//!
//! `timestamp_ns()`, `advance(ns)` and `tick()` are hot-path calls (millions
//! of invocations) and MUST NEVER call [`AuditLogRepository::append`]. This
//! module exposes exactly three emission functions, one per allowed event,
//! called only at the three specific lifecycle points clock.md defines:
//!
//! | Function | `action_type` | When |
//! |---|---|---|
//! | [`emit_ntp_sync`] | `CLOCK_NTP_SYNC` | Once, at startup, after verifying NTP sync (TTR-001) |
//! | [`emit_mode_transition`] | `CLOCK_MODE_TRANSITION` | On `REAL` <-> `SIMULATION` mode transitions |
//! | [`emit_session_close`] | `CLOCK_SESSION_CLOSE` | Once, when a simulation session closes (TTR-002) |
//!
//! ## Catalog vs. payload (clock.md "Persistencia y Perfil de Auditoría")
//!
//! - `entity_type` is always `"CLOCK"`; `entity_id` is the active session's
//!   `session_id` (also carried in the ADR-0020 V2 Group IV `session_id`
//!   catalog field).
//! - `institutional_tag` (Group II) and `process_id` (Group IV) are
//!   mandatory catalog fields per the "Ops / Auditoría" profile — both are
//!   supplied by the caller (the runtime that owns the active session).
//! - The three previously orphaned fields (`ntp_sync_offset`, the
//!   simulation's virtual process identifier, and the accumulated
//!   real/virtual delta) are NOT ADR-0020 V2 catalog columns: they travel as
//!   the event's opaque `details_json` payload, serialized with
//!   `serde_json` using a stable (alphabetical) key order.
//! - Group I (`id`, `created_at`, `updated_at`, `audit_hash`,
//!   `audit_chain_hash`, `event_sequence_id`) is assigned by the audit log
//!   itself on [`AuditLogRepository::append`].

use serde_json::json;

use crate::domain::audit_log::{AuditEvent, AuditEventContent};
use crate::persistence::audit_log::{AuditLogError, AuditLogRepository};

/// The Clock's two execution modes (clock.md TTR-002, `CLOCK_MODE_TRANSITION`).
///
/// Serialized as the literal strings `"REAL"` / `"SIMULATION"` in
/// `details_json`, matching clock.md's vocabulary (TTR-001 `request_type`:
/// `REAL | FAKE`; TTR-002 precondición: modo `SIMULATION`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockMode {
    Real,
    Simulation,
}

impl ClockMode {
    fn as_str(&self) -> &'static str {
        match self {
            ClockMode::Real => "REAL",
            ClockMode::Simulation => "SIMULATION",
        }
    }
}

/// Caller-supplied identity shared by every Clock audit event
/// (clock.md "Gobernanza y Estándares").
///
/// - `session_id` becomes both the event's `entity_id` (`entity_type =
///   "CLOCK"`) and its ADR-0020 V2 Group IV `session_id` catalog field —
///   "el campo canónico para agrupar un runtime" (TTR-002).
/// - `institutional_tag` (Group II) and `process_id` (Group IV) are
///   mandatory per the "Ops / Auditoría" profile (audit-log.md TTR-001:
///   "Toda entrada DEBE incluir `process_id` y `institutional_tag`").
#[derive(Debug, Clone)]
pub struct ClockAuditContext<'a> {
    pub session_id: &'a str,
    pub institutional_tag: &'a str,
    pub process_id: &'a str,
}

impl ClockAuditContext<'_> {
    /// Builds the ADR-0020 V2 "Ops / Auditoría" catalog fields shared by all
    /// three Clock events, leaving `action_type` and `details_json` to be
    /// filled in by each `emit_*` function.
    fn base_content(&self, action_type: &str, details_json: String) -> AuditEventContent {
        AuditEventContent {
            action_type: action_type.to_string(),
            entity_type: "CLOCK".to_string(),
            entity_id: self.session_id.to_string(),
            details_json,
            owner_id: None,
            institutional_tag: self.institutional_tag.to_string(),
            manifest_id: None,
            access_token_id: None,
            process_id: self.process_id.to_string(),
            session_id: Some(self.session_id.to_string()),
            node_id: None,
        }
    }
}

/// Emits the `CLOCK_NTP_SYNC` event (clock.md TTR-001 postcondición).
///
/// Called exactly once, at startup, after the NTP sync check — NEVER on
/// every `timestamp_ns()` read (clock.md "Granularidad de Auditoría").
///
/// `ntp_sync_offset_ns` is the measured NTP delta (ADR-0013), carried as
/// opaque payload in `details_json`: `{"ntp_sync_offset_ns": <i64>}`. It is
/// NOT an ADR-0020 V2 catalog field.
pub async fn emit_ntp_sync(
    repo: &AuditLogRepository<'_>,
    ctx: &ClockAuditContext<'_>,
    ntp_sync_offset_ns: i64,
) -> Result<AuditEvent, AuditLogError> {
    let details_json = json!({ "ntp_sync_offset_ns": ntp_sync_offset_ns }).to_string();
    let content = ctx.base_content("CLOCK_NTP_SYNC", details_json);
    repo.append(content).await
}

/// Emits the `CLOCK_MODE_TRANSITION` event (clock.md "Granularidad de
/// Auditoría").
///
/// Called on every `REAL` <-> `SIMULATION` transition — NEVER on every
/// `advance(ns)`/`tick()` call.
///
/// Payload: `{"from": "REAL|SIMULATION", "to": "REAL|SIMULATION"}`.
pub async fn emit_mode_transition(
    repo: &AuditLogRepository<'_>,
    ctx: &ClockAuditContext<'_>,
    from: ClockMode,
    to: ClockMode,
) -> Result<AuditEvent, AuditLogError> {
    let details_json = json!({ "from": from.as_str(), "to": to.as_str() }).to_string();
    let content = ctx.base_content("CLOCK_MODE_TRANSITION", details_json);
    repo.append(content).await
}

/// Emits the `CLOCK_SESSION_CLOSE` event (clock.md TTR-002 postcondición).
///
/// Called exactly once, when a simulation session closes — NEVER on every
/// `advance(ns)`.
///
/// `virtual_process_id` is the simulation's virtual process identifier
/// (TTR-002: "El identificador del proceso virtual de la simulación viaja
/// como payload"). `real_virtual_delta_ns` is the accumulated delta between
/// real and virtual time. Neither is an ADR-0020 V2 catalog field; both
/// travel in `details_json`:
/// `{"real_virtual_delta_ns": <i64>, "virtual_process_id": <string>}`.
pub async fn emit_session_close(
    repo: &AuditLogRepository<'_>,
    ctx: &ClockAuditContext<'_>,
    virtual_process_id: &str,
    real_virtual_delta_ns: i64,
) -> Result<AuditEvent, AuditLogError> {
    let details_json = json!({
        "real_virtual_delta_ns": real_virtual_delta_ns,
        "virtual_process_id": virtual_process_id,
    })
    .to_string();
    let content = ctx.base_content("CLOCK_SESSION_CLOSE", details_json);
    repo.append(content).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::audit_log::{verify_chain, ChainVerificationResult};
    use crate::domain::clock::DeterministicClock;
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> sqlx::SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("connect in-memory db");
        migrate(&pool).await.expect("apply migrations");
        pool
    }

    fn sample_ctx() -> ClockAuditContext<'static> {
        ClockAuditContext {
            session_id: "session-clock-1",
            institutional_tag: "BACKTEST",
            process_id: "process-clock-1",
        }
    }

    /// `CLOCK_NTP_SYNC` persists with the right catalog fields and a
    /// `details_json` payload of exactly `{"ntp_sync_offset_ns": <i64>}`.
    #[tokio::test]
    async fn ntp_sync_event_persists_with_offset_payload() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AuditLogRepository::new(&pool, &clock);
        let ctx = sample_ctx();

        let event = emit_ntp_sync(&repo, &ctx, 42_500)
            .await
            .expect("emit CLOCK_NTP_SYNC");

        assert_eq!(event.content.action_type, "CLOCK_NTP_SYNC");
        assert_eq!(event.content.entity_type, "CLOCK");
        assert_eq!(event.content.entity_id, "session-clock-1");
        assert_eq!(event.content.session_id, Some("session-clock-1".to_string()));
        assert_eq!(event.content.institutional_tag, "BACKTEST");
        assert_eq!(event.content.process_id, "process-clock-1");
        assert_eq!(event.content.details_json, "{\"ntp_sync_offset_ns\":42500}");
    }

    /// `CLOCK_MODE_TRANSITION` persists with `from`/`to` mode strings in
    /// `details_json`.
    #[tokio::test]
    async fn mode_transition_event_persists_with_from_to_payload() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AuditLogRepository::new(&pool, &clock);
        let ctx = sample_ctx();

        let event = emit_mode_transition(&repo, &ctx, ClockMode::Real, ClockMode::Simulation)
            .await
            .expect("emit CLOCK_MODE_TRANSITION");

        assert_eq!(event.content.action_type, "CLOCK_MODE_TRANSITION");
        assert_eq!(event.content.entity_type, "CLOCK");
        assert_eq!(event.content.entity_id, "session-clock-1");
        assert_eq!(
            event.content.details_json,
            "{\"from\":\"REAL\",\"to\":\"SIMULATION\"}"
        );
    }

    /// `CLOCK_SESSION_CLOSE` persists with the virtual process id and the
    /// accumulated real/virtual delta in `details_json`.
    #[tokio::test]
    async fn session_close_event_persists_with_delta_and_virtual_process_payload() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AuditLogRepository::new(&pool, &clock);
        let ctx = sample_ctx();

        let event = emit_session_close(&repo, &ctx, "virtual-proc-7", -123_456)
            .await
            .expect("emit CLOCK_SESSION_CLOSE");

        assert_eq!(event.content.action_type, "CLOCK_SESSION_CLOSE");
        assert_eq!(event.content.entity_type, "CLOCK");
        assert_eq!(event.content.entity_id, "session-clock-1");
        assert_eq!(
            event.content.details_json,
            "{\"real_virtual_delta_ns\":-123456,\"virtual_process_id\":\"virtual-proc-7\"}"
        );
    }

    /// CLOSING CRITERION (a)+(b): emitting all three Clock events back to
    /// back produces a chain that [`verify_chain`] reports as
    /// [`ChainVerificationResult::Valid`] — the Clock's audit trail does not
    /// break the existing hash chain.
    #[tokio::test]
    async fn emitting_all_three_events_keeps_chain_valid() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AuditLogRepository::new(&pool, &clock);
        let ctx = sample_ctx();

        emit_ntp_sync(&repo, &ctx, 1_000)
            .await
            .expect("emit CLOCK_NTP_SYNC");
        clock.tick();
        emit_mode_transition(&repo, &ctx, ClockMode::Real, ClockMode::Simulation)
            .await
            .expect("emit CLOCK_MODE_TRANSITION");
        clock.tick();
        emit_session_close(&repo, &ctx, "virtual-proc-7", 999)
            .await
            .expect("emit CLOCK_SESSION_CLOSE");

        let chain = repo.load_chain().await.expect("load chain");
        assert_eq!(chain.len(), 3);
        assert_eq!(verify_chain(&chain), ChainVerificationResult::Valid);
    }

    /// CLOSING CRITERION (c): all three Clock events are retrievable via
    /// `events_for_entity("CLOCK", session_id)`, ordered by
    /// `event_sequence_id`, with their respective `action_type`s.
    #[tokio::test]
    async fn events_for_entity_returns_all_clock_events_for_session() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AuditLogRepository::new(&pool, &clock);
        let ctx = sample_ctx();

        emit_ntp_sync(&repo, &ctx, 1_000)
            .await
            .expect("emit CLOCK_NTP_SYNC");
        clock.tick();
        emit_mode_transition(&repo, &ctx, ClockMode::Real, ClockMode::Simulation)
            .await
            .expect("emit CLOCK_MODE_TRANSITION");
        clock.tick();
        emit_session_close(&repo, &ctx, "virtual-proc-7", 999)
            .await
            .expect("emit CLOCK_SESSION_CLOSE");

        let events = repo
            .events_for_entity("CLOCK", "session-clock-1")
            .await
            .expect("query CLOCK events for session");

        assert_eq!(events.len(), 3);
        assert_eq!(events[0].content.action_type, "CLOCK_NTP_SYNC");
        assert_eq!(events[1].content.action_type, "CLOCK_MODE_TRANSITION");
        assert_eq!(events[2].content.action_type, "CLOCK_SESSION_CLOSE");
        assert!(events[0].event_sequence_id < events[1].event_sequence_id);
        assert!(events[1].event_sequence_id < events[2].event_sequence_id);
    }

    /// `events_for_entity` scoped to a different `session_id` returns
    /// nothing — `entity_id` correctly tracks the active session.
    #[tokio::test]
    async fn events_for_entity_is_scoped_to_session_id() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AuditLogRepository::new(&pool, &clock);
        let ctx = sample_ctx();

        emit_ntp_sync(&repo, &ctx, 1_000)
            .await
            .expect("emit CLOCK_NTP_SYNC");

        let other_session_events = repo
            .events_for_entity("CLOCK", "some-other-session")
            .await
            .expect("query CLOCK events for unrelated session");

        assert!(other_session_events.is_empty());
    }
}
