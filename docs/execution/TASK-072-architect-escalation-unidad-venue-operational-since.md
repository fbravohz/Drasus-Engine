# TASK-072 · Escalamiento al Architect — la fecha de operación del bróker viaja sin su unidad sellada

| Campo | Valor |
|---|---|
| **ID** | TASK-072 |
| **Tipo** | Task (escalamiento — nombre de campo fijado por un ADR vs. convención universal del proyecto) |
| **Estado** | ✅ **RESUELTA (2026-07-27)** — el Architect dictaminó que el nombre era notación de documento, no decisión sellada; ADR-0162 corregido a `venue_operational_since_ns`; regla canónica de sufijo de unidad añadida a ADR-0141 (M13); retrofit de código encadenado a la próxima Story que toque `instrument-catalog`/`data-import-wizard` |
| **Épica (Fase)** | EPIC-1 — Soberanía de Datos (`ingest`) |
| **Creada** | 2026-07-27 |
| **Origen** | Barrido de unidades del Tech-Lead sobre la entrega del criterio 13 de STORY-070 (regla activa desde STORY-068, `.agents/skills/tech-lead/SKILL.md` Etapa 7) |
| **Documentos implicados** | `docs/adr/ADR-0162.md` (enmienda 2026-07-26, tabla de esquema de `broker_server_clock`) vs. convención universal de nomenclatura temporal del repositorio |
| **Bloquea** | Nada operativo. El valor es correcto hoy; lo que falta es que diga en qué unidad está. |

---

## 1. El hecho

`venue_operational_since` es **nanosegundos UTC**. Así lo declara ADR-0162 en su tabla de esquema, así lo comenta la migración, y así se compara: contra `declared_range_start_ns`.

**Es el único timestamp de dominio del repositorio cuyo nombre no lleva su unidad.** El barrido sobre los cinco crates de `crates/features/data/` no encontró otra excepción: `declared_range_start_ns`, `declared_range_end_ns`, `bar_open_timestamp_ns`, `bar_close_timestamp_ns`, `event_timestamp_ns`, `target_interval_ns`, `transform_duration_ns`, `avg_holding_time_ns`, `start_ns`, `end_ns`, `now_ns`. La única otra familia sin sufijo es `created_at`/`updated_at`, y esos son Grupo I universal con unidad fijada por ADR-0020 para todo el proyecto — no campos de dominio.

## 2. La prueba de que es descuido de redacción y no decisión

**La misma entrega nombra el mismo valor de las dos maneras, según el archivo.** En el núcleo puro de `data-import-wizard` el campo se llama `venue_operational_since_ns` (`domain.rs`: parámetro de `detect_precedes_venue_operational_date`, variante de error `PrecedesVenueOperationalDate`, dato de prueba). En cuanto cruza al puerto, al orquestador, al esquema y a la columna SQL, pierde el sufijo.

Nadie decidió eso. El instinto del ingeniero en el núcleo fue sellar la unidad; en los bordes copió el nombre literal del ADR. Mismo patrón que TASK-071: la enmienda usa el sufijo en los campos temporales que introduce en la misma sección (`declared_range_start_ns`, `declared_range_end_ns`) y lo omite una vez.

## 3. Por qué importa, en concreto

Este campo existe para **rechazar** un tramo declarado como de un bróker que aún no operaba. Su único uso es una comparación contra un instante en nanosegundos.

Un `i64` llamado "desde cuándo opera" admite lecturas plausibles y silenciosas: segundos desde epoch, milisegundos, o incluso `AAAAMMDD`. Cualquiera de ellas hace que la comparación **siempre se cumpla** — es decir, el gate deja de rechazar nada y nadie se entera, porque no hay error, hay un chequeo que pasa siempre.

Es la misma familia de fallo que el spread de STORY-068: el código es correcto, el valor es correcto, y lo que falta es que el dato diga qué significa. Aquel lo cazó el propietario, no un gate. Este lo caza el barrido que se instauró por aquel.

Agravante propio: el campo **cruza la frontera entre dos features** (`instrument-catalog` lo produce, `data-import-wizard` lo consume por inyección) y en el trayecto cambia de nombre. Quien lea solo un lado no ve la unidad.

## 4. Lo que hay que decidir

1. **¿El nombre `venue_operational_since` de la tabla de esquema de ADR-0162 es parte de la decisión sellada, o notación del documento** que la convención universal de nomenclatura puede corregir?
2. **Si es lo segundo:** ¿se renombra a `venue_operational_since_ns` en columna, structs, puerto y fichas, quedando el repositorio sin excepciones?
3. **¿Amerita enmienda del texto de ADR-0162**, o basta con corregir implementación y fichas?
4. **Pregunta de fondo, más allá de este campo:** ¿debe existir una regla canónica escrita —hoy es convención observada, no norma— que obligue al sufijo de unidad en todo campo temporal de dominio? Sin ella, este mismo descuido se repetirá, y el barrido manual del Tech-Lead es el único filtro que lo detecta.

## 4bis. Resolución del Architect (2026-07-27)

**No acepto la premisa por analogía con TASK-071 — la verifiqué por su cuenta.** TASK-071 era una omisión de un grupo de campos completo (Grupo I) frente a una regla que ya se aplicaba sin excepción a cuatro tablas hermanas en la misma migración: la evidencia de "descuido" era la inconsistencia intra-migración. Aquí la evidencia es distinta y más fuerte todavía: `ADR-0141 §"Timestamps"` (línea 56, vigente desde el ADR original, sin tocar por ninguna enmienda posterior) **ya distingue por nombre** `created_at` (Grupo I, tiempo de procesamiento) de `event_timestamp_ns`/`bar_timestamp_ns` (dominio, tiempo del evento) — es la fuente normativa de tipos que ADR-0162 declara seguir "sin excepción" (banner de origen, línea 6). `venue_operational_since` es un campo de dominio (hecho externo sobre un bróker, no un campo universal de auditoría de fila) y por tanto ya caía, desde antes de escribirse, bajo el patrón `_ns` que ADR-0141 ejemplifica. Nadie argumentó nunca, en ningún punto de ADR-0162 ni de sus tres enmiendas, una razón para que este campo fuera la excepción — el propio ADR-0162 lo describe en prosa como "Nanosegundos UTC" en la misma celda donde omite el sufijo en el nombre. Eso es la firma de una notación de documento, no de una decisión.

**Pregunta 1 — ¿decisión sellada o notación?** Notación. Lo sellado en ADR-0162 es la *semántica* del campo (nullable = no verificado, hecho externo declarado/verificado por el usuario, nunca inferido del archivo, alimenta el rechazo del Gate de Homogeneidad) — no el string literal `venue_operational_since`. Ninguna de las tres enmiendas de ADR-0162 (2026-07-25, 2026-07-26, 2026-07-26bis) discute o defiende la ausencia del sufijo; el campo simplemente se transcribió sin él.

**Pregunta 2 — ¿se renombra sin excepciones?** Sí: `venue_operational_since` → `venue_operational_since_ns` en la columna SQL (`migrations/0022`), en los tres structs de `schemas.rs` de `instrument-catalog`, en `persistence.rs`, en el puerto/orquestador de `data-import-wizard` (el núcleo puro ya usa el nombre correcto — `domain.rs` no cambia) y en toda ficha de datos/feature que lo mencione. Sin cambio de comportamiento, sin cambio de tipo, sin nuevos mutantes — así lo dejó ya acotado el Tech-Lead en §5 de esta Task (sin modificar, sigue vigente).

**Pregunta 3 — ¿amerita enmienda de ADR-0162?** Sí, y ya quedó aplicada: la tabla de esquema de `broker_server_clock` en `docs/adr/ADR-0162.md` es la fuente canónica del nombre de columna — dejarla desalineada del nombre real reproduciría exactamente el defecto que esta Task reporta (el ADR diciendo una cosa, el código otra). Corregida con el mismo patrón que TASK-071 ("Corrección 2026-07-27, TASK-072"): banner al inicio del documento + celda de la tabla + Trazabilidad. No se creó ADR nuevo — es la misma decisión, corregida en su archivo, según el Protocolo de Mantenimiento de ADRs (Anti-Append-Only) de este skill.

**Pregunta 4 — ¿regla canónica nueva, y dónde vive?** Sí, y era una laguna real: el patrón `_ns` para campos de dominio existía como *ejemplo ilustrativo* en ADR-0141 §Timestamps desde el origen del ADR, pero nunca se elevó a regla `FIJO` verificable ni a check del Gate del Tech-Lead (M1–M12) — por eso dependía de que un humano lo notara a mano, dos veces en dos días, en la misma tabla. **Ubicación decidida: ADR-0141, no un ADR nuevo ni la base de conocimiento.** Razón: ADR-0141 es ya la fuente única de las convenciones de nombres y del mapeo de tipos de todo el esquema ("Convenciones de nombres y mapeo de tipos", `FIJO`) y del Gate de coherencia que el Tech-Lead corre en cada migración (M1–M12) — es exactamente donde ya vive la distinción `created_at` vs. `event_timestamp_ns` que esta regla generaliza; crear un ADR nuevo fragmentaría una decisión de tipado que ya tiene dueño, y la base de conocimiento no es el lugar correcto para una regla que el Gate del Tech-Lead debe poder verificar por checklist en cada migración SQL. Se agregó como enmienda 2026-07-27 de ADR-0141: bullet nuevo en "Convenciones (FIJO)" + fila `M13` en la tabla de checks. Ver `docs/adr/ADR-0141.md`.

**Correcciones documentales aplicadas por esta resolución** (Anti-Append-Only, ambas direcciones):
- `docs/adr/ADR-0141.md` — banner de enmienda, bullet de regla en "Convenciones (FIJO)", fila `M13` en la tabla de checks, Trazabilidad.
- `docs/adr/ADR-0162.md` — banner de corrección, columna renombrada en la tabla de esquema de `broker_server_clock`, referencia de prosa en el Gate de Homogeneidad, Trazabilidad.
- `docs/data-model/broker_server_clock.md`, `docs/features/instrument-catalog.md`, `docs/features/pit-data-validator.md`, `docs/features/data-import-wizard.md` — toda mención de `venue_operational_since` pasa a `venue_operational_since_ns`.

**Lo que esta resolución NO hace:** no toca `migrations/0022`, `schemas.rs`, `persistence.rs`, ni el puerto/orquestador de `data-import-wizard` — retrofit de código, alcance del ingeniero que despache el Tech-Lead (mandato del Architect en esta sesión: solo documentación/ADR). §5 abajo, sin cambios, ya listaba el alcance mecánico exacto.

## 5. Alcance del retrofit, si procede

Mecánico y acotado: `migrations/0022` (columna, fase greenfield, edición in-situ), `schemas.rs` × 3 structs y `persistence.rs` de `instrument-catalog`, puerto y orquestador de `data-import-wizard` (el núcleo ya está correcto), `docs/data-model/broker_server_clock.md`, `docs/features/instrument-catalog.md`, `docs/features/pit-data-validator.md`, `docs/features/data-import-wizard.md`.

**Sin cambio de comportamiento:** es un renombrado puro. No añade ni elimina mutantes, así que no exige relanzar el gate de mutación — basta comprobar que el recuento de mutantes no varía.
