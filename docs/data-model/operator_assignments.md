# operator_assignments

**Feature dueña:** [`operator-roles`](../features/operator-roles.md)
**Migración:** `migrations/0020_operator_roles.sql`
**Naturaleza:** Mutable (`row_version`) — un operador tiene UN rol vigente por cuenta; reasignar es UPDATE de la misma fila.
**Perfil ADR-0020:** Perfil D — Grupo I + Grupo II (con `access_token_id`)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | Cambia con cada UPDATE |
| `audit_hash` | TEXT | NO | SHA-256 de esta versión |
| `audit_chain_hash` | TEXT | SÍ | NULL en fila génesis |
| `row_version` | INTEGER | NO | Arranca en 1 |
| `owner_id` | TEXT | NO | Cuenta maestra dueña de la asignación |
| `institutional_tag` | TEXT | NO | Entorno |
| `access_token_id` | TEXT | NO | Ancla de atribución del operador (login humano o conexión MCP) |
| `operator_type` | TEXT | NO | `HUMAN` (login) \| `AGENT` (conexión MCP) |
| `role_id` | TEXT | NO | FK a `operator_roles` — JAMÁS `ON DELETE CASCADE` |
| `status` | TEXT | NO | `ACTIVE`\|`REVOKED` — baja lógica |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`, `role_id → operator_roles(id) ON DELETE RESTRICT`
- **Únicas:** `(owner_id, access_token_id)` — un operador, un rol vigente por cuenta
- **Índices:** `role_id`, `owner_id`

## Quién la referencia (FK entrante)

- Ninguna.

## Notas de diseño

- Mismo catálogo (`operator_type`) para operadores humanos y agentes — la matriz de capacidades no distingue quién invoca, solo qué puede invocar.
