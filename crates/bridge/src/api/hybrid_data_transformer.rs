//! Funciones FFI para el Hybrid Data Transformer — envío de transformaciones
//! (`TradeTick` → `Bars`) y consulta de datasets de origen / registros de
//! transformación hacia Flutter/Dart (STORY-051 Etapa 3).
//!
//! ## Patrón de diseño — mismo `await` sin sondeo que `data_fetcher.rs`
//!
//! El backend (`hybrid_data_transformer::public_interface::execute()`) es
//! inline bloqueante: columnariza, ordena y agrega vía Polars antes de
//! devolver. Por eso `submit_hybrid_data_transformation` se expone como
//! `async fn` sin `#[frb(sync)]` — flutter_rust_bridge la convierte en
//! `Future<TransformOutcomeDto>` en Dart. El Flutter Engineer usa
//! `await submitHybridDataTransformation(...)` con spinner mientras dura la
//! transformación, igual que `await submitDownloadJob(...)`.
//!
//! ## Por qué este archivo define sus propios DTO en vez de reexportar los
//! de la feature
//!
//! `flutter_rust_bridge` solo genera bindings Dart para tipos definidos (o
//! explícitamente espejados) dentro del árbol de `rust_input` (`crate::api`,
//! ver `flutter_rust_bridge.yaml`). El mismo patrón ya establecido en
//! `data_fetcher.rs` (`DownloadJobResult`, `JobStatusDto`,
//! `DownloadRecordDto`) es el que sigue este archivo: DTO local por tipo que
//! cruza la frontera, mapeado campo a campo desde el struct real de la
//! feature — nunca un tipo Dart escrito a mano por fuera del codegen.
//!
//! ## Garantía central del esquema — cuatro magnitudes de volumen, nunca
//! colapsadas (ADR-0155, STORY-051 criterio 14)
//!
//! `BarDto` espeja `BarOutput` campo a campo, incluyendo la nulabilidad
//! exacta: `volume_traded`/`trade_count` son `i64` (obligatorios, siempre
//! calculables desde `TradeTick`); `volume_buy`, `volume_ticks`,
//! `quote_asset_volume`, `spread_reported` y los nueve campos de
//! Futuros/Opciones son `Option<i64>`. `flutter_rust_bridge` traduce
//! `Option<i64>` de Rust a `int?` (nullable) en Dart — un valor ausente
//! cruza como `null`, nunca como `0`. Las pruebas de este archivo verifican
//! el round-trip real (no solo el tipo declarado).

use hybrid_data_transformer::public_interface::{
    execute as execute_transformation, AggressorSideInput, BarOutput, BarTypeInput,
    ExecuteRequest, TradeTickInput, TransformationRepository,
};
use shared::public_interface::{create_pool, run_migrations, SystemClock};
use sovereign_data_fetcher::public_interface::DownloadRepository;

// ── Tipos FFI-safe ────────────────────────────────────────────────────────
//
// Todos los campos son primitivos (`i64`, `u64`, `u32`) o `String`/`Option<..>`.
// flutter_rust_bridge puede transportarlos sin serialización adicional.

/// Una ejecución cruda (`TradeTick`) tal como la envía Dart. Espejo FFI-safe
/// de `TradeTickInput` (`hybrid_data_transformer::public_interface`).
///
/// | Campo                  | Rust             | Dart      | Nullable en Dart |
/// |-------------------------|------------------|-----------|-----------------|
/// | `event_timestamp_ns`    | `i64`            | `int`     | No              |
/// | `price_scaled`          | `i64`            | `int`     | No              |
/// | `trade_volume_scaled`   | `i64`            | `int`     | No              |
/// | `aggressor_side`        | `Option<String>` | `String?` | Sí              |
/// | `origin_seq`            | `u64`            | `BigInt`/`int` | No         |
pub struct TradeTickDto {
    /// Instante del evento de mercado, nanosegundos Unix UTC.
    pub event_timestamp_ns: i64,
    /// Precio de ejecución escalado ×10⁸ (ADR-0141).
    pub price_scaled: i64,
    /// Tamaño de esta ejecución individual, escalado ×10⁸.
    pub trade_volume_scaled: i64,
    /// `"BUY"` | `"SELL"` | `"UNKNOWN"` | `null`. `null` cuando la fuente NO
    /// distingue el lado en absoluto — alimenta la regla "todas conocidas o
    /// `None`" de `BarDto.volume_buy` (ADR-0155 restricción 4).
    pub aggressor_side: Option<String>,
    /// Clave de desempate estable de origen (orden canónico determinista).
    pub origin_seq: u64,
}

/// Una barra OHLCV de salida. Espejo FFI-safe 1:1 de `BarOutput`
/// (`hybrid_data_transformer::public_interface`, ADR-0155 + STORY-057).
///
/// Propietario de memoria: Rust construye el `Vec<BarDto>`; flutter_rust_bridge
/// lo serializa y lo libera. Dart recibe copias en su propio heap — sin
/// memoria compartida entre los dos lados.
pub struct BarDto {
    /// Símbolo/instrumento (ej. `"BTCUSDT"`).
    pub instrument_id: String,
    /// Bróker/venue donde se opera el instrumento.
    pub venue: String,
    /// Quién entregó el dato (puede diferir de `venue`).
    pub source_provider: String,
    /// `"TIME"` | `"TICK_COUNT"` | `"VOLUME"` | `"DOLLAR"` | `"RENKO"`
    /// (vocabulario fijo ADR-0155). Esta feature siempre produce `"TIME"`.
    pub bar_type: String,
    /// Forma de texto legible del intervalo (ej. `"1H"`).
    pub timeframe_value: String,
    /// Referencia de linaje al dataset crudo de entrada.
    pub data_snapshot_id: String,
    /// Inicio del intervalo semiabierto `[bar_timestamp_ns, bar_close_timestamp_ns)`.
    pub bar_timestamp_ns: i64,
    /// Fin del intervalo (exclusivo).
    pub bar_close_timestamp_ns: i64,
    pub open: i64,
    pub high: i64,
    pub low: i64,
    pub close: i64,
    /// Volumen real negociado. SIEMPRE calculable (nunca `Option`).
    pub volume_traded: i64,
    /// Conteo de ejecuciones agregadas. SIEMPRE calculable (nunca `Option`).
    pub trade_count: i64,
    /// Volumen agredido por el lado comprador. `null` en Dart si al menos
    /// una ejecución del intervalo no distinguía el lado — NUNCA `0` cuando
    /// el dato real es "desconocido". Ver garantía central del esquema en
    /// el comentario de módulo.
    pub volume_buy: Option<i64>,
    /// Conteo de cambios de cotización — proxy de actividad, `None` siempre
    /// en esta feature (solo agrega `TradeTick`, nunca barras ya agregadas).
    pub volume_ticks: Option<i64>,
    /// Volumen en la moneda de cotización. `None` siempre en esta feature.
    pub quote_asset_volume: Option<i64>,
    /// Spread que la fuente adjunta a la barra. `None` siempre en esta feature.
    pub spread_reported: Option<i64>,
    // ── Futuros y Opciones, esquema-listo (ADR-0155) — `None` siempre en
    // esta feature: ninguno es derivable de un `TradeTick`.
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

impl From<BarOutput> for BarDto {
    fn from(b: BarOutput) -> Self {
        Self {
            instrument_id: b.instrument_id,
            venue: b.venue,
            source_provider: b.source_provider,
            bar_type: bar_type_to_string(b.bar_type),
            timeframe_value: b.timeframe_value,
            data_snapshot_id: b.data_snapshot_id,
            bar_timestamp_ns: b.bar_timestamp_ns,
            bar_close_timestamp_ns: b.bar_close_timestamp_ns,
            open: b.open,
            high: b.high,
            low: b.low,
            close: b.close,
            volume_traded: b.volume_traded,
            trade_count: b.trade_count,
            volume_buy: b.volume_buy,
            volume_ticks: b.volume_ticks,
            quote_asset_volume: b.quote_asset_volume,
            spread_reported: b.spread_reported,
            open_interest: b.open_interest,
            settlement_price: b.settlement_price,
            implied_volatility_reported: b.implied_volatility_reported,
            delta_reported: b.delta_reported,
            gamma_reported: b.gamma_reported,
            theta_reported: b.theta_reported,
            vega_reported: b.vega_reported,
            rho_reported: b.rho_reported,
            theoretical_price_reported: b.theoretical_price_reported,
        }
    }
}

/// Resultado de una transformación completada o fallida.
///
/// | Campo                   | Rust              | Dart            | Nullable en Dart |
/// |--------------------------|-------------------|-----------------|-----------------|
/// | `record_id`              | `String`          | `String`        | No              |
/// | `rows_in`/`rows_out`     | `i64`             | `int`           | No              |
/// | `logic_hash`/`audit_hash`| `String`          | `String`        | No              |
/// | `transform_duration_ns`  | `i64`             | `int`           | No              |
/// | `bars`                   | `Vec<BarDto>`     | `List<BarDto>`  | No (lista vacía si falló) |
/// | `matrix_out_present`     | `bool`            | `bool`          | No              |
/// | `error`                  | `Option<String>`  | `String?`       | Sí              |
pub struct TransformOutcomeDto {
    /// UUID del registro de transformación persistido. Cadena vacía si falló
    /// antes de persistir.
    pub record_id: String,
    pub rows_in: i64,
    pub rows_out: i64,
    pub logic_hash: String,
    pub audit_hash: String,
    pub transform_duration_ns: i64,
    /// Las barras resultantes, esquema canónico completo (incluye las cuatro
    /// magnitudes de volumen sin colapsar). Vacío si la transformación falló.
    pub bars: Vec<BarDto>,
    /// `true` si `matrix_out` se materializó (dentro del umbral y exacto en `f64`).
    pub matrix_out_present: bool,
    /// Mensaje de error si la transformación falló; `null` en Dart si fue
    /// exitosa. El Flutter Engineer muestra este mensaje en un `Banner` de
    /// tipo error (ADR-0117 Techo Fijo: sin manejo elaborado de errores en
    /// frontend).
    pub error: Option<String>,
}

impl TransformOutcomeDto {
    /// Construye un resultado fallido: contadores en 0, `bars` vacío, `error` poblado.
    fn from_error(msg: String) -> Self {
        Self {
            record_id: String::new(),
            rows_in: 0,
            rows_out: 0,
            logic_hash: String::new(),
            audit_hash: String::new(),
            transform_duration_ns: 0,
            bars: Vec::new(),
            matrix_out_present: false,
            error: Some(msg),
        }
    }
}

/// Un dataset de origen ya ingerido, elegible para poblar el `Dropdown`
/// "Dataset de origen" del panel inspector.
///
/// Fuente: `sovereign_download_records` (tabla de `sovereign-data-fetcher`)
/// — el dataset de origen de una transformación es, por definición, algo
/// que ya se descargó. Solo se listan los registros con `data_snapshot_id`
/// conocido: un registro sin esa referencia no tiene identidad de linaje
/// resoluble para el selector (ver `Notas de implementación` punto 4 de
/// `docs/features/hybrid-data-transformer.md`).
///
/// | Campo              | Rust     | Dart     | Nullable en Dart |
/// |---------------------|----------|----------|-----------------|
/// | `id`                | `String` | `String` | No               |
/// | `data_snapshot_id`  | `String` | `String` | No               |
/// | `source_endpoint`   | `String` | `String` | No               |
/// | `created_at`        | `i64`    | `int`    | No               |
pub struct SourceDatasetDto {
    /// UUID del registro de descarga que originó este dataset.
    pub id: String,
    /// Referencia canónica al dataset (`<exchange>_<symbol>_<timeframe>_<year><month>`,
    /// ADR-0141) — el valor que el `Dropdown` envía como `data_snapshot_id`
    /// del plan de transformación.
    pub data_snapshot_id: String,
    /// URL/endpoint exacto de la fuente que sirvió el dataset.
    pub source_endpoint: String,
    /// Timestamp de creación del registro de descarga, nanosegundos desde epoch.
    pub created_at: i64,
}

/// Un registro de transformación ya persistido — fuente real del Widget de
/// Tablero (`docs/features/hybrid-data-transformer.md` "Widget de Tablero").
///
/// | Campo                   | Rust              | Dart     | Nullable en Dart |
/// |---------------------------|-------------------|----------|-----------------|
/// | `id`                      | `String`          | `String` | No              |
/// | `created_at`               | `i64`             | `int`    | No              |
/// | `data_snapshot_id`         | `Option<String>`  | `String?`| Sí              |
/// | `logic_hash`               | `String`          | `String` | No              |
/// | `rows_in`/`rows_out`       | `i64`             | `int`    | No              |
/// | `transform_duration_ns`    | `i64`             | `int`    | No              |
/// | `schema_version`           | `String`          | `String` | No              |
pub struct TransformationRecordDto {
    pub id: String,
    pub created_at: i64,
    /// Ref al dataset crudo de entrada. `null` en Dart en el caso borde de
    /// una transformación sin linaje declarado (campo `Option` también en
    /// el registro de dominio, ver `TransformationRecord`).
    pub data_snapshot_id: Option<String>,
    pub logic_hash: String,
    pub rows_in: i64,
    pub rows_out: i64,
    pub transform_duration_ns: i64,
    /// Forma del esquema de `Bar` con el que se produjo esta corrida (ej. `"3.0"`).
    pub schema_version: String,
}

// ── Conversión de vocabularios fijos (texto ↔ enum tipado) ─────────────────

/// Convierte el tipo de barra recibido desde Dart al enum tipado del Núcleo,
/// vía el mismo `Deserialize` que ya valida el JSON del CLI/Banco de Pruebas
/// (`BarTypeInput`, SCREAMING_SNAKE_CASE) — evita un `match` duplicado a
/// mano que se pueda desincronizar del vocabulario real de la feature.
fn parse_bar_type(value: &str) -> Result<BarTypeInput, String> {
    serde_json::from_value(serde_json::Value::String(value.to_string())).map_err(|e| {
        format!(
            "bar_type '{value}' no reconocido (se esperaba TIME, TICK_COUNT, VOLUME, DOLLAR o RENKO): {e}"
        )
    })
}

/// Conversión inversa de [`parse_bar_type`] para el DTO de salida — mismo
/// mecanismo (serde), sin un segundo `match` que mantener sincronizado.
fn bar_type_to_string(value: BarTypeInput) -> String {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(s)) => s,
        _ => unreachable!("BarTypeInput siempre serializa a un string SCREAMING_SNAKE_CASE"),
    }
}

/// Convierte el lado agresor recibido desde Dart al enum tipado del Núcleo,
/// mismo mecanismo que [`parse_bar_type`] (`AggressorSideInput`, UPPERCASE).
fn parse_aggressor_side(value: &str) -> Result<AggressorSideInput, String> {
    serde_json::from_value(serde_json::Value::String(value.to_string()))
        .map_err(|e| format!("aggressor_side '{value}' no reconocido (se esperaba BUY, SELL o UNKNOWN): {e}"))
}

/// Traduce el vocabulario FIJO de agregación de la UI (`data_fetcher.dart`
/// ya documenta el mismo vocabulario para `timeframe`) a nanosegundos.
///
/// **Delega en la feature, que es la dueña del vocabulario.** El Bridge no
/// reimplementa la tabla: una tabla duplicada se desincroniza en cuanto
/// alguien añade una temporalidad en un lado y no en el otro, y el síntoma
/// sería una temporalidad válida rechazada solo desde la interfaz gráfica.
/// El Tech-Lead hizo `pub` la función original (2026-07-18) precisamente
/// para eliminar esa duplicación.
fn parse_target_aggregation_ns(value: &str) -> Result<i64, String> {
    hybrid_data_transformer::public_interface::parse_target_aggregation(value)
        .map_err(|e| e.to_string())
}

// ── Funciones FFI ─────────────────────────────────────────────────────────

/// Ejecuta una transformación híbrida (`TradeTick` → `Bars`) y devuelve el
/// resultado completo cuando la transformación termina.
///
/// ## Parámetros
/// - `db_path`: ruta al archivo SQLite de Drasus.
/// - `instrument_id`/`venue`/`source_provider`/`bar_type`/`timeframe_value`/
///   `data_snapshot_id`: los seis campos de identidad de serie obligatorios
///   por `Bar` (STORY-057, ADR-0155 Restricción 8).
/// - `target_aggregation`: timeframe destino (`"1m"`, `"5m"`, `"15m"`,
///   `"1h"`, `"4h"`, `"1d"` — mismo vocabulario que el `Dropdown` de
///   "Agregación destino").
/// - `interop_threshold`: filas máximas antes de rechazar la conversión a `ndarray`.
/// - `polars_threads`: `null`/ausente = automático (`Switch` "Automático" en ON).
/// - `ticks`: ejecuciones (`TradeTick`) explícitas a transformar.
/// - `source_parquet_path`: alternativa a `ticks` — ruta a un Parquet ya
///   descargado. Exactamente una de las dos debe traer datos.
/// - `parent_id`/`node_id`/`process_id`/`session_id`: metadatos de linaje/infraestructura, opcionales.
///
/// ## Ownership al cruzar la frontera FFI
/// `TransformOutcomeDto` (incluido su `Vec<BarDto>`) se serializa en un
/// objeto Dart. Rust libera la memoria; Dart recibe su propia copia — sin
/// memoria compartida entre los dos lados.
///
/// ## Por qué async (mismo patrón que `submit_download_job`)
/// La transformación puede tardar sobre datasets grandes. Si fuera síncrona
/// bloquearía el hilo principal de Flutter. `flutter_rust_bridge` la
/// convierte en `Future<TransformOutcomeDto>` en Dart: el Flutter Engineer
/// usa `await submitHybridDataTransformation(...)` — **sin** `Timer.periodic`
/// ni sondeo — con el botón "Ejecutar Transformación" en `loading: true`
/// mientras el `Future` no resuelve.
///
/// ## Error handling
/// Si algo falla (BD inaccesible, `bar_type`/`aggressor_side` no reconocido,
/// rechazo del Núcleo) se retorna un `TransformOutcomeDto` con `error` no
/// nulo, `bars` vacío y contadores en 0. Nunca hace panic ante input
/// malformado.
#[allow(clippy::too_many_arguments)]
pub async fn submit_hybrid_data_transformation(
    db_path: String,
    instrument_id: String,
    venue: String,
    source_provider: String,
    bar_type: String,
    timeframe_value: String,
    data_snapshot_id: String,
    target_aggregation: String,
    interop_threshold: u64,
    polars_threads: Option<u32>,
    ticks: Vec<TradeTickDto>,
    source_parquet_path: Option<String>,
    parent_id: Option<String>,
    node_id: Option<String>,
    process_id: Option<String>,
    session_id: Option<String>,
) -> TransformOutcomeDto {
    // Valida forma ANTES de tocar la BD -- consistente con el resto del
    // Bridge (ver `verification.rs::dispatch`).
    let target_interval_ns = match parse_target_aggregation_ns(&target_aggregation) {
        Ok(ns) => ns,
        Err(e) => return TransformOutcomeDto::from_error(e),
    };
    let bar_type_input = match parse_bar_type(&bar_type) {
        Ok(b) => b,
        Err(e) => return TransformOutcomeDto::from_error(e),
    };

    let mut mapped_ticks = Vec::with_capacity(ticks.len());
    for t in ticks {
        let aggressor_side = match t.aggressor_side {
            Some(s) => match parse_aggressor_side(&s) {
                Ok(a) => Some(a),
                Err(e) => return TransformOutcomeDto::from_error(e),
            },
            None => None,
        };
        mapped_ticks.push(TradeTickInput {
            event_timestamp_ns: t.event_timestamp_ns,
            price_scaled: t.price_scaled,
            trade_volume_scaled: t.trade_volume_scaled,
            aggressor_side,
            origin_seq: t.origin_seq,
        });
    }

    let url = format!("sqlite://{db_path}");
    let pool = match create_pool(&url).await {
        Ok(p) => p,
        Err(e) => {
            return TransformOutcomeDto::from_error(format!(
                "no se pudo abrir la base de datos '{db_path}': {e}"
            ))
        }
    };
    if let Err(e) = run_migrations(&pool).await {
        return TransformOutcomeDto::from_error(format!("error al aplicar migraciones: {e}"));
    }

    // Reloj de producción: lee SystemTime del sistema operativo.
    let clock = SystemClock::new();

    let request = ExecuteRequest {
        instrument_id,
        venue,
        source_provider,
        bar_type: bar_type_input,
        timeframe_value,
        data_snapshot_id,
        ticks: mapped_ticks,
        source_parquet_path: source_parquet_path.map(std::path::PathBuf::from),
        target_interval_ns,
        interop_threshold: interop_threshold as usize,
        polars_threads: polars_threads.map(|v| v as usize),
        parent_id,
        node_id,
        process_id,
        session_id,
    };

    match execute_transformation(request, &pool, &clock).await {
        Ok(result) => TransformOutcomeDto {
            record_id: result.record_id,
            rows_in: result.rows_in,
            rows_out: result.rows_out,
            logic_hash: result.logic_hash,
            audit_hash: result.audit_hash,
            transform_duration_ns: result.transform_duration_ns,
            matrix_out_present: result.matrix_out.is_some(),
            bars: result.bars.into_iter().map(BarDto::from).collect(),
            error: None,
        },
        Err(e) => TransformOutcomeDto::from_error(e.to_string()),
    }
}

/// Lista los datasets de origen elegibles para el `Dropdown` "Dataset de
/// origen" del panel inspector — requisito señalado por el UI-Designer
/// (`docs/features/hybrid-data-transformer.md` "Notas de implementación"
/// punto 4), sin binding previo.
///
/// ## Parámetros
/// - `db_path`: ruta al archivo SQLite de Drasus.
///
/// ## Ownership al cruzar la frontera FFI
/// `Vec<SourceDatasetDto>` se serializa en una `List<SourceDatasetDto>` Dart.
/// Rust libera el vector; Dart recibe copias de los datos. Sin memoria compartida.
///
/// ## Error handling
/// Devuelve lista vacía si la BD no existe o la query falla — el `Dropdown`
/// mostrará "Sin datasets disponibles" (correcto en BD recién creada o sin
/// descargas previas), mismo patrón que `list_download_records`.
pub async fn list_source_datasets(db_path: String) -> Vec<SourceDatasetDto> {
    let url = format!("sqlite://{db_path}");
    let pool = match create_pool(&url).await {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };
    if run_migrations(&pool).await.is_err() {
        return Vec::new();
    }

    let clock = SystemClock::new();
    let repo = DownloadRepository::new(&pool, &clock);

    match repo.list_all().await {
        Ok(records) => records
            .into_iter()
            .filter_map(|r| {
                // Solo datasets con `data_snapshot_id` conocido son
                // elegibles: un registro con `None` no tiene identidad de
                // linaje resoluble para el selector (nunca se sustituye por
                // una cadena vacía engañosa).
                let data_snapshot_id = r.data_snapshot_id?;
                Some(SourceDatasetDto {
                    id: r.id,
                    data_snapshot_id,
                    source_endpoint: r.source_endpoint,
                    created_at: r.created_at,
                })
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Lista todos los registros de `transformation_records`, del más reciente
/// al más antiguo — fuente real del Widget de Tablero (`docs/features/hybrid-data-transformer.md`
/// "Widget de Tablero": "carga en `initState()` vía `await` sobre un binding
/// tipo `listTransformationRecords({dbPath})`").
///
/// ## Parámetros
/// - `db_path`: ruta al archivo SQLite de Drasus.
///
/// ## Ownership al cruzar la frontera FFI
/// `Vec<TransformationRecordDto>` se serializa en una `List<TransformationRecordDto>`
/// Dart. Rust libera el vector; Dart recibe copias de los datos. Sin memoria compartida.
///
/// ## Error handling
/// Devuelve lista vacía si la BD no existe o la query falla — el Widget
/// mostrará "Sin transformaciones registradas.", correcto en BD recién
/// creada o sin corridas previas.
pub async fn list_transformation_records(db_path: String) -> Vec<TransformationRecordDto> {
    let url = format!("sqlite://{db_path}");
    let pool = match create_pool(&url).await {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };
    if run_migrations(&pool).await.is_err() {
        return Vec::new();
    }

    let clock = SystemClock::new();
    let repo = TransformationRepository::new(&pool, &clock);

    match repo.list_all().await {
        Ok(records) => records
            .into_iter()
            .map(|r| TransformationRecordDto {
                id: r.id,
                created_at: r.created_at,
                data_snapshot_id: r.data_snapshot_id,
                logic_hash: r.logic_hash,
                rows_in: r.rows_in,
                rows_out: r.rows_out,
                transform_duration_ns: r.transform_duration_ns,
                schema_version: r.schema_version,
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

// ── Pruebas ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use sovereign_data_fetcher::public_interface::NewDownloadRecord;

    /// Ruta de BD SQLite temporal única por prueba -- evita colisión entre
    /// pruebas que corren en paralelo (mismo patrón que
    /// `hybrid_data_transformer::public_interface::verify`).
    fn temp_db_path(label: &str) -> String {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("el reloj del sistema es posterior a la época Unix")
            .as_nanos();
        std::env::temp_dir()
            .join(format!("drasus-bridge-test-hdt-{label}-{nanos}.db"))
            .to_string_lossy()
            .to_string()
    }

    fn tick(ts: i64, price: i64, volume: i64, side: Option<&str>, seq: u64) -> TradeTickDto {
        TradeTickDto {
            event_timestamp_ns: ts,
            price_scaled: price,
            trade_volume_scaled: volume,
            aggressor_side: side.map(str::to_string),
            origin_seq: seq,
        }
    }

    /// Firma común de las pruebas que solo varían el vector de ticks --
    /// evita repetir los 16 argumentos posicionales en cada prueba.
    async fn run_smoke_transformation(db_path: String, ticks: Vec<TradeTickDto>) -> TransformOutcomeDto {
        submit_hybrid_data_transformation(
            db_path,
            "BTCUSDT".to_string(),
            "BINANCE".to_string(),
            "BINANCE".to_string(),
            "TIME".to_string(),
            "1H".to_string(),
            "binance_btcusdt_1h_202607".to_string(),
            "1h".to_string(),
            100_000,
            None,
            ticks,
            None,
            None,
            None,
            None,
            None,
        )
        .await
    }

    /// Cuando TODAS las ejecuciones del intervalo traen `aggressor_side`
    /// conocido, `volume_buy` es un `Some` numéricamente DISTINTO de
    /// `volume_traded` -- prueba discriminante de la garantía central del
    /// esquema, ejercitada a través del DTO expuesto por FFI (criterio 14
    /// de STORY-051).
    #[tokio::test(flavor = "multi_thread")]
    async fn volume_buy_reflects_only_the_buy_side_when_source_distinguishes_it() {
        let ticks = vec![
            tick(1_700_000_000_000_000_000, 100_000_000, 10_000_000, Some("BUY"), 0),
            tick(1_700_000_001_000_000_000, 100_500_000, 20_000_000, Some("SELL"), 1),
            tick(1_700_000_002_000_000_000, 101_000_000, 5_000_000, Some("BUY"), 2),
        ];

        let outcome = run_smoke_transformation(temp_db_path("buy-known"), ticks).await;

        assert!(outcome.error.is_none(), "no debe fallar: {:?}", outcome.error);
        assert_eq!(outcome.bars.len(), 1);
        let bar = &outcome.bars[0];
        assert_eq!(bar.volume_traded, 35_000_000, "suma de las tres ejecuciones");
        assert_eq!(
            bar.volume_buy,
            Some(15_000_000),
            "solo BUY: 10_000_000 + 5_000_000, numéricamente distinto de volume_traded"
        );
        assert_ne!(
            bar.volume_buy,
            Some(bar.volume_traded),
            "la garantía central: nunca colapsa a volume_traded"
        );
    }

    /// Cuando NINGUNA ejecución del intervalo trae `aggressor_side`,
    /// `volume_buy` es `None` -- nunca `Some(0)`. Es la mitad crítica del
    /// criterio 14: "vale cero" y "la fuente no reporta" deben ser estados
    /// distinguibles cruzando la frontera FFI. Un `BarDto` que colapsara el
    /// valor ausente a cero pasaría `volume_buy.unwrap_or(0) == 0` pero
    /// fallaría exactamente esta aserción.
    #[tokio::test(flavor = "multi_thread")]
    async fn volume_buy_is_none_not_zero_when_source_never_distinguishes_the_side() {
        let ticks = vec![
            tick(1_700_000_000_000_000_000, 100_000_000, 10_000_000, None, 0),
            tick(1_700_000_001_000_000_000, 100_500_000, 20_000_000, None, 1),
        ];

        let outcome = run_smoke_transformation(temp_db_path("buy-unknown"), ticks).await;

        assert!(outcome.error.is_none(), "no debe fallar: {:?}", outcome.error);
        assert_eq!(outcome.bars.len(), 1);
        let bar = &outcome.bars[0];
        assert_eq!(bar.volume_traded, 30_000_000);
        assert_eq!(bar.volume_buy, None, "ausente, nunca Some(0)");
        assert_ne!(bar.volume_buy, Some(0), "un None colapsado a 0 rompería la garantía del esquema");
    }

    /// Un tick con `aggressor_side` desconocido entre dos conocidos también
    /// produce `None` para toda la barra -- la regla "todas conocidas o
    /// `None`" (ADR-0155 restricción 4), verificada en el DTO expuesto por
    /// FFI, no solo en el dominio interno.
    #[tokio::test(flavor = "multi_thread")]
    async fn volume_buy_is_none_when_only_some_ticks_in_the_interval_know_the_side() {
        let ticks = vec![
            tick(1_700_000_000_000_000_000, 100_000_000, 10_000_000, Some("BUY"), 0),
            tick(1_700_000_001_000_000_000, 100_500_000, 20_000_000, None, 1),
        ];

        let outcome = run_smoke_transformation(temp_db_path("buy-mixed"), ticks).await;

        assert!(outcome.error.is_none(), "no debe fallar: {:?}", outcome.error);
        assert_eq!(outcome.bars[0].volume_buy, None);
    }

    /// Los campos de Futuros/Opciones -- esquema-listo, nunca derivables de
    /// un `TradeTick` -- también llegan como `None`, no como `0`, al DTO FFI.
    #[tokio::test(flavor = "multi_thread")]
    async fn futures_options_fields_are_none_not_zero_across_ffi() {
        let ticks = vec![tick(1_700_000_000_000_000_000, 100_000_000, 10_000_000, None, 0)];

        let outcome = run_smoke_transformation(temp_db_path("futures-none"), ticks).await;

        assert!(outcome.error.is_none(), "no debe fallar: {:?}", outcome.error);
        let bar = &outcome.bars[0];
        assert_eq!(bar.open_interest, None);
        assert_eq!(bar.settlement_price, None);
        assert_eq!(bar.delta_reported, None);
        assert_eq!(bar.quote_asset_volume, None);
        assert_eq!(bar.spread_reported, None);
        assert_eq!(bar.volume_ticks, None);
    }

    /// Un `bar_type` desconocido se rechaza como error explícito ANTES de
    /// tocar la base de datos -- consistente con el vocabulario FIJO de
    /// cinco valores del Núcleo.
    #[tokio::test(flavor = "multi_thread")]
    async fn unknown_bar_type_is_rejected_with_explicit_error() {
        let outcome = submit_hybrid_data_transformation(
            temp_db_path("bad-bar-type"),
            "BTCUSDT".to_string(),
            "BINANCE".to_string(),
            "BINANCE".to_string(),
            "NOT_A_REAL_TYPE".to_string(),
            "1H".to_string(),
            "binance_btcusdt_1h_202607".to_string(),
            "1h".to_string(),
            100_000,
            None,
            vec![],
            None,
            None,
            None,
            None,
            None,
        )
        .await;

        assert!(outcome.error.is_some());
        assert!(outcome.bars.is_empty());
    }

    /// Un `target_aggregation` fuera del vocabulario fijo se rechaza antes
    /// de tocar la base de datos.
    #[tokio::test(flavor = "multi_thread")]
    async fn unknown_target_aggregation_is_rejected_with_explicit_error() {
        let outcome = submit_hybrid_data_transformation(
            temp_db_path("bad-aggregation"),
            "BTCUSDT".to_string(),
            "BINANCE".to_string(),
            "BINANCE".to_string(),
            "TIME".to_string(),
            "1H".to_string(),
            "binance_btcusdt_1h_202607".to_string(),
            "3m".to_string(),
            100_000,
            None,
            vec![],
            None,
            None,
            None,
            None,
            None,
        )
        .await;

        assert!(outcome.error.is_some());
        assert!(outcome.bars.is_empty());
    }

    /// `list_source_datasets` solo expone registros con `data_snapshot_id`
    /// conocido -- un registro sin linaje resoluble (`None`) no aparece en
    /// el selector, en vez de aparecer con un valor vacío engañoso.
    #[tokio::test(flavor = "multi_thread")]
    async fn list_source_datasets_excludes_records_without_a_data_snapshot_id() {
        let db_path = temp_db_path("datasets");
        let url = format!("sqlite://{db_path}");
        let pool = create_pool(&url).await.expect("pool de prueba");
        run_migrations(&pool).await.expect("migraciones de prueba");
        let clock = SystemClock::new();
        let repo = DownloadRepository::new(&pool, &clock);

        repo.record(NewDownloadRecord {
            data_snapshot_id: Some("binance_btcusdt_1h_202607".to_string()),
            logic_hash: Some("hash-1".to_string()),
            node_id: None,
            process_id: None,
            source_endpoint: "https://data.binance.vision/x".to_string(),
        })
        .await
        .expect("insertar registro con dataset conocido");

        repo.record(NewDownloadRecord {
            data_snapshot_id: None,
            logic_hash: None,
            node_id: None,
            process_id: None,
            source_endpoint: "https://data.binance.vision/y".to_string(),
        })
        .await
        .expect("insertar registro sin dataset conocido");

        let datasets = list_source_datasets(db_path).await;

        assert_eq!(datasets.len(), 1, "el registro sin data_snapshot_id no debe aparecer");
        assert_eq!(datasets[0].data_snapshot_id, "binance_btcusdt_1h_202607");
    }

    /// `list_transformation_records` refleja las corridas ya persistidas --
    /// fuente real para el Widget de Tablero.
    #[tokio::test(flavor = "multi_thread")]
    async fn list_transformation_records_reflects_a_persisted_run() {
        let db_path = temp_db_path("records");
        let ticks = vec![tick(1_700_000_000_000_000_000, 100_000_000, 10_000_000, None, 0)];

        let outcome = run_smoke_transformation(db_path.clone(), ticks).await;
        assert!(outcome.error.is_none(), "no debe fallar: {:?}", outcome.error);

        let records = list_transformation_records(db_path).await;

        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].data_snapshot_id.as_deref(),
            Some("binance_btcusdt_1h_202607")
        );
        assert_eq!(records[0].rows_out, outcome.rows_out);
    }
}
