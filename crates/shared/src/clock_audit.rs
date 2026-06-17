//! [SHELL] Emisor del rastro de auditoría del Clock (`docs/features/clock.md`
//! "Gobernanza y Estándares", postcondiciones de TTR-001/TTR-002).
//!
//! El Clock no tiene persistencia propia. Sus tres eventos auditables se
//! emiten a través del puerto existente del Audit Log
//! ([`crate::domain::audit_log::AuditEventContent`] +
//! [`crate::persistence::audit_log::AuditLogRepository::append`]) — no se
//! crea ninguna tabla nueva (ADR-0020 V2 Perfil D, "Ops / Auditoría").
//!
//! Este módulo hace I/O (escribe en SQLite a través de
//! [`AuditLogRepository`]), por eso vive en la cáscara y no en
//! `domain::clock` — el determinismo bit a bit de `domain::clock` debe
//! quedar intacto (ADR-0002/0004, FCIS).
//!
//! ## Granularidad (crítico, clock.md "Granularidad de Auditoría")
//!
//! `timestamp_ns()`, `advance(ns)` y `tick()` son llamadas de camino
//! caliente (millones de invocaciones) y NUNCA deben llamar a
//! [`AuditLogRepository::append`]. Este módulo expone exactamente tres
//! funciones de emisión, una por evento permitido, llamadas solo en los
//! tres puntos específicos del ciclo de vida que define clock.md:
//!
//! | Función | `action_type` | Cuándo |
//! |---|---|---|
//! | [`emit_ntp_sync`] | `CLOCK_NTP_SYNC` | Una vez, al iniciar, tras verificar la sincronización NTP (TTR-001) |
//! | [`emit_mode_transition`] | `CLOCK_MODE_TRANSITION` | En transiciones de modo `REAL` <-> `SIMULATION` |
//! | [`emit_session_close`] | `CLOCK_SESSION_CLOSE` | Una vez, cuando cierra una sesión de simulación (TTR-002) |
//!
//! ## Catálogo vs. payload (clock.md "Persistencia y Perfil de Auditoría")
//!
//! - `entity_type` siempre es `"CLOCK"`; `entity_id` es el `session_id` de
//!   la sesión activa (también viaja en el campo de catálogo
//!   `session_id` del Grupo IV de ADR-0020 V2).
//! - `institutional_tag` (Grupo II) y `process_id` (Grupo IV) son campos
//!   de catálogo obligatorios según el perfil "Ops / Auditoría" — ambos
//!   los provee quien llama (el runtime dueño de la sesión activa).
//! - Los tres campos antes huérfanos (`ntp_sync_offset`, el identificador
//!   de proceso virtual de la simulación, y el delta acumulado
//!   real/virtual) NO son columnas de catálogo de ADR-0020 V2: viajan
//!   como payload opaco `details_json` del evento, serializados con
//!   `serde_json` usando un orden de claves estable (alfabético).
//! - El Grupo I (`id`, `created_at`, `updated_at`, `audit_hash`,
//!   `audit_chain_hash`, `event_sequence_id`) lo asigna el propio audit
//!   log al llamar [`AuditLogRepository::append`].

use serde_json::json;

use crate::domain::audit_log::{AuditEvent, AuditEventContent};
use crate::persistence::audit_log::{AuditLogError, AuditLogRepository};

/// Los dos modos de ejecución del Clock (clock.md TTR-002, `CLOCK_MODE_TRANSITION`).
///
/// Se serializan como las cadenas literales `"REAL"` / `"SIMULATION"` en
/// `details_json`, en línea con el vocabulario de clock.md (TTR-001
/// `request_type`: `REAL | FAKE`; TTR-002 precondición: modo
/// `SIMULATION`).
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

/// Identidad provista por quien llama, compartida por todos los eventos
/// de auditoría del Clock (clock.md "Gobernanza y Estándares").
///
/// - `session_id` se convierte tanto en el `entity_id` del evento
///   (`entity_type = "CLOCK"`) como en su campo de catálogo `session_id`
///   del Grupo IV de ADR-0020 V2 — "el campo canónico para agrupar un
///   runtime" (TTR-002).
/// - `institutional_tag` (Grupo II) y `process_id` (Grupo IV) son
///   obligatorios según el perfil "Ops / Auditoría" (audit-log.md
///   TTR-001: "Toda entrada DEBE incluir `process_id` y
///   `institutional_tag`").
#[derive(Debug, Clone)]
pub struct ClockAuditContext<'a> {
    pub session_id: &'a str,
    pub institutional_tag: &'a str,
    pub process_id: &'a str,
}

impl ClockAuditContext<'_> {
    /// Construye los campos de catálogo "Ops / Auditoría" de ADR-0020 V2
    /// compartidos por los tres eventos del Clock, dejando que cada
    /// función `emit_*` complete `action_type` y `details_json`.
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

/// Emite el evento `CLOCK_NTP_SYNC` (clock.md TTR-001 postcondición).
///
/// Se llama exactamente una vez, al iniciar, tras la verificación de
/// sincronización NTP — NUNCA en cada lectura de `timestamp_ns()`
/// (clock.md "Granularidad de Auditoría").
///
/// `ntp_sync_offset_ns` es el delta NTP medido (ADR-0013), que viaja como
/// payload opaco en `details_json`: `{"ntp_sync_offset_ns": <i64>}`. NO es
/// un campo de catálogo de ADR-0020 V2.
pub async fn emit_ntp_sync(
    repo: &AuditLogRepository<'_>,
    ctx: &ClockAuditContext<'_>,
    ntp_sync_offset_ns: i64,
) -> Result<AuditEvent, AuditLogError> {
    let details_json = json!({ "ntp_sync_offset_ns": ntp_sync_offset_ns }).to_string();
    let content = ctx.base_content("CLOCK_NTP_SYNC", details_json);
    repo.append(content).await
}

/// Emite el evento `CLOCK_MODE_TRANSITION` (clock.md "Granularidad de
/// Auditoría").
///
/// Se llama en cada transición `REAL` <-> `SIMULATION` — NUNCA en cada
/// llamada a `advance(ns)`/`tick()`.
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

/// Emite el evento `CLOCK_SESSION_CLOSE` (clock.md TTR-002 postcondición).
///
/// Se llama exactamente una vez, cuando cierra una sesión de simulación —
/// NUNCA en cada `advance(ns)`.
///
/// `virtual_process_id` es el identificador de proceso virtual de la
/// simulación (TTR-002: "El identificador del proceso virtual de la
/// simulación viaja como payload"). `real_virtual_delta_ns` es el delta
/// acumulado entre tiempo real y virtual. Ninguno de los dos es un campo
/// de catálogo de ADR-0020 V2; ambos viajan en `details_json`:
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

    /// `CLOCK_NTP_SYNC` se persiste con los campos de catálogo correctos
    /// y un payload `details_json` de exactamente
    /// `{"ntp_sync_offset_ns": <i64>}`.
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

    /// `CLOCK_MODE_TRANSITION` se persiste con las cadenas de modo
    /// `from`/`to` en `details_json`.
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

    /// `CLOCK_SESSION_CLOSE` se persiste con el id de proceso virtual y el
    /// delta acumulado real/virtual en `details_json`.
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

    /// CRITERIO DE CIERRE (a)+(b): emitir los tres eventos del Clock uno
    /// tras otro produce una cadena que [`verify_chain`] reporta como
    /// [`ChainVerificationResult::Valid`] — el rastro de auditoría del
    /// Clock no rompe la cadena de hashes existente.
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

    /// CRITERIO DE CIERRE (c): los tres eventos del Clock se pueden
    /// recuperar vía `events_for_entity("CLOCK", session_id)`, ordenados
    /// por `event_sequence_id`, con su `action_type` respectivo.
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

    /// `events_for_entity` acotado a un `session_id` distinto no devuelve
    /// nada — `entity_id` rastrea correctamente la sesión activa.
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
