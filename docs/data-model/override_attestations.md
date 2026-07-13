# override_attestations

**Feature dueña:** [`master-account-hierarchy`](../features/master-account-hierarchy.md)
**Migración:** `migrations/0018_master_account_hierarchy.sql`
**Naturaleza:** Append-only atómica (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE) — cada intento de override (ejecutado o denegado) es un hecho histórico permanente.
**Perfil ADR-0020:** Perfil D — Grupo I + `owner_id`/`parent_owner_id` (II) + `node_id` (IV)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Instante de PERSISTENCIA |
| `updated_at` | INTEGER | NO | == `created_at` |
| `audit_hash` | TEXT | NO | Integridad de ESTA FILA |
| `audit_chain_hash` | TEXT | SÍ | NULL en fila génesis |
| `event_sequence_id` | INTEGER | NO | Posición monótona GLOBAL, `UNIQUE` |
| `owner_id` | TEXT | NO | La hija afectada |
| `parent_owner_id` | TEXT | NO | El fondo que emitió/gobierna |
| `node_id` | TEXT | NO | Máquina que produjo ESTA fila (fondo en ISSUER, hija en EXECUTOR) |
| `attestation_side` | TEXT | NO | `ISSUER` (el fondo emitió) \| `EXECUTOR` (la hija recibió/ejecutó) |
| `command_kind` | TEXT | NO | `ARCHIVE`\|`MODIFY`\|`REQUEST_AUDIT_REPORT` (catálogo cerrado) |
| `target_ref` | TEXT | NO | Recurso de la hija referenciado (estrategia/portafolio/parámetro) |
| `outcome` | TEXT | NO | `EXECUTED` (solo si `ConsentVerdict::Covered` vigente) \| `DENIED` |
| `justification` | TEXT | SÍ | Texto libre opcional |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`
- **Referencias no-FK (soft):** `parent_owner_id → accounts.id` — misma observación abierta que `account_hierarchy`
- **Índices:** `event_sequence_id`, `owner_id`, `parent_owner_id`
- **Triggers:** `trg_override_attestations_no_update`, `trg_override_attestations_no_delete`

## Quién la referencia (FK entrante)

- Ninguna.

## Notas de diseño

- Regla fija #4: toda orden produce EXACTAMENTE una fila de cada lado (`ISSUER` + `EXECUTOR`), nunca una mutación silenciosa. Regla fija #3: `EXECUTED` solo si el `ConsentVerdict` real de `consent-registry` fue `Covered` en el momento de ESTA fila — nunca cacheado ni asumido.
