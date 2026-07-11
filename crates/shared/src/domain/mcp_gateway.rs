//! [CORE] Lógica pura del Gateway MCP — tipos y evaluador de permisos.
//!
//! Sin I/O, sin reloj de sistema, sin azar sin semilla (ADR-0002/0004).
//! Implementa la matriz de permisos de ADR-0123 (Cabina Dual).
//!
//! Perfil de persistencia: Grupo I + Grupo II (Soberanía) + Grupo IV (Hardware)
//! + 4 campos de dominio propio (ADR-0020, Perfil D — Ops/Auditoría).

use uuid::Uuid;

// ────────────────────────────────────────────────────────────────────────────
// Enums del dominio
// ────────────────────────────────────────────────────────────────────────────

/// Los 8 pipelines del sistema (ADR-0123).
///
/// Cada pipeline tiene un nivel de riesgo distinto que determina si el agente
/// puede invocarlo por defecto, condicionalmente, o nunca.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pipeline {
    /// Ingesta de datos — abierto por defecto.
    Ingest,
    /// Generación de estrategias — abierto por defecto.
    Generate,
    /// Validación de estrategias — abierto por defecto.
    Validate,
    /// Incubación/simulación — abierto por defecto.
    Incubate,
    /// Gestión de portafolios — condicionado por `institutional_tag`.
    Manage,
    /// Ejecución real de órdenes — bloqueado por defecto.
    Execute,
    /// Feedback de resultados — abierto por defecto.
    Feedback,
    /// Retiro de capital — bloqueado por defecto.
    Withdraw,
}

/// Etiqueta del objeto afectado por una llamada a `Manage`.
///
/// Solo aplica cuando el pipeline es `Manage`; en los demás casos se omite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstitutionalTag {
    /// Portafolio de capital real — requiere interruptor activo.
    Live,
    /// Portafolio de paper trading / demo — libre sin interruptor.
    Demo,
}

/// Resultado de la evaluación de permisos (ADR-0123).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionOutcome {
    /// La llamada se concede; el Shell puede enrutarla.
    Granted,
    /// La llamada se deniega; el Shell devuelve el motivo al agente.
    Denied {
        /// Motivo legible de la denegación.
        reason: String,
    },
}

// ────────────────────────────────────────────────────────────────────────────
// Structs del dominio
// ────────────────────────────────────────────────────────────────────────────

/// Solicitud de permiso completa (input del evaluador).
///
/// El Shell construye este struct antes de llamar a `evaluate_permission`.
pub struct PermissionRequest {
    /// Pipeline de destino de la llamada del agente.
    pub pipeline: Pipeline,
    /// Etiqueta del objeto afectado (solo relevante para `Manage`).
    pub institutional_tag: Option<InstitutionalTag>,
    /// Estado actual del interruptor de producción (leído de DB por el Shell).
    pub production_override_active: bool,
    /// Identificador de la sesión MCP del agente conectado.
    pub agent_session_id: String,
    /// Pipeline/frontera invocada, ej. "ingest.submit_bar".
    pub requested_scope: String,
}

/// Decisión de permiso registrable (persistida en `permission_decisions`).
///
/// Campos según ADR-0020, Perfil D (Ops/Auditoría):
/// - Grupo I (Identidad & Integridad — universal, 6 campos)
/// - Grupo II (Soberanía — `owner_id`, `institutional_tag`)
/// - Grupo IV (Hardware — `node_id`, `process_id`)
/// - Dominio propio (4 campos específicos de esta feature)
#[derive(Debug)]
pub struct PermissionDecision {
    // ── Grupo I — Identidad & Integridad (universal) ─────────────────────
    /// UUID de esta decisión de permiso.
    pub id: Uuid,
    /// Nanosegundos desde epoch (momento de la evaluación).
    pub created_at: i64,
    /// Igual a `created_at` — inmutable tras inserción (tabla append-only).
    pub updated_at: i64,
    /// SHA-256 de los campos de dominio propio (sin circularidad).
    pub audit_hash: String,
    /// `audit_hash` de la decisión anterior en la cadena. `None` SOLO en la
    /// fila génesis -- sin sentinel de texto `"genesis"` (ADR-0141 M10,
    /// anomalía A4 de la auditoría retroactiva).
    pub audit_chain_hash: Option<String>,
    /// Posición monótona en la cadena (1, 2, 3, …).
    pub event_sequence_id: i64,
    // ── Grupo II — Soberanía ──────────────────────────────────────────────
    /// Propietario que controla el interruptor de producción.
    pub owner_id: Option<String>,
    /// Entorno del objeto afectado ("Live" / "Demo"), cuando aplica.
    pub institutional_tag: Option<String>,
    // ── Grupo IV — Hardware ───────────────────────────────────────────────
    /// Host donde corre el Gateway MCP.
    pub node_id: String,
    /// PID del proceso del Gateway MCP (como i64 para SQLite).
    pub process_id: i64,
    // ── Dominio propio (fuera del catálogo canónico) ──────────────────────
    /// Sesión MCP del agente que originó la llamada.
    pub agent_session_id: String,
    /// Pipeline/frontera invocada, ej. "ingest.submit_bar".
    pub requested_scope: String,
    /// "granted" | "denied:<razón>"
    pub permission_outcome: String,
    /// Estado del interruptor en el momento de la evaluación (0/1 en SQLite).
    pub production_override_active: bool,
}

// ────────────────────────────────────────────────────────────────────────────
// Función pura del evaluador (ADR-0123 — matriz de permisos)
// ────────────────────────────────────────────────────────────────────────────

/// Evalúa si el agente tiene permiso para invocar el pipeline dado.
///
/// Es una función **pura**: sin I/O, sin estado mutable, sin reloj de sistema.
/// El mismo `PermissionRequest` siempre produce el mismo `PermissionOutcome`.
///
/// La matriz de decisión (ADR-0123):
/// - `Ingest`, `Generate`, `Validate`, `Incubate`, `Feedback` → siempre `Granted`.
/// - `Manage` + `Demo` → `Granted`; `Manage` + `Live` sin interruptor → `Denied`.
/// - `Execute`, `Withdraw` → `Denied` salvo que el interruptor esté activo.
pub fn evaluate_permission(req: &PermissionRequest) -> PermissionOutcome {
    match req.pipeline {
        // Pipelines abiertos por defecto (descubrimiento y simulación).
        Pipeline::Ingest
        | Pipeline::Generate
        | Pipeline::Validate
        | Pipeline::Incubate
        | Pipeline::Feedback => PermissionOutcome::Granted,

        // `Manage` — condicionado por `institutional_tag`.
        Pipeline::Manage => match &req.institutional_tag {
            // Live sin interruptor activo → denegado.
            Some(InstitutionalTag::Live) if !req.production_override_active => {
                PermissionOutcome::Denied {
                    reason: "manage/live requiere production_override activo".into(),
                }
            }
            // Demo (libre) o Live con interruptor → concedido.
            _ => PermissionOutcome::Granted,
        },

        // Pipelines bloqueados por defecto (capital real).
        Pipeline::Execute | Pipeline::Withdraw => {
            if req.production_override_active {
                PermissionOutcome::Granted
            } else {
                PermissionOutcome::Denied {
                    reason: format!(
                        "{:?} bloqueado por defecto; activa production_override",
                        req.pipeline
                    ),
                }
            }
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Constructor de la decisión registrable
// ────────────────────────────────────────────────────────────────────────────

impl PermissionDecision {
    /// Construye una `PermissionDecision` completa lista para persistir.
    ///
    /// `prev_hash` es el `audit_hash` de la decisión anterior, o `None` si
    /// esta es la fila génesis de la cadena (ADR-0141 M10: sin sentinel de
    /// texto `"genesis"` -- el génesis se reconoce por `IS NULL`).
    /// `sequence_id` es la posición monótona en la cadena (último
    /// `event_sequence_id` + 1).
    pub fn build(
        req: &PermissionRequest,
        outcome: &PermissionOutcome,
        now_ns: i64,
        prev_hash: Option<String>,
        sequence_id: i64,
        node_id: String,
        pid: i64,
    ) -> Self {
        let outcome_str = outcome_to_string(outcome);

        // `audit_hash` se calcula sobre los campos de dominio propio únicamente.
        // Si incluyera el Grupo I (que contiene `audit_hash` mismo) habría
        // circularidad: necesitas el hash para calcularlo. Para el cómputo
        // (no para lo que se persiste) el génesis usa la constante
        // GENESIS_PREVIOUS_HASH, igual que el resto de las cadenas del
        // sistema (`crate::domain::audit_log`).
        let audit_hash = compute_audit_hash(
            &req.agent_session_id,
            &req.requested_scope,
            &outcome_str,
            req.production_override_active,
            prev_hash
                .as_deref()
                .unwrap_or(crate::domain::audit_log::GENESIS_PREVIOUS_HASH),
            sequence_id,
        );

        PermissionDecision {
            id: Uuid::now_v7(),
            created_at: now_ns,
            updated_at: now_ns,
            audit_hash,
            audit_chain_hash: prev_hash,
            event_sequence_id: sequence_id,
            owner_id: None, // se puede inyectar en el Shell si se conoce el propietario
            institutional_tag: req
                .institutional_tag
                .as_ref()
                .map(institutional_tag_to_string),
            node_id,
            process_id: pid,
            agent_session_id: req.agent_session_id.clone(),
            requested_scope: req.requested_scope.clone(),
            permission_outcome: outcome_str,
            production_override_active: req.production_override_active,
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Helpers puros (sin I/O)
// ────────────────────────────────────────────────────────────────────────────

/// Serializa `PermissionOutcome` al formato de texto de la columna SQL.
pub fn outcome_to_string(outcome: &PermissionOutcome) -> String {
    match outcome {
        PermissionOutcome::Granted => "granted".into(),
        PermissionOutcome::Denied { reason } => format!("denied:{reason}"),
    }
}

/// Serializa `InstitutionalTag` al texto de la columna SQL.
pub fn institutional_tag_to_string(tag: &InstitutionalTag) -> String {
    match tag {
        InstitutionalTag::Live => "Live".into(),
        InstitutionalTag::Demo => "Demo".into(),
    }
}

/// Calcula el `audit_hash` SHA-256 de los campos de dominio propio.
///
/// La entrada es la concatenación de los campos con `|` como separador.
/// Se incluye `prev_hash` y `sequence_id` para que el hash cubra también el
/// encadenamiento — cualquier reordenación de filas lo rompe.
pub fn compute_audit_hash(
    agent_session_id: &str,
    requested_scope: &str,
    outcome_str: &str,
    production_override_active: bool,
    prev_hash: &str,
    sequence_id: i64,
) -> String {
    use sha2::{Digest, Sha256};

    let payload = format!(
        "{agent_session_id}|{requested_scope}|{outcome_str}|{production_override_active}|{prev_hash}|{sequence_id}"
    );

    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    format!("{:x}", hasher.finalize())
}

// ────────────────────────────────────────────────────────────────────────────
// Pruebas unitarias (Capa 1 — ADR-0133)
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn req(pipeline: Pipeline) -> PermissionRequest {
        PermissionRequest {
            pipeline,
            institutional_tag: None,
            production_override_active: false,
            agent_session_id: "session-test".into(),
            requested_scope: "test.scope".into(),
        }
    }

    // Criterio 3: todos los pipelines abiertos devuelven Granted sin interruptor.
    #[test]
    fn ingest_pipeline_is_always_granted() {
        for pipeline in [
            Pipeline::Ingest,
            Pipeline::Generate,
            Pipeline::Validate,
            Pipeline::Incubate,
            Pipeline::Feedback,
        ] {
            let result = evaluate_permission(&req(pipeline));
            assert_eq!(result, PermissionOutcome::Granted);
        }
    }

    // Criterio 4: manage + Demo → Granted sin interruptor.
    #[test]
    fn manage_demo_is_granted_without_override() {
        let r = PermissionRequest {
            pipeline: Pipeline::Manage,
            institutional_tag: Some(InstitutionalTag::Demo),
            production_override_active: false,
            agent_session_id: "s".into(),
            requested_scope: "manage.rebalance".into(),
        };
        assert_eq!(evaluate_permission(&r), PermissionOutcome::Granted);
    }

    // Criterio 5: manage + Live sin interruptor → Denied.
    #[test]
    fn manage_live_is_denied_without_override() {
        let r = PermissionRequest {
            pipeline: Pipeline::Manage,
            institutional_tag: Some(InstitutionalTag::Live),
            production_override_active: false,
            agent_session_id: "s".into(),
            requested_scope: "manage.rebalance".into(),
        };
        assert!(matches!(evaluate_permission(&r), PermissionOutcome::Denied { .. }));
    }

    // Criterio 6: manage + Live con interruptor → Granted.
    #[test]
    fn manage_live_is_granted_with_override() {
        let r = PermissionRequest {
            pipeline: Pipeline::Manage,
            institutional_tag: Some(InstitutionalTag::Live),
            production_override_active: true,
            agent_session_id: "s".into(),
            requested_scope: "manage.rebalance".into(),
        };
        assert_eq!(evaluate_permission(&r), PermissionOutcome::Granted);
    }

    // Criterio 7: execute sin interruptor → Denied.
    #[test]
    fn execute_is_denied_without_override() {
        let result = evaluate_permission(&req(Pipeline::Execute));
        assert!(matches!(result, PermissionOutcome::Denied { .. }));
    }

    // Criterio 8: execute con interruptor → Granted.
    #[test]
    fn execute_is_granted_with_override() {
        let r = PermissionRequest {
            pipeline: Pipeline::Execute,
            institutional_tag: None,
            production_override_active: true,
            agent_session_id: "s".into(),
            requested_scope: "execute.send_order".into(),
        };
        assert_eq!(evaluate_permission(&r), PermissionOutcome::Granted);
    }

    // Criterio 12 (verificación inline): withdraw sin interruptor → Denied.
    #[test]
    fn withdraw_is_denied_without_override() {
        let result = evaluate_permission(&req(Pipeline::Withdraw));
        assert!(matches!(result, PermissionOutcome::Denied { .. }));
    }

    // Sanidad del audit_hash: distintos inputs → distintos hashes.
    #[test]
    fn audit_hash_differs_for_different_inputs() {
        let h1 = compute_audit_hash("session-A", "ingest.bar", "granted", false, "genesis", 1);
        let h2 = compute_audit_hash("session-B", "ingest.bar", "granted", false, "genesis", 1);
        assert_ne!(h1, h2);
    }

    // Sanidad del audit_hash: el mismo input siempre produce el mismo hash (determinismo).
    #[test]
    fn audit_hash_is_deterministic() {
        let h1 = compute_audit_hash("s", "scope", "granted", false, "genesis", 1);
        let h2 = compute_audit_hash("s", "scope", "granted", false, "genesis", 1);
        assert_eq!(h1, h2);
    }
}
