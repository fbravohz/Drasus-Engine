//! [SHELL] Repositorios de persistencia para Instance Continuity
//! (`docs/features/instance-continuity.md`, ADR-0146 -- cimiento #11,
//! ADR-0141, ADR-0020, migración `0017_instance_continuity.sql`,
//! STORY-039).
//!
//! DOS repositorios, uno por tabla (ADR-0146 regla obligatoria #5):
//! - [`BackupRegistryRepository`]: APPEND-ONLY ATÓMICO para
//!   `instance_backups` -- mismo patrón que
//!   [`crate::persistence::enriched_domain_events::DomainEventRepository`]
//!   (`BEGIN IMMEDIATE` + reintento acotado ante contención transitoria).
//! - [`CustodyRepository`]: MUTABLE para `custody_state` -- mismo patrón
//!   de concurrencia optimista que
//!   [`crate::persistence::central_identity::AccountRepository::update_email_verification_status`]
//!   (`row_version`), adaptado al nombre de dominio `custody_epoch` que
//!   exige ADR-0146.
//!
//! La lógica pura (KDF, cifrado, filtro de secretos, decisión de
//! titularidad, hash de auditoría encadenado) vive en
//! [`crate::domain::instance_continuity`] -- este módulo solo le da
//! entradas inyectadas y persiste/carga el resultado.

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::audit_log::GENESIS_PREVIOUS_HASH;
use crate::domain::clock::Clock;
use crate::domain::instance_continuity::{
    compute_backup_audit_hash, compute_custody_audit_hash, decide_custody_claim, CustodyClaimError,
    CustodyState,
};

// ── Registro de respaldos (APPEND-ONLY ATÓMICO) ─────────────────────────────

/// Errores de [`BackupRegistryRepository`].
#[derive(Debug, thiserror::Error)]
pub enum BackupRegistryRepositoryError {
    /// La operación subyacente de SQLite falló.
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// El append no pudo completarse tras agotar los reintentos ante
    /// contención de escritura transitoria -- el respaldo NUNCA se
    /// descarta en silencio (regla "Atomicidad de ledgers append-only",
    /// rust-engineer/SKILL.md §4).
    #[error("no se pudo registrar el respaldo tras {attempts} intentos por contención de escritura")]
    WriteContention { attempts: u32 },
}

/// Número máximo de intentos ante contención de escritura transitoria
/// antes de rendirse -- mismo umbral que
/// [`crate::persistence::enriched_domain_events::MAX_RECORD_ATTEMPTS`].
const MAX_RECORD_ATTEMPTS: u32 = 5;

/// Decide si un error del repositorio es contención TRANSITORIA (algo que
/// reintentar resuelve) -- mismo criterio que
/// `enriched_domain_events::is_transient_write_conflict`: lock ocupado de
/// SQLite, o colisión de `event_sequence_id` bajo `BEGIN IMMEDIATE`
/// (cinturón-y-tirantes).
fn is_transient_write_conflict(error: &BackupRegistryRepositoryError) -> bool {
    let BackupRegistryRepositoryError::Database(sqlx::Error::Database(db)) = error else {
        return false;
    };

    let message = db.message().to_lowercase();
    if message.contains("database is locked") || message.contains("database table is locked") {
        return true;
    }

    db.is_unique_violation() && message.contains("event_sequence_id")
}

/// Entrada para [`BackupRegistryRepository::record_backup`] -- los
/// metadatos de UN snapshot ya cifrado (el ciphertext en sí NO se persiste
/// aquí: va al adaptador de almacén de objetos diferido; este ledger solo
/// registra el HECHO de que un respaldo ocurrió).
#[derive(Debug, Clone)]
pub struct RecordBackupInput {
    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,
    pub snapshot_at_ns: i64,
    pub blob_hash: String,
    pub blob_size_bytes: i64,
    pub nonce_hex: String,
}

/// Una fila de `instance_backups` ya persistida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceBackupRow {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub event_sequence_id: i64,

    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,

    pub snapshot_at_ns: i64,
    pub blob_hash: String,
    pub blob_size_bytes: i64,
    pub nonce_hex: String,
}

/// Repositorio APPEND-ONLY para `instance_backups`.
pub struct BackupRegistryRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> BackupRegistryRepository<'a> {
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Registra UN respaldo: deriva su posición en la cadena GLOBAL,
    /// computa su hash encadenado y lo persiste como fila nueva. Es la
    /// ÚNICA forma de escribir en `instance_backups` -- no existe
    /// `update`/`delete` (los triggers de la migración los rechazarían de
    /// cualquier forma).
    ///
    /// ## Atomicidad bajo concurrencia
    ///
    /// Todo el *read-then-write* (leer el MAX(`event_sequence_id`) y el
    /// `audit_hash` previo, y el `INSERT` final) ocurre dentro de UNA sola
    /// transacción `BEGIN IMMEDIATE` -- ver [`Self::try_record_backup_once`].
    /// Ante contención transitoria se reintenta hasta [`MAX_RECORD_ATTEMPTS`]
    /// veces re-derivando la secuencia; el respaldo NUNCA se descarta en
    /// silencio.
    pub async fn record_backup(
        &self,
        input: RecordBackupInput,
    ) -> Result<InstanceBackupRow, BackupRegistryRepositoryError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_record_backup_once(&input).await {
                Ok(row) => return Ok(row),
                Err(error) => {
                    if is_transient_write_conflict(&error) {
                        if attempt < MAX_RECORD_ATTEMPTS {
                            continue;
                        }
                        return Err(BackupRegistryRepositoryError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    /// Un intento único del append, dentro de `BEGIN IMMEDIATE` -- toma el
    /// lock de escritura de ENTRADA para que ningún otro escritor pueda
    /// intercalar entre la lectura del MAX(`event_sequence_id`) y el
    /// `INSERT` (mismo razonamiento que
    /// `DomainEventRepository::try_record_event_once`).
    async fn try_record_backup_once(
        &self,
        input: &RecordBackupInput,
    ) -> Result<InstanceBackupRow, BackupRegistryRepositoryError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        let tail_row = sqlx::query(
            "SELECT audit_hash, event_sequence_id \
             FROM instance_backups \
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

        let audit_hash = compute_backup_audit_hash(
            &id,
            now_ns,
            event_sequence_id,
            &previous_audit_hash,
            &input.owner_id,
            &input.institutional_tag,
            &input.node_id,
            input.snapshot_at_ns,
            &input.blob_hash,
            input.blob_size_bytes,
            &input.nonce_hex,
        );

        sqlx::query(
            "INSERT INTO instance_backups (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, snapshot_at, blob_hash, blob_size_bytes, nonce_hex\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
        .bind(input.snapshot_at_ns)
        .bind(&input.blob_hash)
        .bind(input.blob_size_bytes)
        .bind(&input.nonce_hex)
        .execute(&mut *tx)
        .await?;

        // Confirma la transacción: recién aquí el lock de escritura se
        // libera y la fila se hace visible a otros escritores.
        tx.commit().await?;

        Ok(InstanceBackupRow {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash,
            event_sequence_id,
            owner_id: input.owner_id.clone(),
            institutional_tag: input.institutional_tag.clone(),
            node_id: input.node_id.clone(),
            snapshot_at_ns: input.snapshot_at_ns,
            blob_hash: input.blob_hash.clone(),
            blob_size_bytes: input.blob_size_bytes,
            nonce_hex: input.nonce_hex.clone(),
        })
    }

    /// Carga la cadena completa, ordenada por `event_sequence_id`
    /// ascendente -- usada por los tests de integridad de la cadena y por
    /// cualquier consumidor futuro que reconstruya el historial de
    /// respaldos.
    pub async fn load_chain(&self) -> Result<Vec<InstanceBackupRow>, BackupRegistryRepositoryError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, node_id, snapshot_at, blob_hash, blob_size_bytes, nonce_hex \
             FROM instance_backups \
             ORDER BY event_sequence_id ASC",
        )
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter().map(row_to_backup).collect())
    }
}

fn row_to_backup(row: sqlx::sqlite::SqliteRow) -> InstanceBackupRow {
    InstanceBackupRow {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        event_sequence_id: row.get("event_sequence_id"),
        owner_id: row.get("owner_id"),
        institutional_tag: row.get("institutional_tag"),
        node_id: row.get("node_id"),
        snapshot_at_ns: row.get("snapshot_at"),
        blob_hash: row.get("blob_hash"),
        blob_size_bytes: row.get("blob_size_bytes"),
        nonce_hex: row.get("nonce_hex"),
    }
}

// ── Estado de custodia (MUTABLE, concurrencia optimista por custody_epoch) ─

/// Errores de [`CustodyRepository`].
#[derive(Debug, thiserror::Error)]
pub enum CustodyRepositoryError {
    /// La operación subyacente de SQLite falló.
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// Concurrencia optimista a nivel de INSTANCIA (ADR-0146): el reclamo
    /// partió de un `custody_epoch` que ya no es el vigente en disco --
    /// otra máquina reclamó la titularidad primero. La máquina que recibe
    /// este error NUNCA escribe la cadena de auditoría en paralelo.
    #[error(
        "conflicto de custodia: el dueño '{owner_id}' ya no está en el epoch {expected_epoch} \
         -- otra máquina reclamó la titularidad primero"
    )]
    CustodyConflict { owner_id: String, expected_epoch: i64 },
}

impl From<CustodyClaimError> for CustodyRepositoryError {
    /// Traduce el conflicto de la decisión pura del Core al error del
    /// repositorio -- mismo tipo de conflicto, ya sea que lo detecte la
    /// decisión en memoria o la guarda `WHERE custody_epoch = ?` contra la
    /// fila real (defensa en profundidad: ambos protegen la MISMA
    /// invariante).
    fn from(error: CustodyClaimError) -> Self {
        match error {
            CustodyClaimError::CustodyConflict { owner_id, expected_epoch } => {
                CustodyRepositoryError::CustodyConflict { owner_id, expected_epoch }
            }
        }
    }
}

/// Una fila de `custody_state` ya persistida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustodyRow {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub custody_epoch: i64,

    pub owner_id: String,
    pub institutional_tag: String,
    pub titular_node_id: String,
}

/// Entrada para [`CustodyRepository::claim_titular`].
#[derive(Debug, Clone)]
pub struct ClaimTitularInput {
    pub owner_id: String,
    pub institutional_tag: String,
    pub claiming_node_id: String,
    /// El `custody_epoch` que el reclamante CREE vigente. Se ignora en el
    /// reclamo BOOTSTRAP (primera vez que se registra custodia para este
    /// `owner_id` -- no hay epoch previo con el que competir).
    pub expected_epoch: i64,
}

/// Repositorio MUTABLE para `custody_state`.
pub struct CustodyRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> CustodyRepository<'a> {
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Carga la fila de custodia vigente para `owner_id`, o `None` si
    /// nunca se registró custodia para esta cuenta (ninguna máquina es
    /// titular todavía).
    pub async fn find_by_owner(&self, owner_id: &str) -> Result<Option<CustodyRow>, CustodyRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, custody_epoch, \
                    owner_id, institutional_tag, titular_node_id \
             FROM custody_state WHERE owner_id = ?",
        )
        .bind(owner_id)
        .fetch_optional(self.pool)
        .await?;

        Ok(row.map(row_to_custody))
    }

    /// Reclama la titularidad de custodia para `input.claiming_node_id`.
    ///
    /// Si NO existe fila de custodia para este `owner_id` todavía, este es
    /// el reclamo BOOTSTRAP: se inserta la fila génesis con
    /// `custody_epoch = 1` -- no hay epoch previo con el que competir, así
    /// que `input.expected_epoch` no se compara.
    ///
    /// Si YA existe una fila, aplica la MISMA guarda de concurrencia
    /// optimista que `AccountRepository::update_email_verification_status`
    /// (`row_version`), renombrada `custody_epoch`: el `UPDATE` filtra por
    /// `owner_id` Y `custody_epoch = expected_epoch`; si otra escritura ya
    /// avanzó el epoch, `rows_affected() == 0` y se devuelve
    /// `CustodyConflict` en vez de pisar el cambio ajeno -- NUNCA dos
    /// máquinas quedan tituales a la vez (regla obligatoria #4, ADR-0146).
    pub async fn claim_titular(&self, input: ClaimTitularInput) -> Result<CustodyRow, CustodyRepositoryError> {
        let now_ns = self.clock.timestamp_ns();

        match self.find_by_owner(&input.owner_id).await? {
            None => self.bootstrap_titular(&input, now_ns).await,
            Some(current) => self.claim_over_existing(&input, current, now_ns).await,
        }
    }

    /// Inserta la fila génesis de custodia (primer reclamo para este
    /// `owner_id`, `custody_epoch = 1`).
    async fn bootstrap_titular(
        &self,
        input: &ClaimTitularInput,
        now_ns: i64,
    ) -> Result<CustodyRow, CustodyRepositoryError> {
        let id = Uuid::now_v7().to_string();
        let custody_epoch = 1;
        let audit_hash = compute_custody_audit_hash(
            &id,
            now_ns,
            custody_epoch,
            None,
            &input.owner_id,
            &input.institutional_tag,
            &input.claiming_node_id,
        );

        sqlx::query(
            "INSERT INTO custody_state (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, custody_epoch, \
                owner_id, institutional_tag, titular_node_id\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(Option::<String>::None)
        .bind(custody_epoch)
        .bind(&input.owner_id)
        .bind(&input.institutional_tag)
        .bind(&input.claiming_node_id)
        .execute(self.pool)
        .await?;

        Ok(CustodyRow {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: None,
            custody_epoch,
            owner_id: input.owner_id.clone(),
            institutional_tag: input.institutional_tag.clone(),
            titular_node_id: input.claiming_node_id.clone(),
        })
    }

    /// Reclama la titularidad sobre una fila de custodia YA existente,
    /// aplicando la guarda de concurrencia optimista por `custody_epoch`.
    async fn claim_over_existing(
        &self,
        input: &ClaimTitularInput,
        current: CustodyRow,
        now_ns: i64,
    ) -> Result<CustodyRow, CustodyRepositoryError> {
        let current_state = CustodyState {
            owner_id: current.owner_id.clone(),
            titular_node_id: current.titular_node_id.clone(),
            custody_epoch: current.custody_epoch,
        };

        // Decisión pura del Core (sin I/O): valida el epoch esperado y
        // calcula el estado siguiente. Esto NO basta por sí solo contra
        // una carrera real entre dos procesos -- la guarda decisiva es el
        // `UPDATE ... WHERE custody_epoch = ?` de abajo.
        let next_state = decide_custody_claim(&current_state, &input.claiming_node_id, input.expected_epoch)?;

        let custody_epoch = next_state.custody_epoch;
        let audit_hash = compute_custody_audit_hash(
            &current.id,
            now_ns,
            custody_epoch,
            Some(&current.audit_hash),
            &input.owner_id,
            &input.institutional_tag,
            &next_state.titular_node_id,
        );

        // La guarda `custody_epoch = ?` es la comparación optimista: solo
        // actualiza si la fila en disco sigue en el epoch que leímos.
        let result = sqlx::query(
            "UPDATE custody_state SET \
                updated_at = ?, audit_hash = ?, audit_chain_hash = ?, custody_epoch = ?, titular_node_id = ? \
             WHERE owner_id = ? AND custody_epoch = ?",
        )
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&current.audit_hash)
        .bind(custody_epoch)
        .bind(&next_state.titular_node_id)
        .bind(&input.owner_id)
        .bind(input.expected_epoch)
        .execute(self.pool)
        .await?;

        // Cero filas afectadas => la fila ya no está en `expected_epoch`
        // (otra máquina reclamó primero). No pisamos: reportamos el conflicto.
        if result.rows_affected() == 0 {
            return Err(CustodyRepositoryError::CustodyConflict {
                owner_id: input.owner_id.clone(),
                expected_epoch: input.expected_epoch,
            });
        }

        Ok(CustodyRow {
            id: current.id,
            created_at_ns: current.created_at_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: Some(current.audit_hash.clone()),
            custody_epoch,
            owner_id: input.owner_id.clone(),
            institutional_tag: input.institutional_tag.clone(),
            titular_node_id: next_state.titular_node_id,
        })
    }

    /// Siembra directamente un estado de custodia inicial para
    /// `owner_id` -- SOLO para el harness de verificación CLI y para
    /// tests que necesitan simular "esta cuenta ya tenía historial de
    /// custodia previo a este epoch" sin recorrer todos los reclamos
    /// intermedios. NUNCA se usa en el flujo real de producción (que
    /// siempre pasa por [`Self::claim_titular`]).
    pub async fn seed_initial_state(
        &self,
        owner_id: &str,
        institutional_tag: &str,
        titular_node_id: &str,
        custody_epoch: i64,
    ) -> Result<CustodyRow, CustodyRepositoryError> {
        let now_ns = self.clock.timestamp_ns();
        let id = Uuid::now_v7().to_string();
        let audit_hash =
            compute_custody_audit_hash(&id, now_ns, custody_epoch, None, owner_id, institutional_tag, titular_node_id);

        sqlx::query(
            "INSERT INTO custody_state (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, custody_epoch, \
                owner_id, institutional_tag, titular_node_id\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(Option::<String>::None)
        .bind(custody_epoch)
        .bind(owner_id)
        .bind(institutional_tag)
        .bind(titular_node_id)
        .execute(self.pool)
        .await?;

        Ok(CustodyRow {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: None,
            custody_epoch,
            owner_id: owner_id.to_string(),
            institutional_tag: institutional_tag.to_string(),
            titular_node_id: titular_node_id.to_string(),
        })
    }
}

fn row_to_custody(row: sqlx::sqlite::SqliteRow) -> CustodyRow {
    CustodyRow {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        custody_epoch: row.get("custody_epoch"),
        owner_id: row.get("owner_id"),
        institutional_tag: row.get("institutional_tag"),
        titular_node_id: row.get("titular_node_id"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::persistence::central_identity::test_support::seed_account;
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    fn sample_backup_input(owner_id: &str, node_id: &str) -> RecordBackupInput {
        RecordBackupInput {
            owner_id: owner_id.to_string(),
            institutional_tag: "DRASUS_LOCAL".to_string(),
            node_id: node_id.to_string(),
            snapshot_at_ns: 900,
            blob_hash: "deadbeef".to_string(),
            blob_size_bytes: 128,
            nonce_hex: "0011223344556677889900aa".to_string(),
        }
    }

    // ── CRITERIO #1 (Orden §5): esquema STRICT + Grupo I + Perfil D ─────────

    #[tokio::test]
    async fn migration_creates_instance_backups_table_strict_append_only() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('instance_backups')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();

        for expected in [
            "id", "created_at", "updated_at", "audit_hash", "audit_chain_hash", "event_sequence_id",
            "owner_id", "institutional_tag", "node_id", "snapshot_at", "blob_hash", "blob_size_bytes", "nonce_hex",
        ] {
            assert!(column_names.contains(&expected.to_string()), "falta la columna: {expected}");
        }

        assert!(
            !column_names.contains(&"custody_epoch".to_string()),
            "instance_backups es APPEND-ONLY: no debe tener custody_epoch/row_version, solo event_sequence_id"
        );

        // Guardarraíl ADR-0093: ninguna columna es clave/secreto maestro.
        for forbidden in ["encryption_key", "master_secret", "broker_credential", "live_ip"] {
            assert!(!column_names.contains(&forbidden.to_string()), "no debe existir la columna: {forbidden}");
        }

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'instance_backups'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(sql.contains("STRICT"), "la tabla instance_backups debe declararse STRICT");
    }

    #[tokio::test]
    async fn migration_creates_custody_state_table_strict_mutable() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('custody_state')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();

        for expected in [
            "id", "created_at", "updated_at", "audit_hash", "audit_chain_hash", "custody_epoch",
            "owner_id", "institutional_tag", "titular_node_id",
        ] {
            assert!(column_names.contains(&expected.to_string()), "falta la columna: {expected}");
        }

        assert!(
            !column_names.contains(&"event_sequence_id".to_string()),
            "custody_state es MUTABLE: no debe tener event_sequence_id, solo custody_epoch"
        );

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'custody_state'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(sql.contains("STRICT"), "la tabla custody_state debe declararse STRICT");
    }

    // ── CRITERIO DE CIERRE (ADR-0141 enmienda 2026-07-11, M6) ────────────────

    /// La FK física `instance_backups.owner_id -> accounts(id)` rechaza un
    /// `owner_id` que no corresponde a ninguna cuenta.
    #[tokio::test]
    async fn record_backup_with_nonexistent_owner_id_is_rejected_by_foreign_key() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = BackupRegistryRepository::new(&pool, &clock);

        let result = repo.record_backup(sample_backup_input("cuenta-que-no-existe", "node-1")).await;

        assert!(
            matches!(result, Err(BackupRegistryRepositoryError::Database(_))),
            "un owner_id huérfano debe rechazarse por la FK, no persistirse: {result:?}"
        );
    }

    /// La FK física `custody_state.owner_id -> accounts(id)` rechaza un
    /// `owner_id` que no corresponde a ninguna cuenta.
    #[tokio::test]
    async fn claim_titular_with_nonexistent_owner_id_is_rejected_by_foreign_key() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = CustodyRepository::new(&pool, &clock);

        let result = repo
            .claim_titular(ClaimTitularInput {
                owner_id: "cuenta-que-no-existe".to_string(),
                institutional_tag: "DRASUS_LOCAL".to_string(),
                claiming_node_id: "node-A".to_string(),
                expected_epoch: 999,
            })
            .await;

        assert!(
            matches!(result, Err(CustodyRepositoryError::Database(_))),
            "un owner_id huérfano debe rechazarse por la FK, no persistirse: {result:?}"
        );
    }

    // ── CRITERIO #7 (Orden §5): append-only -- UPDATE/DELETE rechazados ─────

    #[tokio::test]
    async fn update_is_rejected_by_trigger() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let repo = BackupRegistryRepository::new(&pool, &clock);
        let row = repo.record_backup(sample_backup_input(&owner_id, "node-1")).await.expect("registrar respaldo");

        let result = sqlx::query("UPDATE instance_backups SET blob_hash = 'otro' WHERE id = ?")
            .bind(&row.id)
            .execute(&pool)
            .await;
        assert!(result.is_err(), "UPDATE sobre instance_backups debe ser rechazado por el trigger");
    }

    #[tokio::test]
    async fn delete_is_rejected_by_trigger() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let repo = BackupRegistryRepository::new(&pool, &clock);
        let row = repo.record_backup(sample_backup_input(&owner_id, "node-1")).await.expect("registrar respaldo");

        let result = sqlx::query("DELETE FROM instance_backups WHERE id = ?").bind(&row.id).execute(&pool).await;
        assert!(result.is_err(), "DELETE sobre instance_backups debe ser rechazado por el trigger");
    }

    #[tokio::test]
    async fn duplicating_event_sequence_id_is_rejected_by_unique_constraint() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let repo = BackupRegistryRepository::new(&pool, &clock);
        repo.record_backup(sample_backup_input(&owner_id, "node-1")).await.expect("primer respaldo (event_sequence_id = 1)");

        let duplicate = sqlx::query(
            "INSERT INTO instance_backups (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, snapshot_at, blob_hash, blob_size_bytes, nonce_hex\
            ) VALUES ('id-dup', 0, 0, 'hash', NULL, 1, ?, 'DRASUS_LOCAL', 'node-1', 0, 'h', 1, 'n')",
        )
        .bind(&owner_id)
        .execute(&pool)
        .await;

        assert!(duplicate.is_err(), "duplicar event_sequence_id=1 debe ser rechazado por UNIQUE");
    }

    #[tokio::test]
    async fn audit_chain_hash_is_null_only_in_genesis_row_and_chains_afterwards() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let repo = BackupRegistryRepository::new(&pool, &clock);

        let first = repo.record_backup(sample_backup_input(&owner_id, "node-1")).await.expect("génesis");
        clock.tick();
        let second = repo.record_backup(sample_backup_input(&owner_id, "node-2")).await.expect("segundo");

        assert_eq!(first.audit_chain_hash, None);
        assert_eq!(second.audit_chain_hash, Some(first.audit_hash.clone()));
    }

    // ── CRITERIO #7 (Orden §5): append atómico + 16 escritores concurrentes ─

    /// CRITERIO DE CIERRE: 16 escritores concurrentes sobre el MISMO
    /// pool/ledger, en una BD de ARCHIVO temporal (nunca `:memory:`, donde
    /// cada conexión sería una base distinta). La transacción `BEGIN
    /// IMMEDIATE` + reintento acotado debe garantizar que NINGÚN respaldo
    /// se pierde y que la secuencia queda densa (1..=N sin huecos ni
    /// duplicados). Esta prueba DEBE poder caerse si se quita la
    /// transacción (mismo razonamiento que
    /// `enriched_domain_events::concurrent_record_events_persist_every_event_without_gaps_or_lost_rows`).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_record_backups_persist_every_backup_without_gaps_or_lost_rows() {
        use std::sync::Arc;

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("instance_backups_concurrency.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());
        let pool = connect(&database_url).await.expect("conectar");
        migrate(&pool).await.expect("migrar");

        let clock: Arc<DeterministicClock> = Arc::new(DeterministicClock::new(1_000, 100));
        let owner_id = seed_account(&pool, clock.as_ref(), "owner-concurrente@example.com").await;
        const N: i64 = 16;

        let mut handles = Vec::new();
        for i in 0..N {
            let pool_c = pool.clone();
            let clock_c = clock.clone();
            let owner_id_c = owner_id.clone();
            handles.push(tokio::spawn(async move {
                let repo = BackupRegistryRepository::new(&pool_c, clock_c.as_ref());
                repo.record_backup(RecordBackupInput {
                    owner_id: owner_id_c,
                    institutional_tag: "DRASUS_LOCAL".to_string(),
                    node_id: format!("node-{i}"),
                    snapshot_at_ns: 900 + i,
                    blob_hash: format!("hash-{i}"),
                    blob_size_bytes: 100 + i,
                    nonce_hex: format!("nonce{i:02}"),
                })
                .await
            }));
        }

        for handle in handles {
            handle
                .await
                .expect("la tarea no debe entrar en panic")
                .expect("record_backup debe tener éxito para cada escritor concurrente");
        }

        let repo = BackupRegistryRepository::new(&pool, clock.as_ref());
        let chain = repo.load_chain().await.expect("cargar la cadena completa");

        assert_eq!(chain.len() as i64, N, "deben persistirse las N filas, sin ninguna pérdida");

        let sequence_ids: Vec<i64> = chain.iter().map(|row| row.event_sequence_id).collect();
        let expected: Vec<i64> = (1..=N).collect();
        assert_eq!(sequence_ids, expected, "los event_sequence_id deben ser 1..=N sin huecos ni duplicados");

        for (index, row) in chain.iter().enumerate() {
            let previous_audit_hash = if index == 0 {
                assert_eq!(row.audit_chain_hash, None);
                GENESIS_PREVIOUS_HASH.to_string()
            } else {
                let prev = &chain[index - 1];
                assert_eq!(row.audit_chain_hash.as_deref(), Some(prev.audit_hash.as_str()));
                prev.audit_hash.clone()
            };

            let recomputed = compute_backup_audit_hash(
                &row.id, row.created_at_ns, row.event_sequence_id, &previous_audit_hash,
                &row.owner_id, &row.institutional_tag, &row.node_id,
                row.snapshot_at_ns, &row.blob_hash, row.blob_size_bytes, &row.nonce_hex,
            );
            assert_eq!(recomputed, row.audit_hash, "el audit_hash de cada fila debe ser recomputable");
        }
    }

    // ── CRITERIO #6 (Orden §5): gate de titularidad exclusiva ───────────────

    /// CRITERIO DE CIERRE: el primer reclamo para un `owner_id` nuevo
    /// (BOOTSTRAP) siempre gana, sin importar `expected_epoch`.
    #[tokio::test]
    async fn bootstrap_claim_always_succeeds_for_a_new_owner() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let repo = CustodyRepository::new(&pool, &clock);

        let row = repo
            .claim_titular(ClaimTitularInput {
                owner_id: owner_id.clone(),
                institutional_tag: "DRASUS_LOCAL".to_string(),
                claiming_node_id: "node-A".to_string(),
                expected_epoch: 999, // ignorado en bootstrap
            })
            .await
            .expect("el reclamo bootstrap siempre debe tener éxito");

        assert_eq!(row.custody_epoch, 1);
        assert_eq!(row.titular_node_id, "node-A");
        assert_eq!(row.audit_chain_hash, None);
    }

    /// CRITERIO DE CIERRE (regla obligatoria #4, ADR-0146): dos reclamos
    /// que parten del MISMO `custody_epoch` -- el primero gana (epoch+1),
    /// el segundo (que sigue creyendo estar en el epoch viejo) recibe
    /// `CustodyConflict`, NUNCA ambos tienen éxito.
    ///
    /// Esta prueba FALLA si se quita la guarda `AND custody_epoch = ?` del
    /// UPDATE: sin ella, el segundo reclamo también afectaría 1 fila y
    /// devolvería `Ok`, dejando DOS máquinas creyéndose tituales a la vez.
    #[tokio::test]
    async fn two_claims_from_the_same_epoch_only_one_wins() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let repo = CustodyRepository::new(&pool, &clock);

        // Estado inicial: node-A es titular en el epoch 3 (simulado vía
        // seed_initial_state -- equivalente a "esta cuenta ya existía").
        repo.seed_initial_state(&owner_id, "DRASUS_LOCAL", "node-A", 3)
            .await
            .expect("sembrar estado inicial");

        // Dos "máquinas" leyeron el MISMO epoch vigente (3) y compiten por
        // reclamar la titularidad.
        clock.tick();
        let winner = repo
            .claim_titular(ClaimTitularInput {
                owner_id: owner_id.clone(),
                institutional_tag: "DRASUS_LOCAL".to_string(),
                claiming_node_id: "node-B".to_string(),
                expected_epoch: 3,
            })
            .await
            .expect("el primer reclamo desde el epoch vigente debe ganar");
        assert_eq!(winner.custody_epoch, 4);
        assert_eq!(winner.titular_node_id, "node-B");

        clock.tick();
        let loser = repo
            .claim_titular(ClaimTitularInput {
                owner_id: owner_id.clone(),
                institutional_tag: "DRASUS_LOCAL".to_string(),
                claiming_node_id: "node-C".to_string(),
                expected_epoch: 3, // sigue creyendo que el epoch vigente es 3
            })
            .await;

        assert!(
            matches!(
                &loser,
                Err(CustodyRepositoryError::CustodyConflict { owner_id: conflict_owner, expected_epoch: 3 }) if conflict_owner == &owner_id
            ),
            "el segundo reclamo desde el epoch 3 debe dar CustodyConflict, no éxito silencioso; fue: {loser:?}"
        );

        // La fila en disco conserva al GANADOR (node-B), no al perdedor.
        let reloaded = repo.find_by_owner(&owner_id).await.expect("releer").expect("existe");
        assert_eq!(reloaded.titular_node_id, "node-B");
        assert_eq!(reloaded.custody_epoch, 4);
    }

    /// Reclamar el MISMO nodo desde el epoch vigente (re-afirmar la
    /// titularidad propia, ej. al reiniciar la app) también avanza el
    /// epoch -- el gate no distingue "reclamo propio" de "reclamo ajeno",
    /// solo compara epochs (mismo criterio simple y auditable).
    #[tokio::test]
    async fn reclaiming_ones_own_titularity_advances_the_epoch() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let repo = CustodyRepository::new(&pool, &clock);

        repo.seed_initial_state(&owner_id, "DRASUS_LOCAL", "node-A", 3).await.expect("sembrar");

        clock.tick();
        let row = repo
            .claim_titular(ClaimTitularInput {
                owner_id: owner_id.clone(),
                institutional_tag: "DRASUS_LOCAL".to_string(),
                claiming_node_id: "node-A".to_string(),
                expected_epoch: 3,
            })
            .await
            .expect("re-reclamar la titularidad propia debe tener éxito");

        assert_eq!(row.custody_epoch, 4);
        assert_eq!(row.titular_node_id, "node-A");
    }

    /// `find_by_owner` devuelve `None` para un dueño sin custodia
    /// registrada todavía.
    #[tokio::test]
    async fn find_by_owner_returns_none_for_unknown_owner() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = CustodyRepository::new(&pool, &clock);

        assert_eq!(repo.find_by_owner("owner-inexistente").await.expect("consulta debe tener éxito"), None);
    }

    // ── CRITERIO (QA por mutación, DEBT-018): reintento acotado hasta AGOTAR ──

    /// CRITERIO DE CIERRE (QA por mutación): bajo contención de escritura
    /// SOSTENIDA (otro escritor retiene el lock de `BEGIN IMMEDIATE` y no lo
    /// suelta), el bucle de reintento de `record_backup` debe agotar
    /// EXACTAMENTE `MAX_RECORD_ATTEMPTS` intentos y rendirse con
    /// `WriteContention { attempts: MAX }` -- nunca descartar el respaldo en
    /// silencio, ni rendirse un intento antes o después. Patrón de
    /// referencia: `persistence/data_portability.rs` (STORY-043, DEBT-018).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn record_backup_exhausts_exactly_max_attempts_when_write_lock_is_held() {
        use std::str::FromStr;
        use std::time::Duration;

        use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("instance_continuity_forced_contention.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());

        // Migrar con el pool normal (busy_timeout de 5s).
        let pool = connect(&database_url).await.expect("conectar");
        migrate(&pool).await.expect("migrar");

        // Sembrar la cuenta ANTES de que el escritor A tome el lock -- la FK
        // owner_id->accounts(id) exige que la cuenta ya exista; sembrarla
        // aquí evita interferir con el escenario de contención de abajo.
        let seed_clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &seed_clock, "owner1@example.com").await;

        // Opciones con busy_timeout=0: un lock ocupado falla de INMEDIATO con
        // "database is locked" en vez de esperar 5s -- hace la contención
        // determinista y rápida.
        let immediate_opts = || {
            SqliteConnectOptions::from_str(&database_url)
                .expect("parsear opciones")
                .journal_mode(SqliteJournalMode::Wal)
                .busy_timeout(Duration::from_millis(0))
        };

        // Escritor A: toma el lock de escritura con `BEGIN IMMEDIATE` y NO lo
        // suelta mientras B intenta escribir.
        let lock_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(immediate_opts())
            .await
            .expect("pool que retiene el lock");
        let lock_tx = lock_pool
            .begin_with("BEGIN IMMEDIATE")
            .await
            .expect("tomar el lock de escritura reservado");

        // Escritor B: intenta registrar un respaldo mientras A retiene el
        // lock. Cada `try_record_backup_once` abre `BEGIN IMMEDIATE`, choca
        // con el lock de A, falla con "database is locked" (transitorio) y
        // reintenta, hasta agotar MAX_RECORD_ATTEMPTS.
        let repo_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(immediate_opts())
            .await
            .expect("pool del repositorio");
        let clock = DeterministicClock::new(1_000, 100);
        let repo = BackupRegistryRepository::new(&repo_pool, &clock);

        let result = repo.record_backup(sample_backup_input(&owner_id, "node-contention")).await;

        drop(lock_tx); // libera el lock (limpieza; el resultado ya está tomado)

        match result {
            Err(BackupRegistryRepositoryError::WriteContention { attempts }) => {
                assert_eq!(
                    attempts, MAX_RECORD_ATTEMPTS,
                    "bajo contención sostenida debe agotar EXACTAMENTE MAX_RECORD_ATTEMPTS intentos"
                );
            }
            other => panic!(
                "se esperaba WriteContention {{ attempts: {MAX_RECORD_ATTEMPTS} }} bajo contención sostenida, se obtuvo: {other:?}"
            ),
        }
    }

    // ── CRITERIO (QA por mutación, DEBT-018): clasificador de contención ──────

    /// CRITERIO DE CIERRE (QA por mutación): `is_transient_write_conflict`
    /// distingue una violación UNIQUE PERMANENTE (la PK `id`, que NO se debe
    /// reintentar) de la contención transitoria. Fija que exige AMBAS
    /// condiciones (es violación UNIQUE **y** menciona `event_sequence_id`),
    /// no una sola, y que no clasifica cualquier cosa como transitoria.
    #[tokio::test]
    async fn is_transient_is_false_for_a_permanent_non_sequence_unique_violation() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;

        // Inserta una fila válida y luego otra con el MISMO `id`: viola la
        // PRIMARY KEY `id`, NO el UNIQUE de `event_sequence_id`. Error UNIQUE
        // PERMANENTE cuyo mensaje NO menciona `event_sequence_id`.
        let insert_with_id_dup = |event_sequence_id: i64| {
            sqlx::query(
                "INSERT INTO instance_backups (\
                    id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, node_id, snapshot_at, blob_hash, blob_size_bytes, nonce_hex\
                ) VALUES ('dup-id', 0, 0, 'hash', NULL, ?, ?, 'DRASUS_LOCAL', 'node-1', 0, 'h', 1, 'n')",
            )
            .bind(event_sequence_id)
            .bind(&owner_id)
            .execute(&pool)
        };
        insert_with_id_dup(1).await.expect("primera fila válida");
        let err = insert_with_id_dup(2)
            .await
            .expect_err("la segunda fila debe violar la PRIMARY KEY id");

        let permanent = BackupRegistryRepositoryError::Database(err);
        assert!(
            !is_transient_write_conflict(&permanent),
            "una violación UNIQUE de la PK (no de event_sequence_id) es PERMANENTE, no transitoria"
        );

        // Control: un error que ni siquiera es de base de datos jamás es
        // transitorio (fija la rama temprana `let ... else`).
        let non_database = BackupRegistryRepositoryError::WriteContention { attempts: 5 };
        assert!(
            !is_transient_write_conflict(&non_database),
            "un error no-Database nunca es contención transitoria"
        );
    }

    // ── CRITERIO (QA por mutación, DEBT-018): fidelidad de la fila devuelta ───

    /// CRITERIO DE CIERRE (QA por mutación): la fila que DEVUELVE
    /// `record_backup` es bit-a-bit idéntica a la fila persistida en disco
    /// -- si el literal de retorno de `try_record_backup_once` sustituyera
    /// algún campo (`audit_hash`, `event_sequence_id`, timestamps...) por un
    /// valor por defecto en vez del recién calculado, esta comparación de
    /// igualdad completa lo detectaría.
    #[tokio::test]
    async fn record_backup_returned_row_matches_the_persisted_row_exactly() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let repo = BackupRegistryRepository::new(&pool, &clock);

        let first = repo.record_backup(sample_backup_input(&owner_id, "node-1")).await.expect("primer respaldo");
        clock.tick();
        let second = repo.record_backup(sample_backup_input(&owner_id, "node-2")).await.expect("segundo respaldo");

        let chain = repo.load_chain().await.expect("cargar cadena");
        assert_eq!(
            chain.first(),
            Some(&first),
            "la primera fila devuelta debe ser idéntica a la persistida en disco"
        );
        assert_eq!(
            chain.get(1),
            Some(&second),
            "la segunda fila devuelta debe ser idéntica a la persistida en disco"
        );
        assert_ne!(
            second.audit_hash, first.audit_hash,
            "el audit_hash devuelto debe ser recomputado, no copiado del intento anterior"
        );
        assert_eq!(second.updated_at_ns, 1_100, "el updated_at devuelto debe reflejar el now del reloj tras el tick");
    }
}
