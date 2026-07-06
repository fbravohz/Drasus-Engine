//! [CORE] Lógica pura de Data Anonymization & Aggregation
//! (`docs/features/data-aggregation.md`, ADR-0144 cimiento #9, ADR-0102,
//! ADR-0143, ADR-0137, ADR-0141, ADR-0020, ADR-0093, ADR-0002, STORY-036).
//!
//! Sin I/O, sin reloj de sistema, sin aleatoriedad sin semilla
//! (ADR-0002/0004). Piezas de lógica pura que pide la Orden STORY-036 §4:
//! - [`apply_differential_privacy`]: ruido gaussiano de privacidad
//!   diferencial, generado con un RNG SEMBRADO e inyectado (`seed: u64`)
//!   -- NUNCA `rand::thread_rng()` ni ninguna fuente de entropía del
//!   sistema. Misma semilla + mismo valor crudo -> mismo resultado con
//!   ruido, siempre (reproducible en tests y auditable).
//! - [`hash_strategy_topology`]: comprime cualquier topología/firma de
//!   estrategia cruda a SHA-256 hex ANTES de que entre a cualquier lógica
//!   de agrupación -- el texto crudo nunca sobrevive más allá de esta
//!   llamada (ADR-0102).
//! - [`meets_k_anonymity`]: el tamaño mínimo de cohorte (k-anonimato) --
//!   invariante FIJO, no configurable.
//! - [`aggregate_index`]: EL punto de modelado crítico -- suma los
//!   valores YA filtrados por consentimiento (el orquestador excluye los
//!   no cubiertos ANTES de llegar aquí), aplica el ruido de privacidad
//!   diferencial al agregado y verifica el tamaño de cohorte. Devuelve
//!   `None` (supresión) si la cohorte no alcanza `MIN_COHORT_SIZE`.
//! - [`compute_aggregate_audit_hash`]: hash de auditoría encadenado por
//!   `event_sequence_id` (mismo patrón que
//!   `enriched_domain_events::compute_event_audit_hash` -- esta tabla es
//!   APPEND-ONLY, no `row_version`).
//!
//! ## Por qué el ruido se aplica UNA vez, sobre la suma, no por-input
//!
//! El mecanismo clásico de privacidad diferencial (Gaussian mechanism)
//! perturba la RESPUESTA de la consulta agregada, no cada dato individual
//! -- perturbar cada input por separado desperdiciaría presupuesto de
//! privacidad N veces (una vez por contribuyente) sin ganar nada, porque
//! lo único que sale de este módulo es la suma. [`aggregate_index`] suma
//! primero los valores YA filtrados por consentimiento y aplica el ruido
//! una sola vez sobre ese total.
//!
//! ## Todos los montos son `i64` escalados ×10⁸ (ADR-0141)
//!
//! `noise_level_e8` y `metric_value_e8` son enteros ×10⁸ -- el cálculo
//! interno del ruido gaussiano usa `f64` TRANSITORIO (Box-Muller), pero el
//! resultado final se redondea a `i64` antes de devolverse. Ninguna
//! función de este módulo devuelve ni persiste un `f64`.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;
use sha2::{Digest, Sha256};

/// Codifica bytes crudos a su representación hexadecimal en minúsculas
/// (mismo patrón que `consent_registry::encode_hex` / `usage_metering`).
fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ── Tipo de índice vendible (columna `index_type`) ──────────────────────────

/// Catálogo de índices agregados vendibles (`docs/features/data-aggregation.md`
/// "¿Qué es esta feature?"), tal cual acepta el `CHECK (index_type IN
/// (...))` de la migración `0015_data_aggregation.sql`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IndexType {
    Sentiment,
    Regime,
    BrokerFriction,
    Correlation,
}

impl IndexType {
    /// Representación canónica en texto (la que persiste la columna
    /// `index_type` y la que acepta el `CHECK` de la migración).
    pub fn as_str(&self) -> &'static str {
        match self {
            IndexType::Sentiment => "SENTIMENT",
            IndexType::Regime => "REGIME",
            IndexType::BrokerFriction => "BROKER_FRICTION",
            IndexType::Correlation => "CORRELATION",
        }
    }

    /// Reconstruye el tipo desde su representación en texto, o `None` si
    /// no es ninguno de los cuatro reconocidos.
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "SENTIMENT" => Some(IndexType::Sentiment),
            "REGIME" => Some(IndexType::Regime),
            "BROKER_FRICTION" => Some(IndexType::BrokerFriction),
            "CORRELATION" => Some(IndexType::Correlation),
            _ => None,
        }
    }
}

// ── Canal de destino (columna `channel`, ADR-0143) ──────────────────────────

/// Separación estricta entre el canal interno (crudo, uso lícito del tier
/// gratuito, ADR-0143) y el canal externo (agregado, requiere
/// consentimiento vigente y `EXTERNAL_SALE_ENABLED=true`). Este enum
/// describe el destino de UN agregado ya calculado -- no confundir con el
/// firehose crudo del tier gratuito, que es un flujo separado y diferido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Channel {
    Internal,
    External,
}

impl Channel {
    /// Representación canónica en texto (la que persiste la columna
    /// `channel` y la que acepta el `CHECK` de la migración).
    pub fn as_str(&self) -> &'static str {
        match self {
            Channel::Internal => "INTERNAL",
            Channel::External => "EXTERNAL",
        }
    }

    /// Reconstruye el canal desde su representación en texto, o `None` si
    /// no es ninguno de los dos reconocidos.
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "INTERNAL" => Some(Channel::Internal),
            "EXTERNAL" => Some(Channel::External),
            _ => None,
        }
    }
}

// ── Hash unidireccional de topología (ADR-0102) ─────────────────────────────

/// Comprime una topología o firma de estrategia cruda (ej. la fórmula
/// `RSI(14)+MACD(12,26,9)`) a su hash SHA-256 (hex, minúsculas) --
/// (`docs/adr/ADR-0102.md`: "la topología de la estrategia se comprime a
/// una firma hash unidireccional... antes de ser transmitida").
///
/// Esta es la ÚNICA operación que este módulo hace con una topología en
/// texto plano: convertirla de inmediato en un hash irreversible. Ningún
/// llamador de esta función debe conservar ni propagar el argumento
/// `topology` más allá de esta llamada -- el orquestador la usa y descarta
/// el texto crudo de inmediato (ver `orchestrator::data_aggregation`).
pub fn hash_strategy_topology(topology: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(topology.as_bytes());
    encode_hex(&hasher.finalize())
}

// ── k-anonimato (invariante FIJO, `MIN_COHORT_SIZE`) ────────────────────────

/// Decide si una cohorte de `cohort_size` contribuyentes alcanza el tamaño
/// mínimo `min_cohort` para publicarse (`docs/features/data-aggregation.md`
/// "Parámetros Configurables": `MIN_COHORT_SIZE` es FIJO, no configurable
/// por el operador).
///
/// ## Borde exacto (criterio de cierre de la Orden §5)
///
/// Con `min_cohort = 5`: una cohorte de exactamente 5 contribuyentes
/// publica (`5 >= 5`); una de 4 se suprime (`4 >= 5` es falso). La
/// comparación es `>=`, no `>`: eso es lo que hace que "en el mínimo"
/// publique y "uno menos" suprima.
pub fn meets_k_anonymity(cohort_size: i64, min_cohort: i64) -> bool {
    cohort_size >= min_cohort
}

// ── Ruido de privacidad diferencial (RNG sembrado e inyectado) ──────────────

/// Aplica ruido gaussiano de privacidad diferencial a `raw_value_e8`,
/// escalado por `noise_level_e8` (la desviación estándar del ruido,
/// entero ×10⁸), generado con un RNG **sembrado** (`seed: u64`) --
/// (`docs/features/data-aggregation.md` "Parámetros Configurables":
/// `DP_NOISE_LEVEL`; ADR-0002/0004: "sin aleatoriedad sin semilla").
///
/// ## Por qué el RNG se sembrea explícitamente (y nunca `thread_rng()`)
///
/// Un Core FCIS es puro: mismo input -> mismo output, bit a bit. Si esta
/// función leyera entropía del sistema (`rand::thread_rng()`), la MISMA
/// llamada con los MISMOS argumentos produciría un resultado distinto en
/// cada ejecución -- imposible de probar (ningún `assert_eq!` sobrevive) y
/// imposible de auditar (nadie puede reproducir qué ruido se aplicó a un
/// dato histórico). Sembrando el RNG con `seed_from_u64(seed)`, la MISMA
/// semilla produce SIEMPRE la MISMA secuencia de números pseudoaleatorios
/// -- exactamente el mismo patrón que [`crate::domain::clock::DeterministicClock`]
/// aplica al tiempo: el Core recibe el "azar" ya inyectado, no lo genera
/// por su cuenta.
///
/// ## Box-Muller: de dos uniformes a una normal estándar
///
/// `StdRng` (sembrado) solo sabe producir números uniformes en `[0, 1)`.
/// La transformada de Box-Muller convierte DOS de esos uniformes
/// independientes (`u1`, `u2`) en un número con distribución normal
/// estándar (media 0, desviación 1): `sqrt(-2*ln(u1)) * cos(2*pi*u2)`.
/// `u1` se acota lejos de `0` (`f64::MIN_POSITIVE` como piso) porque
/// `ln(0)` es `-infinito` y arruinaría el resultado -- con un RNG uniforme
/// continuo la probabilidad de tocar exactamente `0.0` es nula en la
/// práctica, pero el piso lo hace matemáticamente total (nunca produce
/// `NaN`/`infinito`) sin alterar el resultado real de ningún caso posible.
///
/// El resultado con ruido se redondea a `i64` antes de devolverse --
/// ninguna columna persistida es `REAL` (ADR-0141).
pub fn apply_differential_privacy(raw_value_e8: i64, noise_level_e8: i64, seed: u64) -> i64 {
    let mut rng = StdRng::seed_from_u64(seed);

    // Dos uniformes independientes en [0, 1) -- u1 acotado lejos de 0 para
    // que ln(u1) nunca sea -infinito.
    let u1: f64 = rng.gen::<f64>().max(f64::MIN_POSITIVE);
    let u2: f64 = rng.gen::<f64>();

    // Box-Muller: transforma (u1, u2) uniformes en UNA muestra de una
    // normal estándar (media 0, desviación 1).
    let standard_normal = (-2.0_f64 * u1.ln()).sqrt() * (2.0_f64 * std::f64::consts::PI * u2).cos();

    // Escala la muestra estándar por el nivel de ruido configurado y la
    // suma al valor crudo -- el resultado final se redondea a entero ×10⁸.
    let noise = standard_normal * noise_level_e8 as f64;
    let noisy_value = raw_value_e8 as f64 + noise;

    noisy_value.round() as i64
}

// ── El índice agregado (puerto `aggregate_out`, ADR-0137) ───────────────────

/// El tipo de puerto `aggregate_out` (ADR-0137 catálogo, ADR-0144): un
/// índice agregado anonimizado, listo para consumo interno o venta
/// externa (`docs/features/data-aggregation.md` "Salida").
///
/// **Guardarraíl ADR-0093/0102 (estructural):** este struct SOLO expone
/// la métrica YA anonimizada, el tamaño de la cohorte y el canal -- ningún
/// campo puede portar un balance crudo, una topología cruda, ni un
/// identificador de usuario. El test
/// [`tests::aggregated_index_json_never_leaks_raw_identifiable_data`] fija
/// esto sobre un caso concreto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AggregatedIndex {
    pub index_type: IndexType,
    pub time_window: String,
    pub cohort_size: i64,
    /// Nivel de ruido de privacidad diferencial aplicado, entero ×10⁸.
    pub noise_level_e8: i64,
    /// Valor de la métrica agregada, YA con ruido aplicado, entero ×10⁸.
    pub metric_value_e8: i64,
    pub channel: Channel,
}

/// EL punto de modelado crítico de esta Story: agrega los valores YA
/// filtrados por consentimiento (`covered_values_e8` -- el orquestador
/// excluyó los no-cubiertos/opt-out ANTES de llamar aquí, consultando el
/// `consent_out` REAL de `consent-registry`), aplica el ruido de
/// privacidad diferencial sobre la suma, y verifica el tamaño de cohorte.
///
/// Devuelve `None` (supresión, `docs/features/data-aggregation.md`
/// "Restricciones") si `covered_values_e8.len()` no alcanza `min_cohort`
/// -- un agregado con cohorte insuficiente NUNCA se construye, ni siquiera
/// en memoria de paso, para que sea imposible persistirlo por accidente.
#[allow(clippy::too_many_arguments)]
pub fn aggregate_index(
    covered_values_e8: &[i64],
    index_type: IndexType,
    time_window: &str,
    channel: Channel,
    min_cohort: i64,
    noise_level_e8: i64,
    seed: u64,
) -> Option<AggregatedIndex> {
    let cohort_size = covered_values_e8.len() as i64;

    // Guardarraíl de k-anonimato -- PRIMERO, antes de calcular nada más.
    // Una cohorte insuficiente ni siquiera llega a sumarse.
    if !meets_k_anonymity(cohort_size, min_cohort) {
        return None;
    }

    // Suma los valores cubiertos -- el ruido se aplica UNA vez sobre este
    // total, no por cada contribuyente (ver doc-comment del módulo).
    let raw_sum_e8: i64 = covered_values_e8.iter().sum();
    let metric_value_e8 = apply_differential_privacy(raw_sum_e8, noise_level_e8, seed);

    Some(AggregatedIndex {
        index_type,
        time_window: time_window.to_string(),
        cohort_size,
        noise_level_e8,
        metric_value_e8,
        channel,
    })
}

// ── Hash de auditoría encadenado (event_sequence_id, APPEND-ONLY) ──────────

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de una fila de
/// `aggregated_indexes`, encadenado al `audit_hash` de la fila anterior en
/// la secuencia GLOBAL (o [`crate::domain::audit_log::GENESIS_PREVIOUS_HASH`]
/// si es la fila génesis, `event_sequence_id == 1`). Mismo patrón que
/// `enriched_domain_events::compute_event_audit_hash` -- la cadena es
/// GLOBAL sobre toda la tabla porque `aggregated_indexes` es APPEND-ONLY
/// (ADR-0141: `event_sequence_id UNIQUE`).
#[allow(clippy::too_many_arguments)]
pub fn compute_aggregate_audit_hash(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    previous_audit_hash: &str,
    owner_id: &str,
    institutional_tag: &str,
    node_id: &str,
    data_snapshot_id: Option<&str>,
    index_type: IndexType,
    time_window: &str,
    cohort_size: i64,
    noise_level_e8: i64,
    metric_value_e8: i64,
    channel: Channel,
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
    push(data_snapshot_id.unwrap_or(""));
    push(index_type.as_str());
    push(time_window);
    push(&cohort_size.to_string());
    push(&noise_level_e8.to_string());
    push(&metric_value_e8.to_string());
    push(channel.as_str());

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    let digest = hasher.finalize();

    encode_hex(&digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── CRITERIO #2 (Orden §5): ruido DP determinista + difiere del crudo ───

    /// CRITERIO DE CIERRE: la MISMA semilla + el MISMO valor crudo producen
    /// SIEMPRE el mismo resultado con ruido -- reproducibilidad. Debe
    /// fallar si el Core usara un RNG sin semilla (`thread_rng()`), donde
    /// dos llamadas con los mismos argumentos producirían valores
    /// distintos.
    #[test]
    fn apply_differential_privacy_is_deterministic_for_the_same_seed() {
        let result_a = apply_differential_privacy(100_000_000_000, 5_000_000_000, 42);
        let result_b = apply_differential_privacy(100_000_000_000, 5_000_000_000, 42);
        assert_eq!(result_a, result_b, "la misma semilla debe producir siempre el mismo resultado con ruido");
    }

    /// CRITERIO DE CIERRE: el resultado con ruido DIFIERE del valor crudo
    /// -- privacidad real, no un pass-through disfrazado. Debe fallar si
    /// `apply_differential_privacy` devolviera el valor crudo sin alterar.
    #[test]
    fn apply_differential_privacy_differs_from_the_raw_value() {
        let raw = 100_000_000_000;
        let noisy = apply_differential_privacy(raw, 5_000_000_000, 42);
        assert_ne!(noisy, raw, "el valor con ruido debe diferir del valor crudo (privacidad real)");
    }

    /// Semillas DISTINTAS producen resultados distintos para el mismo
    /// valor crudo -- confirma que la semilla realmente participa del
    /// cálculo (no es un parámetro decorativo ignorado).
    #[test]
    fn apply_differential_privacy_differs_across_different_seeds() {
        let result_seed_1 = apply_differential_privacy(100_000_000_000, 5_000_000_000, 1);
        let result_seed_2 = apply_differential_privacy(100_000_000_000, 5_000_000_000, 2);
        assert_ne!(result_seed_1, result_seed_2, "semillas distintas deben producir resultados distintos");
    }

    // ── CRITERIO #3 (Orden §5): k-anonimato de borde exacto ─────────────────

    /// CRITERIO DE CIERRE: cohorte EXACTAMENTE en el mínimo publica --
    /// debe fallar si suprimiera en el borde.
    #[test]
    fn meets_k_anonymity_publishes_at_the_exact_minimum() {
        assert!(meets_k_anonymity(5, 5), "una cohorte de exactamente MIN_COHORT_SIZE debe publicar");
    }

    /// CRITERIO DE CIERRE: cohorte UNO menos que el mínimo se suprime --
    /// debe fallar si publicara bajo el mínimo.
    #[test]
    fn meets_k_anonymity_suppresses_one_below_the_minimum() {
        assert!(!meets_k_anonymity(4, 5), "una cohorte de MIN_COHORT_SIZE - 1 debe suprimirse");
    }

    #[test]
    fn aggregate_index_publishes_at_the_exact_cohort_boundary() {
        let covered = vec![100_000_000_000_i64; 5];
        let result = aggregate_index(&covered, IndexType::Sentiment, "2026-W27", Channel::Internal, 5, 1_000_000, 42);
        assert!(result.is_some(), "cohorte de exactamente 5 (min_cohort=5) debe publicar");
        assert_eq!(result.unwrap().cohort_size, 5);
    }

    /// CRITERIO DE CIERRE: un agregado con cohorte insuficiente devuelve
    /// `None` -- NUNCA una fila con un tamaño de cohorte por debajo del
    /// mínimo. Debe fallar si devolviera `Some` bajo el mínimo.
    #[test]
    fn aggregate_index_suppresses_one_below_the_cohort_boundary() {
        let covered = vec![100_000_000_000_i64; 4];
        let result = aggregate_index(&covered, IndexType::Sentiment, "2026-W27", Channel::Internal, 5, 1_000_000, 42);
        assert!(result.is_none(), "cohorte de 4 (min_cohort=5) debe suprimirse -- None, no una fila");
    }

    // ── Suma + ruido sobre el agregado ───────────────────────────────────────

    #[test]
    fn aggregate_index_sums_covered_values_before_applying_noise() {
        let covered = vec![100_000_000_000_i64, 200_000_000_000, 300_000_000_000, 50_000_000_000, 10_000_000_000];
        let raw_sum: i64 = covered.iter().sum();
        let result = aggregate_index(&covered, IndexType::Correlation, "2026-W27", Channel::Internal, 5, 1_000_000, 7)
            .expect("cohorte suficiente debe publicar");

        // El valor final tiene ruido, así que no es EXACTAMENTE la suma
        // cruda, pero debe ser reproducible a partir de la misma suma y
        // semilla vía apply_differential_privacy.
        let expected = apply_differential_privacy(raw_sum, 1_000_000, 7);
        assert_eq!(result.metric_value_e8, expected);
    }

    // ── CRITERIO #4 (Orden §5, ADR-0102): hash unidireccional de topología ──

    #[test]
    fn hash_strategy_topology_is_deterministic_and_never_equals_the_raw_text() {
        let hash_a = hash_strategy_topology("RSI(14)+MACD(12,26,9)");
        let hash_b = hash_strategy_topology("RSI(14)+MACD(12,26,9)");
        assert_eq!(hash_a, hash_b, "la misma topología debe producir siempre el mismo hash");
        assert_ne!(hash_a, "RSI(14)+MACD(12,26,9)", "el hash nunca debe ser igual al texto crudo");
    }

    #[test]
    fn hash_strategy_topology_differs_for_different_topologies() {
        assert_ne!(
            hash_strategy_topology("RSI(14)+MACD(12,26,9)"),
            hash_strategy_topology("EMA(20)+BB(20,2)")
        );
    }

    // ── CRITERIO #5 (Orden §5): datos crudos nunca en la salida ─────────────

    /// CRITERIO DE CIERRE (guardarraíl ADR-0093/0102): el JSON serializado
    /// de `AggregatedIndex` no contiene el balance crudo (pre-ruido), ni
    /// texto de topología cruda, ni identificadores de usuario -- solo las
    /// seis claves esperadas.
    #[test]
    fn aggregated_index_json_never_leaks_raw_identifiable_data() {
        let covered = vec![100_000_000_000_i64; 5]; // suma cruda = 500_000_000_000
        let index = aggregate_index(&covered, IndexType::BrokerFriction, "2026-W27", Channel::External, 5, 1_000_000, 99)
            .expect("cohorte suficiente debe publicar");

        let value = serde_json::to_value(&index).expect("AggregatedIndex debe serializar");
        let object = value.as_object().expect("debe ser un objeto JSON");
        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec!["channel", "cohort_size", "index_type", "metric_value_e8", "noise_level_e8", "time_window"],
            "AggregatedIndex solo debe exponer estas seis claves -- nada de balances/topología/IDs crudos"
        );

        let json = value.to_string();
        for forbidden in ["account_id", "instrument_id", "password", "api_key", "private_key", "192.168.", "10.0.0."] {
            assert!(!json.contains(forbidden), "el JSON de AggregatedIndex no debe contener '{forbidden}'");
        }
        // La suma cruda pre-ruido (500_000_000_000) no debe aparecer intacta
        // en el JSON -- el ruido debe haberla alterado.
        assert!(
            !json.contains("500000000000"),
            "el valor agregado crudo (pre-ruido) no debe aparecer sin alterar en la salida"
        );
    }

    // ── IndexType / Channel: round-trip por string ───────────────────────────

    #[test]
    fn index_type_round_trips_through_its_string_representation() {
        for index_type in [IndexType::Sentiment, IndexType::Regime, IndexType::BrokerFriction, IndexType::Correlation] {
            assert_eq!(IndexType::from_str_value(index_type.as_str()), Some(index_type));
        }
        assert_eq!(IndexType::from_str_value("UNKNOWN"), None);
    }

    #[test]
    fn channel_round_trips_through_its_string_representation() {
        for channel in [Channel::Internal, Channel::External] {
            assert_eq!(Channel::from_str_value(channel.as_str()), Some(channel));
        }
        assert_eq!(Channel::from_str_value("UNKNOWN"), None);
    }

    // ── Hash de auditoría encadenado ─────────────────────────────────────────

    #[test]
    fn compute_aggregate_audit_hash_is_deterministic() {
        let hash_a = compute_aggregate_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1", Some("snapshot-1"),
            IndexType::Sentiment, "2026-W27", 5, 1_000_000, 500_000_010, Channel::Internal,
        );
        let hash_b = compute_aggregate_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1", Some("snapshot-1"),
            IndexType::Sentiment, "2026-W27", 5, 1_000_000, 500_000_010, Channel::Internal,
        );
        assert_eq!(hash_a, hash_b);
    }

    /// CRITERIO DE CIERRE: cambiar `metric_value_e8` cambia el hash -- si
    /// el campo no entrara en el hash, esta prueba fallaría con hashes
    /// iguales.
    #[test]
    fn compute_aggregate_audit_hash_changes_when_metric_value_changes() {
        let original = compute_aggregate_audit_hash(
            "id-1", 2_000, 2, "prev-hash", "owner-1", "DRASUS_LOCAL", "node-1", None,
            IndexType::Regime, "2026-W27", 5, 1_000_000, 100, Channel::Internal,
        );
        let changed = compute_aggregate_audit_hash(
            "id-1", 2_000, 2, "prev-hash", "owner-1", "DRASUS_LOCAL", "node-1", None,
            IndexType::Regime, "2026-W27", 5, 1_000_000, 200, Channel::Internal,
        );
        assert_ne!(original, changed, "cambiar metric_value_e8 debe cambiar el hash de auditoría");
    }

    /// CRITERIO DE CIERRE: cambiar `channel` cambia el hash.
    #[test]
    fn compute_aggregate_audit_hash_changes_when_channel_changes() {
        let internal = compute_aggregate_audit_hash(
            "id-1", 2_000, 2, "prev-hash", "owner-1", "DRASUS_LOCAL", "node-1", None,
            IndexType::Regime, "2026-W27", 5, 1_000_000, 100, Channel::Internal,
        );
        let external = compute_aggregate_audit_hash(
            "id-1", 2_000, 2, "prev-hash", "owner-1", "DRASUS_LOCAL", "node-1", None,
            IndexType::Regime, "2026-W27", 5, 1_000_000, 100, Channel::External,
        );
        assert_ne!(internal, external, "cambiar el canal debe cambiar el hash de auditoría");
    }
}
