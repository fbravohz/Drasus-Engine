//! [SHELL] Composición del puerto `usage_out` para Usage Metering / Libro
//! de Nocional (`docs/features/usage-metering.md`, ADR-0143, ADR-0144,
//! STORY-030).
//!
//! Este es el **primer cableado real entre cimientos** del substrato de
//! monetización (STORY-030 §"Objetivo llano"): [`record_metered_operation`]
//! consume el `PlanLimits` REAL producido por
//! [`crate::orchestrator::plan_tier_quota::build_plan_limits_for_tier`]
//! (cimiento #3, YA CONSTRUIDO) -- NO un stub -- para decidir el veredicto
//! de cuota de cada operación medida. Contraste: `licensing-system` (#2)
//! todavía consume un `PlanLimits` STUB porque cuando se construyó, #3 no
//! existía; `usage-metering` (#4) nace DESPUÉS de #3, así que consume el
//! real desde el día uno.

use sqlx::SqlitePool;

use crate::domain::clock::Clock;
use crate::domain::plan_tier_quota::PlanTier;
use crate::domain::usage_metering::{derive_billing_cycle_id, MeteredOperation, UsageRecord};
use crate::orchestrator::plan_tier_quota::{build_plan_limits_for_tier, BuildPlanLimitsError};
use crate::persistence::usage_metering::{RecordOperationInput, UsageRepository, UsageRepositoryError};

/// Errores al registrar una operación medida a través de la composición
/// completa (resolución de límites + persistencia append-only).
#[derive(Debug, thiserror::Error)]
pub enum RecordMeteredOperationError {
    /// Falló la resolución de `PlanLimits` contra el catálogo REAL de
    /// `plan-tier-quota` (#3) -- ej. el catálogo de desarrollo no se
    /// sembró todavía para este tier.
    #[error("error al resolver los límites del plan: {0}")]
    PlanLimits(#[from] BuildPlanLimitsError),
    /// Falló el cálculo de nocional o la persistencia append-only.
    #[error("error al registrar la operación medida: {0}")]
    Usage(#[from] UsageRepositoryError),
}

/// Registra UNA operación medida (`docs/features/usage-metering.md`
/// "Ciclo de Vida"), recorriendo el camino completo del cimiento #4:
///
/// 1. Resuelve `PlanLimits` REAL para `tier` vía `plan-tier-quota` (#3) --
///    el `notional_limit` que compara el acumulado.
/// 2. Deriva el `billing_cycle_id` vigente del reloj INYECTADO (NUNCA
///    `SystemTime::now()` directo -- determinismo, ADR-0002/0004).
/// 3. Registra la operación append-only vía [`UsageRepository`]: calcula
///    su nocional, lo acumula al ciclo vigente, compara contra el límite
///    y persiste.
///
/// Devuelve el [`UsageRecord`] del puerto `usage_out` -- acumulado del
/// ciclo + veredicto de cuota, sin secretos (ADR-0093).
pub async fn record_metered_operation(
    pool: &SqlitePool,
    clock: &dyn Clock,
    owner_id: &str,
    institutional_tag: &str,
    node_id: &str,
    tier: PlanTier,
    operation: MeteredOperation<'_>,
) -> Result<UsageRecord, RecordMeteredOperationError> {
    // Paso 1 -- consumo REAL del puerto `plan_limits_out` de
    // `plan-tier-quota` (#3, ya construido). Este es el primer cableado
    // real entre cimientos del substrato: ningún stub de por medio.
    let plan_limits = build_plan_limits_for_tier(pool, clock, tier).await?;

    // Paso 2 -- deriva el ciclo de facturación vigente del reloj inyectado.
    let now_ns = clock.timestamp_ns();
    let billing_cycle_id = derive_billing_cycle_id(now_ns);

    // Paso 3 -- registra la operación (Core: nocional + acumulación +
    // veredicto; Shell: persistencia append-only encadenada por hash).
    let repo = UsageRepository::new(pool, clock);
    let row = repo
        .record_operation(RecordOperationInput {
            owner_id: owner_id.to_string(),
            institutional_tag: institutional_tag.to_string(),
            node_id: node_id.to_string(),
            // Sin estado de cumplimiento explícito en esta composición --
            // campo Grupo V nullable, se rellena cuando el gate de
            // licenciamiento anote uno (follow-up de integración futuro).
            compliance_status_id: None,
            billing_cycle_id,
            instrument_id: operation.instrument_id.to_string(),
            size: operation.size,
            price: operation.price,
            notional_limit: plan_limits.notional_limit,
        })
        .await?;

    Ok(UsageRecord {
        billing_cycle_id: row.billing_cycle_id,
        cycle_accumulated: row.cycle_accumulated,
        quota_verdict: row.quota_verdict,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::domain::usage_metering::QuotaVerdict;
    use crate::orchestrator::plan_tier_quota::{seed_default_catalog, LocalStubPlanCatalogConfig};
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    /// CRITERIO DE CIERRE (Orden §5, criterio #4 -- "Consúmelo REAL, no
    /// un stub"): siembra el catálogo REAL de `plan-tier-quota` y verifica
    /// que el `notional_limit` que decide el veredicto de cuota es
    /// EXACTAMENTE el del catálogo sembrado ($10,000.00 para FREE) -- si
    /// `record_metered_operation` usara un stub o un valor inventado, el
    /// veredicto de esta prueba no coincidiría con el límite real.
    #[tokio::test]
    async fn record_metered_operation_uses_real_plan_limits_to_detect_crossing() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);

        seed_default_catalog(&pool, &clock, "drasus-system", "seed-node", "DRASUS_LOCAL", &LocalStubPlanCatalogConfig::default())
            .await
            .expect("sembrar catálogo real de plan-tier-quota");

        // Operación pequeña ($1,000.00) -- muy por debajo del límite FREE real ($10,000.00).
        let within = record_metered_operation(
            &pool,
            &clock,
            "owner-1",
            "DRASUS_LOCAL",
            "node-1",
            PlanTier::Free,
            MeteredOperation { size: 100_000_000, price: 100_000_000_000, instrument_id: "BTCUSDT" },
        )
        .await
        .expect("registrar operación pequeña");
        assert_eq!(within.quota_verdict, QuotaVerdict::Within);

        // Operación grande ($100,000.00) -- el acumulado cruza el límite FREE real ($10,000.00).
        let crossed = record_metered_operation(
            &pool,
            &clock,
            "owner-1",
            "DRASUS_LOCAL",
            "node-1",
            PlanTier::Free,
            MeteredOperation { size: 250_000_000, price: 4_000_000_000_000, instrument_id: "BTCUSDT" },
        )
        .await
        .expect("registrar operación grande");
        assert_eq!(crossed.quota_verdict, QuotaVerdict::Crossed, "debe cruzar el límite FREE real de $10,000.00");
    }

    /// El tier PAID real tiene un límite mucho mayor ($1,000,000.00) --
    /// la misma operación que cruza en FREE debe seguir "dentro" en PAID,
    /// confirmando que el límite consumido depende del tier resuelto.
    #[tokio::test]
    async fn record_metered_operation_uses_the_correct_tier_limit() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);

        seed_default_catalog(&pool, &clock, "drasus-system", "seed-node", "DRASUS_LOCAL", &LocalStubPlanCatalogConfig::default())
            .await
            .expect("sembrar catálogo real de plan-tier-quota");

        let paid_result = record_metered_operation(
            &pool,
            &clock,
            "owner-2",
            "DRASUS_LOCAL",
            "node-1",
            PlanTier::Paid,
            MeteredOperation { size: 250_000_000, price: 4_000_000_000_000, instrument_id: "BTCUSDT" },
        )
        .await
        .expect("registrar operación en tier PAID");
        assert_eq!(paid_result.quota_verdict, QuotaVerdict::Within, "el límite PAID real es mucho mayor -- no debe cruzar");
    }

    /// Si el catálogo de `plan-tier-quota` no se sembró para el tier
    /// pedido, el error se propaga -- no hay fallback silencioso a un
    /// stub.
    #[tokio::test]
    async fn record_metered_operation_fails_when_plan_catalog_is_empty() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);

        let result = record_metered_operation(
            &pool,
            &clock,
            "owner-1",
            "DRASUS_LOCAL",
            "node-1",
            PlanTier::Free,
            MeteredOperation { size: 100_000_000, price: 100_000_000_000, instrument_id: "BTCUSDT" },
        )
        .await;

        assert!(matches!(result, Err(RecordMeteredOperationError::PlanLimits(_))));
    }
}
