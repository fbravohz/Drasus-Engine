# domain_events

**Feature dueña:** [`enriched-domain-events`](../features/enriched-domain-events.md)
**Migración:** `migrations/0012_domain_events.sql`
**Naturaleza:** Append-only (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE) — event-store heterogéneo (una tabla, N tipos de evento vía `event_type` + `payload`).
**Perfil ADR-0020:** Perfil D — Grupo I + Grupo II + `node_id`/`process_id`/`session_id` (IV)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Instante de PERSISTENCIA |
| `updated_at` | INTEGER | NO | == `created_at` |
| `audit_hash` | TEXT | NO | SHA-256 del contenido |
| `audit_chain_hash` | TEXT | SÍ | NULL en fila génesis |
| `event_sequence_id` | INTEGER | NO | Posición monótona GLOBAL, `UNIQUE` |
| `owner_id` | TEXT | NO | Dueño del evento |
| `institutional_tag` | TEXT | NO | Entorno |
| `node_id` | TEXT | NO | Máquina que emitió el evento |
| `process_id` | TEXT | NO | Proceso del motor emisor |
| `session_id` | TEXT | SÍ | Sesión de ejecución (nullable) |
| `event_type` | TEXT | NO | `ORDER_EXECUTED`\|`CAPITAL_FLOW`\|`ACCOUNT_SNAPSHOT`\|`BACKTEST_COMPLETED`\|`REGIME_DETECTED`\|`DRAWDOWN_DETECTED`\|`LIQUIDITY_STRESS`\|`CORRELATION_CHANGE` |
| `payload` | TEXT | NO | JSON canónico específico de la variante (`CHECK json_valid`) |
| `replicate` | INTEGER | NO | 0/1 — si se replica a la Cabina de Mando |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`
- **Índices:** `event_sequence_id`, `event_type`, `owner_id`
- **Triggers:** `trg_domain_events_no_update`, `trg_domain_events_no_delete`

## Quién la referencia (FK entrante)

- Ninguna FK física — pero es la **raíz** citada por `generated_reports.source_event_refs` (JSON de ids, sin FK física) para trazabilidad de reportes.

## Notas de diseño

- Una sola tabla para TODOS los tipos de evento de dominio (no una tabla por tipo) — el Core Rust (`EnrichedDomainEvent` enum) produce el `payload` determinista. `replicate` es solo el flag calculado; el envío real por red es un adaptador diferido.
