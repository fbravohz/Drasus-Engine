-- Migration 0001: Foundation Master Fields (ADR-0020)
--
-- Creates the `foundation_master_fields` table materializing the Global
-- Persistence Contract: the mandatory set of 25 fields every entity
-- in Drasus Engine must carry from its very first migration (ADR-0020,
-- "Foundation Inundation Principle V2").
--
-- This table is the anchor/reference implementation of the contract. Future
-- module-owned tables (ADR-0003: each module owns its own tables) embed
-- this same 25-field set alongside their domain-specific columns.
--
-- Idempotency: `CREATE TABLE IF NOT EXISTS` + `CREATE INDEX IF NOT EXISTS`
-- guarantee that re-running this migration is a no-op if already applied
-- (ADR-0006: "Cada migración debe ser determinista e idempotente").
-- SQLx additionally tracks applied migrations via `_sqlx_migrations` and
-- will not re-run a migration whose checksum is unchanged.
--
-- STRICT mode (ADR-0141 M12, in-situ edit of the GREENFIELD baseline,
-- retroactive audit 2026-07): every column here already used only the
-- canonical SQLite types (`TEXT`/`INTEGER`), so declaring the table
-- `STRICT` rejects future type drift without changing any column.

CREATE TABLE IF NOT EXISTS foundation_master_fields (
    -- I. Identidad & Integridad
    id                     TEXT    NOT NULL PRIMARY KEY, -- UUID
    created_at             INTEGER NOT NULL,             -- Nanoseconds since epoch
    updated_at             INTEGER NOT NULL,             -- Nanoseconds since epoch
    audit_hash             TEXT    NOT NULL,             -- SHA-256
    audit_chain_hash       TEXT,                         -- Blockchain-lite link (NULL for genesis row)
    event_sequence_id      INTEGER NOT NULL,             -- Recovery sequence

    -- II. Soberanía & Propiedad
    owner_id               TEXT,                         -- Dueño capital/IP
    institutional_tag      TEXT,                         -- Environment
    manifest_id            TEXT,                         -- Design Contract
    access_token_id        TEXT,                         -- Auth Tracking

    -- III. Linaje Alpha & Datos
    version_node_id        TEXT,                         -- DAG Link
    parent_id              TEXT,                         -- Puntero Genético
    logic_hash             TEXT,                         -- Commit Código/Binario
    data_snapshot_id       TEXT,                         -- PIT Market Snapshot
    transformation_id      TEXT,                         -- Raw vs Synthetic flag

    -- IV. Infraestructura & Ops
    process_id             TEXT,                         -- Job Anchor
    session_id             TEXT,                         -- Runtime Grouping
    node_id                TEXT,                         -- Hardware ID

    -- V. Forense & Ejecución
    portfolio_container_id TEXT,                         -- Governance
    compliance_status_id   TEXT,                         -- Veredicto Riesgo
    risk_audit_id          TEXT,                         -- Ticket detallado riesgo
    indicator_state_hash   TEXT,                         -- Technical Snapshot
    execution_latency_ms   INTEGER,                      -- Latency in milliseconds
    source_signal_id       TEXT,                         -- Signal link
    signature_hash         TEXT                          -- HMAC signals
) STRICT;

-- Recovery/event-sourcing access path (ADR-0020 Part I, ADR-0006 crash recovery).
CREATE INDEX IF NOT EXISTS idx_foundation_master_fields_event_sequence_id
    ON foundation_master_fields (event_sequence_id);
