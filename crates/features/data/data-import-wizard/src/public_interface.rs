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
    check_declared_range_against_parsed_content, compute_mapping_declaration_audit_hash, compute_source_file_hash,
    detect_csv_columns, detect_csv_delimiter, detect_mt5_bar_template, is_activity_impossible,
    map_manual_row_to_canonical_bar, map_to_canonical_bar, mt5_bar_column_mapping, parse_and_map_manual_bar_file,
    parse_generic_csv_header, parse_generic_timestamp, parse_manual_csv_row, parse_mt5_bar_line, parse_price_to_scaled,
    preview_manual_bar_file, resolve_manual_mapping, scaled_to_price_string, CanonicalBarField, CanonicalBarRow,
    CanonicalMappingError, DeclaredRangeContentMismatchError, FieldParseError, GenericTimestampParseError,
    ManualColumnMapping, ManualImportCoreError, ManualMappingError, ManualRawBarFields, ManualRowParseError,
    ManualRunContext, MappingDeclarationHashInput, Mt5LineParseError, Mt5RawBarFields, ResolvedManualMapping,
    RunContext, TemplateDetectionError, TimestampResolutionError, MT5_BAR_TEMPLATE_CODE, MT5_BAR_TEMPLATE_VERSION,
    PREVIEW_ROW_LIMIT,
};
pub use crate::orchestrator::{
    import_manual_bar_file, import_mt5_bar_file, ImportBarFileError, ImportBarFileInput, ImportBarFileOutcome,
    ManualImportBarFileError, ManualImportBarFileInput, ManualImportBarFileOutcome, CANONICAL_SCHEMA_VERSION,
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
    /// Rango declarado del tramo que cubre el fragmento embebido (ADR-0162
    /// "Enmienda 2026-07-26") -- por defecto, exactamente el día real que
    /// cubre `VERIFY_SAMPLE_CSV` (2000-01-03 UTC).
    #[serde(default = "default_declared_range_start_ns")]
    pub declared_range_start_ns: i64,
    #[serde(default = "default_declared_range_end_ns")]
    pub declared_range_end_ns: i64,
    /// Zona ancla IANA del reloj de origen declarado para el archivo de
    /// muestra. Default: `Some("America/New_York")`. Pasa explícitamente
    /// `null` en el JSON para ejercitar el camino de rechazo por falta de
    /// declaración (criterio de aceptación #3 de STORY-068).
    #[serde(default = "default_source_clock_anchor_timezone_ref")]
    pub source_clock_anchor_timezone_ref: Option<String>,
    /// Desplazamiento fijo en minutos frente al ancla (ADR-0162 "Enmienda
    /// 2026-07-26"). Default: `0` -- reproduce sin pérdida la declaración
    /// plana de zona de hoy. Pasa un valor distinto de cero (ej. `420` para
    /// reproducir el reloj de IC Markets) para ejercitar el mecanismo nuevo.
    #[serde(default)]
    pub source_clock_fixed_offset_minutes: i64,
    /// Convención de unidad de `<VOL>` declarada. Default: `None` (no
    /// declarada) -- para ver el humo con la corrida marcada como no
    /// verificada (criterio de aceptación #8).
    #[serde(default)]
    pub real_volume_unit_declared: Option<String>,
    /// Fecha de operación declarada del bróker (ADR-0162 "Enmienda
    /// 2026-07-26"). Default: `None` -- el chequeo de fecha de operación no
    /// corre. Pasa un instante posterior a `declared_range_start_ns` para
    /// ejercitar el camino de rechazo del Gate de Homogeneidad.
    #[serde(default)]
    pub venue_operational_since_ns: Option<i64>,
}

fn default_instrument_id() -> String {
    "EURUSD".to_string()
}

fn default_venue() -> String {
    "PEPPERSTONE".to_string()
}

/// 2000-01-03T00:00:00Z en nanosegundos UTC -- inicio del día real que
/// cubre `VERIFY_SAMPLE_CSV`.
fn default_declared_range_start_ns() -> i64 {
    946_857_600_000_000_000
}

/// 2000-01-04T00:00:00Z en nanosegundos UTC -- fin exclusivo del mismo día.
fn default_declared_range_end_ns() -> i64 {
    946_944_000_000_000_000
}

fn default_source_clock_anchor_timezone_ref() -> Option<String> {
    Some("America/New_York".to_string())
}

impl Default for VerifyInput {
    fn default() -> Self {
        Self {
            instrument_id: default_instrument_id(),
            venue: default_venue(),
            declared_range_start_ns: default_declared_range_start_ns(),
            declared_range_end_ns: default_declared_range_end_ns(),
            source_clock_anchor_timezone_ref: default_source_clock_anchor_timezone_ref(),
            source_clock_fixed_offset_minutes: 0,
            real_volume_unit_declared: None,
            venue_operational_since_ns: None,
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
/// `drasus verify data-import-wizard --input '{"source_clock_anchor_timezone_ref":null}'`
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
        declared_range_start_ns: input.declared_range_start_ns,
        declared_range_end_ns: input.declared_range_end_ns,
        source_clock_anchor_timezone_ref: input.source_clock_anchor_timezone_ref,
        source_clock_fixed_offset_minutes: input.source_clock_fixed_offset_minutes,
        real_volume_unit_declared: input.real_volume_unit_declared,
        venue_operational_since_ns: input.venue_operational_since_ns,
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
        assert_eq!(input.source_clock_anchor_timezone_ref, Some("America/New_York".to_string()));
        assert_eq!(input.source_clock_fixed_offset_minutes, 0);
        assert!(input.real_volume_unit_declared.is_none());
        assert!(input.venue_operational_since_ns.is_none());
        assert!(
            input.declared_range_end_ns > input.declared_range_start_ns,
            "el rango declarado por defecto debe ser válido (fin > inicio)"
        );
    }

    /// Hallazgo del gate de mutación del Tech-Lead sobre STORY-070:
    /// `default_declared_range_start_ns()` podía mutarse a `0`/`1`/`-1` sin
    /// que ninguna prueba se enterara -- el rango seguía siendo
    /// sintácticamente válido (fin > inicio) y las filas de la muestra caían
    /// igual dentro de un rango más ancho, así que nada discriminaba el
    /// mutante. Esta prueba cierra el hueco comparando el default contra un
    /// ORÁCULO derivado de las fechas REALES de las filas del fragmento
    /// embebido -- nunca repitiendo la constante literal que ya vive en la
    /// función bajo prueba (esa repetición no discrimina nada: una prueba y
    /// el código mutarían juntos si alguien copia el número mal una vez).
    #[test]
    fn default_declared_range_matches_exactly_the_real_day_covered_by_the_sample_csv() {
        use chrono::{Datelike, Duration, TimeZone, Utc};

        // Parsea el CSV real embebido y extrae la fecha de CADA fila de dato
        // (sin la cabecera) -- verificación independiente del contenido real,
        // nunca una suposición sobre qué día "debería" cubrir el fragmento.
        let content = std::str::from_utf8(VERIFY_SAMPLE_CSV).expect("el fragmento embebido debe ser UTF-8 válido");
        let mut lines = content.split('\n');
        lines.next(); // descarta la cabecera
        let dates: Vec<_> = lines
            .filter(|l| !l.is_empty())
            .map(|line| parse_mt5_bar_line(line).expect("cada línea real del fragmento debe parsear").date)
            .collect();
        assert!(!dates.is_empty(), "el fragmento de muestra debe traer al menos una fila de dato real");

        // El fragmento cubre un ÚNICO día calendario -- si esto deja de ser
        // cierto, el resto de esta prueba (que asume un solo día) pierde su
        // premisa y debe revisarse antes que el código de producción.
        let min_date = *dates.iter().min().unwrap();
        let max_date = *dates.iter().max().unwrap();
        assert_eq!(min_date, max_date, "esta prueba asume que el fragmento de muestra cubre un solo día calendario");

        // Oráculo: medianoche UTC del día real, y el día siguiente -- ambos
        // calculados a mano con `chrono`, sin pasar por las constantes de
        // `public_interface.rs`.
        let expected_start = Utc.with_ymd_and_hms(min_date.year(), min_date.month(), min_date.day(), 0, 0, 0).unwrap();
        let expected_end = expected_start + Duration::days(1);

        assert_eq!(
            default_declared_range_start_ns(),
            expected_start.timestamp_nanos_opt().unwrap(),
            "el inicio del rango por defecto debe ser EXACTAMENTE la medianoche UTC del día real que cubre el \
             fragmento de muestra -- no un valor arbitrario ni un cero que declararía un tramo desde 1970"
        );
        assert_eq!(
            default_declared_range_end_ns(),
            expected_end.timestamp_nanos_opt().unwrap(),
            "el fin del rango por defecto debe ser EXACTAMENTE el día siguiente al día real que cubre la muestra"
        );
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

    /// Con `source_clock_anchor_timezone_ref: null` explícito, `verify()`
    /// debe reflejar el rechazo -- ejercita el camino de falta de
    /// declaración desde el harness CLI (criterio de aceptación #3).
    #[tokio::test(flavor = "multi_thread")]
    async fn verify_with_null_timezone_reflects_rejection() {
        let input = VerifyInput { source_clock_anchor_timezone_ref: None, ..VerifyInput::default() };
        let output = verify(input).await;
        assert!(!output.ok, "sin reloj de origen declarado, verify() debe reflejar el rechazo");
        assert!(output.error.is_some());
    }

    /// Criterio de aceptación #11: el harness CLI acepta la FORMA NUEVA
    /// completa de la declaración (ancla + desplazamiento distinto de cero,
    /// estilo IC Markets) y sigue teniendo éxito de punta a punta.
    ///
    /// El rango declarado por defecto (`default_declared_range_*`) está
    /// calibrado EXACTAMENTE al día real que cubre la muestra bajo
    /// desplazamiento CERO (ver el test golden de arriba) -- un
    /// desplazamiento de 7 horas mueve el instante resuelto de cada fila al
    /// día calendario ANTERIOR, así que el rango por defecto, sin ensanchar,
    /// dejaría el contenido real fuera del tramo declarado y el cotejo de
    /// rango (DEBT-039, STORY-073) lo rechazaría -- correctamente: sería una
    /// declaración que ya no describe lo que este `verify()` produce. Se
    /// ensancha un día completo a cada lado para aislar lo que esta prueba
    /// SÍ quiere ejercer (la resolución del desplazamiento), sin relajar el
    /// cotejo de rango en sí.
    #[tokio::test(flavor = "multi_thread")]
    async fn verify_with_ic_markets_style_offset_still_succeeds() {
        const ONE_DAY_NS: i64 = 86_400_000_000_000;
        let default_input = VerifyInput::default();
        let input = VerifyInput {
            source_clock_fixed_offset_minutes: 420,
            declared_range_start_ns: default_input.declared_range_start_ns - ONE_DAY_NS,
            declared_range_end_ns: default_input.declared_range_end_ns + ONE_DAY_NS,
            ..default_input
        };
        let output = verify(input).await;
        assert!(output.ok, "un desplazamiento distinto de cero debe seguir resolviendo el reloj sin error: {:?}", output.error);
        assert_eq!(output.rows_imported, Some(6));
    }

    /// Criterio de aceptación #7: `verify()` refleja el rechazo del Gate de
    /// Homogeneidad cuando el tramo por defecto antecede una fecha de
    /// operación declarada del bróker.
    #[tokio::test(flavor = "multi_thread")]
    async fn verify_reflects_rejection_when_declared_range_precedes_venue_operational_date() {
        let default_input = VerifyInput::default();
        let input =
            VerifyInput { venue_operational_since_ns: Some(default_input.declared_range_start_ns + 1), ..default_input };
        let output = verify(input).await;
        assert!(!output.ok, "un tramo que antecede la fecha de operación declarada debe reflejarse como rechazo");
        assert!(output.error.is_some());
    }
}
