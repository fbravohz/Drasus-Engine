//! Contratos de datos — structs de persistencia para `instrument_master` y
//! `broker_instrument_terms` (Perfil A, ADR-0020 + ADR-0141 + ADR-0162).

use crate::domain::{AssetClass, CommissionModel};

// ── instrument_master ────────────────────────────────────────────────────

/// Datos para dar de alta un instrumento canónico nuevo. Los campos del
/// Grupo I (`id`, `created_at`, `updated_at`, `audit_hash`,
/// `audit_chain_hash`, `row_version`) los genera el repositorio.
#[derive(Debug, Clone)]
pub struct NewInstrumentMaster {
    pub instrument_id: String,
    pub asset_class: AssetClass,
    pub base_currency: Option<String>,
    pub quote_currency: Option<String>,
    pub canonical_pip_size: i64,
    pub canonical_tick_size: i64,
    pub default_session_template_id: Option<String>,
    pub node_id: Option<String>,
    pub process_id: Option<String>,
}

/// Una fila de `instrument_master` ya persistida.
#[derive(Debug, Clone, PartialEq)]
pub struct InstrumentMasterRow {
    // ── Grupo I ──
    pub id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub row_version: i64,

    // ── Grupo IV ──
    pub node_id: Option<String>,
    pub process_id: Option<String>,

    // ── Dominio ──
    pub instrument_id: String,
    pub asset_class: AssetClass,
    pub base_currency: Option<String>,
    pub quote_currency: Option<String>,
    pub canonical_pip_size: i64,
    pub canonical_tick_size: i64,
    pub default_session_template_id: Option<String>,
}

/// Revisión de los campos canónicos editables de un instrumento (TTR-001,
/// "Alta y Edición"). No permite editar `instrument_id` (clave de negocio
/// inmutable tras el alta) ni `asset_class` (cambiar de clase de activo no
/// es una edición, es un instrumento distinto).
#[derive(Debug, Clone)]
pub struct InstrumentMasterRevision {
    pub canonical_pip_size: i64,
    pub canonical_tick_size: i64,
    pub default_session_template_id: Option<String>,
}

// ── broker_instrument_terms ──────────────────────────────────────────────

/// Datos para dar de alta el override de un bróker sobre un instrumento.
#[derive(Debug, Clone, Default)]
pub struct NewBrokerInstrumentTerms {
    pub instrument_id: String,
    pub venue: String,
    pub venue_symbol: Option<String>,
    pub default_spread: Option<i64>,
    pub default_slippage: Option<i64>,
    pub pip_size_override: Option<i64>,
    pub tick_size_override: Option<i64>,
    pub point_value: Option<i64>,
    pub commission_model: Option<CommissionModel>,
    pub commission_value: Option<i64>,
    pub session_template_id_override: Option<String>,
    pub node_id: Option<String>,
    pub process_id: Option<String>,
}

/// Una fila de `broker_instrument_terms` ya persistida.
#[derive(Debug, Clone, PartialEq)]
pub struct BrokerInstrumentTermsRow {
    // ── Grupo I ──
    pub id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub row_version: i64,

    // ── Grupo IV ──
    pub node_id: Option<String>,
    pub process_id: Option<String>,

    // ── Dominio ──
    pub instrument_id: String,
    pub venue: String,
    pub venue_symbol: Option<String>,
    pub default_spread: Option<i64>,
    pub default_slippage: Option<i64>,
    pub pip_size_override: Option<i64>,
    pub tick_size_override: Option<i64>,
    pub point_value: Option<i64>,
    pub commission_model: Option<CommissionModel>,
    pub commission_value: Option<i64>,
    pub session_template_id_override: Option<String>,
}

// ── broker_server_clock (retrofit STORY-070/TASK-071) ───────────────────

/// Datos para dar de alta el reloj de servidor declarado de un bróker/venue
/// (ADR-0162 "Enmienda 2026-07-26").
#[derive(Debug, Clone)]
pub struct NewBrokerServerClock {
    pub venue: String,
    pub clock_anchor_timezone_ref: String,
    pub clock_fixed_offset_minutes: i64,
    pub venue_operational_since_ns: Option<i64>,
    pub node_id: Option<String>,
    pub process_id: Option<String>,
}

/// Una fila de `broker_server_clock` ya persistida.
#[derive(Debug, Clone, PartialEq)]
pub struct BrokerServerClockRow {
    // ── Grupo I ──
    pub id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub row_version: i64,

    // ── Grupo IV ──
    pub node_id: Option<String>,
    pub process_id: Option<String>,

    // ── Dominio ──
    pub venue: String,
    pub clock_anchor_timezone_ref: String,
    pub clock_fixed_offset_minutes: i64,
    pub venue_operational_since_ns: Option<i64>,
}

/// Revisión de los campos editables del reloj de un bróker. No permite
/// editar `venue` (clave de negocio inmutable tras el alta -- cambiar el
/// venue es declarar el reloj de OTRO bróker, no editar el existente, mismo
/// criterio que `InstrumentMasterRevision` con `instrument_id`).
#[derive(Debug, Clone)]
pub struct BrokerServerClockRevision {
    pub clock_anchor_timezone_ref: String,
    pub clock_fixed_offset_minutes: i64,
    pub venue_operational_since_ns: Option<i64>,
}
