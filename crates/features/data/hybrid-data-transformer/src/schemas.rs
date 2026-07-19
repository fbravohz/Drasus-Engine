//! Contratos de datos del Hybrid Data Transformer: configuración configurable
//! y structs de persistencia (Perfil A, ADR-0020 + ADR-0141 + ADR-0155).

use serde::{Deserialize, Serialize};

// ── Configuración del transformador (ADR-0008: parámetros configurables) ────

/// Parámetros de operación del Hybrid Data Transformer.
///
/// Todos los valores tienen defaults razonables. Nunca se hardcodean en la
/// lógica — siempre se inyectan desde aquí (tabla "Parámetros Configurables"
/// de la feature).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformerConfig {
    /// Filas máximas antes de rechazar la conversión a `ndarray`. Default:
    /// 100 000 (`INTEROP_THRESHOLD`).
    pub interop_threshold: usize,
    /// Hilos que Polars usa para el `collect()` del plan. `None` = automático
    /// (Polars decide según los núcleos disponibles).
    pub polars_threads: Option<usize>,
}

impl Default for TransformerConfig {
    fn default() -> Self {
        Self { interop_threshold: 100_000, polars_threads: None }
    }
}

// ── Input físico de la transformación (independiente del marcador de puerto) ─

/// Quién cruzó el spread en una ejecución, tal como llega en JSON/Parquet.
/// Espejo serializable de [`crate::domain::AggressorSide`] (ADR-0155:
/// `TradeTick.aggressor_side`, `CHECK IN ('BUY','SELL','UNKNOWN')`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AggressorSideInput {
    Buy,
    Sell,
    Unknown,
}

impl From<AggressorSideInput> for crate::domain::AggressorSide {
    fn from(value: AggressorSideInput) -> Self {
        match value {
            AggressorSideInput::Buy => crate::domain::AggressorSide::Buy,
            AggressorSideInput::Sell => crate::domain::AggressorSide::Sell,
            AggressorSideInput::Unknown => crate::domain::AggressorSide::Unknown,
        }
    }
}

impl From<crate::domain::AggressorSide> for AggressorSideInput {
    fn from(value: crate::domain::AggressorSide) -> Self {
        match value {
            crate::domain::AggressorSide::Buy => AggressorSideInput::Buy,
            crate::domain::AggressorSide::Sell => AggressorSideInput::Sell,
            crate::domain::AggressorSide::Unknown => AggressorSideInput::Unknown,
        }
    }
}

/// Tipo de barra tal como llega en JSON, espejo serializable de
/// [`crate::domain::BarType`] (ADR-0155: `Bars.bar_type`, `CHECK IN ('TIME',
/// 'TICK_COUNT', 'VOLUME', 'DOLLAR', 'RENKO')`). `SCREAMING_SNAKE_CASE`
/// produce EXACTAMENTE esos cinco valores de texto (`TickCount` ->
/// `"TICK_COUNT"`), sin tabla de mapeo manual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BarTypeInput {
    Time,
    TickCount,
    Volume,
    Dollar,
    Renko,
}

impl From<BarTypeInput> for crate::domain::BarType {
    fn from(value: BarTypeInput) -> Self {
        match value {
            BarTypeInput::Time => crate::domain::BarType::Time,
            BarTypeInput::TickCount => crate::domain::BarType::TickCount,
            BarTypeInput::Volume => crate::domain::BarType::Volume,
            BarTypeInput::Dollar => crate::domain::BarType::Dollar,
            BarTypeInput::Renko => crate::domain::BarType::Renko,
        }
    }
}

impl From<crate::domain::BarType> for BarTypeInput {
    fn from(value: crate::domain::BarType) -> Self {
        match value {
            crate::domain::BarType::Time => BarTypeInput::Time,
            crate::domain::BarType::TickCount => BarTypeInput::TickCount,
            crate::domain::BarType::Volume => BarTypeInput::Volume,
            crate::domain::BarType::Dollar => BarTypeInput::Dollar,
            crate::domain::BarType::Renko => BarTypeInput::Renko,
        }
    }
}

/// Una ejecución cruda (`TradeTick`) tal como la recibe esta feature desde el
/// exterior (JSON del harness de verificación, o desde
/// `sovereign-data-fetcher`/`data-normalization-layer` en producción). El
/// marcador de puerto de canvas para este tipo (ADR-0137, catálogo) es
/// `TradeTick` — el contrato físico real de campos vive aquí, en esta
/// feature, que es la primera en necesitarlo.
///
/// **Retrofit ADR-0155:** este struct SOLO puede representar una ejecución
/// -- no tiene campos `bid`/`ask`. `price_scaled`/`trade_volume_scaled` son
/// obligatorios (igual que `TradeTick.price`/`TradeTick.trade_volume` en
/// ADR-0155): un JSON con forma de cotización (`bid`/`ask`, sin estos dos
/// campos) falla al deserializar en vez de colarse silenciosamente como una
/// ejecución falsa -- es la garantía ejecutable de que una `QuoteTick` nunca
/// contribuye a una barra agregada (ver test
/// `quote_shaped_json_fails_to_deserialize_as_trade_tick` en los tests de
/// integración).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TradeTickInput {
    /// Instante del evento de mercado, nanosegundos Unix UTC.
    pub event_timestamp_ns: i64,
    /// Precio de ejecución escalado ×10⁸ (entero exacto, ADR-0141). Campo
    /// `price` de `TradeTick`, obligatorio.
    pub price_scaled: i64,
    /// Tamaño de esta ejecución individual, escalado ×10⁸. Campo
    /// `trade_volume` de `TradeTick`, obligatorio.
    pub trade_volume_scaled: i64,
    /// Quién cruzó el spread. `None`/ausente si la fuente no distingue este
    /// concepto -- alimenta `Bar::volume_buy` con la regla "todas conocidas
    /// o `None`" (ADR-0155 restricción 4).
    #[serde(default)]
    pub aggressor_side: Option<AggressorSideInput>,
    /// Clave de desempate estable de origen (posición en el archivo fuente,
    /// o contador monótono asignado por el proveedor). Requerida para el
    /// orden canónico determinista (invariante 3).
    pub origin_seq: u64,
}

impl From<TradeTickInput> for crate::domain::RawTrade {
    fn from(t: TradeTickInput) -> Self {
        Self {
            event_timestamp_ns: t.event_timestamp_ns,
            price_scaled: t.price_scaled,
            trade_volume_scaled: t.trade_volume_scaled,
            aggressor_side: t.aggressor_side.map(Into::into),
            origin_seq: t.origin_seq,
        }
    }
}

/// Una barra OHLCV de salida en forma serializable (JSON/output del harness).
/// Espejo 1:1 de [`crate::domain::Bar`] (ADR-0155): volumen en cuatro
/// magnitudes independientes + campos de Futuros/Opciones esquema-listo +
/// seis campos de identidad de serie obligatorios por fila (STORY-057:
/// `instrument_id`, `venue`, `source_provider`, `bar_type`,
/// `timeframe_value`, `data_snapshot_id`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarOutput {
    pub instrument_id: String,
    pub venue: String,
    pub source_provider: String,
    pub bar_type: BarTypeInput,
    pub timeframe_value: String,
    pub data_snapshot_id: String,
    pub bar_timestamp_ns: i64,
    pub bar_close_timestamp_ns: i64,
    pub open: i64,
    pub high: i64,
    pub low: i64,
    pub close: i64,
    pub volume_traded: i64,
    pub trade_count: i64,
    pub volume_buy: Option<i64>,
    pub volume_ticks: Option<i64>,
    pub quote_asset_volume: Option<i64>,
    pub spread_reported: Option<i64>,
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

impl From<crate::domain::Bar> for BarOutput {
    fn from(b: crate::domain::Bar) -> Self {
        Self {
            instrument_id: b.instrument_id,
            venue: b.venue,
            source_provider: b.source_provider,
            bar_type: b.bar_type.into(),
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

/// Solicitud de transformación que llega desde la interfaz pública.
///
/// ## Cambio incompatible (STORY-057, ADR-0155 Restricción 8 + enmienda de
/// tripleta 2026-07-18)
///
/// Seis campos de identidad de serie ahora OBLIGATORIOS: `instrument_id`,
/// `venue`, `source_provider`, `bar_type`, `timeframe_value` y
/// `data_snapshot_id` (este último cambia de `Option<String>` a `String` --
/// un `TransformRequest` sin bróker conocido no es un caso válido con
/// `None`, es un caso inválido). Todo llamador que hoy construye este struct
/// sin estos campos deja de compilar; es la consecuencia directa de que cada
/// `Bar` de salida ahora exige estos mismos seis campos no nulos.
#[derive(Debug, Clone)]
pub struct TransformRequest {
    /// Símbolo/instrumento (ej. "EURUSD", "BTC-USD"). Se copia tal cual a
    /// cada `Bar` de salida.
    pub instrument_id: String,
    /// Bróker/venue donde se opera el instrumento (ej. "PEPPERSTONE") --
    /// determina precio, spread, sesión y convención de cierre; NO es
    /// necesariamente quien entregó el dato (ver `source_provider`).
    pub venue: String,
    /// Quién entregó el dato (puede diferir de `venue`: un vendor de datos
    /// consolidados sobre un venue). Cuando coinciden (caso típico retail),
    /// el llamador repite el mismo valor en ambos campos.
    pub source_provider: String,
    /// Tipo de barra destino. En esta feature (agregación por intervalo de
    /// tiempo fijo) casi siempre `BarTypeInput::Time` -- las otras cuatro
    /// variantes quedan esquema-listo para `algorithmic-bars` (aún no
    /// construida).
    pub bar_type: BarTypeInput,
    /// Forma de texto legible del intervalo de agregación (ej. "1H") --
    /// representación distinta de `target_interval_ns` (la forma numérica
    /// que el Núcleo usa para agregar); el llamador suministra ambos,
    /// coherentes entre sí. NO se deriva aquí una tabla
    /// `nanosegundos -> texto` (fuera de alcance de esta Story).
    pub timeframe_value: String,
    /// Referencia al dataset crudo de entrada (formato canónico ADR-0141:
    /// `<exchange>_<symbol>_<timeframe>_<year><month>`).
    pub data_snapshot_id: String,
    /// Ejecuciones (`TradeTick`) a transformar -- única fuente válida de
    /// agregación a `Bars` (ADR-0155). En esta Story llegan ya en memoria
    /// (desde el JSON del harness o desde el fetcher); la lectura directa
    /// desde Parquet vía `scan_parquet` con un `data_snapshot_id` real queda
    /// cableada en `orchestrator::load_ticks_dataframe`, lista para cuando
    /// exista el escritor de Parquet aguas arriba.
    pub ticks: Vec<TradeTickInput>,
    /// Intervalo de agregación destino, nanosegundos (ej. 3_600_000_000_000 = 1h).
    pub target_interval_ns: i64,
    /// Transformación previa encadenada, si el pipeline tiene varios pasos.
    pub parent_id: Option<String>,
    /// Huella del hardware donde corre este proceso.
    pub node_id: Option<String>,
    /// PID del proceso que ejecuta la transformación.
    pub process_id: Option<String>,
    /// Sesión de ingesta que originó esta transformación.
    pub session_id: Option<String>,
}

/// Resultado de una transformación completada — lo que ve el llamador
/// (Bridge FFI / harness CLI), no lo que se persiste (ver [`TransformationRecord`]).
#[derive(Debug, Clone)]
pub struct TransformResult {
    /// UUID del registro de transformación persistido.
    pub record_id: String,
    /// Filas de entrada (ejecuciones recibidas).
    pub rows_in: i64,
    /// Filas de salida (barras producidas).
    pub rows_out: i64,
    /// Hash reproducible del plan de transformación.
    pub logic_hash: String,
    /// Hash del contenido resultante (barras ya ordenadas).
    pub audit_hash: String,
    /// Duración real del proceso, nanosegundos.
    pub transform_duration_ns: i64,
    /// Las barras resultantes, esquema canónico.
    pub bars: Vec<BarOutput>,
    /// Bloque columnar convertido a matriz densa (`ndarray::Array2<f64>`)
    /// vía el puente Zero-Copy de Polars (`DataFrame::to_ndarray`, TTR-002),
    /// solo si el conteo de filas estuvo dentro de `INTEROP_THRESHOLD` y
    /// todos los valores fueron representables de forma exacta en `f64`.
    /// `None` si se rechazó.
    pub matrix_out: Option<ndarray::Array2<f64>>,
}

// ── Structs de persistencia (tabla transformation_records) ──────────────────
//
// Sin cambios de esquema por ADR-0155: esta tabla registra METADATOS de cada
// EJECUCIÓN del transformador (Grupo I + III + IV, ADR-0020) -- no es el
// dataset de `Bars` en sí (que no tiene columnas Grupo I por fila, ver la
// corrección del Architect en ADR-0155 "Corrección del Architect al insumo
// del Quant-Engineer"). El ledger de transformaciones es un concepto
// ortogonal al esquema de la fila de mercado.

/// Una fila de la tabla `transformation_records` ya persistida.
///
/// Perfil A (ADR-0020): Grupo I (universal) + Grupo III (Linaje) + Grupo IV
/// (Infraestructura) + campos propios de dominio documentados como excepción
/// (`docs/features/hybrid-data-transformer.md` "Persistencia").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformationRecord {
    // ── Grupo I: Identidad & Integridad ──
    pub id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub event_sequence_id: i64,

    // ── Grupo III: Linaje ──
    pub data_snapshot_id: Option<String>,
    pub transformation_id: String,
    pub logic_hash: String,
    pub parent_id: Option<String>,

    // ── Grupo IV: Infraestructura ──
    pub node_id: Option<String>,
    pub process_id: Option<String>,
    pub session_id: Option<String>,

    // ── Campos propios de dominio ──
    pub transform_duration_ns: i64,
    pub rows_in: i64,
    pub rows_out: i64,
    pub schema_version: String,
}

/// Datos para insertar un nuevo registro de transformación. Los campos del
/// Grupo I los genera automáticamente el repositorio.
#[derive(Debug, Clone)]
pub struct NewTransformationRecord {
    pub data_snapshot_id: Option<String>,
    pub transformation_id: String,
    pub logic_hash: String,
    pub parent_id: Option<String>,
    pub node_id: Option<String>,
    pub process_id: Option<String>,
    pub session_id: Option<String>,
    pub transform_duration_ns: i64,
    pub rows_in: i64,
    pub rows_out: i64,
    pub schema_version: String,
    /// El `audit_hash` YA calculado en el Núcleo sobre el contenido de las
    /// barras ordenadas (criterio 5) — el repositorio NO lo recalcula, solo
    /// lo persiste y lo encadena con `audit_chain_hash`.
    pub content_audit_hash: String,
}

// ── Pruebas de la capa de contrato ───────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// El campo `aggressor_side` es opcional y, ausente, se deserializa como
    /// `None` -- una fuente que no lo reporta no rompe el contrato.
    #[test]
    fn trade_tick_input_deserializes_without_aggressor_side() {
        let json = r#"{"event_timestamp_ns":1,"price_scaled":100,"trade_volume_scaled":5,"origin_seq":0}"#;
        let parsed: TradeTickInput = serde_json::from_str(json).expect("debe deserializar sin aggressor_side");
        assert!(parsed.aggressor_side.is_none());
    }

    /// Con `aggressor_side` presente, se mapea correctamente al enum de
    /// dominio a través del `From` -- ida y vuelta sin pérdida.
    #[test]
    fn aggressor_side_round_trips_through_domain_conversion() {
        let json = r#"{"event_timestamp_ns":1,"price_scaled":100,"trade_volume_scaled":5,"aggressor_side":"BUY","origin_seq":0}"#;
        let parsed: TradeTickInput = serde_json::from_str(json).expect("debe deserializar con aggressor_side");
        let domain_trade: crate::domain::RawTrade = parsed.into();
        assert_eq!(domain_trade.aggressor_side, Some(crate::domain::AggressorSide::Buy));
    }
}
