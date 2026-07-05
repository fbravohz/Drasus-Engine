//! [CORE] Lógica pura del Registro de Consentimiento / ToS (Consent Registry)
//! (`docs/features/consent-registry.md`, ADR-0144, ADR-0143, ADR-0141,
//! ADR-0020, STORY-031).
//!
//! Sin I/O, sin reloj de sistema, sin aleatoriedad sin semilla
//! (ADR-0002/0004). Piezas de lógica pura que pide la Feature en su
//! "Estructura Interna (FCIS)" y la Orden STORY-031 §4:
//! - [`needs_reacceptance`]: compara la versión aceptada contra la vigente
//!   -- `REACCEPT_ON_VERSION_CHANGE` es FIJO (siempre true), así que esta
//!   función no tiene parámetro de configuración: cualquier diferencia de
//!   texto exige re-aceptación.
//! - [`resolve_coverage`]: EL punto de correctitud legal de esta Story --
//!   decide si un tipo de dato concreto está cubierto por el consentimiento
//!   VIGENTE de un usuario. El default es **negar** (nunca se asume
//!   consentimiento, GDPR).
//! - [`parse_optout_map`]: parseo puro y determinista del JSON de
//!   opt-outs hacia un `BTreeMap` (ver la nota de abajo sobre por qué
//!   `BTreeMap` y no `HashMap`).
//! - [`apply_consent_action`]: EL punto de modelado crítico -- fusiona el
//!   estado vigente (o ninguno, si es la primera acción) con una acción
//!   nueva (aceptar versión / cambiar opt-outs) y produce el snapshot
//!   COMPLETO que la Shell persistirá como fila-evento nueva. Así se
//!   modela un estado MUTABLE (los opt-outs cambian) sobre una tabla
//!   INMUTABLE (append-only): nunca se edita una fila, se inserta un
//!   snapshot nuevo que ya incorpora el cambio.
//!
//! ## Por qué `BTreeMap<String, bool>` y no `HashMap<String, bool>`
//!
//! El mapa de opt-outs se serializa a JSON para persistirse Y para entrar
//! al cálculo del `audit_hash` encadenado. `HashMap` en Rust NO garantiza
//! un orden de iteración estable entre ejecuciones (usa un `RandomState`
//! con semilla aleatoria por proceso, como protección contra ataques de
//! colisión de hash) -- dos ejecuciones con el MISMO mapa lógico
//! producirían dos JSON con las claves en orden distinto, y por lo tanto
//! dos `audit_hash` distintos para el MISMO estado de consentimiento. Eso
//! rompe el invariante de determinismo (ADR-0002/0004: "mismo input →
//! mismo output, bit a bit"). `BTreeMap` ordena sus claves siempre
//! alfabéticamente, así que `serde_json::to_string` produce SIEMPRE el
//! mismo JSON para el mismo contenido lógico, sin importar cuántas veces
//! se ejecute el proceso.

use std::collections::BTreeMap;

use serde::Serialize;
use sha2::{Digest, Sha256};

/// Codifica bytes crudos a su representación hexadecimal en minúsculas
/// (mismo patrón que `usage_metering::encode_hex` / `licensing_system`).
fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ── Acción de consentimiento (persistida en la columna `consent_action`) ────

/// Qué tipo de evento de consentimiento representa una fila de
/// `consent_records` (`docs/features/consent-registry.md`, migración
/// `0011_consent_registry.sql`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsentAction {
    /// Primera aceptación de un ToS por parte de este usuario.
    Accept,
    /// Re-aceptación de un ToS tras un cambio de versión vigente.
    Reaccept,
    /// El usuario solo ajustó uno o más opt-outs; la versión de ToS no
    /// cambió respecto al evento anterior.
    OptoutChange,
}

impl ConsentAction {
    /// Representación canónica en texto (la que persiste la columna
    /// `consent_action` y la que acepta el `CHECK` de la migración).
    pub fn as_str(&self) -> &'static str {
        match self {
            ConsentAction::Accept => "ACCEPT",
            ConsentAction::Reaccept => "REACCEPT",
            ConsentAction::OptoutChange => "OPTOUT_CHANGE",
        }
    }

    /// Reconstruye la acción desde el valor persistido (o el valor que
    /// llega por el JSON del CLI de verificación), o `None` si no es
    /// ninguno de los tres reconocidos.
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "ACCEPT" => Some(ConsentAction::Accept),
            "REACCEPT" => Some(ConsentAction::Reaccept),
            "OPTOUT_CHANGE" => Some(ConsentAction::OptoutChange),
            _ => None,
        }
    }
}

// ── Razón de no-cobertura ────────────────────────────────────────────────────

/// Por qué [`resolve_coverage`] decidió `NotCovered` para un tipo de dato
/// concreto (`docs/features/consent-registry.md` "Comportamientos
/// Observables").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NotCoveredReason {
    /// La versión de ToS aceptada por el usuario ya no es la vigente --
    /// exige re-aceptación antes de cubrir CUALQUIER tipo de dato
    /// (`REACCEPT_ON_VERSION_CHANGE`, FIJO).
    StaleVersion,
    /// El usuario optó explícitamente por NO participar con este tipo de
    /// dato concreto -- el opt-out granular manda sobre una versión
    /// vigente aceptada.
    OptedOut,
    /// El usuario no tiene NINGÚN evento de consentimiento registrado --
    /// el default es negar, nunca asumir consentimiento implícito.
    NoConsent,
}

// ── Veredicto de consentimiento (puerto `consent_out` -> `ConsentVerdict`) ──

/// El tipo de puerto `ConsentVerdict` (ADR-0137 catálogo, enmienda
/// 2026-07-03): "Cobertura de consentimiento/ToS por tipo de dato (GDPR)".
///
/// **Guardarraíl ADR-0093 (estructural):** este tipo SOLO puede serializar
/// a `{"verdict": "COVERED"}` o `{"verdict": "NOT_COVERED", "reason":
/// "..."}` -- ninguna credencial de bróker, IP de servidor live, ni
/// secreto de ningún tipo. El test
/// [`tests::consent_verdict_json_never_leaks_secret_fields`] fija la lista
/// exacta de claves permitidas en el JSON serializado.
///
/// Serialización "adjacently tagged" (`tag` + `content`): la variante sin
/// datos (`Covered`) serializa solo la clave `verdict`; la variante con
/// datos (`NotCovered`) añade la clave `reason`. Es la forma estándar de
/// Serde de representar un enum con variantes heterogéneas sin escribir un
/// `impl Serialize` manual.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "verdict", content = "reason", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConsentVerdict {
    /// El tipo de dato consultado está cubierto: la versión aceptada es la
    /// vigente Y el tipo no está en opt-out.
    Covered,
    /// No cubierto, con la razón exacta (nunca un booleano ciego -- quien
    /// consume el puerto necesita saber SI debe pedir re-aceptación o
    /// simplemente respetar un opt-out).
    NotCovered(NotCoveredReason),
}

impl ConsentVerdict {
    /// Azúcar de lectura: `true` solo para la variante `Covered`. Útil en
    /// los `if` de la Shell sin tener que hacer `matches!` en cada sitio.
    pub fn is_covered(&self) -> bool {
        matches!(self, ConsentVerdict::Covered)
    }
}

// ── Estado de consentimiento vigente (snapshot completo, event-sourced) ─────

/// El estado de consentimiento VIGENTE de un usuario: el snapshot completo
/// tomado de la fila con `event_sequence_id` MÁXIMO para su `owner_id`
/// (`docs/features/consent-registry.md` "EL punto de modelado crítico").
/// NUNCA se reconstruye por "fold" incremental sobre el historial -- cada
/// fila ya trae el estado completo, así que basta con leer la última.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsentState {
    /// Versión de ToS aceptada en el ÚLTIMO evento del usuario.
    pub accepted_version: String,
    /// Mapa de opt-outs vigente completo (`true` = el tipo de dato está en
    /// opt-out). Ausencia de una clave significa "no opted-out" -- ver
    /// [`resolve_coverage`].
    pub optout_map: BTreeMap<String, bool>,
}

// ── Re-aceptación forzada por cambio de versión (FIJO) ──────────────────────

/// Decide si un usuario necesita re-aceptar el ToS: `true` si la versión
/// que aceptó (`accepted_version`) difiere, en lo que sea, de la versión
/// vigente (`current_version`).
///
/// `REACCEPT_ON_VERSION_CHANGE` es FIJO (`docs/features/consent-registry.md`
/// "Parámetros Configurables") -- por eso esta función no recibe ningún
/// parámetro de configuración: cualquier cambio de versión, sin excepción,
/// exige re-aceptación. Comparación de igualdad de texto exacta -- "v2" y
/// "v2.0" son versiones DISTINTAS a propósito (el operador decide el
/// vocabulario de versión, esta función no normaliza nada).
pub fn needs_reacceptance(accepted_version: &str, current_version: &str) -> bool {
    accepted_version != current_version
}

// ── Resolución de cobertura (EL punto de correctitud legal) ─────────────────

/// Resuelve si `data_type` está cubierto por el consentimiento vigente de
/// un usuario (`docs/features/consent-registry.md` "Restricciones": "NUNCA
/// se procesa dato del usuario sin un consentimiento vigente que lo
/// cubra").
///
/// ## Las tres puertas, en orden (el orden importa para la razón reportada)
///
/// 1. `consent_state` es `None` (el usuario no tiene ningún evento
///    registrado) → [`NotCoveredReason::NoConsent`]. Esta es la puerta del
///    default-niega: sin fila, sin cobertura, nunca se asume lo contrario.
/// 2. La versión aceptada ya no es la vigente
///    ([`needs_reacceptance`] = `true`) → [`NotCoveredReason::StaleVersion`]
///    para TODO tipo de dato, sin excepción -- una versión vieja no cubre
///    nada hasta que el usuario re-acepte.
/// 3. El tipo de dato está en el mapa de opt-outs con valor `true` →
///    [`NotCoveredReason::OptedOut`] -- el opt-out granular manda incluso
///    con la versión vigente aceptada.
///
/// Si las tres puertas se superan: [`ConsentVerdict::Covered`]. Un tipo de
/// dato ausente del mapa de opt-outs (`.get(...)` devuelve `None`) se trata
/// como "no opted-out" (`unwrap_or(false)`) -- ausencia de opt-out NO es lo
/// mismo que ausencia de consentimiento (esa la resuelve la puerta 1).
pub fn resolve_coverage(
    consent_state: Option<&ConsentState>,
    data_type: &str,
    current_version: &str,
) -> ConsentVerdict {
    // Puerta 1 -- sin ninguna fila para este usuario, el default es negar.
    let state = match consent_state {
        Some(state) => state,
        None => return ConsentVerdict::NotCovered(NotCoveredReason::NoConsent),
    };

    // Puerta 2 -- versión obsoleta invalida TODA cobertura, sin importar
    // el tipo de dato consultado.
    if needs_reacceptance(&state.accepted_version, current_version) {
        return ConsentVerdict::NotCovered(NotCoveredReason::StaleVersion);
    }

    // Puerta 3 -- el opt-out granular de ESTE tipo de dato manda.
    if state.optout_map.get(data_type).copied().unwrap_or(false) {
        return ConsentVerdict::NotCovered(NotCoveredReason::OptedOut);
    }

    ConsentVerdict::Covered
}

// ── Parseo puro del mapa de opt-outs ─────────────────────────────────────────

/// Por qué [`parse_optout_map`] no puede completar el parseo.
#[derive(Debug, thiserror::Error)]
pub enum OptoutMapError {
    /// El texto no es JSON válido, o es JSON válido pero no un objeto de
    /// `{string: bool}` (ej. un array, o valores que no son booleanos).
    #[error("optout_map no es un JSON válido de tipo {{string: bool}}: {0}")]
    InvalidJson(String),
}

/// Parsea el JSON persistido en la columna `optout_map` hacia un
/// `BTreeMap<String, bool>` -- puro y determinista, sin I/O (el `CHECK
/// (json_valid(optout_map))` de la migración ya garantiza JSON
/// sintácticamente válido a nivel de BD; esta función además exige la
/// FORMA esperada: un objeto de claves de texto a booleanos).
pub fn parse_optout_map(json: &str) -> Result<BTreeMap<String, bool>, OptoutMapError> {
    serde_json::from_str(json).map_err(|error| OptoutMapError::InvalidJson(error.to_string()))
}

// ── Fusión de una acción nueva sobre el estado vigente (snapshot completo) ──

/// Entrada de UNA acción de consentimiento del usuario, tal como la recibe
/// la Shell ANTES de fusionarla con el estado vigente (`docs/features/
/// consent-registry.md` "Comportamientos Observables": aceptar ToS o
/// ajustar un opt-out).
#[derive(Debug, Clone)]
pub struct ConsentActionInput {
    /// Qué tipo de evento es esta acción.
    pub action: ConsentAction,
    /// Versión de ToS que se acepta -- relevante para `Accept`/`Reaccept`.
    /// En `OptoutChange` se pasa `None` y se conserva la versión
    /// previamente aceptada (el cambio de opt-out no re-acepta nada).
    pub tos_version: Option<String>,
    /// Solo las claves que CAMBIAN en este evento -- no hace falta repetir
    /// todo el mapa vigente, [`apply_consent_action`] fusiona esto sobre
    /// el snapshot previo para producir el snapshot completo nuevo.
    pub optout_changes: BTreeMap<String, bool>,
}

/// Fusiona el estado vigente (`previous`, o `None` si es la primera acción
/// de este usuario) con una acción nueva, produciendo el
/// [`ConsentState`] COMPLETO que la Shell persistirá como fila-evento
/// nueva.
///
/// ## Por qué esta función existe (el corazón de la Story)
///
/// La tabla `consent_records` es append-only: ninguna fila se edita jamás.
/// Pero el ESTADO LÓGICO que le importa al negocio (¿qué versión aceptó
/// este usuario? ¿qué tipos de dato tiene en opt-out ahora mismo?) SÍ
/// cambia con el tiempo. La única forma de modelar un estado mutable sobre
/// un histórico inmutable sin romper ninguna de las dos propiedades es
/// event-sourcing con snapshot completo: cada fila nueva no describe "el
/// delta" (lo cual obligaría a reconstruir el estado recorriendo TODO el
/// historial, y sería frágil ante una fila corrupta a mitad de camino) --
/// describe el ESTADO COMPLETO resultante. Leer el estado vigente se
/// reduce entonces a "leer la última fila", sin ningún fold.
///
/// ## Reglas de fusión
///
/// - `optout_map`: se parte del mapa previo completo (o vacío si no había
///   estado previo) y se sobrescriben SOLO las claves presentes en
///   `input.optout_changes` -- las claves que no cambian se conservan tal
///   cual venían.
/// - `accepted_version`: si `input.tos_version` trae `Some(...)` (acción
///   `Accept`/`Reaccept`), se usa esa versión nueva. Si trae `None`
///   (acción `OptoutChange`), se conserva la versión previamente aceptada
///   -- si tampoco había estado previo, resuelve a cadena vacía (caso de
///   borde sin sentido práctico: un `OptoutChange` sin haber aceptado
///   nunca un ToS -- la Shell debe impedir esta secuencia antes de
///   llegar aquí, pero la función se mantiene total en su dominio).
pub fn apply_consent_action(previous: Option<&ConsentState>, input: &ConsentActionInput) -> ConsentState {
    // Punto de partida: el mapa previo completo, o vacío si es la primera acción.
    let mut optout_map = previous
        .map(|state| state.optout_map.clone())
        .unwrap_or_default();

    // Sobrescribe SOLO las claves que trae esta acción -- el resto del
    // mapa previo queda intacto (snapshot completo, no delta parcial).
    for (data_type, opted_out) in &input.optout_changes {
        optout_map.insert(data_type.clone(), *opted_out);
    }

    let accepted_version = input.tos_version.clone().unwrap_or_else(|| {
        previous
            .map(|state| state.accepted_version.clone())
            .unwrap_or_default()
    });

    ConsentState {
        accepted_version,
        optout_map,
    }
}

// ── Hash de auditoría encadenado (event_sequence_id, APPEND-ONLY) ───────────

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de una fila de
/// `consent_records`, encadenado al `audit_hash` de la fila anterior en la
/// secuencia GLOBAL (o [`crate::domain::audit_log::GENESIS_PREVIOUS_HASH`]
/// si es la fila génesis, `event_sequence_id == 1`). Misma naturaleza que
/// `usage_metering::compute_usage_audit_hash` -- la cadena es GLOBAL sobre
/// toda la tabla, porque `consent_records` es APPEND-ONLY (ADR-0141:
/// `event_sequence_id UNIQUE`).
///
/// El `optout_map` entra al hash como el STRING JSON ya serializado (no
/// como el `BTreeMap`) -- es exactamente el mismo texto que se persiste en
/// la columna `optout_map`, así que el hash es reproducible a partir de la
/// fila tal cual quedó en disco.
#[allow(clippy::too_many_arguments)]
pub fn compute_consent_audit_hash(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    previous_audit_hash: &str,
    owner_id: &str,
    institutional_tag: &str,
    node_id: &str,
    tos_version: &str,
    consent_action: ConsentAction,
    optout_map_json: &str,
    accepted_at_ns: i64,
) -> String {
    const SEP: char = '\u{1F}';

    let mut buffer = String::new();
    let mut push = |field: &str| {
        buffer.push_str(field);
        buffer.push(SEP);
    };

    push(id);
    push(&created_at_ns.to_string());
    push(&event_sequence_id.to_string());
    push(previous_audit_hash);
    push(owner_id);
    push(institutional_tag);
    push(node_id);
    push(tos_version);
    push(consent_action.as_str());
    push(optout_map_json);
    push(&accepted_at_ns.to_string());

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    let digest = hasher.finalize();

    encode_hex(&digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── CRITERIO #4 (Orden §5): re-aceptación forzada por cambio de versión ──

    #[test]
    fn needs_reacceptance_is_false_when_versions_match() {
        assert!(!needs_reacceptance("v2", "v2"));
    }

    /// CRITERIO DE CIERRE: cualquier diferencia de texto exige
    /// re-aceptación -- `REACCEPT_ON_VERSION_CHANGE` es FIJO, no hay
    /// tolerancia de "casi igual".
    #[test]
    fn needs_reacceptance_is_true_when_versions_differ() {
        assert!(needs_reacceptance("v1", "v2"));
        assert!(needs_reacceptance("v2.0", "v2"), "versiones distintas en texto son distintas, sin normalización");
    }

    // ── CRITERIO #2 (Orden §5): las cuatro puertas de resolve_coverage ──────

    /// CRITERIO DE CIERRE: aceptó la versión vigente, sin opt-out del tipo
    /// consultado -> Covered. Debe fallar si devuelve NotCovered.
    #[test]
    fn resolve_coverage_covers_when_version_matches_and_no_optout() {
        let state = ConsentState {
            accepted_version: "v2".to_string(),
            optout_map: BTreeMap::new(),
        };
        assert_eq!(resolve_coverage(Some(&state), "aggregation", "v2"), ConsentVerdict::Covered);
    }

    /// CRITERIO DE CIERRE: opt-out granular manda -- el tipo X con opt-out
    /// da NotCovered{OptedOut}, pero el tipo Y (sin opt-out) sigue Covered
    /// para el MISMO usuario y la MISMA versión vigente. Debe fallar si el
    /// opt-out de un tipo contamina la cobertura de otro tipo.
    #[test]
    fn resolve_coverage_optout_is_granular_per_data_type() {
        let mut optout_map = BTreeMap::new();
        optout_map.insert("aggregation".to_string(), true);
        let state = ConsentState { accepted_version: "v2".to_string(), optout_map };

        assert_eq!(
            resolve_coverage(Some(&state), "aggregation", "v2"),
            ConsentVerdict::NotCovered(NotCoveredReason::OptedOut)
        );
        assert_eq!(
            resolve_coverage(Some(&state), "firehose", "v2"),
            ConsentVerdict::Covered,
            "un tipo de dato SIN opt-out debe seguir cubierto aunque otro tipo esté opted-out"
        );
    }

    /// CRITERIO DE CIERRE: versión obsoleta niega TODO tipo de dato, sin
    /// importar los opt-outs -- debe fallar si cubre con una versión vieja.
    #[test]
    fn resolve_coverage_stale_version_denies_every_data_type() {
        let state = ConsentState {
            accepted_version: "v1".to_string(),
            optout_map: BTreeMap::new(),
        };
        assert_eq!(
            resolve_coverage(Some(&state), "aggregation", "v2"),
            ConsentVerdict::NotCovered(NotCoveredReason::StaleVersion)
        );
        assert_eq!(
            resolve_coverage(Some(&state), "firehose", "v2"),
            ConsentVerdict::NotCovered(NotCoveredReason::StaleVersion),
            "la versión obsoleta invalida TODO tipo de dato, no solo uno"
        );
    }

    /// CRITERIO DE CIERRE (default = negar): sin ninguna fila de
    /// consentimiento, la resolución NUNCA asume cobertura -- debe fallar
    /// si `None` produjera `Covered`.
    #[test]
    fn resolve_coverage_denies_by_default_without_any_consent() {
        assert_eq!(
            resolve_coverage(None, "aggregation", "v2"),
            ConsentVerdict::NotCovered(NotCoveredReason::NoConsent)
        );
    }

    /// Un tipo de dato AUSENTE del mapa de opt-outs (nunca se tocó) se
    /// trata como "no opted-out" -- ausencia de opt-out no es lo mismo que
    /// ausencia de consentimiento (esa es la puerta 1, separada).
    #[test]
    fn resolve_coverage_treats_missing_optout_key_as_not_opted_out() {
        let state = ConsentState {
            accepted_version: "v2".to_string(),
            optout_map: BTreeMap::new(),
        };
        assert_eq!(resolve_coverage(Some(&state), "never_touched_type", "v2"), ConsentVerdict::Covered);
    }

    // ── Parseo del mapa de opt-outs ──────────────────────────────────────────

    #[test]
    fn parse_optout_map_accepts_valid_json_object() {
        let map = parse_optout_map(r#"{"aggregation":true,"firehose":false}"#).expect("debe parsear");
        assert_eq!(map.get("aggregation"), Some(&true));
        assert_eq!(map.get("firehose"), Some(&false));
    }

    /// CRITERIO DE CIERRE: JSON corrupto se rechaza explícitamente, nunca
    /// se interpreta como mapa vacío ni se hace panic.
    #[test]
    fn parse_optout_map_rejects_malformed_json() {
        assert!(parse_optout_map("{not valid json").is_err());
        assert!(parse_optout_map(r#"["not", "an", "object"]"#).is_err(), "un array no es la forma esperada");
        assert!(parse_optout_map(r#"{"aggregation": "not-a-bool"}"#).is_err(), "los valores deben ser booleanos");
    }

    // ── CRITERIO #3 (Orden §5): fusión de acción (snapshot event-sourced) ───

    /// CRITERIO DE CIERRE: la primera acción (sin estado previo) produce
    /// el snapshot inicial completo a partir de sus propios datos.
    #[test]
    fn apply_consent_action_creates_initial_snapshot_without_previous_state() {
        let mut changes = BTreeMap::new();
        changes.insert("aggregation".to_string(), false);
        let input = ConsentActionInput {
            action: ConsentAction::Accept,
            tos_version: Some("v2".to_string()),
            optout_changes: changes,
        };

        let next = apply_consent_action(None, &input);
        assert_eq!(next.accepted_version, "v2");
        assert_eq!(next.optout_map.get("aggregation"), Some(&false));
    }

    /// CRITERIO DE CIERRE (Orden §5, criterio #3 "snapshot event-sourced"):
    /// cambiar UN opt-out produce un snapshot nuevo que CONSERVA las demás
    /// claves del estado previo, intactas -- debe fallar si el cambio
    /// borrara o ignorara el resto del mapa previo.
    #[test]
    fn apply_consent_action_merges_optout_change_over_previous_snapshot() {
        let mut previous_map = BTreeMap::new();
        previous_map.insert("aggregation".to_string(), false);
        previous_map.insert("firehose".to_string(), false);
        let previous = ConsentState { accepted_version: "v2".to_string(), optout_map: previous_map };

        let mut changes = BTreeMap::new();
        changes.insert("aggregation".to_string(), true); // el usuario ahora se opta fuera de "aggregation"
        let input = ConsentActionInput {
            action: ConsentAction::OptoutChange,
            tos_version: None,
            optout_changes: changes,
        };

        let next = apply_consent_action(Some(&previous), &input);

        assert_eq!(next.accepted_version, "v2", "OptoutChange no debe tocar la versión aceptada");
        assert_eq!(next.optout_map.get("aggregation"), Some(&true), "el cambio debe aplicarse");
        assert_eq!(
            next.optout_map.get("firehose"),
            Some(&false),
            "las claves que NO cambiaron deben conservarse intactas del snapshot previo"
        );
    }

    /// Una re-aceptación (`Reaccept`) actualiza la versión pero conserva el
    /// mapa de opt-outs previo si la acción no trae cambios de opt-out.
    #[test]
    fn apply_consent_action_reaccept_updates_version_and_keeps_previous_optouts() {
        let mut previous_map = BTreeMap::new();
        previous_map.insert("aggregation".to_string(), true);
        let previous = ConsentState { accepted_version: "v1".to_string(), optout_map: previous_map };

        let input = ConsentActionInput {
            action: ConsentAction::Reaccept,
            tos_version: Some("v2".to_string()),
            optout_changes: BTreeMap::new(),
        };

        let next = apply_consent_action(Some(&previous), &input);
        assert_eq!(next.accepted_version, "v2");
        assert_eq!(next.optout_map.get("aggregation"), Some(&true), "el opt-out previo sobrevive a la re-aceptación");
    }

    // ── ConsentAction: round-trip por string ─────────────────────────────────

    #[test]
    fn consent_action_round_trips_through_its_string_representation() {
        for action in [ConsentAction::Accept, ConsentAction::Reaccept, ConsentAction::OptoutChange] {
            assert_eq!(ConsentAction::from_str_value(action.as_str()), Some(action));
        }
        assert_eq!(ConsentAction::from_str_value("UNKNOWN"), None);
    }

    // ── CRITERIO #6 (Orden §5): guardarraíl ADR-0093 -- sin secretos ────────

    /// CRITERIO DE CIERRE (guardarraíl ADR-0093): el JSON serializado de
    /// `ConsentVerdict` contiene EXACTAMENTE estas claves, en ambas
    /// variantes, y nunca ningún campo de secreto.
    #[test]
    fn consent_verdict_json_never_leaks_secret_fields() {
        let covered = serde_json::to_value(ConsentVerdict::Covered).expect("Covered debe serializar");
        let covered_object = covered.as_object().expect("Covered debe ser un objeto JSON");
        let mut covered_keys: Vec<&str> = covered_object.keys().map(String::as_str).collect();
        covered_keys.sort_unstable();
        assert_eq!(covered_keys, vec!["verdict"], "Covered solo debe exponer la clave 'verdict'");

        let not_covered = serde_json::to_value(ConsentVerdict::NotCovered(NotCoveredReason::OptedOut))
            .expect("NotCovered debe serializar");
        let not_covered_object = not_covered.as_object().expect("NotCovered debe ser un objeto JSON");
        let mut not_covered_keys: Vec<&str> = not_covered_object.keys().map(String::as_str).collect();
        not_covered_keys.sort_unstable();
        assert_eq!(
            not_covered_keys,
            vec!["reason", "verdict"],
            "NotCovered solo debe exponer 'verdict' y 'reason'"
        );

        for json_string in [covered.to_string(), not_covered.to_string()] {
            for forbidden in ["password", "api_key", "api-key", "broker_secret", "private_key", "signing_key", "192.168.", "10.0.0."] {
                assert!(
                    !json_string.to_lowercase().contains(forbidden),
                    "el JSON de ConsentVerdict no debe contener '{forbidden}'"
                );
            }
        }
    }

    #[test]
    fn consent_verdict_serializes_to_screaming_snake_case() {
        assert_eq!(serde_json::to_string(&ConsentVerdict::Covered).unwrap(), r#"{"verdict":"COVERED"}"#);
        assert_eq!(
            serde_json::to_string(&ConsentVerdict::NotCovered(NotCoveredReason::NoConsent)).unwrap(),
            r#"{"verdict":"NOT_COVERED","reason":"NO_CONSENT"}"#
        );
    }

    #[test]
    fn consent_verdict_is_covered_helper() {
        assert!(ConsentVerdict::Covered.is_covered());
        assert!(!ConsentVerdict::NotCovered(NotCoveredReason::OptedOut).is_covered());
    }

    // ── Hash de auditoría encadenado ─────────────────────────────────────────

    #[test]
    fn compute_consent_audit_hash_is_deterministic() {
        let hash_a = compute_consent_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1",
            "v2", ConsentAction::Accept, r#"{"aggregation":false}"#, 1_000,
        );
        let hash_b = compute_consent_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1",
            "v2", ConsentAction::Accept, r#"{"aggregation":false}"#, 1_000,
        );
        assert_eq!(hash_a, hash_b);
    }

    /// CRITERIO DE CIERRE: cambiar el `optout_map` (aunque sea un solo
    /// booleano) cambia el hash -- si el campo no entrara en el hash, esta
    /// prueba fallaría con hashes iguales.
    #[test]
    fn compute_consent_audit_hash_changes_when_optout_map_changes() {
        let original = compute_consent_audit_hash(
            "id-1", 2_000, 2, "prev-hash", "owner-1", "DRASUS_LOCAL", "node-1",
            "v2", ConsentAction::OptoutChange, r#"{"aggregation":false}"#, 2_000,
        );
        let changed = compute_consent_audit_hash(
            "id-1", 2_000, 2, "prev-hash", "owner-1", "DRASUS_LOCAL", "node-1",
            "v2", ConsentAction::OptoutChange, r#"{"aggregation":true}"#, 2_000,
        );
        assert_ne!(original, changed, "cambiar el opt-out map debe cambiar el hash de auditoría");
    }
}
