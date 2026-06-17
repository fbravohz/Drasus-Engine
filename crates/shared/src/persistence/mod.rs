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
//! - `job`: repositorio para las tablas `jobs` y `job_results` del Async
//!   Job Executor (`docs/features/async-job-executor.md`
//!   TTR-ASYNC-EXECUTOR-001/003/004/006, migración `0003_jobs.sql`).
//! - `pool`: fábrica del pool de conexiones + runner de migraciones embebidas.

pub mod audit_log;
pub mod job;
pub mod pool;
