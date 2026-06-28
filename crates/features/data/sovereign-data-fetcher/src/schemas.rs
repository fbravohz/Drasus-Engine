//! Contratos de datos del Sovereign Data Fetcher: configuración configurable
//! y structs de persistencia (Perfil A, ADR-0020 V2).

use serde::{Deserialize, Serialize};

// ── Configuración del fetcher (ADR-0008: todos los parámetros configurables) ─

/// Parámetros de operación del Sovereign Data Fetcher.
///
/// Todos los valores tienen defaults razonables (ver `Default::default()`).
/// Nunca se hardcodean en la lógica — siempre se inyectan desde aquí.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetcherConfig {
    /// Cuántos archivos Bulk descargar en paralelo. Rango 1–20. Default: 5.
    /// Más alto = más ancho de banda usado; más bajo = más amable con otros pipelines.
    pub concurrent_downloads: usize,
    /// Cuántas veces reintentar la llamada Delta REST antes de rendirse. Rango 1–10. Default: 3.
    pub delta_sync_retry: u32,
    /// URL base del servidor de volcados Bulk del broker (Binance Vision por defecto).
    pub bulk_source_url: String,
}

impl Default for FetcherConfig {
    /// Valores por defecto según la spec de la feature (sovereign-data-fetcher.md).
    fn default() -> Self {
        Self {
            concurrent_downloads: 5,
            delta_sync_retry: 3,
            bulk_source_url: "https://data.binance.vision".to_string(),
        }
    }
}

// ── Structs de persistencia (tabla sovereign_download_records) ───────────────

/// Una fila de la tabla `sovereign_download_records` ya persistida.
///
/// Contiene los campos del Perfil A (ADR-0020 V2):
/// Grupo I (Identidad & Integridad) + Grupo III (Linaje) + Grupo IV (Hardware)
/// + el campo de dominio propio `source_endpoint`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadRecord {
    // ── Grupo I: Identidad & Integridad ──
    /// UUID único del registro de descarga.
    pub id: String,
    /// Timestamp de creación en nanosegundos.
    pub created_at: i64,
    /// Timestamp de última modificación (igual a `created_at` para este registro inmutable).
    pub updated_at: i64,
    /// Hash SHA-256 del contenido de la fila (snapshot de integridad).
    pub audit_hash: String,
    /// Hash del registro anterior en la cadena; `None` para el primer registro.
    pub audit_chain_hash: Option<String>,
    /// Posición monótona en la cadena global de registros de descarga.
    pub event_sequence_id: i64,

    // ── Grupo III: Linaje ──
    /// Referencia al volcado/snapshot del broker que originó el segmento.
    pub data_snapshot_id: Option<String>,
    /// Hash del driver del fetcher que produjo este registro.
    pub logic_hash: Option<String>,

    // ── Grupo IV: Hardware ──
    /// Huella del hardware donde se ejecutó la descarga.
    pub node_id: Option<String>,
    /// PID del worker de descarga.
    pub process_id: Option<String>,

    // ── Campo propio de dominio ──
    /// URL/endpoint de la fuente Bulk o REST de la que provino el dato.
    pub source_endpoint: String,
}

/// Datos para insertar un nuevo registro de descarga.
///
/// Los campos del Grupo I (id, created_at, updated_at, audit_hash,
/// audit_chain_hash, event_sequence_id) los genera automáticamente el
/// repositorio, igual que en el resto de las tablas del sistema.
#[derive(Debug, Clone)]
pub struct NewDownloadRecord {
    // Grupo III
    pub data_snapshot_id: Option<String>,
    pub logic_hash: Option<String>,
    // Grupo IV
    pub node_id: Option<String>,
    pub process_id: Option<String>,
    // Campo propio de dominio
    pub source_endpoint: String,
}

/// Resultado de una operación de descarga completada.
#[derive(Debug, Clone)]
pub struct FetchResult {
    /// UUID del Job de async-job-executor que gestionó esta descarga.
    pub job_id: String,
    /// UUID del registro de descarga persistido en `sovereign_download_records`.
    pub record_id: String,
    /// Número de archivos Bulk descargados exitosamente.
    pub bulk_files_downloaded: usize,
    /// Bytes descargados por el tramo Delta REST (0 si no hubo Delta).
    pub delta_bytes: usize,
    /// Total de bytes descargados (Bulk + Delta).
    pub total_bytes: u64,
}

/// Solicitud de descarga que llega desde la interfaz pública.
#[derive(Debug, Clone)]
pub struct FetchRequest {
    /// Símbolo de mercado a descargar (ej. `"BTCUSDT"`).
    pub symbol: String,
    /// Intervalo temporal del instrumento (ej. `"1m"`, `"1h"`).
    pub interval: String,
    /// Inicio del rango de fechas solicitado (nanosegundos desde epoch).
    pub start_ns: i64,
    /// Fin del rango de fechas solicitado (nanosegundos desde epoch).
    pub end_ns: i64,
    /// Directorio local donde se almacenarán los datos descargados.
    pub dest_dir: std::path::PathBuf,
    /// Timestamp que representa el "ahora" (inyectado, no del reloj del sistema).
    pub now_ns: i64,
    /// Bytes disponibles en disco en el momento de la solicitud.
    pub available_disk_bytes: u64,
}
