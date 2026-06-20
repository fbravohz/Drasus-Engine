//! [SHELL] Servidor MCP sobre stdio — Gateway Agéntico (ADR-0123).
//!
//! Expone como herramientas MCP las operaciones de `shared` ya implementadas:
//! - `drasus_clock_now`         — timestamp actual del reloj del sistema.
//! - `drasus_jobs_list`         — jobs activos (QUEUED + RUNNING).
//! - `drasus_jobs_submit`       — encola un nuevo job.
//! - `drasus_telemetry_latest`  — últimas N muestras de telemetría por métrica.
//!
//! Para cada llamada el servidor:
//!   1. Identifica el pipeline de la herramienta.
//!   2. Lee `production_override_active` de la BD.
//!   3. Llama a `evaluate_permission`.
//!   4. Si `Granted`, ejecuta la herramienta vía la `public_interface`.
//!   5. Registra la decisión en `permission_decisions` (append-only).
//!
//! Transporte: stdio (stdin/stdout), modo local EPIC-0 (ADR-0123).

use rmcp::{
    ErrorData as McpError, RoleServer,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
    schemars,
    service::RequestContext,
    tool, tool_router,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::sync::Arc as StdArc;

use crate::domain::mcp_gateway::{Pipeline, PermissionOutcome, PermissionRequest, evaluate_permission};
use crate::persistence::mcp_gateway::McpGatewayRepository;

// ────────────────────────────────────────────────────────────────────────────
// Tipos de parámetros para las herramientas
// ────────────────────────────────────────────────────────────────────────────

/// Parámetros de `drasus_jobs_submit`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SubmitJobParams {
    /// Tipo de job a encolar, ej. "backtest" o "optimize".
    pub job_type: String,
    /// Payload JSON del job (argumentos específicos del tipo), como string JSON.
    pub payload_json: String,
    /// Identificador del usuario que encola el job.
    pub user_id: String,
}

/// Parámetros de `drasus_telemetry_latest`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TelemetryLatestParams {
    /// Nombre de la métrica a consultar, ej. "ingest.hot_path_latency".
    pub metric_name: String,
    /// Número de muestras a devolver (por defecto 10).
    #[schemars(description = "Número máximo de muestras (por defecto 10)")]
    pub limit: Option<i64>,
}

// ────────────────────────────────────────────────────────────────────────────
// Servidor MCP
// ────────────────────────────────────────────────────────────────────────────

/// Servidor MCP de Drasus Engine (Cabina Dual — ADR-0123).
///
/// Implementa `ServerHandler` vía el macro `#[tool_router(server_handler)]`.
/// El pool de SQLite se comparte con el resto del motor (clonado desde el
/// pool del proceso, que es barato porque `SqlitePool` es un `Arc` interno).
#[derive(Clone)]
pub struct DrasusGateway {
    /// Pool de conexiones SQLite compartido con el motor.
    pool: SqlitePool,
    /// Identificador de sesión del agente MCP conectado (UUID v4 generado al arrancar).
    agent_session_id: StdArc<str>,
    /// Hostname del nodo donde corre el Gateway (ADR-0020 V2, Grupo IV).
    node_id: StdArc<str>,
}

impl DrasusGateway {
    /// Crea una instancia del servidor MCP.
    ///
    /// `agent_session_id` identifica la sesión para el log de auditoría.
    /// `node_id` es el hostname del proceso.
    pub fn new(pool: SqlitePool, agent_session_id: String, node_id: String) -> Self {
        Self {
            pool,
            agent_session_id: StdArc::from(agent_session_id.as_str()),
            node_id: StdArc::from(node_id.as_str()),
        }
    }

    /// Evalúa el permiso, registra la decisión y devuelve el outcome.
    ///
    /// Helper interno: hace I/O (lee el interruptor, escribe la decisión),
    /// por eso vive en el Shell y no en el Core (FCIS, ADR-0002).
    async fn check_and_record(
        &self,
        pipeline: Pipeline,
        scope: &str,
    ) -> Result<PermissionOutcome, McpError> {
        // Lee el estado actual del interruptor desde la BD.
        let production_override_active =
            McpGatewayRepository::get_production_override(&self.pool)
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let req = PermissionRequest {
            pipeline,
            institutional_tag: None, // todas las herramientas de EPIC-0 son abiertos
            production_override_active,
            agent_session_id: self.agent_session_id.to_string(),
            requested_scope: scope.to_string(),
        };

        let outcome = evaluate_permission(&req);

        // Obtiene el extremo de la cadena para encadenar la próxima decisión.
        let (prev_hash, next_seq) =
            McpGatewayRepository::chain_tip(&self.pool)
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?
                .map(|(h, s)| (h, s + 1))
                .unwrap_or_else(|| ("genesis".to_string(), 1));

        // Timestamp de la evaluación (nanosegundos desde epoch).
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as i64)
            .unwrap_or(0);

        let decision = crate::domain::mcp_gateway::PermissionDecision::build(
            &req,
            &outcome,
            now_ns,
            prev_hash,
            next_seq,
            self.node_id.to_string(),
            std::process::id() as i64,
        );

        McpGatewayRepository::append(&self.pool, &decision)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(outcome)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Herramientas MCP (macro-driven, ADR-0123)
// ────────────────────────────────────────────────────────────────────────────

#[tool_router(server_handler)]
impl DrasusGateway {
    /// Devuelve el timestamp actual del motor Drasus (nanosegundos desde el Unix epoch).
    ///
    /// Pipeline: Feedback — siempre Granted (lista abierta, ADR-0123).
    #[tool(description = "Devuelve el timestamp actual del motor Drasus en nanosegundos desde el Unix epoch.")]
    async fn drasus_clock_now(
        &self,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let outcome = self
            .check_and_record(Pipeline::Feedback, "feedback.clock.now")
            .await?;

        match outcome {
            PermissionOutcome::Granted => {
                let ts_ns = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0);
                Ok(CallToolResult::success(vec![Content::text(
                    ts_ns.to_string(),
                )]))
            }
            PermissionOutcome::Denied { reason } => Ok(CallToolResult::error(vec![
                Content::text(format!("Permiso denegado: {reason}")),
            ])),
        }
    }

    /// Lista los jobs activos (QUEUED y RUNNING) con su estado y progreso.
    ///
    /// Pipeline: Feedback — siempre Granted (lista abierta, ADR-0123).
    #[tool(description = "Lista los jobs activos del motor (QUEUED y RUNNING) con su tipo, estado y progreso (0-100).")]
    async fn drasus_jobs_list(
        &self,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let outcome = self
            .check_and_record(Pipeline::Feedback, "feedback.jobs.list")
            .await?;

        match outcome {
            PermissionOutcome::Granted => {
                use crate::persistence::job::JobRepository;
                use crate::orchestrator::SystemClock;
                use crate::domain::job::JobState;
                use std::sync::Arc;

                // Usamos SystemClock aquí: solo necesitamos el reloj para
                // construir el repositorio; list no hace writes que necesiten timestamp.
                let clock = Arc::new(SystemClock::default());
                let repo = JobRepository::new(&self.pool, clock.as_ref());

                let mut jobs_json = Vec::new();

                // Cargamos QUEUED y RUNNING.
                for state in [JobState::Queued, JobState::Running] {
                    let jobs = repo
                        .jobs_in_state(state)
                        .await
                        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

                    for job in jobs {
                        jobs_json.push(serde_json::json!({
                            "id": job.id,
                            "job_type": job.job_type,
                            "state": format!("{:?}", job.state),
                            "progress": job.progress,
                            "created_at_ns": job.created_at_ns,
                        }));
                    }
                }

                let json = serde_json::to_string(&jobs_json)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;

                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            PermissionOutcome::Denied { reason } => Ok(CallToolResult::error(vec![
                Content::text(format!("Permiso denegado: {reason}")),
            ])),
        }
    }

    /// Encola un nuevo job en el motor.
    ///
    /// Pipeline: Ingest — siempre Granted (lista abierta, ADR-0123).
    #[tool(description = "Encola un nuevo job en el motor. Parámetros: job_type, payload_json (JSON del job), user_id.")]
    async fn drasus_jobs_submit(
        &self,
        Parameters(SubmitJobParams {
            job_type,
            payload_json,
            user_id,
        }): Parameters<SubmitJobParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let outcome = self
            .check_and_record(Pipeline::Ingest, "ingest.jobs.submit")
            .await?;

        match outcome {
            PermissionOutcome::Granted => {
                use crate::persistence::job::{JobRepository, NewJob};
                use crate::orchestrator::SystemClock;
                use std::sync::Arc;

                let clock = Arc::new(SystemClock::default());
                let repo = JobRepository::new(&self.pool, clock.as_ref());

                let new_job = NewJob {
                    user_id,
                    job_type,
                    parameters: payload_json,
                    owner_id: None,
                    access_token_id: None,
                    session_id: Some(self.agent_session_id.to_string()),
                    node_id: Some(self.node_id.to_string()),
                    logic_hash: None,
                };

                let job = repo
                    .submit(new_job)
                    .await
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;

                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Job encolado: {} (tipo: {})",
                    job.id, job.job_type
                ))]))
            }
            PermissionOutcome::Denied { reason } => Ok(CallToolResult::error(vec![
                Content::text(format!("Permiso denegado: {reason}")),
            ])),
        }
    }

    /// Devuelve las últimas N muestras de telemetría para una métrica dada.
    ///
    /// Pipeline: Feedback — siempre Granted (lista abierta, ADR-0123).
    #[tool(description = "Devuelve las últimas muestras de telemetría para una métrica. Parámetros: metric_name, limit (opcional, por defecto 10).")]
    async fn drasus_telemetry_latest(
        &self,
        Parameters(TelemetryLatestParams { metric_name, limit }): Parameters<TelemetryLatestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let outcome = self
            .check_and_record(Pipeline::Feedback, "feedback.telemetry.latest")
            .await?;

        match outcome {
            PermissionOutcome::Granted => {
                use crate::persistence::telemetry::TelemetryRepository;

                let repo = TelemetryRepository::new(&self.pool);
                let n = limit.unwrap_or(10);

                // Consultamos los últimos `n` datos: usamos un rango amplio
                // (0..i64::MAX) y luego tomamos los N más recientes.
                // En EPIC-0 no tenemos un `latest_n` dedicado, reutilizamos
                // `query_by_metric` con el rango completo.
                let mut samples = repo
                    .query_by_metric(&metric_name, 0, i64::MAX)
                    .await
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;

                // Toma los últimos `n` (los más recientes — la query devuelve ASC).
                let total = samples.len();
                if total > n as usize {
                    samples = samples.into_iter().skip(total - n as usize).collect();
                }

                let json_items: Vec<serde_json::Value> = samples
                    .into_iter()
                    .map(|s| {
                        serde_json::json!({
                            "id": s.id,
                            "metric_name": s.content.metric_name,
                            "execution_latency_ms": s.content.execution_latency_ms,
                            "created_at_ns": s.created_at_ns,
                            "details_json": s.content.details_json,
                        })
                    })
                    .collect();

                let json = serde_json::to_string(&json_items)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;

                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            PermissionOutcome::Denied { reason } => Ok(CallToolResult::error(vec![
                Content::text(format!("Permiso denegado: {reason}")),
            ])),
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Función pública — lanzada por `drasus start`
// ────────────────────────────────────────────────────────────────────────────

/// Arranca el servidor MCP sobre stdio y bloquea hasta que el cliente cierre.
///
/// Se lanza en un `tokio::spawn` desde `drasus start`.
/// Cuando el proceso padre termina, el handle de stdin/stdout se cierra y
/// el loop del servidor finaliza limpiamente, sin coordinación explícita.
pub async fn run_mcp_server(pool: SqlitePool) -> anyhow::Result<()> {
    use rmcp::{ServiceExt, transport::stdio};

    // UUID de la sesión de este servidor MCP (identifica al agente en el audit log).
    let agent_session_id = uuid::Uuid::new_v4().to_string();

    // Hostname del nodo (Grupo IV de ADR-0020 V2).
    let node_id = hostname::get()
        .map(|h| h.to_string_lossy().into_owned())
        .unwrap_or_else(|_| format!("node-pid-{}", std::process::id()));

    let service = DrasusGateway::new(pool, agent_session_id, node_id);

    // `stdio()` construye el transporte sobre stdin/stdout de Tokio.
    // `.serve()` completa el handshake MCP (initialize) y devuelve el
    // `RunningService` que mantiene el loop de mensajes activo.
    let server = service.serve(stdio()).await?;

    // Bloquea hasta que el cliente cierre la conexión.
    server.waiting().await?;

    Ok(())
}
