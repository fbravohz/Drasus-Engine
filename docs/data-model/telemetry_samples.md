# telemetry_samples

**Feature dueña:** [`telemetry`](../features/telemetry.md)
**Migración:** `migrations/0004_telemetry.sql`
**Naturaleza:** Ni append-only estricta ni mutable — se **poda por retención** (`DELETE WHERE created_at < corte`), sin triggers de bloqueo.
**Perfil ADR-0020:** Grupo I + `institutional_tag` (II) + `logic_hash`/`session_id` (III) + `node_id`/`process_id`/`execution_latency_ms` (IV)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUID |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | Igual a `created_at` |
| `audit_hash` | TEXT | NO | SHA-256 de la muestra |
| `audit_chain_hash` | TEXT | SÍ | Hash de la muestra anterior |
| `event_sequence_id` | INTEGER | NO | Posición monótona, `UNIQUE` |
| `institutional_tag` | TEXT | NO | Entorno (BACKTEST/PAPER/LIVE/...) |
| `logic_hash` | TEXT | SÍ | Versión del emisor de telemetría |
| `session_id` | TEXT | SÍ | Sesión global vinculada |
| `node_id` | TEXT | SÍ | Host físico monitorizado |
| `process_id` | TEXT | NO | PID del proceso muestreado |
| `execution_latency_ms` | INTEGER | SÍ | NULL en heartbeats |
| `metric_name` | TEXT | NO | Ej. `ingest.hot_path_latency` |
| `details_json` | TEXT | SÍ | Contexto extra (`CHECK json_valid`) |

## Claves

- **PK:** `id`
- **FK salientes:** ninguna
- **Índices:** `(metric_name, created_at)`, `created_at` (para la poda)

## Quién la referencia (FK entrante)

- Ninguna.

## Notas de diseño

- Única tabla del sistema con `DELETE` real habilitado por diseño — la poda automática por antigüedad es intencional, no una violación del patrón append-only (esta tabla nunca lo adoptó).
