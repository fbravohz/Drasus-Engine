//! [SHELL] Repositorio de persistencia para el Async Job Executor
//! (`docs/features/async-job-executor.md` TTR-ASYNC-EXECUTOR-001/003/004/006,
//! ADR-0011, ADR-0020).
//!
//! Envuelve las tablas `jobs` y `job_results` (migración `0003_jobs.sql`).
//! Dueño del único I/O para jobs: lecturas/escrituras en SQLite,
//! generación de UUID (azar sin semilla, ADR-0002/0004) y la lectura del
//! puerto [`Clock`]. La máquina de estados en sí ([`JobState`],
//! [`validate_transition`]) es lógica pura de core en
//! [`crate::domain::job`] — este módulo solo le da entradas inyectadas y
//! persiste/carga el resultado, reflejando el patrón de
//! [`crate::persistence::audit_log::AuditLogRepository`].
//!
//! ## Persist-before-ack (TTR-001)
//!
//! [`JobRepository::submit`] ejecuta el `INSERT INTO jobs` y solo retorna
//! después de que el commit ocurre. Quien llama (el orquestador) recibe
//! el UUID del job desde el valor `Ok` de esta llamada — no existe ningún
//! camino que entregue un UUID antes de que la fila exista en disco. Por
//! eso un `kill -9` entre "submit retornó" y "fila visible en disco" es
//! imposible: son el mismo evento.
//!
//! ## `job_results` de solo-apéndice (TTR-003)
//!
//! Forzado por duplicado, igual que en `audit_events`:
//! - **Base de datos**: la migración `0003_jobs.sql` instala triggers
//!   `BEFORE UPDATE` / `BEFORE DELETE` en `job_results` que hacen
//!   `RAISE(ABORT, ...)`.
//! - **Aplicación**: este repositorio solo expone
//!   [`JobRepository::record_result`] y [`JobRepository::result_for_job`]
//!   — no existe ningún método de update/delete.

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::clock::Clock;
use crate::domain::job::{validate_transition, JobState};

/// Errores que devuelven las operaciones de [`JobRepository`].
#[derive(Debug)]
pub enum JobRepositoryError {
    /// La operación subyacente de SQLite falló.
    Database(sqlx::Error),
    /// Una fila de `jobs` tenía un valor de `state` fuera de las cinco
    /// cadenas canónicas (`JobState::from_str_value` devolvió `None`) —
    /// un error de integridad de datos, no un error de transición.
    UnknownState(String),
    /// Una transición de estado solicitada no está permitida
    /// ([`crate::domain::job::validate_transition`]).
    InvalidTransition(crate::domain::job::InvalidTransition),
    /// [`JobRepository::record_result`] no pudo completarse tras agotar los
    /// reintentos ante contención de escritura transitoria (otro escritor
    /// mantuvo el lock de la base de datos más allá del `busy_timeout`, o
    /// hubo colisión repetida al derivar `event_sequence_id`). El resultado
    /// NO se descartó en silencio -- se propaga este error tipado (regla
    /// "Atomicidad de ledgers append-only", rust-engineer/SKILL.md §4).
    WriteContention { attempts: u32 },
}

impl std::fmt::Display for JobRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobRepositoryError::Database(err) => write!(f, "job repository database error: {err}"),
            JobRepositoryError::UnknownState(value) => {
                write!(f, "job repository: unknown state value '{value}' in jobs table")
            }
            JobRepositoryError::InvalidTransition(err) => write!(f, "job repository: {err}"),
            JobRepositoryError::WriteContention { attempts } => {
                write!(f, "no se pudo registrar el job_result tras {attempts} intentos por contención de escritura")
            }
        }
    }
}

impl std::error::Error for JobRepositoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            JobRepositoryError::Database(err) => Some(err),
            JobRepositoryError::UnknownState(_) => None,
            JobRepositoryError::InvalidTransition(err) => Some(err),
            JobRepositoryError::WriteContention { .. } => None,
        }
    }
}

impl From<sqlx::Error> for JobRepositoryError {
    fn from(err: sqlx::Error) -> Self {
        JobRepositoryError::Database(err)
    }
}

impl From<crate::domain::job::InvalidTransition> for JobRepositoryError {
    fn from(err: crate::domain::job::InvalidTransition) -> Self {
        JobRepositoryError::InvalidTransition(err)
    }
}

/// Número máximo de intentos de [`JobRepository::record_result`] ante
/// contención de escritura transitoria antes de rendirse con
/// [`JobRepositoryError::WriteContention`]. Mismo valor y misma
/// justificación que [`crate::persistence::audit_log::MAX_APPEND_ATTEMPTS`]:
/// con `busy_timeout` de 5s (ADR-0141 R2) el lock casi siempre se obtiene
/// sin reintentar.
const MAX_RECORD_ATTEMPTS: u32 = 5;

/// Decide si un error de [`JobRepository::record_result`] es una contención
/// de escritura TRANSITORIA -- algo que reintentar (re-derivando el
/// `event_sequence_id` y reinsertando) puede resolver, sin descartar el
/// resultado. Mismo criterio que
/// `crate::persistence::audit_log::is_transient_write_conflict`.
fn is_transient_write_conflict(error: &JobRepositoryError) -> bool {
    let JobRepositoryError::Database(sqlx::Error::Database(db)) = error else {
        return false;
    };

    let message = db.message().to_lowercase();
    // Lock ocupado: otro escritor tenía el lock de la BD / de la tabla.
    if message.contains("database is locked") || message.contains("database table is locked") {
        return true;
    }

    // Colisión de secuencia: mismo event_sequence_id derivado por dos
    // escritores -- transitorio, re-derivar y reinsertar lo arregla.
    db.is_unique_violation() && message.contains("event_sequence_id")
}

/// Un job nuevo para persistir (TTR-001 "Entrada": `JobRequest(job_type,
/// parameters, user_id)`), más los metadatos de ADR-0020 que provee el
/// orquestador al momento del submit.
#[derive(Debug, Clone)]
pub struct NewJob {
    pub user_id: String,
    pub job_type: String,
    /// Parámetros del job codificados en JSON (opacos para este repositorio).
    pub parameters: String,
    pub owner_id: Option<String>,
    pub access_token_id: Option<String>,
    pub session_id: Option<String>,
    pub node_id: Option<String>,
    pub logic_hash: Option<String>,
}

/// Una fila de job persistida (tabla `jobs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    /// Contador de versión de ESTA fila (arranca en 1, +1 en cada UPDATE).
    /// `jobs` es MUTABLE -- renombrado desde `event_sequence_id` (ADR-0141
    /// "Semántica de secuencias": ese nombre queda reservado para la
    /// posición GLOBAL en un event-store append-only, como `job_results`).
    pub row_version: i64,

    pub process_id: Option<String>,
    pub session_id: Option<String>,
    pub node_id: Option<String>,
    pub logic_hash: Option<String>,

    pub owner_id: Option<String>,
    pub access_token_id: Option<String>,

    pub user_id: String,
    pub job_type: String,
    pub parameters: String,
    pub state: JobState,
    pub progress: u8,
}

/// Un resultado de job nuevo para persistir (TTR-003 "Entrada": `Result
/// object(job_uuid, result_data, error_message, completed_at)`).
#[derive(Debug, Clone)]
pub struct NewJobResult {
    pub job_uuid: String,
    /// Payload del resultado codificado en JSON, `None` si falló.
    pub result_data: Option<String>,
    /// Descripción del error, `None` si tuvo éxito.
    pub error_message: Option<String>,
}

/// Una fila de resultado de job persistida (tabla `job_results`).
/// Inmutable una vez insertada (TTR-003).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobResult {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub event_sequence_id: i64,

    pub job_uuid: String,
    pub result_data: Option<String>,
    pub error_message: Option<String>,
    pub completed_at_ns: i64,
}

/// Un job recuperado en startup (TTR-004): su estado previo (antes de la
/// recuperación) y su identidad, listo para reencolarse y auditarse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveredJob {
    pub job: Job,
    pub previous_state: JobState,
}

/// Repositorio para `jobs` y `job_results`.
///
/// Constrúyelo con un [`SqlitePool`] ya migrado (ver
/// [`crate::persistence::pool::connect`] +
/// [`crate::persistence::pool::migrate`]) y cualquier implementación de
/// [`Clock`] (producción: [`crate::orchestrator::SystemClock`];
/// tests/backtests: [`crate::domain::clock::DeterministicClock`]).
pub struct JobRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> JobRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`. Ambos se toman
    /// prestados por la vida del repositorio — no se toma ownership, así
    /// que el mismo pool/clock se puede compartir con otros repositorios.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Persiste un job nuevo en estado `QUEUED` y lo devuelve (TTR-001:
    /// "Job se guarda ANTES de retornar UUID").
    ///
    /// Genera un UUID v7 fresco (`id`, ordenable por tiempo — ADR-0141
    /// "Claves primarias", confinado a esta cáscara según ADR-0002/0004) y
    /// lee el [`Clock`] actual (`created_at_ns` == `updated_at_ns` para una
    /// fila recién creada). `row_version` arranca en `1` y
    /// `audit_chain_hash` es `None` para un job nuevo (la cadena de
    /// actualizaciones propia de este job todavía no tiene predecesor).
    ///
    /// El `INSERT` de esta llamada es el límite de durabilidad: quien
    /// llama recibe el UUID del job (vía el [`Job::id`] devuelto) solo
    /// después de que este `INSERT` se completó, nunca antes
    /// (persist-before-ack).
    pub async fn submit(&self, request: NewJob) -> Result<Job, JobRepositoryError> {
        let id = Uuid::now_v7().to_string();
        let now_ns = self.clock.timestamp_ns();
        let state = JobState::Queued;
        let progress: u8 = 0;
        let row_version: i64 = 1;
        let audit_hash = compute_job_audit_hash(
            &id,
            now_ns,
            row_version,
            None,
            &request.user_id,
            &request.job_type,
            &request.parameters,
            state,
            progress,
        );

        sqlx::query(
            "INSERT INTO jobs (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                process_id, session_id, node_id, logic_hash, \
                owner_id, access_token_id, \
                user_id, job_type, parameters, state, progress\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(Option::<String>::None)
        .bind(row_version)
        .bind(Option::<String>::None) // process_id: sin asignar hasta que un worker lo tome
        .bind(&request.session_id)
        .bind(&request.node_id)
        .bind(&request.logic_hash)
        .bind(&request.owner_id)
        .bind(&request.access_token_id)
        .bind(&request.user_id)
        .bind(&request.job_type)
        .bind(&request.parameters)
        .bind(state.as_str())
        .bind(progress as i64)
        .execute(self.pool)
        .await?;

        Ok(Job {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: None,
            row_version,
            process_id: None,
            session_id: request.session_id,
            node_id: request.node_id,
            logic_hash: request.logic_hash,
            owner_id: request.owner_id,
            access_token_id: request.access_token_id,
            user_id: request.user_id,
            job_type: request.job_type,
            parameters: request.parameters,
            state,
            progress,
        })
    }

    /// Carga un único job por `id`, o `None` si no existe.
    pub async fn find(&self, id: &str) -> Result<Option<Job>, JobRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    process_id, session_id, node_id, logic_hash, \
                    owner_id, access_token_id, \
                    user_id, job_type, parameters, state, progress \
             FROM jobs WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(row_to_job(row)?)),
            None => Ok(None),
        }
    }

    /// Carga todos los jobs que están actualmente en `state` (TTR-004: la
    /// recuperación en startup escanea por `QUEUED` y `RUNNING`).
    pub async fn jobs_in_state(&self, state: JobState) -> Result<Vec<Job>, JobRepositoryError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    process_id, session_id, node_id, logic_hash, \
                    owner_id, access_token_id, \
                    user_id, job_type, parameters, state, progress \
             FROM jobs WHERE state = ? ORDER BY created_at ASC",
        )
        .bind(state.as_str())
        .fetch_all(self.pool)
        .await?;

        rows.into_iter().map(row_to_job).collect()
    }

    /// Transiciona `job` a `to`, validando la transición vía
    /// [`validate_transition`] antes de escribir nada.
    ///
    /// Incrementa `updated_at` (lectura actual de [`Clock`]),
    /// `row_version` (+1) y fija `audit_chain_hash` al `audit_hash`
    /// previo del job — la misma forma de cadena de hashes que
    /// `audit_events`, acotada al historial de filas propio de este job.
    /// Al transicionar a `RUNNING`, `process_id` se fija a `worker_id`
    /// (TTR-001/002 "Worker ID"); para cualquier otra transición
    /// `process_id` queda sin cambios.
    ///
    /// Devuelve el [`Job`] actualizado si tiene éxito.
    pub async fn transition(
        &self,
        job: &Job,
        to: JobState,
        worker_id: Option<&str>,
    ) -> Result<Job, JobRepositoryError> {
        validate_transition(job.state, to)?;

        let now_ns = self.clock.timestamp_ns();
        let row_version = job.row_version + 1;
        let progress = match to {
            JobState::Running => 0,
            JobState::Completed => 100,
            _ => job.progress,
        };
        let process_id = match to {
            JobState::Running => worker_id.map(str::to_string).or_else(|| job.process_id.clone()),
            _ => job.process_id.clone(),
        };

        let audit_hash = compute_job_audit_hash(
            &job.id,
            now_ns,
            row_version,
            Some(&job.audit_hash),
            &job.user_id,
            &job.job_type,
            &job.parameters,
            to,
            progress,
        );

        sqlx::query(
            "UPDATE jobs SET \
                updated_at = ?, audit_hash = ?, audit_chain_hash = ?, row_version = ?, \
                process_id = ?, state = ?, progress = ? \
             WHERE id = ?",
        )
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&job.audit_hash)
        .bind(row_version)
        .bind(&process_id)
        .bind(to.as_str())
        .bind(progress as i64)
        .bind(&job.id)
        .execute(self.pool)
        .await?;

        Ok(Job {
            id: job.id.clone(),
            created_at_ns: job.created_at_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: Some(job.audit_hash.clone()),
            row_version,
            process_id,
            session_id: job.session_id.clone(),
            node_id: job.node_id.clone(),
            logic_hash: job.logic_hash.clone(),
            owner_id: job.owner_id.clone(),
            access_token_id: job.access_token_id.clone(),
            user_id: job.user_id.clone(),
            job_type: job.job_type.clone(),
            parameters: job.parameters.clone(),
            state: to,
            progress,
        })
    }

    /// Actualiza `progress` (0-100, clampeado por
    /// [`crate::domain::job::Progress`]) para un job en estado `RUNNING`,
    /// sin cambiar su `state` (TTR-005: "Worker actualiza progreso cada
    /// `progress_interval` segundos").
    ///
    /// Incrementa `updated_at`, `row_version` y `audit_chain_hash`
    /// igual que [`Self::transition`]. Devuelve el [`Job`] actualizado.
    pub async fn update_progress(&self, job: &Job, progress: crate::domain::job::Progress) -> Result<Job, JobRepositoryError> {
        let now_ns = self.clock.timestamp_ns();
        let row_version = job.row_version + 1;
        let progress_value = progress.value();

        let audit_hash = compute_job_audit_hash(
            &job.id,
            now_ns,
            row_version,
            Some(&job.audit_hash),
            &job.user_id,
            &job.job_type,
            &job.parameters,
            job.state,
            progress_value,
        );

        sqlx::query(
            "UPDATE jobs SET \
                updated_at = ?, audit_hash = ?, audit_chain_hash = ?, row_version = ?, progress = ? \
             WHERE id = ?",
        )
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&job.audit_hash)
        .bind(row_version)
        .bind(progress_value as i64)
        .bind(&job.id)
        .execute(self.pool)
        .await?;

        Ok(Job {
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: Some(job.audit_hash.clone()),
            row_version,
            progress: progress_value,
            ..job.clone()
        })
    }

    /// Agrega un [`JobResult`] para un job que recién alcanzó un estado
    /// terminal (TTR-003). Solo-apéndice: este es el único camino de
    /// escritura para `job_results`, y la base de datos además rechaza
    /// `UPDATE`/`DELETE` vía triggers (migración `0003_jobs.sql`).
    ///
    /// ## Atomicidad bajo concurrencia (regla "Atomicidad de ledgers append-only")
    ///
    /// El *read-then-write* (leer la cola de la cadena de `job_results` y el
    /// `INSERT` final) ocurre dentro de UNA sola transacción
    /// `BEGIN IMMEDIATE` -- ver [`Self::try_record_once`]. Sin esa
    /// transacción, dos escritores concurrentes derivarían el mismo
    /// `event_sequence_id`, el `UNIQUE` rechazaría a uno y su resultado se
    /// PERDERÍA. Ante contención transitoria se reintenta hasta
    /// [`MAX_RECORD_ATTEMPTS`] veces re-derivando la secuencia; el resultado
    /// NUNCA se descarta en silencio (si se agotan los reintentos se
    /// devuelve [`JobRepositoryError::WriteContention`]).
    pub async fn record_result(&self, new_result: NewJobResult) -> Result<JobResult, JobRepositoryError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_record_once(&new_result).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    // Solo se reintenta ante contención de escritura
                    // transitoria; cualquier otro error se propaga de
                    // inmediato.
                    if is_transient_write_conflict(&error) {
                        if attempt < MAX_RECORD_ATTEMPTS {
                            continue;
                        }
                        // Agotados los reintentos: error tipado, NUNCA
                        // pérdida silenciosa del resultado.
                        return Err(JobRepositoryError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    /// Un intento único de [`Self::record_result`], dentro de una
    /// transacción `BEGIN IMMEDIATE`. Devuelve el error tal cual si algo
    /// falla -- el bucle de `record_result` decide si es transitorio y hay
    /// que reintentar. `BEGIN IMMEDIATE` toma el lock de escritura de
    /// ENTRADA: así ningún otro escritor puede intercalar entre la lectura
    /// de la cola y el `INSERT`, y se evita el interbloqueo de upgrade que
    /// ocurriría si dos transacciones DEFERRED intentaran subir de lectura
    /// a escritura a la vez.
    async fn try_record_once(&self, new_result: &NewJobResult) -> Result<JobResult, JobRepositoryError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        // Lectura (DENTRO de la transacción) -- la cola actual de la cadena
        // GLOBAL de job_results, para derivar el próximo event_sequence_id
        // y encadenar el audit_hash.
        let tail_row = sqlx::query("SELECT audit_hash, event_sequence_id FROM job_results ORDER BY event_sequence_id DESC LIMIT 1")
            .fetch_optional(&mut *tx)
            .await?;

        let (event_sequence_id, previous_hash) = match tail_row {
            Some(row) => {
                let previous_seq: i64 = row.get("event_sequence_id");
                let previous_hash: String = row.get("audit_hash");
                (previous_seq + 1, Some(previous_hash))
            }
            None => (1, None),
        };

        let id = Uuid::now_v7().to_string();
        let now_ns = self.clock.timestamp_ns();

        let audit_hash = compute_result_audit_hash(
            &id,
            now_ns,
            event_sequence_id,
            previous_hash.as_deref(),
            &new_result.job_uuid,
            new_result.result_data.as_deref(),
            new_result.error_message.as_deref(),
        );

        // Escritura (DENTRO de la transacción) -- el INSERT que cierra el
        // read-then-write atómico.
        sqlx::query(
            "INSERT INTO job_results (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                job_uuid, result_data, error_message, completed_at\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&previous_hash)
        .bind(event_sequence_id)
        .bind(&new_result.job_uuid)
        .bind(&new_result.result_data)
        .bind(&new_result.error_message)
        .bind(now_ns)
        .execute(&mut *tx)
        .await?;

        // Confirma la transacción: recién aquí el lock de escritura se
        // libera y la fila se hace visible a otros escritores.
        tx.commit().await?;

        Ok(JobResult {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: previous_hash,
            event_sequence_id,
            job_uuid: new_result.job_uuid.clone(),
            result_data: new_result.result_data.clone(),
            error_message: new_result.error_message.clone(),
            completed_at_ns: now_ns,
        })
    }

    /// Carga el (a lo sumo uno) resultado registrado para `job_uuid`, o
    /// `None` si el job todavía no completó/falló.
    pub async fn result_for_job(&self, job_uuid: &str) -> Result<Option<JobResult>, JobRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    job_uuid, result_data, error_message, completed_at \
             FROM job_results WHERE job_uuid = ? ORDER BY event_sequence_id ASC LIMIT 1",
        )
        .bind(job_uuid)
        .fetch_optional(self.pool)
        .await?;

        Ok(row.map(row_to_result))
    }
}

/// Convierte una fila de `jobs` al tipo [`Job`].
fn row_to_job(row: sqlx::sqlite::SqliteRow) -> Result<Job, JobRepositoryError> {
    let state_value: String = row.get("state");
    let state = JobState::from_str_value(&state_value)
        .ok_or_else(|| JobRepositoryError::UnknownState(state_value.clone()))?;
    let progress: i64 = row.get("progress");

    Ok(Job {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        row_version: row.get("row_version"),
        process_id: row.get("process_id"),
        session_id: row.get("session_id"),
        node_id: row.get("node_id"),
        logic_hash: row.get("logic_hash"),
        owner_id: row.get("owner_id"),
        access_token_id: row.get("access_token_id"),
        user_id: row.get("user_id"),
        job_type: row.get("job_type"),
        parameters: row.get("parameters"),
        state,
        progress: progress as u8,
    })
}

/// Convierte una fila de `job_results` al tipo [`JobResult`].
fn row_to_result(row: sqlx::sqlite::SqliteRow) -> JobResult {
    JobResult {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        event_sequence_id: row.get("event_sequence_id"),
        job_uuid: row.get("job_uuid"),
        result_data: row.get("result_data"),
        error_message: row.get("error_message"),
        completed_at_ns: row.get("completed_at"),
    }
}

/// Calcula un hash de snapshot SHA-256 determinista para una fila de
/// `jobs`, encadenado al `audit_hash` previo de la fila (o `None` para un
/// job recién enviado — misma convención "GENESIS" que
/// [`crate::domain::audit_log::GENESIS_PREVIOUS_HASH`]).
#[allow(clippy::too_many_arguments)]
fn compute_job_audit_hash(
    id: &str,
    updated_at_ns: i64,
    row_version: i64,
    previous_audit_hash: Option<&str>,
    user_id: &str,
    job_type: &str,
    parameters: &str,
    state: JobState,
    progress: u8,
) -> String {
    use sha2::{Digest, Sha256};
    const SEP: char = '\u{1F}';

    let mut buffer = String::new();
    let mut push = |field: &str| {
        buffer.push_str(field);
        buffer.push(SEP);
    };

    push(id);
    push(&updated_at_ns.to_string());
    push(&row_version.to_string());
    push(previous_audit_hash.unwrap_or(crate::domain::audit_log::GENESIS_PREVIOUS_HASH));
    push(user_id);
    push(job_type);
    push(parameters);
    push(state.as_str());
    push(&progress.to_string());

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    let digest = hasher.finalize();

    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Calcula un hash de snapshot SHA-256 determinista para una fila de
/// `job_results`, encadenado al `audit_hash` de la fila de resultado
/// previa (o `None` para el primer resultado registrado).
fn compute_result_audit_hash(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    previous_audit_hash: Option<&str>,
    job_uuid: &str,
    result_data: Option<&str>,
    error_message: Option<&str>,
) -> String {
    use sha2::{Digest, Sha256};
    const SEP: char = '\u{1F}';

    let mut buffer = String::new();
    let mut push = |field: &str| {
        buffer.push_str(field);
        buffer.push(SEP);
    };

    push(id);
    push(&created_at_ns.to_string());
    push(&event_sequence_id.to_string());
    push(previous_audit_hash.unwrap_or(crate::domain::audit_log::GENESIS_PREVIOUS_HASH));
    push(job_uuid);
    push(result_data.unwrap_or(""));
    push(error_message.unwrap_or(""));

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    let digest = hasher.finalize();

    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("connect in-memory db");
        migrate(&pool).await.expect("apply migrations");
        pool
    }

    fn sample_new_job() -> NewJob {
        NewJob {
            user_id: "user-1".to_string(),
            job_type: "BACKTEST".to_string(),
            parameters: "{\"strategy_id\":123}".to_string(),
            owner_id: Some("owner-1".to_string()),
            access_token_id: None,
            session_id: Some("session-1".to_string()),
            node_id: Some("node-1".to_string()),
            logic_hash: Some("executor-v1".to_string()),
        }
    }

    /// TTR-001: enviar un job lo persiste en estado `QUEUED` con progreso
    /// 0, y el UUID devuelto corresponde a una fila que ya existe en
    /// `jobs` (persist-before-ack).
    #[tokio::test]
    async fn submit_persists_job_in_queued_state() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job = repo.submit(sample_new_job()).await.expect("submit job");

        assert_eq!(job.state, JobState::Queued);
        assert_eq!(job.progress, 0);
        assert_eq!(job.row_version, 1);
        assert_eq!(job.audit_chain_hash, None);

        let found = repo.find(&job.id).await.expect("find job").expect("job exists");
        assert_eq!(found, job);
    }

    /// TTR-002: un job en cola transiciona a RUNNING, su process_id se
    /// fija al id del worker, y el progreso se reinicia a 0.
    #[tokio::test]
    async fn transition_to_running_sets_worker_and_resets_progress() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job = repo.submit(sample_new_job()).await.expect("submit job");
        clock.tick();
        let running = repo
            .transition(&job, JobState::Running, Some("worker-7"))
            .await
            .expect("transition to running");

        assert_eq!(running.state, JobState::Running);
        assert_eq!(running.process_id, Some("worker-7".to_string()));
        assert_eq!(running.progress, 0);
        assert_eq!(running.row_version, 2);
        assert_eq!(running.audit_chain_hash, Some(job.audit_hash.clone()));
        assert_ne!(running.audit_hash, job.audit_hash);
    }

    /// TTR-002/003: RUNNING -> COMPLETED fija el progreso a 100.
    #[tokio::test]
    async fn transition_to_completed_sets_progress_100() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job = repo.submit(sample_new_job()).await.expect("submit job");
        clock.tick();
        let running = repo
            .transition(&job, JobState::Running, Some("worker-7"))
            .await
            .expect("transition to running");
        clock.tick();
        let completed = repo
            .transition(&running, JobState::Completed, None)
            .await
            .expect("transition to completed");

        assert_eq!(completed.state, JobState::Completed);
        assert_eq!(completed.progress, 100);
    }

    /// Una transición inválida (ej. QUEUED -> COMPLETED) se rechaza antes
    /// de cualquier escritura, y la fila almacenada queda intacta.
    #[tokio::test]
    async fn transition_rejects_invalid_state_change() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job = repo.submit(sample_new_job()).await.expect("submit job");
        let result = repo.transition(&job, JobState::Completed, None).await;

        assert!(matches!(result, Err(JobRepositoryError::InvalidTransition(_))));

        let found = repo.find(&job.id).await.expect("find job").expect("job exists");
        assert_eq!(found.state, JobState::Queued);
        assert_eq!(found.row_version, 1);
    }

    /// TTR-005: `update_progress` actualiza el progreso sin cambiar el
    /// estado, e incrementa la cadena.
    #[tokio::test]
    async fn update_progress_changes_progress_without_changing_state() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job = repo.submit(sample_new_job()).await.expect("submit job");
        clock.tick();
        let running = repo
            .transition(&job, JobState::Running, Some("worker-7"))
            .await
            .expect("transition to running");
        clock.tick();
        let progressed = repo
            .update_progress(&running, crate::domain::job::Progress::new(45))
            .await
            .expect("update progress");

        assert_eq!(progressed.state, JobState::Running);
        assert_eq!(progressed.progress, 45);
        assert_eq!(progressed.row_version, 3);
    }

    /// TTR-004: `jobs_in_state` devuelve solo los jobs que coinciden con
    /// el estado solicitado.
    #[tokio::test]
    async fn jobs_in_state_filters_correctly() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job_a = repo.submit(sample_new_job()).await.expect("submit job a");
        let job_b = repo.submit(sample_new_job()).await.expect("submit job b");
        clock.tick();
        repo.transition(&job_b, JobState::Running, Some("worker-1"))
            .await
            .expect("transition job b to running");

        let queued = repo.jobs_in_state(JobState::Queued).await.expect("query queued");
        assert_eq!(queued.len(), 1);
        assert_eq!(queued[0].id, job_a.id);

        let running = repo.jobs_in_state(JobState::Running).await.expect("query running");
        assert_eq!(running.len(), 1);
        assert_eq!(running[0].id, job_b.id);
    }

    /// TTR-003: registrar un resultado para un job completado lo persiste
    /// y se puede recuperar vía `result_for_job`.
    #[tokio::test]
    async fn record_result_persists_and_is_retrievable() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job = repo.submit(sample_new_job()).await.expect("submit job");
        clock.tick();
        let running = repo
            .transition(&job, JobState::Running, Some("worker-1"))
            .await
            .expect("transition to running");
        clock.tick();
        let completed = repo
            .transition(&running, JobState::Completed, None)
            .await
            .expect("transition to completed");

        let result = repo
            .record_result(NewJobResult {
                job_uuid: completed.id.clone(),
                result_data: Some("{\"cagr\":0.25}".to_string()),
                error_message: None,
            })
            .await
            .expect("record result");

        assert_eq!(result.job_uuid, completed.id);
        assert_eq!(result.event_sequence_id, 1);
        assert_eq!(result.audit_chain_hash, None);

        let fetched = repo
            .result_for_job(&completed.id)
            .await
            .expect("query result")
            .expect("result exists");
        assert_eq!(fetched, result);
    }

    /// CRITERIO DE CIERRE de TTR-003: `job_results` es de solo-apéndice —
    /// UPDATE y DELETE los rechaza el trigger de la base de datos
    /// (migración 0003_jobs.sql), reflejando `audit_events`.
    #[tokio::test]
    async fn job_results_update_and_delete_are_rejected_by_triggers() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job = repo.submit(sample_new_job()).await.expect("submit job");
        clock.tick();
        let running = repo
            .transition(&job, JobState::Running, Some("worker-1"))
            .await
            .expect("transition to running");
        clock.tick();
        let completed = repo
            .transition(&running, JobState::Completed, None)
            .await
            .expect("transition to completed");

        let result = repo
            .record_result(NewJobResult {
                job_uuid: completed.id.clone(),
                result_data: Some("{\"cagr\":0.25}".to_string()),
                error_message: None,
            })
            .await
            .expect("record result");

        let update_result = sqlx::query("UPDATE job_results SET result_data = ? WHERE id = ?")
            .bind("{\"tampered\":true}")
            .bind(&result.id)
            .execute(&pool)
            .await;
        assert!(update_result.is_err(), "UPDATE on job_results must be rejected");

        let delete_result = sqlx::query("DELETE FROM job_results WHERE id = ?")
            .bind(&result.id)
            .execute(&pool)
            .await;
        assert!(delete_result.is_err(), "DELETE on job_results must be rejected");
    }

    /// Un segundo resultado se encadena al primero vía `audit_chain_hash`.
    #[tokio::test]
    async fn record_result_chains_sequential_results() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job_a = repo.submit(sample_new_job()).await.expect("submit job a");
        let job_b = repo.submit(sample_new_job()).await.expect("submit job b");

        let result_a = repo
            .record_result(NewJobResult {
                job_uuid: job_a.id.clone(),
                result_data: Some("{\"ok\":true}".to_string()),
                error_message: None,
            })
            .await
            .expect("record result a");

        let result_b = repo
            .record_result(NewJobResult {
                job_uuid: job_b.id.clone(),
                result_data: None,
                error_message: Some("Invalid date range".to_string()),
            })
            .await
            .expect("record result b");

        assert_eq!(result_a.event_sequence_id, 1);
        assert_eq!(result_b.event_sequence_id, 2);
        assert_eq!(result_b.audit_chain_hash, Some(result_a.audit_hash));
    }

    // ── Atomicidad bajo concurrencia (STORY-046, patrón de audit_log.rs / data_portability.rs) ──

    /// CRITERIO DE CIERRE (QA por mutación): bajo contención de escritura
    /// SOSTENIDA (otro escritor retiene el lock de `BEGIN IMMEDIATE` y no lo
    /// suelta), el bucle de reintento de `record_result` debe agotar
    /// EXACTAMENTE `MAX_RECORD_ATTEMPTS` intentos y rendirse con
    /// `WriteContention { attempts: MAX }` -- nunca descartar el resultado
    /// en silencio. Fija de forma determinista: el incremento del contador
    /// (`attempt += 1`), el límite de corte (`attempt < MAX`), y que el
    /// clasificador trate "database is locked" como transitorio.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn record_result_exhausts_exactly_max_attempts_when_write_lock_is_held() {
        use std::str::FromStr;
        use std::time::Duration;

        use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("job_results_forced_contention.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());

        // Migrar con el pool normal (busy_timeout de 5s) y crear el job del
        // que colgará el resultado (job_results.job_uuid -> jobs.id es una
        // FK real desde que se activó foreign_keys=ON, hallazgo C1).
        let setup_pool = connect(&database_url).await.expect("conectar (setup)");
        migrate(&setup_pool).await.expect("migrar");
        let clock = DeterministicClock::new(1_000, 100);
        let setup_repo = JobRepository::new(&setup_pool, &clock);
        let job = setup_repo.submit(sample_new_job()).await.expect("crear job de prueba");
        setup_pool.close().await;

        // Opciones con busy_timeout=0: un lock ocupado falla de INMEDIATO con
        // "database is locked" en vez de esperar 5s -- contención
        // determinista y rápida.
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

        // Escritor B: intenta registrar un resultado mientras A retiene el
        // lock. Cada `try_record_once` abre `BEGIN IMMEDIATE`, choca con el
        // lock de A, falla con "database is locked" (transitorio) y
        // reintenta, hasta agotar MAX_RECORD_ATTEMPTS.
        let repo_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(immediate_opts())
            .await
            .expect("pool del repositorio");
        let repo = JobRepository::new(&repo_pool, &clock);

        let result = repo
            .record_result(NewJobResult {
                job_uuid: job.id.clone(),
                result_data: Some("{\"ok\":true}".to_string()),
                error_message: None,
            })
            .await;

        drop(lock_tx); // libera el lock (limpieza; el resultado ya está tomado)

        match result {
            Err(JobRepositoryError::WriteContention { attempts }) => {
                assert_eq!(
                    attempts, MAX_RECORD_ATTEMPTS,
                    "bajo contención sostenida debe agotar EXACTAMENTE MAX_RECORD_ATTEMPTS intentos"
                );
            }
            other => panic!(
                "se esperaba WriteContention {{ attempts: {MAX_RECORD_ATTEMPTS} }} bajo contención sostenida, se obtuvo: {other:?}"
            ),
        }
    }

    /// CRITERIO DE CIERRE (QA por mutación): `is_transient_write_conflict`
    /// distingue una violación UNIQUE PERMANENTE (la PK `id` de
    /// `job_results`, que NO se debe reintentar) de la contención
    /// transitoria. Fija que exige AMBAS condiciones (es violación UNIQUE
    /// **y** menciona `event_sequence_id`), no una sola (`&&` != `||`), y
    /// que no clasifica cualquier cosa como transitoria.
    #[tokio::test]
    async fn is_transient_is_false_for_a_permanent_non_sequence_unique_violation() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);
        let job = repo.submit(sample_new_job()).await.expect("crear job de prueba");

        // Inserta una fila válida y luego otra con el MISMO `id`: viola la
        // PRIMARY KEY `id`, NO el UNIQUE de `event_sequence_id`. Error UNIQUE
        // PERMANENTE cuyo mensaje NO menciona `event_sequence_id`.
        let insert_with_id_dup = |event_sequence_id: i64| {
            sqlx::query(
                "INSERT INTO job_results \
                 (id, created_at, updated_at, audit_hash, event_sequence_id, job_uuid, completed_at) \
                 VALUES ('dup-result-id', 1, 1, 'h', ?, ?, 1)",
            )
            .bind(event_sequence_id)
            .bind(&job.id)
            .execute(&pool)
        };
        insert_with_id_dup(1).await.expect("primera fila válida");
        let err = insert_with_id_dup(2)
            .await
            .expect_err("la segunda fila debe violar la PRIMARY KEY id");

        let permanent = JobRepositoryError::Database(err);
        assert!(
            !is_transient_write_conflict(&permanent),
            "una violación UNIQUE de la PK (no de event_sequence_id) es PERMANENTE, no transitoria"
        );

        // Control: un error que ni siquiera es de base de datos jamás es
        // transitorio (fija la rama temprana `let ... else`).
        let non_database = JobRepositoryError::UnknownState("X".to_string());
        assert!(
            !is_transient_write_conflict(&non_database),
            "un error no-Database nunca es contención transitoria"
        );
    }

    /// CRITERIO DE CIERRE (QA por mutación): la fila que DEVUELVE
    /// `record_result` refleja SIEMPRE valores frescos -- `audit_hash`
    /// recién computado, `audit_chain_hash` encadenado a la fila anterior y
    /// `created_at`/`updated_at` leídos del reloj EN ESE INSTANTE -- nunca
    /// datos por defecto ni copiados de una llamada previa.
    #[tokio::test]
    async fn record_result_returned_row_reflects_fresh_hash_chain_and_timestamp() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = JobRepository::new(&pool, &clock);

        let job = repo.submit(sample_new_job()).await.expect("crear job de prueba");

        let result_a = repo
            .record_result(NewJobResult {
                job_uuid: job.id.clone(),
                result_data: Some("{\"ok\":true}".to_string()),
                error_message: None,
            })
            .await
            .expect("registrar primer resultado");
        assert_eq!(result_a.created_at_ns, 1_000, "precondición: el primer resultado nace en el now inicial");
        assert!(result_a.audit_chain_hash.is_none(), "precondición: el primer resultado no encadena");

        clock.tick(); // 1_000 -> 1_100
        let job_b = repo.submit(sample_new_job()).await.expect("crear segundo job de prueba");
        let result_b = repo
            .record_result(NewJobResult {
                job_uuid: job_b.id.clone(),
                result_data: None,
                error_message: Some("boom".to_string()),
            })
            .await
            .expect("registrar segundo resultado");

        assert_ne!(
            result_b.audit_hash, result_a.audit_hash,
            "record_result debe DEVOLVER el audit_hash recién computado, no uno reutilizado"
        );
        assert_eq!(
            result_b.audit_chain_hash,
            Some(result_a.audit_hash.clone()),
            "el audit_chain_hash devuelto debe encadenar al audit_hash del resultado anterior"
        );
        assert_eq!(
            result_b.created_at_ns, 1_100,
            "el created_at devuelto debe ser el now del reloj tras el tick, no el viejo"
        );
        assert_eq!(result_b.event_sequence_id, 2);
    }
}
