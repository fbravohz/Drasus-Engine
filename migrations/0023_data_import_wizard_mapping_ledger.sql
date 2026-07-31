-- Migration 0023: Data Import Wizard — ledger de declaración de mapeo
-- (docs/features/data-import-wizard.md, ADR-0156, ADR-0155, ADR-0162,
--  ADR-0141, ADR-0020 Perfil A, LDR-0008, STORY-068, STORY-070)
--
-- Crea `import_mapping_declarations`: el registro inmutable de qué declaró
-- el usuario (o qué plantilla de plataforma aplicó Drasus) sobre un archivo
-- concreto de importación, sellado ANTES de que exista la primera fila de
-- dato transformado (ADR-0156 Restricción 7, LDR-0008 condición 2).
--
-- Retrofit STORY-070 (2026-07-26, edición in-situ -- fase GREENFIELD, ADR-0006
-- enmienda 2026-06-28): ADR-0162 "Enmienda 2026-07-26" (TASK-069) cambia el
-- grano de esta declaración de "un archivo" a "un tramo declarado dentro de
-- un archivo" -- un archivo de 25 años no es homogéneo. Cuatro cambios de
-- esquema (ver columnas marcadas abajo): (1) rango declarado explícito
-- (`declared_range_start_ns`/`declared_range_end_ns`); (2) la unicidad de
-- idempotencia se relaja para incluir el rango, no solo el archivo; (3)
-- `source_timezone_declared` se sustituye por el par ancla+desplazamiento
-- (`source_clock_anchor_timezone_ref`/`source_clock_fixed_offset_minutes`);
-- (4) `real_volume_unit_declared`/`spread_reported_unit_declared` sin cambio
-- -- ya vivían en el grano correcto (por fila = por tramo). Las únicas filas
-- que existían en este ledger eran de prueba -- no hay dato real que
-- reinterpretar ni migrar (fase GREENFIELD, se corrige el baseline in-situ).
--
-- Perfil ADR-0020: Perfil A (Datos/Ingest).
--   - Grupo I   (universal, append-only): id, created_at, updated_at,
--     audit_hash, audit_chain_hash, event_sequence_id UNIQUE (ADR-0141
--     "Semántica de secuencias" -- esta tabla es event-store append-only,
--     NUNCA row_version).
--   - Grupo IV  (Infraestructura & Ops): node_id, process_id.
--   - Campos propios de dominio: ver abajo, mínimos exigidos por ADR-0156
--     Restricción 7 + los tres campos que LDR-0008 añade a esa lista.
--
-- NO lleva `data_snapshot_id`: una sola declaración de mapeo puede cubrir
-- muchas particiones Parquet (un archivo de 25 años produce ~300 particiones
-- mensuales) -- la relación es la inversa (ADR-0156 Restricción 8):
-- `transformation_records.data_snapshot_id` referencia (por convención de
-- aplicación, no FK física -- vive en otra feature) el `id` de la fila de
-- este ledger que originó esa corrida. Cuatro saltos por referencia, nunca
-- datos duplicados.
--
-- Idempotencia (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF
-- NOT EXISTS` / `CREATE TRIGGER IF NOT EXISTS` -- volver a correr esta
-- migración es un no-op.
--
-- Enforzamiento append-only (molde `.agents/knowledge/ledger-pattern.md`
-- punto 5, referencia de ESTE punto concreto: `0019_data_portability.sql`):
-- la inmutabilidad NO depende de que el repositorio nunca emita un
-- UPDATE/DELETE -- eso es una convención, no una garantía. Los triggers
-- `trg_import_mapping_declarations_no_update`/`_no_delete` (al final de este
-- archivo) hacen que la propia base de datos rechace cualquier intento de
-- modificar o borrar una fila ya sellada.
--
-- audit_hash / audit_chain_hash: `audit_hash` es el hash SHA-256 del
-- CONTENIDO de la declaración (rango declarado + mapeo + plantilla + vista
-- previa + reloj de origen + versión de esquema), calculado en el Núcleo puro
-- (`domain::compute_mapping_declaration_audit_hash`) -- excluye el propio
-- Grupo I para evitar circularidad. `audit_chain_hash` = audit_hash de la
-- fila anterior; NULL en la fila génesis (ADR-0141 M10, sin sentinel
-- "genesis").

CREATE TABLE IF NOT EXISTS import_mapping_declarations (
    -- ── Grupo I: Identidad & Integridad (universal, ADR-0020) ────────────
    id                            TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at                    INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock)
    updated_at                    INTEGER NOT NULL,             -- Igual a created_at (registro inmutable)
    audit_hash                    TEXT    NOT NULL,             -- SHA-256 del contenido de la declaración
    audit_chain_hash              TEXT,                         -- audit_hash de la fila anterior; NULL en la fila génesis
    event_sequence_id             INTEGER NOT NULL UNIQUE,      -- Posición monótona en la cadena de declaraciones

    -- ── Grupo IV: Infraestructura & Ops ──────────────────────────────────
    node_id                       TEXT,                         -- Huella del hardware que selló la declaración
    process_id                   TEXT,                         -- PID del proceso que la selló

    -- ── Identidad del archivo origen (ADR-0156 Restricción 7, campo 1) ───
    source_file_name              TEXT    NOT NULL,             -- Nombre del archivo tal como lo entregó el usuario
    source_file_hash               TEXT    NOT NULL,            -- SHA-256 del contenido crudo del archivo -- clave de idempotencia (ADR-0163)

    -- ── Identidad de la serie que esta declaración alimenta ──────────────
    -- La misma cadena de negocio que ya circula en QuoteTick/TradeTick/Bars
    -- (ADR-0155 Restricción 8) -- FK física legítima a nivel de esquema SQL
    -- (no crea dependencia de crate Rust hacia `instrument-catalog`; ADR-0137
    -- prohíbe el acceso a tablas ajenas desde CÓDIGO, no una FK declarativa).
    instrument_id                 TEXT    NOT NULL
        REFERENCES instrument_master (instrument_id) ON DELETE RESTRICT,
    venue                          TEXT    NOT NULL,            -- Bróker/venue donde se opera este instrumento
    bar_type                        TEXT    NOT NULL DEFAULT 'TIME'
        CHECK (bar_type IN ('TIME','TICK_COUNT','VOLUME','DOLLAR','RENKO','RANGE')), -- ADR-0155 catálogo de Bars
    timeframe_value                  TEXT    NOT NULL,          -- Ej. "1M" -- fijo para toda la corrida de este archivo

    -- ── Rango declarado del tramo (retrofit STORY-070, ADR-0162 "Enmienda ──
    -- 2026-07-26", cambio de esquema 1) -- el rango que ESTA fila declara
    -- cubrir, EXPLÍCITO, nunca implícito en las filas resultantes que este
    -- lector produce. Intervalo semiabierto [inicio, fin), misma convención
    -- que `Bars` (ADR-0155). Es la declaración del usuario sobre el tramo,
    -- no un cálculo derivado de escanear el mínimo/máximo de las filas
    -- parseadas -- dos declaraciones sobre el mismo archivo pueden traer
    -- rangos que no coinciden exactamente con lo que el archivo contiene.
    declared_range_start_ns           INTEGER NOT NULL,          -- Nanosegundos UTC, inclusive
    declared_range_end_ns              INTEGER NOT NULL,         -- Nanosegundos UTC, exclusive
    -- El CHECK de orden del rango vive al FINAL de la tabla, junto al UNIQUE
    -- -- SQLite exige que toda restricción de tabla (sin nombre de columna)
    -- venga DESPUÉS de todas las columnas, nunca intercalada entre ellas.

    -- ── Mapeo declarado (ADR-0156 Restricción 7, campos 2-4) ─────────────
    mapping_origin                    TEXT    NOT NULL
        CHECK (mapping_origin IN ('PLATFORM_TEMPLATE','MANUAL_USER')), -- LDR-0008: decide quién responde por un mapeo erróneo
    platform_template_code             TEXT,                   -- Ej. "MT5_M1_BAR_V1" -- NULL si mapping_origin = MANUAL_USER
    platform_template_version           TEXT,                  -- Versión de la plantilla aplicada -- NULL si MANUAL_USER
    column_mapping_json                  TEXT    NOT NULL
        CHECK (json_valid(column_mapping_json)), -- Mapeo completo columna origen -> campo canónico, tal como se mostró al usuario

    -- ── Versión de motor y esquema (ADR-0156 Restricción 7 campo 5, LDR-0008 condición técnica 2) ──
    transform_engine_version              TEXT    NOT NULL,     -- Versión del binario/motor que ejecutó el mapeo
    canonical_schema_version               TEXT    NOT NULL,    -- Versión del esquema canónico (ADR-0155) vigente al declarar

    -- ── Aceptación y evidencia de vista previa (ADR-0156 R7 campo 6, LDR-0008 condición técnica 3) ──
    accepted_at_ns                          INTEGER NOT NULL,   -- Nanosegundos UTC del instante de aceptación
    preview_transformed_rows_json            TEXT    NOT NULL
        CHECK (json_valid(preview_transformed_rows_json)), -- Filas de DATO YA TRANSFORMADO (nunca solo esquema) que el usuario vio al aceptar

    -- ── Reloj de origen declarado (retrofit STORY-070, ADR-0162 "Enmienda ──
    -- 2026-07-26", cambio de esquema 3) -- sustituye `source_timezone_declared`
    -- por el par ancla+desplazamiento. NUNCA se infiere de volatilidad ni de
    -- noticia (ADR-0162 Restricción 7/9). NOT NULL a propósito: una
    -- importación sin esta declaración se RECHAZA antes de llegar a este
    -- ledger (criterio de aceptación #3 de STORY-068) -- ninguna fila de
    -- esta tabla representa jamás un reloj asumido. Desplazamiento `0`
    -- reproduce sin pérdida la declaración plana de zona de hoy -- no son
    -- dos mecanismos, uno es el caso particular del otro.
    source_clock_anchor_timezone_ref          TEXT    NOT NULL, -- Identificador IANA elegido por su calendario de DST (ej. "America/New_York")
    source_clock_fixed_offset_minutes          INTEGER NOT NULL DEFAULT 0, -- Constante con signo sumada al offset propio del ancla

    -- ── Convención de unidad del volumen real declarada para ESTA corrida ──
    -- Espejo de `broker_instrument_terms.real_volume_unit_declared`
    -- (ADR-0162 enmienda 2026-07-25) -- se sella aquí también porque es la
    -- evidencia de qué sabía el sistema al momento de ESTA importación
    -- concreta, no solo el estado actual (mutable) del catálogo.
    real_volume_unit_declared                 TEXT
        CHECK (real_volume_unit_declared IS NULL OR real_volume_unit_declared IN
            ('CONTRACTS','LOTS','BASE_ASSET_UNITS','QUOTE_ASSET_UNITS')),

    -- ── Unidad del spread reportado declarada por la PLANTILLA (retrofit ──
    -- STORY-068 criterio de aceptación #13, observación del propietario
    -- 2026-07-25: "10 en EURUSD, en USDJPY, en WTI y en el S&P 500 son
    -- cuatro magnitudes distintas"). A diferencia de
    -- `real_volume_unit_declared` (varía por bróker, por eso es nullable),
    -- esta unidad es un hecho FIJO de la plantilla de plataforma -- MT5
    -- siempre reporta `<SPREAD>` en puntos, nunca en pips. NOT NULL a
    -- propósito: una importación cuya plantilla no declare esta unidad se
    -- RECHAZA antes de llegar a este ledger, mismo trato que
    -- `source_clock_anchor_timezone_ref`. Vocabulario abierto (`TEXT` sin `CHECK`
    -- de enum): la unidad exacta depende de la plantilla de plataforma, y
    -- hoy solo existe una (`POINTS_MT5`) -- fijar un `CHECK` cerrado sería
    -- alucinar el vocabulario de plantillas futuras sin fuente verificada.
    spread_reported_unit_declared              TEXT    NOT NULL,

    -- Un mismo TRAMO (archivo por contenido + rango declarado) no genera dos
    -- declaraciones distintas -- reimportar el mismo tramo idéntico es un
    -- no-op (ADR-0163, criterio de aceptación #10 de STORY-068). Relajado en
    -- STORY-070 (ADR-0162 "Enmienda 2026-07-26", cambio de esquema 2): antes
    -- era `UNIQUE (source_file_hash)` solo -- eso rechazaría por error la
    -- segunda declaración de un archivo genuinamente heterogéneo, donde dos
    -- tramos disjuntos del MISMO archivo son declaraciones legítimas y
    -- distintas. Cinturón y tirantes: la aplicación ya comprueba esto antes
    -- de insertar; el UNIQUE es el respaldo a nivel de base de datos bajo
    -- concurrencia.
    UNIQUE (source_file_hash, declared_range_start_ns, declared_range_end_ns),
    -- Rango invertido o vacío (fin <= inicio) se rechaza a nivel de base de
    -- datos -- criterio de aceptación #6 de STORY-070.
    CHECK (declared_range_end_ns > declared_range_start_ns)
) STRICT;

-- Índices de FK hijo e idempotencia (ADR-0141 M7/M8).
CREATE INDEX IF NOT EXISTS idx_import_mapping_declarations_instrument_id
    ON import_mapping_declarations (instrument_id);
CREATE INDEX IF NOT EXISTS idx_import_mapping_declarations_event_sequence_id
    ON import_mapping_declarations (event_sequence_id);
CREATE INDEX IF NOT EXISTS idx_import_mapping_declarations_node_id
    ON import_mapping_declarations (node_id);

-- Enforzamiento append-only a nivel de base de datos (ledger-pattern.md
-- punto 5) -- ninguna fila de este ledger se modifica ni se borra jamás.
CREATE TRIGGER IF NOT EXISTS trg_import_mapping_declarations_no_update
BEFORE UPDATE ON import_mapping_declarations
BEGIN
    SELECT RAISE(ABORT, 'import_mapping_declarations is append-only: UPDATE is forbidden');
END;

CREATE TRIGGER IF NOT EXISTS trg_import_mapping_declarations_no_delete
BEFORE DELETE ON import_mapping_declarations
BEGIN
    SELECT RAISE(ABORT, 'import_mapping_declarations is append-only: DELETE is forbidden');
END;
