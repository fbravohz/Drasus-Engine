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

-- ─────────────────────────────────────────────────────────────────────────────
-- Tabla de decisiones de permiso (append-only, forense)
-- ─────────────────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS permission_decisions (
    -- Grupo I: Identidad & Integridad (universal, ADR-0020)
    id                          TEXT    PRIMARY KEY NOT NULL, -- UUID v4
    created_at                  INTEGER NOT NULL,             -- Nanosegundos desde epoch
    updated_at                  INTEGER NOT NULL,             -- = created_at (append-only)
    audit_hash                  TEXT    NOT NULL,             -- SHA-256 de campos de dominio propio
    audit_chain_hash            TEXT    NOT NULL,             -- audit_hash de la fila anterior (o "genesis")
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
);

-- Índice para auditoría por sesión de agente.
CREATE INDEX IF NOT EXISTS idx_permission_decisions_session
    ON permission_decisions (agent_session_id, created_at);

-- Índice para verificación rápida de la cadena (tail lookup).
CREATE INDEX IF NOT EXISTS idx_permission_decisions_sequence
    ON permission_decisions (event_sequence_id);

-- ─────────────────────────────────────────────────────────────────────────────
-- Tabla de configuración del Gateway MCP (estado mutable)
-- ─────────────────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS mcp_gateway_config (
    key     TEXT PRIMARY KEY NOT NULL,
    value   TEXT NOT NULL
);

-- Valor inicial del interruptor de producción: desactivado por defecto (ADR-0123).
-- INSERT OR IGNORE: si ya existe no lo sobreescribe — idempotente.
INSERT OR IGNORE INTO mcp_gateway_config (key, value)
    VALUES ('production_override_active', '0');
