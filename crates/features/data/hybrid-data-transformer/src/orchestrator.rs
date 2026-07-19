//! [SHELL] Cáscara imperativa del Hybrid Data Transformer.
//!
//! Aquí vive el código que toca I/O y el motor de DataFrame: construcción y
//! ejecución del `LazyFrame` de Polars (ADR-0105), conversión Zero-Copy a
//! `ndarray` en la frontera de consumo estadístico (ADR-0141 R4), medición de
//! tiempo de proceso y persistencia SQLite.
//!
//! El núcleo puro (agregación OHLCV, redondeo, decisión de umbral) está en
//! `domain.rs` — este módulo solo columnariza, ordena vía Polars, delega el
//! cálculo numéricamente delicado al Núcleo (Polars no expresa de forma
//! segura acumuladores `i128` ni redondeo half-to-even) y persiste.

use std::time::Instant;

use polars::prelude::*;
use sqlx::SqlitePool;

use shared::public_interface::Clock;

use crate::domain::{
    self, aggregate_trades_to_bars, compute_bars_audit_hash, compute_logic_hash, decide_matrix_conversion,
    verify_exact_f64_representable, AggressorSide, BarIdentity, RawTrade, TransformationPlan,
};
use crate::persistence::TransformationRepository;
use crate::public_interface::TransformError;
use crate::schemas::{BarOutput, NewTransformationRecord, TradeTickInput, TransformRequest, TransformResult, TransformerConfig};

/// Nombre del esquema canónico de salida — expuesto en el observable de la
/// SVF y en `VerifyOutput` (ADR-0141 F3: toda escritura Parquet declara su
/// `schema_version`).
///
/// Retrofit ADR-0155 (2026-07-18): se sube de "1.0" a "2.0" -- el esquema de
/// `Bar` cambió de forma incompatible (columna `volume` única reemplazada por
/// cuatro magnitudes independientes + campos de Futuros/Opciones
/// esquema-listo). Un consumidor que asuma "1.0" no debe leer "volume" de una
/// partición "2.0" como si fuera `volume_traded` sin verificar la versión
/// primero (ADR-0141 F3).
///
/// Corrección DEBT-028 (2026-07-18): se sube de "2.0" a "2.1" -- el retiro
/// del campo `vwap` (era un indicador colado en la ingesta, no un primitivo
/// de mercado) también cambia la forma de `Bar` de manera incompatible: un
/// consumidor que lea la columna `vwap` de una partición "2.1" no la
/// encontrará. No sube a "3.0" porque no es una reestructuración de la
/// misma magnitud que el cambio de "1.0"->"2.0" (esa vez cambió la
/// representación entera del volumen; esta vez solo se retira una columna
/// derivada) -- el criterio de versión mayor/menor de este campo queda para
/// cuando exista un consumidor real de `schema_version` que lo necesite.
///
/// STORY-057 (2026-07-18): se sube de "2.1" a "3.0" -- SÍ es una
/// reestructuración de la misma magnitud que "1.0"->"2.0" (ese precedente es
/// el criterio de versión mayor ya escrito arriba, aplicado aquí): seis
/// columnas OBLIGATORIAS nuevas se añaden a la fila física
/// (`instrument_id`, `venue`, `source_provider`, `bar_type`,
/// `timeframe_value`, `data_snapshot_id`, ADR-0155 Restricción 8) Y el
/// contrato de entrada `TransformRequest` rompe de forma incompatible
/// (`data_snapshot_id` deja de ser opcional). Un consumidor que asuma "2.1"
/// no debe leer una partición "3.0" sin verificar la versión primero
/// (ADR-0141 F3).
pub const SCHEMA_VERSION: &str = "3.0";

/// Identificador del paso de transformación dentro del pipeline (Grupo III,
/// ADR-0020) — fijo en esta Story: un único paso de agregación OHLCV.
const TRANSFORMATION_ID: &str = "OHLCV_AGGREGATION";

// ── Función principal de orquestación ────────────────────────────────────────

/// Ejecuta la transformación híbrida: columnariza y ordena las ejecuciones
/// vía Polars, delega la agregación OHLCV exacta al Núcleo, mide la duración
/// real, convierte a `ndarray` si el bloque cabe dentro de
/// `INTEROP_THRESHOLD`, y persiste el registro de auditoría.
pub(crate) async fn transform(
    config: &TransformerConfig,
    request: TransformRequest,
    pool: &SqlitePool,
    clock: &dyn Clock,
) -> Result<TransformResult, TransformError> {
    let start = Instant::now();

    let rows_in = request.ticks.len() as i64;

    // ── Paso 1: columnariza las ejecuciones en un DataFrame de Polars y
    // ordena vía LazyFrame (ADR-0105: "NUNCA se materializan DataFrames
    // intermedios si se puede usar Lazy Evaluation"). Esto es la parte "80"
    // del 80/20: Polars hace el trabajo columnar masivo de ordenar y
    // conformar tipos.
    let sorted_trades = columnarize_and_sort(&request.ticks, config.polars_threads)?;

    // ── Paso 2: agregación OHLCV exacta — Núcleo puro, i128, half-to-even,
    // volumen en cuatro magnitudes (ADR-0155). Polars no puede expresar esto
    // de forma segura (sin acumulador i128 ni redondeo half-to-even nativo),
    // así que la frontera Cáscara/Núcleo cae exactamente aquí: Polars
    // ordena, Rust puro agrega.
    // Identidad de la serie (STORY-057): se copia tal cual del
    // `TransformRequest` -- el Núcleo la materializa por fila en cada `Bar`
    // (ver el doc-comment de `BarIdentity`), nunca la deriva al vuelo.
    let identity = BarIdentity {
        instrument_id: request.instrument_id.clone(),
        venue: request.venue.clone(),
        source_provider: request.source_provider.clone(),
        timeframe_value: request.timeframe_value.clone(),
        data_snapshot_id: request.data_snapshot_id.clone(),
    };
    let bars = aggregate_trades_to_bars(&sorted_trades, request.target_interval_ns, &identity)
        .map_err(|e| TransformError::Domain(e.to_string()))?;

    let rows_out = bars.len() as i64;

    // ── Paso 3: hashes reproducibles (criterios 4 y 5).
    let plan = TransformationPlan {
        target_interval_ns: request.target_interval_ns,
        interop_threshold: config.interop_threshold,
    };
    let logic_hash = compute_logic_hash(&plan);
    let audit_hash = compute_bars_audit_hash(&bars);

    // ── Paso 4: puente Zero-Copy a ndarray (TTR-002), solo si cabe dentro
    // del umbral Y todos los valores son representables de forma exacta.
    let matrix_out = try_build_matrix(&bars, config.interop_threshold);

    let transform_duration_ns = start.elapsed().as_nanos() as i64;

    // ── Paso 5: persiste el registro de auditoría (Perfil A, ADR-0020).
    let repo = TransformationRepository::new(pool, clock);
    let record = repo
        .record(NewTransformationRecord {
            // El ledger `transformation_records` NO cambia de esquema
            // (STORY-057 §3.6): `data_snapshot_id` sigue siendo `Option<String>`
            // ahí -- son metadatos de la CORRIDA, no de la fila de mercado.
            // `TransformRequest.data_snapshot_id` sí es obligatorio ahora
            // (String), así que aquí se envuelve en `Some`.
            data_snapshot_id: Some(request.data_snapshot_id.clone()),
            transformation_id: TRANSFORMATION_ID.to_string(),
            logic_hash: logic_hash.clone(),
            parent_id: request.parent_id.clone(),
            node_id: request.node_id.clone(),
            process_id: request.process_id.clone(),
            session_id: request.session_id.clone(),
            transform_duration_ns,
            rows_in,
            rows_out,
            schema_version: SCHEMA_VERSION.to_string(),
            content_audit_hash: audit_hash.clone(),
        })
        .await
        .map_err(|e| TransformError::Database(e.to_string()))?;

    Ok(TransformResult {
        record_id: record.id,
        rows_in,
        rows_out,
        logic_hash,
        audit_hash,
        transform_duration_ns,
        bars: bars.into_iter().map(BarOutput::from).collect(),
        matrix_out,
    })
}

/// Codifica `Option<AggressorSide>` como texto para la columna Polars:
/// `None` -> `None` (nulo SQL/Arrow real, NO la cadena "UNKNOWN") -- la
/// distinción entre "la fuente no tiene este concepto" (nulo) y "la fuente lo
/// tiene pero no pudo clasificar esta ejecución" (`Some("UNKNOWN")`) es
/// exactamente la que exige ADR-0155 restricción 4 para decidir si
/// `volume_buy` se puede calcular.
fn aggressor_side_to_column_value(side: Option<AggressorSide>) -> Option<String> {
    side.map(|s| match s {
        AggressorSide::Buy => "BUY".to_string(),
        AggressorSide::Sell => "SELL".to_string(),
        AggressorSide::Unknown => "UNKNOWN".to_string(),
    })
}

/// Decodifica la columna de texto de vuelta a `Option<AggressorSide>`.
/// Cualquier valor fuera del vocabulario `CHECK` de ADR-0155 se trata como
/// "no distinguido" (`None`) en vez de hacer pánico -- el Núcleo puro no
/// necesita saber de datos corruptos, solo necesita "sabemos el lado, o no".
fn column_value_to_aggressor_side(value: Option<&str>) -> Option<AggressorSide> {
    match value {
        Some("BUY") => Some(AggressorSide::Buy),
        Some("SELL") => Some(AggressorSide::Sell),
        Some("UNKNOWN") => Some(AggressorSide::Unknown),
        _ => None,
    }
}

/// Construye un `DataFrame` de Polars a partir de las ejecuciones de
/// entrada, aplica el conformado de esquema (columnas `Int64`/`UInt64`,
/// ADR-0141) y ordena vía `LazyFrame` por la clave canónica
/// `(event_timestamp_ns, origin_seq)` — la misma clave que exige el Núcleo,
/// pero ejecutada con el motor columnar multihilo de Polars en vez de un
/// sort secuencial en Rust puro sobre datasets grandes.
///
/// Configura el número de hilos de Polars según `polars_threads` (`None` =
/// automático) ANTES de construir el `LazyFrame` — el resultado del sort no
/// depende del número de hilos (es una comparación total sobre dos columnas
/// enteras), lo que sostiene la invariante de determinismo (criterio 4) sin
/// importar cuántos hilos se usaron para lograrlo.
fn columnarize_and_sort(trades: &[TradeTickInput], polars_threads: Option<usize>) -> Result<Vec<RawTrade>, TransformError> {
    if let Some(threads) = polars_threads {
        // Configura el pool de hilos global de Polars (afecta a esta y a
        // cualquier otra operación Polars del proceso — aceptable en un
        // motor local-first de un solo pipeline activo por vez, ADR-0016).
        // `set_num_threads` de POOL es un no-op silencioso si el pool global
        // ya fue inicializado por una corrida previa dentro del mismo
        // proceso; se documenta como limitación conocida del enfoque
        // (Polars no expone reconfiguración dinámica del pool per-query).
        std::env::set_var("POLARS_MAX_THREADS", threads.to_string());
    }

    if trades.is_empty() {
        return Ok(Vec::new());
    }

    let event_timestamp_ns: Vec<i64> = trades.iter().map(|t| t.event_timestamp_ns).collect();
    let price_scaled: Vec<i64> = trades.iter().map(|t| t.price_scaled).collect();
    let trade_volume_scaled: Vec<i64> = trades.iter().map(|t| t.trade_volume_scaled).collect();
    let aggressor_side: Vec<Option<String>> =
        trades.iter().map(|t| aggressor_side_to_column_value(t.aggressor_side.map(Into::into))).collect();
    let origin_seq: Vec<u64> = trades.iter().map(|t| t.origin_seq).collect();

    let df = df![
        "event_timestamp_ns" => event_timestamp_ns,
        "price_scaled" => price_scaled,
        "trade_volume_scaled" => trade_volume_scaled,
        "aggressor_side" => aggressor_side,
        "origin_seq" => origin_seq,
    ]
    .map_err(|e| TransformError::Polars(e.to_string()))?;

    // Evaluación perezosa (ADR-0105/ restricción FIJA de la feature): se
    // construye el LazyFrame y se ordena; `collect()` es el único punto de
    // materialización. `aggressor_side` no forma parte de la clave de orden
    // -- Polars preserva la correspondencia fila-a-fila de las demás
    // columnas automáticamente al reordenar.
    let sorted = df
        .lazy()
        .sort(
            ["event_timestamp_ns", "origin_seq"],
            SortMultipleOptions::default(),
        )
        .collect()
        .map_err(|e| TransformError::Polars(e.to_string()))?;

    dataframe_to_raw_trades(&sorted)
}

/// Extrae las columnas ya ordenadas por Polars de vuelta a `Vec<RawTrade>`
/// para que el Núcleo puro pueda operar sobre ellas con aritmética `i128`.
fn dataframe_to_raw_trades(df: &DataFrame) -> Result<Vec<RawTrade>, TransformError> {
    let ts = df
        .column("event_timestamp_ns")
        .map_err(|e| TransformError::Polars(e.to_string()))?
        .i64()
        .map_err(|e| TransformError::Polars(e.to_string()))?;
    let price = df
        .column("price_scaled")
        .map_err(|e| TransformError::Polars(e.to_string()))?
        .i64()
        .map_err(|e| TransformError::Polars(e.to_string()))?;
    let trade_volume = df
        .column("trade_volume_scaled")
        .map_err(|e| TransformError::Polars(e.to_string()))?
        .i64()
        .map_err(|e| TransformError::Polars(e.to_string()))?;
    let aggressor_side = df
        .column("aggressor_side")
        .map_err(|e| TransformError::Polars(e.to_string()))?
        .str()
        .map_err(|e| TransformError::Polars(e.to_string()))?;
    let origin = df
        .column("origin_seq")
        .map_err(|e| TransformError::Polars(e.to_string()))?
        .u64()
        .map_err(|e| TransformError::Polars(e.to_string()))?;

    let mut out = Vec::with_capacity(df.height());
    for i in 0..df.height() {
        out.push(RawTrade {
            // Ninguna de las columnas obligatorias admite nulos (se
            // construyeron desde Vec<T> sin huecos) -- el 0 del origin_seq en
            // caso de None es inalcanzable en la práctica; se documenta
            // explícito en vez de silenciarlo con .unwrap_or_default().
            // `aggressor_side` SÍ admite nulo legítimo -- `.get(i)` devuelve
            // `Option<&str>` de forma natural, no hace falta `.expect()`.
            event_timestamp_ns: ts.get(i).expect("columna event_timestamp_ns construida sin nulos"),
            price_scaled: price.get(i).expect("columna price_scaled construida sin nulos"),
            trade_volume_scaled: trade_volume.get(i).expect("columna trade_volume_scaled construida sin nulos"),
            aggressor_side: column_value_to_aggressor_side(aggressor_side.get(i)),
            origin_seq: origin.get(i).expect("columna origin_seq construida sin nulos"),
        });
    }
    Ok(out)
}

/// Intenta construir el bloque columnar denso (`matrix_out`, puerto
/// `ArrowStream`) a partir de las barras resultantes, usando el puente
/// Zero-Copy de Polars (`DataFrame::to_ndarray`, vía Arrow) — TTR-002.
///
/// Columnas: `open, high, low, close, volume_traded` -- `volume_traded` es
/// la única de las cuatro magnitudes SIEMPRE conocida al agregar `TradeTick`
/// (ADR-0155), por eso es la que ocupa el quinto lugar de la matriz densa
/// (mismo shape `[N, 5]` que antes del retrofit).
///
/// Devuelve `None` si el bloque excede `INTEROP_THRESHOLD` o si algún valor
/// no es representable de forma exacta en `f64` (por encima de 2⁵³, criterio
/// 7): el rechazo se decide ANTES de construir el DataFrame, nunca dejando
/// pasar un valor fuera de rango "por accidente" dentro de la conversión de
/// Polars.
fn try_build_matrix(bars: &[domain::Bar], interop_threshold: usize) -> Option<ndarray::Array2<f64>> {
    if decide_matrix_conversion(bars.len(), interop_threshold).is_err() {
        return None;
    }

    for bar in bars {
        for value in [bar.open, bar.high, bar.low, bar.close, bar.volume_traded] {
            if verify_exact_f64_representable(value).is_err() {
                return None;
            }
        }
    }

    if bars.is_empty() {
        return Some(ndarray::Array2::<f64>::zeros((0, 5)));
    }

    // La conversión i64 -> f64 ocurre EXCLUSIVAMENTE aquí, en la Cáscara
    // (ADR-0141 R4) -- cada valor ya pasó la verificación de representación
    // exacta arriba, así que ningún bit de precisión se pierde.
    let open: Vec<f64> = bars.iter().map(|b| b.open as f64).collect();
    let high: Vec<f64> = bars.iter().map(|b| b.high as f64).collect();
    let low: Vec<f64> = bars.iter().map(|b| b.low as f64).collect();
    let close: Vec<f64> = bars.iter().map(|b| b.close as f64).collect();
    let volume_traded: Vec<f64> = bars.iter().map(|b| b.volume_traded as f64).collect();

    let matrix_df = df![
        "open" => open,
        "high" => high,
        "low" => low,
        "close" => close,
        "volume_traded" => volume_traded,
    ]
    .ok()?;

    // Puente Zero-Copy vía Arrow: Polars copia los buffers columnares
    // contiguos directamente al layout denso de `ndarray`, sin pasar por una
    // serialización intermedia fila-por-fila.
    matrix_df.to_ndarray::<Float64Type>(IndexOrder::C).ok()
}

// ── Pruebas unitarias de la Cáscara ─────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// `column_value_to_aggressor_side` es privada -- solo se puede probar
    /// desde dentro de este módulo. El lado agresor alimenta `volume_buy`
    /// (ADR-0155), una de las cuatro magnitudes independientes que no se
    /// pueden colapsar: cada brazo del vocabulario se ejercita con su valor
    /// de texto real, incluido `Some("UNKNOWN")` -- si ese brazo se borrara,
    /// una fuente que SÍ distingue el lado pero no pudo clasificar esta
    /// ejecución en particular pasaría a leerse como "la fuente no distingue
    /// el concepto en absoluto" (`None`), colapsando dos significados
    /// distintos en uno solo, sin que nada lo señale.
    #[test]
    fn column_value_to_aggressor_side_maps_each_vocabulary_value() {
        assert_eq!(column_value_to_aggressor_side(Some("BUY")), Some(AggressorSide::Buy));
        assert_eq!(column_value_to_aggressor_side(Some("SELL")), Some(AggressorSide::Sell));
        assert_eq!(column_value_to_aggressor_side(Some("UNKNOWN")), Some(AggressorSide::Unknown), "'UNKNOWN' es un valor CONOCIDO, distinto de 'la fuente no distingue nada'");
        assert_eq!(column_value_to_aggressor_side(None), None, "columna nula -- la fuente no tiene el concepto");
        assert_eq!(column_value_to_aggressor_side(Some("basura")), None, "un valor fuera del vocabulario CHECK se trata como no distinguido, sin pánico");
    }

    /// Ida y vuelta: `aggressor_side_to_column_value` seguido de
    /// `column_value_to_aggressor_side` debe reproducir el valor original
    /// para los tres lados conocidos y para `None` -- ninguna de las dos
    /// funciones puede fundir un lado con otro al codificar/decodificar.
    #[test]
    fn aggressor_side_round_trips_through_column_encoding_for_every_value() {
        for side in [None, Some(AggressorSide::Buy), Some(AggressorSide::Sell), Some(AggressorSide::Unknown)] {
            let encoded = aggressor_side_to_column_value(side);
            let decoded = column_value_to_aggressor_side(encoded.as_deref());
            assert_eq!(decoded, side, "codificar y decodificar {side:?} debe devolver el mismo valor");
        }
    }
}
