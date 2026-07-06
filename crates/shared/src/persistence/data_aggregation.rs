//! [SHELL] Repositorio de persistencia APPEND-ONLY ATÓMICO para los Índices
//! Agregados (`docs/features/data-aggregation.md`, ADR-0144 cimiento #9,
//! ADR-0102, ADR-0143, ADR-0141, ADR-0020, ADR-0093, migración
//! `0015_data_aggregation.sql`, STORY-036).
//!
//! Envuelve la tabla `aggregated_indexes`. Dueño del único I/O de este
//! cimiento: lecturas/escrituras en SQLite, generación de UUIDv7
//! (ADR-0141) y la lectura del puerto [`Clock`]. La lógica pura (ruido de
//! privacidad diferencial, k-anonimato, hash unidireccional de topología,
//! hash de auditoría encadenado) vive en
//! [`crate::domain::data_aggregation`] -- este módulo solo le da entradas
//! inyectadas y persiste el resultado, reflejando el mismo patrón de
//! [`crate::persistence::enriched_domain_events::DomainEventRepository`]
//! (misma naturaleza APPEND-ONLY: `event_sequence_id UNIQUE`, sin
//! `row_version`).
//!
//! ## Por qué NO existe `update`/`delete` en esta API
//!
//! A propósito: la única operación de escritura que este repositorio
//! expone es [`AggregatedIndexRepository::record_index`] (un INSERT). No
//! hay ningún método de actualización o borrado -- ni falta, porque los
//! triggers `trg_aggregated_indexes_no_update`/`trg_aggregated_indexes_no_delete`
//! de la migración los rechazarían de cualquier forma. La ausencia del
//! método en Rust es la primera línea de defensa; el trigger de SQLite es
//! la segunda (defensa en profundidad).

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::audit_log::GENESIS_PREVIOUS_HASH;
use crate::domain::clock::Clock;
use crate::domain::data_aggregation::{compute_aggregate_audit_hash, AggregatedIndex, Channel, IndexType};

/// Errores que devuelven las operaciones de [`AggregatedIndexRepository`].
#[derive(Debug, thiserror::Error)]
pub enum AggregatedIndexRepositoryError {
    /// La operación subyacente de SQLite falló.
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// Una fila persistida tenía un `index_type` fuera del catálogo --
    /// error de integridad de datos (no debería ocurrir nunca gracias al
    /// `CHECK` de la migración, pero se propaga explícitamente en vez de
    /// hacer panic).
    #[error("index_type desconocido en una fila persistida de aggregated_indexes: {0}")]
    UnknownIndexType(String),
    /// Una fila persistida tenía un `channel` fuera del catálogo.
    #[error("channel desconocido en una fila persistida de aggregated_indexes: {0}")]
    UnknownChannel(String),
    /// El append no pudo completarse tras agotar los reintentos ante
    /// contención de escritura transitoria (otro escritor mantuvo el lock
    /// de la base de datos más allá del `busy_timeout`, o hubo colisión
    /// repetida al derivar `event_sequence_id`). El índice NO se descartó
    /// en silencio -- se propaga este error tipado para que el llamador
    /// decida reintentar a un nivel superior o alertar (`docs/features/
    /// data-aggregation.md`, regla "Atomicidad de ledgers append-only").
    #[error("no se pudo registrar el índice agregado tras {attempts} intentos por contención de escritura")]
    WriteContention { attempts: u32 },
}

/// Número máximo de intentos del append ante contención de escritura
/// transitoria antes de rendirse con
/// [`AggregatedIndexRepositoryError::WriteContention`]. Cinco es holgado:
/// con `busy_timeout` de 5s (ADR-0141 R2) el lock casi siempre se obtiene
/// sin reintentar; el bucle solo actúa si el `busy_timeout` expira bajo
/// una contención extrema.
const MAX_RECORD_ATTEMPTS: u32 = 5;

/// Decide si un error del repositorio es una contención de escritura
/// TRANSITORIA -- es decir, algo que reintentar (re-derivando el
/// `event_sequence_id` y reinsertando) puede resolver, sin descartar el
/// índice.
///
/// Dos causas transitorias (mismo criterio que
/// `enriched_domain_events::is_transient_write_conflict`):
/// - `SQLITE_BUSY` / `SQLITE_LOCKED`: otro escritor tenía el lock de la
///   base de datos cuando esta conexión intentó tomarlo.
/// - Violación de UNIQUE sobre `event_sequence_id`: dos escritores
///   derivaron la misma posición de secuencia. Con `BEGIN IMMEDIATE` esto
///   no debería ocurrir, pero se trata como transitorio de
///   cinturón-y-tirantes.
fn is_transient_write_conflict(error: &AggregatedIndexRepositoryError) -> bool {
    let AggregatedIndexRepositoryError::Database(sqlx::Error::Database(db)) = error else {
        return false;
    };

    let message = db.message().to_lowercase();
    if message.contains("database is locked") || message.contains("database table is locked") {
        return true;
    }

    db.is_unique_violation() && message.contains("event_sequence_id")
}

/// Entrada para [`AggregatedIndexRepository::record_index`] -- todo lo que
/// la Shell necesita para registrar UN índice agregado: el índice ya
/// calculado por el Core ([`AggregatedIndex`]) y la identidad del
/// dueño/máquina del artefacto derivado (Perfil B de ADR-0020) más su
/// linaje al conjunto fuente.
#[derive(Debug, Clone)]
pub struct RecordAggregatedIndexInput {
    /// Dueño del ARTEFACTO derivado (el proceso/agregador que lo calculó),
    /// NUNCA un usuario contribuyente individual de la cohorte (ver
    /// comentario de la migración `0015_data_aggregation.sql`).
    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,
    /// Referencia de linaje al conjunto de eventos fuente que alimentó
    /// este agregado (nullable -- un agregado sembrado sin conjunto fuente
    /// identificado no lo tiene).
    pub data_snapshot_id: Option<String>,
    pub index: AggregatedIndex,
}

/// Una fila de `aggregated_indexes` ya persistida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregatedIndexRow {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub event_sequence_id: i64,

    pub owner_id: String,
    pub institutional_tag: String,
    pub data_snapshot_id: Option<String>,
    pub node_id: String,

    pub index_type: IndexType,
    pub time_window: String,
    pub cohort_size: i64,
    pub noise_level_e8: i64,
    pub metric_value_e8: i64,
    pub channel: Channel,
}

/// Repositorio APPEND-ONLY para `aggregated_indexes`.
///
/// Constrúyelo con un [`SqlitePool`] ya migrado y cualquier implementación
/// de [`Clock`] -- mismo patrón que
/// [`crate::persistence::enriched_domain_events::DomainEventRepository`].
pub struct AggregatedIndexRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> AggregatedIndexRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`. Ambos se toman
    /// prestados por la vida del repositorio -- no se toma ownership.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Registra UN índice agregado: deriva su posición en la cadena
    /// GLOBAL, computa su hash encadenado y lo persiste como fila nueva.
    ///
    /// Es la ÚNICA forma de escribir en `aggregated_indexes` -- no existe
    /// `update`/`delete` en esta API (ver doc-comment del módulo).
    ///
    /// ## Atomicidad bajo concurrencia (regla "Atomicidad de ledgers append-only")
    ///
    /// Todo el *read-then-write* (leer el MAX(`event_sequence_id`) y el
    /// `audit_hash` previo para encadenar, y el `INSERT` final) ocurre
    /// dentro de UNA sola transacción `BEGIN IMMEDIATE` -- ver
    /// [`Self::try_record_index_once`]. Sin esa transacción, dos
    /// escritores concurrentes derivarían el mismo `event_sequence_id`, el
    /// `UNIQUE` rechazaría a uno y su índice se PERDERÍA. Ante contención
    /// transitoria, se reintenta hasta [`MAX_RECORD_ATTEMPTS`] veces
    /// re-derivando la secuencia; el índice NUNCA se descarta en silencio.
    pub async fn record_index(
        &self,
        input: RecordAggregatedIndexInput,
    ) -> Result<AggregatedIndexRow, AggregatedIndexRepositoryError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_record_index_once(&input).await {
                Ok(row) => return Ok(row),
                Err(error) => {
                    if is_transient_write_conflict(&error) {
                        if attempt < MAX_RECORD_ATTEMPTS {
                            continue;
                        }
                        return Err(AggregatedIndexRepositoryError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    /// Un intento único del append, dentro de una transacción
    /// `BEGIN IMMEDIATE` (no el `BEGIN` DEFERRED por defecto de SQLx) --
    /// toma el lock de escritura de ENTRADA, así ningún otro escritor
    /// puede intercalar entre la lectura del MAX(`event_sequence_id`) y el
    /// `INSERT`.
    async fn try_record_index_once(
        &self,
        input: &RecordAggregatedIndexInput,
    ) -> Result<AggregatedIndexRow, AggregatedIndexRepositoryError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        // Lectura (DENTRO de la transacción) -- posición en la cadena
        // GLOBAL: la fila con el event_sequence_id más alto de TODA la
        // tabla, para asignar la siguiente y encadenar su audit_hash.
        let tail_row = sqlx::query(
            "SELECT audit_hash, event_sequence_id \
             FROM aggregated_indexes \
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

        let id = Uuid::now_v7().to_string();
        // Reloj INYECTADO -- nunca SystemTime::now() directo (ADR-0002/0004).
        let now_ns = self.clock.timestamp_ns();

        let audit_hash = compute_aggregate_audit_hash(
            &id,
            now_ns,
            event_sequence_id,
            &previous_audit_hash,
            &input.owner_id,
            &input.institutional_tag,
            &input.node_id,
            input.data_snapshot_id.as_deref(),
            input.index.index_type,
            &input.index.time_window,
            input.index.cohort_size,
            input.index.noise_level_e8,
            input.index.metric_value_e8,
            input.index.channel,
        );

        // Escritura (DENTRO de la transacción) -- el INSERT que cierra el
        // read-then-write atómico.
        sqlx::query(
            "INSERT INTO aggregated_indexes (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, data_snapshot_id, node_id, \
                index_type, time_window, cohort_size, noise_level, metric_value, channel\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&audit_chain_hash)
        .bind(event_sequence_id)
        .bind(&input.owner_id)
        .bind(&input.institutional_tag)
        .bind(&input.data_snapshot_id)
        .bind(&input.node_id)
        .bind(input.index.index_type.as_str())
        .bind(&input.index.time_window)
        .bind(input.index.cohort_size)
        .bind(input.index.noise_level_e8)
        .bind(input.index.metric_value_e8)
        .bind(input.index.channel.as_str())
        .execute(&mut *tx)
        .await?;

        // Confirma la transacción: recién aquí el lock de escritura se
        // libera y la fila se hace visible a otros escritores.
        tx.commit().await?;

        Ok(AggregatedIndexRow {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash,
            event_sequence_id,
            owner_id: input.owner_id.clone(),
            institutional_tag: input.institutional_tag.clone(),
            data_snapshot_id: input.data_snapshot_id.clone(),
            node_id: input.node_id.clone(),
            index_type: input.index.index_type,
            time_window: input.index.time_window.clone(),
            cohort_size: input.index.cohort_size,
            noise_level_e8: input.index.noise_level_e8,
            metric_value_e8: input.index.metric_value_e8,
            channel: input.index.channel,
        })
    }

    /// Carga la cadena completa, ordenada por `event_sequence_id`
    /// ascendente -- usada por los tests de integridad de la cadena y por
    /// cualquier consumidor futuro que reconstruya el historial de
    /// agregados publicados.
    pub async fn load_chain(&self) -> Result<Vec<AggregatedIndexRow>, AggregatedIndexRepositoryError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, data_snapshot_id, node_id, \
                    index_type, time_window, cohort_size, noise_level, metric_value, channel \
             FROM aggregated_indexes \
             ORDER BY event_sequence_id ASC",
        )
        .fetch_all(self.pool)
        .await?;

        rows.into_iter().map(row_to_aggregated_index).collect()
    }
}

/// Convierte una fila de `aggregated_indexes` al tipo
/// [`AggregatedIndexRow`], reconstruyendo `index_type`/`channel` desde su
/// representación en texto y propagando un error tipado si alguno de los
/// dos quedó corrupto (no debería ocurrir nunca gracias al `CHECK` de la
/// migración).
fn row_to_aggregated_index(
    row: sqlx::sqlite::SqliteRow,
) -> Result<AggregatedIndexRow, AggregatedIndexRepositoryError> {
    let index_type_str: String = row.get("index_type");
    let index_type = IndexType::from_str_value(&index_type_str)
        .ok_or(AggregatedIndexRepositoryError::UnknownIndexType(index_type_str))?;

    let channel_str: String = row.get("channel");
    let channel = Channel::from_str_value(&channel_str)
        .ok_or(AggregatedIndexRepositoryError::UnknownChannel(channel_str))?;

    Ok(AggregatedIndexRow {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        event_sequence_id: row.get("event_sequence_id"),
        owner_id: row.get("owner_id"),
        institutional_tag: row.get("institutional_tag"),
        data_snapshot_id: row.get("data_snapshot_id"),
        node_id: row.get("node_id"),
        index_type,
        time_window: row.get("time_window"),
        cohort_size: row.get("cohort_size"),
        noise_level_e8: row.get("noise_level"),
        metric_value_e8: row.get("metric_value"),
        channel,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::domain::data_aggregation::aggregate_index;
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    fn sample_index(seed: u64) -> AggregatedIndex {
        let covered = vec![100_000_000_000_i64; 5];
        aggregate_index(&covered, IndexType::Sentiment, "2026-W27", Channel::Internal, 5, 1_000_000, seed)
            .expect("cohorte suficiente debe publicar")
    }

    fn record_input(index: AggregatedIndex) -> RecordAggregatedIndexInput {
        RecordAggregatedIndexInput {
            owner_id: "drasus-aggregator".to_string(),
            institutional_tag: "DRASUS_LOCAL".to_string(),
            node_id: "node-1".to_string(),
            data_snapshot_id: Some("snapshot-1".to_string()),
            index,
        }
    }

    // ── CRITERIO #1 (Orden §5): esquema STRICT append-only + Grupo I + Perfil B ──

    #[tokio::test]
    async fn migration_creates_aggregated_indexes_table_strict_append_only_with_group_i_and_profile_b() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('aggregated_indexes')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();

        for expected in [
            "id", "created_at", "updated_at", "audit_hash", "audit_chain_hash", "event_sequence_id",
            "owner_id", "institutional_tag", "data_snapshot_id", "node_id",
            "index_type", "time_window", "cohort_size", "noise_level", "metric_value", "channel",
        ] {
            assert!(column_names.contains(&expected.to_string()), "falta la columna: {expected}");
        }

        assert!(
            !column_names.contains(&"row_version".to_string()),
            "aggregated_indexes es APPEND-ONLY (ADR-0141): no debe tener row_version, solo event_sequence_id"
        );

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'aggregated_indexes'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(sql.contains("STRICT"), "la tabla aggregated_indexes debe declararse STRICT");
    }

    // ── CRITERIO #8 (Orden §5): append-only -- UPDATE/DELETE rechazados ──────

    #[tokio::test]
    async fn update_is_rejected_by_trigger() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AggregatedIndexRepository::new(&pool, &clock);

        let row = repo.record_index(record_input(sample_index(1))).await.expect("registrar índice");

        let result = sqlx::query("UPDATE aggregated_indexes SET metric_value = 0 WHERE id = ?")
            .bind(&row.id)
            .execute(&pool)
            .await;

        assert!(result.is_err(), "UPDATE sobre aggregated_indexes debe ser rechazado por el trigger");
    }

    #[tokio::test]
    async fn delete_is_rejected_by_trigger() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AggregatedIndexRepository::new(&pool, &clock);

        let row = repo.record_index(record_input(sample_index(1))).await.expect("registrar índice");

        let result = sqlx::query("DELETE FROM aggregated_indexes WHERE id = ?")
            .bind(&row.id)
            .execute(&pool)
            .await;

        assert!(result.is_err(), "DELETE sobre aggregated_indexes debe ser rechazado por el trigger");
    }

    // ── CRITERIO #8 (Orden §5): event_sequence_id monótono + UNIQUE ──────────

    #[tokio::test]
    async fn event_sequence_id_is_monotonic_across_inserts() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AggregatedIndexRepository::new(&pool, &clock);

        let first = repo.record_index(record_input(sample_index(1))).await.expect("primero");
        clock.tick();
        let second = repo.record_index(record_input(sample_index(2))).await.expect("segundo");
        clock.tick();
        let third = repo.record_index(record_input(sample_index(3))).await.expect("tercero");

        assert_eq!(first.event_sequence_id, 1);
        assert_eq!(second.event_sequence_id, 2);
        assert_eq!(third.event_sequence_id, 3);
    }

    /// CRITERIO DE CIERRE: `CHECK (index_type IN (...))` rechaza un valor
    /// fuera del catálogo -- si el CHECK no existiera, el INSERT tendría
    /// éxito con basura en la columna.
    #[tokio::test]
    async fn database_check_rejects_unknown_index_type() {
        let pool = migrated_pool().await;

        let result = sqlx::query(
            "INSERT INTO aggregated_indexes (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, data_snapshot_id, node_id, \
                index_type, time_window, cohort_size, noise_level, metric_value, channel\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', 'DRASUS_LOCAL', NULL, 'node-1', \
                       'UNKNOWN_TYPE', '2026-W27', 5, 0, 0, 'INTERNAL')",
        )
        .execute(&pool)
        .await;

        assert!(result.is_err(), "un index_type fuera del catálogo debe ser rechazado por el CHECK de la BD");
    }

    /// CRITERIO DE CIERRE: `CHECK (cohort_size > 0)` rechaza una cohorte
    /// no positiva -- refuerzo estructural del k-anonimato a nivel de BD.
    #[tokio::test]
    async fn database_check_rejects_non_positive_cohort_size() {
        let pool = migrated_pool().await;

        let result = sqlx::query(
            "INSERT INTO aggregated_indexes (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, data_snapshot_id, node_id, \
                index_type, time_window, cohort_size, noise_level, metric_value, channel\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', 'DRASUS_LOCAL', NULL, 'node-1', \
                       'SENTIMENT', '2026-W27', 0, 0, 0, 'INTERNAL')",
        )
        .execute(&pool)
        .await;

        assert!(result.is_err(), "cohort_size <= 0 debe ser rechazado por el CHECK de la BD");
    }

    // ── CRITERIO #8 (Orden §5): audit_chain_hash génesis NULL + encadenado ───

    #[tokio::test]
    async fn audit_chain_hash_is_null_only_in_genesis_row_and_chains_afterwards() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AggregatedIndexRepository::new(&pool, &clock);

        let first = repo.record_index(record_input(sample_index(1))).await.expect("génesis");
        clock.tick();
        let second = repo.record_index(record_input(sample_index(2))).await.expect("segundo");
        clock.tick();
        let third = repo.record_index(record_input(sample_index(3))).await.expect("tercero");

        assert_eq!(first.audit_chain_hash, None, "la fila génesis debe tener audit_chain_hash NULL");
        assert_eq!(second.audit_chain_hash, Some(first.audit_hash.clone()), "debe encadenar a la primera");
        assert_eq!(third.audit_chain_hash, Some(second.audit_hash.clone()), "debe encadenar a la segunda");
    }

    // ── CRITERIO #7 (Orden §5): append atómico + concurrencia (16 escritores) ──

    /// CRITERIO DE CIERRE (DEBT-001): 16 escritores concurrentes sobre el
    /// MISMO pool/ledger, con la BD en ARCHIVO temporal (NUNCA `:memory:`,
    /// donde cada conexión sería una base distinta -- la concurrencia real
    /// entre conexiones exige un archivo compartido). La transacción
    /// `BEGIN IMMEDIATE` + reintento acotado debe garantizar que NINGÚN
    /// índice se pierde y que la secuencia queda densa (1..=N sin huecos
    /// ni duplicados).
    ///
    /// Esta prueba DEBE poder caerse si se quita la transacción: con el
    /// `SELECT MAX(...)` y el `INSERT` en sentencias sueltas, dos tareas
    /// leerían el mismo MAX, derivarían el mismo `event_sequence_id`, el
    /// `UNIQUE` rechazaría a una y su fila se perdería -> `chain.len() ==
    /// N` o la secuencia `1..=N` fallarían.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_record_index_persist_every_row_without_gaps_or_lost_rows() {
        use std::sync::Arc;

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("aggregated_indexes_concurrency.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());
        let pool = connect(&database_url).await.expect("conectar");
        migrate(&pool).await.expect("migrar");

        // Reloj compartido (atómico, thread-safe). No se hace `tick`: todas
        // las filas comparten timestamp -- el orden lo fija
        // event_sequence_id, no el reloj.
        let clock: Arc<DeterministicClock> = Arc::new(DeterministicClock::new(1_000, 100));

        const N: i64 = 16;

        let mut handles = Vec::new();
        for i in 0..N {
            let pool_c = pool.clone(); // SqlitePool es un Arc interno: clonar es barato.
            let clock_c = clock.clone();
            handles.push(tokio::spawn(async move {
                let repo = AggregatedIndexRepository::new(&pool_c, clock_c.as_ref());
                repo.record_index(record_input(sample_index(i as u64))).await
            }));
        }

        // (a) TODAS las tareas terminaron OK -- ningún índice se perdió por
        // colisión de secuencia.
        for handle in handles {
            handle
                .await
                .expect("la tarea no debe entrar en panic")
                .expect("record_index debe tener éxito para cada escritor concurrente");
        }

        let repo = AggregatedIndexRepository::new(&pool, clock.as_ref());
        let chain = repo.load_chain().await.expect("cargar la cadena completa");

        assert_eq!(chain.len() as i64, N, "deben persistirse las N filas, sin ninguna pérdida");

        let sequence_ids: Vec<i64> = chain.iter().map(|row| row.event_sequence_id).collect();
        let expected: Vec<i64> = (1..=N).collect();
        assert_eq!(sequence_ids, expected, "los event_sequence_id deben ser 1..=N sin huecos ni duplicados");

        // La cadena audit_chain_hash queda íntegra: génesis con NULL, cada
        // fila encadenada al audit_hash de la anterior.
        for (index, row) in chain.iter().enumerate() {
            if index == 0 {
                assert_eq!(row.audit_chain_hash, None, "la fila génesis debe tener audit_chain_hash NULL");
            } else {
                let prev = &chain[index - 1];
                assert_eq!(
                    row.audit_chain_hash.as_deref(),
                    Some(prev.audit_hash.as_str()),
                    "cada fila debe encadenar al audit_hash de la anterior"
                );
            }
        }
    }
}
