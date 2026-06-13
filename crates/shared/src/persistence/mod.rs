//! [SHELL] Centralized SQLite persistence wiring for Drasus Engine.
//!
//! Owns the single connection pool factory and the embedded SQLx
//! migration runner (ADR-0006: "Migraciones Centralizadas con SQLx
//! Migrator"). Pipeline modules (`ingest`, `generate`, `validate`,
//! `incubate`, `manage`, `execute`, `feedback`, `withdraw`) consume this
//! through `shared`'s public interface to obtain a ready, migrated pool;
//! they never construct their own pool or run migrations independently.
//!
//! Migration scripts live in `/migrations` at the workspace root
//! (ADR-0006), embedded into the binary at compile time via
//! `sqlx::migrate!`.
//!
//! - `audit_log`: append-only repository for the Audit Log
//!   (`docs/features/audit-log.md` TTR-001, migration
//!   `0002_audit_log.sql`).
//! - `job`: repository for the Async Job Executor's `jobs` and
//!   `job_results` tables (`docs/features/async-job-executor.md`
//!   TTR-ASYNC-EXECUTOR-001/003/004/006, migration `0003_jobs.sql`).
//! - `pool`: connection pool factory + embedded migration runner.

pub mod audit_log;
pub mod job;
pub mod pool;
