-- Migración 0016: Verified Account Registry / Registro de Cuentas
-- Verificadas Drasus (docs/features/verified-account-registry.md, ADR-0145
-- cimiento #10 -- rector, ADR-0143, ADR-0093, ADR-0141, ADR-0020, ADR-0137,
-- STORY-037)
--
-- Crea DOS tablas del décimo y último cimiento del substrato de
-- monetización: el pilar de "Cuentas Verificadas" (análogo a myFXbook / MT5
-- Signals, con el diferenciador soberano de que Drasus atestigua
-- criptográficamente lo que su propio motor ejecutó).
--
-- 1) `verified_accounts` -- MUTABLE (el estado de publicación y los ámbitos
--    de atestación cambian con el tiempo). Por ADR-0141 ("PROHIBIDO usar el
--    mismo nombre `event_sequence_id` para lo que es `row_version` y
--    viceversa"), usa `row_version` (contador de versión por fila, arranca
--    en 1, +1 en cada UPDATE) -- mismo patrón que `accounts`
--    (migración `0007_central_identity.sql`).
--
-- 2) `attested_track_records` -- APPEND-ONLY ATÓMICA (cada track calculado
--    es un snapshot inmutable firmado -- un hecho histórico permanente, no
--    se corrige in-place: se calcula uno nuevo). Usa
--    `event_sequence_id INTEGER NOT NULL UNIQUE` (posición monótona GLOBAL)
--    -- mismo patrón que `domain_events` (`0012_domain_events.sql`) y
--    `generated_reports` (`0013_generated_reports.sql`).
--
-- Regla "Atomicidad de ledgers append-only" (rust-engineer/SKILL.md §4,
-- causa raíz DEBT-001): el `event_sequence_id` de `attested_track_records`
-- se deriva DENTRO de una transacción `BEGIN IMMEDIATE` (ver
-- `persistence::verified_account_registry::AttestedTrackRecordRepository`),
-- nunca en sentencias sueltas -- el `UNIQUE` de abajo es cinturón-y-
-- tirantes, no el guardián primario.
--
-- Corrección ADR-0145 (2026-07-06, STORY-038; CONSOLIDADA 2026-07-07,
-- STORY-041/DEBT-016): AMBAS tablas portan el Eje B (realidad del capital,
-- LIVE/PAPER/DEMO/CHALLENGE), ORTOGONAL al Eje A (`scope` -- SOVEREIGN/
-- BROKER_READONLY, quién ejecutó). Una cuenta PAPER/DEMO/CHALLENGE corre en
-- el MISMO entorno determinista de ejecución que LIVE (NO es backtesting) y
-- por tanto SÍ es atestiguable; el Eje B solo etiqueta si el capital
-- arriesgado fue real o virtual, nunca condiciona el Eje A.
--
-- STORY-038 había creado una columna NUEVA `capital_reality` para el Eje B,
-- duplicando el dominio de `institutional_tag` (Grupo II, ya obligatorio en
-- el Perfil D) -- dos columnas con el mismo vocabulario de valores en la
-- misma fila viola "reutilización antes que creación" (ADR-0144 FIJO).
-- Corrección ratificada por el propietario (ADR-0145, 2026-07-07): el Eje B
-- NO es un campo nuevo -- ES `institutional_tag`, con su vocabulario
-- extendido de `PROD`/`PAPER`/`CHALLENGE` a `LIVE`/`PAPER`/`DEMO`/`CHALLENGE`
-- (`LIVE` reemplaza `PROD` como sinónimo más claro en contexto de trading;
-- `DEMO` es valor nuevo del mismo campo). La columna `capital_reality` se
-- ELIMINA de ambas tablas; `institutional_tag` gana el `CHECK` que antes
-- tenía `capital_reality`. Editada IN SITU (fase GREENFIELD, ADR-0006 -- no
-- se crea una migración incremental).
--
-- Perfil ADR-0020 para AMBAS tablas (Perfil D "Ops/Auditoría/Forense",
-- declarado en `docs/features/verified-account-registry.md`
-- "Gobernanza y Estándares" y STORY-037 §3):
--   - Grupo I  (Identidad & Integridad, UNIVERSAL -- `row_version` en
--     `verified_accounts` por ser MUTABLE; `event_sequence_id` en
--     `attested_track_records` por ser APPEND-ONLY, ADR-0141).
--   - Grupo II (Soberanía & Propiedad): owner_id, institutional_tag (en
--     estas DOS tablas, `institutional_tag` porta el Eje B -- realidad de
--     capital -- en vez del vocabulario genérico de otras tablas del
--     substrato; ver corrección ADR-0145 arriba).
--   - Grupo IV (Infraestructura & Ops): node_id.
--   - Subset de Grupo V (Forense/Cumplimiento), SOLO en
--     `attested_track_records`: signature_hash -- firma de integridad
--     REPRODUCIBLE del CONTENIDO del track (distinta de `audit_hash`, que
--     es la integridad de ESTA FILA en el ledger). `verified_accounts` NO
--     lleva `signature_hash`: la cuenta en sí no es un track firmado, solo
--     su registro mutable.
-- El Grupo III (Linaje Alpha & Datos) NO aplica a ninguna de las dos tablas
-- (una cuenta de bróker y un track calculado no tienen linaje genómico) y
-- se omite a propósito (ADR-0020 "Aplicación": "PROHIBIDO copy-paste
-- masivo").
--
-- Columnas propias de `verified_accounts` (fuera del contrato de 25 campos,
-- `docs/features/verified-account-registry.md` "Persistencia"):
--   - broker / leverage / currency / account_type: identifican la cuenta de
--     trading (fondeo/prop/propio).
--   - publication_status: PRIVATE (default FIJO al registrar) | PUBLIC
--     (solo tras opt-in vigente vía consent-registry, #5).
--   - attestation_scopes: lista JSON de los ámbitos de atestación que esta
--     cuenta tiene habilitados (SOVEREIGN y/o BROKER_READONLY, coexistentes)
--     -- `CHECK(json_valid(attestation_scopes))` rechaza JSON corrupto a
--     nivel de BD.
--   - broker_connection_ref: referencia de texto NO SECRETA a la conexión
--     de bróker (nullable) -- las credenciales (investor password/API)
--     siguen en `broker_connections` (cifradas, locales, fuera de esta
--     Story). Guardarraíl ADR-0093: esta columna JAMÁS es una credencial.
--
-- Columnas propias de `attested_track_records`:
--   - verified_account_id: referencia a `verified_accounts.id` (sin FK
--     física -- SQLite STRICT no impone FKs por defecto en este esquema,
--     mismo criterio que el resto del substrato).
--   - scope: SOVEREIGN (ejecución propia, atestada por la cadena de hash) |
--     BROKER_READONLY (cuenta-completa reportada, computada localmente).
--     La distinción es INVIOLABLE (regla obligatoria #1, ADR-0145): nunca
--     se presenta un dato BROKER_READONLY como SOVEREIGN. El Eje B (realidad
--     de capital) NO tiene columna propia aquí -- vive en `institutional_tag`
--     (Grupo II arriba), estampado desde `verified_accounts.institutional_tag`
--     de la cuenta al momento del cálculo -- LIVE | PAPER | DEMO | CHALLENGE.
--     ORTOGONAL a `scope` (Eje A): un track SOVEREIGN + PAPER es válido y
--     atestiguable, pero jamás se presenta sin esta etiqueta de capital
--     virtual.
--   - time_window: la ventana temporal que resume este track (texto libre,
--     ej. "2026-W27" o "2026-Q3" -- el vocabulario de ventana lo fija quien
--     llama, igual que `data-aggregation`).
--   - equity_curve / balance_curve: curvas JSON canónicas (array de pares
--     `[timestamp_ns, valor_e8]`) -- `CHECK(json_valid(...))` rechaza JSON
--     corrupto, defensa en profundidad sobre la serialización determinista
--     del Core.
--   - max_drawdown_e8, gain_pct_e8, win_rate_e8, total_realized_pnl_e8,
--     total_deposits_e8, total_withdrawals_e8: métricas del track, TODAS
--     enteras ×10⁸ (ADR-0141) -- el gain% EXCLUYE depósitos/retiros (regla
--     obligatoria #2, EL diferenciador de cálculo: un depósito NUNCA cuenta
--     como ganancia).
--   - avg_holding_time_ns, trading_days: estadística de trades (INTEGER,
--     no monetaria).
--
-- Guardarraíl ADR-0093: NINGUNA columna de ninguna de las dos tablas puede
-- contener una credencial de bróker, una investor password, ni una IP de
-- servidor live -- `broker_connection_ref` es explícitamente una
-- referencia, no un secreto. El test de integración de este cimiento lo
-- assert explícitamente sobre las filas persistidas.
--
-- Idempotencia (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF
-- NOT EXISTS` / `CREATE TRIGGER IF NOT EXISTS` hacen que volver a correr
-- esta migración sea un no-op.

CREATE TABLE IF NOT EXISTS verified_accounts (
    -- I. Identidad & Integridad (universal, ADR-0020; row_version en vez
    -- de event_sequence_id por ser tabla MUTABLE, ADR-0141).
    id                     TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at             INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock)
    updated_at             INTEGER NOT NULL,             -- Nanosegundos desde epoch; cambia con cada UPDATE
    audit_hash             TEXT    NOT NULL,             -- SHA-256 del contenido de esta versión de fila + enlace previo
    audit_chain_hash       TEXT,                         -- audit_hash de la versión anterior de esta fila (NULL solo en la fila génesis)
    row_version            INTEGER NOT NULL,             -- Contador de versión de esta fila; arranca en 1, +1 en cada UPDATE

    -- II. Soberanía & Propiedad
    owner_id               TEXT    NOT NULL,             -- Dueño Drasus (central-identity, #1) -- multi-cuenta 1:N bajo owner_id, NUNCA tenant_id
    -- Eje B (realidad de capital, corrección ADR-0145 2026-07-07,
    -- STORY-041/DEBT-016): en esta tabla `institutional_tag` NO es el
    -- vocabulario genérico del resto del substrato -- ES el Eje B, valor
    -- ÚNICO por cuenta (no un conjunto como attestation_scopes), ortogonal
    -- al Eje A (scope de attested_track_records). LIVE (capital real) |
    -- PAPER | DEMO | CHALLENGE (capital virtual, mismo entorno determinista
    -- de ejecución que LIVE -- NO backtesting).
    institutional_tag      TEXT    NOT NULL
        CHECK (institutional_tag IN ('LIVE', 'PAPER', 'DEMO', 'CHALLENGE')),

    -- IV. Infraestructura & Ops
    node_id                TEXT    NOT NULL,             -- Máquina que registró la cuenta

    -- Columnas propias de la Feature (verified-account-registry.md "Persistencia")
    broker                 TEXT    NOT NULL,             -- Bróker/venue de la cuenta (ICMarkets, Binance, IBKR, ...)
    leverage               INTEGER NOT NULL,             -- Apalancamiento de la cuenta
    currency               TEXT    NOT NULL,             -- Divisa base de la cuenta
    account_type           TEXT    NOT NULL              -- FUNDED | PROP | OWN
        CHECK (account_type IN ('FUNDED', 'PROP', 'OWN')),
    publication_status     TEXT    NOT NULL              -- PRIVATE (default FIJO) | PUBLIC (solo tras opt-in real de #5)
        CHECK (publication_status IN ('PRIVATE', 'PUBLIC')),
    -- Lista JSON de ámbitos de atestación habilitados (SOVEREIGN y/o
    -- BROKER_READONLY, coexistentes) -- serializada desde un conjunto
    -- ordenado determinista (ADR-0002/0004).
    attestation_scopes     TEXT    NOT NULL CHECK (json_valid(attestation_scopes)),
    -- Referencia NO SECRETA a la conexión de bróker (broker_connections,
    -- cifrada, local -- fuera de esta Story). Nullable: no toda cuenta
    -- tiene ya una conexión read-only vinculada.
    broker_connection_ref  TEXT,

    -- FK física owner_id -> accounts(id) (ADR-0141 enmienda 2026-07-11, M6):
    -- el dueño Drasus de la cuenta verificada DEBE existir en `accounts`.
    -- RESTRICT: nunca se borra una cuenta con cuentas verificadas propias.
    FOREIGN KEY (owner_id) REFERENCES accounts (id) ON DELETE RESTRICT
) STRICT;

-- Query path de "todas las cuentas de este dueño" (panel de cuentas
-- verificadas, consistente con el resto de tablas del substrato).
CREATE INDEX IF NOT EXISTS idx_verified_accounts_owner_id
    ON verified_accounts (owner_id);

CREATE TABLE IF NOT EXISTS attested_track_records (
    -- I. Identidad & Integridad (universal, ADR-0020; event_sequence_id
    -- en vez de row_version por ser tabla APPEND-ONLY, ADR-0141).
    id                    TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock) -- instante de PERSISTENCIA
    updated_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch; append-only => siempre igual a created_at
    audit_hash            TEXT    NOT NULL,             -- SHA-256 del contenido de ESTA FILA + enlace previo (integridad del ledger)
    audit_chain_hash      TEXT,                         -- audit_hash de la fila anterior (NULL solo en la fila génesis)
    event_sequence_id     INTEGER NOT NULL UNIQUE,      -- Posición monótona en la cadena GLOBAL (1, 2, 3, ...)

    -- II. Soberanía & Propiedad
    owner_id              TEXT    NOT NULL,             -- Dueño del track (viene de central-identity)
    -- Eje B (realidad de capital, corrección ADR-0145 2026-07-07,
    -- STORY-041/DEBT-016): en esta tabla `institutional_tag` NO es el
    -- vocabulario genérico del resto del substrato -- ES el Eje B,
    -- estampado desde `verified_accounts.institutional_tag` de la cuenta al
    -- momento del cálculo -- ORTOGONAL al `scope` de abajo (Eje A). LIVE |
    -- PAPER | DEMO | CHALLENGE.
    institutional_tag     TEXT    NOT NULL
        CHECK (institutional_tag IN ('LIVE', 'PAPER', 'DEMO', 'CHALLENGE')),

    -- IV. Infraestructura & Ops
    node_id               TEXT    NOT NULL,             -- Máquina que calculó el track

    -- Subset de V. Forense & Ejecución (Gobernanza/Cumplimiento)
    -- Firma de integridad REPRODUCIBLE del CONTENIDO del track (distinta de
    -- audit_hash, que protege la fila del ledger, no el contenido).
    signature_hash        TEXT    NOT NULL,

    -- Columnas propias de la Feature (verified-account-registry.md "Persistencia")
    verified_account_id   TEXT    NOT NULL,             -- Referencia a verified_accounts.id
    scope                 TEXT    NOT NULL              -- SOVEREIGN | BROKER_READONLY -- distinción INVIOLABLE (ADR-0145)
        CHECK (scope IN ('SOVEREIGN', 'BROKER_READONLY')),
    time_window            TEXT    NOT NULL,             -- Ventana temporal que resume este track (ej. "2026-W27")
    -- Curvas canónicas JSON (array de pares [timestamp_ns, valor_e8]).
    equity_curve           TEXT    NOT NULL CHECK (json_valid(equity_curve)),
    balance_curve          TEXT    NOT NULL CHECK (json_valid(balance_curve)),
    -- Métricas del track, TODAS enteras ×10⁸ (ADR-0141) -- gain_pct_e8
    -- EXCLUYE depósitos/retiros (regla obligatoria #2, ADR-0145).
    max_drawdown_e8        INTEGER NOT NULL,
    gain_pct_e8            INTEGER NOT NULL,
    win_rate_e8            INTEGER NOT NULL,
    avg_holding_time_ns    INTEGER NOT NULL,
    trading_days           INTEGER NOT NULL,
    total_realized_pnl_e8  INTEGER NOT NULL,
    total_deposits_e8      INTEGER NOT NULL,
    total_withdrawals_e8   INTEGER NOT NULL,

    -- FK física owner_id -> accounts(id) (ADR-0141 enmienda 2026-07-11, M6):
    -- el dueño del track DEBE existir en `accounts`. RESTRICT: nunca se
    -- borra una cuenta con tracks atestados asociados.
    FOREIGN KEY (owner_id) REFERENCES accounts (id) ON DELETE RESTRICT
) STRICT;

-- Índice de la posición en la cadena (ADR-0141 M8: "Índice obligatorio en
-- event_sequence_id en tablas append-only") -- acceso cronológico / replay.
CREATE INDEX IF NOT EXISTS idx_attested_track_records_event_sequence_id
    ON attested_track_records (event_sequence_id);

-- Query path de "todos los tracks de esta cuenta verificada" (panel de
-- cuentas verificadas: historial de tracks calculados por cuenta).
CREATE INDEX IF NOT EXISTS idx_attested_track_records_verified_account_id
    ON attested_track_records (verified_account_id);

-- Query path de "todos los tracks de este dueño".
CREATE INDEX IF NOT EXISTS idx_attested_track_records_owner_id
    ON attested_track_records (owner_id);

-- Enforzamiento append-only: rechaza UPDATE (verified-account-registry.md:
-- cada track calculado es un snapshot inmutable firmado).
CREATE TRIGGER IF NOT EXISTS trg_attested_track_records_no_update
BEFORE UPDATE ON attested_track_records
BEGIN
    SELECT RAISE(ABORT, 'attested_track_records is append-only: UPDATE is forbidden');
END;

-- Enforzamiento append-only: rechaza DELETE.
CREATE TRIGGER IF NOT EXISTS trg_attested_track_records_no_delete
BEFORE DELETE ON attested_track_records
BEGIN
    SELECT RAISE(ABORT, 'attested_track_records is append-only: DELETE is forbidden');
END;
