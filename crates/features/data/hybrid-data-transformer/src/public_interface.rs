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
    aggregate_trades_to_bars, checked_scaled_product, compute_bars_audit_hash, compute_logic_hash,
    decide_matrix_conversion, floor_bar_timestamp, reaggregate_bars, round_half_to_even_div_i128,
    sort_trades_canonical, verify_exact_f64_representable, AggressorSide, Bar, RawTrade, TransformationPlan,
    MAX_EXACT_F64_INTEGER,
};
pub use crate::persistence::{TransformationRepository, TransformationRepositoryError};
pub use crate::schemas::{
    AggressorSideInput, BarOutput, NewTransformationRecord, TradeTickInput, TransformRequest, TransformResult,
    TransformationRecord, TransformerConfig,
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
#[derive(Debug, Clone)]
pub struct ExecuteRequest {
    pub data_snapshot_id: Option<String>,
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
fn parse_target_aggregation(value: &str) -> Result<i64, TransformError> {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyInput {
    /// Referencia de linaje al dataset crudo (no se resuelve a un path físico
    /// en esta Story -- ver `load_ticks_from_parquet` para el mecanismo real).
    #[serde(default)]
    pub data_snapshot_id: Option<String>,
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

fn default_target_aggregation() -> String {
    "1h".to_string()
}

fn default_interop_threshold() -> usize {
    100_000
}

impl Default for VerifyInput {
    fn default() -> Self {
        Self {
            data_snapshot_id: None,
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
        // columnas que persistiría en Parquet. Volumen en cuatro magnitudes
        // independientes + campos de Futuros/Opciones esquema-listo.
        let schema = vec![
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
