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
//!   perfil "Ops / Auditoría" de ADR-0020 — `process_id` e
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
//!   del Async Job Executor — mismo perfil de campos ADR-0020, no se
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
//!
//! ## Plan / Tier / Quota (`docs/features/plan-tier-quota.md`, ADR-0143,
//! ## ADR-0144, STORY-029)
//!
//! Cimiento #3 del substrato de monetización: el catálogo configurable de
//! planes. Produce el tipo de puerto `PlanLimits` que `licensing-system`
//! (#2) hoy consume por stub y que `usage-metering` (#4, futuro) necesitará.
//!
//! Vive bajo su propio submódulo público
//! ([`plan_tier_quota`]) en vez de aplanarse a este nivel superior: el
//! doc-comment de `plan_tier_quota` explica por qué (colisión de nombre
//! `PlanLimits` con el stub aún vigente en `licensing_system`).
//!
//! ## Usage Metering / Libro de Nocional (`docs/features/usage-metering.md`,
//! ## ADR-0143, ADR-0144, STORY-030)
//!
//! Cimiento #4 del substrato de monetización: el libro append-only de
//! nocional en USD por ciclo de facturación. Primer cimiento que consume
//! un puerto REAL de otro cimiento -- [`orchestrator::usage_metering::record_metered_operation`]
//! resuelve el `PlanLimits` REAL de `plan_tier_quota` (#3), no un stub.
//!
//! - [`domain::usage_metering::compute_notional`]: nocional de una
//!   operación, reescalado ×10¹⁶→×10⁸ con `i128` y redondeo explícito --
//!   EL punto de correctitud crítico de esta Story.
//! - [`domain::usage_metering::accumulate`] /
//!   [`domain::usage_metering::detect_quota_crossing`] /
//!   [`domain::usage_metering::derive_billing_cycle_id`]: acumulación por
//!   ciclo, veredicto de cuota y derivación del ciclo mensual.
//! - [`domain::usage_metering::MeteredOperation`]: entrada mínima de
//!   metering (placeholder hasta que el `Order` real de `execute`/EPIC-5
//!   exista).
//! - [`domain::usage_metering::UsageRecord`]: el tipo de puerto
//!   `usage_out` (acumulado + veredicto, sin secretos ADR-0093).
//! - [`persistence::usage_metering::UsageRepository`][]: repositorio
//!   APPEND-ONLY (`event_sequence_id`, ADR-0141) para `usage_records`
//!   (migración `0010_usage_metering.sql`).
//! - [`orchestrator::usage_metering::record_metered_operation`][]: la
//!   composición completa -- resuelve `PlanLimits` REAL + deriva el ciclo
//!   + persiste append-only.
//! - [`verify_usage_metering`]: harness CLI (Canal #2, ADR-0142) --
//!   `cargo run -p app -- verify usage-metering --input '{"tier":"FREE","operations":[...]}'`.
//!
//! ## Consent Registry / Registro de Consentimiento ToS (`docs/features/consent-registry.md`,
//! ## ADR-0143, ADR-0144, ADR-0141, STORY-031)
//!
//! Cimiento #5 del substrato de monetización: el registro append-only y
//! versionado de aceptación de ToS, con granularidad opt-in/opt-out por
//! tipo de dato -- la columna vertebral legal (GDPR) del firehose gratuito
//! (ADR-0143) y de `data-aggregation` (#9).
//!
//! - [`domain::consent_registry::needs_reacceptance`]: compara la versión
//!   aceptada contra la vigente (`REACCEPT_ON_VERSION_CHANGE`, FIJO).
//! - [`domain::consent_registry::resolve_coverage`]: EL punto de
//!   correctitud legal -- decide `Covered`/`NotCovered{reason}` para un
//!   tipo de dato; el default es SIEMPRE negar.
//! - [`domain::consent_registry::apply_consent_action`]: EL punto de
//!   modelado crítico -- fusiona el estado vigente con una acción nueva
//!   (aceptar versión / cambiar opt-outs) produciendo el snapshot
//!   COMPLETO que se persiste como fila-evento nueva (event-sourcing).
//! - [`domain::consent_registry::ConsentVerdict`]: el tipo de puerto
//!   `consent_out` (acumulado + veredicto, sin secretos ADR-0093).
//! - [`persistence::consent_registry::ConsentRepository`][]: repositorio
//!   APPEND-ONLY (`event_sequence_id`, ADR-0141) para `consent_records`
//!   (migración `0011_consent_registry.sql`).
//! - [`orchestrator::consent_registry::record_consent_action`] /
//!   [`orchestrator::consent_registry::resolve_consent_verdict`][]: la
//!   composición completa -- registrar un evento y resolver el veredicto
//!   de cobertura.
//! - [`verify_consent_registry`]: harness CLI (Canal #2, ADR-0142) --
//!   `cargo run -p app -- verify consent-registry --input
//!   '{"current_version":"v2","actions":[...],"query":{"data_type":"aggregation"}}'`.

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
pub use crate::domain::licensing_system::{
    canonical_license_bytes, derive_execution_gate, evaluate_heartbeat_status, hardware_matches,
    heartbeat_status_to_compliance_status_id, verify_license_signature, ExecutionGate,
    GateEvaluationInput, GateVerdict, HeartbeatConfig, HeartbeatStatus, LicensePayload,
    LicenseSignatureError, LicenseTier, PlanLimits, DEFAULT_HEARTBEAT_INTERVAL_NS,
};
pub use crate::orchestrator::licensing_system::{
    build_execution_gate, sync_compliance_status, BuildExecutionGateError, ExecutionGateCache,
    ExecutionGateCacheConfig, IssueLicenseRequest, LocalStubLicenseIssuer,
    LocalStubPlanLimitsProvider, PlanLimitsProvider, SignedLicenseFile,
};
pub use crate::persistence::licensing_system::{
    LicenseRecord, LicenseRepository, LicenseRepositoryError, NewLicenseActivation,
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

// ── Harness de verificación CLI de Licensing System (ADR-0142 Fase 1) ───────

/// Input para la verificación de Licensing System vía CLI (`docs/features/licensing-system.md`,
/// STORY-028). Se deserializa desde el JSON que pasa el usuario con
/// `--input '...'`.
///
/// `tier` es el único campo que un uso típico necesita:
/// `cargo run -p app -- verify licensing-system --input '{"tier":"SOVEREIGN"}'`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LicensingSystemVerifyInput {
    /// `"SOVEREIGN"` o `"EXPLORER"` (`docs/features/licensing-system.md`
    /// "Niveles de Licencia").
    #[serde(default = "default_license_tier")]
    pub tier: String,
    /// Correo de la cuenta local a vincular (vía `central-identity`, puerto
    /// `identity_in`). Si se omite, usa un correo fijo de verificación.
    #[serde(default = "default_owner_email")]
    pub owner_email: String,
}

fn default_license_tier() -> String {
    "SOVEREIGN".to_string()
}

fn default_owner_email() -> String {
    "verify-licensing@drasus.local".to_string()
}

/// Output de la verificación de Licensing System. Siempre serializa a JSON
/// válido (ADR-0142). Si `ok` es `true`, refleja EXACTAMENTE lo que expone
/// el puerto `execution_gate_out` -- ningún secreto (ADR-0093).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LicensingSystemVerifyOutput {
    pub ok: bool,
    pub verdict: Option<String>,
    pub tier: Option<String>,
    pub suppress_work_telemetry: Option<bool>,
    pub activations: Option<i64>,
    pub reason: Option<String>,
    pub error: Option<String>,
}

impl LicensingSystemVerifyOutput {
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            verdict: None,
            tier: None,
            suppress_work_telemetry: None,
            activations: None,
            reason: None,
            error: Some(msg),
        }
    }

    fn from_gate(gate: ExecutionGate) -> Self {
        let verdict = match gate.verdict {
            GateVerdict::Allow => "Allow",
            GateVerdict::Deny => "Deny",
            GateVerdict::UpgradeRequired => "UpgradeRequired",
        };
        Self {
            ok: true,
            verdict: Some(verdict.to_string()),
            tier: Some(gate.tier.as_str().to_string()),
            suppress_work_telemetry: Some(gate.suppress_work_telemetry),
            activations: Some(gate.activations),
            reason: Some(gate.reason),
            error: None,
        }
    }
}

/// Ejecuta la verificación de Licensing System con adaptadores reales (BD
/// SQLite temporal + reloj de sistema real + emisor de licencia stub +
/// proveedor de límites stub), recorriendo el camino completo del cimiento
/// #2: vincula una `AccountIdentity` local (reutiliza `central-identity`,
/// puerto `identity_in` -- NO recalcula la huella de hardware), emite y
/// activa una licencia de desarrollo firmada para el `tier` pedido, obtiene
/// `PlanLimits` del stub (puerto `plan_limits_in`), construye el
/// `ExecutionGate` y lo pasa por su caché con TTL antes de reportar.
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify licensing-system --input '{"tier":"SOVEREIGN"}'`
pub async fn verify_licensing_system(input: LicensingSystemVerifyInput) -> LicensingSystemVerifyOutput {
    let tier = match LicenseTier::from_str_value(&input.tier) {
        Some(tier) => tier,
        None => {
            return LicensingSystemVerifyOutput::from_error(format!(
                "tier desconocido: '{}' -- se esperaba SOVEREIGN o EXPLORER",
                input.tier
            ))
        }
    };

    // BD SQLite temporal exclusiva para esta verificación (mismo patrón que
    // verify_central_identity).
    let temp_dir = std::env::temp_dir().join(format!(
        "drasus-verify-licensing-system-{}",
        uuid::Uuid::new_v4()
    ));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return LicensingSystemVerifyOutput::from_error(format!(
            "no se pudo crear el directorio temporal de verificación: {e}"
        ));
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return LicensingSystemVerifyOutput::from_error(format!(
                "no se pudo crear la BD temporal de verificación: {e}"
            ))
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return LicensingSystemVerifyOutput::from_error(format!(
            "error al aplicar migraciones en la BD temporal: {e}"
        ));
    }

    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());

    // Paso 1 -- identity_in: vincula/crea la AccountIdentity local vía
    // central-identity (REUTILIZA su huella de hardware, no la recalcula).
    let machine_identifiers = vec![hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string())];
    let identity_verifier =
        crate::orchestrator::central_identity::LocalStubCentralIdentityVerifier::new(&pool, clock.as_ref());
    let identity = match identity_verifier
        .verify_identity(crate::orchestrator::central_identity::IdentityVerificationRequest {
            email: input.owner_email,
            oauth_provider: None,
            machine_identifiers,
            institutional_tag: "DRASUS_LOCAL_VERIFY".to_string(),
            access_token_id: None,
        })
        .await
    {
        Ok(identity) => identity,
        Err(e) => return LicensingSystemVerifyOutput::from_error(format!("fallo al vincular identidad: {e}")),
    };

    // Paso 2 -- emisor stub: firma una licencia de desarrollo para esta
    // cuenta + esta máquina (la clave privada nunca sale de `issuer`).
    let issuer = LocalStubLicenseIssuer::new();
    let now_ns = clock.timestamp_ns();
    let signed = issuer.issue_license(IssueLicenseRequest {
        owner_id: identity.owner_id.clone(),
        node_id: identity.node_id.clone(),
        tier,
        issued_at_ns: now_ns,
        heartbeat_expires_at_ns: now_ns + DEFAULT_HEARTBEAT_INTERVAL_NS,
    });

    // Paso 3 -- activa (persiste) la licencia firmada para esta máquina.
    let license_repo = LicenseRepository::new(&pool, clock.as_ref());
    let license = match license_repo
        .activate(NewLicenseActivation {
            owner_id: identity.owner_id.clone(),
            institutional_tag: identity.institutional_tag.clone(),
            access_token_id: None,
            node_id: identity.node_id.clone(),
            license_id: signed.license_id.clone(),
            process_id: Some(format!("drasus-pid-{}", std::process::id())),
            signature_hash: signed.signature_hex.clone(),
            tier,
            issued_at_ns: signed.issued_at_ns,
            heartbeat_expires_at_ns: signed.heartbeat_expires_at_ns,
            compliance_status_id: "ACTIVE".to_string(),
        })
        .await
    {
        Ok(license) => license,
        Err(e) => return LicensingSystemVerifyOutput::from_error(format!("fallo al activar licencia: {e}")),
    };

    // Paso 4 -- plan_limits_in: límites del stub (plan-tier-quota real, diferido).
    let plan_limits_provider = LocalStubPlanLimitsProvider::default();
    let plan_limits = plan_limits_provider.plan_limits_for(&identity.owner_id, tier).await;

    // Paso 5 -- construye el veredicto (fuera del hot-path: esta función SÍ
    // hace lecturas de BD local; el hot-path real solo leería la caché).
    let heartbeat_config = HeartbeatConfig::default();
    let gate = match build_execution_gate(
        &pool,
        clock.as_ref(),
        &identity.node_id,
        &license,
        &signed.signature_hex,
        &signed.public_key_hex,
        &heartbeat_config,
        &plan_limits,
    )
    .await
    {
        Ok(gate) => gate,
        Err(e) => return LicensingSystemVerifyOutput::from_error(format!("fallo al construir el gate: {e}")),
    };

    // Paso 6 -- pasa por la caché con TTL antes de reportar, ejercitando el
    // cableado completo que el hot-path real consultaría.
    let cache = ExecutionGateCache::new(clock, ExecutionGateCacheConfig::default());
    cache.set(gate);

    match cache.get() {
        Some(cached_gate) => LicensingSystemVerifyOutput::from_gate(cached_gate),
        // Inalcanzable en la práctica: acabamos de guardar con el TTL por
        // defecto (5 minutos); solo fallaría si el reloj saltara ese tiempo
        // entre `set` y `get`, lo cual no ocurre en una invocación síncrona.
        None => LicensingSystemVerifyOutput::from_error(
            "el veredicto recién guardado ya no está vigente en la caché (inesperado)".to_string(),
        ),
    }
}

// ── Plan / Tier / Quota (STORY-029, vive en `shared` -- ver ADR-0137) ───────

/// Submódulo público del cimiento #3 (`docs/features/plan-tier-quota.md`,
/// ADR-0143, ADR-0144, STORY-029).
///
/// **Por qué un submódulo y no un `pub use` plano como el resto de este
/// archivo:** el puerto `plan_limits_out` de esta Feature produce un tipo
/// llamado `PlanLimits` (ADR-0137, catálogo, enmienda 2026-07-03). Pero
/// `licensing-system` (cimiento #2, STORY-028, YA SELLADO) declaró antes
/// su PROPIO struct `PlanLimits` como stub temporal
/// (`domain::licensing_system::PlanLimits`, sin `notional_limit`), y ese
/// nombre ya está aplanado en este mismo archivo unas líneas arriba.
/// Aplanar aquí el `PlanLimits` real de este cimiento colisionaría
/// (`error[E0255]: the name 'PlanLimits' is defined multiple times`). La
/// Orden de esta Story prohíbe expresamente tocar el código sellado de
/// `licensing-system` para unificarlos ("Re-cableado de licensing-system
/// (#2)... NO parte de esta Orden", STORY-029 §8) -- por eso, mientras ese
/// follow-up de integración no se ejecute, ambos tipos conviven bajo rutas
/// distintas: `public_interface::PlanLimits` (el stub de #2) y
/// `public_interface::plan_tier_quota::PlanLimits` (el real de #3).
pub mod plan_tier_quota {
    pub use crate::domain::plan_tier_quota::{
        canonical_features_json, compute_plan_audit_hash, decode_features_json, resolve_limits,
        validate_plan, PlanCandidate, PlanLimits, PlanSnapshot, PlanTier, PlanValidationError,
        PricingModel,
    };
    pub use crate::orchestrator::plan_tier_quota::{
        build_plan_limits_for_tier, seed_default_catalog, BuildPlanLimitsError,
        LocalStubPlanCatalogConfig, PlanLimitsCache, PlanLimitsCacheConfig,
    };
    pub use crate::persistence::plan_tier_quota::{
        NewPlan, Plan, PlanRepository, PlanRepositoryError,
    };
}

// ── Usage Metering (STORY-030, vive en `shared` -- ver ADR-0137) ───────────

pub use crate::domain::usage_metering::{
    accumulate, compute_notional, compute_usage_audit_hash, derive_billing_cycle_id,
    detect_quota_crossing, MeteredOperation, NotionalError, QuotaVerdict, UsageRecord,
    AMOUNT_SCALE,
};
pub use crate::orchestrator::usage_metering::{record_metered_operation, RecordMeteredOperationError};
pub use crate::persistence::usage_metering::{
    RecordOperationInput, UsageRecordRow, UsageRepository, UsageRepositoryError,
};

/// Una operación de entrada para la verificación de Usage Metering vía CLI
/// -- espejo mínimo de [`MeteredOperation`] pero con campos `String`/`i64`
/// deserializables directamente desde JSON (`MeteredOperation` toma
/// `&str`, no apto para deserializar con ownership propio).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MeteredOperationVerifyInput {
    /// Tamaño operado, `INTEGER` escalado ×10⁸.
    pub size: i64,
    /// Precio de ejecución, `INTEGER` escalado ×10⁸.
    pub price: i64,
    #[serde(default = "default_verify_instrument_id")]
    pub instrument_id: String,
}

fn default_verify_instrument_id() -> String {
    "BTCUSDT".to_string()
}

/// Input para la verificación de Usage Metering vía CLI
/// (`docs/features/usage-metering.md`, STORY-030). Se deserializa desde
/// el JSON que pasa el usuario con `--input '...'`.
///
/// Uso típico:
/// `cargo run -p app -- verify usage-metering --input
/// '{"tier":"FREE","operations":[{"size":250000000,"price":4000000000000}]}'`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UsageMeteringVerifyInput {
    /// `"FREE"` o `"PAID"` (mismo vocabulario que `plan_tier_quota::PlanTier`).
    #[serde(default = "default_plan_tier")]
    pub tier: String,
    /// Las operaciones a registrar, EN ORDEN, contra el mismo dueño y el
    /// mismo ciclo -- cada una se acumula sobre la anterior.
    pub operations: Vec<MeteredOperationVerifyInput>,
}

/// Output de la verificación de Usage Metering. Siempre serializa a JSON
/// válido (ADR-0142). Si `ok` es `true`, refleja EXACTAMENTE lo que expone
/// el puerto `usage_out` tras la ÚLTIMA operación registrada -- ningún
/// secreto (ADR-0093).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UsageMeteringVerifyOutput {
    pub ok: bool,
    pub tier: Option<String>,
    pub billing_cycle_id: Option<String>,
    pub cycle_accumulated: Option<i64>,
    pub quota_verdict: Option<String>,
    /// Cuántas operaciones se registraron con éxito antes de reportar (o
    /// antes de que una fallara).
    pub operations_recorded: usize,
    pub error: Option<String>,
}

impl UsageMeteringVerifyOutput {
    fn from_error(operations_recorded: usize, msg: String) -> Self {
        Self {
            ok: false,
            tier: None,
            billing_cycle_id: None,
            cycle_accumulated: None,
            quota_verdict: None,
            operations_recorded,
            error: Some(msg),
        }
    }

    fn from_record(tier: plan_tier_quota::PlanTier, record: UsageRecord, operations_recorded: usize) -> Self {
        Self {
            ok: true,
            tier: Some(tier.as_str().to_string()),
            billing_cycle_id: Some(record.billing_cycle_id),
            cycle_accumulated: Some(record.cycle_accumulated),
            quota_verdict: Some(record.quota_verdict.as_str().to_string()),
            operations_recorded,
            error: None,
        }
    }
}

/// Ejecuta la verificación de Usage Metering con adaptadores reales (BD
/// SQLite temporal + reloj de sistema real + catálogo REAL de
/// plan-tier-quota), recorriendo el camino completo del cimiento #4:
/// siembra el catálogo Free/Paid real (#3), registra CADA operación de
/// `input.operations` EN ORDEN (acumulando sobre la misma cuenta y el
/// mismo ciclo vigente) vía [`record_metered_operation`], y reporta el
/// `UsageRecord` resultante de la ÚLTIMA operación -- ejercitando Core ->
/// Shell -> puerto tal como lo recorrería un usuario real.
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify usage-metering --input
/// '{"tier":"FREE","operations":[{"size":250000000,"price":4000000000000}]}'`
pub async fn verify_usage_metering(input: UsageMeteringVerifyInput) -> UsageMeteringVerifyOutput {
    let tier = match plan_tier_quota::PlanTier::from_str_value(&input.tier) {
        Some(tier) => tier,
        None => {
            return UsageMeteringVerifyOutput::from_error(
                0,
                format!("tier desconocido: '{}' -- se esperaba FREE o PAID", input.tier),
            )
        }
    };

    // BD SQLite temporal exclusiva para esta verificación (mismo patrón
    // que verify_central_identity / verify_licensing_system / verify_plan_tier_quota).
    let temp_dir = std::env::temp_dir().join(format!(
        "drasus-verify-usage-metering-{}",
        uuid::Uuid::new_v4()
    ));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return UsageMeteringVerifyOutput::from_error(
            0,
            format!("no se pudo crear el directorio temporal de verificación: {e}"),
        );
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return UsageMeteringVerifyOutput::from_error(
                0,
                format!("no se pudo crear la BD temporal de verificación: {e}"),
            )
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return UsageMeteringVerifyOutput::from_error(
            0,
            format!("error al aplicar migraciones en la BD temporal: {e}"),
        );
    }

    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());

    // Paso 1 -- siembra el catálogo REAL de plan-tier-quota (#3) si esta
    // BD temporal todavía no lo tiene. Sin esto, record_metered_operation
    // fallaría con PlanNotFound -- el cableado real exige que el catálogo
    // exista, no hay fallback silencioso a un stub.
    let node_id = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string());
    if let Err(e) = plan_tier_quota::seed_default_catalog(
        &pool,
        clock.as_ref(),
        "drasus-system",
        &node_id,
        "DRASUS_LOCAL_VERIFY",
        &plan_tier_quota::LocalStubPlanCatalogConfig::default(),
    )
    .await
    {
        return UsageMeteringVerifyOutput::from_error(0, format!("fallo al sembrar el catálogo: {e}"));
    }

    // Paso 2 -- registra cada operación EN ORDEN, acumulando sobre la
    // misma cuenta y el mismo ciclo vigente (fuera del hot-path: esta
    // función SÍ hace lecturas/escrituras de BD local).
    let owner_id = "verify-usage-metering-owner";
    let mut last_record: Option<UsageRecord> = None;
    for (index, operation) in input.operations.iter().enumerate() {
        let result = record_metered_operation(
            &pool,
            clock.as_ref(),
            owner_id,
            "DRASUS_LOCAL_VERIFY",
            &node_id,
            tier,
            MeteredOperation {
                size: operation.size,
                price: operation.price,
                instrument_id: &operation.instrument_id,
            },
        )
        .await;

        match result {
            Ok(record) => last_record = Some(record),
            Err(e) => {
                return UsageMeteringVerifyOutput::from_error(
                    index,
                    format!("fallo al registrar la operación #{index}: {e}"),
                )
            }
        }
    }

    match last_record {
        Some(record) => UsageMeteringVerifyOutput::from_record(tier, record, input.operations.len()),
        // Sin operaciones en el input: no hay nada que reportar como
        // UsageRecord, pero tampoco es un error -- el catálogo se sembró
        // y el tier es válido, simplemente no se registró ninguna operación.
        None => UsageMeteringVerifyOutput::from_error(
            0,
            "no se proveyó ninguna operación en 'operations' -- nada que registrar".to_string(),
        ),
    }
}

/// Input para la verificación de Plan / Tier / Quota vía CLI
/// (`docs/features/plan-tier-quota.md`, STORY-029). Se deserializa desde
/// el JSON que pasa el usuario con `--input '...'`.
///
/// `tier` es el único campo que un uso típico necesita:
/// `cargo run -p app -- verify plan-tier-quota --input '{"tier":"FREE"}'`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanTierQuotaVerifyInput {
    /// `"FREE"` o `"PAID"` (`docs/features/plan-tier-quota.md` "Parámetros
    /// Configurables": `TIER_SET`).
    #[serde(default = "default_plan_tier")]
    pub tier: String,
}

fn default_plan_tier() -> String {
    "FREE".to_string()
}

/// Output de la verificación de Plan / Tier / Quota. Siempre serializa a
/// JSON válido (ADR-0142). Si `ok` es `true`, refleja EXACTAMENTE lo que
/// expone el puerto `plan_limits_out` -- ningún secreto (ADR-0093).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanTierQuotaVerifyOutput {
    pub ok: bool,
    pub tier: Option<String>,
    pub notional_limit: Option<i64>,
    pub max_activations: Option<i64>,
    pub features_enabled: Option<Vec<String>>,
    /// `true` si el valor devuelto salió de la caché con TTL en vez de una
    /// resolución fresca contra el catálogo (esta llamada de CLI siempre
    /// pasa por ambos pasos: resuelve y luego cachea).
    pub cached: bool,
    pub error: Option<String>,
}

impl PlanTierQuotaVerifyOutput {
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            tier: None,
            notional_limit: None,
            max_activations: None,
            features_enabled: None,
            cached: false,
            error: Some(msg),
        }
    }

    fn from_limits(tier: plan_tier_quota::PlanTier, limits: plan_tier_quota::PlanLimits) -> Self {
        Self {
            ok: true,
            tier: Some(tier.as_str().to_string()),
            notional_limit: Some(limits.notional_limit),
            max_activations: Some(limits.max_activations),
            features_enabled: Some(limits.features_enabled),
            cached: true,
            error: None,
        }
    }
}

/// Ejecuta la verificación de Plan / Tier / Quota con adaptadores reales
/// (BD SQLite temporal + reloj de sistema real + catálogo de desarrollo
/// stub), recorriendo el camino completo del cimiento #3: siembra el
/// catálogo Free/Paid (si aún no existe en esta BD temporal), resuelve
/// `PlanLimits` para el `tier` pedido, y lo pasa por su caché con TTL antes
/// de reportar -- ejercitando el camino completo Core -> Shell -> puerto
/// que un usuario real recorrería.
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify plan-tier-quota --input '{"tier":"FREE"}'`
pub async fn verify_plan_tier_quota(input: PlanTierQuotaVerifyInput) -> PlanTierQuotaVerifyOutput {
    let tier = match plan_tier_quota::PlanTier::from_str_value(&input.tier) {
        Some(tier) => tier,
        None => {
            return PlanTierQuotaVerifyOutput::from_error(format!(
                "tier desconocido: '{}' -- se esperaba FREE o PAID",
                input.tier
            ))
        }
    };

    // BD SQLite temporal exclusiva para esta verificación (mismo patrón que
    // verify_central_identity / verify_licensing_system).
    let temp_dir = std::env::temp_dir().join(format!(
        "drasus-verify-plan-tier-quota-{}",
        uuid::Uuid::new_v4()
    ));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return PlanTierQuotaVerifyOutput::from_error(format!(
            "no se pudo crear el directorio temporal de verificación: {e}"
        ));
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return PlanTierQuotaVerifyOutput::from_error(format!(
                "no se pudo crear la BD temporal de verificación: {e}"
            ))
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return PlanTierQuotaVerifyOutput::from_error(format!(
            "error al aplicar migraciones en la BD temporal: {e}"
        ));
    }

    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());

    // Paso 1 -- siembra el catálogo de desarrollo (Free + Paid) si esta BD
    // temporal todavía no lo tiene.
    let node_id = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string());
    if let Err(e) = plan_tier_quota::seed_default_catalog(
        &pool,
        clock.as_ref(),
        "drasus-system",
        &node_id,
        "DRASUS_LOCAL_VERIFY",
        &plan_tier_quota::LocalStubPlanCatalogConfig::default(),
    )
    .await
    {
        return PlanTierQuotaVerifyOutput::from_error(format!("fallo al sembrar el catálogo: {e}"));
    }

    // Paso 2 -- resuelve PlanLimits para el tier pedido (fuera del
    // hot-path: esta función SÍ hace lecturas de BD local).
    let limits = match plan_tier_quota::build_plan_limits_for_tier(&pool, clock.as_ref(), tier).await {
        Ok(limits) => limits,
        Err(e) => return PlanTierQuotaVerifyOutput::from_error(format!("fallo al resolver límites: {e}")),
    };

    // Paso 3 -- pasa por la caché con TTL antes de reportar, ejercitando el
    // cableado completo que el hot-path real consultaría.
    let cache = plan_tier_quota::PlanLimitsCache::new(clock, plan_tier_quota::PlanLimitsCacheConfig::default());
    cache.set(tier, limits);

    match cache.get(tier) {
        Some(cached_limits) => PlanTierQuotaVerifyOutput::from_limits(tier, cached_limits),
        // Inalcanzable en la práctica: acabamos de guardar con el TTL por
        // defecto (15 minutos); solo fallaría si el reloj saltara ese
        // tiempo entre `set` y `get`, lo cual no ocurre en una invocación
        // síncrona.
        None => PlanTierQuotaVerifyOutput::from_error(
            "los límites recién guardados ya no están vigentes en la caché (inesperado)".to_string(),
        ),
    }
}

// ── Consent Registry (STORY-031, vive en `shared` -- ver ADR-0137) ─────────

pub use crate::domain::consent_registry::{
    apply_consent_action, compute_consent_audit_hash, needs_reacceptance, parse_optout_map,
    resolve_coverage, ConsentAction, ConsentActionInput, ConsentState, ConsentVerdict,
    NotCoveredReason, OptoutMapError,
};
pub use crate::orchestrator::consent_registry::{record_consent_action, resolve_consent_verdict};
pub use crate::persistence::consent_registry::{
    ConsentRecordRow, ConsentRepository, ConsentRepositoryError, RecordConsentActionInput,
};

/// Una acción de consentimiento de entrada para la verificación vía CLI --
/// espejo de [`ConsentActionInput`] pero con `optout_map` (no
/// `optout_changes`) para que el JSON del usuario sea legible: cada acción
/// trae el mapa de cambios de opt-out que quiere aplicar sobre el estado
/// vigente (`docs/features/consent-registry.md`, STORY-031).
///
/// Uso típico de cada elemento de `actions`:
/// `{"action":"ACCEPT","tos_version":"v2","optout_map":{"aggregation":false}}`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsentActionVerifyInput {
    /// `"ACCEPT"`, `"REACCEPT"` u `"OPTOUT_CHANGE"` (mismo vocabulario que
    /// `ConsentAction::as_str`).
    pub action: String,
    /// Versión de ToS que se acepta -- solo relevante para
    /// `ACCEPT`/`REACCEPT`; se omite (o se manda `null`) en
    /// `OPTOUT_CHANGE`.
    #[serde(default)]
    pub tos_version: Option<String>,
    /// Cambios de opt-out a fusionar sobre el estado vigente -- solo las
    /// claves que cambian, el resto del mapa previo se conserva
    /// ([`apply_consent_action`]).
    #[serde(default)]
    pub optout_map: std::collections::BTreeMap<String, bool>,
}

/// La consulta de cobertura a resolver DESPUÉS de aplicar todas las
/// `actions` -- `(data_type, current_version)` (`current_version` viaja a
/// nivel de [`ConsentRegistryVerifyInput`], no aquí, porque es la MISMA
/// versión vigente contra la que se registraron las acciones).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsentQueryVerifyInput {
    pub data_type: String,
}

/// Input para la verificación de Consent Registry vía CLI
/// (`docs/features/consent-registry.md`, STORY-031). Se deserializa desde
/// el JSON que pasa el usuario con `--input '...'`.
///
/// Uso típico:
/// `cargo run -p app -- verify consent-registry --input
/// '{"current_version":"v2","actions":[{"action":"ACCEPT","tos_version":"v2","optout_map":{"aggregation":false}}],"query":{"data_type":"aggregation"}}'`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsentRegistryVerifyInput {
    /// La versión de ToS vigente contra la que se evalúan tanto las
    /// acciones registradas como la consulta final.
    pub current_version: String,
    /// Las acciones a registrar, EN ORDEN, contra el mismo dueño -- cada
    /// una se fusiona sobre el snapshot que dejó la anterior.
    pub actions: Vec<ConsentActionVerifyInput>,
    /// La consulta de cobertura a resolver tras registrar todas las
    /// acciones.
    pub query: ConsentQueryVerifyInput,
}

/// Output de la verificación de Consent Registry. Siempre serializa a
/// JSON válido (ADR-0142). Si `ok` es `true`, refleja EXACTAMENTE lo que
/// expone el puerto `consent_out` -- ningún secreto (ADR-0093).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsentRegistryVerifyOutput {
    pub ok: bool,
    /// `"COVERED"` o `"NOT_COVERED"`.
    pub verdict: Option<String>,
    /// Presente solo si `verdict` es `"NOT_COVERED"`:
    /// `"STALE_VERSION"` | `"OPTED_OUT"` | `"NO_CONSENT"`.
    pub reason: Option<String>,
    /// Cuántas acciones se registraron con éxito antes de resolver la
    /// consulta (o antes de que una fallara).
    pub actions_recorded: usize,
    pub error: Option<String>,
}

impl ConsentRegistryVerifyOutput {
    fn from_error(actions_recorded: usize, msg: String) -> Self {
        Self {
            ok: false,
            verdict: None,
            reason: None,
            actions_recorded,
            error: Some(msg),
        }
    }

    fn from_verdict(verdict: ConsentVerdict, actions_recorded: usize) -> Self {
        let (verdict_str, reason_str) = match &verdict {
            ConsentVerdict::Covered => ("COVERED".to_string(), None),
            ConsentVerdict::NotCovered(reason) => {
                let reason_str = match reason {
                    NotCoveredReason::StaleVersion => "STALE_VERSION",
                    NotCoveredReason::OptedOut => "OPTED_OUT",
                    NotCoveredReason::NoConsent => "NO_CONSENT",
                };
                ("NOT_COVERED".to_string(), Some(reason_str.to_string()))
            }
        };
        Self {
            ok: true,
            verdict: Some(verdict_str),
            reason: reason_str,
            actions_recorded,
            error: None,
        }
    }
}

/// Ejecuta la verificación de Consent Registry con adaptadores reales (BD
/// SQLite temporal + reloj de sistema real), recorriendo el camino
/// completo del cimiento #5: registra CADA acción de `input.actions` EN
/// ORDEN (fusionando sobre el mismo dueño vía [`record_consent_action`]),
/// y resuelve la consulta final vía [`resolve_consent_verdict`] --
/// ejercitando Core -> Shell -> puerto tal como lo recorrería un usuario
/// real.
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify consent-registry --input
/// '{"current_version":"v2","actions":[{"action":"ACCEPT","tos_version":"v2","optout_map":{"aggregation":false}}],"query":{"data_type":"aggregation"}}'`
pub async fn verify_consent_registry(input: ConsentRegistryVerifyInput) -> ConsentRegistryVerifyOutput {
    // BD SQLite temporal exclusiva para esta verificación (mismo patrón
    // que verify_usage_metering / verify_plan_tier_quota).
    let temp_dir = std::env::temp_dir().join(format!(
        "drasus-verify-consent-registry-{}",
        uuid::Uuid::new_v4()
    ));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return ConsentRegistryVerifyOutput::from_error(
            0,
            format!("no se pudo crear el directorio temporal de verificación: {e}"),
        );
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return ConsentRegistryVerifyOutput::from_error(
                0,
                format!("no se pudo crear la BD temporal de verificación: {e}"),
            )
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return ConsentRegistryVerifyOutput::from_error(
            0,
            format!("error al aplicar migraciones en la BD temporal: {e}"),
        );
    }

    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());
    let node_id = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string());
    let owner_id = "verify-consent-registry-owner";

    // Paso 1 -- registra cada acción EN ORDEN, fusionando sobre el mismo
    // dueño (fuera del hot-path: esta función SÍ hace lecturas/escrituras
    // de BD local).
    for (index, action) in input.actions.iter().enumerate() {
        let parsed_action = match ConsentAction::from_str_value(&action.action) {
            Some(a) => a,
            None => {
                return ConsentRegistryVerifyOutput::from_error(
                    index,
                    format!(
                        "acción #{index} desconocida: '{}' -- se esperaba ACCEPT, REACCEPT u OPTOUT_CHANGE",
                        action.action
                    ),
                )
            }
        };

        let result = record_consent_action(
            &pool,
            clock.as_ref(),
            RecordConsentActionInput {
                owner_id: owner_id.to_string(),
                institutional_tag: "DRASUS_LOCAL_VERIFY".to_string(),
                node_id: node_id.clone(),
                compliance_status_id: None,
                action: parsed_action,
                tos_version: action.tos_version.clone(),
                optout_changes: action.optout_map.clone(),
            },
        )
        .await;

        if let Err(e) = result {
            return ConsentRegistryVerifyOutput::from_error(
                index,
                format!("fallo al registrar la acción #{index}: {e}"),
            );
        }
    }

    // Paso 2 -- resuelve la consulta final tras aplicar todas las acciones.
    let verdict = match resolve_consent_verdict(
        &pool,
        clock.as_ref(),
        owner_id,
        &input.query.data_type,
        &input.current_version,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            return ConsentRegistryVerifyOutput::from_error(
                input.actions.len(),
                format!("fallo al resolver el veredicto de consentimiento: {e}"),
            )
        }
    };

    ConsentRegistryVerifyOutput::from_verdict(verdict, input.actions.len())
}

// ── Enriched Domain Events (STORY-033, vive en `shared` -- ver ADR-0137) ────

/// Submódulo público del cimiento #6 (`docs/features/enriched-domain-events.md`,
/// ADR-0144, ADR-0145, STORY-033) -- la raíz del substrato de monetización.
///
/// Expone los dos puertos de la Feature (ADR-0137): `event_out`
/// (`EnrichedDomainEvent` persistido, Output 1..N) y `gate_in`
/// (`ExecutionGate` consumido, Input 1). Vive bajo su propio submódulo en
/// vez de aplanarse a este nivel superior, por simetría con
/// `plan_tier_quota` y para agrupar el catálogo de tipos de evento (que es
/// grande) bajo un espacio de nombres claro.
///
/// **Guardarraíl ADR-0093:** ningún tipo re-exportado aquí modela un
/// secreto -- ni el evento ni su payload pueden portar credenciales de
/// bróker, IPs live o claves de firma (verificado por el test
/// `no_payload_variant_leaks_secret_looking_fields` del Core).
pub mod enriched_domain_events {
    // event_out: el catálogo de eventos del Core + su serialización canónica
    // + el hash encadenado + la decisión de replicación.
    pub use crate::domain::enriched_domain_events::{
        compute_event_audit_hash, decide_replication, AccountSnapshotPayload,
        BacktestCompletedPayload, CapitalFlowPayload, CapitalFlowSign, CorrelationChangePayload,
        DrawdownDetectedPayload, EnrichedDomainEvent, LiquidityStressPayload, OrderExecutedPayload,
        OrderSide, RegimeDetectedPayload,
    };
    // La composición completa (recibe evento + gate real, deriva replicate,
    // persiste append-only atómico).
    pub use crate::orchestrator::enriched_domain_events::{
        record_domain_event, EventEmissionIdentity,
    };
    pub use crate::persistence::enriched_domain_events::{
        DomainEventRepository, DomainEventRepositoryError, DomainEventRow, RecordDomainEventInput,
    };
    // gate_in: el tipo de puerto de entrada -- se consume el ExecutionGate
    // REAL de licensing-system (#2), no un stub.
    pub use crate::domain::licensing_system::ExecutionGate;
}

/// El evento de entrada para la verificación de Enriched Domain Events vía
/// CLI -- un enum etiquetado por `type` que refleja el catálogo del Core
/// (`docs/features/enriched-domain-events.md`, STORY-033). Los enums del
/// Core (`OrderSide`, `CapitalFlowSign`) llegan aquí como `String` para que
/// el JSON del usuario sea legible; se convierten en
/// [`DomainEventVerifyEvent::into_domain_event`].
///
/// Los montos son `i64` escalados ×10⁸ (ADR-0141) -- exactamente los mismos
/// enteros que porta el Core, sin `f64` en ningún punto del camino.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum DomainEventVerifyEvent {
    OrderExecuted {
        instrument_id: String,
        /// `"BUY"` o `"SELL"`.
        side: String,
        quantity: i64,
        price: i64,
        #[serde(default)]
        slippage: i64,
        #[serde(default)]
        fill_time_ns: i64,
        broker: String,
        notional: i64,
        account_id: String,
        #[serde(default)]
        realized_pnl: i64,
        #[serde(default)]
        mae: i64,
        #[serde(default)]
        mfe: i64,
        #[serde(default)]
        duration_ns: i64,
    },
    CapitalFlow {
        account_id: String,
        /// `"DEPOSIT"`, `"WITHDRAWAL"` o `"TRANSFER"`.
        sign: String,
        amount: i64,
        currency: String,
        #[serde(default)]
        timestamp_ns: i64,
    },
    AccountSnapshot {
        account_id: String,
        equity: i64,
        balance: i64,
        margin_available: i64,
        margin_required: i64,
        #[serde(default)]
        timestamp_ns: i64,
    },
    BacktestCompleted {
        sharpe: i64,
        drawdown: i64,
        pbo: i64,
        regime: String,
    },
    RegimeDetected {
        instrument_id: String,
        regime_label: String,
        #[serde(default)]
        timestamp_ns: i64,
    },
    DrawdownDetected {
        account_id: String,
        drawdown_pct: i64,
        #[serde(default)]
        timestamp_ns: i64,
    },
    LiquidityStress {
        instrument_id: String,
        severity: String,
        #[serde(default)]
        timestamp_ns: i64,
    },
    CorrelationChange {
        instrument_a: String,
        instrument_b: String,
        correlation: i64,
        #[serde(default)]
        timestamp_ns: i64,
    },
}

impl DomainEventVerifyEvent {
    /// Convierte la entrada de CLI en el `EnrichedDomainEvent` del Core,
    /// validando los strings de enum (`side`, `sign`). Devuelve `Err(String)`
    /// con un mensaje legible si un enum no es reconocido -- nunca hace
    /// panic sobre input del usuario.
    fn into_domain_event(self) -> Result<enriched_domain_events::EnrichedDomainEvent, String> {
        use enriched_domain_events::{
            AccountSnapshotPayload, BacktestCompletedPayload, CapitalFlowPayload, CapitalFlowSign,
            CorrelationChangePayload, DrawdownDetectedPayload, EnrichedDomainEvent,
            LiquidityStressPayload, OrderExecutedPayload, OrderSide, RegimeDetectedPayload,
        };

        match self {
            DomainEventVerifyEvent::OrderExecuted {
                instrument_id, side, quantity, price, slippage, fill_time_ns, broker, notional,
                account_id, realized_pnl, mae, mfe, duration_ns,
            } => {
                let side = OrderSide::from_str_value(&side)
                    .ok_or_else(|| format!("side desconocido: '{side}' -- se esperaba BUY o SELL"))?;
                Ok(EnrichedDomainEvent::OrderExecuted(OrderExecutedPayload {
                    instrument_id, side, quantity, price, slippage, fill_time_ns, broker, notional,
                    account_id, realized_pnl, mae, mfe, duration_ns,
                }))
            }
            DomainEventVerifyEvent::CapitalFlow { account_id, sign, amount, currency, timestamp_ns } => {
                let sign = CapitalFlowSign::from_str_value(&sign).ok_or_else(|| {
                    format!("sign desconocido: '{sign}' -- se esperaba DEPOSIT, WITHDRAWAL o TRANSFER")
                })?;
                Ok(EnrichedDomainEvent::CapitalFlow(CapitalFlowPayload {
                    account_id, sign, amount, currency, timestamp_ns,
                }))
            }
            DomainEventVerifyEvent::AccountSnapshot {
                account_id, equity, balance, margin_available, margin_required, timestamp_ns,
            } => Ok(EnrichedDomainEvent::AccountSnapshot(AccountSnapshotPayload {
                account_id, equity, balance, margin_available, margin_required, timestamp_ns,
            })),
            DomainEventVerifyEvent::BacktestCompleted { sharpe, drawdown, pbo, regime } => {
                Ok(EnrichedDomainEvent::BacktestCompleted(BacktestCompletedPayload {
                    sharpe, drawdown, pbo, regime,
                }))
            }
            DomainEventVerifyEvent::RegimeDetected { instrument_id, regime_label, timestamp_ns } => {
                Ok(EnrichedDomainEvent::RegimeDetected(RegimeDetectedPayload {
                    instrument_id, regime_label, timestamp_ns,
                }))
            }
            DomainEventVerifyEvent::DrawdownDetected { account_id, drawdown_pct, timestamp_ns } => {
                Ok(EnrichedDomainEvent::DrawdownDetected(DrawdownDetectedPayload {
                    account_id, drawdown_pct, timestamp_ns,
                }))
            }
            DomainEventVerifyEvent::LiquidityStress { instrument_id, severity, timestamp_ns } => {
                Ok(EnrichedDomainEvent::LiquidityStress(LiquidityStressPayload {
                    instrument_id, severity, timestamp_ns,
                }))
            }
            DomainEventVerifyEvent::CorrelationChange { instrument_a, instrument_b, correlation, timestamp_ns } => {
                Ok(EnrichedDomainEvent::CorrelationChange(CorrelationChangePayload {
                    instrument_a, instrument_b, correlation, timestamp_ns,
                }))
            }
        }
    }
}

/// Input para la verificación de Enriched Domain Events vía CLI
/// (`docs/features/enriched-domain-events.md`, STORY-033). Se deserializa
/// desde el JSON que pasa el usuario con `--input '...'`.
///
/// Uso típico:
/// `cargo run -p app -- verify enriched-domain-events --input
/// '{"tier":"FREE","event":{"type":"CapitalFlow","account_id":"acc-1","sign":"DEPOSIT","amount":100000000000,"currency":"USD"}}'`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnrichedDomainEventsVerifyInput {
    /// El tier que gobierna la supresión de telemetría (ADR-0143). `"FREE"`
    /// (gratuito -> Explorer, no suprime -> replica) o `"PAID"` (pago al
    /// corriente -> Sovereign, suprime -> no replica). También se aceptan
    /// los nombres de licencia crudos `"EXPLORER"`/`"SOVEREIGN"`.
    #[serde(default = "default_domain_event_tier")]
    pub tier: String,
    /// El evento a construir, persistir y observar.
    pub event: DomainEventVerifyEvent,
}

fn default_domain_event_tier() -> String {
    "FREE".to_string()
}

/// Output de la verificación de Enriched Domain Events. Siempre serializa a
/// JSON válido (ADR-0142). Si `ok` es `true`, refleja EXACTAMENTE lo que
/// expone el puerto `event_out` tras persistir el evento -- ningún secreto
/// (ADR-0093).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnrichedDomainEventsVerifyOutput {
    pub ok: bool,
    /// El tier resuelto (`"EXPLORER"` o `"SOVEREIGN"`).
    pub tier: Option<String>,
    /// `true` si el gate real suprime telemetría de trabajo -- espejo del
    /// campo del `ExecutionGate` que gobernó la decisión.
    pub suppress_work_telemetry: Option<bool>,
    /// La decisión derivada: `true` = el evento se replica al proveedor;
    /// `false` = solo local. Es el inverso de `suppress_work_telemetry`.
    pub replicate: Option<bool>,
    pub event_type: Option<String>,
    /// El payload JSON canónico persistido (string, tal cual quedó en la BD).
    pub payload: Option<String>,
    pub event_sequence_id: Option<i64>,
    /// `true` si la fila persistida es la génesis (`audit_chain_hash` NULL).
    pub is_genesis: Option<bool>,
    pub audit_hash: Option<String>,
    pub error: Option<String>,
}

impl EnrichedDomainEventsVerifyOutput {
    fn from_error(msg: String) -> Self {
        Self {
            ok: false,
            tier: None,
            suppress_work_telemetry: None,
            replicate: None,
            event_type: None,
            payload: None,
            event_sequence_id: None,
            is_genesis: None,
            audit_hash: None,
            error: Some(msg),
        }
    }
}

/// Traduce el `tier` del input (`FREE`/`PAID`, o los crudos
/// `EXPLORER`/`SOVEREIGN`) al [`LicenseTier`] de `licensing-system`.
/// `FREE` -> `Explorer` (gratuito, nunca suprime), `PAID` -> `Sovereign`
/// (pago al corriente, suprime). Devuelve `None` si no reconoce el valor.
fn resolve_domain_event_tier(value: &str) -> Option<LicenseTier> {
    match value.to_uppercase().as_str() {
        "FREE" | "EXPLORER" => Some(LicenseTier::Explorer),
        "PAID" | "SOVEREIGN" => Some(LicenseTier::Sovereign),
        _ => None,
    }
}

/// Ejecuta la verificación de Enriched Domain Events con adaptadores reales
/// (BD SQLite temporal + reloj de sistema real + el `ExecutionGate` REAL de
/// `licensing-system` #2), recorriendo el camino completo del cimiento #6:
/// construye una licencia de desarrollo firmada para el tier pedido, deriva
/// el `ExecutionGate` real (no un stub), compone el evento del catálogo,
/// deriva `replicate` y lo persiste append-only atómico vía
/// [`enriched_domain_events::record_domain_event`], y reporta la fila
/// persistida -- ejercitando Core -> Shell -> puerto tal como lo recorrería
/// el motor real.
///
/// Uso típico desde el CLI:
/// `cargo run -p app -- verify enriched-domain-events --input
/// '{"tier":"FREE","event":{"type":"CapitalFlow","account_id":"acc-1","sign":"DEPOSIT","amount":100000000000,"currency":"USD"}}'`
pub async fn verify_enriched_domain_events(
    input: EnrichedDomainEventsVerifyInput,
) -> EnrichedDomainEventsVerifyOutput {
    let tier = match resolve_domain_event_tier(&input.tier) {
        Some(tier) => tier,
        None => {
            return EnrichedDomainEventsVerifyOutput::from_error(format!(
                "tier desconocido: '{}' -- se esperaba FREE o PAID (o EXPLORER/SOVEREIGN)",
                input.tier
            ))
        }
    };

    // Construye el evento del Core ANTES de tocar la BD -- así un input mal
    // formado (side/sign inválido) falla rápido y barato.
    let event = match input.event.into_domain_event() {
        Ok(event) => event,
        Err(msg) => return EnrichedDomainEventsVerifyOutput::from_error(msg),
    };

    // BD SQLite temporal exclusiva para esta verificación (mismo patrón que
    // verify_licensing_system / verify_consent_registry).
    let temp_dir = std::env::temp_dir().join(format!(
        "drasus-verify-enriched-domain-events-{}",
        uuid::Uuid::new_v4()
    ));
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return EnrichedDomainEventsVerifyOutput::from_error(format!(
            "no se pudo crear el directorio temporal de verificación: {e}"
        ));
    }

    let db_path = temp_dir.join("verify.db");
    let db_url = format!("sqlite://{}", db_path.display());
    let pool = match crate::persistence::pool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            return EnrichedDomainEventsVerifyOutput::from_error(format!(
                "no se pudo crear la BD temporal de verificación: {e}"
            ))
        }
    };
    if let Err(e) = crate::persistence::pool::migrate(&pool).await {
        return EnrichedDomainEventsVerifyOutput::from_error(format!(
            "error al aplicar migraciones en la BD temporal: {e}"
        ));
    }

    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(crate::orchestrator::SystemClock::default());

    // Paso 1 -- gate_in: construye el ExecutionGate REAL de #2 (no un stub),
    // recorriendo el mismo camino que verify_licensing_system: vincula una
    // identidad local, emite+activa una licencia de desarrollo firmada para
    // el tier, obtiene PlanLimits del stub y deriva el gate.
    let machine_identifiers = vec![hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string())];
    let identity_verifier =
        crate::orchestrator::central_identity::LocalStubCentralIdentityVerifier::new(&pool, clock.as_ref());
    let identity = match identity_verifier
        .verify_identity(crate::orchestrator::central_identity::IdentityVerificationRequest {
            email: "verify-enriched-domain-events@drasus.local".to_string(),
            oauth_provider: None,
            machine_identifiers,
            institutional_tag: "DRASUS_LOCAL_VERIFY".to_string(),
            access_token_id: None,
        })
        .await
    {
        Ok(identity) => identity,
        Err(e) => return EnrichedDomainEventsVerifyOutput::from_error(format!("fallo al vincular identidad: {e}")),
    };

    let issuer = LocalStubLicenseIssuer::new();
    let now_ns = clock.timestamp_ns();
    let signed = issuer.issue_license(IssueLicenseRequest {
        owner_id: identity.owner_id.clone(),
        node_id: identity.node_id.clone(),
        tier,
        issued_at_ns: now_ns,
        heartbeat_expires_at_ns: now_ns + DEFAULT_HEARTBEAT_INTERVAL_NS,
    });

    let license_repo = LicenseRepository::new(&pool, clock.as_ref());
    let license = match license_repo
        .activate(NewLicenseActivation {
            owner_id: identity.owner_id.clone(),
            institutional_tag: identity.institutional_tag.clone(),
            access_token_id: None,
            node_id: identity.node_id.clone(),
            license_id: signed.license_id.clone(),
            process_id: Some(format!("drasus-pid-{}", std::process::id())),
            signature_hash: signed.signature_hex.clone(),
            tier,
            issued_at_ns: signed.issued_at_ns,
            heartbeat_expires_at_ns: signed.heartbeat_expires_at_ns,
            compliance_status_id: "ACTIVE".to_string(),
        })
        .await
    {
        Ok(license) => license,
        Err(e) => return EnrichedDomainEventsVerifyOutput::from_error(format!("fallo al activar licencia: {e}")),
    };

    let plan_limits_provider = LocalStubPlanLimitsProvider::default();
    let plan_limits = plan_limits_provider.plan_limits_for(&identity.owner_id, tier).await;

    let heartbeat_config = HeartbeatConfig::default();
    let gate = match build_execution_gate(
        &pool,
        clock.as_ref(),
        &identity.node_id,
        &license,
        &signed.signature_hex,
        &signed.public_key_hex,
        &heartbeat_config,
        &plan_limits,
    )
    .await
    {
        Ok(gate) => gate,
        Err(e) => return EnrichedDomainEventsVerifyOutput::from_error(format!("fallo al construir el gate: {e}")),
    };

    let suppress = gate.suppress_work_telemetry;

    // Paso 2 -- event_out: compone el evento + el gate real, deriva replicate
    // y persiste append-only atómico.
    let identity_for_event = enriched_domain_events::EventEmissionIdentity {
        owner_id: identity.owner_id.clone(),
        institutional_tag: identity.institutional_tag.clone(),
        node_id: identity.node_id.clone(),
        process_id: format!("drasus-pid-{}", std::process::id()),
        session_id: None,
    };

    let row = match enriched_domain_events::record_domain_event(
        &pool,
        clock.as_ref(),
        identity_for_event,
        &gate,
        event,
    )
    .await
    {
        Ok(row) => row,
        Err(e) => return EnrichedDomainEventsVerifyOutput::from_error(format!("fallo al persistir el evento: {e}")),
    };

    EnrichedDomainEventsVerifyOutput {
        ok: true,
        tier: Some(tier.as_str().to_string()),
        suppress_work_telemetry: Some(suppress),
        replicate: Some(row.replicate),
        event_type: Some(row.event_type),
        payload: Some(row.payload),
        event_sequence_id: Some(row.event_sequence_id),
        is_genesis: Some(row.audit_chain_hash.is_none()),
        audit_hash: Some(row.audit_hash),
        error: None,
    }
}
