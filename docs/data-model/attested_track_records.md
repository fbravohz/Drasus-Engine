# attested_track_records

**Feature dueña:** [`verified-account-registry`](../features/verified-account-registry.md)
**Migración:** `migrations/0016_verified_account_registry.sql`
**Naturaleza:** Append-only atómica (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE) — cada track calculado es un snapshot inmutable firmado.
**Perfil ADR-0020:** Perfil D — Grupo I + Grupo II + `node_id` (IV) + subset V (`signature_hash`)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Instante de PERSISTENCIA |
| `updated_at` | INTEGER | NO | == `created_at` |
| `audit_hash` | TEXT | NO | Integridad de ESTA FILA |
| `audit_chain_hash` | TEXT | SÍ | NULL en fila génesis |
| `event_sequence_id` | INTEGER | NO | Posición monótona GLOBAL, `UNIQUE` |
| `owner_id` | TEXT | NO | Dueño del track |
| `institutional_tag` | TEXT | NO | Eje B, estampado desde `verified_accounts` al momento del cálculo |
| `node_id` | TEXT | NO | Máquina que calculó el track |
| `signature_hash` | TEXT | NO | Firma REPRODUCIBLE del CONTENIDO (distinta de `audit_hash`) |
| `verified_account_id` | TEXT | NO | Referencia a `verified_accounts.id` |
| `scope` | TEXT | NO | **Eje A**: `SOVEREIGN` (ejecución propia atestada) \| `BROKER_READONLY` (reportado) |
| `time_window` | TEXT | NO | Ej. `"2026-W27"` o `"2026-Q3"` |
| `equity_curve` | TEXT | NO | JSON array `[timestamp_ns, valor_e8]` |
| `balance_curve` | TEXT | NO | JSON array `[timestamp_ns, valor_e8]` |
| `max_drawdown_e8` | INTEGER | NO | ×10⁸ |
| `gain_pct_e8` | INTEGER | NO | ×10⁸ — EXCLUYE depósitos/retiros |
| `win_rate_e8` | INTEGER | NO | ×10⁸ |
| `avg_holding_time_ns` | INTEGER | NO | Nanosegundos |
| `trading_days` | INTEGER | NO | Conteo |
| `total_realized_pnl_e8` | INTEGER | NO | ×10⁸ |
| `total_deposits_e8` | INTEGER | NO | ×10⁸ |
| `total_withdrawals_e8` | INTEGER | NO | ×10⁸ |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`
- **Referencias no-FK (soft):** `verified_account_id → verified_accounts.id`
- **Índices:** `event_sequence_id`, `verified_account_id`, `owner_id`
- **Triggers:** `trg_attested_track_records_no_update`, `trg_attested_track_records_no_delete`

## Quién la referencia (FK entrante)

- Ninguna.

## Notas de diseño

- Regla inviolable: nunca se presenta un dato `BROKER_READONLY` como `SOVEREIGN`. `gain_pct_e8` EXCLUYE depósitos/retiros — un depósito NUNCA cuenta como ganancia (el diferenciador de cálculo frente a competidores).
