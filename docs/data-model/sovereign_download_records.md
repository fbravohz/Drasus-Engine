# sovereign_download_records

**Feature dueña:** [`sovereign-data-fetcher`](../features/sovereign-data-fetcher.md)
**Migración:** `migrations/0006_sovereign_data_fetcher.sql`
**Naturaleza:** Append-only por convención (`event_sequence_id UNIQUE`), sin triggers de bloqueo explícitos.
**Perfil ADR-0020:** Perfil A (Datos de Mercado) — Grupo I + Grupo III (`data_snapshot_id`, `logic_hash`) + Grupo IV

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUID |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | == `created_at` |
| `audit_hash` | TEXT | NO | SHA-256 del contenido |
| `audit_chain_hash` | TEXT | SÍ | NULL en el primer registro |
| `event_sequence_id` | INTEGER | NO | Posición monótona, `UNIQUE` |
| `data_snapshot_id` | TEXT | SÍ | Formato `<exchange>_<symbol>_<timeframe>_<year><month>` (`CHECK GLOB '*_*_*_*'`) |
| `logic_hash` | TEXT | SÍ | Versión del driver del fetcher |
| `node_id` | TEXT | SÍ | Hardware donde corrió la descarga |
| `process_id` | TEXT | SÍ | PID del worker de descarga |
| `source_endpoint` | TEXT | NO | URL/endpoint exacto de la fuente |

## Claves

- **PK:** `id`
- **FK salientes:** ninguna
- **Índices:** `event_sequence_id`, `node_id`

## Quién la referencia (FK entrante)

- Ninguna.

## Notas de diseño

- Sin credenciales (Grupo II omitido a propósito — datos públicos en esta Story). El formato de `data_snapshot_id` solo valida la forma (4 segmentos); el reconciler de Parquet en `ingest` valida cada segmento.
