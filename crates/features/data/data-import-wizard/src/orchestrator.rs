//! [SHELL] Orquestador del lector de barras M1 de MT5: cablea el parseo
//! puro del dominio con el sellado del ledger (ANTES de tocar datos) y el
//! hand-off a la persistencia de datos aguas abajo.
//!
//! El hand-off se modela como un puerto (`bar_sink`, una función que el
//! llamador provee) y NO como una dependencia de crate hacia
//! `hybrid-data-transformer` -- esta feature nunca escribe Parquet
//! directamente (`docs/features/data-import-wizard.md` "Depende de").
//! En producción, la composición (Bridge/CLI) pasa una función que reenvía
//! las filas a esa feature; en pruebas, se inyecta un cierre de prueba que
//! puede simular éxito o fallo (criterio de aceptación #5 de STORY-068).

use sqlx::SqlitePool;

use shared::public_interface::Clock;

use crate::domain::{
    self, detect_mt5_bar_template, map_to_canonical_bar, parse_mt5_bar_line, CanonicalBarRow, CanonicalMappingError,
    Mt5LineParseError, RunContext, TemplateDetectionError,
};
use crate::persistence::{MappingDeclarationRepository, MappingDeclarationRepositoryError};
use crate::schemas::{NewMappingDeclaration, PreviewBarRow};

/// Versión del motor de transformación sellada en cada declaración de
/// mapeo (LDR-0008 condición técnica 2) -- la versión del propio crate.
pub const TRANSFORM_ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Versión del esquema canónico (ADR-0155) vigente al declarar. Sube solo
/// cuando ADR-0155 cambia de forma que afecte el mapeo de columnas.
pub const CANONICAL_SCHEMA_VERSION: &str = "ADR-0155-v1";

/// Cuántas filas de dato ya transformado se sellan en el ledger como vista
/// previa (LDR-0008 condición técnica 1/3) -- mismo espíritu que
/// `PREVIEW_ROWS` de `data-import-wizard.md`, pero fijo en el Núcleo del
/// lector porque aquí no hay UI interactiva que lo haga configurable
/// todavía (esa vía es la del asistente visual, fuera de alcance de
/// STORY-068 — ver §8 de la Orden).
const PREVIEW_ROW_LIMIT: usize = 10;

/// Entrada completa para importar un archivo de barras M1 de MT5.
pub struct ImportBarFileInput {
    pub source_file_name: String,
    /// Contenido crudo del archivo, ya leído en memoria por quien orquesta
    /// (CLI/Bridge) -- esta feature nunca toca el sistema de archivos.
    ///
    /// ponytail: el archivo completo se decodifica en memoria de una sola
    /// vez; para el archivo real de 561 MB esto funciona en el hardware del
    /// propietario (Local-First, ADR-0030) pero no es óptimo. Cambiar a
    /// lectura por streaming (línea a línea) si el volumen crece a un rango
    /// donde la RAM del hardware objetivo deje de alcanzar.
    pub file_bytes: Vec<u8>,
    pub instrument_id: String,
    pub venue: String,
    pub source_provider: String,
    /// Zona horaria IANA declarada por el usuario para TODO el archivo.
    /// `None` hace fallar la importación (criterio de aceptación #3).
    pub source_timezone_declared: Option<String>,
    /// Convención de unidad de `<VOL>` declarada para este bróker/
    /// instrumento. `None` = no declarada (criterio de aceptación #8).
    pub real_volume_unit_declared: Option<String>,
    pub node_id: Option<String>,
    pub process_id: Option<String>,
}

/// Resultado de una importación exitosa (sellada o no-op de reimportación).
#[derive(Debug)]
pub struct ImportBarFileOutcome {
    pub declaration: crate::schemas::MappingDeclarationRow,
    pub bars: Vec<CanonicalBarRow>,
    /// `true` si el archivo (por contenido, `source_file_hash`) ya tenía
    /// una declaración sellada -- la importación fue un no-op: no se selló
    /// una segunda fila de ledger ni se volvió a invocar `bar_sink`
    /// (criterio de aceptación #10, ADR-0163).
    pub was_noop_reimport: bool,
}

/// Errores de una importación. Cada variante corresponde a un motivo de
/// rechazo explícito (ADR-0156 Restricción 1: nunca un error genérico).
#[derive(Debug, thiserror::Error)]
pub enum ImportBarFileError {
    #[error("el archivo está vacío: no hay ni siquiera una cabecera que leer")]
    EmptyFile,
    #[error("el archivo no es UTF-8 válido")]
    InvalidUtf8,
    #[error(transparent)]
    Template(#[from] TemplateDetectionError),
    #[error("error al parsear la línea de dato número {line_number}: {source}")]
    Line {
        line_number: usize,
        #[source]
        source: Mt5LineParseError,
    },
    #[error("error al mapear la línea de dato número {line_number} al esquema canónico: {source}")]
    Mapping {
        line_number: usize,
        #[source]
        source: CanonicalMappingError,
    },
    #[error(transparent)]
    Repository(#[from] MappingDeclarationRepositoryError),
    #[error("la persistencia de datos aguas abajo falló DESPUÉS de sellar el ledger: {0}")]
    Sink(String),
}

/// Importa un archivo de barras M1 de MT5 completo: detecta la plantilla,
/// parsea y mapea cada fila, sella la declaración de mapeo en el ledger
/// (ANTES de invocar `bar_sink`), y entrega las filas a `bar_sink` para su
/// persistencia aguas abajo.
///
/// Si el archivo ya fue importado antes (mismo contenido, mismo
/// `source_file_hash`), la función es un no-op: no vuelve a sellar el
/// ledger ni a invocar `bar_sink`, y devuelve la declaración existente.
///
/// Si `bar_sink` falla, la función devuelve `Err(Sink(..))` pero la
/// declaración YA quedó sellada y es recuperable -- verificable
/// consultando la base de datos directamente (criterio de aceptación #5).
pub async fn import_mt5_bar_file<F>(
    pool: &SqlitePool,
    clock: &dyn Clock,
    input: ImportBarFileInput,
    bar_sink: F,
) -> Result<ImportBarFileOutcome, ImportBarFileError>
where
    F: FnOnce(&[CanonicalBarRow]) -> Result<(), String>,
{
    let content = std::str::from_utf8(&input.file_bytes).map_err(|_| ImportBarFileError::InvalidUtf8)?;
    let mut lines = content.split('\n');

    let header = lines.next().ok_or(ImportBarFileError::EmptyFile)?;
    detect_mt5_bar_template(header)?;

    let source_file_hash = domain::compute_source_file_hash(&input.file_bytes);

    // ── Idempotencia (ADR-0163, criterio de aceptación #10) ─────────────
    // Si este archivo (por contenido) ya tiene una declaración sellada, la
    // importación es un no-op silencioso sin fricción -- ni un segundo
    // sello de ledger, ni una segunda invocación de `bar_sink`.
    let repository = MappingDeclarationRepository::new(pool, clock);
    if let Some(existing) = repository.find_by_source_file_hash(&source_file_hash).await? {
        return Ok(ImportBarFileOutcome { declaration: existing, bars: Vec::new(), was_noop_reimport: true });
    }

    // ── Parseo y mapeo (Núcleo puro) ─────────────────────────────────────
    // Nunca una importación parcial: el primer error de parseo o de mapeo
    // rechaza el archivo completo, sin sellar ni persistir nada.
    // La unidad del spread reportado la afirma la PLANTILLA, no el usuario
    // (criterio de aceptación #13, LDR-0008) -- se puebla siempre desde la
    // plantilla MT5 vigente, nunca desde `input`.
    let spread_reported_unit_declared = domain::mt5_bar_spread_reported_unit();
    let ctx = RunContext {
        instrument_id: &input.instrument_id,
        venue: &input.venue,
        source_provider: &input.source_provider,
        source_timezone_declared: input.source_timezone_declared.as_deref(),
        real_volume_unit_declared: input.real_volume_unit_declared.as_deref(),
        spread_reported_unit_declared: Some(spread_reported_unit_declared),
    };

    let mut bars = Vec::new();
    for (idx, raw_line) in lines.enumerate() {
        // El `split('\n')` final tras el último salto de línea produce una
        // cadena vacía -- se ignora, no es una fila de dato.
        if raw_line.is_empty() {
            continue;
        }
        let line_number = idx + 1; // 1-based, cuenta solo filas de dato (sin la cabecera)
        let raw_fields =
            parse_mt5_bar_line(raw_line).map_err(|source| ImportBarFileError::Line { line_number, source })?;
        let bar =
            map_to_canonical_bar(&raw_fields, &ctx).map_err(|source| ImportBarFileError::Mapping { line_number, source })?;
        bars.push(bar);
    }

    // ── Construcción de la declaración de mapeo ──────────────────────────
    let column_mapping_json =
        serde_json::to_string(&domain::mt5_bar_column_mapping()).expect("un Vec<(String,String)> siempre serializa a JSON");
    let preview_rows: Vec<PreviewBarRow> = bars.iter().take(PREVIEW_ROW_LIMIT).map(PreviewBarRow::from).collect();
    let preview_transformed_rows_json =
        serde_json::to_string(&preview_rows).expect("un Vec<PreviewBarRow> siempre serializa a JSON");
    let accepted_at = clock.timestamp_ns();

    let hash_input = domain::MappingDeclarationHashInput {
        source_file_name: &input.source_file_name,
        source_file_hash: &source_file_hash,
        instrument_id: &input.instrument_id,
        venue: &input.venue,
        mapping_origin: "PLATFORM_TEMPLATE",
        platform_template_code: Some(domain::MT5_BAR_TEMPLATE_CODE),
        platform_template_version: Some(domain::MT5_BAR_TEMPLATE_VERSION),
        column_mapping_json: &column_mapping_json,
        transform_engine_version: TRANSFORM_ENGINE_VERSION,
        canonical_schema_version: CANONICAL_SCHEMA_VERSION,
        accepted_at,
        preview_transformed_rows_json: &preview_transformed_rows_json,
        source_timezone_declared: input.source_timezone_declared.as_deref().unwrap_or_default(),
        real_volume_unit_declared: input.real_volume_unit_declared.as_deref(),
        spread_reported_unit_declared,
    };
    let content_audit_hash = domain::compute_mapping_declaration_audit_hash(&hash_input);

    let new_declaration = NewMappingDeclaration {
        content_audit_hash,
        source_file_name: input.source_file_name.clone(),
        source_file_hash,
        instrument_id: input.instrument_id.clone(),
        venue: input.venue.clone(),
        bar_type: "TIME".to_string(),
        timeframe_value: "1M".to_string(),
        mapping_origin: "PLATFORM_TEMPLATE".to_string(),
        platform_template_code: Some(domain::MT5_BAR_TEMPLATE_CODE.to_string()),
        platform_template_version: Some(domain::MT5_BAR_TEMPLATE_VERSION.to_string()),
        column_mapping_json,
        transform_engine_version: TRANSFORM_ENGINE_VERSION.to_string(),
        canonical_schema_version: CANONICAL_SCHEMA_VERSION.to_string(),
        accepted_at,
        preview_transformed_rows_json,
        source_timezone_declared: input.source_timezone_declared.clone().unwrap_or_default(),
        real_volume_unit_declared: input.real_volume_unit_declared.clone(),
        spread_reported_unit_declared: spread_reported_unit_declared.to_string(),
        node_id: input.node_id.clone(),
        process_id: input.process_id.clone(),
    };

    // ── Sellado del ledger -- ANTES de persistir cualquier dato ──────────
    // Si esto falla, no se llegó a invocar bar_sink -- nada quedó a medias.
    let declaration = repository.record(new_declaration).await?;

    // ── Hand-off a la persistencia de datos aguas abajo ──────────────────
    // Si esto falla, la declaración YA quedó sellada y comprometida --
    // recuperable consultando la base de datos, aunque esta función
    // devuelva Err (criterio de aceptación #5).
    bar_sink(&bars).map_err(ImportBarFileError::Sink)?;

    Ok(ImportBarFileOutcome { declaration, bars, was_noop_reimport: false })
}

// ═══════════════════════════════════════════════════════════════════════
// Pruebas de integración del orquestador -- Capa 2 (persistencia real,
// SQLite en archivo temporal).
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use shared::public_interface::{create_pool, run_migrations, DeterministicClock};
    use std::sync::atomic::{AtomicUsize, Ordering};

    async fn temp_pool_with_eurusd() -> (SqlitePool, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("no se pudo crear directorio temporal");
        let db_path = dir.path().join("test.db");
        let url = format!("sqlite://{}", db_path.display());
        let pool = create_pool(&url).await.expect("no se pudo crear el pool de prueba");
        run_migrations(&pool).await.expect("las migraciones deben aplicar sin error");
        sqlx::query(
            "INSERT INTO instrument_master (id, created_at, updated_at, audit_hash, row_version, \
             instrument_id, asset_class, canonical_pip_size, canonical_tick_size) \
             VALUES ('instrument-eurusd', 0, 0, 'seed-hash', 1, 'EURUSD', 'FOREX', 10000, 1000)",
        )
        .execute(&pool)
        .await
        .expect("la fila semilla de instrument_master debe insertarse");
        (pool, dir)
    }

    fn sample_input(file_bytes: Vec<u8>) -> ImportBarFileInput {
        ImportBarFileInput {
            source_file_name: "EURUSD_M1_sample.csv".to_string(),
            file_bytes,
            instrument_id: "EURUSD".to_string(),
            venue: "PEPPERSTONE".to_string(),
            source_provider: "MT5_TERMINAL_EXPORT".to_string(),
            source_timezone_declared: Some("America/New_York".to_string()),
            real_volume_unit_declared: None,
            node_id: Some("test-node".to_string()),
            process_id: Some("test-process".to_string()),
        }
    }

    const REAL_SAMPLE_CSV: &[u8] = include_bytes!("../tests/fixtures/mt5_m1_sample.csv");
    const REAL_ANOMALY_CSV: &[u8] = include_bytes!("../tests/fixtures/mt5_m1_activity_anomaly.csv");
    const UNRECOGNIZED_HEADER_CSV: &[u8] = include_bytes!("../tests/fixtures/unrecognized_header.csv");

    /// Criterio de aceptación #4: un archivo sin plantilla reconocida se
    /// rechaza, sin sellar el ledger ni persistir ni una fila.
    #[tokio::test(flavor = "multi_thread")]
    async fn unrecognized_header_is_rejected_with_zero_rows_persisted() {
        let (pool, _dir) = temp_pool_with_eurusd().await;
        let clock = DeterministicClock::new(0, 1);

        let result = import_mt5_bar_file(&pool, &clock, sample_input(UNRECOGNIZED_HEADER_CSV.to_vec()), |_rows| Ok(())).await;

        assert!(matches!(result, Err(ImportBarFileError::Template(_))), "cabecera no reconocida debe rechazarse");

        let repository = MappingDeclarationRepository::new(&pool, &clock);
        let declarations = repository.list_all().await.unwrap();
        assert!(declarations.is_empty(), "cero filas de ledger tras un rechazo de plantilla");
    }

    /// Criterio de aceptación #5: si la persistencia de datos aguas abajo
    /// falla, la declaración del ledger YA quedó sellada y es recuperable.
    #[tokio::test(flavor = "multi_thread")]
    async fn ledger_declaration_survives_when_downstream_data_sink_fails() {
        let (pool, _dir) = temp_pool_with_eurusd().await;
        let clock = DeterministicClock::new(0, 1);

        let result = import_mt5_bar_file(&pool, &clock, sample_input(REAL_SAMPLE_CSV.to_vec()), |_rows| {
            Err("fallo simulado de la persistencia de datos aguas abajo".to_string())
        })
        .await;

        assert!(matches!(result, Err(ImportBarFileError::Sink(_))), "debe propagar el fallo del sink");

        // A pesar del Err de arriba, la declaración YA está en la base de
        // datos -- se consulta directamente, sin pasar por el valor de
        // retorno de la función que falló.
        let repository = MappingDeclarationRepository::new(&pool, &clock);
        let declarations = repository.list_all().await.unwrap();
        assert_eq!(declarations.len(), 1, "la declaración del ledger debe sobrevivir al fallo del sink aguas abajo");
    }

    /// Criterio de aceptación #10: reimportar el mismo tramo idéntico es un
    /// no-op -- no duplica el ledger ni vuelve a invocar `bar_sink`.
    #[tokio::test(flavor = "multi_thread")]
    async fn reimporting_identical_file_is_a_noop_with_stable_count_and_no_second_sink_call() {
        let (pool, _dir) = temp_pool_with_eurusd().await;
        let clock = DeterministicClock::new(0, 1);
        let sink_call_count = std::sync::Arc::new(AtomicUsize::new(0));

        let first_sink_count = std::sync::Arc::clone(&sink_call_count);
        let first = import_mt5_bar_file(&pool, &clock, sample_input(REAL_SAMPLE_CSV.to_vec()), move |_rows| {
            first_sink_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .await
        .expect("la primera importación debe tener éxito");
        assert!(!first.was_noop_reimport);

        let second_sink_count = std::sync::Arc::clone(&sink_call_count);
        let second = import_mt5_bar_file(&pool, &clock, sample_input(REAL_SAMPLE_CSV.to_vec()), move |_rows| {
            second_sink_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .await
        .expect("reimportar el mismo contenido debe tratarse como no-op, no como error");

        assert!(second.was_noop_reimport, "la segunda importación del mismo archivo debe marcarse como no-op");
        assert_eq!(sink_call_count.load(Ordering::SeqCst), 1, "bar_sink NO debe volver a invocarse en el no-op");

        let repository = MappingDeclarationRepository::new(&pool, &clock);
        let declarations = repository.list_all().await.unwrap();
        assert_eq!(declarations.len(), 1, "no debe existir una segunda fila de ledger para el mismo contenido");
        assert_eq!(declarations[0].id, first.declaration.id, "debe ser la MISMA declaración, no una nueva");
    }

    /// Criterio de aceptación #9: las dos filas reales del 2019-11-01 con
    /// imposibilidad física, leídas de punta a punta por el orquestador,
    /// llegan marcadas hasta `bar_sink` -- no se descartan ni se persisten
    /// como velas ordinarias sin marca.
    #[tokio::test(flavor = "multi_thread")]
    async fn real_2019_activity_anomaly_rows_reach_bar_sink_flagged() {
        let (pool, _dir) = temp_pool_with_eurusd().await;
        let clock = DeterministicClock::new(0, 1);

        let outcome = import_mt5_bar_file(&pool, &clock, sample_input(REAL_ANOMALY_CSV.to_vec()), |_rows| Ok(()))
            .await
            .expect("el fragmento real de 2019-11-01 debe importar sin error");

        let flagged_count = outcome.bars.iter().filter(|b| b.is_activity_anomaly).count();
        assert_eq!(flagged_count, 2, "el fragmento trae exactamente 2 filas imposibles reales (16:58 y 17:01)");
        assert_eq!(outcome.bars.len(), 8, "el fragmento completo (8 filas de dato) debe persistirse entero, sin descartar ninguna");
    }

    /// Criterio de aceptación #3: sin zona horaria declarada, la
    /// importación completa se rechaza -- cero filas, cero ledger.
    #[tokio::test(flavor = "multi_thread")]
    async fn import_without_declared_timezone_is_rejected_end_to_end() {
        let (pool, _dir) = temp_pool_with_eurusd().await;
        let clock = DeterministicClock::new(0, 1);
        let mut input = sample_input(REAL_SAMPLE_CSV.to_vec());
        input.source_timezone_declared = None;

        let result = import_mt5_bar_file(&pool, &clock, input, |_rows| Ok(())).await;
        assert!(matches!(result, Err(ImportBarFileError::Mapping { .. })), "debe rechazar por falta de zona horaria declarada");

        let repository = MappingDeclarationRepository::new(&pool, &clock);
        assert!(repository.list_all().await.unwrap().is_empty(), "sin zona declarada, cero filas de ledger");
    }

    // ── Gate de mutación, Grupo 1 (parte de integración): la plantilla ──
    // sellada en el ledger es la real, no un JSON cualquiera.

    #[tokio::test(flavor = "multi_thread")]
    async fn sealed_ledger_declaration_contains_the_real_mt5_column_mapping() {
        let (pool, _dir) = temp_pool_with_eurusd().await;
        let clock = DeterministicClock::new(0, 1);
        let outcome =
            import_mt5_bar_file(&pool, &clock, sample_input(REAL_SAMPLE_CSV.to_vec()), |_rows| Ok(())).await.unwrap();

        let parsed: Vec<(String, String)> = serde_json::from_str(&outcome.declaration.column_mapping_json).unwrap();
        assert_eq!(
            parsed,
            vec![
                ("<DATE>".to_string(), "bar_date_source".to_string()),
                ("<TIME>".to_string(), "bar_time_source".to_string()),
                ("<OPEN>".to_string(), "open".to_string()),
                ("<HIGH>".to_string(), "high".to_string()),
                ("<LOW>".to_string(), "low".to_string()),
                ("<CLOSE>".to_string(), "close".to_string()),
                ("<TICKVOL>".to_string(), "volume_ticks".to_string()),
                ("<VOL>".to_string(), "volume_traded".to_string()),
                ("<SPREAD>".to_string(), "spread_reported".to_string()),
            ],
            "el JSON sellado en el ledger debe contener EXACTAMENTE la plantilla real, no un vector vacío ni pares basura"
        );
    }

    // ── Criterio de aceptación #13 (parte de integración): la unidad del
    // spread queda sellada en la declaración de mapeo ──────────────────

    #[tokio::test(flavor = "multi_thread")]
    async fn sealed_ledger_declaration_records_the_spread_unit_as_points_not_pips() {
        let (pool, _dir) = temp_pool_with_eurusd().await;
        let clock = DeterministicClock::new(0, 1);
        let outcome =
            import_mt5_bar_file(&pool, &clock, sample_input(REAL_SAMPLE_CSV.to_vec()), |_rows| Ok(())).await.unwrap();

        assert_eq!(
            outcome.declaration.spread_reported_unit_declared, "POINTS_MT5",
            "la declaración sellada debe registrar la unidad del spread, no dejarla implícita en el nombre de un campo"
        );
    }

    // ── Gate de mutación, Grupo 6 (parte de integración): el rechazo por
    // duplicado se dispara por CONTENIDO, no por nombre de archivo ──────

    #[tokio::test(flavor = "multi_thread")]
    async fn reimport_is_detected_by_content_hash_not_by_file_name() {
        let (pool, _dir) = temp_pool_with_eurusd().await;
        let clock = DeterministicClock::new(0, 1);

        let mut first_input = sample_input(REAL_SAMPLE_CSV.to_vec());
        first_input.source_file_name = "primer_nombre.csv".to_string();
        let first = import_mt5_bar_file(&pool, &clock, first_input, |_rows| Ok(())).await.unwrap();
        assert!(!first.was_noop_reimport);

        // MISMO contenido de bytes, nombre de archivo COMPLETAMENTE distinto.
        let mut second_input = sample_input(REAL_SAMPLE_CSV.to_vec());
        second_input.source_file_name = "nombre_completamente_distinto_del_primero.csv".to_string();
        let second = import_mt5_bar_file(&pool, &clock, second_input, |_rows| Ok(())).await.unwrap();

        assert!(second.was_noop_reimport, "mismo CONTENIDO con nombre de archivo distinto debe detectarse como duplicado");
        assert_eq!(second.declaration.id, first.declaration.id, "debe ser la MISMA declaración, identificada por contenido");
    }

    // ── Gate de mutación, Grupo 7: aritmética del orquestador ────────────

    /// Mata `orchestrator.rs:160` (`+` → `*` en `idx + 1`): construye un
    /// archivo con una línea malformada en una posición CONOCIDA (la 3a
    /// fila de dato) y afirma el número de línea EXACTO que reporta el
    /// error. Bajo el mutante, `idx * 1 == idx`, así que la 3a fila
    /// (idx=2) reportaría línea 2, no 3.
    #[tokio::test(flavor = "multi_thread")]
    async fn parse_error_reports_the_correct_one_based_line_number() {
        let (pool, _dir) = temp_pool_with_eurusd().await;
        let clock = DeterministicClock::new(0, 1);
        let header = "<DATE>\t<TIME>\t<OPEN>\t<HIGH>\t<LOW>\t<CLOSE>\t<TICKVOL>\t<VOL>\t<SPREAD>\r\n";
        let good_line = "2000.01.03\t00:01:00\t1.00730\t1.00750\t1.00730\t1.00730\t5\t0\t50\r\n";
        let bad_line = "2000.01.03\t00:02:00\tNOPE\t1.00790\t1.00790\t1.00790\t1\t0\t50\r\n"; // <OPEN> inválido
        let content = format!("{header}{good_line}{good_line}{bad_line}");

        let result = import_mt5_bar_file(&pool, &clock, sample_input(content.into_bytes()), |_rows| Ok(())).await;

        match result {
            Err(ImportBarFileError::Line { line_number, .. }) => {
                assert_eq!(line_number, 3, "la 3a fila de dato (tras 2 filas buenas) debe reportarse como la línea 3, no la 2")
            }
            other => panic!("se esperaba ImportBarFileError::Line, se obtuvo {other:?}"),
        }
    }
}
