//! Superficie pública del Sovereign Data Fetcher — el ÚNICO módulo `pub`.
//!
//! Expone:
//! - Los puertos de salida (`OutputPorts`) con los marcadores de tipo `Tick` y `Bars`
//!   del catálogo ADR-0137. Son stubs en esta Story: el contrato físico
//!   (Polars/Arrow) lo define el transformador aguas abajo.
//! - Los traits de puerto de entrada de infraestructura (`BulkSource`, `DeltaSource`):
//!   la frontera que permite probar el fetcher sin red real.
//! - La función principal `fetch()` y la función de recuperación `recover_interrupted_downloads()`.
//! - Los tipos de solicitud/respuesta necesarios para llamar a esta feature.
//!
//! Otros crates solo pueden importar desde aquí; prohibido acceder a
//! `domain`, `orchestrator`, `persistence` o `schemas` directamente.

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::SqlitePool;

use shared::public_interface::Clock;
use shared::types::{Bars, Tick};

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
/// En esta Story los tipos son stubs (marcadores): indican QUÉ clase de dato
/// produce la feature, pero su representación física Polars/Arrow la define
/// el transformador aguas abajo (TTR-007, Story diferida).
pub struct OutputPorts {
    /// Transacciones crudas Bid/Ask/Last (volcados de trades + Delta REST).
    pub ticks_out: Tick,
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
