//! [SHELL] Repositorio de solo-apéndice (append-only) para el Audit Log
//! (`docs/features/audit-log.md` TTR-001, ADR-0006, ADR-0020).
//!
//! Envuelve la tabla `audit_events` (migración `0002_audit_log.sql`). Es el
//! único punto de I/O del audit log: lecturas/escrituras en SQLite,
//! generación de UUID (azar sin semilla, ADR-0002/0004) y lectura del
//! puerto [`Clock`]. La construcción y verificación real de la cadena de
//! hashes es lógica pura del núcleo, en [`crate::domain::audit_log`] —
//! este módulo solo le entrega las entradas inyectadas (`id`,
//! `created_at_ns`) y persiste/carga el resultado.
//!
//! El "solo-apéndice" se garantiza por dos vías:
//! - **Base de datos**: la migración `0002_audit_log.sql` instala
//!   triggers `BEFORE UPDATE` / `BEFORE DELETE` sobre `audit_events` que
//!   hacen `RAISE(ABORT, ...)`.
//! - **Aplicación**: este repositorio solo expone
//!   [`AuditLogRepository::append`] y
//!   [`AuditLogRepository::load_chain`]/[`AuditLogRepository::events_for_entity`]
//!   — no existe ningún método `update`/`delete`.

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::audit_log::{chain_event, AuditEvent, AuditEventContent};
use crate::domain::clock::Clock;

/// Errores que pueden devolver las operaciones de [`AuditLogRepository`].
#[derive(Debug)]
pub enum AuditLogError {
    /// La operación de SQLite subyacente falló.
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

/// Repositorio de solo-apéndice para `audit_events`.
///
/// Constrúyelo con un [`SqlitePool`] ya migrado (ver
/// [`crate::persistence::pool::connect`] +
/// [`crate::persistence::pool::migrate`]) y cualquier implementación de
/// [`Clock`] (producción: [`crate::orchestrator::SystemClock`];
/// tests/backtests: [`crate::domain::clock::DeterministicClock`]).
pub struct AuditLogRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> AuditLogRepository<'a> {
    /// Crea un repositorio enlazado a `pool` y `clock`. Ambos se piden en
    /// préstamo por la vida del repositorio — no toma posesión de ellos,
    /// así que el mismo pool/clock se puede compartir con otros
    /// repositorios.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Agrega un nuevo evento a la cadena y lo persiste.
    ///
    /// Lee la cola actual de la cadena (el `event_sequence_id` más alto),
    /// construye el siguiente [`AuditEvent`] vía [`chain_event`] (lógica
    /// pura del núcleo) usando un UUID v4 recién generado (`id`, azar sin
    /// semilla — confinado a esta cáscara según ADR-0002/0004) y la
    /// lectura actual del [`Clock`] (`created_at_ns`), y luego lo inserta.
    ///
    /// Devuelve el [`AuditEvent`] ya persistido, incluyendo su
    /// `audit_hash` y `audit_chain_hash` calculados (TTR-001 "Salida":
    /// `log_id`, `audit_hash`).
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

    /// Carga el evento agregado más recientemente (el `event_sequence_id`
    /// más alto), o `None` si la cadena está vacía (la siguiente llamada a
    /// [`append`](Self::append) creará el evento génesis).
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

    /// Carga la cadena completa, ordenada por `event_sequence_id`
    /// ascendente (el génesis primero). Pensado para alimentar
    /// [`crate::domain::audit_log::verify_chain`].
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

    /// Carga todos los eventos de un `(entity_type, entity_id)` dado,
    /// ordenados por `event_sequence_id` ascendente (audit-log.md: "¿qué
    /// pasó con la estrategia XYZ el 2026-04-07?").
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

/// Convierte una fila de `audit_events` en el tipo [`AuditEvent`] del núcleo.
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

    /// Agregar eventos asigna `event_sequence_id` crecientes empezando en
    /// 1, enlaza el `audit_chain_hash` de cada evento con el `audit_hash`
    /// del anterior, y la cadena persistida verifica como
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

    /// CRITERIO DE CIERRE: un intento de modificar un evento histórico se
    /// rechaza a nivel de base de datos (trigger de solo-apéndice,
    /// migración 0002) Y, si una fila se alterara por fuera de este
    /// camino, [`verify_chain`] detectaría la ruptura (cubierto en
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

        // Intenta modificar los detalles del evento histórico directamente
        // vía SQL (sin pasar por el repositorio, que no expone ningún
        // método de update).
        let result = sqlx::query("UPDATE audit_events SET details_json = ? WHERE id = ?")
            .bind("{\"tampered\":true}")
            .bind(&event.id)
            .execute(&pool)
            .await;

        assert!(
            result.is_err(),
            "UPDATE on audit_events must be rejected by the append-only trigger"
        );

        // La fila guardada queda sin cambios.
        let chain = repo.load_chain().await.expect("load chain");
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].content.details_json, "{\"from\":\"NEW\",\"to\":\"FILLED\"}");
    }

    /// CRITERIO DE CIERRE: un intento de borrar un evento histórico se
    /// rechaza a nivel de base de datos (trigger de solo-apéndice,
    /// migración 0002).
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

    /// `events_for_entity` devuelve solo los eventos del
    /// `(entity_type, entity_id)` solicitado, ordenados por
    /// `event_sequence_id` (audit-log.md: "¿qué pasó con la estrategia
    /// XYZ ...?").
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
