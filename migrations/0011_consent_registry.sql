-- Migración 0011: Consent Registry / Registro de Consentimiento ToS
-- (docs/features/consent-registry.md, ADR-0144, ADR-0143, ADR-0141,
-- ADR-0020, STORY-031)
--
-- Crea la tabla `consent_records`: cimiento #5 del substrato de
-- monetización (ADR-0144). Cada fila es UN EVENTO de consentimiento (el
-- usuario acepta una versión de ToS, la re-acepta tras un cambio de
-- versión, o cambia sus opt-outs granulares por tipo de dato). El firehose
-- del tier gratuito (ADR-0143) y toda venta de datos agregados
-- (`data-aggregation`, #9) son legales SOLO SI el puerto `consent_out` que
-- esta Feature produce devuelve cobertura -- NUNCA se accede directamente
-- a esta tabla desde otra feature (ADR-0137: acceso cross-feature solo por
-- puerto tipado).
--
-- APPEND-ONLY, NO mutable (docs/features/consent-registry.md
-- "Restricciones": "El registro de consentimiento es append-only
-- (inmutable, auditable)"). Por ADR-0141 ("PROHIBIDO usar el mismo nombre
-- `event_sequence_id` para lo que es `row_version` y viceversa"), esta
-- tabla usa `event_sequence_id INTEGER NOT NULL UNIQUE` (posición
-- monótona en la secuencia) en vez de `row_version` -- mismo patrón que
-- `audit_events` (migración `0002_audit_log.sql`) y `usage_records`
-- (migración `0010_usage_metering.sql`).
--
-- EL PUNTO DE MODELADO CRÍTICO (opt-outs MUTABLES sobre tabla INMUTABLE):
-- los opt-outs del usuario cambian con el tiempo, pero ninguna fila de
-- esta tabla se edita jamás. En su lugar, CADA cambio de consentimiento
-- (aceptar una versión, o ajustar un opt-out) inserta una fila NUEVA que
-- captura el estado COMPLETO en ese momento (`tos_version` +
-- `optout_map` ENTERO, no solo el campo que cambió). El estado VIGENTE de
-- un usuario es la fila con el `event_sequence_id` MÁXIMO para su
-- `owner_id` -- "última fila gana" (event-sourcing con snapshot completo,
-- ver `domain::consent_registry` para la función pura que fusiona el
-- estado previo con una acción nueva).
--
-- Perfil ADR-0020 para esta tabla (Perfil D "Ops/Auditoría", declarado
-- en `docs/features/consent-registry.md` §"Gobernanza y Estándares" y
-- STORY-031 §3):
--   - Grupo I  (Identidad & Integridad, UNIVERSAL, con event_sequence_id
--     en vez de row_version por ser tabla APPEND-ONLY, ADR-0141): id,
--     created_at, updated_at, audit_hash, audit_chain_hash,
--     event_sequence_id.
--   - Grupo II (Soberanía & Propiedad): owner_id, institutional_tag.
--   - Grupo IV (Infraestructura & Ops): node_id.
--   - Subset de Grupo V (Gobernanza forense, SOLO si aplica):
--     compliance_status_id -- nullable: no todo evento de consentimiento
--     trae un estado de cumplimiento explícito anotado.
-- El Grupo III (Linaje Alpha & Datos) NO aplica (un evento de
-- consentimiento no tiene linaje genómico) y se omite a propósito
-- (ADR-0020 "Aplicación": "PROHIBIDO copy-paste masivo").
--
-- Columnas propias de la Feature (fuera del contrato de 25 campos,
-- `docs/features/consent-registry.md` "Persistencia"):
--   - tos_version: versión de ToS aceptada EN ESTE evento (texto libre,
--     ej. "v2"). Comparada contra la versión vigente en
--     `domain::consent_registry::needs_reacceptance`.
--   - consent_action: qué tipo de evento es esta fila -- 'ACCEPT' (primera
--     aceptación), 'REACCEPT' (re-aceptación tras cambio de versión) u
--     'OPTOUT_CHANGE' (solo se ajustó un opt-out, la versión no cambió).
--   - optout_map: mapa JSON `{tipo_dato: bool}` -- SNAPSHOT COMPLETO de
--     todos los opt-outs vigentes tras este evento (true = el usuario
--     optó por NO participar con ese tipo de dato). `CHECK(json_valid)`
--     rechaza JSON corrupto a nivel de BD, defensa en profundidad sobre la
--     validación de `domain::consent_registry::parse_optout_map`.
--   - accepted_at: instante de dominio (nanosegundos UTC, puerto Clock
--     inyectado -- NUNCA `SystemTime::now()` directo) en el que el usuario
--     ejecutó esta acción. Distinto de `created_at` (instante de
--     PERSISTENCIA) solo en teoría -- en la práctica ambos usan el mismo
--     reloj inyectado, pero se documentan por separado porque
--     `accepted_at` es el campo de DOMINIO (ADR-0141: "created_at" es
--     tiempo de procesamiento, el campo de dominio del evento va aparte).
--
-- Guardarraíl ADR-0093: esta tabla JAMÁS almacena secretos -- ninguna
-- columna existe para credenciales de bróker, claves de firma ni IPs de
-- servidores live. Solo se registra la versión de ToS aceptada, el mapa
-- de opt-outs y metadatos de auditoría.
--
-- Idempotencia (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF
-- NOT EXISTS` / `CREATE TRIGGER IF NOT EXISTS` hacen que volver a correr
-- esta migración sea un no-op.

CREATE TABLE IF NOT EXISTS consent_records (
    -- I. Identidad & Integridad (universal, ADR-0020; event_sequence_id
    -- en vez de row_version por ser tabla APPEND-ONLY, ADR-0141).
    id                    TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock)
    updated_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch; append-only => siempre igual a created_at
    audit_hash            TEXT    NOT NULL,             -- SHA-256 del contenido de esta fila + enlace previo
    audit_chain_hash      TEXT,                         -- audit_hash de la fila anterior (NULL solo en la fila génesis)
    event_sequence_id     INTEGER NOT NULL UNIQUE,      -- Posición monótona en la cadena global (1, 2, 3, ...)

    -- II. Soberanía & Propiedad
    owner_id              TEXT    NOT NULL,             -- Dueño del consentimiento (viene de central-identity)
    institutional_tag     TEXT    NOT NULL,             -- Entorno/etiqueta institucional

    -- IV. Infraestructura & Ops
    node_id               TEXT    NOT NULL,             -- Máquina que registró el evento de consentimiento

    -- V. Forense (subset, SOLO si aplica -- nullable)
    compliance_status_id  TEXT,                         -- Estado de cumplimiento vigente al momento del evento (nullable)

    -- Columnas propias de la Feature (docs/features/consent-registry.md "Persistencia")
    -- Versión de ToS aceptada EN ESTE evento.
    tos_version            TEXT    NOT NULL,
    -- Tipo de evento de consentimiento.
    consent_action         TEXT    NOT NULL CHECK (consent_action IN ('ACCEPT', 'REACCEPT', 'OPTOUT_CHANGE')),
    -- Snapshot COMPLETO del mapa de opt-outs tras este evento (JSON: {tipo_dato: bool}, true = opted-out).
    optout_map             TEXT    NOT NULL CHECK (json_valid(optout_map)),
    -- Instante de dominio (ns UTC) en que el usuario ejecutó esta acción.
    accepted_at            INTEGER NOT NULL
) STRICT;

-- Índice de la posición en la cadena (ADR-0141 M8: "Índice obligatorio en
-- event_sequence_id en tablas append-only") -- acceso cronológico / replay.
CREATE INDEX IF NOT EXISTS idx_consent_records_event_sequence_id
    ON consent_records (event_sequence_id);

-- Índice del lado propietario (consistente con el resto de columnas Grupo
-- II owner_id de otras tablas del substrato).
CREATE INDEX IF NOT EXISTS idx_consent_records_owner_id
    ON consent_records (owner_id);

-- Query path principal de la resolución de cobertura (consent-registry.md
-- "Ciclo de Vida" - "Proceso": "resuelve cobertura por tipo de dato"): el
-- estado VIGENTE de un owner_id es la fila con MAX(event_sequence_id) para
-- ese owner_id -- este índice compuesto sirve exactamente ese acceso.
CREATE INDEX IF NOT EXISTS idx_consent_records_owner_event_sequence_id
    ON consent_records (owner_id, event_sequence_id);

-- Enforzamiento append-only: rechaza UPDATE (consent-registry.md
-- "Restricciones": "El registro de consentimiento es append-only").
CREATE TRIGGER IF NOT EXISTS trg_consent_records_no_update
BEFORE UPDATE ON consent_records
BEGIN
    SELECT RAISE(ABORT, 'consent_records is append-only: UPDATE is forbidden');
END;

-- Enforzamiento append-only: rechaza DELETE.
CREATE TRIGGER IF NOT EXISTS trg_consent_records_no_delete
BEFORE DELETE ON consent_records
BEGIN
    SELECT RAISE(ABORT, 'consent_records is append-only: DELETE is forbidden');
END;
