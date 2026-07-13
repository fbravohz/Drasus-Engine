# usage_records

**Feature dueña:** [`usage-metering`](../features/usage-metering.md)
**Migración:** `migrations/0010_usage_metering.sql`
**Naturaleza:** Append-only (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE)
**Perfil ADR-0020:** Perfil D — Grupo I + Grupo II + `node_id` (IV) + subset V (`compliance_status_id`, nullable)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | == `created_at` (append-only) |
| `audit_hash` | TEXT | NO | SHA-256 del contenido |
| `audit_chain_hash` | TEXT | SÍ | NULL en fila génesis |
| `event_sequence_id` | INTEGER | NO | Posición monótona, `UNIQUE` |
| `owner_id` | TEXT | NO | Dueño de la operación medida |
| `institutional_tag` | TEXT | NO | Entorno |
| `node_id` | TEXT | NO | Máquina que registró la medición |
| `compliance_status_id` | TEXT | SÍ | Estado de cumplimiento al momento (nullable) |
| `notional_per_op` | INTEGER | NO | Nocional de esta operación, ×10⁸ |
| `cycle_accumulated` | INTEGER | NO | Acumulado del ciclo INMEDIATAMENTE después de esta operación, ×10⁸ |
| `billing_cycle_id` | TEXT | NO | Ej. `"2026-07"` |
| `instrument_id` | TEXT | NO | Ej. `"BTCUSDT"` |
| `quota_verdict` | TEXT | NO | `WITHIN`\|`CROSSED` |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`
- **Índices:** `event_sequence_id`, `owner_id`, `(owner_id, billing_cycle_id)`
- **Triggers:** `trg_usage_records_no_update`, `trg_usage_records_no_delete`

## Quién la referencia (FK entrante)

- Ninguna.

## Notas de diseño

- El reinicio de ciclo de facturación NUNCA borra filas — un `billing_cycle_id` nuevo arranca la acumulación en cero mientras las filas del ciclo anterior permanecen intactas.
- Guardarraíl ADR-0093: solo se mide NOCIONAL, nunca margen ni apalancamiento.
