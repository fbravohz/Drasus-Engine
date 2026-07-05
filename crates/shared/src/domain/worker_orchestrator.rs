//! [CORE] Lógica pura del orquestador de workers aislados.
//! (`docs/features/worker-isolation-orchestrator.md` TTR-001/TTR-002,
//! ADR-0013, ADR-0020).
//!
//! Sin I/O, sin reloj de sistema, sin azar sin semilla (ADR-0002/0004).
//! Sin imports de `std::process`, `std::fs`, `tokio`, `nix` ni `memmap2`.
//!
//! Decide cuántos workers lanzar, a qué job les asigna y cuándo matar.
//! La ejecución real (spawn OS, mmap, señales) vive en la cáscara
//! [`crate::orchestrator::worker_runner`].

use std::collections::HashMap;

/// Configuración del orquestador de workers.
/// Todos los campos son parámetros configurables (no invariantes físicos).
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Límite duro de procesos worker corriendo simultáneamente.
    pub max_concurrent: usize,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self { max_concurrent: 4 }
    }
}

/// Error opaco de backend — el detalle lo conoce la cáscara que lo produce.
#[derive(Debug, Clone)]
pub struct WorkerBackendError(pub String);

impl std::fmt::Display for WorkerBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "worker backend error: {}", self.0)
    }
}

impl std::error::Error for WorkerBackendError {}

/// Contrato que separa la decisión pura de lanzar/matar de los efectos
/// reales del SO (spawn, mmap, señales).
///
/// El dominio nunca implementa este trait — solo lo llama vía inyección
/// de dependencia. La implementación concreta vive en
/// [`crate::orchestrator::worker_runner::OsWorkerBackend`].
pub trait WorkerBackend: Send + Sync {
    /// Lanza un nuevo proceso worker para `job_id`.
    ///
    /// Recibe:
    /// - `shm_path`: ruta del archivo de datos de memoria compartida.
    /// - `keepalive_path`: ruta del archivo centinela de vida del padre.
    ///
    /// Devuelve el PID OS del proceso hijo lanzado.
    fn launch(
        &self,
        job_id: &str,
        shm_path: &str,
        keepalive_path: &str,
    ) -> Result<u32, WorkerBackendError>;

    /// Envía SIGTERM al proceso con `pid` (petición de shutdown graceful).
    fn send_sigterm(&self, pid: u32) -> Result<(), WorkerBackendError>;

    /// Envía SIGKILL al proceso con `pid` (terminación forzada e inmediata).
    fn send_sigkill(&self, pid: u32) -> Result<(), WorkerBackendError>;

    /// Devuelve `true` si el proceso `pid` todavía existe en el SO.
    fn is_alive(&self, pid: u32) -> bool;
}

/// Orquestador de workers aislados — núcleo puro.
///
/// Decide cuántos workers lanzar dado el límite de concurrencia y la lista
/// de jobs en cola, y mantiene el registro de los workers activos.
/// No tiene I/O: la cáscara lo invoca y aplica sus decisiones.
#[derive(Debug)]
pub struct WorkerOrchestrator {
    config: WorkerConfig,
    /// job_id → PID OS del proceso worker activo.
    active: HashMap<String, u32>,
}

impl WorkerOrchestrator {
    /// Crea un orquestador con la configuración dada.
    pub fn new(config: WorkerConfig) -> Self {
        Self {
            config,
            active: HashMap::new(),
        }
    }

    /// Slots disponibles para nuevos workers.
    pub fn available_slots(&self) -> usize {
        self.config.max_concurrent.saturating_sub(self.active.len())
    }

    /// Número de workers activos en este momento.
    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    /// Límite configurado de concurrencia.
    pub fn max_concurrent(&self) -> usize {
        self.config.max_concurrent
    }

    /// Selecciona cuáles IDs de `queued` lanzar ahora.
    ///
    /// Devuelve los primeros `available_slots()` IDs de `queued`, en el
    /// mismo orden de llegada — el orquestador no reordena, la política
    /// de prioridad la aplica quien construya la lista `queued`.
    pub fn jobs_to_launch<'a>(&self, queued: &[&'a str]) -> Vec<&'a str> {
        let slots = self.available_slots();
        queued.iter().take(slots).copied().collect()
    }

    /// Registra que se lanzó un proceso worker con `pid` para `job_id`.
    pub fn record_launched(&mut self, job_id: &str, pid: u32) {
        self.active.insert(job_id.to_string(), pid);
    }

    /// Registra que el worker de `job_id` terminó.
    /// Devuelve el PID que tenía, o `None` si no había worker activo.
    pub fn record_exited(&mut self, job_id: &str) -> Option<u32> {
        self.active.remove(job_id)
    }

    /// PID del worker activo de `job_id`, o `None` si no hay ninguno.
    pub fn pid_for_job(&self, job_id: &str) -> Option<u32> {
        self.active.get(job_id).copied()
    }

    /// Iterador `(job_id, pid)` de todos los workers activos.
    ///
    /// La cáscara lo usa para enviar señales en masa (SIGTERM/SIGKILL)
    /// durante un shutdown graceful o una cancelación de job.
    pub fn active_workers(&self) -> impl Iterator<Item = (&str, u32)> {
        self.active.iter().map(|(k, v)| (k.as_str(), *v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn orch(max: usize) -> WorkerOrchestrator {
        WorkerOrchestrator::new(WorkerConfig { max_concurrent: max })
    }

    // ── Criterio 7: respeta MAX_CONCURRENT_WORKERS ────────────────────────

    /// Con max=2 y 2 workers activos, available_slots()=0 y jobs_to_launch
    /// devuelve vacío — el orquestador no propone lanzar más workers.
    #[test]
    fn worker_respects_max_concurrent_workers() {
        let mut o = orch(2);
        o.record_launched("job-a", 1001);
        o.record_launched("job-b", 1002);

        assert_eq!(o.available_slots(), 0);
        let to_launch = o.jobs_to_launch(&["job-c", "job-d"]);
        assert!(
            to_launch.is_empty(),
            "no debe proponer workers cuando no quedan slots"
        );
    }

    /// Con 4 slots y 3 workers activos, solo se propone 1 job nuevo.
    #[test]
    fn available_slots_reflects_active_count() {
        let mut o = orch(4);
        o.record_launched("job-a", 1001);
        o.record_launched("job-b", 1002);
        o.record_launched("job-c", 1003);

        let to_launch = o.jobs_to_launch(&["job-d", "job-e"]);
        assert_eq!(to_launch, vec!["job-d"], "solo 1 slot disponible");
    }

    /// Al registrar la salida de un worker, el slot se libera.
    #[test]
    fn record_exited_frees_slot() {
        let mut o = orch(1);
        o.record_launched("job-a", 1001);
        assert_eq!(o.available_slots(), 0);

        let pid = o.record_exited("job-a");
        assert_eq!(pid, Some(1001));
        assert_eq!(o.available_slots(), 1);
    }

    /// `pid_for_job` devuelve el PID del worker activo y `None` si no existe.
    #[test]
    fn pid_for_job_returns_correct_pid() {
        let mut o = orch(4);
        o.record_launched("job-x", 9999);

        assert_eq!(o.pid_for_job("job-x"), Some(9999));
        assert_eq!(o.pid_for_job("inexistente"), None);
    }

    /// `active_workers` itera sobre todos los workers registrados.
    #[test]
    fn active_workers_iterates_all_entries() {
        let mut o = orch(4);
        o.record_launched("job-1", 100);
        o.record_launched("job-2", 200);

        let mut pairs: Vec<(&str, u32)> = o.active_workers().collect();
        pairs.sort_by_key(|(id, _)| *id);

        assert_eq!(pairs, vec![("job-1", 100), ("job-2", 200)]);
    }

    /// `jobs_to_launch` respeta el orden de llegada cuando hay varios slots.
    #[test]
    fn jobs_to_launch_selects_first_n_in_order() {
        let o = orch(2);
        let queued = ["job-a", "job-b", "job-c", "job-d"];
        let to_launch = o.jobs_to_launch(&queued);
        assert_eq!(to_launch, vec!["job-a", "job-b"]);
    }

    /// `available_slots` usa saturating_sub: si el estado es inconsistente
    /// (active > max_concurrent), devuelve 0 en vez de entrar en pánico.
    #[test]
    fn available_slots_never_wraps_on_overflow() {
        let mut o = orch(1);
        o.record_launched("job-a", 1);
        o.record_launched("job-b", 2); // excede max — estado inconsistente
        assert_eq!(o.available_slots(), 0);
    }

    /// `record_exited` en un job inexistente devuelve `None` sin pánico.
    #[test]
    fn record_exited_on_unknown_job_is_noop() {
        let mut o = orch(4);
        assert_eq!(o.record_exited("no-existe"), None);
    }
}
