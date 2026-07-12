-- Migración 0005: Gateway MCP (docs/features/agentic-mcp-gateway.md, ADR-0123,
-- ADR-0020 Perfil D — Ops/Auditoría)
--
-- Crea dos tablas:
--   - `permission_decisions`: log inmutable de cada decisión de permiso
--     que toma el Gateway MCP. Append-only: sin UPDATE ni DELETE (igual que
--     `audit_events`). Los 6 campos del Grupo I + Grupo II (Soberanía) +
--     Grupo IV (Hardware) + 4 campos de dominio propio (spec agentic-mcp-gateway.md).
--   - `mcp_gateway_config`: tabla de estado mutable clave-valor que alberga
--     el interruptor de producción (`production_override_active`). Es la tabla
--     de *estado*, no de *hechos*: tiene exactamente una fila por clave.
--
-- Idempotencia (ADR-0006): CREATE TABLE IF NOT EXISTS / INSERT OR IGNORE.
--
-- STRICT mode + triggers append-only + audit_chain_hash NULL (ADR-0141
-- M4/M10/M12, in-situ edit del baseline GREENFIELD, auditoría retroactiva
-- 2026-07):
--   - `permission_decisions` gana los mismos triggers `BEFORE UPDATE`/
--     `BEFORE DELETE` que `audit_events` (0002) y `job_results` (0003) --
--     hasta ahora solo la disciplina del repositorio (sin métodos
--     update/delete) lo protegía; el borrado/edición directo por SQL no
--     estaba bloqueado a nivel de base de datos.
--   - `audit_chain_hash` deja de ser `NOT NULL` con el sentinel de texto
--     `"genesis"` para la primera fila -- pasa a `NULL` en la fila génesis,
--     igual que el resto de las tablas append-only del sistema (anomalía
--     A4 de la auditoría retroactiva; ADR-0141: "audit_chain_hash: NULL en
--     la fila génesis de TODAS las tablas. Sin sentinels 'genesis'.").

-- ─────────────────────────────────────────────────────────────────────────────
-- Tabla de decisiones de permiso (append-only, forense)
-- ─────────────────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS permission_decisions (
    -- Grupo I: Identidad & Integridad (universal, ADR-0020)
    id                          TEXT    PRIMARY KEY NOT NULL, -- UUID v4
    created_at                  INTEGER NOT NULL,             -- Nanosegundos desde epoch
    updated_at                  INTEGER NOT NULL,             -- = created_at (append-only)
    audit_hash                  TEXT    NOT NULL,             -- SHA-256 de campos de dominio propio
    audit_chain_hash            TEXT,                         -- audit_hash de la fila anterior; NULL SOLO en la fila génesis (ADR-0141 M10)
    event_sequence_id           INTEGER NOT NULL UNIQUE,      -- Posición monótona (1, 2, 3, …)

    -- Grupo II: Soberanía
    owner_id                    TEXT,                         -- Propietario del interruptor (nullable en local)
    institutional_tag           TEXT,                         -- "Live" / "Demo" (solo para Manage)

    -- Grupo IV: Hardware / Infraestructura
    node_id                     TEXT    NOT NULL,             -- Host donde corre el Gateway MCP
    process_id                  INTEGER NOT NULL,             -- PID del proceso

    -- Dominio propio (fuera del catálogo canónico — spec agentic-mcp-gateway.md)
    agent_session_id            TEXT    NOT NULL,             -- Sesión MCP del agente
    requested_scope             TEXT    NOT NULL,             -- Pipeline/frontera invocada
    permission_outcome          TEXT    NOT NULL,             -- "granted" | "denied:<razón>"
    production_override_active  INTEGER NOT NULL DEFAULT 0    -- Estado del interruptor (0/1)
        CHECK (production_override_active IN (0, 1)),

    -- FK física owner_id -> accounts(id) (ADR-0141 enmienda 2026-07-11, M6):
    -- nullable -- ejemplo explícito citado por el propio ADR ("el
    -- interruptor de mcp_gateway"). Nota de orden: esta migración (0005) se
    -- aplica ANTES que `0007_central_identity.sql` (crea `accounts`);
    -- SQLite permite la referencia hacia adelante bajo `foreign_keys=ON`
    -- (verificado -- ver nota equivalente en `0002_audit_log.sql`).
    FOREIGN KEY (owner_id) REFERENCES accounts (id) ON DELETE RESTRICT
) STRICT;

-- Índice para auditoría por sesión de agente.
CREATE INDEX IF NOT EXISTS idx_permission_decisions_session
    ON permission_decisions (agent_session_id, created_at);

-- Índice de FK-hijo (ADR-0141 M7): toda columna owner_id con FK requiere su
-- propio índice, aunque sea nullable.
CREATE INDEX IF NOT EXISTS idx_permission_decisions_owner_id
    ON permission_decisions (owner_id);

-- Índice para verificación rápida de la cadena (tail lookup).
CREATE INDEX IF NOT EXISTS idx_permission_decisions_sequence
    ON permission_decisions (event_sequence_id);

-- Enforzamiento append-only: rechaza UPDATE (mismo patrón que
-- `audit_events`/`job_results`, 0002/0003 -- una decisión de permiso
-- registrada es un hecho forense permanente).
CREATE TRIGGER IF NOT EXISTS trg_permission_decisions_no_update
BEFORE UPDATE ON permission_decisions
BEGIN
    SELECT RAISE(ABORT, 'permission_decisions is append-only: UPDATE is forbidden');
END;

-- Enforzamiento append-only: rechaza DELETE.
CREATE TRIGGER IF NOT EXISTS trg_permission_decisions_no_delete
BEFORE DELETE ON permission_decisions
BEGIN
    SELECT RAISE(ABORT, 'permission_decisions is append-only: DELETE is forbidden');
END;

-- ─────────────────────────────────────────────────────────────────────────────
-- Tabla de configuración del Gateway MCP (estado mutable)
-- ─────────────────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS mcp_gateway_config (
    key     TEXT PRIMARY KEY NOT NULL,
    value   TEXT NOT NULL
) STRICT;

-- Valor inicial del interruptor de producción: desactivado por defecto (ADR-0123).
-- INSERT OR IGNORE: si ya existe no lo sobreescribe — idempotente.
INSERT OR IGNORE INTO mcp_gateway_config (key, value)
    VALUES ('production_override_active', '0');
