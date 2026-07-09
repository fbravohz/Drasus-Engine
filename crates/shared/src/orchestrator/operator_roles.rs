//! [SHELL] Composición del cimiento #14 (`docs/features/operator-roles.md`,
//! ADR-0149, ADR-0123, ADR-0141, STORY-044).
//!
//! Capa delgada sobre [`crate::persistence::operator_roles`]: traduce las
//! operaciones que el resto del substrato necesita -- "define un rol",
//! "asigna un operador", "revoca una asignación/rol", "evalúa una llamada
//! de operador" y "decide si puede crear una cuenta hija" -- sin que el
//! llamador tenga que conocer los repositorios ni el esquema de las
//! tablas. Mismo rol que `orchestrator::data_portability` para el
//! cimiento #13.
//!
//! **Transporte de red de la cascada del fondo diferido (STORY-044 §6):**
//! [`apply_authority_override`] modela SOLO la decisión/registro LOCAL de
//! un override de asignación de rol -- el relé genérico cifrado (ADR-0143)
//! y la doble atestación cross-máquina completa de `master-account-hierarchy`
//! (#12) son un adaptador posterior sobre este mismo puerto.

use sqlx::SqlitePool;

use crate::domain::mcp_gateway::PermissionRequest;
use crate::domain::operator_roles::{CapabilityMatrix, ChildAccountVerdict, CombinedVerdict, OperatorRoleChangeType, OperatorType};
use crate::domain::plan_tier_quota::PlanLimits;
use crate::persistence::operator_roles::{
    NewOperatorRole, OperatorAssignmentRepository, OperatorAssignmentRow, OperatorRoleEventRow, OperatorRoleError,
    OperatorRoleRepository, OperatorRoleRow, SetAssignmentInput,
};

/// La identidad Grupo II/IV que acompaña cualquier operación de este
/// cimiento -- mismo rol que `DataPortabilityIdentity` (#13):
/// `owner_id`/`institutional_tag` SIEMPRE salen de `central-identity` (#1),
/// nunca se inventan sueltos.
#[derive(Debug, Clone)]
pub struct OperatorRolesIdentity {
    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,
}

/// El dueño de una cuenta define un rol nuevo (nombre libre + matriz de
/// capacidades) -- crea la fila `operator_roles` y registra el evento
/// `ROLE_CREATED` en la MISMA transacción.
pub async fn define_role(
    pool: &SqlitePool,
    clock: &dyn crate::domain::clock::Clock,
    identity: &OperatorRolesIdentity,
    role_name: &str,
    matrix: CapabilityMatrix,
) -> Result<(OperatorRoleRow, OperatorRoleEventRow), OperatorRoleError> {
    let repo = OperatorRoleRepository::new(pool, clock);
    repo.create_role(
        NewOperatorRole {
            owner_id: identity.owner_id.clone(),
            institutional_tag: identity.institutional_tag.clone(),
            role_name: role_name.to_string(),
            capability_matrix: matrix,
        },
        &identity.node_id,
    )
    .await
}

/// Reclasifica la matriz de un rol existente -- pasa por el guardarraíl
/// transaccional "último admin en pie" del repositorio (ver
/// `persistence::operator_roles::OperatorRoleRepository::update_role_matrix`).
pub async fn update_role_matrix(
    pool: &SqlitePool,
    clock: &dyn crate::domain::clock::Clock,
    identity: &OperatorRolesIdentity,
    role_id: &str,
    new_matrix: CapabilityMatrix,
) -> Result<(OperatorRoleRow, OperatorRoleEventRow), OperatorRoleError> {
    let role_repo = OperatorRoleRepository::new(pool, clock);
    let current = role_repo.get_role(role_id).await?.ok_or_else(|| OperatorRoleError::RoleNotFound(role_id.to_string()))?;
    role_repo.update_role_matrix(&current, new_matrix, &identity.node_id, None).await
}

/// Revoca un rol -- baja lógica (`status = REVOKED`), NUNCA DELETE físico
/// (ADR-0141). Pasa por el MISMO guardarraíl que [`update_role_matrix`].
pub async fn revoke_role(
    pool: &SqlitePool,
    clock: &dyn crate::domain::clock::Clock,
    identity: &OperatorRolesIdentity,
    role_id: &str,
) -> Result<(OperatorRoleRow, OperatorRoleEventRow), OperatorRoleError> {
    let role_repo = OperatorRoleRepository::new(pool, clock);
    let current = role_repo.get_role(role_id).await?.ok_or_else(|| OperatorRoleError::RoleNotFound(role_id.to_string()))?;
    role_repo.revoke_role(&current, &identity.node_id, None).await
}

/// Asigna (o reasigna) un rol a un operador -- `HUMAN` (login) o `AGENT`
/// (conexión MCP), mismo mecanismo para ambos (ADR-0149: "un agente LLM
/// recibe rol explícito del mismo catálogo, nunca un sistema paralelo").
/// Pasa por el guardarraíl transaccional "último admin en pie".
pub async fn assign_operator(
    pool: &SqlitePool,
    clock: &dyn crate::domain::clock::Clock,
    identity: &OperatorRolesIdentity,
    access_token_id: &str,
    operator_type: OperatorType,
    role_id: &str,
) -> Result<(OperatorAssignmentRow, OperatorRoleEventRow), OperatorRoleError> {
    let repo = OperatorAssignmentRepository::new(pool, clock);
    repo.set_assignment(
        SetAssignmentInput {
            owner_id: identity.owner_id.clone(),
            institutional_tag: identity.institutional_tag.clone(),
            access_token_id: access_token_id.to_string(),
            operator_type,
            role_id: role_id.to_string(),
        },
        &identity.node_id,
        None,
        OperatorRoleChangeType::AssignmentSet,
    )
    .await
}

/// Revoca la asignación ACTIVA de un operador -- baja lógica; el operador
/// queda SIN rol (ADR-0149: sin rol = denegado). Pasa por el MISMO
/// guardarraíl que [`assign_operator`].
pub async fn revoke_assignment(
    pool: &SqlitePool,
    clock: &dyn crate::domain::clock::Clock,
    identity: &OperatorRolesIdentity,
    access_token_id: &str,
) -> Result<(OperatorAssignmentRow, OperatorRoleEventRow), OperatorRoleError> {
    let repo = OperatorAssignmentRepository::new(pool, clock);
    let current = repo
        .find_active_assignment(&identity.owner_id, access_token_id)
        .await?
        .ok_or_else(|| OperatorRoleError::AssignmentNotFound(access_token_id.to_string()))?;
    repo.revoke_assignment(&current, &identity.node_id, None, OperatorRoleChangeType::AssignmentRevoked).await
}

/// Resultado de [`evaluate_call`]: el veredicto compuesto (rol AND
/// pipeline) más el `role_id` resuelto (si el operador tenía asignación
/// ACTIVA) -- útil para auditoría/CLI sin tener que re-consultar.
#[derive(Debug, Clone)]
pub struct EvaluateCallResult {
    pub verdict: CombinedVerdict,
    pub resolved_role_id: Option<String>,
}

/// Evalúa una llamada de operador: resuelve su asignación ACTIVA, carga la
/// matriz del rol asignado, y devuelve el `CombinedVerdict` del Core (rol
/// #14 AND pipeline ADR-0123). Si el operador NO tiene asignación ACTIVA
/// -> **denegado** (ADR-0149, regla fija #4: un `AGENT` -- o un `HUMAN` --
/// nunca opera sin rol explícito).
pub async fn evaluate_call(
    pool: &SqlitePool,
    clock: &dyn crate::domain::clock::Clock,
    identity: &OperatorRolesIdentity,
    access_token_id: &str,
    capability_key: &str,
    permission_request: &PermissionRequest,
) -> Result<EvaluateCallResult, OperatorRoleError> {
    let assignment_repo = OperatorAssignmentRepository::new(pool, clock);
    let Some(assignment) = assignment_repo.find_active_assignment(&identity.owner_id, access_token_id).await? else {
        return Ok(EvaluateCallResult {
            verdict: CombinedVerdict::DeniedByRole {
                reason: "el operador no tiene ningún rol asignado en esta cuenta".to_string(),
            },
            resolved_role_id: None,
        });
    };

    let role_repo = OperatorRoleRepository::new(pool, clock);
    let Some(role) = role_repo.get_role(&assignment.role_id).await? else {
        // Guardarraíl de integridad: la FK ON DELETE RESTRICT + la baja
        // lógica hacen esto virtualmente imposible en producción, pero un
        // rol borrado a mano en la BD no debe hacer panic -- se trata
        // igual que "sin rol".
        return Ok(EvaluateCallResult {
            verdict: CombinedVerdict::DeniedByRole { reason: "el rol asignado ya no existe".to_string() },
            resolved_role_id: None,
        });
    };

    let verdict = crate::domain::operator_roles::evaluate_operator_call(&role.capability_matrix, capability_key, permission_request);
    Ok(EvaluateCallResult { verdict, resolved_role_id: Some(role.id) })
}

/// Decide si `actor_access_token_id` puede crear una cuenta maestra hija
/// nueva -- resuelve la matriz del actor y delega al Core
/// ([`crate::domain::operator_roles::can_create_child_account`]) con el
/// `max_child_accounts` de `plan_tier_quota::PlanLimits` (#3). NO crea la
/// cuenta hija (eso es `master-account-hierarchy`, #12); aquí se decide y
/// se puede registrar la autorización.
pub async fn request_child_account(
    pool: &SqlitePool,
    clock: &dyn crate::domain::clock::Clock,
    identity: &OperatorRolesIdentity,
    actor_access_token_id: &str,
    current_child_count: i64,
    plan_limits: &PlanLimits,
) -> Result<ChildAccountVerdict, OperatorRoleError> {
    let assignment_repo = OperatorAssignmentRepository::new(pool, clock);
    let Some(assignment) = assignment_repo.find_active_assignment(&identity.owner_id, actor_access_token_id).await? else {
        return Ok(ChildAccountVerdict::DeniedNotAdmin);
    };

    let role_repo = OperatorRoleRepository::new(pool, clock);
    let Some(role) = role_repo.get_role(&assignment.role_id).await? else {
        return Ok(ChildAccountVerdict::DeniedNotAdmin);
    };

    Ok(crate::domain::operator_roles::can_create_child_account(
        &role.capability_matrix,
        current_child_count,
        plan_limits.max_child_accounts,
    ))
}

/// Siembra el rol ADMIN inicial (`CAPABILITY_MANAGE_ROLES` +
/// `CAPABILITY_CREATE_CHILD_ACCOUNT` en `true`) y lo asigna al `owner_id`
/// raíz como primer operador `HUMAN` -- el "primer admin por defecto" de
/// ADR-0149. IDEMPOTENTE: si la cuenta ya tiene un rol `"Admin"`, no lo
/// vuelve a crear ni a reasignar (evita duplicar `role_name` bajo el
/// `UNIQUE(owner_id, role_name)` y no pisa una asignación ya personalizada
/// por el dueño).
pub async fn seed_admin_bootstrap(
    pool: &SqlitePool,
    clock: &dyn crate::domain::clock::Clock,
    identity: &OperatorRolesIdentity,
    root_access_token_id: &str,
) -> Result<(OperatorRoleRow, OperatorAssignmentRow), OperatorRoleError> {
    const BOOTSTRAP_ROLE_NAME: &str = "Admin";

    let role_repo = OperatorRoleRepository::new(pool, clock);
    let existing_roles = role_repo.load_roles(&identity.owner_id).await?;

    let admin_role = if let Some(existing) = existing_roles.into_iter().find(|r| r.role_name == BOOTSTRAP_ROLE_NAME) {
        existing
    } else {
        let mut matrix = CapabilityMatrix::new();
        matrix.set(crate::domain::operator_roles::CAPABILITY_MANAGE_ROLES, true);
        matrix.set(crate::domain::operator_roles::CAPABILITY_CREATE_CHILD_ACCOUNT, true);

        let (role, _) = define_role(pool, clock, identity, BOOTSTRAP_ROLE_NAME, matrix).await?;
        role
    };

    let assignment_repo = OperatorAssignmentRepository::new(pool, clock);
    let assignment = if let Some(existing) =
        assignment_repo.find_assignment(&identity.owner_id, root_access_token_id).await?
    {
        existing
    } else {
        let (assignment, _) = assign_operator(pool, clock, identity, root_access_token_id, OperatorType::Human, &admin_role.id).await?;
        assignment
    };

    Ok((admin_role, assignment))
}

/// Modela la decisión/registro LOCAL de una cascada de autoridad del fondo
/// sobre la asignación de rol de una cuenta hija (ADR-0149 + #12): aplica
/// el cambio de asignación (o su revocación) y lo etiqueta
/// `AUTHORITY_OVERRIDE` en el ledger -- misma mecánica que una asignación
/// normal, distinta etiqueta de auditoría porque el mando NO se originó en
/// el ADMIN de la propia cuenta hija, sino en la cuenta maestra raíz del
/// fondo.
///
/// **Diferido (STORY-044 §6):** el transporte de red del mando cifrado (el
/// relé genérico de ADR-0143) y la doble atestación cross-máquina completa
/// de `master-account-hierarchy` (#12) -- aquí solo se aplica el efecto
/// LOCAL sobre la cuenta hija, asumiendo que el mando ya llegó autenticado.
pub async fn apply_authority_override(
    pool: &SqlitePool,
    clock: &dyn crate::domain::clock::Clock,
    child_identity: &OperatorRolesIdentity,
    access_token_id: &str,
    new_role_id: Option<&str>,
    operator_type: OperatorType,
) -> Result<(OperatorAssignmentRow, OperatorRoleEventRow), OperatorRoleError> {
    let assignment_repo = OperatorAssignmentRepository::new(pool, clock);

    match new_role_id {
        Some(role_id) => {
            assignment_repo
                .set_assignment(
                    SetAssignmentInput {
                        owner_id: child_identity.owner_id.clone(),
                        institutional_tag: child_identity.institutional_tag.clone(),
                        access_token_id: access_token_id.to_string(),
                        operator_type,
                        role_id: role_id.to_string(),
                    },
                    &child_identity.node_id,
                    None,
                    OperatorRoleChangeType::AuthorityOverride,
                )
                .await
        }
        None => {
            let current = assignment_repo
                .find_active_assignment(&child_identity.owner_id, access_token_id)
                .await?
                .ok_or_else(|| OperatorRoleError::AssignmentNotFound(access_token_id.to_string()))?;
            assignment_repo
                .revoke_assignment(&current, &child_identity.node_id, None, OperatorRoleChangeType::AuthorityOverride)
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::domain::mcp_gateway::Pipeline;
    use crate::domain::operator_roles::{CAPABILITY_CREATE_CHILD_ACCOUNT, CAPABILITY_MANAGE_ROLES};
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    fn sample_identity() -> OperatorRolesIdentity {
        OperatorRolesIdentity { owner_id: "acc-1".to_string(), institutional_tag: "LIVE".to_string(), node_id: "node-A".to_string() }
    }

    fn open_pipeline_request(scope: &str) -> PermissionRequest {
        PermissionRequest {
            pipeline: Pipeline::Generate,
            institutional_tag: None,
            production_override_active: false,
            agent_session_id: "session-test".to_string(),
            requested_scope: scope.to_string(),
        }
    }

    // ── seed_admin_bootstrap: idempotente, primer admin por defecto ─────────

    #[tokio::test]
    async fn seed_admin_bootstrap_creates_admin_role_and_assigns_root_operator() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let identity = sample_identity();

        let (role, assignment) = seed_admin_bootstrap(&pool, &clock, &identity, "tok-owner").await.expect("sembrar admin");

        assert!(role.capability_matrix.allows(CAPABILITY_MANAGE_ROLES));
        assert!(role.capability_matrix.allows(CAPABILITY_CREATE_CHILD_ACCOUNT));
        assert_eq!(assignment.access_token_id, "tok-owner");
        assert_eq!(assignment.role_id, role.id);
    }

    #[tokio::test]
    async fn seed_admin_bootstrap_is_idempotent() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let identity = sample_identity();

        let first = seed_admin_bootstrap(&pool, &clock, &identity, "tok-owner").await.expect("primera siembra");
        clock.tick();
        let second = seed_admin_bootstrap(&pool, &clock, &identity, "tok-owner").await.expect("segunda siembra, no debe duplicar ni fallar");

        assert_eq!(first.0.id, second.0.id, "no debe crear un segundo rol Admin");
        assert_eq!(first.1.id, second.1.id, "no debe reasignar/duplicar la asignación");

        let role_repo = OperatorRoleRepository::new(&pool, &clock);
        let all_roles = role_repo.load_roles(&identity.owner_id).await.expect("cargar roles");
        assert_eq!(all_roles.len(), 1, "sembrar dos veces no debe duplicar el rol Admin");
    }

    // ── evaluate_call: gate compuesto real vía la Shell completa ─────────────

    #[tokio::test]
    async fn evaluate_call_grants_when_admin_role_allows_and_pipeline_is_open() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let identity = sample_identity();
        seed_admin_bootstrap(&pool, &clock, &identity, "tok-owner").await.expect("sembrar admin");

        // El rol ADMIN sembrado no trae "generate.run_search" -- se declara
        // aparte para ejercitar el camino de un rol NO-admin con permiso
        // puntual, más representativo del uso real.
        clock.tick();
        let mut matrix = CapabilityMatrix::new();
        matrix.set("generate.run_search", true);
        let (analyst_role, _) = define_role(&pool, &clock, &identity, "Analyst", matrix).await.expect("crear rol analyst");

        clock.tick();
        assign_operator(&pool, &clock, &identity, "tok-analyst", OperatorType::Human, &analyst_role.id)
            .await
            .expect("asignar analyst");

        let result = evaluate_call(&pool, &clock, &identity, "tok-analyst", "generate.run_search", &open_pipeline_request("generate.run_search"))
            .await
            .expect("evaluar llamada");

        assert_eq!(result.verdict, CombinedVerdict::Granted);
        assert_eq!(result.resolved_role_id, Some(analyst_role.id));
    }

    /// CRITERIO DE CIERRE: un operador (`HUMAN` o `AGENT`) SIN asignación
    /// activa siempre se deniega -- mismo camino para ambos tipos.
    #[tokio::test]
    async fn evaluate_call_denies_operator_without_any_assignment_human_and_agent() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let identity = sample_identity();
        seed_admin_bootstrap(&pool, &clock, &identity, "tok-owner").await.expect("sembrar admin");

        for token in ["tok-human-sin-rol", "tok-agent-sin-rol"] {
            let result = evaluate_call(&pool, &clock, &identity, token, "generate.run_search", &open_pipeline_request("generate.run_search"))
                .await
                .expect("evaluar llamada");
            assert!(matches!(result.verdict, CombinedVerdict::DeniedByRole { .. }), "token '{token}' -- verdict fue: {:?}", result.verdict);
            assert_eq!(result.resolved_role_id, None);
        }
    }

    // ── request_child_account: cuota real vía plan_tier_quota::PlanLimits ───

    #[tokio::test]
    async fn request_child_account_denies_non_admin_and_grants_admin_under_quota() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let identity = sample_identity();
        let (_, admin_assignment) = seed_admin_bootstrap(&pool, &clock, &identity, "tok-owner").await.expect("sembrar admin");

        clock.tick();
        let mut matrix = CapabilityMatrix::new();
        matrix.set("generate.run_search", true);
        let (analyst_role, _) = define_role(&pool, &clock, &identity, "Analyst", matrix).await.expect("crear rol analyst");
        clock.tick();
        assign_operator(&pool, &clock, &identity, "tok-analyst", OperatorType::Human, &analyst_role.id).await.expect("asignar analyst");

        let limits = PlanLimits { notional_limit: 0, max_activations: 0, max_child_accounts: 5, features_enabled: vec![] };

        let non_admin_verdict = request_child_account(&pool, &clock, &identity, "tok-analyst", 0, &limits).await.expect("evaluar");
        assert_eq!(non_admin_verdict, ChildAccountVerdict::DeniedNotAdmin);

        let admin_verdict =
            request_child_account(&pool, &clock, &identity, &admin_assignment.access_token_id, 4, &limits).await.expect("evaluar");
        assert_eq!(admin_verdict, ChildAccountVerdict::Granted);

        let at_quota_verdict =
            request_child_account(&pool, &clock, &identity, &admin_assignment.access_token_id, 5, &limits).await.expect("evaluar");
        assert_eq!(at_quota_verdict, ChildAccountVerdict::DeniedQuotaExceeded);
    }

    // ── apply_authority_override: cascada del fondo (registro local) ────────

    #[tokio::test]
    async fn apply_authority_override_reassigns_a_child_operator_and_tags_the_event() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let child_identity = OperatorRolesIdentity { owner_id: "child-1".to_string(), institutional_tag: "LIVE".to_string(), node_id: "node-child".to_string() };

        let (_, admin_assignment) = seed_admin_bootstrap(&pool, &clock, &child_identity, "tok-child-owner").await.expect("sembrar admin de la hija");

        clock.tick();
        let mut matrix = CapabilityMatrix::new();
        matrix.set("generate.run_search", true);
        let (restricted_role, _) = define_role(&pool, &clock, &child_identity, "Restricted", matrix).await.expect("crear rol restringido");

        // El fondo cambia la asignación del operador de la hija -- necesita
        // un segundo admin para no violar el invariante al mover al único.
        clock.tick();
        let mut second_admin_matrix = CapabilityMatrix::new();
        second_admin_matrix.set(CAPABILITY_MANAGE_ROLES, true);
        let (second_admin_role, _) = define_role(&pool, &clock, &child_identity, "SecondAdmin", second_admin_matrix).await.expect("crear segundo admin");
        clock.tick();
        assign_operator(&pool, &clock, &child_identity, "tok-second-admin", OperatorType::Human, &second_admin_role.id)
            .await
            .expect("asignar segundo admin");

        clock.tick();
        let (overridden, event) = apply_authority_override(
            &pool, &clock, &child_identity, &admin_assignment.access_token_id, Some(&restricted_role.id), OperatorType::Human,
        )
        .await
        .expect("aplicar cascada de autoridad");

        assert_eq!(overridden.role_id, restricted_role.id);
        assert_eq!(event.change_type, OperatorRoleChangeType::AuthorityOverride);
        assert_eq!(event.subject_ref, admin_assignment.access_token_id);
    }
}
