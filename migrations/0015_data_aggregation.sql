-- Migración 0015: Data Anonymization & Aggregation
-- (docs/features/data-aggregation.md, ADR-0144 cimiento #9, ADR-0102,
-- ADR-0143, ADR-0137, ADR-0141, ADR-0020, ADR-0093, STORY-036)
--
-- Crea UNA tabla: el noveno cimiento del substrato de monetización
-- (ADR-0144). Cada fila es un SNAPSHOT INMUTABLE de un índice agregado --
-- una métrica anonimizada (ruido de privacidad diferencial) que resume
-- una cohorte de N usuarios sin exponer a ninguno individualmente. Por
-- eso la tabla es APPEND-ONLY, con la misma naturaleza que
-- `domain_events` (migración `0012_domain_events.sql`) y
-- `consent_records` (migración `0011_consent_registry.sql`):
-- `event_sequence_id INTEGER NOT NULL UNIQUE` + triggers anti
-- UPDATE/DELETE + `audit_chain_hash` encadenado.
--
-- Guardarraíl ADR-0093/0102: ninguna columna de esta tabla guarda un
-- balance en dólares crudo, una IP de servidor live, una llave, ni los
-- parámetros/fórmulas exactos de una estrategia -- solo la métrica YA
-- anonimizada (con ruido), el tamaño de la cohorte que la respalda y el
-- canal de destino. La topología de estrategia, si participó en el
-- cálculo, se hashea (SHA-256) ANTES de llegar aquí y el hash en sí
-- tampoco se persiste en esta tabla -- solo sirvió para agrupar, nunca
-- para identificar.
--
-- Perfil ADR-0020 para esta tabla (Perfil B "IA/R&D", acotado por el Gate
-- de Coherencia de STORY-036 §3 al subset owner_id + institutional_tag de
-- Grupo II, node_id de Grupo IV, y data_snapshot_id de Grupo III):
--   - Grupo I  (Identidad & Integridad, universal, con event_sequence_id
--     por ser tabla APPEND-ONLY, ADR-0141): id, created_at, updated_at,
--     audit_hash, audit_chain_hash, event_sequence_id.
--   - Grupo II (subset): owner_id, institutional_tag. NOTA: aquí
--     `owner_id`/`institutional_tag` identifican al PROCESO/AGREGADOR que
--     calculó y publicó este índice (el substrato de Drasus), NUNCA a un
--     usuario contribuyente individual de la cohorte -- eso sería
--     exactamente el dato crudo identificable que ADR-0093/0102 prohíben
--     exponer. Es la misma semántica de "dueño del artefacto derivado"
--     que ya usa `institutional-report-engine` para sus reportes.
--   - Grupo III (subset): data_snapshot_id -- referencia de linaje al
--     conjunto de eventos fuente que alimentó este agregado (nullable:
--     un agregado sembrado sin conjunto fuente identificado, ej. en
--     pruebas, no lo tiene).
--   - Grupo IV (subset): node_id -- máquina que calculó el agregado.
--
-- Columnas propias de la Feature (`docs/features/data-aggregation.md`
-- "Persistencia"):
--   - index_type: tipo de índice vendible -- SENTIMENT, REGIME,
--     BROKER_FRICTION o CORRELATION.
--   - time_window: ventana temporal que resume este agregado (texto
--     libre, ej. '2026-W27').
--   - cohort_size: cuántos contribuyentes distintos respaldan este
--     agregado -- SIEMPRE >= MIN_COHORT_SIZE (k-anonimato, verificado por
--     el Core antes de llegar a esta tabla; un agregado con cohorte
--     insuficiente NUNCA se persiste, se suprime en memoria).
--   - noise_level: nivel de ruido de privacidad diferencial aplicado,
--     entero escalado ×10⁸ (ADR-0141) -- NUNCA una columna REAL.
--   - metric_value: el valor de la métrica agregada, YA con el ruido
--     aplicado, entero escalado ×10⁸ (ADR-0141) -- NUNCA una columna
--     REAL. Es el único valor "de negocio" que expone esta fila.
--   - channel: INTERNAL (uso interno lícito del tier gratuito, ADR-0143)
--     o EXTERNAL (candidato a venta a terceros, requiere consentimiento
--     vigente Y `EXTERNAL_SALE_ENABLED=true`).
--
-- Idempotencia (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX
-- IF NOT EXISTS` / `CREATE TRIGGER IF NOT EXISTS` hacen que volver a
-- correr esta migración sea un no-op.

CREATE TABLE IF NOT EXISTS aggregated_indexes (
    -- I. Identidad & Integridad (universal, ADR-0020; event_sequence_id
    -- en vez de row_version por ser tabla APPEND-ONLY, ADR-0141).
    id                    TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock)
    updated_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch; append-only => siempre igual a created_at
    audit_hash            TEXT    NOT NULL,             -- SHA-256 del contenido de esta fila + enlace previo
    audit_chain_hash      TEXT,                         -- audit_hash de la fila anterior (NULL solo en la fila génesis)
    event_sequence_id     INTEGER NOT NULL UNIQUE,      -- Posición monótona en la cadena global (1, 2, 3, ...)

    -- II. Soberanía & Propiedad (subset -- dueño del ARTEFACTO derivado, nunca de un usuario contribuyente)
    owner_id              TEXT    NOT NULL,
    institutional_tag     TEXT    NOT NULL,

    -- III. Linaje Alpha & Datos (subset -- referencia al conjunto fuente)
    data_snapshot_id       TEXT,

    -- IV. Infraestructura & Ops (subset)
    node_id               TEXT    NOT NULL,             -- Máquina que calculó este agregado

    -- Columnas propias de la Feature (docs/features/data-aggregation.md "Persistencia")
    -- Tipo de índice vendible.
    index_type             TEXT    NOT NULL CHECK (index_type IN ('SENTIMENT', 'REGIME', 'BROKER_FRICTION', 'CORRELATION')),
    -- Ventana temporal que resume este agregado (texto libre, ej. '2026-W27').
    time_window            TEXT    NOT NULL,
    -- Tamaño de la cohorte que respalda este agregado -- SIEMPRE >= MIN_COHORT_SIZE (k-anonimato).
    cohort_size            INTEGER NOT NULL CHECK (cohort_size > 0),
    -- Nivel de ruido de privacidad diferencial aplicado, entero ×10⁸ (ADR-0141). NUNCA REAL.
    noise_level            INTEGER NOT NULL,
    -- Valor de la métrica agregada, YA con ruido aplicado, entero ×10⁸ (ADR-0141). NUNCA REAL.
    metric_value           INTEGER NOT NULL,
    -- Canal de destino de este agregado.
    channel                TEXT    NOT NULL CHECK (channel IN ('INTERNAL', 'EXTERNAL'))
) STRICT;

-- Índice de la posición en la cadena (ADR-0141 M8: "Índice obligatorio en
-- event_sequence_id en tablas append-only") -- acceso cronológico / replay.
CREATE INDEX IF NOT EXISTS idx_aggregated_indexes_event_sequence_id
    ON aggregated_indexes (event_sequence_id);

-- Query path principal (docs/features/data-aggregation.md "Comportamientos
-- Observables": "cuando se consulta un índice agregado"): servir las
-- consultas por tipo de índice + ventana temporal sin escanear toda la
-- tabla.
CREATE INDEX IF NOT EXISTS idx_aggregated_indexes_type_window
    ON aggregated_indexes (index_type, time_window);

-- Enforzamiento append-only: rechaza UPDATE.
CREATE TRIGGER IF NOT EXISTS trg_aggregated_indexes_no_update
BEFORE UPDATE ON aggregated_indexes
BEGIN
    SELECT RAISE(ABORT, 'aggregated_indexes is append-only: UPDATE is forbidden');
END;

-- Enforzamiento append-only: rechaza DELETE.
CREATE TRIGGER IF NOT EXISTS trg_aggregated_indexes_no_delete
BEFORE DELETE ON aggregated_indexes
BEGIN
    SELECT RAISE(ABORT, 'aggregated_indexes is append-only: DELETE is forbidden');
END;
