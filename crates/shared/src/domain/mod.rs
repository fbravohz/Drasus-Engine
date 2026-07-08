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
//! - `data_aggregation`: ruido gaussiano de privacidad diferencial con RNG
//!   sembrado (Box-Muller), hash unidireccional de topología de estrategia
//!   (SHA-256), verificación de k-anonimato y agregación de índices
//!   vendibles con hash de auditoría encadenado por `event_sequence_id`
//!   (`docs/features/data-aggregation.md`, ADR-0144, ADR-0102, ADR-0143,
//!   ADR-0141). STORY-036.
//! - `instance_continuity`: KDF (Argon2id) + cifrado autenticado
//!   AES-256-GCM con nonce sembrado e inyectado, filtro del delta de
//!   respaldo (excluye secretos de bróker/IPs live) y el gate de
//!   titularidad exclusiva por `custody_epoch` (concurrencia optimista a
//!   nivel de instancia) (`docs/features/instance-continuity.md`,
//!   ADR-0146 cimiento #11, ADR-0093, ADR-0141, ADR-0020). STORY-039.
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
//! - `master_account_hierarchy`: gate de autorización de override por
//!   `ConsentVerdict` REAL (#5), efecto local "eliminar = archivar" (nunca
//!   DELETE) y hash de auditoría encadenado de ambas tablas (jerarquía
//!   MUTABLE por `row_version`, atestaciones APPEND-ONLY por
//!   `event_sequence_id`) (`docs/features/master-account-hierarchy.md`,
//!   ADR-0147 cimiento #12, ADR-0093, ADR-0141, ADR-0020). STORY-040.
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
//! - `third_party_api_gateway`: hash de credencial de API (SHA-256,
//!   ADR-0093), autenticación con revocación prioritaria, ventana de
//!   rate-limit determinista y composición de las cuatro puertas de la
//!   decisión de delegación (`docs/features/third-party-api-gateway.md`,
//!   ADR-0143, ADR-0144, ADR-0141). STORY-035.
//! - `usage_metering`: cálculo de nocional (tamaño × precio, entero
//!   escalado ×10⁸ con reescalado ×10¹⁶→×10⁸), acumulación por ciclo,
//!   detección de cruce de umbral y hash de auditoría encadenado por
//!   `event_sequence_id` (`docs/features/usage-metering.md`, ADR-0143,
//!   ADR-0144, ADR-0141). STORY-030.
//! - `verified_account_registry`: cálculo puro del track record por ámbito
//!   de atestación (soberano/read-only del bróker) a partir de los eventos
//!   de #6 -- gain% que EXCLUYE el flujo de capital, drawdown máximo,
//!   estadística de trades, firma de integridad REPRODUCIBLE del contenido
//!   y hash de auditoría encadenado por `event_sequence_id`
//!   (`docs/features/verified-account-registry.md`, ADR-0145 cimiento #10,
//!   ADR-0093, ADR-0141, ADR-0020). STORY-037.

pub mod audit_log;
pub mod central_identity;
pub mod clock;
pub mod consent_registry;
pub mod data_aggregation;
pub mod enriched_domain_events;
pub mod instance_continuity;
pub mod institutional_report_engine;
pub mod job;
pub mod licensing_system;
pub mod logic;
pub mod master_account_hierarchy;
pub mod mcp_gateway;
pub mod plan_tier_quota;
pub mod telemetry;
pub mod third_party_api_gateway;
pub mod usage_metering;
pub mod verified_account_registry;
pub mod worker_orchestrator;
