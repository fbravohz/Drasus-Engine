# verified_accounts

**Feature dueña:** [`verified-account-registry`](../features/verified-account-registry.md)
**Migración:** `migrations/0016_verified_account_registry.sql`
**Naturaleza:** Mutable (`row_version`) — el estado de publicación y los ámbitos de atestación cambian con el tiempo.
**Perfil ADR-0020:** Perfil D — Grupo I + Grupo II + `node_id` (IV)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | Cambia con cada UPDATE |
| `audit_hash` | TEXT | NO | SHA-256 de esta versión |
| `audit_chain_hash` | TEXT | SÍ | NULL en fila génesis |
| `row_version` | INTEGER | NO | Arranca en 1 |
| `owner_id` | TEXT | NO | Dueño Drasus — multi-cuenta 1:N bajo `owner_id`, NUNCA `tenant_id` |
| `institutional_tag` | TEXT | NO | **Eje B** (realidad de capital): `LIVE`\|`PAPER`\|`DEMO`\|`CHALLENGE` — valor único por cuenta |
| `node_id` | TEXT | NO | Máquina que registró la cuenta |
| `broker` | TEXT | NO | Bróker/venue |
| `leverage` | INTEGER | NO | Apalancamiento |
| `currency` | TEXT | NO | Divisa base |
| `account_type` | TEXT | NO | `FUNDED`\|`PROP`\|`OWN` |
| `publication_status` | TEXT | NO | `PRIVATE` (default FIJO) \| `PUBLIC` (tras opt-in vía consent-registry) |
| `attestation_scopes` | TEXT | NO | JSON lista de ámbitos habilitados (`SOVEREIGN`/`BROKER_READONLY`, coexistentes) |
| `broker_connection_ref` | TEXT | SÍ | Referencia NO SECRETA a la conexión de bróker (nullable) |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`
- **Índices:** `owner_id`

## Quién la referencia (FK entrante)

- `attested_track_records.verified_account_id` — **referencia suave, sin FK física** (mismo criterio del substrato).

## Notas de diseño

- El **Eje B** (`institutional_tag` aquí) es ORTOGONAL al **Eje A** (`scope` de `attested_track_records`): una cuenta PAPER corre en el MISMO entorno determinista que LIVE (no es backtesting) y por tanto SÍ es atestiguable — el Eje B solo etiqueta si el capital fue real o virtual.
- Corrección histórica (STORY-041/DEBT-016): existió una columna `capital_reality` duplicada que se eliminó — el Eje B vive en `institutional_tag`, no en un campo aparte.
