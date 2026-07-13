# audit_events

**Feature dueña:** [`audit-log`](../features/audit-log.md)
**Migración:** `migrations/0002_audit_log.sql`
**Naturaleza:** Append-only (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE)
**Perfil ADR-0020:** Ops/Auditoría — Grupo I + Grupo II + Grupo IV (sin III ni V)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUID |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | Siempre igual a `created_at` (append-only) |
| `audit_hash` | TEXT | NO | SHA-256 del evento + enlace previo |
| `audit_chain_hash` | TEXT | SÍ | `audit_hash` de la fila anterior (NULL en génesis) |
| `event_sequence_id` | INTEGER | NO | Posición monótona en la cadena, `UNIQUE` |
| `owner_id` | TEXT | SÍ | Dueño (no todo evento tiene uno) |
| `institutional_tag` | TEXT | NO | Entorno |
| `manifest_id` | TEXT | SÍ | Contrato de diseño |
| `access_token_id` | TEXT | SÍ | Auth tracking |
| `process_id` | TEXT | NO | Job anchor |
| `session_id` | TEXT | SÍ | Runtime grouping |
| `node_id` | TEXT | SÍ | Hardware ID |
| `action_type` | TEXT | NO | Ej. `ORDER_STATE_CHANGE`, `ANOMALY_DETECTED`, `USER_VETO` |
| `entity_type` | TEXT | NO | Tipo de la entidad referida |
| `entity_id` | TEXT | NO | Identificador de esa entidad |
| `details_json` | TEXT | NO | JSON estructurado (`CHECK json_valid`) |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`
- **Índices:** `event_sequence_id`, `(entity_type, entity_id)`, `owner_id`
- **Triggers:** `trg_audit_events_no_update`, `trg_audit_events_no_delete`

## Quién la referencia (FK entrante)

- Ninguna — es hoja terminal, consumida por lectura desde `feedback` y cualquier feature que necesite reconstruir "qué pasó con la entidad X".

## Notas de diseño

- Fuente de verdad de causalidad del sistema (ADR-0015). Nota de orden: esta migración (0002) corre antes que `accounts` (0007) — SQLite permite la referencia hacia adelante bajo `foreign_keys=ON`, sin riesgo funcional (documentado para visibilidad del Tech-Lead).
