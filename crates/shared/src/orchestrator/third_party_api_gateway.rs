//! [SHELL] Composición del flujo completo del Third-Party API Gateway
//! (`docs/features/third-party-api-gateway.md`, ADR-0144 cimiento #8,
//! ADR-0142, ADR-0093, ADR-0137, ADR-0141, ADR-0020, STORY-035).
//!
//! Capa delgada que compone, EN ORDEN, las piezas del Core
//! ([`crate::domain::third_party_api_gateway`]) con las dos fuentes de
//! I/O: los dos repositorios de este cimiento
//! ([`crate::persistence::third_party_api_gateway`]) y el puerto
//! `consent_out` REAL del cimiento #5
//! ([`crate::orchestrator::consent_registry::resolve_consent_verdict`]) --
//! el mismo patrón que `orchestrator::usage_metering::record_metered_operation`
//! resolviendo el `PlanLimits` REAL de `plan_tier_quota` (#3): esta
//! composición NUNCA usa un stub de consentimiento, siempre el veredicto
//! real contra `consent_records`.
//!
//! [`handle_gateway_request`] es EL punto de entrada de esta Feature: dada
//! una credencial presentada en claro (nunca persistida así) y un
//! endpoint, autentica, limita la tasa, verifica consentimiento, decide
//! delegar (o no) y registra la solicitud -- exactamente el "Ciclo de
//! Vida" que describe `docs/features/third-party-api-gateway.md`.

use sqlx::SqlitePool;

use crate::domain::clock::Clock;
use crate::domain::third_party_api_gateway::{
    authenticate, compute_rate_limit, decide_gateway_outcome, hash_api_credential,
    is_endpoint_enabled, AuthVerdict, RateLimitVerdict, ThirdPartyResponse,
};
use crate::orchestrator::consent_registry::resolve_consent_verdict;
use crate::persistence::consent_registry::ConsentRepositoryError;
use crate::persistence::third_party_api_gateway::{
    ApiCredentialRepository, ApiCredentialRepositoryError, ApiUsageRepository,
    ApiUsageRepositoryError, RecordApiUsageInput,
};

/// El `data_type` con el que este cimiento consulta el puerto `consent_out`
/// de `consent-registry` (#5, ADR-0143) -- toda delegación del gateway pasa
/// SIEMPRE por este mismo tipo de dato, sea cual sea el `endpoint`
/// invocado: lo que el consentimiento gobierna aquí es "¿este dueño acepta
/// que Drasus exponga sus capacidades internas a terceros vía API?", no un
/// matiz por endpoint.
pub const API_GATEWAY_CONSENT_DATA_TYPE: &str = "third_party_api_gateway";

/// Errores que puede devolver [`handle_gateway_request`] -- envuelve los
/// tres puntos de I/O que la composición atraviesa.
#[derive(Debug, thiserror::Error)]
pub enum HandleGatewayRequestError {
    #[error("error del repositorio de credenciales: {0}")]
    Credential(#[from] ApiCredentialRepositoryError),
    #[error("error del repositorio de uso: {0}")]
    Usage(#[from] ApiUsageRepositoryError),
    #[error("error al resolver el consentimiento: {0}")]
    Consent(#[from] ConsentRepositoryError),
}

/// Procesa UNA solicitud externa de punta a punta (`docs/features/
/// third-party-api-gateway.md` "Ciclo de Vida"):
///
/// 1. **Autenticación:** hashea `presented_secret`
///    ([`hash_api_credential`]) y busca la credencial por ese hash. Sin
///    coincidencia -> `Denied` de inmediato, SIN persistir ningún uso (no
///    hay `credential_id` al que atribuirlo).
/// 2. **Endpoint + rate-limit + consentimiento:** SOLO si autenticó, se
///    consulta si el endpoint está habilitado, se cuenta el uso previo en
///    la ventana vigente ([`ApiUsageRepository::count_allowed_in_window`])
///    y se resuelve el consentimiento REAL de #5 -- si la autenticación ya
///    falló, estas tres lecturas se OMITEN (no hay razón para gastarlas si
///    [`decide_gateway_outcome`] va a negar en la primera puerta de
///    cualquier forma).
/// 3. **Decisión + registro:** [`decide_gateway_outcome`] compone las
///    cuatro puertas; el resultado se persiste SIEMPRE (con el
///    `credential_id` ya conocido) vía [`ApiUsageRepository::record_usage`].
pub async fn handle_gateway_request(
    pool: &SqlitePool,
    clock: &dyn Clock,
    presented_secret: &str,
    endpoint: &str,
    current_consent_version: &str,
) -> Result<ThirdPartyResponse, HandleGatewayRequestError> {
    let credential_repo = ApiCredentialRepository::new(pool, clock);
    let usage_repo = ApiUsageRepository::new(pool, clock);

    // Paso 1 -- autenticación por hash. Sin coincidencia, no hay
    // credential_id al que atribuir un registro de uso: se responde de
    // inmediato sin tocar `api_usage_records`.
    let presented_hash = hash_api_credential(presented_secret);
    let credential = match credential_repo.find_by_credential_hash(&presented_hash).await? {
        Some(credential) => credential,
        None => {
            return Ok(ThirdPartyResponse {
                outcome: crate::domain::third_party_api_gateway::GatewayOutcome::Denied,
                delegate_to: None,
                denial_reason: Some("INVALID_CREDENTIAL".to_string()),
            })
        }
    };

    let auth = authenticate(presented_secret, &credential.credential_hash, credential.status);

    // Paso 2 -- endpoint + rate-limit + consentimiento, SOLO si autenticó.
    let (endpoint_enabled, rate_limit_verdict, consent_covered) = if matches!(auth, AuthVerdict::Authenticated) {
        let endpoint_enabled = is_endpoint_enabled(endpoint, &credential.endpoints_enabled);

        let now_ns = clock.timestamp_ns();
        let window_ns = credential.window_seconds.saturating_mul(1_000_000_000);
        let window_start_ns = now_ns.saturating_sub(window_ns);
        let prior_count = usage_repo
            .count_allowed_in_window(&credential.id, window_start_ns)
            .await?;
        let rate_limit_verdict = compute_rate_limit(prior_count, credential.rate_limit_per_window);

        // consent_out REAL de #5 -- NUNCA un stub (misma exigencia que #6
        // consumiendo el ExecutionGate real de #2).
        let consent_verdict = resolve_consent_verdict(
            pool,
            clock,
            &credential.owner_id,
            API_GATEWAY_CONSENT_DATA_TYPE,
            current_consent_version,
        )
        .await?;

        (endpoint_enabled, rate_limit_verdict, consent_verdict.is_covered())
    } else {
        // La autenticación ya falló -- estos valores no afectan el
        // resultado (decide_gateway_outcome niega en la puerta 1 de
        // cualquier forma), pero deben ser valores válidos del tipo.
        (false, RateLimitVerdict::Allow, false)
    };

    let response = decide_gateway_outcome(auth, endpoint_enabled, rate_limit_verdict, consent_covered, endpoint);

    // Paso 3 -- el registro de uso se persiste SIEMPRE que hubo una
    // credencial identificada, sin importar el desenlace (auditar
    // rechazos es tan importante como auditar aceptaciones).
    usage_repo
        .record_usage(RecordApiUsageInput {
            owner_id: credential.owner_id.clone(),
            access_token_id: credential.access_token_id.clone(),
            node_id: credential.node_id.clone(),
            credential_id: credential.id.clone(),
            endpoint: endpoint.to_string(),
            outcome: response.outcome,
        })
        .await?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::domain::third_party_api_gateway::GatewayOutcome;
    use crate::orchestrator::consent_registry::record_consent_action;
    use crate::persistence::central_identity::test_support::seed_account;
    use crate::persistence::consent_registry::RecordConsentActionInput;
    use crate::persistence::pool::{connect, migrate};
    use crate::persistence::third_party_api_gateway::{ApiUsageRepository, NewApiCredential};
    use crate::domain::consent_registry::ConsentAction;

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    /// Crea una credencial ACTIVA con el secreto dado, límites generosos y
    /// `endpoint` habilitado -- estado base para los tests del camino feliz.
    async fn seed_credential(
        pool: &SqlitePool,
        clock: &dyn Clock,
        owner_id: &str,
        secret: &str,
        rate_limit_per_window: i64,
        endpoints_enabled: &[&str],
    ) -> crate::persistence::third_party_api_gateway::ApiCredentialRow {
        let repo = ApiCredentialRepository::new(pool, clock);
        repo.create(NewApiCredential {
            owner_id: owner_id.to_string(),
            access_token_id: None,
            node_id: "node-1".to_string(),
            credential_hash: crate::domain::third_party_api_gateway::hash_api_credential(secret),
            rate_limit_per_window,
            window_seconds: 60,
            endpoints_enabled: endpoints_enabled.iter().map(|s| s.to_string()).collect(),
        })
        .await
        .expect("crear credencial de prueba")
    }

    /// Registra la aceptación de ToS que cubre `API_GATEWAY_CONSENT_DATA_TYPE`
    /// para `owner_id`, con la MISMA versión que se usará al resolver.
    async fn accept_gateway_consent(pool: &SqlitePool, clock: &dyn Clock, owner_id: &str, version: &str) {
        let mut optout_changes = std::collections::BTreeMap::new();
        optout_changes.insert(API_GATEWAY_CONSENT_DATA_TYPE.to_string(), false);
        record_consent_action(
            pool,
            clock,
            RecordConsentActionInput {
                owner_id: owner_id.to_string(),
                institutional_tag: "DRASUS_LOCAL".to_string(),
                node_id: "node-1".to_string(),
                compliance_status_id: None,
                action: ConsentAction::Accept,
                tos_version: Some(version.to_string()),
                optout_changes,
            },
        )
        .await
        .expect("registrar consentimiento de prueba");
    }

    // ── Camino feliz: autentica + endpoint habilitado + dentro de cupo + consentimiento cubre ──

    #[tokio::test]
    async fn handle_gateway_request_allows_and_delegates_when_everything_checks_out() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;

        seed_credential(&pool, &clock, &owner_id, "sk-demo-123", 100, &["CERTIFY"]).await;
        accept_gateway_consent(&pool, &clock, &owner_id, "v2").await;

        let response = handle_gateway_request(&pool, &clock, "sk-demo-123", "CERTIFY", "v2")
            .await
            .expect("la solicitud debe procesarse sin error");

        assert_eq!(response.outcome, GatewayOutcome::Allowed);
        assert_eq!(response.delegate_to, Some("CERTIFY".to_string()));

        // El registro de uso quedó persistido con el desenlace correcto.
        let usage_repo = ApiUsageRepository::new(&pool, &clock);
        let chain = usage_repo.load_chain().await.expect("cargar cadena de uso");
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].outcome, GatewayOutcome::Allowed);
    }

    // ── CRITERIO #6 (Orden §5): gate de consentimiento REAL, no un stub ─────

    /// CRITERIO DE CIERRE: sin NINGÚN evento de consentimiento registrado
    /// para el dueño, la solicitud se niega SIN delegar -- usando el
    /// veredicto REAL de `consent-registry::resolve_coverage` (default
    /// niega, GDPR), no un stub que siempre cubriera.
    #[tokio::test]
    async fn handle_gateway_request_denies_without_delegating_when_consent_not_covered() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;

        seed_credential(&pool, &clock, &owner_id, "sk-demo-123", 100, &["CERTIFY"]).await;
        // Sin accept_gateway_consent: el dueño no tiene ningún consentimiento.

        let response = handle_gateway_request(&pool, &clock, "sk-demo-123", "CERTIFY", "v2")
            .await
            .expect("la solicitud debe procesarse sin error");

        assert_eq!(response.outcome, GatewayOutcome::Denied);
        assert_eq!(response.delegate_to, None, "sin consentimiento, NUNCA se delega");

        let usage_repo = ApiUsageRepository::new(&pool, &clock);
        let chain = usage_repo.load_chain().await.expect("cargar cadena de uso");
        assert_eq!(chain[0].outcome, GatewayOutcome::Denied);
    }

    /// El mismo veredicto real también niega por versión de ToS obsoleta
    /// (`StaleVersion`) -- consentimiento aceptado contra una versión
    /// vieja no cubre nada hasta re-aceptar.
    #[tokio::test]
    async fn handle_gateway_request_denies_when_consent_version_is_stale() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;

        seed_credential(&pool, &clock, &owner_id, "sk-demo-123", 100, &["CERTIFY"]).await;
        accept_gateway_consent(&pool, &clock, &owner_id, "v1").await; // acepta v1

        // Se resuelve contra v2 (vigente) -- v1 quedó obsoleta.
        let response = handle_gateway_request(&pool, &clock, "sk-demo-123", "CERTIFY", "v2")
            .await
            .expect("la solicitud debe procesarse sin error");

        assert_eq!(response.outcome, GatewayOutcome::Denied);
    }

    // ── Autenticación ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn handle_gateway_request_denies_unknown_credential_without_persisting_usage() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);

        let response = handle_gateway_request(&pool, &clock, "sk-nunca-emitida", "CERTIFY", "v2")
            .await
            .expect("la solicitud debe procesarse sin error");

        assert_eq!(response.outcome, GatewayOutcome::Denied);

        let usage_repo = ApiUsageRepository::new(&pool, &clock);
        let chain = usage_repo.load_chain().await.expect("cargar cadena de uso");
        assert!(chain.is_empty(), "una credencial desconocida no tiene credential_id al que atribuir uso");
    }

    /// CRITERIO DE CIERRE (Orden §5, criterio #4): revocar la credencial
    /// niega la SIGUIENTE autenticación, incluso con el secreto correcto y
    /// consentimiento cubierto.
    #[tokio::test]
    async fn handle_gateway_request_denies_after_revocation_even_with_correct_secret() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;

        let credential = seed_credential(&pool, &clock, &owner_id, "sk-demo-123", 100, &["CERTIFY"]).await;
        accept_gateway_consent(&pool, &clock, &owner_id, "v2").await;

        let credential_repo = ApiCredentialRepository::new(&pool, &clock);
        credential_repo.revoke(&credential).await.expect("revocar credencial");

        let response = handle_gateway_request(&pool, &clock, "sk-demo-123", "CERTIFY", "v2")
            .await
            .expect("la solicitud debe procesarse sin error");

        assert_eq!(response.outcome, GatewayOutcome::Denied);
    }

    // ── Endpoint no habilitado ────────────────────────────────────────────

    #[tokio::test]
    async fn handle_gateway_request_denies_when_endpoint_not_enabled() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;

        seed_credential(&pool, &clock, &owner_id, "sk-demo-123", 100, &["FEED"]).await;
        accept_gateway_consent(&pool, &clock, &owner_id, "v2").await;

        let response = handle_gateway_request(&pool, &clock, "sk-demo-123", "CERTIFY", "v2")
            .await
            .expect("la solicitud debe procesarse sin error");

        assert_eq!(response.outcome, GatewayOutcome::Denied);
    }

    // ── Rate-limit ────────────────────────────────────────────────────────

    /// CRITERIO DE CIERRE (Orden §5, criterio #3): con el cupo ya agotado
    /// (`rate_limit_per_window` solicitudes ALLOWED previas ya registradas
    /// en la ventana vigente), la siguiente solicitud se rechaza con
    /// `RateLimited`, NO `Denied`.
    #[tokio::test]
    async fn handle_gateway_request_rate_limits_after_quota_exhausted_in_window() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;

        let credential = seed_credential(&pool, &clock, &owner_id, "sk-demo-123", 2, &["CERTIFY"]).await;
        accept_gateway_consent(&pool, &clock, &owner_id, "v2").await;

        // Siembra 2 solicitudes ALLOWED previas dentro de la ventana --
        // exactamente el cupo (rate_limit_per_window = 2).
        let usage_repo = ApiUsageRepository::new(&pool, &clock);
        for _ in 0..2 {
            usage_repo
                .record_usage(RecordApiUsageInput {
                    owner_id: credential.owner_id.clone(),
                    access_token_id: credential.access_token_id.clone(),
                    node_id: credential.node_id.clone(),
                    credential_id: credential.id.clone(),
                    endpoint: "CERTIFY".to_string(),
                    outcome: GatewayOutcome::Allowed,
                })
                .await
                .expect("sembrar uso previo");
        }

        // La 3ª solicitud (prior_count = 2 >= limit 2) debe ser RATE_LIMITED.
        let response = handle_gateway_request(&pool, &clock, "sk-demo-123", "CERTIFY", "v2")
            .await
            .expect("la solicitud debe procesarse sin error");

        assert_eq!(response.outcome, GatewayOutcome::RateLimited);
        assert_ne!(response.outcome, GatewayOutcome::Denied, "rate-limit y denegación son desenlaces distintos");
    }
}
