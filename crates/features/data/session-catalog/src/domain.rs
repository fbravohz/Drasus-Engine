//! Functional Core — lógica pura, sin I/O.
//!
//! PROHIBIDO: imports de `sqlx`, `tokio`, o cualquier crate que toque el
//! sistema.
//!
//! Implementa dos piezas puras (ADR-0162):
//! - TTR-002: [`compute_expected_fingerprint`] -- deriva la huella de
//!   calendario ESPERADA a partir de las ventanas semanales de una
//!   plantilla (continua / hueco semanal / hueco semanal + diario /
//!   sesión diaria). El Gate de Coherencia que compara esta huella contra
//!   la OBSERVADA del archivo importado vive en `pit-data-validator`, no
//!   aquí -- esta feature solo produce el lado "declarado".
//! - TTR-003: [`resolve_session_template_id`] -- la cadena de resolución
//!   pura (override de bróker -> default de instrumento). Recibe los IDs
//!   YA resueltos como parámetros porque `session-catalog` NUNCA importa
//!   `instrument-catalog` (ADR-0137: aislamiento de crates entre
//!   features) -- ver `crate::lib` para la premisa retada al Tech-Lead.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ── Vocabulario fijo (CHECK de la migración) ─────────────────────────────

/// Día de la semana (`session_template_window.start_day`/`end_day`).
/// `SCREAMING_SNAKE_CASE` hace que la forma JSON (`"MON"`) coincida EXACTO
/// con el vocabulario del `CHECK` de la migración.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl Weekday {
    pub fn as_str(self) -> &'static str {
        match self {
            Weekday::Mon => "MON",
            Weekday::Tue => "TUE",
            Weekday::Wed => "WED",
            Weekday::Thu => "THU",
            Weekday::Fri => "FRI",
            Weekday::Sat => "SAT",
            Weekday::Sun => "SUN",
        }
    }

    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "MON" => Some(Weekday::Mon),
            "TUE" => Some(Weekday::Tue),
            "WED" => Some(Weekday::Wed),
            "THU" => Some(Weekday::Thu),
            "FRI" => Some(Weekday::Fri),
            "SAT" => Some(Weekday::Sat),
            "SUN" => Some(Weekday::Sun),
            _ => None,
        }
    }

    /// Posición del día dentro de la semana lineal que usa
    /// [`week_offset_minutes`] -- `Mon = 0` ... `Sun = 6`.
    fn index(self) -> i32 {
        match self {
            Weekday::Mon => 0,
            Weekday::Tue => 1,
            Weekday::Wed => 2,
            Weekday::Thu => 3,
            Weekday::Fri => 4,
            Weekday::Sat => 5,
            Weekday::Sun => 6,
        }
    }
}

/// Una ventana semanal de apertura de mercado, en hora de
/// `market_timezone_ref` de la plantilla dueña. Espejo de una fila de
/// `session_template_window`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SessionWindow {
    pub start_day: Weekday,
    pub start_time_minutes: i32,
    pub end_day: Weekday,
    pub end_time_minutes: i32,
}

/// Huella de calendario esperada (TTR-002, tabla "Gate de Coherencia de
/// Calendario" de ADR-0162). Cuatro formas de mercado -- el alcance de día
/// 1 (`SESSION_TEMPLATE_SEED_SET`) solo garantiza `Continuous` (Cripto) y
/// `WeeklyGap` (Forex/CFD); las otras dos quedan modeladas para cuando
/// existan plantillas de Futuros/Acciones (fuera de alcance de esta
/// Story).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CalendarFingerprint {
    /// Sin huecos -- 24/7/365 (Cripto).
    Continuous,
    /// Exactamente un hueco semanal, sin huecos diarios (Forex/CFD).
    WeeklyGap,
    /// Hueco semanal + hueco diario de mantenimiento recurrente (Futuros).
    WeeklyGapPlusDailyMaintenance,
    /// Múltiples huecos que dejan solo una ventana corta por día hábil
    /// (Acciones: sesión diaria + noches + fin de semana + feriados).
    DailySession,
}

/// Errores que puede producir [`compute_expected_fingerprint`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SessionCatalogDomainError {
    /// Una plantilla sin ninguna ventana no tiene huella que derivar --
    /// nunca se aproxima a `Continuous` ni a ninguna otra forma por
    /// omisión.
    #[error("no se puede derivar la huella de calendario sin ventanas semanales")]
    NoWindows,
}

const MINUTES_PER_DAY: i32 = 1_440;
const DAYS_PER_WEEK: i32 = 7;
const MINUTES_PER_WEEK: i32 = MINUTES_PER_DAY * DAYS_PER_WEEK; // 10 080

/// Posición absoluta (0..MINUTES_PER_WEEK-1) de un instante dentro de la
/// semana lineal Lunes 00:00 -> Domingo 23:59.
fn week_offset_minutes(day: Weekday, minutes_in_day: i32) -> i32 {
    day.index() * MINUTES_PER_DAY + minutes_in_day
}

/// Deriva la huella de calendario ESPERADA a partir de las ventanas
/// semanales de una plantilla (TTR-002). Función pura: marca en un array
/// booleano de tamaño `MINUTES_PER_WEEK` qué minutos de la semana están
/// cubiertos por al menos una ventana, y clasifica la forma resultante por
/// el número y tamaño de los huecos (minutos NO cubiertos).
///
/// Una ventana cuyo `end` (en la semana lineal) cae ANTES que su `start`
/// representa una ventana que cruza la frontera artificial domingo 23:59 ->
/// lunes 00:00 (ej. Forex: abre domingo 22:00, cierra el viernes
/// SIGUIENTE a las 22:00) -- se cubre en dos tramos: `[start, fin de
/// semana]` + `[inicio de semana, end]`.
pub fn compute_expected_fingerprint(windows: &[SessionWindow]) -> Result<CalendarFingerprint, SessionCatalogDomainError> {
    if windows.is_empty() {
        return Err(SessionCatalogDomainError::NoWindows);
    }

    let mut covered = vec![false; MINUTES_PER_WEEK as usize];
    for window in windows {
        let start = week_offset_minutes(window.start_day, window.start_time_minutes);
        let end = week_offset_minutes(window.end_day, window.end_time_minutes);

        if end >= start {
            for minute in covered.iter_mut().take((end + 1) as usize).skip(start as usize) {
                *minute = true;
            }
        } else {
            // La ventana cruza la frontera lunes/domingo -- dos tramos.
            for minute in covered.iter_mut().skip(start as usize) {
                *minute = true;
            }
            for minute in covered.iter_mut().take((end + 1) as usize) {
                *minute = true;
            }
        }
    }

    let mut gaps: Vec<i32> = Vec::new();
    let mut idx = 0usize;
    while idx < covered.len() {
        if !covered[idx] {
            let gap_start = idx;
            while idx < covered.len() && !covered[idx] {
                idx += 1;
            }
            gaps.push((idx - gap_start) as i32);
        } else {
            idx += 1;
        }
    }

    // La semana es cíclica: un hueco que toca ambos extremos del array
    // (domingo 23:59 y lunes 00:00) es UN SOLO hueco real, no dos --
    // fusiona el primero y el último si ambos existen.
    if gaps.len() >= 2 && !covered[0] && !covered[covered.len() - 1] {
        let first = gaps.remove(0);
        let last = gaps.pop().expect("gaps.len() >= 2 tras el remove anterior garantiza al menos un elemento");
        gaps.push(first + last);
    }

    const LARGE_GAP_THRESHOLD_MINUTES: i32 = 20 * 60; // 20 horas: umbral que distingue un hueco "tipo fin de semana" de uno diario corto.

    match gaps.len() {
        0 => Ok(CalendarFingerprint::Continuous),
        1 => Ok(CalendarFingerprint::WeeklyGap),
        _ => {
            // NOTA DE EQUIVALENCIA (gate de mutación, verificada por el Tech-Lead 2026-07-25):
            // el mutante `>=` → `<` en este filtro es EQUIVALENTE — ninguna entrada puede matarlo.
            // Voltear el predicado sustituye `large_gap_count` por `gaps.len() - large_gap_count`
            // (todo hueco es grande o corto; los dos predicados son complementarios). La condición
            // de clasificación `c >= 1 && c < len` es invariante bajo `c ↔ len - c`, porque
            // `(len - c) >= 1  ⟺  c < len`  y  `(len - c) < len  ⟺  c >= 1`. Por tanto el
            // fingerprint resultante es idéntico para cualquier multiconjunto de huecos.
            let large_gap_count = gaps.iter().filter(|&&size| size >= LARGE_GAP_THRESHOLD_MINUTES).count();
            if large_gap_count >= 1 && large_gap_count < gaps.len() {
                // Un hueco grande (semanal) + al menos un hueco corto
                // (mantenimiento diario) conviviendo -- forma de Futuros.
                Ok(CalendarFingerprint::WeeklyGapPlusDailyMaintenance)
            } else {
                // Varios huecos de tamaño comparable (noches + fin de
                // semana) -- forma de Acciones.
                Ok(CalendarFingerprint::DailySession)
            }
        }
    }
}

// ── SESSION_TEMPLATE_SEED_SET: plantilla representativa de Futuros ──────

/// Ventanas semanales de la plantilla semilla de Futuros estilo CME
/// Globex (`SESSION_TEMPLATE_SEED_SET`, confirmado por el propietario
/// 2026-07-25: Forex 24/5, Cripto 24/7 y Futuros).
///
/// **GOVERNANCE-4.7 -- valor representativo, NO verdad sellada:** los
/// calendarios de futuros son específicos de cada mercado/producto (un
/// contrato de energía no tiene la misma pausa de mantenimiento que un
/// índice bursátil). Esta función fija un default RAZONABLE Y EDITABLE
/// (zona `America/Chicago`, apertura semanal domingo 17:00, pausa de
/// mantenimiento diaria de 15 minutos a las 16:00, cierre semanal viernes
/// 16:00) -- el usuario lo ajusta sobre esta semilla para el producto real
/// que opera. NO se afirma que este sea el horario exacto de ningún
/// contrato CME concreto.
///
/// Produce 5 ventanas (domingo->lunes, y una por cada día hábil
/// lunes-jueves hasta el cierre del viernes) para que la pausa de
/// mantenimiento aparezca CADA día hábil, no una sola vez en la semana --
/// es lo que distingue esta forma (`WeeklyGapPlusDailyMaintenance`) de la
/// de Forex (`WeeklyGap`, sin ninguna pausa intradía).
pub fn cme_globex_representative_windows() -> Vec<SessionWindow> {
    const DAILY_CLOSE_MINUTES: i32 = 959; // 15:59 -- último minuto antes de la pausa de las 16:00.
    const DAILY_REOPEN_MINUTES: i32 = 975; // 16:15 -- primer minuto tras la pausa de 15 minutos.
    const WEEKLY_OPEN_MINUTES: i32 = 1_020; // 17:00 domingo.
    const WEEKLY_CLOSE_MINUTES: i32 = 960; // 16:00 viernes -- cierre semanal.

    vec![
        // Domingo 17:00 -> Lunes 15:59 (cruza la frontera de semana lineal).
        SessionWindow { start_day: Weekday::Sun, start_time_minutes: WEEKLY_OPEN_MINUTES, end_day: Weekday::Mon, end_time_minutes: DAILY_CLOSE_MINUTES },
        // Lunes 16:15 -> Martes 15:59.
        SessionWindow { start_day: Weekday::Mon, start_time_minutes: DAILY_REOPEN_MINUTES, end_day: Weekday::Tue, end_time_minutes: DAILY_CLOSE_MINUTES },
        // Martes 16:15 -> Miércoles 15:59.
        SessionWindow { start_day: Weekday::Tue, start_time_minutes: DAILY_REOPEN_MINUTES, end_day: Weekday::Wed, end_time_minutes: DAILY_CLOSE_MINUTES },
        // Miércoles 16:15 -> Jueves 15:59.
        SessionWindow { start_day: Weekday::Wed, start_time_minutes: DAILY_REOPEN_MINUTES, end_day: Weekday::Thu, end_time_minutes: DAILY_CLOSE_MINUTES },
        // Jueves 16:15 -> Viernes 16:00 (cierre semanal).
        SessionWindow { start_day: Weekday::Thu, start_time_minutes: DAILY_REOPEN_MINUTES, end_day: Weekday::Fri, end_time_minutes: WEEKLY_CLOSE_MINUTES },
    ]
}

// ── TTR-003: cadena de resolución de sesión (pura) ───────────────────────

/// Resuelve qué `template_id` aplica para un `(instrument_id, venue)`
/// dado, siguiendo la cadena: override de bróker -> default de
/// instrumento (ADR-0162, "Proceso" de `session-catalog`). Ambos IDs
/// llegan YA resueltos por el llamador (típicamente
/// `instrument-catalog::resolve_instrument_terms`, que sí tiene acceso a
/// `broker_instrument_terms`/`instrument_master`) -- esta función NUNCA
/// toca esas tablas directamente.
pub fn resolve_session_template_id(
    broker_override_template_id: Option<&str>,
    instrument_default_template_id: Option<&str>,
) -> Option<String> {
    broker_override_template_id
        .map(str::to_string)
        .or_else(|| instrument_default_template_id.map(str::to_string))
}

// ── Integridad de fila (audit_hash encadenado, Grupo I ADR-0020) ────────

const GENESIS_MARKER: &str = "GENESIS";

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Calcula el `audit_hash` de una versión de `session_template`.
pub fn compute_session_template_audit_hash(
    id: &str,
    updated_at_ns: i64,
    row_version: i64,
    previous_audit_hash: Option<&str>,
    template_code: &str,
    display_name: &str,
    market_timezone_ref: &str,
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
    push(template_code);
    push(display_name);
    push(market_timezone_ref);

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    encode_hex(&hasher.finalize())
}

/// Calcula el `audit_hash` de una versión de `session_template_window`.
pub fn compute_session_window_audit_hash(
    id: &str,
    updated_at_ns: i64,
    row_version: i64,
    previous_audit_hash: Option<&str>,
    session_template_id: &str,
    window: SessionWindow,
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
    push(session_template_id);
    push(window.start_day.as_str());
    push(&window.start_time_minutes.to_string());
    push(window.end_day.as_str());
    push(&window.end_time_minutes.to_string());

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    encode_hex(&hasher.finalize())
}

// ── Pruebas de la lógica pura ────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// CRIPTO 24/7: una ventana que cubre la semana entera (lunes 00:00 a
    /// domingo 23:59) no deja ningún hueco -- huella `Continuous`.
    fn crypto_247_windows() -> Vec<SessionWindow> {
        vec![SessionWindow { start_day: Weekday::Mon, start_time_minutes: 0, end_day: Weekday::Sun, end_time_minutes: 1_439 }]
    }

    /// FOREX 24/5: abre domingo 22:00, cierra el viernes SIGUIENTE a las
    /// 22:00 -- cruza la frontera lunes/domingo (`end < start` en la
    /// semana lineal). Deja un único hueco semanal (fin de semana).
    fn forex_245_windows() -> Vec<SessionWindow> {
        vec![SessionWindow { start_day: Weekday::Sun, start_time_minutes: 1_320, end_day: Weekday::Fri, end_time_minutes: 1_320 }]
    }

    // ── CRITERIO #5 (Orden §5): fingerprint_distinguishes_crypto_forex_futures ──

    /// Ventanas 24/7 -> continua; Forex 24/5 -> un hueco semanal; Futuros
    /// (CME Globex representativo) -> hueco semanal + hueco diario de
    /// mantenimiento -- discriminante por forma de mercado: las TRES
    /// huellas se calculan en la MISMA prueba para que un mutante que
    /// colapse cualquier par al mismo resultado no pueda pasar las tres
    /// aserciones a la vez.
    #[test]
    fn fingerprint_distinguishes_crypto_forex_futures() {
        let crypto = compute_expected_fingerprint(&crypto_247_windows()).expect("cripto debe derivar huella");
        assert_eq!(crypto, CalendarFingerprint::Continuous, "24/7 sin huecos debe ser Continuous");

        let forex = compute_expected_fingerprint(&forex_245_windows()).expect("forex debe derivar huella");
        assert_eq!(forex, CalendarFingerprint::WeeklyGap, "24/5 con hueco de fin de semana debe ser WeeklyGap");

        let futures = compute_expected_fingerprint(&cme_globex_representative_windows()).expect("futuros debe derivar huella");
        assert_eq!(
            futures,
            CalendarFingerprint::WeeklyGapPlusDailyMaintenance,
            "el default representativo de Futuros (hueco semanal + pausa diaria) debe ser WeeklyGapPlusDailyMaintenance"
        );

        assert_ne!(crypto, forex, "cripto y forex deben producir huellas DISTINTAS");
        assert_ne!(forex, futures, "forex y futuros deben producir huellas DISTINTAS");
        assert_ne!(crypto, futures, "cripto y futuros deben producir huellas DISTINTAS");
    }

    /// La pausa de mantenimiento diaria del default de Futuros mide
    /// EXACTAMENTE 15 minutos (16:00 a 16:15) en cada uno de los cuatro
    /// días hábiles internos (lunes a jueves) -- valor calculado a mano,
    /// no derivado de la misma fórmula que el código bajo prueba. Si
    /// alguien desplaza `DAILY_REOPEN_MINUTES` por error, esta prueba lo
    /// atrapa aunque la CLASIFICACIÓN general siga siendo la misma.
    #[test]
    fn futures_daily_maintenance_gap_measures_fifteen_minutes() {
        let windows = cme_globex_representative_windows();
        // Ventana 2 (lunes 16:15) empieza justo donde termina la pausa
        // que sigue a la ventana 1 (que cierra domingo->lunes a las 15:59).
        let monday_close = windows[0].end_time_minutes; // 959 (15:59)
        let monday_reopen = windows[1].start_time_minutes; // 975 (16:15)
        assert_eq!(monday_reopen - monday_close - 1, 15, "la pausa de mantenimiento diaria debe medir exactamente 15 minutos");
    }

    /// Sin ventanas, la derivación falla explícitamente -- nunca se
    /// aproxima a `Continuous` por omisión (comprobación de límite
    /// degenerado, `critical-domain-reasoning.md`).
    #[test]
    fn compute_expected_fingerprint_rejects_empty_windows() {
        let result = compute_expected_fingerprint(&[]);
        assert_eq!(result, Err(SessionCatalogDomainError::NoWindows));
    }

    /// El hueco calculado para Forex 24/5 mide EXACTAMENTE lo esperado:
    /// desde el viernes 22:00 (minuto 7080 de la semana) hasta el domingo
    /// 22:00 (minuto 9960), 2 880 minutos (48 horas) menos el minuto de
    /// apertura ya cubierto -- valor calculado a mano, no derivado de la
    /// misma fórmula que el código bajo prueba.
    #[test]
    fn forex_weekly_gap_spans_the_expected_weekend_minutes() {
        // Viernes 22:00 = día índice 4 * 1440 + 1320 = 7080.
        // Domingo 22:00 = día índice 6 * 1440 + 1320 = 9960.
        // El hueco cubre (7080, 9960) exclusivo en ambos extremos -- el
        // propio minuto 7080 y el 9960 están cubiertos por la ventana.
        let expected_gap_start = 4 * 1_440 + 1_320 + 1; // 7081
        let expected_gap_end = 6 * 1_440 + 1_320 - 1; // 9959
        let expected_size = expected_gap_end - expected_gap_start + 1;
        assert_eq!(expected_size, 2_879, "el hueco de fin de semana de Forex debe medir 2879 minutos calculados a mano");

        // La propia clasificación ya confirma "un solo hueco" -- esta
        // prueba fija el TAMAÑO exacto para detectar un desplazamiento de
        // un minuto en los límites de la ventana.
        let fingerprint = compute_expected_fingerprint(&forex_245_windows()).expect("debe derivar");
        assert_eq!(fingerprint, CalendarFingerprint::WeeklyGap);
    }

    // ── FIX C (gate de mutación, hallazgo Tech-Lead 2026-07-25): aritmética
    // interna de compute_expected_fingerprint, más allá de las 3 formas
    // gruesas de las plantillas semilla ──────────────────────────────────

    /// CRITERIO DE CIERRE (fusión cíclica, líneas 184/187): un conjunto de
    /// ventanas fabricado para dejar DOS huecos reales -- uno que
    /// ENVUELVE la frontera domingo-noche→lunes-mañana (partido en dos
    /// tramos por el array lineal: `[0,699]` al inicio y `[9380,10079]`
    /// al final, 700 minutos cada uno) y otro en medio de la semana
    /// (`[5000,5099]`, 100 minutos). Sin la fusión de los dos tramos del
    /// extremo, `gaps.len()` daría 3 y el hueco envolvente se vería como
    /// dos huecos de 700 minutos (ninguno "grande" según el umbral de
    /// 1200) en vez de uno solo de 1400 (sí grande) -- cambiando la
    /// clasificación final de `WeeklyGapPlusDailyMaintenance` a
    /// `DailySession`. Esta prueba fija el resultado CORRECTO, así que
    /// tanto "no fusionar" (184) como "fusionar con la suma equivocada"
    /// (187, ej. `first` solo, sin `+ last`) la hacen fallar.
    #[test]
    fn fingerprint_fuses_the_wraparound_gap_into_one_correctly_sized_hole() {
        let windows = vec![
            // Cubre [700, 4999] -- deja [0,699] sin cubrir al inicio.
            SessionWindow { start_day: Weekday::Mon, start_time_minutes: 700, end_day: Weekday::Thu, end_time_minutes: 679 },
            // Cubre [5100, 9379] -- deja [5000,5099] (hueco medio) y
            // [9380,10079] (resto del envolvente) sin cubrir.
            SessionWindow { start_day: Weekday::Thu, start_time_minutes: 780, end_day: Weekday::Sun, end_time_minutes: 739 },
        ];

        let fingerprint = compute_expected_fingerprint(&windows).expect("debe derivar huella");
        assert_eq!(
            fingerprint,
            CalendarFingerprint::WeeklyGapPlusDailyMaintenance,
            "el hueco envolvente (700+700=1400 min, grande) más el hueco medio (100 min, corto) es la forma de Futuros -- \
             si la fusión no ocurre o suma mal, el envolvente queda como dos huecos de 700 min (ninguno grande) y la huella cambia"
        );
    }

    /// CRITERIO DE CIERRE (mutación `+`→`*` en `gaps.push(first + last)`,
    /// línea 187 -- hallazgo del re-corte del gate 2026-07-25): la prueba
    /// de fusión anterior (700+700=1400) NO mata este mutante porque tanto
    /// la suma como el producto de sus dos mitades caen del lado "grande"
    /// del umbral (1400 grande, 700×700=490 000 también grande) -- el
    /// conteo de huecos grandes queda igual en los dos casos y el
    /// fingerprint no cambia. Para matarlo hace falta que SUMA y
    /// PRODUCTO caigan en LADOS OPUESTOS del umbral de 1200: dos mitades
    /// de 500 y 600 (suma 1100, corto) cuyo producto (300 000) sí es
    /// grande, más un hueco medio de 1300 (grande) para que el CONTEO de
    /// huecos grandes difiera entre los dos casos:
    /// - con `+` (correcto): huecos finales `[1300, 1100]` -- 1 grande,
    ///   2 huecos totales -- `1>=1 && 1<2` → `WeeklyGapPlusDailyMaintenance`.
    /// - con `*` (mutante): huecos finales `[1300, 300000]` -- 2 grandes,
    ///   2 huecos totales -- `2>=1 && 2<2` es `false` → `DailySession`.
    #[test]
    fn fingerprint_wraparound_merge_uses_sum_not_product_of_its_two_halves() {
        let windows = vec![
            // Cubre [500, 4999] -- deja [0,499] (mitad envolvente = 500) sin cubrir al inicio.
            SessionWindow { start_day: Weekday::Mon, start_time_minutes: 500, end_day: Weekday::Thu, end_time_minutes: 679 },
            // Cubre [6300, 9479] -- deja [5000,6299] (hueco medio = 1300, grande) y
            // [9480,10079] (mitad envolvente = 600) sin cubrir.
            SessionWindow { start_day: Weekday::Fri, start_time_minutes: 540, end_day: Weekday::Sun, end_time_minutes: 839 },
        ];

        let fingerprint = compute_expected_fingerprint(&windows).expect("debe derivar huella");
        assert_eq!(
            fingerprint,
            CalendarFingerprint::WeeklyGapPlusDailyMaintenance,
            "el envolvente debe fusionarse por SUMA (500+600=1100, corto) -- si se fusiona por producto \
             (500*600=300000, grande), el conteo de huecos grandes pasa de 1 a 2 y la huella cambia a DailySession"
        );
    }

    /// CRITERIO DE CIERRE (aritmética de borde `end + 1`, línea 161): dos
    /// ventanas que juntas cubren el lunes COMPLETO sin hueco entre ellas
    /// (`00:00-11:59` y `12:00-23:59`, el minuto 719 y el 720 son
    /// adyacentes) -- el resto de la semana (martes a domingo) es el
    /// ÚNICO hueco real. Si el límite superior de una ventana no incluye
    /// su propio `end_time_minutes` (un `end` en vez de `end + 1`), el
    /// último minuto de cada ventana queda sin cubrir, apareciendo como
    /// un hueco fantasma que cambia la clasificación de `WeeklyGap` (un
    /// hueco real) a `WeeklyGapPlusDailyMaintenance` (el hueco real +
    /// el fantasma).
    #[test]
    fn fingerprint_window_end_minute_is_inclusive_no_phantom_gap_at_the_seam() {
        let windows = vec![
            SessionWindow { start_day: Weekday::Mon, start_time_minutes: 0, end_day: Weekday::Mon, end_time_minutes: 719 },
            SessionWindow { start_day: Weekday::Mon, start_time_minutes: 720, end_day: Weekday::Mon, end_time_minutes: 1_439 },
        ];

        let fingerprint = compute_expected_fingerprint(&windows).expect("debe derivar huella");
        assert_eq!(
            fingerprint,
            CalendarFingerprint::WeeklyGap,
            "las dos ventanas del lunes deben empalmar SIN hueco -- si el último minuto de cada una no queda cubierto, \
             aparece un hueco fantasma de 1 minuto en la costura y la huella cambia a WeeklyGapPlusDailyMaintenance"
        );
    }

    /// CRITERIO DE CIERRE (aritmética de borde `end + 1` en la RAMA DE
    /// ENVOLTURA, línea 161 -- distinta ocurrencia de la misma expresión
    /// que la prueba anterior, que solo ejercita la rama recta de línea
    /// 153): una ventana que ENVUELVE la frontera domingo→lunes
    /// (`SUN 12:00 -> TUE 11:59`) seguida de otra que retoma exactamente
    /// donde la primera termina (`TUE 12:00 -> SAT 11:59`) -- el minuto
    /// 2159 (martes 11:59, el ÚLTIMO minuto del tramo que usa `end + 1`
    /// en la rama de envoltura) debe quedar cubierto por la primera
    /// ventana y por lo tanto contiguo con la segunda, sin hueco entre
    /// ambas. Si ese minuto no quedara cubierto (bug `end` en vez de
    /// `end + 1`), aparecería un hueco fantasma de 1 minuto AISLADO
    /// (rodeado de minutos cubiertos a ambos lados), y la huella pasaría
    /// de `WeeklyGap` (un solo hueco real, el resto de la semana) a
    /// `WeeklyGapPlusDailyMaintenance` (ese hueco real + el fantasma).
    #[test]
    fn fingerprint_wraparound_branch_end_minute_is_inclusive_no_phantom_gap() {
        let windows = vec![
            // Envuelve domingo->lunes: tramo1 [9360,10079], tramo2 [0,2159]
            // (este último usa `end + 1` en la rama de envoltura).
            SessionWindow { start_day: Weekday::Sun, start_time_minutes: 720, end_day: Weekday::Tue, end_time_minutes: 719 },
            // Retoma justo en 2160 -- si el minuto 2159 no quedó cubierto
            // por la ventana anterior, queda aislado entre 2158 (cubierto)
            // y 2160 (cubierto por esta ventana).
            SessionWindow { start_day: Weekday::Tue, start_time_minutes: 720, end_day: Weekday::Sat, end_time_minutes: 719 },
        ];

        let fingerprint = compute_expected_fingerprint(&windows).expect("debe derivar huella");
        assert_eq!(
            fingerprint,
            CalendarFingerprint::WeeklyGap,
            "las dos ventanas deben empalmar SIN hueco en la costura de la rama de envoltura -- un hueco fantasma de 1 \
             minuto ahí cambiaría la huella a WeeklyGapPlusDailyMaintenance"
        );
    }

    /// CRITERIO DE CIERRE (umbral de hueco grande, líneas 190/196): un
    /// hueco de 300 minutos (mantenimiento corto) y uno de 1300 minutos
    /// (fin de semana) -- con el umbral correcto (1200 min = 20h), solo
    /// el segundo cuenta como "grande". Si el umbral se calculara mal
    /// (ej. `20 * 60` mutado a `20 + 60 = 80`), el de 300 minutos también
    /// contaría como grande, y la forma pasaría de
    /// `WeeklyGapPlusDailyMaintenance` a `DailySession`.
    #[test]
    fn fingerprint_large_gap_threshold_distinguishes_maintenance_from_weekend_gap() {
        let windows = vec![
            SessionWindow { start_day: Weekday::Mon, start_time_minutes: 0, end_day: Weekday::Mon, end_time_minutes: 999 },
            SessionWindow { start_day: Weekday::Mon, start_time_minutes: 1_300, end_day: Weekday::Fri, end_time_minutes: 239 },
            SessionWindow { start_day: Weekday::Sat, start_time_minutes: 100, end_day: Weekday::Sun, end_time_minutes: 1_439 },
        ];

        let fingerprint = compute_expected_fingerprint(&windows).expect("debe derivar huella");
        assert_eq!(
            fingerprint,
            CalendarFingerprint::WeeklyGapPlusDailyMaintenance,
            "el hueco de 300 min NO debe contar como grande (umbral real 1200 min) -- si el umbral se calcula mal, ambos \
             huecos cuentan como grandes y la huella cambia a DailySession"
        );
    }

    /// CRITERIO DE CIERRE (frontera del umbral, `>=` en línea 196): un
    /// hueco de EXACTAMENTE 1200 minutos (el propio umbral) debe contar
    /// como grande -- discrimina `>=` de `>`. Con `>`, un hueco de
    /// exactamente 1200 minutos NO contaría como grande y la forma
    /// cambiaría de `WeeklyGapPlusDailyMaintenance` a `DailySession`.
    #[test]
    fn fingerprint_large_gap_threshold_boundary_is_inclusive() {
        let windows = vec![
            SessionWindow { start_day: Weekday::Mon, start_time_minutes: 0, end_day: Weekday::Mon, end_time_minutes: 999 },
            SessionWindow { start_day: Weekday::Mon, start_time_minutes: 1_100, end_day: Weekday::Tue, end_time_minutes: 559 },
            SessionWindow { start_day: Weekday::Wed, start_time_minutes: 320, end_day: Weekday::Sun, end_time_minutes: 1_439 },
        ];

        let fingerprint = compute_expected_fingerprint(&windows).expect("debe derivar huella");
        assert_eq!(
            fingerprint,
            CalendarFingerprint::WeeklyGapPlusDailyMaintenance,
            "un hueco de EXACTAMENTE 1200 minutos debe contar como grande (frontera inclusiva `>=`) -- con `>` en vez de \
             `>=`, este hueco no contaría como grande y la huella cambiaría a DailySession"
        );
    }

    /// CRITERIO DE CIERRE ("todos los huecos grandes", línea 197): DOS
    /// huecos, ambos tamaño-fin-de-semana (1300 min cada uno) -- NO debe
    /// clasificarse como Futuros (que exige un hueco grande CONVIVIENDO
    /// con al menos uno corto). Discrimina `<` de `<=` en
    /// `large_gap_count < gaps.len()`: con `<=`, dos huecos igual de
    /// grandes pasarían la condición y se clasificarían mal como
    /// `WeeklyGapPlusDailyMaintenance`.
    #[test]
    fn fingerprint_all_large_gaps_is_not_the_futures_shape() {
        let windows = vec![
            SessionWindow { start_day: Weekday::Mon, start_time_minutes: 0, end_day: Weekday::Mon, end_time_minutes: 999 },
            SessionWindow { start_day: Weekday::Tue, start_time_minutes: 860, end_day: Weekday::Wed, end_time_minutes: 1_119 },
            SessionWindow { start_day: Weekday::Thu, start_time_minutes: 980, end_day: Weekday::Sun, end_time_minutes: 1_439 },
        ];

        let fingerprint = compute_expected_fingerprint(&windows).expect("debe derivar huella");
        assert_eq!(
            fingerprint,
            CalendarFingerprint::DailySession,
            "dos huecos igual de grandes (ninguno corto) NUNCA es la forma de Futuros -- si `<` se relaja a `<=`, \
             se clasificaría mal como WeeklyGapPlusDailyMaintenance"
        );
    }

    // ── Cadena de resolución de sesión (TTR-003) ─────────────────────────

    /// El override de bróker gana cuando ambos existen.
    #[test]
    fn resolve_session_template_id_prefers_broker_override_over_instrument_default() {
        let resolved = resolve_session_template_id(Some("tpl-broker"), Some("tpl-instrument-default"));
        assert_eq!(resolved, Some("tpl-broker".to_string()));
    }

    /// Sin override, cae al default del instrumento.
    #[test]
    fn resolve_session_template_id_falls_back_to_instrument_default_without_override() {
        let resolved = resolve_session_template_id(None, Some("tpl-instrument-default"));
        assert_eq!(resolved, Some("tpl-instrument-default".to_string()));
    }

    /// Sin ninguno de los dos, resuelve `None` -- nunca inventa un template.
    #[test]
    fn resolve_session_template_id_returns_none_without_any_source() {
        let resolved = resolve_session_template_id(None, None);
        assert_eq!(resolved, None);
    }

    // ── Vocabulario: round-trip exacto ───────────────────────────────────

    #[test]
    fn weekday_round_trips_through_its_canonical_string_for_every_variant() {
        for variant in
            [Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri, Weekday::Sat, Weekday::Sun]
        {
            let text = variant.as_str();
            let decoded = Weekday::from_str_value(text).expect("todo texto producido por as_str debe decodificar");
            assert_eq!(decoded, variant, "round-trip debe preservar la variante exacta: {text}");
        }
        assert_eq!(Weekday::from_str_value("FUNDAY"), None);
    }

    // ── Integridad: el hash depende REALMENTE del contenido ─────────────

    #[test]
    fn session_template_audit_hash_depends_on_content_not_just_identity() {
        let hash_a = compute_session_template_audit_hash("id-1", 1_000, 1, None, "FOREX_245", "Forex 24/5", "America/New_York");
        let hash_a_again =
            compute_session_template_audit_hash("id-1", 1_000, 1, None, "FOREX_245", "Forex 24/5", "America/New_York");
        assert_eq!(hash_a, hash_a_again, "las mismas entradas, construidas por separado, deben producir el mismo hash");

        let hash_b = compute_session_template_audit_hash("id-1", 1_000, 1, None, "FOREX_245", "Forex 24/5", "Europe/London");
        assert_ne!(hash_a, hash_b, "cambiar market_timezone_ref debe cambiar el hash resultante");
    }

    #[test]
    fn session_window_audit_hash_depends_on_content_not_just_identity() {
        let window = SessionWindow { start_day: Weekday::Sun, start_time_minutes: 1_320, end_day: Weekday::Fri, end_time_minutes: 1_320 };
        let hash_a = compute_session_window_audit_hash("id-1", 1_000, 1, None, "tpl-1", window);
        let hash_a_again = compute_session_window_audit_hash("id-1", 1_000, 1, None, "tpl-1", window);
        assert_eq!(hash_a, hash_a_again);

        let other_window = SessionWindow { start_day: Weekday::Mon, start_time_minutes: 0, end_day: Weekday::Sun, end_time_minutes: 1_439 };
        let hash_b = compute_session_window_audit_hash("id-1", 1_000, 1, None, "tpl-1", other_window);
        assert_ne!(hash_a, hash_b, "cambiar la ventana debe cambiar el hash resultante");
    }
}
