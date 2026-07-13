# operator_roles

**Feature dueña:** [`operator-roles`](../features/operator-roles.md)
**Migración:** `migrations/0020_operator_roles.sql`
**Naturaleza:** Mutable (`row_version`) — un rol se edita (reclasifica su matriz) o se revoca, NUNCA se borra físicamente.
**Perfil ADR-0020:** Perfil D — Grupo I + Grupo II

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | Cambia con cada UPDATE |
| `audit_hash` | TEXT | NO | SHA-256 de esta versión |
| `audit_chain_hash` | TEXT | SÍ | NULL en fila génesis |
| `row_version` | INTEGER | NO | Arranca en 1 |
| `owner_id` | TEXT | NO | Cuenta maestra dueña del catálogo de roles |
| `institutional_tag` | TEXT | NO | Entorno |
| `role_name` | TEXT | NO | Nombre libre (ej. "Analyst", "Risk Manager") |
| `capability_matrix` | TEXT | NO | JSON `{"<capability_key>": true\|false, ...}` — dato, no código |
| `status` | TEXT | NO | `ACTIVE`\|`REVOKED` — baja lógica, nunca DELETE físico |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`
- **Únicas:** `(owner_id, role_name)` — no se duplican nombres de rol dentro de la misma cuenta

## Quién la referencia (FK entrante)

- `operator_assignments.role_id`

## Notas de diseño

- La unidad gateable es el PUERTO de Feature (clave de capacidad), NUNCA el módulo (ADR-0149). La protección del invariante "último admin en pie" es una validación DINÁMICA en el Core, no un flag estático sobre la fila.
