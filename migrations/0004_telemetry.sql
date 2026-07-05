-- Migración 0004: Telemetría (docs/features/telemetry.md TTR-001, ADR-0015,
-- ADR-0020)
--
-- Crea la tabla `telemetry_samples`: muestras de latencia de hot-path y de
-- señal de vida (heartbeat) de los procesos en segundo plano (ADR-0015:
-- la telemetría es evidencia de infraestructura, distinta del audit-log de
-- negocio).
--
-- Perfil de campos ADR-0020 para esta tabla (perfil técnico, igual que
-- declara la tabla "Persistencia" de la Feature):
--   - Grupo I  (Identidad & Integridad, UNIVERSAL): id, created_at,
--     updated_at, audit_hash, audit_chain_hash, event_sequence_id.
--   - Grupo II (Soberanía & Propiedad): institutional_tag.
--   - Grupo III (Pesos/Arquitectura): logic_hash, session_id.
--   - Grupo IV (Infraestructura & Ops / "Hardware"): node_id, process_id,
--     execution_latency_ms.
-- Los Grupos restantes (Soberanía completa más allá de institutional_tag,
-- Linaje Alpha más allá de logic_hash, Forense & Ejecución) NO aplican al
-- perfil de esta tabla y se omiten a propósito (ADR-0020 "Aplicación":
-- "PROHIBIDO copy-paste masivo").
--
-- Columnas propias de la Feature (fuera del contrato de 25 campos, mismo
-- patrón que action_type/entity_type/entity_id/details_json en
-- audit_events):
--   - metric_name: qué se midió (ej. "ingest.hot_path_latency",
--     "job_executor.heartbeat").
--   - details_json: contexto extra opcional, JSON-encoded.
--
-- A diferencia de `audit_events`/`job_results`, esta tabla NO es
-- append-only: la "PODA AUTOMÁTICA" (docs/features/telemetry.md
-- "Restricciones") borra filas más viejas que el corte de retención a
-- propósito. No se instalan triggers que bloqueen DELETE.
--
-- Idempotencia (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF
-- NOT EXISTS` hacen que volver a correr esta migración sea un no-op.

CREATE TABLE IF NOT EXISTS telemetry_samples (
    -- I. Identidad & Integridad (universal, ADR-0020)
    id                 TEXT    NOT NULL PRIMARY KEY, -- UUID
    created_at         INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock)
    updated_at         INTEGER NOT NULL,             -- Nanosegundos desde epoch; inmutable tras insertar => igual a created_at
    audit_hash         TEXT    NOT NULL,             -- SHA-256 del contenido de esta muestra + enlace previo (cadena en memoria del proceso)
    audit_chain_hash   TEXT,                         -- audit_hash de la muestra anterior (NULL solo para la primera muestra de la cadena)
    event_sequence_id  INTEGER NOT NULL UNIQUE,      -- Posición monótona en la cadena (1, 2, 3, ...)

    -- II. Soberanía
    institutional_tag  TEXT    NOT NULL,             -- Entorno (BACKTEST/PAPER/LIVE/...)

    -- III. Pesos/Arquitectura
    logic_hash         TEXT,                         -- Versión del emisor de telemetría (nullable)
    session_id         TEXT,                         -- Sesión global vinculada (nullable)

    -- IV. Infraestructura / Hardware
    node_id            TEXT,                         -- Host físico monitorizado (nullable)
    process_id         TEXT    NOT NULL,             -- PID del proceso muestreado
    execution_latency_ms INTEGER,                     -- Latencia en ms; NULL en heartbeats, obligatorio en muestras de latencia

    -- Columnas propias de la Feature (telemetry.md "Persistencia")
    metric_name        TEXT    NOT NULL,             -- Qué se midió, ej. "ingest.hot_path_latency"
    details_json       TEXT                          -- Contexto extra opcional (JSON-encoded, nullable)
);

-- Acceso por serie temporal (telemetry.md "Comportamientos Observables":
-- "permite consultar series temporales de performance técnica").
CREATE INDEX IF NOT EXISTS idx_telemetry_samples_metric_created
    ON telemetry_samples (metric_name, created_at);

-- Acceso para la poda (`DELETE FROM telemetry_samples WHERE created_at < ?`,
-- telemetry.md "Restricciones": "PODA AUTOMÁTICA") -- columna líder distinta
-- del índice compuesto de arriba, porque la poda filtra solo por
-- created_at, sin metric_name.
CREATE INDEX IF NOT EXISTS idx_telemetry_samples_created_at
    ON telemetry_samples (created_at);
