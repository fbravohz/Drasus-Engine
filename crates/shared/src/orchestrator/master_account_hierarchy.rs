//! [SHELL] Composición del cimiento #12 (`docs/features/master-account-hierarchy.md`,
//! ADR-0147, ADR-0093, ADR-0141, STORY-040).
//!
//! Capa delgada sobre [`crate::persistence::master_account_hierarchy`]:
//! traduce las operaciones que el resto del substrato necesita -- "vincula
//! esta hija a su fondo", "el fondo emite un override" y "la hija lo
//! recibe y lo ejecuta (o lo rechaza) localmente" -- sin que el llamador
//! tenga que conocer los repositorios ni el esquema de las tablas. Mismo
//! rol que `orchestrator::verified_account_registry::attest_track_record`
//! para el cimiento #10.
//!
//! ## Por qué `issue_override` y `receive_override` resuelven el consentimiento cada una por su cuenta
//!
//! Regla fija #6 (ADR-0147): "la hija conserva su Plano de Control -- esta
//! capa va encima, no reemplaza". La hija NUNCA confía ciegamente en el
//! desenlace que el fondo declaró en su propia fila ISSUER -- vuelve a
//! resolver el `ConsentVerdict` REAL contra `consent-registry` (#5) por su
//! cuenta antes de decidir el efecto local. En este harness de un solo
//! proceso ambas resoluciones comparten el mismo `pool` (misma fila de
//! `consent_records`), pero la separación de llamadas documenta que, en
//! producción, cada lado la resuelve contra SU PROPIA base de datos local.

use sqlx::SqlitePool;

use crate::domain::clock::Clock;
use crate::domain::master_account_hierarchy::{
    apply_local_command_effect, decide_override_authorization, AttestationSide, LocalEffect,
    OverrideCommandKind, OverrideOutcome, OverrideOutcomeLabel,
};
use crate::orchestrator::consent_registry::resolve_consent_verdict;
use crate::persistence::consent_registry::ConsentRepositoryError;
use crate::persistence::master_account_hierarchy::{
    AccountHierarchyRepository, AccountHierarchyRepositoryError, AccountHierarchyRow, NewAccountHierarchy,
    OverrideAttestationRepository, OverrideAttestationRepositoryError, OverrideAttestationRow,
    RecordOverrideAttestationInput,
};

/// Tipo de dato consultado en `consent-registry` (#5) para el gate de
/// override de esta feature -- mismo vocabulario que
/// `verified_account_registry::VERIFIED_ACCOUNT_PUBLICATION_CONSENT_DATA_TYPE`,
/// aplicado al mando elevado de un fondo sobre una hija en vez de a la
/// publicación de una cuenta.
pub const MASTER_ACCOUNT_OVERRIDE_CONSENT_DATA_TYPE: &str = "master_account_override";

/// Error de orquestación de esta feature -- envuelve los tres puntos de
/// fallo posibles: persistir/actualizar la jerarquía, resolver el
/// consentimiento (I/O contra `consent_records`) y persistir una
/// atestación.
#[derive(Debug, thiserror::Error)]
pub enum MasterAccountHierarchyError {
    #[error("error al registrar/actualizar la jerarquía de cuenta: {0}")]
    Hierarchy(#[from] AccountHierarchyRepositoryError),
    #[error("error al resolver el veredicto de consentimiento: {0}")]
    Consent(#[from] ConsentRepositoryError),
    #[error("error al registrar la atestación de override: {0}")]
    Attestation(#[from] OverrideAttestationRepositoryError),
}

/// Vincula `owner_id` (la hija) a `parent_owner_id` (el fondo, o `None` si
/// todavía no tiene padre) -- delgado a propósito: existe como punto de
/// orquestación estable para que `public_interface` no dependa
/// directamente del repositorio.
pub async fn link_child_to_parent(
    pool: &SqlitePool,
    clock: &dyn Clock,
    owner_id: &str,
    parent_owner_id: Option<&str>,
    consent_ref: &str,
    node_id: &str,
) -> Result<AccountHierarchyRow, MasterAccountHierarchyError> {
    let repo = AccountHierarchyRepository::new(pool, clock);
    Ok(repo
        .link_child(NewAccountHierarchy {
            owner_id: owner_id.to_string(),
            parent_owner_id: parent_owner_id.map(str::to_string),
            consent_ref: consent_ref.to_string(),
            node_id: node_id.to_string(),
        })
        .await?)
}

/// Lado FONDO de un override: resuelve el `ConsentVerdict` REAL de la hija
/// contra `consent-registry` (#5), decide el desenlace
/// ([`decide_override_authorization`]) y encadena la fila ISSUER -- SIEMPRE,
/// tanto si el desenlace es `Executed` como `Denied` (regla fija #4: nunca
/// una mutación silenciosa, un intento denegado también se atesta). El
/// mando cifrado en sí para el relé genérico (ADR-0143) es responsabilidad
/// del adaptador de red diferido -- esta función solo produce el desenlace
/// y la fila ISSUER, no transmite nada.
#[allow(clippy::too_many_arguments)]
pub async fn issue_override(
    pool: &SqlitePool,
    clock: &dyn Clock,
    parent_owner_id: &str,
    child_owner_id: &str,
    issuer_node_id: &str,
    command_kind: OverrideCommandKind,
    target_ref: &str,
    justification: Option<&str>,
    consent_version: &str,
) -> Result<(OverrideAttestationRow, OverrideOutcome), MasterAccountHierarchyError> {
    let verdict = resolve_consent_verdict(
        pool,
        clock,
        child_owner_id,
        MASTER_ACCOUNT_OVERRIDE_CONSENT_DATA_TYPE,
        consent_version,
    )
    .await?;
    let outcome = decide_override_authorization(&verdict);

    let repo = OverrideAttestationRepository::new(pool, clock);
    let row = repo
        .record_attestation(RecordOverrideAttestationInput {
            owner_id: child_owner_id.to_string(),
            parent_owner_id: parent_owner_id.to_string(),
            node_id: issuer_node_id.to_string(),
            attestation_side: AttestationSide::Issuer,
            command_kind,
            target_ref: target_ref.to_string(),
            outcome: OverrideOutcomeLabel::from(&outcome),
            justification: justification.map(str::to_string),
        })
        .await?;

    Ok((row, outcome))
}

/// Lado HIJA de un override: re-valida el consentimiento LOCALMENTE (regla
/// fija #6 -- nunca confía ciegamente en lo que el fondo declaró), aplica
/// el efecto local ("eliminar" = archivar, [`apply_local_command_effect`])
/// y encadena la fila EXECUTOR -- SIEMPRE, tanto si ejecutó como si
/// rechazó.
#[allow(clippy::too_many_arguments)]
pub async fn receive_override(
    pool: &SqlitePool,
    clock: &dyn Clock,
    parent_owner_id: &str,
    child_owner_id: &str,
    executor_node_id: &str,
    command_kind: OverrideCommandKind,
    target_ref: &str,
    justification: Option<&str>,
    consent_version: &str,
) -> Result<(OverrideAttestationRow, OverrideOutcome, LocalEffect), MasterAccountHierarchyError> {
    let verdict = resolve_consent_verdict(
        pool,
        clock,
        child_owner_id,
        MASTER_ACCOUNT_OVERRIDE_CONSENT_DATA_TYPE,
        consent_version,
    )
    .await?;
    let outcome = decide_override_authorization(&verdict);
    let local_effect = apply_local_command_effect(command_kind, &outcome);

    let repo = OverrideAttestationRepository::new(pool, clock);
    let row = repo
        .record_attestation(RecordOverrideAttestationInput {
            owner_id: child_owner_id.to_string(),
            parent_owner_id: parent_owner_id.to_string(),
            node_id: executor_node_id.to_string(),
            attestation_side: AttestationSide::Executor,
            command_kind,
            target_ref: target_ref.to_string(),
            outcome: OverrideOutcomeLabel::from(&outcome),
            justification: justification.map(str::to_string),
        })
        .await?;

    Ok((row, outcome, local_effect))
}

/// Resultado completo de ejecutar un comando de override extremo a
/// extremo (emisión + recepción) -- ambas atestaciones ISSUER/EXECUTOR
/// encadenadas más el desenlace de autorización y el efecto local que las
/// gobernó a las dos.
#[derive(Debug, Clone)]
pub struct OverrideExecutionResult {
    pub issuer: OverrideAttestationRow,
    pub executor: OverrideAttestationRow,
    pub outcome: OverrideOutcome,
    pub local_effect: LocalEffect,
}

/// Composición completa: emite el comando desde el fondo Y lo ejecuta
/// localmente en la hija -- recorre el camino end-to-end que el harness
/// CLI y los tests de integración ejercitan. El adaptador de red real del
/// relé genérico (ADR-0143) queda diferido -- en producción, `issue_override`
/// corre en la máquina del fondo y `receive_override` en la de la hija,
/// comunicadas por ese adaptador; aquí ambas comparten `pool` a propósito,
/// para poder demostrar el camino completo sin él.
#[allow(clippy::too_many_arguments)]
pub async fn execute_override(
    pool: &SqlitePool,
    clock: &dyn Clock,
    parent_owner_id: &str,
    child_owner_id: &str,
    issuer_node_id: &str,
    executor_node_id: &str,
    command_kind: OverrideCommandKind,
    target_ref: &str,
    justification: Option<&str>,
    consent_version: &str,
) -> Result<OverrideExecutionResult, MasterAccountHierarchyError> {
    let (issuer, outcome) = issue_override(
        pool,
        clock,
        parent_owner_id,
        child_owner_id,
        issuer_node_id,
        command_kind,
        target_ref,
        justification,
        consent_version,
    )
    .await?;

    let (executor, _executor_outcome, local_effect) = receive_override(
        pool,
        clock,
        parent_owner_id,
        child_owner_id,
        executor_node_id,
        command_kind,
        target_ref,
        justification,
        consent_version,
    )
    .await?;

    Ok(OverrideExecutionResult { issuer, executor, outcome, local_effect })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::domain::consent_registry::ConsentAction;
    use crate::orchestrator::consent_registry::record_consent_action;
    use crate::persistence::consent_registry::RecordConsentActionInput;
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    /// Siembra un consentimiento REAL cubierto (`ACCEPT`, sin opt-out del
    /// tipo de dato de esta feature) para `owner_id` -- nunca un stub.
    async fn seed_covered_consent(pool: &SqlitePool, clock: &dyn Clock, owner_id: &str, version: &str) {
        let mut optout_changes = std::collections::BTreeMap::new();
        optout_changes.insert(MASTER_ACCOUNT_OVERRIDE_CONSENT_DATA_TYPE.to_string(), false);
        record_consent_action(
            pool,
            clock,
            RecordConsentActionInput {
                owner_id: owner_id.to_string(),
                institutional_tag: "DRASUS_LOCAL".to_string(),
                node_id: "node-consent".to_string(),
                compliance_status_id: None,
                action: ConsentAction::Accept,
                tos_version: Some(version.to_string()),
                optout_changes,
            },
        )
        .await
        .expect("registrar consentimiento");
    }

    // ── link_child_to_parent: delgado, refleja el repositorio ───────────────

    #[tokio::test]
    async fn link_child_to_parent_registers_row_version_one() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);

        let row = link_child_to_parent(&pool, &clock, "trader-7", Some("fund-X"), "v1", "node-A")
            .await
            .expect("vincular hija");
        assert_eq!(row.row_version, 1);
        assert_eq!(row.parent_owner_id.as_deref(), Some("fund-X"));
    }

    // ── CRITERIO: gate de consentimiento denegado, end-to-end ────────────────

    /// CRITERIO DE CIERRE: sin NINGÚN evento de consentimiento registrado
    /// para la hija, el `consent_out` REAL de #5 resuelve `NotCovered
    /// (NoConsent)` -- el override se deniega en AMBOS lados y ambas filas
    /// quedan con `outcome = DENIED` (intento denegado atestado, nunca
    /// descartado en silencio).
    #[tokio::test]
    async fn execute_override_denies_both_sides_without_any_real_consent() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);

        let result = execute_override(
            &pool,
            &clock,
            "fund-X",
            "trader-7",
            "node-fund",
            "node-child",
            OverrideCommandKind::Archive,
            "strategy-42",
            Some("riesgo excedido"),
            "v1",
        )
        .await
        .expect("la resolución debe tener éxito");

        assert!(matches!(result.outcome, OverrideOutcome::Denied(_)), "sin consentimiento vigente, debe denegar");
        assert_eq!(result.issuer.outcome, OverrideOutcomeLabel::Denied);
        assert_eq!(result.executor.outcome, OverrideOutcomeLabel::Denied);
        assert_eq!(result.local_effect, LocalEffect::NoEffect, "un ARCHIVE denegado no debe archivar nada");
    }

    // ── CRITERIO: doble atestación, end-to-end ───────────────────────────────

    /// CRITERIO DE CIERRE: un override `Executed` produce EXACTAMENTE una
    /// fila ISSUER y una fila EXECUTOR, ambas con `audit_chain_hash`
    /// encadenado correcto (la EXECUTOR encadena sobre la ISSUER, mismo
    /// ledger GLOBAL).
    #[tokio::test]
    async fn execute_override_produces_exactly_one_issuer_and_one_executor_row_when_executed() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        seed_covered_consent(&pool, &clock, "trader-7", "v1").await;

        let result = execute_override(
            &pool,
            &clock,
            "fund-X",
            "trader-7",
            "node-fund",
            "node-child",
            OverrideCommandKind::Archive,
            "strategy-42",
            Some("riesgo excedido"),
            "v1",
        )
        .await
        .expect("la resolución debe tener éxito");

        assert_eq!(result.outcome, OverrideOutcome::Executed);
        assert_eq!(result.issuer.attestation_side, AttestationSide::Issuer);
        assert_eq!(result.executor.attestation_side, AttestationSide::Executor);
        assert_eq!(result.issuer.outcome, OverrideOutcomeLabel::Executed);
        assert_eq!(result.executor.outcome, OverrideOutcomeLabel::Executed);
        assert_ne!(result.issuer.event_sequence_id, result.executor.event_sequence_id, "deben ser DOS filas distintas");
        assert_eq!(
            result.executor.audit_chain_hash.as_deref(),
            Some(result.issuer.audit_hash.as_str()),
            "la fila EXECUTOR debe encadenar sobre la fila ISSUER en el mismo ledger global"
        );

        let repo = OverrideAttestationRepository::new(&pool, &clock);
        let chain = repo.load_chain().await.expect("cargar cadena");
        assert_eq!(chain.len(), 2, "exactamente dos filas: una ISSUER, una EXECUTOR");
    }

    // ── CRITERIO: "eliminar" = archivar, end-to-end ──────────────────────────

    /// CRITERIO DE CIERRE: un `ARCHIVE` ejecutado deja el efecto local
    /// `Archived` (transición de estado) y las DOS filas de atestación
    /// (issuer + executor) permanecen -- ningún DELETE, ni siquiera sobre
    /// la propia fila ISSUER tras registrar la EXECUTOR.
    #[tokio::test]
    async fn archive_command_archives_and_never_deletes_the_chain() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        seed_covered_consent(&pool, &clock, "trader-7", "v1").await;

        let result = execute_override(
            &pool,
            &clock,
            "fund-X",
            "trader-7",
            "node-fund",
            "node-child",
            OverrideCommandKind::Archive,
            "strategy-42",
            None,
            "v1",
        )
        .await
        .expect("la resolución debe tener éxito");

        assert_eq!(result.local_effect, LocalEffect::Archived);

        let repo = OverrideAttestationRepository::new(&pool, &clock);
        let chain = repo.load_chain().await.expect("cargar cadena");
        assert_eq!(chain.len(), 2, "la fila ISSUER original sigue intacta -- nunca se borra al registrar la EXECUTOR");
        assert_eq!(chain[0], result.issuer);
        assert_eq!(chain[1], result.executor);
    }

    /// Contraprueba: `MODIFY`/`REQUEST_AUDIT_REPORT` nunca producen
    /// `Archived`, aunque el gate ejecute.
    #[tokio::test]
    async fn non_archive_commands_never_produce_a_local_archive_effect() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        seed_covered_consent(&pool, &clock, "trader-7", "v1").await;

        let result = execute_override(
            &pool,
            &clock,
            "fund-X",
            "trader-7",
            "node-fund",
            "node-child",
            OverrideCommandKind::RequestAuditReport,
            "strategy-42",
            None,
            "v1",
        )
        .await
        .expect("la resolución debe tener éxito");

        assert_eq!(result.outcome, OverrideOutcome::Executed);
        assert_eq!(result.local_effect, LocalEffect::NoEffect, "REQUEST_AUDIT_REPORT nunca archiva");
    }
}
