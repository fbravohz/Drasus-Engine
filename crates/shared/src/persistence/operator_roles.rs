//! [SHELL] Repositorios de persistencia de Operator Roles
//! (`docs/features/operator-roles.md`, ADR-0149 -- cimiento #14, ADR-0123,
//! ADR-0141, ADR-0020, migración `0020_operator_roles.sql`, STORY-044).
//!
//! TRES repositorios, uno por tabla (mismo criterio que los trece cimientos
//! previos del substrato):
//! - [`OperatorRoleRepository`][]: `operator_roles`, MUTABLE con
//!   `row_version` (concurrencia optimista -> [`OperatorRoleError::VersionConflict`]).
//! - [`OperatorAssignmentRepository`][]: `operator_assignments`, MUTABLE con
//!   `row_version` -- `set_assignment` es un UPSERT (`UNIQUE(owner_id,
//!   access_token_id)`: un operador tiene UN rol vigente por cuenta,
//!   reasignar actualiza la MISMA fila).
//! - [`OperatorRoleEventRepository`][]: `operator_role_events`, APPEND-ONLY
//!   ATÓMICA (`event_sequence_id UNIQUE`, `BEGIN IMMEDIATE` + reintento
//!   acotado), mismo patrón que
//!   [`crate::persistence::data_portability::DataPortabilityRequestRepository`]
//!   (causa raíz DEBT-001).
//!
//! ## El guardarraíl transaccional "último admin en pie" (STORY-044 §5)
//!
//! Las mutaciones que pueden afectar el invariante ([`OperatorRoleRepository::update_role_matrix`],
//! [`OperatorRoleRepository::revoke_role`], [`OperatorAssignmentRepository::set_assignment`],
//! [`OperatorAssignmentRepository::revoke_assignment`]) hacen, DENTRO de UNA
//! sola transacción `BEGIN IMMEDIATE`: (1) cargar el estado admin-relevante
//! vigente (roles + asignaciones ACTIVOS de la cuenta), (2) llamar a
//! [`crate::domain::operator_roles::check_last_admin_standing`] con el
//! cambio propuesto, (3) abortar (rollback implícito por `Drop` de la
//! transacción sin `commit`) si el Core lo rechaza, (4) solo entonces
//! escribir la fila Y el evento del ledger -- ambos en la MISMA transacción.
//! El chequeo cross-fila NO puede depender solo de `row_version` de una
//! fila: por eso vive en la transacción que serializa los cambios
//! admin-afectantes (regla "Atomicidad de ledgers append-only",
//! rust-engineer/SKILL.md §4).
//!
//! La lógica pura (matriz de capacidades, gate compuesto, invariante
//! "último admin en pie", gate de cuota de cuentas hijas, hashes de
//! auditoría de las tres tablas) vive en [`crate::domain::operator_roles`]
//! -- este módulo solo le da entradas inyectadas y persiste/carga el
//! resultado.

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::audit_log::GENESIS_PREVIOUS_HASH;
use crate::domain::clock::Clock;
use crate::domain::operator_roles::{
    compute_assignment_audit_hash, compute_event_audit_hash, compute_role_audit_hash, AssignmentView,
    CapabilityMatrix, LastAdminViolation, LifecycleStatus, OperatorRoleChangeType, OperatorType, ProposedChange,
    RoleView,
};

/// Número máximo de intentos ante contención de escritura transitoria antes
/// de rendirse con [`OperatorRoleError::WriteContention`] -- mismo valor
/// (cinco) que el resto de los ledgers append-only del substrato.
const MAX_GUARDED_ATTEMPTS: u32 = 5;

/// Errores que devuelven las operaciones de este módulo -- unificado porque
/// las mutaciones guardadas tocan más de una tabla dentro de la MISMA
/// transacción (rol/asignación + ledger).
#[derive(Debug, thiserror::Error)]
pub enum OperatorRoleError {
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// El append/mutación no pudo completarse tras agotar los reintentos
    /// ante contención de escritura transitoria -- el cambio NO se
    /// descartó en silencio.
    #[error("no se pudo completar la escritura tras {attempts} intentos por contención de escritura")]
    WriteContention { attempts: u32 },
    /// Concurrencia optimista (ADR-0141): la escritura partió de un
    /// `row_version` que ya no es el vigente en disco.
    #[error("conflicto de versión: se esperaba row_version {expected}, la fila ya avanzó")]
    VersionConflict { expected: i64 },
    /// El cambio propuesto viola el invariante "último admin en pie"
    /// (ADR-0149) -- la transacción se descartó ANTES de escribir nada.
    #[error(transparent)]
    LastAdmin(#[from] LastAdminViolation),
    /// `set_assignment` referencia un `role_id` que no existe o ya está
    /// revocado en esta cuenta.
    #[error("el rol '{0}' no existe o ya no está activo en esta cuenta")]
    RoleNotFound(String),
    /// Se pidió revocar/consultar la asignación ACTIVA de un operador que
    /// no tiene ninguna -- ADR-0149: sin asignación explícita, el operador
    /// ya está denegado; no hay nada que revocar.
    #[error("el operador '{0}' no tiene ninguna asignación activa en esta cuenta")]
    AssignmentNotFound(String),
    /// Una fila de `operator_assignments` tenía un `operator_type` fuera
    /// del catálogo -- no debería ocurrir dado el `CHECK` de la migración.
    #[error("operator_type desconocido: '{0}'")]
    UnknownOperatorType(String),
    /// Una fila tenía un `status` fuera de ACTIVE/REVOKED.
    #[error("status desconocido: '{0}'")]
    UnknownStatus(String),
    /// Una fila de `operator_role_events` tenía un `change_type` fuera del
    /// catálogo -- no debería ocurrir dado el `CHECK` de la migración.
    #[error("change_type desconocido: '{0}'")]
    UnknownChangeType(String),
    /// `capability_matrix` con JSON inválido -- no debería ocurrir dado el
    /// `CHECK (json_valid(...))` de la migración.
    #[error("capability_matrix con JSON inválido en la fila '{0}'")]
    MalformedCapabilityMatrix(String),
}

/// Decide si un error es una contención de escritura TRANSITORIA -- mismo
/// criterio que `data_portability::is_transient_write_conflict`: "database
/// is locked" (BEGIN IMMEDIATE chocó con otro escritor) o una violación
/// UNIQUE que menciona `event_sequence_id` (dos escritores derivaron la
/// misma posición). Cualquier otra violación UNIQUE (PK, `owner_id +
/// role_name`, `owner_id + access_token_id`) es PERMANENTE -- nunca se
/// reintenta.
fn is_transient_write_conflict(error: &OperatorRoleError) -> bool {
    let OperatorRoleError::Database(sqlx::Error::Database(db)) = error else {
        return false;
    };

    let message = db.message().to_lowercase();
    if message.contains("database is locked") || message.contains("database table is locked") {
        return true;
    }

    db.is_unique_violation() && message.contains("event_sequence_id")
}

// ── `operator_roles` -- MUTABLE ─────────────────────────────────────────────

/// Un rol nuevo para persistir.
#[derive(Debug, Clone)]
pub struct NewOperatorRole {
    pub owner_id: String,
    pub institutional_tag: String,
    pub role_name: String,
    pub capability_matrix: CapabilityMatrix,
}

/// Una fila de `operator_roles` ya persistida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorRoleRow {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub row_version: i64,

    pub owner_id: String,
    pub institutional_tag: String,
    pub role_name: String,
    pub capability_matrix: CapabilityMatrix,
    pub status: LifecycleStatus,
}

/// Repositorio MUTABLE para `operator_roles`.
pub struct OperatorRoleRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> OperatorRoleRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Crea un rol nuevo con `row_version = 1` y registra el evento
    /// `ROLE_CREATED` en la MISMA transacción -- no afecta el invariante
    /// "último admin en pie" (crear nunca resta admins), así que no pasa
    /// por el guardarraíl de conteo, solo por la atomicidad del ledger.
    pub async fn create_role(
        &self,
        new: NewOperatorRole,
        node_id: &str,
    ) -> Result<(OperatorRoleRow, OperatorRoleEventRow), OperatorRoleError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_create_role_once(&new, node_id).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if is_transient_write_conflict(&error) {
                        if attempt < MAX_GUARDED_ATTEMPTS {
                            continue;
                        }
                        return Err(OperatorRoleError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    async fn try_create_role_once(
        &self,
        new: &NewOperatorRole,
        node_id: &str,
    ) -> Result<(OperatorRoleRow, OperatorRoleEventRow), OperatorRoleError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        let id = Uuid::now_v7().to_string();
        let now_ns = self.clock.timestamp_ns();
        let row_version: i64 = 1;
        let status = LifecycleStatus::Active;
        let matrix_json = new.capability_matrix.to_json();

        let audit_hash = compute_role_audit_hash(
            &id,
            now_ns,
            row_version,
            None,
            &new.owner_id,
            &new.institutional_tag,
            &new.role_name,
            &matrix_json,
            status.as_str(),
        );

        sqlx::query(
            "INSERT INTO operator_roles (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                owner_id, institutional_tag, role_name, capability_matrix, status\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(Option::<String>::None)
        .bind(row_version)
        .bind(&new.owner_id)
        .bind(&new.institutional_tag)
        .bind(&new.role_name)
        .bind(&matrix_json)
        .bind(status.as_str())
        .execute(&mut *tx)
        .await?;

        let role = OperatorRoleRow {
            id: id.clone(),
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: None,
            row_version,
            owner_id: new.owner_id.clone(),
            institutional_tag: new.institutional_tag.clone(),
            role_name: new.role_name.clone(),
            capability_matrix: new.capability_matrix.clone(),
            status,
        };

        let event = insert_event_in_tx(
            &mut tx,
            self.clock,
            &new.owner_id,
            &new.institutional_tag,
            node_id,
            None,
            OperatorRoleChangeType::RoleCreated,
            &id,
            None,
        )
        .await?;

        tx.commit().await?;

        Ok((role, event))
    }

    /// Carga un rol por `id`, cualquier `status` -- o `None` si no existe.
    pub async fn get_role(&self, id: &str) -> Result<Option<OperatorRoleRow>, OperatorRoleError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    owner_id, institutional_tag, role_name, capability_matrix, status \
             FROM operator_roles WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        row.map(row_to_role).transpose()
    }

    /// Carga TODOS los roles de una cuenta (cualquier `status`), ordenados
    /// por `role_name` -- panel de roles y operadores.
    pub async fn load_roles(&self, owner_id: &str) -> Result<Vec<OperatorRoleRow>, OperatorRoleError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    owner_id, institutional_tag, role_name, capability_matrix, status \
             FROM operator_roles WHERE owner_id = ? ORDER BY role_name ASC",
        )
        .bind(owner_id)
        .fetch_all(self.pool)
        .await?;

        rows.into_iter().map(row_to_role).collect()
    }

    /// Reclasifica la matriz de capacidades de `current` -- pasa por el
    /// guardarraíl transaccional "último admin en pie" (§5 de la Orden):
    /// si el cambio dejaría la cuenta sin ningún admin, la transacción se
    /// descarta SIN escribir nada (ni la fila ni el evento).
    ///
    /// ## Concurrencia optimista (ADR-0141)
    ///
    /// El `UPDATE` filtra por `id` **y** `row_version = <el de `current`>`.
    /// Si otra escritura ya avanzó la fila, `rows_affected() == 0` y se
    /// devuelve [`OperatorRoleError::VersionConflict`].
    pub async fn update_role_matrix(
        &self,
        current: &OperatorRoleRow,
        new_matrix: CapabilityMatrix,
        node_id: &str,
        compliance_status_id: Option<&str>,
    ) -> Result<(OperatorRoleRow, OperatorRoleEventRow), OperatorRoleError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_update_role_matrix_once(current, &new_matrix, node_id, compliance_status_id).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if is_transient_write_conflict(&error) {
                        if attempt < MAX_GUARDED_ATTEMPTS {
                            continue;
                        }
                        return Err(OperatorRoleError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    async fn try_update_role_matrix_once(
        &self,
        current: &OperatorRoleRow,
        new_matrix: &CapabilityMatrix,
        node_id: &str,
        compliance_status_id: Option<&str>,
    ) -> Result<(OperatorRoleRow, OperatorRoleEventRow), OperatorRoleError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        // 1) Carga el estado admin-relevante VIGENTE dentro de la
        // transacción -- consistente porque BEGIN IMMEDIATE ya tomó el
        // lock de escritura de entrada.
        let roles = load_active_roles_tx(&mut tx, &current.owner_id).await?;
        let assignments = load_active_assignments_tx(&mut tx, &current.owner_id).await?;

        let change = ProposedChange::UpdateRoleMatrix { role_id: current.id.clone(), new_matrix: new_matrix.clone() };

        // 2) Valida el invariante -- si lo rechaza, `tx` se descarta sin
        // commit al salir de la función (rollback implícito).
        check_last_admin_standing_views(&roles, &assignments, &change)?;

        // 3) Escribe el cambio con concurrencia optimista.
        let now_ns = self.clock.timestamp_ns();
        let row_version = current.row_version + 1;
        let matrix_json = new_matrix.to_json();
        let status = current.status;

        let audit_hash = compute_role_audit_hash(
            &current.id,
            now_ns,
            row_version,
            Some(&current.audit_hash),
            &current.owner_id,
            &current.institutional_tag,
            &current.role_name,
            &matrix_json,
            status.as_str(),
        );

        let result = sqlx::query(
            "UPDATE operator_roles SET \
                updated_at = ?, audit_hash = ?, audit_chain_hash = ?, row_version = ?, capability_matrix = ? \
             WHERE id = ? AND row_version = ?",
        )
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&current.audit_hash)
        .bind(row_version)
        .bind(&matrix_json)
        .bind(&current.id)
        .bind(current.row_version)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(OperatorRoleError::VersionConflict { expected: current.row_version });
        }

        let updated_role = OperatorRoleRow {
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: Some(current.audit_hash.clone()),
            row_version,
            capability_matrix: new_matrix.clone(),
            ..current.clone()
        };

        // 4) Registra el evento en la MISMA transacción.
        let event = insert_event_in_tx(
            &mut tx,
            self.clock,
            &current.owner_id,
            &current.institutional_tag,
            node_id,
            compliance_status_id,
            OperatorRoleChangeType::RoleUpdated,
            &current.id,
            None,
        )
        .await?;

        tx.commit().await?;

        Ok((updated_role, event))
    }

    /// Revoca `current` -- baja lógica (`status = REVOKED`), NUNCA DELETE
    /// físico (ADR-0141). Pasa por el MISMO guardarraíl transaccional que
    /// [`Self::update_role_matrix`]: revocar el rol ADMIN cuando es el
    /// único con `CAPABILITY_MANAGE_ROLES` viola el invariante.
    pub async fn revoke_role(
        &self,
        current: &OperatorRoleRow,
        node_id: &str,
        compliance_status_id: Option<&str>,
    ) -> Result<(OperatorRoleRow, OperatorRoleEventRow), OperatorRoleError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_revoke_role_once(current, node_id, compliance_status_id).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if is_transient_write_conflict(&error) {
                        if attempt < MAX_GUARDED_ATTEMPTS {
                            continue;
                        }
                        return Err(OperatorRoleError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    async fn try_revoke_role_once(
        &self,
        current: &OperatorRoleRow,
        node_id: &str,
        compliance_status_id: Option<&str>,
    ) -> Result<(OperatorRoleRow, OperatorRoleEventRow), OperatorRoleError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        let roles = load_active_roles_tx(&mut tx, &current.owner_id).await?;
        let assignments = load_active_assignments_tx(&mut tx, &current.owner_id).await?;

        let change = ProposedChange::RevokeRole { role_id: current.id.clone() };
        check_last_admin_standing_views(&roles, &assignments, &change)?;

        let now_ns = self.clock.timestamp_ns();
        let row_version = current.row_version + 1;
        let matrix_json = current.capability_matrix.to_json();
        let status = LifecycleStatus::Revoked;

        let audit_hash = compute_role_audit_hash(
            &current.id,
            now_ns,
            row_version,
            Some(&current.audit_hash),
            &current.owner_id,
            &current.institutional_tag,
            &current.role_name,
            &matrix_json,
            status.as_str(),
        );

        let result = sqlx::query(
            "UPDATE operator_roles SET \
                updated_at = ?, audit_hash = ?, audit_chain_hash = ?, row_version = ?, status = ? \
             WHERE id = ? AND row_version = ?",
        )
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&current.audit_hash)
        .bind(row_version)
        .bind(status.as_str())
        .bind(&current.id)
        .bind(current.row_version)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(OperatorRoleError::VersionConflict { expected: current.row_version });
        }

        let revoked_role =
            OperatorRoleRow { updated_at_ns: now_ns, audit_hash, audit_chain_hash: Some(current.audit_hash.clone()), row_version, status, ..current.clone() };

        let event = insert_event_in_tx(
            &mut tx,
            self.clock,
            &current.owner_id,
            &current.institutional_tag,
            node_id,
            compliance_status_id,
            OperatorRoleChangeType::RoleRevoked,
            &current.id,
            None,
        )
        .await?;

        tx.commit().await?;

        Ok((revoked_role, event))
    }
}

// ── `operator_assignments` -- MUTABLE, UPSERT ───────────────────────────────

/// Entrada para [`OperatorAssignmentRepository::set_assignment`] -- alta o
/// cambio del rol vigente de un operador.
#[derive(Debug, Clone)]
pub struct SetAssignmentInput {
    pub owner_id: String,
    pub institutional_tag: String,
    pub access_token_id: String,
    pub operator_type: OperatorType,
    pub role_id: String,
}

/// Una fila de `operator_assignments` ya persistida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorAssignmentRow {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub row_version: i64,

    pub owner_id: String,
    pub institutional_tag: String,
    pub access_token_id: String,
    pub operator_type: OperatorType,
    pub role_id: String,
    pub status: LifecycleStatus,
}

/// Repositorio MUTABLE para `operator_assignments`.
pub struct OperatorAssignmentRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> OperatorAssignmentRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Alta o cambio del rol vigente de un operador -- UPSERT por
    /// `(owner_id, access_token_id)` (`UNIQUE` de la migración: "un
    /// operador tiene UN rol vigente por cuenta"). Pasa por el guardarraíl
    /// transaccional "último admin en pie": reasignar al último admin a un
    /// rol no-admin se rechaza ANTES de escribir.
    ///
    /// `change_type` deja que quien llama distinga una asignación normal
    /// (`ASSIGNMENT_SET`) de una cascada de autoridad del fondo
    /// (`AUTHORITY_OVERRIDE`, #12) -- misma mecánica, distinta etiqueta de
    /// auditoría.
    pub async fn set_assignment(
        &self,
        input: SetAssignmentInput,
        node_id: &str,
        compliance_status_id: Option<&str>,
        change_type: OperatorRoleChangeType,
    ) -> Result<(OperatorAssignmentRow, OperatorRoleEventRow), OperatorRoleError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_set_assignment_once(&input, node_id, compliance_status_id, change_type).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if is_transient_write_conflict(&error) {
                        if attempt < MAX_GUARDED_ATTEMPTS {
                            continue;
                        }
                        return Err(OperatorRoleError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    async fn try_set_assignment_once(
        &self,
        input: &SetAssignmentInput,
        node_id: &str,
        compliance_status_id: Option<&str>,
        change_type: OperatorRoleChangeType,
    ) -> Result<(OperatorAssignmentRow, OperatorRoleEventRow), OperatorRoleError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        let roles = load_active_roles_tx(&mut tx, &input.owner_id).await?;
        let assignments = load_active_assignments_tx(&mut tx, &input.owner_id).await?;

        // El rol destino debe existir y estar ACTIVO -- una asignación
        // nunca apunta a un rol inexistente o ya revocado.
        if !roles.iter().any(|role| role.id == input.role_id) {
            return Err(OperatorRoleError::RoleNotFound(input.role_id.clone()));
        }

        let change =
            ProposedChange::SetAssignment { access_token_id: input.access_token_id.clone(), role_id: input.role_id.clone() };
        check_last_admin_standing_views(&roles, &assignments, &change)?;

        let now_ns = self.clock.timestamp_ns();
        let status = LifecycleStatus::Active;

        // Busca si YA existe una fila (cualquier status) para este
        // operador en esta cuenta -- el UNIQUE de la migración exige que
        // reasignar sea un UPDATE de la MISMA fila, nunca un INSERT nuevo.
        let existing = find_assignment_row_tx(&mut tx, &input.owner_id, &input.access_token_id).await?;

        let assignment = match existing {
            Some(current) => {
                let row_version = current.row_version + 1;
                let audit_hash = compute_assignment_audit_hash(
                    &current.id,
                    now_ns,
                    row_version,
                    Some(&current.audit_hash),
                    &current.owner_id,
                    &current.institutional_tag,
                    &current.access_token_id,
                    input.operator_type.as_str(),
                    &input.role_id,
                    status.as_str(),
                );

                let result = sqlx::query(
                    "UPDATE operator_assignments SET \
                        updated_at = ?, audit_hash = ?, audit_chain_hash = ?, row_version = ?, \
                        operator_type = ?, role_id = ?, status = ? \
                     WHERE id = ? AND row_version = ?",
                )
                .bind(now_ns)
                .bind(&audit_hash)
                .bind(&current.audit_hash)
                .bind(row_version)
                .bind(input.operator_type.as_str())
                .bind(&input.role_id)
                .bind(status.as_str())
                .bind(&current.id)
                .bind(current.row_version)
                .execute(&mut *tx)
                .await?;

                if result.rows_affected() == 0 {
                    return Err(OperatorRoleError::VersionConflict { expected: current.row_version });
                }

                OperatorAssignmentRow {
                    updated_at_ns: now_ns,
                    audit_hash,
                    audit_chain_hash: Some(current.audit_hash.clone()),
                    row_version,
                    operator_type: input.operator_type,
                    role_id: input.role_id.clone(),
                    status,
                    ..current
                }
            }
            None => {
                let id = Uuid::now_v7().to_string();
                let row_version: i64 = 1;
                let audit_hash = compute_assignment_audit_hash(
                    &id,
                    now_ns,
                    row_version,
                    None,
                    &input.owner_id,
                    &input.institutional_tag,
                    &input.access_token_id,
                    input.operator_type.as_str(),
                    &input.role_id,
                    status.as_str(),
                );

                sqlx::query(
                    "INSERT INTO operator_assignments (\
                        id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                        owner_id, institutional_tag, access_token_id, operator_type, role_id, status\
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(&id)
                .bind(now_ns)
                .bind(now_ns)
                .bind(&audit_hash)
                .bind(Option::<String>::None)
                .bind(row_version)
                .bind(&input.owner_id)
                .bind(&input.institutional_tag)
                .bind(&input.access_token_id)
                .bind(input.operator_type.as_str())
                .bind(&input.role_id)
                .bind(status.as_str())
                .execute(&mut *tx)
                .await?;

                OperatorAssignmentRow {
                    id,
                    created_at_ns: now_ns,
                    updated_at_ns: now_ns,
                    audit_hash,
                    audit_chain_hash: None,
                    row_version,
                    owner_id: input.owner_id.clone(),
                    institutional_tag: input.institutional_tag.clone(),
                    access_token_id: input.access_token_id.clone(),
                    operator_type: input.operator_type,
                    role_id: input.role_id.clone(),
                    status,
                }
            }
        };

        let event = insert_event_in_tx(
            &mut tx,
            self.clock,
            &input.owner_id,
            &input.institutional_tag,
            node_id,
            compliance_status_id,
            change_type,
            &input.access_token_id,
            None,
        )
        .await?;

        tx.commit().await?;

        Ok((assignment, event))
    }

    /// Revoca la asignación vigente de `current` -- baja lógica (`status =
    /// REVOKED`); el operador queda SIN rol (ADR-0149: sin rol = denegado).
    /// Pasa por el MISMO guardarraíl transaccional que
    /// [`Self::set_assignment`].
    pub async fn revoke_assignment(
        &self,
        current: &OperatorAssignmentRow,
        node_id: &str,
        compliance_status_id: Option<&str>,
        change_type: OperatorRoleChangeType,
    ) -> Result<(OperatorAssignmentRow, OperatorRoleEventRow), OperatorRoleError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_revoke_assignment_once(current, node_id, compliance_status_id, change_type).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if is_transient_write_conflict(&error) {
                        if attempt < MAX_GUARDED_ATTEMPTS {
                            continue;
                        }
                        return Err(OperatorRoleError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    async fn try_revoke_assignment_once(
        &self,
        current: &OperatorAssignmentRow,
        node_id: &str,
        compliance_status_id: Option<&str>,
        change_type: OperatorRoleChangeType,
    ) -> Result<(OperatorAssignmentRow, OperatorRoleEventRow), OperatorRoleError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        let roles = load_active_roles_tx(&mut tx, &current.owner_id).await?;
        let assignments = load_active_assignments_tx(&mut tx, &current.owner_id).await?;

        let change = ProposedChange::RevokeAssignment { access_token_id: current.access_token_id.clone() };
        check_last_admin_standing_views(&roles, &assignments, &change)?;

        let now_ns = self.clock.timestamp_ns();
        let row_version = current.row_version + 1;
        let status = LifecycleStatus::Revoked;

        let audit_hash = compute_assignment_audit_hash(
            &current.id,
            now_ns,
            row_version,
            Some(&current.audit_hash),
            &current.owner_id,
            &current.institutional_tag,
            &current.access_token_id,
            current.operator_type.as_str(),
            &current.role_id,
            status.as_str(),
        );

        let result = sqlx::query(
            "UPDATE operator_assignments SET \
                updated_at = ?, audit_hash = ?, audit_chain_hash = ?, row_version = ?, status = ? \
             WHERE id = ? AND row_version = ?",
        )
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&current.audit_hash)
        .bind(row_version)
        .bind(status.as_str())
        .bind(&current.id)
        .bind(current.row_version)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(OperatorRoleError::VersionConflict { expected: current.row_version });
        }

        let revoked = OperatorAssignmentRow {
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: Some(current.audit_hash.clone()),
            row_version,
            status,
            ..current.clone()
        };

        let event = insert_event_in_tx(
            &mut tx,
            self.clock,
            &current.owner_id,
            &current.institutional_tag,
            node_id,
            compliance_status_id,
            change_type,
            &current.access_token_id,
            None,
        )
        .await?;

        tx.commit().await?;

        Ok((revoked, event))
    }

    /// Carga la asignación vigente (cualquier `status`) de un operador, o
    /// `None` si nunca se le asignó nada.
    pub async fn find_assignment(
        &self,
        owner_id: &str,
        access_token_id: &str,
    ) -> Result<Option<OperatorAssignmentRow>, OperatorRoleError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    owner_id, institutional_tag, access_token_id, operator_type, role_id, status \
             FROM operator_assignments WHERE owner_id = ? AND access_token_id = ?",
        )
        .bind(owner_id)
        .bind(access_token_id)
        .fetch_optional(self.pool)
        .await?;

        row.map(row_to_assignment).transpose()
    }

    /// Carga la asignación ACTIVA de un operador -- query path de
    /// `evaluate_call` (ADR-0149: sin asignación ACTIVA, el operador queda
    /// sin rol -> denegado).
    pub async fn find_active_assignment(
        &self,
        owner_id: &str,
        access_token_id: &str,
    ) -> Result<Option<OperatorAssignmentRow>, OperatorRoleError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    owner_id, institutional_tag, access_token_id, operator_type, role_id, status \
             FROM operator_assignments WHERE owner_id = ? AND access_token_id = ? AND status = 'ACTIVE'",
        )
        .bind(owner_id)
        .bind(access_token_id)
        .fetch_optional(self.pool)
        .await?;

        row.map(row_to_assignment).transpose()
    }

    /// Carga TODAS las asignaciones de una cuenta (cualquier `status`) --
    /// panel de roles y operadores.
    pub async fn load_assignments(&self, owner_id: &str) -> Result<Vec<OperatorAssignmentRow>, OperatorRoleError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    owner_id, institutional_tag, access_token_id, operator_type, role_id, status \
             FROM operator_assignments WHERE owner_id = ? ORDER BY access_token_id ASC",
        )
        .bind(owner_id)
        .fetch_all(self.pool)
        .await?;

        rows.into_iter().map(row_to_assignment).collect()
    }
}

// ── `operator_role_events` -- APPEND-ONLY ATÓMICA ───────────────────────────

/// Entrada para [`OperatorRoleEventRepository::record_event`] -- registrar
/// UN evento de forma AUTÓNOMA (fuera de una mutación guardada de rol/
/// asignación). Ver también [`insert_event_in_tx`], usada internamente por
/// las mutaciones guardadas para registrar su evento en su MISMA
/// transacción.
#[derive(Debug, Clone)]
pub struct RecordOperatorRoleEventInput {
    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,
    pub compliance_status_id: Option<String>,
    pub change_type: OperatorRoleChangeType,
    pub subject_ref: String,
    pub detail: Option<String>,
}

/// Una fila de `operator_role_events` ya persistida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorRoleEventRow {
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

    pub change_type: OperatorRoleChangeType,
    pub subject_ref: String,
    pub detail: Option<String>,
}

/// Repositorio APPEND-ONLY para `operator_role_events`. La única operación
/// de escritura pública es [`Self::record_event`] -- no hay `update`/
/// `delete`; los triggers `trg_operator_role_events_no_update`/`_no_delete`
/// de la migración los rechazarían de todas formas.
pub struct OperatorRoleEventRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> OperatorRoleEventRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Registra UN evento de forma autónoma: deriva su posición en la
    /// cadena GLOBAL, computa su `audit_hash` encadenado y lo persiste como
    /// fila nueva.
    ///
    /// ## Atomicidad bajo concurrencia (regla "Atomicidad de ledgers append-only")
    ///
    /// El *read-then-write* ocurre dentro de UNA sola transacción
    /// `BEGIN IMMEDIATE` (ver [`insert_event_in_tx`]). Ante contención
    /// transitoria se reintenta hasta [`MAX_GUARDED_ATTEMPTS`] veces; nunca
    /// se descarta el evento en silencio.
    pub async fn record_event(
        &self,
        input: RecordOperatorRoleEventInput,
    ) -> Result<OperatorRoleEventRow, OperatorRoleError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_record_once(&input).await {
                Ok(row) => return Ok(row),
                Err(error) => {
                    if is_transient_write_conflict(&error) {
                        if attempt < MAX_GUARDED_ATTEMPTS {
                            continue;
                        }
                        return Err(OperatorRoleError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    async fn try_record_once(
        &self,
        input: &RecordOperatorRoleEventInput,
    ) -> Result<OperatorRoleEventRow, OperatorRoleError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        let event = insert_event_in_tx(
            &mut tx,
            self.clock,
            &input.owner_id,
            &input.institutional_tag,
            &input.node_id,
            input.compliance_status_id.as_deref(),
            input.change_type,
            &input.subject_ref,
            input.detail.as_deref(),
        )
        .await?;

        tx.commit().await?;

        Ok(event)
    }

    /// Carga la cadena completa, ordenada por `event_sequence_id`
    /// ascendente -- usada por los tests de integridad de la cadena.
    pub async fn load_chain(&self) -> Result<Vec<OperatorRoleEventRow>, OperatorRoleError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, node_id, compliance_status_id, \
                    change_type, subject_ref, detail \
             FROM operator_role_events ORDER BY event_sequence_id ASC",
        )
        .fetch_all(self.pool)
        .await?;

        rows.into_iter().map(row_to_event).collect()
    }
}

// ── Helpers internos compartidos ────────────────────────────────────────────

/// Inserta un evento en `operator_role_events` DENTRO de la transacción
/// activa `tx` -- no abre su propia transacción; es el paso común que
/// reutilizan tanto [`OperatorRoleEventRepository::record_event`] (que abre
/// su PROPIA transacción alrededor) como las mutaciones guardadas de rol/
/// asignación (que insertan el evento como parte de una transacción más
/// grande que YA está validando el invariante "último admin en pie").
#[allow(clippy::too_many_arguments)]
async fn insert_event_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    clock: &dyn Clock,
    owner_id: &str,
    institutional_tag: &str,
    node_id: &str,
    compliance_status_id: Option<&str>,
    change_type: OperatorRoleChangeType,
    subject_ref: &str,
    detail: Option<&str>,
) -> Result<OperatorRoleEventRow, OperatorRoleError> {
    let tail_row = sqlx::query(
        "SELECT audit_hash, event_sequence_id FROM operator_role_events ORDER BY event_sequence_id DESC LIMIT 1",
    )
    .fetch_optional(&mut **tx)
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
    let now_ns = clock.timestamp_ns();
    let change_type_str = change_type.as_str();

    let audit_hash = compute_event_audit_hash(
        &id,
        now_ns,
        event_sequence_id,
        &previous_audit_hash,
        owner_id,
        institutional_tag,
        node_id,
        compliance_status_id,
        change_type_str,
        subject_ref,
        detail,
    );

    sqlx::query(
        "INSERT INTO operator_role_events (\
            id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
            owner_id, institutional_tag, node_id, compliance_status_id, \
            change_type, subject_ref, detail\
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(now_ns)
    .bind(now_ns)
    .bind(&audit_hash)
    .bind(&audit_chain_hash)
    .bind(event_sequence_id)
    .bind(owner_id)
    .bind(institutional_tag)
    .bind(node_id)
    .bind(compliance_status_id)
    .bind(change_type_str)
    .bind(subject_ref)
    .bind(detail)
    .execute(&mut **tx)
    .await?;

    Ok(OperatorRoleEventRow {
        id,
        created_at_ns: now_ns,
        updated_at_ns: now_ns,
        audit_hash,
        audit_chain_hash,
        event_sequence_id,
        owner_id: owner_id.to_string(),
        institutional_tag: institutional_tag.to_string(),
        node_id: node_id.to_string(),
        compliance_status_id: compliance_status_id.map(|s| s.to_string()),
        change_type,
        subject_ref: subject_ref.to_string(),
        detail: detail.map(|s| s.to_string()),
    })
}

/// Carga los roles ACTIVOS de una cuenta DENTRO de la transacción activa --
/// paso 1 del guardarraíl "último admin en pie".
async fn load_active_roles_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    owner_id: &str,
) -> Result<Vec<OperatorRoleRow>, OperatorRoleError> {
    let rows = sqlx::query(
        "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                owner_id, institutional_tag, role_name, capability_matrix, status \
         FROM operator_roles WHERE owner_id = ? AND status = 'ACTIVE'",
    )
    .bind(owner_id)
    .fetch_all(&mut **tx)
    .await?;

    rows.into_iter().map(row_to_role).collect()
}

/// Carga las asignaciones ACTIVAS de una cuenta DENTRO de la transacción
/// activa -- paso 1 del guardarraíl "último admin en pie".
async fn load_active_assignments_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    owner_id: &str,
) -> Result<Vec<OperatorAssignmentRow>, OperatorRoleError> {
    let rows = sqlx::query(
        "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                owner_id, institutional_tag, access_token_id, operator_type, role_id, status \
         FROM operator_assignments WHERE owner_id = ? AND status = 'ACTIVE'",
    )
    .bind(owner_id)
    .fetch_all(&mut **tx)
    .await?;

    rows.into_iter().map(row_to_assignment).collect()
}

/// Busca la fila de `operator_assignments` (cualquier `status`) para un
/// operador DENTRO de la transacción activa -- usado por `set_assignment`
/// para decidir INSERT vs. UPDATE.
async fn find_assignment_row_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    owner_id: &str,
    access_token_id: &str,
) -> Result<Option<OperatorAssignmentRow>, OperatorRoleError> {
    let row = sqlx::query(
        "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                owner_id, institutional_tag, access_token_id, operator_type, role_id, status \
         FROM operator_assignments WHERE owner_id = ? AND access_token_id = ?",
    )
    .bind(owner_id)
    .bind(access_token_id)
    .fetch_optional(&mut **tx)
    .await?;

    row.map(row_to_assignment).transpose()
}

/// Proyecta filas ya cargadas a las vistas mínimas que consume
/// `check_last_admin_standing` del Core, y traduce
/// [`LastAdminViolation`] al error de este módulo -- evita repetir la
/// proyección en cada mutación guardada.
fn check_last_admin_standing_views(
    roles: &[OperatorRoleRow],
    assignments: &[OperatorAssignmentRow],
    change: &ProposedChange,
) -> Result<(), OperatorRoleError> {
    let role_views: Vec<RoleView> =
        roles.iter().map(|r| RoleView { role_id: r.id.clone(), matrix: r.capability_matrix.clone() }).collect();
    let assignment_views: Vec<AssignmentView> = assignments
        .iter()
        .map(|a| AssignmentView { access_token_id: a.access_token_id.clone(), role_id: a.role_id.clone() })
        .collect();

    crate::domain::operator_roles::check_last_admin_standing(&role_views, &assignment_views, change)?;
    Ok(())
}

/// Convierte una fila de `operator_roles` al tipo [`OperatorRoleRow`].
fn row_to_role(row: sqlx::sqlite::SqliteRow) -> Result<OperatorRoleRow, OperatorRoleError> {
    let id: String = row.get("id");

    let status_value: String = row.get("status");
    let status = LifecycleStatus::from_str_value(&status_value).ok_or(OperatorRoleError::UnknownStatus(status_value))?;

    let matrix_json: String = row.get("capability_matrix");
    let capability_matrix =
        CapabilityMatrix::from_json(&matrix_json).ok_or_else(|| OperatorRoleError::MalformedCapabilityMatrix(id.clone()))?;

    Ok(OperatorRoleRow {
        id,
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        row_version: row.get("row_version"),
        owner_id: row.get("owner_id"),
        institutional_tag: row.get("institutional_tag"),
        role_name: row.get("role_name"),
        capability_matrix,
        status,
    })
}

/// Convierte una fila de `operator_assignments` al tipo
/// [`OperatorAssignmentRow`], decodificando los enums de texto persistidos.
fn row_to_assignment(row: sqlx::sqlite::SqliteRow) -> Result<OperatorAssignmentRow, OperatorRoleError> {
    let operator_type_value: String = row.get("operator_type");
    let operator_type =
        OperatorType::from_str_value(&operator_type_value).ok_or(OperatorRoleError::UnknownOperatorType(operator_type_value))?;

    let status_value: String = row.get("status");
    let status = LifecycleStatus::from_str_value(&status_value).ok_or(OperatorRoleError::UnknownStatus(status_value))?;

    Ok(OperatorAssignmentRow {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        row_version: row.get("row_version"),
        owner_id: row.get("owner_id"),
        institutional_tag: row.get("institutional_tag"),
        access_token_id: row.get("access_token_id"),
        operator_type,
        role_id: row.get("role_id"),
        status,
    })
}

/// Convierte una fila de `operator_role_events` al tipo
/// [`OperatorRoleEventRow`], decodificando el `change_type` persistido.
fn row_to_event(row: sqlx::sqlite::SqliteRow) -> Result<OperatorRoleEventRow, OperatorRoleError> {
    let change_type_value: String = row.get("change_type");
    let change_type = OperatorRoleChangeType::from_str_value(&change_type_value)
        .ok_or(OperatorRoleError::UnknownChangeType(change_type_value))?;

    Ok(OperatorRoleEventRow {
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
        change_type,
        subject_ref: row.get("subject_ref"),
        detail: row.get("detail"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::domain::operator_roles::CAPABILITY_MANAGE_ROLES;
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    fn admin_matrix() -> CapabilityMatrix {
        let mut m = CapabilityMatrix::new();
        m.set(CAPABILITY_MANAGE_ROLES, true);
        m
    }

    fn analyst_matrix() -> CapabilityMatrix {
        let mut m = CapabilityMatrix::new();
        m.set("generate.run_search", true);
        m
    }

    // ── CRITERIO: esquema STRICT + Grupo I + row_version / event_sequence_id ──

    #[tokio::test]
    async fn migration_creates_all_three_tables_strict_with_expected_columns() {
        let pool = migrated_pool().await;

        for (table, expected_extra, must_not_have) in [
            (
                "operator_roles",
                vec!["row_version", "owner_id", "institutional_tag", "role_name", "capability_matrix", "status"],
                "event_sequence_id",
            ),
            (
                "operator_assignments",
                vec!["row_version", "owner_id", "institutional_tag", "access_token_id", "operator_type", "role_id", "status"],
                "event_sequence_id",
            ),
            (
                "operator_role_events",
                vec!["event_sequence_id", "owner_id", "institutional_tag", "node_id", "change_type", "subject_ref"],
                "row_version",
            ),
        ] {
            let columns = sqlx::query(&format!("SELECT name FROM pragma_table_info('{table}')"))
                .fetch_all(&pool)
                .await
                .unwrap_or_else(|e| panic!("leer table_info de {table}: {e}"));
            let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();

            for expected in ["id", "created_at", "updated_at", "audit_hash", "audit_chain_hash"]
                .into_iter()
                .chain(expected_extra)
            {
                assert!(column_names.contains(&expected.to_string()), "{table}: falta la columna {expected}");
            }
            assert!(!column_names.contains(&must_not_have.to_string()), "{table}: no debe tener {must_not_have}");

            let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = ?")
                .bind(table)
                .fetch_one(&pool)
                .await
                .expect("leer sqlite_master")
                .get(0);
            assert!(sql.contains("STRICT"), "{table} debe declararse STRICT");
        }
    }

    // ── CRITERIO (Orden §8): matriz denegada-por-defecto persistida ──────────

    #[tokio::test]
    async fn create_role_persists_matrix_and_denies_absent_capability_by_default() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = OperatorRoleRepository::new(&pool, &clock);

        let (role, event) = repo
            .create_role(
                NewOperatorRole {
                    owner_id: "acc-1".to_string(),
                    institutional_tag: "LIVE".to_string(),
                    role_name: "Analyst".to_string(),
                    capability_matrix: analyst_matrix(),
                },
                "node-A",
            )
            .await
            .expect("crear rol");

        assert_eq!(role.row_version, 1);
        assert_eq!(role.status, LifecycleStatus::Active);
        assert!(!role.capability_matrix.allows(CAPABILITY_MANAGE_ROLES), "Analyst no debe traer MANAGE_ROLES");
        assert_eq!(event.change_type, OperatorRoleChangeType::RoleCreated);
        assert_eq!(event.subject_ref, role.id);
        assert_eq!(event.event_sequence_id, 1);

        let reloaded = repo.get_role(&role.id).await.expect("releer").expect("debe existir");
        assert_eq!(reloaded, role);
    }

    #[tokio::test]
    async fn create_role_rejects_duplicate_role_name_within_the_same_account() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = OperatorRoleRepository::new(&pool, &clock);

        repo.create_role(
            NewOperatorRole { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), role_name: "Analyst".to_string(), capability_matrix: analyst_matrix() },
            "node-A",
        )
        .await
        .expect("primer rol");

        let dup = repo
            .create_role(
                NewOperatorRole { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), role_name: "Analyst".to_string(), capability_matrix: admin_matrix() },
                "node-A",
            )
            .await;
        assert!(dup.is_err(), "UNIQUE(owner_id, role_name) debe rechazar el nombre duplicado");
    }

    // ── CRITERIO (Orden §8): "último admin en pie" a través de la Shell real ──

    /// Siembra un ÚNICO admin (rol + asignación) y devuelve
    /// (repo_roles, repo_assignments, admin_role, admin_assignment).
    async fn seed_single_admin<'a>(
        pool: &'a SqlitePool,
        clock: &'a DeterministicClock,
    ) -> (OperatorRoleRepository<'a>, OperatorAssignmentRepository<'a>, OperatorRoleRow, OperatorAssignmentRow) {
        let role_repo = OperatorRoleRepository::new(pool, clock);
        let assignment_repo = OperatorAssignmentRepository::new(pool, clock);

        let (admin_role, _) = role_repo
            .create_role(
                NewOperatorRole { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), role_name: "Admin".to_string(), capability_matrix: admin_matrix() },
                "node-A",
            )
            .await
            .expect("crear rol admin");

        let (admin_assignment, _) = assignment_repo
            .set_assignment(
                SetAssignmentInput {
                    owner_id: "acc-1".to_string(),
                    institutional_tag: "LIVE".to_string(),
                    access_token_id: "tok-owner".to_string(),
                    operator_type: OperatorType::Human,
                    role_id: admin_role.id.clone(),
                },
                "node-A",
                None,
                OperatorRoleChangeType::AssignmentSet,
            )
            .await
            .expect("asignar admin");

        (role_repo, assignment_repo, admin_role, admin_assignment)
    }

    /// CRITERIO DE CIERRE: revocar la asignación del ÚNICO admin se
    /// rechaza -- y NADA se escribió (ni la fila ni el evento).
    #[tokio::test]
    async fn revoking_the_only_admin_assignment_is_rejected_and_writes_nothing() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let (_, assignment_repo, _, admin_assignment) = seed_single_admin(&pool, &clock).await;

        let event_repo = OperatorRoleEventRepository::new(&pool, &clock);
        let events_before = event_repo.load_chain().await.expect("cargar cadena").len();

        let result =
            assignment_repo.revoke_assignment(&admin_assignment, "node-A", None, OperatorRoleChangeType::AssignmentRevoked).await;
        assert!(matches!(result, Err(OperatorRoleError::LastAdmin(_))), "resultado fue: {result:?}");

        let reloaded = assignment_repo
            .find_assignment("acc-1", "tok-owner")
            .await
            .expect("releer")
            .expect("debe seguir existiendo");
        assert_eq!(reloaded.status, LifecycleStatus::Active, "la asignación NO debe haberse revocado");

        let events_after = event_repo.load_chain().await.expect("cargar cadena").len();
        assert_eq!(events_before, events_after, "el rechazo no debe dejar NINGÚN evento nuevo en el ledger");
    }

    /// CRITERIO DE CIERRE: con >=2 admins, reasignar a UNO es aceptado --
    /// el otro sigue de pie.
    #[tokio::test]
    async fn reassigning_one_of_two_admins_succeeds() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let (role_repo, assignment_repo, admin_role, _) = seed_single_admin(&pool, &clock).await;

        clock.tick();
        let (analyst_role, _) = role_repo
            .create_role(
                NewOperatorRole { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), role_name: "Analyst".to_string(), capability_matrix: analyst_matrix() },
                "node-A",
            )
            .await
            .expect("crear rol analyst");

        clock.tick();
        assignment_repo
            .set_assignment(
                SetAssignmentInput {
                    owner_id: "acc-1".to_string(),
                    institutional_tag: "LIVE".to_string(),
                    access_token_id: "tok-second-admin".to_string(),
                    operator_type: OperatorType::Human,
                    role_id: admin_role.id.clone(),
                },
                "node-A",
                None,
                OperatorRoleChangeType::AssignmentSet,
            )
            .await
            .expect("segundo admin");

        clock.tick();
        let reassigned = assignment_repo
            .set_assignment(
                SetAssignmentInput {
                    owner_id: "acc-1".to_string(),
                    institutional_tag: "LIVE".to_string(),
                    access_token_id: "tok-owner".to_string(),
                    operator_type: OperatorType::Human,
                    role_id: analyst_role.id.clone(),
                },
                "node-A",
                None,
                OperatorRoleChangeType::AssignmentSet,
            )
            .await;
        assert!(reassigned.is_ok(), "con dos admins, reasignar uno debe aceptarse: {reassigned:?}");
    }

    /// CRITERIO DE CIERRE: editar la matriz del rol ADMIN quitándole
    /// `MANAGE_ROLES` cuando es el único con esa capacidad -> rechazado.
    #[tokio::test]
    async fn stripping_manage_roles_from_the_only_admin_role_is_rejected() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let (role_repo, _, admin_role, _) = seed_single_admin(&pool, &clock).await;

        let mut stripped = admin_matrix();
        stripped.set(CAPABILITY_MANAGE_ROLES, false);

        clock.tick();
        let result = role_repo.update_role_matrix(&admin_role, stripped, "node-A", None).await;
        assert!(matches!(result, Err(OperatorRoleError::LastAdmin(_))), "resultado fue: {result:?}");
    }

    /// CRITERIO DE CIERRE: revocar el rol ADMIN completo cuando es el único
    /// con la capacidad -> rechazado (cuarta vía del invariante).
    #[tokio::test]
    async fn revoking_the_only_admin_role_is_rejected() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let (role_repo, _, admin_role, _) = seed_single_admin(&pool, &clock).await;

        clock.tick();
        let result = role_repo.revoke_role(&admin_role, "node-A", None).await;
        assert!(matches!(result, Err(OperatorRoleError::LastAdmin(_))), "resultado fue: {result:?}");
    }

    // ── CRITERIO (Orden §8): rol revocado = baja lógica, nunca DELETE ────────

    #[tokio::test]
    async fn revoke_role_sets_status_revoked_never_deletes_the_row() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let role_repo = OperatorRoleRepository::new(&pool, &clock);
        let assignment_repo = OperatorAssignmentRepository::new(&pool, &clock);

        // Dos admins para que revocar uno de los roles no viole el invariante.
        let (admin_role, _) = role_repo
            .create_role(
                NewOperatorRole { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), role_name: "Admin".to_string(), capability_matrix: admin_matrix() },
                "node-A",
            )
            .await
            .expect("crear admin");
        clock.tick();
        let (spare_role, _) = role_repo
            .create_role(
                NewOperatorRole { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), role_name: "SpareAdmin".to_string(), capability_matrix: admin_matrix() },
                "node-A",
            )
            .await
            .expect("crear spare admin");
        clock.tick();
        assignment_repo
            .set_assignment(
                SetAssignmentInput { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), access_token_id: "tok-1".to_string(), operator_type: OperatorType::Human, role_id: admin_role.id.clone() },
                "node-A", None, OperatorRoleChangeType::AssignmentSet,
            )
            .await
            .expect("asignar admin");
        clock.tick();
        assignment_repo
            .set_assignment(
                SetAssignmentInput { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), access_token_id: "tok-2".to_string(), operator_type: OperatorType::Human, role_id: spare_role.id.clone() },
                "node-A", None, OperatorRoleChangeType::AssignmentSet,
            )
            .await
            .expect("asignar spare admin");

        clock.tick();
        let (revoked, _) = role_repo.revoke_role(&spare_role, "node-A", None).await.expect("revocar spare admin");
        assert_eq!(revoked.status, LifecycleStatus::Revoked);

        // La fila SIGUE ahí -- ninguna cuenta de filas cambió.
        let count: i64 = sqlx::query("SELECT COUNT(*) FROM operator_roles WHERE id = ?")
            .bind(&spare_role.id)
            .fetch_one(&pool)
            .await
            .expect("contar")
            .get(0);
        assert_eq!(count, 1, "revocar un rol nunca debe borrar la fila -- baja lógica, no DELETE físico");
    }

    /// Un intento de DELETE físico sobre un rol con una asignación VIVA es
    /// rechazado por el `FOREIGN KEY ... ON DELETE RESTRICT` -- protección
    /// cinturón-y-tirantes contra un DELETE accidental que se saltara la
    /// capa Rust.
    #[tokio::test]
    async fn foreign_key_restrict_blocks_physical_delete_of_a_role_with_live_assignment() {
        let pool = migrated_pool().await;
        sqlx::query("PRAGMA foreign_keys = ON").execute(&pool).await.expect("activar FKs");
        let clock = DeterministicClock::new(1_000, 100);
        let (_, _, admin_role, _) = seed_single_admin(&pool, &clock).await;

        let result = sqlx::query("DELETE FROM operator_roles WHERE id = ?").bind(&admin_role.id).execute(&pool).await;
        assert!(result.is_err(), "DELETE físico de un rol con asignación viva debe ser rechazado por la FK RESTRICT");
    }

    // ── CRITERIO (Orden §8): gate compuesto real vía asignación ──────────────

    #[tokio::test]
    async fn operator_without_assignment_has_no_active_role() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let assignment_repo = OperatorAssignmentRepository::new(&pool, &clock);

        let missing = assignment_repo.find_active_assignment("acc-1", "tok-nunca-asignado").await.expect("consultar");
        assert_eq!(missing, None, "un operador sin asignación no debe resolver ningún rol");
    }

    // ── data_portability-style: triggers append-only + CHECK ────────────────

    #[tokio::test]
    async fn update_is_rejected_by_trigger_on_operator_role_events() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let event_repo = OperatorRoleEventRepository::new(&pool, &clock);

        let row = event_repo
            .record_event(RecordOperatorRoleEventInput {
                owner_id: "acc-1".to_string(),
                institutional_tag: "LIVE".to_string(),
                node_id: "node-A".to_string(),
                compliance_status_id: None,
                change_type: OperatorRoleChangeType::RoleCreated,
                subject_ref: "role-1".to_string(),
                detail: None,
            })
            .await
            .expect("registrar evento");

        let result = sqlx::query("UPDATE operator_role_events SET subject_ref = 'x' WHERE id = ?")
            .bind(&row.id)
            .execute(&pool)
            .await;
        assert!(result.is_err(), "UPDATE sobre operator_role_events debe ser rechazado por el trigger");
    }

    #[tokio::test]
    async fn delete_is_rejected_by_trigger_on_operator_role_events() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let event_repo = OperatorRoleEventRepository::new(&pool, &clock);

        let row = event_repo
            .record_event(RecordOperatorRoleEventInput {
                owner_id: "acc-1".to_string(),
                institutional_tag: "LIVE".to_string(),
                node_id: "node-A".to_string(),
                compliance_status_id: None,
                change_type: OperatorRoleChangeType::RoleCreated,
                subject_ref: "role-1".to_string(),
                detail: None,
            })
            .await
            .expect("registrar evento");

        let result = sqlx::query("DELETE FROM operator_role_events WHERE id = ?").bind(&row.id).execute(&pool).await;
        assert!(result.is_err(), "DELETE sobre operator_role_events debe ser rechazado por el trigger");
    }

    #[tokio::test]
    async fn database_check_rejects_unknown_change_type() {
        let pool = migrated_pool().await;
        let result = sqlx::query(
            "INSERT INTO operator_role_events (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, compliance_status_id, \
                change_type, subject_ref, detail\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', 'LIVE', 'node-A', NULL, \
                       'UNKNOWN_CHANGE', 'role-1', NULL)",
        )
        .execute(&pool)
        .await;
        assert!(result.is_err(), "un change_type fuera del catálogo debe ser rechazado por el CHECK");
    }

    #[tokio::test]
    async fn database_check_rejects_unknown_operator_type() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let role_repo = OperatorRoleRepository::new(&pool, &clock);
        let (role, _) = role_repo
            .create_role(
                NewOperatorRole { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), role_name: "Analyst".to_string(), capability_matrix: analyst_matrix() },
                "node-A",
            )
            .await
            .expect("crear rol");

        let result = sqlx::query(
            "INSERT INTO operator_assignments (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                owner_id, institutional_tag, access_token_id, operator_type, role_id, status\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'acc-1', 'LIVE', 'tok-1', 'ROBOT', ?, 'ACTIVE')",
        )
        .bind(&role.id)
        .execute(&pool)
        .await;
        assert!(result.is_err(), "un operator_type fuera de HUMAN/AGENT debe ser rechazado por el CHECK");
    }

    // ── CRITERIO (Orden §8): 16 escritores concurrentes sobre el ledger ──────

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_record_events_persist_every_row_without_gaps_or_lost_rows() {
        use std::sync::Arc;

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("operator_role_events_concurrency.sqlite");
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
                let repo = OperatorRoleEventRepository::new(&pool_c, clock_c.as_ref());
                repo.record_event(RecordOperatorRoleEventInput {
                    owner_id: format!("owner-{i}"),
                    institutional_tag: "LIVE".to_string(),
                    node_id: format!("node-{i}"),
                    compliance_status_id: None,
                    change_type: OperatorRoleChangeType::RoleCreated,
                    subject_ref: format!("role-{i}"),
                    detail: None,
                })
                .await
            }));
        }

        for handle in handles {
            handle.await.expect("la tarea no debe entrar en panic").expect("record_event debe tener éxito");
        }

        let repo = OperatorRoleEventRepository::new(&pool, clock.as_ref());
        let chain = repo.load_chain().await.expect("cargar la cadena completa");

        assert_eq!(chain.len() as i64, N, "deben persistirse las N filas, sin ninguna pérdida");
        let sequence_ids: Vec<i64> = chain.iter().map(|row| row.event_sequence_id).collect();
        let expected: Vec<i64> = (1..=N).collect();
        assert_eq!(sequence_ids, expected, "los event_sequence_id deben ser 1..=N sin huecos ni duplicados");
    }

    // ── CRITERIO (QA por mutación): reintento acotado hasta AGOTAR ────────────

    /// Bajo contención de escritura SOSTENIDA, el bucle de reintento debe
    /// agotar EXACTAMENTE `MAX_GUARDED_ATTEMPTS` intentos y rendirse con
    /// `WriteContention { attempts: MAX }` -- mismo patrón que
    /// `data_portability::record_event_exhausts_exactly_max_attempts_when_write_lock_is_held`.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn record_event_exhausts_exactly_max_attempts_when_write_lock_is_held() {
        use std::str::FromStr;
        use std::time::Duration;

        use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("or_forced_contention.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());

        let pool = connect(&database_url).await.expect("conectar");
        migrate(&pool).await.expect("migrar");

        let immediate_opts = || {
            SqliteConnectOptions::from_str(&database_url)
                .expect("parsear opciones")
                .journal_mode(SqliteJournalMode::Wal)
                .busy_timeout(Duration::from_millis(0))
        };

        let lock_pool = SqlitePoolOptions::new().max_connections(1).connect_with(immediate_opts()).await.expect("pool que retiene el lock");
        let lock_tx = lock_pool.begin_with("BEGIN IMMEDIATE").await.expect("tomar el lock de escritura reservado");

        let repo_pool = SqlitePoolOptions::new().max_connections(1).connect_with(immediate_opts()).await.expect("pool del repositorio");
        let clock = DeterministicClock::new(1_000, 100);
        let repo = OperatorRoleEventRepository::new(&repo_pool, &clock);

        let result = repo
            .record_event(RecordOperatorRoleEventInput {
                owner_id: "acc-contention".to_string(),
                institutional_tag: "LIVE".to_string(),
                node_id: "node-A".to_string(),
                compliance_status_id: None,
                change_type: OperatorRoleChangeType::RoleCreated,
                subject_ref: "role-contention".to_string(),
                detail: None,
            })
            .await;

        drop(lock_tx);

        match result {
            Err(OperatorRoleError::WriteContention { attempts }) => {
                assert_eq!(attempts, MAX_GUARDED_ATTEMPTS, "bajo contención sostenida debe agotar EXACTAMENTE MAX_GUARDED_ATTEMPTS intentos");
            }
            other => panic!("se esperaba WriteContention {{ attempts: {MAX_GUARDED_ATTEMPTS} }}, se obtuvo: {other:?}"),
        }
    }

    // ── CRITERIO (QA por mutación): clasificador de contención ────────────────

    /// `is_transient_write_conflict` distingue una violación UNIQUE
    /// PERMANENTE (la PK `id`, NO `event_sequence_id`) de la contención
    /// transitoria -- mismo patrón que `data_portability`.
    #[tokio::test]
    async fn is_transient_is_false_for_a_permanent_non_sequence_unique_violation() {
        let pool = migrated_pool().await;

        let insert_with_id_dup = |event_sequence_id: i64| {
            sqlx::query(
                "INSERT INTO operator_role_events \
                 (id, created_at, updated_at, audit_hash, event_sequence_id, owner_id, \
                  institutional_tag, node_id, change_type, subject_ref) \
                 VALUES ('dup-id', 1, 1, 'h', ?, 'o', 'LIVE', 'n', 'ROLE_CREATED', 'role-1')",
            )
            .bind(event_sequence_id)
            .execute(&pool)
        };
        insert_with_id_dup(1).await.expect("primera fila válida");
        let err = insert_with_id_dup(2).await.expect_err("la segunda fila debe violar la PRIMARY KEY id");

        let permanent = OperatorRoleError::Database(err);
        assert!(
            !is_transient_write_conflict(&permanent),
            "una violación UNIQUE de la PK (no de event_sequence_id) es PERMANENTE, no transitoria"
        );

        let non_database = OperatorRoleError::UnknownStatus("X".to_string());
        assert!(!is_transient_write_conflict(&non_database), "un error no-Database nunca es contención transitoria");
    }

    // ── CRITERIO (QA por mutación): fidelidad de la fila devuelta ─────────────

    /// La fila que DEVUELVE `update_role_matrix` refleja los valores NUEVOS
    /// (audit_hash recomputado, audit_chain_hash encadenado, updated_at
    /// avanzado), no los viejos copiados de `..current.clone()`.
    #[tokio::test]
    async fn update_role_matrix_returned_row_reflects_new_hash_chain_and_timestamp() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let (role_repo, _, admin_role, _) = seed_single_admin(&pool, &clock).await;

        assert_eq!(admin_role.updated_at_ns, 1_000, "precondición: la fila génesis nace en el now inicial");
        assert!(admin_role.audit_chain_hash.is_none(), "precondición: la fila génesis no encadena");

        clock.tick(); // 1_000 -> 1_100
        let mut expanded = admin_matrix();
        expanded.set("generate.run_search", true);
        let (updated, _) = role_repo.update_role_matrix(&admin_role, expanded, "node-A", None).await.expect("actualizar matriz");

        assert_ne!(updated.audit_hash, admin_role.audit_hash, "debe devolver el audit_hash recomputado, no el viejo");
        assert_eq!(
            updated.audit_chain_hash,
            Some(admin_role.audit_hash.clone()),
            "el audit_chain_hash devuelto debe encadenar al audit_hash de la versión previa"
        );
        assert_eq!(updated.updated_at_ns, 1_100, "el updated_at devuelto debe ser el now del reloj tras el tick");
        assert!(updated.capability_matrix.allows("generate.run_search"), "debe reflejar la matriz NUEVA");
        // El `row_version` devuelto debe ser EXACTAMENTE el anterior + 1
        // (mata `+ 1` -> `- 1`/`* 1` en la aritmética de la versión).
        assert_eq!(updated.row_version, admin_role.row_version + 1, "row_version devuelto debe ser el anterior + 1");
        assert_eq!(updated.status, LifecycleStatus::Active, "editar la matriz no cambia el status");
    }

    /// La fila que DEVUELVE `set_assignment` (camino UPDATE, reasignación)
    /// refleja los valores NUEVOS -- mismo criterio que
    /// `update_role_matrix_returned_row_reflects_new_hash_chain_and_timestamp`.
    #[tokio::test]
    async fn set_assignment_returned_row_reflects_new_hash_chain_and_timestamp_on_reassignment() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let (role_repo, assignment_repo, admin_role, first_assignment) = seed_single_admin(&pool, &clock).await;

        clock.tick();
        let (spare_role, _) = role_repo
            .create_role(
                NewOperatorRole { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), role_name: "SpareAdmin".to_string(), capability_matrix: admin_matrix() },
                "node-A",
            )
            .await
            .expect("crear spare admin");
        clock.tick();
        assignment_repo
            .set_assignment(
                SetAssignmentInput { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), access_token_id: "tok-2".to_string(), operator_type: OperatorType::Human, role_id: spare_role.id.clone() },
                "node-A", None, OperatorRoleChangeType::AssignmentSet,
            )
            .await
            .expect("segundo admin para no violar el invariante al reasignar");

        clock.tick();
        let mut analyst = analyst_matrix();
        analyst.set("execute.send_order", false);
        let (analyst_role, _) = role_repo
            .create_role(
                NewOperatorRole { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), role_name: "Analyst".to_string(), capability_matrix: analyst },
                "node-A",
            )
            .await
            .expect("crear rol analyst");

        clock.tick();
        let (reassigned, _) = assignment_repo
            .set_assignment(
                // operator_type Human -> Agent: DISTINTO del valor previo, para
                // que el campo devuelto discrimine (si se borrara de la
                // proyección, caería al Human previo vía `..current`).
                SetAssignmentInput { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), access_token_id: "tok-owner".to_string(), operator_type: OperatorType::Agent, role_id: analyst_role.id.clone() },
                "node-A", None, OperatorRoleChangeType::AssignmentSet,
            )
            .await
            .expect("reasignar");

        assert_eq!(reassigned.id, first_assignment.id, "reasignar debe UPDATE la MISMA fila, no crear una nueva");
        assert_ne!(reassigned.audit_hash, first_assignment.audit_hash, "debe devolver el audit_hash recomputado");
        assert_eq!(reassigned.audit_chain_hash, Some(first_assignment.audit_hash.clone()));
        assert_eq!(reassigned.role_id, analyst_role.id, "debe reflejar el rol NUEVO");
        // Fidelidad completa de la fila devuelta por el camino UPDATE:
        // row_version = anterior + 1, updated_at = now, operator_type y
        // status reflejados (matan el borrado de cada campo de la proyección).
        assert_eq!(reassigned.row_version, first_assignment.row_version + 1, "row_version devuelto debe ser el anterior + 1");
        assert_eq!(reassigned.updated_at_ns, 1_400, "updated_at devuelto debe ser el now del reloj tras los cuatro ticks");
        assert_eq!(reassigned.operator_type, OperatorType::Agent, "operator_type devuelto debe reflejar el input NUEVO (Agent), no el Human previo");
        assert_eq!(reassigned.status, LifecycleStatus::Active, "una reasignación deja la fila ACTIVE");
        assert_ne!(admin_role.id, analyst_role.id);
    }

    /// CRITERIO (QA por mutación): re-asignar un operador cuya asignación
    /// estaba REVOKED lo REACTIVA -- la fila DEVUELTA trae `status = Active`,
    /// no el `Revoked` de la versión previa. Mata el borrado del campo
    /// `status` de la proyección de `try_set_assignment_once` en el camino
    /// UPDATE (con el campo borrado, caería al `Revoked` previo vía
    /// `..current`).
    #[tokio::test]
    async fn set_assignment_reactivates_a_revoked_operator_returning_active_status() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let (role_repo, assignment_repo, _admin_role, _) = seed_single_admin(&pool, &clock).await;

        // Operador NO admin: revocarlo y reactivarlo no toca el invariante
        // "último admin en pie" (el admin `tok-owner` permanece).
        clock.tick();
        let (analyst_role, _) = role_repo
            .create_role(
                NewOperatorRole { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), role_name: "Analyst".to_string(), capability_matrix: analyst_matrix() },
                "node-A",
            )
            .await
            .expect("crear rol analyst");

        let analyst_input = || SetAssignmentInput {
            owner_id: "acc-1".to_string(),
            institutional_tag: "LIVE".to_string(),
            access_token_id: "tok-analyst".to_string(),
            operator_type: OperatorType::Human,
            role_id: analyst_role.id.clone(),
        };

        clock.tick();
        let (active, _) = assignment_repo
            .set_assignment(analyst_input(), "node-A", None, OperatorRoleChangeType::AssignmentSet)
            .await
            .expect("asignar analyst");
        assert_eq!(active.status, LifecycleStatus::Active);

        clock.tick();
        let (revoked, _) = assignment_repo
            .revoke_assignment(&active, "node-A", None, OperatorRoleChangeType::AssignmentRevoked)
            .await
            .expect("revocar analyst");
        assert_eq!(revoked.status, LifecycleStatus::Revoked, "precondición: la fila quedó REVOKED");

        clock.tick();
        let (reactivated, _) = assignment_repo
            .set_assignment(analyst_input(), "node-A", None, OperatorRoleChangeType::AssignmentSet)
            .await
            .expect("reactivar analyst");
        assert_eq!(
            reactivated.status,
            LifecycleStatus::Active,
            "reactivar un operador REVOKED debe devolver status ACTIVE, no el REVOKED previo"
        );
        assert_eq!(reactivated.id, active.id, "reactivar es UPDATE de la MISMA fila");
        assert_eq!(reactivated.row_version, revoked.row_version + 1, "row_version devuelto debe ser el anterior + 1");
    }

    // ── Concurrencia optimista: VersionConflict real ─────────────────────────

    #[tokio::test]
    async fn update_role_matrix_from_stale_version_conflicts_instead_of_overwriting() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let (role_repo, _, admin_role, _) = seed_single_admin(&pool, &clock).await;
        let stale_view = admin_role.clone();

        clock.tick();
        let mut expanded = admin_matrix();
        expanded.set("generate.run_search", true);
        role_repo.update_role_matrix(&admin_role, expanded, "node-A", None).await.expect("primera actualización");

        clock.tick();
        let mut other_change = admin_matrix();
        other_change.set("execute.send_order", true);
        let conflict = role_repo.update_role_matrix(&stale_view, other_change, "node-A", None).await;
        assert!(matches!(conflict, Err(OperatorRoleError::VersionConflict { expected: 1 })), "resultado fue: {conflict:?}");
    }

    // ── CRITERIO (QA por mutación): contención sostenida por OPERACIÓN ────────
    //
    // Cada operación mutable tiene su PROPIO bucle inline de reintento
    // (`attempt += 1` / `if attempt < MAX_GUARDED_ATTEMPTS`). `cargo-mutants`
    // muta cada sitio por separado, así que cada bucle necesita ejercitarse
    // bajo contención SOSTENIDA -- un segundo escritor que retiene
    // `BEGIN IMMEDIATE` y no lo suelta, con `busy_timeout = 0` para que el
    // choque falle de inmediato. Mismo patrón que
    // `record_event_exhausts_exactly_max_attempts_when_write_lock_is_held`.

    /// Opciones de conexión con `busy_timeout = 0`: un lock ocupado falla de
    /// INMEDIATO con "database is locked" en vez de esperar.
    fn immediate_connect_options(database_url: &str) -> sqlx::sqlite::SqliteConnectOptions {
        use std::str::FromStr;
        use std::time::Duration;

        use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};

        SqliteConnectOptions::from_str(database_url)
            .expect("parsear opciones")
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_millis(0))
    }

    /// Un pool de UNA conexión con `busy_timeout = 0`.
    async fn immediate_pool(database_url: &str) -> SqlitePool {
        use sqlx::sqlite::SqlitePoolOptions;

        SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(immediate_connect_options(database_url))
            .await
            .expect("pool con busy_timeout=0")
    }

    /// Monta una DB de archivo temporal ya migrada (NUNCA `:memory:`, donde
    /// cada conexión sería una base distinta) + un pool normal para sembrar
    /// precondiciones. El `TempDir` devuelto debe mantenerse vivo mientras
    /// dure el test (al soltarse, borra la carpeta).
    async fn contention_fixture() -> (tempfile::TempDir, String, SqlitePool) {
        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("or_op_contention.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());
        let setup_pool = connect(&database_url).await.expect("conectar");
        migrate(&setup_pool).await.expect("migrar");
        (temp_dir, database_url, setup_pool)
    }

    /// Afirma que una operación mutable agotó EXACTAMENTE
    /// `MAX_GUARDED_ATTEMPTS` intentos -- mata el contador `attempt += 1` y
    /// el límite `attempt < MAX` del bucle inline de esa operación.
    fn assert_exhausted_contention<T: std::fmt::Debug>(result: Result<T, OperatorRoleError>) {
        match result {
            Err(OperatorRoleError::WriteContention { attempts }) => {
                assert_eq!(attempts, MAX_GUARDED_ATTEMPTS, "debe agotar EXACTAMENTE MAX_GUARDED_ATTEMPTS intentos");
            }
            other => panic!("se esperaba WriteContention {{ attempts: {MAX_GUARDED_ATTEMPTS} }}, se obtuvo: {other:?}"),
        }
    }

    /// Datos de un rol admin listos para sembrar en un test.
    fn new_admin_role(name: &str) -> NewOperatorRole {
        NewOperatorRole { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), role_name: name.to_string(), capability_matrix: admin_matrix() }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn create_role_exhausts_max_attempts_under_sustained_write_lock() {
        let (_dir, url, _setup) = contention_fixture().await;
        let clock = DeterministicClock::new(1_000, 100);

        let lock_pool = immediate_pool(&url).await;
        let lock_tx = lock_pool.begin_with("BEGIN IMMEDIATE").await.expect("tomar el lock de escritura");

        let repo_pool = immediate_pool(&url).await;
        let repo = OperatorRoleRepository::new(&repo_pool, &clock);
        let result = repo.create_role(new_admin_role("Admin"), "node-A").await;

        drop(lock_tx);
        assert_exhausted_contention(result);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn update_role_matrix_exhausts_max_attempts_under_sustained_write_lock() {
        let (_dir, url, setup) = contention_fixture().await;
        let clock = DeterministicClock::new(1_000, 100);

        let setup_repo = OperatorRoleRepository::new(&setup, &clock);
        let (admin_role, _) = setup_repo.create_role(new_admin_role("Admin"), "node-A").await.expect("sembrar rol admin");

        let lock_pool = immediate_pool(&url).await;
        let lock_tx = lock_pool.begin_with("BEGIN IMMEDIATE").await.expect("tomar el lock de escritura");

        let repo_pool = immediate_pool(&url).await;
        let repo = OperatorRoleRepository::new(&repo_pool, &clock);
        let mut expanded = admin_matrix();
        expanded.set("generate.run_search", true);
        let result = repo.update_role_matrix(&admin_role, expanded, "node-A", None).await;

        drop(lock_tx);
        assert_exhausted_contention(result);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn revoke_role_exhausts_max_attempts_under_sustained_write_lock() {
        let (_dir, url, setup) = contention_fixture().await;
        let clock = DeterministicClock::new(1_000, 100);

        let setup_role_repo = OperatorRoleRepository::new(&setup, &clock);
        let setup_assign_repo = OperatorAssignmentRepository::new(&setup, &clock);
        let (admin_role, _) = setup_role_repo.create_role(new_admin_role("Admin"), "node-A").await.expect("admin");
        setup_assign_repo
            .set_assignment(
                SetAssignmentInput { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), access_token_id: "tok-owner".to_string(), operator_type: OperatorType::Human, role_id: admin_role.id.clone() },
                "node-A", None, OperatorRoleChangeType::AssignmentSet,
            )
            .await
            .expect("asignar admin");
        // Rol desechable a revocar -- revocarlo deja el admin en pie.
        let (spare_role, _) = setup_role_repo
            .create_role(
                NewOperatorRole { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), role_name: "Disposable".to_string(), capability_matrix: analyst_matrix() },
                "node-A",
            )
            .await
            .expect("rol desechable");

        let lock_pool = immediate_pool(&url).await;
        let lock_tx = lock_pool.begin_with("BEGIN IMMEDIATE").await.expect("tomar el lock de escritura");

        let repo_pool = immediate_pool(&url).await;
        let repo = OperatorRoleRepository::new(&repo_pool, &clock);
        let result = repo.revoke_role(&spare_role, "node-A", None).await;

        drop(lock_tx);
        assert_exhausted_contention(result);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn set_assignment_exhausts_max_attempts_under_sustained_write_lock() {
        let (_dir, url, setup) = contention_fixture().await;
        let clock = DeterministicClock::new(1_000, 100);

        let setup_repo = OperatorRoleRepository::new(&setup, &clock);
        let (admin_role, _) = setup_repo.create_role(new_admin_role("Admin"), "node-A").await.expect("sembrar admin");

        let lock_pool = immediate_pool(&url).await;
        let lock_tx = lock_pool.begin_with("BEGIN IMMEDIATE").await.expect("tomar el lock de escritura");

        let repo_pool = immediate_pool(&url).await;
        let repo = OperatorAssignmentRepository::new(&repo_pool, &clock);
        let result = repo
            .set_assignment(
                SetAssignmentInput { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), access_token_id: "tok-new".to_string(), operator_type: OperatorType::Agent, role_id: admin_role.id.clone() },
                "node-A", None, OperatorRoleChangeType::AssignmentSet,
            )
            .await;

        drop(lock_tx);
        assert_exhausted_contention(result);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn revoke_assignment_exhausts_max_attempts_under_sustained_write_lock() {
        let (_dir, url, setup) = contention_fixture().await;
        let clock = DeterministicClock::new(1_000, 100);

        let setup_role_repo = OperatorRoleRepository::new(&setup, &clock);
        let setup_assign_repo = OperatorAssignmentRepository::new(&setup, &clock);
        let (admin_role, _) = setup_role_repo.create_role(new_admin_role("Admin"), "node-A").await.expect("admin");
        setup_assign_repo
            .set_assignment(
                SetAssignmentInput { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), access_token_id: "tok-1".to_string(), operator_type: OperatorType::Human, role_id: admin_role.id.clone() },
                "node-A", None, OperatorRoleChangeType::AssignmentSet,
            )
            .await
            .expect("primer admin");
        let (spare_role, _) = setup_role_repo.create_role(new_admin_role("SpareAdmin"), "node-A").await.expect("spare admin role");
        let (spare_assignment, _) = setup_assign_repo
            .set_assignment(
                SetAssignmentInput { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), access_token_id: "tok-2".to_string(), operator_type: OperatorType::Human, role_id: spare_role.id.clone() },
                "node-A", None, OperatorRoleChangeType::AssignmentSet,
            )
            .await
            .expect("segundo admin");

        let lock_pool = immediate_pool(&url).await;
        let lock_tx = lock_pool.begin_with("BEGIN IMMEDIATE").await.expect("tomar el lock de escritura");

        let repo_pool = immediate_pool(&url).await;
        let repo = OperatorAssignmentRepository::new(&repo_pool, &clock);
        let result = repo.revoke_assignment(&spare_assignment, "node-A", None, OperatorRoleChangeType::AssignmentRevoked).await;

        drop(lock_tx);
        assert_exhausted_contention(result);
    }

    // ── CRITERIO (QA por mutación): fidelidad de la fila devuelta al REVOCAR ──

    /// La fila que DEVUELVE `revoke_role` refleja la baja lógica (`REVOKED`),
    /// el `row_version` incrementado y el hash recomputado/encadenado -- no
    /// los valores viejos copiados de `..current.clone()`.
    #[tokio::test]
    async fn revoke_role_returned_row_reflects_revoked_status_chain_and_row_version() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let (role_repo, _, _admin_role, _) = seed_single_admin(&pool, &clock).await;

        // Rol desechable (no admin, sin asignación): revocarlo no toca el
        // invariante "último admin en pie".
        clock.tick(); // -> 1_100
        let (disposable, _) = role_repo
            .create_role(
                NewOperatorRole { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), role_name: "Disposable".to_string(), capability_matrix: analyst_matrix() },
                "node-A",
            )
            .await
            .expect("crear rol desechable");
        assert_eq!(disposable.status, LifecycleStatus::Active);

        clock.tick(); // -> 1_200
        let (revoked, _) = role_repo.revoke_role(&disposable, "node-A", None).await.expect("revocar rol");

        assert_eq!(revoked.status, LifecycleStatus::Revoked, "la baja lógica devuelta debe ser REVOKED");
        assert_eq!(revoked.row_version, disposable.row_version + 1, "row_version devuelto debe ser el anterior + 1");
        assert_ne!(revoked.audit_hash, disposable.audit_hash, "debe devolver el audit_hash recomputado");
        assert_eq!(revoked.audit_chain_hash, Some(disposable.audit_hash.clone()), "el chain devuelto encadena a la versión previa");
        assert_eq!(revoked.updated_at_ns, 1_200, "updated_at devuelto debe ser el now del reloj");
        assert_eq!(revoked.role_name, disposable.role_name, "el nombre del rol se conserva en la baja");
    }

    /// La fila que DEVUELVE `revoke_assignment` refleja la baja lógica
    /// (`REVOKED`), el `row_version` incrementado y el hash recomputado/
    /// encadenado. También fija la guarda `rows_affected() == 0` (un revoke
    /// exitoso NO debe caer en la rama de `VersionConflict`).
    #[tokio::test]
    async fn revoke_assignment_returned_row_reflects_revoked_status_chain_and_row_version() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let (role_repo, assignment_repo, _admin_role, _) = seed_single_admin(&pool, &clock).await;

        // Segundo admin para que revocar UNA asignación no viole el invariante.
        clock.tick(); // -> 1_100
        let (spare_role, _) = role_repo.create_role(new_admin_role("SpareAdmin"), "node-A").await.expect("crear spare admin");
        clock.tick(); // -> 1_200
        let (spare_assignment, _) = assignment_repo
            .set_assignment(
                SetAssignmentInput { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), access_token_id: "tok-2".to_string(), operator_type: OperatorType::Human, role_id: spare_role.id.clone() },
                "node-A", None, OperatorRoleChangeType::AssignmentSet,
            )
            .await
            .expect("segundo admin");
        assert_eq!(spare_assignment.status, LifecycleStatus::Active);

        clock.tick(); // -> 1_300
        let (revoked, _) = assignment_repo
            .revoke_assignment(&spare_assignment, "node-A", None, OperatorRoleChangeType::AssignmentRevoked)
            .await
            .expect("revocar asignación");

        assert_eq!(revoked.status, LifecycleStatus::Revoked, "la baja lógica devuelta debe ser REVOKED");
        assert_eq!(revoked.row_version, spare_assignment.row_version + 1, "row_version devuelto debe ser el anterior + 1");
        assert_ne!(revoked.audit_hash, spare_assignment.audit_hash, "debe devolver el audit_hash recomputado");
        assert_eq!(revoked.audit_chain_hash, Some(spare_assignment.audit_hash.clone()), "el chain devuelto encadena a la versión previa");
        assert_eq!(revoked.updated_at_ns, 1_300, "updated_at devuelto debe ser el now del reloj");
        assert_eq!(revoked.access_token_id, "tok-2", "el operador se conserva en la baja");
        assert_eq!(revoked.role_id, spare_role.id, "el role_id se conserva en la baja");
    }

    // ── CRITERIO (QA por mutación): load_assignments nunca devuelve vacío ─────

    /// `load_assignments` de una cuenta con ≥1 asignación devuelve esa
    /// asignación (mata el reemplazo del cuerpo por `Ok(vec![])`).
    #[tokio::test]
    async fn load_assignments_returns_the_created_assignment_not_empty() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let (_, _, admin_role, admin_assignment) = seed_single_admin(&pool, &clock).await;

        let repo = OperatorAssignmentRepository::new(&pool, &clock);
        let all = repo.load_assignments("acc-1").await.expect("cargar asignaciones");

        assert_eq!(all.len(), 1, "debe devolver la asignación creada, nunca un vector vacío");
        assert_eq!(all[0].access_token_id, admin_assignment.access_token_id, "debe traer el operador correcto");
        assert_eq!(all[0].role_id, admin_role.id, "la asignación debe apuntar al rol admin");
    }
}
