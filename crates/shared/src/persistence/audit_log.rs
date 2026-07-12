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
    /// El append no pudo completarse tras agotar los reintentos ante
    /// contención de escritura transitoria (otro escritor mantuvo el lock
    /// de la base de datos más allá del `busy_timeout`, o hubo colisión
    /// repetida al derivar `event_sequence_id`). El evento NO se descartó
    /// en silencio -- se propaga este error tipado para que el llamador
    /// decida reintentar a un nivel superior o alertar (regla "Atomicidad
    /// de ledgers append-only", DEBT-001).
    WriteContention { attempts: u32 },
}

impl std::fmt::Display for AuditLogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditLogError::Database(err) => write!(f, "audit log database error: {err}"),
            AuditLogError::WriteContention { attempts } => {
                write!(f, "no se pudo agregar el evento de auditoría tras {attempts} intentos por contención de escritura")
            }
        }
    }
}

impl std::error::Error for AuditLogError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AuditLogError::Database(err) => Some(err),
            AuditLogError::WriteContention { .. } => None,
        }
    }
}

impl From<sqlx::Error> for AuditLogError {
    fn from(err: sqlx::Error) -> Self {
        AuditLogError::Database(err)
    }
}

/// Número máximo de intentos del append ante contención de escritura
/// transitoria antes de rendirse con [`AuditLogError::WriteContention`].
/// Mismo valor y misma justificación que
/// [`crate::persistence::consent_registry::MAX_RECORD_ATTEMPTS`]: con
/// `busy_timeout` de 5s (ADR-0141 R2) el lock casi siempre se obtiene sin
/// reintentar.
const MAX_APPEND_ATTEMPTS: u32 = 5;

/// Decide si un error de [`AuditLogRepository::append`] es una contención de
/// escritura TRANSITORIA -- algo que reintentar (re-derivando el
/// `event_sequence_id` y reinsertando) puede resolver, sin descartar el
/// evento. Mismo criterio que
/// `crate::persistence::consent_registry::is_transient_write_conflict`.
fn is_transient_write_conflict(error: &AuditLogError) -> bool {
    let AuditLogError::Database(sqlx::Error::Database(db)) = error else {
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
    ///
    /// ## Atomicidad bajo concurrencia (regla "Atomicidad de ledgers append-only")
    ///
    /// El *read-then-write* (leer la cola de la cadena y el `INSERT` final)
    /// ocurre dentro de UNA sola transacción `BEGIN IMMEDIATE` -- ver
    /// [`Self::try_append_once`]. Sin esa transacción, dos escritores
    /// concurrentes derivarían el mismo `event_sequence_id`, el `UNIQUE`
    /// rechazaría a uno y su evento se PERDERÍA. Ante contención transitoria
    /// se reintenta hasta [`MAX_APPEND_ATTEMPTS`] veces re-derivando la
    /// secuencia; el evento NUNCA se descarta en silencio (si se agotan los
    /// reintentos se devuelve [`AuditLogError::WriteContention`]).
    pub async fn append(&self, content: AuditEventContent) -> Result<AuditEvent, AuditLogError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_append_once(&content).await {
                Ok(event) => return Ok(event),
                Err(error) => {
                    // Solo se reintenta ante contención de escritura
                    // transitoria; cualquier otro error se propaga de
                    // inmediato.
                    if is_transient_write_conflict(&error) {
                        if attempt < MAX_APPEND_ATTEMPTS {
                            continue;
                        }
                        // Agotados los reintentos: error tipado, NUNCA
                        // pérdida silenciosa del evento.
                        return Err(AuditLogError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    /// Un intento único del append, dentro de una transacción
    /// `BEGIN IMMEDIATE`. Devuelve el error tal cual si algo falla -- el
    /// bucle de [`Self::append`] decide si es transitorio y hay que
    /// reintentar. `BEGIN IMMEDIATE` toma el lock de escritura de ENTRADA:
    /// así ningún otro escritor puede intercalar entre la lectura de la cola
    /// y el `INSERT`, y se evita el interbloqueo de upgrade que ocurriría si
    /// dos transacciones DEFERRED intentaran subir de lectura a escritura a
    /// la vez.
    async fn try_append_once(&self, content: &AuditEventContent) -> Result<AuditEvent, AuditLogError> {
        // Abre la transacción tomando el lock de escritura de inmediato.
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        // Lectura (DENTRO de la transacción) -- la cola actual de la cadena
        // GLOBAL, para asignar el siguiente event_sequence_id y encadenar el
        // audit_hash.
        let tail_row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, manifest_id, access_token_id, \
                    process_id, session_id, node_id, \
                    action_type, entity_type, entity_id, details_json \
             FROM audit_events \
             ORDER BY event_sequence_id DESC \
             LIMIT 1",
        )
        .fetch_optional(&mut *tx)
        .await?;
        let previous = tail_row.map(row_to_event);

        // PK UUIDv7 (ADR-0141 M3): ordenable temporalmente, no v4.
        let id = Uuid::now_v7().to_string();
        let created_at_ns = self.clock.timestamp_ns();

        let event = chain_event(id, created_at_ns, content.clone(), previous.as_ref());

        // Escritura (DENTRO de la transacción) -- el INSERT que cierra el
        // read-then-write atómico.
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
        .execute(&mut *tx)
        .await?;

        // Confirma la transacción: recién aquí el lock de escritura se
        // libera y la fila se hace visible a otros escritores.
        tx.commit().await?;

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
    use crate::persistence::central_identity::test_support::seed_account;
    use crate::persistence::pool::{connect, migrate};

    fn sample_content(action_type: &str, entity_id: &str, owner_id: &str) -> AuditEventContent {
        AuditEventContent {
            action_type: action_type.to_string(),
            entity_type: "ORDER".to_string(),
            entity_id: entity_id.to_string(),
            details_json: "{\"from\":\"NEW\",\"to\":\"FILLED\"}".to_string(),
            owner_id: Some(owner_id.to_string()),
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
        let owner_id = seed_account(&pool, &clock, "user@example.com").await;
        let repo = AuditLogRepository::new(&pool, &clock);

        let first = repo
            .append(sample_content("ORDER_STATE_CHANGE", "order-1", &owner_id))
            .await
            .expect("append first event");
        assert_eq!(first.event_sequence_id, 1);
        assert_eq!(first.audit_chain_hash, None);

        clock.tick();
        let second = repo
            .append(sample_content("USER_VETO", "order-1", &owner_id))
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
        let owner_id = seed_account(&pool, &clock, "user@example.com").await;
        let repo = AuditLogRepository::new(&pool, &clock);

        let event = repo
            .append(sample_content("ORDER_STATE_CHANGE", "order-1", &owner_id))
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
        let owner_id = seed_account(&pool, &clock, "user@example.com").await;
        let repo = AuditLogRepository::new(&pool, &clock);

        let event = repo
            .append(sample_content("ORDER_STATE_CHANGE", "order-1", &owner_id))
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
        let owner_id = seed_account(&pool, &clock, "user@example.com").await;
        let repo = AuditLogRepository::new(&pool, &clock);

        repo.append(sample_content("ORDER_STATE_CHANGE", "order-1", &owner_id))
            .await
            .expect("append 1");
        clock.tick();
        repo.append(sample_content("ANOMALY_DETECTED", "order-2", &owner_id))
            .await
            .expect("append 2");
        clock.tick();
        repo.append(sample_content("USER_VETO", "order-1", &owner_id))
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

    // ── Atomicidad bajo concurrencia (regla "Atomicidad de ledgers append-only", DEBT-001) ──

    /// CRITERIO DE CIERRE (DEBT-001): N escritores concurrentes sobre el
    /// MISMO pool/ledger. La transacción `BEGIN IMMEDIATE` + reintento
    /// acotado debe garantizar que NINGÚN evento se pierde y que la
    /// secuencia queda densa (1..=N sin huecos ni duplicados) con la cadena
    /// de hashes íntegra.
    ///
    /// Esta prueba DEBE poder caerse si se quita la transacción: con el
    /// `SELECT ... ORDER BY event_sequence_id DESC LIMIT 1` y el `INSERT` en
    /// sentencias sueltas (la forma vieja de `append`), dos tareas leen la
    /// misma cola, derivan el mismo `event_sequence_id`, el `UNIQUE`
    /// rechaza a una y su evento se pierde -> la aserción (a)
    /// `chain.len()==N` o la (b) `1..=N` fallaría con
    /// `UNIQUE constraint failed: audit_events.event_sequence_id`. Se usa
    /// una BD en ARCHIVO temporal (nunca `:memory:`, donde cada conexión
    /// sería una base distinta) para que la concurrencia entre conexiones
    /// sea real.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_appends_persist_every_event_without_gaps_or_lost_rows() {
        use std::sync::Arc;

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("audit_log_concurrency.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());
        let pool = connect(&database_url).await.expect("conectar");
        migrate(&pool).await.expect("migrar");

        // Reloj compartido (atómico, thread-safe). Sin `tick`: todas las
        // filas comparten timestamp -- válido, el orden lo fija
        // event_sequence_id, no el reloj.
        let clock: Arc<DeterministicClock> = Arc::new(DeterministicClock::new(1_000, 100));
        let owner_id = seed_account(&pool, clock.as_ref(), "user@example.com").await;

        const N: i64 = 16;

        // Lanza N tareas en paralelo, cada una agregando un evento DISTINTO
        // (mismo entity_id, para poder verificar filtrado también).
        let mut handles = Vec::new();
        for i in 0..N {
            let pool_c = pool.clone(); // SqlitePool es un Arc interno: clonar es barato.
            let clock_c = clock.clone();
            let owner_id_c = owner_id.clone();
            handles.push(tokio::spawn(async move {
                let repo = AuditLogRepository::new(&pool_c, clock_c.as_ref());
                repo.append(sample_content("CONCURRENT_EVENT", &format!("order-{i}"), &owner_id_c))
                    .await
            }));
        }

        // (a) TODAS las tareas terminaron OK -- ningún evento se perdió por
        // colisión de secuencia (una tarea que perdiera la carrera y no
        // reintentara devolvería Err aquí).
        for handle in handles {
            handle
                .await
                .expect("la tarea no debe entrar en panic")
                .expect("append debe tener éxito para cada escritor concurrente");
        }

        let repo = AuditLogRepository::new(&pool, clock.as_ref());
        let chain = repo.load_chain().await.expect("cargar la cadena completa");

        // (a) se persistieron TODAS las N filas.
        assert_eq!(chain.len() as i64, N, "deben persistirse las N filas, sin ninguna pérdida");

        // (b) los event_sequence_id son exactamente 1..=N (densa, sin
        // huecos ni duplicados). `load_chain` ya ordena ascendente por la
        // columna.
        let sequence_ids: Vec<i64> = chain.iter().map(|event| event.event_sequence_id).collect();
        let expected: Vec<i64> = (1..=N).collect();
        assert_eq!(sequence_ids, expected, "los event_sequence_id deben ser 1..=N sin huecos ni duplicados");

        // (c) la cadena queda íntegra: verify_chain recalcula cada
        // audit_hash y cada enlace audit_chain_hash a partir del contenido
        // persistido -- si la transacción no fuera atómica, un evento
        // perdido rompería la densidad de (b) antes de siquiera llegar
        // aquí; esta aserción confirma además que el contenido de cada fila
        // es recomputable (integridad completa, no solo conteo).
        assert_eq!(verify_chain(&chain), ChainVerificationResult::Valid);
    }
}
