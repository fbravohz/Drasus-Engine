# jobs

**Feature dueña:** [`async-job-executor`](../features/async-job-executor.md)
**Migración:** `migrations/0003_jobs.sql`
**Naturaleza:** Mutable (`row_version`)
**Perfil ADR-0020:** Grupo I (con `row_version`) + concurrencia/integridad (`process_id`, `session_id`, `node_id`, `logic_hash`) + soberanía acotada (`owner_id`, `access_token_id`)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUID — es el `job_uuid` |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | Se actualiza en cada cambio de estado/progreso |
| `audit_hash` | TEXT | NO | SHA-256 de esta versión |
| `audit_chain_hash` | TEXT | SÍ | Hash de la versión anterior de esta fila |
| `row_version` | INTEGER | NO | Contador de versión, arranca en 1 |
| `process_id` | TEXT | SÍ | Worker que tocó este job por última vez |
| `session_id` | TEXT | SÍ | Sesión de ejecución |
| `node_id` | TEXT | SÍ | Hardware del nodo ejecutor |
| `logic_hash` | TEXT | SÍ | Versión del ejecutor que corrió este job |
| `owner_id` | TEXT | SÍ | Dueño (no todo job tiene uno) |
| `access_token_id` | TEXT | SÍ | Auth tracking |
| `user_id` | TEXT | NO | Usuario solicitante |
| `job_type` | TEXT | NO | Ej. `BACKTEST`, `GENERATE_CANDIDATES`, `OPTIMIZE_PORTFOLIO` |
| `parameters` | TEXT | NO | JSON de parámetros (`CHECK json_valid`) |
| `state` | TEXT | NO | `QUEUED`\|`RUNNING`\|`COMPLETED`\|`FAILED`\|`CANCELLED` |
| `progress` | INTEGER | NO | 0–100, default 0 |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`
- **Índices:** `state`, `owner_id`

## Quién la referencia (FK entrante)

- `job_results.job_uuid → jobs.id`

## Notas de diseño

- Persist-before-ack: el UUID se genera y persiste ANTES de devolverlo al cliente. Recuperación post-crash: al arrancar, escanea `state IN ('QUEUED','RUNNING')` para reanudar.
