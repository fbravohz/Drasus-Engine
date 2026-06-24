//! Preset canónico del pipeline Drasus Engine (ADR-0137).
//!
//! Este crate es una **composición preset** — no contiene lógica de negocio.
//! Su único propósito es declarar el cableado por defecto entre features
//! para que `app` pueda instanciar el pipeline completo con una sola
//! dependencia.
//!
//! Un usuario avanzado puede ignorar este preset y cablear features
//! directamente en el Canvas [Forge/Reactor] (ADR-0136).
//!
//! ## Orden del pipeline
//!
//! ```text
//! ingest → generate → validate → incubate → manage → execute → feedback → withdraw
//! ```
//!
//! ## Cómo añadir una feature al preset
//!
//! 1. Añade la dependencia en `Cargo.toml`
//! 2. Re-exporta su `public_interface` desde aquí
//! 3. Declara el wiring (qué output → qué input) en esta doc o en un módulo
//!    de configuración

// ── Re-exports de features (se añaden al construirlas en el ROADMAP) ──
// pub use backtest_engine::public_interface;
// pub use monte_carlo_simulator::public_interface;
// ...

#[cfg(test)]
mod tests {
    #[test]
    fn preset_compiles() {
        assert_eq!(2 + 2, 4);
    }
}
