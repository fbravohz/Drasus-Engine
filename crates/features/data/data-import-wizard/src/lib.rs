//! Feature: `data-import-wizard` — lector del archivo de barras de
//! MetaTrader (vía de plantilla de plataforma) y ledger append-only de
//! declaración de mapeo.
//!
//! **Dominio:** data
//! **Primer consumidor (ADR-0118):** EPIC-1 (Ingest), STORY-068
//! **Perfil ADR-0020:** A (Datos/Ingest — Identidad + Linaje + Hardware)
//!
//! Crate hexagonal de dominio (ADR-0137): expone solo sus puertos tipados a
//! través de `public_interface`. La lógica pura (parseo de línea, conversión
//! de escala, resolución de zona horaria, detección de imposibilidad física)
//! vive en `domain` (sin ningún import de I/O); los adaptadores de
//! infraestructura (SQLite, reloj de sistema) en `orchestrator`/`persistence`.
//!
//! ## Alcance de esta Story (STORY-068)
//!
//! Solo la vía de **plantilla de plataforma** para el archivo de BARRAS de
//! MT5 (M1). La vía de mapeo manual (arrastrar archivo, vista previa
//! interactiva, confirmación del usuario) y el archivo de TICKS quedan para
//! Stories futuras — ver `docs/execution/STORY-068-mt5-bar-reader.md` §0 y §8.
//!
//! ## Reglas de frontera (ADR-0002 + ADR-0137)
//!
//! - `public_interface` es el ÚNICO módulo `pub` de este crate.
//! - `domain` no importa nada que toque I/O (`sqlx`, `tokio`, reloj de
//!   sistema) — sí puede resolver zonas horarias IANA (`chrono-tz`), porque
//!   esa resolución es una tabla estática determinista, no un reloj.
//! - Prohibido depender de otros feature crates — solo `shared`. En
//!   particular, esta feature NUNCA consulta directamente las tablas de
//!   `instrument-catalog` (`instrument_master`/`broker_instrument_terms`):
//!   los hechos que esas tablas ya conocen (zona horaria declarada, unidad
//!   de volumen declarada) llegan como parámetros de entrada de quien
//!   compone el pipeline (Bridge/CLI), nunca por acceso cruzado a tablas
//!   ajenas (ADR-0137 "Prohibido acceso cross-module a tablas ajenas").
//! - Esta feature nunca escribe Parquet directamente — el hand-off a
//!   `hybrid-data-transformer` para la escritura atómica de partición
//!   (ADR-0163) se modela aquí como un puerto de salida (`BarSink`), no como
//!   una dependencia de crate.

mod domain;
mod orchestrator;
mod persistence;
mod schemas;

pub mod public_interface;

#[cfg(test)]
mod tests {
    /// Prueba de humo: el crate compila y enlaza correctamente.
    #[test]
    fn crate_compiles_and_links() {
        assert_eq!(2 + 2, 4);
    }
}
