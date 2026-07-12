-- Migración 0014: Third-Party API Gateway
-- (docs/features/third-party-api-gateway.md, ADR-0144, ADR-0142, ADR-0093,
-- ADR-0137, ADR-0141, ADR-0020, STORY-035)
--
-- Crea DOS tablas: cimiento #8 del substrato de monetización (ADR-0144).
-- El gateway autentica solicitudes externas (terceros), limita su tasa de
-- uso y mide cada llamada -- sin exponer nunca el secreto de la credencial
-- en claro (ADR-0093) ni delegar datos sin el consentimiento vigente de
-- `consent-registry` (#5).
--
-- ── api_credentials (MUTABLE, row_version) ──────────────────────────────────
--
-- Una fila por credencial de API emitida a un tercero. MUTABLE porque el
-- estado de la credencial cambia con el tiempo (se puede revocar) -- por
-- ADR-0141 ("PROHIBIDO usar `event_sequence_id` para lo que es en realidad
-- `row_version`"), esta tabla usa `row_version INTEGER NOT NULL` con
-- concurrencia optimista (mismo patrón que `accounts`, migración
-- `0007_central_identity.sql`).
--
-- NUNCA se guarda el secreto de la credencial en claro (ADR-0093,
-- `docs/features/third-party-api-gateway.md` "Restricciones") -- solo su
-- hash SHA-256 (`credential_hash`). Autenticar = hashear la credencial
-- presentada y comparar contra este hash (`domain::third_party_api_gateway::
-- authenticate`).
--
-- Perfil ADR-0020 para esta tabla (Perfil D "Ops/Auditoría", acotado por el
-- Gate de Coherencia de STORY-035 §3 al subset `owner_id` + `access_token_id`
-- de Grupo II, más `node_id` de Grupo IV -- SIN `institutional_tag` ni
-- `manifest_id`, que el Filtro de Relevancia de ADR-0020 descarta para esta
-- Feature):
--   - Grupo I  (Identidad & Integridad, universal, con row_version por ser
--     tabla MUTABLE, ADR-0141): id, created_at, updated_at, audit_hash,
--     audit_chain_hash, row_version.
--   - Grupo II (subset): owner_id, access_token_id.
--   - Grupo IV (subset): node_id.
--
-- Columnas propias de la Feature (`docs/features/third-party-api-gateway.md`
-- "Persistencia"):
--   - credential_hash: SHA-256 hex de la credencial de API -- NUNCA el
--     secreto en claro.
--   - status: 'ACTIVE' o 'REVOKED' -- una credencial revocada niega
--     TODA autenticación futura, sin importar si el secreto es correcto.
--   - rate_limit_per_window / window_seconds: la ventana de rate-limit
--     configurable de esta credencial (`RATE_LIMIT_DEFAULT`, CONFIG).
--   - endpoints_enabled: JSON array de los endpoints que esta credencial
--     puede invocar (`ENDPOINTS_ENABLED`, CONFIG). `CHECK(json_valid)`
--     rechaza JSON corrupto a nivel de BD.
--
-- ── api_usage_records (APPEND-ONLY, event_sequence_id) ──────────────────────
--
-- Una fila por SOLICITUD procesada por el gateway (autenticada o no). Nunca
-- se edita ni se borra -- es el ledger de auditoría de uso de la API
-- pública, con la misma naturaleza append-only que `usage_records`
-- (migración `0010_usage_metering.sql`) y `consent_records` (migración
-- `0011_consent_registry.sql`): `event_sequence_id INTEGER NOT NULL UNIQUE`
-- + triggers anti UPDATE/DELETE + `audit_chain_hash` encadenado.
--
-- Mismo Perfil D acotado que `api_credentials` (owner_id + access_token_id +
-- node_id, denormalizados desde la credencial en el momento de la
-- solicitud -- así el ledger de uso es auto-contenido sin JOIN obligatorio
-- contra `api_credentials` para reportar).
--
-- Columnas propias de la Feature:
--   - credential_id: referencia a `api_credentials.id` -- NUNCA el secreto,
--     solo el identificador opaco de la credencial (ADR-0093).
--   - endpoint: el endpoint invocado (texto libre, ej. 'CERTIFY').
--   - outcome: el desenlace observable de esta solicitud -- 'ALLOWED',
--     'RATE_LIMITED' o 'DENIED' (autenticación inválida, credencial
--     revocada, endpoint no habilitado, o consentimiento no cubierto se
--     colapsan todos a 'DENIED' -- el motivo detallado vive solo en la
--     respuesta en memoria del gateway, nunca en esta columna).
--
-- Guardarraíl ADR-0093: ninguna columna de ninguna de las dos tablas
-- almacena el secreto de la credencial en claro, ni credenciales de bróker,
-- ni IPs de servidores live.
--
-- Idempotencia (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF
-- NOT EXISTS` / `CREATE TRIGGER IF NOT EXISTS` hacen que volver a correr
-- esta migración sea un no-op.

CREATE TABLE IF NOT EXISTS api_credentials (
    -- I. Identidad & Integridad (universal, ADR-0020; row_version en vez de
    -- event_sequence_id por ser tabla MUTABLE, ADR-0141).
    id                    TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock)
    updated_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch; avanza en cada UPDATE
    audit_hash            TEXT    NOT NULL,             -- SHA-256 del contenido de esta versión de fila
    audit_chain_hash      TEXT,                         -- audit_hash de la versión anterior (NULL solo en row_version=1)
    row_version           INTEGER NOT NULL,             -- Concurrencia optimista (ADR-0141): 1, 2, 3, ...

    -- II. Soberanía & Propiedad (subset acotado por el Gate de Coherencia)
    owner_id              TEXT    NOT NULL,             -- Dueño de la credencial (viene de central-identity)
    access_token_id       TEXT,                         -- Sesión/token que emitió esta credencial (nullable)

    -- IV. Infraestructura & Ops (subset)
    node_id               TEXT    NOT NULL,             -- Máquina que emitió/administra esta credencial

    -- Columnas propias de la Feature (docs/features/third-party-api-gateway.md "Persistencia")
    -- Hash SHA-256 (hex) de la credencial de API -- NUNCA el secreto en claro (ADR-0093).
    credential_hash        TEXT    NOT NULL,
    -- Estado de la credencial: una vez REVOKED, toda autenticación futura se niega.
    status                 TEXT    NOT NULL CHECK (status IN ('ACTIVE', 'REVOKED')),
    -- Solicitudes permitidas por ventana (RATE_LIMIT_DEFAULT, CONFIG).
    rate_limit_per_window  INTEGER NOT NULL,
    -- Duración de la ventana de rate-limit, en segundos.
    window_seconds         INTEGER NOT NULL,
    -- JSON array de los endpoints habilitados para esta credencial (ENDPOINTS_ENABLED, CONFIG).
    endpoints_enabled      TEXT    NOT NULL CHECK (json_valid(endpoints_enabled)),

    -- FK física owner_id -> accounts(id) (ADR-0141 enmienda 2026-07-11, M6):
    -- el dueño de la credencial DEBE existir en `accounts`. RESTRICT: nunca
    -- se borra una cuenta con credenciales emitidas.
    FOREIGN KEY (owner_id) REFERENCES accounts (id) ON DELETE RESTRICT
) STRICT;

-- Índice de unicidad del hash: dos credenciales jamás comparten el mismo
-- secreto -- también sirve de acceso directo para la autenticación
-- (buscar por hash en vez de recorrer la tabla completa).
CREATE UNIQUE INDEX IF NOT EXISTS idx_api_credentials_credential_hash
    ON api_credentials (credential_hash);

-- Índice del lado propietario (consistente con el resto de columnas Grupo
-- II owner_id de otras tablas del substrato).
CREATE INDEX IF NOT EXISTS idx_api_credentials_owner_id
    ON api_credentials (owner_id);

CREATE TABLE IF NOT EXISTS api_usage_records (
    -- I. Identidad & Integridad (universal, ADR-0020; event_sequence_id en
    -- vez de row_version por ser tabla APPEND-ONLY, ADR-0141).
    id                    TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock)
    updated_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch; append-only => siempre igual a created_at
    audit_hash            TEXT    NOT NULL,             -- SHA-256 del contenido de esta fila + enlace previo
    audit_chain_hash      TEXT,                         -- audit_hash de la fila anterior (NULL solo en la fila génesis)
    event_sequence_id     INTEGER NOT NULL UNIQUE,      -- Posición monótona en la cadena global (1, 2, 3, ...)

    -- II. Soberanía & Propiedad (subset, denormalizado desde api_credentials)
    owner_id              TEXT    NOT NULL,
    access_token_id       TEXT,

    -- IV. Infraestructura & Ops (subset, denormalizado desde api_credentials)
    node_id               TEXT    NOT NULL,

    -- Columnas propias de la Feature
    -- Referencia opaca a la credencial que hizo la solicitud (NUNCA el secreto).
    credential_id          TEXT    NOT NULL,
    -- Endpoint invocado (texto libre, ej. 'CERTIFY').
    endpoint               TEXT    NOT NULL,
    -- Desenlace observable de esta solicitud.
    outcome                TEXT    NOT NULL CHECK (outcome IN ('ALLOWED', 'RATE_LIMITED', 'DENIED')),

    -- FK física owner_id -> accounts(id) (ADR-0141 enmienda 2026-07-11, M6):
    -- el dueño de la solicitud DEBE existir en `accounts`. RESTRICT: nunca
    -- se borra una cuenta con historial de uso de API.
    FOREIGN KEY (owner_id) REFERENCES accounts (id) ON DELETE RESTRICT
) STRICT;

-- Índice de la posición en la cadena (ADR-0141 M8: "Índice obligatorio en
-- event_sequence_id en tablas append-only") -- acceso cronológico / replay.
CREATE INDEX IF NOT EXISTS idx_api_usage_records_event_sequence_id
    ON api_usage_records (event_sequence_id);

-- Índice del lado propietario (M7 ADR-0141: toda columna FK-hijo requiere
-- su propio índice; el compuesto de abajo lidera por credential_id, no
-- sirve para lookups directos por owner_id).
CREATE INDEX IF NOT EXISTS idx_api_usage_records_owner_id
    ON api_usage_records (owner_id);

-- Query path principal de la ventana de rate-limit (third-party-api-gateway.md
-- "Ciclo de Vida" - "Proceso": "verifica rate-limit"): contar las
-- solicitudes ALLOWED de UNA credencial dentro de la ventana vigente exige
-- filtrar por (credential_id, created_at) -- este índice compuesto sirve
-- exactamente ese acceso sin escanear toda la tabla.
CREATE INDEX IF NOT EXISTS idx_api_usage_records_credential_created_at
    ON api_usage_records (credential_id, created_at);

-- Enforzamiento append-only: rechaza UPDATE.
CREATE TRIGGER IF NOT EXISTS trg_api_usage_records_no_update
BEFORE UPDATE ON api_usage_records
BEGIN
    SELECT RAISE(ABORT, 'api_usage_records is append-only: UPDATE is forbidden');
END;

-- Enforzamiento append-only: rechaza DELETE.
CREATE TRIGGER IF NOT EXISTS trg_api_usage_records_no_delete
BEFORE DELETE ON api_usage_records
BEGIN
    SELECT RAISE(ABORT, 'api_usage_records is append-only: DELETE is forbidden');
END;
