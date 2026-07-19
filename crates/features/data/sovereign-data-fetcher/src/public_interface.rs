//! Superficie pública del Sovereign Data Fetcher — el ÚNICO módulo `pub`.
//!
//! Expone:
//! - Los puertos de salida (`OutputPorts`) con los marcadores de tipo
//!   `QuoteTick`, `TradeTick` y `Bars` del catálogo ADR-0137. Son stubs en
//!   esta Story: el contrato físico (Polars/Arrow) lo define el
//!   transformador aguas abajo.
//! - Los traits de puerto de entrada de infraestructura (`BulkSource`, `DeltaSource`):
//!   la frontera que permite probar el fetcher sin red real.
//! - La función principal `fetch()` y la función de recuperación `recover_interrupted_downloads()`.
//! - Los tipos de solicitud/respuesta necesarios para llamar a esta feature.
//!
//! Otros crates solo pueden importar desde aquí; prohibido acceder a
//! `domain`, `orchestrator`, `persistence` o `schemas` directamente.

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use shared::public_interface::{Clock, SystemClock, create_pool, run_migrations};
use shared::types::{Bars, QuoteTick, TradeTick};

use crate::orchestrator;

// ── Re-exports para que los tests de integración puedan acceder a ellos ──────

// Tipos de dominio
pub use crate::domain::{
    compute_delta_range, reconcile_boundary, BulkFileInfo, DiskSpaceResult, DownloadPlan,
    TimeRange,
};
// Schemas / config
pub use crate::schemas::{
    DownloadRecord, FetchRequest, FetchResult, FetcherConfig, NewDownloadRecord,
};
// Repositorio de descarga (para verificar persistencia en tests)
pub use crate::persistence::{DownloadRepository, DownloadRepositoryError};

// ── Tipo de error público ────────────────────────────────────────────────────

/// Errores que puede producir la operación de descarga soberana.
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    /// Espacio en disco insuficiente para el tamaño estimado del Bulk.
    /// La descarga se aborta antes de iniciar cualquier transferencia.
    #[error("disco insuficiente: se necesitan {required_bytes} bytes pero solo hay {available_bytes} disponibles")]
    InsufficientDiskSpace {
        required_bytes: u64,
        available_bytes: u64,
    },
    /// El adaptador de fuente Bulk falló después de todos los reintentos.
    #[error("error en fuente Bulk: {0}")]
    BulkSourceFailed(String),
    /// El adaptador de fuente Delta falló después de todos los reintentos.
    #[error("error en fuente Delta: {0}")]
    DeltaSourceFailed(String),
    /// Error de persistencia en SQLite.
    #[error("error de base de datos: {0}")]
    Database(String),
    /// Error de I/O en el sistema de archivos local.
    #[error("error de sistema de archivos: {0}")]
    Io(String),
    /// Error interno no esperado.
    #[error("error interno: {0}")]
    Internal(String),
}

// ── Traits de puertos de infraestructura (frontera de inyección de dependencias) ─

/// Puerto para listar e descargar archivos del inventario Bulk del broker.
///
/// La implementación real (`ReqwestBulkSource` en `orchestrator.rs`) usa HTTP.
/// La implementación de test usa datos en memoria — jamás toca la red.
#[async_trait]
pub trait BulkSource: Send + Sync {
    /// Lista los archivos Bulk disponibles para el rango temporal dado.
    /// Devuelve sus metadatos (nombre, URL, rango temporal, tamaño estimado).
    async fn list_inventory(&self, range: &TimeRange) -> Result<Vec<BulkFileInfo>, FetchError>;

    /// Descarga un archivo Bulk al path local indicado y devuelve los bytes escritos.
    async fn download_file(
        &self,
        file: &BulkFileInfo,
        dest_path: &std::path::Path,
    ) -> Result<u64, FetchError>;
}

/// Puerto para pedir el tramo Delta (datos recientes) a la API REST.
///
/// La implementación real usa HTTP REST. La implementación de test devuelve
/// bytes predefinidos en memoria sin contactar ningún servidor.
#[async_trait]
pub trait DeltaSource: Send + Sync {
    /// Descarga el segmento de datos del rango temporal dado y devuelve
    /// los bytes crudos (CSV, JSON u otro formato según el broker).
    async fn fetch_range(&self, range: &TimeRange) -> Result<Vec<u8>, FetchError>;
}

// ── Puertos de salida del Canvas (marcadores de tipo, ADR-0137) ──────────────

/// Puertos de salida del Sovereign Data Fetcher hacia el Canvas.
///
/// La feature descarga el mismo flujo (Bulk + Delta) pero produce dos
/// primitivos físicos distintos y mutuamente excluyentes por solicitud
/// (ADR-0155): una cotización (`QuoteTick`, Bid/Ask) nunca es lo mismo que
/// una ejecución (`TradeTick`, precio/volumen realmente negociado) — mezclar
/// ambas en un solo puerto produce backtests que asumen precios a los que
/// nadie operó. El demultiplexado de fuentes que mezclan los dos eventos en
/// un mismo archivo (MT5) es responsabilidad de `data-normalization-layer`,
/// no de este crate.
///
/// En esta Story los tipos son stubs (marcadores): indican QUÉ clase de dato
/// produce la feature, pero su representación física Polars/Arrow la define
/// el transformador aguas abajo (TTR-007, Story diferida).
pub struct OutputPorts {
    /// Cotizaciones crudas Bid/Ask descargadas (volcados de quotes + Delta REST).
    pub quote_ticks_out: QuoteTick,
    /// Ejecuciones crudas Last/Volumen descargadas (volcados de trades + Delta REST).
    pub trade_ticks_out: TradeTick,
    /// Barras OHLCV crudas cuando la fuente entrega volcados de klines.
    pub bars_out: Bars,
}

// ── Función principal ────────────────────────────────────────────────────────

/// Ejecuta la descarga híbrida soberana (Bulk + Delta REST).
///
/// Flujo:
/// 1. Lista el inventario Bulk disponible para el rango solicitado.
/// 2. Planifica la descarga (Bulk-first; Delta solo cubre el residuo).
/// 3. Verifica el espacio en disco antes de iniciar cualquier transferencia.
/// 4. Crea un Job durable en SQLite (recuperable tras un crash).
/// 5. Descarga los archivos Bulk concurrentemente (respeta `CONCURRENT_DOWNLOADS`).
/// 6. Reintenta los archivos Bulk fallidos automáticamente.
/// 7. Descarga el tramo Delta, reintentando hasta `DELTA_SYNC_RETRY` veces.
/// 8. Registra la operación en `sovereign_download_records` (Perfil A).
/// 9. Transiciona el Job a COMPLETED/FAILED.
///
/// `pool` debe apuntar a una base de datos ya migrada (incluye migración 0006).
/// `clock` se inyecta — nunca se llama al reloj del sistema directamente.
pub async fn fetch(
    config: &FetcherConfig,
    request: FetchRequest,
    bulk_source: Arc<dyn BulkSource>,
    delta_source: &dyn DeltaSource,
    pool: &SqlitePool,
    clock: &dyn Clock,
) -> Result<FetchResult, FetchError> {
    orchestrator::fetch(config, request, bulk_source, delta_source, pool, clock).await
}

/// Recupera las descargas interrumpidas tras un reinicio del sistema.
///
/// Escanea la tabla `jobs` buscando Jobs de tipo `"SOVEREIGN_FETCH"` que
/// estaban en estado `RUNNING` cuando el proceso terminó abruptamente.
/// Los reencola (RUNNING → QUEUED) para que el sistema los retome al
/// siguiente ciclo de ejecución.
///
/// Devuelve el número de Jobs reencolados.
pub async fn recover_interrupted_downloads(
    pool: &SqlitePool,
    clock: &dyn Clock,
) -> Result<usize, FetchError> {
    orchestrator::recover_interrupted_downloads(pool, clock).await
}

// ── Harness de verificación CLI (ADR-0142 Fase 1) ────────────────────────────

/// Input para la verificación del Sovereign Data Fetcher vía CLI.
///
/// Se deserializa desde el JSON que pasa el usuario con `--input '...'`.
/// Todos los campos tienen valores por defecto para que el comando funcione
/// sin argumentos adicionales.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyInput {
    /// Símbolo del mercado a descargar (ej. `"BTCUSDT"`).
    pub symbol: String,
    /// Intervalo temporal del instrumento (ej. `"1m"`, `"1h"`, `"1d"`).
    pub interval: String,
    /// Número de días hacia atrás que abarca la verificación. Default: 1.
    /// Mantenerlo bajo evita saturar la red; 1 día es suficiente para probar.
    pub days: Option<u32>,
}

impl Default for VerifyInput {
    /// Valores por defecto para una prueba de humo rápida contra Binance.
    fn default() -> Self {
        Self {
            symbol: "BTCUSDT".to_string(),
            interval: "1h".to_string(),
            days: Some(1),
        }
    }
}

/// Output de la verificación del Sovereign Data Fetcher.
///
/// Siempre serializa a JSON válido. Si `ok` es `true`, los campos de
/// resultado están rellenos. Si `ok` es `false`, `error` describe el problema
/// y el resto de los campos opcionales son `null`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOutput {
    /// `true` si la descarga completó sin errores; `false` en caso contrario.
    pub ok: bool,
    /// UUID del Job de descarga creado en SQLite.
    pub job_id: Option<String>,
    /// UUID del registro de descarga persistido en `sovereign_download_records`.
    pub record_id: Option<String>,
    /// Número de archivos Bulk descargados exitosamente.
    pub bulk_files_downloaded: Option<usize>,
    /// Bytes descargados por el tramo Delta REST (0 si no hubo Delta).
    pub delta_bytes: Option<usize>,
    /// Total de bytes descargados (Bulk + Delta).
    pub total_bytes: Option<u64>,
    /// Descripción del error en caso de fallo; `null` si la operación fue exitosa.
    pub error: Option<String>,
}

impl VerifyOutput {
    /// Construye un output de error con todos los campos de resultado en `None`.
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            job_id: None,
            record_id: None,
            bulk_files_downloaded: None,
            delta_bytes: None,
            total_bytes: None,
            error: Some(msg),
        }
    }
}

impl From<FetchResult> for VerifyOutput {
    /// Convierte un FetchResult exitoso en un VerifyOutput con `ok: true`.
    fn from(r: FetchResult) -> Self {
        Self {
            ok: true,
            job_id: Some(r.job_id),
            record_id: Some(r.record_id),
            bulk_files_downloaded: Some(r.bulk_files_downloaded),
            delta_bytes: Some(r.delta_bytes),
            total_bytes: Some(r.total_bytes),
            error: None,
        }
    }
}

/// Ejecuta la verificación del Sovereign Data Fetcher con adaptadores inyectables.
///
/// Permite usar fuentes falsas en tests sin tocar la red real.
/// La capa CLI inyecta los adaptadores reales (`ReqwestBulkSource` / `ReqwestDeltaSource`);
/// los tests de integración inyectan fuentes falsas.
///
/// `dest_dir` es la carpeta donde se escriben los archivos Bulk descargados.
/// El llamador es responsable de crear el directorio antes de llamar a esta función.
pub async fn verify_with_sources(
    input: &VerifyInput,
    dest_dir: &std::path::Path,
    bulk_source: Arc<dyn BulkSource>,
    delta_source: &dyn DeltaSource,
    pool: &SqlitePool,
    clock: &dyn Clock,
) -> VerifyOutput {
    // Calcula el rango temporal: desde `days` días atrás hasta el instante actual del reloj.
    let now_ns = clock.timestamp_ns();
    let days = input.days.unwrap_or(1) as i64;
    // 1 día = 86 400 segundos; 1 segundo = 1 000 000 000 nanosegundos.
    let start_ns = now_ns - days * 86_400 * 1_000_000_000;

    let config = FetcherConfig::default();
    let request = FetchRequest {
        symbol: input.symbol.clone(),
        interval: input.interval.clone(),
        start_ns,
        end_ns: now_ns,
        dest_dir: dest_dir.to_path_buf(),
        now_ns,
        // 10 GiB declarados como disponibles: pasa el check de disco sin consultar el sistema.
        // En una verificación controlada no queremos que el check de disco detenga la prueba.
        available_disk_bytes: 10 * 1024 * 1024 * 1024,
    };

    // Delega en la función principal de la feature; esta función es solo un adaptador de Shell.
    match fetch(&config, request, bulk_source, delta_source, pool, clock).await {
        Ok(result) => VerifyOutput::from(result),
        Err(e) => VerifyOutput::from_error(e.to_string()),
    }
}

// ── Ejecución de producción (entrypoint del Bridge FFI) ─────────────────────

/// Parámetros para una descarga de producción iniciada desde el Bridge FFI.
///
/// A diferencia de `VerifyInput`, estos parámetros vienen directamente de la
/// UI: el usuario seleccionó broker, símbolo, rango y timeframe en la SVF.
/// El Bridge construye esta estructura y llama a `execute()`.
#[derive(Debug, Clone)]
pub struct ExecuteRequest {
    /// Símbolo del mercado a descargar (ej. `"BTCUSDT"`).
    pub symbol: String,
    /// Intervalo temporal del instrumento (ej. `"1m"`, `"1h"`, `"1d"`).
    pub interval: String,
    /// Inicio del rango solicitado en nanosegundos desde epoch.
    pub start_ns: i64,
    /// Fin del rango solicitado en nanosegundos desde epoch.
    pub end_ns: i64,
    /// Directorio local donde se escribirán los archivos Bulk descargados.
    /// Debe existir antes de llamar a `execute()` — el Bridge lo crea si
    /// es necesario con `std::fs::create_dir_all`.
    pub data_dir: std::path::PathBuf,
    /// URL base del servidor de volcados Bulk del broker
    /// (ej. `"https://data.binance.vision"`).
    pub bulk_source_url: String,
}

/// Ejecuta una descarga soberana de producción con adaptadores HTTP reales.
///
/// A diferencia de `verify()`, esta función:
/// - Usa la base de datos de producción indicada en `pool` (ya migrada).
/// - Escribe los datos en `request.data_dir` (debe existir antes de llamar).
/// - No elimina los archivos descargados al terminar.
///
/// Es el entrypoint que el Bridge FFI llama desde `submit_download_job`.
/// El reloj `clock` se inyecta para facilitar pruebas sin cambiar la firma.
///
/// ## Límite de disco en EPIC-1
///
/// `available_disk_bytes` se fija a 50 GiB porque la UI no expone un
/// selector de ruta ni lee el espacio disponible. Si el usuario tiene menos
/// de 50 GiB libres, las descargas fallará por error de I/O (no por el
/// chequeo de disco). En una Story futura se puede leer el espacio real
/// con `statvfs` / `GetDiskFreeSpaceEx` y pasarlo aquí.
pub async fn execute(
    request: ExecuteRequest,
    pool: &SqlitePool,
    clock: &dyn Clock,
) -> Result<FetchResult, FetchError> {
    // Crea el adaptador Bulk real (HTTP con TLS puro en Rust).
    let bulk_source = orchestrator::ReqwestBulkSource::new(request.bulk_source_url.clone())
        .map_err(|e| FetchError::Internal(format!("error al crear el adaptador Bulk: {e}")))?;

    // Crea el adaptador Delta real apuntando a la API REST de Binance.
    // El endpoint base de Binance se fija en EPIC-1; en una Story futura
    // puede derivarse del `broker_url` o de un mapa de brokers.
    let delta_source = orchestrator::ReqwestDeltaSource::new(
        "https://api.binance.com".to_string(),
        request.symbol.clone(),
        request.interval.clone(),
    )
    .map_err(|e| FetchError::Internal(format!("error al crear el adaptador Delta: {e}")))?;

    let config = FetcherConfig {
        bulk_source_url: request.bulk_source_url,
        ..FetcherConfig::default()
    };

    // Fija el timestamp "ahora" usando el reloj inyectado.
    let now_ns = clock.timestamp_ns();

    let fetch_request = FetchRequest {
        symbol: request.symbol,
        interval: request.interval,
        start_ns: request.start_ns,
        end_ns: request.end_ns,
        dest_dir: request.data_dir,
        now_ns,
        // 50 GiB declarados como disponibles en EPIC-1 (sin lectura real de disco).
        // TODO (Story futura): leer el espacio real vía statvfs/GetDiskFreeSpaceEx.
        available_disk_bytes: 50 * 1024 * 1024 * 1024,
    };

    fetch(
        &config,
        fetch_request,
        Arc::new(bulk_source),
        &delta_source,
        pool,
        clock,
    )
    .await
}

/// Ejecuta la verificación del Sovereign Data Fetcher con adaptadores reales (HTTP + disco).
///
/// Crea automáticamente un directorio temporal y una base de datos SQLite temporal
/// para no contaminar los datos de producción. Los adaptadores HTTP conectan a
/// los mismos endpoints de Binance que en producción.
///
/// El directorio temporal se conserva tras la ejecución para que el usuario pueda
/// inspeccionar los datos descargados. Ruta: `$TMPDIR/drasus-verify-<uuid>/`.
///
/// Uso típico desde el CLI:
/// `drasus verify sovereign-data-fetcher --input '{"symbol":"BTCUSDT","interval":"1h"}'`
pub async fn verify(input: VerifyInput) -> VerifyOutput {
    // Crea un directorio temporal único para esta verificación.
    // Se conserva para que el usuario pueda inspeccionar los datos descargados.
    let temp_dir = std::env::temp_dir().join(format!("drasus-verify-{}", Uuid::new_v4()));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return VerifyOutput::from_error(format!(
            "no se pudo crear el directorio temporal de verificación: {e}"
        ));
    }

    // Crea una BD SQLite temporal exclusiva para esta verificación.
    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match create_pool(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return VerifyOutput::from_error(format!(
                "no se pudo crear la BD temporal de verificación: {e}"
            ))
        }
    };
    if let Err(e) = run_migrations(&pool).await {
        return VerifyOutput::from_error(format!(
            "error al aplicar migraciones en la BD temporal: {e}"
        ));
    }

    // Reloj de producción: toma el tiempo real del sistema operativo.
    let clock = SystemClock::default();

    // Adaptador Bulk real: conecta a Binance Vision para listar e descargar históricos.
    // `list_inventory` es un stub en esta Story (devuelve inventario vacío);
    // la integración real con el directorio de Binance Vision se completa en la Story siguiente.
    let config = FetcherConfig::default();
    let bulk_source = match orchestrator::ReqwestBulkSource::new(config.bulk_source_url.clone()) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            return VerifyOutput::from_error(format!(
                "error al crear el adaptador Bulk HTTP: {e}"
            ))
        }
    };

    // Adaptador Delta real: conecta a la API REST de Binance para datos de los últimos días.
    let delta_source =
        match orchestrator::ReqwestDeltaSource::new(
            "https://api.binance.com".to_string(),
            input.symbol.clone(),
            input.interval.clone(),
        ) {
            Ok(s) => s,
            Err(e) => {
                return VerifyOutput::from_error(format!(
                    "error al crear el adaptador Delta HTTP: {e}"
                ))
            }
        };

    verify_with_sources(&input, &temp_dir, bulk_source, &delta_source, &pool, &clock).await
}

// ── Pruebas de contrato de puerto (STORY-058) ────────────────────────────────

#[cfg(test)]
mod port_contract_tests {
    use super::*;
    use shared::types::TypedPort;

    // Prueba discriminante: antes de STORY-058, `OutputPorts` solo tenía el
    // campo `ticks_out: Tick` — este test no compilaba (ni existían
    // `quote_ticks_out`/`trade_ticks_out`). Si alguien revierte la migración
    // y vuelve a colapsar los dos puertos en uno, este archivo deja de
    // compilar y el test falla, no solo una aserción en tiempo de ejecución.
    #[test]
    fn output_ports_expone_quote_y_trade_ticks_por_separado() {
        let ports = OutputPorts {
            quote_ticks_out: QuoteTick,
            trade_ticks_out: TradeTick,
            bars_out: Bars,
        };

        // Cada campo debe corresponder al tipo de puerto canónico de ADR-0155:
        // cotización y ejecución nunca comparten un solo marcador de tipo.
        assert_eq!(
            <QuoteTick as TypedPort>::type_id(),
            "QuoteTick",
            "quote_ticks_out debe tipar contra el marcador QuoteTick del catálogo"
        );
        assert_eq!(
            <TradeTick as TypedPort>::type_id(),
            "TradeTick",
            "trade_ticks_out debe tipar contra el marcador TradeTick del catálogo"
        );
        // El campo existe y es del tipo correcto: si esto compila, el struct
        // literal de arriba ya lo demostró. Se referencia aquí para dejar
        // explícito en el test que no es un campo muerto.
        let _ = ports.quote_ticks_out;
        let _ = ports.trade_ticks_out;
        let _ = ports.bars_out;
    }
}
