//! Functional Core del Hybrid Data Transformer — lógica pura, sin I/O.
//!
//! PROHIBIDO: imports de `polars`, `sqlx`, `tokio`, `std::fs` o cualquier
//! dependencia que toque el sistema operativo, la red o cualquier motor de
//! DataFrame. PERMITIDO: tipos de la librería estándar de Rust que sean
//! puramente de datos.
//!
//! Toda función de este módulo es determinista: los mismos argumentos de
//! entrada producen exactamente la misma salida, sin efectos secundarios, sin
//! `f64` en ningún camino de agregación de precio/volumen (ADR-0141).
//!
//! ## Retrofit ADR-0155 (2026-07-18)
//!
//! Esta feature agrega EXCLUSIVAMENTE `TradeTick` (ejecuciones reales) —
//! nunca `QuoteTick` (cotizaciones). El tipo `RawTrade` de este módulo no
//! tiene campos `bid`/`ask`: una cotización no puede representarse con este
//! struct, así que no puede entrar al pipeline de agregación (separación de
//! tipos físicos, no un chequeo en tiempo de ejecución). El volumen se
//! descompone en cuatro magnitudes independientes en [`Bar`]
//! (`volume_traded`/`trade_count`/`volume_buy`/`volume_ticks`) — ninguna se
//! rellena a partir de otra ni se asume por defecto.
//!
//! ## Corrección DEBT-028 (2026-07-18): `Bar` ya no tiene `vwap`
//!
//! `Bar` NO expone un campo de precio ponderado por volumen. Doctrina del
//! propietario: los primitivos de mercado (apertura, máximo, mínimo, cierre,
//! compra, venta, volumen) son materia prima de la INGESTA; cualquier cosa
//! calculada a partir de ellos —VWAP, ATR, RSI, media móvil— es un
//! INDICADOR y pertenece al motor de indicadores (EPIC-2/EPIC-3), no a esta
//! feature. La aritmética ancha que el VWAP hubiera necesitado
//! ([`checked_scaled_product`] + [`round_half_to_even_div_i128`]) SÍ se
//! conserva aquí, con sus pruebas: es correcta y el motor de indicadores la
//! reutilizará. Detalle completo en `docs/debt/DEBT-028.md`.

use std::collections::BTreeMap;

// ── Tipos de datos de dominio (puras estructuras de datos) ──────────────────

/// Quién cruzó el spread en una ejecución — mismo vocabulario que
/// `TradeTick.aggressor_side` de ADR-0155 (`CHECK IN ('BUY','SELL','UNKNOWN')`).
///
/// La diferencia entre "la fuente no distingue el lado" y "la fuente
/// distingue el lado pero no pudo clasificar esta operación" importa para el
/// cálculo de `volume_buy` (restricción 4 de ADR-0155): un tick SIN este
/// campo (`aggressor_side: None` en [`RawTrade`]) es distinto de un tick CON
/// `Some(AggressorSide::Unknown)` — el primero significa "la fuente ni
/// siquiera tiene este concepto"; el segundo significa "la fuente lo tiene,
/// pero no pudo determinarlo para esta operación puntual".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggressorSide {
    Buy,
    Sell,
    Unknown,
}

/// Una ejecución cruda ya conformada al esquema canónico de `TradeTick`
/// (ADR-0155 + ADR-0141): precio y volumen como enteros escalados ×10⁸,
/// tiempo en nanosegundos Unix UTC.
///
/// Sin campos `bid`/`ask`: este struct SOLO puede representar una ejecución.
/// Una cotización (`QuoteTick`) no tiene forma de convertirse en un
/// `RawTrade` — es la garantía de tipo que impide que una cotización
/// contribuya jamás a una barra agregada (ADR-0155 restricción 2).
///
/// `origin_seq` es la clave de desempate de origen (invariante 3 de la
/// Orden): dos ejecuciones del mismo `event_timestamp_ns` pero de origen
/// distinto (dos trades reales simultáneos) deben ordenarse de forma estable
/// y reproducible, nunca por el orden de llegada accidental de un pipeline
/// paralelo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawTrade {
    /// Instante del evento de mercado, nanosegundos Unix UTC.
    pub event_timestamp_ns: i64,
    /// Precio de ejecución escalado ×10⁸ (entero exacto, ADR-0141). Campo
    /// `price` de `TradeTick`, obligatorio en ADR-0155.
    pub price_scaled: i64,
    /// Tamaño de ESTA ejecución individual, escalado ×10⁸. Campo
    /// `trade_volume` de `TradeTick`, obligatorio en ADR-0155 -- por eso
    /// `volume_traded`/`trade_count` en [`Bar`] SIEMPRE se pueden calcular al
    /// agregar: la fuente nunca deja de distinguir el tamaño de una ejecución
    /// individual, solo puede dejar de distinguir el LADO (`aggressor_side`).
    pub trade_volume_scaled: i64,
    /// Quién cruzó el spread en esta ejecución. `None` cuando la fuente no
    /// distingue este concepto en absoluto (ni siquiera como `UNKNOWN`) --
    /// ver la nota de [`AggressorSide`]. Alimenta `volume_buy` en [`Bar`].
    pub aggressor_side: Option<AggressorSide>,
    /// Clave de desempate estable cuando dos ejecuciones comparten
    /// `event_timestamp_ns` (ej. posición de origen en el archivo Bulk, o un
    /// contador monótono asignado por la fuente). NO es un timestamp — es
    /// puramente un desempate determinista.
    pub origin_seq: u64,
}

/// Una barra OHLCV ya agregada, esquema canónico ADR-0155 + ADR-0141: todo
/// entero escalado ×10⁸, sin `f64`, volumen en cuatro magnitudes
/// independientes, campos de Futuros/Opciones esquema-listo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bar {
    /// Inicio del intervalo semiabierto `[bar_timestamp_ns, bar_close_timestamp_ns)`.
    pub bar_timestamp_ns: i64,
    /// Fin del intervalo (exclusivo). `bar_timestamp_ns + interval_ns`.
    pub bar_close_timestamp_ns: i64,
    /// Precio del primer tick del intervalo (por orden canónico).
    pub open: i64,
    /// Precio máximo observado en el intervalo.
    pub high: i64,
    /// Precio mínimo observado en el intervalo.
    pub low: i64,
    /// Precio del último tick del intervalo (por orden canónico).
    pub close: i64,

    // ── Volumen en cuatro magnitudes independientes (ADR-0155 restricción 3/4) ──
    // NUNCA se colapsan una en otra. Cada una se puebla solo si la fuente
    // concreta la distingue -- el resto queda `None`, jamás se asume
    // `volume_traded` por defecto.
    /// Volumen real negociado: suma de `trade_volume_scaled` de todas las
    /// ejecuciones del intervalo. SIEMPRE calculable al agregar `TradeTick`
    /// (el campo fuente es obligatorio) -- no es `Option`.
    pub volume_traded: i64,
    /// Conteo de ejecuciones discretas agregadas en la barra. SIEMPRE
    /// calculable al agregar `TradeTick` (es un conteo de filas de entrada,
    /// no depende de que la fuente distinga nada adicional) -- no es `Option`.
    pub trade_count: i64,
    /// Volumen agredido por el lado comprador: suma de `trade_volume_scaled`
    /// de las ejecuciones con `aggressor_side == Buy`. `Some` únicamente si
    /// TODAS las ejecuciones del intervalo traen `aggressor_side` conocido
    /// (aunque sea `Unknown`); si al menos una lo desconoce por completo
    /// (`None`), la barra completa queda `None` -- no se puede afirmar un
    /// volumen comprador parcial como si fuera el total.
    pub volume_buy: Option<i64>,
    /// Conteo de cambios de precio/cotización -- proxy de actividad, NO
    /// volumen real (ADR-0155). Esta feature agrega EXCLUSIVAMENTE
    /// `TradeTick` con volumen real conocido; nunca fabrica este proxy desde
    /// ejecuciones reales -- queda `None` siempre en este camino de
    /// agregación (se puebla solo cuando una fuente entrega la barra ya
    /// agregada con esta magnitud propia, puerto `bars_in`, fuera de esta
    /// Story).
    pub volume_ticks: Option<i64>,
    /// Volumen en la moneda de cotización -- magnitud distinta de
    /// `volume_traded`, no derivable de un `TradeTick` individual (requiere
    /// que la fuente la reporte directamente). `None` siempre en este camino
    /// de agregación.
    pub quote_asset_volume: Option<i64>,
    /// Spread que la fuente adjunta a la barra (MT5 `CopyRates.spread`) --
    /// observación independiente, no `high - low` ni ninguna resta derivada.
    /// No derivable de `TradeTick`; `None` siempre en este camino.
    pub spread_reported: Option<i64>,

    // ── Futuros y Opciones, esquema-listo (ADR-0155) ────────────────────
    // Columnas con costo de diseño cero; ninguna la puebla esta feature
    // porque agrega solo `TradeTick` -- estos valores vienen de una
    // instantánea de mercado (open interest, liquidación, griegas), no de la
    // secuencia de ejecuciones. `None` siempre, listas para cuando exista la
    // fuente de Futuros/Opciones (ADR-0140).
    pub open_interest: Option<i64>,
    pub settlement_price: Option<i64>,
    pub implied_volatility_reported: Option<i64>,
    pub delta_reported: Option<i64>,
    pub gamma_reported: Option<i64>,
    pub theta_reported: Option<i64>,
    pub vega_reported: Option<i64>,
    pub rho_reported: Option<i64>,
    pub theoretical_price_reported: Option<i64>,
}

/// Errores del núcleo de transformación. Ninguno de estos representa un
/// fallo de infraestructura (eso vive en `orchestrator`/`FetchError`-like en
/// `public_interface`) — son rechazos explícitos de una operación numérica
/// que de otro modo perdería precisión o produciría basura silenciosa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransformError {
    /// Un producto o una suma de magnitudes escaladas desbordó incluso el
    /// acumulador ancho (invariante 1 de la Orden: prácticamente
    /// inalcanzable con datos de mercado reales, pero se rechaza explícito
    /// en vez de envolver silenciosamente).
    Overflow,
    /// El intervalo de agregación no es positivo — no define una ventana de
    /// tiempo válida.
    NonPositiveInterval,
    /// Un valor escalado, tras la agregación, excede el límite de
    /// representación entera exacta de `f64` (2⁵³) — no puede cruzar a
    /// `ndarray` sin perder precisión (invariante 5 de la Orden, TTR-002).
    ExceedsFloatExactRange { value: i64 },
    /// El bloque a convertir supera `INTEROP_THRESHOLD` filas — se rechaza la
    /// materialización de la matriz en vez de arriesgar la RAM.
    ExceedsInteropThreshold { rows: usize, threshold: usize },
}

impl std::fmt::Display for TransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransformError::Overflow => write!(f, "desbordamiento aritmético en la agregación escalada"),
            TransformError::NonPositiveInterval => write!(f, "el intervalo de agregación debe ser positivo"),
            TransformError::ExceedsFloatExactRange { value } => write!(
                f,
                "el valor escalado {value} excede 2^53: no es representable de forma exacta en f64"
            ),
            TransformError::ExceedsInteropThreshold { rows, threshold } => write!(
                f,
                "el bloque de {rows} filas excede INTEROP_THRESHOLD ({threshold}); conversión a ndarray rechazada"
            ),
        }
    }
}

impl std::error::Error for TransformError {}

/// Límite de representación entera exacta en `f64` (2⁵³). Por encima de este
/// valor, dos enteros distintos pueden colapsar al mismo `f64` — la
/// conversión deja de ser exacta (ADR-0141, TTR-002).
pub const MAX_EXACT_F64_INTEGER: i64 = 9_007_199_254_740_992; // 2^53

// ── Orden canónico (invariante 3: determinismo bajo paralelismo) ────────────

/// Ordena las ejecuciones por clave canónica `(event_timestamp_ns,
/// origin_seq)` antes de cualquier agrupamiento. Esta es la ÚNICA fuente de
/// orden que el resto del pipeline puede asumir — sin importar cuántos hilos
/// ejecutaron la lectura/columnarización previa, el resultado de esta
/// función es siempre el mismo para el mismo conjunto de ejecuciones.
pub fn sort_trades_canonical(trades: &mut [RawTrade]) {
    trades.sort_by_key(|t| (t.event_timestamp_ns, t.origin_seq));
}

// ── Convención temporal de la barra (anti look-ahead) ────────────────────────

/// Calcula el timestamp de inicio de la barra que contiene a `event_timestamp_ns`
/// para un intervalo de `interval_ns` nanosegundos: `floor(t / interval) * interval`.
///
/// Intervalo semiabierto `[inicio, inicio + interval_ns)` — un evento en el
/// instante exacto del cierre pertenece a la barra SIGUIENTE, nunca a la que
/// cierra en ese instante (evita que una barra "sepa" del evento que la abre).
pub fn floor_bar_timestamp(event_timestamp_ns: i64, interval_ns: i64) -> Result<i64, TransformError> {
    if interval_ns <= 0 {
        return Err(TransformError::NonPositiveInterval);
    }
    // `div_euclid` redondea siempre hacia el infinito negativo (floor real),
    // a diferencia de la división entera de Rust que trunca hacia cero — la
    // distinción importa si algún día se admiten timestamps negativos
    // (eventos pre-epoch), aunque en datos de mercado reales no ocurre.
    Ok(event_timestamp_ns.div_euclid(interval_ns) * interval_ns)
}

// ── Aritmética entera exacta (invariante 1) ──────────────────────────────────

/// Multiplica dos magnitudes ya escaladas ×10⁸ en un acumulador `i128` — el
/// único ancho seguro para `precio_escalado × volumen_escalado` (un precio de
/// ~10⁵ por un volumen de ~10⁷, ambos ×10⁸, desborda `i64` por varios
/// órdenes de magnitud). Devuelve `None` solo si incluso `i128` desborda
/// (inalcanzable con magnitudes de mercado reales, pero la función nunca
/// asume eso — el llamador decide qué hacer con `None`).
pub fn checked_scaled_product(a: i64, b: i64) -> Option<i128> {
    (a as i128).checked_mul(b as i128)
}

/// Divide `numerator / denominator` (ambos en `i128`, resultado reescalado a
/// `i64`) con redondeo **half-to-even** (al par más cercano) — la regla FIJA
/// de la feature para toda división que reintroduce la escala ×10⁸.
/// Redondear siempre hacia arriba acumula sesgo positivo sistemático sobre
/// millones de barras; half-to-even no acumula dirección.
///
/// Devuelve `None` si `denominator == 0` (llamador debe verificarlo antes;
/// esta función no decide qué significa "sin volumen") o si el resultado no
/// cabe en `i64` tras redondear.
pub fn round_half_to_even_div_i128(numerator: i128, denominator: i128) -> Option<i64> {
    if denominator == 0 {
        return None;
    }

    // Aritmética con signo: normaliza a positivo y recuerda el signo final,
    // para que el redondeo half-to-even opere siempre sobre magnitudes
    // positivas (la mitad exacta se detecta comparando restos, no signos).
    let sign = if (numerator < 0) != (denominator < 0) { -1i128 } else { 1i128 };
    let num_abs = numerator.unsigned_abs() as i128;
    let den_abs = denominator.unsigned_abs() as i128;

    let quotient = num_abs / den_abs;
    let remainder = num_abs % den_abs;
    let twice_remainder = remainder.checked_mul(2)?;

    // Decide si redondea hacia arriba: solo si el resto supera la mitad, o
    // si es EXACTAMENTE la mitad y el cociente es impar (half-to-even: en el
    // empate exacto, el resultado par gana).
    let round_up = twice_remainder > den_abs || (twice_remainder == den_abs && quotient % 2 != 0);

    let rounded_abs = if round_up { quotient + 1 } else { quotient };
    let signed = sign * rounded_abs;

    i64::try_from(signed).ok()
}

// ── Puente de interoperabilidad Zero-Copy (invariante 5, TTR-002) ──────────

/// Verifica que un valor escalado sea representable de forma EXACTA como
/// `f64` (por debajo de 2⁵³) antes de cruzar la frontera hacia `ndarray`.
/// Rechaza explícitamente en vez de dejar pasar un valor que colapsaría con
/// su vecino al perder el último bit de precisión.
///
/// Compara en `u64` sin volver a `i64`: `value.unsigned_abs()` de
/// `i64::MIN` es `2^63`, un valor que NO cabe en `i64` -- convertirlo de
/// vuelta con `as i64` le da la vuelta a negativo y la comparación `>
/// MAX_EXACT_F64_INTEGER` pasa por accidente (un negativo nunca es mayor que
/// un positivo), dejando pasar el valor por la razón equivocada. Comparar
/// `u64` contra `u64` no puede desbordar para ningún `i64` de entrada.
pub fn verify_exact_f64_representable(value: i64) -> Result<(), TransformError> {
    if value.unsigned_abs() > MAX_EXACT_F64_INTEGER as u64 {
        return Err(TransformError::ExceedsFloatExactRange { value });
    }
    Ok(())
}

/// Decide si un bloque de `row_count` filas puede materializarse como matriz
/// densa contra `interop_threshold` (parámetro configurable `INTEROP_THRESHOLD`).
/// Núcleo puro: la materialización real (Polars → Arrow → ndarray) vive en la Cáscara.
pub fn decide_matrix_conversion(row_count: usize, interop_threshold: usize) -> Result<(), TransformError> {
    if row_count > interop_threshold {
        return Err(TransformError::ExceedsInteropThreshold {
            rows: row_count,
            threshold: interop_threshold,
        });
    }
    Ok(())
}

// ── Volumen en cuatro magnitudes: suma condicionada a distinción de fuente ──

/// Suma una secuencia de magnitudes opcionales SOLO si TODAS están presentes
/// -- si al menos un elemento es `None` (la fuente no distinguió esa
/// magnitud para ese elemento), el agregado completo es `None`. Es la regla
/// mecánica detrás de ADR-0155 restricción 4: "si el dataset de entrada no
/// aclara cuál trae, los demás quedan vacíos -- nunca se asume por defecto".
///
/// Aritmética comprobada (`checked_add`): un desbordamiento aquí se rechaza
/// igual que en cualquier otro acumulador de esta feature.
fn sum_if_all_known<I: IntoIterator<Item = Option<i64>>>(values: I) -> Result<Option<i64>, TransformError> {
    let mut total: i64 = 0;
    for v in values {
        match v {
            Some(x) => total = total.checked_add(x).ok_or(TransformError::Overflow)?,
            None => return Ok(None),
        }
    }
    Ok(Some(total))
}

// ── Agregación OHLCV (invariantes 1, 2, 3, 4, 6, 8 + ADR-0155) ─────────────

/// Agrega ejecuciones (`TradeTick`) ya ordenadas canónicamente en barras
/// OHLCV de `interval_ns` nanosegundos, con volumen en cuatro magnitudes
/// independientes (ADR-0155).
///
/// Precondición: `trades` debe estar ordenado por
/// [`sort_trades_canonical`] — esta función NO ordena (mantiene la frontera
/// núcleo/cáscara clara: el llamador decide cuándo ordenar; aquí solo se
/// agrupa y calcula).
///
/// Nunca descarta una ejecución: cada una pertenece a exactamente una barra
/// (la que determina [`floor_bar_timestamp`] sobre su `event_timestamp_ns`).
/// El conteo de ejecuciones de entrada frente a las agregadas en las barras
/// de salida solo cambia por la agregación declarada — invariante "sin
/// descarte oculto" (criterio 8).
///
/// Una `QuoteTick` no puede llegar aquí: esta función solo acepta
/// `&[RawTrade]`, y `RawTrade` no tiene forma de representar una cotización
/// (sin campos `bid`/`ask`) — la separación de tipos, no un chequeo en
/// tiempo de ejecución, es lo que garantiza ADR-0155 restricción 2.
pub fn aggregate_trades_to_bars(trades: &[RawTrade], interval_ns: i64) -> Result<Vec<Bar>, TransformError> {
    if interval_ns <= 0 {
        return Err(TransformError::NonPositiveInterval);
    }

    // Agrupa por timestamp de inicio de barra preservando el orden de
    // inserción de las claves (BTreeMap: además las deja ordenadas
    // cronológicamente, que es exactamente el orden de salida que se espera).
    let mut groups: BTreeMap<i64, Vec<RawTrade>> = BTreeMap::new();
    for trade in trades {
        let bar_start = floor_bar_timestamp(trade.event_timestamp_ns, interval_ns)?;
        groups.entry(bar_start).or_default().push(*trade);
    }

    let mut bars = Vec::with_capacity(groups.len());
    for (bar_timestamp_ns, group_trades) in groups {
        // Los ticks dentro del grupo llegan en el mismo orden relativo que
        // en `trades` (BTreeMap preserva el orden de inserción del Vec por
        // clave) — por eso open/close son el primero/último de forma segura
        // SIEMPRE que el llamador haya respetado la precondición de orden.
        let open = group_trades.first().expect("un grupo nunca está vacío: se creó al insertar al menos 1 ejecución").price_scaled;
        let close = group_trades.last().expect("un grupo nunca está vacío: se creó al insertar al menos 1 ejecución").price_scaled;
        let high = group_trades.iter().map(|t| t.price_scaled).max().expect("grupo no vacío");
        let low = group_trades.iter().map(|t| t.price_scaled).min().expect("grupo no vacío");

        // volume_traded: suma comprobada -- SIEMPRE calculable, trade_volume
        // es obligatorio en TradeTick (ADR-0155).
        let mut volume_traded: i64 = 0;
        for t in &group_trades {
            volume_traded = volume_traded.checked_add(t.trade_volume_scaled).ok_or(TransformError::Overflow)?;
        }

        // trade_count: SIEMPRE calculable, es un conteo de filas de entrada.
        let trade_count = group_trades.len() as i64;

        // volume_buy: Some(suma) solo si TODA ejecución del grupo trae
        // aggressor_side conocido (aunque sea Unknown) -- si alguna lo
        // desconoce por completo, la barra entera queda None (restricción 4).
        let buy_contributions = group_trades.iter().map(|t| match t.aggressor_side {
            Some(AggressorSide::Buy) => Some(t.trade_volume_scaled),
            Some(AggressorSide::Sell) | Some(AggressorSide::Unknown) => Some(0),
            None => None,
        });
        let volume_buy = sum_if_all_known(buy_contributions)?;

        bars.push(Bar {
            bar_timestamp_ns,
            bar_close_timestamp_ns: bar_timestamp_ns + interval_ns,
            open,
            high,
            low,
            close,
            volume_traded,
            trade_count,
            volume_buy,
            // Esta feature agrega solo TradeTick -- estas magnitudes no son
            // derivables de una secuencia de ejecuciones reales, quedan
            // esquema-listo (ADR-0155) sin poblar en este camino.
            volume_ticks: None,
            quote_asset_volume: None,
            spread_reported: None,
            open_interest: None,
            settlement_price: None,
            implied_volatility_reported: None,
            delta_reported: None,
            gamma_reported: None,
            theta_reported: None,
            vega_reported: None,
            rho_reported: None,
            theoretical_price_reported: None,
        });
    }

    Ok(bars)
}

/// Reagrega barras ya construidas a un intervalo mayor (barra→barra, ej.
/// 1m→1h). El timestamp resultante se deriva SIEMPRE del calendario
/// (`floor_bar_timestamp` sobre el `bar_timestamp_ns` de cada sub-barra),
/// NUNCA copiando el timestamp de la primera sub-barra del grupo — si la
/// ventana empieza con un hueco (ninguna sub-barra cubre el primer minuto de
/// la hora), copiar desalinearía la etiqueta en silencio.
///
/// Las cuatro magnitudes de volumen se propagan con la misma regla "todas
/// conocidas o `None`" que en la agregación desde `TradeTick` (ADR-0155
/// restricción 4), vía [`sum_if_all_known`]. Los campos de Futuros/Opciones y
/// `spread_reported` no tienen una semántica de "suma" bien definida entre
/// barras (son instantáneas puntuales, no magnitudes acumulables) -- quedan
/// `None` tras la reagregación, esquema-listo sin inventar una convención no
/// fijada por ningún ADR.
pub fn reaggregate_bars(bars: &[Bar], new_interval_ns: i64) -> Result<Vec<Bar>, TransformError> {
    if new_interval_ns <= 0 {
        return Err(TransformError::NonPositiveInterval);
    }

    let mut groups: BTreeMap<i64, Vec<Bar>> = BTreeMap::new();
    for bar in bars {
        // Deriva el timestamp del calendario -- nunca copia el de la
        // sub-barra directamente, aunque coincida en el caso común.
        let bucket = floor_bar_timestamp(bar.bar_timestamp_ns, new_interval_ns)?;
        groups.entry(bucket).or_default().push(*bar);
    }

    let mut result = Vec::with_capacity(groups.len());
    for (bar_timestamp_ns, mut sub_bars) in groups {
        // Ordena las sub-barras del grupo por su propio timestamp para que
        // open/close tomen el primero/último cronológico real, sin depender
        // del orden de iteración de `bars`.
        sub_bars.sort_by_key(|b| b.bar_timestamp_ns);

        let open = sub_bars.first().expect("grupo no vacío").open;
        let close = sub_bars.last().expect("grupo no vacío").close;
        let high = sub_bars.iter().map(|b| b.high).max().expect("grupo no vacío");
        let low = sub_bars.iter().map(|b| b.low).min().expect("grupo no vacío");

        let mut volume_traded: i64 = 0;
        let mut trade_count: i64 = 0;
        for b in &sub_bars {
            volume_traded = volume_traded.checked_add(b.volume_traded).ok_or(TransformError::Overflow)?;
            trade_count = trade_count.checked_add(b.trade_count).ok_or(TransformError::Overflow)?;
        }
        let volume_buy = sum_if_all_known(sub_bars.iter().map(|b| b.volume_buy))?;
        let volume_ticks = sum_if_all_known(sub_bars.iter().map(|b| b.volume_ticks))?;
        let quote_asset_volume = sum_if_all_known(sub_bars.iter().map(|b| b.quote_asset_volume))?;

        result.push(Bar {
            bar_timestamp_ns,
            bar_close_timestamp_ns: bar_timestamp_ns + new_interval_ns,
            open,
            high,
            low,
            close,
            volume_traded,
            trade_count,
            volume_buy,
            volume_ticks,
            quote_asset_volume,
            spread_reported: None,
            open_interest: None,
            settlement_price: None,
            implied_volatility_reported: None,
            delta_reported: None,
            gamma_reported: None,
            theta_reported: None,
            vega_reported: None,
            rho_reported: None,
            theoretical_price_reported: None,
        });
    }

    Ok(result)
}

// ── Hashes reproducibles (logic_hash + audit_hash de contenido) ────────────

/// Plan de transformación canonicalizado (lo que decide QUÉ se transforma,
/// nunca CUÁNTOS hilos lo ejecutan — el número de hilos de Polars no forma
/// parte del hash: dos corridas con distinto `polars_threads` deben producir
/// el mismo `logic_hash`, invariante 3 de la Orden).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformationPlan {
    /// Intervalo de agregación destino, nanosegundos (ej. 3_600_000_000_000 = 1h).
    pub target_interval_ns: i64,
    /// Filas máximas antes de rechazar la conversión a `ndarray`.
    pub interop_threshold: usize,
}

/// Calcula el `logic_hash`: SHA-256 determinista del plan canonicalizado.
/// El mismo plan siempre produce el mismo hash, sin importar el dataset ni
/// el número de hilos con el que se ejecute.
pub fn compute_logic_hash(plan: &TransformationPlan) -> String {
    use sha2::{Digest, Sha256};
    let canonical = format!("{}\u{1F}{}", plan.target_interval_ns, plan.interop_threshold);
    let digest = Sha256::digest(canonical.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Calcula el `audit_hash` del contenido: SHA-256 sobre la representación YA
/// ORDENADA de las barras resultantes (invariante 3 y criterio 5 —
/// reproducible entre corridas, independiente del número de hilos que
/// paralelizaron la lectura/columnarización previa).
///
/// Incluye TODOS los campos de [`Bar`] en la representación canónica: las dos
/// marcas de tiempo del intervalo, OHLC, las cuatro magnitudes de volumen (y
/// su presencia/ausencia `Option`), `spread_reported` y los nueve campos de
/// Futuros/Opciones esquema-listo. Dos barras que difirieran en cualquiera de
/// estos campos deben producir un `audit_hash` distinto, nunca el mismo.
///
/// **Corrección de la auditoría del Tech-Lead (2026-07-18):** la versión
/// anterior de esta función omitía `bar_close_timestamp_ns`, `spread_reported`
/// y los nueve campos de Futuros/Opciones de la representación canónica, pese
/// a que este doc-comment ya afirmaba proteger "el contenido completo, no
/// solo OHLC" — una afirmación falsa hasta esta corrección. Ahora sí es
/// cierta: dos barras que difieran SOLO en uno de esos campos producen hashes
/// distintos (ver la prueba `audit_hash_differs_when_only_a_futures_options_field_differs`).
pub fn compute_bars_audit_hash(bars: &[Bar]) -> String {
    use sha2::{Digest, Sha256};

    // Serializa un `Option<i64>` de forma que "ausente" y "presente con
    // valor 0" nunca colapsen en la misma cadena -- distinción necesaria
    // para que el hash detecte un cambio de "no distinguido" a "distinguido
    // en 0", exactamente el tipo de colapso que ADR-0155 prohíbe.
    fn opt(v: Option<i64>) -> String {
        v.map(|x| format!("S{x}")).unwrap_or_else(|| "N".to_string())
    }

    let mut buf = String::new();
    for b in bars {
        // Construye la lista de campos como `Vec<String>` y la une con el
        // separador de campo -- evita contar a mano cuántos `{}` van en un
        // `format!` con 21 argumentos, fuente típica de errores silenciosos
        // (un campo que se queda fuera del hash sin que nada lo señale).
        let fields = [
            b.bar_timestamp_ns.to_string(),
            b.bar_close_timestamp_ns.to_string(),
            b.open.to_string(),
            b.high.to_string(),
            b.low.to_string(),
            b.close.to_string(),
            b.volume_traded.to_string(),
            b.trade_count.to_string(),
            opt(b.volume_buy),
            opt(b.volume_ticks),
            opt(b.quote_asset_volume),
            opt(b.spread_reported),
            opt(b.open_interest),
            opt(b.settlement_price),
            opt(b.implied_volatility_reported),
            opt(b.delta_reported),
            opt(b.gamma_reported),
            opt(b.theta_reported),
            opt(b.vega_reported),
            opt(b.rho_reported),
            opt(b.theoretical_price_reported),
        ];
        buf.push_str(&fields.join("\u{1F}"));
        buf.push('\u{1E}');
    }
    let digest = Sha256::digest(buf.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

// ── Pruebas unitarias del núcleo puro ───────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Construye un `RawTrade` de prueba con `aggressor_side: None` (fuente
    /// que no distingue el lado) — el caso común de la mayoría de las
    /// pruebas heredadas, que no versan sobre `volume_buy`.
    fn trade(event_timestamp_ns: i64, price_scaled: i64, trade_volume_scaled: i64, origin_seq: u64) -> RawTrade {
        RawTrade { event_timestamp_ns, price_scaled, trade_volume_scaled, aggressor_side: None, origin_seq }
    }

    // ── Criterio 1: agregación exacta, sin f64 ──────────────────────────

    /// Agregación OHLCV sobre precios/volúmenes escalados: los resultados
    /// deben coincidir con un oráculo de aritmética entera de referencia
    /// calculado a mano.
    #[test]
    fn ohlcv_aggregation_matches_integer_oracle() {
        // 3 ejecuciones en la misma barra de 1h (interval_ns pequeño para el test):
        // t=0: price=100_00000000 (100.0), vol=2_00000000 (2.0)
        // t=10: price=110_00000000 (110.0), vol=1_00000000 (1.0)
        // t=20: price=90_00000000 (90.0), vol=3_00000000 (3.0)
        let trades = vec![
            trade(0, 100_00000000, 2_00000000, 0),
            trade(10, 110_00000000, 1_00000000, 1),
            trade(20, 90_00000000, 3_00000000, 2),
        ];
        let interval_ns = 1_000; // barra amplia: las 3 ejecuciones caen en la misma barra
        let bars = aggregate_trades_to_bars(&trades, interval_ns).expect("agregación debe tener éxito");

        assert_eq!(bars.len(), 1, "las 3 ejecuciones deben caer en una sola barra");
        let bar = &bars[0];
        assert_eq!(bar.open, 100_00000000, "open = primera ejecución");
        assert_eq!(bar.close, 90_00000000, "close = última ejecución");
        assert_eq!(bar.high, 110_00000000, "high = máximo");
        assert_eq!(bar.low, 90_00000000, "low = mínimo");
        assert_eq!(bar.volume_traded, 6_00000000, "volume_traded = suma exacta (2+1+3=6)");
        assert_eq!(bar.trade_count, 3, "trade_count = número de ejecuciones agregadas");
    }

    // ── Criterio 2: sin desbordamiento silencioso ───────────────────────

    /// Con precio y volumen en el borde superior de magnitud realista, el
    /// producto escalado desborda `i64` pero NO `i128`: `checked_scaled_product`
    /// debe devolver `Some` con el valor exacto (i64 hubiera producido basura
    /// envuelta silenciosamente).
    #[test]
    fn scaled_product_overflows_i64_but_i128_holds_exact_value() {
        let price_scaled = 100_000 * 100_000_000i64; // 10^13
        let volume_scaled = 10_000_000 * 100_000_000i64; // 10^15

        assert!(
            price_scaled.checked_mul(volume_scaled).is_none(),
            "premisa del test: el producto debe desbordar i64"
        );

        let product = checked_scaled_product(price_scaled, volume_scaled);
        assert!(product.is_some(), "el mismo producto en i128 no debe desbordar");
        assert_eq!(
            product.unwrap(),
            (price_scaled as i128) * (volume_scaled as i128),
            "el valor en i128 debe ser el producto exacto, sin envoltura"
        );
    }

    /// Un cálculo compuesto producto+división (`checked_scaled_product` +
    /// `round_half_to_even_div_i128`) con ese mismo borde de magnitud
    /// realista debe completarse sin desbordar y producir el valor correcto
    /// -- no un error ni un valor envuelto. `Bar` ya no tiene campo `vwap`
    /// (DEBT-028: es un indicador, no un primitivo de ingesta), pero esta
    /// aritmética ancha es exactamente la que el motor de indicadores
    /// reutilizará para calcularlo -- de ahí que se ejercite directamente,
    /// sin pasar por `aggregate_trades_to_bars`.
    #[test]
    fn wide_accumulator_division_succeeds_at_realistic_upper_bound_magnitude() {
        let price_scaled = 100_000 * 100_000_000i64;
        let volume_scaled = 10_000_000 * 100_000_000i64;

        let product = checked_scaled_product(price_scaled, volume_scaled).expect("no debe desbordar con i128");
        let result = round_half_to_even_div_i128(product, volume_scaled as i128).expect("no debe desbordar al redondear");

        // Con una sola magnitud, precio*volumen/volumen = precio exacto --
        // el mismo caso borde de una barra con una única ejecución.
        assert_eq!(result, price_scaled);
    }

    // ── Criterio 3: redondeo half-to-even, al menos 3 empates exactos ──

    #[test]
    fn half_to_even_rounds_down_to_even_quotient() {
        assert_eq!(round_half_to_even_div_i128(5, 2), Some(2));
    }

    #[test]
    fn half_to_even_rounds_up_to_even_quotient() {
        assert_eq!(round_half_to_even_div_i128(7, 2), Some(4));
    }

    #[test]
    fn half_to_even_handles_negative_exact_tie() {
        assert_eq!(round_half_to_even_div_i128(-5, 2), Some(-2));
    }

    #[test]
    fn half_to_even_exact_division_has_no_rounding() {
        assert_eq!(round_half_to_even_div_i128(4, 2), Some(2));
    }

    // ── Criterio 4: determinismo bajo paralelismo (orden canónico) ─────

    #[test]
    fn same_timestamp_different_origin_sorts_deterministically_by_origin_seq() {
        let mut trades_a = vec![trade(100, 1, 1, 5), trade(100, 2, 1, 1)];
        let mut trades_b = vec![trade(100, 2, 1, 1), trade(100, 1, 1, 5)];

        sort_trades_canonical(&mut trades_a);
        sort_trades_canonical(&mut trades_b);

        assert_eq!(trades_a, trades_b, "el orden canónico no debe depender del orden de llegada");
        assert_eq!(trades_a[0].origin_seq, 1, "el desempate por origin_seq debe ser estable");
        assert_eq!(trades_a[1].origin_seq, 5);
    }

    #[test]
    fn aggregation_is_deterministic_regardless_of_input_arrival_order() {
        let base_trades = vec![trade(0, 100, 1, 0), trade(5, 200, 1, 1), trade(5, 150, 1, 2), trade(9, 300, 1, 3)];

        let mut run_a = base_trades.clone();
        sort_trades_canonical(&mut run_a);
        let bars_a = aggregate_trades_to_bars(&run_a, 1_000).unwrap();

        let mut run_b: Vec<RawTrade> = base_trades.iter().rev().copied().collect();
        sort_trades_canonical(&mut run_b);
        let bars_b = aggregate_trades_to_bars(&run_b, 1_000).unwrap();

        assert_eq!(bars_a, bars_b, "el resultado no debe depender del orden de llegada de las ejecuciones");
        assert_eq!(bars_a[0].open, 100);
        assert_eq!(bars_a[0].close, 300);
    }

    // ── Criterio 5: auditoría reproducible ──────────────────────────────

    #[test]
    fn audit_hash_is_reproducible_across_runs() {
        let trades = vec![trade(0, 100, 1, 0), trade(5, 200, 2, 1)];

        let mut run_a = trades.clone();
        sort_trades_canonical(&mut run_a);
        let bars_a = aggregate_trades_to_bars(&run_a, 1_000).unwrap();
        let hash_a = compute_bars_audit_hash(&bars_a);

        let mut run_b = trades;
        sort_trades_canonical(&mut run_b);
        let bars_b = aggregate_trades_to_bars(&run_b, 1_000).unwrap();
        let hash_b = compute_bars_audit_hash(&bars_b);

        assert_eq!(hash_a, hash_b, "el audit_hash debe ser idéntico entre corridas del mismo dataset");

        let plan = TransformationPlan { target_interval_ns: 1_000, interop_threshold: 100_000 };
        assert_eq!(
            compute_logic_hash(&plan),
            compute_logic_hash(&plan),
            "el logic_hash depende solo del plan, no del dataset"
        );
    }

    #[test]
    fn audit_hash_differs_for_different_content() {
        let trades_a = vec![trade(0, 100, 1, 0)];
        let trades_b = vec![trade(0, 999, 1, 0)];

        let bars_a = aggregate_trades_to_bars(&trades_a, 1_000).unwrap();
        let bars_b = aggregate_trades_to_bars(&trades_b, 1_000).unwrap();

        assert_ne!(compute_bars_audit_hash(&bars_a), compute_bars_audit_hash(&bars_b));
    }

    /// Dos barras que difieren SOLO en un campo de Futuros/Opciones (aquí,
    /// `open_interest`) deben producir `audit_hash` distintos. Corrección de
    /// la auditoría del Tech-Lead (2026-07-18): antes de esta corrección, los
    /// nueve campos de Futuros/Opciones (y `bar_close_timestamp_ns` y
    /// `spread_reported`) no entraban en la representación canónica pese a
    /// que el doc-comment de `compute_bars_audit_hash` afirmaba proteger el
    /// contenido completo de la barra.
    #[test]
    fn audit_hash_differs_when_only_a_futures_options_field_differs() {
        let base = Bar {
            bar_timestamp_ns: 0, bar_close_timestamp_ns: 1_000, open: 1, high: 1, low: 1, close: 1,
            volume_traded: 1, trade_count: 1, volume_buy: None, volume_ticks: None,
            quote_asset_volume: None, spread_reported: None,
            open_interest: None, settlement_price: None, implied_volatility_reported: None,
            delta_reported: None, gamma_reported: None, theta_reported: None, vega_reported: None,
            rho_reported: None, theoretical_price_reported: None,
        };
        let with_open_interest = Bar { open_interest: Some(500), ..base };

        assert_ne!(
            compute_bars_audit_hash(&[base]),
            compute_bars_audit_hash(&[with_open_interest]),
            "el hash debe distinguir barras que solo difieren en un campo de Futuros/Opciones"
        );
    }

    // ── Criterio 6: anti look-ahead ──────────────────────────────────────

    #[test]
    fn event_at_exact_boundary_belongs_to_next_bar_not_current() {
        let interval_ns = 100;
        let trades = vec![trade(99, 10, 1, 0), trade(100, 20, 1, 1)];
        let bars = aggregate_trades_to_bars(&trades, interval_ns).unwrap();

        assert_eq!(bars.len(), 2, "deben resultar 2 barras distintas");
        assert_eq!(bars[0].bar_timestamp_ns, 0);
        assert_eq!(bars[0].bar_close_timestamp_ns, 100);
        assert_eq!(bars[0].close, 10, "la barra [0,100) NO debe incluir la ejecución de t=100");
        assert_eq!(bars[1].bar_timestamp_ns, 100);
        assert_eq!(bars[1].open, 20, "la ejecución de t=100 pertenece a la barra [100,200)");
    }

    #[test]
    fn reaggregation_derives_timestamp_from_calendar_not_first_subbar() {
        let sub_bars = vec![
            Bar {
                bar_timestamp_ns: 30, bar_close_timestamp_ns: 40, open: 1, high: 1, low: 1, close: 1,
                volume_traded: 1, trade_count: 1, volume_buy: Some(1), volume_ticks: None,
                quote_asset_volume: None, spread_reported: None,
                open_interest: None, settlement_price: None, implied_volatility_reported: None,
                delta_reported: None, gamma_reported: None, theta_reported: None, vega_reported: None,
                rho_reported: None, theoretical_price_reported: None,
            },
            Bar {
                bar_timestamp_ns: 50, bar_close_timestamp_ns: 60, open: 2, high: 2, low: 2, close: 2,
                volume_traded: 1, trade_count: 1, volume_buy: Some(1), volume_ticks: None,
                quote_asset_volume: None, spread_reported: None,
                open_interest: None, settlement_price: None, implied_volatility_reported: None,
                delta_reported: None, gamma_reported: None, theta_reported: None, vega_reported: None,
                rho_reported: None, theoretical_price_reported: None,
            },
        ];
        let result = reaggregate_bars(&sub_bars, 100).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].bar_timestamp_ns, 0,
            "el timestamp debe derivarse del calendario (floor sobre 100), no copiar el de la sub-barra en t=30"
        );
    }

    // ── Criterio 7: frontera de coma flotante honesta ────────────────────

    #[test]
    fn value_below_2_53_is_accepted_as_exact() {
        assert!(verify_exact_f64_representable(MAX_EXACT_F64_INTEGER).is_ok());
        assert!(verify_exact_f64_representable(1_000_000).is_ok());
    }

    #[test]
    fn value_above_2_53_is_rejected_explicitly() {
        let over_limit = MAX_EXACT_F64_INTEGER + 1;
        let result = verify_exact_f64_representable(over_limit);
        assert_eq!(result, Err(TransformError::ExceedsFloatExactRange { value: over_limit }));
    }

    /// `i64::MIN` es el caso borde que exponía el defecto de la guarda
    /// (`unsigned_abs() as i64` le daba la vuelta a negativo y el valor
    /// pasaba el filtro por la razón equivocada). Con la guarda corregida
    /// (comparación en `u64`, sin volver a `i64`), `i64::MIN` se rechaza
    /// explícitamente -- su magnitud (2⁶³) excede 2⁵³ igual que cualquier
    /// otro valor fuera de rango, sin importar que ese valor concreto sea,
    /// por casualidad, exacto en `f64` (toda potencia de dos lo es).
    #[test]
    fn i64_min_is_rejected_without_wrapping_the_guard() {
        let result = verify_exact_f64_representable(i64::MIN);
        assert_eq!(result, Err(TransformError::ExceedsFloatExactRange { value: i64::MIN }));
    }

    #[test]
    fn conversion_rejected_when_row_count_exceeds_threshold() {
        let result = decide_matrix_conversion(100_001, 100_000);
        assert_eq!(result, Err(TransformError::ExceedsInteropThreshold { rows: 100_001, threshold: 100_000 }));
        assert!(decide_matrix_conversion(100_000, 100_000).is_ok(), "el borde igual al umbral debe aceptarse");
    }

    // ── Criterio 8: sin descarte oculto ──────────────────────────────────

    #[test]
    fn duplicate_timestamp_distinct_content_both_contribute_to_the_bar() {
        let trades = vec![trade(50, 100, 1, 0), trade(50, 200, 1, 1)];
        let bars = aggregate_trades_to_bars(&trades, 1_000).unwrap();

        assert_eq!(bars.len(), 1);
        assert_eq!(bars[0].volume_traded, 2, "ambas ejecuciones del mismo timestamp deben contribuir, ninguna se descarta");
        assert_eq!(bars[0].trade_count, 2, "trade_count también debe reflejar ambas ejecuciones");
    }

    #[test]
    fn output_row_count_changes_only_by_declared_aggregation() {
        let trades = vec![trade(0, 1, 1, 0), trade(100, 2, 1, 1), trade(200, 3, 1, 2)];
        let bars_one = aggregate_trades_to_bars(&trades, 1_000_000).unwrap();
        assert_eq!(bars_one.len(), 1);

        let bars_three = aggregate_trades_to_bars(&trades, 100).unwrap();
        assert_eq!(bars_three.len(), 3, "3 ejecuciones en intervalos distintos deben producir 3 barras, ninguna se pierde");
    }

    // ── Retrofit ADR-0155: cuatro magnitudes de volumen, nunca colapsadas ──

    /// Cuando NINGUNA ejecución del grupo trae `aggressor_side` (la fuente no
    /// distingue el lado comprador en absoluto), `volume_buy` debe quedar
    /// `None` -- NUNCA `Some(0)` ni, mucho menos, igual a `volume_traded`.
    /// Esta es la prueba discriminante de la restricción 4 de ADR-0155:
    /// "prohibido asumir el volumen negociado por defecto".
    #[test]
    fn volume_buy_stays_none_when_source_never_distinguishes_aggressor_side() {
        let trades = vec![trade(0, 100, 5, 0), trade(0, 100, 7, 1)];
        let bars = aggregate_trades_to_bars(&trades, 1_000).unwrap();

        assert_eq!(bars.len(), 1);
        assert_eq!(bars[0].volume_traded, 12, "volume_traded SÍ es siempre calculable (trade_volume es obligatorio)");
        assert_eq!(bars[0].volume_buy, None, "volume_buy debe quedar None -- la fuente nunca distinguió el lado");
        assert_eq!(bars[0].volume_ticks, None, "volume_ticks nunca se fabrica desde TradeTick agregado");
        assert_eq!(bars[0].quote_asset_volume, None, "quote_asset_volume no es derivable de TradeTick individual");
    }

    /// Cuando SÍ se distingue el lado para TODAS las ejecuciones del grupo,
    /// `volume_buy` debe reflejar EXACTAMENTE la suma de las de lado
    /// comprador -- una magnitud distinta e independiente de `volume_traded`
    /// (que suma TODO, sin importar el lado). Prueba que las cuatro
    /// magnitudes no se colapsan entre sí.
    #[test]
    fn volume_buy_reflects_only_buy_side_and_never_equals_volume_traded() {
        let trades = vec![
            RawTrade { event_timestamp_ns: 0, price_scaled: 100, trade_volume_scaled: 5, aggressor_side: Some(AggressorSide::Buy), origin_seq: 0 },
            RawTrade { event_timestamp_ns: 0, price_scaled: 100, trade_volume_scaled: 3, aggressor_side: Some(AggressorSide::Sell), origin_seq: 1 },
            RawTrade { event_timestamp_ns: 0, price_scaled: 100, trade_volume_scaled: 2, aggressor_side: Some(AggressorSide::Unknown), origin_seq: 2 },
        ];
        let bars = aggregate_trades_to_bars(&trades, 1_000).unwrap();

        assert_eq!(bars.len(), 1);
        assert_eq!(bars[0].volume_traded, 10, "volume_traded suma TODAS las ejecuciones (5+3+2)");
        assert_eq!(bars[0].trade_count, 3);
        assert_eq!(bars[0].volume_buy, Some(5), "volume_buy suma SOLO el lado comprador (5), no las 10 totales");
        assert_ne!(
            bars[0].volume_buy.unwrap(),
            bars[0].volume_traded,
            "las dos magnitudes deben ser numéricamente distintas -- prueba directa de que no se colapsan"
        );
    }

    /// Un solo tick sin `aggressor_side` dentro de un grupo donde el resto sí
    /// lo trae basta para invalidar `volume_buy` de TODA la barra -- no se
    /// puede afirmar un comprador parcial como si fuera el total conocido.
    #[test]
    fn single_trade_with_unknown_side_nullifies_volume_buy_for_the_whole_bar() {
        let trades = vec![
            RawTrade { event_timestamp_ns: 0, price_scaled: 100, trade_volume_scaled: 5, aggressor_side: Some(AggressorSide::Buy), origin_seq: 0 },
            RawTrade { event_timestamp_ns: 0, price_scaled: 100, trade_volume_scaled: 3, aggressor_side: None, origin_seq: 1 },
        ];
        let bars = aggregate_trades_to_bars(&trades, 1_000).unwrap();

        assert_eq!(bars[0].volume_buy, None, "una sola ejecución sin lado conocido invalida volume_buy de toda la barra");
        assert_eq!(bars[0].volume_traded, 8, "volume_traded no se ve afectado -- sigue siendo siempre calculable");
    }

    /// La reagregación barra→barra propaga `volume_buy` con la misma regla
    /// "todas conocidas o `None`": si una sub-barra no lo conoce, el
    /// resultado combinado tampoco.
    #[test]
    fn reaggregation_propagates_volume_buy_none_when_any_subbar_lacks_it() {
        let known = Bar {
            bar_timestamp_ns: 0, bar_close_timestamp_ns: 10, open: 1, high: 1, low: 1, close: 1,
            volume_traded: 5, trade_count: 1, volume_buy: Some(5), volume_ticks: None,
            quote_asset_volume: None, spread_reported: None,
            open_interest: None, settlement_price: None, implied_volatility_reported: None,
            delta_reported: None, gamma_reported: None, theta_reported: None, vega_reported: None,
            rho_reported: None, theoretical_price_reported: None,
        };
        let unknown = Bar { volume_buy: None, bar_timestamp_ns: 10, bar_close_timestamp_ns: 20, ..known };

        let result = reaggregate_bars(&[known, unknown], 100).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].volume_buy, None, "una sub-barra sin volume_buy invalida el agregado completo");
        assert_eq!(result[0].volume_traded, 10, "volume_traded sí se propaga siempre (checked_add simple)");
    }

    // ── Retrofit ADR-0155: separación física TradeTick / QuoteTick ────────

    /// `RawTrade` no tiene campos `bid`/`ask`: no existe forma de construir
    /// un valor de este tipo que represente una cotización. Esta prueba deja
    /// constancia ejecutable de la garantía de tipo (una cotización JAMÁS
    /// puede llegar a `aggregate_trades_to_bars`, porque su firma solo acepta
    /// `&[RawTrade]`) construyendo el struct con exactamente los campos de
    /// una ejecución -- si alguien le añadiera `bid`/`ask` a `RawTrade` en el
    /// futuro, este test seguiría compilando pero la intención documentada
    /// aquí serviría de ancla de revisión.
    #[test]
    fn raw_trade_has_no_quote_shaped_fields_only_execution_fields() {
        let t = trade(0, 100, 1, 0);
        // Ejecución válida: price_scaled + trade_volume_scaled, ambos
        // obligatorios -- exactamente el esquema de TradeTick, nunca el de
        // QuoteTick (bid/ask).
        assert_eq!(t.price_scaled, 100);
        assert_eq!(t.trade_volume_scaled, 1);
    }

    // ── Pruebas auxiliares ────────────────────────────────────────────────

    #[test]
    fn floor_bar_timestamp_rejects_non_positive_interval() {
        assert_eq!(floor_bar_timestamp(100, 0), Err(TransformError::NonPositiveInterval));
        assert_eq!(floor_bar_timestamp(100, -5), Err(TransformError::NonPositiveInterval));
    }

    #[test]
    fn floor_bar_timestamp_computes_interval_start() {
        assert_eq!(floor_bar_timestamp(250, 100).unwrap(), 200);
        assert_eq!(floor_bar_timestamp(300, 100).unwrap(), 300); // borde exacto
        assert_eq!(floor_bar_timestamp(0, 100).unwrap(), 0);
    }
}
