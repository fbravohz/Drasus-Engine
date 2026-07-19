//! Superficie pública del Hybrid Data Transformer — el ÚNICO módulo `pub`.
//!
//! Expone:
//! - Los puertos de salida (`OutputPorts`) con los marcadores de tipo `Bars`
//!   y `ArrowStream` del catálogo ADR-0137.
//! - La función principal `transform()` y el entrypoint de producción `execute()`.
//! - Los tipos de solicitud/respuesta necesarios para llamar a esta feature.
//! - El harness de verificación CLI (`verify`, ADR-0142 Fase 1).
//!
//! Otros crates solo pueden importar desde aquí; prohibido acceder a
//! `domain`, `orchestrator`, `persistence` o `schemas` directamente.

use std::path::{Path, PathBuf};

use polars::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use shared::public_interface::{create_pool, run_migrations, Clock, SystemClock};
use shared::types::{ArrowStream, Bars};

use crate::orchestrator;

// ── Re-exports para que los tests de integración puedan acceder a ellos ─────

pub use crate::domain::{
    aggregate_trades_to_bars, build_series_identity_triplet, checked_scaled_product, compute_bars_audit_hash,
    compute_logic_hash, decide_matrix_conversion, floor_bar_timestamp, reaggregate_bars, round_half_to_even_div_i128,
    sort_trades_canonical, verify_exact_f64_representable, AggressorSide, Bar, BarIdentity, BarType, RawTrade,
    TransformationPlan, MAX_EXACT_F64_INTEGER,
};
pub use crate::persistence::{TransformationRepository, TransformationRepositoryError};
pub use crate::schemas::{
    AggressorSideInput, BarOutput, BarTypeInput, NewTransformationRecord, TradeTickInput, TransformRequest,
    TransformResult, TransformationRecord, TransformerConfig,
};

// ── Tipo de error público ────────────────────────────────────────────────────

/// Errores que puede producir la operación de transformación híbrida.
#[derive(Debug, thiserror::Error)]
pub enum TransformError {
    /// El Núcleo rechazó la operación (desbordamiento, intervalo inválido,
    /// frontera de coma flotante, umbral de interoperabilidad).
    #[error("error de dominio: {0}")]
    Domain(String),
    /// El motor de DataFrame (Polars) falló al columnarizar, ordenar o
    /// convertir.
    #[error("error de Polars: {0}")]
    Polars(String),
    /// Error de persistencia en SQLite.
    #[error("error de base de datos: {0}")]
    Database(String),
    /// Error de I/O en el sistema de archivos local (lectura de Parquet).
    #[error("error de sistema de archivos: {0}")]
    Io(String),
    /// Error interno no esperado.
    #[error("error interno: {0}")]
    Internal(String),
}

// ── Puertos de salida del Canvas (marcadores de tipo, ADR-0137) ─────────────

/// Puertos de salida del Hybrid Data Transformer hacia el Canvas.
pub struct OutputPorts {
    /// Barras transformadas con esquema canónico, listas para normalización
    /// y validación (puerto `bars_out`, cardinalidad 1).
    pub bars_out: Bars,
    /// Bloque columnar Arrow para consumo estadístico (puerto `matrix_out`,
    /// cardinalidad 0..1).
    pub matrix_out: ArrowStream,
}

// ── Función principal ────────────────────────────────────────────────────────

/// Ejecuta la transformación híbrida: columnariza y ordena los ticks vía
/// Polars, agrega OHLCV con aritmética entera exacta en el Núcleo, convierte
/// a `ndarray` en la frontera si corresponde, y persiste el registro de
/// auditoría.
///
/// `pool` debe apuntar a una base de datos ya migrada (incluye migración
/// 0021). `clock` se inyecta — nunca se llama al reloj del sistema
/// directamente para los campos de auditoría (la duración de proceso SÍ usa
/// tiempo real de pared, `std::time::Instant`, por ser una métrica de
/// infraestructura, no un timestamp de negocio).
pub async fn transform(
    config: &TransformerConfig,
    request: TransformRequest,
    pool: &SqlitePool,
    clock: &dyn Clock,
) -> Result<TransformResult, TransformError> {
    orchestrator::transform(config, request, pool, clock).await
}

// ── Lectura desde Parquet (ADR-0105: scan_parquet, Lazy Evaluation) ─────────

/// Lee ejecuciones crudas (`TradeTick`) desde un archivo Parquet con el
/// esquema canónico (`event_timestamp_ns`, `price_scaled`,
/// `trade_volume_scaled`, `origin_seq`, `aggressor_side` opcional) vía
/// `LazyFrame::scan_parquet` — evaluación perezosa, sin materializar el
/// archivo completo hasta el `collect()` final.
///
/// `aggressor_side` es OPCIONAL a nivel de archivo (no solo de fila): si la
/// partición Parquet ni siquiera declara esa columna (fuente que nunca
/// distinguió el lado comprador), se trata como ausente en todas las filas
/// -- consistente con ADR-0155 restricción 4, nunca se inventa un valor.
///
/// El escritor real de Parquet aguas arriba (`sovereign-data-fetcher`,
/// TTR-007 posterior) todavía no existe — esta función queda cableada y
/// probada contra un archivo Parquet propio (round-trip) para el día que
/// exista un `data_snapshot_id` real que resolver a un path físico.
pub async fn load_ticks_from_parquet(path: &Path) -> Result<Vec<TradeTickInput>, TransformError> {
    let pl_path = PlRefPath::try_from_path(path).map_err(|e| TransformError::Polars(e.to_string()))?;
    let lf = LazyFrame::scan_parquet(pl_path, ScanArgsParquet::default())
        .map_err(|e| TransformError::Polars(e.to_string()))?;
    let df = lf.collect().map_err(|e| TransformError::Polars(e.to_string()))?;

    let ts = df.column("event_timestamp_ns").map_err(|e| TransformError::Polars(e.to_string()))?.i64().map_err(|e| TransformError::Polars(e.to_string()))?;
    let price = df.column("price_scaled").map_err(|e| TransformError::Polars(e.to_string()))?.i64().map_err(|e| TransformError::Polars(e.to_string()))?;
    let trade_volume = df.column("trade_volume_scaled").map_err(|e| TransformError::Polars(e.to_string()))?.i64().map_err(|e| TransformError::Polars(e.to_string()))?;
    let origin = df.column("origin_seq").map_err(|e| TransformError::Polars(e.to_string()))?.u64().map_err(|e| TransformError::Polars(e.to_string()))?;
    // La columna es opcional a nivel de ARCHIVO: si no existe, cada fila
    // recibe `None` sin error -- distinto de una columna presente con nulos
    // por fila (que sí se leería y se propagaría fila a fila).
    let aggressor_side_col = df.column("aggressor_side").ok().and_then(|c| c.str().ok());

    let mut out = Vec::with_capacity(df.height());
    for i in 0..df.height() {
        let aggressor_side = aggressor_side_col
            .and_then(|c| c.get(i))
            .and_then(|s| match s {
                "BUY" => Some(AggressorSideInput::Buy),
                "SELL" => Some(AggressorSideInput::Sell),
                "UNKNOWN" => Some(AggressorSideInput::Unknown),
                _ => None,
            });
        out.push(TradeTickInput {
            event_timestamp_ns: ts.get(i).ok_or_else(|| TransformError::Polars("event_timestamp_ns nulo".to_string()))?,
            price_scaled: price.get(i).ok_or_else(|| TransformError::Polars("price_scaled nulo".to_string()))?,
            trade_volume_scaled: trade_volume.get(i).ok_or_else(|| TransformError::Polars("trade_volume_scaled nulo".to_string()))?,
            aggressor_side,
            origin_seq: origin.get(i).ok_or_else(|| TransformError::Polars("origin_seq nulo".to_string()))?,
        });
    }
    Ok(out)
}

// ── Ejecución de producción (entrypoint del Bridge FFI) ─────────────────────

/// Parámetros para una transformación de producción iniciada desde el
/// Bridge FFI o desde el harness CLI.
///
/// Exactamente una de `ticks` / `source_parquet_path` debe traer datos: si
/// ambas están vacías, `execute()` devuelve `TransformError::Internal`. Si
/// ambas traen datos, `ticks` gana (permite pruebas deterministas sin tocar
/// disco incluso cuando `source_parquet_path` está presente por completitud
/// del contrato).
///
/// **STORY-057:** los seis campos de identidad de serie son obligatorios --
/// mismo cambio incompatible que en [`TransformRequest`], del que este
/// struct es el equivalente de producción (Bridge FFI/CLI).
#[derive(Debug, Clone)]
pub struct ExecuteRequest {
    pub instrument_id: String,
    pub venue: String,
    pub source_provider: String,
    pub bar_type: BarTypeInput,
    pub timeframe_value: String,
    pub data_snapshot_id: String,
    pub ticks: Vec<TradeTickInput>,
    pub source_parquet_path: Option<PathBuf>,
    pub target_interval_ns: i64,
    pub interop_threshold: usize,
    pub polars_threads: Option<usize>,
    pub parent_id: Option<String>,
    pub node_id: Option<String>,
    pub process_id: Option<String>,
    pub session_id: Option<String>,
}

/// Ejecuta una transformación de producción. Resuelve el origen de los ticks
/// (directo o Parquet) y delega en [`transform`].
pub async fn execute(
    request: ExecuteRequest,
    pool: &SqlitePool,
    clock: &dyn Clock,
) -> Result<TransformResult, TransformError> {
    let ticks = if !request.ticks.is_empty() {
        request.ticks
    } else if let Some(path) = &request.source_parquet_path {
        load_ticks_from_parquet(path).await?
    } else {
        return Err(TransformError::Internal(
            "execute() requiere 'ticks' o 'source_parquet_path' con al menos un dato".to_string(),
        ));
    };

    let config = TransformerConfig {
        interop_threshold: request.interop_threshold,
        polars_threads: request.polars_threads,
    };
    let transform_request = TransformRequest {
        instrument_id: request.instrument_id,
        venue: request.venue,
        source_provider: request.source_provider,
        bar_type: request.bar_type,
        timeframe_value: request.timeframe_value,
        data_snapshot_id: request.data_snapshot_id,
        ticks,
        target_interval_ns: request.target_interval_ns,
        parent_id: request.parent_id,
        node_id: request.node_id,
        process_id: request.process_id,
        session_id: request.session_id,
    };

    transform(&config, transform_request, pool, clock).await
}

// ── Harness de verificación CLI (ADR-0142 Fase 1) ────────────────────────────

/// Mapea el vocabulario de timeframe de la UI (`data_fetcher.dart`) a
/// nanosegundos. Vocabulario FIJO: `1m`, `5m`, `15m`, `1h`, `4h`, `1d`.
///
/// **Pública a propósito, y debe seguir siéndolo:** el puente FFI necesita
/// esta misma traducción para el camino de la interfaz gráfica. Mientras
/// estuvo privada, el puente se vio obligado a duplicar la tabla — y una
/// tabla de vocabulario duplicada se desincroniza en cuanto alguien añade una
/// temporalidad en un lado y no en el otro. Esta feature es la dueña del
/// vocabulario; nadie más lo reimplementa.
pub fn parse_target_aggregation(value: &str) -> Result<i64, TransformError> {
    const NS_PER_SEC: i64 = 1_000_000_000;
    match value {
        "1m" => Ok(60 * NS_PER_SEC),
        "5m" => Ok(5 * 60 * NS_PER_SEC),
        "15m" => Ok(15 * 60 * NS_PER_SEC),
        "1h" => Ok(3_600 * NS_PER_SEC),
        "4h" => Ok(4 * 3_600 * NS_PER_SEC),
        "1d" => Ok(86_400 * NS_PER_SEC),
        other => Err(TransformError::Internal(format!(
            "target_aggregation desconocido: '{other}' -- se esperaba uno de 1m, 5m, 15m, 1h, 4h, 1d"
        ))),
    }
}

/// Construye una muestra sintética pequeña y determinista de ejecuciones
/// (`TradeTick`) para una verificación de humo sin `--input` explícito. NO se
/// usa para casos de prueba de criterios de aceptación (esos usan datos
/// fabricados nombrados o la muestra congelada de cripto en
/// `tests/fixtures/`) — solo para que `drasus verify hybrid-data-transformer`
/// sin argumentos ya produzca un resultado significativo.
fn default_smoke_ticks() -> Vec<TradeTickInput> {
    vec![
        // Precio/volumen ×10⁸ (ADR-0141): 43 000.00 -> 4_300_000_000_000; 1.5 -> 150_000_000.
        TradeTickInput { event_timestamp_ns: 1_700_000_000_000_000_000, price_scaled: 4_300_000_000_000, trade_volume_scaled: 150_000_000, aggressor_side: None, origin_seq: 0 },
        TradeTickInput { event_timestamp_ns: 1_700_000_030_000_000_000, price_scaled: 4_305_000_000_000, trade_volume_scaled: 200_000_000, aggressor_side: None, origin_seq: 1 },
        TradeTickInput { event_timestamp_ns: 1_700_000_060_000_000_000, price_scaled: 4_298_000_000_000, trade_volume_scaled: 75_000_000, aggressor_side: None, origin_seq: 2 },
    ]
}

/// Input para la verificación del Hybrid Data Transformer vía CLI.
/// Se deserializa desde el JSON que pasa el usuario con `--input '...'`.
///
/// **STORY-057:** los cinco campos de identidad nuevos y `data_snapshot_id`
/// (ahora obligatorio en `TransformRequest`) tienen default de humo -- para
/// que `drasus verify hybrid-data-transformer` sin `--input` siga
/// funcionando sin que el usuario tenga que declarar identidad de mercado
/// real para una verificación de cableado.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyInput {
    /// Símbolo/instrumento. Default: `"SMOKE"`.
    #[serde(default = "default_instrument_id")]
    pub instrument_id: String,
    /// Bróker/venue. Default: `"SMOKE_VENUE"`.
    #[serde(default = "default_venue")]
    pub venue: String,
    /// Proveedor del dato. Default: `"SMOKE_PROVIDER"`.
    #[serde(default = "default_source_provider")]
    pub source_provider: String,
    /// Tipo de barra. Default: `Time` (única variante que esta feature produce).
    #[serde(default = "default_bar_type")]
    pub bar_type: BarTypeInput,
    /// Forma de texto del intervalo. Default: `"1H"` (coherente con
    /// `target_aggregation` default `"1h"`).
    #[serde(default = "default_timeframe_value")]
    pub timeframe_value: String,
    /// Referencia de linaje al dataset crudo (no se resuelve a un path físico
    /// en esta Story -- ver `load_ticks_from_parquet` para el mecanismo real).
    /// Default satisface el `CHECK` de formato de `transformation_records`
    /// (ADR-0141: mínimo 4 segmentos separados por `_`).
    #[serde(default = "default_data_snapshot_id")]
    pub data_snapshot_id: String,
    /// Timeframe destino: `1m`, `5m`, `15m`, `1h`, `4h`, `1d`. Default: `1h`.
    #[serde(default = "default_target_aggregation")]
    pub target_aggregation: String,
    /// Filas máximas antes de rechazar la conversión a `ndarray`. Default: 100 000.
    #[serde(default = "default_interop_threshold")]
    pub interop_threshold: usize,
    /// Hilos de Polars. `None`/ausente = automático.
    #[serde(default)]
    pub polars_threads: Option<usize>,
    /// Ejecuciones (`TradeTick`) explícitas para la verificación. Si se
    /// omite, se usa una muestra sintética pequeña y determinista.
    #[serde(default)]
    pub ticks: Option<Vec<TradeTickInput>>,
}

fn default_instrument_id() -> String {
    "SMOKE".to_string()
}

fn default_venue() -> String {
    "SMOKE_VENUE".to_string()
}

fn default_source_provider() -> String {
    "SMOKE_PROVIDER".to_string()
}

fn default_bar_type() -> BarTypeInput {
    BarTypeInput::Time
}

fn default_timeframe_value() -> String {
    "1H".to_string()
}

fn default_data_snapshot_id() -> String {
    "smoke_symbol_1h_202607".to_string()
}

fn default_target_aggregation() -> String {
    "1h".to_string()
}

fn default_interop_threshold() -> usize {
    100_000
}

impl Default for VerifyInput {
    fn default() -> Self {
        Self {
            instrument_id: default_instrument_id(),
            venue: default_venue(),
            source_provider: default_source_provider(),
            bar_type: default_bar_type(),
            timeframe_value: default_timeframe_value(),
            data_snapshot_id: default_data_snapshot_id(),
            target_aggregation: default_target_aggregation(),
            interop_threshold: default_interop_threshold(),
            polars_threads: None,
            ticks: None,
        }
    }
}

/// Output de la verificación del Hybrid Data Transformer. Siempre serializa
/// a JSON válido (ADR-0142).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOutput {
    pub ok: bool,
    pub record_id: Option<String>,
    pub rows_in: Option<i64>,
    pub rows_out: Option<i64>,
    pub logic_hash: Option<String>,
    pub audit_hash: Option<String>,
    pub transform_duration_ns: Option<i64>,
    /// Columnas del esquema resultante como `"nombre:tipo"` (ej. `"open:Int64"`).
    pub schema: Option<Vec<String>>,
    /// `true` si `matrix_out` se materializó (dentro del umbral y exacto en f64).
    pub matrix_out_present: Option<bool>,
    pub error: Option<String>,
}

impl VerifyOutput {
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            record_id: None,
            rows_in: None,
            rows_out: None,
            logic_hash: None,
            audit_hash: None,
            transform_duration_ns: None,
            schema: None,
            matrix_out_present: None,
            error: Some(msg),
        }
    }

    fn from_result(r: TransformResult) -> Self {
        // Esquema canónico de la barra (ADR-0155 + ADR-0141): mismo orden de
        // columnas que persistiría en Parquet. Seis campos de identidad de
        // serie (STORY-057, texto/`Utf8`) + volumen en cuatro magnitudes
        // independientes + campos de Futuros/Opciones esquema-listo.
        let schema = vec![
            "instrument_id:Utf8".to_string(),
            "venue:Utf8".to_string(),
            "source_provider:Utf8".to_string(),
            "bar_type:Utf8".to_string(),
            "timeframe_value:Utf8".to_string(),
            "data_snapshot_id:Utf8".to_string(),
            "bar_timestamp_ns:Int64".to_string(),
            "bar_close_timestamp_ns:Int64".to_string(),
            "open:Int64".to_string(),
            "high:Int64".to_string(),
            "low:Int64".to_string(),
            "close:Int64".to_string(),
            "volume_traded:Int64".to_string(),
            "trade_count:Int64".to_string(),
            "volume_buy:Int64".to_string(),
            "volume_ticks:Int64".to_string(),
            "quote_asset_volume:Int64".to_string(),
            "spread_reported:Int64".to_string(),
            "open_interest:Int64".to_string(),
            "settlement_price:Int64".to_string(),
            "implied_volatility_reported:Int64".to_string(),
            "delta_reported:Int64".to_string(),
            "gamma_reported:Int64".to_string(),
            "theta_reported:Int64".to_string(),
            "vega_reported:Int64".to_string(),
            "rho_reported:Int64".to_string(),
            "theoretical_price_reported:Int64".to_string(),
        ];
        Self {
            ok: true,
            record_id: Some(r.record_id),
            rows_in: Some(r.rows_in),
            rows_out: Some(r.rows_out),
            logic_hash: Some(r.logic_hash),
            audit_hash: Some(r.audit_hash),
            transform_duration_ns: Some(r.transform_duration_ns),
            schema: Some(schema),
            matrix_out_present: Some(r.matrix_out.is_some()),
            error: None,
        }
    }
}

/// Ejecuta la verificación del Hybrid Data Transformer con adaptadores
/// reales: BD SQLite temporal + reloj de sistema real + motor Polars real.
/// No toca la red -- los ticks vienen del `--input` o de una muestra
/// sintética embebida.
///
/// Uso típico desde el CLI:
/// `drasus verify hybrid-data-transformer --input '{"target_aggregation":"1h"}'`
pub async fn verify(input: VerifyInput) -> VerifyOutput {
    let target_interval_ns = match parse_target_aggregation(&input.target_aggregation) {
        Ok(ns) => ns,
        Err(e) => return VerifyOutput::from_error(e.to_string()),
    };

    let temp_dir = std::env::temp_dir().join(format!("drasus-verify-hybrid-data-transformer-{}", Uuid::new_v4()));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return VerifyOutput::from_error(format!("no se pudo crear el directorio temporal de verificación: {e}"));
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match create_pool(&db_url).await {
        Ok(p) => p,
        Err(e) => return VerifyOutput::from_error(format!("no se pudo crear la BD temporal de verificación: {e}")),
    };
    if let Err(e) = run_migrations(&pool).await {
        return VerifyOutput::from_error(format!("error al aplicar migraciones en la BD temporal: {e}"));
    }

    let clock = SystemClock::default();
    let ticks = input.ticks.unwrap_or_else(default_smoke_ticks);

    let config = TransformerConfig { interop_threshold: input.interop_threshold, polars_threads: input.polars_threads };
    let request = TransformRequest {
        instrument_id: input.instrument_id,
        venue: input.venue,
        source_provider: input.source_provider,
        bar_type: input.bar_type,
        timeframe_value: input.timeframe_value,
        data_snapshot_id: input.data_snapshot_id,
        ticks,
        target_interval_ns,
        parent_id: None,
        node_id: None,
        process_id: Some(format!("drasus-pid-{}", std::process::id())),
        session_id: None,
    };

    match transform(&config, request, &pool, &clock).await {
        Ok(result) => VerifyOutput::from_result(result),
        Err(e) => VerifyOutput::from_error(e.to_string()),
    }
}

// ── Pruebas unitarias de la superficie pública ──────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// PRUEBA DE VALORES DORADOS (hallazgo de mutación): ningún test previo
    /// verificaba el valor exacto en nanosegundos de cada token del
    /// vocabulario -- los operadores `*` de `parse_target_aggregation` podían
    /// mutarse a `/` o `+` sin que nada protestara. Esta tabla determina el
    /// ancho de cada barra; si `5m` produjera un número equivocado, las
    /// barras saldrían agrupadas a una temporalidad distinta de la pedida sin
    /// que ningún dato se corrompiera -- el error es silencioso aguas abajo.
    /// Cada valor esperado está calculado a mano, no derivado de la misma
    /// fórmula que el código bajo prueba (evita la tautología "el código se
    /// compara consigo mismo").
    #[test]
    fn parse_target_aggregation_matches_hand_calculated_nanosecond_values() {
        // 1 minuto = 60 segundos * 1_000_000_000 ns/s = 60_000_000_000 ns.
        assert_eq!(parse_target_aggregation("1m").unwrap(), 60_000_000_000, "1m debe ser exactamente 60 * 10^9 ns");
        // 5 minutos = 300 segundos * 1_000_000_000 ns/s = 300_000_000_000 ns.
        assert_eq!(parse_target_aggregation("5m").unwrap(), 300_000_000_000, "5m debe ser exactamente 300 * 10^9 ns");
        // 15 minutos = 900 segundos * 1_000_000_000 ns/s = 900_000_000_000 ns.
        assert_eq!(parse_target_aggregation("15m").unwrap(), 900_000_000_000, "15m debe ser exactamente 900 * 10^9 ns");
        // 1 hora = 3_600 segundos * 1_000_000_000 ns/s = 3_600_000_000_000 ns.
        assert_eq!(parse_target_aggregation("1h").unwrap(), 3_600_000_000_000, "1h debe ser exactamente 3600 * 10^9 ns");
        // 4 horas = 14_400 segundos * 1_000_000_000 ns/s = 14_400_000_000_000 ns.
        assert_eq!(parse_target_aggregation("4h").unwrap(), 14_400_000_000_000, "4h debe ser exactamente 14400 * 10^9 ns");
        // 1 día = 86_400 segundos * 1_000_000_000 ns/s = 86_400_000_000_000 ns.
        assert_eq!(parse_target_aggregation("1d").unwrap(), 86_400_000_000_000, "1d debe ser exactamente 86400 * 10^9 ns");
    }

    /// Un token fuera del vocabulario fijo (`1m`, `5m`, `15m`, `1h`, `4h`,
    /// `1d`) debe rechazarse explícitamente en vez de aproximarse a alguno de
    /// los válidos -- el puente FFI delega en esta misma función, así que un
    /// token desconocido debe fallar igual por CLI y por interfaz gráfica.
    #[test]
    fn parse_target_aggregation_rejects_unknown_token() {
        let err = parse_target_aggregation("2m").expect_err("un token fuera del vocabulario fijo debe rechazarse");
        assert!(
            matches!(err, TransformError::Internal(_)),
            "el rechazo debe ser un error interno explícito, no un pánico ni una aproximación"
        );
    }

    /// PRUEBA DE VALORES DORADOS (hallazgo de mutación): los `default_*` de
    /// `VerifyInput` son centinelas explícitos del arranque de humo (dicen
    /// literalmente "SMOKE", nunca fabrican identidad de forma silenciosa) --
    /// disposición del Tech-Lead: correctos como diseño, no se cambian. Lo
    /// que faltaba es fijar estos valores con una prueba, para que nadie los
    /// cambie sin darse cuenta de que está cambiando el contrato del arranque
    /// de humo del comando de verificación (`drasus verify
    /// hybrid-data-transformer` sin `--input`).
    #[test]
    fn verify_input_default_values_are_golden_smoke_sentinels() {
        assert_eq!(default_instrument_id(), "SMOKE");
        assert_eq!(default_venue(), "SMOKE_VENUE");
        assert_eq!(default_source_provider(), "SMOKE_PROVIDER");
        assert_eq!(default_timeframe_value(), "1H");
        assert_eq!(default_interop_threshold(), 100_000);

        // Verificación de cableado: `VerifyInput::default()` debe usar
        // EXACTAMENTE estos mismos valores, no una copia divergente.
        let input = VerifyInput::default();
        assert_eq!(input.instrument_id, "SMOKE");
        assert_eq!(input.venue, "SMOKE_VENUE");
        assert_eq!(input.source_provider, "SMOKE_PROVIDER");
        assert_eq!(input.bar_type, BarTypeInput::Time);
        assert_eq!(input.timeframe_value, "1H");
        assert_eq!(input.data_snapshot_id, "smoke_symbol_1h_202607");
        assert_eq!(input.target_aggregation, "1h");
        assert_eq!(input.interop_threshold, 100_000);
        assert!(input.polars_threads.is_none());
        assert!(input.ticks.is_none());
    }

    /// El lado agresor leído de un archivo Parquet real alimenta
    /// `volume_buy` (una de las cuatro magnitudes independientes que
    /// ADR-0155 exige no colapsar). Cada brazo del `match` de texto (`BUY`,
    /// `SELL`, `UNKNOWN`) se ejercita con su valor de entrada real -- si un
    /// brazo se borrara, ese lado conocido pasaría a leerse como `None`
    /// (desconocido) sin que nada lo señale, porque `None` es un valor
    /// legítimo en este campo.
    #[tokio::test(flavor = "multi_thread")]
    async fn load_ticks_from_parquet_maps_each_aggressor_side_text_value() {
        let dir = tempfile::tempdir().expect("crear directorio temporal");
        let path = dir.path().join("aggressor_sides.parquet");

        let mut df = df![
            "event_timestamp_ns" => [1_i64, 2, 3],
            "price_scaled" => [100_i64, 200, 300],
            "trade_volume_scaled" => [10_i64, 20, 30],
            "origin_seq" => [0_u64, 1, 2],
            "aggressor_side" => ["BUY", "SELL", "UNKNOWN"],
        ]
        .expect("construir DataFrame de prueba");

        let file = std::fs::File::create(&path).expect("crear archivo Parquet de prueba");
        ParquetWriter::new(file).finish(&mut df).expect("escribir Parquet de prueba");

        let ticks = load_ticks_from_parquet(&path).await.expect("leer Parquet de prueba");

        assert_eq!(ticks.len(), 3);
        assert_eq!(ticks[0].aggressor_side, Some(AggressorSideInput::Buy), "brazo 'BUY' debe mapear a Buy");
        assert_eq!(ticks[1].aggressor_side, Some(AggressorSideInput::Sell), "brazo 'SELL' debe mapear a Sell");
        assert_eq!(ticks[2].aggressor_side, Some(AggressorSideInput::Unknown), "brazo 'UNKNOWN' debe mapear a Unknown -- distinto de None");
    }
}
