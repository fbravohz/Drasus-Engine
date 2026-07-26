//! Superficie pública de `data-import-wizard` — el ÚNICO módulo `pub`.
//!
//! Expone:
//! - El puerto de salida (`OutputPorts`) con el marcador de tipo `Bars`
//!   del catálogo ADR-0137, poblado con las filas concretas que este lector
//!   produce (`CanonicalBarRow`).
//! - El orquestador de importación y el repositorio del ledger, para que el
//!   Bridge/CLI compongan el hand-off real hacia `hybrid-data-transformer`
//!   (fuera de esta feature -- ver banner de `lib.rs`).
//! - El harness de verificación CLI (`verify`, ADR-0142 Fase 1).
//!
//! Otros crates solo pueden importar desde aquí; prohibido acceder a
//! `domain`, `orchestrator`, `persistence` o `schemas` directamente.

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use shared::public_interface::{create_pool, run_migrations, SystemClock};

// ── Re-exports para que Bridge/CLI y los tests de integración accedan ──────

pub use crate::domain::{
    compute_mapping_declaration_audit_hash, compute_source_file_hash, detect_mt5_bar_template, is_activity_impossible,
    map_to_canonical_bar, mt5_bar_column_mapping, parse_mt5_bar_line, parse_price_to_scaled, scaled_to_price_string,
    CanonicalBarRow, CanonicalMappingError, FieldParseError, MappingDeclarationHashInput, Mt5LineParseError,
    Mt5RawBarFields, RunContext, TemplateDetectionError, TimestampResolutionError, MT5_BAR_TEMPLATE_CODE,
    MT5_BAR_TEMPLATE_VERSION,
};
pub use crate::orchestrator::{
    import_mt5_bar_file, ImportBarFileError, ImportBarFileInput, ImportBarFileOutcome, CANONICAL_SCHEMA_VERSION,
    TRANSFORM_ENGINE_VERSION,
};
pub use crate::persistence::{MappingDeclarationRepository, MappingDeclarationRepositoryError};
pub use crate::schemas::{MappingDeclarationRow, NewMappingDeclaration, PreviewBarRow};

// ── Puerto de salida del Canvas (marcador de tipo, ADR-0137) ────────────────

/// Puerto de salida de `data-import-wizard` hacia el Canvas: las barras ya
/// mapeadas al esquema canónico (`shared::types::Bars` es el marcador de
/// tipo de puerto; `CanonicalBarRow` es la forma concreta que esta feature
/// produce, igual que `ResolvedInstrumentTerms` en `instrument-catalog`).
pub struct OutputPorts {
    /// Puerto `bars_out` -- filas de barra ya canónicas, listas para que
    /// `hybrid-data-transformer` las escriba en Parquet (fuera de esta
    /// feature, ver banner de `lib.rs`).
    pub bars_out: Vec<CanonicalBarRow>,
}

// ── Harness de verificación CLI (ADR-0142 Fase 1) ────────────────────────────

/// Fragmento real embebido en el binario para el humo de verificación --
/// las primeras 6 filas reales de `datasets/EURUSD_M1_2000-01-01_2025-05-01.csv`
/// (recortadas dentro del repositorio, ver STORY-068 "recorta los
/// fragmentos que necesites como material de prueba").
const VERIFY_SAMPLE_CSV: &[u8] = include_bytes!("../tests/fixtures/mt5_m1_sample.csv");

/// Input para la verificación de `data-import-wizard` vía CLI. Se
/// deserializa desde el JSON que pasa el usuario con `--input '...'`.
///
/// Autocontenido: si el instrumento no existe todavía en la BD temporal de
/// verificación, `verify()` lo da de alta primero (mismo patrón que
/// `instrument-catalog::VerifyInput`) -- así `drasus verify
/// data-import-wizard` sin argumentos produce un resultado significativo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyInput {
    /// Símbolo del instrumento a importar. Default: `"EURUSD"`.
    #[serde(default = "default_instrument_id")]
    pub instrument_id: String,
    /// Bróker/venue del instrumento. Default: `"PEPPERSTONE"`.
    #[serde(default = "default_venue")]
    pub venue: String,
    /// Zona horaria IANA declarada para el archivo de muestra. Default:
    /// `Some("America/New_York")`. Pasa explícitamente `null` en el JSON
    /// para ejercitar el camino de rechazo por falta de declaración
    /// (criterio de aceptación #3 de STORY-068).
    #[serde(default = "default_source_timezone")]
    pub source_timezone_declared: Option<String>,
    /// Convención de unidad de `<VOL>` declarada. Default: `None` (no
    /// declarada) -- para ver el humo con la corrida marcada como no
    /// verificada (criterio de aceptación #8).
    #[serde(default)]
    pub real_volume_unit_declared: Option<String>,
}

fn default_instrument_id() -> String {
    "EURUSD".to_string()
}

fn default_venue() -> String {
    "PEPPERSTONE".to_string()
}

fn default_source_timezone() -> Option<String> {
    Some("America/New_York".to_string())
}

impl Default for VerifyInput {
    fn default() -> Self {
        Self {
            instrument_id: default_instrument_id(),
            venue: default_venue(),
            source_timezone_declared: default_source_timezone(),
            real_volume_unit_declared: None,
        }
    }
}

/// Output de la verificación de `data-import-wizard`. Siempre serializa a
/// JSON válido (ADR-0142).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOutput {
    pub ok: bool,
    pub rows_imported: Option<usize>,
    pub rows_flagged_activity_anomaly: Option<usize>,
    pub volume_unit_unverified: Option<bool>,
    pub was_noop_reimport: Option<bool>,
    pub mapping_declaration_id: Option<String>,
    pub mapping_declaration_event_sequence_id: Option<i64>,
    pub error: Option<String>,
}

impl VerifyOutput {
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            rows_imported: None,
            rows_flagged_activity_anomaly: None,
            volume_unit_unverified: None,
            was_noop_reimport: None,
            mapping_declaration_id: None,
            mapping_declaration_event_sequence_id: None,
            error: Some(msg),
        }
    }

    fn from_outcome(outcome: ImportBarFileOutcome) -> Self {
        let flagged = outcome.bars.iter().filter(|b| b.is_activity_anomaly).count();
        let volume_unit_unverified = outcome.bars.first().map(|b| b.volume_unit_unverified);
        Self {
            ok: true,
            rows_imported: Some(outcome.bars.len()),
            rows_flagged_activity_anomaly: Some(flagged),
            volume_unit_unverified,
            was_noop_reimport: Some(outcome.was_noop_reimport),
            mapping_declaration_id: Some(outcome.declaration.id),
            mapping_declaration_event_sequence_id: Some(outcome.declaration.event_sequence_id),
            error: None,
        }
    }
}

/// Ejecuta la verificación de `data-import-wizard` con adaptadores reales:
/// BD SQLite temporal + reloj de sistema real, contra el fragmento real
/// embebido del archivo de barras M1 de MT5. Da de alta el instrumento en
/// la BD temporal si no existe, y no llama a ningún `bar_sink` real (el
/// hand-off a `hybrid-data-transformer` es responsabilidad de la
/// composición, fuera de esta feature) -- el sink de verificación acepta
/// siempre.
///
/// Uso típico desde el CLI:
/// `drasus verify data-import-wizard --input '{"source_timezone_declared":null}'`
/// (para ver el rechazo por falta de zona horaria declarada)
pub async fn verify(input: VerifyInput) -> VerifyOutput {
    let temp_dir = std::env::temp_dir().join(format!("drasus-verify-data-import-wizard-{}", Uuid::new_v4()));
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

    // Da de alta el instrumento en `instrument_master` si no existe --
    // autocontenido, igual que `instrument-catalog::verify`. `INSERT OR
    // IGNORE` porque el instrumento pudo quedar sembrado por una corrida
    // anterior contra la misma BD (no aplica aquí, cada verify() usa un
    // directorio temporal nuevo, pero es el patrón defensivo correcto).
    let seed_id = format!("verify-instrument-{}", input.instrument_id);
    if let Err(e) = sqlx::query(
        "INSERT OR IGNORE INTO instrument_master \
         (id, created_at, updated_at, audit_hash, row_version, instrument_id, asset_class, canonical_pip_size, canonical_tick_size) \
         VALUES (?, 0, 0, 'verify-seed-hash', 1, ?, 'FOREX', 10000, 1000)",
    )
    .bind(&seed_id)
    .bind(&input.instrument_id)
    .execute(&pool)
    .await
    {
        return VerifyOutput::from_error(format!("no se pudo dar de alta el instrumento de verificación: {e}"));
    }

    let clock = SystemClock::default();
    let import_input = ImportBarFileInput {
        source_file_name: "EURUSD_M1_verify_sample.csv".to_string(),
        file_bytes: VERIFY_SAMPLE_CSV.to_vec(),
        instrument_id: input.instrument_id,
        venue: input.venue,
        source_provider: "MT5_TERMINAL_EXPORT".to_string(),
        source_timezone_declared: input.source_timezone_declared,
        real_volume_unit_declared: input.real_volume_unit_declared,
        node_id: Some(format!("drasus-verify-pid-{}", std::process::id())),
        process_id: Some(format!("drasus-pid-{}", std::process::id())),
    };

    match import_mt5_bar_file(&pool, &clock, import_input, |_rows| Ok(())).await {
        Ok(outcome) => VerifyOutput::from_outcome(outcome),
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
        assert_eq!(input.source_timezone_declared, Some("America/New_York".to_string()));
        assert!(input.real_volume_unit_declared.is_none());
    }

    /// La verificación de punta a punta (adaptadores reales, BD temporal)
    /// debe dar `ok: true` con los defaults, sobre el fragmento real
    /// embebido -- humo sin argumentos.
    #[tokio::test(flavor = "multi_thread")]
    async fn verify_with_defaults_succeeds_against_real_embedded_fragment() {
        let output = verify(VerifyInput::default()).await;
        assert!(output.ok, "verify() con defaults debe tener éxito: {:?}", output.error);
        assert_eq!(output.rows_imported, Some(6), "el fragmento embebido trae 6 filas de dato reales");
        assert_eq!(output.volume_unit_unverified, Some(true), "sin unidad declarada, debe marcarse como no verificada");
        assert_eq!(output.was_noop_reimport, Some(false));
    }

    /// Con `source_timezone_declared: null` explícito, `verify()` debe
    /// reflejar el rechazo -- ejercita el camino de falta de declaración
    /// desde el harness CLI (criterio de aceptación #3).
    #[tokio::test(flavor = "multi_thread")]
    async fn verify_with_null_timezone_reflects_rejection() {
        let input = VerifyInput { source_timezone_declared: None, ..VerifyInput::default() };
        let output = verify(input).await;
        assert!(!output.ok, "sin zona horaria declarada, verify() debe reflejar el rechazo");
        assert!(output.error.is_some());
    }
}
