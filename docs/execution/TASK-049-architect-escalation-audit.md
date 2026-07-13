# TASK-049 · Paquete de escalamiento al Architect (auditoría retroactiva EPIC-0)

> Task SIN código. El Tech-Lead reúne aquí, con evidencia concreta, los hallazgos de la auditoría retroactiva que **NO son suyos de corregir** (tocan ADR/SAD/CLAUDE.md o requieren una decisión de diseño). El Architect es quien edita esos documentos. Se entrega al usuario para que invoque al Architect (`/architect`).

| Campo | Valor |
|---|---|
| **ID** | TASK-049 |
| **Tipo** | Task (escalamiento) |
| **Épica** | EPIC-0 — Fundación (retroactiva) |
| **Estado** | ✅ Resuelta (2026-07-11/12) — Architect decidió E1–E6; ver §Resolución |
| **Creada** | 2026-07-10 |
| **Origen** | Diagnóstico de la auditoría (6 lotes) — plan `.agents/plans/magical-sprouting-quasar.md` §"Desincronización documental" |

## Regla de contraste (para el Architect)

Cada ítem se presenta con **contraste bidireccional**: puede estar equivocado el documento, o puede estar equivocada la premisa del hallazgo. El Architect decide. El Tech-Lead NO ha tocado ninguno de estos documentos.

## Ítems de decisión / edición del Architect

### E1 — ADR-0137 nunca enmendado para los cimientos #11–#14
- **Evidencia:** los feature docs `instance-continuity.md`, `master-account-hierarchy.md`, `data-portability.md`, `operator-roles.md` afirman que sus tipos de puerto están "registrados en el catálogo de ADR-0137 vía la enmienda de ADR-014N", pero `docs/adr/ADR-0137.md` solo tiene enmiendas reales hasta 2026-07-04 (pilar #10). Los 4 cimientos son de 2026-07-06/07, posteriores.
- **Naturaleza:** deuda de gobernanza, NO bug de código — la residencia en `crates/shared` es arquitectónicamente correcta por la excepción bendecida; solo falta reflejar los tipos `textLabel` en el catálogo canónico.
- **Acción propuesta:** enmienda quirúrgica de ADR-0137 (tabla análoga a las de 2026-07-03/07-04) registrando los tipos de puerto de #11–#14.

### E2 — `CLAUDE.md §1` lista solo 6 features de infraestructura bendecidas
- **Evidencia:** `CLAUDE.md:15` enumera `clock, audit-log, async-job-executor, telemetry, worker-isolation-orchestrator, agentic-mcp-gateway`; pero ADR-0137 (enmiendas) ya extendió la excepción a los 14 cimientos del substrato, que el código correctamente coloca en `shared`.
- **Naturaleza:** el mapa (`CLAUDE.md`) quedó atrás respecto al ADR. Es la cara inversa de E1 — conviene reconciliarlos juntos.
- **Acción propuesta:** el Architect actualiza `CLAUDE.md §1` para reflejar la lista vigente (o remite a ADR-0137 como fuente canónica en vez de duplicar la lista).

### E3 — `owner_id` sin FK física a `accounts` (sistémico)
- **Evidencia:** `usage_records` (`migrations/0010`) y `consent_records` (`migrations/0011`) — y el patrón se repite en el substrato — declaran `owner_id` sin `REFERENCES accounts(id)`; la dependencia hacia `central-identity` (#1) vive solo en la prosa de los feature docs.
- **Naturaleza:** decisión de modelado relacional canónica — relacionada con la activación de `PRAGMA foreign_keys=ON` (hallazgo C1, ya corregido en STORY-045). Con las FK ahora activas, conviene decidir si el `owner_id`→`accounts` se vuelve FK física en todo el substrato (implica orden de creación/borrado) o se mantiene como referencia lógica documentada.
- **Acción propuesta:** decisión única en ADR-0141 (no parches sueltos por tabla). Si es FK física → genera una Story de esquema greenfield.

### E4 — `proptest` (ADR-0133 Capa 3) nunca activado — contraste no resuelto
- **Evidencia:** 0 dependencias `proptest` en el workspace, pese a funciones cuantitativas financieras puras: `usage_metering.rs::compute_notional/accumulate`, `data_aggregation.rs::apply_differential_privacy`. Los property tests existentes son **enumeración exhaustiva manual** (ej. `operator_roles.rs`).
- **Contraste bidireccional:** ¿el texto de ADR-0133 exige `proptest` como herramienta (→ falta implementarlo), o la enumeración exhaustiva manual satisface la intención de la Capa 3 (→ enmendar ADR-0133 para aceptarla explícitamente)?
- **Acción propuesta:** el Architect resuelve: (a) añadir `proptest` a esas funciones, o (b) enmendar ADR-0133 aclarando que la enumeración exhaustiva cerrada es equivalente cuando el dominio de entrada es finito.

### E5 — Reconciliación de la infraestructura del lienzo visual (Canvas)
- **Contexto:** decisión previa del usuario de construir la infra Canvas al arrancar la auditoría, reordenando el ROADMAP (reconciliar ADR-0117/0136).
- **Hallazgo que REDUCE el frente:** el diagnóstico (Lote 5) confirmó que **la infraestructura genérica de Canvas YA existe** (`ui/lib/tabs/canvas_tab.dart`: drag-drop, nodos). El escalamiento ya no es "construir la infra", sino: (a) decidir el patrón del nodo por-feature + inspector panel; (b) acotar la deuda real (DEBT-004) al remanente correcto.
- **Corrección del propietario sobre el nombre (2026-07-11):** el código y **los identificadores de variables deben permanecer NEUTRALES ("canvas")** — nunca "Forge"/"Reactor" horneados en el código, exactamente igual que se hizo con `drasus_ui`→`custom_ui`. El nombre de exhibición (Forge/Reactor/lo-que-sea) es **UN solo string global paramétrico** (análogo a `kAppName`), intercambiable desde un único punto. Hardcoding concreto a erradicar: `canvas_tab.dart:450` hornea el literal `'Canvas · Forge'` (+ comentario en `:431`); y los tokens `reactorGreen`/`gradReactor` (`gx_tokens.dart:284`, usados en `dashboard_tab`/`fab`/`split_button`) acoplan "Reactor" a identificadores — **verificar si es un codename de color o un acople al nombre del lienzo**, y neutralizar si es lo segundo.
- **Acción propuesta:** el Architect (a) ratifica en ADR-0136 el **principio** (código/variables neutrales + nombre de exhibición como constante global única paramétrica), NO un nombre horneado; (b) decide el valor del string de exhibición o lo deja explícitamente diferido; (c) define el contrato del nodo Canvas por-feature. La parametrización mecánica del literal la ejecuta luego el TL/ingeniero; el TL reformula DEBT-004.

### E6 — Correcciones documentales menores (el Architect edita `docs/`)
- **`docs/adr/ADR-0136.md`:** dice "Forge/Reactor — TBD". Ver **E5**: el nombre NO es un valor a fijar en código, sino un string de exhibición paramétrico. Ratificar en ADR-0136 el principio (código neutral + constante global única), no un nombre horneado.
- **`migrations/0007/0008/0009` (comentarios):** prometen auditoría vía `audit_events`; el código usa (correctamente) hash-chain propio por fila, igual que `jobs`. Corregir el comentario. *(Nota: el comentario está en archivos SQL; si el usuario prefiere, el TL puede corregir el texto del comentario sin cambiar esquema — pero se lista aquí por coherencia con la promesa de diseño.)*
- **`docs/features/licensing-system.md` TTR-001:** describe "HMAC-SHA256" — es la huella de `node_id`, no la firma de licencia (que es Ed25519, correcta). Aclarar para no confundir con una violación de ADR-0093.
- **`docs/features/institutional-report-engine.md`:** cita ADR-0101 (que es transpilación AST→MQL4/5, no reportes) para justificar su render Tera. Redactar ADR propio o ampliar el alcance de otro ADR al construir la Story de render.
- **`docs/features/verified-account-registry.md`:** contradicción interna — el banner dice "pagado (STORY-041)", el cuerpo (línea 95) dice "retrabajo pendiente". Alinear al estado real (pagado).

## Qué NO entra aquí (lo corrige el Tech-Lead, no el Architect)

- DEBT.md: ampliar DEBT-018 (#4/#7), reformular DEBT-004/005 — dominio del TL.
- PROGRESS.md: corregir la nota rezagada de #14 — dominio del TL.
- Todo lo de código (STORY-045…048) — dominio del TL/ingenieros.

## Cierre

Este paquete se entrega al usuario. El Architect (Opus, invocado por el usuario con `/architect`) decide E1–E6 y edita los documentos de `docs/`/`CLAUDE.md` que correspondan. Tras su decisión, el Tech-Lead relee los documentos modificados y traduce cada regla nueva a checks del Gate de Coherencia (cierre del bucle de trazabilidad).

## Resolución (2026-07-11/12) — verificada por el Tech-Lead archivo por archivo y en git

El Architect procesó el paquete en una sesión previa. Estado de cada ítem:

- **E1 ✅ RESUELTO** — `docs/adr/ADR-0137.md` gana la **enmienda 2026-07-11** que registra formalmente los tipos de puerto `textLabel` de #11–#14 en el bloque "Infraestructura (crosscutting)". Deuda de gobernanza cerrada.
- **E2 ✅ RESUELTO** — `CLAUDE.md §1` remite ahora a ADR-0137 como fuente canónica de la lista de cimientos (14), en vez de duplicar los 6 originales.
- **E3 ✅ RESUELTO** — FK física `owner_id → accounts(id) ON DELETE RESTRICT` + índice de FK-hijo (M7) homogeneizada en el substrato (commit `7e870f0`, ADR-0141 enmienda 2026-07-11). 10 migraciones portan `REFERENCES accounts`.
- **E4 ✅ DECIDIDO — con cabo de código pendiente** — `docs/adr/ADR-0133.md` gana la **enmienda 2026-07-11**: la enumeración exhaustiva de un dominio finito satisface la Capa 3 (más fuerte que `proptest`); `proptest` sigue obligatorio en dominios numéricos NO acotados. **Residual de código (dominio del TL):** falta `proptest` en `usage_metering.rs::compute_notional`/`accumulate` y `data_aggregation.rs::apply_differential_privacy` → rastreado como **DEBT-023**.
- **E5 ✅ RESUELTO** — nombre del lienzo parametrizado en `kCanvasName` (`ui/lib/app_meta.dart`, commit `032727c`); identificadores de código neutrales. `reactorGreen`/`gradReactor` verificados como **codename de color** (token de "encendido"), no acople al nombre del lienzo → sin acción.
- **E6 ✅ RESUELTO** — comentarios de `migrations/0008/0009` corregidos (commit `2bcee71`); contradicción de `verified-account-registry.md` (banner vs. línea 95) alineada al estado real (pagado, STORY-041).

**Cierre del bucle de trazabilidad:** las reglas nuevas (distinción de cardinalidad de ADR-0133; FK física de ADR-0141; catálogo cerrado de ADR-0137) se incorporan al Gate de Coherencia por referencia a sus ADR. Único trabajo de código remanente: **DEBT-023** (proptest numérico).
