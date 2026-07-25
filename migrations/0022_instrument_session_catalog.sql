-- Migration 0022: Catálogo de Instrumento × Bróker × Sesión
-- (docs/features/instrument-catalog.md, docs/features/session-catalog.md,
--  ADR-0162, ADR-0141, ADR-0140, ADR-0155, ADR-0020, STORY-066)
--
-- Activa el registro concreto `Instrument` (ADR-0140) con jerarquía de tres
-- niveles y herencia por NULL explícito (ADR-0162): `instrument_master`
-- (canónico, una fila por símbolo) -> `broker_instrument_terms` (override
-- sparse por bróker) -> `session_template`/`session_template_window`
-- (plantillas de calendario compartidas, nunca copiadas por bróker×símbolo).
--
-- Orden de creación: las tablas de `session-catalog` van primero porque
-- `instrument_master.default_session_template_id` las referencia por FK.
--
-- Perfil ADR-0020 para las CUATRO tablas (Perfil A, "Grupo I aplicado"
-- declarado en ADR-0162 antes de listar el esquema de cada tabla): son
-- Referencia/Mutable, NO event-stores append-only -- el campo de secuencia
-- del Grupo I es `row_version` (arranca en 1, +1 en cada UPDATE), NUNCA
-- `event_sequence_id` (ADR-0141 "Semántica de secuencias", corrección
-- explícita del Architect en ADR-0162). `audit_chain_hash` es NULL en la
-- fila génesis de las cuatro tablas -- sin sentinel "genesis" (ADR-0141 M10).
-- Grupo IV (`node_id`/`process_id`, huella de hardware/proceso que hizo el
-- alta/edición) también aplica a las cuatro -- declarado en la sección
-- "Persistencia" de ambos documentos de feature.
--
-- Idempotencia (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF
-- NOT EXISTS` -- volver a correr esta migración es un no-op.

-- ════════════════════════════════════════════════════════════════════════
-- session_template — Referencia, compartida, nunca copiada por bróker×símbolo.
-- Propiedad de `session-catalog`. Una fila por FORMA DE MERCADO
-- (`FOREX_245`, `CRYPTO_247`, `NYSE_REGULAR`...).
-- ════════════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS session_template (
    -- Grupo I: Identidad & Integridad (universal, ADR-0020; row_version
    -- porque esta tabla es MUTABLE, no un event-store -- ADR-0141).
    id                   TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at           INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock)
    updated_at           INTEGER NOT NULL,             -- Nanosegundos desde epoch; cambia con cada edición
    audit_hash           TEXT    NOT NULL,              -- SHA-256 de esta versión de la fila
    audit_chain_hash     TEXT,                          -- audit_hash de la versión anterior; NULL en la fila génesis
    row_version          INTEGER NOT NULL,              -- Contador de versión; arranca en 1, +1 en cada UPDATE

    -- Grupo IV: Infraestructura & Ops (huella de quién hizo el alta/edición).
    node_id              TEXT,                          -- Huella del hardware que registró la escritura
    process_id           TEXT,                          -- PID del proceso que ejecutó la escritura

    -- Columnas propias de dominio.
    template_code        TEXT    NOT NULL UNIQUE,       -- Ej. "FOREX_245", "CRYPTO_247" -- clave de negocio de la plantilla
    display_name         TEXT    NOT NULL,               -- Etiqueta legible para el usuario
    -- Identificador IANA (ej. "America/New_York"), NUNCA una convención de
    -- desplazamiento inventada tipo "EST+07" (Corrección del Architect #3,
    -- ADR-0162). La conversión de zona ocurre en tiempo de LECTURA -- este
    -- campo nunca guarda un UTC ya convertido.
    market_timezone_ref  TEXT    NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_session_template_node_id
    ON session_template (node_id);

-- ════════════════════════════════════════════════════════════════════════
-- session_template_window — Referencia, hijas de `session_template`.
-- Propiedad de `session-catalog`. Cada fila es una ventana semanal de
-- apertura de mercado, en hora de `market_timezone_ref` del padre.
-- ════════════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS session_template_window (
    -- Grupo I
    id                   TEXT    NOT NULL PRIMARY KEY,
    created_at           INTEGER NOT NULL,
    updated_at           INTEGER NOT NULL,
    audit_hash           TEXT    NOT NULL,
    audit_chain_hash     TEXT,
    row_version          INTEGER NOT NULL,

    -- Grupo IV
    node_id              TEXT,
    process_id           TEXT,

    -- Columnas propias de dominio.
    session_template_id  TEXT    NOT NULL               -- Padre dueño de esta ventana
        REFERENCES session_template (id) ON DELETE RESTRICT,
    -- Día de inicio/fin de la ventana (vocabulario FIJO de 3 letras).
    start_day            TEXT    NOT NULL
        CHECK (start_day IN ('MON','TUE','WED','THU','FRI','SAT','SUN')),
    end_day              TEXT    NOT NULL
        CHECK (end_day IN ('MON','TUE','WED','THU','FRI','SAT','SUN')),
    -- Minutos desde medianoche (0-1439), hora de `market_timezone_ref` del padre.
    start_time_minutes   INTEGER NOT NULL CHECK (start_time_minutes BETWEEN 0 AND 1439),
    end_time_minutes     INTEGER NOT NULL CHECK (end_time_minutes BETWEEN 0 AND 1439)
) STRICT;

-- Índice de FK hijo (ADR-0141 M7): toda columna FK necesita su índice.
CREATE INDEX IF NOT EXISTS idx_session_template_window_session_template_id
    ON session_template_window (session_template_id);

CREATE INDEX IF NOT EXISTS idx_session_template_window_node_id
    ON session_template_window (node_id);

-- ════════════════════════════════════════════════════════════════════════
-- instrument_master — Referencia/Mutable, una fila por símbolo, sin bróker.
-- Propiedad de `instrument-catalog`. Activa el registro `Instrument`
-- (ADR-0140) con los campos generales de trading + los atributos de
-- Opciones/Futuros ya fijados por ADR-0140/ADR-0155 (este ADR no los
-- reinventa, los aloja en la misma tabla que ahora se activa).
-- ════════════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS instrument_master (
    -- Grupo I
    id                          TEXT    NOT NULL PRIMARY KEY, -- Clave TÉCNICA, UUIDv7 (ADR-0141 PK universal)
    created_at                  INTEGER NOT NULL,
    updated_at                  INTEGER NOT NULL,
    audit_hash                  TEXT    NOT NULL,
    audit_chain_hash            TEXT,
    row_version                 INTEGER NOT NULL,

    -- Grupo IV
    node_id                     TEXT,
    process_id                  TEXT,

    -- Identidad de NEGOCIO (Corrección del Architect #1, ADR-0162): la
    -- misma cadena ("EURUSD") que ya circula en QuoteTick/TradeTick/Bars
    -- (ADR-0155 Restricción 8). `broker_instrument_terms` referencia ESTA
    -- columna por FK, nunca el UUID técnico `id`.
    instrument_id                TEXT    NOT NULL UNIQUE,
    asset_class                  TEXT    NOT NULL
        CHECK (asset_class IN ('FOREX','FUTURE','CFD','CRYPTO','EQUITY')),
    base_currency                 TEXT,                        -- Forex/CFD; NULL si no aplica
    quote_currency                TEXT,                        -- Forex/CFD; NULL si no aplica
    -- Convención de mercado (no de un bróker concreto), INTEGER ×10⁸ (ADR-0141 M1).
    canonical_pip_size            INTEGER NOT NULL CHECK (canonical_pip_size >= 0),
    canonical_tick_size           INTEGER NOT NULL CHECK (canonical_tick_size >= 0),
    -- Calendario por defecto del instrumento; NULL = sin plantilla asignada
    -- todavía. `broker_instrument_terms.session_template_id_override` puede
    -- sobreescribirlo por bróker concreto.
    default_session_template_id   TEXT
        REFERENCES session_template (id) ON DELETE RESTRICT,

    -- Atributos de Opciones/Futuros (ADR-0140/ADR-0155 §"Atributos
    -- estáticos de Opciones y Futuros"; ADR-0162 los aloja sin reinventar
    -- el vocabulario). Todos NULL cuando `asset_class` no los necesita.
    strike_price                  INTEGER,                      -- ×10⁸
    expiration_date                INTEGER,                     -- Nanosegundos UTC
    option_type                    TEXT
        CHECK (option_type IS NULL OR option_type IN ('CALL','PUT')),
    option_style                    TEXT
        CHECK (option_style IS NULL OR option_style IN ('AMERICAN','EUROPEAN','OTHER')),
    contract_multiplier             INTEGER,                    -- Contador plano (típicamente 100), SIN escala ×10⁸
    occ_symbol                      TEXT,                       -- Formato OSI/OCC de 21 caracteres
    contract_code                   TEXT,                       -- Futuros
    contract_expiration_date        INTEGER                     -- Futuros, nanosegundos UTC
) STRICT;

CREATE INDEX IF NOT EXISTS idx_instrument_master_node_id
    ON instrument_master (node_id);

-- Índice de FK hijo (ADR-0141 M7).
CREATE INDEX IF NOT EXISTS idx_instrument_master_default_session_template_id
    ON instrument_master (default_session_template_id);

-- ════════════════════════════════════════════════════════════════════════
-- broker_instrument_terms — Referencia/Mutable, sparse: una fila SOLO
-- cuando un bróker concreto ofrece el instrumento. Propiedad de
-- `instrument-catalog`. Todo campo propio es NULLABLE por diseño: NULL
-- significa "hereda el valor canónico de instrument_master", NUNCA "dato
-- faltante por error" (Restricción FIJA #1 de ADR-0162).
-- ════════════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS broker_instrument_terms (
    -- Grupo I
    id                       TEXT    NOT NULL PRIMARY KEY,
    created_at               INTEGER NOT NULL,
    updated_at               INTEGER NOT NULL,
    audit_hash               TEXT    NOT NULL,
    audit_chain_hash         TEXT,
    row_version               INTEGER NOT NULL,

    -- Grupo IV
    node_id                   TEXT,
    process_id                 TEXT,

    -- Identidad de la fila override: referencia la clave de NEGOCIO de
    -- `instrument_master` (Corrección del Architect #1), nunca su UUID técnico.
    instrument_id               TEXT    NOT NULL
        REFERENCES instrument_master (instrument_id) ON DELETE RESTRICT,
    venue                       TEXT    NOT NULL,               -- Bróker/venue
    venue_symbol                 TEXT,                          -- El `broker_symbol` que `data-normalization-layer` ya declaraba

    -- Términos de trading -- TODOS nullable = herencia del canónico.
    default_spread                INTEGER CHECK (default_spread IS NULL OR default_spread >= 0),     -- ×10⁸, piso declarado (Política de Integridad de Spread)
    default_slippage               INTEGER CHECK (default_slippage IS NULL OR default_slippage >= 0), -- ×10⁸
    pip_size_override               INTEGER CHECK (pip_size_override IS NULL OR pip_size_override >= 0),   -- ×10⁸
    tick_size_override               INTEGER CHECK (tick_size_override IS NULL OR tick_size_override >= 0), -- ×10⁸
    point_value                       INTEGER CHECK (point_value IS NULL OR point_value >= 0),             -- ×10⁸
    order_size_multiplier              INTEGER CHECK (order_size_multiplier IS NULL OR order_size_multiplier >= 0), -- Contador plano, SIN escala
    order_size_step                     INTEGER CHECK (order_size_step IS NULL OR order_size_step >= 0),   -- ×10⁸ (tamaño de lote fraccionario)
    min_distance                         INTEGER CHECK (min_distance IS NULL OR min_distance >= 0),        -- ×10⁸

    commission_model                      TEXT
        CHECK (commission_model IS NULL OR commission_model IN ('NONE','PER_TRADE','PERCENTAGE','SIZE_BASED','STOCKPICKER')),
    commission_value                       INTEGER CHECK (commission_value IS NULL OR commission_value >= 0), -- ×10⁸

    -- Vocabulario de `swap_type` DIFERIDO -- sin fuente verificada todavía
    -- (Restricción explícita de ADR-0162: NO se inventa un CHECK sin
    -- fuente). Queda TEXT libre a propósito.
    swap_type                                TEXT,
    swap_long_points                          INTEGER,          -- ×10⁸, puntos de swap (signo permite largo/corto negativo)
    swap_short_points                          INTEGER,         -- ×10⁸
    -- Día de triple swap: mismo vocabulario de día que las ventanas de sesión.
    triple_swap_day                             TEXT
        CHECK (triple_swap_day IS NULL OR triple_swap_day IN ('MON','TUE','WED','THU','FRI','SAT','SUN')),
    swap_rollout_hour                            INTEGER CHECK (swap_rollout_hour IS NULL OR swap_rollout_hour BETWEEN 0 AND 23),

    -- Override de calendario -- solo cuando el calendario de ESTE bróker
    -- difiere del calendario por defecto del instrumento.
    session_template_id_override                  TEXT
        REFERENCES session_template (id) ON DELETE RESTRICT,

    -- Un bróker concreto tiene UNA fila de override por instrumento.
    UNIQUE (instrument_id, venue)
) STRICT;

-- Índices de FK hijo (ADR-0141 M7).
CREATE INDEX IF NOT EXISTS idx_broker_instrument_terms_instrument_id
    ON broker_instrument_terms (instrument_id);
CREATE INDEX IF NOT EXISTS idx_broker_instrument_terms_session_template_id_override
    ON broker_instrument_terms (session_template_id_override);
CREATE INDEX IF NOT EXISTS idx_broker_instrument_terms_node_id
    ON broker_instrument_terms (node_id);
