-- Migración 0007: Central Identity (docs/features/central-identity.md,
-- ADR-0143, ADR-0144, ADR-0141, ADR-0020 V2)
--
-- Crea la tabla `accounts`: la cuenta local de usuario, cimiento #1 del
-- substrato de monetización (ADR-0144). Es el ancla de la que dependen
-- `licensing-system`, `usage-metering` y `consent-registry` (todas
-- necesitan `owner_id`).
--
-- A diferencia de `audit_events`/`job_results` (append-only), esta tabla es
-- MUTABLE: el estado de verificación de correo cambia con el tiempo. Por
-- ADR-0141 ("PROHIBIDO usar el mismo nombre `event_sequence_id` para lo que
-- es `row_version` y viceversa"), esta tabla usa `row_version` (contador de
-- versión por fila, arranca en 1, se incrementa con cada UPDATE) en vez de
-- `event_sequence_id UNIQUE` (ese patrón es solo para tablas append-only).
--
-- Perfil ADR-0020 V2 para esta tabla (Perfil D "Ops/Auditoría", según
-- declara `docs/features/central-identity.md` "Persistencia"):
--   - Grupo I  (Identidad & Integridad, UNIVERSAL, con la sustitución de
--     row_version por event_sequence_id que exige ADR-0141 para tablas
--     mutables): id, created_at, updated_at, audit_hash, audit_chain_hash,
--     row_version.
--   - Grupo II (Soberanía & Propiedad): owner_id, institutional_tag,
--     access_token_id.
--   - Grupo IV (Infraestructura & Ops / "Hardware"): node_id (huella de
--     hardware determinista, NO un hostname crudo).
-- Los Grupos III y V NO aplican a esta tabla (sin linaje Alpha, sin
-- ejecución/forense) y se omiten a propósito (ADR-0020 V2 "Aplicación":
-- "PROHIBIDO copy-paste masivo").
--
-- Columnas propias de la Feature (fuera del contrato de 25 campos, mismo
-- patrón que action_type/entity_type en audit_events):
--   - email: correo de la cuenta, único (una cuenta por correo).
--   - email_verification_status: PENDING | VERIFIED | REJECTED.
--   - oauth_provider: proveedor de identidad federada vinculado (nullable,
--     NULL si la cuenta solo usa correo+contraseña). Sin CHECK de valores
--     fijos porque OAUTH_PROVIDERS es CONFIG (central-identity.md
--     "Parámetros Configurables"), no un enum fijo de esquema.
--
-- Guardarraíl ADR-0093: esta tabla NUNCA almacena contraseñas en texto
-- plano, ni credenciales de bróker, ni IPs de servidores live -- ninguna
-- columna existe para eso. La contraseña (si el usuario registra
-- correo+contraseña) se hashea con sal antes de llegar a esta tabla; esa
-- lógica es del adaptador de registro, diferido (ver Story).
--
-- Idempotencia (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF
-- NOT EXISTS` hacen que volver a correr esta migración sea un no-op.

CREATE TABLE IF NOT EXISTS accounts (
    -- I. Identidad & Integridad (universal, ADR-0020 V2; row_version en vez
    -- de event_sequence_id por ser tabla mutable, ADR-0141).
    id                 TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at         INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock)
    updated_at         INTEGER NOT NULL,             -- Nanosegundos desde epoch; cambia con cada UPDATE
    audit_hash         TEXT    NOT NULL,             -- SHA-256 del contenido de esta versión de fila + enlace previo
    audit_chain_hash   TEXT,                         -- audit_hash de la versión anterior de esta fila (NULL solo en la fila génesis)
    row_version        INTEGER NOT NULL,             -- Contador de versión de esta fila; arranca en 1, +1 en cada UPDATE

    -- II. Soberanía & Propiedad
    owner_id           TEXT    NOT NULL,             -- Dueño capital/IP; una cuenta retail es dueña de sí misma (== id al crearse)
    institutional_tag  TEXT    NOT NULL,             -- Entorno/etiqueta institucional de la cuenta
    access_token_id    TEXT,                         -- Auth Tracking (nullable: no toda cuenta tiene un token de sesión activo)

    -- IV. Infraestructura & Ops ("Hardware")
    node_id            TEXT    NOT NULL,             -- Huella de hardware determinista (SHA-256 de identificadores de máquina)

    -- Columnas propias de la Feature (central-identity.md "Persistencia")
    email                      TEXT NOT NULL,        -- Correo de la cuenta
    email_verification_status TEXT NOT NULL          -- PENDING | VERIFIED | REJECTED
        CHECK (email_verification_status IN ('PENDING', 'VERIFIED', 'REJECTED')),
    oauth_provider             TEXT                  -- Proveedor de identidad federada vinculado (nullable)
) STRICT;

-- Una cuenta por correo (central-identity.md "Comportamientos Observables":
-- "el sistema envía verificación y no activa la cuenta hasta confirmarla" --
-- implica que el correo es la clave natural de registro).
CREATE UNIQUE INDEX IF NOT EXISTS idx_accounts_email
    ON accounts (email);

-- Acceso por huella de hardware (central-identity.md "Comportamientos
-- Observables": "Cuando se crean N identidades desde el mismo hardware ->
-- se marcan para revisión anti-abuso (señal, no bloqueo automático)").
CREATE INDEX IF NOT EXISTS idx_accounts_node_id
    ON accounts (node_id);
