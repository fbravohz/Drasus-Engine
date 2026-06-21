//! [SHELL] Capa anticorrupción stub de NautilusTrader v2 (ADR-0107 — decisión de integración NT).
//! Solo este crate puede importar tipos NT; ningún otro crate del workspace debe tener
//! `use nautilus_*` directamente. La implementación real de la capa anticorrupción
//! (mapeo a tipos propios del dominio) corresponde a EPIC-2 y EPIC-5.

// Módulo público que re-exporta los tipos NT necesarios para la capa anticorrupción.
// Solo re-exportaciones — cero lógica de negocio en este archivo.
pub mod stub {
    // Re-exporta el enum AccountType del modelo de dominio de NT v2.
    // AccountType describe el tipo de cuenta que ofrece un broker o venue
    // (por ejemplo: dinero real, papel simulado, margen, etc.).
    // Es el tipo NT más simple disponible: un enum sin parámetros ni contexto especial.
    pub use nautilus_model::enums::AccountType;
}
