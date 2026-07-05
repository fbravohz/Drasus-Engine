//! [CORE] Lógica pura de Usage Metering / Libro de Nocional
//! (`docs/features/usage-metering.md`, ADR-0144, ADR-0143, ADR-0141,
//! ADR-0020 V2, STORY-030).
//!
//! Sin I/O, sin reloj de sistema, sin aleatoriedad sin semilla
//! (ADR-0002/0004). Piezas de lógica pura que pide la Feature en su
//! "Estructura Interna (FCIS)" y la Orden STORY-030 §4.2:
//! - [`compute_notional`]: nocional de una operación (tamaño × precio),
//!   entero escalado ×10⁸ -- EL punto de correctitud crítico de esta
//!   Story (reescalado ×10¹⁶→×10⁸, `i128`, redondeo explícito).
//! - [`accumulate`]: suma el nocional de una operación al acumulado del
//!   ciclo, sin overflow silencioso.
//! - [`detect_quota_crossing`]: veredicto de cuota (dentro / cruzada)
//!   comparando el acumulado contra el `notional_limit` del plan.
//! - [`derive_billing_cycle_id`]: identificador de ciclo mensual ("YYYY-MM")
//!   derivado del reloj inyectado -- sin dependencia de calendario externa
//!   (algoritmo de Howard Hinnant, dominio público).
//! - [`compute_usage_audit_hash`]: hash de auditoría encadenado por
//!   `event_sequence_id` (mismo patrón que `audit_log::compute_audit_hash`
//!   -- esta tabla es APPEND-ONLY, no `row_version`).
//!
//! ## Sobre `Order` (placeholder) y [`MeteredOperation`]
//!
//! El puerto `order_in` del catálogo (ADR-0137) apunta al tipo `Order`,
//! pero `Order` hoy es un `pub struct Order;` vacío en
//! `crate::types::Order` -- el tipo real es del módulo `execute` (EPIC-5),
//! todavía no construido (STORY-030 §3 "Tipo `Order` de entrada
//! (placeholder)"). [`MeteredOperation`] modela la ENTRADA MÍNIMA que este
//! cimiento necesita para derivar el nocional (tamaño, precio,
//! instrumento) -- no un `Order` completo. El mapeo `Order` real →
//! `MeteredOperation` es un follow-up futuro, cuando `execute` exista.

use serde::Serialize;
use sha2::{Digest, Sha256};

/// Factor de escala fijo del sistema para precios y volumen: ×10⁸ (8
/// decimales), ADR-0141. Toda cantidad monetaria en el Core es un
/// `i64` en esta escala -- NUNCA `f64`/`REAL`.
pub const AMOUNT_SCALE: i64 = 100_000_000;

/// Codifica bytes crudos a su representación hexadecimal en minúsculas
/// (mismo patrón que `licensing_system::encode_hex` / `plan_tier_quota`).
fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ── Cálculo de nocional (EL punto de correctitud crítico) ───────────────────

/// Por qué [`compute_notional`] o [`accumulate`] no puede completar su
/// cálculo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotionalError {
    /// El tamaño de la operación es negativo -- no existe un tamaño
    /// operado negativo (un short se representa con otro campo de
    /// dirección, no con un tamaño negativo).
    #[error("el tamaño de la operación no puede ser negativo")]
    NegativeSize,
    /// El precio de ejecución es negativo -- no existe un precio negativo
    /// en ningún instrumento soportado.
    #[error("el precio de la operación no puede ser negativo")]
    NegativePrice,
    /// El resultado (nocional reescalado, o el acumulado tras sumar) no
    /// cabe en `i64` -- se rechaza explícitamente en vez de envolver
    /// (`wrapping`) o truncar en silencio, lo cual facturaría mal.
    #[error("el resultado de la operación aritmética desborda i64")]
    Overflow,
}

/// Calcula el nocional en USD de una operación: `tamaño × precio`, con
/// ambos operandos en la escala fija ×10⁸ (ADR-0141) y el resultado
/// también en ×10⁸.
///
/// ## Por qué hay que reescalar (el corazón de esta Story)
///
/// `size` y `price` están cada uno en ×10⁸. Multiplicarlos directamente
/// produce un resultado en ×10¹⁶ (el producto de dos escalas ×10⁸ suma los
/// exponentes: 10⁸ × 10⁸ = 10¹⁶), no en ×10⁸. Ejemplo con los valores de
/// la Orden: tamaño 2.5 (`250_000_000` en ×10⁸) × precio $40,000.00
/// (`4_000_000_000_000` en ×10⁸) da un producto crudo de
/// `1_000_000_000_000_000_000_000` (×10¹⁶) -- pero el nocional real es
/// $100,000.00, que en ×10⁸ es `10_000_000_000_000`. Hay que DIVIDIR el
/// producto crudo entre `AMOUNT_SCALE` (10⁸) para volver a la escala ×10⁸
/// correcta.
///
/// ## Por qué `i128` y no `i64` para el producto intermedio
///
/// El producto crudo (×10¹⁶) de dos `i64` en ×10⁸ puede exceder
/// `i64::MAX` (~9.22×10¹⁸) mucho antes de que el resultado FINAL
/// (reescalado a ×10⁸) lo haga -- multiplicar dos `i64` directamente
/// haría *overflow* (panic en debug, envoltura silenciosa en release) del
/// producto crudo aunque el nocional final fuera perfectamente válido.
/// `i128` (rango ~±1.7×10³⁸) tiene margen de sobra para el producto crudo
/// de cualquier par de montos ×10⁸ representables en `i64`.
///
/// ## Por qué NUNCA `f64`
///
/// Un `f64` solo representa enteros exactos hasta 2⁵³ (~9×10¹⁵) -- por
/// encima de eso empieza a perder precisión de los últimos dígitos. Un
/// nocional de $100,000.00 (`10_000_000_000_000` en ×10⁸) todavía cabe,
/// pero una posición institucional grande no: el error de redondeo de
/// punto flotante en un cálculo de facturación es dinero real perdido o
/// cobrado de más. Por eso todo este módulo opera EXCLUSIVAMENTE en
/// enteros (`i64`/`i128`).
///
/// ## Política de redondeo (explícita y determinista)
///
/// El reescalado ×10¹⁶→×10⁸ divide entre `AMOUNT_SCALE`. Si el producto
/// crudo no es múltiplo exacto de `AMOUNT_SCALE`, se redondea al entero
/// más cercano, con el punto medio exacto (`.5`) redondeando HACIA ARRIBA
/// ("half up") -- se suma `AMOUNT_SCALE / 2` antes de dividir con división
/// entera truncada. Es determinista: el mismo input siempre produce el
/// mismo output (ADR-0002/0004).
pub fn compute_notional(size: i64, price: i64) -> Result<i64, NotionalError> {
    if size < 0 {
        return Err(NotionalError::NegativeSize);
    }
    if price < 0 {
        return Err(NotionalError::NegativePrice);
    }

    // Producto crudo en i128 -- ver doc-comment: dos operandos ×10⁸
    // producen un producto en ×10¹⁶, que puede desbordar i64 aunque el
    // resultado final (reescalado) no lo haga.
    let raw_product: i128 = (size as i128) * (price as i128);

    let scale: i128 = AMOUNT_SCALE as i128;
    // Redondeo "half up": sumar la mitad de la escala antes de la
    // división entera trunca hacia el entero más cercano, con el empate
    // exacto (.5) subiendo. División entera normal (sin el `+ half`)
    // truncaría siempre hacia abajo ("floor"), perdiendo el último
    // dígito significativo de forma sesgada.
    let half = scale / 2;
    let rescaled: i128 = (raw_product + half) / scale;

    // El resultado reescalado debe caber en i64 -- si no cabe, se
    // rechaza explícitamente (Err) en vez de truncar/envolver en
    // silencio, que facturaría mal.
    i64::try_from(rescaled).map_err(|_| NotionalError::Overflow)
}

/// Suma el nocional de una operación al acumulado previo del ciclo de
/// facturación vigente. Pura: sin I/O, solo aritmética exacta con
/// verificación de desborde (`checked_add`) -- un acumulado que
/// desbordara `i64` se rechaza en vez de envolver en silencio.
pub fn accumulate(previous_cumulative: i64, notional: i64) -> Result<i64, NotionalError> {
    previous_cumulative
        .checked_add(notional)
        .ok_or(NotionalError::Overflow)
}

// ── Veredicto de cuota ───────────────────────────────────────────────────────

/// Veredicto de cuota tras acumular una operación (`docs/features/usage-metering.md`
/// "Comportamientos Observables": "Cuando el acumulado del ciclo cruza el
/// límite del plan -> se emite un evento de cuota alcanzada").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QuotaVerdict {
    /// El acumulado del ciclo sigue dentro (o igual a) el límite del plan.
    Within,
    /// El acumulado del ciclo superó (estrictamente) el límite del plan.
    Crossed,
}

impl QuotaVerdict {
    /// Representación canónica en texto (la que persiste la columna
    /// `quota_verdict` y la que acepta el `CHECK` de la migración).
    pub fn as_str(&self) -> &'static str {
        match self {
            QuotaVerdict::Within => "WITHIN",
            QuotaVerdict::Crossed => "CROSSED",
        }
    }

    /// Reconstruye el veredicto desde el valor persistido, o `None` si no
    /// es ninguno de los dos reconocidos (integridad de datos).
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "WITHIN" => Some(QuotaVerdict::Within),
            "CROSSED" => Some(QuotaVerdict::Crossed),
            _ => None,
        }
    }
}

/// Compara el acumulado del ciclo contra el `notional_limit` REAL del plan
/// (`plan_tier_quota::PlanLimits`, consumido por quien llama -- este
/// módulo no depende de `plan_tier_quota`, solo recibe el límite ya
/// resuelto como `i64`).
///
/// ## Semántica de `notional_limit == 0`
///
/// `plan-tier-quota` documenta que `0` es un valor VÁLIDO que codifica
/// "sin tope de nocional propio" (un plan puede limitar solo por
/// activaciones). Por eso `notional_limit == 0` siempre resuelve
/// [`QuotaVerdict::Within`] -- nunca hay cruce de un límite que no existe.
///
/// ## Decisión explícita: "cruzar" es superar ESTRICTAMENTE el límite
///
/// Tocar el límite exacto (`cumulative == notional_limit`) todavía cuenta
/// como [`QuotaVerdict::Within`] -- "cruzar" (superar hacia el otro lado)
/// requiere `cumulative > notional_limit`. Esta es una decisión de diseño
/// explícita y documentada, no un accidente de `>=` vs `>`.
pub fn detect_quota_crossing(cumulative: i64, notional_limit: i64) -> QuotaVerdict {
    if notional_limit == 0 {
        return QuotaVerdict::Within;
    }
    if cumulative > notional_limit {
        QuotaVerdict::Crossed
    } else {
        QuotaVerdict::Within
    }
}

// ── Derivación del ciclo de facturación ──────────────────────────────────────

/// Deriva el identificador de ciclo de facturación MENSUAL ("YYYY-MM", UTC)
/// a partir de un timestamp en nanosegundos desde el Unix epoch (el
/// puerto [`crate::domain::clock::Clock`] inyectado -- NUNCA
/// `SystemTime::now()` directo, `docs/features/usage-metering.md`
/// "Restricciones" + patrón general del proyecto de reloj inyectado).
///
/// Implementación pura, sin dependencia de calendario externa: usa el
/// algoritmo `civil_from_days` de Howard Hinnant (dominio público,
/// <http://howardhinnant.github.io/date_algorithms.html>) para convertir
/// un conteo de días desde el epoch a (año, mes, día) gregoriano
/// proléptico con aritmética entera exacta -- determinista
/// (ADR-0002/0004), sin `f64`.
pub fn derive_billing_cycle_id(timestamp_ns: i64) -> String {
    const NANOS_PER_DAY: i64 = 86_400_000_000_000;
    // div_euclid: división que redondea hacia -infinito, correcta también
    // para timestamps anteriores a 1970 (no se esperan en producción,
    // pero la función se mantiene total en su dominio de todos modos).
    let days_since_epoch = timestamp_ns.div_euclid(NANOS_PER_DAY);
    let (year, month, _day) = civil_from_days(days_since_epoch);
    format!("{year:04}-{month:02}")
}

/// Algoritmo `civil_from_days` de Howard Hinnant -- convierte un conteo de
/// días desde 1970-01-01 (día 0) a (año, mes, día) gregoriano proléptico.
/// Solo aritmética entera; sin I/O, sin dependencias externas. Devuelve
/// `(year, month, day)` con `month` en `[1, 12]` y `day` en `[1, 31]`.
fn civil_from_days(z_in: i64) -> (i64, u32, u32) {
    let z = z_in + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year, m, d)
}

// ── Entrada mínima de metering (placeholder de `Order`) ─────────────────────

/// Entrada mínima de metering derivada de una orden ejecutada -- ver el
/// doc-comment del módulo ("Sobre `Order` (placeholder)") para la
/// justificación de por qué esto NO es el `Order` completo.
#[derive(Debug, Clone, Copy)]
pub struct MeteredOperation<'a> {
    /// Tamaño operado, `INTEGER` escalado ×10⁸.
    pub size: i64,
    /// Precio de ejecución, `INTEGER` escalado ×10⁸.
    pub price: i64,
    /// Instrumento operado (ej. `"BTCUSDT"`).
    pub instrument_id: &'a str,
}

// ── Puerto `usage_out` -> `UsageRecord` (ADR-0137) ──────────────────────────

/// El tipo de puerto `UsageRecord` (ADR-0137 catálogo, enmienda
/// 2026-07-03): "Acumulado de nocional por ciclo + veredicto de cuota".
///
/// **Guardarraíl ADR-0093 (estructural):** este struct SOLO tiene los tres
/// campos de abajo -- ninguna credencial de bróker, IP de servidor live,
/// ni secreto de ningún tipo. El test
/// [`tests::usage_record_json_never_leaks_secret_fields`] fija la lista
/// exacta de claves permitidas en el JSON serializado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UsageRecord {
    /// Identificador del ciclo de facturación vigente (ej. `"2026-07"`).
    pub billing_cycle_id: String,
    /// Nocional acumulado del ciclo, `INTEGER` escalado ×10⁸.
    pub cycle_accumulated: i64,
    /// Veredicto de cuota tras la última operación registrada.
    pub quota_verdict: QuotaVerdict,
}

// ── Hash de auditoría encadenado (event_sequence_id, APPEND-ONLY) ───────────

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de una fila de
/// `usage_records`, encadenado al `audit_hash` de la fila anterior en la
/// secuencia GLOBAL (o [`crate::domain::audit_log::GENESIS_PREVIOUS_HASH`]
/// si es la fila génesis, `event_sequence_id == 1`). A diferencia de
/// `plan_tier_quota::compute_plan_audit_hash` (que encadena POR FILA
/// mutable vía `row_version`), aquí la cadena es GLOBAL sobre toda la
/// tabla -- mismo patrón que `audit_log::compute_audit_hash`, porque
/// `usage_records` es APPEND-ONLY (ADR-0141: `event_sequence_id UNIQUE`).
#[allow(clippy::too_many_arguments)]
pub fn compute_usage_audit_hash(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    previous_audit_hash: &str,
    owner_id: &str,
    institutional_tag: &str,
    node_id: &str,
    billing_cycle_id: &str,
    instrument_id: &str,
    notional_per_op: i64,
    cycle_accumulated: i64,
    quota_verdict: QuotaVerdict,
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
    push(billing_cycle_id);
    push(instrument_id);
    push(&notional_per_op.to_string());
    push(&cycle_accumulated.to_string());
    push(quota_verdict.as_str());

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    let digest = hasher.finalize();

    encode_hex(&digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── CRITERIO #2 (Orden §5, EL punto de correctitud crítico) ─────────────

    /// CRITERIO DE CIERRE: reescalado ×10¹⁶→×10⁸ con los valores conocidos
    /// de la Orden -- tamaño 2.5 ($`250_000_000`) × precio $40,000.00
    /// (`4_000_000_000_000`) = nocional $100,000.00 (`10_000_000_000_000`).
    /// Falla si la función multiplica sin reescalar (daría `10^21`) o si
    /// usara `f64` (perdería precisión en un número de esta magnitud).
    #[test]
    fn compute_notional_rescales_known_values_exactly() {
        let size = 250_000_000; // 2.5 * 1e8
        let price = 4_000_000_000_000; // $40,000.00 * 1e8
        let notional = compute_notional(size, price).expect("debe calcular el nocional");
        assert_eq!(notional, 10_000_000_000_000, "$100,000.00 * 1e8");
    }

    /// CRITERIO DE CIERRE: valores grandes NO producen overflow. El
    /// producto crudo (`5_000_000_000_000 * 5_000_000_000_000` =
    /// `2.5e25`) desbordaría `i64` (máx. ~9.22e18) si se multiplicara
    /// directamente en esa anchura -- solo `i128` lo sostiene. El
    /// resultado FINAL, ya reescalado a ×10⁸, sí cabe en `i64`.
    #[test]
    fn compute_notional_handles_large_values_without_overflow() {
        let size = 5_000_000_000_000; // $50,000.00 * 1e8
        let price = 5_000_000_000_000; // $50,000.00 * 1e8
        let notional = compute_notional(size, price).expect("no debe desbordar con i128 intermedio");
        assert_eq!(notional, 250_000_000_000_000_000);
    }

    /// CRITERIO DE CIERRE: si el resultado reescalado (no solo el
    /// intermedio) excede `i64::MAX`, se rechaza explícitamente en vez de
    /// envolver en silencio -- un desborde silencioso facturaría un monto
    /// arbitrario y equivocado.
    #[test]
    fn compute_notional_rejects_when_final_result_overflows_i64() {
        let size = 9_000_000_000_000_000_000; // ~9e18, deliberadamente enorme
        let price = 9_000_000_000_000_000_000;
        assert_eq!(compute_notional(size, price), Err(NotionalError::Overflow));
    }

    #[test]
    fn compute_notional_rejects_negative_size_or_price() {
        assert_eq!(compute_notional(-1, 100), Err(NotionalError::NegativeSize));
        assert_eq!(compute_notional(100, -1), Err(NotionalError::NegativePrice));
    }

    /// CRITERIO DE CIERRE: el redondeo en el borde exacto (`.5`) sube --
    /// un producto crudo de `150_000_000` (1.5 en la escala del cociente)
    /// redondea a `2`, no a `1` (lo que daría una división entera
    /// truncada sin `+ half`).
    #[test]
    fn compute_notional_rounds_up_at_exact_half() {
        // size=150_000_000, price=1 -> producto crudo = 150_000_000
        // (1.5 * AMOUNT_SCALE exactamente) -> debe redondear a 2.
        let notional = compute_notional(150_000_000, 1).expect("debe calcular");
        assert_eq!(notional, 2, "el punto medio exacto (.5) debe redondear hacia arriba");
    }

    /// Justo debajo del punto medio, el redondeo se mantiene hacia abajo
    /// (no hay sesgo sistemático hacia arriba).
    #[test]
    fn compute_notional_stays_down_just_below_half() {
        // Producto crudo = 149_999_999 (justo debajo de 1.5 * AMOUNT_SCALE).
        let notional = compute_notional(149_999_999, 1).expect("debe calcular");
        assert_eq!(notional, 1, "por debajo del punto medio no debe redondear hacia arriba");
    }

    // ── CRITERIO #3 (Orden §5): acumulación por ciclo ───────────────────────

    #[test]
    fn accumulate_sums_exactly() {
        assert_eq!(accumulate(1_000, 500).expect("debe sumar"), 1_500);
        assert_eq!(accumulate(0, 250).expect("debe sumar"), 250);
    }

    /// CRITERIO DE CIERRE: un acumulado que desbordara `i64` se rechaza en
    /// vez de envolver en silencio (`i64::MAX + 1` picaría a negativo con
    /// una suma `wrapping`).
    #[test]
    fn accumulate_rejects_overflow() {
        assert_eq!(accumulate(i64::MAX, 1), Err(NotionalError::Overflow));
    }

    // ── CRITERIO #4 (Orden §5): cruce de umbral ──────────────────────────────

    #[test]
    fn detect_quota_crossing_is_within_below_the_limit() {
        assert_eq!(detect_quota_crossing(500, 1_000), QuotaVerdict::Within);
    }

    #[test]
    fn detect_quota_crossing_is_within_exactly_at_the_limit() {
        // Decisión explícita: tocar el límite exacto NO es "cruzarlo".
        assert_eq!(detect_quota_crossing(1_000, 1_000), QuotaVerdict::Within);
    }

    /// CRITERIO DE CIERRE: superar estrictamente el límite es "cruzada" --
    /// si el umbral se ignorara, esta prueba vería `Within` en su lugar.
    #[test]
    fn detect_quota_crossing_is_crossed_above_the_limit() {
        assert_eq!(detect_quota_crossing(1_001, 1_000), QuotaVerdict::Crossed);
    }

    /// `notional_limit == 0` codifica "sin tope de nocional propio"
    /// (semántica de `plan-tier-quota`) -- nunca cruza.
    #[test]
    fn detect_quota_crossing_treats_zero_limit_as_unlimited() {
        assert_eq!(detect_quota_crossing(1_000_000_000_000, 0), QuotaVerdict::Within);
    }

    // ── Derivación del ciclo de facturación ──────────────────────────────────

    /// Valor conocido: el mismo timestamp de ejemplo usado en
    /// `domain::clock` (2020-01-01, ver comentario de esa prueba) cae en
    /// el ciclo "2020-01".
    #[test]
    fn derive_billing_cycle_id_known_value() {
        const TIMESTAMP_2020_01_01: i64 = 1_577_869_800_000_000_000;
        assert_eq!(derive_billing_cycle_id(TIMESTAMP_2020_01_01), "2020-01");
    }

    /// CRITERIO DE CIERRE: cruzar la medianoche del último día del mes
    /// cambia el ciclo -- día 30 desde el epoch es 1970-01-31 (ciclo
    /// "1970-01"); día 31 es 1970-02-01 (ciclo "1970-02").
    #[test]
    fn derive_billing_cycle_id_changes_across_month_boundary() {
        const NANOS_PER_DAY: i64 = 86_400_000_000_000;
        assert_eq!(derive_billing_cycle_id(30 * NANOS_PER_DAY), "1970-01");
        assert_eq!(derive_billing_cycle_id(31 * NANOS_PER_DAY), "1970-02");
    }

    /// Cruzar el fin de año también cambia el ciclo -- día 364 es
    /// 1970-12-31 (año no bisiesto, 365 días: 0..=364); día 365 es
    /// 1971-01-01.
    #[test]
    fn derive_billing_cycle_id_changes_across_year_boundary() {
        const NANOS_PER_DAY: i64 = 86_400_000_000_000;
        assert_eq!(derive_billing_cycle_id(364 * NANOS_PER_DAY), "1970-12");
        assert_eq!(derive_billing_cycle_id(365 * NANOS_PER_DAY), "1971-01");
    }

    // ── CRITERIO #6 (Orden §5): guardarraíl ADR-0093 -- sin secretos ────────

    /// CRITERIO DE CIERRE (guardarraíl ADR-0093): el JSON serializado de
    /// `UsageRecord` contiene EXACTAMENTE estas tres claves.
    #[test]
    fn usage_record_json_never_leaks_secret_fields() {
        let record = UsageRecord {
            billing_cycle_id: "2026-07".to_string(),
            cycle_accumulated: 10_000_000_000_000,
            quota_verdict: QuotaVerdict::Within,
        };

        let json = serde_json::to_value(&record).expect("UsageRecord debe serializar a JSON");
        let object = json.as_object().expect("el JSON de UsageRecord debe ser un objeto");

        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();

        assert_eq!(
            keys,
            vec!["billing_cycle_id", "cycle_accumulated", "quota_verdict"],
            "UsageRecord solo puede exponer estas tres claves (ADR-0093)"
        );

        let json_string = json.to_string();
        for forbidden in ["password", "api_key", "api-key", "broker_secret", "private_key", "signing_key", "192.168.", "10.0.0."] {
            assert!(
                !json_string.to_lowercase().contains(forbidden),
                "el JSON de UsageRecord no debe contener '{forbidden}'"
            );
        }
    }

    #[test]
    fn quota_verdict_serializes_to_screaming_snake_case() {
        assert_eq!(serde_json::to_string(&QuotaVerdict::Within).unwrap(), "\"WITHIN\"");
        assert_eq!(serde_json::to_string(&QuotaVerdict::Crossed).unwrap(), "\"CROSSED\"");
    }

    #[test]
    fn quota_verdict_round_trips_through_its_string_representation() {
        for verdict in [QuotaVerdict::Within, QuotaVerdict::Crossed] {
            assert_eq!(QuotaVerdict::from_str_value(verdict.as_str()), Some(verdict));
        }
        assert_eq!(QuotaVerdict::from_str_value("UNKNOWN"), None);
    }

    // ── Hash de auditoría encadenado ─────────────────────────────────────────

    #[test]
    fn compute_usage_audit_hash_is_deterministic() {
        let hash_a = compute_usage_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1", "2026-07", "BTCUSDT",
            10_000_000_000_000, 10_000_000_000_000, QuotaVerdict::Within,
        );
        let hash_b = compute_usage_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1", "2026-07", "BTCUSDT",
            10_000_000_000_000, 10_000_000_000_000, QuotaVerdict::Within,
        );
        assert_eq!(hash_a, hash_b);
    }

    /// CRITERIO DE CIERRE (Orden §5, criterio #7): cambiar el
    /// `cycle_accumulated` cambia el hash -- si el campo no entrara en el
    /// hash, esta prueba fallaría con hashes iguales.
    #[test]
    fn compute_usage_audit_hash_changes_when_cycle_accumulated_changes() {
        let original = compute_usage_audit_hash(
            "id-1", 2_000, 2, "prev-hash", "owner-1", "DRASUS_LOCAL", "node-1", "2026-07", "BTCUSDT",
            5_000_000_000_000, 15_000_000_000_000, QuotaVerdict::Crossed,
        );
        let changed = compute_usage_audit_hash(
            "id-1", 2_000, 2, "prev-hash", "owner-1", "DRASUS_LOCAL", "node-1", "2026-07", "BTCUSDT",
            5_000_000_000_000, 25_000_000_000_000, QuotaVerdict::Crossed,
        );
        assert_ne!(original, changed, "cambiar el acumulado debe cambiar el hash de auditoría");
    }
}
