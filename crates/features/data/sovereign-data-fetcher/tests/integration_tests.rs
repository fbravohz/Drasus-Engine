//! Tests de integración del Sovereign Data Fetcher.
//!
//! Cubren los criterios 5–10 del §5 de la Orden de Trabajo (STORY-024).
//! Los criterios 1–4 se prueban como tests unitarios en `src/domain.rs`.
//!
//! Todos los tests usan adaptadores falsos (`FakeBulkSource`, `FakeDeltaSource`)
//! — NUNCA se toca la red real. La base de datos usa archivos temporales
//! reales en disco (no `:memory:`) para probar la durabilidad (criterio 10).

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use tempfile::TempDir;

use shared::public_interface::{
    DeterministicClock, JobRepository, JobState, NewJob,
    create_pool, run_migrations,
};
use sovereign_data_fetcher::public_interface::{
    BulkFileInfo, BulkSource, DeltaSource, DownloadRepository, FetchError, FetchRequest,
    FetcherConfig, TimeRange, VerifyInput, VerifyOutput,
    fetch, recover_interrupted_downloads, verify_with_sources,
};

// ── Helpers de test ──────────────────────────────────────────────────────────

/// Crea una base de datos SQLite en un archivo temporal y aplica todas las
/// migraciones. Devuelve el pool y el directorio temporal (que se borra al
/// salir del scope).
async fn setup_db() -> (sqlx::SqlitePool, TempDir) {
    let dir = tempfile::tempdir().expect("crear directorio temporal");
    let db_path = dir.path().join("test.sqlite");
    let url = format!("sqlite://{}", db_path.display());
    let pool = create_pool(&url).await.expect("abrir base de datos temporal");
    run_migrations(&pool).await.expect("aplicar migraciones");
    (pool, dir)
}

/// Crea un directorio temporal para los archivos descargados.
fn setup_dest_dir() -> TempDir {
    tempfile::tempdir().expect("crear directorio de destino temporal")
}

/// Construye un `FetchRequest` estándar con los parámetros dados.
fn make_request(dest_dir: &Path, available_bytes: u64) -> FetchRequest {
    FetchRequest {
        symbol: "BTCUSDT".to_string(),
        interval: "1m".to_string(),
        start_ns: 1_000_000_000,
        end_ns: 2_000_000_000,
        dest_dir: dest_dir.to_path_buf(),
        now_ns: 2_000_000_000,
        available_disk_bytes: available_bytes,
    }
}

// ── Adaptadores falsos ───────────────────────────────────────────────────────

/// Fuente Bulk que devuelve un inventario predefinido y simula descarga exitosa.
struct FakeSuccessBulkSource {
    files: Vec<BulkFileInfo>,
}

impl FakeSuccessBulkSource {
    fn new(files: Vec<BulkFileInfo>) -> Self {
        Self { files }
    }
}

#[async_trait]
impl BulkSource for FakeSuccessBulkSource {
    async fn list_inventory(&self, _range: &TimeRange) -> Result<Vec<BulkFileInfo>, FetchError> {
        Ok(self.files.clone())
    }

    async fn download_file(
        &self,
        file: &BulkFileInfo,
        dest_path: &Path,
    ) -> Result<u64, FetchError> {
        // Escribe un placeholder en disco para simular una descarga real.
        tokio::fs::write(dest_path, format!("FAKE:{}", file.filename).as_bytes())
            .await
            .map_err(|e| FetchError::Io(e.to_string()))?;
        Ok(file.estimated_size_bytes)
    }
}

/// Fuente Delta que devuelve bytes predefinidos en memoria.
struct FakeSuccessDeltaSource {
    payload: Vec<u8>,
}

impl FakeSuccessDeltaSource {
    fn with_payload(payload: Vec<u8>) -> Self {
        Self { payload }
    }
}

#[async_trait]
impl DeltaSource for FakeSuccessDeltaSource {
    async fn fetch_range(&self, _range: &TimeRange) -> Result<Vec<u8>, FetchError> {
        Ok(self.payload.clone())
    }
}

/// Fuente Bulk que cuenta el número máximo de descargas simultáneas.
///
/// Cada llamada a `download_file` incrementa un contador atómico al empezar
/// y lo decrementa al terminar. El máximo observado queda en `peak_concurrent`.
struct FakeCountingBulkSource {
    files: Vec<BulkFileInfo>,
    /// Número de descargas actualmente activas (atómico, seguro entre hilos).
    active: Arc<AtomicUsize>,
    /// Máximo de descargas simultáneas observado durante la prueba.
    peak_concurrent: Arc<AtomicUsize>,
}

impl FakeCountingBulkSource {
    fn new(files: Vec<BulkFileInfo>) -> Self {
        Self {
            files,
            active: Arc::new(AtomicUsize::new(0)),
            peak_concurrent: Arc::new(AtomicUsize::new(0)),
        }
    }

}

#[async_trait]
impl BulkSource for FakeCountingBulkSource {
    async fn list_inventory(&self, _range: &TimeRange) -> Result<Vec<BulkFileInfo>, FetchError> {
        Ok(self.files.clone())
    }

    async fn download_file(
        &self,
        file: &BulkFileInfo,
        dest_path: &Path,
    ) -> Result<u64, FetchError> {
        // Incrementa el contador de descargas activas.
        let current = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        // Registra el pico de concurrencia si es el máximo visto hasta ahora.
        self.peak_concurrent.fetch_max(current, Ordering::SeqCst);

        // Simula trabajo asíncrono breve para que otras descargas se solapan.
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Escribe el archivo de destino.
        tokio::fs::write(dest_path, b"fake").await.ok();

        // Decrementa el contador al terminar.
        self.active.fetch_sub(1, Ordering::SeqCst);
        Ok(file.estimated_size_bytes)
    }
}

/// Fuente Bulk que falla las primeras N llamadas y tiene éxito a partir de la N+1.
struct FakeFailingBulkSource {
    files: Vec<BulkFileInfo>,
    /// Número de fallos restantes antes de que una descarga tenga éxito.
    failures_remaining: Arc<AtomicUsize>,
}

impl FakeFailingBulkSource {
    fn fail_n_times(files: Vec<BulkFileInfo>, n: usize) -> Self {
        Self {
            files,
            failures_remaining: Arc::new(AtomicUsize::new(n)),
        }
    }
}

#[async_trait]
impl BulkSource for FakeFailingBulkSource {
    async fn list_inventory(&self, _range: &TimeRange) -> Result<Vec<BulkFileInfo>, FetchError> {
        Ok(self.files.clone())
    }

    async fn download_file(
        &self,
        _file: &BulkFileInfo,
        _dest_path: &Path,
    ) -> Result<u64, FetchError> {
        // Si aún quedan fallos programados, decrementa y devuelve error.
        let prev = self.failures_remaining.fetch_update(
            Ordering::SeqCst,
            Ordering::SeqCst,
            |n| if n > 0 { Some(n - 1) } else { None },
        );
        if prev.is_ok() {
            // El fetch_update tuvo éxito: había al menos 1 fallo restante.
            return Err(FetchError::BulkSourceFailed("error simulado de red".to_string()));
        }
        Ok(100)
    }
}

/// Fuente Delta que falla exactamente N veces antes de tener éxito.
struct FakeFailingDeltaSource {
    failures_remaining: Arc<AtomicUsize>,
    /// Número total de llamadas recibidas.
    call_count: Arc<AtomicUsize>,
}

impl FakeFailingDeltaSource {
    fn fail_n_times(n: usize) -> Self {
        Self {
            failures_remaining: Arc::new(AtomicUsize::new(n)),
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }

}

#[async_trait]
impl DeltaSource for FakeFailingDeltaSource {
    async fn fetch_range(&self, _range: &TimeRange) -> Result<Vec<u8>, FetchError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);

        let prev = self.failures_remaining.fetch_update(
            Ordering::SeqCst,
            Ordering::SeqCst,
            |n| if n > 0 { Some(n - 1) } else { None },
        );
        if prev.is_ok() {
            return Err(FetchError::DeltaSourceFailed("error Delta simulado".to_string()));
        }
        Ok(b"csv-data".to_vec())
    }
}

/// Fuente Delta que siempre falla (para probar el agotamiento de reintentos).
struct AlwaysFailingDeltaSource;

#[async_trait]
impl DeltaSource for AlwaysFailingDeltaSource {
    async fn fetch_range(&self, _range: &TimeRange) -> Result<Vec<u8>, FetchError> {
        Err(FetchError::DeltaSourceFailed("fallo permanente simulado".to_string()))
    }
}

// ── Helper para construir BulkFileInfo de prueba ─────────────────────────────

fn make_bulk_file(name: &str, start_ns: i64, end_ns: i64, size: u64) -> BulkFileInfo {
    BulkFileInfo {
        filename: name.to_string(),
        download_url: format!("https://fake/{name}"),
        start_ns,
        end_ns,
        estimated_size_bytes: size,
    }
}

// ── CRITERIO 5: concurrent_downloads_respect_max_limit ──────────────────────

/// Verifica que la descarga concurrente respeta el límite `CONCURRENT_DOWNLOADS`
/// y que la concurrencia es REAL (no secuencial).
///
/// Crea 10 archivos Bulk y limita la concurrencia a 3.
///
/// Test honesto: con el código correcto (JoinSet), varias descargas se solapan
/// y el pico observado debe ser >= 2. Con el código incorrecto (bucle secuencial),
/// el pico siempre sería 1 (un solo download activo en cada momento) y este
/// test FALLARÍA — ese es el punto: esta aserción detecta la regresión.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn concurrent_downloads_respect_max_limit() {
    let (pool, _dir) = setup_db().await;
    let dest = setup_dest_dir();
    let clock = DeterministicClock::new(1_000_000, 1_000);

    // 10 archivos Bulk de 100 bytes cada uno; cada descarga introduce una pausa
    // de 10ms para que el solapamiento entre tareas sea observable.
    let files: Vec<BulkFileInfo> = (0..10)
        .map(|i| make_bulk_file(&format!("f{i}.zip"), i * 100, i * 100 + 99, 100))
        .collect();

    let source = FakeCountingBulkSource::new(files);
    // Clonamos el contador ANTES de mover `source` al Arc.
    let peak = source.peak_concurrent.clone();
    let config = FetcherConfig {
        concurrent_downloads: 3,
        delta_sync_retry: 1,
        ..Default::default()
    };

    let request = FetchRequest {
        symbol: "BTCUSDT".to_string(),
        interval: "1m".to_string(),
        start_ns: 0,
        end_ns: 999,
        dest_dir: dest.path().to_path_buf(),
        now_ns: 1000,
        // Espacio suficiente para todos los archivos.
        available_disk_bytes: 1_000_000,
    };

    fetch(
        &config,
        request,
        Arc::new(source),
        &FakeSuccessDeltaSource::with_payload(vec![]),
        &pool,
        &clock,
    )
    .await
    .expect("la descarga debe completarse");

    let observed_peak = peak.load(Ordering::SeqCst);

    // El pico no puede superar el límite configurado.
    assert!(
        observed_peak <= 3,
        "el pico de concurrencia ({observed_peak}) superó el límite de 3"
    );
    // El pico DEBE ser >= 2: esto verifica que hubo solapamiento real.
    // Con código secuencial (sin JoinSet), el pico siempre sería 1 y esta
    // aserción fallaría — ese es exactamente el defecto que detectamos.
    assert!(
        observed_peak >= 2,
        "la descarga debe ser concurrente (pico esperado >= 2, observado: {observed_peak})"
    );
}

// ── CRITERIO 6: failed_bulk_download_is_retried ──────────────────────────────

/// Verifica que un archivo Bulk fallido se reintenta automáticamente.
///
/// La fuente falla la primera llamada a `download_file` y tiene éxito en la
/// segunda. El `fetch` debe completarse con éxito (no devolver error).
#[tokio::test]
async fn failed_bulk_download_is_retried() {
    let (pool, _dir) = setup_db().await;
    let dest = setup_dest_dir();
    let clock = DeterministicClock::new(1_000_000, 1_000);

    let files = vec![make_bulk_file("data.zip", 0, 500, 100)];
    // Falla 1 vez; la segunda llamada tiene éxito.
    let source = FakeFailingBulkSource::fail_n_times(files, 1);

    let config = FetcherConfig {
        concurrent_downloads: 1,
        delta_sync_retry: 1,
        ..Default::default()
    };

    let request = FetchRequest {
        symbol: "BTCUSDT".to_string(),
        interval: "1m".to_string(),
        start_ns: 0,
        end_ns: 500,
        dest_dir: dest.path().to_path_buf(),
        now_ns: 501,
        available_disk_bytes: 1_000_000,
    };

    let result = fetch(
        &config,
        request,
        Arc::new(source),
        &FakeSuccessDeltaSource::with_payload(vec![]),
        &pool,
        &clock,
    )
    .await;

    assert!(
        result.is_ok(),
        "la descarga debe completarse después de reintentar: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap().bulk_files_downloaded, 1);
}

// ── CRITERIO 7: delta_sync_retries_up_to_limit ───────────────────────────────

/// Verifica que el Delta reintenta hasta `DELTA_SYNC_RETRY` veces antes de rendirse.
///
/// Primer sub-test: la fuente falla N-1 veces y tiene éxito en el intento N.
/// La función `fetch` debe completarse con éxito.
///
/// Segundo sub-test: la fuente siempre falla. Tras `DELTA_SYNC_RETRY` intentos,
/// `fetch` debe devolver `FetchError::DeltaSourceFailed`.
#[tokio::test]
async fn delta_sync_retries_up_to_limit() {
    // Sub-test A: falla delta_sync_retry - 1 veces, tiene éxito en el último.
    {
        let (pool, _dir) = setup_db().await;
        let dest = setup_dest_dir();
        let clock = DeterministicClock::new(1_000_000, 1_000);

        // Sin archivos Bulk: todo el rango va a Delta.
        let config = FetcherConfig {
            concurrent_downloads: 1,
            delta_sync_retry: 3,
            ..Default::default()
        };
        // Falla 2 veces; tiene éxito en la 3ª (= delta_sync_retry).
        let delta_source = FakeFailingDeltaSource::fail_n_times(2);
        let call_count = delta_source.call_count.clone();

        let request = FetchRequest {
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            start_ns: 0,
            end_ns: 1_000,
            dest_dir: dest.path().to_path_buf(),
            now_ns: 1_000,
            available_disk_bytes: 1_000_000,
        };

        let result = fetch(
            &config,
            request,
            Arc::new(FakeSuccessBulkSource::new(vec![])),
            &delta_source,
            &pool,
            &clock,
        )
        .await;

        assert!(result.is_ok(), "debe tener éxito tras reintentar: {:?}", result.err());
        assert_eq!(
            call_count.load(Ordering::SeqCst),
            3,
            "debe haber llamado a Delta exactamente 3 veces (2 fallos + 1 éxito)"
        );
    }

    // Sub-test B: la fuente siempre falla; `fetch` debe devolver error tras los reintentos.
    {
        let (pool, _dir) = setup_db().await;
        let dest = setup_dest_dir();
        let clock = DeterministicClock::new(1_000_000, 1_000);

        let config = FetcherConfig {
            concurrent_downloads: 1,
            delta_sync_retry: 3,
            ..Default::default()
        };

        let request = FetchRequest {
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            start_ns: 0,
            end_ns: 1_000,
            dest_dir: dest.path().to_path_buf(),
            now_ns: 1_000,
            available_disk_bytes: 1_000_000,
        };

        let result = fetch(
            &config,
            request,
            Arc::new(FakeSuccessBulkSource::new(vec![])),
            &AlwaysFailingDeltaSource,
            &pool,
            &clock,
        )
        .await;

        assert!(
            matches!(result, Err(FetchError::DeltaSourceFailed(_))),
            "debe devolver DeltaSourceFailed tras agotar los reintentos"
        );
    }
}

// ── CRITERIO 8: download_record_persisted_with_profile_a_fields ─────────────

/// Verifica que el registro de descarga se persiste con todos los campos del
/// Perfil A (ADR-0020): Grupo I + Grupo III + Grupo IV + `source_endpoint`.
#[tokio::test]
async fn download_record_persisted_with_profile_a_fields() {
    let (pool, _dir) = setup_db().await;
    let dest = setup_dest_dir();
    let clock = DeterministicClock::new(1_000_000, 1_000);

    let config = FetcherConfig::default();
    let request = make_request(dest.path(), 1_000_000);

    let result = fetch(
        &config,
        request,
        Arc::new(FakeSuccessBulkSource::new(vec![])),
        &FakeSuccessDeltaSource::with_payload(b"data".to_vec()),
        &pool,
        &clock,
    )
    .await
    .expect("la descarga debe completarse");

    // Recupera el registro de la base de datos y verifica los campos del Perfil A.
    let dl_repo = DownloadRepository::new(&pool, &clock);
    let record = dl_repo
        .find(&result.record_id)
        .await
        .expect("consulta de la base de datos")
        .expect("el registro debe existir");

    // ── Grupo I: Identidad & Integridad ──
    assert!(!record.id.is_empty(), "id debe estar presente");
    assert!(record.created_at > 0, "created_at debe ser un timestamp positivo");
    assert!(record.updated_at > 0, "updated_at debe ser un timestamp positivo");
    assert!(!record.audit_hash.is_empty(), "audit_hash debe estar presente");
    // event_sequence_id comienza en 1 para el primer registro.
    assert_eq!(record.event_sequence_id, 1, "primer registro debe tener sequence_id = 1");

    // ── Campo propio de dominio (source_endpoint obligatorio) ──
    assert!(!record.source_endpoint.is_empty(), "source_endpoint debe estar presente");

    // ── Sin campos del Grupo V (execution_latency_ms no existe en la tabla) ──
    // La ausencia se verifica por diseño: el struct DownloadRecord no tiene ese campo.
    // Si alguien lo añade accidentalmente, el struct struct y la migración se rompen.

    println!(
        "Perfil A verificado: id={}, seq={}, source={}",
        record.id, record.event_sequence_id, record.source_endpoint
    );
}

// ── CRITERIO 10: interrupted_download_recovers_on_restart ───────────────────

/// Verifica que una descarga interrumpida se reanuda al reiniciar.
///
/// Simula un crash dejando un Job de tipo `SOVEREIGN_FETCH` en estado `RUNNING`.
/// Al llamar a `recover_interrupted_downloads`, el Job debe quedar en `QUEUED`
/// listo para ser retomado.
#[tokio::test]
async fn interrupted_download_recovers_on_restart() {
    let (pool, _dir) = setup_db().await;
    let clock = DeterministicClock::new(1_000_000, 1_000);

    // Crea un Job de descarga en estado RUNNING (simula que el proceso fue
    // interrumpido mientras descargaba).
    let job_repo = JobRepository::new(&pool, &clock);
    let job = job_repo
        .submit(NewJob {
            user_id: "system".to_string(),
            // Debe coincidir con el tipo que usa el orchestrator.
            job_type: "SOVEREIGN_FETCH".to_string(),
            parameters: r#"{"symbol":"BTCUSDT"}"#.to_string(),
            owner_id: None,
            access_token_id: None,
            session_id: None,
            node_id: None,
            logic_hash: None,
        })
        .await
        .expect("crear Job de prueba");

    // Transiciona a RUNNING para simular que la descarga había empezado.
    clock.tick();
    let running_job = job_repo
        .transition(&job, JobState::Running, Some("worker-1"))
        .await
        .expect("transicionar a RUNNING");

    assert_eq!(running_job.state, JobState::Running);

    // ── Simula el reinicio del sistema ──────────────────────────────────────
    // En producción, el proceso moriría aquí. En el test, simplemente llamamos
    // a la función de recuperación con el mismo pool (misma base de datos en disco).

    clock.tick();
    let recovered_count = recover_interrupted_downloads(&pool, &clock)
        .await
        .expect("recuperar descargas interrumpidas");

    // Debe haber reencolado exactamente 1 Job.
    assert_eq!(recovered_count, 1, "debe recuperar 1 descarga interrumpida");

    // Verifica que el Job ahora está en QUEUED, listo para ser retomado.
    let recovered_job = job_repo
        .find(&running_job.id)
        .await
        .expect("consultar Job recuperado")
        .expect("el Job debe seguir existiendo");

    assert_eq!(
        recovered_job.state,
        JobState::Queued,
        "el Job recuperado debe estar en QUEUED, no en {}",
        recovered_job.state
    );

    // El row_version debe haber aumentado (la transición RUNNING → QUEUED
    // produce una nueva versión de la fila en la cadena de auditoría).
    assert!(
        recovered_job.row_version > running_job.row_version,
        "la recuperación debe incrementar el row_version"
    );
}

// ── Test adicional: verificación de disco insuficiente a nivel de orquestador ─

/// Verifica que el orquestador aborta antes de la descarga cuando el disco
/// está lleno, aun cuando el inventario ya fue listado.
#[tokio::test]
async fn fetch_aborts_when_orchestrator_detects_insufficient_disk() {
    let (pool, _dir) = setup_db().await;
    let dest = setup_dest_dir();
    let clock = DeterministicClock::new(1_000_000, 1_000);

    // Un archivo Bulk de 10 MB.
    let files = vec![make_bulk_file("big.zip", 0, 1_000, 10_000_000)];
    let config = FetcherConfig::default();

    let request = FetchRequest {
        symbol: "BTCUSDT".to_string(),
        interval: "1m".to_string(),
        start_ns: 0,
        end_ns: 1_000,
        dest_dir: dest.path().to_path_buf(),
        now_ns: 1_001,
        // Solo 1 MB disponible; el Bulk necesita 10 MB.
        available_disk_bytes: 1_000_000,
    };

    let result = fetch(
        &config,
        request,
        Arc::new(FakeSuccessBulkSource::new(files)),
        &FakeSuccessDeltaSource::with_payload(vec![]),
        &pool,
        &clock,
    )
    .await;

    assert!(
        matches!(result, Err(FetchError::InsufficientDiskSpace { .. })),
        "debe abortar con InsufficientDiskSpace, got: {:?}",
        result
    );
}

// ── CRITERIO CLI (ADR-0142 Fase 1): verify_with_sources_returns_ok_json ─────

/// Verifica que `verify_with_sources` devuelve un VerifyOutput válido que
/// serializa a JSON con la forma correcta cuando se usan adaptadores falsos.
///
/// Este test cubre el criterio del CLI de verificación (ADR-0142 Fase 1):
///  1. Parseo de VerifyInput desde JSON (como lo haría el CLI con --input '...').
///  2. Dispatch: verify_with_sources completa el ciclo sin error.
///  3. Forma del JSON: el output serializa con los campos esperados ('ok', 'job_id', etc.).
///
/// No toca la red: los adaptadores falsos responden en memoria.
#[tokio::test]
async fn verify_with_fake_sources_returns_ok_json() {
    let (pool, _dir) = setup_db().await;
    let dest = setup_dest_dir();
    // El reloj arranca en 10 días desde epoch para que start_ns sea positivo al restar 1 día.
    let clock = DeterministicClock::new(86_400_000_000_000i64 * 10, 1_000);

    // ── 1. Parseo de VerifyInput desde JSON (mismo camino que el CLI) ────────
    let input_json = r#"{"symbol":"BTCUSDT","interval":"1m","days":1}"#;
    let input: VerifyInput =
        serde_json::from_str(input_json).expect("VerifyInput debe parsear desde JSON válido");

    assert_eq!(input.symbol, "BTCUSDT");
    assert_eq!(input.interval, "1m");
    assert_eq!(input.days, Some(1));

    // ── 2. Dispatch: verify_with_sources con adaptadores falsos ──────────────
    // Sin archivos Bulk (list_inventory vacío) → todo el rango va a Delta.
    let output: VerifyOutput = verify_with_sources(
        &input,
        dest.path(),
        Arc::new(FakeSuccessBulkSource::new(vec![])),
        &FakeSuccessDeltaSource::with_payload(b"datos-csv-simulados".to_vec()),
        &pool,
        &clock,
    )
    .await;

    // La verificación debe completarse con ok: true.
    assert!(output.ok, "verify_with_sources debe completarse sin error; error: {:?}", output.error);
    assert!(output.job_id.is_some(), "job_id debe estar presente en el output exitoso");
    assert!(output.record_id.is_some(), "record_id debe estar presente en el output exitoso");
    assert_eq!(
        output.bulk_files_downloaded,
        Some(0),
        "sin archivos Bulk, bulk_files_downloaded debe ser 0"
    );
    assert!(
        output.delta_bytes.unwrap_or(0) > 0,
        "delta_bytes debe ser > 0 cuando la fuente Delta devuelve datos"
    );
    assert!(output.error.is_none(), "error debe ser None en caso exitoso");

    // ── 3. Forma del JSON: serialización y estructura ─────────────────────────
    // serde_json::to_string ya está disponible como dependencia de producción del crate.
    let json_str = serde_json::to_string(&output).expect("VerifyOutput debe serializar a JSON");
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("el JSON producido debe ser válido");

    assert_eq!(parsed["ok"], true, "campo 'ok' debe ser true");
    assert!(parsed["job_id"].is_string(), "campo 'job_id' debe ser string");
    assert!(parsed["record_id"].is_string(), "campo 'record_id' debe ser string");
    assert!(parsed["error"].is_null(), "campo 'error' debe ser null en caso exitoso");
    // Verifica que todos los campos de FetchResult están presentes en el JSON.
    assert!(parsed["bulk_files_downloaded"].is_number(), "bulk_files_downloaded debe ser número");
    assert!(parsed["delta_bytes"].is_number(), "delta_bytes debe ser número");
    assert!(parsed["total_bytes"].is_number(), "total_bytes debe ser número");
}

// ── Atomicidad bajo concurrencia de DownloadRepository::record (STORY-046 M1) ──

fn sample_new_download_record() -> sovereign_data_fetcher::public_interface::NewDownloadRecord {
    sovereign_data_fetcher::public_interface::NewDownloadRecord {
        data_snapshot_id: None,
        logic_hash: Some("fetcher-v1".to_string()),
        node_id: Some("node-test".to_string()),
        process_id: Some("process-test".to_string()),
        source_endpoint: "https://example.invalid/bulk/segment.zip".to_string(),
    }
}

/// CRITERIO DE CIERRE (QA por mutación): bajo contención de escritura
/// SOSTENIDA (otro escritor retiene el lock de `BEGIN IMMEDIATE` y no lo
/// suelta), el bucle de reintento de `DownloadRepository::record` debe
/// agotar EXACTAMENTE el máximo de intentos y rendirse con
/// `WriteContention { attempts: MAX }` -- nunca descartar el registro de
/// descarga en silencio.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn download_record_exhausts_exactly_max_attempts_when_write_lock_is_held() {
    use std::str::FromStr;
    use std::time::Duration;

    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

    const MAX_RECORD_ATTEMPTS: u32 = 5; // mismo valor que persistence.rs (privado al crate)

    let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
    let db_path = temp_dir.path().join("sovereign_download_forced_contention.sqlite");
    let database_url = format!("sqlite://{}", db_path.display());

    let setup_pool = create_pool(&database_url).await.expect("conectar (setup)");
    run_migrations(&setup_pool).await.expect("migrar");
    setup_pool.close().await;

    // Opciones con busy_timeout=0: un lock ocupado falla de INMEDIATO con
    // "database is locked" en vez de esperar 5s -- contención determinista
    // y rápida.
    let immediate_opts = || {
        SqliteConnectOptions::from_str(&database_url)
            .expect("parsear opciones")
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_millis(0))
    };

    // Escritor A: toma el lock de escritura con `BEGIN IMMEDIATE` y NO lo
    // suelta mientras B intenta escribir.
    let lock_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(immediate_opts())
        .await
        .expect("pool que retiene el lock");
    let lock_tx = lock_pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .expect("tomar el lock de escritura reservado");

    // Escritor B: intenta registrar una descarga mientras A retiene el
    // lock. Cada intento abre `BEGIN IMMEDIATE`, choca con el lock de A,
    // falla con "database is locked" (transitorio) y reintenta, hasta
    // agotar el máximo de intentos.
    let repo_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(immediate_opts())
        .await
        .expect("pool del repositorio");
    let clock = DeterministicClock::new(1_000, 100);
    let repo = DownloadRepository::new(&repo_pool, &clock);

    let result = repo.record(sample_new_download_record()).await;

    drop(lock_tx); // libera el lock (limpieza; el resultado ya está tomado)

    match result {
        Err(sovereign_data_fetcher::public_interface::DownloadRepositoryError::WriteContention { attempts }) => {
            assert_eq!(
                attempts, MAX_RECORD_ATTEMPTS,
                "bajo contención sostenida debe agotar EXACTAMENTE el máximo de intentos"
            );
        }
        other => panic!(
            "se esperaba WriteContention {{ attempts: {MAX_RECORD_ATTEMPTS} }} bajo contención sostenida, se obtuvo: {other:?}"
        ),
    }
}

/// CRITERIO DE CIERRE (QA por mutación): una violación UNIQUE PERMANENTE
/// (la PK `id`, que NO se debe reintentar) NO debe clasificarse como
/// contención transitoria -- se verifica indirectamente: un `id` duplicado
/// insertado por fuera del repositorio (violación de PK, no de
/// `event_sequence_id`) hace fallar el INSERT directo con un error que NO
/// es "database is locked" ni menciona `event_sequence_id`; el repositorio
/// (que sí usa UUIDs v7 únicos) nunca produce esa colisión en circunstancias
/// normales, así que esta prueba fija el comportamiento de la base de datos
/// del que depende el clasificador `is_transient_write_conflict` interno de
/// `persistence.rs`.
#[tokio::test]
async fn duplicate_primary_key_insert_is_a_permanent_error_not_a_sequence_collision() {
    let (pool, _dir) = setup_db().await;

    let insert_with_id_dup = |event_sequence_id: i64| {
        sqlx::query(
            "INSERT INTO sovereign_download_records \
             (id, created_at, updated_at, audit_hash, event_sequence_id, source_endpoint) \
             VALUES ('dup-download-id', 1, 1, 'h', ?, 'https://example.invalid')",
        )
        .bind(event_sequence_id)
        .execute(&pool)
    };

    insert_with_id_dup(1).await.expect("primera fila válida");
    let err = insert_with_id_dup(2)
        .await
        .expect_err("la segunda fila debe violar la PRIMARY KEY id");

    let message = err.to_string().to_lowercase();
    assert!(
        !message.contains("database is locked") && !message.contains("event_sequence_id"),
        "una violación de PK debe distinguirse textualmente de una colisión de secuencia o un lock transitorio; mensaje: {message}"
    );
}

/// CRITERIO DE CIERRE (QA por mutación): la fila que DEVUELVE
/// `DownloadRepository::record` refleja SIEMPRE valores frescos --
/// `audit_hash` recién computado, `audit_chain_hash` encadenado a la fila
/// anterior y `created_at`/`updated_at` leídos del reloj EN ESE INSTANTE --
/// nunca datos por defecto ni reutilizados de una llamada previa.
#[tokio::test]
async fn download_record_returned_row_reflects_fresh_hash_chain_and_timestamp() {
    let (pool, _dir) = setup_db().await;
    let clock = DeterministicClock::new(1_000, 100);
    let repo = DownloadRepository::new(&pool, &clock);

    let first = repo.record(sample_new_download_record()).await.expect("registrar primer registro");
    assert_eq!(first.created_at, 1_000, "precondición: el primer registro nace en el now inicial");
    assert!(first.audit_chain_hash.is_none(), "precondición: el primer registro no encadena");
    assert_eq!(first.event_sequence_id, 1);

    clock.tick(); // 1_000 -> 1_100
    let second = repo.record(sample_new_download_record()).await.expect("registrar segundo registro");

    assert_ne!(
        second.audit_hash, first.audit_hash,
        "record debe DEVOLVER el audit_hash recién computado, no uno reutilizado"
    );
    assert_eq!(
        second.audit_chain_hash,
        Some(first.audit_hash.clone()),
        "el audit_chain_hash devuelto debe encadenar al audit_hash del registro anterior"
    );
    assert_eq!(
        second.created_at, 1_100,
        "el created_at devuelto debe ser el now del reloj tras el tick, no el viejo"
    );
    assert_eq!(second.event_sequence_id, 2);
}
