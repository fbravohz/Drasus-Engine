//! [SHELL] Cableado centralizado de persistencia SQLite para Drasus Engine.
//!
//! Dueño de la única fábrica del pool de conexiones y del runner de
//! migraciones de SQLx embebidas (ADR-0006: "Migraciones Centralizadas
//! con SQLx Migrator"). Los módulos del pipeline (`ingest`, `generate`,
//! `validate`, `incubate`, `manage`, `execute`, `feedback`, `withdraw`)
//! consumen esto a través de la interfaz pública de `shared` para
//! obtener un pool ya migrado y listo; nunca construyen su propio pool ni
//! corren migraciones por su cuenta.
//!
//! Los scripts de migración viven en `/migrations` en la raíz del
//! workspace (ADR-0006), embebidos en el binario en tiempo de
//! compilación vía `sqlx::migrate!`.
//!
//! - `audit_log`: repositorio de solo-apéndice para el Audit Log
//!   (`docs/features/audit-log.md` TTR-001, migración
//!   `0002_audit_log.sql`).
//! - `central_identity`: repositorio MUTABLE (con `row_version`, ADR-0141)
//!   para la tabla `accounts` (`docs/features/central-identity.md`,
//!   ADR-0143, ADR-0144, migración `0007_central_identity.sql`). STORY-027.
//! - `consent_registry`: repositorio APPEND-ONLY (con `event_sequence_id`,
//!   ADR-0141) para la tabla `consent_records` (`docs/features/consent-registry.md`,
//!   ADR-0143, ADR-0144, migración `0011_consent_registry.sql`). STORY-031.
//! - `job`: repositorio para las tablas `jobs` y `job_results` del Async
//!   Job Executor (`docs/features/async-job-executor.md`
//!   TTR-ASYNC-EXECUTOR-001/003/004/006, migración `0003_jobs.sql`).
//! - `licensing_system`: repositorio MUTABLE (con `row_version`, ADR-0141)
//!   para la tabla `licenses` (`docs/features/licensing-system.md`,
//!   ADR-0143, ADR-0144, migración `0008_licensing_system.sql`). STORY-028.
//! - `pool`: fábrica del pool de conexiones + runner de migraciones embebidas.
//! - `plan_tier_quota`: repositorio MUTABLE (con `row_version`, ADR-0141)
//!   para la tabla `plans` (`docs/features/plan-tier-quota.md`, ADR-0143,
//!   ADR-0144, migración `0009_plan_tier_quota.sql`). STORY-029.
//! - `telemetry`: repositorio para `telemetry_samples` (insertar por lote,
//!   purgar por corte, consultar por `metric_name` + rango) —
//!   `docs/features/telemetry.md` TTR-001, migración `0004_telemetry.sql`.
//! - `usage_metering`: repositorio APPEND-ONLY (con `event_sequence_id`,
//!   ADR-0141) para la tabla `usage_records` (`docs/features/usage-metering.md`,
//!   ADR-0143, ADR-0144, migración `0010_usage_metering.sql`). STORY-030.

pub mod audit_log;
pub mod central_identity;
pub mod consent_registry;
pub mod enriched_domain_events;
pub mod job;
pub mod licensing_system;
pub mod mcp_gateway;
pub mod plan_tier_quota;
pub mod pool;
pub mod telemetry;
pub mod usage_metering;
