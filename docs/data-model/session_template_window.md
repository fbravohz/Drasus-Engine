# session_template_window

**Feature dueña:** [`session-catalog`](../features/session-catalog.md)
**Migración:** `migrations/0022_instrument_session_catalog.sql`
**Naturaleza:** Mutable (`row_version`) — hija de `session_template`, una fila por ventana semanal de apertura.
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
| `session_template_id` | TEXT | NO | FK → `session_template(id)` — padre dueño de esta ventana |
| `start_day` | TEXT | NO | `CHECK IN ('MON','TUE','WED','THU','FRI','SAT','SUN')` |
| `end_day` | TEXT | NO | `CHECK IN ('MON','TUE','WED','THU','FRI','SAT','SUN')` |
| `start_time_minutes` | INTEGER | NO | `CHECK BETWEEN 0 AND 1439` — minutos desde medianoche, hora de `market_timezone_ref` del padre |
| `end_time_minutes` | INTEGER | NO | `CHECK BETWEEN 0 AND 1439` |

## Claves

- **PK:** `id`
- **FK salientes:** `session_template_id` → `session_template(id)` `ON DELETE RESTRICT`
- **Índices:** `session_template_id` (FK hijo, ADR-0141 M7), `node_id`

## Quién la referencia (FK entrante)

- Ninguna (es tabla hoja).

## Notas de diseño

- Una ventana cuyo `end` (en la semana lineal lunes 00:00 → domingo 23:59) cae ANTES que su `start` representa una ventana que CRUZA la frontera domingo 23:59 → lunes 00:00 (ej. Forex: abre domingo 22:00, cierra el viernes SIGUIENTE a las 22:00). `session-catalog::domain::compute_expected_fingerprint` cubre este caso en dos tramos — ver la lección Docente de STORY-066 para el detalle del algoritmo.
- Alta/edición SIEMPRE dentro de una transacción `BEGIN IMMEDIATE` junto con el `INSERT` de `session_template`: una plantilla sin ninguna ventana (o con solo algunas de sus ventanas persistidas por un fallo a medio camino) es un estado inválido que esta atomicidad impide por construcción.
