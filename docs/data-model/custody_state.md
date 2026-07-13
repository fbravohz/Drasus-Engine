# custody_state

**Feature dueña:** [`instance-continuity`](../features/instance-continuity.md)
**Migración:** `migrations/0017_instance_continuity.sql`
**Naturaleza:** Mutable — `custody_epoch` (mismo rol que `row_version`, nombre de dominio fijado por ADR-0146: concurrencia optimista aplicada a nivel de INSTANCIA COMPLETA).
**Perfil ADR-0020:** Perfil D — Grupo I (con `custody_epoch`) + Grupo II + `titular_node_id` (IV)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | Cambia con cada reclamo de titularidad |
| `audit_hash` | TEXT | NO | SHA-256 de esta versión |
| `audit_chain_hash` | TEXT | SÍ | NULL en fila génesis |
| `custody_epoch` | INTEGER | NO | Arranca en 1, +1 por reclamo exitoso |
| `owner_id` | TEXT | NO | **`UNIQUE`** — exactamente una fila de custodia por dueño |
| `institutional_tag` | TEXT | NO | Entorno |
| `titular_node_id` | TEXT | NO | Máquina titular ESCRITORA vigente de la cadena de auditoría |

## Claves

- **PK:** `id`
- **FK salientes:** `owner_id → accounts(id) ON DELETE RESTRICT`
- **Únicas:** `owner_id` (una fila de custodia por cuenta)

## Quién la referencia (FK entrante)

- Ninguna.

## Notas de diseño

- Implementa el "maestro itinerante" (ADR-0146): la titularidad escritora se mueve de máquina en máquina, pero solo una máquina es titular a la vez por cuenta. `owner_id UNIQUE` hace que ese invariante sea irrompible a nivel de esquema, no solo de disciplina de código.
