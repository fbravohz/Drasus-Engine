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
    self, check_declared_range_against_parsed_content, check_declared_range_against_venue_operational_date,
    detect_mt5_bar_template, map_to_canonical_bar, parse_and_map_manual_bar_file, parse_mt5_bar_line, CanonicalBarRow,
    CanonicalMappingError, DeclaredRangeContentMismatchError, ManualColumnMapping, ManualImportCoreError,
    ManualRunContext, Mt5LineParseError, ProvenanceHomogeneityError, RunContext, TemplateDetectionError,
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
/// `PREVIEW_ROWS` de `data-import-wizard.md`. Reexportada desde
/// `domain::PREVIEW_ROW_LIMIT` (STORY-073, ajuste de rendimiento
/// 2026-07-31): una sola fuente de verdad, porque ahora el Núcleo TAMBIÉN
/// acota su propio parseo a este mismo número en `preview_manual_bar_file`
/// -- si vivieran duplicadas, podrían divergir en silencio.
use domain::PREVIEW_ROW_LIMIT;

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
    /// Rango declarado del TRAMO que esta importación cubre (ADR-0162
    /// "Enmienda 2026-07-26") -- nanosegundos UTC, intervalo semiabierto
    /// [inicio, fin). EXPLÍCITO: quien orquesta lo declara, nunca se deriva
    /// escaneando el mínimo/máximo de las filas que este lector produce.
    pub declared_range_start_ns: i64,
    pub declared_range_end_ns: i64,
    /// Zona ancla IANA del reloj de origen declarado por el usuario para
    /// TODO el tramo. `None` hace fallar la importación (criterio de
    /// aceptación #3 de STORY-068).
    pub source_clock_anchor_timezone_ref: Option<String>,
    /// Desplazamiento fijo en minutos frente al ancla (ADR-0162 "Enmienda
    /// 2026-07-26"). `0` reproduce sin pérdida la declaración plana de zona
    /// de hoy -- irrelevante cuando el ancla es `None`.
    pub source_clock_fixed_offset_minutes: i64,
    /// Convención de unidad de `<VOL>` declarada para este bróker/
    /// instrumento. `None` = no declarada (criterio de aceptación #8).
    pub real_volume_unit_declared: Option<String>,
    /// Fecha de operación declarada del bróker/venue
    /// (`broker_server_clock.venue_operational_since_ns`, hecho externo
    /// inyectado por quien orquesta -- esta feature nunca consulta esa
    /// tabla directamente, ADR-0137). `None` = no verificada; el chequeo de
    /// fecha de operación del Gate de Homogeneidad de Procedencia NO corre
    /// y no se asume ninguna fecha (ADR-0162 "Enmienda 2026-07-26").
    pub venue_operational_since_ns: Option<i64>,
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
    /// Rechazo del Gate de Homogeneidad de Procedencia (ADR-0162 "Enmienda
    /// 2026-07-26") -- el tramo declarado antecede la fecha de operación
    /// declarada del bróker/venue. Se comprueba ANTES de parsear el archivo
    /// (fail-fast): un tramo imposible no necesita gastar el parseo.
    #[error(transparent)]
    ProvenanceHomogeneity(#[from] ProvenanceHomogeneityError),
    /// Rechazo del cotejo elemental de rango (DEBT-039, STORY-073) -- el
    /// rango declarado no cubre lo que el archivo realmente contiene. Se
    /// comprueba DESPUÉS de parsear (necesita el mínimo/máximo real de las
    /// filas), pero ANTES de sellar cualquier fila del ledger.
    #[error(transparent)]
    DeclaredRangeContentMismatch(#[from] DeclaredRangeContentMismatchError),
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

    // ── Idempotencia (ADR-0163, criterio de aceptación #10; ── grano
    // relajado a TRAMO en ADR-0162 "Enmienda 2026-07-26", criterio de
    // aceptación #5 de STORY-070). Si este TRAMO exacto (mismo contenido de
    // archivo, mismo rango declarado) ya tiene una declaración sellada, la
    // importación es un no-op silencioso sin fricción -- ni un segundo
    // sello de ledger, ni una segunda invocación de `bar_sink`.
    let repository = MappingDeclarationRepository::new(pool, clock);
    if let Some(existing) = repository
        .find_by_source_file_hash_and_range(&source_file_hash, input.declared_range_start_ns, input.declared_range_end_ns)
        .await?
    {
        return Ok(ImportBarFileOutcome { declaration: existing, bars: Vec::new(), was_noop_reimport: true });
    }

    // ── Gate de Homogeneidad de Procedencia -- chequeo de fecha de ──────
    // operación (ADR-0162 "Enmienda 2026-07-26", criterio de aceptación #7
    // de STORY-070). Corre ANTES de parsear: un tramo imposible no necesita
    // gastar el parseo del archivo completo. Detectar la fractura no es
    // inferir su causa -- solo compara dos instantes (ver doc del dominio).
    check_declared_range_against_venue_operational_date(
        &input.venue,
        input.declared_range_start_ns,
        input.venue_operational_since_ns,
    )?;

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
        source_clock_anchor_timezone_ref: input.source_clock_anchor_timezone_ref.as_deref(),
        source_clock_fixed_offset_minutes: input.source_clock_fixed_offset_minutes,
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

    // ── Cotejo elemental del rango declarado contra el contenido real ───
    // (DEBT-039, STORY-073) -- DESPUÉS de parsear (necesita el mínimo/máximo
    // real), ANTES de sellar: un rango que el archivo no respalda no debe
    // llegar al ledger, ni siquiera como aviso ignorable.
    check_declared_range_against_parsed_content(input.declared_range_start_ns, input.declared_range_end_ns, &bars)?;

    // ── Construcción de la declaración de mapeo ──────────────────────────
    let column_mapping_json =
        serde_json::to_string(&domain::mt5_bar_column_mapping()).expect("un Vec<(String,String)> siempre serializa a JSON");
    let preview_rows: Vec<PreviewBarRow> = bars.iter().take(PREVIEW_ROW_LIMIT).map(PreviewBarRow::from).collect();
    let preview_transformed_rows_json =
        serde_json::to_string(&preview_rows).expect("un Vec<PreviewBarRow> siempre serializa a JSON");
    let accepted_at_ns = clock.timestamp_ns();

    let hash_input = domain::MappingDeclarationHashInput {
        source_file_name: &input.source_file_name,
        source_file_hash: &source_file_hash,
        instrument_id: &input.instrument_id,
        venue: &input.venue,
        declared_range_start_ns: input.declared_range_start_ns,
        declared_range_end_ns: input.declared_range_end_ns,
        mapping_origin: "PLATFORM_TEMPLATE",
        platform_template_code: Some(domain::MT5_BAR_TEMPLATE_CODE),
        platform_template_version: Some(domain::MT5_BAR_TEMPLATE_VERSION),
        column_mapping_json: &column_mapping_json,
        transform_engine_version: TRANSFORM_ENGINE_VERSION,
        canonical_schema_version: CANONICAL_SCHEMA_VERSION,
        accepted_at_ns,
        preview_transformed_rows_json: &preview_transformed_rows_json,
        source_clock_anchor_timezone_ref: input.source_clock_anchor_timezone_ref.as_deref().unwrap_or_default(),
        source_clock_fixed_offset_minutes: input.source_clock_fixed_offset_minutes,
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
        declared_range_start_ns: input.declared_range_start_ns,
        declared_range_end_ns: input.declared_range_end_ns,
        mapping_origin: "PLATFORM_TEMPLATE".to_string(),
        platform_template_code: Some(domain::MT5_BAR_TEMPLATE_CODE.to_string()),
        platform_template_version: Some(domain::MT5_BAR_TEMPLATE_VERSION.to_string()),
        column_mapping_json,
        transform_engine_version: TRANSFORM_ENGINE_VERSION.to_string(),
        canonical_schema_version: CANONICAL_SCHEMA_VERSION.to_string(),
        accepted_at_ns,
        preview_transformed_rows_json,
        source_clock_anchor_timezone_ref: input.source_clock_anchor_timezone_ref.clone().unwrap_or_default(),
        source_clock_fixed_offset_minutes: input.source_clock_fixed_offset_minutes,
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
// Vía de mapeo manual declarado por el usuario (STORY-073, LDR-0008) --
// mismo patrón que `import_mt5_bar_file`: sella el ledger ANTES de invocar
// `bar_sink`, es idempotente por (hash de archivo, rango declarado), y
// aplica el mismo Gate de Homogeneidad de Procedencia y cotejo de rango
// (DEBT-039). La diferencia central: no hay plantilla que detectar --el
// mapeo lo trae el llamador ya construido (Paso 2 del asistente)-- y la
// declaración se sella con `mapping_origin = "MANUAL_USER"`, nunca
// "PLATFORM_TEMPLATE" (criterio de aceptación #4).
// ═══════════════════════════════════════════════════════════════════════

/// Entrada completa para importar un archivo bajo la vía de mapeo manual.
pub struct ManualImportBarFileInput {
    pub source_file_name: String,
    /// Contenido crudo del archivo -- mismo ponytail que
    /// `ImportBarFileInput::file_bytes` (decodificación completa en
    /// memoria, ver esa doc).
    pub file_bytes: Vec<u8>,
    pub instrument_id: String,
    pub venue: String,
    pub source_provider: String,
    pub declared_range_start_ns: i64,
    pub declared_range_end_ns: i64,
    pub source_clock_anchor_timezone_ref: Option<String>,
    pub source_clock_fixed_offset_minutes: i64,
    pub real_volume_unit_declared: Option<String>,
    /// Unidad del spread reportado -- a diferencia de la vía de plantilla
    /// (donde la plantilla la afirma), aquí la declara el usuario sobre su
    /// propio archivo. `None` rechaza el mapeo de cada fila (criterio de
    /// aceptación #13, mismo trato que la vía de plantilla).
    pub spread_reported_unit_declared: Option<String>,
    pub venue_operational_since_ns: Option<i64>,
    /// Etiqueta de temporalidad declarada (ej. "5M") -- se sella tal cual en
    /// el ledger.
    pub timeframe_value: String,
    /// Duración de una barra en nanosegundos -- ver
    /// `domain::ManualRunContext::bar_interval_ns`.
    pub bar_interval_ns: i64,
    /// El mapeo columna origen -> campo canónico que el usuario declaró y
    /// confirmó en el Paso 2 del asistente -- se sella tal cual en
    /// `column_mapping_json` (evidencia de LDR-0008).
    pub column_mappings: Vec<ManualColumnMapping>,
    pub node_id: Option<String>,
    pub process_id: Option<String>,
}

/// Resultado de una importación manual exitosa (sellada o no-op de
/// reimportación) -- misma forma que [`ImportBarFileOutcome`].
#[derive(Debug)]
pub struct ManualImportBarFileOutcome {
    pub declaration: crate::schemas::MappingDeclarationRow,
    pub bars: Vec<CanonicalBarRow>,
    pub was_noop_reimport: bool,
}

/// Errores de una importación manual. El subconjunto de parseo/mapeo puro
/// llega envuelto tal cual desde [`ManualImportCoreError`] -- el resto son
/// los mismos motivos de rechazo que la vía de plantilla comparte (Gate de
/// Homogeneidad, cotejo de rango, repositorio, sink).
#[derive(Debug, thiserror::Error)]
pub enum ManualImportBarFileError {
    #[error(transparent)]
    Core(#[from] ManualImportCoreError),
    #[error(transparent)]
    ProvenanceHomogeneity(#[from] ProvenanceHomogeneityError),
    #[error(transparent)]
    DeclaredRangeContentMismatch(#[from] DeclaredRangeContentMismatchError),
    #[error(transparent)]
    Repository(#[from] MappingDeclarationRepositoryError),
    #[error("la persistencia de datos aguas abajo falló DESPUÉS de sellar el ledger: {0}")]
    Sink(String),
}

/// Importa un archivo bajo la vía de mapeo manual declarado por el usuario:
/// resuelve y aplica el mapeo (sin plantilla), sella la declaración de
/// mapeo en el ledger con `mapping_origin = "MANUAL_USER"` (ANTES de
/// invocar `bar_sink`), y entrega las filas a `bar_sink` para su
/// persistencia aguas abajo.
///
/// Reutiliza [`parse_and_map_manual_bar_file`] -- la MISMA función pura que
/// produce la vista previa que el usuario confirmó -- para que lo sellado
/// sea exactamente lo que se le mostró (LDR-0008 condición técnica 1),
/// nunca una segunda derivación que pudiera divergir.
pub async fn import_manual_bar_file<F>(
    pool: &SqlitePool,
    clock: &dyn Clock,
    input: ManualImportBarFileInput,
    bar_sink: F,
) -> Result<ManualImportBarFileOutcome, ManualImportBarFileError>
where
    F: FnOnce(&[CanonicalBarRow]) -> Result<(), String>,
{
    let source_file_hash = domain::compute_source_file_hash(&input.file_bytes);

    // ── Idempotencia -- mismo mecanismo que la vía de plantilla: un mismo
    // tramo exacto (hash de archivo + rango declarado) ya sellado es un
    // no-op silencioso, sin importar por cuál vía se selló originalmente.
    let repository = MappingDeclarationRepository::new(pool, clock);
    if let Some(existing) = repository
        .find_by_source_file_hash_and_range(&source_file_hash, input.declared_range_start_ns, input.declared_range_end_ns)
        .await?
    {
        return Ok(ManualImportBarFileOutcome { declaration: existing, bars: Vec::new(), was_noop_reimport: true });
    }

    check_declared_range_against_venue_operational_date(
        &input.venue,
        input.declared_range_start_ns,
        input.venue_operational_since_ns,
    )?;

    // ── Parseo y mapeo (Núcleo puro) -- la MISMA llamada que produciría la
    // vista previa; lo que se sella abajo es exactamente esto.
    let ctx = ManualRunContext {
        instrument_id: &input.instrument_id,
        venue: &input.venue,
        source_provider: &input.source_provider,
        source_clock_anchor_timezone_ref: input.source_clock_anchor_timezone_ref.as_deref(),
        source_clock_fixed_offset_minutes: input.source_clock_fixed_offset_minutes,
        real_volume_unit_declared: input.real_volume_unit_declared.as_deref(),
        spread_reported_unit_declared: input.spread_reported_unit_declared.as_deref(),
        timeframe_value: &input.timeframe_value,
        bar_interval_ns: input.bar_interval_ns,
    };
    let bars = parse_and_map_manual_bar_file(&input.file_bytes, &input.column_mappings, &ctx)?;

    // ── Cotejo elemental del rango declarado (DEBT-039) -- mismo chequeo,
    // misma frontera (después de parsear, antes de sellar) que la vía de
    // plantilla.
    check_declared_range_against_parsed_content(input.declared_range_start_ns, input.declared_range_end_ns, &bars)?;

    // ── Construcción de la declaración de mapeo -- el mapeo sellado es
    // EXACTAMENTE el que el usuario declaró (LDR-0008), nunca una plantilla.
    let column_mapping_json = serde_json::to_string(
        &input.column_mappings.iter().map(|m| (m.source_column.clone(), m.canonical_field.label().to_string())).collect::<Vec<_>>(),
    )
    .expect("un Vec<(String,String)> siempre serializa a JSON");
    let preview_rows: Vec<PreviewBarRow> = bars.iter().take(PREVIEW_ROW_LIMIT).map(PreviewBarRow::from).collect();
    let preview_transformed_rows_json =
        serde_json::to_string(&preview_rows).expect("un Vec<PreviewBarRow> siempre serializa a JSON");
    let accepted_at_ns = clock.timestamp_ns();
    let spread_reported_unit_declared = input.spread_reported_unit_declared.clone().unwrap_or_default();

    let hash_input = domain::MappingDeclarationHashInput {
        source_file_name: &input.source_file_name,
        source_file_hash: &source_file_hash,
        instrument_id: &input.instrument_id,
        venue: &input.venue,
        declared_range_start_ns: input.declared_range_start_ns,
        declared_range_end_ns: input.declared_range_end_ns,
        mapping_origin: "MANUAL_USER",
        platform_template_code: None,
        platform_template_version: None,
        column_mapping_json: &column_mapping_json,
        transform_engine_version: TRANSFORM_ENGINE_VERSION,
        canonical_schema_version: CANONICAL_SCHEMA_VERSION,
        accepted_at_ns,
        preview_transformed_rows_json: &preview_transformed_rows_json,
        source_clock_anchor_timezone_ref: input.source_clock_anchor_timezone_ref.as_deref().unwrap_or_default(),
        source_clock_fixed_offset_minutes: input.source_clock_fixed_offset_minutes,
        real_volume_unit_declared: input.real_volume_unit_declared.as_deref(),
        spread_reported_unit_declared: &spread_reported_unit_declared,
    };
    let content_audit_hash = domain::compute_mapping_declaration_audit_hash(&hash_input);

    let new_declaration = NewMappingDeclaration {
        content_audit_hash,
        source_file_name: input.source_file_name.clone(),
        source_file_hash,
        instrument_id: input.instrument_id.clone(),
        venue: input.venue.clone(),
        bar_type: "TIME".to_string(),
        timeframe_value: input.timeframe_value.clone(),
        declared_range_start_ns: input.declared_range_start_ns,
        declared_range_end_ns: input.declared_range_end_ns,
        mapping_origin: "MANUAL_USER".to_string(),
        platform_template_code: None,
        platform_template_version: None,
        column_mapping_json,
        transform_engine_version: TRANSFORM_ENGINE_VERSION.to_string(),
        canonical_schema_version: CANONICAL_SCHEMA_VERSION.to_string(),
        accepted_at_ns,
        preview_transformed_rows_json,
        source_clock_anchor_timezone_ref: input.source_clock_anchor_timezone_ref.clone().unwrap_or_default(),
        source_clock_fixed_offset_minutes: input.source_clock_fixed_offset_minutes,
        real_volume_unit_declared: input.real_volume_unit_declared.clone(),
        spread_reported_unit_declared,
        node_id: input.node_id.clone(),
        process_id: input.process_id.clone(),
    };

    // ── Sellado del ledger -- ANTES de persistir cualquier dato ──────────
    let declaration = repository.record(new_declaration).await?;

    // ── Hand-off a la persistencia de datos aguas abajo ──────────────────
    bar_sink(&bars).map_err(ManualImportBarFileError::Sink)?;

    Ok(ManualImportBarFileOutcome { declaration, bars, was_noop_reimport: false })
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

    /// Rango declarado por defecto para las pruebas de este módulo -- amplio
    /// a propósito (2000-01-01 a 2030-01-01) para cubrir sin fricción tanto
    /// `REAL_SAMPLE_CSV` (2000-01-03) como `REAL_ANOMALY_CSV` (2019-11-01):
    /// esta Story no filtra filas por rango (ver domain.rs, "Rango declarado
    /// del tramo"), así que el valor exacto no afecta el parseo -- solo el
    /// chequeo de fecha de operación y la clave de idempotencia.
    const DEFAULT_DECLARED_RANGE_START_NS: i64 = 946_684_800_000_000_000; // 2000-01-01T00:00:00Z
    const DEFAULT_DECLARED_RANGE_END_NS: i64 = 1_893_456_000_000_000_000; // 2030-01-01T00:00:00Z

    fn sample_input(file_bytes: Vec<u8>) -> ImportBarFileInput {
        ImportBarFileInput {
            source_file_name: "EURUSD_M1_sample.csv".to_string(),
            file_bytes,
            instrument_id: "EURUSD".to_string(),
            venue: "PEPPERSTONE".to_string(),
            source_provider: "MT5_TERMINAL_EXPORT".to_string(),
            declared_range_start_ns: DEFAULT_DECLARED_RANGE_START_NS,
            declared_range_end_ns: DEFAULT_DECLARED_RANGE_END_NS,
            source_clock_anchor_timezone_ref: Some("America/New_York".to_string()),
            source_clock_fixed_offset_minutes: 0,
            real_volume_unit_declared: None,
            venue_operational_since_ns: None,
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
        input.source_clock_anchor_timezone_ref = None;

        let result = import_mt5_bar_file(&pool, &clock, input, |_rows| Ok(())).await;
        assert!(matches!(result, Err(ImportBarFileError::Mapping { .. })), "debe rechazar por falta de zona horaria declarada");

        let repository = MappingDeclarationRepository::new(&pool, &clock);
        assert!(repository.list_all().await.unwrap().is_empty(), "sin zona declarada, cero filas de ledger");
    }

    // ── Criterio de aceptación #7 (STORY-070): fecha de operación del bróker, de punta a punta ──

    /// Con fecha de operación declarada, un tramo que la antecede se
    /// rechaza de punta a punta -- cero filas, cero ledger, motivo
    /// explícito (`ProvenanceHomogeneity`, nunca un error genérico).
    #[tokio::test(flavor = "multi_thread")]
    async fn import_rejected_end_to_end_when_declared_range_precedes_venue_operational_date() {
        let (pool, _dir) = temp_pool_with_eurusd().await;
        let clock = DeterministicClock::new(0, 1);
        let mut input = sample_input(REAL_SAMPLE_CSV.to_vec());
        // El tramo declara empezar en DEFAULT_DECLARED_RANGE_START_NS; el
        // bróker declara operar desde un instante POSTERIOR -- el tramo es
        // imposible bajo esa identidad de bróker.
        input.venue_operational_since_ns = Some(DEFAULT_DECLARED_RANGE_START_NS + 1);

        let result = import_mt5_bar_file(&pool, &clock, input, |_rows| Ok(())).await;
        assert!(
            matches!(result, Err(ImportBarFileError::ProvenanceHomogeneity(_))),
            "debe rechazar por antecedente a la fecha de operación declarada del bróker: {result:?}"
        );

        let repository = MappingDeclarationRepository::new(&pool, &clock);
        assert!(repository.list_all().await.unwrap().is_empty(), "cero filas de ledger tras el rechazo por fecha de operación");
    }

    /// Sin fecha de operación declarada, el MISMO tramo (arbitrariamente
    /// antiguo) se acepta de punta a punta -- el chequeo NO corre, nunca se
    /// asume una fecha que nadie declaró.
    #[tokio::test(flavor = "multi_thread")]
    async fn import_accepted_end_to_end_when_venue_operational_date_is_not_declared() {
        let (pool, _dir) = temp_pool_with_eurusd().await;
        let clock = DeterministicClock::new(0, 1);
        let mut input = sample_input(REAL_SAMPLE_CSV.to_vec());
        input.venue_operational_since_ns = None;

        let outcome = import_mt5_bar_file(&pool, &clock, input, |_rows| Ok(()))
            .await
            .expect("sin fecha de operación declarada, la importación debe aceptarse sin rechazo");
        assert!(!outcome.was_noop_reimport);
        assert_eq!(outcome.bars.len(), 6, "el fragmento real completo (6 filas) debe importarse sin rechazo");
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

    // ═══════════════════════════════════════════════════════════════════
    // Vía de mapeo manual (STORY-073) -- criterios de aceptación 1, 3, 4 y 5.
    // ═══════════════════════════════════════════════════════════════════

    use crate::domain::CanonicalBarField;

    /// Archivo con columnas ARBITRARIAS (delimitador coma, nombres en
    /// español, orden barajado, timestamp combinado) -- deliberadamente
    /// distinto de la plantilla MT5 (criterio de aceptación #1). Dos filas
    /// de dato, cadencia de 5 minutos.
    const ARBITRARY_CSV: &[u8] = b"SpreadPts,Cierre,Fecha,VolumenReal,Maximo,Minimo,VolumenTicks,Apertura\n\
         5,2.50500000,2024-03-01 10:00:00,1000,2.51000000,2.49000000,10,2.50000000\n\
         6,2.50900000,2024-03-01 10:05:00,1200,2.51200000,2.50400000,12,2.50500000\n";

    fn arbitrary_column_mapping() -> Vec<ManualColumnMapping> {
        vec![
            ManualColumnMapping { source_column: "Fecha".to_string(), canonical_field: CanonicalBarField::Timestamp },
            ManualColumnMapping { source_column: "Apertura".to_string(), canonical_field: CanonicalBarField::Open },
            ManualColumnMapping { source_column: "Maximo".to_string(), canonical_field: CanonicalBarField::High },
            ManualColumnMapping { source_column: "Minimo".to_string(), canonical_field: CanonicalBarField::Low },
            ManualColumnMapping { source_column: "Cierre".to_string(), canonical_field: CanonicalBarField::Close },
            ManualColumnMapping {
                source_column: "VolumenTicks".to_string(),
                canonical_field: CanonicalBarField::VolumeTicks,
            },
            ManualColumnMapping {
                source_column: "VolumenReal".to_string(),
                canonical_field: CanonicalBarField::VolumeTraded,
            },
            ManualColumnMapping {
                source_column: "SpreadPts".to_string(),
                canonical_field: CanonicalBarField::SpreadReported,
            },
        ]
    }

    /// Rango declarado por defecto de las pruebas manuales: el día completo
    /// 2024-03-01 UTC, calculado con `chrono::Utc` de forma independiente.
    fn arbitrary_declared_range() -> (i64, i64) {
        use chrono::{TimeZone, Utc};
        let start = Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap().timestamp_nanos_opt().unwrap();
        let end = Utc.with_ymd_and_hms(2024, 3, 2, 0, 0, 0).unwrap().timestamp_nanos_opt().unwrap();
        (start, end)
    }

    fn manual_sample_input(file_bytes: Vec<u8>) -> ManualImportBarFileInput {
        let (declared_range_start_ns, declared_range_end_ns) = arbitrary_declared_range();
        ManualImportBarFileInput {
            source_file_name: "broker_generico_export.csv".to_string(),
            file_bytes,
            instrument_id: "EURUSD".to_string(),
            venue: "GENERIC_BROKER".to_string(),
            source_provider: "MANUAL_UPLOAD".to_string(),
            declared_range_start_ns,
            declared_range_end_ns,
            source_clock_anchor_timezone_ref: Some("UTC".to_string()),
            source_clock_fixed_offset_minutes: 0,
            real_volume_unit_declared: Some("CONTRACTS".to_string()),
            spread_reported_unit_declared: Some("POINTS_GENERIC".to_string()),
            venue_operational_since_ns: None,
            timeframe_value: "5M".to_string(),
            bar_interval_ns: 300_000_000_000,
            column_mappings: arbitrary_column_mapping(),
            node_id: Some("test-node".to_string()),
            process_id: Some("test-process".to_string()),
        }
    }

    /// Criterio de aceptación #1: un archivo sin plantilla conocida (CSV de
    /// columnas arbitrarias) se importa de punta a punta declarando el
    /// mapeo manualmente.
    #[tokio::test(flavor = "multi_thread")]
    async fn import_manual_bar_file_with_arbitrary_csv_succeeds_end_to_end() {
        let (pool, _dir) = temp_pool_with_eurusd().await;
        let clock = DeterministicClock::new(0, 1);

        let outcome = import_manual_bar_file(&pool, &clock, manual_sample_input(ARBITRARY_CSV.to_vec()), |_rows| Ok(()))
            .await
            .expect("un CSV de columnas arbitrarias con mapeo manual completo debe importar sin error");

        assert!(!outcome.was_noop_reimport);
        assert_eq!(outcome.bars.len(), 2, "las dos filas de dato del archivo deben importarse");
    }

    /// Criterio de aceptación #4: la vía manual sella `mapping_origin =
    /// "MANUAL_USER"`, distinguible de `"PLATFORM_TEMPLATE"` que sella la
    /// vía de plantilla -- ambas vías probadas en la MISMA base de datos.
    #[tokio::test(flavor = "multi_thread")]
    async fn manual_and_template_imports_seal_distinguishable_mapping_origin() {
        let (pool, _dir) = temp_pool_with_eurusd().await;
        let clock = DeterministicClock::new(0, 1);

        let manual_outcome =
            import_manual_bar_file(&pool, &clock, manual_sample_input(ARBITRARY_CSV.to_vec()), |_rows| Ok(())).await.unwrap();
        let template_outcome =
            import_mt5_bar_file(&pool, &clock, sample_input(REAL_SAMPLE_CSV.to_vec()), |_rows| Ok(())).await.unwrap();

        assert_eq!(manual_outcome.declaration.mapping_origin, "MANUAL_USER");
        assert_eq!(template_outcome.declaration.mapping_origin, "PLATFORM_TEMPLATE");
        assert_ne!(
            manual_outcome.declaration.mapping_origin, template_outcome.declaration.mapping_origin,
            "las dos vías deben sellar un origen de mapeo DISTINGUIBLE, nunca el mismo valor"
        );

        let repository = MappingDeclarationRepository::new(&pool, &clock);
        let all = repository.list_all().await.unwrap();
        assert_eq!(all.len(), 2, "ambas declaraciones deben quedar selladas en el mismo ledger");
    }

    /// Foco 1 del gate de mutación del Tech-Lead sobre STORY-073
    /// (2026-07-31), defensa en profundidad: el `column_mapping_json`
    /// SELLADO en el ledger -- la evidencia de LDR-0008 de qué se le
    /// mostró al usuario -- debe llevar las etiquetas REALES de los campos
    /// canónicos, no una cadena vacía ni basura. Oráculo hardcodeado, nunca
    /// construido llamando a `CanonicalBarField::label()` (eso volvería la
    /// prueba autorreferencial: un mutante en `label()` produciría el mismo
    /// valor equivocado en la aserción y en la producción, sin discriminar
    /// nada) -- mismo patrón que
    /// `sealed_ledger_declaration_contains_the_real_mt5_column_mapping`.
    #[tokio::test(flavor = "multi_thread")]
    async fn manual_import_seals_column_mapping_json_with_real_canonical_field_labels() {
        let (pool, _dir) = temp_pool_with_eurusd().await;
        let clock = DeterministicClock::new(0, 1);
        let outcome =
            import_manual_bar_file(&pool, &clock, manual_sample_input(ARBITRARY_CSV.to_vec()), |_rows| Ok(())).await.unwrap();

        let parsed: Vec<(String, String)> = serde_json::from_str(&outcome.declaration.column_mapping_json).unwrap();
        let expected: Vec<(String, String)> = vec![
            ("Fecha".to_string(), "Timestamp".to_string()),
            ("Apertura".to_string(), "Open".to_string()),
            ("Maximo".to_string(), "High".to_string()),
            ("Minimo".to_string(), "Low".to_string()),
            ("Cierre".to_string(), "Close".to_string()),
            ("VolumenTicks".to_string(), "VolumeTicks".to_string()),
            ("VolumenReal".to_string(), "VolumeTraded".to_string()),
            ("SpreadPts".to_string(), "SpreadReported".to_string()),
        ];
        assert_eq!(
            parsed, expected,
            "el JSON sellado en el ledger debe contener EXACTAMENTE las etiquetas reales de los campos canónicos \
             que el usuario declaró -- nunca cadenas vacías ni pares basura"
        );
    }

    /// Idempotencia de la vía manual (ADR-0163, mismo criterio de aceptación
    /// #10 que la vía de plantilla ya prueba): reimportar el MISMO tramo se
    /// trata como no-op. Lo que esto afirma NO es solo "no hay una segunda
    /// fila en el ledger" -- eso lo garantiza el repositorio compartido, ya
    /// probado, y no dice nada sobre el cableado propio de
    /// `import_manual_bar_file`. Lo que SÍ es propio de este cableado, y lo
    /// que esta prueba mide directamente, es que `bar_sink` NO se invoca una
    /// segunda vez: un "no-op" que igual vuelca 9,2 millones de filas al
    /// consumidor no es un no-op, y el ledger se vería IDÉNTICO en ambos
    /// casos -- por eso el conteo de invocaciones del sink es la aserción
    /// que realmente discrimina, no el conteo de filas del ledger (que
    /// también se afirma, pero como confirmación adicional, no como prueba
    /// principal).
    #[tokio::test(flavor = "multi_thread")]
    async fn reimporting_identical_manual_tramo_is_a_noop_with_no_second_sink_call() {
        let (pool, _dir) = temp_pool_with_eurusd().await;
        let clock = DeterministicClock::new(0, 1);
        let sink_call_count = std::sync::Arc::new(AtomicUsize::new(0));

        let first_sink_count = std::sync::Arc::clone(&sink_call_count);
        let first = import_manual_bar_file(&pool, &clock, manual_sample_input(ARBITRARY_CSV.to_vec()), move |_rows| {
            first_sink_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .await
        .expect("la primera importación manual debe tener éxito");
        assert!(!first.was_noop_reimport);

        let second_sink_count = std::sync::Arc::clone(&sink_call_count);
        let second = import_manual_bar_file(&pool, &clock, manual_sample_input(ARBITRARY_CSV.to_vec()), move |_rows| {
            second_sink_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .await
        .expect("reimportar el mismo tramo manual debe tratarse como no-op, no como error");

        assert!(second.was_noop_reimport, "la segunda importación del mismo tramo manual debe marcarse como no-op");
        assert_eq!(
            sink_call_count.load(Ordering::SeqCst),
            1,
            "bar_sink NO debe volver a invocarse en el no-op -- esto es lo que realmente prueba que la fila no se \
             emite dos veces, no solo que el ledger no creció"
        );

        let repository = MappingDeclarationRepository::new(&pool, &clock);
        let declarations = repository.list_all().await.unwrap();
        assert_eq!(declarations.len(), 1, "no debe existir una segunda fila de ledger para el mismo tramo manual");
        assert_eq!(declarations[0].id, first.declaration.id, "debe ser la MISMA declaración, no una nueva");
    }

    /// Criterio de aceptación #3: pedir la vista previa (llamando la función
    /// pura de parseo+mapeo, sin `SqlitePool`) tantas veces como se quiera
    /// NUNCA escribe una fila del ledger -- se prueba contra un pool real
    /// que la función de vista previa jamás recibió.
    #[tokio::test(flavor = "multi_thread")]
    async fn requesting_preview_multiple_times_never_writes_to_the_ledger() {
        let (pool, _dir) = temp_pool_with_eurusd().await;
        let clock = DeterministicClock::new(0, 1);

        let ctx = crate::domain::ManualRunContext {
            instrument_id: "EURUSD",
            venue: "GENERIC_BROKER",
            source_provider: "MANUAL_UPLOAD",
            source_clock_anchor_timezone_ref: Some("UTC"),
            source_clock_fixed_offset_minutes: 0,
            real_volume_unit_declared: Some("CONTRACTS"),
            spread_reported_unit_declared: Some("POINTS_GENERIC"),
            timeframe_value: "5M",
            bar_interval_ns: 300_000_000_000,
        };
        let mapping = arbitrary_column_mapping();

        // Cinco vistas previas -- ninguna toca `pool` (la función ni lo
        // recibe como parámetro).
        for _ in 0..5 {
            let preview = parse_and_map_manual_bar_file(ARBITRARY_CSV, &mapping, &ctx)
                .expect("la vista previa debe poder pedirse repetidamente sin error");
            assert_eq!(preview.len(), 2);
        }

        let repository = MappingDeclarationRepository::new(&pool, &clock);
        let all = repository.list_all().await.unwrap();
        assert!(all.is_empty(), "cinco vistas previas no deben haber sellado ninguna fila del ledger");
    }

    /// Criterio de aceptación #5 (DEBT-039), vía manual: un rango declarado
    /// deliberadamente falso (que no cubre el contenido real del archivo)
    /// se rechaza, y nada se sella.
    #[tokio::test(flavor = "multi_thread")]
    async fn import_manual_bar_file_rejects_declared_range_that_does_not_cover_content() {
        let (pool, _dir) = temp_pool_with_eurusd().await;
        let clock = DeterministicClock::new(0, 1);
        let mut input = manual_sample_input(ARBITRARY_CSV.to_vec());
        // El archivo real es del 2024-03-01 -- se declara un tramo de un año
        // completo DISTINTO (2019), sin relación alguna con el contenido.
        use chrono::{TimeZone, Utc};
        input.declared_range_start_ns = Utc.with_ymd_and_hms(2019, 1, 1, 0, 0, 0).unwrap().timestamp_nanos_opt().unwrap();
        input.declared_range_end_ns = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap().timestamp_nanos_opt().unwrap();

        let result = import_manual_bar_file(&pool, &clock, input, |_rows| Ok(())).await;
        assert!(
            matches!(result, Err(ManualImportBarFileError::DeclaredRangeContentMismatch(_))),
            "un rango declarado que no cubre el contenido real debe rechazarse: {result:?}"
        );

        let repository = MappingDeclarationRepository::new(&pool, &clock);
        assert!(repository.list_all().await.unwrap().is_empty(), "nada debe sellarse cuando el rango declarado se rechaza");
    }

    /// Criterio de aceptación #5 (DEBT-039), vía de plantilla MT5: el mismo
    /// cotejo cierra el hueco que dejaba `import_mt5_bar_file` -- un rango
    /// declarado sin relación con el contenido real del archivo se rechaza,
    /// sin sellar nada.
    #[tokio::test(flavor = "multi_thread")]
    async fn import_mt5_bar_file_rejects_declared_range_that_does_not_cover_content() {
        let (pool, _dir) = temp_pool_with_eurusd().await;
        let clock = DeterministicClock::new(0, 1);
        let mut input = sample_input(REAL_SAMPLE_CSV.to_vec());
        // REAL_SAMPLE_CSV es del 2000-01-03 -- se declara un tramo de 2019,
        // sin relación con el contenido real del archivo.
        use chrono::{TimeZone, Utc};
        input.declared_range_start_ns = Utc.with_ymd_and_hms(2019, 1, 1, 0, 0, 0).unwrap().timestamp_nanos_opt().unwrap();
        input.declared_range_end_ns = Utc.with_ymd_and_hms(2019, 1, 2, 0, 0, 0).unwrap().timestamp_nanos_opt().unwrap();

        let result = import_mt5_bar_file(&pool, &clock, input, |_rows| Ok(())).await;
        assert!(
            matches!(result, Err(ImportBarFileError::DeclaredRangeContentMismatch(_))),
            "DEBT-039: un rango declarado sin relación con el contenido real debe rechazarse: {result:?}"
        );

        let repository = MappingDeclarationRepository::new(&pool, &clock);
        assert!(repository.list_all().await.unwrap().is_empty(), "nada debe sellarse cuando el rango declarado se rechaza");
    }
}
