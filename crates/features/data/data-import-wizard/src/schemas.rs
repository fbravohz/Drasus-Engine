//! [SHELL] DTOs de persistencia para `import_mapping_declarations`
//! (migración 0023) y para la vista previa serializable que el ledger sella.
//!
//! Separados del `domain::CanonicalBarRow` interno: estos tipos existen
//! solo para cruzar la frontera de serialización (SQLite/JSON), no para
//! lógica de negocio.

use serde::{Deserialize, Serialize};

/// Datos necesarios para insertar una nueva declaración de mapeo. El
/// `content_audit_hash` llega YA CALCULADO por el Núcleo
/// (`domain::compute_mapping_declaration_audit_hash`) -- la persistencia
/// nunca recalcula el hash de contenido, solo lo guarda y lo encadena.
#[derive(Debug, Clone)]
pub struct NewMappingDeclaration {
    pub content_audit_hash: String,
    pub source_file_name: String,
    pub source_file_hash: String,
    pub instrument_id: String,
    pub venue: String,
    pub bar_type: String,
    pub timeframe_value: String,
    /// Rango declarado del tramo (ADR-0162 "Enmienda 2026-07-26") --
    /// nanosegundos UTC, intervalo semiabierto [inicio, fin). Explícito,
    /// nunca implícito en las filas resultantes que el lector produce.
    pub declared_range_start_ns: i64,
    pub declared_range_end_ns: i64,
    pub mapping_origin: String,
    pub platform_template_code: Option<String>,
    pub platform_template_version: Option<String>,
    pub column_mapping_json: String,
    pub transform_engine_version: String,
    pub canonical_schema_version: String,
    pub accepted_at_ns: i64,
    pub preview_transformed_rows_json: String,
    /// Zona ancla IANA del reloj de origen declarado (ADR-0162 "Enmienda
    /// 2026-07-26") -- sustituye al antiguo `source_timezone_declared`.
    pub source_clock_anchor_timezone_ref: String,
    /// Desplazamiento fijo en minutos frente al ancla. `0` reproduce sin
    /// pérdida la declaración plana de zona de hoy.
    pub source_clock_fixed_offset_minutes: i64,
    pub real_volume_unit_declared: Option<String>,
    /// Unidad del spread reportado declarada por la plantilla (criterio de
    /// aceptación #13) -- siempre presente, nunca `Option`: una importación
    /// sin esta declaración se rechaza antes de llegar aquí.
    pub spread_reported_unit_declared: String,
    pub node_id: Option<String>,
    pub process_id: Option<String>,
}

/// Una fila ya persistida de `import_mapping_declarations`, con los campos
/// de Grupo I ya resueltos (id, timestamps, cadena de hashes, secuencia).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MappingDeclarationRow {
    pub id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub event_sequence_id: i64,
    pub node_id: Option<String>,
    pub process_id: Option<String>,
    pub source_file_name: String,
    pub source_file_hash: String,
    pub instrument_id: String,
    pub venue: String,
    pub bar_type: String,
    pub timeframe_value: String,
    pub declared_range_start_ns: i64,
    pub declared_range_end_ns: i64,
    pub mapping_origin: String,
    pub platform_template_code: Option<String>,
    pub platform_template_version: Option<String>,
    pub column_mapping_json: String,
    pub transform_engine_version: String,
    pub canonical_schema_version: String,
    pub accepted_at_ns: i64,
    pub preview_transformed_rows_json: String,
    pub source_clock_anchor_timezone_ref: String,
    pub source_clock_fixed_offset_minutes: i64,
    pub real_volume_unit_declared: Option<String>,
    pub spread_reported_unit_declared: String,
}

/// Proyección serializable de una `CanonicalBarRow`, usada exclusivamente
/// para construir `preview_transformed_rows_json` -- la vista previa de
/// DATO YA TRANSFORMADO que LDR-0008 condición técnica 1 exige (nunca solo
/// el esquema de mapeo).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewBarRow {
    pub instrument_id: String,
    pub venue: String,
    pub bar_open_timestamp_ns: i64,
    pub bar_close_timestamp_ns: i64,
    pub open: i64,
    pub high: i64,
    pub low: i64,
    pub close: i64,
    pub volume_ticks: i64,
    pub volume_traded_raw: i128,
    pub spread_reported_raw: i64,
    pub is_activity_anomaly: bool,
}

impl From<&crate::domain::CanonicalBarRow> for PreviewBarRow {
    fn from(bar: &crate::domain::CanonicalBarRow) -> Self {
        Self {
            instrument_id: bar.instrument_id.clone(),
            venue: bar.venue.clone(),
            bar_open_timestamp_ns: bar.bar_open_timestamp_ns,
            bar_close_timestamp_ns: bar.bar_close_timestamp_ns,
            open: bar.open,
            high: bar.high,
            low: bar.low,
            close: bar.close,
            volume_ticks: bar.volume_ticks,
            volume_traded_raw: bar.volume_traded_raw,
            spread_reported_raw: bar.spread_reported_raw,
            is_activity_anomaly: bar.is_activity_anomaly,
        }
    }
}
