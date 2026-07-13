# instance_backups

**Feature dueña:** [`instance-continuity`](../features/instance-continuity.md)
**Migración:** `migrations/0017_instance_continuity.sql`
**Naturaleza:** Append-only atómica (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE)
**Perfil ADR-0020:** Perfil D — Grupo I + Grupo II + `node_id` (IV)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Instante de PERSISTENCIA |
| `updated_at` | INTEGER | NO | == `created_at` |
| `audit_hash` | TEXT | NO | Integridad de ESTA FILA |
| `audit_chain_hash` | TEXT | SÍ | NULL en fila génesis |
| `event_sequence_id` | INTEGER | NO | Posición monótona GLOBAL, `UNIQUE` |
| `owner_id` | TEXT | NO | Dueño de la cuenta |
| `institutional_tag` | TEXT | NO | Entorno |
| `node_id` | TEXT | NO | Máquina que produjo el respaldo |
| `snapshot_at` | INTEGER | NO | Instante del snapshot (distinto de `created_at`) |
| `blob_hash` | TEXT | NO | SHA-256 del blob cifrado |
| `blob_size_bytes` | INTEGER | NO | Tamaño del blob, bytes |
| `nonce_hex` | TEXT | NO | Nonce AES-GCM (no secreto, necesario junto a la clave para descifrar) |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`
- **Índices:** `event_sequence_id`, `owner_id`
- **Triggers:** `trg_instance_backups_no_update`, `trg_instance_backups_no_delete`

## Quién la referencia (FK entrante)

- Ninguna.

## Notas de diseño

- Guardarraíl ADR-0093 estructural: NINGUNA columna puede contener la clave de cifrado ni el secreto maestro — solo bytes opacos (blob) + metadatos. El adaptador de subida real al almacén de objetos del proveedor está diferido; esta tabla es el registro local del respaldo.
