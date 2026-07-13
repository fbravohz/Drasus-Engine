# consent_records

**Feature dueña:** [`consent-registry`](../features/consent-registry.md)
**Migración:** `migrations/0011_consent_registry.sql`
**Naturaleza:** Append-only (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE) — el estado vigente es "última fila gana" (`MAX(event_sequence_id)` por `owner_id`), nunca UPDATE.
**Perfil ADR-0020:** Perfil D — Grupo I + Grupo II + `node_id` (IV) + subset V (`compliance_status_id`, nullable)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | == `created_at` |
| `audit_hash` | TEXT | NO | SHA-256 del contenido |
| `audit_chain_hash` | TEXT | SÍ | NULL en fila génesis |
| `event_sequence_id` | INTEGER | NO | Posición monótona, `UNIQUE` |
| `owner_id` | TEXT | NO | Titular del consentimiento |
| `institutional_tag` | TEXT | NO | Entorno |
| `node_id` | TEXT | NO | Máquina que registró el evento |
| `compliance_status_id` | TEXT | SÍ | Estado de cumplimiento (nullable) |
| `tos_version` | TEXT | NO | Versión de ToS aceptada EN ESTE evento |
| `consent_action` | TEXT | NO | `ACCEPT`\|`REACCEPT`\|`OPTOUT_CHANGE` |
| `optout_map` | TEXT | NO | JSON `{tipo_dato: bool}` — snapshot COMPLETO (`CHECK json_valid`) |
| `accepted_at` | INTEGER | NO | Instante de dominio (puede diferir de `created_at`) |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`
- **Índices:** `event_sequence_id`, `owner_id`, `(owner_id, event_sequence_id)` (resolución de estado vigente)
- **Triggers:** `trg_consent_records_no_update`, `trg_consent_records_no_delete`

## Quién la referencia (FK entrante)

- Ninguna directa (`data-aggregation` y el firehose del tier gratuito consultan cobertura vía el puerto `consent_out`, nunca acceso directo a la tabla — ADR-0137).

## Notas de diseño

- Cada cambio de opt-out inserta una fila NUEVA con el estado COMPLETO de todos los opt-outs, no solo el campo que cambió — event-sourcing con snapshot completo, nunca un patch parcial.
