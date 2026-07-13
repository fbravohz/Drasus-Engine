# permission_decisions

**Feature dueña:** [`agentic-mcp-gateway`](../features/agentic-mcp-gateway.md)
**Migración:** `migrations/0005_mcp_gateway.sql`
**Naturaleza:** Append-only (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE)
**Perfil ADR-0020:** Ops/Auditoría — Grupo I + Grupo II (subset) + Grupo IV

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUID v4 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | == `created_at` (append-only) |
| `audit_hash` | TEXT | NO | SHA-256 de campos de dominio |
| `audit_chain_hash` | TEXT | SÍ | NULL solo en fila génesis |
| `event_sequence_id` | INTEGER | NO | Posición monótona, `UNIQUE` |
| `owner_id` | TEXT | SÍ | Propietario del interruptor (nullable en local) |
| `institutional_tag` | TEXT | SÍ | "Live"/"Demo" (solo para Manage) |
| `node_id` | TEXT | NO | Host donde corre el Gateway MCP |
| `process_id` | INTEGER | NO | PID del proceso |
| `agent_session_id` | TEXT | NO | Sesión MCP del agente |
| `requested_scope` | TEXT | NO | Pipeline/frontera invocada |
| `permission_outcome` | TEXT | NO | `"granted"` \| `"denied:<razón>"` |
| `production_override_active` | INTEGER | NO | Estado del interruptor (0/1), default 0 |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT` (nullable)
- **Índices:** `(agent_session_id, created_at)`, `owner_id`, `event_sequence_id`
- **Triggers:** `trg_permission_decisions_no_update`, `trg_permission_decisions_no_delete`

## Quién la referencia (FK entrante)

- Ninguna.

## Notas de diseño

- El estado *actual* del interruptor de producción vive aparte, en `mcp_gateway_config` — esta tabla es el log forense de decisiones, no el estado vigente.
