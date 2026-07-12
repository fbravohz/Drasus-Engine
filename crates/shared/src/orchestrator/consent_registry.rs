//! [SHELL] Composición del puerto `consent_out` para el Registro de
//! Consentimiento / ToS (`docs/features/consent-registry.md`, ADR-0143,
//! ADR-0144, STORY-031).
//!
//! Capa delgada sobre [`crate::persistence::consent_registry::ConsentRepository`]:
//! traduce las dos operaciones que el resto del substrato necesita --
//! "registra este evento de consentimiento" y "¿este tipo de dato está
//! cubierto para este usuario, ahora mismo?" -- sin que el llamador tenga
//! que conocer el repositorio ni el esquema de la tabla. Es el mismo rol
//! que cumple `orchestrator::usage_metering::record_metered_operation`
//! para el cimiento #4: la composición completa detrás de UNA función.

use sqlx::SqlitePool;

use crate::domain::clock::Clock;
use crate::domain::consent_registry::{resolve_coverage, ConsentState, ConsentVerdict};
use crate::persistence::consent_registry::{
    ConsentRecordRow, ConsentRepository, ConsentRepositoryError, RecordConsentActionInput,
};

/// Registra UN evento de consentimiento (aceptar/re-aceptar una versión de
/// ToS, o cambiar uno o más opt-outs) para `input.owner_id`.
///
/// Delgado a propósito: solo instancia el repositorio y delega. Existe
/// como función de orquestación (en vez de que el llamador use
/// [`ConsentRepository`] directamente) para que `public_interface` tenga
/// UN punto de entrada estable, igual que
/// `orchestrator::usage_metering::record_metered_operation`.
pub async fn record_consent_action(
    pool: &SqlitePool,
    clock: &dyn Clock,
    input: RecordConsentActionInput,
) -> Result<ConsentRecordRow, ConsentRepositoryError> {
    let repo = ConsentRepository::new(pool, clock);
    repo.record_action(input).await
}

/// Resuelve el puerto `consent_out` -> [`ConsentVerdict`] para
/// `(owner_id, data_type)` contra la versión de ToS vigente
/// (`current_version`, que el llamador -- `data-aggregation`, el firehose
/// -- ya conoce por su propia configuración `TOS_VERSION_ACTUAL`).
///
/// Carga el estado vigente del dueño ([`ConsentRepository::load_latest_for_owner`])
/// y delega la decisión legal al Core puro ([`resolve_coverage`]) -- esta
/// función no toma NINGUNA decisión de cobertura por sí misma, solo trae
/// el dato y pregunta.
pub async fn resolve_consent_verdict(
    pool: &SqlitePool,
    clock: &dyn Clock,
    owner_id: &str,
    data_type: &str,
    current_version: &str,
) -> Result<ConsentVerdict, ConsentRepositoryError> {
    let repo = ConsentRepository::new(pool, clock);
    let latest = repo.load_latest_for_owner(owner_id).await?;

    let state = latest.map(|row| ConsentState {
        accepted_version: row.tos_version,
        optout_map: row.optout_map,
    });

    Ok(resolve_coverage(state.as_ref(), data_type, current_version))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::domain::consent_registry::{ConsentAction, NotCoveredReason};
    use crate::persistence::central_identity::test_support::seed_account;
    use crate::persistence::pool::{connect, migrate};
    use std::collections::BTreeMap;

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    /// Sin ningún evento de consentimiento registrado, el puerto niega por
    /// default -- ejercitando la composición completa (repo -> Core), no
    /// solo la función pura.
    #[tokio::test]
    async fn resolve_consent_verdict_denies_by_default_without_any_event() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);

        let verdict = resolve_consent_verdict(&pool, &clock, "owner-sin-consentimiento", "aggregation", "v2")
            .await
            .expect("la consulta debe tener éxito");

        assert_eq!(verdict, ConsentVerdict::NotCovered(NotCoveredReason::NoConsent));
    }

    /// Tras registrar una aceptación de la versión vigente sin opt-out del
    /// tipo consultado, el puerto responde `Covered` -- camino feliz
    /// completo: registrar -> resolver.
    #[tokio::test]
    async fn record_then_resolve_covers_when_version_matches_and_no_optout() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;

        let mut optout_changes = BTreeMap::new();
        optout_changes.insert("aggregation".to_string(), false);

        record_consent_action(
            &pool,
            &clock,
            RecordConsentActionInput {
                owner_id: owner_id.clone(),
                institutional_tag: "DRASUS_LOCAL".to_string(),
                node_id: "node-1".to_string(),
                compliance_status_id: None,
                action: ConsentAction::Accept,
                tos_version: Some("v2".to_string()),
                optout_changes,
            },
        )
        .await
        .expect("registrar aceptación");

        let verdict = resolve_consent_verdict(&pool, &clock, &owner_id, "aggregation", "v2")
            .await
            .expect("la consulta debe tener éxito");

        assert_eq!(verdict, ConsentVerdict::Covered);
    }
}
