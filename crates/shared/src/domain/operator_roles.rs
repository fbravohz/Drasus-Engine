//! [CORE] Lógica pura de Operator Roles (`docs/features/operator-roles.md`,
//! ADR-0149 -- cimiento #14, ADR-0123, ADR-0141, ADR-0020, ADR-0137,
//! STORY-044).
//!
//! Sin I/O, sin reloj de sistema, sin aleatoriedad sin semilla
//! (ADR-0002/0004). Infraestructura de gobernanza transversal (mismo nivel
//! que `mcp_gateway`/`master_account_hierarchy`), NO dominio de trading: da
//! al dueño de una cuenta maestra un catálogo de roles a la carta (matriz de
//! capacidades por puerto de Feature) y asigna esos roles a operadores
//! (`HUMAN` o `AGENT`).
//!
//! Piezas de lógica pura:
//! - [`CapabilityMatrix`]: envuelve un `BTreeMap<String, bool>` -- **nunca**
//!   `HashMap`, porque el orden importa para que el hash de auditoría sea
//!   determinista. Denegado por defecto si la clave no está presente
//!   (refuerza el bloqueo-por-defecto de ADR-0123).
//! - [`evaluate_role_capability`] / [`RoleVerdict`]: resuelve si la matriz de
//!   un rol permite UNA capacidad puntual.
//! - [`evaluate_operator_call`] / [`CombinedVerdict`]: COMPONE el gate de rol
//!   con [`crate::domain::mcp_gateway::evaluate_permission`] (ADR-0123, ya
//!   existe, NUNCA se modifica) -- concede solo si AMBOS conceden.
//! - [`check_last_admin_standing`] / [`admins_remaining_after`]: el
//!   invariante "último admin en pie", una función pura y property-testable
//!   sobre el estado propuesto -- nunca un flag estático sobre un rol.
//! - [`can_create_child_account`] / [`ChildAccountVerdict`]: gate de creación
//!   de cuentas hijas, reutilizando `max_child_accounts` de
//!   `plan_tier_quota::PlanLimits` (#3), sin reinventar cuota.
//! - [`compute_role_audit_hash`] / [`compute_assignment_audit_hash`] /
//!   [`compute_event_audit_hash`]: los hashes de auditoría de las tres
//!   tablas -- encadenados por `row_version` (catálogo y asignaciones,
//!   MUTABLES) o por `event_sequence_id` (ledger, APPEND-ONLY).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Codifica bytes crudos a su representación hexadecimal en minúsculas --
/// mismo patrón que el resto del substrato (`data_portability::encode_hex`).
fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ── Capacidades reservadas (dato, no código -- ADR-0149) ────────────────────

/// La capacidad ADMIN: "gestionar operadores y roles". Protege el invariante
/// "último admin en pie" -- ningún cambio puede dejar la cuenta con cero
/// operadores que la retengan.
pub const CAPABILITY_MANAGE_ROLES: &str = "operator-roles.manage_roles";

/// La capacidad de crear una cuenta maestra hija nueva bajo un fondo --
/// siempre exige `CAPABILITY_MANAGE_ROLES` además de cuota disponible
/// (ver [`can_create_child_account`]).
pub const CAPABILITY_CREATE_CHILD_ACCOUNT: &str = "operator-roles.create_child_account";

// ── Vocabulario de tipo de operador (columna `operator_type`) ──────────────

/// El tipo de operador que recibe un rol -- catálogo CERRADO de dos valores,
/// el que acepta el `CHECK (operator_type IN ('HUMAN','AGENT'))` de la
/// migración. Un agente LLM (`Agent`) recibe rol EXPLÍCITO igual que un
/// humano -- nunca hereda permisos por el solo hecho de conectarse por MCP
/// (ADR-0149, regla fija #4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OperatorType {
    Human,
    Agent,
}

impl OperatorType {
    /// Representación canónica en texto -- la que acepta el `CHECK` de la
    /// migración.
    pub fn as_str(&self) -> &'static str {
        match self {
            OperatorType::Human => "HUMAN",
            OperatorType::Agent => "AGENT",
        }
    }

    /// Reconstruye el tipo desde su representación en texto, o `None` si no
    /// es ninguno de los dos reconocidos.
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "HUMAN" => Some(OperatorType::Human),
            "AGENT" => Some(OperatorType::Agent),
            _ => None,
        }
    }
}

// ── Vocabulario de baja lógica (columna `status` de las tablas MUTABLES) ───

/// Estado de vigencia de una fila MUTABLE (`operator_roles` /
/// `operator_assignments`) -- catálogo CERRADO de dos valores. "Eliminar" un
/// rol o una asignación NUNCA es un DELETE físico (ADR-0141): es una baja
/// lógica que mueve la fila a `Revoked`, preservando el historial y
/// respetando el `FOREIGN KEY ... ON DELETE RESTRICT` que protege las
/// asignaciones vivas de un rol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LifecycleStatus {
    Active,
    Revoked,
}

impl LifecycleStatus {
    /// Representación canónica en texto -- la que acepta el `CHECK` de la
    /// migración.
    pub fn as_str(&self) -> &'static str {
        match self {
            LifecycleStatus::Active => "ACTIVE",
            LifecycleStatus::Revoked => "REVOKED",
        }
    }

    /// Reconstruye el estado desde su representación en texto, o `None` si
    /// no es ninguno de los dos reconocidos.
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "ACTIVE" => Some(LifecycleStatus::Active),
            "REVOKED" => Some(LifecycleStatus::Revoked),
            _ => None,
        }
    }
}

// ── Vocabulario de tipo de evento del ledger (columna `change_type`) ───────

/// El tipo de cambio que registra UN evento de `operator_role_events` --
/// catálogo CERRADO de seis valores, el que acepta el `CHECK` de la
/// migración.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OperatorRoleChangeType {
    RoleCreated,
    RoleUpdated,
    RoleRevoked,
    AssignmentSet,
    AssignmentRevoked,
    AuthorityOverride,
}

impl OperatorRoleChangeType {
    /// Representación canónica en texto -- la que acepta el `CHECK` de la
    /// migración.
    pub fn as_str(&self) -> &'static str {
        match self {
            OperatorRoleChangeType::RoleCreated => "ROLE_CREATED",
            OperatorRoleChangeType::RoleUpdated => "ROLE_UPDATED",
            OperatorRoleChangeType::RoleRevoked => "ROLE_REVOKED",
            OperatorRoleChangeType::AssignmentSet => "ASSIGNMENT_SET",
            OperatorRoleChangeType::AssignmentRevoked => "ASSIGNMENT_REVOKED",
            OperatorRoleChangeType::AuthorityOverride => "AUTHORITY_OVERRIDE",
        }
    }

    /// Reconstruye el tipo de cambio desde su representación en texto, o
    /// `None` si no es ninguno de los seis reconocidos.
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "ROLE_CREATED" => Some(OperatorRoleChangeType::RoleCreated),
            "ROLE_UPDATED" => Some(OperatorRoleChangeType::RoleUpdated),
            "ROLE_REVOKED" => Some(OperatorRoleChangeType::RoleRevoked),
            "ASSIGNMENT_SET" => Some(OperatorRoleChangeType::AssignmentSet),
            "ASSIGNMENT_REVOKED" => Some(OperatorRoleChangeType::AssignmentRevoked),
            "AUTHORITY_OVERRIDE" => Some(OperatorRoleChangeType::AuthorityOverride),
            _ => None,
        }
    }
}

// ── Matriz de capacidades -- dato, no código (ADR-0149) ─────────────────────

/// La matriz de capacidades de UN rol -- envuelve un `BTreeMap<String,
/// bool>` en vez de un `HashMap` a propósito: `BTreeMap` itera en orden de
/// clave, lo que hace que su serialización JSON (y por tanto el hash de
/// auditoría que la incluye) sea DETERMINISTA. Un `HashMap` reordena sus
/// claves entre ejecuciones del proceso -- el mismo contenido lógico
/// produciría hashes distintos, rompiendo la reproducibilidad exigida por
/// ADR-0002.
///
/// `#[serde(transparent)]` hace que esta struct serialice/deserialice
/// EXACTAMENTE como el mapa interno (sin envoltorio) -- es la forma que
/// espera la columna `capability_matrix` (`CHECK (json_valid(...))`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CapabilityMatrix(BTreeMap<String, bool>);

impl CapabilityMatrix {
    /// Construye una matriz vacía -- toda capacidad queda denegada por
    /// defecto hasta que se declare explícitamente.
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Declara (o redeclara) el permiso de UNA capacidad -- consumidor
    /// típico: construir la matriz inicial de un rol nuevo.
    pub fn set(&mut self, capability_key: impl Into<String>, allowed: bool) -> &mut Self {
        self.0.insert(capability_key.into(), allowed);
        self
    }

    /// Resuelve si la matriz permite `capability_key`. **Denegado por
    /// defecto**: si la clave no está presente en el mapa, o está presente
    /// con valor `false`, el resultado es `false` -- nunca se asume permiso
    /// por ausencia (refuerza el bloqueo-por-defecto de ADR-0123).
    pub fn allows(&self, capability_key: &str) -> bool {
        self.0.get(capability_key).copied().unwrap_or(false)
    }

    /// Serializa la matriz a JSON canónico (claves ordenadas por ser
    /// `BTreeMap`) -- listo para la columna `capability_matrix`.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self)
            // Vec<(String,bool)> siempre serializa: no hay floats ni claves
            // de mapa no-string en juego.
            .expect("CapabilityMatrix siempre serializa a JSON")
    }

    /// Reconstruye una matriz desde su JSON persistido, o `None` si el
    /// contenido no es un objeto `{clave: bool, ...}` válido.
    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

// ── Gate de rol puntual ──────────────────────────────────────────────────

/// Veredicto de UNA evaluación de capacidad contra la matriz de un rol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoleVerdict {
    Granted,
    Denied { reason: String },
}

/// Evalúa si la matriz de un rol permite `capability_key` -- pura,
/// determinista. Es el primer insumo (compuerta #2 de ADR-0149) de
/// [`evaluate_operator_call`].
pub fn evaluate_role_capability(matrix: &CapabilityMatrix, capability_key: &str) -> RoleVerdict {
    if matrix.allows(capability_key) {
        RoleVerdict::Granted
    } else {
        RoleVerdict::Denied {
            reason: format!("el rol no otorga la capacidad '{capability_key}'"),
        }
    }
}

// ── Gate compuesto: rol (#14) AND pipeline (ADR-0123) ───────────────────────

/// Veredicto de una llamada de operador -- distingue POR QUÉ compuerta se
/// denegó, porque el rol y el pipeline son evaluadores independientes
/// (ADR-0149: "el rol de operador es una compuerta adicional, no
/// sustituta").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombinedVerdict {
    Granted,
    DeniedByRole { reason: String },
    DeniedByPipeline { reason: String },
}

/// Evalúa una llamada de operador COMPONIENDO el gate de rol (#14) con el
/// evaluador de permisos ya existente de `mcp_gateway`
/// ([`crate::domain::mcp_gateway::evaluate_permission`], ADR-0123) -- pura,
/// determinista.
///
/// **Nunca modifica `mcp_gateway`** -- lo importa y compone; la matriz de
/// rol se evalúa PRIMERO (más barato, sin construir el `PermissionRequest`
/// completo si el rol ya deniega), pero la semántica es simétrica: la
/// llamada se concede solo si AMBAS compuertas conceden.
pub fn evaluate_operator_call(
    matrix: &CapabilityMatrix,
    capability_key: &str,
    permission_request: &crate::domain::mcp_gateway::PermissionRequest,
) -> CombinedVerdict {
    if let RoleVerdict::Denied { reason } = evaluate_role_capability(matrix, capability_key) {
        return CombinedVerdict::DeniedByRole { reason };
    }

    match crate::domain::mcp_gateway::evaluate_permission(permission_request) {
        crate::domain::mcp_gateway::PermissionOutcome::Granted => CombinedVerdict::Granted,
        crate::domain::mcp_gateway::PermissionOutcome::Denied { reason } => {
            CombinedVerdict::DeniedByPipeline { reason }
        }
    }
}

// ── Invariante "último admin en pie" ────────────────────────────────────────

/// Vista mínima de UN rol -- solo lo que el invariante necesita (su
/// identificador y su matriz), proyectada desde
/// `persistence::operator_roles::OperatorRoleRow`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleView {
    pub role_id: String,
    pub matrix: CapabilityMatrix,
}

/// Vista mínima de UNA asignación vigente -- solo lo que el invariante
/// necesita (el operador y el rol que tiene asignado AHORA), proyectada
/// desde `persistence::operator_roles::OperatorAssignmentRow`. Únicamente se
/// alimentan aquí las asignaciones con `status = ACTIVE` -- una asignación
/// revocada ya no cuenta a su operador como admin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignmentView {
    pub access_token_id: String,
    pub role_id: String,
}

/// Un cambio propuesto sobre el catálogo/asignaciones de una cuenta --
/// cubre las cuatro vías por las que un cambio podría dejar la cuenta sin
/// ningún admin (ADR-0149, STORY-044 §4.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposedChange {
    /// Editar la matriz de un rol existente (ej. quitarle
    /// `CAPABILITY_MANAGE_ROLES`).
    UpdateRoleMatrix { role_id: String, new_matrix: CapabilityMatrix },
    /// Asignar (o reasignar) un rol a un operador -- si el operador ya
    /// tenía una asignación vigente, la reemplaza.
    SetAssignment { access_token_id: String, role_id: String },
    /// Revocar la asignación vigente de un operador (queda sin rol).
    RevokeAssignment { access_token_id: String },
    /// Revocar un rol completo -- ninguna asignación vigente a ese rol
    /// vuelve a contar tras el cambio.
    RevokeRole { role_id: String },
}

/// Violación del invariante "último admin en pie" -- el cambio propuesto
/// dejaría la cuenta con cero operadores que retengan
/// [`CAPABILITY_MANAGE_ROLES`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("este cambio dejaría la cuenta sin ningún operador con la capacidad de gestionar roles")]
pub struct LastAdminViolation;

/// Calcula cuántos operadores quedarían con [`CAPABILITY_MANAGE_ROLES`] tras
/// aplicar `change` sobre `roles`/`assignments` -- pura, determinista, sin
/// tocar el estado de entrada (trabaja sobre copias en memoria).
///
/// La cuenta final es: para cada asignación (ya con el cambio aplicado),
/// busca su rol (ya con el cambio aplicado); si el rol existe y su matriz
/// otorga `CAPABILITY_MANAGE_ROLES`, cuenta. Un rol revocado simplemente
/// deja de estar en la lista de roles -- las asignaciones que apuntaban a
/// él dejan de contar sin necesidad de un caso especial.
pub fn admins_remaining_after(roles: &[RoleView], assignments: &[AssignmentView], change: &ProposedChange) -> usize {
    let mut roles: Vec<RoleView> = roles.to_vec();
    let mut assignments: Vec<AssignmentView> = assignments.to_vec();

    match change {
        ProposedChange::UpdateRoleMatrix { role_id, new_matrix } => {
            if let Some(role) = roles.iter_mut().find(|r| &r.role_id == role_id) {
                role.matrix = new_matrix.clone();
            }
        }
        ProposedChange::SetAssignment { access_token_id, role_id } => {
            if let Some(existing) = assignments.iter_mut().find(|a| &a.access_token_id == access_token_id) {
                existing.role_id = role_id.clone();
            } else {
                assignments.push(AssignmentView {
                    access_token_id: access_token_id.clone(),
                    role_id: role_id.clone(),
                });
            }
        }
        ProposedChange::RevokeAssignment { access_token_id } => {
            assignments.retain(|a| &a.access_token_id != access_token_id);
        }
        ProposedChange::RevokeRole { role_id } => {
            roles.retain(|r| &r.role_id != role_id);
        }
    }

    assignments
        .iter()
        .filter(|assignment| {
            roles
                .iter()
                .find(|role| role.role_id == assignment.role_id)
                .map(|role| role.matrix.allows(CAPABILITY_MANAGE_ROLES))
                .unwrap_or(false)
        })
        .count()
}

/// Guardarraíl del invariante "último admin en pie" -- `Err` si el cambio
/// propuesto dejaría la cuenta con CERO operadores capaces de gestionar
/// roles. Property-testable: para cualquier estado y cambio generados, si
/// esta función devuelve `Ok`, [`admins_remaining_after`] sobre el MISMO
/// estado y cambio es `>= 1`.
pub fn check_last_admin_standing(
    roles: &[RoleView],
    assignments: &[AssignmentView],
    change: &ProposedChange,
) -> Result<(), LastAdminViolation> {
    if admins_remaining_after(roles, assignments, change) == 0 {
        Err(LastAdminViolation)
    } else {
        Ok(())
    }
}

// ── Gate de creación de cuentas hijas (reutiliza la cuota de #3) ───────────

/// Veredicto de una solicitud de creación de cuenta maestra hija.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildAccountVerdict {
    Granted,
    /// El actor no tiene `CAPABILITY_MANAGE_ROLES` vigente.
    DeniedNotAdmin,
    /// El actor SÍ es admin, pero la cuenta ya alcanzó `max_child_accounts`.
    DeniedQuotaExceeded,
}

/// Decide si un actor puede crear una cuenta maestra hija nueva -- pura:
/// exige AMBAS condiciones (ADR-0149, regla fija #3): capacidad ADMIN
/// vigente Y cuota disponible. `max_child_accounts` se reutiliza TAL CUAL
/// de `plan_tier_quota::PlanLimits` (#3) -- este cimiento no reinventa
/// cuota, solo la consulta.
pub fn can_create_child_account(
    actor_matrix: &CapabilityMatrix,
    current_child_count: i64,
    max_child_accounts: i64,
) -> ChildAccountVerdict {
    if !actor_matrix.allows(CAPABILITY_MANAGE_ROLES) {
        return ChildAccountVerdict::DeniedNotAdmin;
    }
    if current_child_count >= max_child_accounts {
        return ChildAccountVerdict::DeniedQuotaExceeded;
    }
    ChildAccountVerdict::Granted
}

// ── Hashes de auditoría ──────────────────────────────────────────────────

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de una VERSIÓN de fila
/// de `operator_roles`, encadenado a la versión anterior de la MISMA fila
/// (`previous_audit_hash: None` en la versión génesis, `row_version == 1`).
/// La matriz entra al hash ya serializada de forma ORDENADA
/// (`CapabilityMatrix::to_json`, `BTreeMap`) -- mismo input, mismo hash.
#[allow(clippy::too_many_arguments)]
pub fn compute_role_audit_hash(
    id: &str,
    created_at_ns: i64,
    row_version: i64,
    previous_audit_hash: Option<&str>,
    owner_id: &str,
    institutional_tag: &str,
    role_name: &str,
    capability_matrix_json: &str,
    status: &str,
) -> String {
    const SEP: char = '\u{1F}';

    let mut buffer = String::new();
    let mut push = |field: &str| {
        buffer.push_str(field);
        buffer.push(SEP);
    };

    push(id);
    push(&created_at_ns.to_string());
    push(&row_version.to_string());
    push(previous_audit_hash.unwrap_or(""));
    push(owner_id);
    push(institutional_tag);
    push(role_name);
    push(capability_matrix_json);
    push(status);

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    encode_hex(&hasher.finalize())
}

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de una VERSIÓN de fila
/// de `operator_assignments`, encadenado a la versión anterior de la MISMA
/// fila -- mismo estilo que [`compute_role_audit_hash`].
#[allow(clippy::too_many_arguments)]
pub fn compute_assignment_audit_hash(
    id: &str,
    created_at_ns: i64,
    row_version: i64,
    previous_audit_hash: Option<&str>,
    owner_id: &str,
    institutional_tag: &str,
    access_token_id: &str,
    operator_type: &str,
    role_id: &str,
    status: &str,
) -> String {
    const SEP: char = '\u{1F}';

    let mut buffer = String::new();
    let mut push = |field: &str| {
        buffer.push_str(field);
        buffer.push(SEP);
    };

    push(id);
    push(&created_at_ns.to_string());
    push(&row_version.to_string());
    push(previous_audit_hash.unwrap_or(""));
    push(owner_id);
    push(institutional_tag);
    push(access_token_id);
    push(operator_type);
    push(role_id);
    push(status);

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    encode_hex(&hasher.finalize())
}

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de UNA fila de
/// `operator_role_events`, encadenado al `audit_hash` de la fila anterior en
/// la secuencia GLOBAL (o [`crate::domain::audit_log::GENESIS_PREVIOUS_HASH`]
/// si es la fila génesis) -- mismo estilo que
/// `data_portability::compute_request_audit_hash`.
#[allow(clippy::too_many_arguments)]
pub fn compute_event_audit_hash(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    previous_audit_hash: &str,
    owner_id: &str,
    institutional_tag: &str,
    node_id: &str,
    compliance_status_id: Option<&str>,
    change_type: &str,
    subject_ref: &str,
    detail: Option<&str>,
) -> String {
    const SEP: char = '\u{1F}';

    let mut buffer = String::new();
    let mut push = |field: &str| {
        buffer.push_str(field);
        buffer.push(SEP);
    };

    push(id);
    push(&created_at_ns.to_string());
    push(&event_sequence_id.to_string());
    push(previous_audit_hash);
    push(owner_id);
    push(institutional_tag);
    push(node_id);
    push(compliance_status_id.unwrap_or(""));
    push(change_type);
    push(subject_ref);
    push(detail.unwrap_or(""));

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    encode_hex(&hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Enums: round-trip de representación en texto ────────────────────────

    #[test]
    fn operator_type_round_trips_through_its_string_representation() {
        for variant in [OperatorType::Human, OperatorType::Agent] {
            assert_eq!(OperatorType::from_str_value(variant.as_str()), Some(variant));
        }
        assert_eq!(OperatorType::from_str_value("UNKNOWN"), None);
    }

    #[test]
    fn lifecycle_status_round_trips_through_its_string_representation() {
        for variant in [LifecycleStatus::Active, LifecycleStatus::Revoked] {
            assert_eq!(LifecycleStatus::from_str_value(variant.as_str()), Some(variant));
        }
        assert_eq!(LifecycleStatus::from_str_value("UNKNOWN"), None);
    }

    #[test]
    fn change_type_round_trips_through_its_string_representation() {
        for variant in [
            OperatorRoleChangeType::RoleCreated,
            OperatorRoleChangeType::RoleUpdated,
            OperatorRoleChangeType::RoleRevoked,
            OperatorRoleChangeType::AssignmentSet,
            OperatorRoleChangeType::AssignmentRevoked,
            OperatorRoleChangeType::AuthorityOverride,
        ] {
            assert_eq!(OperatorRoleChangeType::from_str_value(variant.as_str()), Some(variant));
        }
        assert_eq!(OperatorRoleChangeType::from_str_value("UNKNOWN"), None);
    }

    // ── CRITERIO (Orden §8): matriz denegada-por-defecto ─────────────────────

    #[test]
    fn capability_matrix_denies_by_default_when_key_is_absent() {
        let matrix = CapabilityMatrix::new();
        assert!(!matrix.allows(CAPABILITY_MANAGE_ROLES), "una capacidad ausente debe denegarse, nunca permitirse");
    }

    #[test]
    fn capability_matrix_denies_when_key_is_explicitly_false() {
        let mut matrix = CapabilityMatrix::new();
        matrix.set("generate.run_search", false);
        assert!(!matrix.allows("generate.run_search"));
    }

    #[test]
    fn capability_matrix_grants_only_when_key_is_explicitly_true() {
        let mut matrix = CapabilityMatrix::new();
        matrix.set("generate.run_search", true);
        assert!(matrix.allows("generate.run_search"));
        assert!(!matrix.allows("execute.send_order"), "una capacidad distinta, nunca declarada, sigue denegada");
    }

    // ── CRITERIO (Orden §8): hash determinista con matriz ordenada ──────────

    #[test]
    fn capability_matrix_json_is_deterministic_regardless_of_insertion_order() {
        let mut a = CapabilityMatrix::new();
        a.set("zeta.op", true);
        a.set("alpha.op", false);

        let mut b = CapabilityMatrix::new();
        b.set("alpha.op", false);
        b.set("zeta.op", true);

        assert_eq!(a.to_json(), b.to_json(), "el orden de inserción no debe afectar el JSON -- BTreeMap ordena por clave");
    }

    #[test]
    fn capability_matrix_round_trips_through_json() {
        let mut matrix = CapabilityMatrix::new();
        matrix.set(CAPABILITY_MANAGE_ROLES, true);
        matrix.set("generate.run_search", false);

        let json = matrix.to_json();
        let restored = CapabilityMatrix::from_json(&json).expect("debe parsear de vuelta");
        assert_eq!(matrix, restored);
    }

    // ── CRITERIO (Orden §8): gate de rol puntual ─────────────────────────────

    #[test]
    fn evaluate_role_capability_grants_when_matrix_allows() {
        let mut matrix = CapabilityMatrix::new();
        matrix.set("generate.run_search", true);
        assert_eq!(evaluate_role_capability(&matrix, "generate.run_search"), RoleVerdict::Granted);
    }

    #[test]
    fn evaluate_role_capability_denies_when_absent() {
        let matrix = CapabilityMatrix::new();
        assert!(matches!(evaluate_role_capability(&matrix, "generate.run_search"), RoleVerdict::Denied { .. }));
    }

    // ── CRITERIO (Orden §8): gate compuesto -- exige AMBAS compuertas ────────

    fn open_pipeline_request() -> crate::domain::mcp_gateway::PermissionRequest {
        crate::domain::mcp_gateway::PermissionRequest {
            pipeline: crate::domain::mcp_gateway::Pipeline::Generate,
            institutional_tag: None,
            production_override_active: false,
            agent_session_id: "session-test".to_string(),
            requested_scope: "generate.run_search".to_string(),
        }
    }

    fn blocked_pipeline_request() -> crate::domain::mcp_gateway::PermissionRequest {
        crate::domain::mcp_gateway::PermissionRequest {
            pipeline: crate::domain::mcp_gateway::Pipeline::Execute,
            institutional_tag: None,
            production_override_active: false,
            agent_session_id: "session-test".to_string(),
            requested_scope: "execute.send_order".to_string(),
        }
    }

    #[test]
    fn evaluate_operator_call_grants_only_when_both_role_and_pipeline_grant() {
        let mut matrix = CapabilityMatrix::new();
        matrix.set("generate.run_search", true);

        let verdict = evaluate_operator_call(&matrix, "generate.run_search", &open_pipeline_request());
        assert_eq!(verdict, CombinedVerdict::Granted);
    }

    #[test]
    fn evaluate_operator_call_denies_by_role_when_role_forbids_even_with_open_pipeline() {
        let matrix = CapabilityMatrix::new(); // sin declarar nada -> denegado
        let verdict = evaluate_operator_call(&matrix, "generate.run_search", &open_pipeline_request());
        assert!(matches!(verdict, CombinedVerdict::DeniedByRole { .. }), "verdict fue: {verdict:?}");
    }

    #[test]
    fn evaluate_operator_call_denies_by_pipeline_when_role_grants_but_pipeline_blocks() {
        let mut matrix = CapabilityMatrix::new();
        matrix.set("execute.send_order", true);

        let verdict = evaluate_operator_call(&matrix, "execute.send_order", &blocked_pipeline_request());
        assert!(matches!(verdict, CombinedVerdict::DeniedByPipeline { .. }), "verdict fue: {verdict:?}");
    }

    // ── CRITERIO (Orden §8): "último admin en pie" ───────────────────────────

    fn admin_matrix() -> CapabilityMatrix {
        let mut m = CapabilityMatrix::new();
        m.set(CAPABILITY_MANAGE_ROLES, true);
        m
    }

    fn non_admin_matrix() -> CapabilityMatrix {
        let mut m = CapabilityMatrix::new();
        m.set("generate.run_search", true);
        m
    }

    /// Revocar/reasignar al ÚNICO admin -> `LastAdminViolation`.
    #[test]
    fn revoking_the_only_admin_assignment_violates_the_invariant() {
        let roles = vec![RoleView { role_id: "role-admin".to_string(), matrix: admin_matrix() }];
        let assignments = vec![AssignmentView { access_token_id: "tok-1".to_string(), role_id: "role-admin".to_string() }];

        let change = ProposedChange::RevokeAssignment { access_token_id: "tok-1".to_string() };
        assert_eq!(check_last_admin_standing(&roles, &assignments, &change), Err(LastAdminViolation));
    }

    #[test]
    fn reassigning_the_only_admin_to_a_non_admin_role_violates_the_invariant() {
        let roles = vec![
            RoleView { role_id: "role-admin".to_string(), matrix: admin_matrix() },
            RoleView { role_id: "role-analyst".to_string(), matrix: non_admin_matrix() },
        ];
        let assignments = vec![AssignmentView { access_token_id: "tok-1".to_string(), role_id: "role-admin".to_string() }];

        let change = ProposedChange::SetAssignment { access_token_id: "tok-1".to_string(), role_id: "role-analyst".to_string() };
        assert_eq!(check_last_admin_standing(&roles, &assignments, &change), Err(LastAdminViolation));
    }

    /// Con >=2 admins, reasignar a UNO -> OK (el otro sigue de pie).
    #[test]
    fn reassigning_one_of_two_admins_is_allowed() {
        let roles = vec![
            RoleView { role_id: "role-admin".to_string(), matrix: admin_matrix() },
            RoleView { role_id: "role-analyst".to_string(), matrix: non_admin_matrix() },
        ];
        let assignments = vec![
            AssignmentView { access_token_id: "tok-1".to_string(), role_id: "role-admin".to_string() },
            AssignmentView { access_token_id: "tok-2".to_string(), role_id: "role-admin".to_string() },
        ];

        let change = ProposedChange::SetAssignment { access_token_id: "tok-1".to_string(), role_id: "role-analyst".to_string() };
        assert_eq!(check_last_admin_standing(&roles, &assignments, &change), Ok(()));
    }

    /// Editar la matriz del rol ADMIN para quitarle `MANAGE_ROLES` cuando es
    /// el único admin -> `LastAdminViolation`.
    #[test]
    fn stripping_manage_roles_from_the_only_admin_role_violates_the_invariant() {
        let roles = vec![RoleView { role_id: "role-admin".to_string(), matrix: admin_matrix() }];
        let assignments = vec![AssignmentView { access_token_id: "tok-1".to_string(), role_id: "role-admin".to_string() }];

        let mut stripped = admin_matrix();
        stripped.set(CAPABILITY_MANAGE_ROLES, false);
        let change = ProposedChange::UpdateRoleMatrix { role_id: "role-admin".to_string(), new_matrix: stripped };
        assert_eq!(check_last_admin_standing(&roles, &assignments, &change), Err(LastAdminViolation));
    }

    /// Revocar el rol ADMIN completo cuando es el único con la capacidad ->
    /// `LastAdminViolation` (cubre la cuarta vía: revocar el rol en sí).
    #[test]
    fn revoking_the_only_admin_role_violates_the_invariant() {
        let roles = vec![RoleView { role_id: "role-admin".to_string(), matrix: admin_matrix() }];
        let assignments = vec![AssignmentView { access_token_id: "tok-1".to_string(), role_id: "role-admin".to_string() }];

        let change = ProposedChange::RevokeRole { role_id: "role-admin".to_string() };
        assert_eq!(check_last_admin_standing(&roles, &assignments, &change), Err(LastAdminViolation));
    }

    /// Property test (Capa 3, ADR-0133): para CUALQUIER estado generado
    /// dentro del espacio combinatorio (número de admins, roles adicionales
    /// de ruido, y las cuatro vías de cambio posibles), si
    /// `check_last_admin_standing` acepta el cambio, recalcular
    /// `admins_remaining_after` sobre el MISMO estado/cambio siempre da
    /// `>= 1` -- fija la coherencia entre el guardarraíl y la función de
    /// conteo que lo sostiene. Recorre el espacio EXHAUSTIVAMENTE (sin
    /// depender de un crate de generación aleatoria externo, que este
    /// workspace no usa en ningún otro punto) en vez de muestrear al azar --
    /// más fuerte que una muestra aleatoria para un espacio de este tamaño.
    #[test]
    fn property_accepted_changes_always_leave_at_least_one_admin() {
        for admin_count in 1usize..5 {
            for extra_role_count in 0usize..3 {
                for change_seed in 0u8..4 {
                    let mut roles = Vec::new();
                    let mut assignments = Vec::new();

                    // Un único rol ADMIN, con `admin_count` operadores asignados.
                    roles.push(RoleView { role_id: "role-admin".to_string(), matrix: admin_matrix() });
                    for i in 0..admin_count {
                        assignments.push(AssignmentView {
                            access_token_id: format!("tok-admin-{i}"),
                            role_id: "role-admin".to_string(),
                        });
                    }

                    // Roles no-admin adicionales, sin asignaciones -- ruido del espacio de estados.
                    for i in 0..extra_role_count {
                        roles.push(RoleView { role_id: format!("role-extra-{i}"), matrix: non_admin_matrix() });
                    }

                    // Cambios candidatos sobre el primer admin (tok-admin-0) -- las cuatro vías.
                    let change = match change_seed {
                        0 => ProposedChange::RevokeAssignment { access_token_id: "tok-admin-0".to_string() },
                        1 => ProposedChange::SetAssignment {
                            access_token_id: "tok-admin-0".to_string(),
                            role_id: "role-admin".to_string(),
                        },
                        2 => {
                            let mut m = admin_matrix();
                            m.set(CAPABILITY_MANAGE_ROLES, false);
                            ProposedChange::UpdateRoleMatrix { role_id: "role-admin".to_string(), new_matrix: m }
                        }
                        _ => ProposedChange::RevokeRole { role_id: "role-admin".to_string() },
                    };

                    let verdict = check_last_admin_standing(&roles, &assignments, &change);
                    let remaining = admins_remaining_after(&roles, &assignments, &change);

                    if verdict.is_ok() {
                        assert!(
                            remaining >= 1,
                            "el guardarraíl aceptó un cambio que deja {remaining} admins \
                             (admin_count={admin_count}, extra_role_count={extra_role_count}, change_seed={change_seed})"
                        );
                    } else {
                        assert_eq!(
                            remaining, 0,
                            "el guardarraíl rechazó un cambio que en realidad deja {remaining} admins \
                             (admin_count={admin_count}, extra_role_count={extra_role_count}, change_seed={change_seed})"
                        );
                    }
                }
            }
        }
    }

    // ── CRITERIO (Orden §8): cuota de cuentas hijas -- borde exacto ─────────

    #[test]
    fn can_create_child_account_denies_non_admin() {
        let verdict = can_create_child_account(&non_admin_matrix(), 0, 5);
        assert_eq!(verdict, ChildAccountVerdict::DeniedNotAdmin);
    }

    #[test]
    fn can_create_child_account_denies_at_exact_quota_limit() {
        let verdict = can_create_child_account(&admin_matrix(), 5, 5);
        assert_eq!(verdict, ChildAccountVerdict::DeniedQuotaExceeded, "current_child_count == max_child_accounts debe denegar, no permitir uno de más");
    }

    #[test]
    fn can_create_child_account_grants_admin_under_quota() {
        let verdict = can_create_child_account(&admin_matrix(), 4, 5);
        assert_eq!(verdict, ChildAccountVerdict::Granted);
    }

    // ── Hashes: determinismo + sensibilidad a cada campo ─────────────────────

    #[test]
    fn compute_role_audit_hash_is_deterministic() {
        let a = compute_role_audit_hash("id-1", 1_000, 1, None, "owner-1", "LIVE", "Analyst", "{}", "ACTIVE");
        let b = compute_role_audit_hash("id-1", 1_000, 1, None, "owner-1", "LIVE", "Analyst", "{}", "ACTIVE");
        assert_eq!(a, b);
    }

    #[test]
    fn compute_role_audit_hash_changes_when_matrix_json_changes() {
        let without = compute_role_audit_hash("id-1", 1_000, 1, None, "owner-1", "LIVE", "Analyst", "{}", "ACTIVE");
        let with = compute_role_audit_hash(
            "id-1", 1_000, 1, None, "owner-1", "LIVE", "Analyst",
            "{\"operator-roles.manage_roles\":true}", "ACTIVE",
        );
        assert_ne!(without, with, "cambiar la matriz debe cambiar el hash");
    }

    #[test]
    fn compute_role_audit_hash_changes_when_status_changes() {
        let active = compute_role_audit_hash("id-1", 1_000, 1, None, "owner-1", "LIVE", "Analyst", "{}", "ACTIVE");
        let revoked = compute_role_audit_hash("id-1", 1_000, 1, None, "owner-1", "LIVE", "Analyst", "{}", "REVOKED");
        assert_ne!(active, revoked, "revocar el rol debe cambiar el hash");
    }

    #[test]
    fn compute_assignment_audit_hash_changes_when_role_id_changes() {
        let a = compute_assignment_audit_hash("id-1", 1_000, 1, None, "owner-1", "LIVE", "tok-1", "HUMAN", "role-admin", "ACTIVE");
        let b = compute_assignment_audit_hash("id-1", 1_000, 1, None, "owner-1", "LIVE", "tok-1", "HUMAN", "role-analyst", "ACTIVE");
        assert_ne!(a, b, "reasignar a otro rol debe cambiar el hash");
    }

    #[test]
    fn compute_event_audit_hash_changes_when_subject_ref_changes() {
        let a = compute_event_audit_hash("id-1", 1_000, 1, "GENESIS", "owner-1", "LIVE", "node-A", None, "ROLE_CREATED", "role-1", None);
        let b = compute_event_audit_hash("id-1", 1_000, 1, "GENESIS", "owner-1", "LIVE", "node-A", None, "ROLE_CREATED", "role-2", None);
        assert_ne!(a, b);
    }
}
