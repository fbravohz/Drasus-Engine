# exportable_data_catalog

**Feature dueña:** [`data-portability`](../features/data-portability.md)
**Migración:** `migrations/0019_data_portability.sql`
**Naturaleza:** Mutable (`row_version`) — metadato de ESQUEMA (análogo a `foundation_master_fields`), no un hecho ligado a un dueño.
**Perfil ADR-0020:** Solo Grupo I — sin `owner_id`/`institutional_tag` (no aplica: es catálogo de esquema, no dato de usuario)

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUIDv7 |
| `created_at` | INTEGER | NO | Nanosegundos desde epoch |
| `updated_at` | INTEGER | NO | Cambia con cada UPDATE |
| `audit_hash` | TEXT | NO | SHA-256 de esta versión |
| `audit_chain_hash` | TEXT | SÍ | NULL en fila génesis |
| `row_version` | INTEGER | NO | Arranca en 1 |
| `table_name` | TEXT | NO | **`UNIQUE`** — nombre de la tabla catalogada |
| `feature_name` | TEXT | NO | Feature dueña de esa tabla |
| `owner_id_column` | TEXT | NO | Nombre de la columna `owner_id` EN esa tabla |
| `retention_exempt` | INTEGER | NO | 1 = obligación de retención legal (se pseudonimiza, nunca se purga el contenido) |

## Claves

- **PK:** `id`
- **FK salientes:** ninguna
- **Únicas:** `table_name`

## Quién la referencia (FK entrante)

- Ninguna.

## Notas de diseño

- Es el catálogo declarativo de qué tablas del substrato portan `owner_id`, poblado incrementalmente por cada feature nueva (mismo mecanismo de Inundación de Fundaciones que `foundation_master_fields`). Lo lee `data_portability_requests` al ejecutar un export/olvido.
