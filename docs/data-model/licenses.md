# licenses

**Feature dueña:** [`licensing-system`](../features/licensing-system.md)
**Migración:** `migrations/0008_licensing_system.sql`
**Naturaleza:** Mutable (`row_version`) — cada fila es UNA ACTIVACIÓN (owner_id + máquina), no la licencia en abstracto.
**Perfil ADR-0020:** Perfil D — Grupo I + Grupo II + `node_id`/`process_id` (IV) + subset de Grupo V (`signature_hash`, `compliance_status_id`)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | Cambia con cada refresco de heartbeat |
| `audit_hash` | TEXT | NO | SHA-256 de esta versión |
| `audit_chain_hash` | TEXT | SÍ | NULL en fila génesis |
| `row_version` | INTEGER | NO | Arranca en 1 |
| `license_id` | TEXT | NO | ID de la LICENCIA firmada (distinto de `id`, que es la PK de esta ACTIVACIÓN) |
| `owner_id` | TEXT | NO | Dueño de la licencia |
| `institutional_tag` | TEXT | NO | Entorno |
| `access_token_id` | TEXT | SÍ | Sesión de auth |
| `node_id` | TEXT | NO | Huella de hardware REUTILIZADA de `accounts.node_id` |
| `process_id` | TEXT | SÍ | Proceso que activó |
| `signature_hash` | TEXT | NO | Firma Ed25519 (pública, nunca la clave privada) |
| `compliance_status_id` | TEXT | NO | `ACTIVE`\|`GRACE`\|`EXPIRED`\|`REVOKED` |
| `tier` | TEXT | NO | `SOVEREIGN`\|`EXPLORER` |
| `issued_at` | INTEGER | NO | Cuándo el emisor firmó el payload vigente |
| `heartbeat_expires_at` | INTEGER | NO | Vence del heartbeat vigente |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`
- **Únicas:** `(owner_id, node_id)` — una activación por máquina por dueño
- **Índices:** `owner_id`, `node_id`, `license_id`

## Quién la referencia (FK entrante)

- Ninguna.

## Notas de diseño

- Varias filas por `owner_id`: el modelo permite varias activaciones simultáneas por tier (`ACTIVATIONS_PER_TIER`, config). Reactivar la MISMA máquina reutiliza su fila (vía el índice único), nunca duplica.
- Guardarraíl ADR-0093: JAMÁS almacena la clave privada de firma — solo la firma pública verificable.
