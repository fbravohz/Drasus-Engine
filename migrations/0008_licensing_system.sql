-- Migración 0008: Licensing System (docs/features/licensing-system.md,
-- ADR-0143, ADR-0144, ADR-0141, ADR-0020 V2, STORY-028)
--
-- Crea la tabla `licenses`: cimiento #2 del substrato de monetización
-- (ADR-0144). Cada fila es UNA ACTIVACIÓN -- la licencia de un `owner_id`
-- (cuenta de `central-identity`, migración 0007) en UNA máquina concreta,
-- identificada por su huella de hardware (`node_id`, la MISMA que produce
-- `central-identity` -- esta tabla NUNCA la recalcula, ADR-0144 FIJO).
--
-- Por qué varias filas por owner_id: el modelo de negocio permite varias
-- "activaciones simultáneas por tier" (Sovereign = 3, Explorer = 1,
-- licensing-system.md "Parámetros Configurables": ACTIVATIONS_PER_TIER).
-- Cada máquina distinta que activa la MISMA licencia es una fila nueva;
-- reactivar la MISMA máquina (mismo owner_id + mismo node_id) reutiliza su
-- fila existente vía el índice único de abajo -- nunca duplica.
--
-- MUTABLE, no append-only: el heartbeat refresca `heartbeat_expires_at` y
-- `compliance_status_id` EN SITIO cada vez que la instancia revalida contra
-- la Cabina de Mando (o, en esta Story, contra el stub local). Por ADR-0141
-- ("PROHIBIDO usar el mismo nombre `event_sequence_id` para lo que es
-- `row_version` y viceversa"), esta tabla usa `row_version` (concurrencia
-- optimista) en vez de `event_sequence_id UNIQUE` (ese patrón es solo para
-- tablas append-only). El historial de cada cambio de licencia se audita
-- en `audit_events` (feature `audit-log` ya existente, migración 0002) --
-- esta tabla NO es su propio historial, solo el estado vigente.
--
-- Perfil ADR-0020 V2 para esta tabla (Perfil D "Ops/Auditoría", declarado en
-- `docs/features/licensing-system.md` §8 "Gobernanza y Estándares"):
--   - Grupo I  (Identidad & Integridad, UNIVERSAL, con row_version en vez de
--     event_sequence_id por ser tabla mutable, ADR-0141): id, created_at,
--     updated_at, audit_hash, audit_chain_hash, row_version.
--   - Grupo II (Soberanía & Propiedad): owner_id, institutional_tag,
--     access_token_id.
--   - Grupo IV (Infraestructura & Ops): node_id (huella de hardware
--     REUTILIZADA de `accounts.node_id`, NO recalculada), process_id.
--   - Grupo V  (Forense & Ejecución): signature_hash (firma Ed25519 del
--     archivo de licencia), compliance_status_id (veredicto vigente de la
--     licencia).
-- El Grupo III (Linaje Alpha & Datos) NO aplica (sin linaje genómico) y se
-- omite a propósito (ADR-0020 V2 "Aplicación": "PROHIBIDO copy-paste
-- masivo").
--
-- Columnas propias de la Feature (fuera del contrato de 25 campos, mismo
-- patrón que email/email_verification_status en accounts):
--   - tier: SOVEREIGN | EXPLORER (licensing-system.md "Niveles de Licencia").
--   - heartbeat_expires_at: instante (ns UTC) en que vence el heartbeat
--     vigente; se recalcula en cada refresco (issued_at + HEARTBEAT_INTERVAL,
--     CONFIG, default 90 días).
--
-- Guardarraíl ADR-0093: esta tabla JAMÁS almacena la clave PRIVADA de firma
-- (esa vive solo en el emisor -- Cabina de Mando real, o el stub local de
-- desarrollo -- nunca en la BD del cliente). `signature_hash` es la FIRMA
-- (dato público verificable), no la clave. Ninguna columna existe para
-- credenciales de bróker ni IPs de servidores live.
--
-- Idempotencia (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF
-- NOT EXISTS` hacen que volver a correr esta migración sea un no-op.

CREATE TABLE IF NOT EXISTS licenses (
    -- I. Identidad & Integridad (universal, ADR-0020 V2; row_version en vez
    -- de event_sequence_id por ser tabla mutable, ADR-0141).
    id                 TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at         INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock)
    updated_at         INTEGER NOT NULL,             -- Nanosegundos desde epoch; cambia con cada refresco de heartbeat
    audit_hash         TEXT    NOT NULL,             -- SHA-256 del contenido de esta versión de fila + enlace previo
    audit_chain_hash   TEXT,                         -- audit_hash de la versión anterior de esta fila (NULL solo en la fila génesis)
    row_version        INTEGER NOT NULL,             -- Contador de versión de esta fila; arranca en 1, +1 en cada refresco

    -- Identificador de la LICENCIA firmada (distinto de `id`, que es la PK de
    -- esta fila de ACTIVACIÓN). Una sola licencia (un `license_id`) puede
    -- tener varias filas de activación -- una por máquina, hasta el límite
    -- de `ACTIVATIONS_PER_TIER` -- porque el mismo archivo de licencia se
    -- activa en varias máquinas del mismo dueño. Es parte del payload
    -- firmado (`LicensePayload::license_id`): se persiste tal cual para
    -- poder reconstruir EXACTAMENTE los mismos bytes que el emisor firmó.
    license_id         TEXT    NOT NULL,

    -- II. Soberanía & Propiedad
    owner_id           TEXT    NOT NULL,             -- Dueño de la licencia; referencia a accounts.id (central-identity)
    institutional_tag  TEXT    NOT NULL,             -- Entorno/etiqueta institucional
    access_token_id    TEXT,                         -- Auth Tracking (nullable: no toda activación tiene un token de sesión activo)

    -- IV. Infraestructura & Ops
    node_id            TEXT    NOT NULL,             -- Huella de hardware REUTILIZADA de AccountIdentity.node_id (NO recalculada aquí)
    process_id         TEXT,                         -- Proceso que realizó la activación (nullable, informativo)

    -- V. Forense & Ejecución
    signature_hash        TEXT NOT NULL,             -- Firma Ed25519 (hex) del archivo de licencia -- dato PÚBLICO verificable, nunca la clave privada
    compliance_status_id  TEXT NOT NULL               -- Veredicto vigente de la licencia
        CHECK (compliance_status_id IN ('ACTIVE', 'GRACE', 'EXPIRED', 'REVOKED')),

    -- Columnas propias de la Feature (licensing-system.md "Niveles de Licencia" / "Parámetros Configurables")
    tier                  TEXT    NOT NULL CHECK (tier IN ('SOVEREIGN', 'EXPLORER')),
    -- Instante (ns UTC) en que el EMISOR firmó el payload vigente -- parte del
    -- contenido firmado (LicensePayload), INMUTABLE mientras no se re-firme.
    -- Distinto de `created_at`: `created_at` es cuándo esta FILA se persistió
    -- localmente; `issued_at` es cuándo el emisor produjo la firma que
    -- `signature_hash` acredita. Necesario para poder reconstruir EXACTAMENTE
    -- los mismos bytes que se firmaron y re-verificar la firma en cualquier
    -- momento posterior (ADR-0141: integridad cruzada del dato persistido).
    issued_at             INTEGER NOT NULL,
    heartbeat_expires_at  INTEGER NOT NULL,           -- Nanosegundos UTC en que vence el heartbeat vigente; se actualiza (y re-firma) en cada refresco

    -- Referencia a la cuenta dueña (misma BD/crate `shared` -- no cruza el
    -- límite hexagonal: `accounts` y `licenses` viven en el mismo `shared`,
    -- ADR-0137 excepción bendecida). ON DELETE RESTRICT (ADR-0141: CASCADE
    -- prohibido, protege la inmutabilidad del rastro forense de licencias).
    FOREIGN KEY (owner_id) REFERENCES accounts (id) ON DELETE RESTRICT
) STRICT;

-- Índice del lado FK (ADR-0141 M7: toda columna FK hijo necesita su índice).
CREATE INDEX IF NOT EXISTS idx_licenses_owner_id
    ON licenses (owner_id);

-- Búsqueda por huella de hardware (comparación de licencia vs. instancia,
-- licensing-system.md "Validación de Huella de Hardware").
CREATE INDEX IF NOT EXISTS idx_licenses_node_id
    ON licenses (node_id);

-- Todas las activaciones (filas) de UNA licencia concreta (varias máquinas
-- del mismo dueño comparten `license_id`).
CREATE INDEX IF NOT EXISTS idx_licenses_license_id
    ON licenses (license_id);

-- Una activación por máquina por dueño (licensing-system.md §3: "Una sola
-- instancia por máquina (FIJO)... Un segundo arranque en la misma máquina
-- comparte la huella y NO cuenta como una segunda activación"). El índice
-- único es lo que hace ese invariante irrompible a nivel de esquema: activar
-- dos veces la misma máquina reutiliza la fila, nunca inserta una segunda.
CREATE UNIQUE INDEX IF NOT EXISTS idx_licenses_owner_node
    ON licenses (owner_id, node_id);
