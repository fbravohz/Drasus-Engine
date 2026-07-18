-- Migration 0021: Hybrid Data Transformer — registro de transformaciones
-- (docs/features/hybrid-data-transformer.md, ADR-0105, ADR-0141, ADR-0020 Perfil A)
--
-- Crea la tabla `transformation_records`: el registro inmutable de cada
-- ejecución del motor de transformación híbrido (Polars + puente ndarray).
--
-- Perfil ADR-0020: Perfil A (Datos de Mercado).
--   - Grupo I   (universal): id, created_at, updated_at, audit_hash,
--     audit_chain_hash, event_sequence_id.
--   - Grupo III (Linaje Alpha & Datos): data_snapshot_id, transformation_id,
--     logic_hash, parent_id.
--   - Grupo IV  (Infraestructura & Ops): node_id, process_id, session_id.
--   - Campos propios de dominio: transform_duration_ns, rows_in, rows_out,
--     schema_version (fuera del catálogo de 25, justificado en
--     docs/features/hybrid-data-transformer.md "Persistencia" — sustituyen a
--     execution_latency_ms del Grupo V, no autorizado en Perfil A).
-- OMITIDO: Grupo II (Soberanía — sin credenciales, esta feature no toca
-- brokers ni secretos), Grupo V (Forense — `transform_duration_ns` es el
-- campo propio equivalente, ya declarado arriba).
--
-- Idempotencia (ADR-0006): CREATE TABLE IF NOT EXISTS + índices IF NOT
-- EXISTS — volver a correr esta migración es un no-op.
--
-- audit_hash / audit_chain_hash (semántica propia de esta tabla, distinta de
-- sovereign_download_records): `audit_hash` es el hash del CONTENIDO
-- resultante (las barras ya ordenadas, calculado en el Núcleo puro sobre la
-- representación determinista — criterio 5, reproducible entre corridas
-- distintas sin importar el número de hilos de Polars usados). Encadenado
-- vía `audit_chain_hash` = audit_hash de la fila anterior (NULL en la fila
-- génesis, ADR-0141 M10) — la cadena de hashes protege la integridad del
-- pipeline ETL completo, no solo de esta fila individual.

CREATE TABLE IF NOT EXISTS transformation_records (
    -- ── Grupo I: Identidad & Integridad (universal, ADR-0020) ─────────
    id                     TEXT    NOT NULL PRIMARY KEY,  -- UUIDv7 único del registro de transformación
    created_at             INTEGER NOT NULL,              -- Nanosegundos desde epoch (puerto Clock)
    updated_at             INTEGER NOT NULL,              -- Nanosegundos desde epoch; igual a created_at (registro inmutable)
    audit_hash             TEXT    NOT NULL,              -- SHA-256 del DataFrame resultante (barras ya ordenadas)
    audit_chain_hash       TEXT,                          -- audit_hash de la fila anterior; NULL para el primer registro
    event_sequence_id      INTEGER NOT NULL UNIQUE,       -- Posición monótona en la cadena global de transformaciones

    -- ── Grupo III: Linaje Alpha & Datos ──────────────────────────────────
    data_snapshot_id       TEXT                           -- Referencia al dataset crudo de entrada
                                                           -- Formato canónico (ADR-0141): <exchange>_<symbol>_<timeframe>_<year><month>, ej. "binance_BTCUSDT_1m_202601"
        CHECK (data_snapshot_id IS NULL OR data_snapshot_id GLOB '*_*_*_*'),
    transformation_id      TEXT    NOT NULL,              -- ID del paso de transformación dentro del pipeline (ej. "OHLCV_AGGREGATION")
    logic_hash             TEXT    NOT NULL,              -- Hash reproducible del plan de transformación (intervalo destino + umbral)
    parent_id              TEXT,                          -- Transformación previa encadenada, cuando el pipeline tiene varios pasos
                                                           -- Referencia suave (sin FK física): igual criterio que account_hierarchy.parent_owner_id
                                                           -- (docs/DATA-MODEL.md "Referencias suaves") — un pipeline multi-paso encadena
                                                           -- transformaciones entre sí, no cuentas.

    -- ── Grupo IV: Infraestructura & Ops ──────────────────────────────────
    node_id                TEXT,                          -- Huella del hardware donde corrió la transformación
    process_id             TEXT,                          -- PID del proceso que ejecutó la transformación
    session_id             TEXT,                          -- Sesión de ingesta que originó esta transformación

    -- ── Campos propios de dominio ─────────────────────────────────────────
    transform_duration_ns  INTEGER NOT NULL,              -- Duración real del proceso (nanosegundos de pared)
    rows_in                INTEGER NOT NULL,              -- Filas de entrada (ticks recibidos)
    rows_out               INTEGER NOT NULL,              -- Filas de salida (barras producidas)
    schema_version         TEXT    NOT NULL               -- Versión semver del esquema de salida (ADR-0141 F3, ej. "1.0")
) STRICT;

-- Acceso por secuencia (para la cadena de hashes y la recuperación ordenada).
CREATE INDEX IF NOT EXISTS idx_transformation_records_event_sequence_id
    ON transformation_records (event_sequence_id);

-- Acceso por node_id (para auditar qué nodo ejecutó qué transformación).
CREATE INDEX IF NOT EXISTS idx_transformation_records_node_id
    ON transformation_records (node_id);

-- Acceso por parent_id (para recorrer la cadena de un pipeline multi-paso).
CREATE INDEX IF NOT EXISTS idx_transformation_records_parent_id
    ON transformation_records (parent_id);
