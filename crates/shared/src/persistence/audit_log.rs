//! [SHELL] Append-only repository for the Audit Log
//! (`docs/features/audit-log.md` TTR-001, ADR-0006, ADR-0020 V2).
//!
//! Wraps the `audit_events` table (migration `0002_audit_log.sql`). Owns
//! the only I/O for the audit log: SQLite reads/writes, UUID generation
//! (unseeded randomness, ADR-0002/0004) and the [`Clock`] port read. The
//! actual hash-chain construction and verification are pure core logic in
//! [`crate::domain::audit_log`] — this module only feeds it injected
//! inputs (`id`, `created_at_ns`) and persists/loads the result.
//!
//! Append-only is enforced twice:
//! - **Database**: migration `0002_audit_log.sql` installs `BEFORE UPDATE`
//!   / `BEFORE DELETE` triggers on `audit_events` that `RAISE(ABORT, ...)`.
//! - **Application**: this repository exposes [`AuditLogRepository::append`]
//!   and [`AuditLogRepository::load_chain`]/[`AuditLogRepository::events_for_entity`]
//!   only — no `update`/`delete` method exists.

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::audit_log::{chain_event, AuditEvent, AuditEventContent};
use crate::domain::clock::Clock;

/// Errors returned by [`AuditLogRepository`] operations.
#[derive(Debug)]
pub enum AuditLogError {
    /// The underlying SQLite operation failed.
    Database(sqlx::Error),
}

impl std::fmt::Display for AuditLogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditLogError::Database(err) => write!(f, "audit log database error: {err}"),
        }
    }
}

impl std::error::Error for AuditLogError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AuditLogError::Database(err) => Some(err),
        }
    }
}

impl From<sqlx::Error> for AuditLogError {
    fn from(err: sqlx::Error) -> Self {
        AuditLogError::Database(err)
    }
}

/// Append-only repository for `audit_events`.
///
/// Construct with a migrated [`SqlitePool`] (see
/// [`crate::persistence::pool::connect`] +
/// [`crate::persistence::pool::migrate`]) and any [`Clock`] implementation
/// (production: [`crate::orchestrator::SystemClock`]; tests/backtests:
/// [`crate::domain::clock::DeterministicClock`]).
pub struct AuditLogRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> AuditLogRepository<'a> {
    /// Creates a repository bound to `pool` and `clock`. Both are borrowed
    /// for the lifetime of the repository — no ownership is taken, so the
    /// same pool/clock can be shared with other repositories.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Appends a new event to the chain and persists it.
    ///
    /// Reads the current chain tail (highest `event_sequence_id`), builds
    /// the next [`AuditEvent`] via [`chain_event`] (pure core logic) using
    /// a freshly generated UUID v4 (`id`, unseeded randomness — confined to
    /// this shell per ADR-0002/0004) and the current [`Clock`] reading
    /// (`created_at_ns`), then inserts it.
    ///
    /// Returns the persisted [`AuditEvent`], including its computed
    /// `audit_hash` and `audit_chain_hash` (TTR-001 "Salida": `log_id`,
    /// `audit_hash`).
    pub async fn append(&self, content: AuditEventContent) -> Result<AuditEvent, AuditLogError> {
        let previous = self.load_tail().await?;

        let id = Uuid::new_v4().to_string();
        let created_at_ns = self.clock.timestamp_ns();

        let event = chain_event(id, created_at_ns, content, previous.as_ref());

        sqlx::query(
            "INSERT INTO audit_events (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, manifest_id, access_token_id, \
                process_id, session_id, node_id, \
                action_type, entity_type, entity_id, details_json\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&event.id)
        .bind(event.created_at_ns)
        .bind(event.updated_at_ns)
        .bind(&event.audit_hash)
        .bind(&event.audit_chain_hash)
        .bind(event.event_sequence_id)
        .bind(&event.content.owner_id)
        .bind(&event.content.institutional_tag)
        .bind(&event.content.manifest_id)
        .bind(&event.content.access_token_id)
        .bind(&event.content.process_id)
        .bind(&event.content.session_id)
        .bind(&event.content.node_id)
        .bind(&event.content.action_type)
        .bind(&event.content.entity_type)
        .bind(&event.content.entity_id)
        .bind(&event.content.details_json)
        .execute(self.pool)
        .await?;

        Ok(event)
    }

    /// Loads the most recently appended event (highest `event_sequence_id`),
    /// or `None` if the chain is empty (next [`append`](Self::append) call
    /// will create the genesis event).
    pub async fn load_tail(&self) -> Result<Option<AuditEvent>, AuditLogError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, manifest_id, access_token_id, \
                    process_id, session_id, node_id, \
                    action_type, entity_type, entity_id, details_json \
             FROM audit_events \
             ORDER BY event_sequence_id DESC \
             LIMIT 1",
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(row.map(row_to_event))
    }

    /// Loads the entire chain, ordered by ascending `event_sequence_id`
    /// (genesis first). Intended for [`crate::domain::audit_log::verify_chain`].
    pub async fn load_chain(&self) -> Result<Vec<AuditEvent>, AuditLogError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, manifest_id, access_token_id, \
                    process_id, session_id, node_id, \
                    action_type, entity_type, entity_id, details_json \
             FROM audit_events \
             ORDER BY event_sequence_id ASC",
        )
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter().map(row_to_event).collect())
    }

    /// Loads every event for a given `(entity_type, entity_id)`, ordered by
    /// ascending `event_sequence_id` (audit-log.md: "¿qué pasó con la
    /// estrategia XYZ el 2026-04-07?").
    pub async fn events_for_entity(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<AuditEvent>, AuditLogError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, manifest_id, access_token_id, \
                    process_id, session_id, node_id, \
                    action_type, entity_type, entity_id, details_json \
             FROM audit_events \
             WHERE entity_type = ? AND entity_id = ? \
             ORDER BY event_sequence_id ASC",
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter().map(row_to_event).collect())
    }
}

/// Converts a `audit_events` row into the core [`AuditEvent`] type.
fn row_to_event(row: sqlx::sqlite::SqliteRow) -> AuditEvent {
    AuditEvent {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        event_sequence_id: row.get("event_sequence_id"),
        content: AuditEventContent {
            action_type: row.get("action_type"),
            entity_type: row.get("entity_type"),
            entity_id: row.get("entity_id"),
            details_json: row.get("details_json"),
            owner_id: row.get("owner_id"),
            institutional_tag: row.get("institutional_tag"),
            manifest_id: row.get("manifest_id"),
            access_token_id: row.get("access_token_id"),
            process_id: row.get("process_id"),
            session_id: row.get("session_id"),
            node_id: row.get("node_id"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::audit_log::{verify_chain, ChainVerificationResult};
    use crate::domain::clock::DeterministicClock;
    use crate::persistence::pool::{connect, migrate};

    fn sample_content(action_type: &str, entity_id: &str) -> AuditEventContent {
        AuditEventContent {
            action_type: action_type.to_string(),
            entity_type: "ORDER".to_string(),
            entity_id: entity_id.to_string(),
            details_json: "{\"from\":\"NEW\",\"to\":\"FILLED\"}".to_string(),
            owner_id: Some("owner-1".to_string()),
            institutional_tag: "BACKTEST".to_string(),
            manifest_id: None,
            access_token_id: None,
            process_id: "process-1".to_string(),
            session_id: Some("session-1".to_string()),
            node_id: None,
        }
    }

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("connect in-memory db");
        migrate(&pool).await.expect("apply migrations");
        pool
    }

    /// Appending events assigns increasing `event_sequence_id`s starting at
    /// 1, links each event's `audit_chain_hash` to the previous event's
    /// `audit_hash`, and the persisted chain verifies as
    /// [`ChainVerificationResult::Valid`].
    #[tokio::test]
    async fn append_builds_a_valid_chain() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AuditLogRepository::new(&pool, &clock);

        let first = repo
            .append(sample_content("ORDER_STATE_CHANGE", "order-1"))
            .await
            .expect("append first event");
        assert_eq!(first.event_sequence_id, 1);
        assert_eq!(first.audit_chain_hash, None);

        clock.tick();
        let second = repo
            .append(sample_content("USER_VETO", "order-1"))
            .await
            .expect("append second event");
        assert_eq!(second.event_sequence_id, 2);
        assert_eq!(second.audit_chain_hash, Some(first.audit_hash.clone()));

        let chain = repo.load_chain().await.expect("load chain");
        assert_eq!(chain.len(), 2);
        assert_eq!(verify_chain(&chain), ChainVerificationResult::Valid);
    }

    /// CLOSING CRITERION: an attempt to mutate a historical event is
    /// rejected at the database level (append-only trigger, migration
    /// 0002) AND, if a row were altered out-of-band, [`verify_chain`]
    /// would detect the break (covered in
    /// `crate::domain::audit_log::tests::verify_chain_detects_mutation_of_historical_event`).
    #[tokio::test]
    async fn update_on_audit_events_is_rejected_by_append_only_trigger() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AuditLogRepository::new(&pool, &clock);

        let event = repo
            .append(sample_content("ORDER_STATE_CHANGE", "order-1"))
            .await
            .expect("append event");

        // Attempt to mutate the historical event's details directly via
        // SQL (bypassing the repository, which exposes no update method).
        let result = sqlx::query("UPDATE audit_events SET details_json = ? WHERE id = ?")
            .bind("{\"tampered\":true}")
            .bind(&event.id)
            .execute(&pool)
            .await;

        assert!(
            result.is_err(),
            "UPDATE on audit_events must be rejected by the append-only trigger"
        );

        // The stored row is unchanged.
        let chain = repo.load_chain().await.expect("load chain");
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].content.details_json, "{\"from\":\"NEW\",\"to\":\"FILLED\"}");
    }

    /// CLOSING CRITERION: an attempt to delete a historical event is
    /// rejected at the database level (append-only trigger, migration
    /// 0002).
    #[tokio::test]
    async fn delete_on_audit_events_is_rejected_by_append_only_trigger() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AuditLogRepository::new(&pool, &clock);

        let event = repo
            .append(sample_content("ORDER_STATE_CHANGE", "order-1"))
            .await
            .expect("append event");

        let result = sqlx::query("DELETE FROM audit_events WHERE id = ?")
            .bind(&event.id)
            .execute(&pool)
            .await;

        assert!(
            result.is_err(),
            "DELETE on audit_events must be rejected by the append-only trigger"
        );

        let chain = repo.load_chain().await.expect("load chain");
        assert_eq!(chain.len(), 1);
    }

    /// `events_for_entity` returns only events for the requested
    /// `(entity_type, entity_id)`, ordered by `event_sequence_id`
    /// (audit-log.md: "¿qué pasó con la estrategia XYZ ...?").
    #[tokio::test]
    async fn events_for_entity_filters_and_orders() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AuditLogRepository::new(&pool, &clock);

        repo.append(sample_content("ORDER_STATE_CHANGE", "order-1"))
            .await
            .expect("append 1");
        clock.tick();
        repo.append(sample_content("ANOMALY_DETECTED", "order-2"))
            .await
            .expect("append 2");
        clock.tick();
        repo.append(sample_content("USER_VETO", "order-1"))
            .await
            .expect("append 3");

        let events = repo
            .events_for_entity("ORDER", "order-1")
            .await
            .expect("query by entity");

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].content.action_type, "ORDER_STATE_CHANGE");
        assert_eq!(events[1].content.action_type, "USER_VETO");
        assert!(events[0].event_sequence_id < events[1].event_sequence_id);
    }
}
