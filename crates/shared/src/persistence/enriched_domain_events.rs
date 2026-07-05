//! [SHELL] Repositorio de persistencia APPEND-ONLY ATÓMICO para los Eventos
//! de Dominio Enriquecidos (`docs/features/enriched-domain-events.md`,
//! ADR-0144 cimiento #6, ADR-0145, ADR-0141, ADR-0020, ADR-0093,
//! migración `0012_domain_events.sql`, STORY-033).
//!
//! Envuelve la tabla `domain_events`. Dueño del único I/O de este
//! cimiento: lecturas/escrituras en SQLite, generación de UUIDv7
//! (ADR-0141) y la lectura del puerto [`Clock`]. La lógica pura (catálogo
//! de eventos, serialización canónica del payload, hash encadenado) vive
//! en [`crate::domain::enriched_domain_events`] -- este módulo solo le da
//! entradas inyectadas y persiste el resultado, reflejando el patrón de
//! [`crate::persistence::consent_registry::ConsentRepository`] (misma
//! naturaleza APPEND-ONLY: `event_sequence_id UNIQUE`, sin `row_version`).
//!
//! ## Por qué NO existe `update`/`delete` en esta API
//!
//! A propósito: la única operación de escritura que este repositorio
//! expone es [`DomainEventRepository::record_event`] (un INSERT). No hay
//! ningún método de actualización o borrado -- ni falta, porque los
//! triggers `trg_domain_events_no_update`/`trg_domain_events_no_delete` de
//! la migración los rechazarían de cualquier forma. La ausencia del método
//! en Rust es la primera línea de defensa; el trigger de SQLite es la
//! segunda (defensa en profundidad).

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::audit_log::GENESIS_PREVIOUS_HASH;
use crate::domain::clock::Clock;
use crate::domain::enriched_domain_events::{compute_event_audit_hash, EnrichedDomainEvent};

/// Errores que devuelven las operaciones de [`DomainEventRepository`].
#[derive(Debug, thiserror::Error)]
pub enum DomainEventRepositoryError {
    /// La operación subyacente de SQLite falló.
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// El append no pudo completarse tras agotar los reintentos ante
    /// contención de escritura transitoria (otro escritor mantuvo el lock
    /// de la base de datos más allá del `busy_timeout`, o hubo colisión
    /// repetida al derivar `event_sequence_id`). El evento NO se descartó
    /// en silencio -- se propaga este error tipado para que el llamador
    /// decida reintentar a un nivel superior o alertar (`docs/features/
    /// enriched-domain-events.md`, regla "Atomicidad de ledgers append-only").
    #[error("no se pudo registrar el evento tras {attempts} intentos por contención de escritura")]
    WriteContention { attempts: u32 },
}

/// Número máximo de intentos del append ante contención de escritura
/// transitoria antes de rendirse con [`DomainEventRepositoryError::WriteContention`].
/// Cinco es holgado: con `busy_timeout` de 5s (ADR-0141 R2) el lock casi
/// siempre se obtiene sin reintentar; el bucle solo actúa si el
/// `busy_timeout` expira bajo una contención extrema.
const MAX_RECORD_ATTEMPTS: u32 = 5;

/// Decide si un error del repositorio es una contención de escritura
/// TRANSITORIA -- es decir, algo que reintentar (re-derivando el
/// `event_sequence_id` y reinsertando) puede resolver, sin descartar el
/// evento.
///
/// Dos causas transitorias:
/// - `SQLITE_BUSY` / `SQLITE_LOCKED`: otro escritor tenía el lock de la
///   base de datos cuando esta conexión intentó tomarlo. El driver de
///   SQLite reporta estos con los mensajes canónicos "database is locked"
///   / "database table is locked" (ver el `Display` de `SqliteError` en
///   sqlx) -- son el criterio robusto, independiente del código primario
///   vs. extendido.
/// - Violación de UNIQUE sobre `event_sequence_id`: dos escritores
///   derivaron la misma posición de secuencia. Con `BEGIN IMMEDIATE` esto
///   no debería ocurrir (los escritores se serializan), pero se trata como
///   transitorio de cinturón-y-tirantes: re-derivar el MAX y reinsertar lo
///   resuelve. Cualquier OTRA violación de UNIQUE (p. ej. el `id`) NO es
///   transitoria y NO se reintenta.
fn is_transient_write_conflict(error: &DomainEventRepositoryError) -> bool {
    let DomainEventRepositoryError::Database(sqlx::Error::Database(db)) = error else {
        return false;
    };

    let message = db.message().to_lowercase();
    // Lock ocupado: otro escritor tenía el lock de la BD / de la tabla.
    if message.contains("database is locked") || message.contains("database table is locked") {
        return true;
    }

    // Colisión de secuencia: mismo event_sequence_id derivado por dos
    // escritores -- transitorio, re-derivar y reinsertar lo arregla.
    db.is_unique_violation() && message.contains("event_sequence_id")
}

/// Entrada para [`DomainEventRepository::record_event`] -- todo lo que la
/// Shell necesita para registrar UN evento de dominio: el evento en sí
/// (del Core), la identidad del dueño/máquina/proceso (Perfil D de
/// ADR-0020) y la decisión de replicación ya derivada del `ExecutionGate`.
#[derive(Debug, Clone)]
pub struct RecordDomainEventInput {
    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,
    pub process_id: String,
    /// Sesión de ejecución que agrupa el evento (nullable -- no todo evento
    /// ocurre dentro de una sesión agrupable).
    pub session_id: Option<String>,
    pub event: EnrichedDomainEvent,
    /// Decisión de replicación hacia la Cabina de Mando, ya derivada del
    /// `ExecutionGate` real por el orchestrator (`decide_replication`).
    pub replicate: bool,
}

/// Una fila de `domain_events` ya persistida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainEventRow {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub event_sequence_id: i64,

    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,
    pub process_id: String,
    pub session_id: Option<String>,

    pub event_type: String,
    pub payload: String,
    pub replicate: bool,
}

/// Repositorio APPEND-ONLY para `domain_events`.
///
/// Constrúyelo con un [`SqlitePool`] ya migrado y cualquier implementación
/// de [`Clock`] -- mismo patrón que
/// [`crate::persistence::consent_registry::ConsentRepository`].
pub struct DomainEventRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> DomainEventRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`. Ambos se toman
    /// prestados por la vida del repositorio -- no se toma ownership.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Registra UN evento de dominio: deriva su posición en la cadena
    /// GLOBAL, computa su hash encadenado y lo persiste como fila nueva.
    ///
    /// Es la ÚNICA forma de escribir en `domain_events` -- no existe
    /// `update`/`delete` en esta API (ver doc-comment del módulo).
    ///
    /// ## Atomicidad bajo concurrencia (regla "Atomicidad de ledgers append-only")
    ///
    /// Todo el *read-then-write* (leer el MAX(`event_sequence_id`) y el
    /// `audit_hash` previo para encadenar, y el `INSERT` final) ocurre
    /// dentro de UNA sola transacción `BEGIN IMMEDIATE` -- ver
    /// [`Self::try_record_event_once`]. Sin esa transacción, dos escritores
    /// concurrentes derivarían el mismo `event_sequence_id`, el `UNIQUE`
    /// rechazaría a uno y su evento se PERDERÍA. Ante contención transitoria
    /// (`SQLITE_BUSY` tras expirar el `busy_timeout`, o colisión de
    /// secuencia), se reintenta hasta [`MAX_RECORD_ATTEMPTS`] veces
    /// re-derivando la secuencia; el evento NUNCA se descarta en silencio
    /// (si se agotan los reintentos se devuelve
    /// [`DomainEventRepositoryError::WriteContention`]).
    pub async fn record_event(
        &self,
        input: RecordDomainEventInput,
    ) -> Result<DomainEventRow, DomainEventRepositoryError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_record_event_once(&input).await {
                Ok(row) => return Ok(row),
                Err(error) => {
                    // Solo se reintenta ante contención de escritura
                    // transitoria; cualquier otro error se propaga de
                    // inmediato.
                    if is_transient_write_conflict(&error) {
                        if attempt < MAX_RECORD_ATTEMPTS {
                            continue;
                        }
                        // Agotados los reintentos: error tipado, NUNCA
                        // pérdida silenciosa del evento.
                        return Err(DomainEventRepositoryError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    /// Un intento único del append, dentro de una transacción
    /// `BEGIN IMMEDIATE`. Devuelve el error de SQLite tal cual si algo falla
    /// -- el bucle de [`Self::record_event`] decide si es transitorio y hay
    /// que reintentar. La transacción se abre con `BEGIN IMMEDIATE` (no el
    /// `BEGIN` DEFERRED por defecto de SQLx) para tomar el lock de escritura
    /// de ENTRADA: así ningún otro escritor puede intercalar entre la
    /// lectura del MAX(`event_sequence_id`) y el `INSERT`, y se evita además
    /// el interbloqueo de upgrade que ocurriría si dos transacciones
    /// DEFERRED intentaran subir de lectura a escritura a la vez.
    async fn try_record_event_once(
        &self,
        input: &RecordDomainEventInput,
    ) -> Result<DomainEventRow, DomainEventRepositoryError> {
        // Abre la transacción tomando el lock de escritura de inmediato.
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        // Lectura (DENTRO de la transacción) -- posición en la cadena
        // GLOBAL: la fila con el event_sequence_id más alto de TODA la
        // tabla, para asignar la siguiente y encadenar su audit_hash.
        let tail_row = sqlx::query(
            "SELECT audit_hash, event_sequence_id \
             FROM domain_events \
             ORDER BY event_sequence_id DESC \
             LIMIT 1",
        )
        .fetch_optional(&mut *tx)
        .await?;

        let (event_sequence_id, audit_chain_hash, previous_audit_hash) = match tail_row {
            Some(row) => {
                let previous_seq: i64 = row.get("event_sequence_id");
                let previous_hash: String = row.get("audit_hash");
                (previous_seq + 1, Some(previous_hash.clone()), previous_hash)
            }
            None => (1, None, GENESIS_PREVIOUS_HASH.to_string()),
        };

        // Núcleo puro: serializa el payload canónico determinista del evento.
        let event_type = input.event.event_type();
        let payload = input.event.canonical_payload_json();

        let id = Uuid::now_v7().to_string();
        // Reloj INYECTADO -- nunca SystemTime::now() directo (ADR-0002/0004).
        // En este repositorio created_at y updated_at usan la misma lectura
        // del reloj: es una tabla append-only, la fila nunca se modifica.
        let now_ns = self.clock.timestamp_ns();

        let audit_hash = compute_event_audit_hash(
            &id,
            now_ns,
            event_sequence_id,
            &previous_audit_hash,
            &input.owner_id,
            &input.institutional_tag,
            &input.node_id,
            &input.process_id,
            input.session_id.as_deref(),
            event_type,
            &payload,
            input.replicate,
        );

        // Escritura (DENTRO de la transacción) -- el INSERT que cierra el
        // read-then-write atómico. `replicate` se persiste como 0/1.
        sqlx::query(
            "INSERT INTO domain_events (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, process_id, session_id, \
                event_type, payload, replicate\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&audit_chain_hash)
        .bind(event_sequence_id)
        .bind(&input.owner_id)
        .bind(&input.institutional_tag)
        .bind(&input.node_id)
        .bind(&input.process_id)
        .bind(&input.session_id)
        .bind(event_type)
        .bind(&payload)
        .bind(i64::from(input.replicate))
        .execute(&mut *tx)
        .await?;

        // Confirma la transacción: recién aquí el lock de escritura se
        // libera y la fila se hace visible a otros escritores.
        tx.commit().await?;

        Ok(DomainEventRow {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash,
            event_sequence_id,
            owner_id: input.owner_id.clone(),
            institutional_tag: input.institutional_tag.clone(),
            node_id: input.node_id.clone(),
            process_id: input.process_id.clone(),
            session_id: input.session_id.clone(),
            event_type: event_type.to_string(),
            payload,
            replicate: input.replicate,
        })
    }

    /// Carga la cadena completa, ordenada por `event_sequence_id`
    /// ascendente -- usada por los tests de integridad de la cadena
    /// (génesis con `audit_chain_hash = NULL`, resto encadenado) y por
    /// cualquier consumidor futuro que reconstruya el historial.
    pub async fn load_chain(&self) -> Result<Vec<DomainEventRow>, DomainEventRepositoryError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, node_id, process_id, session_id, \
                    event_type, payload, replicate \
             FROM domain_events \
             ORDER BY event_sequence_id ASC",
        )
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter().map(row_to_domain_event).collect())
    }
}

/// Convierte una fila de `domain_events` al tipo [`DomainEventRow`],
/// mapeando `replicate` (0/1) de vuelta a `bool`.
fn row_to_domain_event(row: sqlx::sqlite::SqliteRow) -> DomainEventRow {
    let replicate_int: i64 = row.get("replicate");

    DomainEventRow {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        event_sequence_id: row.get("event_sequence_id"),
        owner_id: row.get("owner_id"),
        institutional_tag: row.get("institutional_tag"),
        node_id: row.get("node_id"),
        process_id: row.get("process_id"),
        session_id: row.get("session_id"),
        event_type: row.get("event_type"),
        payload: row.get("payload"),
        // La columna tiene CHECK(replicate IN (0,1)); cualquier valor
        // distinto de 0 se trata como true, pero el CHECK garantiza que
        // solo lleguen 0 o 1.
        replicate: replicate_int != 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::domain::enriched_domain_events::{
        CapitalFlowPayload, CapitalFlowSign, OrderExecutedPayload, OrderSide,
    };
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    fn capital_flow_event(account_id: &str, amount: i64) -> EnrichedDomainEvent {
        EnrichedDomainEvent::CapitalFlow(CapitalFlowPayload {
            account_id: account_id.to_string(),
            sign: CapitalFlowSign::Deposit,
            amount,
            currency: "USD".to_string(),
            timestamp_ns: 1_000,
        })
    }

    fn record_input(event: EnrichedDomainEvent, replicate: bool) -> RecordDomainEventInput {
        RecordDomainEventInput {
            owner_id: "owner-1".to_string(),
            institutional_tag: "DRASUS_LOCAL".to_string(),
            node_id: "node-1".to_string(),
            process_id: "process-1".to_string(),
            session_id: Some("session-1".to_string()),
            event,
            replicate,
        }
    }

    // ── CRITERIO #1 (Orden §5): esquema STRICT append-only + Grupo I ────────

    #[tokio::test]
    async fn migration_creates_domain_events_table_strict_with_group_i_and_event_sequence_id() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('domain_events')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();

        for expected in [
            "id", "created_at", "updated_at", "audit_hash", "audit_chain_hash", "event_sequence_id",
            "owner_id", "institutional_tag", "node_id", "process_id", "session_id",
            "event_type", "payload", "replicate",
        ] {
            assert!(column_names.contains(&expected.to_string()), "falta la columna: {expected}");
        }

        assert!(
            !column_names.contains(&"row_version".to_string()),
            "domain_events es APPEND-ONLY (ADR-0141): no debe tener row_version, solo event_sequence_id"
        );

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'domain_events'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(sql.contains("STRICT"), "la tabla domain_events debe declararse STRICT");
    }

    // ── CRITERIO #6 (Orden §5): append-only -- UPDATE/DELETE rechazados ─────

    /// CRITERIO DE CIERRE: un `UPDATE` sobre `domain_events` es rechazado
    /// por el trigger de la migración -- si el trigger no existiera, esta
    /// prueba fallaría con `Ok`.
    #[tokio::test]
    async fn update_is_rejected_by_trigger() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = DomainEventRepository::new(&pool, &clock);

        let row = repo
            .record_event(record_input(capital_flow_event("acc-1", 100_000_000_000), true))
            .await
            .expect("registrar evento");

        let result = sqlx::query("UPDATE domain_events SET replicate = 0 WHERE id = ?")
            .bind(&row.id)
            .execute(&pool)
            .await;

        assert!(result.is_err(), "UPDATE sobre domain_events debe ser rechazado por el trigger");
    }

    /// CRITERIO DE CIERRE: un `DELETE` sobre `domain_events` es rechazado
    /// por el trigger de la migración.
    #[tokio::test]
    async fn delete_is_rejected_by_trigger() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = DomainEventRepository::new(&pool, &clock);

        let row = repo
            .record_event(record_input(capital_flow_event("acc-1", 100_000_000_000), true))
            .await
            .expect("registrar evento");

        let result = sqlx::query("DELETE FROM domain_events WHERE id = ?")
            .bind(&row.id)
            .execute(&pool)
            .await;

        assert!(result.is_err(), "DELETE sobre domain_events debe ser rechazado por el trigger");
    }

    // ── CRITERIO #6 (Orden §5): event_sequence_id monótono y UNIQUE ─────────

    #[tokio::test]
    async fn event_sequence_id_is_monotonic_across_inserts() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = DomainEventRepository::new(&pool, &clock);

        let first = repo.record_event(record_input(capital_flow_event("acc-1", 100), true)).await.expect("primero");
        clock.tick();
        let second = repo.record_event(record_input(capital_flow_event("acc-2", 200), true)).await.expect("segundo");
        clock.tick();
        let third = repo.record_event(record_input(capital_flow_event("acc-1", 300), true)).await.expect("tercero");

        assert_eq!(first.event_sequence_id, 1);
        assert_eq!(second.event_sequence_id, 2);
        assert_eq!(third.event_sequence_id, 3);
    }

    /// CRITERIO DE CIERRE: duplicar una posición ya usada es rechazado por
    /// el `UNIQUE` de la migración -- se inserta directamente con SQL crudo
    /// para ejercitar el guardarraíl de la BD en sí mismo.
    #[tokio::test]
    async fn duplicating_event_sequence_id_is_rejected_by_unique_constraint() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = DomainEventRepository::new(&pool, &clock);

        repo.record_event(record_input(capital_flow_event("acc-1", 100), true))
            .await
            .expect("primer evento (event_sequence_id = 1)");

        let duplicate = sqlx::query(
            "INSERT INTO domain_events (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, process_id, session_id, \
                event_type, payload, replicate\
            ) VALUES ('id-dup', 0, 0, 'hash', NULL, 1, 'owner-1', 'DRASUS_LOCAL', 'node-1', 'process-1', NULL, \
                       'CAPITAL_FLOW', '{}', 1)",
        )
        .execute(&pool)
        .await;

        assert!(duplicate.is_err(), "duplicar event_sequence_id=1 debe ser rechazado por UNIQUE");
    }

    // ── CHECK de event_type y json_valid en la BD ────────────────────────────

    /// CRITERIO DE CIERRE: un `event_type` fuera del catálogo es rechazado
    /// por el `CHECK` de la BD.
    #[tokio::test]
    async fn database_check_rejects_unknown_event_type() {
        let pool = migrated_pool().await;

        let result = sqlx::query(
            "INSERT INTO domain_events (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, process_id, session_id, \
                event_type, payload, replicate\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', 'DRASUS_LOCAL', 'node-1', 'process-1', NULL, \
                       'UNKNOWN_EVENT', '{}', 1)",
        )
        .execute(&pool)
        .await;

        assert!(result.is_err(), "un event_type fuera del catálogo debe ser rechazado por el CHECK de la BD");
    }

    /// CRITERIO DE CIERRE: `CHECK (json_valid(payload))` rechaza JSON
    /// corrupto -- si el CHECK no existiera, el INSERT tendría éxito con
    /// basura en la columna.
    #[tokio::test]
    async fn database_check_rejects_invalid_json_in_payload() {
        let pool = migrated_pool().await;

        let result = sqlx::query(
            "INSERT INTO domain_events (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, process_id, session_id, \
                event_type, payload, replicate\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', 'DRASUS_LOCAL', 'node-1', 'process-1', NULL, \
                       'CAPITAL_FLOW', '{not valid json', 1)",
        )
        .execute(&pool)
        .await;

        assert!(result.is_err(), "payload con JSON corrupto debe ser rechazado por el CHECK(json_valid) de la BD");
    }

    // ── json_valid del payload real que produce el Core ─────────────────────

    /// El payload que el Core serializa (`canonical_payload_json`) pasa el
    /// `CHECK(json_valid(payload))` -- se persiste sin error, cerrando el
    /// lazo Core -> Shell -> BD.
    #[tokio::test]
    async fn core_payload_is_valid_json_and_persists() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = DomainEventRepository::new(&pool, &clock);

        let event = EnrichedDomainEvent::OrderExecuted(OrderExecutedPayload {
            instrument_id: "BTCUSDT".to_string(),
            side: OrderSide::Sell,
            quantity: 250_000_000,
            price: 4_000_000_000_000,
            slippage: -1_000_000,
            fill_time_ns: 1_000,
            broker: "IBKR".to_string(),
            notional: 10_000_000_000_000,
            account_id: "acc-1".to_string(),
            realized_pnl: 500_000_000,
            mae: -200_000_000,
            mfe: 800_000_000,
            duration_ns: 3_600_000_000_000,
        });

        let row = repo
            .record_event(record_input(event, false))
            .await
            .expect("el payload del Core debe ser JSON válido y persistir");

        assert_eq!(row.event_type, "ORDER_EXECUTED");
        assert!(!row.replicate, "replicate false se persiste como 0 y se lee de vuelta como false");

        // El payload releído de la BD debe parsear como JSON.
        let reloaded = repo.load_chain().await.expect("cargar cadena");
        assert_eq!(reloaded.len(), 1);
        assert!(serde_json::from_str::<serde_json::Value>(&reloaded[0].payload).is_ok());
    }

    // ── CRITERIO #6 (Orden §5): audit_chain_hash génesis NULL + encadenado ──

    /// CRITERIO DE CIERRE: la primera fila (génesis) tiene
    /// `audit_chain_hash = NULL`; las siguientes encadenan al `audit_hash`
    /// de la fila anterior.
    #[tokio::test]
    async fn audit_chain_hash_is_null_only_in_genesis_row_and_chains_afterwards() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = DomainEventRepository::new(&pool, &clock);

        let first = repo.record_event(record_input(capital_flow_event("acc-1", 100), true)).await.expect("génesis");
        clock.tick();
        let second = repo.record_event(record_input(capital_flow_event("acc-2", 200), true)).await.expect("segundo");
        clock.tick();
        let third = repo.record_event(record_input(capital_flow_event("acc-1", 300), true)).await.expect("tercero");

        assert_eq!(first.audit_chain_hash, None, "la fila génesis debe tener audit_chain_hash NULL");
        assert_eq!(second.audit_chain_hash, Some(first.audit_hash.clone()), "debe encadenar a la primera");
        assert_eq!(third.audit_chain_hash, Some(second.audit_hash.clone()), "debe encadenar a la segunda");
    }

    // ── CRITERIO #2 (Orden §5): append atómico + concurrencia ────────────────

    /// CRITERIO DE CIERRE (DEBT-001): N escritores concurrentes sobre el
    /// MISMO pool/ledger. La transacción `BEGIN IMMEDIATE` + reintento
    /// acotado debe garantizar que NINGÚN evento se pierde y que la
    /// secuencia queda densa (1..=N sin huecos ni duplicados) con la cadena
    /// de hashes íntegra y recomputable.
    ///
    /// Esta prueba DEBE poder caerse si se quita la transacción: con el
    /// `SELECT MAX(...)` y el `INSERT` en sentencias sueltas, dos tareas
    /// leen el mismo MAX, derivan el mismo `event_sequence_id`, el `UNIQUE`
    /// rechaza a una y su fila se pierde -> la aserción (a) `chain.len()==N`
    /// o la (b) `1..=N` fallaría. Se usa una BD en ARCHIVO temporal (nunca
    /// `:memory:`, donde cada conexión sería una base distinta) para que la
    /// concurrencia entre conexiones sea real.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_record_events_persist_every_event_without_gaps_or_lost_rows() {
        use std::sync::Arc;

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("domain_events_concurrency.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());
        let pool = connect(&database_url).await.expect("conectar");
        migrate(&pool).await.expect("migrar");

        // Reloj compartido (atómico, thread-safe). No se hace `tick`: todas
        // las filas comparten timestamp, lo cual es válido -- el orden lo
        // fija `event_sequence_id`, no el reloj.
        let clock: Arc<DeterministicClock> = Arc::new(DeterministicClock::new(1_000, 100));

        const N: i64 = 16;

        // Lanza N tareas en paralelo, cada una registrando un evento de
        // flujo de capital distinto.
        let mut handles = Vec::new();
        for i in 0..N {
            let pool_c = pool.clone(); // SqlitePool es un Arc interno: clonar es barato.
            let clock_c = clock.clone();
            handles.push(tokio::spawn(async move {
                let repo = DomainEventRepository::new(&pool_c, clock_c.as_ref());
                repo.record_event(RecordDomainEventInput {
                    owner_id: "owner-concurrente".to_string(),
                    institutional_tag: "DRASUS_LOCAL".to_string(),
                    node_id: "node-1".to_string(),
                    process_id: "process-1".to_string(),
                    session_id: None,
                    event: capital_flow_event(&format!("acc-{i}"), (i + 1) * 100_000_000),
                    replicate: true,
                })
                .await
            }));
        }

        // (a) TODAS las tareas terminaron OK -- ningún evento se perdió por
        // colisión de secuencia.
        for handle in handles {
            handle
                .await
                .expect("la tarea no debe entrar en panic")
                .expect("record_event debe tener éxito para cada escritor concurrente");
        }

        let repo = DomainEventRepository::new(&pool, clock.as_ref());
        let chain = repo.load_chain().await.expect("cargar la cadena completa");

        // (a) se persistieron TODAS las N filas.
        assert_eq!(chain.len() as i64, N, "deben persistirse las N filas, sin ninguna pérdida");

        // (b) los event_sequence_id son exactamente 1..=N (densa, sin huecos
        // ni duplicados). `load_chain` ya ordena ascendente por la columna.
        let sequence_ids: Vec<i64> = chain.iter().map(|row| row.event_sequence_id).collect();
        let expected: Vec<i64> = (1..=N).collect();
        assert_eq!(sequence_ids, expected, "los event_sequence_id deben ser 1..=N sin huecos ni duplicados");

        // (c) la cadena audit_chain_hash queda íntegra: génesis con NULL,
        // cada fila encadenada al audit_hash de la anterior, y cada
        // audit_hash recomputable (integridad de contenido completa).
        for (index, row) in chain.iter().enumerate() {
            let previous_audit_hash = if index == 0 {
                assert_eq!(row.audit_chain_hash, None, "la fila génesis debe tener audit_chain_hash NULL");
                GENESIS_PREVIOUS_HASH.to_string()
            } else {
                let prev = &chain[index - 1];
                assert_eq!(
                    row.audit_chain_hash.as_deref(),
                    Some(prev.audit_hash.as_str()),
                    "cada fila debe encadenar al audit_hash de la anterior"
                );
                prev.audit_hash.clone()
            };

            let recomputed = compute_event_audit_hash(
                &row.id,
                row.created_at_ns,
                row.event_sequence_id,
                &previous_audit_hash,
                &row.owner_id,
                &row.institutional_tag,
                &row.node_id,
                &row.process_id,
                row.session_id.as_deref(),
                &row.event_type,
                &row.payload,
                row.replicate,
            );
            assert_eq!(recomputed, row.audit_hash, "el audit_hash de cada fila debe ser recomputable (integridad de la cadena)");
        }
    }
}
