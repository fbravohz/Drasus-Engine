//! [SHELL] Repositorios de persistencia de Data Portability
//! (`docs/features/data-portability.md`, ADR-0148 -- cimiento #13,
//! ADR-0093, ADR-0141, ADR-0020, migración `0019_data_portability.sql`,
//! STORY-043).
//!
//! DOS repositorios, uno por tabla (mismo criterio que los doce cimientos
//! previos del substrato):
//! - [`ExportableDataCatalogRepository`][]: `exportable_data_catalog`,
//!   MUTABLE con `row_version` (concurrencia optimista ->
//!   [`ExportableDataCatalogRepositoryError::VersionConflict`]), mismo
//!   patrón que [`crate::persistence::plan_tier_quota::PlanRepository`].
//!   `declare_table` es IDEMPOTENTE por `table_name` -- registrar la misma
//!   tabla dos veces no duplica la fila.
//! - [`DataPortabilityRequestRepository`][]: `data_portability_requests`,
//!   APPEND-ONLY ATÓMICA (`event_sequence_id UNIQUE`, `BEGIN IMMEDIATE` +
//!   reintento acotado), mismo patrón que
//!   [`crate::persistence::verified_account_registry::AttestedTrackRecordRepository`]
//!   (causa raíz DEBT-001).
//!
//! La lógica pura (decisión de disposición del olvido, filtro de secretos,
//! manifiesto de exportación, hashes de auditoría de ambas tablas) vive en
//! [`crate::domain::data_portability`] -- este módulo solo le da entradas
//! inyectadas y persiste/carga el resultado.

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::audit_log::GENESIS_PREVIOUS_HASH;
use crate::domain::clock::Clock;
use crate::domain::data_portability::{
    compute_catalog_audit_hash, compute_request_audit_hash, CatalogEntry, RequestStatus, RequestType,
};

// ── `exportable_data_catalog` -- MUTABLE, row_version, idempotente ─────────

/// Errores que devuelven las operaciones de [`ExportableDataCatalogRepository`].
#[derive(Debug, thiserror::Error)]
pub enum ExportableDataCatalogRepositoryError {
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// Concurrencia optimista (ADR-0141): el UPDATE de reclasificación
    /// partió de un `row_version` que ya no es el vigente en disco -- otra
    /// escritura reclasificó la fila en el ínterin.
    #[error("conflicto de versión en el catálogo de la tabla '{table_name}': se esperaba row_version {expected}, la fila ya avanzó")]
    VersionConflict { table_name: String, expected: i64 },
}

/// Una tabla candidata a declarar en el catálogo -- todo lo que hace falta
/// para auto-declararse (`docs/features/data-portability.md`: "catálogo
/// declarativo de qué tablas portan `owner_id`").
#[derive(Debug, Clone)]
pub struct NewCatalogEntry {
    pub table_name: String,
    pub feature_name: String,
    pub owner_id_column: String,
    pub retention_exempt: bool,
}

/// Una fila de `exportable_data_catalog` ya persistida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportableDataCatalogRow {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub row_version: i64,

    pub table_name: String,
    pub feature_name: String,
    pub owner_id_column: String,
    pub retention_exempt: bool,
}

impl From<&ExportableDataCatalogRow> for CatalogEntry {
    /// Proyecta la fila persistida al tipo de dominio [`CatalogEntry`] que
    /// consume el Core (`build_export_manifest`/`build_forget_disposition_detail`)
    /// -- el Core nunca ve columnas SQL, solo este tipo liviano.
    fn from(row: &ExportableDataCatalogRow) -> Self {
        CatalogEntry {
            table_name: row.table_name.clone(),
            feature_name: row.feature_name.clone(),
            owner_id_column: row.owner_id_column.clone(),
            retention_exempt: row.retention_exempt,
        }
    }
}

/// Repositorio MUTABLE para `exportable_data_catalog`.
pub struct ExportableDataCatalogRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> ExportableDataCatalogRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Declara una tabla en el catálogo -- IDEMPOTENTE por `table_name`:
    /// si ya existe una fila con ese `table_name`, la devuelve tal cual
    /// (sin duplicar, sin tocar disco de nuevo); solo si NO existe, la
    /// inserta con `row_version = 1`. Mismo espíritu que
    /// `plan_tier_quota`/`consent_registry` "seed_default_*": la
    /// auto-declaración de una Feature nueva nunca debe fallar por
    /// reintentarse.
    pub async fn declare_table(
        &self,
        new: NewCatalogEntry,
    ) -> Result<ExportableDataCatalogRow, ExportableDataCatalogRepositoryError> {
        if let Some(existing) = self.find_by_table_name(&new.table_name).await? {
            return Ok(existing);
        }

        let id = Uuid::now_v7().to_string();
        let now_ns = self.clock.timestamp_ns();
        let row_version: i64 = 1;

        let audit_hash = compute_catalog_audit_hash(
            &id,
            now_ns,
            row_version,
            None,
            &new.table_name,
            &new.feature_name,
            &new.owner_id_column,
            new.retention_exempt,
        );

        sqlx::query(
            "INSERT INTO exportable_data_catalog (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                table_name, feature_name, owner_id_column, retention_exempt\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(Option::<String>::None)
        .bind(row_version)
        .bind(&new.table_name)
        .bind(&new.feature_name)
        .bind(&new.owner_id_column)
        .bind(new.retention_exempt as i64)
        .execute(self.pool)
        .await?;

        Ok(ExportableDataCatalogRow {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: None,
            row_version,
            table_name: new.table_name,
            feature_name: new.feature_name,
            owner_id_column: new.owner_id_column,
            retention_exempt: new.retention_exempt,
        })
    }

    /// Carga una fila del catálogo por `table_name`, o `None` si nunca se
    /// declaró.
    pub async fn find_by_table_name(
        &self,
        table_name: &str,
    ) -> Result<Option<ExportableDataCatalogRow>, ExportableDataCatalogRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    table_name, feature_name, owner_id_column, retention_exempt \
             FROM exportable_data_catalog WHERE table_name = ?",
        )
        .bind(table_name)
        .fetch_optional(self.pool)
        .await?;

        Ok(row.map(row_to_catalog_entry))
    }

    /// Carga TODO el catálogo, ordenado por `table_name` -- query path
    /// principal de `build_export_manifest`/`build_forget_disposition_detail`
    /// (el Core ya vuelve a ordenar, este orden es solo para que un dump
    /// crudo también sea determinista).
    pub async fn load_all(&self) -> Result<Vec<ExportableDataCatalogRow>, ExportableDataCatalogRepositoryError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    table_name, feature_name, owner_id_column, retention_exempt \
             FROM exportable_data_catalog ORDER BY table_name ASC",
        )
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter().map(row_to_catalog_entry).collect())
    }

    /// Reclasifica `current` (cambia `retention_exempt`) -- el catálogo
    /// puede reclasificarse si una obligación legal nueva alcanza a una
    /// tabla (STORY-043 §3.1: "el catálogo puede reclasificarse").
    ///
    /// ## Concurrencia optimista (ADR-0141)
    ///
    /// El `UPDATE` filtra por `id` **y** `row_version = <el de `current`>`.
    /// Si otra escritura ya avanzó la fila desde que se leyó `current`, el
    /// `WHERE` no encuentra ninguna fila y se devuelve
    /// [`ExportableDataCatalogRepositoryError::VersionConflict`] en vez de
    /// pisar el cambio ajeno -- mismo patrón que
    /// `plan_tier_quota::PlanRepository::update_limits`.
    pub async fn reclassify(
        &self,
        current: &ExportableDataCatalogRow,
        new_retention_exempt: bool,
    ) -> Result<ExportableDataCatalogRow, ExportableDataCatalogRepositoryError> {
        let now_ns = self.clock.timestamp_ns();
        let row_version = current.row_version + 1;

        let audit_hash = compute_catalog_audit_hash(
            &current.id,
            now_ns,
            row_version,
            Some(&current.audit_hash),
            &current.table_name,
            &current.feature_name,
            &current.owner_id_column,
            new_retention_exempt,
        );

        let result = sqlx::query(
            "UPDATE exportable_data_catalog SET \
                updated_at = ?, audit_hash = ?, audit_chain_hash = ?, row_version = ?, retention_exempt = ? \
             WHERE id = ? AND row_version = ?",
        )
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&current.audit_hash)
        .bind(row_version)
        .bind(new_retention_exempt as i64)
        .bind(&current.id)
        .bind(current.row_version)
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ExportableDataCatalogRepositoryError::VersionConflict {
                table_name: current.table_name.clone(),
                expected: current.row_version,
            });
        }

        Ok(ExportableDataCatalogRow {
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: Some(current.audit_hash.clone()),
            row_version,
            retention_exempt: new_retention_exempt,
            ..current.clone()
        })
    }
}

/// Convierte una fila de `exportable_data_catalog` al tipo
/// [`ExportableDataCatalogRow`].
fn row_to_catalog_entry(row: sqlx::sqlite::SqliteRow) -> ExportableDataCatalogRow {
    let retention_exempt_int: i64 = row.get("retention_exempt");
    ExportableDataCatalogRow {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        row_version: row.get("row_version"),
        table_name: row.get("table_name"),
        feature_name: row.get("feature_name"),
        owner_id_column: row.get("owner_id_column"),
        retention_exempt: retention_exempt_int != 0,
    }
}

// ── `data_portability_requests` -- APPEND-ONLY ATÓMICA ─────────────────────

/// Errores que devuelven las operaciones de [`DataPortabilityRequestRepository`].
#[derive(Debug, thiserror::Error)]
pub enum DataPortabilityRequestRepositoryError {
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// El append no pudo completarse tras agotar los reintentos ante
    /// contención de escritura transitoria -- la solicitud NO se descartó
    /// en silencio (regla "Atomicidad de ledgers append-only",
    /// rust-engineer/SKILL.md §4).
    #[error("no se pudo registrar el evento de portabilidad tras {attempts} intentos por contención de escritura")]
    WriteContention { attempts: u32 },
    /// Una fila persistida tenía un `request_type` fuera del catálogo --
    /// error de integridad de datos, no debería ocurrir si el `CHECK` de la
    /// migración se respeta.
    #[error("request_type desconocido en data_portability_requests: '{0}'")]
    UnknownRequestType(String),
    /// Análogo para `status`.
    #[error("status desconocido en data_portability_requests: '{0}'")]
    UnknownStatus(String),
}

/// Número máximo de intentos del append ante contención de escritura
/// transitoria antes de rendirse con
/// [`DataPortabilityRequestRepositoryError::WriteContention`]. Mismo valor
/// (cinco) que el resto de los ledgers append-only del substrato.
const MAX_RECORD_ATTEMPTS: u32 = 5;

/// Decide si un error del repositorio es una contención de escritura
/// TRANSITORIA -- mismo criterio que
/// `master_account_hierarchy::is_transient_write_conflict`.
fn is_transient_write_conflict(error: &DataPortabilityRequestRepositoryError) -> bool {
    let DataPortabilityRequestRepositoryError::Database(sqlx::Error::Database(db)) = error else {
        return false;
    };

    let message = db.message().to_lowercase();
    if message.contains("database is locked") || message.contains("database table is locked") {
        return true;
    }

    db.is_unique_violation() && message.contains("event_sequence_id")
}

/// Entrada para [`DataPortabilityRequestRepository::record_event`] -- todo
/// lo que la Shell necesita para registrar UN evento de una solicitud
/// (creación o avance de estado, mismo `request_group_id`).
#[derive(Debug, Clone)]
pub struct RecordDataPortabilityRequestInput {
    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,
    pub compliance_status_id: Option<String>,
    pub request_type: RequestType,
    pub status: RequestStatus,
    /// Agrupa TODOS los eventos de UNA solicitud lógica -- el mismo valor a
    /// través de RECEIVED -> PROCESSING -> COMPLETED. Lo genera quien
    /// inicia la solicitud (el orquestador, `Uuid::now_v7()`), nunca este
    /// repositorio.
    pub request_group_id: String,
    /// JSON con el detalle de disposición (solo FORGET) -- `None` para
    /// EXPORT o para un FORGET sin detalle todavía resuelto.
    pub disposition_detail: Option<String>,
}

/// Una fila de `data_portability_requests` ya persistida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPortabilityRequestRow {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub event_sequence_id: i64,

    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,
    pub compliance_status_id: Option<String>,

    pub request_type: RequestType,
    pub status: RequestStatus,
    pub request_group_id: String,
    pub disposition_detail: Option<String>,
}

/// Repositorio APPEND-ONLY para `data_portability_requests`.
///
/// Al igual que `AttestedTrackRecordRepository`/`OverrideAttestationRepository`,
/// la única operación de escritura expuesta es [`Self::record_event`] (un
/// INSERT) -- no hay `update`/`delete`; los triggers
/// `trg_data_portability_requests_no_update`/`_no_delete` de la migración
/// los rechazarían de todas formas.
pub struct DataPortabilityRequestRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> DataPortabilityRequestRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Registra UN evento de solicitud: deriva su posición en la cadena
    /// GLOBAL, computa su `audit_hash` encadenado y lo persiste como fila
    /// nueva.
    ///
    /// ## Atomicidad bajo concurrencia (regla "Atomicidad de ledgers append-only")
    ///
    /// El *read-then-write* (leer el MAX(`event_sequence_id`)/`audit_hash`
    /// previo, y el `INSERT`) ocurre dentro de UNA sola transacción
    /// `BEGIN IMMEDIATE` -- ver [`Self::try_record_once`]. Sin ella, dos
    /// escritores concurrentes derivarían el mismo `event_sequence_id`, el
    /// `UNIQUE` rechazaría a uno y su evento se PERDERÍA. Ante contención
    /// transitoria se reintenta hasta [`MAX_RECORD_ATTEMPTS`] veces
    /// re-derivando la secuencia; nunca se descarta el evento en silencio.
    pub async fn record_event(
        &self,
        input: RecordDataPortabilityRequestInput,
    ) -> Result<DataPortabilityRequestRow, DataPortabilityRequestRepositoryError> {
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
                        return Err(DataPortabilityRequestRepositoryError::WriteContention { attempts: attempt });
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
        input: &RecordDataPortabilityRequestInput,
    ) -> Result<DataPortabilityRequestRow, DataPortabilityRequestRepositoryError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        let tail_row = sqlx::query(
            "SELECT audit_hash, event_sequence_id \
             FROM data_portability_requests \
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

        let request_type_str = input.request_type.as_str();
        let status_str = input.status.as_str();

        let audit_hash = compute_request_audit_hash(
            &id,
            now_ns,
            event_sequence_id,
            &previous_audit_hash,
            &input.owner_id,
            &input.institutional_tag,
            &input.node_id,
            &input.request_group_id,
            request_type_str,
            status_str,
            input.disposition_detail.as_deref(),
        );

        sqlx::query(
            "INSERT INTO data_portability_requests (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, compliance_status_id, \
                request_type, status, request_group_id, disposition_detail\
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
        .bind(&input.compliance_status_id)
        .bind(request_type_str)
        .bind(status_str)
        .bind(&input.request_group_id)
        .bind(&input.disposition_detail)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(DataPortabilityRequestRow {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash,
            event_sequence_id,
            owner_id: input.owner_id.clone(),
            institutional_tag: input.institutional_tag.clone(),
            node_id: input.node_id.clone(),
            compliance_status_id: input.compliance_status_id.clone(),
            request_type: input.request_type,
            status: input.status,
            request_group_id: input.request_group_id.clone(),
            disposition_detail: input.disposition_detail.clone(),
        })
    }

    /// Carga la cadena completa, ordenada por `event_sequence_id`
    /// ascendente -- usada por los tests de integridad de la cadena.
    pub async fn load_chain(&self) -> Result<Vec<DataPortabilityRequestRow>, DataPortabilityRequestRepositoryError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, node_id, compliance_status_id, \
                    request_type, status, request_group_id, disposition_detail \
             FROM data_portability_requests \
             ORDER BY event_sequence_id ASC",
        )
        .fetch_all(self.pool)
        .await?;

        rows.into_iter().map(row_to_request).collect()
    }

    /// Deriva el estado VIGENTE de una solicitud lógica -- el `status` del
    /// evento con `event_sequence_id` más alto para `request_group_id`, o
    /// `None` si esa solicitud nunca se registró. El avance de estado se
    /// modela SIEMPRE como un evento nuevo (nunca un UPDATE de la fila
    /// anterior, ver el trigger `trg_data_portability_requests_no_update`
    /// de la migración) -- este método es la única forma correcta de leer
    /// "en qué estado está esta solicitud AHORA".
    pub async fn latest_status_for(
        &self,
        request_group_id: &str,
    ) -> Result<Option<RequestStatus>, DataPortabilityRequestRepositoryError> {
        let row = sqlx::query(
            "SELECT status FROM data_portability_requests \
             WHERE request_group_id = ? \
             ORDER BY event_sequence_id DESC LIMIT 1",
        )
        .bind(request_group_id)
        .fetch_optional(self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };
        let status_value: String = row.get("status");
        let status = RequestStatus::from_str_value(&status_value)
            .ok_or(DataPortabilityRequestRepositoryError::UnknownStatus(status_value))?;
        Ok(Some(status))
    }
}

/// Convierte una fila de `data_portability_requests` al tipo
/// [`DataPortabilityRequestRow`], decodificando los enums de texto
/// persistidos.
fn row_to_request(
    row: sqlx::sqlite::SqliteRow,
) -> Result<DataPortabilityRequestRow, DataPortabilityRequestRepositoryError> {
    let request_type_value: String = row.get("request_type");
    let request_type = RequestType::from_str_value(&request_type_value)
        .ok_or(DataPortabilityRequestRepositoryError::UnknownRequestType(request_type_value))?;

    let status_value: String = row.get("status");
    let status = RequestStatus::from_str_value(&status_value)
        .ok_or(DataPortabilityRequestRepositoryError::UnknownStatus(status_value))?;

    Ok(DataPortabilityRequestRow {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        event_sequence_id: row.get("event_sequence_id"),
        owner_id: row.get("owner_id"),
        institutional_tag: row.get("institutional_tag"),
        node_id: row.get("node_id"),
        compliance_status_id: row.get("compliance_status_id"),
        request_type,
        status,
        request_group_id: row.get("request_group_id"),
        disposition_detail: row.get("disposition_detail"),
    })
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

    fn sample_catalog_entry() -> NewCatalogEntry {
        NewCatalogEntry {
            table_name: "verified_accounts".to_string(),
            feature_name: "verified-account-registry".to_string(),
            owner_id_column: "owner_id".to_string(),
            retention_exempt: false,
        }
    }

    fn sample_request_input(owner_id: &str, request_group_id: &str, status: RequestStatus) -> RecordDataPortabilityRequestInput {
        RecordDataPortabilityRequestInput {
            owner_id: owner_id.to_string(),
            institutional_tag: "LIVE".to_string(),
            node_id: "node-A".to_string(),
            compliance_status_id: None,
            request_type: RequestType::Export,
            status,
            request_group_id: request_group_id.to_string(),
            disposition_detail: None,
        }
    }

    // ── CRITERIO: esquema STRICT + Grupo I + row_version / event_sequence_id ──

    #[tokio::test]
    async fn migration_creates_catalog_strict_with_row_version_and_no_event_sequence_id() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('exportable_data_catalog')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();

        for expected in [
            "id", "created_at", "updated_at", "audit_hash", "audit_chain_hash", "row_version",
            "table_name", "feature_name", "owner_id_column", "retention_exempt",
        ] {
            assert!(column_names.contains(&expected.to_string()), "falta la columna: {expected}");
        }
        assert!(
            !column_names.contains(&"event_sequence_id".to_string()),
            "exportable_data_catalog es MUTABLE (ADR-0141): no debe tener event_sequence_id"
        );

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'exportable_data_catalog'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(sql.contains("STRICT"), "exportable_data_catalog debe declararse STRICT");
    }

    #[tokio::test]
    async fn migration_creates_requests_strict_with_event_sequence_id_and_no_row_version() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('data_portability_requests')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();

        for expected in [
            "id", "created_at", "updated_at", "audit_hash", "audit_chain_hash", "event_sequence_id",
            "owner_id", "institutional_tag", "node_id", "compliance_status_id",
            "request_type", "status", "request_group_id", "disposition_detail",
        ] {
            assert!(column_names.contains(&expected.to_string()), "falta la columna: {expected}");
        }
        assert!(
            !column_names.contains(&"row_version".to_string()),
            "data_portability_requests es APPEND-ONLY (ADR-0141): no debe tener row_version"
        );

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'data_portability_requests'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(sql.contains("STRICT"), "data_portability_requests debe declararse STRICT");
    }

    // ── CRITERIO (Orden §8): catálogo idempotente ────────────────────────────

    /// CRITERIO DE CIERRE: declarar la MISMA tabla dos veces no duplica la
    /// fila -- la segunda llamada devuelve la fila ya existente.
    #[tokio::test]
    async fn declare_table_is_idempotent_by_table_name() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = ExportableDataCatalogRepository::new(&pool, &clock);

        let first = repo.declare_table(sample_catalog_entry()).await.expect("primera declaración");
        clock.tick();
        let second = repo.declare_table(sample_catalog_entry()).await.expect("segunda declaración");

        assert_eq!(first, second, "la segunda declaración debe devolver la MISMA fila, no una nueva");

        let all = repo.load_all().await.expect("cargar catálogo");
        assert_eq!(all.len(), 1, "no debe haber ninguna fila duplicada");
    }

    /// CRITERIO DE CIERRE: `reclassify` con un `row_version` viejo (otra
    /// escritura ya avanzó la fila) devuelve `VersionConflict`.
    #[tokio::test]
    async fn reclassify_with_stale_row_version_conflicts_instead_of_overwriting() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = ExportableDataCatalogRepository::new(&pool, &clock);

        let entry = repo.declare_table(sample_catalog_entry()).await.expect("declarar");
        let stale_view = entry.clone();

        clock.tick();
        let updated = repo.reclassify(&entry, true).await.expect("primera reclasificación");
        assert_eq!(updated.row_version, 2);
        assert!(updated.retention_exempt);

        clock.tick();
        let conflict = repo.reclassify(&stale_view, false).await;
        assert!(
            matches!(conflict, Err(ExportableDataCatalogRepositoryError::VersionConflict { expected: 1, .. })),
            "la reclasificación desde la versión vieja debe dar VersionConflict; fue: {conflict:?}"
        );

        let reloaded = repo.find_by_table_name("verified_accounts").await.expect("releer").expect("existe");
        assert_eq!(reloaded.row_version, 2);
        assert!(reloaded.retention_exempt, "debe conservarse el cambio de la primera reclasificación");
    }

    // ── data_portability_requests: append-only, secuencia, estado vigente ──

    // ── CRITERIO DE CIERRE (ADR-0141 enmienda 2026-07-11, M6) ────────────────

    /// La FK física `data_portability_requests.owner_id -> accounts(id)`
    /// rechaza un `owner_id` que no corresponde a ninguna cuenta -- un
    /// huérfano ya no es un bug silencioso, la base de datos lo atrapa.
    #[tokio::test]
    async fn record_event_with_nonexistent_owner_id_is_rejected_by_foreign_key() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = DataPortabilityRequestRepository::new(&pool, &clock);

        let result = repo
            .record_event(sample_request_input("cuenta-que-no-existe", "grp-1", RequestStatus::Received))
            .await;

        assert!(
            matches!(result, Err(DataPortabilityRequestRepositoryError::Database(_))),
            "un owner_id huérfano debe rechazarse por la FK, no persistirse: {result:?}"
        );
    }

    #[tokio::test]
    async fn update_is_rejected_by_trigger_on_data_portability_requests() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let repo = DataPortabilityRequestRepository::new(&pool, &clock);

        let row = repo
            .record_event(sample_request_input(&owner_id, "grp-1", RequestStatus::Received))
            .await
            .expect("registrar evento");

        let result = sqlx::query("UPDATE data_portability_requests SET status = 'COMPLETED' WHERE id = ?")
            .bind(&row.id)
            .execute(&pool)
            .await;
        assert!(result.is_err(), "UPDATE sobre data_portability_requests debe ser rechazado por el trigger");
    }

    #[tokio::test]
    async fn delete_is_rejected_by_trigger_on_data_portability_requests() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let repo = DataPortabilityRequestRepository::new(&pool, &clock);

        let row = repo
            .record_event(sample_request_input(&owner_id, "grp-1", RequestStatus::Received))
            .await
            .expect("registrar evento");

        let result = sqlx::query("DELETE FROM data_portability_requests WHERE id = ?").bind(&row.id).execute(&pool).await;
        assert!(result.is_err(), "DELETE sobre data_portability_requests debe ser rechazado por el trigger");
    }

    #[tokio::test]
    async fn database_check_rejects_unknown_request_type() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let result = sqlx::query(
            "INSERT INTO data_portability_requests (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, compliance_status_id, \
                request_type, status, request_group_id, disposition_detail\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, ?, 'LIVE', 'node-A', NULL, \
                       'UNKNOWN_TYPE', 'RECEIVED', 'grp-1', NULL)",
        )
        .bind(&owner_id)
        .execute(&pool)
        .await;
        assert!(result.is_err(), "un request_type fuera del catálogo debe ser rechazado por el CHECK");
    }

    #[tokio::test]
    async fn database_check_rejects_invalid_json_in_disposition_detail() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let result = sqlx::query(
            "INSERT INTO data_portability_requests (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, compliance_status_id, \
                request_type, status, request_group_id, disposition_detail\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, ?, 'LIVE', 'node-A', NULL, \
                       'FORGET', 'RECEIVED', 'grp-1', '{not valid json')",
        )
        .bind(&owner_id)
        .execute(&pool)
        .await;
        assert!(result.is_err(), "disposition_detail con JSON corrupto debe ser rechazado por el CHECK(json_valid)");
    }

    #[tokio::test]
    async fn event_sequence_id_is_monotonic_and_chain_is_recomputable() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let repo = DataPortabilityRequestRepository::new(&pool, &clock);

        let first = repo.record_event(sample_request_input(&owner_id, "grp-1", RequestStatus::Received)).await.expect("primero");
        clock.tick();
        let second = repo.record_event(sample_request_input(&owner_id, "grp-2", RequestStatus::Received)).await.expect("segundo");

        assert_eq!(first.event_sequence_id, 1);
        assert_eq!(second.event_sequence_id, 2);
        assert_eq!(first.audit_chain_hash, None, "génesis debe tener audit_chain_hash NULL");
        assert_eq!(second.audit_chain_hash, Some(first.audit_hash.clone()));

        let chain = repo.load_chain().await.expect("cargar cadena");
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0], first);
        assert_eq!(chain[1], second);
    }

    /// CRITERIO DE CIERRE (Orden §8): dos eventos del MISMO
    /// `request_group_id` (RECEIVED -> PROCESSING) -- `latest_status_for`
    /// debe devolver el estado del evento MÁS RECIENTE, no el primero.
    #[tokio::test]
    async fn latest_status_for_returns_the_most_recent_event_of_the_group() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let repo = DataPortabilityRequestRepository::new(&pool, &clock);

        repo.record_event(sample_request_input(&owner_id, "grp-1", RequestStatus::Received)).await.expect("evento 1");
        clock.tick();
        repo.record_event(sample_request_input(&owner_id, "grp-1", RequestStatus::Processing)).await.expect("evento 2");

        let status = repo.latest_status_for("grp-1").await.expect("consultar estado vigente");
        assert_eq!(status, Some(RequestStatus::Processing), "debe devolver el estado del evento MÁS RECIENTE");

        let missing = repo.latest_status_for("grp-nunca-registrado").await.expect("consultar inexistente");
        assert_eq!(missing, None);
    }

    // ── CRITERIO (Orden §8): append atómico + concurrencia (16 escritores) ──

    /// CRITERIO DE CIERRE: 16 escritores concurrentes sobre el MISMO
    /// pool/ledger, en un archivo SQLite temporal (NUNCA `:memory:`, donde
    /// cada conexión sería una base distinta). La transacción
    /// `BEGIN IMMEDIATE` + reintento acotado debe garantizar que NINGUNA
    /// solicitud se pierde y que la secuencia queda densa (1..=N). Esta
    /// prueba DEBE poder caerse si se quita la transacción (dos escritores
    /// leerían el mismo MAX, el UNIQUE rechazaría a uno y su fila se
    /// perdería).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_record_events_persist_every_row_without_gaps_or_lost_rows() {
        use std::sync::Arc;

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("data_portability_requests_concurrency.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());
        let pool = connect(&database_url).await.expect("conectar");
        migrate(&pool).await.expect("migrar");

        let clock: Arc<DeterministicClock> = Arc::new(DeterministicClock::new(1_000, 100));
        const N: i64 = 16;

        // Sembrar N cuentas reales ANTES de la fase concurrente -- la FK
        // owner_id->accounts(id) exige que cada una ya exista; sembrarlas
        // secuencialmente aquí no interfiere con la concurrencia que se
        // ejercita abajo (esa es sobre `record_event`, no sobre el alta de
        // cuentas).
        let mut owner_ids = Vec::with_capacity(N as usize);
        for i in 0..N {
            owner_ids.push(seed_account(&pool, clock.as_ref(), &format!("owner{i}@example.com")).await);
        }

        let mut handles = Vec::new();
        for i in 0..N {
            let pool_c = pool.clone();
            let clock_c = clock.clone();
            let owner_id_c = owner_ids[i as usize].clone();
            handles.push(tokio::spawn(async move {
                let repo = DataPortabilityRequestRepository::new(&pool_c, clock_c.as_ref());
                repo.record_event(RecordDataPortabilityRequestInput {
                    owner_id: owner_id_c,
                    institutional_tag: "LIVE".to_string(),
                    node_id: format!("node-{i}"),
                    compliance_status_id: None,
                    request_type: RequestType::Export,
                    status: RequestStatus::Received,
                    request_group_id: format!("grp-{i}"),
                    disposition_detail: None,
                })
                .await
            }));
        }

        for handle in handles {
            handle
                .await
                .expect("la tarea no debe entrar en panic")
                .expect("record_event debe tener éxito para cada escritor concurrente");
        }

        let repo = DataPortabilityRequestRepository::new(&pool, clock.as_ref());
        let chain = repo.load_chain().await.expect("cargar la cadena completa");

        assert_eq!(chain.len() as i64, N, "deben persistirse las N filas, sin ninguna pérdida");

        let sequence_ids: Vec<i64> = chain.iter().map(|row| row.event_sequence_id).collect();
        let expected: Vec<i64> = (1..=N).collect();
        assert_eq!(sequence_ids, expected, "los event_sequence_id deben ser 1..=N sin huecos ni duplicados");
    }

    // ── CRITERIO (QA por mutación): reintento acotado hasta AGOTAR ────────────

    /// CRITERIO DE CIERRE (QA por mutación): bajo contención de escritura
    /// SOSTENIDA (otro escritor retiene el lock de `BEGIN IMMEDIATE` y no lo
    /// suelta), el bucle de reintento debe agotar EXACTAMENTE
    /// `MAX_RECORD_ATTEMPTS` intentos y rendirse con
    /// `WriteContention { attempts: MAX }` -- nunca descartar el evento en
    /// silencio, ni rendirse un intento antes o después. Fija de forma
    /// determinista: el incremento del contador (`attempt += 1`), el límite
    /// de corte (`attempt < MAX`), y que el clasificador trate
    /// "database is locked" como transitorio (`is_transient` != `false`,
    /// `||` != `&&` en la rama de lock).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn record_event_exhausts_exactly_max_attempts_when_write_lock_is_held() {
        use std::str::FromStr;
        use std::time::Duration;

        use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("dp_forced_contention.sqlite");
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
        // determinista y rápida (sin esto, 5 reintentos × 5s = 25s de espera).
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

        // Escritor B: intenta registrar un evento mientras A retiene el lock.
        // Cada `try_record_once` abre `BEGIN IMMEDIATE`, choca con el lock de
        // A, falla con "database is locked" (transitorio) y reintenta, hasta
        // agotar MAX_RECORD_ATTEMPTS.
        let repo_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(immediate_opts())
            .await
            .expect("pool del repositorio");
        let clock = DeterministicClock::new(1_000, 100);
        let repo = DataPortabilityRequestRepository::new(&repo_pool, &clock);

        let result = repo
            .record_event(sample_request_input(&owner_id, "grp-contention", RequestStatus::Received))
            .await;

        drop(lock_tx); // libera el lock (limpieza; el resultado ya está tomado)

        match result {
            Err(DataPortabilityRequestRepositoryError::WriteContention { attempts }) => {
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

    // ── CRITERIO (QA por mutación): clasificador de contención ────────────────

    /// CRITERIO DE CIERRE (QA por mutación): `is_transient_write_conflict`
    /// distingue una violación UNIQUE PERMANENTE (la PK `id`, que NO se debe
    /// reintentar) de la contención transitoria. Fija que exige AMBAS
    /// condiciones (es violación UNIQUE **y** menciona `event_sequence_id`),
    /// no una sola (`&&` != `||`), y que no clasifica cualquier cosa como
    /// transitoria (`is_transient` != `true`).
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
                "INSERT INTO data_portability_requests \
                 (id, created_at, updated_at, audit_hash, event_sequence_id, owner_id, \
                  institutional_tag, node_id, request_type, status, request_group_id) \
                 VALUES ('dup-id', 1, 1, 'h', ?, ?, 'LIVE', 'n', 'EXPORT', 'RECEIVED', 'g')",
            )
            .bind(event_sequence_id)
            .bind(&owner_id)
            .execute(&pool)
        };
        insert_with_id_dup(1).await.expect("primera fila válida");
        let err = insert_with_id_dup(2)
            .await
            .expect_err("la segunda fila debe violar la PRIMARY KEY id");

        let permanent = DataPortabilityRequestRepositoryError::Database(err);
        assert!(
            !is_transient_write_conflict(&permanent),
            "una violación UNIQUE de la PK (no de event_sequence_id) es PERMANENTE, no transitoria"
        );

        // Control: un error que ni siquiera es de base de datos jamás es
        // transitorio (fija la rama temprana `let ... else`).
        let non_database = DataPortabilityRequestRepositoryError::UnknownStatus("X".to_string());
        assert!(
            !is_transient_write_conflict(&non_database),
            "un error no-Database nunca es contención transitoria"
        );
    }

    // ── CRITERIO (QA por mutación): fidelidad de la fila devuelta ─────────────

    /// CRITERIO DE CIERRE (QA por mutación): la fila que DEVUELVE `reclassify`
    /// refleja los valores NUEVOS (audit_hash recomputado, audit_chain_hash
    /// encadenado a la versión previa, updated_at avanzado al reloj), no los
    /// viejos copiados de la fila de entrada vía `..current.clone()`.
    #[tokio::test]
    async fn reclassify_returned_row_reflects_new_hash_chain_and_timestamp() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = ExportableDataCatalogRepository::new(&pool, &clock);

        let entry = repo.declare_table(sample_catalog_entry()).await.expect("declarar");
        assert_eq!(entry.updated_at_ns, 1_000, "precondición: la fila génesis nace en el now inicial");
        assert!(entry.audit_chain_hash.is_none(), "precondición: la fila génesis no encadena");

        clock.tick(); // 1_000 -> 1_100
        let updated = repo.reclassify(&entry, true).await.expect("reclasificar");

        assert_ne!(
            updated.audit_hash, entry.audit_hash,
            "reclassify debe DEVOLVER el audit_hash recomputado, no el viejo copiado"
        );
        assert_eq!(
            updated.audit_chain_hash,
            Some(entry.audit_hash.clone()),
            "el audit_chain_hash devuelto debe encadenar al audit_hash de la versión previa"
        );
        assert_eq!(
            updated.updated_at_ns, 1_100,
            "el updated_at devuelto debe ser el now del reloj tras el tick, no el viejo"
        );
    }
}
