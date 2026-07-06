//! [CORE] Lógica pura del Third-Party API Gateway (`docs/features/
//! third-party-api-gateway.md`, ADR-0144 cimiento #8, ADR-0142, ADR-0093,
//! ADR-0137, ADR-0141, ADR-0020, STORY-035).
//!
//! Sin I/O, sin reloj de sistema, sin aleatoriedad sin semilla
//! (ADR-0002/0004). Piezas de lógica pura que la Orden STORY-035 §4 pide:
//! - [`hash_api_credential`]: hash SHA-256 de la credencial presentada --
//!   la única forma en que este módulo "conoce" un secreto es para
//!   convertirlo en un hash irreversible; nunca lo persiste ni lo compara
//!   en claro (ADR-0093).
//! - [`authenticate`]: EL punto de correctitud de seguridad de esta Story
//!   -- decide si una credencial presentada autentica contra el hash
//!   almacenado, con la revocación ganando SIEMPRE sobre un hash correcto.
//! - [`compute_rate_limit`]: la ventana de rate-limit determinista -- cuenta
//!   cuántas solicitudes YA se hicieron en la ventana vigente y decide si
//!   una más cabe.
//! - [`is_endpoint_enabled`]: pertenencia pura al conjunto configurable de
//!   endpoints habilitados de una credencial (`ENDPOINTS_ENABLED`, CONFIG).
//! - [`decide_gateway_outcome`]: EL punto de modelado crítico -- compone
//!   las cuatro puertas (autenticación, endpoint, rate-limit, consentimiento)
//!   en el [`ThirdPartyResponse`] final que la Shell persistirá y devolverá.
//!
//! ## Por qué la revocación gana sobre un hash correcto
//!
//! [`authenticate`] revisa el `status` de la credencial ANTES de comparar
//! el hash. Si se comparara el hash primero, una credencial revocada pero
//! con el secreto correcto "pasaría" la comparación de hash y solo
//! fallaría en un segundo chequeo -- funcionalmente equivalente, pero deja
//! una ventana conceptual para que un refactor futuro olvide el segundo
//! chequeo y la revocación deje de tener efecto. Revisar el estado PRIMERO
//! hace que la revocación sea la puerta de entrada, no un chequeo
//! secundario que se pueda perder.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Codifica bytes crudos a su representación hexadecimal en minúsculas
/// (mismo patrón que `consent_registry::encode_hex` / `usage_metering`).
fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ── Hash de la credencial (ADR-0093: nunca el secreto en claro) ─────────────

/// Hashea una credencial de API presentada con SHA-256 (hex, minúsculas).
///
/// Esta es la ÚNICA operación que este módulo hace con un secreto en texto
/// plano: convertirlo de inmediato en un hash irreversible. Ni
/// [`authenticate`] ni ningún otro punto de este archivo persisten ni
/// registran el argumento `secret` -- solo el resultado de esta función
/// viaja hacia la capa de persistencia (`credential_hash`, migración
/// `0014_api_gateway.sql`).
pub fn hash_api_credential(secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    encode_hex(&hasher.finalize())
}

// ── Estado de la credencial (columna `status`) ──────────────────────────────

/// Estado de una credencial de API en la tabla MUTABLE `api_credentials`
/// (`docs/features/third-party-api-gateway.md`, migración
/// `0014_api_gateway.sql`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialStatus {
    /// La credencial puede autenticar solicitudes normalmente.
    Active,
    /// La credencial fue revocada -- NINGUNA solicitud futura autentica,
    /// sin importar si el secreto presentado es correcto.
    Revoked,
}

impl CredentialStatus {
    /// Representación canónica en texto (la que persiste la columna
    /// `status` y la que acepta el `CHECK` de la migración).
    pub fn as_str(&self) -> &'static str {
        match self {
            CredentialStatus::Active => "ACTIVE",
            CredentialStatus::Revoked => "REVOKED",
        }
    }

    /// Reconstruye el estado desde el valor persistido, o `None` si no es
    /// ninguno de los dos reconocidos.
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "ACTIVE" => Some(CredentialStatus::Active),
            "REVOKED" => Some(CredentialStatus::Revoked),
            _ => None,
        }
    }
}

// ── Veredicto de autenticación ──────────────────────────────────────────────

/// Por qué [`authenticate`] negó el acceso.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthDenialReason {
    /// El secreto presentado no coincide con el hash almacenado.
    InvalidCredential,
    /// La credencial fue revocada -- gana sobre cualquier hash correcto.
    Revoked,
}

/// El veredicto de [`authenticate`]: o la credencial autentica, o queda
/// denegada con una razón explícita (nunca un booleano ciego -- quien
/// orquesta necesita saber SI fue el secreto o la revocación).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "verdict", content = "reason", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthVerdict {
    Authenticated,
    Denied(AuthDenialReason),
}

/// Autentica una credencial de API presentada contra el hash almacenado y
/// el estado de la fila (`docs/features/third-party-api-gateway.md`
/// "Comportamientos Observables": "Cuando la credencial se revoca → el
/// acceso cesa de inmediato").
///
/// ## Las dos puertas, en orden (el orden importa -- ver doc-comment del módulo)
///
/// 1. `status == Revoked` → [`AuthDenialReason::Revoked`], SIN comparar el
///    hash -- una credencial revocada nunca autentica, ni siquiera con el
///    secreto correcto.
/// 2. `hash_api_credential(presented) != stored_hash` →
///    [`AuthDenialReason::InvalidCredential`].
///
/// Si ambas puertas se superan: [`AuthVerdict::Authenticated`].
pub fn authenticate(presented: &str, stored_hash: &str, status: CredentialStatus) -> AuthVerdict {
    // Puerta 1 -- revocada gana siempre, sin importar el secreto.
    if status == CredentialStatus::Revoked {
        return AuthVerdict::Denied(AuthDenialReason::Revoked);
    }

    // Puerta 2 -- el secreto presentado debe hashear exactamente al hash
    // almacenado.
    if hash_api_credential(presented) != stored_hash {
        return AuthVerdict::Denied(AuthDenialReason::InvalidCredential);
    }

    AuthVerdict::Authenticated
}

// ── Ventana de rate-limit ────────────────────────────────────────────────────

/// El veredicto de [`compute_rate_limit`]: o la solicitud cabe en la
/// ventana, o la credencial ya agotó su cupo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimitVerdict {
    Allow,
    RateLimited,
}

/// Decide si UNA solicitud más cabe en la ventana de rate-limit vigente de
/// una credencial (`docs/features/third-party-api-gateway.md`
/// "Comportamientos Observables": "Cuando supera su límite de tasa → recibe
/// rechazo").
///
/// `requests_in_window` es el conteo de solicitudes YA PERMITIDAS
/// (`ALLOWED`) dentro de la ventana vigente, ANTES de contar la solicitud
/// actual (la Shell lo obtiene contando filas de `api_usage_records` --
/// ver `persistence::third_party_api_gateway::ApiUsageRepository::
/// count_allowed_in_window`). `limit` es `rate_limit_per_window` de la
/// credencial (`RATE_LIMIT_DEFAULT`, CONFIG).
///
/// ## Borde exacto (criterio de cierre de la Orden §5)
///
/// Con `limit = 100`: si ya hubo 99 solicitudes previas (`requests_in_window
/// = 99`), la solicitud actual sería la centésima -- exactamente en el
/// límite -- y se permite (`99 < 100`). Si ya hubo 100 (`requests_in_window
/// = 100`), la actual sería la 101ª -- una de más -- y se rechaza (`100 <
/// 100` es falso). La comparación es estrictamente `<`, no `<=`: eso es lo
/// que hace que "en el límite" permita y "uno más" rechace.
pub fn compute_rate_limit(requests_in_window: i64, limit: i64) -> RateLimitVerdict {
    if requests_in_window < limit {
        RateLimitVerdict::Allow
    } else {
        RateLimitVerdict::RateLimited
    }
}

// ── Pertenencia al conjunto de endpoints habilitados ────────────────────────

/// Comprueba si `endpoint` está dentro del conjunto configurable de
/// endpoints habilitados de una credencial (`ENDPOINTS_ENABLED`, CONFIG,
/// `docs/features/third-party-api-gateway.md` "Parámetros Configurables").
/// Comparación de igualdad de texto exacta, sin normalización.
pub fn is_endpoint_enabled(endpoint: &str, enabled_endpoints: &[String]) -> bool {
    enabled_endpoints.iter().any(|candidate| candidate == endpoint)
}

// ── Desenlace observable persistido (columna `outcome`) ─────────────────────

/// El desenlace observable de UNA solicitud procesada por el gateway
/// (`docs/features/third-party-api-gateway.md` "Comportamientos
/// Observables"), tal cual se persiste en la columna `outcome` de
/// `api_usage_records`.
///
/// Autenticación inválida, credencial revocada, endpoint no habilitado y
/// consentimiento no cubierto colapsan TODOS a `Denied` en esta columna --
/// el motivo detallado (cuál de las cuatro puertas cerró el paso) vive
/// solo en [`ThirdPartyResponse::denial_reason`], nunca en el ledger
/// persistido (la migración solo declara tres valores en el `CHECK`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GatewayOutcome {
    Allowed,
    RateLimited,
    Denied,
}

impl GatewayOutcome {
    /// Representación canónica en texto (la que persiste la columna
    /// `outcome` y la que acepta el `CHECK` de la migración).
    pub fn as_str(&self) -> &'static str {
        match self {
            GatewayOutcome::Allowed => "ALLOWED",
            GatewayOutcome::RateLimited => "RATE_LIMITED",
            GatewayOutcome::Denied => "DENIED",
        }
    }

    /// Reconstruye el desenlace desde el valor persistido, o `None` si no
    /// es ninguno de los tres reconocidos.
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "ALLOWED" => Some(GatewayOutcome::Allowed),
            "RATE_LIMITED" => Some(GatewayOutcome::RateLimited),
            "DENIED" => Some(GatewayOutcome::Denied),
            _ => None,
        }
    }
}

// ── Puertos de la Feature (ADR-0137: api_request_in / api_response_out) ────

/// El tipo de puerto `api_request_in` (ADR-0137 catálogo): una solicitud
/// externa autenticada al gateway (`docs/features/third-party-api-gateway.md`
/// "Ciclo de Vida" - "Entrada"). `credential` es el secreto CRUDO
/// presentado por el tercero -- viaja en memoria únicamente para
/// [`hash_api_credential`]/[`authenticate`]; nunca se serializa a un
/// registro persistido (ADR-0093).
#[derive(Debug, Clone)]
pub struct ThirdPartyRequest {
    pub credential: String,
    pub endpoint: String,
}

/// El tipo de puerto `api_response_out` (ADR-0137 catálogo): la respuesta
/// que el gateway devuelve al tercero (`docs/features/
/// third-party-api-gateway.md` "Ciclo de Vida" - "Salida").
///
/// **Guardarraíl ADR-0093 (estructural):** ningún campo de este struct
/// puede portar el secreto de la credencial -- el test
/// [`tests::third_party_response_json_never_leaks_the_presented_secret`]
/// fija esto sobre un caso concreto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ThirdPartyResponse {
    pub outcome: GatewayOutcome,
    /// El puerto interno al que el gateway DELEGARÍA esta solicitud --
    /// `Some(endpoint)` solo cuando `outcome == Allowed`. La delegación
    /// REAL a los puertos internos (#7 report, #9 feeds, `execute`) es
    /// futura (STORY-035 §8) -- este campo modela la DECISIÓN, no la
    /// ejecuta.
    pub delegate_to: Option<String>,
    /// Motivo legible de una respuesta no `Allowed` -- `None` cuando
    /// `outcome == Allowed`. Nunca se persiste en la columna `outcome`
    /// (que solo admite ALLOWED/RATE_LIMITED/DENIED); vive solo aquí, en
    /// la respuesta en memoria devuelta al llamador.
    pub denial_reason: Option<String>,
}

impl ThirdPartyResponse {
    /// Construye una respuesta de rechazo con el motivo dado. Azúcar para
    /// no repetir la construcción del struct en cada puerta de
    /// [`decide_gateway_outcome`].
    fn denied(reason: &str) -> Self {
        Self {
            outcome: GatewayOutcome::Denied,
            delegate_to: None,
            denial_reason: Some(reason.to_string()),
        }
    }
}

/// EL punto de modelado crítico de esta Story: compone las CUATRO puertas
/// (`docs/features/third-party-api-gateway.md` "Comportamientos
/// Observables") en la [`ThirdPartyResponse`] final.
///
/// ## Las cuatro puertas, en orden
///
/// 1. `auth` no es [`AuthVerdict::Authenticated`] → `Denied` (nunca un
///    tercero accede sin autenticación, restricción FIJA de la Feature).
/// 2. `endpoint_enabled` es `false` → `Denied` (el plan de esta credencial
///    no cubre este endpoint).
/// 3. `rate_limit` es [`RateLimitVerdict::RateLimited`] → `RateLimited`
///    (única puerta que NO colapsa a `Denied` -- tiene su propio valor de
///    `outcome`, para que el tercero sepa distinguir "sin acceso" de
///    "demasiado rápido, reintenta luego").
/// 4. `consent_covered` es `false` → `Denied` (el gateway NUNCA expone
///    datos crudos que violen consentimiento, restricción FIJA).
///
/// Si las cuatro puertas se superan: `Allowed`, con `delegate_to =
/// Some(endpoint)` -- la decisión de A QUÉ puerto interno iría, sin
/// cablear la delegación real (diferida, STORY-035 §8).
#[allow(clippy::too_many_arguments)]
pub fn decide_gateway_outcome(
    auth: AuthVerdict,
    endpoint_enabled: bool,
    rate_limit: RateLimitVerdict,
    consent_covered: bool,
    endpoint: &str,
) -> ThirdPartyResponse {
    // Puerta 1 -- autenticación. Nunca un tercero accede sin ella.
    if !matches!(auth, AuthVerdict::Authenticated) {
        return ThirdPartyResponse::denied("AUTHENTICATION_FAILED");
    }

    // Puerta 2 -- el endpoint debe estar habilitado para esta credencial.
    if !endpoint_enabled {
        return ThirdPartyResponse::denied("ENDPOINT_NOT_ENABLED");
    }

    // Puerta 3 -- rate-limit. Tiene su propio outcome, distinto de Denied.
    if matches!(rate_limit, RateLimitVerdict::RateLimited) {
        return ThirdPartyResponse {
            outcome: GatewayOutcome::RateLimited,
            delegate_to: None,
            denial_reason: Some("RATE_LIMIT_EXCEEDED".to_string()),
        };
    }

    // Puerta 4 -- consentimiento vigente. El gateway NUNCA delega sin él.
    if !consent_covered {
        return ThirdPartyResponse::denied("CONSENT_NOT_COVERED");
    }

    ThirdPartyResponse {
        outcome: GatewayOutcome::Allowed,
        delegate_to: Some(endpoint.to_string()),
        denial_reason: None,
    }
}

// ── Hash de auditoría encadenado -- api_credentials (row_version, MUTABLE) ──

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de UNA versión de fila
/// de `api_credentials`, encadenado al `audit_hash` de la versión anterior
/// (o [`crate::domain::audit_log::GENESIS_PREVIOUS_HASH`] si es
/// `row_version == 1`). Mismo patrón que
/// `central_identity::compute_account_audit_hash` -- tabla MUTABLE,
/// encadenada por `row_version`, no por `event_sequence_id`.
#[allow(clippy::too_many_arguments)]
pub fn compute_api_credential_audit_hash(
    id: &str,
    created_at_ns: i64,
    row_version: i64,
    previous_audit_hash: Option<&str>,
    owner_id: &str,
    access_token_id: Option<&str>,
    node_id: &str,
    credential_hash: &str,
    status: CredentialStatus,
    rate_limit_per_window: i64,
    window_seconds: i64,
    endpoints_enabled_json: &str,
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
    push(previous_audit_hash.unwrap_or(crate::domain::audit_log::GENESIS_PREVIOUS_HASH));
    push(owner_id);
    push(access_token_id.unwrap_or(""));
    push(node_id);
    push(credential_hash);
    push(status.as_str());
    push(&rate_limit_per_window.to_string());
    push(&window_seconds.to_string());
    push(endpoints_enabled_json);

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    encode_hex(&hasher.finalize())
}

// ── Hash de auditoría encadenado -- api_usage_records (event_sequence_id) ───

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de UNA fila de
/// `api_usage_records`, encadenado al `audit_hash` de la fila anterior en
/// la secuencia GLOBAL (o `GENESIS_PREVIOUS_HASH` si es la fila génesis).
/// Misma naturaleza que `usage_metering::compute_usage_audit_hash` --
/// tabla APPEND-ONLY, cadena GLOBAL sobre toda la tabla.
#[allow(clippy::too_many_arguments)]
pub fn compute_api_usage_audit_hash(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    previous_audit_hash: &str,
    owner_id: &str,
    access_token_id: Option<&str>,
    node_id: &str,
    credential_id: &str,
    endpoint: &str,
    outcome: GatewayOutcome,
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
    push(access_token_id.unwrap_or(""));
    push(node_id);
    push(credential_id);
    push(endpoint);
    push(outcome.as_str());

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    encode_hex(&hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── CRITERIO #2 (Orden §5): credencial hasheada, nunca en claro ─────────

    #[test]
    fn hash_api_credential_is_deterministic_and_never_equals_the_secret() {
        let hash_a = hash_api_credential("sk-demo-123");
        let hash_b = hash_api_credential("sk-demo-123");
        assert_eq!(hash_a, hash_b, "el mismo secreto debe producir el mismo hash siempre");
        assert_ne!(hash_a, "sk-demo-123", "el hash nunca debe ser igual al secreto en claro");
    }

    #[test]
    fn hash_api_credential_differs_for_different_secrets() {
        assert_ne!(hash_api_credential("sk-demo-123"), hash_api_credential("sk-demo-456"));
    }

    /// CRITERIO DE CIERRE: autenticar con la credencial correcta y estado
    /// ACTIVE autentica -- debe fallar si el hash no coincidiera.
    #[test]
    fn authenticate_succeeds_with_correct_credential_and_active_status() {
        let stored_hash = hash_api_credential("sk-demo-123");
        let verdict = authenticate("sk-demo-123", &stored_hash, CredentialStatus::Active);
        assert_eq!(verdict, AuthVerdict::Authenticated);
    }

    /// CRITERIO DE CIERRE: autenticar con una credencial incorrecta se
    /// niega con `InvalidCredential` -- debe fallar si autenticara.
    #[test]
    fn authenticate_denies_with_incorrect_credential() {
        let stored_hash = hash_api_credential("sk-demo-123");
        let verdict = authenticate("sk-wrong", &stored_hash, CredentialStatus::Active);
        assert_eq!(verdict, AuthVerdict::Denied(AuthDenialReason::InvalidCredential));
    }

    /// CRITERIO DE CIERRE (Orden §5, criterio #4): una credencial REVOCADA
    /// se niega SIEMPRE, incluso con el secreto correcto -- debe fallar si
    /// el hash correcto ganara sobre la revocación.
    #[test]
    fn authenticate_denies_revoked_credential_even_with_correct_secret() {
        let stored_hash = hash_api_credential("sk-demo-123");
        let verdict = authenticate("sk-demo-123", &stored_hash, CredentialStatus::Revoked);
        assert_eq!(verdict, AuthVerdict::Denied(AuthDenialReason::Revoked));
    }

    #[test]
    fn credential_status_round_trips_through_its_string_representation() {
        for status in [CredentialStatus::Active, CredentialStatus::Revoked] {
            assert_eq!(CredentialStatus::from_str_value(status.as_str()), Some(status));
        }
        assert_eq!(CredentialStatus::from_str_value("UNKNOWN"), None);
    }

    // ── CRITERIO #3 (Orden §5): rate-limit de borde exacto ──────────────────

    /// CRITERIO DE CIERRE: en el límite exacto (99 previas, límite 100 -->
    /// esta sería la centésima) se permite -- debe fallar si rechazara.
    #[test]
    fn compute_rate_limit_allows_at_the_exact_boundary() {
        assert_eq!(compute_rate_limit(99, 100), RateLimitVerdict::Allow);
    }

    /// CRITERIO DE CIERRE: una solicitud de más (100 previas, límite 100 --
    /// esta sería la 101ª) se rechaza -- debe fallar si el umbral se
    /// ignorara y permitiera.
    #[test]
    fn compute_rate_limit_rejects_one_past_the_boundary() {
        assert_eq!(compute_rate_limit(100, 100), RateLimitVerdict::RateLimited);
    }

    #[test]
    fn compute_rate_limit_allows_well_below_the_limit() {
        assert_eq!(compute_rate_limit(0, 100), RateLimitVerdict::Allow);
    }

    // ── Pertenencia al conjunto de endpoints habilitados ────────────────────

    #[test]
    fn is_endpoint_enabled_matches_exact_text() {
        let enabled = vec!["CERTIFY".to_string(), "FEED".to_string()];
        assert!(is_endpoint_enabled("CERTIFY", &enabled));
        assert!(!is_endpoint_enabled("ROUTE", &enabled), "un endpoint fuera del conjunto no debe habilitarse");
    }

    // ── CRITERIO #6 (Orden §5): gate de consentimiento + delegación ─────────

    /// CRITERIO DE CIERRE: las cuatro puertas superadas producen `Allowed`
    /// con la decisión de delegación -- debe fallar si negara u omitiera
    /// `delegate_to`.
    #[test]
    fn decide_gateway_outcome_allows_and_delegates_when_all_four_gates_pass() {
        let response = decide_gateway_outcome(
            AuthVerdict::Authenticated,
            true,
            RateLimitVerdict::Allow,
            true,
            "CERTIFY",
        );
        assert_eq!(response.outcome, GatewayOutcome::Allowed);
        assert_eq!(response.delegate_to, Some("CERTIFY".to_string()));
        assert_eq!(response.denial_reason, None);
    }

    /// CRITERIO DE CIERRE: autenticación fallida niega ANTES de mirar
    /// cualquier otra puerta -- debe fallar si delegara.
    #[test]
    fn decide_gateway_outcome_denies_on_failed_authentication_first() {
        let response = decide_gateway_outcome(
            AuthVerdict::Denied(AuthDenialReason::InvalidCredential),
            true,
            RateLimitVerdict::Allow,
            true,
            "CERTIFY",
        );
        assert_eq!(response.outcome, GatewayOutcome::Denied);
        assert_eq!(response.delegate_to, None);
    }

    #[test]
    fn decide_gateway_outcome_denies_when_endpoint_not_enabled() {
        let response = decide_gateway_outcome(
            AuthVerdict::Authenticated,
            false,
            RateLimitVerdict::Allow,
            true,
            "CERTIFY",
        );
        assert_eq!(response.outcome, GatewayOutcome::Denied);
        assert_eq!(response.denial_reason, Some("ENDPOINT_NOT_ENABLED".to_string()));
    }

    /// CRITERIO DE CIERRE: rate-limit excedido devuelve `RateLimited`, NO
    /// `Denied` -- son desenlaces distintos, el tercero debe poder
    /// diferenciarlos.
    #[test]
    fn decide_gateway_outcome_returns_rate_limited_not_denied() {
        let response = decide_gateway_outcome(
            AuthVerdict::Authenticated,
            true,
            RateLimitVerdict::RateLimited,
            true,
            "CERTIFY",
        );
        assert_eq!(response.outcome, GatewayOutcome::RateLimited);
        assert_ne!(response.outcome, GatewayOutcome::Denied);
    }

    /// CRITERIO DE CIERRE (Orden §5, criterio #6): consentimiento que NO
    /// cubre niega SIN delegar -- debe fallar si delegara igualmente.
    #[test]
    fn decide_gateway_outcome_denies_without_delegating_when_consent_not_covered() {
        let response = decide_gateway_outcome(
            AuthVerdict::Authenticated,
            true,
            RateLimitVerdict::Allow,
            false,
            "CERTIFY",
        );
        assert_eq!(response.outcome, GatewayOutcome::Denied);
        assert_eq!(response.delegate_to, None, "sin consentimiento, NUNCA se delega");
        assert_eq!(response.denial_reason, Some("CONSENT_NOT_COVERED".to_string()));
    }

    // ── Guardarraíl ADR-0093: el secreto nunca aparece en la respuesta ──────

    /// CRITERIO DE CIERRE (ADR-0093): el JSON serializado de
    /// `ThirdPartyResponse` nunca contiene el secreto presentado, ni en el
    /// camino feliz ni en el de rechazo.
    #[test]
    fn third_party_response_json_never_leaks_the_presented_secret() {
        let secret = "sk-super-secret-do-not-leak";
        let stored_hash = hash_api_credential(secret);
        let auth = authenticate(secret, &stored_hash, CredentialStatus::Active);

        let allowed = decide_gateway_outcome(auth, true, RateLimitVerdict::Allow, true, "CERTIFY");
        let denied = decide_gateway_outcome(
            AuthVerdict::Denied(AuthDenialReason::InvalidCredential),
            true,
            RateLimitVerdict::Allow,
            true,
            "CERTIFY",
        );

        for response in [allowed, denied] {
            let json = serde_json::to_string(&response).expect("ThirdPartyResponse debe serializar");
            assert!(!json.contains(secret), "el JSON de ThirdPartyResponse jamás debe contener el secreto");
            assert!(!json.contains(&stored_hash), "el JSON tampoco debe filtrar ni siquiera el hash almacenado");
        }
    }

    // ── Hash de auditoría encadenado -- api_credentials ─────────────────────

    #[test]
    fn compute_api_credential_audit_hash_is_deterministic() {
        let hash_a = compute_api_credential_audit_hash(
            "id-1", 1_000, 1, None, "owner-1", None, "node-1",
            "hash-abc", CredentialStatus::Active, 100, 60, r#"["CERTIFY"]"#,
        );
        let hash_b = compute_api_credential_audit_hash(
            "id-1", 1_000, 1, None, "owner-1", None, "node-1",
            "hash-abc", CredentialStatus::Active, 100, 60, r#"["CERTIFY"]"#,
        );
        assert_eq!(hash_a, hash_b);
    }

    /// CRITERIO DE CIERRE: cambiar el `status` (revocar) cambia el hash --
    /// si `status` no entrara en el hash, esta prueba fallaría con hashes
    /// iguales.
    #[test]
    fn compute_api_credential_audit_hash_changes_when_status_changes() {
        let active = compute_api_credential_audit_hash(
            "id-1", 2_000, 2, Some("prev-hash"), "owner-1", None, "node-1",
            "hash-abc", CredentialStatus::Active, 100, 60, r#"["CERTIFY"]"#,
        );
        let revoked = compute_api_credential_audit_hash(
            "id-1", 2_000, 2, Some("prev-hash"), "owner-1", None, "node-1",
            "hash-abc", CredentialStatus::Revoked, 100, 60, r#"["CERTIFY"]"#,
        );
        assert_ne!(active, revoked, "revocar debe cambiar el hash de auditoría");
    }

    // ── Hash de auditoría encadenado -- api_usage_records ───────────────────

    #[test]
    fn compute_api_usage_audit_hash_is_deterministic() {
        let hash_a = compute_api_usage_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", None, "node-1",
            "cred-1", "CERTIFY", GatewayOutcome::Allowed,
        );
        let hash_b = compute_api_usage_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", None, "node-1",
            "cred-1", "CERTIFY", GatewayOutcome::Allowed,
        );
        assert_eq!(hash_a, hash_b);
    }

    /// CRITERIO DE CIERRE: cambiar el `outcome` cambia el hash -- si el
    /// campo no entrara en el hash, esta prueba fallaría con hashes
    /// iguales.
    #[test]
    fn compute_api_usage_audit_hash_changes_when_outcome_changes() {
        let allowed = compute_api_usage_audit_hash(
            "id-1", 2_000, 2, "prev-hash", "owner-1", None, "node-1",
            "cred-1", "CERTIFY", GatewayOutcome::Allowed,
        );
        let denied = compute_api_usage_audit_hash(
            "id-1", 2_000, 2, "prev-hash", "owner-1", None, "node-1",
            "cred-1", "CERTIFY", GatewayOutcome::Denied,
        );
        assert_ne!(allowed, denied, "cambiar el outcome debe cambiar el hash de auditoría");
    }

    #[test]
    fn gateway_outcome_round_trips_through_its_string_representation() {
        for outcome in [GatewayOutcome::Allowed, GatewayOutcome::RateLimited, GatewayOutcome::Denied] {
            assert_eq!(GatewayOutcome::from_str_value(outcome.as_str()), Some(outcome));
        }
        assert_eq!(GatewayOutcome::from_str_value("UNKNOWN"), None);
    }
}
