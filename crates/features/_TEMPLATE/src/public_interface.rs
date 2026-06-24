//! Superficie pública del feature crate — el ÚNICO módulo `pub`.
//!
//! Expone los puertos de entrada y salida con tipos del catálogo ADR-0137.
//! Otros crates solo pueden importar desde aquí; prohibido acceder a
//! `domain`, `orchestrator`, `persistence` o `schemas` directamente.

use shared::types::*;

// ── Puertos de entrada ──────────────────────────────────────────────────────

/// Datos que esta feature acepta de otros nodos en el Canvas.
pub struct InputPorts {
    // TODO: declarar puertos con tipos del catálogo ADR-0137.
    // Ejemplo:
    // pub bars: Port<Bars>,
}

/// Datos que esta feature produce hacia otros nodos en el Canvas.
pub struct OutputPorts {
    // TODO: declarar puertos con tipos del catálogo ADR-0137.
    // Ejemplo:
    // pub results: Port<BacktestResult>,
}
