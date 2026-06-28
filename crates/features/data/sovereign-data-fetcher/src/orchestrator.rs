//! [SHELL] Cáscara imperativa del Sovereign Data Fetcher.
//!
//! Aquí vive el código que toca I/O: cliente HTTP asíncrono, descompresor
//! ZIP, sistema de archivos, pool SQLite y gestión del ciclo de vida del Job.
//!
//! El núcleo puro (lógica de planificación, reconciliación, verificación de disco)
//! está en `domain.rs` — este módulo solo lo llama con los datos que recoge del exterior.

use std::path::Path;
use std::sync::Arc;

use sqlx::SqlitePool;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use shared::public_interface::{Clock, Job, JobRepository, JobState, NewJob};

use crate::domain::{check_disk_space, plan_downloads, DiskSpaceResult, TimeRange};
use crate::persistence::DownloadRepository;
use crate::public_interface::{BulkSource, DeltaSource, FetchError};
use crate::schemas::{FetchRequest, FetchResult, FetcherConfig, NewDownloadRecord};

// ── Constantes internas ──────────────────────────────────────────────────────

/// Tipo de Job que identifica una descarga soberana en la tabla `jobs`.
/// Se usa para localizar Jobs huérfanos durante la recuperación al reiniciar.
const JOB_TYPE_SOVEREIGN_FETCH: &str = "SOVEREIGN_FETCH";

// ── Función principal de orquestación ───────────────────────────────────────

/// Ejecuta la descarga híbrida soberana (Bulk + Delta REST).
///
/// Toda la lógica de I/O está aquí. Las decisiones de qué descargar y cómo
/// unir los datos las toma `domain.rs` en puro.
pub(crate) async fn fetch(
    config: &FetcherConfig,
    request: FetchRequest,
    bulk_source: Arc<dyn BulkSource>,
    delta_source: &dyn DeltaSource,
    pool: &SqlitePool,
    clock: &dyn Clock,
) -> Result<FetchResult, FetchError> {
    let requested_range = TimeRange {
        start_ns: request.start_ns,
        end_ns: request.end_ns,
    };

    // ── Paso 1: Lista el inventario Bulk disponible para el rango solicitado.
    let inventory = bulk_source
        .list_inventory(&requested_range)
        .await
        .map_err(|e| FetchError::BulkSourceFailed(e.to_string()))?;

    // ── Paso 2: Planifica la descarga (Bulk-first; Delta solo cubre el residuo).
    // La planificación es pura: llama al dominio sin I/O.
    let plan = plan_downloads(requested_range, &inventory);

    // ── Paso 3: Verifica el espacio en disco ANTES de iniciar cualquier descarga.
    // Si es insuficiente, aborta de inmediato sin descargar nada.
    if let DiskSpaceResult::Insufficient {
        required_bytes,
        available_bytes,
    } = check_disk_space(plan.total_estimated_bytes, request.available_disk_bytes)
    {
        return Err(FetchError::InsufficientDiskSpace {
            required_bytes,
            available_bytes,
        });
    }

    // ── Paso 4: Crea un Job durable en SQLite (ADR-0011 — recuperable tras crash).
    // El Job se persiste ANTES de iniciar la descarga (persist-before-ack).
    let job_repo = JobRepository::new(pool, clock);
    let parameters = serde_json::json!({
        "symbol": request.symbol,
        "interval": request.interval,
        "start_ns": request.start_ns,
        "end_ns": request.end_ns,
    })
    .to_string();
    let job = job_repo
        .submit(NewJob {
            user_id: "system".to_string(),
            job_type: JOB_TYPE_SOVEREIGN_FETCH.to_string(),
            parameters,
            owner_id: None,
            access_token_id: None,
            session_id: None,
            node_id: None,
            logic_hash: None,
        })
        .await
        .map_err(|e| FetchError::Database(e.to_string()))?;

    // Transiciona a RUNNING para registrar que la descarga ya empezó.
    let job = job_repo
        .transition(&job, JobState::Running, Some("sovereign-fetcher"))
        .await
        .map_err(|e| FetchError::Database(e.to_string()))?;

    // Ejecuta la descarga y captura el resultado para poder transicionar el Job.
    let result = execute_download(config, &request, bulk_source, delta_source, plan, &job, pool, clock).await;

    // ── Paso 9: Transiciona el Job a su estado final según el resultado.
    let final_state = if result.is_ok() {
        JobState::Completed
    } else {
        JobState::Failed
    };
    // Ignoramos el error de la transición final: si falla, el Job queda en RUNNING
    // y la recuperación al reiniciar lo reencolará. No es catastrófico.
    let _ = job_repo.transition(&job, final_state, None).await;

    result
}

/// Ejecuta la descarga efectiva (pasos 5–8). Separado de `fetch` para poder
/// transicionar el Job a FAILED si algo falla en cualquier paso.
///
/// Recibe `bulk_source` como `Arc<dyn BulkSource>` porque se comparte entre
/// tareas concurrentes lanzadas con `JoinSet::spawn`. Una referencia no puede
/// cruzar el límite de `spawn` ('static), pero un `Arc` sí puede clonarse.
///
/// Los 8 parámetros reflejan las dependencias inyectadas de la cáscara
/// (config, request, fuentes Bulk/Delta, plan, job, pool, clock) — agruparlos
/// en un struct añadiría boilerplate sin mejorar la legibilidad del sitio de llamada.
#[allow(clippy::too_many_arguments)]
async fn execute_download(
    config: &FetcherConfig,
    request: &FetchRequest,
    bulk_source: Arc<dyn BulkSource>,
    delta_source: &dyn DeltaSource,
    plan: crate::domain::DownloadPlan,
    job: &Job,
    pool: &SqlitePool,
    clock: &dyn Clock,
) -> Result<FetchResult, FetchError> {
    // Clonamos el PathBuf para poder moverlo a las tareas paralelas sin lifetimes.
    let dest_dir = request.dest_dir.clone();

    // Separamos los campos del plan antes de consumir `bulk_files` en el JoinSet.
    let bulk_files = plan.bulk_files;
    let delta_range = plan.delta_range;

    // ── Paso 5: Descarga los archivos Bulk con concurrencia real (JoinSet).
    //
    // PATRÓN CORRECTO: se lanzan TODAS las tareas de descarga en paralelo con
    // `JoinSet::spawn`. Cada tarea espera el permiso del semáforo POR SU CUENTA,
    // sin bloquear el bucle principal. El resultado:
    //   - Hasta `concurrent_downloads` descargas corren simultáneamente.
    //   - En cuanto una termina y libera su permiso, otra empieza de inmediato.
    //
    // CONTRASTE con el patrón INCORRECTO (versión anterior):
    //   - Adquirir el permiso FUERA de la tarea (en el bucle principal) +
    //     `download_file(...).await` completo + `drop(permit)` + siguiente iteración.
    //   - Resultado: nunca hay más de 1 descarga activa porque el bucle espera
    //     a que cada descarga termine antes de iniciar la siguiente.
    //     El semáforo en ese patrón es puramente decorativo.
    let semaphore = Arc::new(Semaphore::new(config.concurrent_downloads));
    let mut join_set: JoinSet<Result<u64, FetchError>> = JoinSet::new();

    for file in bulk_files.into_iter() {
        let sem = Arc::clone(&semaphore);
        let src = Arc::clone(&bulk_source);
        let dest_path = dest_dir.join(&file.filename);

        // La tarea se registra y queda "lista" en el runtime pero no empieza
        // a ejecutar hasta que el caller hace `.await` (en `join_next` abajo).
        // En ese momento, múltiples tareas compiten por los permisos del semáforo.
        join_set.spawn(async move {
            // El permiso se adquiere DENTRO de la tarea, no fuera.
            // Así varias tareas pueden estar esperando el permiso al mismo tiempo,
            // produciendo solapamiento real tan pronto como un permiso se libera.
            let _permit = sem
                .acquire_owned()
                .await
                // El semáforo nunca se cierra aquí; si falla es error de programación.
                .expect("el semáforo no debería cerrarse durante la descarga");

            let mut bytes = 0u64;
            let mut last_err = None;

            // ── Paso 6: Reintentos de descarga Bulk — fijo en 3.
            // El parámetro configurable `delta_sync_retry` aplica solo al tramo REST
            // porque la API puede tener throttling variable por broker. Los servidores
            // de archivos estáticos del Bulk (p.ej. Binance Vision) no tienen
            // throttling; 3 reintentos cubre los fallos transitorios habituales
            // (timeout, reset TCP). Si en el futuro se necesita configurar reintentos
            // Bulk, añadir `bulk_download_retry: u32` a `FetcherConfig` — pendiente
            // escalamiento al Tech-Lead antes de introducir ese campo.
            for attempt in 1..=3u32 {
                match src.download_file(&file, &dest_path).await {
                    Ok(b) => {
                        bytes = b;
                        last_err = None;
                        break;
                    }
                    Err(e) => {
                        last_err = Some(e);
                        if attempt < 3 {
                            // Espera exponencial breve entre reintentos (50ms, 100ms).
                            tokio::time::sleep(
                                tokio::time::Duration::from_millis(50 * attempt as u64),
                            )
                            .await;
                        }
                    }
                }
            }

            // Si agotó los 3 intentos, la tarea falla con el último error.
            if let Some(err) = last_err {
                return Err(FetchError::BulkSourceFailed(format!(
                    "archivo '{}' falló después de 3 intentos: {}",
                    file.filename, err
                )));
            }

            Ok(bytes)
        });
    }

    // Recoge los resultados de las tareas conforme terminan (en cualquier orden).
    // Si alguna tarea falla, propagamos el error de inmediato (fail-fast).
    let mut bulk_bytes_total: u64 = 0;
    let mut bulk_files_downloaded = 0;

    while let Some(task_result) = join_set.join_next().await {
        match task_result {
            // La tarea completó y la descarga fue exitosa.
            Ok(Ok(bytes)) => {
                bulk_bytes_total += bytes;
                bulk_files_downloaded += 1;
            }
            // La tarea completó pero la descarga falló.
            Ok(Err(e)) => return Err(e),
            // La tarea fue cancelada o entró en pánico — error interno irrecuperable.
            Err(join_err) => {
                return Err(FetchError::Internal(format!(
                    "tarea de descarga terminada abruptamente: {join_err}"
                )));
            }
        }
    }

    // ── Paso 7: Descarga el tramo Delta con reintentos hasta `delta_sync_retry`.
    let mut delta_bytes = 0usize;
    if let Some(delta_range) = delta_range {
        let mut last_err = None;
        for attempt in 1..=config.delta_sync_retry {
            match delta_source.fetch_range(&delta_range).await {
                Ok(data) => {
                    delta_bytes = data.len();
                    last_err = None;
                    break;
                }
                Err(e) => {
                    last_err = Some(e);
                    if attempt < config.delta_sync_retry {
                        // Espera breve exponencial entre reintentos.
                        tokio::time::sleep(tokio::time::Duration::from_millis(50 * attempt as u64)).await;
                    }
                }
            }
        }

        if let Some(err) = last_err {
            return Err(FetchError::DeltaSourceFailed(format!(
                "Delta falló después de {} intentos: {}",
                config.delta_sync_retry, err
            )));
        }
    }

    let total_bytes = bulk_bytes_total + delta_bytes as u64;

    // ── Paso 8: Registra la operación en `sovereign_download_records` (Perfil A).
    let dl_repo = DownloadRepository::new(pool, clock);
    let record = dl_repo
        .record(NewDownloadRecord {
            data_snapshot_id: None,
            logic_hash: None,
            node_id: None,
            process_id: job.process_id.clone(),
            source_endpoint: config.bulk_source_url.clone(),
        })
        .await
        .map_err(|e| FetchError::Database(e.to_string()))?;

    Ok(FetchResult {
        job_id: job.id.clone(),
        record_id: record.id,
        bulk_files_downloaded,
        delta_bytes,
        total_bytes,
    })
}

// ── Recuperación al reiniciar (ADR-0011) ────────────────────────────────────

/// Escanea los Jobs de tipo `SOVEREIGN_FETCH` que quedaron en estado RUNNING
/// (el proceso terminó abruptamente durante una descarga) y los reencola
/// (RUNNING → QUEUED) para que sean retomados.
///
/// Devuelve el número de Jobs reencolados.
pub(crate) async fn recover_interrupted_downloads(
    pool: &SqlitePool,
    clock: &dyn Clock,
) -> Result<usize, FetchError> {
    let job_repo = JobRepository::new(pool, clock);

    // Busca todos los Jobs en estado RUNNING — cualquier tipo, porque la tabla
    // no tiene índice por tipo y el volumen en un nodo local es pequeño.
    let running_jobs = job_repo
        .jobs_in_state(JobState::Running)
        .await
        .map_err(|e| FetchError::Database(e.to_string()))?;

    let mut recovered = 0;
    for job in running_jobs {
        // Solo procesa los Jobs de descarga soberana; ignora los de otros tipos.
        if job.job_type != JOB_TYPE_SOVEREIGN_FETCH {
            continue;
        }
        // Transiciona RUNNING → QUEUED: el sistema recuperó este Job y lo
        // reencola para que se retome en el próximo ciclo de descarga.
        job_repo
            .transition(&job, JobState::Queued, None)
            .await
            .map_err(|e| FetchError::Database(e.to_string()))?;
        recovered += 1;
    }

    Ok(recovered)
}

// ── Adaptador real de red (compilado pero no probado contra red real en CI) ──

/// Adaptador real de descarga Bulk que usa `reqwest` con `rustls-tls`.
#[allow(dead_code)] // No se conecta a la app todavía; se usa en producción.
///
/// En CI y en las pruebas de esta Story se usa el adaptador falso
/// (`FakeBulkSource` en los tests de integración). Este adaptador solo se
/// activa en producción al ser inyectado por la capa de la aplicación.
pub struct ReqwestBulkSource {
    client: reqwest::Client,
    base_url: String,
}

impl ReqwestBulkSource {
    /// Crea el adaptador HTTP. El cliente usa TLS puro en Rust (sin OpenSSL).
    #[allow(dead_code)]
    pub fn new(base_url: String) -> Result<Self, FetchError> {
        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .build()
            .map_err(|e| FetchError::Internal(e.to_string()))?;
        Ok(Self { client, base_url })
    }
}

#[async_trait::async_trait]
impl BulkSource for ReqwestBulkSource {
    /// Lista el inventario de archivos Bulk disponibles para el rango dado.
    ///
    /// En la implementación real, haría una petición HTTP al directorio
    /// del broker (ej. `data.binance.vision`) y parsearía el listado.
    /// En esta Story, la funcionalidad de parseo del directorio es un stub
    /// que se completa en la integración con el broker real.
    async fn list_inventory(&self, _range: &TimeRange) -> Result<Vec<crate::domain::BulkFileInfo>, FetchError> {
        // La enumeración del inventario real del broker se implementa
        // en la Story de integración con Binance Vision. Aquí devolvemos
        // una lista vacía como placeholder para que compile en producción.
        Ok(vec![])
    }

    /// Descarga un archivo Bulk comprimido (.zip) al path local dado.
    ///
    /// Descarga el contenido completo en memoria y luego lo escribe al disco.
    /// Para archivos muy grandes, este esquema se puede reemplazar por streaming
    /// en una mejora futura.
    async fn download_file(
        &self,
        file: &crate::domain::BulkFileInfo,
        dest_path: &Path,
    ) -> Result<u64, FetchError> {
        // Descarga el archivo ZIP del servidor remoto.
        let response = self
            .client
            .get(&file.download_url)
            .send()
            .await
            .map_err(|e| FetchError::BulkSourceFailed(e.to_string()))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| FetchError::BulkSourceFailed(e.to_string()))?;

        let bytes_len = bytes.len() as u64;

        // Escribe el ZIP al disco en el path de destino.
        tokio::fs::write(dest_path, &bytes)
            .await
            .map_err(|e| FetchError::Io(e.to_string()))?;

        Ok(bytes_len)
    }
}

/// Adaptador real de Delta REST que usa `reqwest` con `rustls-tls`.
#[allow(dead_code)] // No se conecta a la app todavía; se usa en producción.
pub struct ReqwestDeltaSource {
    client: reqwest::Client,
    endpoint_url: String,
    symbol: String,
    interval: String,
}

impl ReqwestDeltaSource {
    /// Crea el adaptador. Recibe el endpoint base, símbolo e intervalo.
    #[allow(dead_code)]
    pub fn new(
        endpoint_url: String,
        symbol: String,
        interval: String,
    ) -> Result<Self, FetchError> {
        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .build()
            .map_err(|e| FetchError::Internal(e.to_string()))?;
        Ok(Self { client, endpoint_url, symbol, interval })
    }
}

#[async_trait::async_trait]
impl DeltaSource for ReqwestDeltaSource {
    /// Descarga el tramo Delta de la API REST del broker.
    ///
    /// Convierte el rango de timestamps (nanosegundos) a milisegundos
    /// para la llamada a la API de Binance, que trabaja en milisegundos.
    async fn fetch_range(&self, range: &TimeRange) -> Result<Vec<u8>, FetchError> {
        // La API de Binance trabaja en milisegundos; convertimos desde nanosegundos.
        let start_ms = range.start_ns / 1_000_000;
        let end_ms = range.end_ns / 1_000_000;

        let url = format!(
            "{}/api/v3/klines?symbol={}&interval={}&startTime={}&endTime={}&limit=1000",
            self.endpoint_url, self.symbol, self.interval, start_ms, end_ms
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| FetchError::DeltaSourceFailed(e.to_string()))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| FetchError::DeltaSourceFailed(e.to_string()))
            .map(|b| b.to_vec())?;

        Ok(bytes)
    }
}
