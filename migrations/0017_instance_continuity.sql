-- Migración 0017: Instance Continuity / Continuidad y Portabilidad de
-- Instancia (docs/features/instance-continuity.md, ADR-0146 -- cimiento #11
-- rector, ADR-0093, ADR-0143, ADR-0141, ADR-0020, ADR-0137, STORY-039)
--
-- Crea DOS tablas del undécimo cimiento del substrato de monetización: el
-- respaldo cifrado client-side de la DB local (el proveedor solo guarda
-- bytes opacos, adaptador de subida diferido) y el relevo de custodia
-- "maestro itinerante" (exactamente una máquina titular por cuenta).
--
-- 1) `instance_backups` -- APPEND-ONLY ATÓMICA (cada snapshot subido es un
--    hecho histórico permanente -- no se corrige in-place: se registra uno
--    nuevo). Usa `event_sequence_id INTEGER NOT NULL UNIQUE` (posición
--    monótona GLOBAL) -- mismo patrón que `domain_events`
--    (`0012_domain_events.sql`) y `attested_track_records`
--    (`0016_verified_account_registry.sql`).
--
--    Regla "Atomicidad de ledgers append-only" (rust-engineer/SKILL.md §4,
--    causa raíz DEBT-001): el `event_sequence_id` se deriva DENTRO de una
--    transacción `BEGIN IMMEDIATE` (ver
--    `persistence::instance_continuity::BackupRegistryRepository`), nunca en
--    sentencias sueltas -- el `UNIQUE` de abajo es cinturón-y-tirantes, no
--    el guardián primario.
--
-- 2) `custody_state` -- MUTABLE (la titularidad cambia con el tiempo: el
--    "maestro itinerante" se mueve de máquina en máquina). Por ADR-0141
--    ("PROHIBIDO usar el mismo nombre `event_sequence_id` para lo que es
--    `row_version` y viceversa"), usa `custody_epoch` -- el contador de
--    versión por fila (arranca en 1, +1 en cada reclamo de titularidad
--    exitoso), adaptado a nivel de INSTANCIA COMPLETA en vez de un campo
--    de negocio cualquiera (ADR-0146: "concurrencia optimista... aplicado a
--    nivel de instancia completa en vez de una fila"). Cumple el MISMO rol
--    que `row_version` en `accounts` (`0007_central_identity.sql`), pero
--    con el nombre de dominio que ADR-0146 exige.
--
-- Perfil ADR-0020 para AMBAS tablas (Perfil D "Ops/Auditoría",
-- `docs/features/instance-continuity.md` "Gobernanza y Estándares" y
-- STORY-039 §3):
--   - Grupo I  (Identidad & Integridad, UNIVERSAL -- `event_sequence_id` en
--     `instance_backups` por ser APPEND-ONLY; `custody_epoch` en
--     `custody_state` por ser MUTABLE, ADR-0141).
--   - Grupo II (Soberanía & Propiedad): owner_id, institutional_tag.
--   - Grupo IV (Infraestructura & Ops): node_id / titular_node_id -- qué
--     máquina respaldó / qué máquina es la titular vigente.
-- El Grupo III (Linaje Alpha & Datos) NO aplica (un respaldo cifrado y un
-- estado de custodia no tienen linaje genómico) y se omite a propósito
-- (ADR-0020 "Aplicación": "PROHIBIDO copy-paste masivo").
--
-- Columnas propias de `instance_backups` (fuera del contrato de 25 campos,
-- `docs/features/instance-continuity.md` "Persistencia"):
--   - snapshot_at: marca de tiempo del snapshot que este respaldo cubre
--     (nanosegundos, puerto Clock) -- distinta de `created_at` (instante de
--     PERSISTENCIA de esta fila del ledger).
--   - blob_hash: SHA-256 hex del blob cifrado (ciphertext + tag GCM) --
--     permite verificar integridad del blob remoto sin descifrarlo.
--   - blob_size_bytes: tamaño en bytes del blob cifrado, INTEGER (ADR-0141:
--     nunca REAL).
--   - nonce_hex: el nonce de AES-GCM usado para este blob, en hexadecimal.
--     NO es secreto (regla obligatoria #2, ADR-0002): se necesita junto con
--     la clave para descifrar después, así que viaja con los metadatos.
--
-- Columnas propias de `custody_state`:
--   - titular_node_id: la máquina que es la titular ESCRITORA vigente de
--     la cadena de auditoría de esta cuenta -- exactamente una por fila
--     (y exactamente una fila por owner_id, ver `UNIQUE` abajo).
--
-- Guardarraíl ADR-0093 (estructural, no solo por convención): NINGUNA
-- columna de ninguna de las dos tablas puede contener la clave de cifrado,
-- el secreto maestro, una credencial de bróker, ni una IP de servidor
-- live. El test de integración de este cimiento lo assert explícitamente
-- sobre las filas persistidas y sobre el catálogo de columnas.
--
-- Idempotencia (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF
-- NOT EXISTS` / `CREATE TRIGGER IF NOT EXISTS` hacen que volver a correr
-- esta migración sea un no-op.

CREATE TABLE IF NOT EXISTS instance_backups (
    -- I. Identidad & Integridad (universal, ADR-0020; event_sequence_id
    -- en vez de custody_epoch/row_version por ser tabla APPEND-ONLY,
    -- ADR-0141).
    id                    TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock) -- instante de PERSISTENCIA
    updated_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch; append-only => siempre igual a created_at
    audit_hash            TEXT    NOT NULL,             -- SHA-256 del contenido de ESTA FILA + enlace previo (integridad del ledger)
    audit_chain_hash      TEXT,                         -- audit_hash de la fila anterior (NULL solo en la fila génesis)
    event_sequence_id     INTEGER NOT NULL UNIQUE,      -- Posición monótona en la cadena GLOBAL (1, 2, 3, ...)

    -- II. Soberanía & Propiedad
    owner_id              TEXT    NOT NULL,             -- Dueño de la cuenta (central-identity, #1)
    institutional_tag     TEXT    NOT NULL,             -- Entorno/etiqueta institucional

    -- IV. Infraestructura & Ops
    node_id               TEXT    NOT NULL,             -- Máquina que produjo este respaldo

    -- Columnas propias de la Feature (instance-continuity.md "Persistencia")
    snapshot_at           INTEGER NOT NULL,             -- Nanosegundos: instante del snapshot que este respaldo cubre
    blob_hash             TEXT    NOT NULL,             -- SHA-256 hex del blob cifrado (ciphertext + tag GCM)
    blob_size_bytes       INTEGER NOT NULL CHECK (blob_size_bytes >= 0), -- Tamaño del blob cifrado, en bytes
    nonce_hex             TEXT    NOT NULL              -- Nonce de AES-GCM usado, en hex -- NO es secreto (ADR-0002)
) STRICT;

-- Índice de la posición en la cadena (ADR-0141 M8: "Índice obligatorio en
-- event_sequence_id en tablas append-only") -- acceso cronológico / replay.
CREATE INDEX IF NOT EXISTS idx_instance_backups_event_sequence_id
    ON instance_backups (event_sequence_id);

-- Query path de "todos los respaldos de este dueño" (panel de continuidad
-- de instancia: historial de snapshots subidos).
CREATE INDEX IF NOT EXISTS idx_instance_backups_owner_id
    ON instance_backups (owner_id);

-- Enforzamiento append-only: rechaza UPDATE (instance-continuity.md: cada
-- snapshot subido es un hecho histórico permanente).
CREATE TRIGGER IF NOT EXISTS trg_instance_backups_no_update
BEFORE UPDATE ON instance_backups
BEGIN
    SELECT RAISE(ABORT, 'instance_backups is append-only: UPDATE is forbidden');
END;

-- Enforzamiento append-only: rechaza DELETE.
CREATE TRIGGER IF NOT EXISTS trg_instance_backups_no_delete
BEFORE DELETE ON instance_backups
BEGIN
    SELECT RAISE(ABORT, 'instance_backups is append-only: DELETE is forbidden');
END;

CREATE TABLE IF NOT EXISTS custody_state (
    -- I. Identidad & Integridad (universal, ADR-0020; custody_epoch en vez
    -- de row_version -- MISMO rol, nombre de dominio fijado por ADR-0146:
    -- concurrencia optimista aplicada a nivel de INSTANCIA COMPLETA).
    id                    TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock)
    updated_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch; cambia con cada reclamo de titularidad
    audit_hash            TEXT    NOT NULL,             -- SHA-256 del contenido de esta versión de fila + enlace previo
    audit_chain_hash      TEXT,                         -- audit_hash de la versión anterior de esta fila (NULL solo en la fila génesis)
    custody_epoch         INTEGER NOT NULL,             -- Contador monótono de titularidad; arranca en 1, +1 en cada reclamo exitoso

    -- II. Soberanía & Propiedad
    owner_id              TEXT    NOT NULL UNIQUE,      -- Dueño de la cuenta -- EXACTAMENTE una fila de custodia por owner_id
    institutional_tag     TEXT    NOT NULL,             -- Entorno/etiqueta institucional

    -- IV. Infraestructura & Ops
    titular_node_id       TEXT    NOT NULL              -- Máquina que es la titular ESCRITORA vigente de la cadena de auditoría
) STRICT;

-- Nota: `owner_id` ya es `UNIQUE` arriba -- SQLite crea automáticamente un
-- índice para esa restricción, así que no hace falta declarar uno explícito
-- adicional para el lookup por dueño (a diferencia de `instance_backups`,
-- donde `owner_id` NO es único y sí necesita su propio índice).
