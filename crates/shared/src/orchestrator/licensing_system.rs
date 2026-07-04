//! [SHELL] Emisor de licencias de desarrollo (stub) + proveedor de límites de
//! plan (stub) + caché con TTL del veredicto de ejecución
//! (`docs/features/licensing-system.md`, ADR-0143, ADR-0144, STORY-028).
//!
//! ## El emisor real y su stub (ADR-0144: "puerto ahora, adaptador después")
//!
//! La Cabina de Mando Central del proveedor (ADR-0143) todavía no existe --
//! nadie firma licencias reales todavía. [`LocalStubLicenseIssuer`] genera
//! un par de claves Ed25519 **de desarrollo** (con aleatoriedad real del
//! sistema operativo -- por eso vive aquí, en la Shell, y NO en
//! `domain::licensing_system`, que debe permanecer puro) y firma licencias
//! de prueba con esa clave. El día que la Cabina de Mando exista, firmará
//! con SU clave privada (que nunca sale del servidor) y el cliente seguirá
//! verificando exactamente igual, con [`crate::domain::licensing_system::verify_license_signature`]
//! -- ninguna otra pieza del sistema cambia.
//!
//! ## `PlanLimits` (puerto `plan_limits_in`, stub hasta `plan-tier-quota`)
//!
//! [`LocalStubPlanLimitsProvider`] fija los límites de `ACTIVATIONS_PER_TIER`
//! (licensing-system.md "Parámetros Configurables": Explorer 1, Sovereign 3)
//! hasta que la feature `plan-tier-quota` (cimiento #3, aún no construida)
//! provea el adaptador real.
//!
//! ## Caché del veredicto con TTL (hot-path, ADR-0039)
//!
//! [`ExecutionGateCache`] es EXACTAMENTE el mismo patrón que
//! [`crate::orchestrator::central_identity::IdentityCache`]: guarda el
//! último [`ExecutionGate`] evaluado junto al instante (del reloj inyectado)
//! en que se guardó. Mientras no expire el TTL, `get()` es una lectura en
//! memoria -- sin tocar disco ni red -- que es lo único que el hot-path de
//! `execute`/`telemetry` puede permitirse llamar (ADR-0039: prohibida la
//! llamada de red síncrona en el hot-path). El refresco (releer la BD,
//! recontar activaciones, decidir el nuevo veredicto) ocurre FUERA del
//! hot-path, en la tarea asíncrona que llama a [`build_execution_gate`].

use std::sync::{Arc, Mutex as StdMutex};

use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use sqlx::SqlitePool;

use crate::domain::clock::Clock;
use crate::domain::licensing_system::{
    canonical_license_bytes, derive_execution_gate, evaluate_heartbeat_status,
    hardware_matches, heartbeat_status_to_compliance_status_id, verify_license_signature,
    ExecutionGate, GateEvaluationInput, HeartbeatConfig, LicensePayload, LicenseSignatureError,
    LicenseTier, PlanLimits,
};
use crate::persistence::licensing_system::{LicenseRecord, LicenseRepository, LicenseRepositoryError};

// ── Emisor de licencias de desarrollo (Shell -- genera claves con azar real) ──

/// Par de claves Ed25519 de un emisor de desarrollo. La clave privada
/// (`signing_key`) **nunca** se serializa ni se expone fuera de este struct
/// -- solo [`LocalStubLicenseIssuer::issue_license`] la usa, internamente,
/// para firmar. Lo único que sale de aquí hacia el cliente es la clave
/// PÚBLICA (`verifying_key_hex`) y las firmas ya calculadas (ADR-0093 §3).
struct DevKeypair {
    signing_key: SigningKey,
    verifying_key_hex: String,
}

impl DevKeypair {
    /// Genera un par de claves nuevo usando el generador criptográfico del
    /// sistema operativo (`OsRng`) -- la ÚNICA operación de este módulo que
    /// necesita aleatoriedad real, y por eso vive en la Shell, no en
    /// `domain` (ADR-0002/0004: el Core no tiene azar sin semilla).
    fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key_hex = signing_key
            .verifying_key()
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        Self { signing_key, verifying_key_hex }
    }
}

/// Un archivo de licencia ya firmado -- lo que el usuario "importa"
/// (licensing-system.md "Ciclo de Vida" - "Entrada": "Archivo de licencia
/// firmado criptográficamente").
#[derive(Debug, Clone)]
pub struct SignedLicenseFile {
    pub license_id: String,
    pub owner_id: String,
    pub node_id: String,
    pub tier: LicenseTier,
    pub issued_at_ns: i64,
    pub heartbeat_expires_at_ns: i64,
    /// Firma Ed25519 (hex) -- dato público verificable.
    pub signature_hex: String,
    /// Clave pública (hex) incrustada para verificar -- NO la privada
    /// (ADR-0093 §3).
    pub public_key_hex: String,
}

/// Solicitud para emitir una licencia de desarrollo.
#[derive(Debug, Clone)]
pub struct IssueLicenseRequest {
    pub owner_id: String,
    pub node_id: String,
    pub tier: LicenseTier,
    pub issued_at_ns: i64,
    pub heartbeat_expires_at_ns: i64,
}

/// Emisor de licencias de desarrollo (stub local -- ADR-0144: "puerto
/// ahora, adaptador después"). La Cabina de Mando Central real firmará con
/// su propia clave privada, que jamás llega al cliente; este stub simula
/// exactamente ese contrato observable para poder probar el resto del
/// sistema sin depender de un servidor que todavía no existe.
pub struct LocalStubLicenseIssuer {
    keypair: DevKeypair,
}

impl LocalStubLicenseIssuer {
    /// Crea un emisor nuevo con un par de claves Ed25519 fresco. Cada
    /// instancia de este struct tiene su PROPIA clave -- dos instancias
    /// nunca comparten emisor, igual que dos despliegues de la Cabina de
    /// Mando real tendrían cada uno la suya.
    pub fn new() -> Self {
        Self { keypair: DevKeypair::generate() }
    }

    /// La clave PÚBLICA de este emisor -- la que se "incrusta en el
    /// cliente" para poder verificar. Nunca hay un método equivalente para
    /// la privada.
    pub fn public_key_hex(&self) -> &str {
        &self.keypair.verifying_key_hex
    }

    /// Firma una licencia de desarrollo nueva y devuelve el archivo listo
    /// para "importar". Usa [`canonical_license_bytes`] (la MISMA función
    /// pura que [`verify_license_signature`] usa para reconstruir el
    /// mensaje) -- firmar y verificar comparten un único punto de verdad
    /// sobre qué bytes se firman.
    pub fn issue_license(&self, request: IssueLicenseRequest) -> SignedLicenseFile {
        let license_id = uuid::Uuid::now_v7().to_string();
        let payload = LicensePayload {
            license_id: &license_id,
            owner_id: &request.owner_id,
            node_id: &request.node_id,
            tier: request.tier,
            issued_at_ns: request.issued_at_ns,
            heartbeat_expires_at_ns: request.heartbeat_expires_at_ns,
        };
        let message = canonical_license_bytes(&payload);
        // Ed25519: firmar es determinista dado (clave privada, mensaje) --
        // no requiere azar en el momento de firmar, solo la clave ya lo tuvo
        // al generarse.
        let signature = self.keypair.signing_key.sign(&message);
        let signature_hex = signature.to_bytes().iter().map(|byte| format!("{byte:02x}")).collect();

        SignedLicenseFile {
            license_id,
            owner_id: request.owner_id,
            node_id: request.node_id,
            tier: request.tier,
            issued_at_ns: request.issued_at_ns,
            heartbeat_expires_at_ns: request.heartbeat_expires_at_ns,
            signature_hex,
            public_key_hex: self.keypair.verifying_key_hex.clone(),
        }
    }
}

impl Default for LocalStubLicenseIssuer {
    fn default() -> Self {
        Self::new()
    }
}

// ── Proveedor de límites de plan (puerto `plan_limits_in`, stub) ────────────

/// Proveedor de [`PlanLimits`] vigentes para un dueño y tier -- el puerto
/// `plan_limits_in` de la Feature. **Cualquier** `impl` puede sustituir a
/// [`LocalStubPlanLimitsProvider`] sin tocar el resto del sistema (mismo
/// patrón que `CentralIdentityVerifier`).
#[async_trait::async_trait]
pub trait PlanLimitsProvider: Send + Sync {
    /// Devuelve los límites vigentes del plan de `owner_id` para `tier`.
    async fn plan_limits_for(&self, owner_id: &str, tier: LicenseTier) -> PlanLimits;
}

/// Implementación stub local: fija `ACTIVATIONS_PER_TIER` por defecto
/// (licensing-system.md: Explorer 1, Sovereign 3) sin consultar ningún
/// catálogo real de planes. `plan-tier-quota` (cimiento #3, aún no
/// construido) reemplazará esto con límites configurables de verdad.
pub struct LocalStubPlanLimitsProvider {
    pub sovereign_max_activations: i64,
    pub explorer_max_activations: i64,
}

impl Default for LocalStubPlanLimitsProvider {
    /// Defaults declarados por la Feature: Explorer = 1, Sovereign = 3
    /// (típico: 1 laptop personal + 2 nodos VPS headless).
    fn default() -> Self {
        Self { sovereign_max_activations: 3, explorer_max_activations: 1 }
    }
}

#[async_trait::async_trait]
impl PlanLimitsProvider for LocalStubPlanLimitsProvider {
    async fn plan_limits_for(&self, _owner_id: &str, tier: LicenseTier) -> PlanLimits {
        let max_activations = match tier {
            LicenseTier::Sovereign => self.sovereign_max_activations,
            LicenseTier::Explorer => self.explorer_max_activations,
        };
        PlanLimits { max_activations, features_enabled: vec![] }
    }
}

// ── Caché del veredicto con TTL (hot-path, ADR-0039) ────────────────────────

/// Configuración de la caché del veredicto de ejecución.
#[derive(Debug, Clone, Copy)]
pub struct ExecutionGateCacheConfig {
    /// Cuánto tiempo (ns) vale el último veredicto antes de exigir un
    /// recálculo. Deliberadamente corto comparado con `IDENTITY_CACHE_TTL`
    /// -- el veredicto de licencia debe reflejar cambios de heartbeat/gracia
    /// con más frecuencia que la identidad de cuenta.
    pub ttl_ns: i64,
}

impl Default for ExecutionGateCacheConfig {
    fn default() -> Self {
        const NANOS_PER_MINUTE: i64 = 60 * 1_000_000_000;
        Self { ttl_ns: 5 * NANOS_PER_MINUTE }
    }
}

struct CachedGate {
    gate: ExecutionGate,
    cached_at_ns: i64,
}

/// Caché local del último [`ExecutionGate`] evaluado, en memoria del
/// proceso -- mismo patrón que
/// [`crate::orchestrator::central_identity::IdentityCache`]. Es la ÚNICA
/// superficie que el hot-path de `execute`/`telemetry` debe consultar
/// (ADR-0039): [`Self::get`] no toca disco ni red, solo compara el reloj
/// inyectado contra el instante en que se guardó.
pub struct ExecutionGateCache {
    clock: Arc<dyn Clock>,
    config: ExecutionGateCacheConfig,
    entry: StdMutex<Option<CachedGate>>,
}

impl ExecutionGateCache {
    pub fn new(clock: Arc<dyn Clock>, config: ExecutionGateCacheConfig) -> Self {
        Self { clock, config, entry: StdMutex::new(None) }
    }

    /// Devuelve el veredicto cacheado si sigue dentro del TTL, o `None` si
    /// expiró o nunca se guardó nada -- en ambos casos quien llama debe
    /// disparar (fuera del hot-path) un recálculo vía [`build_execution_gate`].
    pub fn get(&self) -> Option<ExecutionGate> {
        let now_ns = self.clock.timestamp_ns();
        let guard = self.entry.lock().expect("mutex de caché de gate envenenado");

        match guard.as_ref() {
            Some(cached) if now_ns - cached.cached_at_ns < self.config.ttl_ns => Some(cached.gate.clone()),
            _ => None,
        }
    }

    /// Guarda `gate` como vigente a partir de "ahora". Sobrescribe
    /// cualquier entrada previa.
    pub fn set(&self, gate: ExecutionGate) {
        let now_ns = self.clock.timestamp_ns();
        let mut guard = self.entry.lock().expect("mutex de caché de gate envenenado");
        *guard = Some(CachedGate { gate, cached_at_ns: now_ns });
    }
}

// ── Composición: construir el ExecutionGate (fuera del hot-path) ───────────

/// Errores al construir un [`ExecutionGate`] contra la BD y el emisor.
#[derive(Debug, thiserror::Error)]
pub enum BuildExecutionGateError {
    #[error("error de persistencia de licencia: {0}")]
    Database(#[from] LicenseRepositoryError),
}

/// Construye el [`ExecutionGate`] vigente para una licencia ya activada,
/// combinando: verificación de firma (Core, dado el archivo firmado),
/// comparación de huella contra `identity.node_id` (Core, SIN recalcular),
/// heartbeat/gracia contra el reloj inyectado (Core), y el conteo de
/// activaciones distintas ya persistidas (Shell, lectura SQLite local --
/// NO es la llamada de red que ADR-0039 prohíbe en el hot-path; esta
/// función se llama para REFRESCAR la caché, no en el hot-path mismo).
///
/// Quien llama es responsable de, después, guardar el resultado en un
/// [`ExecutionGateCache`] -- eso es lo que el hot-path real consulta.
#[allow(clippy::too_many_arguments)]
pub async fn build_execution_gate(
    pool: &SqlitePool,
    clock: &dyn Clock,
    identity_node_id: &str,
    license: &LicenseRecord,
    signature_hex: &str,
    public_key_hex: &str,
    heartbeat_config: &HeartbeatConfig,
    plan_limits: &PlanLimits,
) -> Result<ExecutionGate, BuildExecutionGateError> {
    let repo = LicenseRepository::new(pool, clock);
    let activations = repo.count_distinct_activations(&license.owner_id).await?;

    let now_ns = clock.timestamp_ns();
    let heartbeat_status = evaluate_heartbeat_status(now_ns, license.heartbeat_expires_at_ns, heartbeat_config);
    let hardware_match = hardware_matches(&license.node_id, identity_node_id);

    // Reconstruye EXACTAMENTE el payload que el emisor firmó -- `issued_at_ns`
    // es el campo persistido e inmutable de la fila (columna `issued_at`,
    // distinto de `created_at`), no un valor recalculado. Si el heartbeat fue
    // refrescado (re-firmado), `license.heartbeat_expires_at_ns` y
    // `signature_hash` ya avanzaron juntos -- el payload sigue siendo
    // consistente con la firma vigente en `signature_hex`.
    let payload = LicensePayload {
        license_id: &license.license_id,
        owner_id: &license.owner_id,
        node_id: &license.node_id,
        tier: license.tier,
        issued_at_ns: license.issued_at_ns,
        heartbeat_expires_at_ns: license.heartbeat_expires_at_ns,
    };
    let signature_valid =
        verify_license_signature(&payload, signature_hex, public_key_hex).is_ok();

    Ok(derive_execution_gate(GateEvaluationInput {
        signature_valid,
        hardware_match,
        heartbeat_status,
        tier: license.tier,
        activations,
        plan_limits,
    }))
}

/// Refresca `compliance_status_id` de una licencia según su
/// [`crate::domain::licensing_system::HeartbeatStatus`] vigente -- función
/// de conveniencia usada por el harness de verificación y por el futuro job
/// de revalidación periódica.
pub async fn sync_compliance_status(
    pool: &SqlitePool,
    clock: &dyn Clock,
    license: &LicenseRecord,
    heartbeat_config: &HeartbeatConfig,
) -> Result<LicenseRecord, LicenseRepositoryError> {
    let now_ns = clock.timestamp_ns();
    let status = evaluate_heartbeat_status(now_ns, license.heartbeat_expires_at_ns, heartbeat_config);
    let compliance_status_id = heartbeat_status_to_compliance_status_id(status);

    if compliance_status_id == license.compliance_status_id {
        // Sin cambio -- no genera una versión nueva por nada (evita ruido
        // en la cadena de auditoría).
        return Ok(license.clone());
    }

    // `heartbeat_expires_at` NO cambia aquí -- solo se reetiqueta el estado
    // de cumplimiento según la MISMA fecha ya firmada. La firma vigente
    // sigue siendo válida para ese mismo valor, así que se conserva tal cual
    // (no hace falta re-firmar cuando el heartbeat en sí no se extiende).
    let repo = LicenseRepository::new(pool, clock);
    repo.refresh_heartbeat(
        license,
        license.heartbeat_expires_at_ns,
        compliance_status_id,
        &license.signature_hash,
    )
    .await
}

/// Reexporta el tipo de error de firma para quien construya el harness de
/// verificación sin tener que importar `domain::licensing_system` directo.
pub type SignatureError = LicenseSignatureError;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::central_identity::EmailVerificationStatus;
    use crate::domain::clock::DeterministicClock;
    use crate::domain::licensing_system::GateVerdict;
    use crate::persistence::central_identity::{AccountRepository, NewAccount};
    use crate::persistence::licensing_system::NewLicenseActivation;
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    async fn seed_account(pool: &SqlitePool, clock: &dyn Clock, email: &str) -> String {
        let repo = AccountRepository::new(pool, clock);
        let account = repo
            .create(NewAccount {
                email: email.to_string(),
                oauth_provider: None,
                institutional_tag: "DRASUS_LOCAL".to_string(),
                access_token_id: None,
                node_id: "seed-node".to_string(),
                owner_id: None,
            })
            .await
            .expect("crear cuenta semilla");
        assert_eq!(account.email_verification_status, EmailVerificationStatus::Pending);
        account.owner_id
    }

    // ── Emisor de desarrollo: firma verificable de punta a punta ────────────

    #[test]
    fn issued_license_signature_verifies_against_issuer_public_key() {
        let issuer = LocalStubLicenseIssuer::new();
        let signed = issuer.issue_license(IssueLicenseRequest {
            owner_id: "owner-1".to_string(),
            node_id: "node-A".to_string(),
            tier: LicenseTier::Sovereign,
            issued_at_ns: 1_000,
            heartbeat_expires_at_ns: 100_000,
        });

        let payload = LicensePayload {
            license_id: &signed.license_id,
            owner_id: &signed.owner_id,
            node_id: &signed.node_id,
            tier: signed.tier,
            issued_at_ns: signed.issued_at_ns,
            heartbeat_expires_at_ns: signed.heartbeat_expires_at_ns,
        };

        assert_eq!(
            verify_license_signature(&payload, &signed.signature_hex, &signed.public_key_hex),
            Ok(())
        );
    }

    /// Dos emisores distintos tienen claves distintas -- la firma de uno NO
    /// verifica con la clave pública del otro (simula un atacante intentando
    /// hacer pasar una licencia firmada por un emisor no reconocido).
    #[test]
    fn signature_from_one_issuer_does_not_verify_with_another_issuers_key() {
        let issuer_a = LocalStubLicenseIssuer::new();
        let issuer_b = LocalStubLicenseIssuer::new();

        let signed = issuer_a.issue_license(IssueLicenseRequest {
            owner_id: "owner-1".to_string(),
            node_id: "node-A".to_string(),
            tier: LicenseTier::Sovereign,
            issued_at_ns: 1_000,
            heartbeat_expires_at_ns: 100_000,
        });

        let payload = LicensePayload {
            license_id: &signed.license_id,
            owner_id: &signed.owner_id,
            node_id: &signed.node_id,
            tier: signed.tier,
            issued_at_ns: signed.issued_at_ns,
            heartbeat_expires_at_ns: signed.heartbeat_expires_at_ns,
        };

        assert!(verify_license_signature(&payload, &signed.signature_hex, issuer_b.public_key_hex()).is_err());
    }

    // ── Proveedor de límites stub ─────────────────────────────────────────

    #[tokio::test]
    async fn stub_plan_limits_provider_returns_documented_defaults() {
        let provider = LocalStubPlanLimitsProvider::default();

        let sovereign = provider.plan_limits_for("owner-1", LicenseTier::Sovereign).await;
        assert_eq!(sovereign.max_activations, 3);

        let explorer = provider.plan_limits_for("owner-1", LicenseTier::Explorer).await;
        assert_eq!(explorer.max_activations, 1);
    }

    // ── Caché de gate con TTL (reloj determinista) ──────────────────────────

    fn sample_gate() -> ExecutionGate {
        ExecutionGate {
            verdict: GateVerdict::Allow,
            suppress_work_telemetry: true,
            tier: LicenseTier::Sovereign,
            activations: 1,
            reason: "licencia válida dentro de los límites del plan".to_string(),
        }
    }

    #[test]
    fn execution_gate_cache_returns_value_within_ttl() {
        let det_clock = Arc::new(DeterministicClock::new(0, 0));
        let clock: Arc<dyn Clock> = det_clock.clone();
        let cache = ExecutionGateCache::new(clock, ExecutionGateCacheConfig { ttl_ns: 1_000 });

        cache.set(sample_gate());
        det_clock.advance(500);

        assert_eq!(cache.get(), Some(sample_gate()));
    }

    #[test]
    fn execution_gate_cache_expires_after_ttl() {
        let det_clock = Arc::new(DeterministicClock::new(0, 0));
        let clock: Arc<dyn Clock> = det_clock.clone();
        let cache = ExecutionGateCache::new(clock, ExecutionGateCacheConfig { ttl_ns: 1_000 });

        cache.set(sample_gate());
        det_clock.advance(1_000);

        assert_eq!(cache.get(), None, "pasado el TTL, la caché debe exigir recálculo");
    }

    // ── build_execution_gate: composición completa contra BD real ──────────

    #[tokio::test]
    async fn build_execution_gate_allows_a_freshly_activated_sovereign_license() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "user@example.com").await;

        let issuer = LocalStubLicenseIssuer::new();
        let signed = issuer.issue_license(IssueLicenseRequest {
            owner_id: owner_id.clone(),
            node_id: "node-A".to_string(),
            tier: LicenseTier::Sovereign,
            issued_at_ns: clock.timestamp_ns(),
            heartbeat_expires_at_ns: clock.timestamp_ns() + 1_000_000_000_000, // muy lejos -> Fresh
        });

        let license_repo = LicenseRepository::new(&pool, &clock);
        let license = license_repo
            .activate(NewLicenseActivation {
                owner_id: owner_id.clone(),
                institutional_tag: "DRASUS_LOCAL".to_string(),
                access_token_id: None,
                node_id: signed.node_id.clone(),
                license_id: signed.license_id.clone(),
                process_id: Some("pid-test".to_string()),
                signature_hash: signed.signature_hex.clone(),
                tier: signed.tier,
                issued_at_ns: signed.issued_at_ns,
                heartbeat_expires_at_ns: signed.heartbeat_expires_at_ns,
                compliance_status_id: "ACTIVE".to_string(),
            })
            .await
            .expect("activar licencia");

        // El payload firmado usó `issued_at_ns = clock.timestamp_ns()` en el
        // momento de emitir; la fila persistida usa `created_at_ns` como
        // ancla estable -- en este test coinciden porque el reloj es el
        // mismo determinista sin avanzar entre emitir y activar.
        let heartbeat_config = HeartbeatConfig::default();
        let plan_limits = PlanLimits { max_activations: 3, features_enabled: vec![] };

        let gate = build_execution_gate(
            &pool,
            &clock,
            &signed.node_id,
            &license,
            &signed.signature_hex,
            &signed.public_key_hex,
            &heartbeat_config,
            &plan_limits,
        )
        .await
        .expect("construir gate");

        assert_eq!(gate.verdict, GateVerdict::Allow, "reason: {}", gate.reason);
        assert!(gate.suppress_work_telemetry, "Sovereign al corriente debe suprimir telemetría de trabajo");
        assert_eq!(gate.activations, 1);
    }

    #[tokio::test]
    async fn build_execution_gate_denies_when_hardware_fingerprint_differs() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "user@example.com").await;

        let issuer = LocalStubLicenseIssuer::new();
        let signed = issuer.issue_license(IssueLicenseRequest {
            owner_id: owner_id.clone(),
            node_id: "node-A".to_string(),
            tier: LicenseTier::Sovereign,
            issued_at_ns: clock.timestamp_ns(),
            heartbeat_expires_at_ns: clock.timestamp_ns() + 1_000_000_000_000,
        });

        let license_repo = LicenseRepository::new(&pool, &clock);
        let license = license_repo
            .activate(NewLicenseActivation {
                owner_id: owner_id.clone(),
                institutional_tag: "DRASUS_LOCAL".to_string(),
                access_token_id: None,
                node_id: signed.node_id.clone(),
                license_id: signed.license_id.clone(),
                process_id: None,
                signature_hash: signed.signature_hex.clone(),
                tier: signed.tier,
                issued_at_ns: signed.issued_at_ns,
                heartbeat_expires_at_ns: signed.heartbeat_expires_at_ns,
                compliance_status_id: "ACTIVE".to_string(),
            })
            .await
            .expect("activar licencia");

        let heartbeat_config = HeartbeatConfig::default();
        let plan_limits = PlanLimits { max_activations: 3, features_enabled: vec![] };

        // La instancia reporta una huella DISTINTA a la de la licencia -- simula
        // que el archivo de licencia se copió a otra máquina.
        let gate = build_execution_gate(
            &pool,
            &clock,
            "node-DISTINTO-DE-LA-LICENCIA",
            &license,
            &signed.signature_hex,
            &signed.public_key_hex,
            &heartbeat_config,
            &plan_limits,
        )
        .await
        .expect("construir gate");

        assert_eq!(gate.verdict, GateVerdict::Deny);
    }

    // ── sync_compliance_status ───────────────────────────────────────────────

    /// Cuando el estado calculado coincide con el ya persistido, no genera
    /// una versión nueva -- devuelve la misma fila, sin refrescar nada.
    #[tokio::test]
    async fn sync_compliance_status_is_noop_when_status_unchanged() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "user@example.com").await;
        let repo = LicenseRepository::new(&pool, &clock);

        let license = repo
            .activate(NewLicenseActivation {
                owner_id,
                institutional_tag: "DRASUS_LOCAL".to_string(),
                access_token_id: None,
                node_id: "node-A".to_string(),
                license_id: "license-1".to_string(),
                process_id: None,
                signature_hash: "sig-1".to_string(),
                tier: LicenseTier::Sovereign,
                issued_at_ns: 1_000,
                // Muy lejos en el futuro -> sigue Fresh -> compliance ya es ACTIVE.
                heartbeat_expires_at_ns: 1_000_000_000_000,
                compliance_status_id: "ACTIVE".to_string(),
            })
            .await
            .expect("activar licencia");

        let synced = sync_compliance_status(&pool, &clock, &license, &HeartbeatConfig::default())
            .await
            .expect("sincronizar estado");

        assert_eq!(synced.row_version, license.row_version, "sin cambio de estado no debe crear una versión nueva");
        assert_eq!(synced, license);
    }

    /// Cuando el heartbeat ya pasó su período de gracia, `sync_compliance_status`
    /// refresca la fila a `EXPIRED` -- conservando `signature_hash` (no hace
    /// falta re-firmar: el heartbeat en sí no cambió, solo su interpretación).
    #[tokio::test]
    async fn sync_compliance_status_transitions_to_expired_and_preserves_signature() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "user@example.com").await;
        let repo = LicenseRepository::new(&pool, &clock);

        let license = repo
            .activate(NewLicenseActivation {
                owner_id,
                institutional_tag: "DRASUS_LOCAL".to_string(),
                access_token_id: None,
                node_id: "node-A".to_string(),
                license_id: "license-1".to_string(),
                process_id: None,
                signature_hash: "sig-original".to_string(),
                tier: LicenseTier::Sovereign,
                issued_at_ns: 1_000,
                // Con la ventana de gracia corta de abajo (50ns), este valor
                // ya quedó atrás del final de la gracia en el momento de sincronizar.
                heartbeat_expires_at_ns: 500,
                compliance_status_id: "ACTIVE".to_string(),
            })
            .await
            .expect("activar licencia");

        // Ventana de gracia deliberadamente corta (a diferencia del default de
        // 7 días) para que el reloj determinista de este test la agote sin
        // tener que avanzar nanosegundos astronómicos.
        let short_grace_config = HeartbeatConfig { recheck_window_ns: 100, grace_period_ns: 50 };

        clock.tick();
        let synced = sync_compliance_status(&pool, &clock, &license, &short_grace_config)
            .await
            .expect("sincronizar estado");

        assert_eq!(synced.compliance_status_id, "EXPIRED");
        assert_eq!(synced.row_version, license.row_version + 1);
        assert_eq!(synced.signature_hash, license.signature_hash, "no re-firma si el heartbeat en sí no se extendió");
    }

    // ── Defaults (smoke) ──────────────────────────────────────────────────

    #[test]
    fn local_stub_license_issuer_default_produces_a_usable_keypair() {
        let issuer = LocalStubLicenseIssuer::default();
        let signed = issuer.issue_license(IssueLicenseRequest {
            owner_id: "owner-1".to_string(),
            node_id: "node-A".to_string(),
            tier: LicenseTier::Explorer,
            issued_at_ns: 0,
            heartbeat_expires_at_ns: 1_000,
        });

        let payload = LicensePayload {
            license_id: &signed.license_id,
            owner_id: &signed.owner_id,
            node_id: &signed.node_id,
            tier: signed.tier,
            issued_at_ns: signed.issued_at_ns,
            heartbeat_expires_at_ns: signed.heartbeat_expires_at_ns,
        };
        assert_eq!(
            verify_license_signature(&payload, &signed.signature_hex, &signed.public_key_hex),
            Ok(())
        );
    }

    #[test]
    fn execution_gate_cache_config_default_is_five_minutes() {
        const NANOS_PER_MINUTE: i64 = 60 * 1_000_000_000;
        assert_eq!(ExecutionGateCacheConfig::default().ttl_ns, 5 * NANOS_PER_MINUTE);
    }
}
