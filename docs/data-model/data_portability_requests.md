# data_portability_requests

**Feature dueña:** [`data-portability`](../features/data-portability.md)
**Migración:** `migrations/0019_data_portability.sql`
**Naturaleza:** Append-only atómica (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE) — el avance de estado es una fila NUEVA (`request_group_id` agrupa), nunca un UPDATE.
**Perfil ADR-0020:** Perfil D — Grupo I + Grupo II + `node_id` (IV) + subset V (`compliance_status_id`, nullable)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Instante de PERSISTENCIA de ESTE evento |
| `updated_at` | INTEGER | NO | == `created_at` |
| `audit_hash` | TEXT | NO | Integridad de ESTA FILA |
| `audit_chain_hash` | TEXT | SÍ | NULL en fila génesis |
| `event_sequence_id` | INTEGER | NO | Posición monótona GLOBAL, `UNIQUE` |
| `owner_id` | TEXT | NO | Titular que pide export/olvido |
| `institutional_tag` | TEXT | NO | Entorno |
| `node_id` | TEXT | NO | Máquina que registró ESTE evento |
| `compliance_status_id` | TEXT | SÍ | Estado de cumplimiento al momento (nullable) |
| `request_type` | TEXT | NO | `EXPORT` (Art. 15/20) \| `FORGET` (Art. 17) |
| `status` | TEXT | NO | `RECEIVED`\|`PROCESSING`\|`COMPLETED` — vigente es el evento más reciente por `request_group_id` |
| `request_group_id` | TEXT | NO | Agrupa TODOS los eventos de UNA solicitud lógica |
| `disposition_detail` | TEXT | SÍ | JSON: qué tablas se pseudonimizaron-y-retuvieron vs. purgaron (solo `FORGET`) |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`
- **Índices:** `event_sequence_id`, `owner_id`, `request_group_id`
- **Triggers:** `trg_data_portability_requests_no_update`, `trg_data_portability_requests_no_delete`

## Quién la referencia (FK entrante)

- Ninguna.

## Notas de diseño

- Regla fija #3 (ADR-0148): el olvido NUNCA hace DELETE físico — siempre pseudonimización, incluso en tablas sin retención. `disposition_detail` documenta el destino real de cada tabla del catálogo, nunca "se borró".
