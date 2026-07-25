# broker_instrument_terms

**Feature dueña:** [`instrument-catalog`](../features/instrument-catalog.md)
**Migración:** `migrations/0022_instrument_session_catalog.sql`
**Naturaleza:** Mutable (`row_version`) — override SPARSE, una fila solo cuando un bróker concreto ofrece el instrumento.
**Perfil ADR-0020:** Perfil A (Datos/Ingest) — Grupo I (universal) + Grupo IV (`node_id`/`process_id`)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | Cambia con cada edición |
| `audit_hash` | TEXT | NO | SHA-256 de esta versión de la fila |
| `audit_chain_hash` | TEXT | SÍ | NULL en la fila génesis |
| `row_version` | INTEGER | NO | Arranca en 1, +1 en cada UPDATE |
| `node_id` | TEXT | SÍ | Huella del hardware que registró el alta/edición |
| `process_id` | TEXT | SÍ | PID del proceso que ejecutó la escritura |
| `instrument_id` | TEXT | NO | FK → `instrument_master(instrument_id)` — clave de NEGOCIO, no el UUID técnico |
| `venue` | TEXT | NO | Bróker/venue |
| `venue_symbol` | TEXT | SÍ | El `broker_symbol` que `data-normalization-layer` ya declaraba — reubicado aquí como su hogar estructural |
| `default_spread` | INTEGER | SÍ | ×10⁸ — piso declarado, consumido por la Política de Integridad de Spread de `data-sanitizer-pipeline`. NULL = herencia |
| `default_slippage` | INTEGER | SÍ | ×10⁸. NULL = herencia |
| `pip_size_override` | INTEGER | SÍ | ×10⁸. NULL = herencia |
| `tick_size_override` | INTEGER | SÍ | ×10⁸. NULL = herencia |
| `point_value` | INTEGER | SÍ | ×10⁸. NULL = herencia |
| `order_size_multiplier` | INTEGER | SÍ | Contador plano, SIN escala. NULL = herencia |
| `order_size_step` | INTEGER | SÍ | ×10⁸ (tamaño de lote fraccionario). NULL = herencia |
| `min_distance` | INTEGER | SÍ | ×10⁸. NULL = herencia |
| `commission_model` | TEXT | SÍ | `CHECK (NULL OR IN ('NONE','PER_TRADE','PERCENTAGE','SIZE_BASED','STOCKPICKER'))` |
| `commission_value` | INTEGER | SÍ | ×10⁸. NULL = herencia |
| `swap_type` | TEXT | SÍ | **Sin `CHECK`** — vocabulario diferido, ADR-0162 Restricción #10 (sin fuente verificada aún) |
| `swap_long_points` | INTEGER | SÍ | ×10⁸, puntos de swap |
| `swap_short_points` | INTEGER | SÍ | ×10⁸ |
| `triple_swap_day` | TEXT | SÍ | `CHECK (NULL OR IN ('MON'..'SUN'))` — mismo vocabulario de día que las ventanas de sesión |
| `swap_rollout_hour` | INTEGER | SÍ | `CHECK (NULL OR BETWEEN 0 AND 23)` |
| `session_template_id_override` | TEXT | SÍ | FK → `session_template(id)` — solo cuando el calendario de ESTE bróker difiere del default del instrumento |

## Claves

- **PK:** `id`
- **FK salientes:** `instrument_id` → `instrument_master(instrument_id)` `ON DELETE RESTRICT`; `session_template_id_override` → `session_template(id)` `ON DELETE RESTRICT`
- **Únicas:** `UNIQUE(instrument_id, venue)` — un bróker tiene UNA sola fila de override por instrumento
- **Índices:** `instrument_id` (FK hijo), `session_template_id_override` (FK hijo), `node_id`

## Quién la referencia (FK entrante)

- Ninguna (es tabla hoja del catálogo — nada referencia esta tabla).

## Notas de diseño

- **`NULL` es herencia declarada, NUNCA "dato faltante por error"** (Restricción FIJA #1, ADR-0162) — la resolución (`instrument-catalog::domain::resolve_terms`) hace `COALESCE` campo a campo: override no-`NULL` gana, `NULL` hereda el canónico de `instrument_master`.
- Borrar una fila de esta tabla es DELETE real (no soft-delete) y es la operación esperada: "el usuario borra el override de un bróker" (criterio de aceptación #3) hace que el instrumento resuelva 100% desde el canónico. No contradice la prohibición de soft-delete de ADR-0141 (esa regla aplica a tablas de estado tipo `jobs`, no a este override sparse).
- NUNCA se fusiona con la tabla de capacidades (ADR-0154): esa declara qué puede traer un bróker; esta declara en qué condiciones se simula una operación con él.
