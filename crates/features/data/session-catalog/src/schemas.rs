//! Contratos de datos — structs de persistencia para `session_template` y
//! `session_template_window` (Perfil A, ADR-0020 + ADR-0141 + ADR-0162).

use crate::domain::SessionWindow;

// ── session_template ──────────────────────────────────────────────────────

/// Datos para dar de alta una plantilla de sesión nueva, con sus ventanas.
/// Los campos del Grupo I los genera el repositorio.
#[derive(Debug, Clone)]
pub struct NewSessionTemplate {
    pub template_code: String,
    pub display_name: String,
    pub market_timezone_ref: String,
    pub windows: Vec<SessionWindow>,
    pub node_id: Option<String>,
    pub process_id: Option<String>,
}

/// Una fila de `session_template` ya persistida (sin sus ventanas -- ver
/// [`SessionCalendar`] para la plantilla + ventanas juntas).
#[derive(Debug, Clone, PartialEq)]
pub struct SessionTemplateRow {
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
    pub template_code: String,
    pub display_name: String,
    pub market_timezone_ref: String,
}

/// Una fila de `session_template_window` ya persistida.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionTemplateWindowRow {
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
    pub session_template_id: String,
    pub window: SessionWindow,
}

/// Plantilla de sesión resuelta con sus ventanas y zona horaria (puerto de
/// salida `session_calendar_out`, ADR-0137).
#[derive(Debug, Clone, PartialEq)]
pub struct SessionCalendar {
    pub template: SessionTemplateRow,
    pub windows: Vec<SessionTemplateWindowRow>,
}
