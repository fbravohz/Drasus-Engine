//! Funciones FFI para el Sovereign Data Fetcher — lanzamiento y consulta de
//! descargas soberanas de históricos de mercado hacia Flutter/Dart.
//!
//! ## Decisión de diseño — Opción A (await, sin polling)
//!
//! El backend (`sovereign_data_fetcher::public_interface::execute()`) es
//! inline bloqueante: crea el Job, transiciona a RUNNING y ejecuta la descarga
//! completa (Bulk + Delta) antes de devolver. No existe un mecanismo de
//! submit-y-polling en EPIC-1.
//!
//! Por eso `submit_download_job` se expone como `async fn` sin `#[frb(sync)]`:
//! flutter_rust_bridge lo convierte en `Future<DownloadJobResult>` en Dart.
//! El Flutter Engineer usa `await submitDownloadJob(...)` con spinner mientras
//! dura la descarga; cuando el Future resuelve, la UI muestra el resultado en
//! la Zona B y llama a `listDownloadRecords()` para refrescar la Zona C.
//!
//! ## Gap de polling documentado (para el Tech Lead / Rust Engineer)
//!
//! El patrón submit-y-polling (botón "Descargar" devuelve `job_id` de inmediato;
//! la UI hace polling con `getJobStatus` cada 2 s vía `Timer.periodic`) requiere
//! un entrypoint de submit asíncrono en segundo plano. El `async-job-executor`
//! (ADR-0011) de `shared` tiene la infraestructura de colas, pero `SOVEREIGN_FETCH`
//! no está registrado como tipo de trabajo que el ejecutor procese de forma
//! no bloqueante. Esto es una Story de Rust Engineer separada.
//!
//! ## Gap de datos en `listDownloadRecords` (para el Flutter Engineer)
//!
//! La tabla `sovereign_download_records` no almacena `symbol`, `bytes_total`
//! ni `status` — esos campos no están en la migración 0006. El historial
//! visible en la Zona C solo muestra `id`, `created_at` y `source_endpoint`.
//! El símbolo descargado y los bytes totales vienen del `DownloadJobResult`
//! devuelto por `submitDownloadJob`; el Flutter Engineer debe guardarlos en
//! estado local (`_lastResult`) para poblar la Zona B.

use shared::public_interface::{create_pool, run_migrations, JobRepository, SystemClock};
use sovereign_data_fetcher::public_interface::{DownloadRepository, ExecuteRequest, execute};

// ── Tipos FFI-safe ────────────────────────────────────────────────────────────
//
// Todos los campos son primitivos (`i64`, `u64`, `u8`) o `String` / `Option<String>`.
// flutter_rust_bridge puede transportarlos sin serialización adicional.

/// Resultado de una descarga completada o fallida.
///
/// Propietario de memoria: Rust crea esta estructura; flutter_rust_bridge la
/// serializa y la libera. Dart recibe una copia en su propio heap — sin memoria
/// compartida entre los dos lados.
///
/// | Campo                   | Rust            | Dart     | Nullable en Dart |
/// |-------------------------|-----------------|----------|-----------------|
/// | `job_id`                | `String`        | `String` | No               |
/// | `record_id`             | `String`        | `String` | No               |
/// | `bulk_files_downloaded` | `u64`           | `int`    | No               |
/// | `delta_bytes`           | `u64`           | `int`    | No               |
/// | `total_bytes`           | `u64`           | `int`    | No               |
/// | `error`                 | `Option<String>`| `String?`| Sí               |
pub struct DownloadJobResult {
    /// UUID del Job persistido en la tabla `jobs`. Cadena vacía si falló
    /// antes de crear el Job (ej. error de conexión a la BD).
    /// Propietario: Rust genera el UUID; Dart lo recibe de solo lectura.
    pub job_id: String,
    /// UUID del registro persistido en `sovereign_download_records`.
    /// Cadena vacía si la descarga falló antes de persistir el registro.
    pub record_id: String,
    /// Archivos Bulk descargados exitosamente. Vale 0 si la descarga falló.
    pub bulk_files_downloaded: u64,
    /// Bytes del tramo Delta REST descargado. Vale 0 si no hubo Delta o falló.
    pub delta_bytes: u64,
    /// Total de bytes descargados (Bulk + Delta). Vale 0 si falló.
    pub total_bytes: u64,
    /// Descripción del error si la descarga falló; `null` en Dart si fue
    /// exitosa. El Flutter Engineer muestra este mensaje en un `GlowBanner`
    /// de tipo error en la Zona B.
    pub error: Option<String>,
}

impl DownloadJobResult {
    /// Construye un resultado fallido con campos numéricos en 0 y el
    /// mensaje de error poblado. Se usa cuando el Bridge no puede ni
    /// llegar a llamar al backend (ej. BD inaccesible, directorio inexistente).
    fn from_error(msg: String) -> Self {
        Self {
            job_id: String::new(),
            record_id: String::new(),
            bulk_files_downloaded: 0,
            delta_bytes: 0,
            total_bytes: 0,
            error: Some(msg),
        }
    }
}

/// Estado de un Job de descarga soberana leído desde la tabla `jobs`.
///
/// Útil para que el Flutter Engineer muestre el estado final en la Zona B
/// después de que el Future de `submitDownloadJob` resuelva. En EPIC-1 no
/// se usa para polling periódico (ver §Decisión de diseño arriba).
///
/// | Campo        | Rust  | Dart | Nullable en Dart |
/// |--------------|-------|------|-----------------|
/// | `id`         | `String` | `String` | No         |
/// | `state`      | `String` | `String` | No         |
/// | `progress`   | `u8`     | `int`    | No         |
/// | `created_at` | `i64`    | `int`    | No         |
/// | `updated_at` | `i64`    | `int`    | No         |
pub struct JobStatusDto {
    /// UUID del Job (mismo valor que `DownloadJobResult.job_id`).
    pub id: String,
    /// Estado del Job como cadena. Valores posibles:
    /// "QUEUED", "RUNNING", "COMPLETED", "FAILED", "CANCELLED".
    /// El Flutter Engineer mapea estos valores a los chips de color del
    /// §Estados Semánticos de la Cáscara Visual.
    pub state: String,
    /// Porcentaje de avance (0–100). El backend fija 100 al completar.
    /// En EPIC-1 este campo siempre llega a 100 al resolver el Future —
    /// no hay actualizaciones intermedias (sin polling).
    pub progress: u8,
    /// Timestamp de creación del Job en nanosegundos desde epoch.
    pub created_at: i64,
    /// Timestamp de la última actualización del Job en nanosegundos desde epoch.
    pub updated_at: i64,
}

/// Registro de una descarga persistido en `sovereign_download_records`.
///
/// Solo expone los campos disponibles en la tabla (ver migración 0006).
/// Los campos `symbol`, `bytes_total` y `status` no existen en esta tabla —
/// vienen del `DownloadJobResult` devuelto por `submitDownloadJob`.
///
/// | Campo             | Rust     | Dart     | Nullable en Dart |
/// |-------------------|----------|----------|-----------------|
/// | `id`              | `String` | `String` | No               |
/// | `created_at`      | `i64`    | `int`    | No               |
/// | `source_endpoint` | `String` | `String` | No               |
pub struct DownloadRecordDto {
    /// UUID único del registro de descarga.
    pub id: String,
    /// Timestamp de creación en nanosegundos desde epoch.
    /// Conversión a Dart: `DateTime.fromMicrosecondsSinceEpoch(createdAt ~/ 1000)`.
    pub created_at: i64,
    /// URL/endpoint exacto de la fuente (Bulk S3 o REST) que sirvió los datos.
    /// Truncar a 40 caracteres en la `GlowTable` con `GlowTooltip` al hover.
    pub source_endpoint: String,
}

// ── Funciones FFI ─────────────────────────────────────────────────────────────

/// Ejecuta una descarga híbrida soberana (Bulk + Delta REST) y devuelve el
/// resultado completo cuando la descarga termina.
///
/// ## Parámetros
/// - `db_path`: ruta al archivo SQLite de Drasus (ej. `/home/usuario/drasus.db`).
/// - `data_dir`: directorio donde se guardarán los archivos descargados.
///   Se crea automáticamente con `create_dir_all` si no existe.
/// - `symbol`: símbolo del mercado en mayúsculas (ej. `"BTCUSDT"`).
/// - `broker_url`: URL base del servidor Bulk (ej. `"https://data.binance.vision"`).
/// - `start_ns`, `end_ns`: límites del rango en nanosegundos desde epoch.
///   Conversión desde Dart: `dateTime.microsecondsSinceEpoch * 1000`.
/// - `timeframe`: intervalo temporal (ej. `"1m"`, `"5m"`, `"1h"`, `"1d"`).
/// - `output_type`: tipo de salida (`"ticks"` o `"bars"`). Informativo en EPIC-1;
///   el transformer aguas abajo lo usará en una Story posterior.
///
/// ## Ownership al cruzar la frontera FFI
/// `DownloadJobResult` se serializa en un objeto Dart. Rust libera la memoria;
/// Dart recibe su propia copia. No hay memoria compartida entre los dos lados.
///
/// ## Por qué async (Opción A)
/// La descarga puede durar segundos o minutos. Si fuera síncrona (`#[frb(sync)]`)
/// bloquearía el hilo principal de Flutter causando janks. flutter_rust_bridge
/// la convierte en `Future<DownloadJobResult>` en Dart: el Flutter Engineer usa
/// `await submitDownloadJob(...)` con un spinner de `transitionIndigo` mientras dura.
///
/// ## Error handling
/// Si algo falla (BD inaccesible, error de red, disco lleno) se retorna un
/// `DownloadJobResult` con `error` no nulo y campos numéricos en 0.
/// El Flutter Engineer muestra `GlowBanner(type: error, message: result.error!)`
/// en la Zona B (ADR-0117 Techo Fijo: sin manejo elaborado de errores en EPIC-1).
// Los 8 parámetros reflejan directamente los 8 campos del formulario de la SVF.
// Agruparlos en un struct añadiría boilerplate sin mejorar la legibilidad en Dart,
// ya que FRB genera named parameters individuales en el lado Dart.
#[allow(clippy::too_many_arguments)]
pub async fn submit_download_job(
    db_path: String,
    data_dir: String,
    symbol: String,
    broker_url: String,
    start_ns: i64,
    end_ns: i64,
    timeframe: String,
    // output_type es informativo en EPIC-1: el transformer aguas abajo (TTR-007,
    // Story diferida) lo usará para decidir entre Tick y Bars. Se recibe aquí
    // para que la firma FFI sea estable cuando se cablee la Story del transformer.
    output_type: String,
) -> DownloadJobResult {
    // Silencia la advertencia de campo no usado en EPIC-1.
    // El transformer lo consumirá en una Story posterior.
    let _ = output_type;

    // Construye la URL de conexión y abre el pool de conexiones SQLite.
    let url = format!("sqlite://{db_path}");
    let pool = match create_pool(&url).await {
        Ok(p) => p,
        Err(e) => {
            return DownloadJobResult::from_error(format!(
                "no se pudo abrir la base de datos '{db_path}': {e}"
            ))
        }
    };

    // Aplica las migraciones pendientes (idempotente si ya están aplicadas).
    if let Err(e) = run_migrations(&pool).await {
        return DownloadJobResult::from_error(format!("error al aplicar migraciones: {e}"));
    }

    // Crea el directorio de datos si no existe. El backend escribe los
    // archivos Bulk descargados en esta ruta.
    let data_path = std::path::PathBuf::from(&data_dir);
    if let Err(e) = std::fs::create_dir_all(&data_path) {
        return DownloadJobResult::from_error(format!(
            "no se pudo crear el directorio de datos '{data_dir}': {e}"
        ));
    }

    // Reloj de producción: lee SystemTime del sistema operativo.
    let clock = SystemClock::new();

    // Construye la solicitud de ejecución y llama al backend.
    // Esta llamada bloquea (en el sentido de Rust async: cede el control al
    // runtime pero no avanza hasta que la descarga termina) — puede tardar
    // desde segundos hasta varios minutos según el rango solicitado.
    let request = ExecuteRequest {
        symbol,
        interval: timeframe,
        start_ns,
        end_ns,
        data_dir: data_path,
        bulk_source_url: broker_url,
    };

    match execute(request, &pool, &clock).await {
        Ok(result) => DownloadJobResult {
            job_id: result.job_id,
            record_id: result.record_id,
            bulk_files_downloaded: result.bulk_files_downloaded as u64,
            delta_bytes: result.delta_bytes as u64,
            total_bytes: result.total_bytes,
            error: None,
        },
        Err(e) => DownloadJobResult::from_error(e.to_string()),
    }
}

/// Consulta el estado actual de un Job de descarga desde la tabla `jobs`.
///
/// ## Parámetros
/// - `db_path`: ruta al archivo SQLite de Drasus.
/// - `job_id`: UUID del Job devuelto en `DownloadJobResult.job_id`.
///
/// ## Cuándo llamarlo (EPIC-1)
/// El Flutter Engineer llama a esta función UNA VEZ después de que el
/// Future de `submitDownloadJob` resuelva, para poblar la Zona B con el
/// estado final y el progreso. En EPIC-1 no hay polling periódico —
/// ese patrón requiere el submit asíncrono en segundo plano (gap documentado).
///
/// ## Ownership al cruzar la frontera FFI
/// `Option<JobStatusDto>` se serializa como `JobStatusDto?` en Dart.
/// Rust libera la memoria; Dart recibe su copia. Sin memoria compartida.
///
/// ## Error handling
/// Devuelve `null` en Dart si el `job_id` no existe en la BD o si la
/// conexión falla. El Flutter Engineer comprueba el null antes de renderizar.
pub async fn get_job_status(db_path: String, job_id: String) -> Option<JobStatusDto> {
    // Abre el pool; un error aquí no colapsa la UI: devuelve null.
    let url = format!("sqlite://{db_path}");
    let pool = create_pool(&url).await.ok()?;
    run_migrations(&pool).await.ok()?;

    // SystemClock es requerido por JobRepository pero no se invoca en lecturas.
    let clock = SystemClock::new();
    let repo = JobRepository::new(&pool, &clock);

    // Busca el Job; devuelve None si no existe o si hay error de BD.
    let job = repo.find(&job_id).await.ok()??;

    Some(JobStatusDto {
        id: job.id,
        // `as_str()` devuelve "QUEUED", "RUNNING", "COMPLETED", "FAILED"
        // o "CANCELLED" — cadenas estables que el Flutter Engineer mapea
        // a los colores del §Estados Semánticos de la Cáscara Visual.
        state: job.state.as_str().to_string(),
        progress: job.progress,
        created_at: job.created_at_ns,
        updated_at: job.updated_at_ns,
    })
}

/// Lista todos los registros de `sovereign_download_records`, del más reciente
/// al más antiguo. Sin paginación — adecuado para el volumen de EPIC-1.
///
/// ## Parámetros
/// - `db_path`: ruta al archivo SQLite de Drasus.
///
/// ## Ownership al cruzar la frontera FFI
/// `Vec<DownloadRecordDto>` se serializa en una `List<DownloadRecordDto>` Dart.
/// Rust libera el vector; Dart recibe copias de los datos. Sin memoria compartida.
///
/// ## Error handling
/// Devuelve lista vacía si la BD no existe o la query falla. La Zona C
/// mostrará `GlowEmpty({message: 'Sin descargas aún.'})` — correcto en
/// BD recién creada o sin descargas previas.
///
/// ## Nota para el Flutter Engineer
/// Esta función solo expone `id`, `created_at` y `source_endpoint`.
/// La tabla `sovereign_download_records` no almacena `symbol`, `bytes_total`
/// ni `status` (ver migración 0006 — esos campos no existen). El símbolo y los
/// bytes del último job vienen del `DownloadJobResult` de `submitDownloadJob`;
/// guardarlos en `_lastResult` en el estado del widget para la Zona B.
pub async fn list_download_records(db_path: String) -> Vec<DownloadRecordDto> {
    // Abre el pool; un error devuelve lista vacía (UI muestra estado vacío).
    let url = format!("sqlite://{db_path}");
    let pool = match create_pool(&url).await {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };
    if run_migrations(&pool).await.is_err() {
        return Vec::new();
    }

    // SystemClock es requerido por DownloadRepository pero no se usa en lecturas.
    let clock = SystemClock::new();
    let repo = DownloadRepository::new(&pool, &clock);

    // Devuelve lista vacía silenciosamente si la query falla — la UI muestra
    // el estado vacío sin colapsar, igual que `get_jobs_summary` en jobs.rs.
    match repo.list_all().await {
        Ok(records) => records
            .into_iter()
            .map(|r| DownloadRecordDto {
                id: r.id,
                created_at: r.created_at,
                source_endpoint: r.source_endpoint,
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}
