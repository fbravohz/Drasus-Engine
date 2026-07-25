//! Superficie pública de `instrument-catalog` — el ÚNICO módulo `pub`.
//!
//! Expone:
//! - El puerto de salida (`OutputPorts`) con el marcador de tipo
//!   `ResolvedInstrumentTerms` del catálogo ADR-0137.
//! - Los repositorios y funciones de dominio necesarios para que el
//!   Bridge/CLI compongan altas, ediciones y resolución.
//! - El harness de verificación CLI (`verify`, ADR-0142 Fase 1).
//!
//! Otros crates solo pueden importar desde aquí; prohibido acceder a
//! `domain`, `orchestrator`, `persistence` o `schemas` directamente.

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use shared::public_interface::{create_pool, run_migrations, SystemClock};

use crate::orchestrator;

// ── Re-exports para que Bridge/CLI y los tests de integración accedan ──────

pub use crate::domain::{
    compute_broker_terms_audit_hash, compute_instrument_master_audit_hash, resolve_terms, AssetClass, BrokerOverride,
    CanonicalTerms, CommissionModel, ResolvedInstrumentTerms,
};
pub use crate::orchestrator::{resolve_instrument_terms, ResolveTermsError};
pub use crate::persistence::{InstrumentRepository, InstrumentRepositoryError};
pub use crate::schemas::{
    BrokerInstrumentTermsRow, InstrumentMasterRevision, InstrumentMasterRow, NewBrokerInstrumentTerms,
    NewInstrumentMaster,
};

// ── Puerto de salida del Canvas (marcador de tipo, ADR-0137) ────────────────

/// Puerto de salida de `instrument-catalog` hacia el Canvas.
pub struct OutputPorts {
    /// Términos de trading ya resueltos (canónico + override combinados),
    /// puerto `resolved_terms_out`, cardinalidad 1.
    pub resolved_terms_out: ResolvedInstrumentTerms,
}

// ── Harness de verificación CLI (ADR-0142 Fase 1) ────────────────────────────

/// Input para la verificación de `instrument-catalog` vía CLI. Se
/// deserializa desde el JSON que pasa el usuario con `--input '...'`.
///
/// Autocontenido: si el instrumento/override que describe no existe
/// todavía en la BD temporal de verificación, `verify()` los da de alta
/// primero -- así `drasus verify instrument-catalog --input
/// '{"instrument_id":"EURUSD","venue":"PEPPERSTONE"}'` produce un
/// resultado significativo sin que el usuario tenga que hacer un alta por
/// separado.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyInput {
    /// Símbolo del instrumento a resolver. Default: `"EURUSD"`.
    #[serde(default = "default_instrument_id")]
    pub instrument_id: String,
    /// Bróker/venue a resolver. Default: `"PEPPERSTONE"`.
    #[serde(default = "default_venue")]
    pub venue: String,
    /// Clase de activo del instrumento (si hay que darlo de alta). Default: `FOREX`.
    #[serde(default = "default_asset_class")]
    pub asset_class: AssetClass,
    /// Pip size canónico ×10⁸ (si hay que darlo de alta). Default: 10 000
    /// (0.0001, convención Forex de 5 decimales).
    #[serde(default = "default_pip_size")]
    pub canonical_pip_size: i64,
    /// Tick size canónico ×10⁸ (si hay que darlo de alta). Default: 1 000.
    #[serde(default = "default_tick_size")]
    pub canonical_tick_size: i64,
    /// Piso de spread declarado por el bróker (si hay que dar de alta el
    /// override). `None`/ausente = no crea override -- la resolución
    /// devuelve 100% el canónico.
    #[serde(default)]
    pub default_spread: Option<i64>,
}

fn default_instrument_id() -> String {
    "EURUSD".to_string()
}

fn default_venue() -> String {
    "PEPPERSTONE".to_string()
}

fn default_asset_class() -> AssetClass {
    AssetClass::Forex
}

fn default_pip_size() -> i64 {
    10_000
}

fn default_tick_size() -> i64 {
    1_000
}

impl Default for VerifyInput {
    fn default() -> Self {
        Self {
            instrument_id: default_instrument_id(),
            venue: default_venue(),
            asset_class: default_asset_class(),
            canonical_pip_size: default_pip_size(),
            canonical_tick_size: default_tick_size(),
            default_spread: None,
        }
    }
}

/// Output de la verificación de `instrument-catalog`. Siempre serializa a
/// JSON válido (ADR-0142).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOutput {
    pub ok: bool,
    pub instrument_id: Option<String>,
    pub venue: Option<String>,
    pub pip_size: Option<i64>,
    pub tick_size: Option<i64>,
    pub spread: Option<i64>,
    pub resolved_session_template_id: Option<String>,
    pub error: Option<String>,
}

impl VerifyOutput {
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            instrument_id: None,
            venue: None,
            pip_size: None,
            tick_size: None,
            spread: None,
            resolved_session_template_id: None,
            error: Some(msg),
        }
    }

    fn from_resolved(r: ResolvedInstrumentTerms) -> Self {
        Self {
            ok: true,
            instrument_id: Some(r.instrument_id),
            venue: r.venue,
            pip_size: Some(r.pip_size),
            tick_size: Some(r.tick_size),
            spread: r.spread,
            resolved_session_template_id: r.resolved_session_template_id,
            error: None,
        }
    }
}

/// Ejecuta la verificación de `instrument-catalog` con adaptadores reales:
/// BD SQLite temporal + reloj de sistema real. Da de alta el instrumento
/// (y el override, si `default_spread` viene presente) y devuelve la
/// resolución.
///
/// Uso típico desde el CLI:
/// `drasus verify instrument-catalog --input '{"instrument_id":"EURUSD","venue":"PEPPERSTONE"}'`
pub async fn verify(input: VerifyInput) -> VerifyOutput {
    let temp_dir = std::env::temp_dir().join(format!("drasus-verify-instrument-catalog-{}", Uuid::new_v4()));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return VerifyOutput::from_error(format!("no se pudo crear el directorio temporal de verificación: {e}"));
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool: SqlitePool = match create_pool(&db_url).await {
        Ok(p) => p,
        Err(e) => return VerifyOutput::from_error(format!("no se pudo crear la BD temporal de verificación: {e}")),
    };
    if let Err(e) = run_migrations(&pool).await {
        return VerifyOutput::from_error(format!("error al aplicar migraciones en la BD temporal: {e}"));
    }

    let clock = SystemClock::default();
    let repo = crate::persistence::InstrumentRepository::new(&pool, &clock);

    if let Err(e) = repo
        .create_master(crate::schemas::NewInstrumentMaster {
            instrument_id: input.instrument_id.clone(),
            asset_class: input.asset_class,
            base_currency: None,
            quote_currency: None,
            canonical_pip_size: input.canonical_pip_size,
            canonical_tick_size: input.canonical_tick_size,
            default_session_template_id: None,
            node_id: Some(format!("drasus-verify-pid-{}", std::process::id())),
            process_id: Some(format!("drasus-pid-{}", std::process::id())),
        })
        .await
    {
        return VerifyOutput::from_error(format!("no se pudo dar de alta el instrumento de verificación: {e}"));
    }

    if let Some(default_spread) = input.default_spread {
        if let Err(e) = repo
            .create_broker_terms(crate::schemas::NewBrokerInstrumentTerms {
                instrument_id: input.instrument_id.clone(),
                venue: input.venue.clone(),
                default_spread: Some(default_spread),
                ..Default::default()
            })
            .await
        {
            return VerifyOutput::from_error(format!("no se pudo dar de alta el override de bróker: {e}"));
        }
    }

    match orchestrator::resolve_instrument_terms(&pool, &clock, &input.instrument_id, &input.venue).await {
        Ok(resolved) => VerifyOutput::from_resolved(resolved),
        Err(e) => VerifyOutput::from_error(e.to_string()),
    }
}

// ── Pruebas de la superficie pública ────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// El arranque de humo sin `--input` (todos los defaults) debe
    /// producir una verificación exitosa -- fija los valores golden del
    /// arranque de humo.
    #[test]
    fn verify_input_default_values_are_golden_smoke_sentinels() {
        let input = VerifyInput::default();
        assert_eq!(input.instrument_id, "EURUSD");
        assert_eq!(input.venue, "PEPPERSTONE");
        assert_eq!(input.asset_class, AssetClass::Forex);
        assert_eq!(input.canonical_pip_size, 10_000);
        assert_eq!(input.canonical_tick_size, 1_000);
        assert!(input.default_spread.is_none());
    }

    /// La verificación de punta a punta (adaptadores reales, BD temporal)
    /// debe dar `ok: true` con los defaults -- humo sin argumentos.
    #[tokio::test(flavor = "multi_thread")]
    async fn verify_with_defaults_succeeds() {
        let output = verify(VerifyInput::default()).await;
        assert!(output.ok, "verify() con defaults debe tener éxito: {:?}", output.error);
        assert_eq!(output.instrument_id, Some("EURUSD".to_string()));
        assert_eq!(output.pip_size, Some(10_000));
    }

    /// Con `default_spread` presente en el input, `verify()` da de alta el
    /// override y la resolución lo refleja -- ejercita el camino con
    /// override desde el harness CLI.
    #[tokio::test(flavor = "multi_thread")]
    async fn verify_with_default_spread_creates_override_and_resolves_it() {
        let input = VerifyInput { default_spread: Some(200_000_000), ..VerifyInput::default() };
        let output = verify(input).await;
        assert!(output.ok, "verify() con override debe tener éxito: {:?}", output.error);
        assert_eq!(output.spread, Some(200_000_000));
    }
}
