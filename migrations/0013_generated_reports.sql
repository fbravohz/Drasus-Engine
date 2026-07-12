-- Migración 0013: Institutional Report Engine / Motor de Reportes
-- Institucionales (docs/features/institutional-report-engine.md,
-- ADR-0144 cimiento #7, ADR-0101 plantillas Tera diferidas, ADR-0027
-- trazabilidad al audit-log, ADR-0141, ADR-0020, ADR-0093, STORY-034)
--
-- Crea la tabla `generated_reports`: el séptimo cimiento del substrato de
-- monetización (ADR-0144). Cada fila es UN REPORTE INSTITUCIONAL generado
-- a partir de un resultado del guantelete de validación/backtest/ejecución
-- -- un documento firmado (`signature_hash`) y trazable a los eventos
-- fuente del event-store (#6, `docs/features/enriched-domain-events.md`) /
-- audit-log (ADR-0027).
--
-- APPEND-ONLY, NO mutable (docs/features/institutional-report-engine.md
-- "Restricciones": "NUNCA un reporte altera los datos fuente: solo los
-- presenta" -- por la misma razón, el reporte generado en sí mismo tampoco
-- se edita: cada generación es un hecho histórico permanente). Por
-- ADR-0141 ("PROHIBIDO usar el mismo nombre `event_sequence_id` para lo
-- que es `row_version` y viceversa"), esta tabla usa
-- `event_sequence_id INTEGER NOT NULL UNIQUE` (posición monótona GLOBAL en
-- la secuencia) -- mismo patrón que `domain_events` (migración
-- `0012_domain_events.sql`), `usage_records` y `consent_records`.
--
-- Regla "Atomicidad de ledgers append-only" (rust-engineer/SKILL.md §4,
-- causa raíz DEBT-001): el `event_sequence_id` de cada fila nueva se
-- deriva DENTRO de una transacción `BEGIN IMMEDIATE` (ver
-- `persistence::institutional_report_engine::GeneratedReportRepository`),
-- nunca en sentencias sueltas -- el `UNIQUE` de abajo es cinturón-y-
-- tirantes, no el guardián primario.
--
-- Perfil ADR-0020 para esta tabla (Perfil D "Ops/Auditoría", declarado en
-- `docs/features/institutional-report-engine.md` §"Persistencia" y
-- STORY-034 §3):
--   - Grupo I  (Identidad & Integridad, UNIVERSAL, con event_sequence_id
--     en vez de row_version por ser tabla APPEND-ONLY, ADR-0141): id,
--     created_at, updated_at, audit_hash, audit_chain_hash,
--     event_sequence_id.
--   - Grupo II (Soberanía & Propiedad): owner_id, institutional_tag.
--   - Grupo IV (Infraestructura & Ops): node_id.
--   - Subset de Grupo V (Forense/Cumplimiento): signature_hash (firma de
--     integridad REPRODUCIBLE del CONTENIDO del reporte -- distinta de
--     `audit_hash`, que es la integridad de ESTA FILA en el ledger),
--     compliance_status_id (nullable: no todo reporte trae un veredicto de
--     cumplimiento anotado en el momento de generarse).
-- El Grupo III (Linaje Alpha & Datos) NO aplica (un reporte generado no
-- tiene linaje genómico propio -- linaje de a qué resultado pertenece se
-- cubre con `source_result_ref`, columna propia de la Feature) y se omite
-- a propósito (ADR-0020 "Aplicación": "PROHIBIDO copy-paste masivo").
--
-- Columnas propias de la Feature (fuera del contrato de 25 campos,
-- `docs/features/institutional-report-engine.md` "Persistencia"):
--   - report_type: qué clase de reporte es esta fila. El catálogo cubre
--     los tres tipos que el guantelete YA produce hoy (validación,
--     backtest, ejecución) más los cuatro tipos de producto anticipados
--     por ADR-0144 punto 7 (stress test, validación de modelo,
--     certificación de backtest, forense de drawdown) -- sus adaptadores
--     de negocio quedan diferidos, pero el valor del catálogo ya existe
--     (Inundación de Fundaciones, ADR-0020: más barato tenerlo ahora que
--     migrar después).
--   - source_result_ref: referencia (id de texto libre) al resultado
--     fuente del guantelete que este reporte presenta -- nullable, porque
--     no todo reporte nace atado a un único resultado identificable por id
--     (ej. un reporte agregado sobre varios resultados).
--   - source_event_refs: lista JSON de ids de eventos del event-store (#6)
--     / audit-log que este reporte cita, para trazabilidad (ADR-0027) --
--     `CHECK(json_valid(source_event_refs))` rechaza JSON corrupto a nivel
--     de BD.
--   - report_body: el contenido JSON canónico COMPLETO del reporte (el
--     mismo string que produce `domain::institutional_report_engine::
--     InstitutionalReport::canonical_report_json` y que
--     `compute_report_signature` hashea) -- `CHECK(json_valid(report_body))`
--     defensa en profundidad sobre la serialización determinista del Core
--     (BTreeMap ordenado, mismo patrón que `enriched_domain_events`).
--
-- Guardarraíl ADR-0093: esta tabla JAMÁS almacena secretos -- ni
-- `report_body` ni ninguna otra columna puede incluir credenciales de
-- bróker, claves de firma ni IPs de servidores live. El Core que produce
-- el reporte solo modela métricas de negocio (nombradas, enteras ×10⁸) y
-- metadatos de trazabilidad; el test de integración de este cimiento lo
-- assert explícitamente sobre el JSON persistido.
--
-- Idempotencia (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF
-- NOT EXISTS` / `CREATE TRIGGER IF NOT EXISTS` hacen que volver a correr
-- esta migración sea un no-op.

CREATE TABLE IF NOT EXISTS generated_reports (
    -- I. Identidad & Integridad (universal, ADR-0020; event_sequence_id
    -- en vez de row_version por ser tabla APPEND-ONLY, ADR-0141).
    id                    TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock) -- instante de PERSISTENCIA
    updated_at            INTEGER NOT NULL,             -- Nanosegundos desde epoch; append-only => siempre igual a created_at
    audit_hash            TEXT    NOT NULL,             -- SHA-256 del contenido de ESTA FILA + enlace previo (integridad del ledger)
    audit_chain_hash      TEXT,                         -- audit_hash de la fila anterior (NULL solo en la fila génesis)
    event_sequence_id     INTEGER NOT NULL UNIQUE,      -- Posición monótona en la cadena GLOBAL (1, 2, 3, ...)

    -- II. Soberanía & Propiedad
    owner_id              TEXT    NOT NULL,             -- Dueño del reporte (viene de central-identity)
    institutional_tag     TEXT    NOT NULL,             -- Entorno/etiqueta institucional

    -- IV. Infraestructura & Ops
    node_id               TEXT    NOT NULL,             -- Máquina que generó el reporte

    -- Subset de V. Forense & Ejecución (Gobernanza/Cumplimiento)
    signature_hash        TEXT    NOT NULL,             -- Firma de integridad REPRODUCIBLE del CONTENIDO del reporte (distinta de audit_hash)
    compliance_status_id  TEXT,                         -- Veredicto de cumplimiento vigente al momento de generar el reporte (nullable)

    -- Columnas propias de la Feature (docs/features/institutional-report-engine.md "Persistencia")
    -- Qué clase de reporte es esta fila -- catálogo de los tipos que el
    -- guantelete produce hoy más los de producto anticipados por ADR-0144.
    report_type           TEXT    NOT NULL CHECK (report_type IN (
                               'VALIDATION',
                               'BACKTEST',
                               'EXECUTION',
                               'STRESS_TEST',
                               'MODEL_VALIDATION',
                               'BACKTEST_CERTIFICATION',
                               'DRAWDOWN_FORENSICS'
                           )),
    -- Referencia de texto libre al resultado fuente del guantelete
    -- (nullable -- no todo reporte nace atado a un único id de resultado).
    source_result_ref     TEXT,
    -- Lista JSON de ids de eventos del event-store (#6) / audit-log que
    -- este reporte cita -- trazabilidad (ADR-0027).
    source_event_refs     TEXT    NOT NULL CHECK (json_valid(source_event_refs)),
    -- Contenido JSON canónico COMPLETO del reporte (claves ordenadas,
    -- serializado desde un BTreeMap -- determinista, ADR-0002/0004). Es
    -- exactamente lo que `signature_hash` hashea.
    report_body           TEXT    NOT NULL CHECK (json_valid(report_body)),

    -- FK física owner_id -> accounts(id) (ADR-0141 enmienda 2026-07-11, M6):
    -- el dueño del reporte DEBE existir en `accounts`. RESTRICT: nunca se
    -- borra una cuenta con reportes generados asociados.
    FOREIGN KEY (owner_id) REFERENCES accounts (id) ON DELETE RESTRICT
) STRICT;

-- Índice de la posición en la cadena (ADR-0141 M8: "Índice obligatorio en
-- event_sequence_id en tablas append-only") -- acceso cronológico / replay.
CREATE INDEX IF NOT EXISTS idx_generated_reports_event_sequence_id
    ON generated_reports (event_sequence_id);

-- Query path de "todos los reportes de tipo X" (ej. todos los reportes de
-- stress test generados hasta ahora).
CREATE INDEX IF NOT EXISTS idx_generated_reports_report_type
    ON generated_reports (report_type);

-- Query path de "todos los reportes de este dueño" (agregación / listado
-- de reportes por owner_id, consistente con el resto de tablas del
-- substrato).
CREATE INDEX IF NOT EXISTS idx_generated_reports_owner_id
    ON generated_reports (owner_id);

-- Enforzamiento append-only: rechaza UPDATE (institutional-report-engine.md
-- "Restricciones": "NUNCA un reporte altera los datos fuente" -- por
-- extensión, tampoco se altera a sí mismo una vez generado).
CREATE TRIGGER IF NOT EXISTS trg_generated_reports_no_update
BEFORE UPDATE ON generated_reports
BEGIN
    SELECT RAISE(ABORT, 'generated_reports is append-only: UPDATE is forbidden');
END;

-- Enforzamiento append-only: rechaza DELETE.
CREATE TRIGGER IF NOT EXISTS trg_generated_reports_no_delete
BEFORE DELETE ON generated_reports
BEGIN
    SELECT RAISE(ABORT, 'generated_reports is append-only: DELETE is forbidden');
END;
