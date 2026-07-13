# job_results

**Feature dueña:** [`async-job-executor`](../features/async-job-executor.md)
**Migración:** `migrations/0003_jobs.sql`
**Naturaleza:** Append-only (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE)
**Perfil ADR-0020:** Grupo I únicamente — el contexto de ejecución (proceso/sesión/nodo/dueño) vive en `jobs`, se alcanza vía `job_uuid`, nunca se duplica.

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUID de esta fila de resultado |
| `created_at` | INTEGER | NO | == `completed_at` |
| `updated_at` | INTEGER | NO | Siempre igual a `created_at` (append-only) |
| `audit_hash` | TEXT | NO | SHA-256 del contenido |
| `audit_chain_hash` | TEXT | SÍ | Hash del resultado anterior (NULL en el primero) |
| `event_sequence_id` | INTEGER | NO | Posición monótona global, `UNIQUE` |
| `job_uuid` | TEXT | NO | El job al que pertenece este resultado |
| `result_data` | TEXT | SÍ | JSON del payload (NULL si falló) |
| `error_message` | TEXT | SÍ | Descripción del error (NULL si tuvo éxito) |
| `completed_at` | INTEGER | NO | Cuándo el job llegó a estado terminal |

## Claves

- **PK:** `id`
- **FK salientes:** `job_uuid → jobs(id) ON DELETE RESTRICT`
- **Índices:** `job_uuid`, `event_sequence_id`
- **Triggers:** `trg_job_results_no_update`, `trg_job_results_no_delete`

## Quién la referencia (FK entrante)

- Ninguna — es hoja terminal.

## Notas de diseño

- Un job es mutable mientras corre (`jobs`), pero su resultado final es inmutable en cuanto se escribe — nunca se corrige in-place.
