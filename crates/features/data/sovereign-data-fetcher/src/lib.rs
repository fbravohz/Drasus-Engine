//! Feature: `sovereign-data-fetcher` — Descarga híbrida soberana de datos de mercado.
//!
//! **Dominio:** data
//! **Primer consumidor (ADR-0118):** EPIC-1 (Ingest)
//! **Perfil ADR-0020 V2:** A (Datos de Mercado)
//!
//! Primer crate hexagonal de dominio del proyecto (ADR-0137): expone solo sus
//! puertos tipados a través de `public_interface`. La lógica pura vive en
//! `domain` (sin ningún import de I/O); los adaptadores de infraestructura
//! en `orchestrator`.
//!
//! ## Reglas de frontera (ADR-0002 + ADR-0137)
//!
//! - `public_interface` es el ÚNICO módulo `pub` de este crate.
//! - `domain` no importa nada que toque I/O (reqwest, tokio, std::fs, sqlx).
//! - Prohibido depender de otros feature crates — solo `shared`.

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
