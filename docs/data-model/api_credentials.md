# api_credentials

**Feature dueña:** [`third-party-api-gateway`](../features/third-party-api-gateway.md)
**Migración:** `migrations/0014_api_gateway.sql`
**Naturaleza:** Mutable (`row_version`) — el estado (`status`) cambia con el tiempo (revocación).
**Perfil ADR-0020:** Perfil D acotado — Grupo I + `owner_id`/`access_token_id` (II) + `node_id` (IV)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | Avanza en cada UPDATE |
| `audit_hash` | TEXT | NO | SHA-256 de esta versión |
| `audit_chain_hash` | TEXT | SÍ | NULL en `row_version=1` |
| `row_version` | INTEGER | NO | Concurrencia optimista |
| `owner_id` | TEXT | NO | Dueño de la credencial |
| `access_token_id` | TEXT | SÍ | Sesión/token que la emitió |
| `node_id` | TEXT | NO | Máquina que emite/administra |
| `credential_hash` | TEXT | NO | SHA-256 hex — NUNCA el secreto en claro |
| `status` | TEXT | NO | `ACTIVE`\|`REVOKED` |
| `rate_limit_per_window` | INTEGER | NO | Solicitudes permitidas por ventana |
| `window_seconds` | INTEGER | NO | Duración de la ventana |
| `endpoints_enabled` | TEXT | NO | JSON array de endpoints habilitados (`CHECK json_valid`) |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`
- **Únicas:** `credential_hash`
- **Índices:** `owner_id`

## Quién la referencia (FK entrante)

- `api_usage_records.credential_id` — **referencia suave, sin FK física** (denormalizada a propósito, el ledger de uso es auto-contenido).

## Notas de diseño

- Autenticar = hashear la credencial presentada y comparar contra `credential_hash`. Una vez `REVOKED`, toda autenticación futura se niega sin importar si el secreto es correcto. Guardarraíl ADR-0093: nunca guarda el secreto en claro.
