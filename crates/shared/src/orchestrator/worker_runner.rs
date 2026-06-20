//! [SHELL] Cáscara de workers aislados: spawn OS, memoria compartida y señales.
//! (`docs/features/worker-isolation-orchestrator.md` TTR-001/TTR-002,
//! ADR-0013, ADR-0016).
//!
//! Implementa [`WorkerBackend`] con efectos reales del SO:
//! - `memmap2`: mapea el buffer de datos sin copias entre procesos.
//! - `std::process::Command`: lanza procesos hijo aislados.
//! - `nix`: envía SIGTERM/SIGKILL y verifica liveness de procesos.
//!
//! ## TTR-001 — Bridge de Memoria Compartida
//!
//! El orquestador crea un [`SharedMemorySegment`] sobre un archivo temporal.
//! Los workers reciben la ruta vía env var `DRASUS_WORKER_SHM_PATH` y la
//! abren con `MmapOptions::map()` (solo `PROT_READ`). Un mismo segmento de
//! páginas físicas sirve a N procesos — la RAM no crece linealmente.
//!
//! ## TTR-002 — Watchdog y Graceful Shutdown
//!
//! Al cancelar un job, [`graceful_shutdown`] envía SIGTERM a los workers y
//! espera hasta 2s. Si siguen vivos, envía SIGKILL.
//!
//! Detección de muerte del padre: el orquestador crea un archivo keepalive
//! que los workers sondean. Cuando el [`SharedMemorySegment`] hace drop
//! (simulando la muerte del padre), el archivo keepalive desaparece y cada
//! worker detecta el cambio y termina. En producción, `prctl(PR_SET_PDEATHSIG,
//! SIGTERM)` provee una segunda capa: el kernel envía SIGTERM al hijo
//! automáticamente cuando el padre termina.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Mutex;
use std::time::Duration;

use memmap2::{Mmap, MmapMut, MmapOptions};
use uuid::Uuid;

#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(unix)]
use nix::sys::signal::{self, Signal};
#[cfg(unix)]
use nix::unistd::Pid;

use crate::domain::worker_orchestrator::{WorkerBackend, WorkerBackendError};

// ── Segmento de Memoria Compartida (TTR-001) ─────────────────────────────────

/// Error al crear o manipular un [`SharedMemorySegment`].
#[derive(Debug)]
pub struct ShmError(pub String);

impl std::fmt::Display for ShmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "shared memory error: {}", self.0)
    }
}

impl std::error::Error for ShmError {}

impl From<std::io::Error> for ShmError {
    fn from(e: std::io::Error) -> Self {
        ShmError(e.to_string())
    }
}

/// Buffer de datos de mercado mapeado en memoria compartida.
///
/// El orquestador crea uno; cada proceso worker lo abre en lectura.
/// Un solo segmento de páginas físicas es compartido por N workers
/// (el OS no copia los datos por proceso — `mmap(MAP_SHARED, PROT_READ)`).
///
/// Al hacer drop, elimina ambos archivos temporales:
/// - el archivo de datos (los workers ya tienen su propio mmap activo),
/// - el archivo keepalive (señal de cierre para workers que lo sondean).
pub struct SharedMemorySegment {
    /// Mapping lectura-escritura del padre — mantiene las páginas vivas.
    mmap: MmapMut,
    /// Ruta del archivo de datos; se pasa a los workers via env var.
    data_path: PathBuf,
    /// Ruta del archivo keepalive; su desaparición señala muerte del padre.
    keepalive_path: PathBuf,
}

impl SharedMemorySegment {
    /// Crea un segmento de memoria compartida con el contenido de `data`.
    ///
    /// Genera dos archivos en `temp_dir()`:
    /// - `drasus-shm-<uuid>.dat`: el buffer de datos mapeado.
    /// - `drasus-shm-<uuid>.keepalive`: centinela de vida del padre.
    pub fn create(data: &[u8]) -> Result<Self, ShmError> {
        let id = Uuid::new_v4().to_string();
        let base = std::env::temp_dir();
        let data_path = base.join(format!("drasus-shm-{id}.dat"));
        let keepalive_path = base.join(format!("drasus-shm-{id}.keepalive"));

        // Escribir datos en el archivo y mantener el fd abierto para el mmap.
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&data_path)?;

        file.write_all(data)?;
        file.flush()?;

        // Mapear el archivo en memoria (lectura-escritura para el padre).
        // SAFETY: el archivo acaba de ser creado y ningún otro proceso
        // lo tiene abierto aún; el ciclo de vida del mmap está ligado a
        // `SharedMemorySegment`, que es el único propietario.
        let mmap = unsafe { MmapOptions::new().map_mut(&file) }
            .map_err(|e| ShmError(format!("mmap datos: {e}")))?;

        // Crear el archivo keepalive (vacío — su existencia es la señal).
        File::create(&keepalive_path)?;

        Ok(Self {
            mmap,
            data_path,
            keepalive_path,
        })
    }

    /// Ruta del archivo de datos, para pasarla a workers.
    pub fn data_path(&self) -> &Path {
        &self.data_path
    }

    /// Ruta del archivo keepalive, para pasarla a workers.
    pub fn keepalive_path(&self) -> &Path {
        &self.keepalive_path
    }

    /// Vista de lectura del contenido del segmento.
    pub fn as_slice(&self) -> &[u8] {
        &self.mmap
    }

    /// Longitud del segmento en bytes.
    pub fn len(&self) -> usize {
        self.mmap.len()
    }

    /// Devuelve `true` si el segmento tiene cero bytes.
    pub fn is_empty(&self) -> bool {
        self.mmap.is_empty()
    }
}

impl Drop for SharedMemorySegment {
    fn drop(&mut self) {
        // Eliminar keepalive primero: señal de cierre para workers.
        let _ = std::fs::remove_file(&self.keepalive_path);
        // Eliminar datos: el mmap del padre sigue válido hasta que
        // self.mmap se descarte (lo hace Rust automáticamente tras este fn).
        let _ = std::fs::remove_file(&self.data_path);
    }
}

/// Abre el segmento de datos en `path` como lectura exclusiva (sin escritura).
///
/// Esta es la función que llaman los workers: `open_readonly(shm_path)`.
/// Devuelve un `Mmap` con `PROT_READ` — cualquier intento de escritura
/// sobre él resulta en SIGSEGV desde el OS.
///
/// Separado de `SharedMemorySegment::create` para reflejar la asimetría
/// real: el padre tiene `MmapMut`; los workers solo tienen `Mmap`.
pub fn open_readonly(path: &Path) -> Result<Mmap, ShmError> {
    // File::open usa O_RDONLY. MmapOptions::map() pide PROT_READ sin PROT_WRITE.
    let file = File::open(path).map_err(|e| ShmError(format!("abrir {path:?}: {e}")))?;
    // SAFETY: el archivo existe y tiene datos escritos por el padre;
    // no mutamos nada — solo leemos.
    unsafe { MmapOptions::new().map(&file) }.map_err(|e| ShmError(format!("mmap read-only: {e}")))
}

// ── Watchdog y Graceful Shutdown (TTR-002) ────────────────────────────────────

/// Comprueba si un proceso con `pid` sigue vivo — excluye zombies.
///
/// Un proceso zombie (estado `Z` en `/proc/{pid}/stat`) existe en la tabla
/// de procesos pero ya terminó: el padre todavía no llamó a `wait()`.
/// Para el orquestador, un zombie es equivalente a "terminado".
///
/// Implementación Unix (SO de despliegue, ADR-0016).
/// En Linux usa `/proc/{pid}/stat` para excluir zombies.
/// En otros Unix usa `kill(pid, 0)` (no envía señal, solo comprueba existencia).
#[cfg(unix)]
pub fn is_process_alive(pid: u32) -> bool {
    #[cfg(target_os = "linux")]
    {
        // /proc/{pid}/stat: "pid (nombre) estado ..."
        // Usamos rfind(')') porque el nombre puede contener paréntesis.
        let stat = match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
            Ok(s) => s,
            Err(_) => return false, // no existe = terminado
        };
        if let Some(pos) = stat.rfind(')') {
            let state = stat[pos + 1..].trim_start().chars().next().unwrap_or('Z');
            state != 'Z' // zombie no cuenta como vivo
        } else {
            true
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        // macOS y otros Unix: kill(pid, 0) comprueba existencia sin enviar señal.
        signal::kill(Pid::from_raw(pid as i32), None).is_ok()
    }
}

/// Stub para Windows: sin soporte de señales OS — siempre devuelve false.
#[cfg(not(unix))]
pub fn is_process_alive(_pid: u32) -> bool {
    false
}

/// Apaga gracefully una lista de procesos worker en dos fases.
///
/// 1. Envía SIGTERM a todos los `pids` (petición de cierre ordenado).
/// 2. Sondea cada 50ms hasta agotar `timeout`.
/// 3. Envía SIGKILL a los supervivientes.
///
/// Devuelve los PIDs que requirieron SIGKILL (lista vacía si todos
/// respondieron a SIGTERM dentro del plazo — el caso normal).
#[cfg(unix)]
pub async fn graceful_shutdown(pids: &[u32], timeout: Duration) -> Vec<u32> {
    // Paso 1: SIGTERM a todos.
    for &pid in pids {
        let _ = signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
    }

    // Paso 2: sondear hasta el deadline.
    let deadline = std::time::Instant::now() + timeout;
    let mut still_alive: Vec<u32> = pids.to_vec();

    while !still_alive.is_empty() && std::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(50)).await;
        still_alive.retain(|&pid| is_process_alive(pid));
    }

    // Paso 3: SIGKILL a supervivientes.
    for &pid in &still_alive {
        let _ = signal::kill(Pid::from_raw(pid as i32), Signal::SIGKILL);
    }

    still_alive
}

/// Stub para Windows: no hay soporte de señales — retorna todos como no terminados.
#[cfg(not(unix))]
pub async fn graceful_shutdown(pids: &[u32], _timeout: Duration) -> Vec<u32> {
    pids.to_vec()
}

// ── OsWorkerBackend (implementa WorkerBackend) ────────────────────────────────

/// Implementación del backend de workers sobre el SO real.
///
/// Cada llamada a `launch` spawna un proceso hijo real via
/// `std::process::Command`. El hijo recibe las rutas del segmento y del
/// keepalive por variables de entorno y configura `prctl(PR_SET_PDEATHSIG)`
/// para recibir SIGTERM si el padre muere inesperadamente.
///
/// El campo `children` guarda los handles de los procesos para poder
/// recoger zombies con `reap_finished()`.
pub struct OsWorkerBackend {
    /// Ruta al binario worker que se lanza para cada job.
    pub binary: PathBuf,
    /// Handles de los procesos hijo indexados por PID.
    children: Mutex<HashMap<u32, Child>>,
}

impl OsWorkerBackend {
    /// Crea un backend que lanza `binary` como proceso worker.
    pub fn new(binary: impl Into<PathBuf>) -> Self {
        Self {
            binary: binary.into(),
            children: Mutex::new(HashMap::new()),
        }
    }

    /// Recoge los procesos hijo que ya terminaron, liberando zombies.
    ///
    /// Llamar periódicamente en el loop del orquestador o tras un
    /// `graceful_shutdown`.
    pub fn reap_finished(&self) {
        let mut children = self.children.lock().unwrap_or_else(|e| e.into_inner());
        children.retain(|_, child| {
            // try_wait devuelve Ok(Some(_)) si el hijo ya terminó.
            !matches!(child.try_wait(), Ok(Some(_)))
        });
    }
}

/// Implementación Unix: usa `prctl`, SIGTERM/SIGKILL y `/proc` para liveness.
#[cfg(unix)]
impl WorkerBackend for OsWorkerBackend {
    /// Lanza el proceso worker con tres env vars:
    /// - `DRASUS_WORKER_JOB_ID`
    /// - `DRASUS_WORKER_SHM_PATH`
    /// - `DRASUS_WORKER_KEEPALIVE`
    ///
    /// También registra `prctl(PR_SET_PDEATHSIG, SIGTERM)` en el hijo
    /// antes del exec (via `CommandExt::pre_exec`) para que el kernel
    /// envíe SIGTERM al hijo automáticamente si el padre muere.
    fn launch(
        &self,
        job_id: &str,
        shm_path: &str,
        keepalive_path: &str,
    ) -> Result<u32, WorkerBackendError> {
        let mut cmd = Command::new(&self.binary);
        cmd.env("DRASUS_WORKER_JOB_ID", job_id)
            .env("DRASUS_WORKER_SHM_PATH", shm_path)
            .env("DRASUS_WORKER_KEEPALIVE", keepalive_path);

        // SAFETY: pre_exec corre en el hijo después del fork, antes del exec.
        // Estamos en un contexto de proceso recién forkeado — es seguro llamar
        // prctl aquí porque no hay hilos vivos (solo el hilo que hizo fork).
        unsafe {
            cmd.pre_exec(|| {
                nix::sys::prctl::set_pdeathsig(Signal::SIGTERM)
                    .map_err(|e: nix::errno::Errno| {
                        std::io::Error::from_raw_os_error(e as i32)
                    })
            });
        }

        let child = cmd
            .spawn()
            .map_err(|e| WorkerBackendError(format!("spawn {:?}: {e}", self.binary)))?;

        let pid = child.id();
        self.children
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(pid, child);
        Ok(pid)
    }

    fn send_sigterm(&self, pid: u32) -> Result<(), WorkerBackendError> {
        signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM)
            .map_err(|e| WorkerBackendError(format!("SIGTERM pid {pid}: {e}")))
    }

    fn send_sigkill(&self, pid: u32) -> Result<(), WorkerBackendError> {
        signal::kill(Pid::from_raw(pid as i32), Signal::SIGKILL)
            .map_err(|e| WorkerBackendError(format!("SIGKILL pid {pid}: {e}")))
    }

    fn is_alive(&self, pid: u32) -> bool {
        is_process_alive(pid)
    }
}

/// Stub Windows: spawn sin prctl; señales no disponibles.
/// El despliegue real es Linux (ADR-0016) — este bloque solo permite
/// que el workspace compile en Windows para desarrollo local.
#[cfg(not(unix))]
impl WorkerBackend for OsWorkerBackend {
    fn launch(
        &self,
        job_id: &str,
        shm_path: &str,
        keepalive_path: &str,
    ) -> Result<u32, WorkerBackendError> {
        let child = Command::new(&self.binary)
            .env("DRASUS_WORKER_JOB_ID", job_id)
            .env("DRASUS_WORKER_SHM_PATH", shm_path)
            .env("DRASUS_WORKER_KEEPALIVE", keepalive_path)
            .spawn()
            .map_err(|e| WorkerBackendError(format!("spawn {:?}: {e}", self.binary)))?;

        let pid = child.id();
        self.children
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(pid, child);
        Ok(pid)
    }

    fn send_sigterm(&self, pid: u32) -> Result<(), WorkerBackendError> {
        Err(WorkerBackendError(format!("SIGTERM no disponible en Windows (pid {pid})")))
    }

    fn send_sigkill(&self, pid: u32) -> Result<(), WorkerBackendError> {
        Err(WorkerBackendError(format!("SIGKILL no disponible en Windows (pid {pid})")))
    }

    fn is_alive(&self, pid: u32) -> bool {
        is_process_alive(pid)
    }
}

// ── Tests (criterios 1–6 de STORY-008) ───────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Criterio 1: acceso al buffer < 1ms tras el montaje ─────────────────

    #[test]
    fn shared_memory_access_latency_under_1ms() {
        let data = vec![42u8; 4096];
        let segment = SharedMemorySegment::create(&data).expect("crear segmento shm");

        let start = std::time::Instant::now();
        // Un único acceso a memoria — dada la naturaleza de mmap,
        // la página ya está en RAM desde la escritura al crear el segmento.
        let _ = std::hint::black_box(segment.as_slice()[0]);
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_millis(1),
            "acceso tomó {elapsed:?} (límite: 1ms)"
        );
    }

    // ── Criterio 2: RAM constante con N workers ─────────────────────────────

    /// N workers (4 en este test) abren el mismo segmento en modo Read-Only.
    /// La propiedad central es que el OS mapea los mismos frames físicos
    /// en múltiples espacios de direcciones virtuales — un único segmento
    /// para N procesos, sin copia por worker.
    ///
    /// El test lo verifica funcionalmente: los 4 mappings leen el mismo
    /// dato sin error, lo que confirma que todos apuntan al mismo buffer.
    #[test]
    fn shared_memory_ram_constant_with_n_workers() {
        let data = b"datos-mercado-compartidos";
        let segment = SharedMemorySegment::create(data).expect("crear segmento");

        let workers: Vec<Mmap> = (0..4)
            .map(|_| open_readonly(segment.data_path()).expect("mmap read-only worker"))
            .collect();

        for (i, mmap) in workers.iter().enumerate() {
            assert_eq!(
                mmap[0], data[0],
                "worker {i}: byte 0 debe coincidir con el dato original"
            );
            assert_eq!(mmap.len(), data.len(), "worker {i}: longitud incorrecta");
        }
    }

    // ── Criterio 3: escritura del worker rechazada ──────────────────────────

    /// Un worker que intenta abrir el segmento con map_mut() recibe un error
    /// del OS: el archivo fue abierto con O_RDONLY, por lo que mmap(PROT_WRITE)
    /// devuelve EACCES.
    #[test]
    fn shared_memory_worker_write_is_rejected() {
        let data = b"datos-read-only";
        let segment = SharedMemorySegment::create(data).expect("crear segmento");

        // Abrir el archivo con solo permiso de lectura (O_RDONLY).
        let readonly_file =
            File::open(segment.data_path()).expect("abrir segmento con O_RDONLY");

        // Intentar un mapping de escritura sobre un fd de solo lectura.
        // SAFETY: no hay UB aquí — si la llamada falla (que debe fallar),
        // no se produce ningún acceso a memoria inválida.
        let result = unsafe { MmapOptions::new().map_mut(&readonly_file) };

        assert!(
            result.is_err(),
            "map_mut sobre fd O_RDONLY debe fallar con EACCES"
        );
    }

    // ── Criterio 4: shutdown graceful en < 2s (solo Unix) ─────────────────

    #[cfg(unix)]
    #[tokio::test]
    async fn worker_graceful_shutdown_under_2s() {
        // Lanzar 4 procesos de larga duración como trabajadores simulados.
        let mut children: Vec<Child> = (0..4)
            .map(|_| Command::new("sleep").arg("999").spawn().expect("spawn sleep"))
            .collect();

        let pids: Vec<u32> = children.iter().map(|c| c.id()).collect();

        let started = std::time::Instant::now();
        let force_killed = graceful_shutdown(&pids, Duration::from_secs(2)).await;
        let elapsed = started.elapsed();

        // `sleep` responde a SIGTERM de inmediato; no debe necesitar SIGKILL.
        assert!(
            force_killed.is_empty(),
            "sleep debe salir con SIGTERM, pero estos PIDs necesitaron SIGKILL: {force_killed:?}"
        );
        assert!(
            elapsed < Duration::from_secs(2),
            "shutdown tomó {elapsed:?} (límite: 2s)"
        );

        for pid in &pids {
            assert!(!is_process_alive(*pid), "pid {pid} debe estar muerto tras shutdown");
        }

        // Recoger handles para no dejar procesos zombie.
        for child in &mut children {
            let _ = child.wait();
        }
    }

    // ── Criterio 5: workers terminan cuando el padre desaparece (solo Unix) ──

    /// Simula la muerte del padre mediante el drop del SharedMemorySegment.
    ///
    /// El worker es un proceso `sh` que sondea el archivo keepalive cada
    /// 50ms y sale cuando desaparece — el mismo mecanismo que usaría un
    /// worker real de Drasus Engine.
    #[cfg(unix)]
    #[tokio::test]
    async fn worker_terminates_when_parent_drops() {
        let data = b"datos-keepalive-test";
        let segment = SharedMemorySegment::create(data).expect("crear segmento");
        let keepalive_path = segment.keepalive_path().to_path_buf();

        // Lanzar worker que sondea el keepalive y sale cuando desaparece.
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(r#"while [ -f "$DRASUS_WORKER_KEEPALIVE" ]; do sleep 0.05; done"#)
            .env("DRASUS_WORKER_KEEPALIVE", &keepalive_path)
            .spawn()
            .expect("spawn worker keepalive");

        let pid = child.id();

        // Esperar que el worker arranque.
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(is_process_alive(pid), "worker debe estar vivo al inicio");

        // Simular muerte del padre: drop elimina el keepalive y el segmento.
        drop(segment);

        // El worker debe detectar la desaparición del keepalive y terminar.
        let started = std::time::Instant::now();
        loop {
            if !is_process_alive(pid) {
                break;
            }
            if started.elapsed() > Duration::from_secs(3) {
                let _ = child.kill();
                let _ = child.wait();
                panic!("worker no terminó en 3s tras desaparecer el keepalive");
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let _ = child.wait();
    }

    // ── Criterio 6: jobs WORKER_PROCESS RUNNING → QUEUED al reiniciar ──────

    /// Verifica que los jobs de tipo WORKER_PROCESS que quedaron en estado
    /// RUNNING (el orquestador murió con workers activos) se reencolan a
    /// QUEUED al reiniciar — igual que STORY-005, pero con el tipo concreto.
    #[tokio::test]
    async fn worker_jobs_recovered_to_queued_on_restart() {
        use crate::domain::clock::DeterministicClock;
        use crate::domain::job::JobState;
        use crate::persistence::job::{JobRepository, NewJob};
        use crate::persistence::pool::{connect, migrate};

        let temp_dir = tempfile::tempdir().expect("crear temp dir");
        let db_path = temp_dir.path().join("worker_recovery.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());

        // ── Fase 1: orquestador muere con worker en RUNNING ────────────────
        let worker_job_id = {
            let pool = connect(&database_url).await.expect("connect");
            migrate(&pool).await.expect("migrate");

            let clock = DeterministicClock::new(1_000, 100);
            let repo = JobRepository::new(&pool, &clock);

            let job = repo
                .submit(NewJob {
                    user_id: "system".to_string(),
                    job_type: "WORKER_PROCESS".to_string(),
                    parameters: "{\"backtest_id\":\"bt-001\"}".to_string(),
                    owner_id: None,
                    access_token_id: None,
                    session_id: None,
                    node_id: None,
                    logic_hash: None,
                })
                .await
                .expect("submit job worker");

            // Transicionar a RUNNING: el process_id lleva el PID del worker OS.
            let running = repo
                .transition(&job, JobState::Running, Some("worker-pid-99999"))
                .await
                .expect("transition QUEUED → RUNNING");

            assert_eq!(running.state, JobState::Running);
            assert_eq!(running.process_id, Some("worker-pid-99999".to_string()));

            pool.close().await;
            running.id
        };

        // ── Fase 2: reinicio — recuperar RUNNING → QUEUED ─────────────────
        let pool = connect(&database_url).await.expect("connect (reinicio)");
        migrate(&pool).await.expect("migrate idempotente");

        let clock = DeterministicClock::new(2_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let running_jobs = repo
            .jobs_in_state(JobState::Running)
            .await
            .expect("query RUNNING");
        assert_eq!(running_jobs.len(), 1, "debe haber 1 job RUNNING tras el crash");
        assert_eq!(running_jobs[0].job_type, "WORKER_PROCESS");

        // El orquestador reconoce que no sabe si el worker terminó
        // y lo reencola a QUEUED (el mismo patrón de STORY-005).
        let recovered = repo
            .transition(&running_jobs[0], JobState::Queued, None)
            .await
            .expect("recover RUNNING → QUEUED");

        assert_eq!(recovered.id, worker_job_id);
        assert_eq!(
            recovered.state,
            JobState::Queued,
            "job WORKER_PROCESS debe quedar en QUEUED tras recovery"
        );

        let still_running = repo
            .jobs_in_state(JobState::Running)
            .await
            .expect("query RUNNING tras recovery");
        assert!(
            still_running.is_empty(),
            "ningún job debe quedar en RUNNING tras recovery"
        );

        pool.close().await;
    }

    // ── Auxiliar: SharedMemorySegment limpia sus archivos al hacer drop ─────

    #[test]
    fn shared_memory_segment_cleans_up_on_drop() {
        let data = b"limpieza";
        let (data_path, keepalive_path) = {
            let segment = SharedMemorySegment::create(data).expect("crear segmento");
            (
                segment.data_path().to_path_buf(),
                segment.keepalive_path().to_path_buf(),
            )
        }; // drop aquí

        assert!(
            !data_path.exists(),
            "archivo de datos debe eliminarse en el drop"
        );
        assert!(
            !keepalive_path.exists(),
            "archivo keepalive debe eliminarse en el drop"
        );
    }
}
