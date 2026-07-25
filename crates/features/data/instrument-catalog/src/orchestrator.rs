//! Imperative Shell — adaptadores de infraestructura.
//!
//! Implementa la Frontera Pública declarada en
//! `docs/features/instrument-catalog.md`: `ResolveInstrumentTerms(instrument_id,
//! venue) → ResolvedInstrumentTerms` (TTR-003). Compone el repositorio
//! (I/O) con la función pura de resolución del Núcleo (`domain::resolve_terms`).

use sqlx::SqlitePool;

use shared::public_interface::Clock;

use crate::domain::{resolve_terms, BrokerOverride, CanonicalTerms, ResolvedInstrumentTerms};
use crate::persistence::{InstrumentRepository, InstrumentRepositoryError};

/// Errores que puede producir [`resolve_instrument_terms`].
#[derive(Debug, thiserror::Error)]
pub enum ResolveTermsError {
    /// El repositorio falló al leer alguna de las dos tablas.
    #[error("error de persistencia: {0}")]
    Repository(#[from] InstrumentRepositoryError),
    /// No existe ningún instrumento canónico con ese `instrument_id` --
    /// no hay nada que resolver.
    #[error("instrumento canónico no encontrado: '{0}'")]
    InstrumentNotFound(String),
}

/// Resuelve los términos de trading de `(instrument_id, venue)`: carga el
/// canónico (obligatorio) y el override del bróker (opcional, `None` si
/// ese bróker no tiene fila de override) y los combina vía
/// `domain::resolve_terms` -- override no-`NULL` gana, `NULL` hereda.
///
/// Es la implementación real del puerto de salida `resolved_terms_out`
/// (ADR-0137) que consumen `data-sanitizer-pipeline` (piso de spread) y
/// `backtest-engine` (fallback `SPREAD_SOURCE_MODE=FIXED`).
pub async fn resolve_instrument_terms(
    pool: &SqlitePool,
    clock: &dyn Clock,
    instrument_id: &str,
    venue: &str,
) -> Result<ResolvedInstrumentTerms, ResolveTermsError> {
    let repo = InstrumentRepository::new(pool, clock);

    let canonical_row = repo
        .find_master_by_instrument_id(instrument_id)
        .await?
        .ok_or_else(|| ResolveTermsError::InstrumentNotFound(instrument_id.to_string()))?;

    let override_row = repo.find_broker_terms(instrument_id, venue).await?;

    let canonical = CanonicalTerms {
        instrument_id: canonical_row.instrument_id,
        asset_class: canonical_row.asset_class,
        canonical_pip_size: canonical_row.canonical_pip_size,
        canonical_tick_size: canonical_row.canonical_tick_size,
        default_session_template_id: canonical_row.default_session_template_id,
    };

    let override_terms = override_row.map(|o| BrokerOverride {
        venue: o.venue,
        default_spread: o.default_spread,
        default_slippage: o.default_slippage,
        pip_size_override: o.pip_size_override,
        tick_size_override: o.tick_size_override,
        point_value: o.point_value,
        commission_model: o.commission_model,
        commission_value: o.commission_value,
        session_template_id_override: o.session_template_id_override,
    });

    Ok(resolve_terms(&canonical, override_terms.as_ref()))
}

// ── Pruebas del Shell (composición completa) ─────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use shared::public_interface::{create_pool, run_migrations, DeterministicClock};

    use crate::domain::AssetClass;
    use crate::schemas::{NewBrokerInstrumentTerms, NewInstrumentMaster};

    async fn migrated_pool() -> (SqlitePool, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = dir.path().join("test.sqlite");
        let url = format!("sqlite://{}", db_path.display());
        let pool = create_pool(&url).await.expect("abrir base de datos temporal");
        run_migrations(&pool).await.expect("aplicar migraciones");
        (pool, dir)
    }

    /// Sin override para ese bróker, `resolve_instrument_terms` resuelve
    /// 100% desde el canónico -- ejercita el camino Shell completo
    /// (persistencia real + composición con el Núcleo), no solo la
    /// función pura aislada.
    #[tokio::test]
    async fn resolve_instrument_terms_without_broker_override_uses_canonical_only() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = InstrumentRepository::new(&pool, &clock);

        repo.create_master(NewInstrumentMaster {
            instrument_id: "BTCUSDT".to_string(),
            asset_class: AssetClass::Crypto,
            base_currency: Some("BTC".to_string()),
            quote_currency: Some("USDT".to_string()),
            canonical_pip_size: 100,
            canonical_tick_size: 10,
            default_session_template_id: None,
            node_id: None,
            process_id: None,
        })
        .await
        .expect("crear canónico");

        let resolved = resolve_instrument_terms(&pool, &clock, "BTCUSDT", "BINANCE")
            .await
            .expect("debe resolver sin override");

        assert_eq!(resolved.pip_size, 100);
        assert_eq!(resolved.spread, None, "sin override, spread no tiene fuente");
    }

    /// Con override, el término declarado por el bróker gana.
    #[tokio::test]
    async fn resolve_instrument_terms_with_broker_override_applies_it() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = InstrumentRepository::new(&pool, &clock);

        repo.create_master(NewInstrumentMaster {
            instrument_id: "XAUUSD".to_string(),
            asset_class: AssetClass::Cfd,
            base_currency: None,
            quote_currency: Some("USD".to_string()),
            canonical_pip_size: 1_000,
            canonical_tick_size: 100,
            default_session_template_id: None,
            node_id: None,
            process_id: None,
        })
        .await
        .expect("crear canónico");
        repo.create_broker_terms(NewBrokerInstrumentTerms {
            instrument_id: "XAUUSD".to_string(),
            venue: "ICMARKETS".to_string(),
            default_spread: Some(250_000_000),
            ..Default::default()
        })
        .await
        .expect("crear override");

        let resolved = resolve_instrument_terms(&pool, &clock, "XAUUSD", "ICMARKETS")
            .await
            .expect("debe resolver con override");

        assert_eq!(resolved.spread, Some(250_000_000));
        assert_eq!(resolved.venue, Some("ICMARKETS".to_string()));
    }

    /// Instrumento inexistente produce un error explícito, no un
    /// `ResolvedInstrumentTerms` vacío o inventado.
    #[tokio::test]
    async fn resolve_instrument_terms_missing_instrument_returns_explicit_error() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);

        let result = resolve_instrument_terms(&pool, &clock, "DOESNOTEXIST", "ANY").await;
        assert!(
            matches!(result, Err(ResolveTermsError::InstrumentNotFound(ref id)) if id == "DOESNOTEXIST"),
            "un instrument_id sin canónico debe fallar explícitamente; fue: {result:?}"
        );
    }
}
