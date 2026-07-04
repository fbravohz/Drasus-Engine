//! [SHELL] Catálogo de desarrollo (stub) + caché con TTL de límites
//! resueltos + composición del puerto `plan_limits_out`
//! (`docs/features/plan-tier-quota.md`, ADR-0143, ADR-0144, STORY-029).
//!
//! ## El catálogo real y su stub (ADR-0144: "puerto ahora, adaptador después")
//!
//! La Cabina de Mando Central del proveedor (ADR-0143) todavía no existe --
//! nadie define planes reales todavía. [`seed_default_catalog`] siembra un
//! catálogo de planes de DESARROLLO (Free/Paid con sus cuotas, tal como
//! pide `docs/features/plan-tier-quota.md` "Parámetros Configurables") en
//! la base de datos local. El día que la Cabina de Mando exista, un
//! adaptador real reemplazará esta siembra por sincronización remota; el
//! resto del sistema (Core, caché, puerto) no cambia.
//!
//! ## Caché de límites resueltos con TTL (hot-path, ADR-0039)
//!
//! [`PlanLimitsCache`] es el mismo patrón que
//! [`crate::orchestrator::licensing_system::ExecutionGateCache`] /
//! [`crate::orchestrator::central_identity::IdentityCache`], pero KEYED por
//! [`PlanTier`] -- a diferencia de esos dos (una sola entidad por
//! instancia), aquí conviven varios tiers resueltos simultáneamente (Free Y
//! Paid pueden consultarse en la misma corrida, ej. al comparar planes).

use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};

use sqlx::SqlitePool;

use crate::domain::clock::Clock;
use crate::domain::plan_tier_quota::{resolve_limits, PlanLimits, PlanSnapshot, PlanTier, PricingModel};
use crate::persistence::plan_tier_quota::{NewPlan, PlanRepository, PlanRepositoryError};

// ── Catálogo de desarrollo (stub local) ─────────────────────────────────────

/// Cuotas por defecto del catálogo de desarrollo -- documentadas en
/// `docs/features/plan-tier-quota.md` "Parámetros Configurables"
/// (`NOTIONAL_LIMIT_FREE`) y reutilizando `ACTIVATIONS_PER_TIER` de
/// `licensing-system.md` (Explorer/Free = 1, Sovereign/Paid = 3) como el
/// límite REAL del que `licensing-system` hoy solo tiene un stub.
#[derive(Debug, Clone)]
pub struct LocalStubPlanCatalogConfig {
    /// Volumen nocional del plan Free, INTEGER escalado ×10⁸ (default
    /// $10,000.00/mes -> `1_000_000_000_000`).
    pub free_notional_limit: i64,
    pub free_max_activations: i64,
    /// Volumen nocional del plan Paid, INTEGER escalado ×10⁸ (default
    /// $1,000,000.00/mes -> `100_000_000_000_000`).
    pub paid_notional_limit: i64,
    pub paid_max_activations: i64,
    /// Precio del plan Paid, INTEGER escalado ×10⁸ (default $49.00/mes ->
    /// `4_900_000_000`).
    pub paid_price: i64,
}

impl Default for LocalStubPlanCatalogConfig {
    fn default() -> Self {
        Self {
            free_notional_limit: 1_000_000_000_000,
            free_max_activations: 1,
            paid_notional_limit: 100_000_000_000_000,
            paid_max_activations: 3,
            paid_price: 4_900_000_000,
        }
    }
}

/// Siembra el catálogo de planes de DESARROLLO (Free + Paid) en la base de
/// datos local -- **stub** hasta que exista la Cabina de Mando Central
/// (ADR-0144: "puerto ahora, adaptador después"). Es **idempotente por
/// tier**: si ya existe un plan para ese tier, lo reutiliza en vez de
/// crear uno duplicado (mismo espíritu que
/// `LicenseRepository::activate` -- reusar en vez de multiplicar filas).
pub async fn seed_default_catalog(
    pool: &SqlitePool,
    clock: &dyn Clock,
    owner_id: &str,
    node_id: &str,
    institutional_tag: &str,
    config: &LocalStubPlanCatalogConfig,
) -> Result<(), PlanRepositoryError> {
    let repo = PlanRepository::new(pool, clock);

    if repo.find_latest_by_tier(PlanTier::Free).await?.is_none() {
        repo.create(NewPlan {
            owner_id: owner_id.to_string(),
            institutional_tag: institutional_tag.to_string(),
            node_id: node_id.to_string(),
            tier: PlanTier::Free,
            notional_limit: config.free_notional_limit,
            max_activations: config.free_max_activations,
            price: 0,
            pricing_model: PricingModel::Flat,
            features_enabled: vec![],
        })
        .await?;
    }

    if repo.find_latest_by_tier(PlanTier::Paid).await?.is_none() {
        repo.create(NewPlan {
            owner_id: owner_id.to_string(),
            institutional_tag: institutional_tag.to_string(),
            node_id: node_id.to_string(),
            tier: PlanTier::Paid,
            notional_limit: config.paid_notional_limit,
            max_activations: config.paid_max_activations,
            price: config.paid_price,
            pricing_model: PricingModel::Flat,
            features_enabled: vec!["priority_support".to_string()],
        })
        .await?;
    }

    Ok(())
}

// ── Caché de límites resueltos con TTL (hot-path, ADR-0039) ─────────────────

/// Configuración de la caché de límites resueltos.
#[derive(Debug, Clone, Copy)]
pub struct PlanLimitsCacheConfig {
    /// Cuánto tiempo (ns) vale un `PlanLimits` cacheado antes de exigir un
    /// recálculo contra la BD/catálogo.
    pub ttl_ns: i64,
}

impl Default for PlanLimitsCacheConfig {
    fn default() -> Self {
        const NANOS_PER_MINUTE: i64 = 60 * 1_000_000_000;
        Self { ttl_ns: 15 * NANOS_PER_MINUTE }
    }
}

struct CachedPlanLimits {
    limits: PlanLimits,
    cached_at_ns: i64,
}

/// Caché local de [`PlanLimits`] ya resueltos, en memoria del proceso --
/// KEYED por [`PlanTier`] (a diferencia de
/// [`crate::orchestrator::licensing_system::ExecutionGateCache`], que solo
/// guarda una entrada porque una instancia tiene una sola licencia activa;
/// aquí varios tiers pueden resolverse en la misma corrida).
pub struct PlanLimitsCache {
    clock: Arc<dyn Clock>,
    config: PlanLimitsCacheConfig,
    entries: StdMutex<HashMap<PlanTier, CachedPlanLimits>>,
}

impl PlanLimitsCache {
    pub fn new(clock: Arc<dyn Clock>, config: PlanLimitsCacheConfig) -> Self {
        Self { clock, config, entries: StdMutex::new(HashMap::new()) }
    }

    /// Devuelve los límites cacheados de `tier` si siguen dentro del TTL, o
    /// `None` si expiraron o nunca se guardaron -- en ambos casos quien
    /// llama debe disparar (fuera del hot-path) un recálculo vía
    /// [`build_plan_limits_for_tier`].
    pub fn get(&self, tier: PlanTier) -> Option<PlanLimits> {
        let now_ns = self.clock.timestamp_ns();
        let guard = self.entries.lock().expect("mutex de caché de límites de plan envenenado");

        match guard.get(&tier) {
            Some(cached) if now_ns - cached.cached_at_ns < self.config.ttl_ns => Some(cached.limits.clone()),
            _ => None,
        }
    }

    /// Guarda `limits` como vigentes para `tier` a partir de "ahora".
    /// Sobrescribe cualquier entrada previa de ese mismo tier.
    pub fn set(&self, tier: PlanTier, limits: PlanLimits) {
        let now_ns = self.clock.timestamp_ns();
        let mut guard = self.entries.lock().expect("mutex de caché de límites de plan envenenado");
        guard.insert(tier, CachedPlanLimits { limits, cached_at_ns: now_ns });
    }
}

// ── Composición: construir PlanLimits para un tier (fuera del hot-path) ────

/// Errores al construir un [`PlanLimits`] contra el catálogo.
#[derive(Debug, thiserror::Error)]
pub enum BuildPlanLimitsError {
    #[error("error de persistencia del catálogo de planes: {0}")]
    Database(#[from] PlanRepositoryError),
    /// No existe ningún plan para el tier pedido -- el catálogo de
    /// desarrollo debería haberse sembrado antes de resolver límites
    /// (ver [`seed_default_catalog`]).
    #[error("no existe ningún plan para el tier '{0}' en el catálogo")]
    PlanNotFound(&'static str),
}

/// Resuelve el [`PlanLimits`] vigente para `tier`, cargando el plan más
/// reciente del catálogo y aplicando la lógica pura
/// [`crate::domain::plan_tier_quota::resolve_limits`]. Quien llama es
/// responsable de, después, guardar el resultado en un [`PlanLimitsCache`]
/// -- eso es lo que el hot-path real de `licensing-system`/`usage-metering`
/// consultaría (cuando su re-cableado real llegue, STORY-029 §8).
pub async fn build_plan_limits_for_tier(
    pool: &SqlitePool,
    clock: &dyn Clock,
    tier: PlanTier,
) -> Result<PlanLimits, BuildPlanLimitsError> {
    let repo = PlanRepository::new(pool, clock);
    let plan = repo
        .find_latest_by_tier(tier)
        .await?
        .ok_or(BuildPlanLimitsError::PlanNotFound(tier.as_str()))?;

    Ok(resolve_limits(&PlanSnapshot {
        tier: plan.tier,
        notional_limit: plan.notional_limit,
        max_activations: plan.max_activations,
        features_enabled: &plan.features_enabled,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::persistence::pool::{connect, migrate};
    use sqlx::Row;

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    // ── Catálogo de desarrollo ────────────────────────────────────────────

    #[tokio::test]
    async fn seed_default_catalog_creates_a_plan_for_each_tier() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);

        seed_default_catalog(&pool, &clock, "drasus-system", "seed-node", "DRASUS_LOCAL", &LocalStubPlanCatalogConfig::default())
            .await
            .expect("sembrar catálogo");

        let repo = PlanRepository::new(&pool, &clock);
        assert!(repo.find_latest_by_tier(PlanTier::Free).await.expect("buscar free").is_some());
        assert!(repo.find_latest_by_tier(PlanTier::Paid).await.expect("buscar paid").is_some());
    }

    /// CRITERIO DE CIERRE: sembrar dos veces no duplica los planes -- si
    /// `seed_default_catalog` no fuera idempotente por tier, esta prueba
    /// vería más de un plan Free tras la segunda siembra.
    #[tokio::test]
    async fn seed_default_catalog_is_idempotent() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let config = LocalStubPlanCatalogConfig::default();

        seed_default_catalog(&pool, &clock, "drasus-system", "seed-node", "DRASUS_LOCAL", &config)
            .await
            .expect("primera siembra");
        seed_default_catalog(&pool, &clock, "drasus-system", "seed-node", "DRASUS_LOCAL", &config)
            .await
            .expect("segunda siembra");

        let count: i64 = sqlx::query("SELECT COUNT(*) FROM plans WHERE tier = 'FREE'")
            .fetch_one(&pool)
            .await
            .expect("contar planes free")
            .get::<i64, _>(0);
        assert_eq!(count, 1, "sembrar dos veces no debe duplicar el plan Free");
    }

    // ── build_plan_limits_for_tier: composición completa ────────────────────

    #[tokio::test]
    async fn build_plan_limits_for_tier_resolves_free_and_paid_quotas() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        seed_default_catalog(&pool, &clock, "drasus-system", "seed-node", "DRASUS_LOCAL", &LocalStubPlanCatalogConfig::default())
            .await
            .expect("sembrar catálogo");

        let free_limits = build_plan_limits_for_tier(&pool, &clock, PlanTier::Free).await.expect("resolver free");
        assert_eq!(free_limits.max_activations, 1);
        assert_eq!(free_limits.notional_limit, 1_000_000_000_000);

        let paid_limits = build_plan_limits_for_tier(&pool, &clock, PlanTier::Paid).await.expect("resolver paid");
        assert_eq!(paid_limits.max_activations, 3);
        assert_eq!(paid_limits.notional_limit, 100_000_000_000_000);
    }

    #[tokio::test]
    async fn build_plan_limits_for_tier_fails_when_catalog_is_empty() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);

        let result = build_plan_limits_for_tier(&pool, &clock, PlanTier::Free).await;
        assert!(matches!(result, Err(BuildPlanLimitsError::PlanNotFound(_))));
    }

    // ── CRITERIO #9 (Orden §5): caché con TTL usando reloj determinista ─────

    fn sample_limits() -> PlanLimits {
        PlanLimits { notional_limit: 1_000_000_000_000, max_activations: 1, features_enabled: vec![] }
    }

    #[test]
    fn plan_limits_cache_returns_value_within_ttl() {
        let det_clock = Arc::new(DeterministicClock::new(0, 0));
        let clock: Arc<dyn Clock> = det_clock.clone();
        let cache = PlanLimitsCache::new(clock, PlanLimitsCacheConfig { ttl_ns: 1_000 });

        cache.set(PlanTier::Free, sample_limits());
        det_clock.advance(500);

        assert_eq!(cache.get(PlanTier::Free), Some(sample_limits()));
    }

    /// CRITERIO DE CIERRE: pasado el TTL, la caché exige recálculo -- si el
    /// TTL no se respetara, esta prueba seguiría viendo el valor cacheado
    /// tras `advance(1_000)`.
    #[test]
    fn plan_limits_cache_expires_after_ttl() {
        let det_clock = Arc::new(DeterministicClock::new(0, 0));
        let clock: Arc<dyn Clock> = det_clock.clone();
        let cache = PlanLimitsCache::new(clock, PlanLimitsCacheConfig { ttl_ns: 1_000 });

        cache.set(PlanTier::Free, sample_limits());
        det_clock.advance(1_000);

        assert_eq!(cache.get(PlanTier::Free), None, "pasado el TTL, la caché debe exigir recálculo");
    }

    /// Dos tiers distintos se cachean de forma independiente -- expirar
    /// Free no debe afectar la entrada de Paid.
    #[test]
    fn plan_limits_cache_keys_entries_independently_per_tier() {
        let det_clock = Arc::new(DeterministicClock::new(0, 0));
        let clock: Arc<dyn Clock> = det_clock.clone();
        let cache = PlanLimitsCache::new(clock, PlanLimitsCacheConfig { ttl_ns: 1_000 });

        let paid_limits = PlanLimits { notional_limit: 100_000_000_000_000, max_activations: 3, features_enabled: vec![] };

        cache.set(PlanTier::Free, sample_limits());
        det_clock.advance(500);
        cache.set(PlanTier::Paid, paid_limits.clone());
        det_clock.advance(600); // Free ya lleva 1_100ns (expirado); Paid solo 600ns (vigente).

        assert_eq!(cache.get(PlanTier::Free), None, "Free debe haber expirado");
        assert_eq!(cache.get(PlanTier::Paid), Some(paid_limits), "Paid sigue vigente de forma independiente");
    }

    #[test]
    fn plan_limits_cache_config_default_is_fifteen_minutes() {
        const NANOS_PER_MINUTE: i64 = 60 * 1_000_000_000;
        assert_eq!(PlanLimitsCacheConfig::default().ttl_ns, 15 * NANOS_PER_MINUTE);
    }
}
