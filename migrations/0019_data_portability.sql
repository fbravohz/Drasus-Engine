-- Migración 0019: Data Portability / Portabilidad de Datos
-- (docs/features/verified-account-registry.md NO aplica -- ver
-- docs/execution/STORY-043-data-portability.md, ADR-0148 -- cimiento #13,
-- ADR-0141, ADR-0093, ADR-0020, ADR-0137, STORY-043)
--
-- Crea DOS tablas del decimotercer cimiento: el catálogo declarativo de qué
-- tablas del substrato portan `owner_id` (metadato de esquema, análogo a
-- `foundation_master_fields` de `0001`) y el registro append-only de
-- solicitudes de acceso/portabilidad/olvido (GDPR Art. 15/17/20).
--
-- 1) `exportable_data_catalog` -- MUTABLE (una tabla puede reclasificarse:
--    hoy no exenta de retención, mañana sí, si una obligación legal nueva
--    la alcanza). Por ADR-0141, usa `row_version` (contador de versión por
--    fila, arranca en 1, +1 en cada UPDATE) -- mismo patrón que `plans`
--    (`0009_plan_tier_quota.sql`). NO tiene columnas Grupo II
--    (`owner_id`/`institutional_tag`): es metadato de ESQUEMA, no un hecho
--    ligado a un dueño concreto -- mismo espíritu que
--    `foundation_master_fields` (`0001`).
--
-- 2) `data_portability_requests` -- APPEND-ONLY ATÓMICA (cada solicitud de
--    exportación/olvido, y cada avance de su estado, es un hecho histórico
--    permanente -- nunca se corrige in-place). Usa
--    `event_sequence_id INTEGER NOT NULL UNIQUE` (posición monótona
--    GLOBAL) -- mismo patrón que `override_attestations`
--    (`0018_master_account_hierarchy.sql`). El avance de estado
--    (RECEIVED -> PROCESSING -> COMPLETED) se modela como eventos NUEVOS
--    que comparten el mismo `request_group_id`, nunca como un UPDATE de la
--    fila anterior -- el estado vigente es el del evento más reciente por
--    `request_group_id` (ver `DataPortabilityRequestRepository::latest_status_for`).
--
--    Regla "Atomicidad de ledgers append-only" (rust-engineer/SKILL.md §4,
--    causa raíz DEBT-001): el `event_sequence_id` se deriva DENTRO de una
--    transacción `BEGIN IMMEDIATE` (ver
--    `persistence::data_portability::DataPortabilityRequestRepository`),
--    nunca en sentencias sueltas -- el `UNIQUE` de abajo es cinturón-y-
--    tirantes, no el guardián primario.
--
-- Perfil ADR-0020 para `data_portability_requests` (Perfil D
-- "Ops/Auditoría", STORY-043 §3):
--   - Grupo I  (Identidad & Integridad, UNIVERSAL -- `event_sequence_id`
--     por ser APPEND-ONLY, ADR-0141).
--   - Grupo II (Soberanía & Propiedad): `owner_id` (el titular que pide su
--     export/olvido) + `institutional_tag`.
--   - Grupo IV (Infraestructura & Ops): `node_id` -- qué máquina registró
--     ESTE evento de la solicitud.
--   - Subset V (Forense & Ejecución): `compliance_status_id` -- estado de
--     cumplimiento vigente al momento de este evento (nullable).
-- El Grupo III (Linaje Alpha & Datos) NO aplica (una solicitud de
-- cumplimiento no tiene linaje genómico) y se omite a propósito.
--
-- Regla FIJA #3 (ADR-0148, STORY-043 §7): el olvido NUNCA hace DELETE
-- físico de una fila -- SIEMPRE pseudonimización (ADR-0141), incluso para
-- tablas sin retención. `disposition_detail` es el JSON que documenta, por
-- cada tabla del catálogo, si se pseudonimizó-y-retuvo (retención legal) o
-- pseudonimizó-y-purgó (sin retención) -- nunca "se borró".
--
-- Guardarraíl ADR-0093 (estructural): NINGUNA columna de ninguna de las dos
-- tablas puede contener una credencial de bróker, una clave de cifrado ni
-- una IP de servidor live -- el filtro de exclusión de secretos
-- (`domain::data_portability::is_excluded_from_export`) corre ANTES de que
-- cualquier tabla del catálogo entre al manifiesto de exportación.
--
-- Idempotencia (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF
-- NOT EXISTS` / `CREATE TRIGGER IF NOT EXISTS` hacen que volver a correr
-- esta migración sea un no-op.

CREATE TABLE IF NOT EXISTS exportable_data_catalog (
    -- I. Identidad & Integridad (universal, ADR-0020; row_version en vez de
    -- event_sequence_id por ser tabla MUTABLE, ADR-0141).
    id                TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at        INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock)
    updated_at        INTEGER NOT NULL,             -- Nanosegundos desde epoch; cambia con cada UPDATE
    audit_hash        TEXT    NOT NULL,             -- SHA-256 del contenido de esta versión de fila + enlace previo
    audit_chain_hash  TEXT,                         -- audit_hash de la versión anterior de esta fila (NULL solo en la fila génesis)
    row_version       INTEGER NOT NULL,             -- Contador de versión de esta fila; arranca en 1, +1 en cada UPDATE

    -- Columnas propias de la Feature (STORY-043 §3.1) -- metadato de
    -- ESQUEMA, no un hecho ligado a un dueño (sin columnas Grupo II).
    table_name        TEXT    NOT NULL UNIQUE,      -- Nombre de la tabla catalogada -- auto-declaración idempotente
    feature_name      TEXT    NOT NULL,             -- Feature dueña de esa tabla (docs/features/<feature_name>.md)
    owner_id_column   TEXT    NOT NULL,             -- Nombre de la columna owner_id EN esa tabla
    retention_exempt  INTEGER NOT NULL              -- 1 = obligación de retención legal (se pseudonimiza, NUNCA se purga el contenido)
        CHECK (retention_exempt IN (0, 1))
) STRICT;

CREATE TABLE IF NOT EXISTS data_portability_requests (
    -- I. Identidad & Integridad (universal, ADR-0020; event_sequence_id en
    -- vez de row_version por ser tabla APPEND-ONLY, ADR-0141).
    id                 TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at         INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock) -- instante de PERSISTENCIA de ESTE evento
    updated_at         INTEGER NOT NULL,             -- Nanosegundos desde epoch; append-only => siempre igual a created_at
    audit_hash         TEXT    NOT NULL,             -- SHA-256 del contenido de ESTA FILA + enlace previo (integridad del ledger)
    audit_chain_hash   TEXT,                         -- audit_hash de la fila anterior (NULL solo en la fila génesis)
    event_sequence_id  INTEGER NOT NULL UNIQUE,      -- Posición monótona en la cadena GLOBAL (1, 2, 3, ...)

    -- II. Soberanía & Propiedad
    owner_id           TEXT    NOT NULL,             -- El titular que pide su acceso/portabilidad/olvido
    institutional_tag  TEXT    NOT NULL,             -- Entorno/etiqueta institucional

    -- IV. Infraestructura & Ops
    node_id            TEXT    NOT NULL,             -- Máquina que registró ESTE evento de la solicitud

    -- Subset de V. Forense & Ejecución (Gobernanza/Cumplimiento)
    compliance_status_id TEXT,                       -- Estado de cumplimiento vigente al momento de ESTE evento (nullable)

    -- Columnas propias de la Feature (STORY-043 §3.2)
    request_type       TEXT    NOT NULL              -- EXPORT (Art. 15/20) | FORGET (Art. 17)
        CHECK (request_type IN ('EXPORT', 'FORGET')),
    status              TEXT    NOT NULL             -- Estado de ESTE evento -- el vigente es el del event_sequence_id más alto por request_group_id
        CHECK (status IN ('RECEIVED', 'PROCESSING', 'COMPLETED')),
    request_group_id    TEXT    NOT NULL,            -- Agrupa TODOS los eventos de UNA solicitud lógica (mismo id a través de RECEIVED->PROCESSING->COMPLETED)
    disposition_detail  TEXT                         -- JSON: qué tablas se pseudonimizaron-y-retuvieron vs. pseudonimizaron-y-purgaron (solo FORGET, nullable)
        CHECK (disposition_detail IS NULL OR json_valid(disposition_detail)),

    -- FK física owner_id -> accounts(id) (ADR-0141 enmienda 2026-07-11, M6):
    -- el titular que pide la portabilidad/olvido DEBE existir en `accounts`.
    -- RESTRICT: nunca se borra una cuenta con solicitudes de portabilidad.
    FOREIGN KEY (owner_id) REFERENCES accounts (id) ON DELETE RESTRICT
) STRICT;

-- Índice de la posición en la cadena (ADR-0141 M8: "Índice obligatorio en
-- event_sequence_id en tablas append-only") -- acceso cronológico / replay.
CREATE INDEX IF NOT EXISTS idx_data_portability_requests_event_sequence_id
    ON data_portability_requests (event_sequence_id);

-- Query path de "todas las solicitudes de este titular" (panel de
-- cumplimiento / autoservicio del usuario).
CREATE INDEX IF NOT EXISTS idx_data_portability_requests_owner_id
    ON data_portability_requests (owner_id);

-- Query path de "todos los eventos de ESTA solicitud lógica" -- usado por
-- `latest_status_for` para derivar el estado vigente sin recorrer la cadena
-- global completa.
CREATE INDEX IF NOT EXISTS idx_data_portability_requests_request_group_id
    ON data_portability_requests (request_group_id);

-- Enforzamiento append-only: rechaza UPDATE (STORY-043: cada evento de una
-- solicitud es un hecho histórico permanente -- el avance de estado se
-- modela como una fila NUEVA, nunca como una corrección in-place).
CREATE TRIGGER IF NOT EXISTS trg_data_portability_requests_no_update
BEFORE UPDATE ON data_portability_requests
BEGIN
    SELECT RAISE(ABORT, 'data_portability_requests is append-only: UPDATE is forbidden');
END;

-- Enforzamiento append-only: rechaza DELETE.
CREATE TRIGGER IF NOT EXISTS trg_data_portability_requests_no_delete
BEFORE DELETE ON data_portability_requests
BEGIN
    SELECT RAISE(ABORT, 'data_portability_requests is append-only: DELETE is forbidden');
END;
