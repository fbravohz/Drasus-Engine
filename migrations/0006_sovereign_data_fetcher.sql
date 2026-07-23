-- Migration 0006: Sovereign Data Fetcher — registro de descargas
-- (docs/features/sovereign-data-fetcher.md, ADR-0034, ADR-0020 Perfil A)
--
-- Crea la tabla `sovereign_download_records`: el registro inmutable de cada
-- descarga ejecutada por el fetcher híbrido (Bulk + Delta REST).
--
-- Perfil ADR-0020: Perfil A (Datos de Mercado).
--   - Grupo I  (universal): id, created_at, updated_at, audit_hash,
--     audit_chain_hash, event_sequence_id.
--   - Grupo III (Linaje Alpha & Datos): data_snapshot_id, logic_hash.
--   - Grupo IV  (Infraestructura & Ops): node_id, process_id.
--   - Campo propio de dominio: source_endpoint (provenance de la fuente;
--     fuera del catálogo de 25, justificado por soberanía de datos).
-- OMITIDOS: Grupo II (Soberanía — sin credenciales en esta Story, datos
-- públicos), Grupo V (Forense — execution_latency_ms lo lleva el Job de
-- async-job-executor, no este registro).
--
-- Idempotencia (ADR-0006): CREATE TABLE IF NOT EXISTS + índices IF NOT
-- EXISTS + triggers IF NOT EXISTS — volver a correr esta migración es un
-- no-op.
--
-- Candados append-only (DEBT-032, retrofit 2026-07-19, patrón canónico
-- `.agents/knowledge/ledger-pattern.md` punto 5, referencia limpia
-- `0019_data_portability.sql`): esta tabla nació sin los triggers que
-- rechazan UPDATE/DELETE a nivel de base de datos -- su inmutabilidad
-- dependía solo de que el repositorio nunca emitiera esas sentencias, un
-- acuerdo de buena voluntad, no una garantía. Se añaden aquí in-situ
-- (fase GREENFIELD, ADR-0006).
--
-- STRICT mode + UNIQUE + CHECK de formato (ADR-0141 M6/M9/M11/M12, in-situ
-- edit del baseline GREENFIELD, auditoría retroactiva 2026-07):
--   - `event_sequence_id` gana `UNIQUE`: esta tabla es append-only (cada
--     descarga es un hecho histórico), así que su secuencia debe ser única
--     igual que en `audit_events`/`job_results`/`telemetry_samples` -- antes
--     solo tenía índice, sin la restricción de unicidad real.
--   - `data_snapshot_id` (cuando no es NULL) valida el formato canónico de
--     ADR-0141: `<exchange>_<symbol>_<timeframe>_<year><month>` (ejemplo:
--     `binance_BTCUSDT_1m_202601`) -- el CHECK exige al menos los 4
--     segmentos separados por `_`; no valida cada segmento individualmente
--     (eso lo hace el reconciler de Parquet en el módulo Ingest).

CREATE TABLE IF NOT EXISTS sovereign_download_records (
    -- ── Grupo I: Identidad & Integridad (universal, ADR-0020) ─────────
    id                TEXT    NOT NULL PRIMARY KEY,   -- UUID único del registro de descarga
    created_at        INTEGER NOT NULL,               -- Nanosegundos desde epoch (puerto Clock)
    updated_at        INTEGER NOT NULL,               -- Nanosegundos desde epoch; igual a created_at (registro inmutable)
    audit_hash        TEXT    NOT NULL,               -- SHA-256 del contenido de la fila (snapshot de integridad)
    audit_chain_hash  TEXT,                           -- audit_hash del registro previo; NULL para el primer registro
    event_sequence_id INTEGER NOT NULL UNIQUE,        -- Posición monótona en la cadena global de registros de descarga

    -- ── Grupo III: Linaje Alpha & Datos ──────────────────────────────────
    data_snapshot_id  TEXT                            -- Referencia al volcado/snapshot del broker que originó el segmento
                                                       -- Formato canónico (ADR-0141): <exchange>_<symbol>_<timeframe>_<year><month>, ej. "binance_BTCUSDT_1m_202601"
        CHECK (data_snapshot_id IS NULL OR data_snapshot_id GLOB '*_*_*_*'),
    logic_hash        TEXT,                           -- Hash del driver del fetcher que produjo este registro (versión del ejecutor)

    -- ── Grupo IV: Infraestructura & Ops ──────────────────────────────────
    node_id           TEXT,                           -- Huella del hardware donde se ejecutó la descarga
    process_id        TEXT,                           -- PID del worker de descarga

    -- ── Campo propio de dominio (provenance — soberanía de datos) ────────
    source_endpoint   TEXT    NOT NULL                -- URL/endpoint exacto de la fuente Bulk o REST que sirvió el dato
) STRICT;

-- Acceso por secuencia (para la cadena de hashes y la recuperación ordenada).
CREATE INDEX IF NOT EXISTS idx_sovereign_download_records_event_sequence_id
    ON sovereign_download_records (event_sequence_id);

-- Acceso por node_id (para auditar qué nodo hizo qué descarga).
CREATE INDEX IF NOT EXISTS idx_sovereign_download_records_node_id
    ON sovereign_download_records (node_id);

-- Enforzamiento append-only: rechaza UPDATE (DEBT-032 -- cada registro de
-- descarga es un hecho histórico permanente; si un registro está mal, se
-- registra una descarga nueva, nunca se corrige in-place).
CREATE TRIGGER IF NOT EXISTS trg_sovereign_download_records_no_update
BEFORE UPDATE ON sovereign_download_records
BEGIN
    SELECT RAISE(ABORT, 'sovereign_download_records is append-only: UPDATE is forbidden');
END;

-- Enforzamiento append-only: rechaza DELETE.
CREATE TRIGGER IF NOT EXISTS trg_sovereign_download_records_no_delete
BEFORE DELETE ON sovereign_download_records
BEGIN
    SELECT RAISE(ABORT, 'sovereign_download_records is append-only: DELETE is forbidden');
END;
