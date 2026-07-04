-- Migración 0009: Plan / Tier / Quota (docs/features/plan-tier-quota.md,
-- ADR-0144, ADR-0143, ADR-0141, ADR-0020 V2, STORY-029)
--
-- Crea la tabla `plans`: cimiento #3 del substrato de monetización
-- (ADR-0144). Cada fila es UN PLAN del catálogo -- dato, no código: define
-- el tier, sus cuotas (volumen nocional permitido, activaciones máximas,
-- features habilitadas) y su precio. `licensing-system` (cimiento #2) y
-- `usage-metering` (cimiento #4, futuro) leen este catálogo a través del
-- puerto `plan_limits_out` que esta Feature produce -- NUNCA acceden
-- directamente a esta tabla (ADR-0137: acceso cross-feature solo por
-- puerto tipado).
--
-- MUTABLE, no append-only: un plan cambia límite/precio EN SITIO ("en la
-- siguiente revalidación se refleja", plan-tier-quota.md "Comportamientos
-- Observables"). Por ADR-0141 ("PROHIBIDO usar el mismo nombre
-- `event_sequence_id` para lo que es `row_version` y viceversa"), esta
-- tabla usa `row_version` (concurrencia optimista) en vez de
-- `event_sequence_id UNIQUE`. El historial de cada cambio de plan se
-- audita en `audit_events` (feature `audit-log` ya existente, migración
-- 0002) -- esta tabla NO es su propio historial, solo el estado vigente.
--
-- Perfil ADR-0020 V2 para esta tabla (Perfil D "Ops/Auditoría", declarado
-- en `docs/features/plan-tier-quota.md` §"Gobernanza y Estándares" y
-- STORY-029 §3 -- SIN el subset de Grupo V que sí lleva `licenses`, porque
-- un plan no es, en sí mismo, un veredicto forense):
--   - Grupo I  (Identidad & Integridad, UNIVERSAL, con row_version en vez
--     de event_sequence_id por ser tabla mutable, ADR-0141): id,
--     created_at, updated_at, audit_hash, audit_chain_hash, row_version.
--   - Grupo II (Soberanía & Propiedad, subset acotado por la Orden): owner_id
--     (creador del plan -- NO es FK a `accounts`: el catálogo real lo define
--     la Cabina de Mando, que no es una fila local de `accounts`; en el
--     stub de desarrollo es un identificador de sistema fijo),
--     institutional_tag.
--   - Grupo IV (Infraestructura & Ops): node_id (máquina que registró la
--     definición del plan -- informativo, no ancla ninguna huella de
--     hardware de usuario).
-- El Grupo III (Linaje Alpha & Datos) y el subset de Grupo V de gobernanza
-- forense NO aplican (un plan no tiene linaje genómico ni es un veredicto
-- de cumplimiento) y se omiten a propósito (ADR-0020 V2 "Aplicación":
-- "PROHIBIDO copy-paste masivo").
--
-- Columnas propias de la Feature (fuera del contrato de 25 campos):
--   - tier: FREE | PAID (plan-tier-quota.md "Parámetros Configurables":
--     TIER_SET). Vocabulario PROPIO de este catálogo -- distinto del
--     `tier` SOVEREIGN/EXPLORER de `licenses` (licensing-system): son dos
--     conceptos que hoy conviven sin unificar (ver comentario de
--     `domain::plan_tier_quota` sobre el re-cableado diferido).
--   - notional_limit: volumen nocional permitido, INTEGER escalado ×10⁸
--     (ADR-0141 -- NUNCA REAL). 0 es un valor válido en sí mismo (ej. un
--     plan sin tope de nocional propio codifica "sin límite" con
--     max_activations como única cuota); lo que NUNCA es válido es AMBAS
--     cuotas en cero a la vez (Core lo rechaza, ver `validate_plan`).
--   - max_activations: activaciones máximas (máquinas distintas) permitidas
--     para este plan (licensing-system.md "ACTIVATIONS_PER_TIER" -- el
--     límite REAL, del que licensing-system hoy solo tiene un stub).
--   - price: precio del plan, INTEGER escalado ×10⁸ (ADR-0141 -- NUNCA
--     REAL). 0 es válido (plan gratuito).
--   - pricing_model: FLAT | VOLUME (plan-tier-quota.md "PRICING_MODEL") --
--     cómo lee el adaptador de billing este catálogo; el catálogo es el
--     mismo para ambos modelos (plan-tier-quota.md "Comportamientos
--     Observables").
--   - features_enabled: conjunto de features habilitadas, codificado como
--     JSON de una lista de strings ORDENADA alfabéticamente y sin
--     duplicados (ver `domain::plan_tier_quota::canonical_features_json`)
--     -- determinismo bit-a-bit (ADR-0002/0004): el MISMO conjunto de
--     features, insertado en cualquier orden, produce el mismo texto
--     persistido. Se eligió TEXT-JSON en vez de una tabla hija M:N
--     (ADR-0141 "Patrón M:N") porque el conjunto no tiene atributos
--     propios (peso, fecha de alta) que justificarían una tabla aparte.
--
-- Guardarraíl ADR-0093: esta tabla JAMÁS almacena secretos -- ninguna
-- columna existe para credenciales de bróker, claves de firma ni IPs de
-- servidores live. Es, deliberadamente, dato de catálogo comercial puro.
--
-- Idempotencia (ADR-0006): `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF
-- NOT EXISTS` hacen que volver a correr esta migración sea un no-op.

CREATE TABLE IF NOT EXISTS plans (
    -- I. Identidad & Integridad (universal, ADR-0020 V2; row_version en vez
    -- de event_sequence_id por ser tabla mutable, ADR-0141).
    id                 TEXT    NOT NULL PRIMARY KEY, -- UUIDv7 (Uuid::now_v7())
    created_at         INTEGER NOT NULL,             -- Nanosegundos desde epoch (puerto Clock)
    updated_at         INTEGER NOT NULL,             -- Nanosegundos desde epoch; cambia con cada revisión de límite/precio
    audit_hash         TEXT    NOT NULL,             -- SHA-256 del contenido de esta versión de fila + enlace previo
    audit_chain_hash   TEXT,                         -- audit_hash de la versión anterior de esta fila (NULL solo en la fila génesis)
    row_version        INTEGER NOT NULL,             -- Contador de versión de esta fila; arranca en 1, +1 en cada revisión

    -- II. Soberanía & Propiedad (subset acotado por la Orden -- sin access_token_id: un plan no tiene sesión de auth propia)
    owner_id           TEXT    NOT NULL,             -- Creador del plan; identificador de sistema en el stub local, sin FK a accounts (el catálogo real lo define la Cabina de Mando)
    institutional_tag  TEXT    NOT NULL,             -- Entorno/etiqueta institucional

    -- IV. Infraestructura & Ops
    node_id            TEXT    NOT NULL,             -- Máquina que registró la definición del plan (informativo)

    -- Columnas propias de la Feature (plan-tier-quota.md "Parámetros Configurables")
    tier                  TEXT    NOT NULL CHECK (tier IN ('FREE', 'PAID')),
    -- Volumen nocional permitido, INTEGER escalado ×10⁸ (ADR-0141 M1 -- NUNCA REAL).
    notional_limit        INTEGER NOT NULL CHECK (notional_limit >= 0),
    -- Activaciones (máquinas distintas) máximas permitidas para este plan.
    max_activations       INTEGER NOT NULL CHECK (max_activations >= 0),
    -- Precio del plan, INTEGER escalado ×10⁸ (ADR-0141 M1 -- NUNCA REAL).
    price                 INTEGER NOT NULL CHECK (price >= 0),
    pricing_model         TEXT    NOT NULL CHECK (pricing_model IN ('FLAT', 'VOLUME')),
    -- Lista JSON ordenada alfabéticamente de features habilitadas (ver comentario arriba).
    features_enabled      TEXT    NOT NULL DEFAULT '[]' CHECK (json_valid(features_enabled))
) STRICT;

-- Índice del lado propietario (ADR-0141: consistente con el resto de
-- columnas Grupo II owner_id de otras tablas del substrato, aunque aquí no
-- sea FK -- sigue siendo el eje de un query path plausible: "planes que
-- registró tal creador").
CREATE INDEX IF NOT EXISTS idx_plans_owner_id
    ON plans (owner_id);

-- Búsqueda por tier (plan-tier-quota.md "Ciclo de Vida" -- "Proceso":
-- "resuelve los límites aplicables a una licencia dada" -- la resolución
-- parte del tier). Query path principal de `resolve_limits`.
CREATE INDEX IF NOT EXISTS idx_plans_tier
    ON plans (tier);
