-- Migration 0002: Audit Log (docs/features/audit-log.md TTR-001, ADR-0015,
-- ADR-0020, ADR-0027)
--
-- Creates the `audit_events` table: the append-only, hash-chained record
-- of every significant system event (ADR-0015: "Arquitectura de
-- Causalidad" — the audit log is the source of truth for Feedback;
-- ADR-0027: "Event Sourcing & Inventory Reconstruction").
--
-- ADR-0020 field profile for this table ("Ops / Auditoría: Identidad +
-- Soberanía + Hardware", per architect/SKILL.md filter):
--   - Group I  (Identidad & Integridad, UNIVERSAL): id, created_at,
--     updated_at, audit_hash, audit_chain_hash, event_sequence_id.
--   - Group II (Soberanía & Propiedad): owner_id, institutional_tag,
--     manifest_id, access_token_id.
--   - Group IV (Infraestructura & Ops / "Hardware"): process_id,
--     session_id, node_id.
-- Groups III and V are NOT applicable to this table's profile (no Alpha
-- lineage, no execution/forensic fields) and are intentionally omitted —
-- ADR-0020 "Aplicación": "PROHIBIDO copy-paste masivo".
--
-- Feature-specific columns (docs/features/audit-log.md TTR-001 "Entrada"
-- and "Restricciones" — mandatory fields: timestamp, action type, entity
-- type, entity id, details):
--   - action_type, entity_type, entity_id, details_json.
--
-- Append-only enforcement (docs/features/audit-log.md "Restricciones":
-- "NUNCA un evento se borra" / "NUNCA un evento se modifica"):
--   - Triggers reject any UPDATE or DELETE against `audit_events` at the
--     database level, in addition to the domain-level hash-chain
--     verification (crates/shared/src/domain/audit_log.rs) that detects
--     tampering with historical rows.
--
-- Idempotency (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF
-- NOT EXISTS` / `CREATE TRIGGER IF NOT EXISTS` make re-running this
-- migration a no-op.
--
-- STRICT mode + CHECK(json_valid) (ADR-0141 M4/M5/M12, in-situ edit of the
-- GREENFIELD baseline, retroactive audit 2026-07).

CREATE TABLE IF NOT EXISTS audit_events (
    -- I. Identidad & Integridad (universal, ADR-0020)
    id                 TEXT    NOT NULL PRIMARY KEY, -- UUID (TTR-001 "log_id")
    created_at         INTEGER NOT NULL,             -- Nanoseconds since epoch (Clock port)
    updated_at         INTEGER NOT NULL,             -- Nanoseconds since epoch; append-only => always equals created_at
    audit_hash         TEXT    NOT NULL,             -- SHA-256 of this event's content + previous link
    audit_chain_hash   TEXT,                         -- Previous row's audit_hash (NULL only for the genesis row)
    event_sequence_id  INTEGER NOT NULL UNIQUE,      -- Monotonic chain position (1, 2, 3, ...)

    -- II. Soberanía & Propiedad
    owner_id           TEXT,                         -- Dueño capital/IP (nullable: not every event has one)
    institutional_tag  TEXT    NOT NULL,             -- Environment (TTR-001: mandatory)
    manifest_id        TEXT,                         -- Design Contract (nullable)
    access_token_id    TEXT,                         -- Auth Tracking (nullable)

    -- IV. Infraestructura & Ops ("Hardware")
    process_id         TEXT    NOT NULL,             -- Job Anchor (TTR-001: mandatory)
    session_id         TEXT,                         -- Runtime Grouping (nullable)
    node_id            TEXT,                         -- Hardware ID (nullable)

    -- TTR-001 feature-specific fields (audit-log.md "Entrada" / "Restricciones")
    action_type        TEXT    NOT NULL,             -- e.g. ORDER_STATE_CHANGE, ANOMALY_DETECTED, USER_VETO
    entity_type        TEXT    NOT NULL,             -- Type of the entity the event refers to
    entity_id          TEXT    NOT NULL,             -- Identifier of that entity
    details_json       TEXT    NOT NULL              -- Structured event details (JSON-encoded)
        CHECK (json_valid(details_json))
) STRICT;

-- Chronological / replay access path (ADR-0027 event sourcing recovery).
CREATE INDEX IF NOT EXISTS idx_audit_events_event_sequence_id
    ON audit_events (event_sequence_id);

-- Investigation access path (audit-log.md: "¿qué pasó con la estrategia
-- XYZ el 2026-04-07?" -> lookup by entity).
CREATE INDEX IF NOT EXISTS idx_audit_events_entity
    ON audit_events (entity_type, entity_id);

-- Append-only enforcement: reject UPDATE (audit-log.md: "NUNCA un evento se
-- modifica después de ser grabado").
CREATE TRIGGER IF NOT EXISTS trg_audit_events_no_update
BEFORE UPDATE ON audit_events
BEGIN
    SELECT RAISE(ABORT, 'audit_events is append-only: UPDATE is forbidden');
END;

-- Append-only enforcement: reject DELETE (audit-log.md: "NUNCA un evento se
-- borra del Audit Log. Append-only absoluto.").
CREATE TRIGGER IF NOT EXISTS trg_audit_events_no_delete
BEFORE DELETE ON audit_events
BEGIN
    SELECT RAISE(ABORT, 'audit_events is append-only: DELETE is forbidden');
END;
