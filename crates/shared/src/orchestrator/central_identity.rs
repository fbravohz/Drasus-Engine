//! [SHELL] Caché de identidad con TTL + puerto de verificación central con
//! stub local (`docs/features/central-identity.md`, ADR-0143, ADR-0144).
//!
//! ## El puerto y su adaptador diferido (ADR-0144: "puerto ahora, adaptador
//! ## después")
//!
//! La Cabina de Mando Central del proveedor (ADR-0143) todavía no existe.
//! [`CentralIdentityVerifier`] es el contrato que cualquier verificación de
//! identidad debe cumplir -- HOY solo existe
//! [`LocalStubCentralIdentityVerifier`], que crea/recupera la cuenta
//! LOCALMENTE (sin contactar ningún servidor). El día que la Cabina de
//! Mando exista, se escribe un segundo `impl CentralIdentityVerifier` que sí
//! hace la llamada gRPC real -- el resto del sistema ([`IdentityCache`], el
//! CLI de verificación, el puerto `identity_out`) no cambia una línea,
//! porque todos dependen del trait, no del stub concreto.
//!
//! ## Caché con TTL (`docs/features/central-identity.md` "Parámetros
//! ## Configurables": `IDENTITY_CACHE_TTL`)
//!
//! [`IdentityCache`] guarda la última [`AccountIdentity`] verificada junto
//! con el instante en que se guardó (leído del [`Clock`] inyectado, NUNCA
//! `SystemTime::now()` -- así los tests pueden simular el paso del tiempo
//! sin `sleep`). Mientras no pasen más de `ttl_ns` desde ese instante,
//! [`IdentityCache::get`] devuelve la identidad cacheada sin tocar la base
//! de datos ni la red -- esto es lo que permite operar offline
//! (central-identity.md: "consulta a la Cabina de Mando la identidad
//! vinculada y la cachea para operación offline").

use std::sync::{Arc, Mutex as StdMutex};

use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::domain::central_identity::{
    compute_hardware_fingerprint, AccountIdentity, EmailFormatError, HardwareFingerprintError,
};
use crate::domain::clock::Clock;
use crate::persistence::central_identity::{AccountRepository, AccountRepositoryError, NewAccount};

// ── Caché de identidad con TTL ───────────────────────────────────────────────

/// Configuración de la caché de identidad (`central-identity.md`
/// "Parámetros Configurables": `IDENTITY_CACHE_TTL`, default 24h, rango
/// 1h-30d, CONFIG).
#[derive(Debug, Clone, Copy)]
pub struct IdentityCacheConfig {
    /// Cuánto tiempo (en nanosegundos) vale la identidad cacheada antes de
    /// exigir revalidación.
    pub ttl_ns: i64,
}

impl Default for IdentityCacheConfig {
    /// 24 horas, el default declarado por la Feature.
    fn default() -> Self {
        const NANOS_PER_HOUR: i64 = 60 * 60 * 1_000_000_000;
        Self { ttl_ns: 24 * NANOS_PER_HOUR }
    }
}

/// Entrada cacheada: la identidad más el instante (leído del [`Clock`]
/// inyectado) en que se guardó.
#[derive(Debug, Clone)]
struct CachedEntry {
    identity: AccountIdentity,
    cached_at_ns: i64,
}

/// Caché local de identidad con TTL, en memoria del proceso (sin
/// persistencia propia -- si el proceso reinicia, la caché arranca vacía y
/// la siguiente llamada revalida contra [`CentralIdentityVerifier`]).
pub struct IdentityCache {
    clock: Arc<dyn Clock>,
    config: IdentityCacheConfig,
    entry: StdMutex<Option<CachedEntry>>,
}

impl IdentityCache {
    /// Crea una caché vacía ligada a `clock` (para poder simular el paso
    /// del tiempo en tests con [`crate::domain::clock::DeterministicClock`])
    /// y `config` (el TTL vigente).
    pub fn new(clock: Arc<dyn Clock>, config: IdentityCacheConfig) -> Self {
        Self { clock, config, entry: StdMutex::new(None) }
    }

    /// Devuelve la identidad cacheada si sigue dentro del TTL desde que se
    /// guardó, o `None` si expiró o nunca se guardó nada -- en ambos casos
    /// quien llama debe revalidar contra [`CentralIdentityVerifier`] antes
    /// de confiar en la identidad.
    pub fn get(&self) -> Option<AccountIdentity> {
        let now_ns = self.clock.timestamp_ns();
        let guard = self.entry.lock().expect("mutex de caché de identidad envenenado");

        match guard.as_ref() {
            // Vigente: el tiempo transcurrido desde que se guardó es menor al TTL configurado.
            Some(cached) if now_ns - cached.cached_at_ns < self.config.ttl_ns => Some(cached.identity.clone()),
            // Expirado (o nunca hubo nada guardado): exige revalidación.
            _ => None,
        }
    }

    /// Guarda `identity` como vigente a partir de "ahora" (lectura del
    /// [`Clock`] inyectado). Sobrescribe cualquier entrada previa.
    pub fn set(&self, identity: AccountIdentity) {
        let now_ns = self.clock.timestamp_ns();
        let mut guard = self.entry.lock().expect("mutex de caché de identidad envenenado");
        *guard = Some(CachedEntry { identity, cached_at_ns: now_ns });
    }
}

// ── Puerto de verificación central + stub local ──────────────────────────────

/// Solicitud de verificación/vinculación de identidad
/// (`docs/features/central-identity.md` "Ciclo de Vida" - "Entrada":
/// credenciales del usuario + identificadores de la máquina).
#[derive(Debug, Clone)]
pub struct IdentityVerificationRequest {
    pub email: String,
    pub oauth_provider: Option<String>,
    /// Identificadores de máquina SIN procesar (ej. UUID de placa madre,
    /// serial de disco) -- el verificador calcula la huella de hardware a
    /// partir de esta lista, quien llama no la calcula de antemano.
    pub machine_identifiers: Vec<String>,
    pub institutional_tag: String,
    pub access_token_id: Option<String>,
}

/// Errores de la verificación de identidad.
#[derive(Debug, thiserror::Error)]
pub enum CentralIdentityError {
    #[error("formato de correo inválido: {0}")]
    InvalidEmail(#[from] EmailFormatError),
    #[error("huella de hardware no derivable: {0}")]
    HardwareFingerprint(#[from] HardwareFingerprintError),
    #[error("error de persistencia de la cuenta: {0}")]
    Database(#[from] AccountRepositoryError),
}

/// El puerto de verificación contra la Cabina de Mando Central (ADR-0144:
/// "puerto ahora, adaptador después"). Cualquier `impl` de este trait puede
/// sustituir a [`LocalStubCentralIdentityVerifier`] sin tocar el resto del
/// sistema -- el caché, el CLI de verificación y el puerto `identity_out`
/// solo conocen este trait, nunca el struct concreto.
#[async_trait]
pub trait CentralIdentityVerifier: Send + Sync {
    /// Verifica (o vincula) la identidad descrita por `request` y devuelve
    /// la proyección pública [`AccountIdentity`] (sin secretos, ADR-0093).
    async fn verify_identity(
        &self,
        request: IdentityVerificationRequest,
    ) -> Result<AccountIdentity, CentralIdentityError>;
}

/// Implementación stub local del puerto de verificación central.
///
/// La Cabina de Mando Central del proveedor (ADR-0143) todavía NO existe.
/// En su lugar, este stub crea o recupera la cuenta LOCALMENTE: busca por
/// correo, y si no existe la crea con estado `PENDING` (ninguna
/// verificación real contra un servidor central ocurrió -- es trabajo
/// diferido). Cuando el adaptador real se construya (una llamada gRPC
/// contra la Cabina de Mando), reemplaza a este struct implementando el
/// mismo trait [`CentralIdentityVerifier`]; nada más en el sistema cambia.
pub struct LocalStubCentralIdentityVerifier<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> LocalStubCentralIdentityVerifier<'a> {
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }
}

#[async_trait]
impl<'a> CentralIdentityVerifier for LocalStubCentralIdentityVerifier<'a> {
    async fn verify_identity(
        &self,
        request: IdentityVerificationRequest,
    ) -> Result<AccountIdentity, CentralIdentityError> {
        let repo = AccountRepository::new(self.pool, self.clock);

        // Idempotencia: busca primero por correo antes de crear -- reintentar
        // la misma verificación no duplica la cuenta (central-identity.md:
        // "el motor local nunca es fuente de verdad de identidad -- solo
        // cachea"; localmente, "cachea" empieza por no duplicar filas).
        if let Some(existing) = repo.find_by_email(&request.email).await? {
            return Ok(AccountIdentity::from(&existing));
        }

        // Propaga el error si no hay material de máquina utilizable
        // (Defecto 2 del QA): sin `?`, una lista vacía daría un node_id
        // constante compartido por todas las máquinas.
        let node_id = compute_hardware_fingerprint(&request.machine_identifiers)?;

        let created = repo
            .create(NewAccount {
                email: request.email,
                oauth_provider: request.oauth_provider,
                institutional_tag: request.institutional_tag,
                access_token_id: request.access_token_id,
                node_id,
                owner_id: None,
            })
            .await?;

        Ok(AccountIdentity::from(&created))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::central_identity::EmailVerificationStatus;
    use crate::domain::clock::DeterministicClock;
    use crate::persistence::pool::{connect, migrate};
    use sqlx::Row;

    fn sample_identity() -> AccountIdentity {
        AccountIdentity {
            owner_id: "owner-1".to_string(),
            email: "user@example.com".to_string(),
            email_verification_status: EmailVerificationStatus::Pending,
            node_id: "node-1".to_string(),
            institutional_tag: "DRASUS_LOCAL".to_string(),
        }
    }

    // ── CRITERIO DE CIERRE: caché TTL con reloj determinista ────────────────

    /// Dentro del TTL, la identidad guardada se devuelve tal cual, sin
    /// necesidad de revalidar.
    #[test]
    fn get_returns_cached_identity_within_ttl() {
        let det_clock = Arc::new(DeterministicClock::new(0, 0));
        let clock: Arc<dyn Clock> = det_clock.clone();
        let cache = IdentityCache::new(clock, IdentityCacheConfig { ttl_ns: 1_000 });

        cache.set(sample_identity());

        // Avanza 500ns -- todavía dentro del TTL de 1000ns.
        det_clock.advance(500);

        assert_eq!(cache.get(), Some(sample_identity()));
    }

    /// Pasado el TTL desde que se guardó, `get` devuelve `None` -- exige
    /// revalidación. Medido con el reloj inyectado (`DeterministicClock`),
    /// nunca con `SystemTime`/`sleep` real.
    #[test]
    fn get_returns_none_after_ttl_expires() {
        let det_clock = Arc::new(DeterministicClock::new(0, 0));
        let clock: Arc<dyn Clock> = det_clock.clone();
        let cache = IdentityCache::new(clock, IdentityCacheConfig { ttl_ns: 1_000 });

        cache.set(sample_identity());

        // Avanza 1000ns -- exactamente el TTL: `now - cached_at < ttl_ns` ya es falso.
        det_clock.advance(1_000);

        assert_eq!(
            cache.get(),
            None,
            "pasado el TTL, la caché debe exigir revalidación (devolver None)"
        );
    }

    /// Antes de la primera llamada a `set`, la caché está vacía y `get`
    /// devuelve `None` (caso de arranque en frío).
    #[test]
    fn get_returns_none_before_anything_is_cached() {
        let clock: Arc<dyn Clock> = Arc::new(DeterministicClock::new(0, 0));
        let cache = IdentityCache::new(clock, IdentityCacheConfig::default());

        assert_eq!(cache.get(), None);
    }

    /// El default de configuración es exactamente 24 horas en nanosegundos
    /// (central-identity.md: `IDENTITY_CACHE_TTL` default "24 h").
    #[test]
    fn default_ttl_is_24_hours_in_nanoseconds() {
        let expected_ns: i64 = 24 * 60 * 60 * 1_000_000_000;
        assert_eq!(IdentityCacheConfig::default().ttl_ns, expected_ns);
    }

    // ── Verificador stub local ───────────────────────────────────────────────

    /// La primera verificación crea la cuenta; una segunda verificación con
    /// el mismo correo es idempotente -- no duplica la fila, devuelve la
    /// misma identidad (mismo `owner_id`).
    #[tokio::test]
    async fn local_stub_verifier_is_idempotent_for_the_same_email() {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        let clock = DeterministicClock::new(1_000, 100);
        let verifier = LocalStubCentralIdentityVerifier::new(&pool, &clock);

        let request = IdentityVerificationRequest {
            email: "user@example.com".to_string(),
            oauth_provider: None,
            machine_identifiers: vec!["motherboard-1".to_string()],
            institutional_tag: "DRASUS_LOCAL".to_string(),
            access_token_id: None,
        };

        let first = verifier.verify_identity(request.clone()).await.expect("primera verificación");
        let second = verifier.verify_identity(request).await.expect("segunda verificación");

        assert_eq!(first.owner_id, second.owner_id, "la segunda verificación debe reusar la misma cuenta");

        let count: i64 = sqlx::query("SELECT COUNT(*) FROM accounts")
            .fetch_one(&pool)
            .await
            .expect("contar filas")
            .get::<i64, _>(0);
        assert_eq!(count, 1, "verificar dos veces el mismo correo no debe duplicar la cuenta");
    }

    /// La cuenta creada por el stub nace con estado `PENDING` -- ninguna
    /// verificación real contra un servidor ocurrió (es la promesa
    /// diferida, ADR-0144).
    #[tokio::test]
    async fn local_stub_verifier_creates_account_in_pending_status() {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        let clock = DeterministicClock::new(1_000, 100);
        let verifier = LocalStubCentralIdentityVerifier::new(&pool, &clock);

        let identity = verifier
            .verify_identity(IdentityVerificationRequest {
                email: "pending@example.com".to_string(),
                oauth_provider: None,
                machine_identifiers: vec!["disk-1".to_string()],
                institutional_tag: "DRASUS_LOCAL".to_string(),
                access_token_id: None,
            })
            .await
            .expect("verificar identidad");

        assert_eq!(identity.email_verification_status, EmailVerificationStatus::Pending);
    }

    /// CRITERIO DE CIERRE (Defecto 2 del QA, camino Shell): verificar una
    /// identidad sin identificadores de máquina utilizables NO crea una
    /// cuenta con `node_id` constante -- propaga el error de huella y no
    /// inserta nada.
    #[tokio::test]
    async fn local_stub_verifier_rejects_empty_machine_identifiers() {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        let clock = DeterministicClock::new(1_000, 100);
        let verifier = LocalStubCentralIdentityVerifier::new(&pool, &clock);

        let result = verifier
            .verify_identity(IdentityVerificationRequest {
                email: "no-hardware@example.com".to_string(),
                oauth_provider: None,
                machine_identifiers: vec![], // sin material de máquina
                institutional_tag: "DRASUS_LOCAL".to_string(),
                access_token_id: None,
            })
            .await;

        assert!(
            matches!(result, Err(CentralIdentityError::HardwareFingerprint(_))),
            "sin identificadores de máquina utilizables, verify_identity debe fallar; fue: {result:?}"
        );

        let count: i64 = sqlx::query("SELECT COUNT(*) FROM accounts")
            .fetch_one(&pool)
            .await
            .expect("contar filas")
            .get::<i64, _>(0);
        assert_eq!(count, 0, "un fallo de huella no debe insertar ninguna cuenta");
    }

    /// El stub también resuelve la idempotencia case-insensitive: registrar
    /// `Case@Example.com` y luego `case@example.com` reusa la misma cuenta
    /// (Defecto 3 del QA, camino Shell).
    #[tokio::test]
    async fn local_stub_verifier_is_case_insensitive_for_email() {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        let clock = DeterministicClock::new(1_000, 100);
        let verifier = LocalStubCentralIdentityVerifier::new(&pool, &clock);

        let upper = verifier
            .verify_identity(IdentityVerificationRequest {
                email: "Case@Example.COM".to_string(),
                oauth_provider: None,
                machine_identifiers: vec!["disk-1".to_string()],
                institutional_tag: "DRASUS_LOCAL".to_string(),
                access_token_id: None,
            })
            .await
            .expect("registro con mayúsculas");

        let lower = verifier
            .verify_identity(IdentityVerificationRequest {
                email: "case@example.com".to_string(),
                oauth_provider: None,
                machine_identifiers: vec!["disk-1".to_string()],
                institutional_tag: "DRASUS_LOCAL".to_string(),
                access_token_id: None,
            })
            .await
            .expect("registro con minúsculas");

        assert_eq!(upper.owner_id, lower.owner_id, "ambas variantes deben resolver a la misma cuenta");
        assert_eq!(lower.email, "case@example.com");

        let count: i64 = sqlx::query("SELECT COUNT(*) FROM accounts")
            .fetch_one(&pool)
            .await
            .expect("contar filas")
            .get::<i64, _>(0);
        assert_eq!(count, 1, "las dos cajas del mismo correo no deben duplicar la cuenta");
    }
}
