//! [SHELL] Repositorio de persistencia para el Hybrid Data Transformer.
//!
//! Envuelve la tabla `transformation_records` (migración 0021). Usa el pool
//! de `shared` — este módulo nunca crea su propio pool; siempre recibe uno
//! ya inicializado y migrado desde el exterior.
//!
//! Mismo patrón de atomicidad append-only que
//! `sovereign_data_fetcher::persistence::DownloadRepository` (regla
//! "Atomicidad de ledgers append-only", `rust-engineer/SKILL.md` §4):
//! read-then-write (derivar `event_sequence_id` + leer la cola de la cadena)
//! dentro de UNA sola transacción `BEGIN IMMEDIATE`, con reintento acotado
//! ante contención transitoria.

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use shared::public_interface::Clock;

use crate::schemas::{NewTransformationRecord, TransformationRecord};

// ── Error del repositorio ────────────────────────────────────────────────────

/// Error que devuelven las operaciones del repositorio de transformaciones.
#[derive(Debug)]
pub enum TransformationRepositoryError {
    /// La operación subyacente de SQLite falló.
    Database(sqlx::Error),
    /// [`TransformationRepository::record`] no pudo completarse tras agotar
    /// los reintentos ante contención de escritura transitoria — el registro
    /// de transformación NO se descartó en silencio.
    WriteContention { attempts: u32 },
}

impl std::fmt::Display for TransformationRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransformationRepositoryError::Database(err) => {
                write!(f, "transformation repository database error: {err}")
            }
            TransformationRepositoryError::WriteContention { attempts } => write!(
                f,
                "no se pudo registrar la transformación tras {attempts} intentos por contención de escritura"
            ),
        }
    }
}

impl std::error::Error for TransformationRepositoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TransformationRepositoryError::Database(err) => Some(err),
            TransformationRepositoryError::WriteContention { .. } => None,
        }
    }
}

impl From<sqlx::Error> for TransformationRepositoryError {
    fn from(err: sqlx::Error) -> Self {
        TransformationRepositoryError::Database(err)
    }
}

/// Número máximo de intentos de [`TransformationRepository::record`] ante
/// contención de escritura transitoria antes de rendirse con
/// [`TransformationRepositoryError::WriteContention`]. Mismo valor y misma
/// justificación que el resto de los ledgers append-only del sistema.
const MAX_RECORD_ATTEMPTS: u32 = 5;

/// Decide si un error de [`TransformationRepository::record`] es una
/// contención de escritura TRANSITORIA — algo que reintentar (re-derivando
/// el `event_sequence_id` y reinsertando) puede resolver, sin descartar el
/// registro. Mismo criterio que
/// `sovereign_data_fetcher::persistence::is_transient_write_conflict`.
fn is_transient_write_conflict(error: &TransformationRepositoryError) -> bool {
    let TransformationRepositoryError::Database(sqlx::Error::Database(db)) = error else {
        return false;
    };

    let message = db.message().to_lowercase();
    if message.contains("database is locked") || message.contains("database table is locked") {
        return true;
    }

    db.is_unique_violation() && message.contains("event_sequence_id")
}

// ── Repositorio ──────────────────────────────────────────────────────────────

/// Repositorio para la tabla `transformation_records`.
///
/// Constrúyelo con un `SqlitePool` ya migrado (incluye la migración 0021) y
/// una implementación de `Clock`.
pub struct TransformationRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> TransformationRepository<'a> {
    /// Crea un repositorio asociado al pool y clock dados.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Inserta un nuevo registro de transformación y lo devuelve ya
    /// persistido.
    ///
    /// Genera automáticamente: UUID v7, timestamps (del Clock inyectado) y
    /// el encadenamiento con el registro previo (`audit_chain_hash` = el
    /// `audit_hash` de la fila anterior; `NULL` en la fila génesis). El
    /// `audit_hash` de la fila NO se recalcula aquí — llega ya calculado
    /// desde el Núcleo sobre el contenido de las barras ordenadas
    /// (`new.content_audit_hash`, criterio 5: reproducible entre corridas).
    /// El `event_sequence_id` es el siguiente número monótono en la tabla.
    ///
    /// ## Atomicidad bajo concurrencia
    ///
    /// El *read-then-write* (leer la cola de la cadena y el `INSERT` final)
    /// ocurre dentro de UNA sola transacción `BEGIN IMMEDIATE` — ver
    /// [`Self::try_record_once`]. Ante contención transitoria se reintenta
    /// hasta [`MAX_RECORD_ATTEMPTS`] veces re-derivando la secuencia; el
    /// registro NUNCA se descarta en silencio.
    pub async fn record(
        &self,
        new: NewTransformationRecord,
    ) -> Result<TransformationRecord, TransformationRepositoryError> {
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
                        return Err(TransformationRepositoryError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    /// Un intento único de [`Self::record`], dentro de una transacción
    /// `BEGIN IMMEDIATE` — toma el lock de escritura de ENTRADA, evitando
    /// tanto la intercalación de otro escritor entre la lectura de la cola y
    /// el `INSERT` como el interbloqueo de upgrade de dos transacciones
    /// DEFERRED.
    async fn try_record_once(
        &self,
        new: &NewTransformationRecord,
    ) -> Result<TransformationRecord, TransformationRepositoryError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        let tail_row = sqlx::query(
            "SELECT audit_hash, event_sequence_id FROM transformation_records \
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
            "INSERT INTO transformation_records (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id,\
                data_snapshot_id, transformation_id, logic_hash, parent_id,\
                node_id, process_id, session_id,\
                transform_duration_ns, rows_in, rows_out, schema_version\
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&new.content_audit_hash)
        .bind(&previous_hash)
        .bind(event_sequence_id)
        .bind(&new.data_snapshot_id)
        .bind(&new.transformation_id)
        .bind(&new.logic_hash)
        .bind(&new.parent_id)
        .bind(&new.node_id)
        .bind(&new.process_id)
        .bind(&new.session_id)
        .bind(new.transform_duration_ns)
        .bind(new.rows_in)
        .bind(new.rows_out)
        .bind(&new.schema_version)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(TransformationRecord {
            id,
            created_at: now_ns,
            updated_at: now_ns,
            audit_hash: new.content_audit_hash.clone(),
            audit_chain_hash: previous_hash,
            event_sequence_id,
            data_snapshot_id: new.data_snapshot_id.clone(),
            transformation_id: new.transformation_id.clone(),
            logic_hash: new.logic_hash.clone(),
            parent_id: new.parent_id.clone(),
            node_id: new.node_id.clone(),
            process_id: new.process_id.clone(),
            session_id: new.session_id.clone(),
            transform_duration_ns: new.transform_duration_ns,
            rows_in: new.rows_in,
            rows_out: new.rows_out,
            schema_version: new.schema_version.clone(),
        })
    }

    /// Lista todos los registros de transformación, del más reciente al más
    /// antiguo. Sin paginación — adecuado para el volumen de EPIC-1.
    pub async fn list_all(&self) -> Result<Vec<TransformationRecord>, TransformationRepositoryError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
             data_snapshot_id, transformation_id, logic_hash, parent_id, \
             node_id, process_id, session_id, \
             transform_duration_ns, rows_in, rows_out, schema_version \
             FROM transformation_records ORDER BY event_sequence_id DESC",
        )
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter().map(row_to_record).collect())
    }

    /// Carga un registro de transformación por `id`, o `None` si no existe.
    pub async fn find(&self, id: &str) -> Result<Option<TransformationRecord>, TransformationRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
             data_snapshot_id, transformation_id, logic_hash, parent_id, \
             node_id, process_id, session_id, \
             transform_duration_ns, rows_in, rows_out, schema_version \
             FROM transformation_records WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        Ok(row.map(row_to_record))
    }
}

/// Mapea una fila cruda de SQLite a [`TransformationRecord`]. Factor común
/// entre `list_all` y `find` para no duplicar el mapeo campo por campo.
fn row_to_record(r: sqlx::sqlite::SqliteRow) -> TransformationRecord {
    TransformationRecord {
        id: r.get("id"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
        audit_hash: r.get("audit_hash"),
        audit_chain_hash: r.get("audit_chain_hash"),
        event_sequence_id: r.get("event_sequence_id"),
        data_snapshot_id: r.get("data_snapshot_id"),
        transformation_id: r.get("transformation_id"),
        logic_hash: r.get("logic_hash"),
        parent_id: r.get("parent_id"),
        node_id: r.get("node_id"),
        process_id: r.get("process_id"),
        session_id: r.get("session_id"),
        transform_duration_ns: r.get("transform_duration_ns"),
        rows_in: r.get("rows_in"),
        rows_out: r.get("rows_out"),
        schema_version: r.get("schema_version"),
    }
}
