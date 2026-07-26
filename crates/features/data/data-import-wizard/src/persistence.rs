//! [SHELL] Repositorio de persistencia para `import_mapping_declarations`
//! (migración 0023). Usa el pool de `shared` -- este módulo nunca crea su
//! propio pool; siempre recibe uno ya inicializado y migrado desde el
//! exterior.
//!
//! Molde canónico: `.agents/knowledge/ledger-pattern.md`. El *read-then-
//! write* (derivar `event_sequence_id` + leer la cola de la cadena) ocurre
//! dentro de UNA sola transacción `BEGIN IMMEDIATE`, con reintento acotado
//! ante contención transitoria -- nunca sentencias sueltas.

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use shared::public_interface::Clock;

use crate::schemas::{MappingDeclarationRow, NewMappingDeclaration};

// ── Error del repositorio ────────────────────────────────────────────────────

/// Error que devuelven las operaciones del repositorio de declaraciones de mapeo.
#[derive(Debug)]
pub enum MappingDeclarationRepositoryError {
    /// La operación subyacente de SQLite falló.
    Database(sqlx::Error),
    /// [`MappingDeclarationRepository::record`] no pudo completarse tras
    /// agotar los reintentos ante contención de escritura transitoria -- la
    /// declaración NO se descartó en silencio.
    WriteContention { attempts: u32 },
}

impl std::fmt::Display for MappingDeclarationRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MappingDeclarationRepositoryError::Database(err) => {
                write!(f, "mapping declaration repository database error: {err}")
            }
            MappingDeclarationRepositoryError::WriteContention { attempts } => write!(
                f,
                "no se pudo sellar la declaración de mapeo tras {attempts} intentos por contención de escritura"
            ),
        }
    }
}

impl std::error::Error for MappingDeclarationRepositoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MappingDeclarationRepositoryError::Database(err) => Some(err),
            MappingDeclarationRepositoryError::WriteContention { .. } => None,
        }
    }
}

impl From<sqlx::Error> for MappingDeclarationRepositoryError {
    fn from(err: sqlx::Error) -> Self {
        MappingDeclarationRepositoryError::Database(err)
    }
}

/// Número máximo de intentos de [`MappingDeclarationRepository::record`]
/// ante contención de escritura transitoria antes de rendirse con
/// [`MappingDeclarationRepositoryError::WriteContention`]. Mismo valor que
/// el resto de los ledgers append-only del sistema.
const MAX_RECORD_ATTEMPTS: u32 = 5;

/// Decide si un error de [`MappingDeclarationRepository::record`] es una
/// contención de escritura TRANSITORIA -- algo que reintentar (re-derivando
/// el `event_sequence_id` y reinsertando) puede resolver, sin descartar la
/// declaración -- o PERMANENTE (violación de la PK `id`, dato mal formado),
/// que reintentar jamás resolvería.
///
/// Este es el hueco histórico que `.agents/knowledge/ledger-pattern.md`
/// documenta ("Hueco 1"): la prueba de este clasificador DEBE llamarlo
/// directamente con sus tres casos, nunca verificar el mensaje de un
/// `INSERT` crudo "indirectamente" -- ver `tests` abajo.
fn is_transient_write_conflict(error: &MappingDeclarationRepositoryError) -> bool {
    let MappingDeclarationRepositoryError::Database(sqlx::Error::Database(db)) = error else {
        return false;
    };

    let message = db.message().to_lowercase();
    if message.contains("database is locked") || message.contains("database table is locked") {
        return true;
    }

    db.is_unique_violation() && message.contains("event_sequence_id")
}

// ── Repositorio ──────────────────────────────────────────────────────────────

/// Repositorio para la tabla `import_mapping_declarations`.
///
/// Constrúyelo con un `SqlitePool` ya migrado (incluye la migración 0023) y
/// una implementación de `Clock`.
pub struct MappingDeclarationRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> MappingDeclarationRepository<'a> {
    /// Crea un repositorio asociado al pool y clock dados.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Busca una declaración ya sellada por el hash de contenido del
    /// archivo origen -- soporte de la reimportación idempotente (ADR-0163,
    /// criterio de aceptación #10 de STORY-068): si ya existe, el llamador
    /// trata la importación como un no-op en vez de sellar una segunda fila.
    pub async fn find_by_source_file_hash(
        &self,
        source_file_hash: &str,
    ) -> Result<Option<MappingDeclarationRow>, MappingDeclarationRepositoryError> {
        let sql = format!("SELECT {DECLARATION_COLUMNS} FROM import_mapping_declarations WHERE source_file_hash = ?");
        let row = sqlx::query(&sql).bind(source_file_hash).fetch_optional(self.pool).await?;
        Ok(row.as_ref().map(row_to_declaration))
    }

    /// Sella una nueva declaración de mapeo y la devuelve ya persistida.
    ///
    /// Genera automáticamente: UUID v7, timestamps (del Clock inyectado) y
    /// el encadenamiento con la declaración previa (`audit_chain_hash` = el
    /// `audit_hash` de la fila anterior; `NULL` en la fila génesis). El
    /// `audit_hash` de la fila NO se recalcula aquí -- llega ya calculado
    /// desde el Núcleo (`new.content_audit_hash`). El `event_sequence_id` es
    /// el siguiente número monótono en la tabla.
    ///
    /// ## Atomicidad bajo concurrencia
    ///
    /// El *read-then-write* (leer la cola de la cadena y el `INSERT` final)
    /// ocurre dentro de UNA sola transacción `BEGIN IMMEDIATE` -- ver
    /// [`Self::try_record_once`]. Ante contención transitoria se reintenta
    /// hasta [`MAX_RECORD_ATTEMPTS`] veces re-derivando la secuencia; la
    /// declaración NUNCA se descarta en silencio.
    pub async fn record(
        &self,
        new: NewMappingDeclaration,
    ) -> Result<MappingDeclarationRow, MappingDeclarationRepositoryError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_record_once(&new).await {
                Ok(record) => return Ok(record),
                Err(error) => {
                    if is_transient_write_conflict(&error) {
                        if attempt < MAX_RECORD_ATTEMPTS {
                            continue;
                        }
                        return Err(MappingDeclarationRepositoryError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    /// Un intento único de [`Self::record`], dentro de una transacción
    /// `BEGIN IMMEDIATE` -- toma el lock de escritura de ENTRADA, evitando
    /// tanto la intercalación de otro escritor entre la lectura de la cola y
    /// el `INSERT` como el interbloqueo de upgrade de dos transacciones DEFERRED.
    async fn try_record_once(
        &self,
        new: &NewMappingDeclaration,
    ) -> Result<MappingDeclarationRow, MappingDeclarationRepositoryError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        let tail_row = sqlx::query(
            "SELECT audit_hash, event_sequence_id FROM import_mapping_declarations \
             ORDER BY event_sequence_id DESC LIMIT 1",
        )
        .fetch_optional(&mut *tx)
        .await?;

        let (event_sequence_id, previous_hash) = match tail_row {
            Some(row) => {
                let previous_seq: i64 = row.get("event_sequence_id");
                let previous_hash: String = row.get("audit_hash");
                (previous_seq + 1, Some(previous_hash))
            }
            None => (1, None),
        };

        let id = Uuid::now_v7().to_string();
        let now_ns = self.clock.timestamp_ns();

        sqlx::query(
            "INSERT INTO import_mapping_declarations (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                node_id, process_id, \
                source_file_name, source_file_hash, instrument_id, venue, bar_type, timeframe_value, \
                mapping_origin, platform_template_code, platform_template_version, column_mapping_json, \
                transform_engine_version, canonical_schema_version, accepted_at, \
                preview_transformed_rows_json, source_timezone_declared, real_volume_unit_declared, \
                spread_reported_unit_declared \
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&new.content_audit_hash)
        .bind(&previous_hash)
        .bind(event_sequence_id)
        .bind(&new.node_id)
        .bind(&new.process_id)
        .bind(&new.source_file_name)
        .bind(&new.source_file_hash)
        .bind(&new.instrument_id)
        .bind(&new.venue)
        .bind(&new.bar_type)
        .bind(&new.timeframe_value)
        .bind(&new.mapping_origin)
        .bind(&new.platform_template_code)
        .bind(&new.platform_template_version)
        .bind(&new.column_mapping_json)
        .bind(&new.transform_engine_version)
        .bind(&new.canonical_schema_version)
        .bind(new.accepted_at)
        .bind(&new.preview_transformed_rows_json)
        .bind(&new.source_timezone_declared)
        .bind(&new.real_volume_unit_declared)
        .bind(&new.spread_reported_unit_declared)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(MappingDeclarationRow {
            id,
            created_at: now_ns,
            updated_at: now_ns,
            audit_hash: new.content_audit_hash.clone(),
            audit_chain_hash: previous_hash,
            event_sequence_id,
            node_id: new.node_id.clone(),
            process_id: new.process_id.clone(),
            source_file_name: new.source_file_name.clone(),
            source_file_hash: new.source_file_hash.clone(),
            instrument_id: new.instrument_id.clone(),
            venue: new.venue.clone(),
            bar_type: new.bar_type.clone(),
            timeframe_value: new.timeframe_value.clone(),
            mapping_origin: new.mapping_origin.clone(),
            platform_template_code: new.platform_template_code.clone(),
            platform_template_version: new.platform_template_version.clone(),
            column_mapping_json: new.column_mapping_json.clone(),
            transform_engine_version: new.transform_engine_version.clone(),
            canonical_schema_version: new.canonical_schema_version.clone(),
            accepted_at: new.accepted_at,
            preview_transformed_rows_json: new.preview_transformed_rows_json.clone(),
            source_timezone_declared: new.source_timezone_declared.clone(),
            real_volume_unit_declared: new.real_volume_unit_declared.clone(),
            spread_reported_unit_declared: new.spread_reported_unit_declared.clone(),
        })
    }

    /// Lista todas las declaraciones, de la más reciente a la más antigua.
    /// Sin paginación -- adecuado para el volumen de EPIC-1.
    pub async fn list_all(&self) -> Result<Vec<MappingDeclarationRow>, MappingDeclarationRepositoryError> {
        let sql = format!("SELECT {DECLARATION_COLUMNS} FROM import_mapping_declarations ORDER BY event_sequence_id DESC");
        let rows = sqlx::query(&sql).fetch_all(self.pool).await?;
        Ok(rows.iter().map(row_to_declaration).collect())
    }
}

/// Lista única de columnas de `import_mapping_declarations`, compartida por
/// todas las consultas `SELECT` de este repositorio -- una sola fuente de
/// verdad para que un campo nuevo no se agregue en una consulta y se
/// olvide en otra.
const DECLARATION_COLUMNS: &str = "id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
     node_id, process_id, \
     source_file_name, source_file_hash, instrument_id, venue, bar_type, timeframe_value, \
     mapping_origin, platform_template_code, platform_template_version, column_mapping_json, \
     transform_engine_version, canonical_schema_version, accepted_at, \
     preview_transformed_rows_json, source_timezone_declared, real_volume_unit_declared, \
     spread_reported_unit_declared";

fn row_to_declaration(row: &sqlx::sqlite::SqliteRow) -> MappingDeclarationRow {
    MappingDeclarationRow {
        id: row.get("id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        event_sequence_id: row.get("event_sequence_id"),
        node_id: row.get("node_id"),
        process_id: row.get("process_id"),
        source_file_name: row.get("source_file_name"),
        source_file_hash: row.get("source_file_hash"),
        instrument_id: row.get("instrument_id"),
        venue: row.get("venue"),
        bar_type: row.get("bar_type"),
        timeframe_value: row.get("timeframe_value"),
        mapping_origin: row.get("mapping_origin"),
        platform_template_code: row.get("platform_template_code"),
        platform_template_version: row.get("platform_template_version"),
        column_mapping_json: row.get("column_mapping_json"),
        transform_engine_version: row.get("transform_engine_version"),
        canonical_schema_version: row.get("canonical_schema_version"),
        accepted_at: row.get("accepted_at"),
        preview_transformed_rows_json: row.get("preview_transformed_rows_json"),
        source_timezone_declared: row.get("source_timezone_declared"),
        real_volume_unit_declared: row.get("real_volume_unit_declared"),
        spread_reported_unit_declared: row.get("spread_reported_unit_declared"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Pruebas -- Capa 2 de la pirámide ADR-0133 (integración + persistencia).
// Durabilidad/recuperación: SQLite en ARCHIVO temporal, nunca `:memory:`.
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use shared::public_interface::{create_pool, run_migrations, DeterministicClock};
    use std::sync::Arc;

    /// Crea un pool de prueba en un archivo SQLite temporal real (nunca
    /// `:memory:` -- regla de durabilidad de la pirámide ADR-0133 Capa 2),
    /// con las migraciones aplicadas y una fila mínima de `instrument_master`
    /// para "EURUSD" -- el ledger de esta feature tiene una FK física hacia
    /// esa tabla (ADR-0141 M6), así que toda declaración de prueba necesita
    /// que el instrumento exista primero, igual que en producción (pregunta
    /// 9 del barrido del Architect: "el instrumento debe existir ANTES de
    /// importar").
    async fn temp_pool() -> (SqlitePool, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("no se pudo crear directorio temporal");
        let db_path = dir.path().join("test.db");
        let url = format!("sqlite://{}", db_path.display());
        let pool = create_pool(&url).await.expect("no se pudo crear el pool de prueba");
        run_migrations(&pool).await.expect("las migraciones deben aplicar sin error");

        sqlx::query(
            "INSERT INTO instrument_master (id, created_at, updated_at, audit_hash, row_version, \
             instrument_id, asset_class, canonical_pip_size, canonical_tick_size) \
             VALUES ('instrument-eurusd', 0, 0, 'seed-hash', 1, 'EURUSD', 'FOREX', 10000, 1000)",
        )
        .execute(&pool)
        .await
        .expect("la fila semilla de instrument_master debe insertarse para satisfacer la FK del ledger");

        (pool, dir)
    }

    fn sample_new_declaration(file_hash: &str) -> NewMappingDeclaration {
        NewMappingDeclaration {
            content_audit_hash: format!("hash-of-{file_hash}"),
            source_file_name: "EURUSD_M1.csv".to_string(),
            source_file_hash: file_hash.to_string(),
            instrument_id: "EURUSD".to_string(),
            venue: "PEPPERSTONE".to_string(),
            bar_type: "TIME".to_string(),
            timeframe_value: "1M".to_string(),
            mapping_origin: "PLATFORM_TEMPLATE".to_string(),
            platform_template_code: Some("MT5_M1_BAR_V1".to_string()),
            platform_template_version: Some("1.0.0".to_string()),
            column_mapping_json: "{}".to_string(),
            transform_engine_version: "0.1.0".to_string(),
            canonical_schema_version: "ADR-0155-v1".to_string(),
            accepted_at: 1_000,
            preview_transformed_rows_json: "[]".to_string(),
            source_timezone_declared: "America/New_York".to_string(),
            real_volume_unit_declared: None,
            spread_reported_unit_declared: "POINTS_MT5".to_string(),
            node_id: Some("test-node".to_string()),
            process_id: Some("test-process".to_string()),
        }
    }

    // ── Prueba obligatoria 1/4: concurrencia real ────────────────────────

    /// 16 escritores en paralelo, runtime multihilo, sin perder ni duplicar
    /// secuencia -- prueba obligatoria del molde de `ledger-pattern.md`.
    #[tokio::test(flavor = "multi_thread")]
    async fn sixteen_concurrent_writers_never_lose_or_duplicate_sequence() {
        let (pool, _dir) = temp_pool().await;
        let clock = DeterministicClock::new(0, 1);
        let repo = Arc::new((pool, clock));

        let mut handles = Vec::new();
        for i in 0..16 {
            let repo = Arc::clone(&repo);
            handles.push(tokio::spawn(async move {
                let (pool, clock) = &*repo;
                let repository = MappingDeclarationRepository::new(pool, clock);
                repository.record(sample_new_declaration(&format!("writer-{i}"))).await
            }));
        }

        let mut sequence_ids = Vec::new();
        for handle in handles {
            let result = handle.await.expect("la tarea no debe hacer panic").expect("cada escritor debe lograr sellar su declaración");
            sequence_ids.push(result.event_sequence_id);
        }

        sequence_ids.sort_unstable();
        let expected: Vec<i64> = (1..=16).collect();
        assert_eq!(sequence_ids, expected, "16 escritores concurrentes deben producir la secuencia 1..=16 sin huecos ni duplicados");
    }

    // ── Prueba obligatoria 2/4: contención sostenida hasta agotar reintentos ──

    /// Un segundo escritor retiene `BEGIN IMMEDIATE` mientras el repositorio
    /// intenta escribir -- debe agotar los reintentos y devolver
    /// `WriteContention` con el conteo exacto de intentos.
    #[tokio::test(flavor = "multi_thread")]
    async fn sustained_contention_exhausts_retries_and_returns_typed_error_with_exact_attempt_count() {
        let (pool, _dir) = temp_pool().await;
        let clock = DeterministicClock::new(0, 1);

        // Un segundo escritor toma el lock de escritura de la base con
        // `BEGIN IMMEDIATE` y NUNCA lo suelta durante la prueba -- simula
        // contención sostenida real, no una simulación de mensaje de error.
        let mut blocking_tx = pool.begin_with("BEGIN IMMEDIATE").await.expect("el segundo escritor debe poder tomar el lock");
        sqlx::query("INSERT INTO import_mapping_declarations (id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, source_file_name, source_file_hash, instrument_id, venue, bar_type, timeframe_value, mapping_origin, column_mapping_json, transform_engine_version, canonical_schema_version, accepted_at, preview_transformed_rows_json, source_timezone_declared, spread_reported_unit_declared) VALUES ('blocker-id', 0, 0, 'blocker-hash', NULL, 1, 'f.csv', 'blocker-file-hash', 'EURUSD', 'PEPPERSTONE', 'TIME', '1M', 'PLATFORM_TEMPLATE', '{}', '0.1.0', 'v1', 0, '[]', 'America/New_York', 'POINTS_MT5')")
            .execute(&mut *blocking_tx)
            .await
            .expect("el INSERT del bloqueador debe ejecutar dentro de su propia transacción");

        let repository = MappingDeclarationRepository::new(&pool, &clock);
        let result = repository.record(sample_new_declaration("contended-file")).await;

        // Libera el bloqueo para no dejar el pool colgado tras la prueba.
        blocking_tx.rollback().await.expect("el rollback del bloqueador debe completar");

        match result {
            Err(MappingDeclarationRepositoryError::WriteContention { attempts }) => {
                assert_eq!(attempts, MAX_RECORD_ATTEMPTS, "debe agotar EXACTAMENTE el máximo de intentos configurado");
            }
            other => panic!("se esperaba WriteContention tras agotar los reintentos, se obtuvo: {other:?}"),
        }
    }

    // ── Prueba obligatoria 3/4: clasificador directo (el hueco histórico) ──

    /// Ejercita `is_transient_write_conflict` LLAMÁNDOLO DIRECTAMENTE con
    /// sus tres casos -- nunca verificando el mensaje de un `INSERT` crudo
    /// "indirectamente" (Hueco 1 de `ledger-pattern.md`).
    #[tokio::test(flavor = "multi_thread")]
    async fn is_transient_write_conflict_distinguishes_permanent_pk_from_transient_sequence_and_non_database_errors() {
        let (pool, _dir) = temp_pool().await;

        let insert = |id: &'static str, event_sequence_id: i64| {
            let pool = pool.clone();
            async move {
                sqlx::query(
                    "INSERT INTO import_mapping_declarations (id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, source_file_name, source_file_hash, instrument_id, venue, bar_type, timeframe_value, mapping_origin, column_mapping_json, transform_engine_version, canonical_schema_version, accepted_at, preview_transformed_rows_json, source_timezone_declared, spread_reported_unit_declared) VALUES (?, 0, 0, 'h', NULL, ?, 'f.csv', ?, 'EURUSD', 'PEPPERSTONE', 'TIME', '1M', 'PLATFORM_TEMPLATE', '{}', '0.1.0', 'v1', 0, '[]', 'America/New_York', 'POINTS_MT5')",
                )
                .bind(id)
                .bind(event_sequence_id)
                .bind(format!("hash-{id}-{event_sequence_id}"))
                .execute(&pool)
                .await
            }
        };

        insert("row-a", 1).await.expect("la primera inserción debe pasar");

        // Caso 1 -- PERMANENTE: mismo `id`, distinto `event_sequence_id`. El
        // mensaje de SQLite menciona la PK, NO `event_sequence_id`.
        let pk_violation = insert("row-a", 2).await.expect_err("debe violar la PK");
        let pk_error = MappingDeclarationRepositoryError::from(pk_violation);
        assert!(
            !is_transient_write_conflict(&pk_error),
            "una violación UNIQUE de la PK (no de event_sequence_id) es PERMANENTE, nunca transitoria"
        );

        // Caso 2 -- TRANSITORIO: `id` distinto, MISMO `event_sequence_id`.
        let seq_violation = insert("row-b", 1).await.expect_err("debe violar el UNIQUE de event_sequence_id");
        let seq_error = MappingDeclarationRepositoryError::from(seq_violation);
        assert!(
            is_transient_write_conflict(&seq_error),
            "una violación UNIQUE de event_sequence_id es TRANSITORIA -- debe reintentarse"
        );

        // Caso 3 -- error que NO es de base de datos (WriteContention ya
        // agotada): jamás transitorio, sea cual sea el conteo.
        let non_database = MappingDeclarationRepositoryError::WriteContention { attempts: 5 };
        assert!(!is_transient_write_conflict(&non_database), "WriteContention ya agotada no es un error transitorio reintentable");
    }

    // ── Prueba obligatoria 4/4: fidelidad de la fila devuelta ────────────

    /// La segunda declaración sellada debe encadenar con la primera:
    /// `audit_chain_hash` == `audit_hash` de la anterior, y ambos hashes son
    /// DISTINTOS entre sí (contenido distinto -- nunca `..current.clone()`
    /// disfrazado de fila nueva).
    #[tokio::test(flavor = "multi_thread")]
    async fn second_sealed_declaration_chains_to_first_with_fresh_distinct_hash() {
        let (pool, _dir) = temp_pool().await;
        let clock = DeterministicClock::new(0, 1);
        let repository = MappingDeclarationRepository::new(&pool, &clock);

        let first = repository.record(sample_new_declaration("file-one")).await.expect("primera declaración debe sellar");
        let second = repository.record(sample_new_declaration("file-two")).await.expect("segunda declaración debe sellar");

        assert_eq!(first.audit_chain_hash, None, "la fila génesis debe tener audit_chain_hash NULL, sin sentinel");
        assert_eq!(
            second.audit_chain_hash.as_deref(),
            Some(first.audit_hash.as_str()),
            "la segunda fila debe encadenar EXACTAMENTE con el audit_hash de la primera"
        );
        assert_ne!(first.audit_hash, second.audit_hash, "contenido distinto (source_file_hash distinto) debe producir audit_hash distinto");
        assert_eq!(second.event_sequence_id, first.event_sequence_id + 1);
    }

    // ── Idempotencia: buscar por hash de archivo ─────────────────────────

    #[tokio::test(flavor = "multi_thread")]
    async fn find_by_source_file_hash_returns_none_when_absent_and_some_when_sealed() {
        let (pool, _dir) = temp_pool().await;
        let clock = DeterministicClock::new(0, 1);
        let repository = MappingDeclarationRepository::new(&pool, &clock);

        assert!(repository.find_by_source_file_hash("nunca-sellado").await.unwrap().is_none());

        repository.record(sample_new_declaration("archivo-x")).await.unwrap();
        let found = repository.find_by_source_file_hash("archivo-x").await.unwrap();
        assert!(found.is_some(), "tras sellar, debe encontrarse por su source_file_hash");
    }

    // ── Criterio de aceptación #6: inmutabilidad a nivel de base de datos ──

    /// Los triggers `trg_import_mapping_declarations_no_update`/`_no_delete`
    /// (migración 0023) deben rechazar CUALQUIER intento de `UPDATE`/
    /// `DELETE` sobre una fila ya sellada -- la inmutabilidad del ledger no
    /// puede depender de que el repositorio nunca emita esas sentencias
    /// (eso sería una convención, no una garantía).
    #[tokio::test(flavor = "multi_thread")]
    async fn import_mapping_declarations_rejects_update_and_delete_at_database_level() {
        let (pool, _dir) = temp_pool().await;
        let clock = DeterministicClock::new(0, 1);
        let repository = MappingDeclarationRepository::new(&pool, &clock);
        let sealed = repository.record(sample_new_declaration("archivo-inmutable")).await.unwrap();

        let update_result = sqlx::query("UPDATE import_mapping_declarations SET source_file_name = 'otro.csv' WHERE id = ?")
            .bind(&sealed.id)
            .execute(&pool)
            .await;
        let update_err = update_result.expect_err("un UPDATE sobre el ledger debe ser rechazado por la base de datos");
        assert!(
            update_err.to_string().to_lowercase().contains("append-only"),
            "el mensaje de rechazo debe explicar por qué (append-only), no ser un error genérico: {update_err}"
        );

        let delete_result = sqlx::query("DELETE FROM import_mapping_declarations WHERE id = ?").bind(&sealed.id).execute(&pool).await;
        let delete_err = delete_result.expect_err("un DELETE sobre el ledger debe ser rechazado por la base de datos");
        assert!(delete_err.to_string().to_lowercase().contains("append-only"));

        // La fila sigue intacta después de ambos intentos rechazados.
        let still_there = repository.find_by_source_file_hash("archivo-inmutable").await.unwrap();
        assert!(still_there.is_some(), "la fila debe seguir existiendo, intacta, tras los intentos rechazados");
    }

    // ── Gate de mutación, Grupo 8: mensajes de error ─────────────────────

    /// Mata `persistence.rs:33` (`Display::fmt` → `Ok(Default::default())`,
    /// que produciría un mensaje VACÍO): el texto debe incluir el conteo
    /// exacto de intentos y explicar la causa (contención), no ser genérico.
    #[test]
    fn write_contention_display_message_includes_attempt_count_and_cause() {
        let err = MappingDeclarationRepositoryError::WriteContention { attempts: 7 };
        let message = err.to_string();
        assert!(message.contains('7'), "el mensaje debe incluir el conteo exacto de intentos: {message}");
        assert!(
            message.to_lowercase().contains("contención"),
            "el mensaje debe explicar la causa (contención de escritura), no estar vacío: {message}"
        );
    }

    /// Mata `persistence.rs:47` (`Error::source` → siempre `None`):
    /// `WriteContention` no envuelve ningún error subyacente (`None` es
    /// CORRECTO ahí), pero la variante `Database` SÍ debe encadenar el
    /// `sqlx::Error` real que envuelve -- si el mutante siempre devolviera
    /// `None`, esta mitad de la prueba lo atraparía.
    #[tokio::test(flavor = "multi_thread")]
    async fn error_source_is_none_for_write_contention_but_some_for_database_variant() {
        let contention = MappingDeclarationRepositoryError::WriteContention { attempts: 3 };
        assert!(std::error::Error::source(&contention).is_none(), "WriteContention no envuelve ningún error subyacente");

        let (pool, _dir) = temp_pool().await;
        // Provoca un error real de sqlx (violación UNIQUE de la PK) para
        // envolver un `sqlx::Error` genuino -- no se construye a mano.
        sqlx::query(
            "INSERT INTO import_mapping_declarations (id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, source_file_name, source_file_hash, instrument_id, venue, bar_type, timeframe_value, mapping_origin, column_mapping_json, transform_engine_version, canonical_schema_version, accepted_at, preview_transformed_rows_json, source_timezone_declared, spread_reported_unit_declared) VALUES ('dup-pk', 0, 0, 'h', NULL, 1, 'f.csv', 'hash-1', 'EURUSD', 'PEPPERSTONE', 'TIME', '1M', 'PLATFORM_TEMPLATE', '{}', '0.1.0', 'v1', 0, '[]', 'America/New_York', 'POINTS_MT5')",
        )
        .execute(&pool)
        .await
        .unwrap();
        let dup_err = sqlx::query(
            "INSERT INTO import_mapping_declarations (id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, source_file_name, source_file_hash, instrument_id, venue, bar_type, timeframe_value, mapping_origin, column_mapping_json, transform_engine_version, canonical_schema_version, accepted_at, preview_transformed_rows_json, source_timezone_declared, spread_reported_unit_declared) VALUES ('dup-pk', 0, 0, 'h', NULL, 2, 'f.csv', 'hash-2', 'EURUSD', 'PEPPERSTONE', 'TIME', '1M', 'PLATFORM_TEMPLATE', '{}', '0.1.0', 'v1', 0, '[]', 'America/New_York', 'POINTS_MT5')",
        )
        .execute(&pool)
        .await
        .unwrap_err();

        let wrapped = MappingDeclarationRepositoryError::from(dup_err);
        assert!(wrapped.to_string().contains("database error"), "Display de la variante Database debe mencionar el error de base de datos: {wrapped}");
        assert!(std::error::Error::source(&wrapped).is_some(), "la variante Database DEBE encadenar el error subyacente de sqlx");
    }
}
