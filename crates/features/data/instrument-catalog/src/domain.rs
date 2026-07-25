//! Functional Core — lógica pura, sin I/O.
//!
//! PROHIBIDO: imports de `sqlx`, `tokio`, o cualquier crate que toque el
//! sistema. PERMITIDO: `shared::types` (si aplica).
//!
//! Implementa la resolución `COALESCE` campo a campo (TTR-003,
//! ADR-0162): dado el canónico (`instrument_master`) y, si existe, el
//! override de un bróker (`broker_instrument_terms`), produce
//! [`ResolvedInstrumentTerms`] -- override no-`NULL` gana, `NULL` hereda
//! del canónico. Es una función pura: mismo input, mismo output,
//! bit-a-bit (ADR-0002).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ── Vocabulario fijo (enums que el CHECK de la migración también valida --
// "doble validación", ADR-0141) ───────────────────────────────────────────

/// Clase de activo (`instrument_master.asset_class`). Reusa el vocabulario
/// de ADR-0140. `SCREAMING_SNAKE_CASE` hace que la forma JSON (`"FOREX"`)
/// coincida EXACTO con el vocabulario del `CHECK` de la migración -- mismo
/// patrón que `hybrid_data_transformer::schemas::BarTypeInput`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssetClass {
    Forex,
    Future,
    Cfd,
    Crypto,
    Equity,
}

impl AssetClass {
    /// Forma canónica que persiste en la columna `asset_class` -- debe
    /// coincidir EXACTO con el vocabulario del `CHECK` de la migración.
    pub fn as_str(self) -> &'static str {
        match self {
            AssetClass::Forex => "FOREX",
            AssetClass::Future => "FUTURE",
            AssetClass::Cfd => "CFD",
            AssetClass::Crypto => "CRYPTO",
            AssetClass::Equity => "EQUITY",
        }
    }

    /// Decodifica el string persistido de vuelta al enum. `None` si el
    /// valor no pertenece al vocabulario -- nunca se aproxima a un valor
    /// parecido.
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "FOREX" => Some(AssetClass::Forex),
            "FUTURE" => Some(AssetClass::Future),
            "CFD" => Some(AssetClass::Cfd),
            "CRYPTO" => Some(AssetClass::Crypto),
            "EQUITY" => Some(AssetClass::Equity),
            _ => None,
        }
    }
}

/// Modelo de comisión (`broker_instrument_terms.commission_model`). Mismo
/// criterio de `rename_all` que [`AssetClass`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommissionModel {
    None,
    PerTrade,
    Percentage,
    SizeBased,
    Stockpicker,
}

impl CommissionModel {
    pub fn as_str(self) -> &'static str {
        match self {
            CommissionModel::None => "NONE",
            CommissionModel::PerTrade => "PER_TRADE",
            CommissionModel::Percentage => "PERCENTAGE",
            CommissionModel::SizeBased => "SIZE_BASED",
            CommissionModel::Stockpicker => "STOCKPICKER",
        }
    }

    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "NONE" => Some(CommissionModel::None),
            "PER_TRADE" => Some(CommissionModel::PerTrade),
            "PERCENTAGE" => Some(CommissionModel::Percentage),
            "SIZE_BASED" => Some(CommissionModel::SizeBased),
            "STOCKPICKER" => Some(CommissionModel::Stockpicker),
            _ => None,
        }
    }
}

// ── Entradas de la resolución ────────────────────────────────────────────

/// Vista mínima de `instrument_master` que necesita la resolución -- solo
/// los campos que `broker_instrument_terms` puede sobreescribir, más la
/// identidad. El repositorio (Shell) mapea la fila completa a este tipo.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalTerms {
    pub instrument_id: String,
    pub asset_class: AssetClass,
    pub canonical_pip_size: i64,
    pub canonical_tick_size: i64,
    pub default_session_template_id: Option<String>,
}

/// Vista mínima de una fila de `broker_instrument_terms` -- todo campo
/// propio es `Option`: `None` es herencia declarada, nunca "dato
/// faltante por error" (Restricción FIJA #1, ADR-0162).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BrokerOverride {
    pub venue: String,
    pub default_spread: Option<i64>,
    pub default_slippage: Option<i64>,
    pub pip_size_override: Option<i64>,
    pub tick_size_override: Option<i64>,
    pub point_value: Option<i64>,
    pub commission_model: Option<CommissionModel>,
    pub commission_value: Option<i64>,
    pub session_template_id_override: Option<String>,
}

/// Salida de la resolución: términos de trading YA combinados
/// (`resolved_terms_out`, ADR-0137) -- el consumidor nunca tiene que
/// implementar la lógica de herencia por su cuenta.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedInstrumentTerms {
    pub instrument_id: String,
    pub venue: Option<String>,
    pub asset_class: AssetClass,
    pub pip_size: i64,
    pub tick_size: i64,
    /// `None` si el override no declaró piso de spread propio Y el
    /// canónico no tiene ningún valor por defecto (el spread real, a
    /// diferencia de pip/tick size, no tiene un valor canónico universal
    /// -- es puramente por bróker).
    pub spread: Option<i64>,
    pub slippage: Option<i64>,
    pub point_value: Option<i64>,
    pub commission_model: Option<CommissionModel>,
    pub commission_value: Option<i64>,
    /// Cadena de resolución de calendario: el override de bróker gana
    /// sobre el default del instrumento (TTR-003). Este campo es el
    /// `template_id` YA resuelto -- session-catalog lo consume tal cual,
    /// sin tocar las tablas de esta feature (ADR-0137: aislamiento de
    /// crates).
    pub resolved_session_template_id: Option<String>,
}

/// Resuelve los términos de un instrumento para un bróker concreto:
/// `COALESCE` campo a campo -- override no-`NULL` gana, `NULL` hereda el
/// valor canónico. `pip_size`/`tick_size` SIEMPRE tienen valor (el
/// canónico es NOT NULL en la migración); el resto son genuinamente
/// opcionales incluso después de resolver, porque `instrument_master` no
/// declara un valor canónico para ellos (spread/comisión son inherentemente
/// dato de bróker, ADR-0162 "Restricciones").
///
/// `override_terms == None` es el caso "el usuario borra el override de un
/// bróker" (criterio de aceptación #3): el resultado resuelve 100% desde
/// el canónico, sin ningún campo de bróker.
pub fn resolve_terms(canonical: &CanonicalTerms, override_terms: Option<&BrokerOverride>) -> ResolvedInstrumentTerms {
    let pip_size = override_terms
        .and_then(|o| o.pip_size_override)
        .unwrap_or(canonical.canonical_pip_size);
    let tick_size = override_terms
        .and_then(|o| o.tick_size_override)
        .unwrap_or(canonical.canonical_tick_size);

    // Cadena de resolución de calendario (TTR-003): override de bróker ->
    // default del instrumento. Reimplementa la MISMA regla pura que
    // `session_catalog::domain::resolve_session_template_id` -- se
    // duplica intencionalmente (no se importa el otro crate, ADR-0137
    // prohíbe el acoplamiento entre features) en vez de compartir código.
    let resolved_session_template_id = override_terms
        .and_then(|o| o.session_template_id_override.clone())
        .or_else(|| canonical.default_session_template_id.clone());

    ResolvedInstrumentTerms {
        instrument_id: canonical.instrument_id.clone(),
        venue: override_terms.map(|o| o.venue.clone()),
        asset_class: canonical.asset_class,
        pip_size,
        tick_size,
        spread: override_terms.and_then(|o| o.default_spread),
        slippage: override_terms.and_then(|o| o.default_slippage),
        point_value: override_terms.and_then(|o| o.point_value),
        commission_model: override_terms.and_then(|o| o.commission_model),
        commission_value: override_terms.and_then(|o| o.commission_value),
        resolved_session_template_id,
    }
}

// ── Integridad de fila (audit_hash encadenado, Grupo I ADR-0020) ────────

/// Texto que alimenta el hash de la fila génesis, cuando no hay
/// `audit_hash` anterior que encadenar. NUNCA se persiste en la columna
/// `audit_chain_hash` (que queda `NULL` en la génesis, ADR-0141 M10) --
/// solo es el relleno determinista del contenido que SÍ se hashea.
const GENESIS_MARKER: &str = "GENESIS";

/// Codifica bytes crudos como hexadecimal en minúsculas. Mismo patrón
/// duplicado localmente que el resto de los módulos de dominio del
/// substrato (`shared::domain::plan_tier_quota::encode_hex` y análogos) --
/// una función pura de una línea no justifica una dependencia compartida.
fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Calcula el `audit_hash` de una versión de `instrument_master`. Hashea
/// SOLO los campos que definen el contenido de negocio de la fila (nunca
/// `id`/`created_at`, que no cambian el significado del instrumento) +
/// `updated_at` + `row_version` + el hash anterior encadenado -- misma
/// receta que `plan_tier_quota::compute_plan_audit_hash`.
#[allow(clippy::too_many_arguments)]
pub fn compute_instrument_master_audit_hash(
    id: &str,
    updated_at_ns: i64,
    row_version: i64,
    previous_audit_hash: Option<&str>,
    instrument_id: &str,
    asset_class: AssetClass,
    canonical_pip_size: i64,
    canonical_tick_size: i64,
    default_session_template_id: Option<&str>,
) -> String {
    const SEP: char = '\u{1F}';
    let mut buffer = String::new();
    let mut push = |field: &str| {
        buffer.push_str(field);
        buffer.push(SEP);
    };

    push(id);
    push(&updated_at_ns.to_string());
    push(&row_version.to_string());
    push(previous_audit_hash.unwrap_or(GENESIS_MARKER));
    push(instrument_id);
    push(asset_class.as_str());
    push(&canonical_pip_size.to_string());
    push(&canonical_tick_size.to_string());
    push(default_session_template_id.unwrap_or(""));

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    encode_hex(&hasher.finalize())
}

/// Calcula el `audit_hash` de una versión de `broker_instrument_terms`.
/// Mismo criterio que [`compute_instrument_master_audit_hash`]: hashea el
/// contenido de negocio + el encadenamiento, nunca `id`/`created_at`.
#[allow(clippy::too_many_arguments)]
pub fn compute_broker_terms_audit_hash(
    id: &str,
    updated_at_ns: i64,
    row_version: i64,
    previous_audit_hash: Option<&str>,
    instrument_id: &str,
    venue: &str,
    default_spread: Option<i64>,
    session_template_id_override: Option<&str>,
) -> String {
    const SEP: char = '\u{1F}';
    let mut buffer = String::new();
    let mut push = |field: &str| {
        buffer.push_str(field);
        buffer.push(SEP);
    };

    push(id);
    push(&updated_at_ns.to_string());
    push(&row_version.to_string());
    push(previous_audit_hash.unwrap_or(GENESIS_MARKER));
    push(instrument_id);
    push(venue);
    push(&default_spread.map(|v| v.to_string()).unwrap_or_default());
    push(session_template_id_override.unwrap_or(""));

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    encode_hex(&hasher.finalize())
}

// ── Pruebas de la lógica pura ────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_canonical() -> CanonicalTerms {
        CanonicalTerms {
            instrument_id: "EURUSD".to_string(),
            asset_class: AssetClass::Forex,
            canonical_pip_size: 10_000, // 0.0001 * 1e8
            canonical_tick_size: 1_000, // 0.00001 * 1e8
            default_session_template_id: Some("tpl-forex-245".to_string()),
        }
    }

    // ── CRITERIO #2 (Orden §5): resolve_terms_null_inherits_nonnull_overrides ──

    /// Un campo `NULL` en el override devuelve el valor canónico; uno
    /// no-`NULL` devuelve el del override -- discriminante campo a campo:
    /// mezcla ambos casos en la MISMA llamada para que un mutante que
    /// invierta la prioridad (canónico gana siempre, u override gana
    /// siempre) no pueda pasar las dos aserciones a la vez.
    #[test]
    fn resolve_terms_null_inherits_nonnull_overrides() {
        let canonical = sample_canonical();
        let override_terms = BrokerOverride {
            venue: "PEPPERSTONE".to_string(),
            // pip_size_override es NULL -> debe heredar el canónico.
            pip_size_override: None,
            // default_spread NO es NULL -> debe ganar sobre "sin canónico".
            default_spread: Some(150_000_000), // 1.5 pip * 1e8 escala de ejemplo
            ..Default::default()
        };

        let resolved = resolve_terms(&canonical, Some(&override_terms));

        assert_eq!(resolved.pip_size, canonical.canonical_pip_size, "NULL en el override debe heredar el canónico");
        assert_eq!(resolved.spread, Some(150_000_000), "un override no-NULL debe ganar sobre el canónico");
    }

    /// Cuando el override SÍ declara `pip_size_override`, ese valor debe
    /// GANAR sobre el canónico -- complemento de la prueba anterior (evita
    /// que "siempre hereda" pase por accidente).
    #[test]
    fn resolve_terms_nonnull_pip_size_override_wins_over_canonical() {
        let canonical = sample_canonical();
        let override_terms = BrokerOverride {
            venue: "IC_MARKETS".to_string(),
            pip_size_override: Some(20_000),
            ..Default::default()
        };

        let resolved = resolve_terms(&canonical, Some(&override_terms));
        assert_eq!(resolved.pip_size, 20_000, "un pip_size_override no-NULL debe ganar sobre el canónico");
        assert_ne!(resolved.pip_size, canonical.canonical_pip_size, "el valor resuelto debe ser DISTINTO del canónico, no una copia accidental");
    }

    // ── CRITERIO #3 (Orden §5): deleting_override_resolves_from_canonical ──

    /// Sin override (`None`, equivalente a "el usuario lo borró"), TODOS
    /// los campos de bróker resuelven `None` y `pip_size`/`tick_size`
    /// resuelven 100% desde el canónico -- sin tocar `instrument_master`.
    #[test]
    fn deleting_override_resolves_from_canonical() {
        let canonical = sample_canonical();

        let resolved = resolve_terms(&canonical, None);

        assert_eq!(resolved.pip_size, canonical.canonical_pip_size);
        assert_eq!(resolved.tick_size, canonical.canonical_tick_size);
        assert_eq!(resolved.spread, None, "sin override, spread no tiene fuente -- debe ser None, no un valor inventado");
        assert_eq!(resolved.venue, None, "sin override, no hay bróker asociado");
        assert_eq!(
            resolved.resolved_session_template_id,
            canonical.default_session_template_id,
            "sin override de calendario, debe resolver el default del instrumento"
        );
    }

    // ── Cadena de resolución de calendario (TTR-003, replicada aquí) ────

    /// El override de calendario del bróker debe ganar sobre el default
    /// del instrumento cuando ambos existen.
    #[test]
    fn resolve_terms_session_template_override_wins_over_instrument_default() {
        let canonical = sample_canonical();
        let override_terms = BrokerOverride {
            venue: "OANDA".to_string(),
            session_template_id_override: Some("tpl-broker-specific".to_string()),
            ..Default::default()
        };

        let resolved = resolve_terms(&canonical, Some(&override_terms));
        assert_eq!(resolved.resolved_session_template_id, Some("tpl-broker-specific".to_string()));
    }

    // ── Vocabulario: round-trip exacto (evita mutantes en las tablas match) ──

    #[test]
    fn asset_class_round_trips_through_its_canonical_string_for_every_variant() {
        for variant in [AssetClass::Forex, AssetClass::Future, AssetClass::Cfd, AssetClass::Crypto, AssetClass::Equity] {
            let text = variant.as_str();
            let decoded = AssetClass::from_str_value(text).expect("todo texto producido por as_str debe decodificar");
            assert_eq!(decoded, variant, "round-trip debe preservar la variante exacta: {text}");
        }
        assert_eq!(AssetClass::from_str_value("BOND"), None, "un valor fuera del vocabulario fijo nunca debe aproximarse a una variante");
    }

    #[test]
    fn commission_model_round_trips_through_its_canonical_string_for_every_variant() {
        for variant in [
            CommissionModel::None,
            CommissionModel::PerTrade,
            CommissionModel::Percentage,
            CommissionModel::SizeBased,
            CommissionModel::Stockpicker,
        ] {
            let text = variant.as_str();
            let decoded = CommissionModel::from_str_value(text).expect("todo texto producido por as_str debe decodificar");
            assert_eq!(decoded, variant, "round-trip debe preservar la variante exacta: {text}");
        }
        assert_eq!(CommissionModel::from_str_value("FLAT_FEE"), None);
    }

    // ── Integridad: el hash depende REALMENTE del contenido (no tautología) ──

    /// Dos llamadas con los MISMOS argumentos, construidos por separado
    /// (nunca la misma variable en ambos lados), producen el mismo hash;
    /// cambiar un solo campo (`canonical_pip_size`) produce un hash
    /// DISTINTO. Sin esto, una función que devolviera una constante fija
    /// pasaría cualquier prueba que solo comparara `f(x)` consigo misma.
    #[test]
    fn instrument_master_audit_hash_depends_on_content_not_just_identity() {
        let hash_a = compute_instrument_master_audit_hash(
            "id-1", 1_000, 1, None, "EURUSD", AssetClass::Forex, 10_000, 1_000, None,
        );
        let hash_a_again = compute_instrument_master_audit_hash(
            "id-1", 1_000, 1, None, "EURUSD", AssetClass::Forex, 10_000, 1_000, None,
        );
        assert_eq!(hash_a, hash_a_again, "las mismas entradas, construidas por separado, deben producir el mismo hash");

        let hash_b = compute_instrument_master_audit_hash(
            "id-1", 1_000, 1, None, "EURUSD", AssetClass::Forex, 20_000, 1_000, None,
        );
        assert_ne!(hash_a, hash_b, "cambiar canonical_pip_size debe cambiar el hash resultante");
    }

    #[test]
    fn broker_terms_audit_hash_depends_on_content_not_just_identity() {
        let hash_a = compute_broker_terms_audit_hash("id-1", 1_000, 1, None, "EURUSD", "OANDA", Some(150_000_000), None);
        let hash_a_again =
            compute_broker_terms_audit_hash("id-1", 1_000, 1, None, "EURUSD", "OANDA", Some(150_000_000), None);
        assert_eq!(hash_a, hash_a_again);

        let hash_b = compute_broker_terms_audit_hash("id-1", 1_000, 1, None, "EURUSD", "OANDA", Some(200_000_000), None);
        assert_ne!(hash_a, hash_b, "cambiar default_spread debe cambiar el hash resultante");
    }
}
