# account_hierarchy

**Feature dueña:** [`master-account-hierarchy`](../features/master-account-hierarchy.md)
**Migración:** `migrations/0018_master_account_hierarchy.sql`
**Naturaleza:** Mutable (`row_version`) — es el PUNTERO, no el árbol: cada fila solo sabe su propio `parent_owner_id`.
**Perfil ADR-0020:** Perfil D — Grupo I + `owner_id`/`parent_owner_id` (II, sin `institutional_tag`) + `node_id` (IV)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | Cambia con cada UPDATE |
| `audit_hash` | TEXT | NO | SHA-256 de esta versión |
| `audit_chain_hash` | TEXT | SÍ | NULL en fila génesis |
| `row_version` | INTEGER | NO | Arranca en 1 |
| `owner_id` | TEXT | NO | **`UNIQUE`** — la hija; exactamente una fila por hija |
| `parent_owner_id` | TEXT | SÍ | El fondo — NULL = sin padre (huérfana todavía no vinculada) |
| `consent_ref` | TEXT | NO | Referencia CACHEADA al consentimiento vigente (no la verdad legal — esa se re-resuelve contra `consent-registry`) |
| `node_id` | TEXT | NO | Máquina que registró/actualizó |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`
- **Únicas:** `owner_id`
- **Referencias no-FK (soft):** `parent_owner_id → accounts.id` (nullable) — **observación abierta**: fuera del alcance textual de la enmienda ADR-0141 M6 (que solo fija la regla para columnas literalmente nombradas `owner_id`), reportada al Tech-Lead en la propia migración
- **Índices:** `parent_owner_id`

## Quién la referencia (FK entrante)

- Ninguna.

## Notas de diseño

- Anti-`tenant_id`: no existe columna ni índice que reconstruya "todas las hijas de un fondo" salvo la consulta explícita por `parent_owner_id` (regla fija #1, ADR-0147).
