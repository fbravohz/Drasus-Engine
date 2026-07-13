# aggregated_indexes

**Feature dueña:** [`data-aggregation`](../features/data-aggregation.md)
**Migración:** `migrations/0015_data_aggregation.sql`
**Naturaleza:** Append-only (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE)
**Perfil ADR-0020:** Perfil B (IA/R&D) acotado — Grupo I + `owner_id`/`institutional_tag` (II) + `data_snapshot_id` (III) + `node_id` (IV)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | == `created_at` |
| `audit_hash` | TEXT | NO | SHA-256 del contenido |
| `audit_chain_hash` | TEXT | SÍ | NULL en fila génesis |
| `event_sequence_id` | INTEGER | NO | Posición monótona, `UNIQUE` |
| `owner_id` | TEXT | NO | El PROCESO/AGREGADOR que calculó y publicó — **nunca** un usuario contribuyente individual |
| `institutional_tag` | TEXT | NO | Entorno |
| `data_snapshot_id` | TEXT | SÍ | Linaje al conjunto de eventos fuente (nullable) |
| `node_id` | TEXT | NO | Máquina que calculó el agregado |
| `index_type` | TEXT | NO | `SENTIMENT`\|`REGIME`\|`BROKER_FRICTION`\|`CORRELATION` |
| `time_window` | TEXT | NO | Ej. `'2026-W27'` |
| `cohort_size` | INTEGER | NO | Contribuyentes distintos, siempre `>= MIN_COHORT_SIZE` (k-anonimato) |
| `noise_level` | INTEGER | NO | Ruido de privacidad diferencial aplicado, ×10⁸ |
| `metric_value` | INTEGER | NO | Valor de la métrica agregada, ya con ruido, ×10⁸ |
| `channel` | TEXT | NO | `INTERNAL`\|`EXTERNAL` |

## Claves

- **PK:** `id`
- **FK salientes:** ninguna (`owner_id` aquí NO es la cuenta de un usuario, es el agregador del sistema — sin FK a propósito)
- **Índices:** `event_sequence_id`, `(index_type, time_window)`
- **Triggers:** `trg_aggregated_indexes_no_update`, `trg_aggregated_indexes_no_delete`

## Quién la referencia (FK entrante)

- Ninguna.

## Notas de diseño

- Guardarraíl ADR-0093/0102: ningún balance crudo, IP, llave, ni parámetros/fórmulas exactos de estrategia — solo la métrica YA anonimizada. Un agregado con cohorte insuficiente NUNCA se persiste (se suprime en memoria antes de llegar aquí).
