//! Superficie pública de `session-catalog` — el ÚNICO módulo `pub`.
//!
//! Expone:
//! - Los puertos de salida (`OutputPorts`) con los marcadores de tipo
//!   `SessionCalendar` y la huella de calendario esperada del catálogo
//!   ADR-0137.
//! - El repositorio, la función de dominio y la semilla
//!   `SESSION_TEMPLATE_SEED_SET` necesarios para que el Bridge/CLI
//!   compongan altas y resoluciones.
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
    cme_globex_representative_windows, compute_expected_fingerprint, compute_session_template_audit_hash,
    compute_session_window_audit_hash, resolve_session_template_id, CalendarFingerprint, SessionCatalogDomainError,
    SessionWindow, Weekday,
};
pub use crate::orchestrator::{
    expected_fingerprint, resolve_session, seed_default_templates, session_template_seed_set, SessionOrchestratorError,
};
pub use crate::persistence::{SessionRepository, SessionRepositoryError};
pub use crate::schemas::{NewSessionTemplate, SessionCalendar, SessionTemplateRow, SessionTemplateWindowRow};

// ── Puerto de salida del Canvas (marcador de tipo, ADR-0137) ────────────────

/// Puerto de salida de `session-catalog` hacia el Canvas.
pub struct OutputPorts {
    /// Plantilla de sesión resuelta (ventanas + zona horaria), puerto
    /// `session_calendar_out`, cardinalidad 1.
    pub session_calendar_out: SessionCalendar,
    /// Huella de calendario esperada, puerto `expected_fingerprint_out` --
    /// payload de comparación consumido directamente por
    /// `pit-data-validator`, no cableable de forma independiente en el
    /// Canvas (ver `docs/features/session-catalog.md` "Puertos de
    /// Integración").
    pub expected_fingerprint_out: CalendarFingerprint,
}

// ── Harness de verificación CLI (ADR-0142 Fase 1) ────────────────────────────

/// Input para la verificación de `session-catalog` vía CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyInput {
    /// Código de la plantilla a verificar. Default: `"FOREX_245"`.
    #[serde(default = "default_template_code")]
    pub template_code: String,
}

fn default_template_code() -> String {
    "FOREX_245".to_string()
}

impl Default for VerifyInput {
    fn default() -> Self {
        Self { template_code: default_template_code() }
    }
}

/// Output de la verificación de `session-catalog`. Siempre serializa a
/// JSON válido (ADR-0142).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOutput {
    pub ok: bool,
    pub template_code: Option<String>,
    pub market_timezone_ref: Option<String>,
    pub window_count: Option<usize>,
    pub fingerprint: Option<CalendarFingerprint>,
    pub error: Option<String>,
}

impl VerifyOutput {
    fn from_error(msg: String) -> Self {
        Self { ok: false, template_code: None, market_timezone_ref: None, window_count: None, fingerprint: None, error: Some(msg) }
    }
}

/// Ejecuta la verificación de `session-catalog` con adaptadores reales: BD
/// SQLite temporal + reloj de sistema real. Siembra
/// `SESSION_TEMPLATE_SEED_SET` (idempotente) y resuelve la huella de la
/// plantilla pedida.
///
/// Uso típico desde el CLI:
/// `drasus verify session-catalog --input '{"template_code":"FOREX_245"}'`
pub async fn verify(input: VerifyInput) -> VerifyOutput {
    let temp_dir = std::env::temp_dir().join(format!("drasus-verify-session-catalog-{}", Uuid::new_v4()));
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

    if let Err(e) = orchestrator::seed_default_templates(&pool, &clock).await {
        return VerifyOutput::from_error(format!("no se pudo sembrar SESSION_TEMPLATE_SEED_SET: {e}"));
    }

    let calendar = match crate::persistence::SessionRepository::new(&pool, &clock).find_template_by_code(&input.template_code).await {
        Ok(Some(c)) => c,
        Ok(None) => return VerifyOutput::from_error(format!("template_code '{}' no encontrado", input.template_code)),
        Err(e) => return VerifyOutput::from_error(e.to_string()),
    };

    let fingerprint = match orchestrator::expected_fingerprint(&pool, &clock, &input.template_code).await {
        Ok(f) => f,
        Err(e) => return VerifyOutput::from_error(e.to_string()),
    };

    VerifyOutput {
        ok: true,
        template_code: Some(calendar.template.template_code),
        market_timezone_ref: Some(calendar.template.market_timezone_ref),
        window_count: Some(calendar.windows.len()),
        fingerprint: Some(fingerprint),
        error: None,
    }
}

// ── Pruebas de la superficie pública ────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_input_default_value_is_golden_smoke_sentinel() {
        let input = VerifyInput::default();
        assert_eq!(input.template_code, "FOREX_245");
    }

    /// Humo de punta a punta con los defaults: debe sembrar y resolver
    /// `FOREX_245` con éxito.
    #[tokio::test(flavor = "multi_thread")]
    async fn verify_with_defaults_succeeds() {
        let output = verify(VerifyInput::default()).await;
        assert!(output.ok, "verify() con defaults debe tener éxito: {:?}", output.error);
        assert_eq!(output.template_code, Some("FOREX_245".to_string()));
        assert_eq!(output.fingerprint, Some(CalendarFingerprint::WeeklyGap));
    }

    /// Con `template_code = "FUTURES_CME_GLOBEX"`, debe resolver la
    /// tercera plantilla semilla con su huella
    /// `WeeklyGapPlusDailyMaintenance` -- ejercita el camino CLI completo
    /// para la plantilla añadida en la actualización de alcance
    /// 2026-07-25.
    #[tokio::test(flavor = "multi_thread")]
    async fn verify_resolves_futures_template_via_cli_input() {
        let input = VerifyInput { template_code: "FUTURES_CME_GLOBEX".to_string() };
        let output = verify(input).await;
        assert!(output.ok, "verify() para Futuros debe tener éxito: {:?}", output.error);
        assert_eq!(output.market_timezone_ref, Some("America/Chicago".to_string()));
        assert_eq!(output.fingerprint, Some(CalendarFingerprint::WeeklyGapPlusDailyMaintenance));
    }

    /// Un `template_code` que no existe (ni en la semilla) produce
    /// `ok: false` con mensaje explícito, no un pánico.
    #[tokio::test(flavor = "multi_thread")]
    async fn verify_with_unknown_template_code_reports_explicit_error() {
        let input = VerifyInput { template_code: "DOES_NOT_EXIST".to_string() };
        let output = verify(input).await;
        assert!(!output.ok);
        assert!(output.error.is_some());
    }
}
