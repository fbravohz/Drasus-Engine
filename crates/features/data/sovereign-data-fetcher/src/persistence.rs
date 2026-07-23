//! [SHELL] Repositorio de persistencia para el Sovereign Data Fetcher.
//!
//! Envuelve la tabla `sovereign_download_records` (migración 0006).
//! Usa el pool de `shared` — este módulo nunca crea su propio pool;
//! siempre recibe uno ya inicializado y migrado desde el exterior.

use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use shared::public_interface::Clock;

use crate::schemas::{DownloadRecord, NewDownloadRecord};

// ── Error del repositorio ────────────────────────────────────────────────────

/// Error que devuelven las operaciones del repositorio de descarga.
#[derive(Debug)]
pub enum DownloadRepositoryError {
    /// La operación subyacente de SQLite falló.
    Database(sqlx::Error),
    /// [`DownloadRepository::record`] no pudo completarse tras agotar los
    /// reintentos ante contención de escritura transitoria -- el registro
    /// de descarga NO se descartó en silencio (regla "Atomicidad de
    /// ledgers append-only", rust-engineer/SKILL.md §4).
    WriteContention { attempts: u32 },
}

impl std::fmt::Display for DownloadRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadRepositoryError::Database(err) => {
                write!(f, "download repository database error: {err}")
            }
            DownloadRepositoryError::WriteContention { attempts } => {
                write!(f, "no se pudo registrar la descarga tras {attempts} intentos por contención de escritura")
            }
        }
    }
}

impl std::error::Error for DownloadRepositoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DownloadRepositoryError::Database(err) => Some(err),
            DownloadRepositoryError::WriteContention { .. } => None,
        }
    }
}

impl From<sqlx::Error> for DownloadRepositoryError {
    fn from(err: sqlx::Error) -> Self {
        DownloadRepositoryError::Database(err)
    }
}

/// Número máximo de intentos de [`DownloadRepository::record`] ante
/// contención de escritura transitoria antes de rendirse con
/// [`DownloadRepositoryError::WriteContention`]. Mismo valor y misma
/// justificación que los demás ledgers append-only del sistema
/// (`crate::persistence::audit_log::MAX_APPEND_ATTEMPTS` en `shared`).
const MAX_RECORD_ATTEMPTS: u32 = 5;

/// Decide si un error de [`DownloadRepository::record`] es una contención
/// de escritura TRANSITORIA -- algo que reintentar (re-derivando el
/// `event_sequence_id` y reinsertando) puede resolver, sin descartar el
/// registro. Mismo criterio que
/// `shared::persistence::audit_log::is_transient_write_conflict`.
fn is_transient_write_conflict(error: &DownloadRepositoryError) -> bool {
    let DownloadRepositoryError::Database(sqlx::Error::Database(db)) = error else {
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

// ── Repositorio ──────────────────────────────────────────────────────────────

/// Repositorio para la tabla `sovereign_download_records`.
///
/// Constrúyelo con un `SqlitePool` ya migrado (incluye la migración 0006)
/// y una implementación de `Clock`. No crea ni migra la base de datos
/// por su cuenta — esa responsabilidad pertenece al crate `shared`.
pub struct DownloadRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> DownloadRepository<'a> {
    /// Crea un repositorio asociado al pool y clock dados.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Inserta un nuevo registro de descarga y lo devuelve ya persistido.
    ///
    /// Genera automáticamente: UUID v7, timestamps (del Clock inyectado),
    /// `audit_hash` (SHA-256 del contenido) y encadenamiento con el
    /// registro previo (`audit_chain_hash`). El `event_sequence_id` es el
    /// siguiente número monótono en la tabla.
    ///
    /// ## Atomicidad bajo concurrencia (regla "Atomicidad de ledgers append-only")
    ///
    /// El *read-then-write* (leer la cola de la cadena y el `INSERT` final)
    /// ocurre dentro de UNA sola transacción `BEGIN IMMEDIATE` -- ver
    /// [`Self::try_record_once`]. Sin esa transacción, dos escritores
    /// concurrentes derivarían el mismo `event_sequence_id`, el `UNIQUE`
    /// (migración `0006`) rechazaría a uno y su registro se PERDERÍA. Ante
    /// contención transitoria se reintenta hasta [`MAX_RECORD_ATTEMPTS`]
    /// veces re-derivando la secuencia; el registro NUNCA se descarta en
    /// silencio (si se agotan los reintentos se devuelve
    /// [`DownloadRepositoryError::WriteContention`]).
    pub async fn record(
        &self,
        new: NewDownloadRecord,
    ) -> Result<DownloadRecord, DownloadRepositoryError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_record_once(&new).await {
                Ok(record) => return Ok(record),
                Err(error) => {
                    // Solo se reintenta ante contención de escritura
                    // transitoria; cualquier otro error se propaga de
                    // inmediato.
                    if is_transient_write_conflict(&error) {
                        if attempt < MAX_RECORD_ATTEMPTS {
                            continue;
                        }
                        // Agotados los reintentos: error tipado, NUNCA
                        // pérdida silenciosa del registro.
                        return Err(DownloadRepositoryError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    /// Un intento único de [`Self::record`], dentro de una transacción
    /// `BEGIN IMMEDIATE` -- toma el lock de escritura de ENTRADA, evitando
    /// tanto la intercalación de otro escritor entre la lectura de la cola
    /// y el `INSERT` como el interbloqueo de upgrade de dos transacciones
    /// DEFERRED.
    async fn try_record_once(
        &self,
        new: &NewDownloadRecord,
    ) -> Result<DownloadRecord, DownloadRepositoryError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        // Lectura (DENTRO de la transacción) -- la cola actual de la cadena
        // GLOBAL de sovereign_download_records, para derivar el próximo
        // event_sequence_id y encadenar el audit_hash.
        let tail_row = sqlx::query(
            "SELECT audit_hash, event_sequence_id FROM sovereign_download_records \
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

        // Calcula el hash de auditoría del nuevo registro.
        let audit_hash = compute_audit_hash(
            &id,
            now_ns,
            event_sequence_id,
            previous_hash.as_deref(),
            new.data_snapshot_id.as_deref(),
            new.logic_hash.as_deref(),
            new.node_id.as_deref(),
            new.process_id.as_deref(),
            &new.source_endpoint,
        );

        // Escritura (DENTRO de la transacción) -- el INSERT que cierra el
        // read-then-write atómico.
        sqlx::query(
            "INSERT INTO sovereign_download_records (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id,\
                data_snapshot_id, logic_hash, node_id, process_id, source_endpoint\
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&previous_hash)
        .bind(event_sequence_id)
        .bind(&new.data_snapshot_id)
        .bind(&new.logic_hash)
        .bind(&new.node_id)
        .bind(&new.process_id)
        .bind(&new.source_endpoint)
        .execute(&mut *tx)
        .await?;

        // Confirma la transacción: recién aquí el lock de escritura se
        // libera y la fila se hace visible a otros escritores.
        tx.commit().await?;

        Ok(DownloadRecord {
            id,
            created_at: now_ns,
            updated_at: now_ns,
            audit_hash,
            audit_chain_hash: previous_hash,
            event_sequence_id,
            data_snapshot_id: new.data_snapshot_id.clone(),
            logic_hash: new.logic_hash.clone(),
            node_id: new.node_id.clone(),
            process_id: new.process_id.clone(),
            source_endpoint: new.source_endpoint.clone(),
        })
    }

    /// Lista todos los registros de descarga, del más reciente al más antiguo.
    ///
    /// Sin paginación — adecuado para el volumen de EPIC-1 (decenas de
    /// registros). Si la tabla está vacía, devuelve un Vec vacío.
    pub async fn list_all(&self) -> Result<Vec<DownloadRecord>, DownloadRepositoryError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
             data_snapshot_id, logic_hash, node_id, process_id, source_endpoint \
             FROM sovereign_download_records ORDER BY event_sequence_id DESC",
        )
        .fetch_all(self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| DownloadRecord {
                id: r.get("id"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                audit_hash: r.get("audit_hash"),
                audit_chain_hash: r.get("audit_chain_hash"),
                event_sequence_id: r.get("event_sequence_id"),
                data_snapshot_id: r.get("data_snapshot_id"),
                logic_hash: r.get("logic_hash"),
                node_id: r.get("node_id"),
                process_id: r.get("process_id"),
                source_endpoint: r.get("source_endpoint"),
            })
            .collect())
    }

    /// Carga un registro de descarga por `id`, o `None` si no existe.
    pub async fn find(
        &self,
        id: &str,
    ) -> Result<Option<DownloadRecord>, DownloadRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
             data_snapshot_id, logic_hash, node_id, process_id, source_endpoint \
             FROM sovereign_download_records WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        Ok(row.map(|r| DownloadRecord {
            id: r.get("id"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            audit_hash: r.get("audit_hash"),
            audit_chain_hash: r.get("audit_chain_hash"),
            event_sequence_id: r.get("event_sequence_id"),
            data_snapshot_id: r.get("data_snapshot_id"),
            logic_hash: r.get("logic_hash"),
            node_id: r.get("node_id"),
            process_id: r.get("process_id"),
            source_endpoint: r.get("source_endpoint"),
        }))
    }

}

// ── Hash de auditoría ────────────────────────────────────────────────────────

/// Calcula el hash SHA-256 determinista para una fila de descarga.
///
/// El hash cubre todos los campos de contenido del registro (excluyendo el
/// propio `audit_hash` para evitar circularidad). Se encadena al hash del
/// registro previo mediante `audit_chain_hash` (o a la constante GENESIS
/// si es el primer registro).
#[allow(clippy::too_many_arguments)]
fn compute_audit_hash(
    id: &str,
    created_at: i64,
    event_sequence_id: i64,
    previous_hash: Option<&str>,
    data_snapshot_id: Option<&str>,
    logic_hash: Option<&str>,
    node_id: Option<&str>,
    process_id: Option<&str>,
    source_endpoint: &str,
) -> String {
    // El separador U+001F (ASCII Unit Separator) es el mismo que usa
    // `shared::domain::audit_log` — mantiene consistencia en toda la cadena.
    const SEP: char = '\u{1F}';
    // Constante de génesis del shared crate: la cadena de hash para el
    // primer elemento de cualquier secuencia de auditoría.
    const GENESIS: &str = "GENESIS";

    let mut buf = String::new();
    let mut push = |s: &str| {
        buf.push_str(s);
        buf.push(SEP);
    };

    push(id);
    push(&created_at.to_string());
    push(&event_sequence_id.to_string());
    push(previous_hash.unwrap_or(GENESIS));
    push(data_snapshot_id.unwrap_or(""));
    push(logic_hash.unwrap_or(""));
    push(node_id.unwrap_or(""));
    push(process_id.unwrap_or(""));
    push(source_endpoint);

    let digest = Sha256::digest(buf.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

// ── Pruebas del clasificador de contención (DEBT-032) ────────────────────────
//
// La prueba original (en `tests/integration_tests.rs`, ya eliminada) NUNCA
// llamaba a `is_transient_write_conflict`: verificaba el mensaje de un
// INSERT crudo "por fuera", con un comentario que admitía que era una
// comprobación indirecta. El gate de mutación demostró que el clasificador
// podía mutarse a devolver siempre `true` sin que ninguna prueba fallara.
//
// Estas tres pruebas llaman a la función DIRECTAMENTE, por su nombre, con
// sus tres casos (molde canónico `.agents/knowledge/ledger-pattern.md`
// punto 7 -- "clasificador directo"). Viven aquí, dentro del propio módulo,
// porque `is_transient_write_conflict` es privada al crate: solo un test
// en el mismo archivo puede invocarla sin exponerla en la API pública.
#[cfg(test)]
mod tests {
    use super::*;
    use shared::public_interface::{create_pool, run_migrations};

    /// Base de datos SQLite en memoria con la migración 0006 ya aplicada.
    async fn migrated_pool() -> SqlitePool {
        let pool = create_pool("sqlite::memory:")
            .await
            .expect("conectar la base de datos en memoria");
        run_migrations(&pool).await.expect("aplicar migraciones");
        pool
    }

    /// Inserta una fila cruda en `sovereign_download_records`, saltándose el
    /// repositorio -- para provocar a propósito las violaciones UNIQUE que
    /// el clasificador debe distinguir.
    async fn insert_raw(
        pool: &SqlitePool,
        id: &str,
        event_sequence_id: i64,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        sqlx::query(
            "INSERT INTO sovereign_download_records \
             (id, created_at, updated_at, audit_hash, event_sequence_id, source_endpoint) \
             VALUES (?, 1, 1, 'h', ?, 'https://example.invalid')",
        )
        .bind(id)
        .bind(event_sequence_id)
        .execute(pool)
        .await
    }

    /// CASO 1 (PERMANENTE): una violación de la PRIMARY KEY `id` -- mismo
    /// `id`, distinto `event_sequence_id` -- reintentar NUNCA la resuelve
    /// (el segundo intento produciría el mismo `id` de nuevo). El
    /// clasificador debe devolver `false`.
    #[tokio::test]
    async fn is_transient_write_conflict_is_false_for_primary_key_violation() {
        let pool = migrated_pool().await;
        insert_raw(&pool, "dup-id", 1).await.expect("primera fila válida");
        let err = insert_raw(&pool, "dup-id", 2)
            .await
            .expect_err("la segunda fila debe violar la PRIMARY KEY id");

        let error = DownloadRepositoryError::Database(err);
        assert!(
            !is_transient_write_conflict(&error),
            "una violación de PK (no de event_sequence_id) es PERMANENTE, no transitoria"
        );
    }

    /// CASO 2 (TRANSITORIO): una colisión de secuencia -- dos filas con
    /// `id` distinto pero el MISMO `event_sequence_id` -- SÍ se resuelve
    /// reintentando (re-derivar la secuencia y reinsertar). El clasificador
    /// debe devolver `true`.
    #[tokio::test]
    async fn is_transient_write_conflict_is_true_for_sequence_collision() {
        let pool = migrated_pool().await;
        insert_raw(&pool, "id-a", 1).await.expect("primera fila válida");
        let err = insert_raw(&pool, "id-b", 1)
            .await
            .expect_err("la segunda fila debe violar UNIQUE(event_sequence_id)");

        let error = DownloadRepositoryError::Database(err);
        assert!(
            is_transient_write_conflict(&error),
            "una colisión de event_sequence_id (mismo valor, distinto id) es TRANSITORIA"
        );
    }

    /// CASO 3 (control, no-BD): un error que ni siquiera envuelve un
    /// `sqlx::Error::Database` -- fija la rama temprana `let ... else` del
    /// clasificador. Nunca puede clasificarse como transitorio.
    #[test]
    fn is_transient_write_conflict_is_false_for_non_database_error() {
        let error = DownloadRepositoryError::WriteContention { attempts: 5 };
        assert!(
            !is_transient_write_conflict(&error),
            "un error que no envuelve sqlx::Error::Database jamás es contención transitoria"
        );
    }
}
