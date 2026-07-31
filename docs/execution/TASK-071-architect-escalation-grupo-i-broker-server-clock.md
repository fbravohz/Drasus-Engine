# TASK-071 · Escalamiento al Architect — la tabla del reloj de bróker nació sin su contrato de auditoría

| Campo | Valor |
|---|---|
| **ID** | TASK-071 |
| **Tipo** | Task (escalamiento — inconsistencia entre dos decisiones vigentes) |
| **Estado** | ✅ **RESUELTA (2026-07-26)** — el Architect dictaminó que el Grupo I era implícito; texto de ADR-0162 corregido; implementación encadenada a STORY-070 como criterio 13 |
| **Épica (Fase)** | EPIC-1 — Soberanía de Datos (`ingest`) |
| **Creada** | 2026-07-26 |
| **Origen** | El Rust-Engineer lo señaló al implementar STORY-070: implementó la letra exacta del ADR y **reportó la discrepancia en vez de desviarse en silencio** — comportamiento correcto |
| **Documentos en conflicto** | `docs/adr/ADR-0162.md` (enmienda 2026-07-26, especificación de `broker_server_clock`) vs. `docs/adr/ADR-0020.md` (Grupo I universal) |
| **Bloquea** | Nada. La tabla funciona; lo que falta es su rastro de auditoría y su control de concurrencia. |

---

## 1. El hecho

La enmienda de ADR-0162 especifica `broker_server_clock` con **cinco campos**: `id`, `venue`, `clock_anchor_timezone_ref`, `clock_fixed_offset_minutes`, `venue_operational_since`.

ADR-0020 exige el **Grupo I universal** en toda tabla de persistencia: `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, y —según la naturaleza— `event_sequence_id` (append-only) o `row_version` (mutable).

**Las tres tablas hermanas del mismo catálogo, en la misma migración, sí lo llevan.** `session_template` declara `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `row_version`, `node_id`, `process_id` antes de sus campos de dominio. `broker_server_clock` declara solo `id`.

## 2. Por qué no lo resolvió el ingeniero, y por qué tampoco el Tech-Lead

El ingeniero implementó **la letra del ADR** y lo señaló. Es lo correcto: un ADR es una decisión sellada, y desviarse de ella en silencio —aunque sea "para mejorarla"— es exactamente el fallo que la disciplina documental existe para impedir.

El Tech-Lead tiene autoridad para corregir una feature que contradiga un ADR, pero **no para corregir un ADR** — y aquí el conflicto es entre dos ADRs vigentes, no entre una feature y un ADR. Decide el Architect.

## 3. Por qué importa, en concreto

No es simetría cosmética. Esta tabla guarda dos hechos con consecuencias:

- **El reloj del bróker**, que es el borrador desde el que se pre-rellena la declaración que el usuario sella como evidencia inmutable.
- **La fecha desde la que ese bróker opera**, que es lo que permite rechazar un tramo declarado como suyo cuando antecede a su existencia.

Sin `row_version`, dos ediciones concurrentes de esa fila se pisan sin que nada avise — el resto del catálogo usa concurrencia optimista precisamente para que no pase. Sin `created_at`/`updated_at`/`audit_hash`, no queda rastro de qué decía el catálogo cuando se selló una declaración anterior: la evidencia de cada importación sí queda completa en su propio registro, pero **el borrador que la originó es irreconstruible**.

## 4. Lo que hay que decidir

1. **¿La lista de cinco campos del ADR era la especificación completa, o solo los campos de dominio** dando el Grupo I por implícito? La redacción no lo dice, y las tres tablas hermanas sugieren lo segundo.
2. **Si es lo segundo, ¿qué perfil y qué grupos le corresponden** a una tabla de referencia mutable de este catálogo? Las hermanas usan Grupo I completo + `row_version` + `node_id`/`process_id`.
3. **¿Amerita enmienda del texto de ADR-0162** para que la próxima lectura no reproduzca la misma ambigüedad, o basta con corregir la implementación?

## 4bis. Resolución del Architect (2026-07-26)

**Los cinco campos eran únicamente los de dominio; el Grupo I estaba implícito.** No es lectura discrecional: ADR-0020 lo fija sin margen —"Grupo I universal: aparece en toda tabla sin excepción"— y solo los Grupos II–V se filtran por perfil. No existe mecanismo para eximir a una tabla del Grupo I, y nada en la enmienda del 2026-07-26 argumentó abrir esa excepción. **Fue omisión de redacción** al añadir la tabla en una sección separada de la que trae el recordatorio del contrato universal, no una decisión deliberada.

`broker_server_clock` es Referencia/Mutable, propiedad de `instrument-catalog`, misma naturaleza que sus tres hermanas del catálogo — por tanto lleva Grupo I completo con control de concurrencia optimista (no secuencia de eventos, que es de las tablas de solo-añadir) más el Grupo IV de infraestructura.

**Texto corregido** en `docs/adr/ADR-0162.md`: el bloque de esquema de la tabla ahora declara explícitamente qué grupos lleva y por qué, cita la regla de ADR-0020 que lo obliga, y aclara que los cinco campos listados son solo el dominio. La trazabilidad del ADR referencia esta Task.

**Implementación:** encadenada a [STORY-070](./STORY-070-declaracion-por-tramo-retrofit.md) como **criterio de aceptación 13** — misma tabla, misma migración y mismo crate que esa Story ya tiene abiertos, así que el gate de mutación corre una sola vez cubriendo ambos alcances.

## 5. Trabajo de implementación que se derivará

Si el Architect resuelve que faltan campos, el retrofit es de la misma naturaleza que STORY-070: edición in-situ de `migrations/0022` en fase greenfield, más la ficha `docs/data-model/broker_server_clock.md` y el repositorio de la tabla. Acotado, sin dato real que migrar.
