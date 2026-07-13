# operator_role_events

**Feature dueña:** [`operator-roles`](../features/operator-roles.md)
**Migración:** `migrations/0020_operator_roles.sql`
**Naturaleza:** Append-only atómica (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE)
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
| `owner_id` | TEXT | NO | Cuenta maestra afectada |
| `institutional_tag` | TEXT | NO | Entorno |
| `node_id` | TEXT | NO | Máquina que registró ESTE evento |
| `compliance_status_id` | TEXT | SÍ | Estado de cumplimiento al momento (nullable) |
| `change_type` | TEXT | NO | `ROLE_CREATED`\|`ROLE_UPDATED`\|`ROLE_REVOKED`\|`ASSIGNMENT_SET`\|`ASSIGNMENT_REVOKED`\|`AUTHORITY_OVERRIDE` |
| `subject_ref` | TEXT | NO | El `role_id` o `access_token_id` afectado |
| `detail` | TEXT | SÍ | JSON opcional con detalle adicional |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`
- **Índices:** `event_sequence_id`, `owner_id`
- **Triggers:** `trg_operator_role_events_no_update`, `trg_operator_role_events_no_delete`

## Quién la referencia (FK entrante)

- Ninguna.

## Notas de diseño

- El estado VIGENTE vive en `operator_roles`/`operator_assignments` (mutables) — esta tabla es solo el log de auditoría de CÓMO se llegó a ese estado, nunca se consulta para saber el estado actual.
