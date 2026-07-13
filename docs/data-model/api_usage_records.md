# api_usage_records

**Feature dueña:** [`third-party-api-gateway`](../features/third-party-api-gateway.md)
**Migración:** `migrations/0014_api_gateway.sql`
**Naturaleza:** Append-only (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE)
**Perfil ADR-0020:** Mismo Perfil D acotado que `api_credentials`, denormalizado en cada fila (sin JOIN obligatorio para reportar)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | == `created_at` |
| `audit_hash` | TEXT | NO | SHA-256 del contenido |
| `audit_chain_hash` | TEXT | SÍ | NULL en fila génesis |
| `event_sequence_id` | INTEGER | NO | Posición monótona, `UNIQUE` |
| `owner_id` | TEXT | NO | Denormalizado desde `api_credentials` |
| `access_token_id` | TEXT | SÍ | Denormalizado |
| `node_id` | TEXT | NO | Denormalizado |
| `credential_id` | TEXT | NO | Referencia opaca a la credencial (nunca el secreto) |
| `endpoint` | TEXT | NO | Ej. `'CERTIFY'` |
| `outcome` | TEXT | NO | `ALLOWED`\|`RATE_LIMITED`\|`DENIED` |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`
- **Índices:** `event_sequence_id`, `owner_id`, `(credential_id, created_at)` (ventana de rate-limit)
- **Referencias no-FK (soft):** `credential_id → api_credentials.id` — denormalizado a propósito
- **Triggers:** `trg_api_usage_records_no_update`, `trg_api_usage_records_no_delete`

## Quién la referencia (FK entrante)

- Ninguna.

## Notas de diseño

- `outcome` colapsa todos los motivos de rechazo (auth inválida, credencial revocada, endpoint no habilitado, consentimiento no cubierto) a `DENIED` — el motivo detallado vive solo en la respuesta en memoria del gateway, nunca en esta columna.
