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
}

impl std::fmt::Display for DownloadRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadRepositoryError::Database(err) => {
                write!(f, "download repository database error: {err}")
            }
        }
    }
}

impl std::error::Error for DownloadRepositoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DownloadRepositoryError::Database(err) => Some(err),
        }
    }
}

impl From<sqlx::Error> for DownloadRepositoryError {
    fn from(err: sqlx::Error) -> Self {
        DownloadRepositoryError::Database(err)
    }
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
    /// Genera automáticamente: UUID, timestamps (del Clock inyectado),
    /// `audit_hash` (SHA-256 del contenido) y encadenamiento con el
    /// registro previo (`audit_chain_hash`). El `event_sequence_id` es el
    /// siguiente número monótono en la tabla.
    pub async fn record(
        &self,
        new: NewDownloadRecord,
    ) -> Result<DownloadRecord, DownloadRepositoryError> {
        // Lee el hash del último registro para construir la cadena de auditoría.
        let previous_hash = self.load_latest_hash().await?;
        let id = Uuid::new_v4().to_string();
        let now_ns = self.clock.timestamp_ns();
        // El event_sequence_id es 1 para el primer registro, luego monótonamente creciente.
        let event_sequence_id = self.next_sequence_id().await?;

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
        .execute(self.pool)
        .await?;

        Ok(DownloadRecord {
            id,
            created_at: now_ns,
            updated_at: now_ns,
            audit_hash,
            audit_chain_hash: previous_hash,
            event_sequence_id,
            data_snapshot_id: new.data_snapshot_id,
            logic_hash: new.logic_hash,
            node_id: new.node_id,
            process_id: new.process_id,
            source_endpoint: new.source_endpoint,
        })
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

    /// Lee el `audit_hash` del último registro insertado, o `None` si la
    /// tabla está vacía (el próximo registro será el primero de la cadena).
    async fn load_latest_hash(&self) -> Result<Option<String>, DownloadRepositoryError> {
        let row = sqlx::query(
            "SELECT audit_hash FROM sovereign_download_records \
             ORDER BY event_sequence_id DESC LIMIT 1",
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(row.map(|r| r.get::<String, _>(0)))
    }

    /// Devuelve el próximo `event_sequence_id` (1 para la primera fila,
    /// luego incrementa monótonamente).
    async fn next_sequence_id(&self) -> Result<i64, DownloadRepositoryError> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(event_sequence_id), 0) FROM sovereign_download_records",
        )
        .fetch_one(self.pool)
        .await?;

        let max: i64 = row.get(0);
        Ok(max + 1)
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
