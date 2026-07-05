//! [SHELL] Async Job Executor (`docs/features/async-job-executor.md`
//! TTR-ASYNC-EXECUTOR-001/002/004/005/006, ADR-0011, ADR-0016).
//!
//! Coordina la máquina de estados pura de [`crate::domain::job`] con:
//! - [`crate::persistence::job::JobRepository`] (durabilidad — SQLite,
//!   `jobs`/`job_results`).
//! - [`crate::persistence::audit_log::AuditLogRepository`] (el evento de
//!   auditoría `JOB_RECOVERED_AT_STARTUP`, TTR-004).
//! - Una cola en memoria de Tokio + un pool de workers acotado (TTR-002,
//!   `max_concurrent_jobs`).
//!
//! ## Patrón de tres fases (ADR-0011)
//!
//! 1. **Disparo (submit)**: [`JobExecutor::submit`] persiste el job vía
//!    [`JobRepository::submit`] — durable en disco — y solo entonces
//!    encola su id en memoria y devuelve el UUID (persist-before-ack,
//!    TTR-001).
//! 2. **Monitoreo (poll)**: [`JobExecutor::status`] /
//!    [`JobExecutor::cancel`] dejan que quien llama inspeccione o cancele
//!    un job.
//! 3. **Recuperación (fetch)**: [`JobExecutor::result`] devuelve el
//!    [`JobResult`] inmutable una vez que un job alcanza un estado
//!    terminal.
//!
//! ## Recuperación en startup (TTR-004)
//!
//! [`JobExecutor::recover_at_startup`] DEBE llamarse una vez, después de
//! construir y antes de [`JobExecutor::spawn_workers`]. Escanea `jobs`
//! buscando filas en `QUEUED` o `RUNNING`:
//! - Las filas `QUEUED` se reencolan tal cual.
//! - Las filas `RUNNING` transicionan a `QUEUED` (no se sabe si
//!   terminaron — ADR-0011 "Auto-Recovery") y se reencolan.
//!
//! Para cada job recuperado, se agrega un evento de auditoría
//! `JOB_RECOVERED_AT_STARTUP` vía [`AuditLogRepository::append`] (el
//! audit log existente — `docs/features/async-job-executor.md` TTR-004:
//! "Registrar en audit: ... NO inventes otra forma de auditar"), con
//! `job_uuid` y `previous_state` en `details_json`.
//!
//! ## Pool de workers (TTR-002)
//!
//! [`JobExecutor::spawn_workers`] arranca una tarea dispatcher que extrae
//! ids de job de la cola en memoria y, por cada uno, adquiere uno de los
//! permisos de [`tokio::sync::Semaphore`] de `max_concurrent_jobs` antes
//! de lanzar la tarea de ejecución del job — forzando el límite duro de
//! concurrencia (`docs/features/async-job-executor.md` "Restricciones":
//! "NUNCA se ejecutan más de `max_concurrent_jobs` jobs simultáneamente").
//!
//! ## Handlers (TTR-002 "Entrada": "Funciones callback que ejecutar")
//!
//! [`JobHandler`] es el callback enchufable que invoca un worker para
//! hacer el trabajo real (costoso). TTR-ASYNC-EXECUTOR-007 — conectar
//! handlers reales desde `generate`/`validate`/`manage`/`incubate`/
//! `feedback` — está explícitamente FUERA DE ALCANCE para esta historia;
//! esos módulos todavía no existen. Un job cuyo `job_type` no tiene
//! handler registrado falla de inmediato con un [`JobOutcome::Failure`]
//! descriptivo — esto es comportamiento genérico del executor, no lógica
//! de negocio.
//!
//! ## Cancelación (TTR-006)
//!
//! - Un job `QUEUED` se cancela de inmediato:
//!   [`JobExecutor::cancel`] lo transiciona directo a `CANCELLED` en
//!   SQLite. El dispatcher se saltea ids cuyo estado almacenado ya no es
//!   `QUEUED` cuando les toca correr.
//! - Un job `RUNNING` se cancela cooperativamente:
//!   [`JobExecutor::cancel`] activa un [`CancellationToken`] por job, que
//!   se espera que el [`JobHandler::execute`] en ejecución consulte
//!   ([`CancellationToken::is_cancelled`]). Una vez que el handler
//!   retorna, el worker transiciona el job a `CANCELLED` y no registra
//!   ningún resultado.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde_json::json;
use sqlx::SqlitePool;
use tokio::sync::{mpsc, Mutex, Semaphore};

use crate::domain::audit_log::AuditEventContent;
use crate::domain::clock::Clock;
use crate::domain::job::{JobState, Progress};
use crate::persistence::audit_log::{AuditLogError, AuditLogRepository};
use crate::persistence::job::{Job, JobRepository, JobRepositoryError, NewJob, NewJobResult, RecoveredJob};

/// `action_type` para el evento de auditoría de recuperación en startup
/// (TTR-004: "Registrar en audit: 'JOB_RECOVERED_AT_STARTUP:
/// job_uuid=..., previous_state=...'").
pub const JOB_RECOVERED_AT_STARTUP: &str = "JOB_RECOVERED_AT_STARTUP";

/// Configuración para un [`JobExecutor`]
/// (`docs/features/async-job-executor.md` "Parámetros Configurables").
/// Solo se incluyen los parámetros que implementa esta historia;
/// `job_timeout`, `job_queue_size`, `result_retention_days` quedan
/// diferidos a historias posteriores.
#[derive(Debug, Clone)]
pub struct JobExecutorConfig {
    /// Límite duro de jobs corriendo simultáneamente (default 3, rango 1-16).
    pub max_concurrent_jobs: usize,
    /// Segundos entre actualizaciones de progreso que se espera que emita
    /// un worker (TTR-005 `progress_interval`, default 5). Esto es un
    /// contrato para las implementaciones de handler; el executor en sí
    /// no fuerza ningún temporizador.
    pub progress_interval_seconds: u64,
}

impl Default for JobExecutorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_jobs: 3,
            progress_interval_seconds: 5,
        }
    }
}

/// Metadatos de ADR-0020 que identifican la instancia/proceso del
/// executor (`docs/features/async-job-executor.md` "Gobernanza y
/// Estándares": `process_id`, `session_id`, `node_id`, `logic_hash`, más
/// el `institutional_tag` obligatorio del audit log). Inyectado por
/// quien llama — nunca se inventa dentro de esta cáscara.
#[derive(Debug, Clone)]
pub struct ExecutorIdentity {
    /// Worker ID / Job Anchor para los jobs que toma esta instancia del
    /// executor, y `process_id` en el evento de auditoría
    /// `JOB_RECOVERED_AT_STARTUP`.
    pub process_id: String,
    /// Agrupación de runtime, estampada en cada job que envía este executor.
    pub session_id: Option<String>,
    /// Huella de hardware, estampada en cada job que envía este executor.
    pub node_id: Option<String>,
    /// Versión del executor (hash de commit/build), estampada en cada
    /// job que envía este executor.
    pub logic_hash: Option<String>,
    /// Obligatorio en todo evento de auditoría (audit-log.md TTR-001).
    pub institutional_tag: String,
}

/// Señal de cancelación cooperativa para un job en ejecución (TTR-006).
///
/// Clonar comparte la misma bandera subyacente — el executor tiene un
/// clon, la tarea del job lanzada tiene otro, y (si el handler la
/// propaga) el handler tiene un tercero.
#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Señaliza la cancelación. Idempotente.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Devuelve `true` una vez que se llamó a [`Self::cancel`].
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Reporta progreso para un job en ejecución (TTR-005).
///
/// Envuelve [`JobRepository::update_progress`] más la lectura de [`Clock`]
/// que necesita [`crate::domain::job::estimate_remaining_seconds`]. Se
/// entrega a [`JobHandler::execute`] para que los handlers nunca toquen
/// el pool/clock directamente.
pub struct ProgressReporter<'a> {
    repo: &'a JobRepository<'a>,
    job: Job,
}

impl<'a> ProgressReporter<'a> {
    /// Persiste `progress` (0-100, clampeado) para el job envuelto y
    /// actualiza el snapshot interno del reporter para que las llamadas
    /// subsecuentes encadenen correctamente (`event_sequence_id`/
    /// `audit_chain_hash`).
    pub async fn report(&mut self, progress: u8) -> Result<(), JobRepositoryError> {
        let updated = self.repo.update_progress(&self.job, Progress::new(progress)).await?;
        self.job = updated;
        Ok(())
    }

    /// El `created_at` actual del job, en nanosegundos desde el Unix
    /// epoch — los handlers usan esto junto con el [`Clock`] del
    /// executor para calcular el tiempo transcurrido para
    /// [`crate::domain::job::estimate_remaining_seconds`].
    pub fn started_at_ns(&self) -> i64 {
        self.job.created_at_ns
    }
}

/// Resultado de [`JobHandler::execute`] (TTR-002/003).
#[derive(Debug, Clone)]
pub enum JobOutcome {
    /// El job terminó con éxito. `result_data` es el payload codificado
    /// en JSON que se persiste en `job_results.result_data`.
    Success { result_data: String },
    /// El job falló. `error_message` se persiste en
    /// `job_results.error_message` (TTR-002 "Si job tira excepción, se
    /// captura y guarda como FAILED").
    Failure { error_message: String },
}

/// El callback enchufable que invoca un worker para realizar el trabajo
/// real de un job (TTR-002 "Entrada": "Funciones callback que ejecutar").
///
/// TTR-ASYNC-EXECUTOR-007 (conectar `generate`/`validate`/`manage`/
/// `incubate`/`feedback` como handlers reales) está explícitamente fuera
/// de alcance para esta historia.
#[async_trait::async_trait]
pub trait JobHandler: Send + Sync {
    /// Ejecuta `job`. Las implementaciones deberían llamar
    /// periódicamente a `progress.report(...)` (TTR-005) y revisar
    /// `cancel.is_cancelled()` (TTR-006), retornando en cuanto se observe
    /// la cancelación.
    async fn execute(&self, job: &Job, progress: &mut ProgressReporter<'_>, cancel: &CancellationToken) -> JobOutcome;
}

/// Errores que devuelven las operaciones de [`JobExecutor`].
#[derive(Debug)]
pub enum JobExecutorError {
    Repository(JobRepositoryError),
    Audit(AuditLogError),
    /// Se llamó a [`JobExecutor::cancel`] para un id de job que no
    /// existe, o que ya está en estado terminal.
    JobNotCancellable(String),
}

impl std::fmt::Display for JobExecutorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobExecutorError::Repository(err) => write!(f, "job executor repository error: {err}"),
            JobExecutorError::Audit(err) => write!(f, "job executor audit error: {err}"),
            JobExecutorError::JobNotCancellable(id) => {
                write!(f, "job '{id}' cannot be cancelled (not found or already terminal)")
            }
        }
    }
}

impl std::error::Error for JobExecutorError {}

impl From<JobRepositoryError> for JobExecutorError {
    fn from(err: JobRepositoryError) -> Self {
        JobExecutorError::Repository(err)
    }
}

impl From<AuditLogError> for JobExecutorError {
    fn from(err: AuditLogError) -> Self {
        JobExecutorError::Audit(err)
    }
}

/// Estado compartido detrás del handle barato-de-clonar de [`JobExecutor`].
struct Shared {
    pool: SqlitePool,
    clock: Arc<dyn Clock>,
    identity: ExecutorIdentity,
    config: JobExecutorConfig,
    handlers: HashMap<String, Arc<dyn JobHandler>>,
    semaphore: Arc<Semaphore>,
    cancel_tokens: Mutex<HashMap<String, CancellationToken>>,
}

/// El Async Job Executor (`docs/features/async-job-executor.md`).
///
/// Barato de clonar (un handle `Arc`): los clones comparten la misma
/// cola en memoria, el mismo semáforo y los mismos tokens de
/// cancelación, todos respaldados por el mismo pool de SQLite.
#[derive(Clone)]
pub struct JobExecutor {
    shared: Arc<Shared>,
    queue_tx: mpsc::UnboundedSender<String>,
    queue_rx: Arc<Mutex<mpsc::UnboundedReceiver<String>>>,
}

impl JobExecutor {
    /// Crea un executor nuevo asociado a `pool` (ya migrado — ver
    /// [`crate::persistence::pool::connect`] +
    /// [`crate::persistence::pool::migrate`]) y `clock`.
    ///
    /// `handlers` mapea `job_type` -> [`JobHandler`]. Un job cuyo
    /// `job_type` no tiene entrada falla de inmediato al ser tomado (ver
    /// la documentación del módulo).
    ///
    /// NO arranca workers y NO recupera jobs de una corrida anterior —
    /// llama a [`Self::recover_at_startup`] y luego a
    /// [`Self::spawn_workers`] explícitamente, en ese orden.
    pub fn new(
        pool: SqlitePool,
        clock: Arc<dyn Clock>,
        identity: ExecutorIdentity,
        config: JobExecutorConfig,
        handlers: HashMap<String, Arc<dyn JobHandler>>,
    ) -> Self {
        let (queue_tx, queue_rx) = mpsc::unbounded_channel();

        Self {
            shared: Arc::new(Shared {
                pool,
                clock,
                identity,
                semaphore: Arc::new(Semaphore::new(config.max_concurrent_jobs.max(1))),
                config,
                handlers,
                cancel_tokens: Mutex::new(HashMap::new()),
            }),
            queue_tx,
            queue_rx: Arc::new(Mutex::new(queue_rx)),
        }
    }

    fn repo(&self) -> JobRepository<'_> {
        JobRepository::new(&self.shared.pool, self.shared.clock.as_ref())
    }

    fn audit_repo(&self) -> AuditLogRepository<'_> {
        AuditLogRepository::new(&self.shared.pool, self.shared.clock.as_ref())
    }

    /// **Disparo (TTR-001)**: persiste `request` (durable, SQLite —
    /// persist-before-ack) y encola su id para que un worker lo tome.
    /// Devuelve el UUID del job nuevo.
    pub async fn submit(&self, request: NewJob) -> Result<String, JobExecutorError> {
        // Estampa los metadatos de identidad del executor cuando quien
        // llama no provee los suyos (ADR-0020 "Gobernanza y Estándares").
        let request = NewJob {
            session_id: request.session_id.or_else(|| self.shared.identity.session_id.clone()),
            node_id: request.node_id.or_else(|| self.shared.identity.node_id.clone()),
            logic_hash: request.logic_hash.or_else(|| self.shared.identity.logic_hash.clone()),
            ..request
        };

        let job = self.repo().submit(request).await?;
        self.enqueue(&job.id);

        Ok(job.id)
    }

    /// Empuja `job_id` a la cola en memoria. Helper interno compartido
    /// por [`Self::submit`] y [`Self::recover_at_startup`].
    fn enqueue(&self, job_id: &str) {
        // Un envío en un canal sin límite solo falla si el receptor se
        // descartó, lo cual solo pasa si este `JobExecutor` (y todos sus
        // clones) ya no existen — en ese punto no hay nada útil que
        // hacer con el error, y el job sigue durablemente QUEUED en
        // SQLite para la siguiente pasada de recuperación en startup.
        let _ = self.queue_tx.send(job_id.to_string());
    }

    /// **Recuperación en startup (TTR-004)**.
    ///
    /// DEBE llamarse una vez, después de [`Self::new`] y antes de
    /// [`Self::spawn_workers`].
    ///
    /// Escanea `jobs` buscando filas en `QUEUED` o `RUNNING`:
    /// - Las filas `QUEUED` se reencolan tal cual.
    /// - Las filas `RUNNING` transicionan a `QUEUED` (ADR-0011
    ///   "Auto-Recovery": no se sabe si terminaron tras un crash) y
    ///   luego se reencolan.
    ///
    /// Para cada job recuperado (tanto `QUEUED` como ex-`RUNNING`), agrega
    /// un evento de auditoría `JOB_RECOVERED_AT_STARTUP` vía
    /// [`AuditLogRepository::append`] con `job_uuid` y `previous_state`
    /// en `details_json`.
    ///
    /// Devuelve la lista de [`RecoveredJob`]s (job después de la
    /// recuperación + `previous_state`), ordenada por `created_at` — útil
    /// para tests y logging de startup. Un resultado vacío significa que
    /// no había nada que recuperar.
    pub async fn recover_at_startup(&self) -> Result<Vec<RecoveredJob>, JobExecutorError> {
        let repo = self.repo();
        let audit = self.audit_repo();

        let mut queued = repo.jobs_in_state(JobState::Queued).await?;
        let running = repo.jobs_in_state(JobState::Running).await?;

        let mut recovered = Vec::with_capacity(queued.len() + running.len());

        // Primero RUNNING -> QUEUED, para que la lista combinada de abajo
        // refleje el estado post-recuperación de cada job.
        for job in running {
            let previous_state = job.state;
            let requeued = repo.transition(&job, JobState::Queued, None).await?;

            self.append_recovery_audit_event(&audit, &requeued.id, previous_state).await?;

            recovered.push(RecoveredJob {
                job: requeued,
                previous_state,
            });
        }

        for job in queued.drain(..) {
            let previous_state = job.state; // == JobState::Queued
            self.append_recovery_audit_event(&audit, &job.id, previous_state).await?;

            recovered.push(RecoveredJob { job, previous_state });
        }

        // Orden estable entre ambos grupos (cronológico por created_at),
        // igual al orden propio de jobs_in_state.
        recovered.sort_by_key(|r| r.job.created_at_ns);

        for recovered_job in &recovered {
            self.enqueue(&recovered_job.job.id);
        }

        Ok(recovered)
    }

    async fn append_recovery_audit_event(
        &self,
        audit: &AuditLogRepository<'_>,
        job_uuid: &str,
        previous_state: JobState,
    ) -> Result<(), JobExecutorError> {
        let details_json = json!({
            "job_uuid": job_uuid,
            "previous_state": previous_state.as_str(),
        })
        .to_string();

        audit
            .append(AuditEventContent {
                action_type: JOB_RECOVERED_AT_STARTUP.to_string(),
                entity_type: "JOB".to_string(),
                entity_id: job_uuid.to_string(),
                details_json,
                owner_id: None,
                institutional_tag: self.shared.identity.institutional_tag.clone(),
                manifest_id: None,
                access_token_id: None,
                process_id: self.shared.identity.process_id.clone(),
                session_id: self.shared.identity.session_id.clone(),
                node_id: self.shared.identity.node_id.clone(),
            })
            .await?;

        Ok(())
    }

    /// **Monitoreo (poll, TTR-002/005)**: devuelve la fila [`Job`] actual,
    /// o `None` si `job_id` no existe.
    pub async fn status(&self, job_id: &str) -> Result<Option<Job>, JobExecutorError> {
        Ok(self.repo().find(job_id).await?)
    }

    /// **Recuperación (fetch, TTR-003)**: devuelve el
    /// [`crate::persistence::job::JobResult`] del job, o `None` si el job
    /// todavía no alcanzó un estado terminal con resultado registrado.
    pub async fn result(&self, job_id: &str) -> Result<Option<crate::persistence::job::JobResult>, JobExecutorError> {
        Ok(self.repo().result_for_job(job_id).await?)
    }

    /// **Cancelación (TTR-006)**.
    ///
    /// - Si `job_id` está `QUEUED`, lo transiciona directo a
    ///   `CANCELLED`. El dispatcher lo saltea cuando llega al frente de
    ///   la cola (su estado persistido se revisa antes de que arranque
    ///   la ejecución).
    /// - Si `job_id` está `RUNNING`, activa su [`CancellationToken`]; se
    ///   espera que el [`JobHandler`] en ejecución lo observe y retorne.
    ///   El worker entonces transiciona el job a `CANCELLED`.
    /// - Cualquier otro estado (ya terminal, o id desconocido) devuelve
    ///   [`JobExecutorError::JobNotCancellable`].
    pub async fn cancel(&self, job_id: &str) -> Result<(), JobExecutorError> {
        let repo = self.repo();
        let job = repo
            .find(job_id)
            .await?
            .ok_or_else(|| JobExecutorError::JobNotCancellable(job_id.to_string()))?;

        match job.state {
            JobState::Queued => {
                repo.transition(&job, JobState::Cancelled, None).await?;
                Ok(())
            }
            JobState::Running => {
                let tokens = self.shared.cancel_tokens.lock().await;
                match tokens.get(job_id) {
                    Some(token) => {
                        token.cancel();
                        Ok(())
                    }
                    // RUNNING en la base de datos pero sin token
                    // registrado significa que ningún worker vivo lo
                    // posee en este proceso (ej. estaba RUNNING antes de
                    // un crash y la recuperación todavía no lo tomó con
                    // el dispatcher). La recuperación ya lo degradó a
                    // QUEUED antes de reencolarlo, así que para cuando un
                    // worker lo corriera, el `find` de arriba habría
                    // devuelto QUEUED -- esta rama es defensiva.
                    None => Err(JobExecutorError::JobNotCancellable(job_id.to_string())),
                }
            }
            JobState::Completed | JobState::Failed | JobState::Cancelled => {
                Err(JobExecutorError::JobNotCancellable(job_id.to_string()))
            }
        }
    }

    /// **Pool de workers (TTR-002)**: lanza una tarea dispatcher que
    /// extrae ids de job de la cola en memoria y, por cada uno, adquiere
    /// uno de los permisos de semáforo de `max_concurrent_jobs` antes de
    /// lanzar la tarea de ejecución del job. Retorna de inmediato; el
    /// dispatcher corre hasta que cada clon de este [`JobExecutor`] (y
    /// por lo tanto el sender de la cola) se descarta.
    ///
    /// Debe llamarse sobre un runtime de Tokio (`#[tokio::main]` /
    /// `#[tokio::test]`).
    pub fn spawn_workers(&self) -> tokio::task::JoinHandle<()> {
        let executor = self.clone();

        tokio::spawn(async move {
            loop {
                let job_id = {
                    let mut rx = executor.queue_rx.lock().await;
                    match rx.recv().await {
                        Some(id) => id,
                        None => break, // todos los senders se descartaron -> apagar
                    }
                };

                let permit = match executor.shared.semaphore.clone().acquire_owned().await {
                    Ok(permit) => permit,
                    Err(_) => break, // semáforo cerrado -> apagando
                };

                let worker = executor.clone();
                tokio::spawn(async move {
                    worker.run_job(job_id).await;
                    drop(permit);
                });
            }
        })
    }

    /// Ejecuta un único job de punta a punta: QUEUED -> RUNNING ->
    /// terminal, invocando el [`JobHandler`] registrado (si hay) y
    /// registrando el resultado. Saltea jobs que se cancelaron mientras
    /// estaban en cola (TTR-006).
    async fn run_job(&self, job_id: String) {
        let repo = self.repo();

        let job = match repo.find(&job_id).await {
            Ok(Some(job)) => job,
            Ok(None) | Err(_) => return, // el job desapareció o hubo error de DB: nada que correr
        };

        // Saltea jobs cancelados mientras todavía estaban QUEUED (TTR-006).
        if job.state != JobState::Queued {
            return;
        }

        let running = match repo.transition(&job, JobState::Running, Some(&self.shared.identity.process_id)).await {
            Ok(running) => running,
            Err(_) => return,
        };

        let cancel_token = CancellationToken::new();
        {
            let mut tokens = self.shared.cancel_tokens.lock().await;
            tokens.insert(job_id.clone(), cancel_token.clone());
        }

        let handler = self.shared.handlers.get(&running.job_type).cloned();

        let outcome = match handler {
            Some(handler) => {
                let mut progress = ProgressReporter {
                    repo: &repo,
                    job: running.clone(),
                };
                handler.execute(&running, &mut progress, &cancel_token).await
            }
            None => JobOutcome::Failure {
                error_message: format!("no handler registered for job_type '{}'", running.job_type),
            },
        };

        {
            let mut tokens = self.shared.cancel_tokens.lock().await;
            tokens.remove(&job_id);
        }

        // Relee el estado actual: el handler puede haber observado la
        // cancelación (TTR-006), o algún otro camino puede haber movido
        // el job ya.
        let current = match repo.find(&job_id).await {
            Ok(Some(job)) => job,
            _ => return,
        };

        if cancel_token.is_cancelled() && current.state == JobState::Running {
            let _ = repo.transition(&current, JobState::Cancelled, None).await;
            return;
        }

        if current.state != JobState::Running {
            // Ya se movió fuera de RUNNING por otro camino (ej. la
            // cancelación corrió en carrera con la finalización) -- no
            // registrar el resultado dos veces.
            return;
        }

        match outcome {
            JobOutcome::Success { result_data } => {
                if let Ok(completed) = repo.transition(&current, JobState::Completed, None).await {
                    let _ = repo
                        .record_result(NewJobResult {
                            job_uuid: completed.id,
                            result_data: Some(result_data),
                            error_message: None,
                        })
                        .await;
                }
            }
            JobOutcome::Failure { error_message } => {
                if let Ok(failed) = repo.transition(&current, JobState::Failed, None).await {
                    let _ = repo
                        .record_result(NewJobResult {
                            job_uuid: failed.id,
                            result_data: None,
                            error_message: Some(error_message),
                        })
                        .await;
                }
            }
        }
    }

    /// La configuración con la que se construyó este executor.
    pub fn config(&self) -> &JobExecutorConfig {
        &self.shared.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::persistence::pool::{connect, migrate};

    /// ADR-0020 "Gobernanza y Estándares": identidad mínima para
    /// tests — sin significado de negocio, solo strings estables para
    /// estampar en jobs/eventos de auditoría.
    fn test_identity() -> ExecutorIdentity {
        ExecutorIdentity {
            process_id: "test-process".to_string(),
            session_id: Some("test-session".to_string()),
            node_id: Some("test-node".to_string()),
            logic_hash: Some("executor-v1".to_string()),
            institutional_tag: "DRASUS_TEST".to_string(),
        }
    }

    fn sample_new_job(job_type: &str) -> NewJob {
        NewJob {
            user_id: "user-1".to_string(),
            job_type: job_type.to_string(),
            parameters: "{\"strategy_id\":123}".to_string(),
            owner_id: Some("owner-1".to_string()),
            access_token_id: None,
            session_id: None,
            node_id: None,
            logic_hash: None,
        }
    }

    /// **Gate de cierre de EPIC-0**
    /// (`docs/execution/STORY-005-async-job-executor.md` §4,
    /// TTR-ASYNC-EXECUTOR-004): los jobs sobreviven a un `kill -9`
    /// simulado y se recuperan al reiniciar.
    ///
    /// Usa una base de datos SQLite en un **archivo temporal** (no
    /// `sqlite::memory:`) porque una base de datos en memoria no
    /// sobrevive cerrar y reabrir el pool — no puede demostrar
    /// durabilidad ni recuperación de crash.
    ///
    /// Pasos:
    /// 1. Abre un pool respaldado por archivo, migra, envía tres jobs y
    ///    lleva uno a `RUNNING` sin nunca completarlo (simula el momento
    ///    en que un crash interrumpe un job en vuelo).
    /// 2. Cierra ese pool por completo — el estado en disco es ahora la
    ///    única fuente de verdad, exactamente como tras un `kill -9`.
    /// 3. Abre un pool completamente nuevo sobre el MISMO archivo de base
    ///    de datos, construye un [`JobExecutor`] nuevo encima, y llama a
    ///    [`JobExecutor::recover_at_startup`].
    /// 4. Verifica el gate: (a) el job QUEUED se recupera y se reencola;
    ///    (b) el job RUNNING ahora está QUEUED (no se perdió, no quedó
    ///    atascado en RUNNING); (c) ningún job se pierde (el conteo total
    ///    no cambia); (d) se registró un evento de auditoría
    ///    `JOB_RECOVERED_AT_STARTUP` con `previous_state` para cada job
    ///    recuperado.
    #[tokio::test]
    async fn jobs_survive_simulated_crash_and_are_recovered_on_restart() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("job_executor_crash.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());

        let (queued_job_id, running_job_id, completed_job_id) = {
            // --- Proceso antes del crash ---------------------------------------------
            let pool = connect(&database_url).await.expect("connect (pre-crash)");
            migrate(&pool).await.expect("migrate (pre-crash)");

            let clock: Arc<dyn Clock> = Arc::new(DeterministicClock::new(1_000, 100));
            let executor = JobExecutor::new(
                pool.clone(),
                clock,
                test_identity(),
                JobExecutorConfig::default(),
                HashMap::new(),
            );

            // No hay corrida previa que recuperar -- el test del gate se
            // enfoca en la pasada de reinicio de abajo, pero llamar esto
            // aquí también ejercita el camino documentado "no-op con
            // tabla jobs vacía".
            let pre_recovery = executor.recover_at_startup().await.expect("pre-crash recover (no-op)");
            assert!(pre_recovery.is_empty(), "fresh database has nothing to recover");

            // El job A queda QUEUED (nunca se toma antes del "crash").
            let queued_job_id = executor.submit(sample_new_job("BACKTEST")).await.expect("submit queued job");

            // El job B se lleva a RUNNING y se deja ahí -- este es el
            // job que un worker en vuelo habría estado ejecutando en el
            // momento del crash.
            let running_job_id = executor.submit(sample_new_job("BACKTEST")).await.expect("submit running job");
            {
                let repo = executor.repo();
                let job = repo.find(&running_job_id).await.expect("find running job").expect("job exists");
                repo.transition(&job, JobState::Running, Some("worker-pre-crash"))
                    .await
                    .expect("transition to running");
            }

            // El job C completa normalmente antes del crash -- la
            // recuperación debe dejarlo intacto (no está QUEUED ni RUNNING).
            let completed_job_id = executor.submit(sample_new_job("BACKTEST")).await.expect("submit completed job");
            {
                let repo = executor.repo();
                let job = repo
                    .find(&completed_job_id)
                    .await
                    .expect("find completed job")
                    .expect("job exists");
                let running = repo
                    .transition(&job, JobState::Running, Some("worker-pre-crash"))
                    .await
                    .expect("transition completed job to running");
                repo.transition(&running, JobState::Completed, None)
                    .await
                    .expect("transition completed job to completed");
            }

            // Simula `kill -9`: descarta el executor y cierra el pool
            // sin nunca terminar el job B. El estado en disco (QUEUED,
            // RUNNING, COMPLETED) es la única verdad que sobrevive.
            drop(executor);
            pool.close().await;

            (queued_job_id, running_job_id, completed_job_id)
        };

        // --- Reinicio: pool completamente nuevo sobre el MISMO archivo ----------
        let pool = connect(&database_url).await.expect("connect (restart)");
        migrate(&pool).await.expect("migrate (restart) must be idempotent");

        let clock: Arc<dyn Clock> = Arc::new(DeterministicClock::new(2_000, 100));
        let executor = JobExecutor::new(pool.clone(), clock, test_identity(), JobExecutorConfig::default(), HashMap::new());

        let recovered = executor.recover_at_startup().await.expect("recover at startup after crash");

        // (c) Ningún job se pierde: se recuperan exactamente los dos
        // jobs no-terminales (A y B). El job completado C queda intacto.
        assert_eq!(recovered.len(), 2, "exactly the QUEUED and RUNNING jobs must be recovered");

        let recovered_ids: HashMap<&str, &RecoveredJob> = recovered.iter().map(|r| (r.job.id.as_str(), r)).collect();

        // (a) El job QUEUED se recupera con su estado sin cambios.
        let recovered_queued = recovered_ids
            .get(queued_job_id.as_str())
            .expect("queued job is in the recovered list");
        assert_eq!(recovered_queued.previous_state, JobState::Queued);
        assert_eq!(recovered_queued.job.state, JobState::Queued);

        // (b) El job RUNNING se degrada a QUEUED -- no se perdió, no
        // quedó atascado en RUNNING.
        let recovered_running = recovered_ids
            .get(running_job_id.as_str())
            .expect("running job is in the recovered list");
        assert_eq!(recovered_running.previous_state, JobState::Running);
        assert_eq!(recovered_running.job.state, JobState::Queued);

        // (c) El conteo total de jobs no cambia -- nada se perdió ni se
        // duplicó -- y el job completado queda intacto.
        let repo = executor.repo();
        let all_queued = repo.jobs_in_state(JobState::Queued).await.expect("query queued after recovery");
        let all_running = repo.jobs_in_state(JobState::Running).await.expect("query running after recovery");
        let all_completed = repo.jobs_in_state(JobState::Completed).await.expect("query completed after recovery");

        assert_eq!(all_running.len(), 0, "no job must remain stuck in RUNNING after recovery");
        assert_eq!(all_queued.len(), 2, "both recovered jobs must now be QUEUED");
        assert_eq!(all_completed.len(), 1, "the already-completed job must be untouched by recovery");
        assert_eq!(all_completed[0].id, completed_job_id);

        let queued_ids: std::collections::HashSet<&str> = all_queued.iter().map(|j| j.id.as_str()).collect();
        assert!(queued_ids.contains(queued_job_id.as_str()));
        assert!(queued_ids.contains(running_job_id.as_str()));

        // (d) Se registró un evento de auditoría JOB_RECOVERED_AT_STARTUP
        // con `previous_state` para cada job recuperado (reusando el
        // audit log existente -- TTR-004: "Registrar en audit ... NO
        // inventes otra forma de auditar").
        let audit = executor.audit_repo();

        let queued_audit_events = audit
            .events_for_entity("JOB", &queued_job_id)
            .await
            .expect("load audit events for queued job");
        assert_eq!(queued_audit_events.len(), 1);
        assert_eq!(queued_audit_events[0].content.action_type, JOB_RECOVERED_AT_STARTUP);
        let queued_details: serde_json::Value =
            serde_json::from_str(&queued_audit_events[0].content.details_json).expect("parse details_json");
        assert_eq!(queued_details["job_uuid"], queued_job_id);
        assert_eq!(queued_details["previous_state"], "QUEUED");

        let running_audit_events = audit
            .events_for_entity("JOB", &running_job_id)
            .await
            .expect("load audit events for running job");
        assert_eq!(running_audit_events.len(), 1);
        assert_eq!(running_audit_events[0].content.action_type, JOB_RECOVERED_AT_STARTUP);
        let running_details: serde_json::Value =
            serde_json::from_str(&running_audit_events[0].content.details_json).expect("parse details_json");
        assert_eq!(running_details["job_uuid"], running_job_id);
        assert_eq!(running_details["previous_state"], "RUNNING");

        // El job completado NO debe tener un evento de auditoría de
        // recuperación -- nunca estuvo QUEUED ni RUNNING al momento del
        // reinicio.
        let completed_audit_events = audit
            .events_for_entity("JOB", &completed_job_id)
            .await
            .expect("load audit events for completed job");
        assert!(
            completed_audit_events.is_empty(),
            "a job that was already COMPLETED before the crash must not get a recovery event"
        );

        pool.close().await;
    }

    /// Llamar a [`JobExecutor::recover_at_startup`] sobre una base de
    /// datos recién migrada y vacía es un no-op documentado: nada que
    /// recuperar, ningún evento de auditoría escrito. Complementa el
    /// test del gate de recuperación de crash de arriba cubriendo
    /// explícitamente la rama de "nada que hacer".
    #[tokio::test]
    async fn recover_at_startup_on_empty_database_is_a_noop() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("job_executor_empty.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());

        let pool = connect(&database_url).await.expect("connect");
        migrate(&pool).await.expect("migrate");

        let clock: Arc<dyn Clock> = Arc::new(DeterministicClock::new(1_000, 100));
        let executor = JobExecutor::new(pool.clone(), clock, test_identity(), JobExecutorConfig::default(), HashMap::new());

        let recovered = executor.recover_at_startup().await.expect("recover on empty db");
        assert!(recovered.is_empty());

        let audit = executor.audit_repo();
        let chain = audit.load_chain().await.expect("load chain");
        assert!(chain.is_empty(), "no audit events on an empty database");

        pool.close().await;
    }
}
