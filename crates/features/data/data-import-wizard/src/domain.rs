//! [CORE] Lógica pura del lector de barras de MT5: parseo de línea,
//! conversión de escala, resolución de zona horaria y detección de
//! imposibilidad física. Sin I/O, sin reloj de sistema, sin acceso a base
//! de datos (ADR-0002/0004, FCIS).
//!
//! Todo lo que este módulo necesita para producir un resultado determinista
//! llega como parámetro — nunca lo busca por su cuenta (ni en disco, ni en
//! otra tabla, ni en el reloj del sistema).

use chrono::{NaiveDate, NaiveTime};
use chrono_tz::Tz;
use sha2::{Digest, Sha256};
use std::str::FromStr;

// ═══════════════════════════════════════════════════════════════════════
// Plantilla de plataforma MT5 — el mapeo de columnas lo afirma Drasus
// (LDR-0008: una plantilla mal construida es defecto de producto, nunca
// responsabilidad transferida al usuario).
// ═══════════════════════════════════════════════════════════════════════

/// Código estable de la plantilla, sellado en cada fila del ledger
/// (`platform_template_code`) para que una corrección futura de esta
/// plantilla sea rastreable por versión, nunca silenciosa.
pub const MT5_BAR_TEMPLATE_CODE: &str = "MT5_M1_BAR_V1";

/// Versión de la plantilla. Sube cuando el mapeo de columnas cambia (ej. si
/// MetaTrader agrega una columna nueva a su exportación).
pub const MT5_BAR_TEMPLATE_VERSION: &str = "1.0.0";

/// Unidad del campo `<SPREAD>` que ESTA plantilla afirma (STORY-068
/// criterio de aceptación #13, LDR-0008: la plantilla la afirma Drasus, no
/// el usuario). MetaTrader documenta `MqlRates.spread` como "spread value
/// in points" -- el último dígito de precio (para EURUSD a 5 decimales,
/// 1 punto = 0.00001), NUNCA un pip completo (1 pip = 10 puntos en un par
/// de 5 dígitos). Es un hecho FIJO de la plantilla, no una declaración que
/// varíe por corrida como `real_volume_unit_declared`: MT5 nunca reporta
/// este campo en otra unidad.
///
/// Verificación empírica que sostiene esta declaración (no una suposición):
/// `datasets/EURUSD_M1_....csv`, junio 2024, valor más frecuente `0`
/// (27 473 filas de un mes), picos en `14`/`12`/`4`/`1`/`13`/`11`. Leídos
/// como puntos, un pico de 14 puntos = 1.4 pips -- coherente con un spread
/// real de EURUSD. Leídos como pips, serían spreads de 10-14 pips --
/// físicamente implausibles para ese par en ese periodo. El factor de diez
/// entre ambas lecturas es exactamente el riesgo que este campo cierra.
pub const MT5_SPREAD_REPORTED_UNIT: &str = "POINTS_MT5";

/// Expone la unidad de spread que la plantilla MT5 M1 declara. Función
/// (no solo la constante) para que el llamador (orquestador) trate esta
/// declaración igual que cualquier otro hecho de la plantilla -- y para que
/// una plantilla futura que no la declare (hoy no existe ninguna así) sea
/// un `None` explícito en vez de un valor inventado.
pub fn mt5_bar_spread_reported_unit() -> &'static str {
    MT5_SPREAD_REPORTED_UNIT
}

/// Cabecera exacta que un archivo debe traer para que esta plantilla lo
/// reconozca. Comparación literal, sensible a mayúsculas: MT5 siempre
/// exporta esta cabecera tal cual cuando el usuario pide "Open Prices" con
/// separador de tabulación desde el terminal.
const MT5_BAR_EXPECTED_HEADER: &str = "<DATE>\t<TIME>\t<OPEN>\t<HIGH>\t<LOW>\t<CLOSE>\t<TICKVOL>\t<VOL>\t<SPREAD>";

/// Cuántas columnas trae una fila de dato bien formada (mismo conteo que la cabecera).
const MT5_BAR_EXPECTED_FIELD_COUNT: usize = 9;

/// Un archivo cuya cabecera no coincide con ninguna plantilla conocida se
/// rechaza con este motivo explícito — nunca una importación parcial ni una
/// interpretación adivinada (ADR-0156 Restricción 1).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TemplateDetectionError {
    #[error(
        "la cabecera del archivo no coincide con ninguna plantilla de plataforma reconocida \
         (se esperaba exactamente: {expected:?}, se recibió: {received:?}) — \
         importa manualmente declarando el mapeo, o verifica que el archivo venga de MetaTrader 5"
    )]
    UnrecognizedHeader { expected: String, received: String },
}

/// Verifica si `header_line` (la primera línea del archivo, tal como llega,
/// con su posible `\r` de Windows) coincide con la plantilla de barras M1 de
/// MT5. Devuelve `Ok(())` si coincide; si no, un error tipado con el motivo.
pub fn detect_mt5_bar_template(header_line: &str) -> Result<(), TemplateDetectionError> {
    let stripped = strip_line_ending(header_line);
    if stripped == MT5_BAR_EXPECTED_HEADER {
        Ok(())
    } else {
        Err(TemplateDetectionError::UnrecognizedHeader {
            expected: MT5_BAR_EXPECTED_HEADER.to_string(),
            received: stripped.to_string(),
        })
    }
}

/// El mapeo columna origen → campo canónico que esta plantilla declara.
/// Se sella tal cual en `column_mapping_json` del ledger (ADR-0156
/// Restricción 7, campo 2) — es la evidencia de qué le mostró Drasus al
/// usuario, o qué aplicó por su cuenta en la vía de plantilla.
pub fn mt5_bar_column_mapping() -> Vec<(String, String)> {
    [
        ("<DATE>", "bar_date_source"),
        ("<TIME>", "bar_time_source"),
        ("<OPEN>", "open"),
        ("<HIGH>", "high"),
        ("<LOW>", "low"),
        ("<CLOSE>", "close"),
        ("<TICKVOL>", "volume_ticks"),
        ("<VOL>", "volume_traded"),
        ("<SPREAD>", "spread_reported"),
    ]
    .into_iter()
    .map(|(a, b)| (a.to_string(), b.to_string()))
    .collect()
}

// ═══════════════════════════════════════════════════════════════════════
// Parseo de línea cruda
// ═══════════════════════════════════════════════════════════════════════

/// Quita el final de línea de una cadena, sea `\r\n` (Windows, MT5) o solo
/// `\n`. Si la línea no trae ninguno de los dos (última línea de un archivo
/// sin salto final), la devuelve intacta. Esto es lo que hace que el
/// parseo NUNCA falle silenciosamente ni incluya un `\r` fantasma en el
/// último campo de la fila (`<SPREAD>`).
fn strip_line_ending(line: &str) -> &str {
    let without_trailing_lf = line.strip_suffix('\n').unwrap_or(line);
    without_trailing_lf.strip_suffix('\r').unwrap_or(without_trailing_lf)
}

/// Errores posibles al convertir un campo individual de texto a su tipo
/// numérico. Cada variante identifica el campo para que el mensaje de
/// rechazo sea accionable, no un `ParseIntError` genérico.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FieldParseError {
    #[error("el campo de precio {field:?} no es un decimal válido: {raw:?}")]
    InvalidPrice { field: &'static str, raw: String },
    #[error(
        "el campo de precio {field:?} trae más de 8 decimales ({raw:?}) — se rechaza en vez de \
         truncar en silencio y perder precisión"
    )]
    ExcessPricePrecision { field: &'static str, raw: String },
    #[error("el campo entero {field:?} no es un entero válido: {raw:?}")]
    InvalidInteger { field: &'static str, raw: String },
}

/// Escala fija del Core para todo precio/magnitud monetaria (ADR-0141):
/// entero exacto equivalente a multiplicar el valor decimal por 10⁸.
const PRICE_SCALE: i64 = 100_000_000;

/// Convierte un precio decimal de texto (ej. `"1.11488"`) al entero escalado
/// ×10⁸ que el Core usa internamente (ADR-0002: precios como enteros
/// exactos, nunca coma flotante en el dominio).
///
/// Implementación por texto, NUNCA por `f64::parse` + multiplicación: un
/// `f64` no representa exactamente la mayoría de los decimales financieros
/// (`0.1_f64 * 1e8` ya no es `10_000_000` exacto en binario) — el redondeo
/// resultante rompería la ida y vuelta exacta que el criterio de aceptación
/// #2 de STORY-068 exige. Separar la parte entera de la fraccionaria como
/// cadenas y hacer aritmética de enteros evita ese problema por completo.
pub fn parse_price_to_scaled(field: &'static str, raw: &str) -> Result<i64, FieldParseError> {
    // Sin `.trim()` a propósito: un campo con espacio en blanco alrededor
    // (incluido un `\r` colado) es una fila malformada, no un valor válido
    // con ruido cosmético -- la limpieza de fin de línea es
    // responsabilidad exclusiva de `strip_line_ending` sobre la LÍNEA
    // completa, nunca de un parseo de campo permisivo que enmascare el
    // problema (un `\r` es whitespace para `str::trim`, así que confiar en
    // `trim` aquí ocultaría un `strip_line_ending` roto en vez de exponerlo).
    let (sign, unsigned) = match raw.strip_prefix('-') {
        Some(rest) => (-1_i64, rest),
        None => (1_i64, raw),
    };

    let mut parts = unsigned.splitn(2, '.');
    let integer_part = parts.next().unwrap_or("");
    let fractional_part = parts.next().unwrap_or("");

    if integer_part.is_empty() || !integer_part.bytes().all(|b| b.is_ascii_digit()) {
        return Err(FieldParseError::InvalidPrice { field, raw: raw.to_string() });
    }
    if !fractional_part.bytes().all(|b| b.is_ascii_digit()) {
        return Err(FieldParseError::InvalidPrice { field, raw: raw.to_string() });
    }
    // Más de 8 decimales perdería precisión al escalar a ×10⁸ -- se rechaza
    // en vez de truncar en silencio (guarda el hecho tal cual llegó).
    if fractional_part.len() > 8 {
        return Err(FieldParseError::ExcessPricePrecision { field, raw: raw.to_string() });
    }

    let integer_value: i64 = integer_part
        .parse()
        .map_err(|_| FieldParseError::InvalidPrice { field, raw: raw.to_string() })?;

    // Completa la parte fraccionaria a la derecha con ceros hasta 8 dígitos
    // (ej. "11488" con 5 decimales -> "11488000" con 8), y la interpreta
    // como entero -- así "1.11488" se vuelve exactamente 111_488_000.
    let mut padded_fraction = fractional_part.to_string();
    while padded_fraction.len() < 8 {
        padded_fraction.push('0');
    }
    let fractional_value: i64 = padded_fraction
        .parse()
        .map_err(|_| FieldParseError::InvalidPrice { field, raw: raw.to_string() })?;

    Ok(sign * (integer_value * PRICE_SCALE + fractional_value))
}

/// Función inversa de [`parse_price_to_scaled`] para la prueba de ida y
/// vuelta (criterio de aceptación #2): reconstruye la cadena decimal con
/// exactamente `decimals` dígitos fraccionarios, la misma cantidad que
/// traía el archivo de origen.
pub fn scaled_to_price_string(scaled: i64, decimals: u32) -> String {
    let sign = if scaled < 0 { "-" } else { "" };
    let magnitude = scaled.unsigned_abs();
    let integer_part = magnitude / (PRICE_SCALE as u64);
    let fractional_full = magnitude % (PRICE_SCALE as u64);
    // `fractional_full` siempre tiene 8 dígitos internamente (ceros a la
    // izquierda incluidos); se recorta a `decimals` dígitos como los traía
    // el archivo de origen.
    let fractional_str_8 = format!("{fractional_full:08}");
    let fractional_str = &fractional_str_8[..decimals as usize];
    format!("{sign}{integer_part}.{fractional_str}")
}

/// Convierte un campo entero plano (contador, sin escala) como `<TICKVOL>`
/// o `<SPREAD>` a `i64`. A diferencia de `parse_price_to_scaled`, no hay
/// escala ×10⁸ -- son conteos/puntos enteros tal como los reporta MT5.
pub fn parse_counter_field(field: &'static str, raw: &str) -> Result<i64, FieldParseError> {
    // Sin `.trim()` -- mismo motivo que `parse_price_to_scaled`: un `\r`
    // colado (el último campo de la línea, `<SPREAD>`, es precisamente
    // donde aparecería si `strip_line_ending` fallara) es whitespace para
    // `str::trim`, así que confiar en `trim` aquí enmascararía ese fallo en
    // vez de exponerlo -- criterio de aceptación #1 de STORY-068.
    raw.parse::<i64>().map_err(|_| FieldParseError::InvalidInteger { field, raw: raw.to_string() })
}

/// Convierte `<VOL>` (volumen real) a `i128`. Ensanche de ancho respecto al
/// resto de contadores (ADR-0141 enmienda 2026-07-19, TASK-061): un activo
/// de alta emisión puede desbordar `i64` bajo operación normal; este campo
/// nunca lo hará dentro del rango de un archivo de barras real, pero se usa
/// el mismo tipo que `Bars.volume_traded` para no introducir una asimetría
/// de tipos entre el lector y el esquema que alimenta.
pub fn parse_volume_field(raw: &str) -> Result<i128, FieldParseError> {
    // Sin `.trim()` -- mismo motivo que `parse_counter_field`.
    raw.parse::<i128>()
        .map_err(|_| FieldParseError::InvalidInteger { field: "<VOL>", raw: raw.to_string() })
}

/// Los nueve campos de una fila de barra M1 de MT5, ya parseados a sus
/// tipos nativos pero SIN combinar fecha+hora en un timestamp todavía (esa
/// combinación necesita la zona horaria declarada, que no es un dato de la
/// fila -- es un dato de la corrida completa).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mt5RawBarFields {
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub open: i64,
    pub high: i64,
    pub low: i64,
    pub close: i64,
    pub volume_ticks: i64,
    pub volume_traded_raw: i128,
    pub spread_reported_raw: i64,
}

/// Errores de parseo de una línea completa: además de los errores por
/// campo, cubre el conteo de columnas y el formato de fecha/hora.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Mt5LineParseError {
    #[error(
        "la fila no tiene {MT5_BAR_EXPECTED_FIELD_COUNT} columnas separadas por tabulador \
         (se encontraron {found}): {raw:?}"
    )]
    UnexpectedFieldCount { found: usize, raw: String },
    #[error("la fecha {raw:?} no cumple el formato MT5 \"YYYY.MM.DD\"")]
    InvalidDate { raw: String },
    #[error("la hora {raw:?} no cumple el formato MT5 \"HH:MM:SS\"")]
    InvalidTime { raw: String },
    #[error(transparent)]
    Field(#[from] FieldParseError),
}

/// Parsea una línea cruda de dato (nunca la cabecera) del archivo de barras
/// M1 de MT5. Tolera el final de línea `\r\n` de Windows intacto -- lo
/// primero que hace es quitarlo, así el último campo (`<SPREAD>`) nunca
/// sale contaminado con un `\r` (criterio de aceptación #1 de STORY-068).
pub fn parse_mt5_bar_line(line: &str) -> Result<Mt5RawBarFields, Mt5LineParseError> {
    let stripped = strip_line_ending(line);
    let fields: Vec<&str> = stripped.split('\t').collect();
    if fields.len() != MT5_BAR_EXPECTED_FIELD_COUNT {
        return Err(Mt5LineParseError::UnexpectedFieldCount { found: fields.len(), raw: line.to_string() });
    }

    // Los índices siguen el orden fijo de la cabecera:
    // <DATE> <TIME> <OPEN> <HIGH> <LOW> <CLOSE> <TICKVOL> <VOL> <SPREAD>
    let date = NaiveDate::parse_from_str(fields[0], "%Y.%m.%d")
        .map_err(|_| Mt5LineParseError::InvalidDate { raw: fields[0].to_string() })?;
    let time = NaiveTime::parse_from_str(fields[1], "%H:%M:%S")
        .map_err(|_| Mt5LineParseError::InvalidTime { raw: fields[1].to_string() })?;
    let open = parse_price_to_scaled("<OPEN>", fields[2])?;
    let high = parse_price_to_scaled("<HIGH>", fields[3])?;
    let low = parse_price_to_scaled("<LOW>", fields[4])?;
    let close = parse_price_to_scaled("<CLOSE>", fields[5])?;
    let volume_ticks = parse_counter_field("<TICKVOL>", fields[6])?;
    let volume_traded_raw = parse_volume_field(fields[7])?;
    let spread_reported_raw = parse_counter_field("<SPREAD>", fields[8])?;

    Ok(Mt5RawBarFields { date, time, open, high, low, close, volume_ticks, volume_traded_raw, spread_reported_raw })
}

// ═══════════════════════════════════════════════════════════════════════
// Resolución de zona horaria declarada — NUNCA se infiere (ADR-0162
// Restricción 7).
// ═══════════════════════════════════════════════════════════════════════

/// La duración de una barra de 1 minuto en nanosegundos, usada para derivar
/// `bar_close_timestamp_ns` a partir de `bar_open_timestamp_ns`.
const ONE_MINUTE_NS: i64 = 60_000_000_000;

/// Errores al resolver el instante UTC de una fila a partir de su
/// fecha+hora local y el reloj de origen declarado.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TimestampResolutionError {
    /// Sin declaración de reloj de origen, la importación se rechaza en vez
    /// de asumir UTC (criterio de aceptación #3 de STORY-068; ADR-0162
    /// Restricción 7).
    #[error(
        "el archivo no trae un reloj de origen declarado por el usuario -- \
         la importación se rechaza en vez de asumir UTC (ADR-0162 Restricción 7)"
    )]
    MissingSourceTimezone,
    #[error("{0:?} no es un identificador de zona horaria IANA reconocido")]
    UnknownTimezone(String),
    /// La hora local cae en el hueco inexistente del cambio de horario de
    /// primavera (ej. las 02:30 de un día que salta directo de 02:00 a
    /// 03:00) -- ese instante local, literalmente, no ocurrió.
    #[error("la hora local {date} {time} no existe en la zona {timezone:?} (hueco del cambio de horario)")]
    NonexistentLocalTime { date: NaiveDate, time: NaiveTime, timezone: String },
}

/// El reloj de origen que el usuario declara para todo un archivo (ADR-0162
/// "Enmienda 2026-07-26"): una zona ancla IANA -- elegida por su calendario
/// de cambios de horario de verano, NO necesariamente por coincidir con el
/// offset literal del bróker -- más un desplazamiento fijo en minutos,
/// sumado al offset propio del ancla en todo instante.
///
/// `fixed_offset_minutes == 0` reproduce SIN PÉRDIDA la declaración plana de
/// una sola zona IANA (el mecanismo de hoy): no son dos mecanismos, uno es
/// el caso particular del otro. Caso real que sí necesita un desplazamiento
/// distinto de cero: IC Markets opera a GMT+2 en invierno y GMT+3 en verano,
/// pero cambia de horario en las fechas de Estados Unidos, no las europeas
/// -- ningún identificador IANA plano describe eso; ancla
/// `"America/New_York"` (hereda las fechas de cambio correctas, incluidas
/// las históricas) + `fixed_offset_minutes = 420` (7 horas) sí lo describe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceClockDeclaration<'a> {
    pub anchor_timezone_ref: &'a str,
    pub fixed_offset_minutes: i64,
}

/// Resuelve el instante de apertura de una barra en nanosegundos UTC, a
/// partir de su fecha+hora local (tal como el archivo la reporta) y el
/// reloj de origen (ancla + desplazamiento) que el usuario declaró para
/// todo el archivo.
///
/// `source_clock` es `None` cuando el usuario no declaró ningún reloj -- en
/// ese caso la función falla explícitamente en vez de asumir UTC. La
/// resolución del ancla usa las tablas estáticas de `chrono-tz` (reglas de
/// DST ya auditadas del estándar IANA, incluidas las históricas -- ej. el
/// cambio de fechas de DST de EE. UU. en 2007): es determinista y no toca el
/// reloj del sistema, por eso puede vivir en el Núcleo puro.
///
/// Algoritmo: primero se localiza la hora local COMO SI fuera hora del
/// ancla (con las reglas de DST del ancla) -- esto da el instante UTC que
/// produciría una declaración de zona plana sin desplazamiento. Luego se
/// resta el desplazamiento fijo declarado: si `local = UTC_ancla + offset(t)`
/// y el reloj real es `local = UTC_real + offset(t) + fixed_offset`, entonces
/// `UTC_real = UTC_ancla - fixed_offset`. Con `fixed_offset_minutes == 0`
/// esta resta es un no-op -- de ahí la equivalencia exacta con el mecanismo
/// de zona plana (criterio de aceptación #3 de STORY-070).
///
/// Ambigüedad de "hora repetida" (cambio de horario de otoño, una misma
/// hora local ocurre dos veces): se resuelve tomando siempre la PRIMERA
/// ocurrencia (el offset de horario de verano, antes del retraso de
/// reloj) -- elección determinista y documentada, no una adivinanza.
pub fn resolve_event_timestamp_ns(
    date: NaiveDate,
    time: NaiveTime,
    source_clock: Option<SourceClockDeclaration<'_>>,
) -> Result<i64, TimestampResolutionError> {
    let clock = source_clock.ok_or(TimestampResolutionError::MissingSourceTimezone)?;
    let tz: Tz = Tz::from_str(clock.anchor_timezone_ref)
        .map_err(|_| TimestampResolutionError::UnknownTimezone(clock.anchor_timezone_ref.to_string()))?;

    let naive_datetime = date.and_time(time);
    use chrono::offset::LocalResult;
    use chrono::TimeZone;
    let local_result = tz.from_local_datetime(&naive_datetime);
    let localized = match local_result {
        LocalResult::Single(dt) => dt,
        // Hora ambigua (cambio de horario de otoño): se toma la primera
        // ocurrencia -- ver doc-comment de la función.
        LocalResult::Ambiguous(earlier, _later) => earlier,
        LocalResult::None => {
            return Err(TimestampResolutionError::NonexistentLocalTime {
                date,
                time,
                timezone: clock.anchor_timezone_ref.to_string(),
            })
        }
    };

    use chrono::Utc;
    let anchor_utc = localized.with_timezone(&Utc);
    // Resta el desplazamiento fijo declarado -- ver doc-comment de la
    // función para la derivación exacta. Con desplazamiento 0 esta resta no
    // cambia nada (`Duration::minutes(0)` es la identidad de la resta).
    let true_utc = anchor_utc - chrono::Duration::minutes(clock.fixed_offset_minutes);
    Ok(true_utc.timestamp_nanos_opt().expect("un timestamp de un archivo de mercado real cabe en i64 nanosegundos"))
}

/// Deriva `bar_close_timestamp_ns` de `bar_open_timestamp_ns` para una
/// barra de temporalidad fija de 1 minuto (`bar_type = 'TIME'`,
/// `timeframe_value = "1M"`) -- ver ADR-0155 "Convención temporal":
/// `bar_open_timestamp_ns = floor(event_timestamp_ns / interval_ns) *
/// interval_ns`, intervalo semiabierto `[apertura, cierre)`. MT5 ya reporta
/// el timestamp alineado al minuto, así que aquí solo se suma la duración.
pub fn derive_bar_close_timestamp_ns(bar_open_timestamp_ns: i64) -> i64 {
    bar_open_timestamp_ns + ONE_MINUTE_NS
}

// ═══════════════════════════════════════════════════════════════════════
// Gate de Homogeneidad de Procedencia (ADR-0162 "Enmienda 2026-07-26") --
// SOLO el chequeo de fecha de operación del bróker entra en esta Story
// (STORY-070 §4.1 "Lo que NO entra"). El chequeo de fractura de magnitud/
// disponibilidad queda como especificación de `pit-data-validator`, feature
// que no existe todavía en código.
// ═══════════════════════════════════════════════════════════════════════

/// Rechazo del chequeo de fecha de operación del Gate de Homogeneidad de
/// Procedencia -- caso (b) de la enmienda: un tramo declarado no es tráfico
/// genuino del bróker declarado porque antecede su fecha de operación.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProvenanceHomogeneityError {
    #[error(
        "este tramo antecede la fecha de operación declarada de {venue:?} -- el tramo declara \
         empezar en {declared_range_start_ns} ns UTC, pero {venue:?} opera desde \
         {venue_operational_since_ns} ns UTC (ADR-0162 \"Enmienda 2026-07-26\"): declara este \
         tramo bajo un venue/source_provider distinto, o exclúyelo del rango declarado"
    )]
    PrecedesVenueOperationalDate { venue: String, declared_range_start_ns: i64, venue_operational_since_ns: i64 },
}

/// Compara el inicio de un tramo declarado contra la fecha de operación
/// declarada del bróker/venue (`broker_server_clock.venue_operational_since_ns`,
/// hecho externo verificado, inyectado como parámetro -- esta feature nunca
/// consulta esa tabla directamente, ADR-0137). Detectar la fractura NO es
/// inferir su causa (frontera fijada en TASK-069): esta función solo compara
/// dos instantes, nunca decide qué hacer con el tramo rechazado.
///
/// `venue_operational_since_ns` es `None` cuando ese hecho no está declarado
/// -- en ese caso el chequeo NO CORRE y no se asume ninguna fecha (nunca un
/// rechazo por un dato que nadie declaró, ADR-0162 "Enmienda 2026-07-26").
pub fn check_declared_range_against_venue_operational_date(
    venue: &str,
    declared_range_start_ns: i64,
    venue_operational_since_ns: Option<i64>,
) -> Result<(), ProvenanceHomogeneityError> {
    let Some(operational_since) = venue_operational_since_ns else {
        // Sin fecha declarada, el chequeo no corre -- nunca se asume.
        return Ok(());
    };
    if declared_range_start_ns < operational_since {
        return Err(ProvenanceHomogeneityError::PrecedesVenueOperationalDate {
            venue: venue.to_string(),
            declared_range_start_ns,
            venue_operational_since_ns: operational_since,
        });
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Detección de imposibilidad física (ADR-0156 Restricción 6, pit-data-
// validator TTR-001 -- esa feature aún no existe como crate propio; esta
// Story implementa el chequeo puntual sobre la fila que ella misma produce,
// no el resto del Gate de Coherencia de Calendario que sí pertenece a esa
// feature futura).
// ═══════════════════════════════════════════════════════════════════════

/// Una barra con movimiento de precio real (`open != close` o `high !=
/// low`) y `volume_ticks == 0` es matemáticamente imposible bajo la propia
/// definición de `volume_ticks` (cuenta cambios de precio/cotización -- si
/// hubo movimiento, hubo al menos un cambio contado). Verificado real:
/// `datasets/EURUSD_M1_....csv`, 2019-11-01 16:58 y 17:01, coincide con el
/// cambio de horario de otoño de EE. UU.
pub fn is_activity_impossible(open: i64, high: i64, low: i64, close: i64, volume_ticks: i64) -> bool {
    volume_ticks == 0 && (open != close || high != low)
}

// ═══════════════════════════════════════════════════════════════════════
// Fila canónica de barra -- el tipo concreto que esta feature produce.
// `shared::types::Bars` es solo el marcador de puerto (ADR-0137); el
// struct con campos reales vive aquí, en el dominio de la primera feature
// que lo puebla.
// ═══════════════════════════════════════════════════════════════════════

/// Una fila de `Bars` (ADR-0155) ya mapeada desde MT5 al esquema canónico,
/// con la identidad completa de la serie y las dos marcas de auditoría que
/// esta Story introduce (imposibilidad física, unidad de volumen sin
/// declarar). "Guarda el hecho, calcula la interpretación": ningún campo
/// aquí es un promedio, un spread derivado ni ningún indicador -- es mapeo
/// de columna, más una marca cuando el propio hecho es imposible.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalBarRow {
    pub instrument_id: String,
    pub venue: String,
    pub source_provider: String,
    pub bar_open_timestamp_ns: i64,
    pub bar_close_timestamp_ns: i64,
    pub bar_type: &'static str,
    pub timeframe_value: &'static str,
    pub open: i64,
    pub high: i64,
    pub low: i64,
    pub close: i64,
    /// `<TICKVOL>` -- proxy de actividad, NUNCA volumen real (ADR-0155).
    pub volume_ticks: i64,
    /// `<VOL>` crudo, sin interpretar -- se persiste siempre, aunque su
    /// unidad no esté declarada (criterio de aceptación #8).
    pub volume_traded_raw: i128,
    /// `<SPREAD>` crudo de MT5 (puntos enteros, semántica exacta
    /// open/close/promedio sin verificar por bróker -- pregunta 6 resuelta
    /// del barrido del Architect: opcional, no bloqueante). Se persiste tal
    /// cual llega, sin multiplicar por ningún tamaño de punto asumido --
    /// esa conversión exigiría conocer `canonical_tick_size` del
    /// instrumento, un hecho que vive en `instrument-catalog` y que esta
    /// feature no puede consultar sin violar el aislamiento de crates de
    /// ADR-0137. Documentado explícitamente como decisión de esta Story,
    /// no como una casilla ya resuelta por el encargo.
    pub spread_reported_raw: i64,
    /// Unidad en la que está expresado `spread_reported_raw` (criterio de
    /// aceptación #13) -- para la plantilla MT5 M1 siempre
    /// `MT5_SPREAD_REPORTED_UNIT` ("puntos", nunca pips). Viaja CON el
    /// dato: un consumidor futuro no necesita leer el código para saber
    /// contra qué magnitud reconstruir el spread real.
    pub spread_reported_unit: String,
    /// `true` si `is_activity_impossible` detectó la imposibilidad física
    /// para esta fila (criterio de aceptación #9) -- la fila SE PERSISTE
    /// igual (nunca se descarta en silencio), pero marcada.
    pub is_activity_anomaly: bool,
    /// `true` si la corrida que produjo esta fila no tenía declarada la
    /// convención de unidad del volumen real (criterio de aceptación #8).
    pub volume_unit_unverified: bool,
}

/// Todo lo que el mapeo de una fila necesita, que NO viene en la propia
/// línea del archivo -- son hechos de la corrida completa (identidad de la
/// serie, reloj de origen declarado, unidad de volumen declarada), inyectados
/// por quien orquesta el import (nunca leídos por esta feature desde otra
/// tabla).
#[derive(Debug, Clone)]
pub struct RunContext<'a> {
    pub instrument_id: &'a str,
    pub venue: &'a str,
    pub source_provider: &'a str,
    /// Zona ancla IANA del reloj de origen -- `None` cuando el usuario no
    /// declaró ningún reloj (rechaza el mapeo, criterio de aceptación #3).
    pub source_clock_anchor_timezone_ref: Option<&'a str>,
    /// Desplazamiento fijo en minutos frente al ancla (ADR-0162 "Enmienda
    /// 2026-07-26") -- irrelevante cuando el ancla es `None`. `0` reproduce
    /// sin pérdida la declaración plana de zona de hoy.
    pub source_clock_fixed_offset_minutes: i64,
    pub real_volume_unit_declared: Option<&'a str>,
    /// Unidad del spread reportado que la plantilla de plataforma declara
    /// (criterio de aceptación #13). `None` hace fallar el mapeo -- una
    /// plantilla que no declare esto es un defecto de Drasus (LDR-0008),
    /// nunca un dato que este lector deba asumir. Para la plantilla MT5 M1
    /// vigente, el orquestador SIEMPRE la puebla con
    /// `mt5_bar_spread_reported_unit()`; el `None` existe para que una
    /// plantilla futura mal construida, o un error del orquestador, no se
    /// cuele en silencio.
    pub spread_reported_unit_declared: Option<&'a str>,
}

/// Errores al mapear una fila ya parseada al esquema canónico.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CanonicalMappingError {
    #[error(transparent)]
    Timestamp(#[from] TimestampResolutionError),
    #[error(
        "la plantilla de plataforma no declara la unidad del spread reportado -- se rechaza en \
         vez de asumirla (STORY-068 criterio de aceptación #13, LDR-0008)"
    )]
    MissingSpreadUnitDeclaration,
}

/// Combina una fila cruda ya parseada (`Mt5RawBarFields`) con el contexto
/// de la corrida (`RunContext`) para producir la fila canónica completa.
/// Pura: ningún I/O, mismo input siempre produce el mismo output.
pub fn map_to_canonical_bar(raw: &Mt5RawBarFields, ctx: &RunContext<'_>) -> Result<CanonicalBarRow, CanonicalMappingError> {
    let source_clock = ctx.source_clock_anchor_timezone_ref.map(|anchor_timezone_ref| SourceClockDeclaration {
        anchor_timezone_ref,
        fixed_offset_minutes: ctx.source_clock_fixed_offset_minutes,
    });
    let bar_open_timestamp_ns = resolve_event_timestamp_ns(raw.date, raw.time, source_clock)?;
    let bar_close_timestamp_ns = derive_bar_close_timestamp_ns(bar_open_timestamp_ns);
    let is_activity_anomaly = is_activity_impossible(raw.open, raw.high, raw.low, raw.close, raw.volume_ticks);
    let spread_reported_unit =
        ctx.spread_reported_unit_declared.ok_or(CanonicalMappingError::MissingSpreadUnitDeclaration)?.to_string();

    Ok(CanonicalBarRow {
        instrument_id: ctx.instrument_id.to_string(),
        venue: ctx.venue.to_string(),
        source_provider: ctx.source_provider.to_string(),
        bar_open_timestamp_ns,
        bar_close_timestamp_ns,
        bar_type: "TIME",
        timeframe_value: "1M",
        open: raw.open,
        high: raw.high,
        low: raw.low,
        close: raw.close,
        volume_ticks: raw.volume_ticks,
        volume_traded_raw: raw.volume_traded_raw,
        spread_reported_raw: raw.spread_reported_raw,
        spread_reported_unit,
        is_activity_anomaly,
        volume_unit_unverified: ctx.real_volume_unit_declared.is_none(),
    })
}

// ═══════════════════════════════════════════════════════════════════════
// Hash de contenido de la declaración de mapeo (ledger) -- excluye Grupo I
// (id/created_at/audit_chain_hash/event_sequence_id) para evitar
// circularidad, mismo criterio que `hybrid-data-transformer::content_audit_hash`.
// ═══════════════════════════════════════════════════════════════════════

/// Los campos de CONTENIDO (no de Grupo I) de una declaración de mapeo,
/// sobre los que se calcula `audit_hash`. Cualquier cambio en cualquiera de
/// estos campos debe cambiar el hash resultante -- es lo que la prueba de
/// no-autorreferencia de STORY-051 exige (nunca comparar `f(x)` consigo
/// misma, sino verificar que entradas distintas producen salidas distintas).
#[derive(Debug, Clone)]
pub struct MappingDeclarationHashInput<'a> {
    pub source_file_name: &'a str,
    pub source_file_hash: &'a str,
    pub instrument_id: &'a str,
    pub venue: &'a str,
    /// Rango declarado del tramo (ADR-0162 "Enmienda 2026-07-26") -- dos
    /// tramos distintos del mismo archivo deben sellar hashes distintos, así
    /// que el rango es CONTENIDO, no Grupo I.
    pub declared_range_start_ns: i64,
    pub declared_range_end_ns: i64,
    pub mapping_origin: &'a str,
    pub platform_template_code: Option<&'a str>,
    pub platform_template_version: Option<&'a str>,
    pub column_mapping_json: &'a str,
    pub transform_engine_version: &'a str,
    pub canonical_schema_version: &'a str,
    pub accepted_at_ns: i64,
    pub preview_transformed_rows_json: &'a str,
    pub source_clock_anchor_timezone_ref: &'a str,
    pub source_clock_fixed_offset_minutes: i64,
    pub real_volume_unit_declared: Option<&'a str>,
    /// Unidad del spread reportado declarada por la plantilla (criterio de
    /// aceptación #13) -- se sella siempre presente porque una importación
    /// sin esta declaración se rechaza antes de llegar al ledger.
    pub spread_reported_unit_declared: &'a str,
}

/// Calcula el `audit_hash` (SHA-256, hex) del contenido de una declaración
/// de mapeo. Determinista: concatena los campos en un orden fijo separados
/// por un byte que no puede aparecer dentro de ningún campo individual
/// (`\0`), para que dos declaraciones con campos distintos nunca colisionen
/// por concatenación ambigua (ej. `("ab","c")` vs `("a","bc")`).
pub fn compute_mapping_declaration_audit_hash(input: &MappingDeclarationHashInput<'_>) -> String {
    let mut hasher = Sha256::new();
    let mut push = |s: &str| {
        hasher.update(s.as_bytes());
        hasher.update([0u8]);
    };
    push(input.source_file_name);
    push(input.source_file_hash);
    push(input.instrument_id);
    push(input.venue);
    push(&input.declared_range_start_ns.to_string());
    push(&input.declared_range_end_ns.to_string());
    push(input.mapping_origin);
    push(input.platform_template_code.unwrap_or(""));
    push(input.platform_template_version.unwrap_or(""));
    push(input.column_mapping_json);
    push(input.transform_engine_version);
    push(input.canonical_schema_version);
    push(&input.accepted_at_ns.to_string());
    push(input.preview_transformed_rows_json);
    push(input.source_clock_anchor_timezone_ref);
    push(&input.source_clock_fixed_offset_minutes.to_string());
    push(input.real_volume_unit_declared.unwrap_or(""));
    push(input.spread_reported_unit_declared);
    format!("{:x}", hasher.finalize())
}

/// Calcula el hash de identidad del archivo origen (`source_file_hash`) a
/// partir de sus bytes crudos. Pura respecto al contenido dado -- la
/// lectura del archivo en disco es responsabilidad de la Cáscara.
pub fn compute_source_file_hash(file_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(file_bytes);
    format!("{:x}", hasher.finalize())
}

// ═══════════════════════════════════════════════════════════════════════
// Pruebas unitarias (Capa 1 de la pirámide ADR-0133) -- lógica pura.
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Criterio de aceptación #1: parseo con \r\n intacto ──────────────

    /// Oráculo: una línea real del archivo, con su \r\n de Windows intacto,
    /// se parsea completa y el último campo (`<SPREAD>`) sale como entero.
    /// Línea real: `datasets/EURUSD_M1_2000-01-01_2025-05-01.csv`, fila 2.
    #[test]
    fn parse_mt5_bar_line_oracle_with_windows_line_ending_parses_spread_as_integer() {
        let line = "2000.01.03\t00:01:00\t1.00730\t1.00750\t1.00730\t1.00730\t5\t0\t50\r\n";
        let parsed = parse_mt5_bar_line(line).expect("la línea real del archivo debe parsear sin error");
        assert_eq!(parsed.spread_reported_raw, 50, "el SPREAD debe salir como entero, nunca contaminado por \\r");
        assert_eq!(parsed.volume_ticks, 5);
        assert_eq!(parsed.volume_traded_raw, 0);
        assert_eq!(parsed.date, NaiveDate::from_ymd_opt(2000, 1, 3).unwrap());
        assert_eq!(parsed.time, NaiveTime::from_hms_opt(0, 1, 0).unwrap());
    }

    /// Demuestra que sin el `strip_line_ending` el `\r` contaminaría el
    /// parseo del último campo -- ejercita el camino negativo directamente
    /// (sin `strip_line_ending`, `"50\r".parse::<i64>()` falla).
    #[test]
    fn spread_field_with_raw_carriage_return_fails_to_parse_as_integer_directly() {
        assert!(
            "50\r".parse::<i64>().is_err(),
            "un \\r pegado al campo SPREAD debe romper el parseo directo -- por eso strip_line_ending es obligatorio"
        );
    }

    // ── Criterio de aceptación #2: ida y vuelta de precio sin pérdida ───

    /// Ida y vuelta sobre valores reales del archivo, incluidos los
    /// extremos observados (un valor bajo real, 0.82310, y el máximo 1.60380 del
    /// archivo de 25 años).
    #[test]
    fn price_scale_round_trip_preserves_real_observed_values_including_extremes() {
        for raw in ["1.00730", "1.11488", "0.82310", "1.60380", "1.11670"] {
            let scaled = parse_price_to_scaled("<OPEN>", raw).expect("precio real debe parsear");
            let back = scaled_to_price_string(scaled, 5);
            assert_eq!(back, raw, "la ida y vuelta debe reconstruir exactamente el valor original: {raw}");
        }
    }

    /// Más de 8 decimales se rechaza en vez de truncar en silencio.
    #[test]
    fn price_with_excess_precision_is_rejected_not_truncated() {
        let err = parse_price_to_scaled("<OPEN>", "1.123456789").unwrap_err();
        assert!(matches!(err, FieldParseError::ExcessPricePrecision { .. }));
    }

    // ── Criterio de aceptación #3 (STORY-068): zona horaria declarada, obligatoria ──

    /// Con reloj declarado (desplazamiento 0 -- zona plana), el instante
    /// exacto se calcula usando las reglas de DST correctas de esa zona (no
    /// un offset fijo inventado).
    #[test]
    fn resolve_timestamp_with_declared_timezone_produces_exact_utc_instant() {
        // 2000-01-03 00:01:00 en America/New_York (UTC-5 en enero, sin DST)
        // debe caer en 2000-01-03 05:01:00 UTC.
        let date = NaiveDate::from_ymd_opt(2000, 1, 3).unwrap();
        let time = NaiveTime::from_hms_opt(0, 1, 0).unwrap();
        let clock = SourceClockDeclaration { anchor_timezone_ref: "America/New_York", fixed_offset_minutes: 0 };
        let ns = resolve_event_timestamp_ns(date, time, Some(clock)).expect("con reloj declarado debe resolver sin error");

        use chrono::{TimeZone, Utc};
        let expected = Utc.with_ymd_and_hms(2000, 1, 3, 5, 1, 0).unwrap();
        assert_eq!(ns, expected.timestamp_nanos_opt().unwrap());
    }

    /// Sin reloj declarado, la importación falla con un error tipado en vez
    /// de asumir UTC -- este es el comportamiento que protege contra la
    /// clase de error que ADR-0162 Restricción 7 prohíbe.
    #[test]
    fn resolve_timestamp_without_declared_timezone_fails_instead_of_assuming_utc() {
        let date = NaiveDate::from_ymd_opt(2000, 1, 3).unwrap();
        let time = NaiveTime::from_hms_opt(0, 1, 0).unwrap();
        let err = resolve_event_timestamp_ns(date, time, None).unwrap_err();
        assert_eq!(err, TimestampResolutionError::MissingSourceTimezone);
    }

    // ── Criterio de aceptación #1 (STORY-070): reloj de IC Markets, dos estaciones ──

    /// Oráculo de invierno: IC Markets declara GMT+2 en invierno. Con ancla
    /// "America/New_York" (EST=UTC-5 en enero) + desplazamiento 420 min
    /// (7h), el offset efectivo es -5h+7h=+2h -- coincide con el GMT+2 real
    /// del bróker. 2000-01-03 00:01:00 hora de IC Markets (invierno) debe
    /// caer en 2000-01-02 22:01:00 UTC (local menos 2h).
    #[test]
    fn ic_markets_winter_anchor_plus_offset_resolves_exact_gmt_plus_2_instant() {
        let date = NaiveDate::from_ymd_opt(2000, 1, 3).unwrap();
        let time = NaiveTime::from_hms_opt(0, 1, 0).unwrap();
        let clock = SourceClockDeclaration { anchor_timezone_ref: "America/New_York", fixed_offset_minutes: 420 };
        let ns = resolve_event_timestamp_ns(date, time, Some(clock)).expect("debe resolver sin error");

        use chrono::{TimeZone, Utc};
        let expected = Utc.with_ymd_and_hms(2000, 1, 2, 22, 1, 0).unwrap();
        assert_eq!(ns, expected.timestamp_nanos_opt().unwrap(), "invierno: local 00:01 GMT+2 == 22:01 UTC del día anterior");
    }

    /// Oráculo de verano: IC Markets declara GMT+3 en verano. Con la misma
    /// ancla+desplazamiento (America/New_York en EDT=UTC-4 en julio,
    /// -4h+7h=+3h), 2024-07-15 12:00:00 hora de IC Markets debe caer en
    /// 2024-07-15 09:00:00 UTC (local menos 3h) -- desfase DISTINTO al de
    /// invierno, demostrando que la misma declaración resuelve ambas
    /// estaciones sin código nuevo por estación.
    #[test]
    fn ic_markets_summer_anchor_plus_offset_resolves_exact_gmt_plus_3_instant_distinct_from_winter() {
        let date = NaiveDate::from_ymd_opt(2024, 7, 15).unwrap();
        let time = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        let clock = SourceClockDeclaration { anchor_timezone_ref: "America/New_York", fixed_offset_minutes: 420 };
        let ns = resolve_event_timestamp_ns(date, time, Some(clock)).expect("debe resolver sin error");

        use chrono::{TimeZone, Utc};
        let expected = Utc.with_ymd_and_hms(2024, 7, 15, 9, 0, 0).unwrap();
        assert_eq!(
            ns,
            expected.timestamp_nanos_opt().unwrap(),
            "verano: local 12:00 GMT+3 == 09:00 UTC, desfase de 3h, distinto de las 2h de invierno"
        );
    }

    // ── Criterio de aceptación #2 (STORY-070): cambio de estación en la fecha del ancla, no la europea ──

    /// 2019-03-15 cae DESPUÉS del segundo domingo de marzo de EE. UU.
    /// (2019-03-10, ya en horario de verano de EE. UU.) pero ANTES del
    /// último domingo de marzo de Europa (2019-03-31, todavía en horario de
    /// invierno europeo) -- la ventana exacta de discrepancia entre ambos
    /// calendarios. Con ancla "America/New_York", este día YA debe resolver
    /// con el offset de VERANO del ancla (EDT=UTC-4) -- si el mecanismo
    /// usara por error un ancla europea (todavía invierno ese día), el
    /// resultado sería distinto.
    #[test]
    fn seasonal_change_follows_the_anchor_calendar_not_the_european_one() {
        let date = NaiveDate::from_ymd_opt(2019, 3, 15).unwrap();
        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let clock = SourceClockDeclaration { anchor_timezone_ref: "America/New_York", fixed_offset_minutes: 0 };
        let ns = resolve_event_timestamp_ns(date, time, Some(clock)).expect("debe resolver sin error");

        use chrono::{TimeZone, Utc};
        // EDT (verano de EE. UU.) = UTC-4 -- si el ancla fuera europea (CET,
        // todavía invierno ese día = UTC+1), el resultado sería
        // 2019-03-15 09:00:00 UTC, no el que se afirma aquí.
        let expected_us_summer = Utc.with_ymd_and_hms(2019, 3, 15, 14, 0, 0).unwrap();
        assert_eq!(
            ns,
            expected_us_summer.timestamp_nanos_opt().unwrap(),
            "el 15 de marzo de 2019 debe resolver con horario de VERANO de EE. UU. (offset -4h), no con el invierno europeo"
        );
    }

    // ── Criterio de aceptación #3 (STORY-070): equivalencia con la declaración plana anterior ──

    /// Desplazamiento cero debe producir EXACTAMENTE el mismo instante que
    /// la implementación anterior (zona plana) devolvía para la misma fecha
    /// real -- el valor esperado es el mismo golden que
    /// `resolve_timestamp_with_declared_timezone_produces_exact_utc_instant`
    /// ya fijaba antes de esta Story (2000-01-03 05:01:00 UTC), calculado de
    /// forma independiente con `chrono::Utc` (no reutilizando la función bajo
    /// prueba para producir su propio oráculo).
    #[test]
    fn zero_offset_declaration_is_exactly_equivalent_to_the_previous_flat_zone_mechanism() {
        let date = NaiveDate::from_ymd_opt(2000, 1, 3).unwrap();
        let time = NaiveTime::from_hms_opt(0, 1, 0).unwrap();
        let clock = SourceClockDeclaration { anchor_timezone_ref: "America/New_York", fixed_offset_minutes: 0 };
        let ns = resolve_event_timestamp_ns(date, time, Some(clock)).unwrap();

        use chrono::{TimeZone, Utc};
        let value_the_old_flat_zone_mechanism_produced =
            Utc.with_ymd_and_hms(2000, 1, 3, 5, 1, 0).unwrap().timestamp_nanos_opt().unwrap();
        assert_eq!(
            ns, value_the_old_flat_zone_mechanism_produced,
            "desplazamiento 0 debe reproducir EXACTO el valor que la declaración de zona plana anterior producía"
        );
    }

    // ── Criterio de aceptación #7 (STORY-070): fecha de operación del bróker ──

    /// Con fecha de operación declarada, un tramo que la antecede se
    /// rechaza con motivo explícito.
    #[test]
    fn declared_range_before_venue_operational_date_is_rejected_with_explicit_reason() {
        let venue_operational_since_ns = 1_000_000_000_000; // fecha de fundación declarada
        let declared_range_start_ns = 500_000_000_000; // antecede la fundación
        let err = check_declared_range_against_venue_operational_date(
            "IC_MARKETS",
            declared_range_start_ns,
            Some(venue_operational_since_ns),
        )
        .unwrap_err();
        assert_eq!(
            err,
            ProvenanceHomogeneityError::PrecedesVenueOperationalDate {
                venue: "IC_MARKETS".to_string(),
                declared_range_start_ns,
                venue_operational_since_ns,
            }
        );
    }

    /// Sin fecha de operación declarada, el chequeo NO corre -- un tramo
    /// arbitrariamente antiguo se acepta, nunca se asume una fecha que nadie
    /// declaró.
    #[test]
    fn declared_range_without_venue_operational_date_is_accepted_without_assuming_any_date() {
        let result = check_declared_range_against_venue_operational_date("IC_MARKETS", 0, None);
        assert!(result.is_ok(), "sin fecha de operación declarada, el chequeo no debe rechazar nada");
    }

    /// Contraprueba: un tramo que empieza EN o DESPUÉS de la fecha de
    /// operación declarada se acepta -- discrimina el borde exacto (mata el
    /// mutante `<` → `<=`).
    #[test]
    fn declared_range_starting_exactly_at_venue_operational_date_is_accepted() {
        let result = check_declared_range_against_venue_operational_date("IC_MARKETS", 1_000, Some(1_000));
        assert!(result.is_ok(), "un tramo que empieza EXACTAMENTE en la fecha de operación no antecede nada");
    }

    // ── Criterio de aceptación #4: plantilla no reconocida ──────────────

    #[test]
    fn detect_mt5_bar_template_rejects_unrecognized_header_with_explicit_reason() {
        let err = detect_mt5_bar_template("<TIME>\t<OPEN>\t<CLOSE>\r\n").unwrap_err();
        assert!(matches!(err, TemplateDetectionError::UnrecognizedHeader { .. }));
    }

    #[test]
    fn detect_mt5_bar_template_accepts_the_real_header_with_windows_line_ending() {
        let header = "<DATE>\t<TIME>\t<OPEN>\t<HIGH>\t<LOW>\t<CLOSE>\t<TICKVOL>\t<VOL>\t<SPREAD>\r\n";
        assert!(detect_mt5_bar_template(header).is_ok());
    }

    // ── Criterio de aceptación #9: imposibilidad física ─────────────────

    /// Las dos filas reales del archivo (2019-11-01, líneas 7187218 y
    /// 7187221) que exhiben la imposibilidad: `TICKVOL=0` con rango de
    /// precio real.
    #[test]
    fn real_2019_11_01_rows_with_tickvol_zero_and_real_price_range_are_flagged_as_impossible() {
        // 16:58:00  open=1.11488 high=1.11496 low=1.11485 close=1.11486 tickvol=0
        let row_1 = parse_mt5_bar_line("2019.11.01\t16:58:00\t1.11488\t1.11496\t1.11485\t1.11486\t0\t0\t0\r\n")
            .expect("fila real debe parsear");
        assert!(is_activity_impossible(row_1.open, row_1.high, row_1.low, row_1.close, row_1.volume_ticks));

        // 17:01:00  open=1.11670 high=1.11686 low=1.11637 close=1.11650 tickvol=0
        let row_2 = parse_mt5_bar_line("2019.11.01\t17:01:00\t1.11670\t1.11686\t1.11637\t1.11650\t0\t0\t0\r\n")
            .expect("fila real debe parsear");
        assert!(is_activity_impossible(row_2.open, row_2.high, row_2.low, row_2.close, row_2.volume_ticks));
    }

    /// Contraprueba: una barra normal (movimiento real con `volume_ticks >
    /// 0`, o `volume_ticks == 0` sin ningún movimiento) NUNCA se marca --
    /// discrimina el caso positivo del negativo.
    #[test]
    fn ordinary_bar_with_activity_or_flat_bar_without_activity_is_never_flagged() {
        // Movimiento real, pero con actividad contada -- no es imposible.
        assert!(!is_activity_impossible(100, 110, 90, 105, 3));
        // Sin movimiento y sin actividad -- perfectamente posible (vela plana en silencio).
        assert!(!is_activity_impossible(100, 100, 100, 100, 0));
    }

    /// `map_to_canonical_bar` debe propagar la marca hasta la fila final.
    #[test]
    fn map_to_canonical_bar_propagates_activity_anomaly_flag() {
        let raw = parse_mt5_bar_line("2019.11.01\t16:58:00\t1.11488\t1.11496\t1.11485\t1.11486\t0\t0\t0\r\n").unwrap();
        let ctx = RunContext {
            instrument_id: "EURUSD",
            venue: "PEPPERSTONE",
            source_provider: "MT5_TERMINAL_EXPORT",
            source_clock_anchor_timezone_ref: Some("America/New_York"),
            source_clock_fixed_offset_minutes: 0,
            real_volume_unit_declared: None,
            spread_reported_unit_declared: Some(MT5_SPREAD_REPORTED_UNIT),
        };
        let bar = map_to_canonical_bar(&raw, &ctx).expect("con zona declarada debe mapear sin error");
        assert!(bar.is_activity_anomaly, "la fila imposible del 2019-11-01 debe quedar marcada");
    }

    // ── Criterio de aceptación #8: unidad de volumen sin declarar ───────

    #[test]
    fn map_to_canonical_bar_persists_raw_volume_and_marks_unverified_when_unit_not_declared() {
        let raw = parse_mt5_bar_line("2010.06.03\t00:00:00\t1.20000\t1.20010\t1.19990\t1.20005\t12\t153\t2\r\n").unwrap();
        let ctx = RunContext {
            instrument_id: "EURUSD",
            venue: "PEPPERSTONE",
            source_provider: "MT5_TERMINAL_EXPORT",
            source_clock_anchor_timezone_ref: Some("America/New_York"),
            source_clock_fixed_offset_minutes: 0,
            real_volume_unit_declared: None,
            spread_reported_unit_declared: Some(MT5_SPREAD_REPORTED_UNIT),
        };
        let bar = map_to_canonical_bar(&raw, &ctx).unwrap();
        assert_eq!(bar.volume_traded_raw, 153, "el valor crudo se persiste intacto aunque la unidad no esté declarada");
        assert!(bar.volume_unit_unverified, "sin unidad declarada la corrida debe quedar marcada como no verificada");
    }

    #[test]
    fn map_to_canonical_bar_marks_volume_as_verified_when_unit_is_declared() {
        let raw = parse_mt5_bar_line("2010.06.03\t00:00:00\t1.20000\t1.20010\t1.19990\t1.20005\t12\t153\t2\r\n").unwrap();
        let ctx = RunContext {
            instrument_id: "EURUSD",
            venue: "PEPPERSTONE",
            source_provider: "MT5_TERMINAL_EXPORT",
            source_clock_anchor_timezone_ref: Some("America/New_York"),
            source_clock_fixed_offset_minutes: 0,
            real_volume_unit_declared: Some("CONTRACTS"),
            spread_reported_unit_declared: Some(MT5_SPREAD_REPORTED_UNIT),
        };
        let bar = map_to_canonical_bar(&raw, &ctx).unwrap();
        assert!(!bar.volume_unit_unverified, "con unidad declarada la corrida NO debe marcarse como no verificada");
    }

    // ── Hash de la declaración de mapeo: no autorreferencial ────────────

    /// Dos declaraciones con contenido DISTINTO (construidas por separado,
    /// nunca la misma variable a ambos lados) deben producir hashes
    /// distintos -- prueba la propiedad real (la salida depende de la
    /// entrada), no la tautología `f(x) == f(x)` que STORY-051 enseñó a
    /// evitar.
    #[test]
    fn mapping_declaration_hash_differs_for_different_content() {
        let base = MappingDeclarationHashInput {
            source_file_name: "EURUSD_M1.csv",
            source_file_hash: "abc123",
            instrument_id: "EURUSD",
            venue: "PEPPERSTONE",
            declared_range_start_ns: 0,
            declared_range_end_ns: 1_000_000_000,
            mapping_origin: "PLATFORM_TEMPLATE",
            platform_template_code: Some(MT5_BAR_TEMPLATE_CODE),
            platform_template_version: Some(MT5_BAR_TEMPLATE_VERSION),
            column_mapping_json: "{}",
            transform_engine_version: "0.1.0",
            canonical_schema_version: "ADR-0155-v1",
            accepted_at_ns: 1_000,
            preview_transformed_rows_json: "[]",
            source_clock_anchor_timezone_ref: "America/New_York",
            source_clock_fixed_offset_minutes: 0,
            real_volume_unit_declared: None,
            spread_reported_unit_declared: MT5_SPREAD_REPORTED_UNIT,
        };
        let hash_a = compute_mapping_declaration_audit_hash(&base);

        // Construida por separado, con UN solo campo distinto (el hash del
        // archivo origen) -- debe cambiar el resultado.
        let mut different = base.clone_with_different_file_hash("def456");
        let hash_b = compute_mapping_declaration_audit_hash(&different);
        assert_ne!(hash_a, hash_b, "cambiar source_file_hash debe cambiar el audit_hash resultante");

        // Y una reconstrucción con los MISMOS valores (por separado, no la
        // misma variable) debe reproducir el mismo hash -- determinismo.
        different.source_file_hash = "abc123";
        let hash_c = compute_mapping_declaration_audit_hash(&different);
        assert_eq!(hash_a, hash_c, "el mismo contenido reconstruido por separado debe producir el mismo hash");
    }

    /// Dos declaraciones con el MISMO contenido salvo el rango declarado
    /// deben producir hashes distintos -- el rango es CONTENIDO (dos tramos
    /// del mismo archivo son declaraciones legítimamente distintas,
    /// ADR-0162 "Enmienda 2026-07-26"), nunca circunstancia ignorada por el
    /// hash de auditoría.
    #[test]
    fn mapping_declaration_hash_differs_for_different_declared_range() {
        let tramo_uno = MappingDeclarationHashInput {
            source_file_name: "EURUSD_M1.csv",
            source_file_hash: "abc123",
            instrument_id: "EURUSD",
            venue: "PEPPERSTONE",
            declared_range_start_ns: 0,
            declared_range_end_ns: 1_000_000_000,
            mapping_origin: "PLATFORM_TEMPLATE",
            platform_template_code: Some(MT5_BAR_TEMPLATE_CODE),
            platform_template_version: Some(MT5_BAR_TEMPLATE_VERSION),
            column_mapping_json: "{}",
            transform_engine_version: "0.1.0",
            canonical_schema_version: "ADR-0155-v1",
            accepted_at_ns: 1_000,
            preview_transformed_rows_json: "[]",
            source_clock_anchor_timezone_ref: "America/New_York",
            source_clock_fixed_offset_minutes: 0,
            real_volume_unit_declared: None,
            spread_reported_unit_declared: MT5_SPREAD_REPORTED_UNIT,
        };
        // Tramo dos: construido por separado, MISMO archivo, rango distinto.
        let tramo_dos = MappingDeclarationHashInput {
            source_file_name: "EURUSD_M1.csv",
            source_file_hash: "abc123",
            instrument_id: "EURUSD",
            venue: "PEPPERSTONE",
            declared_range_start_ns: 1_000_000_000,
            declared_range_end_ns: 2_000_000_000,
            mapping_origin: "PLATFORM_TEMPLATE",
            platform_template_code: Some(MT5_BAR_TEMPLATE_CODE),
            platform_template_version: Some(MT5_BAR_TEMPLATE_VERSION),
            column_mapping_json: "{}",
            transform_engine_version: "0.1.0",
            canonical_schema_version: "ADR-0155-v1",
            accepted_at_ns: 1_000,
            preview_transformed_rows_json: "[]",
            source_clock_anchor_timezone_ref: "America/New_York",
            source_clock_fixed_offset_minutes: 0,
            real_volume_unit_declared: None,
            spread_reported_unit_declared: MT5_SPREAD_REPORTED_UNIT,
        };
        assert_ne!(
            compute_mapping_declaration_audit_hash(&tramo_uno),
            compute_mapping_declaration_audit_hash(&tramo_dos),
            "dos tramos distintos del mismo archivo deben sellar hashes de contenido distintos"
        );
    }

    impl<'a> MappingDeclarationHashInput<'a> {
        /// Helper de prueba: clona con un `source_file_hash` distinto, para
        /// construir la segunda entrada "por separado" sin reutilizar la
        /// misma variable en ambos lados de la aserción.
        fn clone_with_different_file_hash(&self, new_hash: &'a str) -> Self {
            Self { source_file_hash: new_hash, ..self.clone() }
        }
    }

    // ── Gate de mutación, Grupo 1: la plantilla de mapeo verificada ──────

    /// Afirma las NUEVE correspondencias exactas columna→campo canónico --
    /// sin esta prueba, `mt5_bar_column_mapping` podía devolver `vec![]` o
    /// pares basura (`"xyzzy"→"xyzzy"`) sin que nada fallara. La plantilla
    /// la afirma Drasus (LDR-0008): si está mal construida, respondemos
    /// nosotros, así que su contenido exacto debe estar bajo prueba.
    #[test]
    fn mt5_bar_column_mapping_declares_exactly_the_nine_canonical_correspondences() {
        let mapping = mt5_bar_column_mapping();
        let expected = vec![
            ("<DATE>".to_string(), "bar_date_source".to_string()),
            ("<TIME>".to_string(), "bar_time_source".to_string()),
            ("<OPEN>".to_string(), "open".to_string()),
            ("<HIGH>".to_string(), "high".to_string()),
            ("<LOW>".to_string(), "low".to_string()),
            ("<CLOSE>".to_string(), "close".to_string()),
            ("<TICKVOL>".to_string(), "volume_ticks".to_string()),
            ("<VOL>".to_string(), "volume_traded".to_string()),
            ("<SPREAD>".to_string(), "spread_reported".to_string()),
        ];
        assert_eq!(mapping, expected, "la plantilla debe declarar exactamente estas nueve correspondencias, en este orden");
    }

    // ── Gate de mutación, Grupo 2: convertidor de precio a entero escalado ──

    /// Mata `domain.rs:141` (`delete -` en el manejo del signo): un precio
    /// negativo debe producir un escalado negativo exacto. También mata
    /// `domain.rs:176` (`*` → `+` en `sign * (...)`- ver razonamiento en el
    /// doc-comment de la prueba de ida y vuelta): con signo negativo,
    /// `sign * X` y `sign + X` divergen radicalmente (`-X` vs. `X - 1`), a
    /// diferencia del caso positivo donde `1 * X` y `1 + X` solo difieren
    /// en 1 unidad -- una diferencia que la prueba de ida y vuelta (con
    /// solo 5 decimales de precisión mostrados) no alcanzaba a ver.
    #[test]
    fn parse_price_to_scaled_produces_exact_signed_value_for_round_numbers() {
        assert_eq!(parse_price_to_scaled("<OPEN>", "1.00000000").unwrap(), 100_000_000, "un entero positivo exacto debe escalar exactamente ×10⁸");
        assert_eq!(parse_price_to_scaled("<OPEN>", "-1.23000000").unwrap(), -123_000_000, "el signo negativo debe propagarse EXACTO, no aproximado ni perdido");
    }

    /// Mata `domain.rs:149` (`||` → `&&` en la validación de la parte
    /// entera): un `+` explícito en la parte entera falla la comprobación
    /// manual de "todo dígito ASCII" (B=true) pero no la comprobación de
    /// vacío (A=false) -- bajo `||` esto rechaza (correcto, MT5 nunca
    /// emite un `+`); bajo `&&` (A=false Y B=true → falso) el chequeo se
    /// salta, y como `i64::parse` SÍ acepta un `+` líder, el mutante
    /// aceptaría en silencio un campo malformado.
    #[test]
    fn price_with_explicit_plus_sign_in_integer_part_is_rejected() {
        let err = parse_price_to_scaled("<OPEN>", "+1.00000000").unwrap_err();
        assert!(matches!(err, FieldParseError::InvalidPrice { .. }), "un '+' explícito en la parte entera nunca es un campo válido de MT5");
    }

    /// Mata `domain.rs:157` (`>` → `>=` en el conteo de decimales): el
    /// borde exacto (8 decimales, el máximo permitido) debe ACEPTARSE.
    #[test]
    fn price_with_exactly_eight_decimals_is_accepted_at_the_boundary() {
        let scaled = parse_price_to_scaled("<OPEN>", "1.12345678").expect("exactamente 8 decimales es el máximo permitido, no debe rechazarse");
        assert_eq!(scaled, 112_345_678);
    }

    // ── Gate de mutación, Grupo 3: formateador inverso ───────────────────

    /// Mata `domain.rs:184` (`<` → `==` y `<` → `<=` en `scaled < 0`): el
    /// borde exacto `scaled == 0` NUNCA debe llevar signo negativo. Bajo
    /// `==` o `<=`, `0 == 0`/`0 <= 0` son `true` -- el mutante produciría
    /// `"-0.00000"` en vez de `"0.00000"`.
    #[test]
    fn scaled_to_price_string_of_zero_has_no_negative_sign() {
        assert_eq!(scaled_to_price_string(0, 5), "0.00000", "cero no lleva signo negativo, ni con < ni con <= ni con ==");
    }

    // ── Gate de mutación, Grupo 4: instante de cierre de la barra ────────

    /// Mata los cuatro mutantes de `domain.rs:360` (constantes `0`/`1`/`-1`
    /// y `+` → `-`): ninguna prueba afirmaba el valor exacto del cierre.
    /// Dos entradas distintas descartan que el mutante "constante" pase
    /// por coincidencia con una sola entrada.
    #[test]
    fn derive_bar_close_timestamp_ns_adds_exactly_one_minute() {
        assert_eq!(derive_bar_close_timestamp_ns(0), 60_000_000_000, "la apertura en 0 debe cerrar exactamente 1 minuto después");
        assert_eq!(derive_bar_close_timestamp_ns(1_000), 60_000_001_000, "sumar (nunca restar) la duración fija de la barra");
    }

    // ── Gate de mutación, Grupo 5: detección de anomalía de actividad ───

    /// Mata `domain.rs:378` (`||` → `&&`): las dos filas reales de
    /// 2019-11-01 difieren en AMBAS condiciones a la vez (apertura≠cierre
    /// Y máximo≠mínimo), así que pasan igual con `&&`. Estos dos casos
    /// construidos difieren en UNA SOLA condición cada uno.
    #[test]
    fn is_activity_impossible_detects_either_condition_alone_not_only_both_together() {
        // Solo abre≠cierra; máximo==mínimo. Con `&&`: true && false = false (mutante falla en detectar).
        assert!(is_activity_impossible(100, 100, 100, 105, 0), "abre≠cierra por sí solo, con volume_ticks=0, ya es imposible");
        // Solo máximo≠mínimo; abre==cierra. Con `&&`: false && true = false (mutante falla en detectar).
        assert!(is_activity_impossible(100, 110, 90, 100, 0), "máximo≠mínimo por sí solo, con volume_ticks=0, ya es imposible");
    }

    // ── Gate de mutación, Grupo 6: hash del archivo origen ───────────────

    /// Mata los dos mutantes de `domain.rs:542` (cadena vacía / cadena
    /// constante "xyzzy"): sin esta prueba, `compute_source_file_hash`
    /// podía devolver siempre la misma constante y la prueba de
    /// reimportación idéntica seguiría en verde -- pero dos archivos
    /// DISTINTOS se tratarían como el mismo, y la evidencia dejaría de
    /// identificar nada. No autorreferencial: los dos contenidos se
    /// construyen por separado, nunca la misma variable a ambos lados.
    #[test]
    fn compute_source_file_hash_differs_for_different_content_and_matches_for_identical_content_built_separately() {
        let content_a = b"contenido real del archivo A, distinto del B".to_vec();
        let content_b = b"contenido real del archivo B, distinto del A".to_vec();
        let hash_a = compute_source_file_hash(&content_a);
        let hash_b = compute_source_file_hash(&content_b);
        assert_ne!(hash_a, hash_b, "contenidos distintos deben producir hashes distintos");
        assert!(!hash_a.is_empty(), "el hash nunca debe ser una cadena vacía");

        // Reconstruye el MISMO contenido de `content_a` con bytes literales
        // NUEVOS (no la misma variable) -- el hash debe repetirse exacto.
        let content_a_rebuilt = b"contenido real del archivo A, distinto del B".to_vec();
        let hash_a_rebuilt = compute_source_file_hash(&content_a_rebuilt);
        assert_eq!(hash_a, hash_a_rebuilt, "el mismo contenido reconstruido por separado debe producir el mismo hash");
    }

    // ── Criterio de aceptación #13: unidad del spread reportado ──────────

    #[test]
    fn mt5_bar_spread_reported_unit_is_points_not_pips() {
        assert_eq!(mt5_bar_spread_reported_unit(), "POINTS_MT5", "MT5 reporta <SPREAD> en puntos, nunca en pips");
    }

    /// La fila canónica debe llevar la unidad tal como la declaró la
    /// plantilla -- el consumidor aguas abajo no necesita leer código para
    /// saber contra qué magnitud reconstruir el spread real.
    #[test]
    fn map_to_canonical_bar_seals_the_declared_spread_reported_unit() {
        let raw = parse_mt5_bar_line("2010.06.03\t00:00:00\t1.20000\t1.20010\t1.19990\t1.20005\t12\t153\t2\r\n").unwrap();
        let ctx = RunContext {
            instrument_id: "EURUSD",
            venue: "PEPPERSTONE",
            source_provider: "MT5_TERMINAL_EXPORT",
            source_clock_anchor_timezone_ref: Some("America/New_York"),
            source_clock_fixed_offset_minutes: 0,
            real_volume_unit_declared: None,
            spread_reported_unit_declared: Some(MT5_SPREAD_REPORTED_UNIT),
        };
        let bar = map_to_canonical_bar(&raw, &ctx).unwrap();
        assert_eq!(bar.spread_reported_unit, "POINTS_MT5");
    }

    /// Prueba discriminante del criterio #13: una plantilla que no declare
    /// la unidad del spread se rechaza en vez de asumirla -- mismo trato
    /// que la zona horaria ausente (criterio #3).
    #[test]
    fn map_to_canonical_bar_rejects_when_spread_unit_is_not_declared() {
        let raw = parse_mt5_bar_line("2010.06.03\t00:00:00\t1.20000\t1.20010\t1.19990\t1.20005\t12\t153\t2\r\n").unwrap();
        let ctx = RunContext {
            instrument_id: "EURUSD",
            venue: "PEPPERSTONE",
            source_provider: "MT5_TERMINAL_EXPORT",
            source_clock_anchor_timezone_ref: Some("America/New_York"),
            source_clock_fixed_offset_minutes: 0,
            real_volume_unit_declared: None,
            spread_reported_unit_declared: None,
        };
        let err = map_to_canonical_bar(&raw, &ctx).unwrap_err();
        assert_eq!(err, CanonicalMappingError::MissingSpreadUnitDeclaration);
    }
}
