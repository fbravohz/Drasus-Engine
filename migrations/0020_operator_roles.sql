-- Migración 0020: Operator Roles / Roles de Operador a la Carta
-- (docs/features/operator-roles.md, ADR-0149 -- cimiento #14 y ÚLTIMO del
-- substrato de monetización, ADR-0123, ADR-0141, ADR-0093, ADR-0020,
-- ADR-0137, STORY-044)
--
-- Crea TRES tablas del decimocuarto cimiento: el catálogo de roles a la
-- carta por cuenta maestra (MUTABLE), la asignación operador->rol
-- (MUTABLE) y el registro append-only de cambios (auditoría).
--
-- 1) `operator_roles` -- MUTABLE (un rol se edita: se reclasifica su
--    matriz de capacidades, o se revoca -- NUNCA se borra físicamente).
--    Por ADR-0141, usa `row_version` (contador de versión por fila, arranca
--    en 1, +1 en cada UPDATE) -- mismo patrón que `plans`
--    (`0009_plan_tier_quota.sql`).
--
--    `capability_matrix` es un objeto JSON `{ "<capability_key>": true|false,
--    ... }` -- dato, no código (ADR-0149): la unidad gateable es el puerto
--    de Feature (clave de capacidad), NUNCA el módulo. Sin columna de
--    "inmutabilidad" (STORY-044 §3.1, corrección 2026-07-07): la protección
--    del invariante "último admin en pie" es una validación DINÁMICA en el
--    Core (`domain::operator_roles::check_last_admin_standing`), nunca un
--    flag estático sobre una fila.
--
--    `status` (ACTIVE/REVOKED) es la baja lógica de un rol -- "eliminar" un
--    rol NUNCA es un DELETE físico (ADR-0141): mueve la fila a REVOKED,
--    preservando el historial para que las asignaciones que lo referenciaron
--    sigan siendo auditables. El `FOREIGN KEY ... ON DELETE RESTRICT` de
--    `operator_assignments` es la protección cinturón-y-tirantes contra un
--    DELETE físico accidental; el guardián primario es que este esquema
--    nunca emite un DELETE sobre esta tabla.
--
-- 2) `operator_assignments` -- MUTABLE (un operador tiene UN rol vigente
--    por cuenta; reasignar es un UPDATE de la misma fila, no una fila
--    nueva -- el `UNIQUE(owner_id, access_token_id)` de abajo lo hace
--    cumplir). Misma mecánica de `row_version` + `status` que
--    `operator_roles`.
--
-- 3) `operator_role_events` -- APPEND-ONLY ATÓMICA (cada creación, edición,
--    revocación de rol, y cada alta/baja de asignación, es un hecho
--    histórico permanente -- nunca se corrige in-place). Usa
--    `event_sequence_id INTEGER NOT NULL UNIQUE` (posición monótona
--    GLOBAL) -- mismo patrón que `data_portability_requests`
--    (`0019_data_portability.sql`).
--
--    Regla "Atomicidad de ledgers append-only" (rust-engineer/SKILL.md §4,
--    causa raíz DEBT-001): el `event_sequence_id` se deriva DENTRO de una
--    transacción `BEGIN IMMEDIATE` (ver
--    `persistence::operator_roles::OperatorRoleEventRepository` y las
--    escrituras guardadas del invariante "último admin en pie", que
--    registran su evento en la MISMA transacción que valida y escribe el
--    cambio) -- el `UNIQUE` de abajo es cinturón-y-tirantes, no el guardián
--    primario.
--
-- Perfil ADR-0020 (Perfil D "Ops/Auditoría", STORY-044 §3):
--   - Grupo I  (Identidad & Integridad, UNIVERSAL -- `row_version` en las
--     dos primeras tablas por ser MUTABLES; `event_sequence_id` en la
--     tercera por ser APPEND-ONLY, ADR-0141).
--   - Grupo II (Soberanía & Propiedad): `owner_id` (cuenta dueña del
--     catálogo) + `institutional_tag` en las TRES tablas;
--     `access_token_id` (ancla de atribución del operador, ADR-0020) SOLO
--     en `operator_assignments`.
--   - Grupo IV (Infraestructura & Ops): `node_id` SOLO en
--     `operator_role_events` -- qué máquina registró ESTE evento (el
--     catálogo y las asignaciones son metadato de gobernanza de la cuenta,
--     no atado a una máquina concreta).
--   - Subset V (Forense & Ejecución): `compliance_status_id` (nullable) en
--     `operator_role_events` -- estado de cumplimiento vigente al momento
--     de este evento.
-- El Grupo III (Linaje Alpha & Datos) NO aplica (un rol de operador no
-- tiene linaje genómico) y se omite a propósito.
--
-- Guardarraíl ADR-0093 (estructural): NINGUNA columna de ninguna de las
-- tres tablas puede contener una credencial de bróker, una clave de
-- cifrado ni una IP de servidor live -- estas tablas solo gobiernan QUIÉN
-- puede invocar QUÉ puerto, nunca transportan el secreto en sí.
--
-- Idempotencia (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF
-- NOT EXISTS` / `CREATE TRIGGER IF NOT EXISTS` hacen que volver a correr
-- esta migración sea un no-op.

CREATE TABLE IF NOT EXISTS operator_roles (
    -- I. Identidad & Integridad (universal, ADR-0020; row_version en vez de
    -- event_sequence_id por ser tabla MUTABLE, ADR-0141).
    id                TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at        INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock)
    updated_at        INTEGER NOT NULL,             -- Nanosegundos desde epoch; cambia con cada UPDATE
    audit_hash        TEXT    NOT NULL,             -- SHA-256 del contenido de esta versión de fila + enlace previo
    audit_chain_hash  TEXT,                         -- audit_hash de la versión anterior de esta fila (NULL solo en la fila génesis)
    row_version       INTEGER NOT NULL,             -- Contador de versión de esta fila; arranca en 1, +1 en cada UPDATE

    -- II. Soberanía & Propiedad
    owner_id          TEXT    NOT NULL,             -- Cuenta maestra dueña de este catálogo de roles
    institutional_tag TEXT    NOT NULL,             -- Entorno/etiqueta institucional

    -- Columnas propias de la Feature (STORY-044 §3.1)
    role_name          TEXT    NOT NULL,            -- Nombre libre del rol dentro de la cuenta (ej. "Analyst", "Risk Manager")
    capability_matrix  TEXT    NOT NULL             -- JSON objeto { "<capability_key>": true|false, ... } -- dato, no código
        CHECK (json_valid(capability_matrix)),
    status             TEXT    NOT NULL             -- ACTIVE | REVOKED -- baja lógica, NUNCA DELETE físico (ADR-0141)
        CHECK (status IN ('ACTIVE', 'REVOKED')),

    -- No se duplican nombres de rol dentro de la MISMA cuenta.
    UNIQUE (owner_id, role_name)
) STRICT;

-- Query path de "todos los roles de esta cuenta" (panel de roles y
-- operadores; también usado para cargar el estado admin-relevante dentro
-- de la transacción del guardarraíl "último admin en pie").
CREATE INDEX IF NOT EXISTS idx_operator_roles_owner_id
    ON operator_roles (owner_id);

CREATE TABLE IF NOT EXISTS operator_assignments (
    -- I. Identidad & Integridad (universal, ADR-0020; row_version en vez de
    -- event_sequence_id por ser tabla MUTABLE, ADR-0141).
    id                TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at        INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock)
    updated_at        INTEGER NOT NULL,             -- Nanosegundos desde epoch; cambia con cada UPDATE
    audit_hash        TEXT    NOT NULL,             -- SHA-256 del contenido de esta versión de fila + enlace previo
    audit_chain_hash  TEXT,                         -- audit_hash de la versión anterior de esta fila (NULL solo en la fila génesis)
    row_version       INTEGER NOT NULL,             -- Contador de versión de esta fila; arranca en 1, +1 en cada UPDATE

    -- II. Soberanía & Propiedad
    owner_id          TEXT    NOT NULL,             -- Cuenta maestra dueña de esta asignación
    institutional_tag TEXT    NOT NULL,             -- Entorno/etiqueta institucional
    access_token_id   TEXT    NOT NULL,             -- Ancla de atribución del operador (ADR-0020) -- login humano o conexión MCP

    -- Columnas propias de la Feature (STORY-044 §3.2)
    operator_type      TEXT    NOT NULL             -- HUMAN (login) | AGENT (conexión MCP) -- mismo catálogo para ambos
        CHECK (operator_type IN ('HUMAN', 'AGENT')),
    role_id             TEXT    NOT NULL             -- FK a operator_roles -- JAMÁS ON DELETE CASCADE (ADR-0141 M7)
        REFERENCES operator_roles (id) ON DELETE RESTRICT,
    status               TEXT    NOT NULL            -- ACTIVE | REVOKED -- baja lógica, NUNCA DELETE físico (ADR-0141)
        CHECK (status IN ('ACTIVE', 'REVOKED')),

    -- Un operador tiene UN rol vigente por cuenta -- reasignar es un UPDATE
    -- de esta misma fila, no una fila nueva.
    UNIQUE (owner_id, access_token_id)
) STRICT;

-- Índice obligatorio en la FK (ADR-0141 M7/M8: toda FK lleva su índice).
CREATE INDEX IF NOT EXISTS idx_operator_assignments_role_id
    ON operator_assignments (role_id);

-- Query path de "todas las asignaciones de esta cuenta" (panel de roles y
-- operadores; también usado para cargar el estado admin-relevante dentro
-- de la transacción del guardarraíl "último admin en pie").
CREATE INDEX IF NOT EXISTS idx_operator_assignments_owner_id
    ON operator_assignments (owner_id);

CREATE TABLE IF NOT EXISTS operator_role_events (
    -- I. Identidad & Integridad (universal, ADR-0020; event_sequence_id en
    -- vez de row_version por ser tabla APPEND-ONLY, ADR-0141).
    id                 TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at         INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock) -- instante de PERSISTENCIA de ESTE evento
    updated_at         INTEGER NOT NULL,             -- Nanosegundos desde epoch; append-only => siempre igual a created_at
    audit_hash         TEXT    NOT NULL,             -- SHA-256 del contenido de ESTA FILA + enlace previo (integridad del ledger)
    audit_chain_hash   TEXT,                         -- audit_hash de la fila anterior (NULL solo en la fila génesis)
    event_sequence_id  INTEGER NOT NULL UNIQUE,      -- Posición monótona en la cadena GLOBAL (1, 2, 3, ...)

    -- II. Soberanía & Propiedad
    owner_id           TEXT    NOT NULL,             -- Cuenta maestra afectada por este cambio
    institutional_tag  TEXT    NOT NULL,             -- Entorno/etiqueta institucional

    -- IV. Infraestructura & Ops
    node_id            TEXT    NOT NULL,             -- Máquina que registró ESTE evento

    -- Subset de V. Forense & Ejecución (Gobernanza/Cumplimiento)
    compliance_status_id TEXT,                       -- Estado de cumplimiento vigente al momento de ESTE evento (nullable)

    -- Columnas propias de la Feature (STORY-044 §3.3)
    change_type        TEXT    NOT NULL              -- Catálogo cerrado de seis tipos de cambio (ADR-0008)
        CHECK (change_type IN (
            'ROLE_CREATED', 'ROLE_UPDATED', 'ROLE_REVOKED',
            'ASSIGNMENT_SET', 'ASSIGNMENT_REVOKED', 'AUTHORITY_OVERRIDE'
        )),
    subject_ref         TEXT    NOT NULL,            -- El role_id o access_token_id afectado por este cambio
    detail               TEXT                        -- JSON opcional con detalle adicional del cambio (nullable)
        CHECK (detail IS NULL OR json_valid(detail))
) STRICT;

-- Índice de la posición en la cadena (ADR-0141 M8: "Índice obligatorio en
-- event_sequence_id en tablas append-only") -- acceso cronológico / replay.
CREATE INDEX IF NOT EXISTS idx_operator_role_events_event_sequence_id
    ON operator_role_events (event_sequence_id);

-- Query path de "todos los eventos de esta cuenta" (panel de auditoría).
CREATE INDEX IF NOT EXISTS idx_operator_role_events_owner_id
    ON operator_role_events (owner_id);

-- Enforzamiento append-only: rechaza UPDATE (STORY-044: cada evento es un
-- hecho histórico permanente -- el estado vigente vive en `operator_roles`/
-- `operator_assignments`, nunca se corrige un evento pasado in-place).
CREATE TRIGGER IF NOT EXISTS trg_operator_role_events_no_update
BEFORE UPDATE ON operator_role_events
BEGIN
    SELECT RAISE(ABORT, 'operator_role_events is append-only: UPDATE is forbidden');
END;

-- Enforzamiento append-only: rechaza DELETE.
CREATE TRIGGER IF NOT EXISTS trg_operator_role_events_no_delete
BEFORE DELETE ON operator_role_events
BEGIN
    SELECT RAISE(ABORT, 'operator_role_events is append-only: DELETE is forbidden');
END;
