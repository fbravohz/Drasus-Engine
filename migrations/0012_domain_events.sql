-- Migración 0012: Enriched Domain Events / Eventos de Dominio Enriquecidos
-- (docs/features/enriched-domain-events.md, ADR-0144 cimiento #6, ADR-0145
-- enriquecimiento, ADR-0143, ADR-0141, ADR-0020, ADR-0093, STORY-033)
--
-- Crea la tabla `domain_events`: la raíz del substrato de monetización
-- (ADR-0144, cimiento #6). Es un event-store HETEROGÉNEO -- una sola tabla
-- guarda TODOS los tipos de evento de dominio (orden ejecutada, flujo de
-- capital, snapshot de cuenta, backtest completado, régimen, drawdown,
-- estrés de liquidez, cambio de correlación). No hay una tabla por tipo de
-- evento: cada fila trae `event_type` (qué variante es) + `payload` (el
-- contenido específico de esa variante, como JSON canónico). El Core en
-- Rust (`domain::enriched_domain_events::EnrichedDomainEvent`) es el enum
-- que produce ese payload de forma determinista.
--
-- APPEND-ONLY, NO mutable (docs/features/enriched-domain-events.md
-- "Restricciones": "Los eventos son inmutables (append-only, encadenados
-- con audit_chain_hash)"). Por ADR-0141 ("PROHIBIDO usar el mismo nombre
-- `event_sequence_id` para lo que es `row_version` y viceversa"), esta
-- tabla usa `event_sequence_id INTEGER NOT NULL UNIQUE` (posición
-- monótona GLOBAL en la secuencia, no por owner) -- mismo patrón que
-- `audit_events` (migración `0002_audit_log.sql`), `usage_records`
-- (migración `0010_usage_metering.sql`) y `consent_records` (migración
-- `0011_consent_registry.sql`).
--
-- Regla nueva "Atomicidad de ledgers append-only" (rust-engineer/SKILL.md
-- §4, causa raíz DEBT-001): el `event_sequence_id` de cada fila nueva se
-- deriva DENTRO de una transacción `BEGIN IMMEDIATE` (ver
-- `persistence::enriched_domain_events::DomainEventRepository`), nunca en
-- sentencias sueltas -- el `UNIQUE` de abajo es cinturón-y-tirantes, no el
-- guardián primario.
--
-- Perfil ADR-0020 para esta tabla (Perfil D "Ops/Auditoría", declarado en
-- `docs/features/enriched-domain-events.md` §"Gobernanza y Estándares" y
-- STORY-033 §3):
--   - Grupo I  (Identidad & Integridad, UNIVERSAL, con event_sequence_id
--     en vez de row_version por ser tabla APPEND-ONLY, ADR-0141): id,
--     created_at, updated_at, audit_hash, audit_chain_hash,
--     event_sequence_id.
--   - Grupo II (Soberanía & Propiedad): owner_id, institutional_tag.
--   - Grupo IV (Infraestructura & Ops): node_id, process_id (obligatorio,
--     ancla el evento al proceso del motor que lo emitió), session_id
--     (nullable -- no todo evento ocurre dentro de una sesión de
--     ejecución agrupable).
-- Los Grupos III y V NO aplican a esta tabla (un evento de dominio no
-- tiene linaje genómico ni es un veredicto forense de riesgo por sí
-- mismo) y se omiten a propósito (ADR-0020 "Aplicación": "PROHIBIDO
-- copy-paste masivo").
--
-- Columnas propias de la Feature (fuera del contrato de 25 campos,
-- `docs/features/enriched-domain-events.md` "Persistencia"):
--   - event_type: qué variante de EnrichedDomainEvent es esta fila. El
--     catálogo cubre la orden-con-fricción reforzada (ADR-0145) más los
--     dos eventos nuevos de ADR-0145 (flujo de capital, snapshot de
--     cuenta) más los cuatro ya previstos por ADR-0144 (backtest, régimen,
--     drawdown, liquidez, correlación).
--   - payload: contenido JSON canónico específico de esa variante --
--     `CHECK(json_valid(payload))` rechaza JSON corrupto a nivel de BD,
--     defensa en profundidad sobre la serialización determinista del
--     Core (BTreeMap ordenado, mismo patrón que `consent_registry`).
--   - replicate: la decisión de si este evento se replica hacia la Cabina
--     de Mando del proveedor (0/1), derivada del `ExecutionGate` real de
--     `licensing-system` (#2, ADR-0143: "gratis" nunca suprime -> replica;
--     "pago al corriente" suprime -> NO replica). Esta columna es SOLO el
--     flag calculado -- el envío real por red es un adaptador futuro
--     diferido (ver banner de deudas de la Orden STORY-033 §8); esta
--     migración no incluye tabla de cola de envío.
--
-- Guardarraíl ADR-0093: esta tabla JAMÁS almacena secretos -- ninguna
-- columna ni el contenido de `payload` puede incluir credenciales de
-- bróker, claves de firma ni IPs de servidores live. El Core que produce
-- el payload solo modela campos de negocio (instrumento, montos, cuenta,
-- métricas); el test de integración de este cimiento lo assert
-- explícitamente sobre el JSON persistido.
--
-- Idempotencia (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF
-- NOT EXISTS` / `CREATE TRIGGER IF NOT EXISTS` hacen que volver a correr
-- esta migración sea un no-op.

CREATE TABLE IF NOT EXISTS domain_events (
    -- I. Identidad & Integridad (universal, ADR-0020; event_sequence_id
    -- en vez de row_version por ser tabla APPEND-ONLY, ADR-0141).
    id                    TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock) -- instante de PERSISTENCIA
    updated_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch; append-only => siempre igual a created_at
    audit_hash            TEXT    NOT NULL,             -- SHA-256 del contenido de esta fila + enlace previo
    audit_chain_hash      TEXT,                         -- audit_hash de la fila anterior (NULL solo en la fila génesis)
    event_sequence_id     INTEGER NOT NULL UNIQUE,      -- Posición monótona en la cadena GLOBAL (1, 2, 3, ...)

    -- II. Soberanía & Propiedad
    owner_id              TEXT    NOT NULL,             -- Dueño del evento (viene de central-identity)
    institutional_tag     TEXT    NOT NULL,             -- Entorno/etiqueta institucional

    -- IV. Infraestructura & Ops
    node_id               TEXT    NOT NULL,             -- Máquina que emitió el evento
    process_id            TEXT    NOT NULL,             -- Proceso del motor que emitió el evento (ancla de job)
    session_id            TEXT,                         -- Sesión de ejecución que agrupa el evento (nullable)

    -- Columnas propias de la Feature (docs/features/enriched-domain-events.md "Persistencia")
    -- Qué variante de EnrichedDomainEvent es esta fila -- catálogo completo
    -- (orden reforzada ADR-0145 + los dos eventos nuevos ADR-0145 + los
    -- cuatro previstos por ADR-0144).
    event_type            TEXT    NOT NULL CHECK (event_type IN (
                               'ORDER_EXECUTED',
                               'CAPITAL_FLOW',
                               'ACCOUNT_SNAPSHOT',
                               'BACKTEST_COMPLETED',
                               'REGIME_DETECTED',
                               'DRAWDOWN_DETECTED',
                               'LIQUIDITY_STRESS',
                               'CORRELATION_CHANGE'
                           )),
    -- Contenido JSON canónico específico de la variante (claves ordenadas,
    -- serializado desde un BTreeMap -- determinista, ADR-0002/0004).
    payload                TEXT    NOT NULL CHECK (json_valid(payload)),
    -- Decisión de replicación hacia la Cabina de Mando (0 = no replica /
    -- solo local, 1 = replica), derivada del ExecutionGate real de
    -- licensing-system (#2) en el momento de persistir este evento.
    replicate              INTEGER NOT NULL CHECK (replicate IN (0, 1))
) STRICT;

-- Índice de la posición en la cadena (ADR-0141 M8: "Índice obligatorio en
-- event_sequence_id en tablas append-only") -- acceso cronológico / replay.
CREATE INDEX IF NOT EXISTS idx_domain_events_event_sequence_id
    ON domain_events (event_sequence_id);

-- Query path de "todos los eventos de tipo X" (ej. reconstruir el libro de
-- flujos de capital de una cuenta, o todas las órdenes ejecutadas).
CREATE INDEX IF NOT EXISTS idx_domain_events_event_type
    ON domain_events (event_type);

-- Query path de "todos los eventos de este dueño" (agregación / reportes
-- por owner_id, consistente con el resto de tablas del substrato).
CREATE INDEX IF NOT EXISTS idx_domain_events_owner_id
    ON domain_events (owner_id);

-- Enforzamiento append-only: rechaza UPDATE (enriched-domain-events.md
-- "Restricciones": "Los eventos son inmutables").
CREATE TRIGGER IF NOT EXISTS trg_domain_events_no_update
BEFORE UPDATE ON domain_events
BEGIN
    SELECT RAISE(ABORT, 'domain_events is append-only: UPDATE is forbidden');
END;

-- Enforzamiento append-only: rechaza DELETE.
CREATE TRIGGER IF NOT EXISTS trg_domain_events_no_delete
BEFORE DELETE ON domain_events
BEGIN
    SELECT RAISE(ABORT, 'domain_events is append-only: DELETE is forbidden');
END;
