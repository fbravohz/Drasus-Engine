# accounts

**Feature dueña:** [`central-identity`](../features/central-identity.md)
**Migración:** `migrations/0007_central_identity.sql`
**Naturaleza:** Mutable (`row_version`) — el ancla de todo el substrato de monetización.
**Perfil ADR-0020:** Perfil D (Ops/Auditoría) — Grupo I (con `row_version`) + Grupo II + `node_id` (IV)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | Cambia con cada UPDATE |
| `audit_hash` | TEXT | NO | SHA-256 de esta versión |
| `audit_chain_hash` | TEXT | SÍ | NULL solo en la fila génesis |
| `row_version` | INTEGER | NO | Arranca en 1, +1 por UPDATE |
| `owner_id` | TEXT | NO | Una cuenta retail es dueña de sí misma (== `id` al crearse) |
| `institutional_tag` | TEXT | NO | Entorno/etiqueta institucional |
| `access_token_id` | TEXT | SÍ | Token de sesión activo |
| `node_id` | TEXT | NO | Huella de hardware determinista (SHA-256) |
| `email` | TEXT | NO | Correo de la cuenta |
| `email_verification_status` | TEXT | NO | `PENDING`\|`VERIFIED`\|`REJECTED` |
| `oauth_provider` | TEXT | SÍ | Proveedor de identidad federada (nullable) |

## Claves

- **PK:** `id`
- **FK salientes:** ninguna (es el ancla)
- **Únicas:** `email` (una cuenta por correo)
- **Índices:** `node_id` (detección anti-abuso multi-identidad desde el mismo hardware)

## Quién la referencia (FK entrante)

Prácticamente todo el substrato — 18 tablas tienen `owner_id → accounts(id) ON DELETE RESTRICT`: `audit_events`, `jobs`, `licenses`, `usage_records`, `consent_records`, `domain_events`, `generated_reports`, `api_credentials`, `api_usage_records`, `verified_accounts`, `attested_track_records`, `instance_backups`, `custody_state`, `account_hierarchy`, `override_attestations`, `data_portability_requests`, `operator_roles`, `operator_assignments`, `operator_role_events`. Además, `permission_decisions` la referencia opcionalmente (nullable).

## Notas de diseño

- Guardarraíl ADR-0093: NUNCA almacena contraseñas en texto plano, credenciales de bróker ni IPs de servidores live — ninguna columna existe para eso.
- `ON DELETE RESTRICT` en todas las FK entrantes: nunca se puede borrar una cuenta con historial asociado.
