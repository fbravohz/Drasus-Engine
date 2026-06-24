//! Feature: [NOMBRE] — [descripción en una frase].
//!
//! **Dominio:** [data | generation | validation | execution | portfolio | lifecycle]
//! **Primer consumidor (ADR-0118):** [EPIC-X]
//! **Perfil ADR-0020 V2:** [A | B | C | D]
//!
//! Cada feature crate es un hexágono (ADR-0137): expone solo sus puertos
//! tipados (`InputPorts` / `OutputPorts`) a través de `public_interface`.
//! La lógica pura vive en `domain`; los adaptadores de infraestructura en
//! `orchestrator`.
//!
//! ## Reglas
//!
//! - `public_interface` es el ÚNICO módulo `pub` del crate.
//! - `domain` no importa nada de `shared` que toque I/O.
//! - Prohibido depender de otros feature crates — solo `shared`.

mod domain;
mod orchestrator;
mod persistence;
mod schemas;

pub mod public_interface;

#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles_and_links() {
        assert_eq!(2 + 2, 4);
    }
}
