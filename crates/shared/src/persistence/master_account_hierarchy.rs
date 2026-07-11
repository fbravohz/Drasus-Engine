//! [SHELL] Repositorios de persistencia de la Jerarquía de Cuenta Maestra
//! (`docs/features/master-account-hierarchy.md`, ADR-0147 -- cimiento #12,
//! ADR-0093, ADR-0141, ADR-0020, migración `0018_master_account_hierarchy.sql`,
//! STORY-040).
//!
//! DOS repositorios, uno por tabla (mismo criterio que los once cimientos
//! previos del substrato):
//! - [`AccountHierarchyRepository`]: `account_hierarchy`, MUTABLE con
//!   `row_version` (concurrencia optimista ->
//!   [`AccountHierarchyRepositoryError::VersionConflict`]), mismo patrón
//!   que [`crate::persistence::verified_account_registry::VerifiedAccountRepository`].
//! - [`OverrideAttestationRepository`][]: `override_attestations`,
//!   APPEND-ONLY ATÓMICA (`event_sequence_id UNIQUE`, `BEGIN IMMEDIATE` +
//!   reintento acotado), mismo patrón que
//!   [`crate::persistence::verified_account_registry::AttestedTrackRecordRepository`]
//!   (causa raíz DEBT-001).
//!
//! La lógica pura (gate de consentimiento, efecto local de "eliminar =
//! archivar", hash de auditoría encadenado de ambas tablas) vive en
//! [`crate::domain::master_account_hierarchy`] -- este módulo solo le da
//! entradas inyectadas y persiste/carga el resultado.

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::audit_log::GENESIS_PREVIOUS_HASH;
use crate::domain::clock::Clock;
use crate::domain::master_account_hierarchy::{
    compute_hierarchy_audit_hash, compute_override_audit_hash, AttestationSide, OverrideCommandKind,
    OverrideOutcomeLabel,
};

// ── `account_hierarchy` -- MUTABLE, row_version ─────────────────────────────

/// Errores que devuelven las operaciones de [`AccountHierarchyRepository`].
#[derive(Debug, thiserror::Error)]
pub enum AccountHierarchyRepositoryError {
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// Concurrencia optimista (ADR-0141): el UPDATE partió de un
    /// `row_version` que ya no es el vigente en disco -- otra escritura
    /// actualizó la fila en el ínterin. Mismo patrón que
    /// `verified_account_registry::VerifiedAccountRepositoryError::VersionConflict`.
    #[error("conflicto de versión en la jerarquía de la cuenta '{id}': se esperaba row_version {expected}, la fila ya avanzó")]
    VersionConflict { id: String, expected: i64 },
}

/// Una jerarquía nueva a registrar -- el "vincular hija a fondo" inicial
/// (`docs/features/master-account-hierarchy.md` "Ciclo de Vida").
#[derive(Debug, Clone)]
pub struct NewAccountHierarchy {
    pub owner_id: String,
    pub parent_owner_id: Option<String>,
    pub consent_ref: String,
    pub node_id: String,
}

/// Una fila de `account_hierarchy` ya persistida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountHierarchyRow {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub row_version: i64,

    pub owner_id: String,
    pub parent_owner_id: Option<String>,
    pub consent_ref: String,
    pub node_id: String,
}

/// Repositorio MUTABLE para `account_hierarchy`.
pub struct AccountHierarchyRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> AccountHierarchyRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Vincula una hija nueva a su fondo (o registra una fila sin padre
    /// todavía, `parent_owner_id: None`) -- `row_version = 1`. Regla fija
    /// #1 (ADR-0147): esta fila es el PUNTERO de la hija, nunca el árbol
    /// completo del fondo.
    pub async fn link_child(
        &self,
        new: NewAccountHierarchy,
    ) -> Result<AccountHierarchyRow, AccountHierarchyRepositoryError> {
        let id = Uuid::now_v7().to_string();
        let now_ns = self.clock.timestamp_ns();
        let row_version: i64 = 1;

        let audit_hash = compute_hierarchy_audit_hash(
            &id,
            now_ns,
            row_version,
            None,
            &new.owner_id,
            new.parent_owner_id.as_deref(),
            &new.consent_ref,
            &new.node_id,
        );

        sqlx::query(
            "INSERT INTO account_hierarchy (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                owner_id, parent_owner_id, consent_ref, node_id\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(Option::<String>::None)
        .bind(row_version)
        .bind(&new.owner_id)
        .bind(&new.parent_owner_id)
        .bind(&new.consent_ref)
        .bind(&new.node_id)
        .execute(self.pool)
        .await?;

        Ok(AccountHierarchyRow {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: None,
            row_version,
            owner_id: new.owner_id,
            parent_owner_id: new.parent_owner_id,
            consent_ref: new.consent_ref,
            node_id: new.node_id,
        })
    }

    /// Carga la jerarquía vigente de una hija por su `owner_id`, o `None`
    /// si nunca se registró (todavía huérfana).
    pub async fn find_by_owner(
        &self,
        owner_id: &str,
    ) -> Result<Option<AccountHierarchyRow>, AccountHierarchyRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    owner_id, parent_owner_id, consent_ref, node_id \
             FROM account_hierarchy WHERE owner_id = ?",
        )
        .bind(owner_id)
        .fetch_optional(self.pool)
        .await?;

        Ok(row.map(row_to_hierarchy))
    }

    /// Actualiza `parent_owner_id`/`consent_ref` de `current` -- re-vincula
    /// la hija a otro fondo, o renueva su referencia de consentimiento.
    ///
    /// ## Concurrencia optimista (ADR-0141)
    ///
    /// El `UPDATE` filtra por `id` **y** `row_version = <el de `current`>`.
    /// Si otra escritura ya avanzó la fila desde que se leyó `current`, el
    /// `WHERE` no encuentra ninguna fila y se devuelve
    /// [`AccountHierarchyRepositoryError::VersionConflict`] en vez de pisar
    /// el cambio ajeno -- mismo patrón que
    /// `verified_account_registry::VerifiedAccountRepository::update_publication_and_scopes`.
    pub async fn update_parent_and_consent(
        &self,
        current: &AccountHierarchyRow,
        new_parent_owner_id: Option<&str>,
        new_consent_ref: &str,
    ) -> Result<AccountHierarchyRow, AccountHierarchyRepositoryError> {
        let now_ns = self.clock.timestamp_ns();
        let row_version = current.row_version + 1;

        let audit_hash = compute_hierarchy_audit_hash(
            &current.id,
            now_ns,
            row_version,
            Some(&current.audit_hash),
            &current.owner_id,
            new_parent_owner_id,
            new_consent_ref,
            &current.node_id,
        );

        // La guarda `row_version = ?` es la comparación optimista: solo
        // actualiza si la fila en disco sigue en la versión que leímos.
        let result = sqlx::query(
            "UPDATE account_hierarchy SET \
                updated_at = ?, audit_hash = ?, audit_chain_hash = ?, row_version = ?, \
                parent_owner_id = ?, consent_ref = ? \
             WHERE id = ? AND row_version = ?",
        )
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&current.audit_hash)
        .bind(row_version)
        .bind(new_parent_owner_id)
        .bind(new_consent_ref)
        .bind(&current.id)
        .bind(current.row_version)
        .execute(self.pool)
        .await?;

        // Cero filas afectadas => la fila ya no está en `current.row_version`
        // (otra escritura la adelantó). No pisamos: reportamos el conflicto.
        if result.rows_affected() == 0 {
            return Err(AccountHierarchyRepositoryError::VersionConflict {
                id: current.id.clone(),
                expected: current.row_version,
            });
        }

        Ok(AccountHierarchyRow {
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: Some(current.audit_hash.clone()),
            row_version,
            parent_owner_id: new_parent_owner_id.map(str::to_string),
            consent_ref: new_consent_ref.to_string(),
            ..current.clone()
        })
    }
}

/// Convierte una fila de `account_hierarchy` al tipo [`AccountHierarchyRow`].
fn row_to_hierarchy(row: sqlx::sqlite::SqliteRow) -> AccountHierarchyRow {
    AccountHierarchyRow {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        row_version: row.get("row_version"),
        owner_id: row.get("owner_id"),
        parent_owner_id: row.get("parent_owner_id"),
        consent_ref: row.get("consent_ref"),
        node_id: row.get("node_id"),
    }
}

// ── `override_attestations` -- APPEND-ONLY ATÓMICA ──────────────────────────

/// Errores que devuelven las operaciones de [`OverrideAttestationRepository`].
#[derive(Debug, thiserror::Error)]
pub enum OverrideAttestationRepositoryError {
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// El append no pudo completarse tras agotar los reintentos ante
    /// contención de escritura transitoria -- la atestación NO se descartó
    /// en silencio (regla "Atomicidad de ledgers append-only",
    /// rust-engineer/SKILL.md §4).
    #[error("no se pudo registrar la atestación de override tras {attempts} intentos por contención de escritura")]
    WriteContention { attempts: u32 },
    /// Una fila persistida tenía un `attestation_side` fuera del catálogo --
    /// error de integridad de datos, no debería ocurrir si el `CHECK` de la
    /// migración se respeta.
    #[error("attestation_side desconocido en override_attestations: '{0}'")]
    UnknownAttestationSide(String),
    /// Análogo para `command_kind`.
    #[error("command_kind desconocido en override_attestations: '{0}'")]
    UnknownCommandKind(String),
    /// Análogo para `outcome`.
    #[error("outcome desconocido en override_attestations: '{0}'")]
    UnknownOutcome(String),
}

/// Número máximo de intentos del append ante contención de escritura
/// transitoria antes de rendirse con
/// [`OverrideAttestationRepositoryError::WriteContention`]. Mismo valor
/// (cinco) que el resto de los ledgers append-only del substrato.
const MAX_RECORD_ATTEMPTS: u32 = 5;

/// Decide si un error del repositorio es una contención de escritura
/// TRANSITORIA -- mismo criterio que
/// `verified_account_registry::is_transient_write_conflict`.
fn is_transient_write_conflict(error: &OverrideAttestationRepositoryError) -> bool {
    let OverrideAttestationRepositoryError::Database(sqlx::Error::Database(db)) = error else {
        return false;
    };

    let message = db.message().to_lowercase();
    if message.contains("database is locked") || message.contains("database table is locked") {
        return true;
    }

    db.is_unique_violation() && message.contains("event_sequence_id")
}

/// Entrada para [`OverrideAttestationRepository::record_attestation`] --
/// todo lo que la Shell necesita para registrar UNA fila de atestación
/// (fondo o hija).
#[derive(Debug, Clone)]
pub struct RecordOverrideAttestationInput {
    pub owner_id: String,
    pub parent_owner_id: String,
    pub node_id: String,
    pub attestation_side: AttestationSide,
    pub command_kind: OverrideCommandKind,
    pub target_ref: String,
    pub outcome: OverrideOutcomeLabel,
    pub justification: Option<String>,
}

/// Una fila de `override_attestations` ya persistida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverrideAttestationRow {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub event_sequence_id: i64,

    pub owner_id: String,
    pub parent_owner_id: String,
    pub node_id: String,

    pub attestation_side: AttestationSide,
    pub command_kind: OverrideCommandKind,
    pub target_ref: String,
    pub outcome: OverrideOutcomeLabel,
    pub justification: Option<String>,
}

/// Repositorio APPEND-ONLY para `override_attestations`.
///
/// Al igual que `AttestedTrackRecordRepository`/`BackupRegistryRepository`,
/// la única operación de escritura expuesta es
/// [`Self::record_attestation`] (un INSERT) -- no hay `update`/`delete`;
/// los triggers `trg_override_attestations_no_update`/`_no_delete` de la
/// migración los rechazarían de todas formas.
pub struct OverrideAttestationRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> OverrideAttestationRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Registra UNA fila de atestación (ISSUER o EXECUTOR): deriva su
    /// posición en la cadena GLOBAL, computa su `audit_hash` encadenado y
    /// la persiste como fila nueva.
    ///
    /// ## Atomicidad bajo concurrencia (regla "Atomicidad de ledgers append-only")
    ///
    /// El *read-then-write* (leer el MAX(`event_sequence_id`)/`audit_hash`
    /// previo, y el `INSERT`) ocurre dentro de UNA sola transacción
    /// `BEGIN IMMEDIATE` -- ver [`Self::try_record_once`]. Sin ella, dos
    /// escritores concurrentes derivarían el mismo `event_sequence_id`, el
    /// `UNIQUE` rechazaría a uno y su atestación se PERDERÍA. Ante
    /// contención transitoria se reintenta hasta [`MAX_RECORD_ATTEMPTS`]
    /// veces re-derivando la secuencia; nunca se descarta la atestación en
    /// silencio.
    pub async fn record_attestation(
        &self,
        input: RecordOverrideAttestationInput,
    ) -> Result<OverrideAttestationRow, OverrideAttestationRepositoryError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_record_once(&input).await {
                Ok(row) => return Ok(row),
                Err(error) => {
                    if is_transient_write_conflict(&error) {
                        if attempt < MAX_RECORD_ATTEMPTS {
                            continue;
                        }
                        return Err(OverrideAttestationRepositoryError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    /// Un intento único del append, dentro de una transacción
    /// `BEGIN IMMEDIATE` -- toma el lock de escritura de ENTRADA, evitando
    /// tanto la intercalación de otro escritor entre lectura e inserción
    /// como el interbloqueo de upgrade de dos transacciones DEFERRED.
    async fn try_record_once(
        &self,
        input: &RecordOverrideAttestationInput,
    ) -> Result<OverrideAttestationRow, OverrideAttestationRepositoryError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        let tail_row = sqlx::query(
            "SELECT audit_hash, event_sequence_id \
             FROM override_attestations \
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
        let now_ns = self.clock.timestamp_ns();

        let attestation_side_str = input.attestation_side.as_str();
        let command_kind_str = input.command_kind.as_str();
        let outcome_str = input.outcome.as_str();

        let audit_hash = compute_override_audit_hash(
            &id,
            now_ns,
            event_sequence_id,
            &previous_audit_hash,
            &input.owner_id,
            &input.parent_owner_id,
            &input.node_id,
            attestation_side_str,
            command_kind_str,
            &input.target_ref,
            outcome_str,
        );

        sqlx::query(
            "INSERT INTO override_attestations (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, parent_owner_id, node_id, \
                attestation_side, command_kind, target_ref, outcome, justification\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&audit_chain_hash)
        .bind(event_sequence_id)
        .bind(&input.owner_id)
        .bind(&input.parent_owner_id)
        .bind(&input.node_id)
        .bind(attestation_side_str)
        .bind(command_kind_str)
        .bind(&input.target_ref)
        .bind(outcome_str)
        .bind(&input.justification)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(OverrideAttestationRow {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash,
            event_sequence_id,
            owner_id: input.owner_id.clone(),
            parent_owner_id: input.parent_owner_id.clone(),
            node_id: input.node_id.clone(),
            attestation_side: input.attestation_side,
            command_kind: input.command_kind,
            target_ref: input.target_ref.clone(),
            outcome: input.outcome,
            justification: input.justification.clone(),
        })
    }

    /// Carga la cadena completa, ordenada por `event_sequence_id`
    /// ascendente -- usada por los tests de integridad de la cadena y por
    /// cualquier consumidor futuro que reconstruya el historial de
    /// overrides.
    pub async fn load_chain(&self) -> Result<Vec<OverrideAttestationRow>, OverrideAttestationRepositoryError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, parent_owner_id, node_id, \
                    attestation_side, command_kind, target_ref, outcome, justification \
             FROM override_attestations \
             ORDER BY event_sequence_id ASC",
        )
        .fetch_all(self.pool)
        .await?;

        rows.into_iter().map(row_to_attestation).collect()
    }
}

/// Convierte una fila de `override_attestations` al tipo
/// [`OverrideAttestationRow`], decodificando los enums de texto persistidos.
fn row_to_attestation(
    row: sqlx::sqlite::SqliteRow,
) -> Result<OverrideAttestationRow, OverrideAttestationRepositoryError> {
    let attestation_side_value: String = row.get("attestation_side");
    let attestation_side = AttestationSide::from_str_value(&attestation_side_value)
        .ok_or(OverrideAttestationRepositoryError::UnknownAttestationSide(attestation_side_value))?;

    let command_kind_value: String = row.get("command_kind");
    let command_kind = OverrideCommandKind::from_str_value(&command_kind_value)
        .ok_or(OverrideAttestationRepositoryError::UnknownCommandKind(command_kind_value))?;

    let outcome_value: String = row.get("outcome");
    let outcome = OverrideOutcomeLabel::from_str_value(&outcome_value)
        .ok_or(OverrideAttestationRepositoryError::UnknownOutcome(outcome_value))?;

    Ok(OverrideAttestationRow {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        event_sequence_id: row.get("event_sequence_id"),
        owner_id: row.get("owner_id"),
        parent_owner_id: row.get("parent_owner_id"),
        node_id: row.get("node_id"),
        attestation_side,
        command_kind,
        target_ref: row.get("target_ref"),
        outcome,
        justification: row.get("justification"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    fn sample_new_hierarchy() -> NewAccountHierarchy {
        NewAccountHierarchy {
            owner_id: "trader-7".to_string(),
            parent_owner_id: Some("fund-X".to_string()),
            consent_ref: "v1".to_string(),
            node_id: "node-A".to_string(),
        }
    }

    fn sample_attestation_input(side: AttestationSide, outcome: OverrideOutcomeLabel) -> RecordOverrideAttestationInput {
        RecordOverrideAttestationInput {
            owner_id: "trader-7".to_string(),
            parent_owner_id: "fund-X".to_string(),
            node_id: "node-A".to_string(),
            attestation_side: side,
            command_kind: OverrideCommandKind::Archive,
            target_ref: "strategy-42".to_string(),
            outcome,
            justification: Some("riesgo excedido".to_string()),
        }
    }

    // ── CRITERIO: esquema STRICT + Grupo I + row_version / event_sequence_id ──

    #[tokio::test]
    async fn migration_creates_account_hierarchy_strict_with_row_version_and_no_event_sequence_id() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('account_hierarchy')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();

        for expected in [
            "id", "created_at", "updated_at", "audit_hash", "audit_chain_hash", "row_version",
            "owner_id", "parent_owner_id", "consent_ref", "node_id",
        ] {
            assert!(column_names.contains(&expected.to_string()), "falta la columna: {expected}");
        }
        assert!(
            !column_names.contains(&"event_sequence_id".to_string()),
            "account_hierarchy es MUTABLE (ADR-0141): no debe tener event_sequence_id"
        );

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'account_hierarchy'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(sql.contains("STRICT"), "account_hierarchy debe declararse STRICT");
    }

    #[tokio::test]
    async fn migration_creates_override_attestations_strict_with_event_sequence_id_and_no_row_version() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('override_attestations')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();

        for expected in [
            "id", "created_at", "updated_at", "audit_hash", "audit_chain_hash", "event_sequence_id",
            "owner_id", "parent_owner_id", "node_id",
            "attestation_side", "command_kind", "target_ref", "outcome", "justification",
        ] {
            assert!(column_names.contains(&expected.to_string()), "falta la columna: {expected}");
        }
        assert!(
            !column_names.contains(&"row_version".to_string()),
            "override_attestations es APPEND-ONLY (ADR-0141): no debe tener row_version"
        );

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'override_attestations'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(sql.contains("STRICT"), "override_attestations debe declararse STRICT");
    }

    // ── account_hierarchy: link_child / concurrencia optimista ──────────────

    #[tokio::test]
    async fn link_child_persists_row_version_one_and_reloads_identically() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AccountHierarchyRepository::new(&pool, &clock);

        let row = repo.link_child(sample_new_hierarchy()).await.expect("vincular hija");
        assert_eq!(row.row_version, 1);
        assert_eq!(row.audit_chain_hash, None);
        assert_eq!(row.parent_owner_id.as_deref(), Some("fund-X"));

        let reloaded = repo.find_by_owner("trader-7").await.expect("releer").expect("debe existir");
        assert_eq!(reloaded, row);
    }

    /// CRITERIO DE CIERRE: dos actualizaciones que parten de la MISMA
    /// versión en memoria no pueden ambas tener éxito -- la primera avanza
    /// (1 -> 2); la segunda, que sigue creyendo estar en la versión 1,
    /// devuelve `VersionConflict` en vez de pisar el cambio de la primera.
    #[tokio::test]
    async fn concurrent_updates_from_same_row_version_conflict_instead_of_overwriting() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AccountHierarchyRepository::new(&pool, &clock);

        let hierarchy = repo.link_child(sample_new_hierarchy()).await.expect("vincular hija");
        let first_writer_view = hierarchy.clone();
        let second_writer_view = hierarchy;

        clock.tick();
        let updated = repo
            .update_parent_and_consent(&first_writer_view, Some("fund-Y"), "v2")
            .await
            .expect("el primer update debe tener éxito");
        assert_eq!(updated.row_version, 2);
        assert_eq!(updated.parent_owner_id.as_deref(), Some("fund-Y"));

        clock.tick();
        let conflict = repo.update_parent_and_consent(&second_writer_view, Some("fund-Z"), "v3").await;
        assert!(
            matches!(conflict, Err(AccountHierarchyRepositoryError::VersionConflict { expected: 1, .. })),
            "el segundo update desde la versión 1 debe dar VersionConflict; fue: {conflict:?}"
        );

        let reloaded = repo.find_by_owner("trader-7").await.expect("releer").expect("existe");
        assert_eq!(reloaded.row_version, 2);
        assert_eq!(
            reloaded.parent_owner_id.as_deref(),
            Some("fund-Y"),
            "debe conservarse el cambio del PRIMER writer, no el del segundo"
        );
    }

    // ── CRITERIO (QA por mutación, DEBT-018): fidelidad de la fila devuelta ───

    /// CRITERIO DE CIERRE (QA por mutación): la fila que DEVUELVE
    /// `update_parent_and_consent` refleja los valores FRESCOS/persistidos
    /// campo por campo -- si el literal `..current.clone()` de la función
    /// perdiera alguno de los cuatro campos que la operación produce nuevos
    /// (`updated_at_ns`, `audit_hash`, `audit_chain_hash`, `consent_ref`),
    /// esta comparación contra lo persistido en disco lo detectaría. Cada
    /// valor nuevo se elige DISTINTO del de la fila génesis para que el
    /// mutante (que caería al valor viejo copiado) produzca un resultado
    /// diferente al esperado. Patrón de referencia:
    /// `persistence/data_portability.rs::reclassify_returned_row_reflects_new_hash_chain_and_timestamp`
    /// (STORY-043, DEBT-018).
    #[tokio::test]
    async fn update_parent_and_consent_returned_row_reflects_fresh_fields_and_matches_persisted() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AccountHierarchyRepository::new(&pool, &clock);

        // Génesis: parent "fund-X", consent "v1", updated_at 1_000, sin cadena.
        let hierarchy = repo.link_child(sample_new_hierarchy()).await.expect("vincular hija");
        assert_eq!(hierarchy.updated_at_ns, 1_000, "precondición: la fila génesis nace en el now inicial");
        assert!(hierarchy.audit_chain_hash.is_none(), "precondición: la fila génesis no encadena");
        assert_eq!(hierarchy.consent_ref, "v1", "precondición: consent génesis");

        clock.tick(); // 1_000 -> 1_100
        // consent NUEVO ("v2") distinto del génesis ("v1") para discriminar
        // el mutante "delete field consent_ref".
        let updated = repo
            .update_parent_and_consent(&hierarchy, Some("fund-Y"), "v2")
            .await
            .expect("actualizar padre y consentimiento");

        // Campos NUEVOS de la operación en la fila DEVUELTA -- cada uno
        // distinto de su valor génesis (que es donde caería el mutante).
        assert_eq!(
            updated.updated_at_ns, 1_100,
            "el updated_at devuelto debe ser el now del reloj tras el tick, no el viejo (1_000)"
        );
        assert_ne!(
            updated.audit_hash, hierarchy.audit_hash,
            "el audit_hash devuelto debe ser el recomputado, no el viejo copiado"
        );
        assert_eq!(
            updated.audit_chain_hash,
            Some(hierarchy.audit_hash.clone()),
            "el audit_chain_hash devuelto debe encadenar al audit_hash génesis (no quedar en None)"
        );
        assert_eq!(updated.consent_ref, "v2", "el consent_ref devuelto debe ser el nuevo, no el viejo copiado");

        // Espejo contra lo PERSISTIDO en disco -- la fila devuelta debe ser
        // bit-a-bit igual a la que quedó en la BD (mata cualquier campo
        // borrado que aún no cubran las aserciones puntuales de arriba).
        let reloaded = repo.find_by_owner("trader-7").await.expect("releer").expect("existe");
        assert_eq!(
            updated, reloaded,
            "la fila devuelta por update_parent_and_consent debe coincidir campo por campo con la persistida"
        );
    }

    #[tokio::test]
    async fn database_check_rejects_unknown_attestation_side() {
        let pool = migrated_pool().await;
        let result = sqlx::query(
            "INSERT INTO override_attestations (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, parent_owner_id, node_id, attestation_side, command_kind, target_ref, outcome, justification\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'trader-7', 'fund-X', 'node-A', \
                       'UNKNOWN_SIDE', 'ARCHIVE', 'strategy-42', 'EXECUTED', NULL)",
        )
        .execute(&pool)
        .await;
        assert!(result.is_err(), "un attestation_side fuera del catálogo debe ser rechazado por el CHECK");
    }

    #[tokio::test]
    async fn database_check_rejects_unknown_command_kind() {
        let pool = migrated_pool().await;
        let result = sqlx::query(
            "INSERT INTO override_attestations (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, parent_owner_id, node_id, attestation_side, command_kind, target_ref, outcome, justification\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'trader-7', 'fund-X', 'node-A', \
                       'ISSUER', 'UNKNOWN_KIND', 'strategy-42', 'EXECUTED', NULL)",
        )
        .execute(&pool)
        .await;
        assert!(result.is_err(), "un command_kind fuera del catálogo debe ser rechazado por el CHECK");
    }

    #[tokio::test]
    async fn database_check_rejects_unknown_outcome() {
        let pool = migrated_pool().await;
        let result = sqlx::query(
            "INSERT INTO override_attestations (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, parent_owner_id, node_id, attestation_side, command_kind, target_ref, outcome, justification\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'trader-7', 'fund-X', 'node-A', \
                       'ISSUER', 'ARCHIVE', 'strategy-42', 'UNKNOWN_OUTCOME', NULL)",
        )
        .execute(&pool)
        .await;
        assert!(result.is_err(), "un outcome fuera del catálogo debe ser rechazado por el CHECK");
    }

    // ── override_attestations: append-only -- UPDATE/DELETE rechazados ──────

    #[tokio::test]
    async fn update_is_rejected_by_trigger_on_override_attestations() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = OverrideAttestationRepository::new(&pool, &clock);

        let row = repo
            .record_attestation(sample_attestation_input(AttestationSide::Issuer, OverrideOutcomeLabel::Executed))
            .await
            .expect("registrar atestación");

        let result = sqlx::query("UPDATE override_attestations SET outcome = 'DENIED' WHERE id = ?")
            .bind(&row.id)
            .execute(&pool)
            .await;
        assert!(result.is_err(), "UPDATE sobre override_attestations debe ser rechazado por el trigger");
    }

    #[tokio::test]
    async fn delete_is_rejected_by_trigger_on_override_attestations() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = OverrideAttestationRepository::new(&pool, &clock);

        let row = repo
            .record_attestation(sample_attestation_input(AttestationSide::Issuer, OverrideOutcomeLabel::Executed))
            .await
            .expect("registrar atestación");

        let result = sqlx::query("DELETE FROM override_attestations WHERE id = ?").bind(&row.id).execute(&pool).await;
        assert!(result.is_err(), "DELETE sobre override_attestations debe ser rechazado por el trigger");
    }

    #[tokio::test]
    async fn event_sequence_id_is_monotonic_and_chain_is_recomputable() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = OverrideAttestationRepository::new(&pool, &clock);

        let issuer = repo
            .record_attestation(sample_attestation_input(AttestationSide::Issuer, OverrideOutcomeLabel::Executed))
            .await
            .expect("issuer");
        clock.tick();
        let executor = repo
            .record_attestation(sample_attestation_input(AttestationSide::Executor, OverrideOutcomeLabel::Executed))
            .await
            .expect("executor");

        assert_eq!(issuer.event_sequence_id, 1);
        assert_eq!(executor.event_sequence_id, 2);
        assert_eq!(issuer.audit_chain_hash, None, "génesis debe tener audit_chain_hash NULL");
        assert_eq!(executor.audit_chain_hash, Some(issuer.audit_hash.clone()));

        let chain = repo.load_chain().await.expect("cargar cadena");
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0], issuer);
        assert_eq!(chain[1], executor);
    }

    // ── CRITERIO: append atómico + concurrencia (16 escritores) ─────────────

    /// CRITERIO DE CIERRE: 16 escritores concurrentes sobre el MISMO
    /// pool/ledger, en un archivo SQLite temporal (NUNCA `:memory:`, donde
    /// cada conexión sería una base distinta). La transacción
    /// `BEGIN IMMEDIATE` + reintento acotado debe garantizar que NINGUNA
    /// atestación se pierde y que la secuencia queda densa (1..=N). Esta
    /// prueba DEBE poder caerse si se quita la transacción (dos escritores
    /// leerían el mismo MAX, el UNIQUE rechazaría a uno y su fila se
    /// perdería).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_record_attestations_persist_every_row_without_gaps_or_lost_rows() {
        use std::sync::Arc;

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("override_attestations_concurrency.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());
        let pool = connect(&database_url).await.expect("conectar");
        migrate(&pool).await.expect("migrar");

        let clock: Arc<DeterministicClock> = Arc::new(DeterministicClock::new(1_000, 100));
        const N: i64 = 16;

        let mut handles = Vec::new();
        for i in 0..N {
            let pool_c = pool.clone();
            let clock_c = clock.clone();
            handles.push(tokio::spawn(async move {
                let repo = OverrideAttestationRepository::new(&pool_c, clock_c.as_ref());
                let side = if i % 2 == 0 { AttestationSide::Issuer } else { AttestationSide::Executor };
                repo.record_attestation(RecordOverrideAttestationInput {
                    owner_id: format!("trader-{i}"),
                    parent_owner_id: "fund-X".to_string(),
                    node_id: format!("node-{i}"),
                    attestation_side: side,
                    command_kind: OverrideCommandKind::Archive,
                    target_ref: format!("strategy-{i}"),
                    outcome: OverrideOutcomeLabel::Executed,
                    justification: None,
                })
                .await
            }));
        }

        for handle in handles {
            handle
                .await
                .expect("la tarea no debe entrar en panic")
                .expect("record_attestation debe tener éxito para cada escritor concurrente");
        }

        let repo = OverrideAttestationRepository::new(&pool, clock.as_ref());
        let chain = repo.load_chain().await.expect("cargar la cadena completa");

        assert_eq!(chain.len() as i64, N, "deben persistirse las N filas, sin ninguna pérdida");

        let sequence_ids: Vec<i64> = chain.iter().map(|row| row.event_sequence_id).collect();
        let expected: Vec<i64> = (1..=N).collect();
        assert_eq!(sequence_ids, expected, "los event_sequence_id deben ser 1..=N sin huecos ni duplicados");
    }

    // ── CRITERIO (QA por mutación, DEBT-018): reintento acotado hasta AGOTAR ──

    /// CRITERIO DE CIERRE (QA por mutación): bajo contención de escritura
    /// SOSTENIDA (otro escritor retiene el lock de `BEGIN IMMEDIATE` y no lo
    /// suelta), el bucle de reintento de `record_attestation` debe agotar
    /// EXACTAMENTE `MAX_RECORD_ATTEMPTS` intentos y rendirse con
    /// `WriteContention { attempts: MAX }` -- nunca descartar la atestación
    /// en silencio, ni rendirse un intento antes o después. Patrón de
    /// referencia: `persistence/data_portability.rs` (STORY-043, DEBT-018).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn record_attestation_exhausts_exactly_max_attempts_when_write_lock_is_held() {
        use std::str::FromStr;
        use std::time::Duration;

        use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("master_account_hierarchy_forced_contention.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());

        // Migrar con el pool normal (busy_timeout de 5s).
        let pool = connect(&database_url).await.expect("conectar");
        migrate(&pool).await.expect("migrar");

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

        // Escritor B: intenta registrar una atestación mientras A retiene el
        // lock. Cada `try_record_once` abre `BEGIN IMMEDIATE`, choca con el
        // lock de A, falla con "database is locked" (transitorio) y
        // reintenta, hasta agotar MAX_RECORD_ATTEMPTS.
        let repo_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(immediate_opts())
            .await
            .expect("pool del repositorio");
        let clock = DeterministicClock::new(1_000, 100);
        let repo = OverrideAttestationRepository::new(&repo_pool, &clock);

        let result = repo
            .record_attestation(sample_attestation_input(AttestationSide::Issuer, OverrideOutcomeLabel::Executed))
            .await;

        drop(lock_tx); // libera el lock (limpieza; el resultado ya está tomado)

        match result {
            Err(OverrideAttestationRepositoryError::WriteContention { attempts }) => {
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

        // Inserta una fila válida y luego otra con el MISMO `id`: viola la
        // PRIMARY KEY `id`, NO el UNIQUE de `event_sequence_id`. Error UNIQUE
        // PERMANENTE cuyo mensaje NO menciona `event_sequence_id`.
        let insert_with_id_dup = |event_sequence_id: i64| {
            sqlx::query(
                "INSERT INTO override_attestations (\
                    id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, parent_owner_id, node_id, attestation_side, command_kind, target_ref, outcome, justification\
                ) VALUES ('dup-id', 0, 0, 'hash', NULL, ?, 'trader-7', 'fund-X', 'node-A', \
                           'ISSUER', 'ARCHIVE', 'strategy-42', 'EXECUTED', NULL)",
            )
            .bind(event_sequence_id)
            .execute(&pool)
        };
        insert_with_id_dup(1).await.expect("primera fila válida");
        let err = insert_with_id_dup(2)
            .await
            .expect_err("la segunda fila debe violar la PRIMARY KEY id");

        let permanent = OverrideAttestationRepositoryError::Database(err);
        assert!(
            !is_transient_write_conflict(&permanent),
            "una violación UNIQUE de la PK (no de event_sequence_id) es PERMANENTE, no transitoria"
        );

        // Control: un error que ni siquiera es de base de datos jamás es
        // transitorio (fija la rama temprana `let ... else`).
        let non_database = OverrideAttestationRepositoryError::UnknownOutcome("X".to_string());
        assert!(
            !is_transient_write_conflict(&non_database),
            "un error no-Database nunca es contención transitoria"
        );
    }
}
