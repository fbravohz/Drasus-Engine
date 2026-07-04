//! [CORE] Lógica pura de Plan / Tier / Quota (`docs/features/plan-tier-quota.md`,
//! ADR-0144, ADR-0143, ADR-0141, ADR-0020 V2, STORY-029).
//!
//! Sin I/O, sin reloj de sistema, sin aleatoriedad sin semilla
//! (ADR-0002/0004). Piezas de lógica pura, tal como las pide la Feature en
//! su "Estructura Interna (FCIS)" y la Orden STORY-029 §4.2:
//! - [`validate_plan`]: coherencia de un plan candidato (tier + cuotas +
//!   precio) -- "NUNCA un plan sin tier ni sin cuota declarada"
//!   (plan-tier-quota.md "Restricciones").
//! - [`resolve_limits`]: dado un plan ya cargado, produce el veredicto
//!   `PlanLimits` que el puerto `plan_limits_out` expone.
//! - [`canonical_features_json`] / [`decode_features_json`]: codificación
//!   determinista del conjunto de features habilitadas (sin `REAL`, sin
//!   orden ambiguo).
//! - [`compute_plan_audit_hash`]: hash de auditoría encadenado por
//!   `row_version` de la fila de `plans` (mismo patrón que
//!   `licensing_system::compute_license_audit_hash`).
//!
//! ## Sobre el nombre `PlanLimits` y su convivencia con `licensing_system::PlanLimits`
//!
//! El catálogo de tipos de puerto (ADR-0137, enmienda 2026-07-03) fija
//! `PlanLimits` como el tipo que **esta** Feature produce por el puerto
//! `plan_limits_out`. Cuando `licensing-system` (cimiento #2, STORY-028) se
//! construyó, `plan-tier-quota` (este cimiento #3) todavía no existía --
//! por eso `licensing-system` declaró su PROPIO struct `PlanLimits`
//! (`domain::licensing_system::PlanLimits`, con solo `max_activations` +
//! `features_enabled`, SIN `notional_limit`) como **stub** temporal
//! (ADR-0144: "puerto ahora, adaptador después"). Ese código ya está
//! sellado (STORY-028 cerrada) y sus tests usan literales `PlanLimits {
//! max_activations, features_enabled }` -- añadirle aquí un campo
//! `notional_limit` reventaría esos literales sin tocar la Story #2, que
//! la Orden de ESTA Story prohíbe expresamente ("Re-cableado de
//! licensing-system... NO parte de esta Orden", STORY-029 §8).
//!
//! Por eso el `PlanLimits` de ESTE módulo es un tipo Rust propio, completo
//! (con `notional_limit` incluido, tal como pide el catálogo), que
//! convive bajo su propio namespace (`public_interface::plan_tier_quota`)
//! sin colisionar con el símbolo ya existente. El día que el "follow-up de
//! integración" (STORY-029 §8) re-cablee `licensing-system` para consumir
//! ESTE `PlanLimits` real, el stub de `licensing_system.rs` se retira y
//! ambos convergen en un solo tipo -- no antes.

use serde::Serialize;
use sha2::{Digest, Sha256};

/// Codifica bytes crudos a su representación hexadecimal en minúsculas
/// (mismo patrón que `licensing_system::encode_hex`/`central_identity`).
fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ── Tier del catálogo de planes ─────────────────────────────────────────────

/// Tier de un plan del catálogo (`docs/features/plan-tier-quota.md`
/// "Parámetros Configurables": `TIER_SET`, default Free/Paid).
///
/// **Vocabulario propio de este catálogo** -- distinto de
/// `licensing_system::LicenseTier` (`SOVEREIGN`/`EXPLORER`). Ambos
/// conceptos describen la misma idea de negocio (gratuito vs. de pago),
/// pero hoy son dos tipos Rust independientes porque nacieron en Stories
/// distintas; unificarlos es, otra vez, el "follow-up de integración"
/// diferido de STORY-029 §8, no esta Story.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum PlanTier {
    /// Plan gratuito -- cuotas más bajas, sin costo.
    Free,
    /// Plan de pago -- cuotas más altas, con precio > 0 (típicamente).
    Paid,
}

impl PlanTier {
    /// Representación canónica en texto (la que se persiste en la columna
    /// `tier` y la que acepta el `CHECK` de la migración).
    pub fn as_str(&self) -> &'static str {
        match self {
            PlanTier::Free => "FREE",
            PlanTier::Paid => "PAID",
        }
    }

    /// Reconstruye el tier desde el valor persistido/JSON, o `None` si no
    /// es ninguno de los dos reconocidos (integridad de datos).
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "FREE" => Some(PlanTier::Free),
            "PAID" => Some(PlanTier::Paid),
            _ => None,
        }
    }
}

// ── Modelo de precios ────────────────────────────────────────────────────────

/// Cómo lee el adaptador de billing este catálogo (`docs/features/plan-tier-quota.md`
/// "Parámetros Configurables": `PRICING_MODEL`). El catálogo es el mismo
/// para ambos modelos -- "el adaptador de billing elige cómo leerlo"
/// (plan-tier-quota.md "Comportamientos Observables").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PricingModel {
    /// Tarifa fija por ciclo de facturación.
    Flat,
    /// Cobro proporcional al volumen nocional operado.
    Volume,
}

impl PricingModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            PricingModel::Flat => "FLAT",
            PricingModel::Volume => "VOLUME",
        }
    }

    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "FLAT" => Some(PricingModel::Flat),
            "VOLUME" => Some(PricingModel::Volume),
            _ => None,
        }
    }
}

// ── Codificación determinista del conjunto de features habilitadas ─────────

/// Codifica el conjunto de features habilitadas de un plan a su
/// representación canónica JSON -- una lista ORDENADA alfabéticamente y
/// sin duplicados, para que el MISMO conjunto de features, insertado en
/// cualquier orden, produzca bit-a-bit el mismo texto persistido
/// (ADR-0002/0004: determinismo). Ver el comentario de la columna
/// `features_enabled` en la migración `0009_plan_tier_quota.sql` para la
/// justificación de por qué es JSON-de-lista-TEXT y no una tabla hija M:N.
pub fn canonical_features_json(features: &[String]) -> String {
    let mut sorted: Vec<&str> = features.iter().map(String::as_str).collect();
    sorted.sort_unstable();
    sorted.dedup();
    // Un `Vec<&str>` siempre serializa a JSON válido -- no hay claves de
    // mapa no compatibles ni ciclos que puedan hacer fallar a serde_json.
    serde_json::to_string(&sorted).expect("Vec<&str> siempre serializa a JSON válido")
}

/// Decodifica la representación canónica JSON de vuelta a la lista de
/// features -- inverso de [`canonical_features_json`]. Devuelve `None` si
/// el texto persistido no es JSON válido de lista de strings (error de
/// integridad de datos -- no debería ocurrir si solo esta Feature escribió
/// la columna, protegido además por el `CHECK (json_valid(...))` de la
/// migración).
pub fn decode_features_json(json: &str) -> Option<Vec<String>> {
    serde_json::from_str(json).ok()
}

// ── Validación de coherencia de un plan candidato ───────────────────────────

/// Un plan candidato a validar -- los campos crudos que alguien propone
/// para un plan nuevo o una revisión, ANTES de persistir. `tier` es
/// `Option` deliberadamente: es la única forma de representar "no se
/// declaró tier" en un tipo fuertemente tipado (la restricción "NUNCA un
/// plan sin tier" se prueba pasando `None` aquí).
#[derive(Debug, Clone, Copy)]
pub struct PlanCandidate<'a> {
    pub tier: Option<PlanTier>,
    pub notional_limit: i64,
    pub max_activations: i64,
    pub price: i64,
    pub pricing_model: PricingModel,
    pub features_enabled: &'a [String],
}

/// Por qué un [`PlanCandidate`] no pasa [`validate_plan`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PlanValidationError {
    /// `docs/features/plan-tier-quota.md` "Restricciones": "NUNCA un plan
    /// sin tier".
    #[error("el plan no declara un tier")]
    MissingTier,
    /// `docs/features/plan-tier-quota.md` "Restricciones": "NUNCA un plan
    /// ... sin cuota declarada" -- ambas cuotas (nocional y activaciones)
    /// en cero a la vez significa que el plan no otorga NINGÚN acceso.
    #[error("el plan no declara ninguna cuota (volumen nocional y activaciones máximas son ambos cero)")]
    MissingQuota,
    #[error("el límite de volumen nocional no puede ser negativo")]
    NegativeNotionalLimit,
    #[error("las activaciones máximas no pueden ser negativas")]
    NegativeMaxActivations,
    #[error("el precio no puede ser negativo")]
    NegativePrice,
}

/// Valida la coherencia de un [`PlanCandidate`] -- tier presente, al menos
/// una cuota declarada (nocional o activaciones), y ningún monto negativo.
///
/// Pura y determinista (ADR-0002/0004): sin I/O, sin reloj, sin azar --
/// dado el mismo candidato, siempre devuelve el mismo resultado. La
/// validación de que `tier`/`pricing_model` son uno de los valores
/// reconocidos ya la garantiza el TIPO (un `PlanTier`/`PricingModel`
/// inválido no puede construirse) y, en la frontera de persistencia, el
/// `CHECK` de la migración (doble validación, ADR-0141).
pub fn validate_plan(candidate: &PlanCandidate<'_>) -> Result<(), PlanValidationError> {
    // Guarda #1: sin tier declarado, el candidato se rechaza de entrada --
    // ninguna otra validación tiene sentido sobre un plan sin identidad de
    // tier.
    if candidate.tier.is_none() {
        return Err(PlanValidationError::MissingTier);
    }

    if candidate.notional_limit < 0 {
        return Err(PlanValidationError::NegativeNotionalLimit);
    }
    if candidate.max_activations < 0 {
        return Err(PlanValidationError::NegativeMaxActivations);
    }
    if candidate.price < 0 {
        return Err(PlanValidationError::NegativePrice);
    }

    // Guarda #2: ambas cuotas en cero -> el plan no otorga ningún acceso
    // real, lo cual la Feature declara explícitamente como inválido.
    if candidate.notional_limit == 0 && candidate.max_activations == 0 {
        return Err(PlanValidationError::MissingQuota);
    }

    Ok(())
}

// ── Resolución de límites (puerto `plan_limits_out`, ADR-0137) ──────────────

/// Vista mínima de un plan YA CARGADO, suficiente para resolver sus
/// límites -- deliberadamente NO es la fila persistida completa (esa vive
/// en `persistence::plan_tier_quota::Plan`, que es responsabilidad de la
/// Shell). El Core nunca depende de un tipo de la Shell (FCIS, ADR-0002):
/// la Shell mapea su fila a este snapshot antes de llamar a
/// [`resolve_limits`].
#[derive(Debug, Clone, Copy)]
pub struct PlanSnapshot<'a> {
    pub tier: PlanTier,
    pub notional_limit: i64,
    pub max_activations: i64,
    pub features_enabled: &'a [String],
}

/// El tipo de puerto `PlanLimits` (ADR-0137 catálogo, enmienda 2026-07-03):
/// "Límites vigentes de un plan (volumen nocional, activaciones, features
/// habilitadas)".
///
/// **Guardarraíl ADR-0093 (estructural):** este struct SOLO tiene los tres
/// campos de abajo -- ninguna credencial de bróker, IP de servidor live, ni
/// secreto de ningún tipo. El test
/// [`tests::plan_limits_json_never_leaks_secret_fields`] fija la lista
/// exacta de claves permitidas en el JSON serializado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanLimits {
    /// Volumen nocional permitido (INTEGER escalado ×10⁸, ADR-0141 --
    /// NUNCA float en ninguna capa).
    pub notional_limit: i64,
    /// Activaciones (máquinas distintas) máximas permitidas.
    pub max_activations: i64,
    /// Features del catálogo habilitadas para este plan.
    pub features_enabled: Vec<String>,
}

/// Resuelve los límites vigentes de un plan ya cargado -- "¿qué límites
/// aplican a esta licencia?" (plan-tier-quota.md "Ciclo de Vida" -
/// "Salida"). Pura: solo repackage de campos ya evaluados, sin I/O.
pub fn resolve_limits(snapshot: &PlanSnapshot<'_>) -> PlanLimits {
    PlanLimits {
        notional_limit: snapshot.notional_limit,
        max_activations: snapshot.max_activations,
        features_enabled: snapshot.features_enabled.to_vec(),
    }
}

// ── Hash de auditoría encadenado (row_version, ADR-0141) ─────────────────────

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de una VERSIÓN de fila
/// de `plans`, encadenado al `audit_hash` de la versión anterior de esa
/// misma fila (o [`crate::domain::audit_log::GENESIS_PREVIOUS_HASH`] si es
/// la versión génesis, `row_version == 1`). Mismo patrón que
/// `licensing_system::compute_license_audit_hash` -- la cadena es POR
/// PLAN (cada fila de `plans` encadena sus propias versiones), no una
/// cadena global entre todos los planes.
#[allow(clippy::too_many_arguments)]
pub fn compute_plan_audit_hash(
    id: &str,
    updated_at_ns: i64,
    row_version: i64,
    previous_audit_hash: Option<&str>,
    owner_id: &str,
    tier: PlanTier,
    notional_limit: i64,
    max_activations: i64,
    price: i64,
    pricing_model: PricingModel,
    features_enabled_json: &str,
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
    push(previous_audit_hash.unwrap_or(crate::domain::audit_log::GENESIS_PREVIOUS_HASH));
    push(owner_id);
    push(tier.as_str());
    push(&notional_limit.to_string());
    push(&max_activations.to_string());
    push(&price.to_string());
    push(pricing_model.as_str());
    push(features_enabled_json);

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    let digest = hasher.finalize();

    encode_hex(&digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_candidate<'a>(features: &'a [String]) -> PlanCandidate<'a> {
        PlanCandidate {
            tier: Some(PlanTier::Free),
            notional_limit: 1_000_000_000_000, // $10,000.00 * 1e8
            max_activations: 1,
            price: 0,
            pricing_model: PricingModel::Flat,
            features_enabled: features,
        }
    }

    // ── CRITERIO #3 (Orden §5): validate_plan rechaza plan sin tier/cuota ──

    /// CRITERIO DE CIERRE: un candidato sin tier se rechaza -- discriminante:
    /// si `validate_plan` aceptara cualquier cosa, esta prueba fallaría.
    #[test]
    fn validate_plan_rejects_missing_tier() {
        let features: Vec<String> = vec![];
        let mut candidate = valid_candidate(&features);
        candidate.tier = None;

        assert_eq!(validate_plan(&candidate), Err(PlanValidationError::MissingTier));
    }

    /// CRITERIO DE CIERRE: un candidato con AMBAS cuotas en cero se
    /// rechaza, aunque tenga tier.
    #[test]
    fn validate_plan_rejects_missing_quota() {
        let features: Vec<String> = vec![];
        let mut candidate = valid_candidate(&features);
        candidate.notional_limit = 0;
        candidate.max_activations = 0;

        assert_eq!(validate_plan(&candidate), Err(PlanValidationError::MissingQuota));
    }

    #[test]
    fn validate_plan_accepts_a_coherent_candidate() {
        let features: Vec<String> = vec![];
        let candidate = valid_candidate(&features);
        assert_eq!(validate_plan(&candidate), Ok(()));
    }

    #[test]
    fn validate_plan_accepts_when_only_activations_quota_is_declared() {
        let features: Vec<String> = vec![];
        let mut candidate = valid_candidate(&features);
        candidate.notional_limit = 0;
        candidate.max_activations = 1;
        assert_eq!(validate_plan(&candidate), Ok(()));
    }

    #[test]
    fn validate_plan_rejects_negative_amounts() {
        let features: Vec<String> = vec![];

        let mut negative_notional = valid_candidate(&features);
        negative_notional.notional_limit = -1;
        assert_eq!(validate_plan(&negative_notional), Err(PlanValidationError::NegativeNotionalLimit));

        let mut negative_activations = valid_candidate(&features);
        negative_activations.max_activations = -1;
        assert_eq!(validate_plan(&negative_activations), Err(PlanValidationError::NegativeMaxActivations));

        let mut negative_price = valid_candidate(&features);
        negative_price.price = -1;
        assert_eq!(validate_plan(&negative_price), Err(PlanValidationError::NegativePrice));
    }

    // ── CRITERIO #4 (Orden §5): resolve_limits por tier ─────────────────────

    #[test]
    fn resolve_limits_maps_free_tier_quota() {
        let features = vec!["basic_backtest".to_string()];
        let snapshot = PlanSnapshot {
            tier: PlanTier::Free,
            notional_limit: 1_000_000_000_000,
            max_activations: 1,
            features_enabled: &features,
        };
        let limits = resolve_limits(&snapshot);
        assert_eq!(limits.notional_limit, 1_000_000_000_000);
        assert_eq!(limits.max_activations, 1);
        assert_eq!(limits.features_enabled, vec!["basic_backtest".to_string()]);
    }

    #[test]
    fn resolve_limits_maps_paid_tier_quota() {
        let features: Vec<String> = vec![];
        let snapshot = PlanSnapshot {
            tier: PlanTier::Paid,
            notional_limit: 100_000_000_000_000,
            max_activations: 3,
            features_enabled: &features,
        };
        let limits = resolve_limits(&snapshot);
        assert_eq!(limits.notional_limit, 100_000_000_000_000);
        assert_eq!(limits.max_activations, 3);
    }

    // ── CRITERIO #8 (Orden §5): guardarraíl ADR-0093 -- sin secretos ────────

    /// CRITERIO DE CIERRE (guardarraíl ADR-0093): el JSON serializado de
    /// `PlanLimits` contiene EXACTAMENTE estas tres claves -- ninguna
    /// credencial de bróker, IP de servidor live, ni clave de firma.
    #[test]
    fn plan_limits_json_never_leaks_secret_fields() {
        let limits = PlanLimits {
            notional_limit: 1_000_000_000_000,
            max_activations: 1,
            features_enabled: vec!["basic_backtest".to_string()],
        };

        let json = serde_json::to_value(&limits).expect("PlanLimits debe serializar a JSON");
        let object = json.as_object().expect("el JSON de PlanLimits debe ser un objeto");

        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();

        assert_eq!(
            keys,
            vec!["features_enabled", "max_activations", "notional_limit"],
            "PlanLimits solo puede exponer estas tres claves (ADR-0093)"
        );

        let json_string = json.to_string();
        for forbidden in ["password", "api_key", "api-key", "broker_secret", "private_key", "signing_key", "192.168.", "10.0.0."] {
            assert!(
                !json_string.to_lowercase().contains(forbidden),
                "el JSON de PlanLimits no debe contener '{forbidden}'"
            );
        }
    }

    // ── Tier / PricingModel: round-trip de representación en texto ─────────

    #[test]
    fn plan_tier_round_trips_through_its_string_representation() {
        for tier in [PlanTier::Free, PlanTier::Paid] {
            assert_eq!(PlanTier::from_str_value(tier.as_str()), Some(tier));
        }
        assert_eq!(PlanTier::from_str_value("UNKNOWN"), None);
    }

    #[test]
    fn pricing_model_round_trips_through_its_string_representation() {
        for model in [PricingModel::Flat, PricingModel::Volume] {
            assert_eq!(PricingModel::from_str_value(model.as_str()), Some(model));
        }
        assert_eq!(PricingModel::from_str_value("UNKNOWN"), None);
    }

    // ── Codificación determinista de features_enabled ──────────────────────

    /// CRITERIO DE CIERRE: el MISMO conjunto de features, en distinto orden
    /// de entrada, produce el MISMO JSON persistido -- si la función no
    /// ordenara, esta prueba fallaría.
    #[test]
    fn canonical_features_json_is_order_independent() {
        let a = vec!["vps_headless".to_string(), "advanced_backtest".to_string()];
        let b = vec!["advanced_backtest".to_string(), "vps_headless".to_string()];
        assert_eq!(canonical_features_json(&a), canonical_features_json(&b));
    }

    #[test]
    fn canonical_features_json_deduplicates() {
        let with_dupe = vec!["a".to_string(), "a".to_string(), "b".to_string()];
        let without_dupe = vec!["a".to_string(), "b".to_string()];
        assert_eq!(canonical_features_json(&with_dupe), canonical_features_json(&without_dupe));
    }

    #[test]
    fn features_json_round_trips() {
        let features = vec!["b".to_string(), "a".to_string()];
        let json = canonical_features_json(&features);
        let decoded = decode_features_json(&json).expect("JSON canónico debe decodificar");
        assert_eq!(decoded, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn decode_features_json_rejects_malformed_json() {
        assert_eq!(decode_features_json("no es json"), None);
    }

    // ── Hash de auditoría encadenado ─────────────────────────────────────────

    #[test]
    fn compute_plan_audit_hash_is_deterministic() {
        let hash_a = compute_plan_audit_hash(
            "id-1", 1_000, 1, None, "owner-1", PlanTier::Free, 1_000_000_000_000, 1, 0, PricingModel::Flat, "[]",
        );
        let hash_b = compute_plan_audit_hash(
            "id-1", 1_000, 1, None, "owner-1", PlanTier::Free, 1_000_000_000_000, 1, 0, PricingModel::Flat, "[]",
        );
        assert_eq!(hash_a, hash_b);
    }

    /// CRITERIO DE CIERRE (Orden §5, criterio #5): cambiar el
    /// `notional_limit` cambia el hash de auditoría -- si el campo no
    /// entrara en el hash, esta prueba fallaría con hashes iguales.
    #[test]
    fn compute_plan_audit_hash_changes_when_notional_limit_changes() {
        let original = compute_plan_audit_hash(
            "id-1", 2_000, 2, Some("prev"), "owner-1", PlanTier::Paid, 100_000_000_000_000, 3, 4_900_000_000, PricingModel::Flat, "[]",
        );
        let changed = compute_plan_audit_hash(
            "id-1", 2_000, 2, Some("prev"), "owner-1", PlanTier::Paid, 200_000_000_000_000, 3, 4_900_000_000, PricingModel::Flat, "[]",
        );
        assert_ne!(original, changed, "cambiar el límite nocional debe cambiar el hash de auditoría");
    }
}
