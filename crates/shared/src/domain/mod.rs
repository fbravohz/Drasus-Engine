//! [CORE] Lógica de negocio pura para `shared`.
//!
//! Sin I/O, sin reloj de sistema, sin azar sin semilla (ADR-0002/0004).
//!
//! - `audit_log`: construcción y verificación de la cadena de hashes del
//!   Audit Log (`docs/features/audit-log.md` TTR-001, ADR-0015, ADR-0020,
//!   ADR-0027).
//! - `central_identity`: huella de hardware determinista, validación de
//!   formato de correo, verificación de firma OAuth y el hash de auditoría
//!   encadenado por `row_version` de la cuenta (`docs/features/central-identity.md`,
//!   ADR-0143, ADR-0144, ADR-0141). STORY-027.
//! - `clock`: el puerto `Clock` y la implementación de reloj determinista
//!   (lista para backtest) (W3, `docs/features/clock.md` TTR-001/TTR-002).
//! - `consent_registry`: fusión pura de una acción de consentimiento sobre
//!   el estado vigente (event-sourcing con snapshot completo), resolución
//!   de cobertura por tipo de dato (`ConsentVerdict`) y hash de auditoría
//!   encadenado por `event_sequence_id` (`docs/features/consent-registry.md`,
//!   ADR-0143, ADR-0144, ADR-0141). STORY-031.
//! - `institutional_report_engine`: ensamblado puro de reportes
//!   institucionales, serialización canónica del reporte, firma de
//!   integridad REPRODUCIBLE (`compute_report_signature`) y hash de
//!   auditoría encadenado por `event_sequence_id` de la fila del ledger
//!   (`compute_report_audit_hash`), distinto en rol de la firma
//!   (`docs/features/institutional-report-engine.md`, ADR-0144, ADR-0027,
//!   ADR-0141, ADR-0020, ADR-0093). STORY-034.
//! - `job`: la máquina de estados de jobs asíncronos -- transiciones
//!   válidas, progreso y estimación de tiempo restante
//!   (`docs/features/async-job-executor.md`
//!   TTR-ASYNC-EXECUTOR-002/004/005/006, ADR-0004, ADR-0011).
//! - `licensing_system`: verificación de firma Ed25519 asimétrica, huella de
//!   hardware (comparación, NO recálculo), heartbeat/gracia, supresión de
//!   telemetría por tier y derivación del veredicto `ExecutionGate`
//!   (`docs/features/licensing-system.md`, ADR-0143, ADR-0144, ADR-0093,
//!   ADR-0141). STORY-028.
//! - `logic`: placeholder vacío, solo estructura (F0/W1).
//! - `mcp_gateway`: evaluador de permisos puro del Gateway MCP (ADR-0123) —
//!   tipos `Pipeline`, `PermissionRequest`, `PermissionDecision` y la función
//!   `evaluate_permission` (sin I/O). STORY-010.
//! - `plan_tier_quota`: catálogo configurable de planes -- validación de
//!   coherencia de un plan (tier + cuotas + precio), resolución de límites
//!   (`PlanLimits`) y hash de auditoría encadenado por `row_version`
//!   (`docs/features/plan-tier-quota.md`, ADR-0143, ADR-0144, ADR-0141).
//!   STORY-029.
//! - `telemetry`: construcción pura de muestras de latencia/heartbeat y la
//!   decisión de poda por ventana de retención (`docs/features/telemetry.md`
//!   TTR-001, ADR-0015, ADR-0020).
//! - `usage_metering`: cálculo de nocional (tamaño × precio, entero
//!   escalado ×10⁸ con reescalado ×10¹⁶→×10⁸), acumulación por ciclo,
//!   detección de cruce de umbral y hash de auditoría encadenado por
//!   `event_sequence_id` (`docs/features/usage-metering.md`, ADR-0143,
//!   ADR-0144, ADR-0141). STORY-030.

pub mod audit_log;
pub mod central_identity;
pub mod clock;
pub mod consent_registry;
pub mod enriched_domain_events;
pub mod institutional_report_engine;
pub mod job;
pub mod licensing_system;
pub mod logic;
pub mod mcp_gateway;
pub mod plan_tier_quota;
pub mod telemetry;
pub mod usage_metering;
pub mod worker_orchestrator;
