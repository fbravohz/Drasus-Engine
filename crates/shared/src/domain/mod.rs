//! [CORE] Lógica de negocio pura para `shared`.
//!
//! Sin I/O, sin reloj de sistema, sin azar sin semilla (ADR-0002/0004).
//!
//! - `audit_log`: construcción y verificación de la cadena de hashes del
//!   Audit Log (`docs/features/audit-log.md` TTR-001, ADR-0015, ADR-0020 V2,
//!   ADR-0027).
//! - `clock`: el puerto `Clock` y la implementación de reloj determinista
//!   (lista para backtest) (W3, `docs/features/clock.md` TTR-001/TTR-002).
//! - `job`: la máquina de estados de jobs asíncronos -- transiciones
//!   válidas, progreso y estimación de tiempo restante
//!   (`docs/features/async-job-executor.md`
//!   TTR-ASYNC-EXECUTOR-002/004/005/006, ADR-0004, ADR-0011).
//! - `logic`: placeholder vacío, solo estructura (F0/W1).
//! - `mcp_gateway`: evaluador de permisos puro del Gateway MCP (ADR-0123) —
//!   tipos `Pipeline`, `PermissionRequest`, `PermissionDecision` y la función
//!   `evaluate_permission` (sin I/O). STORY-010.
//! - `telemetry`: construcción pura de muestras de latencia/heartbeat y la
//!   decisión de poda por ventana de retención (`docs/features/telemetry.md`
//!   TTR-001, ADR-0015, ADR-0020 V2).

pub mod audit_log;
pub mod clock;
pub mod job;
pub mod logic;
pub mod mcp_gateway;
pub mod telemetry;
pub mod worker_orchestrator;
