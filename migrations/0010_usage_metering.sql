-- Migración 0010: Usage Metering / Libro de Nocional (docs/features/usage-metering.md,
-- ADR-0144, ADR-0143, ADR-0141, ADR-0020, STORY-030)
--
-- Crea la tabla `usage_records`: cimiento #4 del substrato de monetización
-- (ADR-0144). Cada fila es UNA OPERACIÓN medida -- registro append-only del
-- nocional en USD (tamaño × precio) de una orden ejecutada, más el
-- acumulado del ciclo de facturación vigente hasta esa fila y el veredicto
-- de cuota resultante. `licensing-system` (cimiento #2, gate) y el billing
-- futuro leen este libro a través del puerto `usage_out` que esta Feature
-- produce -- NUNCA acceden directamente a esta tabla (ADR-0137: acceso
-- cross-feature solo por puerto tipado).
--
-- APPEND-ONLY, NO mutable (docs/features/usage-metering.md "Restricciones":
-- "NUNCA se modifica un registro del libro: es append-only"). Por
-- ADR-0141 ("PROHIBIDO usar el mismo nombre `event_sequence_id` para lo
-- que es `row_version` y viceversa"), esta tabla usa `event_sequence_id
-- INTEGER NOT NULL UNIQUE` (posición monótona en la secuencia) en vez de
-- `row_version` -- mismo patrón que `audit_events` (migración
-- `0002_audit_log.sql`). El reinicio de ciclo de facturación NO borra
-- filas: un `billing_cycle_id` nuevo hace que la acumulación arranque en
-- cero para ese ciclo mientras las filas del ciclo anterior permanecen
-- intactas (el histórico se conserva -- usage-metering.md "Comportamientos
-- Observables": "el acumulado se reinicia (el histórico se conserva)").
--
-- Perfil ADR-0020 para esta tabla (Perfil D "Ops/Auditoría", declarado en
-- `docs/features/usage-metering.md` §"Gobernanza y Estándares" y STORY-030
-- §3):
--   - Grupo I  (Identidad & Integridad, UNIVERSAL, con event_sequence_id en
--     vez de row_version por ser tabla APPEND-ONLY, ADR-0141): id,
--     created_at, updated_at, audit_hash, audit_chain_hash,
--     event_sequence_id.
--   - Grupo II (Soberanía & Propiedad): owner_id, institutional_tag.
--   - Grupo IV (Infraestructura & Ops): node_id.
--   - Subset de Grupo V (Gobernanza forense, SOLO si aplica):
--     compliance_status_id -- nullable: no toda operación medida trae un
--     estado de cumplimiento explícito (solo aplica si el gate de
--     licenciamiento anotó uno al momento de la operación).
-- El Grupo III (Linaje Alpha & Datos) NO aplica (una operación medida no
-- tiene linaje genómico) y se omite a propósito (ADR-0020 "Aplicación":
-- "PROHIBIDO copy-paste masivo").
--
-- Columnas propias de la Feature (fuera del contrato de 25 campos,
-- `docs/features/usage-metering.md` "Persistencia"):
--   - notional_per_op: nocional de ESTA operación, INTEGER escalado ×10⁸
--     (ADR-0141 -- NUNCA REAL). Resultado de `domain::usage_metering::
--     compute_notional(size, price)` -- ver ese módulo para el reescalado
--     ×10¹⁶→×10⁸ con redondeo explícito.
--   - cycle_accumulated: el acumulado del ciclo de facturación INMEDIATAMENTE
--     DESPUÉS de sumar esta operación (snapshot histórico de la
--     acumulación en el momento en que se grabó esta fila) -- INTEGER
--     escalado ×10⁸.
--   - billing_cycle_id: identificador del ciclo de facturación vigente al
--     momento de la operación (formato "YYYY-MM" para ciclo mensual,
--     `domain::usage_metering::derive_billing_cycle_id`).
--   - instrument_id: instrumento operado (ej. "BTCUSDT").
--   - quota_verdict: veredicto de cuota tras esta operación -- 'WITHIN'
--     (dentro del límite del plan) o 'CROSSED' (cruzó el límite).
--
-- Guardarraíl ADR-0093: esta tabla JAMÁS almacena secretos -- ninguna
-- columna existe para credenciales de bróker, claves de firma ni IPs de
-- servidores live. Solo se mide NOCIONAL (nunca margen ni apalancamiento,
-- usage-metering.md "Restricciones").
--
-- Idempotencia (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF
-- NOT EXISTS` / `CREATE TRIGGER IF NOT EXISTS` hacen que volver a correr
-- esta migración sea un no-op.

CREATE TABLE IF NOT EXISTS usage_records (
    -- I. Identidad & Integridad (universal, ADR-0020; event_sequence_id
    -- en vez de row_version por ser tabla APPEND-ONLY, ADR-0141).
    id                    TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock)
    updated_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch; append-only => siempre igual a created_at
    audit_hash            TEXT    NOT NULL,             -- SHA-256 del contenido de esta fila + enlace previo
    audit_chain_hash      TEXT,                         -- audit_hash de la fila anterior (NULL solo en la fila génesis)
    event_sequence_id     INTEGER NOT NULL UNIQUE,      -- Posición monótona en la cadena global (1, 2, 3, ...)

    -- II. Soberanía & Propiedad
    owner_id              TEXT    NOT NULL,             -- Dueño de la operación medida
    institutional_tag     TEXT    NOT NULL,             -- Entorno/etiqueta institucional

    -- IV. Infraestructura & Ops
    node_id               TEXT    NOT NULL,             -- Máquina que registró la medición

    -- V. Forense (subset, SOLO si aplica -- nullable)
    compliance_status_id  TEXT,                         -- Estado de cumplimiento vigente al momento de la operación (nullable)

    -- Columnas propias de la Feature (docs/features/usage-metering.md "Persistencia")
    -- Nocional de esta operación, INTEGER escalado ×10⁸ (ADR-0141 -- NUNCA REAL).
    notional_per_op        INTEGER NOT NULL CHECK (notional_per_op >= 0),
    -- Acumulado del ciclo tras sumar esta operación, INTEGER escalado ×10⁸.
    cycle_accumulated      INTEGER NOT NULL CHECK (cycle_accumulated >= 0),
    -- Identificador del ciclo de facturación vigente (ej. "2026-07").
    billing_cycle_id       TEXT    NOT NULL,
    -- Instrumento operado.
    instrument_id          TEXT    NOT NULL,
    -- Veredicto de cuota tras esta operación.
    quota_verdict          TEXT    NOT NULL CHECK (quota_verdict IN ('WITHIN', 'CROSSED')),

    -- FK física owner_id -> accounts(id) (ADR-0141 enmienda 2026-07-11, M6):
    -- la cuenta dueña de esta operación medida DEBE existir en `accounts`
    -- (creada en la migración 0007, previa a esta). RESTRICT: nunca se
    -- borra una cuenta con operaciones medidas asociadas.
    FOREIGN KEY (owner_id) REFERENCES accounts (id) ON DELETE RESTRICT
) STRICT;

-- Índice de la posición en la cadena (M8 ADR-0141: "Índice obligatorio en
-- event_sequence_id en tablas append-only") -- acceso cronológico / replay.
CREATE INDEX IF NOT EXISTS idx_usage_records_event_sequence_id
    ON usage_records (event_sequence_id);

-- Índice del lado propietario (consistente con el resto de columnas Grupo
-- II owner_id de otras tablas del substrato).
CREATE INDEX IF NOT EXISTS idx_usage_records_owner_id
    ON usage_records (owner_id);

-- Query path principal de la acumulación por ciclo (usage-metering.md
-- "Ciclo de Vida" - "Proceso": "lo acumula en el ciclo vigente"): sumar
-- `notional_per_op` de todas las filas de un mismo (owner_id,
-- billing_cycle_id).
CREATE INDEX IF NOT EXISTS idx_usage_records_owner_billing_cycle
    ON usage_records (owner_id, billing_cycle_id);

-- Enforzamiento append-only: rechaza UPDATE (usage-metering.md
-- "Restricciones": "NUNCA se modifica un registro del libro").
CREATE TRIGGER IF NOT EXISTS trg_usage_records_no_update
BEFORE UPDATE ON usage_records
BEGIN
    SELECT RAISE(ABORT, 'usage_records is append-only: UPDATE is forbidden');
END;

-- Enforzamiento append-only: rechaza DELETE.
CREATE TRIGGER IF NOT EXISTS trg_usage_records_no_delete
BEFORE DELETE ON usage_records
BEGIN
    SELECT RAISE(ABORT, 'usage_records is append-only: DELETE is forbidden');
END;
