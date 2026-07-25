//! Feature: `instrument-catalog` — catálogo canónico de instrumentos y sus
//! términos de trading por bróker, con herencia por `NULL` explícito.
//!
//! **Dominio:** data
//! **Primer consumidor (ADR-0118):** EPIC-1 (Ingest), STORY-066
//! **Perfil ADR-0020:** A (Datos/Ingest — Identidad + Linaje + Hardware)
//!
//! Crate hexagonal de dominio (ADR-0137): expone solo sus puertos tipados a
//! través de `public_interface`. La lógica pura vive en `domain` (la
//! resolución `COALESCE` campo a campo, sin ningún import de I/O); los
//! adaptadores de infraestructura en `orchestrator`/`persistence`.
//!
//! ## Reglas de frontera (ADR-0002 + ADR-0137)
//!
//! - `public_interface` es el ÚNICO módulo `pub` de este crate.
//! - `domain` no importa nada que toque I/O (`sqlx`, `tokio`).
//! - Prohibido depender de otros feature crates — solo `shared`. En
//!   particular, la cadena de resolución de calendario (override de bróker
//!   → default de instrumento) NUNCA llama al crate `session-catalog`; el
//!   `template_id` ya resuelto es el dato que esta feature expone en
//!   `ResolvedInstrumentTerms.resolved_session_template_id`, listo para
//!   que la capa de composición se lo pase a `session-catalog`.

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
