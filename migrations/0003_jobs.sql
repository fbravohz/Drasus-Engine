-- Migration 0003: Async Job Executor (docs/features/async-job-executor.md
-- TTR-ASYNC-EXECUTOR-001..006, ADR-0011, ADR-0020)
--
-- Creates the `jobs` and `job_results` tables: the durable backing store for
-- the three-phase async job pattern (Disparo -> Monitoreo -> Recuperación,
-- ADR-0011). `jobs` is mutable (state/progress are updated as a job
-- advances); `job_results` is append-only (a job's final result, once
-- inserted, is never modified -- async-job-executor.md "Restricciones":
-- "NUNCA un job se modifica después de completar. Resultado es inmutable.").
--
-- ADR-0020 field profile for these tables (async-job-executor.md
-- "Gobernanza y Estándares"):
--   - Group I  (Identidad & Integridad, UNIVERSAL): id, created_at,
--     updated_at, audit_hash, audit_chain_hash, event_sequence_id.
--   - Concurrencia e integridad (explicit list in the feature doc):
--     process_id (Worker ID), session_id, node_id (Hardware Fingerprint),
--     logic_hash (Executor version). `audit_chain_hash` and
--     `event_sequence_id` are already covered by Group I.
--   - Soberanía: owner_id, access_token_id.
-- Groups III (Linaje Alpha, beyond logic_hash) and V (Forense & Ejecución)
-- are NOT listed by the feature doc and are intentionally omitted --
-- ADR-0020 "Aplicación": "PROHIBIDO copy-paste masivo".
--
-- `job_results` carries Group I (universal, ADR-0020: every entity
-- carries it from its first migration) plus its own functional columns.
-- The concurrency/integrity/sovereignty metadata (process_id, session_id,
-- node_id, logic_hash, owner_id, access_token_id) describes the *job's*
-- execution context and lives on `jobs`; `job_results` reaches it via
-- `job_uuid` -> `jobs.id` rather than duplicating those columns.
--
-- Feature-specific columns (async-job-executor.md TTR-001/003/005/006):
--   - jobs: uuid (== Group I `id`, this table's primary key -- the feature
--     doc's "UUID único del job"), user_id, job_type, parameters (JSON),
--     state (QUEUED|RUNNING|COMPLETED|FAILED|CANCELLED), progress (0-100).
--     `created_at`/`updated_at` (Group I) serve as the feature doc's
--     "timestamps".
--   - job_results: job_uuid (FK -> jobs.id), result_data (JSON),
--     error_message, completed_at.
--
-- Append-only enforcement on `job_results` (docs/features/async-job-executor.md
-- "Restricciones" + migration 0002_audit_log.sql pattern): triggers reject
-- any UPDATE or DELETE at the database level.
--
-- Idempotency (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF
-- NOT EXISTS` / `CREATE TRIGGER IF NOT EXISTS` make re-running this
-- migration a no-op.
--
-- STRICT mode + CHECK constraints (ADR-0141 M4/M5/M6/M9/M12, in-situ edit of
-- the GREENFIELD baseline, retroactive audit 2026-07):
--   - `jobs` is MUTABLE (state/progress/process_id are updated in place), so
--     its monotonic version counter is renamed `event_sequence_id` ->
--     `row_version` (ADR-0141 "Semántica de secuencias": that name is
--     reserved for the GLOBAL append-only position of an event-store;
--     `jobs` never had a `UNIQUE` on it, confirming it was always a
--     per-row version counter, just mis-named).
--   - `jobs.state` gets `CHECK (state IN (...))` over the five canonical
--     values of `crate::domain::job::JobState`.
--   - `jobs.parameters` and `job_results.result_data` get
--     `CHECK (json_valid(...))` (the latter nullable, since a failed job has
--     no result payload).
--   - The FK `job_results.job_uuid -> jobs.id` gets `ON DELETE RESTRICT`
--     explicit (was implicit `NO ACTION`; `jobs` rows are never deleted in
--     this system, so this only makes the invariant readable, not a
--     behavior change).

CREATE TABLE IF NOT EXISTS jobs (
    -- I. Identidad & Integridad (universal, ADR-0020). `id` IS the job's
    -- UUID (TTR-001 "UUID único del job"), generated before the row is
    -- inserted (persist-before-ack, TTR-001 "Job se guarda ANTES de
    -- retornar UUID").
    id                 TEXT    NOT NULL PRIMARY KEY, -- UUID (TTR-001 "job_uuid")
    created_at         INTEGER NOT NULL,             -- Nanoseconds since epoch (Clock port)
    updated_at         INTEGER NOT NULL,             -- Nanoseconds since epoch; bumped on every state/progress update
    audit_hash         TEXT    NOT NULL,             -- SHA-256 snapshot of this row's content
    audit_chain_hash   TEXT,                         -- Previous audit_hash for this job (NULL for the first write)
    row_version        INTEGER NOT NULL,             -- Version counter for THIS row; starts at 1, +1 on every UPDATE (ADR-0141)

    -- Concurrencia e integridad (async-job-executor.md "Gobernanza y Estándares")
    process_id         TEXT,                         -- Worker ID that last touched this job (NULL while QUEUED)
    session_id         TEXT,                         -- Runtime Grouping (executor session)
    node_id            TEXT,                         -- Hardware Fingerprint of the executing node
    logic_hash         TEXT,                         -- Executor version (commit/build hash) that ran this job

    -- Soberanía
    owner_id           TEXT,                         -- Dueño capital/IP (nullable: not every job has one)
    access_token_id    TEXT,                         -- Auth Tracking (nullable)

    -- TTR-001/002/005/006 feature-specific fields
    user_id            TEXT    NOT NULL,             -- Requesting user (TTR-001 "Entrada": JobRequest.user_id)
    job_type           TEXT    NOT NULL,             -- e.g. BACKTEST, GENERATE_CANDIDATES, OPTIMIZE_PORTFOLIO
    parameters         TEXT    NOT NULL              -- JSON-encoded job parameters (JobRequest.parameters)
        CHECK (json_valid(parameters)),
    state              TEXT    NOT NULL              -- QUEUED | RUNNING | COMPLETED | FAILED | CANCELLED
        CHECK (state IN ('QUEUED', 'RUNNING', 'COMPLETED', 'FAILED', 'CANCELLED')),
    progress           INTEGER NOT NULL DEFAULT 0    -- 0-100 (TTR-005)
) STRICT;

-- Recovery access path (TTR-004: startup scan for QUEUED/RUNNING jobs).
CREATE INDEX IF NOT EXISTS idx_jobs_state
    ON jobs (state);

CREATE TABLE IF NOT EXISTS job_results (
    -- I. Identidad & Integridad (universal, ADR-0020).
    id                 TEXT    NOT NULL PRIMARY KEY, -- UUID for this result row
    created_at         INTEGER NOT NULL,             -- Nanoseconds since epoch (Clock port); == completed_at
    updated_at         INTEGER NOT NULL,             -- Nanoseconds since epoch; append-only => always equals created_at
    audit_hash         TEXT    NOT NULL,             -- SHA-256 snapshot of this row's content
    audit_chain_hash   TEXT,                         -- Previous result's audit_hash (NULL for the first result ever)
    event_sequence_id  INTEGER NOT NULL UNIQUE,      -- Monotonic chain position across all job_results (1, 2, 3, ...)

    -- TTR-003/005 feature-specific fields ("Result object")
    job_uuid           TEXT    NOT NULL,             -- FK -> jobs.id (the job this result belongs to)
    result_data        TEXT                          -- JSON-encoded result payload (NULL on failure)
        CHECK (result_data IS NULL OR json_valid(result_data)),
    error_message      TEXT,                         -- Error description (NULL on success)
    completed_at       INTEGER NOT NULL,             -- Nanoseconds since epoch (Clock port) when the job reached a terminal state

    FOREIGN KEY (job_uuid) REFERENCES jobs (id) ON DELETE RESTRICT
) STRICT;

-- Lookup access path (async-job-executor.md "GET /api/jobs/{uuid}/result").
CREATE INDEX IF NOT EXISTS idx_job_results_job_uuid
    ON job_results (job_uuid);

CREATE INDEX IF NOT EXISTS idx_job_results_event_sequence_id
    ON job_results (event_sequence_id);

-- Append-only enforcement: reject UPDATE (async-job-executor.md
-- "Restricciones": "NUNCA un job se modifica después de completar.
-- Resultado es inmutable.").
CREATE TRIGGER IF NOT EXISTS trg_job_results_no_update
BEFORE UPDATE ON job_results
BEGIN
    SELECT RAISE(ABORT, 'job_results is append-only: UPDATE is forbidden');
END;

-- Append-only enforcement: reject DELETE (async-job-executor.md
-- "Restricciones": "NUNCA se pierden job results. Append-only en tabla
-- SQLite.").
CREATE TRIGGER IF NOT EXISTS trg_job_results_no_delete
BEFORE DELETE ON job_results
BEGIN
    SELECT RAISE(ABORT, 'job_results is append-only: DELETE is forbidden');
END;
