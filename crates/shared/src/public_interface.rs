//! [SHELL] Interfaz pública (puerto) de `shared`.
//!
//! Esta es la única superficie de la que pueden depender los módulos del
//! pipeline (`ingest`, `generate`, `validate`, `incubate`, `manage`,
//! `execute`, `feedback`, `withdraw`) al reusar componentes comunes
//! (ADR-0003).
//!
//! ## Clock (W3, `docs/features/clock.md`)
//!
//! Cada módulo que necesita la hora actual depende del puerto [`Clock`]
//! en vez de llamar directo al reloj del sistema:
//!
//! - [`SystemClock`]: implementación de producción (TTR-001,
//!   `request_type = REAL`), con precisión de nanosegundos y monótona no
//!   decreciente.
//! - [`DeterministicClock`]: implementación de backtest/test (TTR-002,
//!   `request_type = FAKE`), que solo avanza vía llamadas explícitas a
//!   `advance(ns)` / `tick()` — la misma semilla
//!   (`initial_timestamp_ns`, `step_ns`) y la misma secuencia de llamadas
//!   producen una secuencia de timestamps idéntica, bit a bit.
//!
//! ## Audit Log (`docs/features/audit-log.md` TTR-001)
//!
//! Cada módulo dispara eventos de auditoría a través de
//! [`AuditLogRepository::append`] en vez de escribir logs directamente
//! (audit-log.md: "El Core nunca escribe logs. En su lugar, dispara
//! eventos al puerto de auditoría injected.").
//!
//! - [`AuditEventContent`]: el payload del evento (`action_type`,
//!   `entity_type`, `entity_id`, `details_json`, más los campos del
//!   perfil "Ops / Auditoría" de ADR-0020 V2 — `process_id` e
//!   `institutional_tag` son obligatorios).
//! - [`AuditEvent`]: un evento persistido y encadenado por hash
//!   (`audit_hash`, `audit_chain_hash`, `event_sequence_id`).
//! - [`AuditLogRepository`]: repositorio de solo-apéndice (`append`,
//!   `load_chain`, `events_for_entity`) — no existe superficie de
//!   update/delete.
//! - [`verify_chain`] / [`ChainVerificationResult`]: verificación pura de
//!   la cadena de hashes, detecta manipulación de eventos históricos.
//! - [`AuditLogError`]: tipo de error para operaciones del repositorio.
//!
//! ## Rastro de Auditoría del Clock (`docs/features/clock.md` "Gobernanza y Estándares")
//!
//! El Clock no tiene persistencia propia — sus tres eventos auditables se
//! emiten vía [`AuditLogRepository::append`] a través de
//! [`ClockAuditContext`] y las tres funciones `emit_*` de abajo. La
//! granularidad está fija en exactamente estos tres eventos;
//! `timestamp_ns()`, `advance(ns)` y `tick()` nunca emiten eventos de
//! auditoría.
//!
//! - [`ClockAuditContext`]: identidad provista por quien llama
//!   (`session_id`, `institutional_tag`, `process_id`) compartida por
//!   los tres eventos.
//! - [`ClockMode`]: `REAL` / `SIMULATION`, usado por
//!   [`emit_mode_transition`].
//! - [`emit_ntp_sync`]: `CLOCK_NTP_SYNC` (TTR-001, una vez al iniciar).
//! - [`emit_mode_transition`]: `CLOCK_MODE_TRANSITION` (en transiciones
//!   `REAL` <-> `SIMULATION`).
//! - [`emit_session_close`]: `CLOCK_SESSION_CLOSE` (TTR-002, una vez
//!   cuando cierra una sesión de simulación).
//!
//! ## Async Job Executor (`docs/features/async-job-executor.md`)
//!
//! Patrón de job asíncrono de tres fases (ADR-0011): enviar un job,
//! sondear su estado y progreso, recuperar su resultado inmutable una
//! vez terminal.
//!
//! - [`JobState`]: los cinco estados de la máquina de estados del job +
//!   [`validate_transition`] puro (TTR-002/004/006).
//! - [`Progress`] / [`estimate_remaining_seconds`]: progreso 0-100 y
//!   estimación de tiempo restante (TTR-005).
//! - [`Job`] / [`JobResult`] / [`NewJob`] / [`NewJobResult`] /
//!   [`RecoveredJob`]: tipos de la capa de persistencia
//!   (`jobs`/`job_results`, migración `0003_jobs.sql`).
//! - [`JobRepository`] / [`JobRepositoryError`]: el repositorio de
//!   `jobs`/`job_results` (TTR-001/003/004).
//! - [`JobExecutor`] / [`JobExecutorConfig`] / [`ExecutorIdentity`] /
//!   [`JobExecutorError`]: la cáscara del executor — enviar, recuperar
//!   en startup, levantar el pool de workers, sondear estado/resultado,
//!   cancelar (TTR-001/002/004/006).
//! - [`JobHandler`] / [`JobOutcome`] / [`ProgressReporter`] /
//!   [`CancellationToken`]: el contrato de callback enchufable por
//!   `job_type` (TTR-002/005/006). TTR-ASYNC-EXECUTOR-007 (conectar
//!   handlers reales desde `generate`/`validate`/`manage`/`incubate`/
//!   `feedback`) está fuera de alcance para esta historia.
//!
//! ## Telemetría (`docs/features/telemetry.md` TTR-001)
//!
//! Buffer de alta velocidad: cualquier módulo registra una muestra de
//! latencia o un heartbeat sin esperar al disco; una tarea de fondo vacía
//! la cola a SQLite por lotes.
//!
//! - [`TelemetrySample`] / [`TelemetrySampleContent`] / [`build_sample`] /
//!   [`expired_sample_ids`]: núcleo puro — construcción de una muestra
//!   encadenada y la decisión de poda por ventana de retención.
//! - [`TelemetryRepository`] / [`TelemetryError`]: repositorio de
//!   `telemetry_samples` (insertar por lote, purgar, consultar por
//!   `metric_name` + rango, migración `0004_telemetry.sql`).
//! - [`TelemetryBuffer`] / [`TelemetryBufferConfig`]: la cáscara — cola en
//!   memoria no bloqueante (`record_latency`/`record_heartbeat`), siembra
//!   de la cadena al iniciar (`bootstrap`), vaciado por lotes en segundo
//!   plano (`spawn_flush_task`) y poda (`purge`). Reusa [`ExecutorIdentity`]
//!   del Async Job Executor — mismo perfil de campos ADR-0020 V2, no se
//!   duplica el tipo.
//!
//! ## Central Identity (`docs/features/central-identity.md`, ADR-0143,
//! ## ADR-0144, STORY-027)
//!
//! Cimiento #1 del substrato de monetización: la cuenta LOCAL de usuario.
//! `licensing-system`, `usage-metering` y `consent-registry` dependen de su
//! `owner_id`.
//!
//! - [`AccountIdentity`]: el tipo de puerto `identity_out` (ADR-0137,
//!   catálogo) — identidad de cuenta + estado de verificación, SIN
//!   secretos (ADR-0093).
//! - [`compute_hardware_fingerprint`] / [`validate_email_format`] /
//!   [`verify_oauth_signature`]: el núcleo puro (sin I/O, ADR-0002/0004).
//! - [`Account`] / [`NewAccount`] / [`AccountRepository`]: la tabla
//!   `accounts` (migración `0007_central_identity.sql`), MUTABLE con
//!   `row_version` (ADR-0141), no append-only.
//! - [`IdentityCache`] / [`IdentityCacheConfig`]: caché local con TTL
//!   (`IDENTITY_CACHE_TTL`, default 24h) para operación offline.
//! - [`CentralIdentityVerifier`] / [`LocalStubCentralIdentityVerifier`]: el
//!   puerto de verificación contra la Cabina de Mando Central, con su
//!   implementación stub local (ADR-0144: "puerto ahora, adaptador
//!   después" — la Cabina de Mando todavía no existe).
//! - [`verify_central_identity`]: harness CLI (Canal #2, ADR-0142) —
//!   `cargo run -p app -- verify central-identity --input '{"email":"a@b.com"}'`.

pub use crate::clock_audit::{
    emit_mode_transition, emit_ntp_sync, emit_session_close, ClockAuditContext, ClockMode,
};
pub use crate::domain::audit_log::{
    AuditEvent, AuditEventContent, ChainVerificationResult, verify_chain,
};
pub use crate::domain::central_identity::{
    compute_account_audit_hash, compute_hardware_fingerprint, normalize_email,
    validate_email_format, verify_oauth_signature, AccountIdentity, EmailFormatError,
    EmailVerificationStatus, HardwareFingerprintError, OAuthTokenMaterial,
};
pub use crate::orchestrator::central_identity::{
    CentralIdentityError, CentralIdentityVerifier, IdentityCache, IdentityCacheConfig,
    IdentityVerificationRequest, LocalStubCentralIdentityVerifier,
};
pub use crate::persistence::central_identity::{
    Account, AccountRepository, AccountRepositoryError, NewAccount,
};
pub use crate::domain::clock::{Clock, DeterministicClock};
pub use crate::domain::job::{estimate_remaining_seconds, validate_transition, InvalidTransition, JobState, Progress};
pub use crate::orchestrator::job_executor::{
    CancellationToken, ExecutorIdentity, JobExecutor, JobExecutorConfig, JobExecutorError, JobHandler, JobOutcome,
    ProgressReporter, JOB_RECOVERED_AT_STARTUP,
};
pub use crate::orchestrator::telemetry::{TelemetryBuffer, TelemetryBufferConfig};
pub use crate::orchestrator::SystemClock;
pub use crate::persistence::audit_log::{AuditLogError, AuditLogRepository};
pub use crate::persistence::pool::{connect as create_pool, migrate as run_migrations};
pub use crate::persistence::job::{Job, JobRepository, JobRepositoryError, JobResult, NewJob, NewJobResult, RecoveredJob};
pub use crate::persistence::telemetry::{TelemetryError, TelemetryRepository};
pub use crate::domain::telemetry::{build_sample, expired_sample_ids, TelemetrySample, TelemetrySampleContent};
pub use crate::domain::worker_orchestrator::{WorkerBackend, WorkerBackendError, WorkerConfig, WorkerOrchestrator};
pub use crate::orchestrator::worker_runner::{graceful_shutdown, is_process_alive, open_readonly, OsWorkerBackend, SharedMemorySegment, ShmError};
pub use crate::domain::mcp_gateway::{
    evaluate_permission, compute_audit_hash, outcome_to_string, institutional_tag_to_string,
    InstitutionalTag, PermissionDecision, PermissionOutcome, PermissionRequest, Pipeline,
};
pub use crate::orchestrator::mcp_server::run_mcp_server;
pub use crate::persistence::mcp_gateway::{McpGatewayError, McpGatewayRepository};

// ── Harness de verificación CLI de Central Identity (ADR-0142 Fase 1) ───────

/// Input para la verificación de Central Identity vía CLI (`docs/features/central-identity.md`,
/// STORY-027). Se deserializa desde el JSON que pasa el usuario con
/// `--input '...'`.
///
/// `email` es el único campo obligatorio: `cargo run -p app -- verify
/// central-identity --input '{"email":"a@b.com"}'` ya es una invocación
/// válida. El resto tiene valores por defecto razonables para una
/// verificación de humo rápida.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CentralIdentityVerifyInput {
    /// Correo con el que se registra/vincula la cuenta.
    pub email: String,
    /// Proveedor de identidad federada, si el login fue vía OAuth.
    #[serde(default)]
    pub oauth_provider: Option<String>,
    /// Identificadores de máquina sin procesar para calcular la huella de
    /// hardware. Si se omite, usa el hostname del proceso como único
    /// identificador (suficiente para una verificación de humo local).
    #[serde(default)]
    pub machine_identifiers: Option<Vec<String>>,
    /// Entorno/etiqueta institucional de la cuenta.
    #[serde(default = "default_institutional_tag")]
    pub institutional_tag: String,
}

/// Valor por defecto de `institutional_tag` cuando el usuario no lo pasa en
/// `--input` -- una verificación de humo local no pertenece a ningún
/// entorno de producción real.
fn default_institutional_tag() -> String {
    "DRASUS_LOCAL_VERIFY".to_string()
}

/// Output de la verificación de Central Identity. Siempre serializa a JSON
/// válido (ADR-0142: "JSON estructurado en el CLI, FIJO").
///
/// Si `ok` es `true`, los campos de identidad están rellenos y coinciden
/// EXACTAMENTE con lo que expondría el puerto `identity_out` -- ningún
/// campo adicional, ningún secreto (ADR-0093).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CentralIdentityVerifyOutput {
    /// `true` si la verificación completó sin errores.
    pub ok: bool,
    pub owner_id: Option<String>,
    pub email: Option<String>,
    pub email_verification_status: Option<String>,
    pub node_id: Option<String>,
    pub institutional_tag: Option<String>,
    /// `true` si el valor devuelto salió de la caché con TTL en vez de una
    /// verificación fresca contra el verificador (en esta llamada de CLI,
    /// siempre pasa por ambos pasos: verifica y luego cachea, así que
    /// `cached` confirma que el cableado caché -> puerto quedó correcto).
    pub cached: bool,
    pub error: Option<String>,
}

impl CentralIdentityVerifyOutput {
    /// Construye un output de error con todos los campos de identidad en
    /// `None`.
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            owner_id: None,
            email: None,
            email_verification_status: None,
            node_id: None,
            institutional_tag: None,
            cached: false,
            error: Some(msg),
        }
    }

    /// Construye un output exitoso a partir de la identidad ya cacheada.
    fn from_identity(identity: AccountIdentity) -> Self {
        Self {
            ok: true,
            owner_id: Some(identity.owner_id),
            email: Some(identity.email),
            email_verification_status: Some(identity.email_verification_status.as_str().to_string()),
            node_id: Some(identity.node_id),
            institutional_tag: Some(identity.institutional_tag),
            cached: true,
            error: None,
        }
    }
}

/// Ejecuta la verificación de Central Identity con adaptadores reales
/// (BD SQLite temporal + reloj de sistema real + verificador stub local).
///
/// Crea una BD SQLite temporal exclusiva para esta verificación (mismo
/// patrón que `sovereign-data-fetcher::public_interface::verify`), aplica
/// las migraciones embebidas, verifica/vincula la identidad vía
/// [`LocalStubCentralIdentityVerifier`], la guarda en una [`IdentityCache`]
/// recién creada y devuelve lo que la caché reporta -- ejercitando el
/// camino completo Core -> Shell -> puerto que un usuario real recorrería.
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify central-identity --input '{"email":"a@b.com"}'`
pub async fn verify_central_identity(input: CentralIdentityVerifyInput) -> CentralIdentityVerifyOutput {
    // BD SQLite temporal exclusiva para esta verificación -- no contamina
    // datos de producción (mismo patrón que sovereign-data-fetcher::verify).
    let temp_dir = std::env::temp_dir().join(format!(
        "drasus-verify-central-identity-{}",
        uuid::Uuid::new_v4()
    ));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return CentralIdentityVerifyOutput::from_error(format!(
            "no se pudo crear el directorio temporal de verificación: {e}"
        ));
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return CentralIdentityVerifyOutput::from_error(format!(
                "no se pudo crear la BD temporal de verificación: {e}"
            ))
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return CentralIdentityVerifyOutput::from_error(format!(
            "error al aplicar migraciones en la BD temporal: {e}"
        ));
    }

    // Reloj de producción: la caché mide el TTL contra la hora real.
    // `Arc<dyn Clock>` porque tanto el verificador como la caché necesitan
    // su propia referencia compartida al mismo reloj.
    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());

    // Sin identificadores de máquina explícitos: usa el hostname del
    // proceso como único identificador -- suficiente para una verificación
    // de humo local (no se espera acceso a hardware real en CI).
    let machine_identifiers = input.machine_identifiers.unwrap_or_else(|| {
        vec![hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown-host".to_string())]
    });

    let verifier = crate::orchestrator::central_identity::LocalStubCentralIdentityVerifier::new(&pool, clock.as_ref());
    let request = crate::orchestrator::central_identity::IdentityVerificationRequest {
        email: input.email,
        oauth_provider: input.oauth_provider,
        machine_identifiers,
        institutional_tag: input.institutional_tag,
        access_token_id: None,
    };

    let identity = match verifier.verify_identity(request).await {
        Ok(identity) => identity,
        Err(e) => return CentralIdentityVerifyOutput::from_error(e.to_string()),
    };

    // Pasa por la caché con TTL antes de reportar -- ejercita el cableado
    // completo que el observable de la Story describe ("identidad cacheada
    // + estado"), no solo la verificación cruda.
    let cache = crate::orchestrator::central_identity::IdentityCache::new(
        clock,
        crate::orchestrator::central_identity::IdentityCacheConfig::default(),
    );
    cache.set(identity);

    match cache.get() {
        Some(cached_identity) => CentralIdentityVerifyOutput::from_identity(cached_identity),
        // Inalcanzable en la práctica: acabamos de guardar con TTL de 24h;
        // solo fallaría si el reloj del sistema saltara 24h entre `set` y
        // `get`, lo cual no ocurre en una sola invocación síncrona del CLI.
        None => CentralIdentityVerifyOutput::from_error(
            "la identidad recién guardada ya no está vigente en la caché (inesperado)".to_string(),
        ),
    }
}
