# Plantilla: Modelo de Datos (una tabla real de SQLite)

**¿Cuándo usar?** Una ficha por cada tabla `CREATE TABLE` que exista en `migrations/*.sql`. Es la única categoría de documento del proyecto donde el detalle técnico literal (nombre de columna, tipo SQL, `CHECK`, `FOREIGN KEY`) **no está prohibido** — al revés que ADR/Feature/TTR (que describen comportamiento, nunca código), el propósito exclusivo de esta ficha es reflejar la tabla real 1:1, para que el propietario entienda relaciones entre tablas sin abrir el `.sql` ni el código.

**Regla rectora:** si esta ficha y la migración real (`migrations/*.sql`) alguna vez difieren, la migración gana — y la ficha se corrige de inmediato en el mismo cambio que tocó la migración (regla de sincronización obligatoria, ver `docs/DATA-MODEL.md` §Protocolo).

## Formato

```markdown
# <nombre_tabla>

**Feature dueña:** [`<feature>`](../features/<feature>.md)
**Migración:** `migrations/00NN_<archivo>.sql`
**Naturaleza:** MUTABLE (`row_version`) | APPEND-ONLY (`event_sequence_id` + triggers anti UPDATE/DELETE) | ESTADO (clave-valor) | REFERENCIA (catálogo/ancla)
**Perfil ADR-0020:** <perfil o subset de grupos>

## Columnas

| Columna | Tipo SQL | Null | Descripción |
|---|---|---|---|
| `id` | TEXT | NO | PK, UUID/UUIDv7 |
| ... | ... | ... | ... |

## Claves

- **PK:** `id`
- **FK salientes:** `<columna>` → `<tabla>(<columna>)` `ON DELETE <política>`
- **Únicas:** `UNIQUE(<columnas>)` — qué invariante de negocio protege
- **Referencias no-FK (soft, informativas):** `<columna>` → `<tabla>.<columna>` — por qué no lleva FK física

## Quién la referencia (FK entrante)

- `<tabla_hija>.<columna>` — para qué

## Notas de diseño

- Por qué es MUTABLE/APPEND-ONLY, cualquier decisión de esquema no obvia (citar el ADR si aplica).
```

## Protocolo de sincronización (obligatorio)

Toda migración nueva o modificada en `migrations/*.sql` actualiza su ficha `docs/data-model/<tabla>.md` correspondiente **en el mismo cambio** — igual que una Feature nueva exige su ADR. Si la migración crea una tabla sin ficha, o la ficha describe una columna que la migración ya no tiene, el 100% de responsabilidad es de quien tocó la migración (Tech-Lead/ingeniero Rust), no del Architect — este archivo es la plantilla, `docs/DATA-MODEL.md` es el índice con el protocolo completo.

---

Ver reglas transversales generales (Regla de Oro, Checklist) en [`TEMPLATES.md`](./TEMPLATES.md) — **excepto** la prohibición de pseudocódigo/nombres literales, que no aplica a esta plantilla por diseño.
