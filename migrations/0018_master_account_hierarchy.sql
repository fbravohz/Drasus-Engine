-- Migración 0018: Master Account Hierarchy / Jerarquía de Cuenta Maestra
-- (docs/features/master-account-hierarchy.md, ADR-0147 -- cimiento #12 rector
-- y ÚLTIMO del substrato de monetización, ADR-0143, ADR-0141, ADR-0093,
-- ADR-0020, ADR-0137, STORY-040)
--
-- Crea DOS tablas del duodécimo cimiento: el puntero de jerarquía que cada
-- hija cachea hacia su fondo (MUTABLE) y la doble atestación ISSUER/EXECUTOR
-- de cada orden de override que el fondo emite sobre una hija (APPEND-ONLY
-- ATÓMICA).
--
-- 1) `account_hierarchy` -- MUTABLE (el padre y la referencia de
--    consentimiento contractual pueden cambiar con el tiempo -- una hija
--    puede re-vincularse a otro fondo, o renovar su consentimiento). Por
--    ADR-0141, usa `row_version` (contador de versión por fila, arranca en
--    1, +1 en cada UPDATE) -- mismo patrón que `verified_accounts`
--    (`0016_verified_account_registry.sql`).
--
--    Regla fija #1 (ADR-0147): esta tabla es el PUNTERO, no el árbol -- cada
--    fila solo sabe su propio `parent_owner_id` (o NULL, sin padre). No
--    existe columna ni índice que reconstruya "todas las hijas de un fondo"
--    salvo una consulta explícita por `parent_owner_id` (ver el índice de
--    abajo) -- anti-`tenant_id`.
--
-- 2) `override_attestations` -- APPEND-ONLY ATÓMICA (cada intento de
--    override -- ejecutado o denegado -- es un hecho histórico permanente,
--    nunca se corrige in-place). Usa `event_sequence_id INTEGER NOT NULL
--    UNIQUE` (posición monótona GLOBAL) -- mismo patrón que
--    `attested_track_records` (`0016_verified_account_registry.sql`) e
--    `instance_backups` (`0017_instance_continuity.sql`).
--
--    Regla "Atomicidad de ledgers append-only" (rust-engineer/SKILL.md §4,
--    causa raíz DEBT-001): el `event_sequence_id` se deriva DENTRO de una
--    transacción `BEGIN IMMEDIATE` (ver
--    `persistence::master_account_hierarchy::OverrideAttestationRepository`),
--    nunca en sentencias sueltas -- el `UNIQUE` de abajo es cinturón-y-
--    tirantes, no el guardián primario.
--
-- Perfil ADR-0020 para AMBAS tablas (Perfil D "Ops/Auditoría",
-- `docs/features/master-account-hierarchy.md` "Gobernanza y Estándares" y
-- STORY-040 §3):
--   - Grupo I  (Identidad & Integridad, UNIVERSAL -- `row_version` en
--     `account_hierarchy` por ser MUTABLE; `event_sequence_id` en
--     `override_attestations` por ser APPEND-ONLY, ADR-0141).
--   - Grupo II (Soberanía & Propiedad): `owner_id` (la hija) y
--     `parent_owner_id` (el fondo) en AMBAS tablas -- la relación cruza DOS
--     dueños a propósito. `institutional_tag` NO se declara para este
--     cimiento (STORY-040 §3: la jerarquía cruza dueños, no es un campo de
--     entorno de un único dueño -- omitido a propósito, ADR-0020
--     "Aplicación": "PROHIBIDO copy-paste masivo").
--   - Grupo IV (Infraestructura & Ops): `node_id` -- qué máquina produjo
--     ESTA fila (la del fondo en `ISSUER`, la de la hija en `EXECUTOR`).
-- El Grupo III (Linaje Alpha & Datos) NO aplica (un puntero de jerarquía y
-- una atestación de mando no tienen linaje genómico) y se omite a propósito.
--
-- Columnas propias de `account_hierarchy` (fuera del contrato de 25 campos,
-- `docs/features/master-account-hierarchy.md` "Persistencia"):
--   - consent_ref: referencia de texto libre al consentimiento contractual
--     VIGENTE cacheado (ej. una versión de ToS) -- NO es la verdad legal en
--     sí (esa se re-resuelve SIEMPRE contra `consent-registry` real, #5, en
--     el momento de cada override); es solo un puntero informativo, igual
--     de espíritu que `broker_connection_ref` en `verified_account_registry`.
--
-- Columnas propias de `override_attestations`:
--   - attestation_side: ISSUER (el fondo emitió la orden) | EXECUTOR (la
--     hija la recibió y la ejecutó o rechazó) -- regla fija #4: toda orden
--     produce EXACTAMENTE una fila de cada lado, nunca una mutación
--     silenciosa.
--   - command_kind: ARCHIVE | MODIFY | REQUEST_AUDIT_REPORT -- catálogo
--     `OVERRIDE_COMMANDS` cerrado (ADR-0008), nunca texto libre.
--   - target_ref: qué recurso de la hija referencia esta orden (estrategia,
--     portafolio, parámetro) -- texto libre, el vocabulario lo fija quien
--     llama (mismo criterio que `time_window` en `verified_account_registry`).
--   - outcome: EXECUTED | DENIED -- la etiqueta persistida de dos valores,
--     derivada SIEMPRE de `OverrideOutcome` (nunca construida por separado).
--     Regla fija #3: `EXECUTED` únicamente si el `ConsentVerdict` REAL de
--     `consent-registry` (#5) fue `Covered` en el momento de ESTA fila.
--   - justification: texto libre opcional -- por qué se emitió/ejecutó el
--     override (ej. "riesgo excedido"). Nullable: no toda orden trae una
--     justificación explícita.
--
-- Guardarraíl ADR-0093 (estructural): NINGUNA columna de ninguna de las dos
-- tablas puede contener una credencial de bróker, una IP de servidor live,
-- ni ningún secreto -- el mando que viaja por el relé genérico (ADR-0143,
-- adaptador de red diferido) es un comando cifrado que este esquema NO
-- almacena en claro; solo el HECHO auditado del intento (side/kind/outcome).
--
-- Idempotencia (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF
-- NOT EXISTS` / `CREATE TRIGGER IF NOT EXISTS` hacen que volver a correr
-- esta migración sea un no-op.

CREATE TABLE IF NOT EXISTS account_hierarchy (
    -- I. Identidad & Integridad (universal, ADR-0020; row_version en vez de
    -- event_sequence_id por ser tabla MUTABLE, ADR-0141).
    id                TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at        INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock)
    updated_at        INTEGER NOT NULL,             -- Nanosegundos desde epoch; cambia con cada UPDATE
    audit_hash        TEXT    NOT NULL,             -- SHA-256 del contenido de esta versión de fila + enlace previo
    audit_chain_hash  TEXT,                         -- audit_hash de la versión anterior de esta fila (NULL solo en la fila génesis)
    row_version       INTEGER NOT NULL,             -- Contador de versión de esta fila; arranca en 1, +1 en cada UPDATE

    -- II. Soberanía & Propiedad -- la relación cruza DOS dueños a propósito.
    owner_id          TEXT    NOT NULL UNIQUE,      -- La hija -- EXACTAMENTE una fila de jerarquía por hija
    parent_owner_id   TEXT,                         -- El fondo -- NULL = sin padre (cuenta huérfana todavía no vinculada)

    -- Columnas propias de la Feature (master-account-hierarchy.md "Persistencia")
    consent_ref       TEXT    NOT NULL,             -- Referencia CACHEADA al consentimiento contractual vigente (no la verdad legal en sí)

    -- IV. Infraestructura & Ops
    node_id           TEXT    NOT NULL,             -- Máquina que registró/actualizó esta jerarquía

    -- FK física owner_id -> accounts(id) (ADR-0141 enmienda 2026-07-11, M6):
    -- la hija DEBE existir en `accounts`. RESTRICT: nunca se borra una
    -- cuenta con fila de jerarquía propia. `owner_id` ya es UNIQUE arriba,
    -- por lo que el índice de esa restricción sirve también a la FK.
    FOREIGN KEY (owner_id) REFERENCES accounts (id) ON DELETE RESTRICT
) STRICT;

-- Query path de "todas las hijas de este fondo" (panel de jerarquía de
-- cuenta maestra: consulta explícita, nunca una columna de árbol completo --
-- regla fija #1, anti-tenant_id).
CREATE INDEX IF NOT EXISTS idx_account_hierarchy_parent_owner_id
    ON account_hierarchy (parent_owner_id);

CREATE TABLE IF NOT EXISTS override_attestations (
    -- I. Identidad & Integridad (universal, ADR-0020; event_sequence_id en
    -- vez de row_version por ser tabla APPEND-ONLY, ADR-0141).
    id                 TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at         INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock) -- instante de PERSISTENCIA
    updated_at         INTEGER NOT NULL,             -- Nanosegundos desde epoch; append-only => siempre igual a created_at
    audit_hash         TEXT    NOT NULL,             -- SHA-256 del contenido de ESTA FILA + enlace previo (integridad del ledger)
    audit_chain_hash   TEXT,                         -- audit_hash de la fila anterior (NULL solo en la fila génesis)
    event_sequence_id  INTEGER NOT NULL UNIQUE,      -- Posición monótona en la cadena GLOBAL (1, 2, 3, ...)

    -- II. Soberanía & Propiedad -- ambos dueños de la relación jerárquica.
    owner_id           TEXT    NOT NULL,             -- La hija afectada por este override
    parent_owner_id    TEXT    NOT NULL,             -- El fondo que gobierna/emitió este override

    -- IV. Infraestructura & Ops
    node_id            TEXT    NOT NULL,             -- Máquina que produjo ESTA fila (la del fondo en ISSUER, la de la hija en EXECUTOR)

    -- Columnas propias de la Feature (master-account-hierarchy.md "Persistencia")
    attestation_side   TEXT    NOT NULL              -- ISSUER (el fondo emitió) | EXECUTOR (la hija recibió/ejecutó) -- regla fija #4
        CHECK (attestation_side IN ('ISSUER', 'EXECUTOR')),
    command_kind       TEXT    NOT NULL              -- Catálogo OVERRIDE_COMMANDS cerrado (ADR-0008)
        CHECK (command_kind IN ('ARCHIVE', 'MODIFY', 'REQUEST_AUDIT_REPORT')),
    target_ref         TEXT    NOT NULL,             -- Qué recurso de la hija referencia esta orden (estrategia/portafolio/parámetro)
    outcome            TEXT    NOT NULL              -- EXECUTED (solo con ConsentVerdict::Covered vigente, regla fija #3) | DENIED
        CHECK (outcome IN ('EXECUTED', 'DENIED')),
    justification      TEXT,                         -- Texto libre opcional -- por qué se emitió/ejecutó el override

    -- FK física owner_id -> accounts(id) (ADR-0141 enmienda 2026-07-11, M6):
    -- la hija afectada DEBE existir en `accounts`. RESTRICT: nunca se borra
    -- una cuenta con overrides asociados. `parent_owner_id` (el fondo) NO
    -- lleva FK: fuera del alcance textual de la enmienda, que fija la regla
    -- solo para columnas literalmente nombradas `owner_id` -- reportado al
    -- Tech-Lead como observación abierta.
    FOREIGN KEY (owner_id) REFERENCES accounts (id) ON DELETE RESTRICT
) STRICT;

-- Índice de la posición en la cadena (ADR-0141 M8: "Índice obligatorio en
-- event_sequence_id en tablas append-only") -- acceso cronológico / replay.
CREATE INDEX IF NOT EXISTS idx_override_attestations_event_sequence_id
    ON override_attestations (event_sequence_id);

-- Query path de "todos los overrides de esta hija" (panel de jerarquía:
-- historial de mando elevado recibido).
CREATE INDEX IF NOT EXISTS idx_override_attestations_owner_id
    ON override_attestations (owner_id);

-- Query path de "todos los overrides emitidos por este fondo".
CREATE INDEX IF NOT EXISTS idx_override_attestations_parent_owner_id
    ON override_attestations (parent_owner_id);

-- Enforzamiento append-only: rechaza UPDATE (master-account-hierarchy.md:
-- cada intento de override es un hecho histórico permanente -- regla fija
-- #5, "eliminar" nunca se aplica a este propio ledger).
CREATE TRIGGER IF NOT EXISTS trg_override_attestations_no_update
BEFORE UPDATE ON override_attestations
BEGIN
    SELECT RAISE(ABORT, 'override_attestations is append-only: UPDATE is forbidden');
END;

-- Enforzamiento append-only: rechaza DELETE.
CREATE TRIGGER IF NOT EXISTS trg_override_attestations_no_delete
BEFORE DELETE ON override_attestations
BEGIN
    SELECT RAISE(ABORT, 'override_attestations is append-only: DELETE is forbidden');
END;
