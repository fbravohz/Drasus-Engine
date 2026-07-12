//! [SHELL] Composición del flujo completo del Registro de Cuentas
//! Verificadas (`docs/features/verified-account-registry.md`, ADR-0145
//! cimiento #10 -- rector, ADR-0093, ADR-0141, ADR-0020, STORY-037).
//!
//! Orquesta el flujo descrito en la Orden STORY-037 §4: registrar cuenta
//! (default PRIVATE) -> agrupar eventos de #6 por cuenta -> calcular track
//! por ámbito -> firmar -> gate de publicación con consentimiento REAL de
//! #5 (sin opt-in vigente, NUNCA publica) -> persistir. Mismo rol que
//! `orchestrator::data_aggregation::run_aggregation` para el cimiento #9:
//! la composición completa detrás de funciones pequeñas, para que
//! `public_interface` tenga puntos de entrada estables.

use sqlx::SqlitePool;

use crate::domain::clock::Clock;
use crate::domain::enriched_domain_events::EnrichedDomainEvent;
use crate::domain::verified_account_registry::{
    compute_track_record, compute_track_record_signature, decide_publication, AttestationScope,
    CapitalReality, PublicationStatus,
};
use crate::orchestrator::consent_registry::resolve_consent_verdict;
use crate::persistence::consent_registry::ConsentRepositoryError;
use crate::persistence::verified_account_registry::{
    AttestedTrackRecordRepository, AttestedTrackRecordRepositoryError, AttestedTrackRecordRow,
    NewVerifiedAccount, RecordTrackRecordInput, VerifiedAccountRepository,
    VerifiedAccountRepositoryError, VerifiedAccountRow,
};

/// Tipo de dato consultado en `consent-registry` (#5) para el gate de
/// publicación de esta feature -- mismo vocabulario que
/// `data_aggregation::DATA_AGGREGATION_CONSENT_DATA_TYPE`, aplicado a la
/// publicación de una cuenta verificada en vez de a la agregación.
pub const VERIFIED_ACCOUNT_PUBLICATION_CONSENT_DATA_TYPE: &str = "verified_account_publication";

/// Error de orquestación de esta feature -- envuelve los tres puntos de
/// fallo posibles: persistir/actualizar la cuenta, resolver el
/// consentimiento (I/O contra `consent_records`) y persistir el track
/// atestado.
#[derive(Debug, thiserror::Error)]
pub enum VerifiedAccountRegistryError {
    #[error("error al registrar/actualizar la cuenta verificada: {0}")]
    Account(#[from] VerifiedAccountRepositoryError),
    #[error("error al resolver el veredicto de consentimiento: {0}")]
    Consent(#[from] ConsentRepositoryError),
    #[error("error al persistir el track record atestado: {0}")]
    TrackRecord(#[from] AttestedTrackRecordRepositoryError),
    /// El `institutional_tag` de una cuenta ya persistida no parsea como
    /// [`CapitalReality`] (Eje B) -- no debería ocurrir si el `CHECK` de la
    /// migración se respeta, pero se valida aquí también en vez de asumir
    /// un default silencioso (STORY-041/DEBT-016).
    #[error("institutional_tag (Eje B) desconocido en la cuenta verificada: '{0}'")]
    UnknownInstitutionalTag(String),
}

/// Registra una cuenta verificada nueva -- delgado a propósito: el default
/// `PRIVATE` (regla obligatoria #4) ya es estructural en
/// [`NewVerifiedAccount`] (no tiene campo `publication_status`), así que
/// esta función solo delega al repositorio. Existe como punto de
/// orquestación estable para que `public_interface` no dependa
/// directamente del repositorio.
pub async fn register_account(
    pool: &SqlitePool,
    clock: &dyn Clock,
    new_account: NewVerifiedAccount,
) -> Result<VerifiedAccountRow, VerifiedAccountRegistryError> {
    let repo = VerifiedAccountRepository::new(pool, clock);
    Ok(repo.create(new_account).await?)
}

/// Calcula el track record de `account` para el ámbito `scope` a partir de
/// `events` (ya filtrados/agrupados por quien llama a la cuenta que le
/// corresponde -- [`compute_track_record`] también filtra internamente por
/// `account.id`, así que pasar eventos de otras cuentas es inofensivo, solo
/// se ignoran), firma el contenido de forma reproducible y lo persiste
/// append-only atómico.
///
/// Esta función NUNCA decide si el track se publica -- eso es
/// [`request_publication`], una puerta completamente separada (regla
/// obligatoria #4: publicar es un acto explícito y posterior, no un efecto
/// secundario de calcular el track).
pub async fn attest_track_record(
    pool: &SqlitePool,
    clock: &dyn Clock,
    account: &VerifiedAccountRow,
    scope: AttestationScope,
    time_window: &str,
    events: &[EnrichedDomainEvent],
) -> Result<AttestedTrackRecordRow, VerifiedAccountRegistryError> {
    let metrics = compute_track_record(events, &account.id);
    // Eje B: se copia SIEMPRE de la cuenta -- la fuente de verdad es
    // `account.institutional_tag` (STORY-041: en esta tabla ES el Eje B),
    // nunca un parámetro que el llamador pudiera mislabelar (Orden
    // STORY-038 §4 punto 11, consolidada por STORY-041).
    let capital_reality = CapitalReality::from_str_value(&account.institutional_tag)
        .ok_or_else(|| VerifiedAccountRegistryError::UnknownInstitutionalTag(account.institutional_tag.clone()))?;
    let signature_hash =
        compute_track_record_signature(&metrics, scope, capital_reality, &account.id, time_window);

    let repo = AttestedTrackRecordRepository::new(pool, clock);
    let row = repo
        .record_track_record(RecordTrackRecordInput {
            owner_id: account.owner_id.clone(),
            institutional_tag: account.institutional_tag.clone(),
            node_id: account.node_id.clone(),
            verified_account_id: account.id.clone(),
            scope,
            time_window: time_window.to_string(),
            metrics,
            signature_hash,
        })
        .await?;

    Ok(row)
}

/// Resuelve el gate de publicación con el `consent_out` REAL de
/// `consent-registry` (#5, `resolve_consent_verdict`, NUNCA un stub) y
/// actualiza `publication_status` de `account` si corresponde.
///
/// Sin opt-in vigente para `VERIFIED_ACCOUNT_PUBLICATION_CONSENT_DATA_TYPE`,
/// [`decide_publication`] devuelve el estado ACTUAL sin cambios -- en ese
/// caso esta función NO llama al repositorio (evita bumps de `row_version`
/// sin cambio real) y devuelve `account` tal cual. Solo cuando el estado
/// resultante difiere del actual se persiste la actualización.
pub async fn request_publication(
    pool: &SqlitePool,
    clock: &dyn Clock,
    account: &VerifiedAccountRow,
    requested_status: PublicationStatus,
    consent_version: &str,
) -> Result<VerifiedAccountRow, VerifiedAccountRegistryError> {
    let verdict = resolve_consent_verdict(
        pool,
        clock,
        &account.owner_id,
        VERIFIED_ACCOUNT_PUBLICATION_CONSENT_DATA_TYPE,
        consent_version,
    )
    .await?;

    let new_status = decide_publication(account.publication_status, requested_status, &verdict);

    if new_status == account.publication_status {
        // Nada cambió (o el gate lo negó): no se toca la fila -- ni el
        // estado avanza sin consentimiento, ni se desperdicia una versión.
        return Ok(account.clone());
    }

    let repo = VerifiedAccountRepository::new(pool, clock);
    let updated = repo
        .update_publication_and_scopes(account, new_status, &account.attestation_scopes)
        .await?;
    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::domain::consent_registry::ConsentAction;
    use crate::domain::enriched_domain_events::{
        AccountSnapshotPayload, CapitalFlowPayload, CapitalFlowSign, OrderExecutedPayload, OrderSide,
    };
    use crate::domain::verified_account_registry::{AccountType, NS_PER_DAY};
    use crate::orchestrator::consent_registry::record_consent_action;
    use crate::persistence::central_identity::test_support::seed_account;
    use crate::persistence::consent_registry::RecordConsentActionInput;
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    fn sample_new_account(owner_id: &str) -> NewVerifiedAccount {
        sample_new_account_with_capital_reality(owner_id, CapitalReality::Live)
    }

    /// Igual que [`sample_new_account`], pero con el Eje B configurable --
    /// necesario para el test discriminante `SOVEREIGN`+`PAPER`. Escribe
    /// `capital_reality` en `institutional_tag` (STORY-041: en esta tabla
    /// ES el Eje B, ya no acepta el placeholder genérico "DRASUS_LOCAL").
    fn sample_new_account_with_capital_reality(owner_id: &str, capital_reality: CapitalReality) -> NewVerifiedAccount {
        NewVerifiedAccount {
            owner_id: owner_id.to_string(),
            institutional_tag: capital_reality.as_str().to_string(),
            node_id: "node-1".to_string(),
            broker: "ICMarkets".to_string(),
            leverage: 100,
            currency: "USD".to_string(),
            account_type: AccountType::Own,
            attestation_scopes: vec![AttestationScope::Sovereign],
            broker_connection_ref: None,
        }
    }

    // ── register_account: default PRIVATE estructural ───────────────────────

    #[tokio::test]
    async fn register_account_always_starts_private() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;

        let account = register_account(&pool, &clock, sample_new_account(&owner_id)).await.expect("registrar");
        assert_eq!(account.publication_status, PublicationStatus::Private);
        assert_eq!(account.row_version, 1);
    }

    // ── CRITERIO #2 (Orden §5): gain% excluye flujo de capital, end-to-end ──

    #[tokio::test]
    async fn attest_track_record_computes_gain_pct_excluding_capital_flow_end_to_end() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;

        let account = register_account(&pool, &clock, sample_new_account(&owner_id)).await.expect("registrar");

        let events = vec![
            EnrichedDomainEvent::AccountSnapshot(AccountSnapshotPayload {
                account_id: account.id.clone(),
                equity: 1_000_000_000_000,
                balance: 1_000_000_000_000,
                margin_available: 1_000_000_000_000,
                margin_required: 0,
                timestamp_ns: 0,
            }),
            EnrichedDomainEvent::OrderExecuted(OrderExecutedPayload {
                instrument_id: "BTCUSDT".to_string(),
                side: OrderSide::Buy,
                quantity: 100_000_000,
                price: 100_000_000_000,
                slippage: 0,
                fill_time_ns: NS_PER_DAY,
                broker: "ICMarkets".to_string(),
                notional: 100_000_000_000,
                account_id: account.id.clone(),
                realized_pnl: 4_410_000_000_000,
                mae: 0,
                mfe: 0,
                duration_ns: 3_600_000_000_000,
            }),
            EnrichedDomainEvent::CapitalFlow(CapitalFlowPayload {
                account_id: account.id.clone(),
                sign: CapitalFlowSign::Deposit,
                amount: 35_000_000_000,
                currency: "USD".to_string(),
                timestamp_ns: 0,
            }),
            EnrichedDomainEvent::CapitalFlow(CapitalFlowPayload {
                account_id: account.id.clone(),
                sign: CapitalFlowSign::Withdrawal,
                amount: 47_698_000_000,
                currency: "USD".to_string(),
                timestamp_ns: 0,
            }),
        ];

        let track = attest_track_record(&pool, &clock, &account, AttestationScope::Sovereign, "2026-W27", &events)
            .await
            .expect("calcular y persistir el track");

        assert_eq!(track.metrics.gain_pct_e8, 441_000_000, "441% -- el flujo de capital no debe alterar el gain%");
        assert_eq!(track.metrics.total_deposits_e8, 35_000_000_000);
        assert_eq!(track.metrics.total_withdrawals_e8, 47_698_000_000);
        assert_eq!(track.scope, AttestationScope::Sovereign);
        assert!(!track.signature_hash.is_empty());
    }

    // ── CRITERIO #3 (Orden §5): ámbito inviolable, end-to-end ───────────────

    #[tokio::test]
    async fn sovereign_and_broker_readonly_tracks_of_the_same_account_stay_distinct() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let account = register_account(&pool, &clock, sample_new_account(&owner_id)).await.expect("registrar");

        let events = vec![EnrichedDomainEvent::OrderExecuted(OrderExecutedPayload {
            instrument_id: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            quantity: 100_000_000,
            price: 100_000_000_000,
            slippage: 0,
            fill_time_ns: 0,
            broker: "ICMarkets".to_string(),
            notional: 100_000_000_000,
            account_id: account.id.clone(),
            realized_pnl: 100_000_000,
            mae: 0,
            mfe: 0,
            duration_ns: 1_000,
        })];

        let sovereign = attest_track_record(&pool, &clock, &account, AttestationScope::Sovereign, "2026-W27", &events)
            .await
            .expect("track soberano");
        let readonly = attest_track_record(&pool, &clock, &account, AttestationScope::BrokerReadonly, "2026-W27", &events)
            .await
            .expect("track read-only");

        assert_ne!(sovereign.scope, readonly.scope, "los ámbitos deben quedar distintos");
        assert_ne!(
            sovereign.signature_hash, readonly.signature_hash,
            "el ámbito debe formar parte de la firma -- nunca colisionan"
        );

        use crate::domain::verified_account_registry::AttestedTrackRecord;
        assert!(AttestedTrackRecord::from(&sovereign).is_attested_by_drasus);
        assert!(!AttestedTrackRecord::from(&readonly).is_attested_by_drasus);
    }

    // ── EL punto de DEBT-014 (Orden §4 punto 14): SOVEREIGN + PAPER, end-to-end ──

    /// CRITERIO DE CIERRE: recorre el camino COMPLETO (registrar cuenta
    /// PAPER -> calcular y firmar el track SOVEREIGN -> persistir) y
    /// demuestra que los dos ejes son ORTOGONALES: el track queda atestado
    /// (Eje A = SOVEREIGN, `is_attested_by_drasus = true`) porque el motor
    /// de Drasus lo ejecutó en el mismo entorno determinista que producción,
    /// pero `is_real_capital = false` y `capital_reality = "PAPER"` porque
    /// arriesgó capital virtual -- jamás se presenta como si fuera LIVE.
    #[tokio::test]
    async fn sovereign_paper_account_is_attested_but_never_presented_as_real_capital() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let account = register_account(&pool, &clock, sample_new_account_with_capital_reality(&owner_id, CapitalReality::Paper))
            .await
            .expect("registrar cuenta PAPER");
        assert_eq!(
            account.institutional_tag,
            CapitalReality::Paper.as_str(),
            "el Eje B vive en institutional_tag (STORY-041)"
        );

        let events = vec![EnrichedDomainEvent::OrderExecuted(OrderExecutedPayload {
            instrument_id: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            quantity: 100_000_000,
            price: 100_000_000_000,
            slippage: 0,
            fill_time_ns: 0,
            broker: "ICMarkets".to_string(),
            notional: 100_000_000_000,
            account_id: account.id.clone(),
            realized_pnl: 100_000_000,
            mae: 0,
            mfe: 0,
            duration_ns: 1_000,
        })];

        let track = attest_track_record(&pool, &clock, &account, AttestationScope::Sovereign, "2026-W27", &events)
            .await
            .expect("track soberano sobre cuenta PAPER");
        assert_eq!(
            track.institutional_tag,
            CapitalReality::Paper.as_str(),
            "el orquestador debe estampar el Eje B desde la cuenta"
        );

        use crate::domain::verified_account_registry::AttestedTrackRecord;
        let projected = AttestedTrackRecord::from(&track);
        assert!(projected.is_attested_by_drasus, "SOVEREIGN debe seguir atestado -- el motor de Drasus lo ejecutó");
        assert!(!projected.is_real_capital, "PAPER nunca es capital real -- los ejes son ortogonales");
        assert_eq!(projected.capital_reality, "PAPER", "jamás se presenta sin la etiqueta de capital virtual");
    }

    // ── CRITERIO #5 (Orden §5): publicación opt-in con consentimiento REAL ──

    #[tokio::test]
    async fn request_publication_denies_without_any_real_consent() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let account = register_account(&pool, &clock, sample_new_account(&owner_id)).await.expect("registrar");
        assert_eq!(account.publication_status, PublicationStatus::Private);

        // Ningún evento de consentimiento registrado -- el consent_out REAL
        // de #5 resuelve NotCovered(NoConsent), NUNCA un stub que cubra por
        // defecto.
        let result = request_publication(&pool, &clock, &account, PublicationStatus::Public, "v1")
            .await
            .expect("la resolución debe tener éxito");

        assert_eq!(result.publication_status, PublicationStatus::Private, "sin opt-in vigente, NUNCA debe publicar");
        assert_eq!(result.row_version, account.row_version, "sin cambio real, no debe bumpear row_version");
    }

    #[tokio::test]
    async fn request_publication_publishes_with_real_covered_consent() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let account = register_account(&pool, &clock, sample_new_account(&owner_id)).await.expect("registrar");

        // Registra el opt-in REAL vía consent-registry (#5) -- no un stub.
        let mut optout_changes = std::collections::BTreeMap::new();
        optout_changes.insert(VERIFIED_ACCOUNT_PUBLICATION_CONSENT_DATA_TYPE.to_string(), false);
        record_consent_action(
            &pool,
            &clock,
            RecordConsentActionInput {
                owner_id: account.owner_id.clone(),
                institutional_tag: "DRASUS_LOCAL".to_string(),
                node_id: "node-1".to_string(),
                compliance_status_id: None,
                action: ConsentAction::Accept,
                tos_version: Some("v1".to_string()),
                optout_changes,
            },
        )
        .await
        .expect("registrar consentimiento");

        let result = request_publication(&pool, &clock, &account, PublicationStatus::Public, "v1")
            .await
            .expect("la resolución debe tener éxito");

        assert_eq!(result.publication_status, PublicationStatus::Public, "con opt-in vigente real, debe publicar");
        assert_eq!(result.row_version, 2, "el cambio real debe persistirse con row_version incrementado");
    }
}
