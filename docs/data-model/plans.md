# plans

**Feature dueña:** [`plan-tier-quota`](../features/plan-tier-quota.md)
**Migración:** `migrations/0009_plan_tier_quota.sql`
**Naturaleza:** Mutable (`row_version`) — catálogo comercial, dato no código.
**Perfil ADR-0020:** Perfil D acotado — Grupo I + `owner_id`/`institutional_tag` (II, sin `access_token_id`) + `node_id` (IV)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | Cambia con cada revisión de límite/precio |
| `audit_hash` | TEXT | NO | SHA-256 de esta versión |
| `audit_chain_hash` | TEXT | SÍ | NULL en fila génesis |
| `row_version` | INTEGER | NO | Arranca en 1 |
| `owner_id` | TEXT | NO | Creador del plan — **sin FK**, el catálogo real lo define la Cabina de Mando |
| `institutional_tag` | TEXT | NO | Entorno |
| `node_id` | TEXT | NO | Máquina que registró la definición (informativo) |
| `tier` | TEXT | NO | `FREE`\|`PAID` (vocabulario propio, distinto del `tier` de `licenses`) |
| `notional_limit` | INTEGER | NO | Volumen nocional permitido, ×10⁸. `0` = sin tope propio |
| `max_activations` | INTEGER | NO | Activaciones máximas (máquinas distintas) |
| `price` | INTEGER | NO | Precio del plan, ×10⁸. `0` = gratuito |
| `pricing_model` | TEXT | NO | `FLAT`\|`VOLUME` |
| `features_enabled` | TEXT | NO | JSON lista ordenada de features habilitadas, default `'[]'` |
| `max_child_accounts` | INTEGER | NO | Cuentas maestras hijas permitidas (#12), default 0 |

## Claves

- **PK:** `id`
- **FK salientes:** ninguna (`owner_id` no es FK en esta tabla — excepción documentada)
- **Índices:** `owner_id`, `tier`

## Quién la referencia (FK entrante)

- Ninguna (`licensing-system` y `usage-metering` LEEN este catálogo vía el puerto `plan_limits_out`, nunca por FK ni acceso directo — ADR-0137).

## Notas de diseño

- NUNCA ambas cuotas (`notional_limit`, `max_activations`) en cero a la vez (rechazado por el Core `validate_plan`).
- `features_enabled` es JSON en vez de tabla hija M:N porque el conjunto no tiene atributos propios (ADR-0141 "Patrón M:N").
