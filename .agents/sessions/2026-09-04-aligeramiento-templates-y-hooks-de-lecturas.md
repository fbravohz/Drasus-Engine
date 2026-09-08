# Sesión 2026-09-04 — Aligeramiento de templates y hooks de lecturas obligatorias

| Campo | Valor |
|---|---|
| **Archivo** | `.agents/sessions/2026-09-04-aligeramiento-templates-y-hooks-de-lecturas.md` |
| **Abierta** | 2026-09-04 00:10 |
| **Rol principal** | Context Engineer |
| **Estado** | 🟢 en curso |

---

## Índice

- [Turno 1 — 00:10 — Formato ligero de los templates, primera ronda de deuda](#turno-1--0010--formato-ligero-de-los-templates-primera-ronda-de-deuda)
- [Turno 2 — 01:05 — Migración de context-engineer al formato ligero](#turno-2--0105--migración-de-context-engineer-al-formato-ligero)
- [Turno 3 — 01:25 — Regla de citas huérfanas + fixes de Tamaño/Slug](#turno-3--0125--regla-de-citas-huérfanas--fixes-de-tamañoslug)
- [Turno 4 — 01:45 — Sección H del plan: campo Tags en el template](#turno-4--0145--sección-h-del-plan-campo-tags-en-el-template)
- [Turno 5 — 01:55 — Estado de ejecución real registrado en el plan](#turno-5--0155--estado-de-ejecución-real-registrado-en-el-plan)
- [Turno 6 — 02:05 — Poda de citas redundantes en el Pipeline](#turno-6--0205--poda-de-citas-redundantes-en-el-pipeline)
- [Turno 7 — 02:15 — Hook SessionStart para lecturas SIEMPRE universales](#turno-7--0215--hook-sessionstart-para-lecturas-siempre-universales)
- [Turno 8 — 02:30 — Gate de lecturas SIEMPRE propias de skill](#turno-8--0230--gate-de-lecturas-siempre-propias-de-skill)
- [Turno 9 — 02:48 — Reporte de sesión + cierre del plan](#turno-9--0248--reporte-de-sesión--cierre-del-plan)
- [Turno 10 — 2026-09-05 — Explicación de los 3 hooks de determinismo](#turno-10--2026-09-05--explicación-de-los-3-hooks-de-determinismo)
- [Turno 11 — Sección B1: `inject-context.py` con SQLite FTS5](#turno-11--sección-b1-inject-contextpy-con-sqlite-fts5)
- [Turno 12 — Mecánica de FTS5/bm25 + intento de subir a 10.000 + conflicto con H5](#turno-12--mecánica-de-fts5bm25--intento-de-subir-a-10000--conflicto-con-h5)
- [Turno 13 — 4 preguntas + embeddings vs. router LLM + decisión de diseño (H3)](#turno-13--4-preguntas--embeddings-vs-router-llm--decisión-de-diseño-h3)
- [Turno 14 — Pinecone/pgvector y análisis de indexado por Sección](#turno-14--pineconepgvector-y-análisis-de-indexado-por-sección)
- [Turno 15 — Indexado por Sección + Tags por Sección construidos](#turno-15--indexado-por-sección--tags-por-sección-construidos)
- [Turno 16 — Verificación: Pipeline y Ejemplos ya funcionaban](#turno-16--verificación-pipeline-y-ejemplos-ya-funcionaban)
- [Turno 17 — Presupuesto de inyección obsoleto + switch `FORCE_FULL_CONTENT`](#turno-17--presupuesto-de-inyección-obsoleto--switch-force_full_content)
- [Turno 18 — Reporte de sesión actualizado + orden de migración de skills](#turno-18--reporte-de-sesión-actualizado--orden-de-migración-de-skills)
- [Turno 19 — `skill-common.md` construido con ayuda de un subagente](#turno-19--skill-commonmd-construido-con-ayuda-de-un-subagente)
- [Turno 20 — `tech-lead/SKILL.md` migrado, primer skill del lote](#turno-20--tech-leadskillmd-migrado-primer-skill-del-lote)

---

## Turno 1 — 00:10 — Formato ligero de los templates, primera ronda de deuda

### Qué pediste

Tomar `write-like-gemini.md` como molde de estilo y reescribir `template-knowledge.md` y `template-instruction-formats.md` en un formato ligero, revisando de paso el plan grande de aligeramiento (`haz-el-plan-de-agile-sketch.md`). Regla de decisión: todo lo que no exista en `write-like-gemini.md` se elimina del template.

### Qué hice

Esta entrada consolida varios intercambios de antes de una compactación de contexto — no tengo el detalle turno a turno con la misma fidelidad que el resto del reporte, así que va agrupado.

Entré en modo Plan, validé con preguntas puntuales los 4 puntos con consecuencia técnica real (sub-IDs de bullet retirados, Lecturas en lista plana con flechas en vez de tabla, headings con `:`, Filtro de Verificación retirado ya sin esperar al juez `Stop`), y al salir de modo Plan reescribí ambos templates completos con `/high-density-writing`. Modifiqué `validate-agent-docs.py` para que el campo Tamaño se validara también en los dos templates (antes exentos) y para reconocer el nuevo formato de bullet — con mecanismo de "bloqueo + fix sugerido": el hook calcula el número exacto y lo escribe en el mensaje de error, nunca corrige solo.

En rondas sucesivas detecté y eliminé duplicación real entre ambos documentos: `template-knowledge.md` pasó de 4.254 a 1.809 caracteres (57% menos) al quitar bloques que solo repetían en prosa lo que `template-instruction-formats.md` ya definía como catálogo.

Cerré la sesión con una reflexión pedida explícitamente sobre lecciones aprendidas: agregué `CONTEXT-ENGINEER-6.3` (duplicación entre documentos con relación de catálogo) y su ítem `CONTEXT-ENGINEER-FV-8` al skill `context-engineer`, para que una reescritura completa futura no vuelva a perder de vista esa relación.

### Archivos tocados

| Archivo | Qué cambió | Por qué |
|---|---|---|
| `.agents/templates/template-knowledge.md` | Reescrito completo a formato ligero, luego podado en 3 rondas hasta 1.809 caracteres | Duplicaba el catálogo de `template-instruction-formats.md` |
| `.agents/templates/template-instruction-formats.md` | Reescrito completo a formato ligero | Catálogo de formatos de bloque, molde para el resto del corpus |
| `.agents/hooks/validate-agent-docs.py` | `TEMPLATE_FILES_SIZE_CHECKED` nuevo, `RE_SIZE` acepta bullet | Los dos templates ya no estaban exentos del chequeo de Tamaño |
| `.agents/skills/context-engineer/SKILL.md` | Agregado `CONTEXT-ENGINEER-6.3` + `CONTEXT-ENGINEER-FV-8` | Lección de la sesión: evitar duplicación catálogo/consumidor |

### Comprobado

| Qué revisé | Resultado |
|---|---|
| `validate-agent-docs.py --audit` sobre ambos templates | 0 violaciones duras, 1 aviso blando cada uno (bloque FV retirado a propósito) |

### Decisiones y por qué

- **Campo Tamaño se conserva declarado (no se calcula silencioso):** el plan grande original decía que debía desaparecer; el usuario restauró el campo a mano y pidió mantenerlo — el hook lo valida con "bloqueo + fix sugerido" en vez de auto-corregirlo, para no gastar doble contexto de relectura en cada guardado.
- **Filtro de Verificación retirado ya, sin esperar al juez `Stop`:** hueco de auto-chequeo aceptado explícitamente por el usuario mientras esa pieza (Sección G1 del plan grande) no exista.

### Pendientes y bloqueos

- **Aplicar el formato ligero a `context-engineer/SKILL.md`:** el usuario lo nombró como "próximo trabajo" pero pidió explícitamente no empezarlo todavía porque iba a compactar la conversación.

---

## Turno 2 — 01:05 — Migración de context-engineer al formato ligero

### Qué pediste

Tras la compactación: "seguimos con la siguiente tarea, cuál era recuerdas?", y al confirmarte cuál era, "adelante".

### Qué hice

Reescribí `context-engineer/SKILL.md` completo aplicando el mismo molde ya fijado en los dos templates: cabecera con bullets (`SLUG-ID`, `Tamaño`, `Lecturas` con flechas), Pipeline de 12 pasos sin cambios de fondo, 5 secciones (`CONTEXT-ENGINEER-1`, `-2`, `-3`, `-5`, `-6`) con bullets planos sin sub-ID, Filtro de Verificación retirado. Antes de escribir tuve que corregir el propio hook: `RE_SLUG` solo reconocía el literal viejo `Slug ID:` y bloqueaba cualquier archivo real (no solo los dos templates exentos) que usara `SLUG-ID:` en bullet — lo amplié para aceptar ambas formas, condición de arranque para que el formato ligero funcione sobre cualquier documento del corpus, no solo sobre plantillas.

### Archivos tocados

| Archivo | Qué cambió | Por qué |
|---|---|---|
| `.agents/skills/context-engineer/SKILL.md` | Reescrito completo a formato ligero: 20.323 → 8.755 caracteres | Aplicar el molde ya validado en los templates al propio skill |
| `.agents/hooks/validate-agent-docs.py` | `RE_SLUG` acepta `Slug ID:` (viejo) y `SLUG-ID:` (nuevo, bullet) | Bloqueaba cualquier documento real en formato ligero, no solo los templates |

### Comprobado

| Qué revisé | Resultado |
|---|---|
| `validate-agent-docs.py --audit` sobre `SKILL.md` | 0 violaciones duras, 1 aviso blando (bloque FV retirado a propósito) |
| Grep de sub-IDs (`CONTEXT-ENGINEER-N.M`) dentro del propio archivo reescrito | Limpio, ninguno colgando |

### Decisiones y por qué

- **Extender el regex del Slug en vez de solo exentar los dos templates:** el hueco no era exclusivo de plantillas — cualquier skill real que adoptara el formato ligero se habría bloqueado igual. La corrección universal evita repetir el mismo parche archivo por archivo.

---

## Turno 3 — 01:25 — Regla de citas huérfanas + fixes de Tamaño/Slug

### Qué pediste

Agregar a las instrucciones del propio Context Engineer que las citas huérfanas o IDs mal apuntados deben corregirse al tocar el archivo correspondiente, o registrarse como deuda si no. De paso, arreglar los dos hallazgos que había mencionado antes (`write-like-gemini.md` y `template-knowledge.md`).

### Qué hice

Agregué un cuarto bullet a `CONTEXT-ENGINEER-6` con la regla pedida, y la apliqué de inmediato: registré `CONTEXT-DEBT-010` con las 7 citas externas a sub-IDs de `CONTEXT-ENGINEER` que habían quedado huérfanas al colapsar los sub-IDs en el turno anterior (`hard-prohibitions.md`, 5 planes, `memory/`). Corregí `write-like-gemini.md` (los campos `SLUG-ID`/`Tamaño` tenían indentación y sintaxis rotas — no pasaban el hook) y el desfase de 14 caracteres en el campo Tamaño de `template-knowledge.md`.

### Archivos tocados

| Archivo | Qué cambió | Por qué |
|---|---|---|
| `.agents/skills/context-engineer/SKILL.md` | Nuevo bullet en `CONTEXT-ENGINEER-6`: citas huérfanas → corregir o `CONTEXT-DEBT` | Pedido explícito del usuario |
| `.agents/debt/CONTEXT-DEBT-010.md` *(nuevo)* | Ficha de deuda con las 7 citas huérfanas listadas | Aplicar la regla recién agregada de inmediato |
| `.agents/CONTEXT-DEBT.md` | Fila nueva para `CONTEXT-DEBT-010` | Registro en el índice |
| `.agents/knowledge/write-like-gemini.md` | Cabecera corregida (`SLUG-ID`/`Tamaño` en bullet real) | No pasaba el hook, indentación y sintaxis rotas |
| `.agents/templates/template-knowledge.md` | Tamaño corregido: 1.809 → 1.795 | Desfase real detectado |

### Comprobado

| Qué revisé | Resultado |
|---|---|
| `validate-agent-docs.py --audit` sobre todo `.agents/` | 48 archivos, 0 violaciones duras |

---

## Turno 4 — 01:45 — Sección H del plan: campo Tags en el template

### Qué pediste

Revisar los cambios que agregaste al plan (`haz-el-plan-de-agile-sketch.md`) e implementar lo que aplicara a los templates ya tocados y a `context-engineer`, si hacía falta.

### Qué hice

El diff del plan agregaba la Sección H completa (sistema de tags/keywords). Lo único con dónde vivir hoy era H1 (el campo `Tags:` va justo después de `Tamaño`): lo agregué a la plantilla de cabecera de `template-knowledge.md`, marcado opcional, con nota de que su vocabulario controlado (`tags-vocabulary.md`) aún no existe — solo se fija el formato. No toqué `template-instruction-formats.md` (Tags es un campo de cabecera, no un formato de bloque) ni `context-engineer/SKILL.md` (ya remite genéricamente a los templates; reexplicar el campo ahí violaría la regla de duplicación que agregamos en el turno 1).

### Archivos tocados

| Archivo | Qué cambió | Por qué |
|---|---|---|
| `.agents/templates/template-knowledge.md` | Campo `Tags:` agregado a la cabecera, nota en `TEMPLATE-KNOWLEDGE-2` | H1 de la Sección H nueva del plan |

### Comprobado

| Qué revisé | Resultado |
|---|---|
| `validate-agent-docs.py --audit` sobre `.agents/` | 0 violaciones duras |

### Decisiones y por qué

- **No tocar `context-engineer/SKILL.md`:** aplicando su propia regla `CONTEXT-ENGINEER-6.3` (no duplicar lo que el catálogo/template ya define).
- **No construir el resto de H (H2-H7):** vocabulario, `inject-context.py`, `RE_TAGS`, `validate-docs-tags.py` — infraestructura mucho más grande, fuera de lo pedido en este turno.

---

## Turno 5 — 01:55 — Estado de ejecución real registrado en el plan

### Qué pediste

Revisar el plan y actualizarlo hasta donde ya se había hecho.

### Qué hice

Agregué la sección "Estado de ejecución real" al plan, justo después de "División de trabajo", documentando con honestidad las desviaciones: el Paso 0 del hook se hizo extendiendo el check en vez de eliminarlo; la Sección A y el paso 4 de la Sección E (context-engineer) los ejecutó esta sesión y no propietario+Gemini como preveía la división original; `CONTEXT-DEBT-010` como consecuencia directa; la regla nueva de citas huérfanas; y qué parte de la Sección H se implementó.

### Archivos tocados

| Archivo | Qué cambió | Por qué |
|---|---|---|
| `.agents/plans/haz-el-plan-de-agile-sketch.md` | Nueva sección "Estado de ejecución real" | Dejar registrado qué se hizo y cómo se desvió de lo planeado |

---

## Turno 6 — 02:05 — Poda de citas redundantes en el Pipeline

### Qué pediste

Señalaste que el Pipeline de `context-engineer/SKILL.md` tenía muchísimas referencias a IDs del propio documento, y preguntaste si eso servía de algo o era relleno — tu hipótesis: solo tiene sentido citar un ID para no releer otro documento completo con grep, no para navegar el mismo documento que ya se está leyendo entero.

### Qué hice

Confirmé la hipótesis con evidencia, no solo con acuerdo: el documento se lee siempre completo, y bajo el formato ligero (sin sub-IDs) la cita ya no apunta más fino que "en algún lugar de esa Sección completa" — cinco pasos del Pipeline citaban el mismo `(CONTEXT-ENGINEER-2)` sin distinguir nada entre ellos. Grepeé si algún documento externo cita `CONTEXT-ENGINEER-P`: cero resultados, confirmando que no hay caso real de cita externa que lo justifique. Quité las 9 citas redundantes del Pipeline y corregí `TEMPLATE-INSTRUCTION-FORMATS-6` (formato Pipeline del catálogo) para que ya no lo exija — solo se cita un ID si remite a otro documento o a una Sección fuera del orden esperado. Dejé intactas las citas que sí declaran una relación real (ej. precedencia en `CONTEXT-ENGINEER-3`).

### Archivos tocados

| Archivo | Qué cambió | Por qué |
|---|---|---|
| `.agents/templates/template-instruction-formats.md` | `TEMPLATE-INSTRUCTION-FORMATS-6` ya no exige citar el ID de sección en cada paso del Pipeline; ejemplo `AUDITOR-P` corregido | Era relleno bajo el formato ligero — confirmado con grep, cero citas externas reales |
| `.agents/skills/context-engineer/SKILL.md` | 9 citas quitadas del Pipeline: 9.310 → 9.080 caracteres | Aplicar la corrección del catálogo al propio archivo |

### Comprobado

| Qué revisé | Resultado |
|---|---|
| `grep -rn "CONTEXT-ENGINEER-P"` fuera del propio archivo | 0 resultados — ninguna cita externa que dependiera del Pipeline citando IDs |
| `validate-agent-docs.py --audit` | 0 violaciones duras |

---

## Turno 7 — 02:15 — Hook SessionStart para lecturas SIEMPRE universales

### Qué pediste

Primero señalaste que mi primer intento de responder a la pregunta de determinismo (dos bullets nuevos en `TEMPLATE-INSTRUCTION-FORMATS-4`) era relleno verboso que violaba `write-like-gemini.md`, y que además no resolvía lo pedido: necesitabas un hook real que cargue en el contexto o fuerce la lectura de las lecturas obligatorias desde el inicio, no más prosa explicando el problema.

### Qué hice

Antes de escribir código, despaché una investigación (`claude-code-guide`) para verificar contra la documentación oficial el contrato exacto de `SessionStart` y si existía algo equivalente a interceptar la invocación de un Skill — no quería improvisar sintaxis de hooks de memoria. Confirmado: `SessionStart` se dispara una sola vez por razón de arranque (`startup`/`resume`/`clear`/`compact`/`fork`), antes de que el modelo procese el primer turno, e inyecta contenido por stdout plano sin depender de que el modelo decida leer nada.

Con eso, construí `.agents/hooks/session-start-siempre.py`: lee `governance.md` y `hard-prohibitions.md` completos y los imprime por stdout al arrancar cualquier sesión. Lo registré en `.claude/settings.json`. Probado en vivo simulando el JSON del evento: inyecta 20.169 caracteres correctamente. Reescribí las dos líneas verbosas del catálogo (las que habías señalado como relleno) en 3 bullets densos que ahora describen el mecanismo real, no una promesa.

Esto cerró de raíz `CONTEXT-DEBT-011`, que registré en el mismo turno tras verificar con grep que el hueco (`hard-prohibitions.md` marcado `SIEMPRE` sin ningún paso que lo forzara) existía en 14 de los 15 skills y en los 16 stubs de `.claude/skills/` — no solo en `context-engineer`.

### Archivos tocados

| Archivo | Qué cambió | Por qué |
|---|---|---|
| `.agents/hooks/session-start-siempre.py` *(nuevo)* | Hook `SessionStart`: inyecta `governance.md` + `hard-prohibitions.md` por stdout | Determinismo real de harness para las lecturas SIEMPRE universales |
| `.claude/settings.json` | Registrado el hook nuevo bajo `SessionStart` | Activarlo |
| `.agents/debt/CONTEXT-DEBT-011.md` *(nuevo)* | Ficha de deuda, luego marcada `✅ Pagada` en el mismo turno | El hueco se confirmó sistémico (14 skills + 16 stubs) y se resolvió con el hook, no archivo por archivo |
| `.agents/CONTEXT-DEBT.md` | Fila nueva para `CONTEXT-DEBT-011` | Registro en el índice |
| `.agents/templates/template-instruction-formats.md` | `TEMPLATE-INSTRUCTION-FORMATS-4` reescrito en 3 bullets densos reflejando el mecanismo real | El texto anterior era verboso y ya no era cierto una vez construido el hook |

### Comprobado

| Qué revisé | Resultado |
|---|---|
| Simulación del evento `SessionStart` vía stdin | Inyecta 20.169 caracteres correctos (`governance.md` + `hard-prohibitions.md`) |
| `grep -c "hard-prohibitions"` sobre los 15 `SKILL.md` | Exactamente 1 mención (solo la fila de la tabla) en 14 de ellos; 0 en los 16 stubs |
| `validate-agent-docs.py --audit` | 0 violaciones duras |

### Decisiones y por qué

- **Investigar antes de escribir el hook:** el propio skill exige verificar contra documentación oficial antes de crear un mecanismo nativo — evitó improvisar un contrato JSON incorrecto.
- **Resolver la deuda con infraestructura, no archivo por archivo:** parchear 14 skills y 16 stubs habría sido mucho trabajo repetido para el mismo resultado que un solo hook logra de raíz.

---

## Turno 8 — 02:30 — Gate de lecturas SIEMPRE propias de skill

### Qué pediste

Construir el mecanismo que falta: las lecturas SIEMPRE propias de un skill concreto (no las 2 universales, que ya cubre el hook anterior).

### Qué hice

No existe un hook nativo "al invocar un skill", pero `PreToolUse` puede interceptar el tool `Skill` (ve `tool_input.skill`) y negar la invocación con una razón — el modelo la recibe y puede actuar antes de reintentar. Construí dos hooks que trabajan juntos:

- `track-siempre-reads.py` (`PostToolUse` sobre `Read`): cada vez que se lee un `.md` de `.agents/knowledge/`, deja una marca por sesión.
- `gate-skill-siempre.py` (`PreToolUse` sobre `Skill`): antes de dejar invocar un skill, abre su `SKILL.md` (soporta el formato ligero nuevo y el de tabla legado), extrae las filas `SIEMPRE` con alcance distinto de "Todos" (esas ya las cubre el hook del turno anterior), y si falta alguna marca, deniega con la lista exacta de qué leer.

Probé la cadena completa simulando los tres eventos (deny en frío → marca de lectura → allow) y luego contra los 15 skills reales, uno por uno. Encontré un bug real en el camino: `rust-engineer` declara `docs/execution/<ID>.md` como SIEMPRE — es un placeholder, nunca un archivo literal, así que el gate lo habría bloqueado para siempre. Agregué detección de rutas con `<...>` para excluirlas antes de dar el mecanismo por bueno. También corregí que el slug de la marca de lectura heredaba el punto de `.agents/` y quedaba como archivo oculto — cosmético, pero lo dejé limpio.

### Archivos tocados

| Archivo | Qué cambió | Por qué |
|---|---|---|
| `.agents/hooks/track-siempre-reads.py` *(nuevo)* | Hook `PostToolUse` sobre `Read`: marca lecturas por sesión | Necesario para que el gate sepa qué ya se leyó |
| `.agents/hooks/gate-skill-siempre.py` *(nuevo)* | Hook `PreToolUse` sobre `Skill`: bloquea si faltan lecturas SIEMPRE propias | Mecanismo pedido — determinismo para lecturas por skill |
| `.claude/settings.json` | Registrados ambos hooks nuevos | Activarlos |
| `.gitignore` | Ignorado `.agents/state/.session-reads/` | Estado efímero por sesión, no se versiona |
| `.agents/templates/template-instruction-formats.md` | `TEMPLATE-INSTRUCTION-FORMATS-4` actualizado con el mecanismo del gate | Reflejar el tercer y último caso ya resuelto |

### Comprobado

| Qué revisé | Resultado |
|---|---|
| Cadena deny → lectura → allow, simulada con JSON | Funciona: niega, marca, permite |
| Los 15 skills reales, uno por uno | 9 piden lectura real (`adversary`, `architect`, `auditor`, `bridge-engineer`, `context-engineer`, `general-counsel`, `refactoring-engineer`, `social-strategist`, `tech-lead`), 6 permiten directo |
| `rust-engineer` con la ruta placeholder `docs/execution/<ID>.md` | Antes del fix bloqueaba para siempre; después, excluida correctamente |
| `validate-agent-docs.py --audit` | 0 violaciones duras |

### Decisiones y por qué

- **Parsear la tabla de Lecturas en vivo, no un manifiesto aparte:** evita que el gate quede desincronizado si un skill cambia sus lecturas SIEMPRE — la fuente de verdad es el propio `SKILL.md`.
- **Fallar abierto (permitir) si no puede resolver el skill:** un gate roto no debe tumbar la ejecución normal de todo el sistema de skills.

### Pendientes y bloqueos

- **Disparador por palabra clave (bajo demanda) sigue sin mecanismo determinista** — depende de `UserPromptSubmit` + FTS5 (`inject-context.py`, Sección B1 del plan grande), que no se ha construido.

---

## Turno 9 — 02:48 — Reporte de sesión + cierre del plan

### Qué pediste

Registrar la sesión de hoy en `.agents/sessions/` con la plantilla, y agregar lo hecho hoy al plan que ya se estaba trabajando, creando la sección si no existía.

### Qué hice

Escribí este reporte completo. En el plan, la sección "Estado de ejecución real" ya existía (del turno 5) — le agregué tres bullets nuevos con lo de hoy: la poda del Pipeline (turno 6), y el par de hooks de determinismo de Lecturas (turnos 7 y 8), dejando explícito que el gate de skill-específicas es un adelanto real de lo que la Sección G del plan grande preveía para más adelante.

### Archivos tocados

| Archivo | Qué cambió | Por qué |
|---|---|---|
| `.agents/sessions/2026-09-04-aligeramiento-templates-y-hooks-de-lecturas.md` *(nuevo)* | Este reporte, 9 turnos | Pedido explícito |
| `.agents/plans/haz-el-plan-de-agile-sketch.md` | 3 bullets nuevos en "Estado de ejecución real" | Registrar el trabajo de hoy en el plan vigente |

---

## Turno 10 — 2026-09-05 — Explicación de los 3 hooks de determinismo

### Qué pediste

Antes de seguir con el plan: cómo funcionan los tres hooks construidos en la sesión anterior (qué esperar, cómo usarlos).

### Qué hice

Expliqué cada uno: `session-start-siempre.py` (`SessionStart`, se dispara solo en cada arranque/resume/compact, inyecta `governance.md` + `hard-prohibitions.md` por stdout, sin que nadie lo pida); `gate-skill-siempre.py` (`PreToolUse` sobre `Skill`, deniega la invocación si faltan lecturas `SIEMPRE` propias del skill, con el mensaje exacto de qué leer); `track-siempre-reads.py` (`PostToolUse` sobre `Read`, deja la marca que el gate consulta). Sin archivos tocados, solo explicación.

---

## Turno 11 — Sección B1: `inject-context.py` con SQLite FTS5

### Qué pediste

"Supongo que el paso natural es el siguiente: lecturas bajo demanda por palabra clave/disparador" — avanzar la Sección B1 del plan grande.

### Qué hice

Antes de escribir el hook, verifiqué el contrato de `UserPromptSubmit` contra la documentación oficial (no de memoria) — un subagente de apoyo (`claude-code-guide`) trajo un dato erróneo (mezcló los valores de `matcher` de `SessionStart` con este evento, que no admite `matcher`), lo descarté tras verificar directo con `WebFetch` contra `code.claude.com/docs/en/hooks`: confirmado sin `matcher`, stdin trae `prompt`, exit 2 bloquea y borra el prompt, ambas formas de inyección (stdout plano o `hookSpecificOutput.additionalContext`) funcionan.

Construí `.agents/hooks/inject-context.py`: indexa con SQLite FTS5 (`tokenize="unicode61 remove_diacritics 2"`, verificado disponible en el `sqlite3` de Python) el contenido completo de `.agents/knowledge/*.md` y `.agents/skills/*/SKILL.md`, reconstruye el índice si algún archivo fuente cambió más recientemente que el índice (`.agents/hooks/.fts-index.db`, no versionado), tokeniza el prompt filtrando stopwords, y rankea con `bm25()`. Primera versión: hasta 3 documentos completos si cabían en ~6.000 caracteres combinados, si no, manifiesto de wikilinks. Registrado en `.claude/settings.json`, sin `matcher`.

### Archivos tocados

| Archivo | Qué cambió | Por qué |
|---|---|---|
| `.agents/hooks/inject-context.py` *(nuevo)* | Hook `UserPromptSubmit`, FTS5 + bm25, primera versión | Sección B1 del plan grande |
| `.claude/settings.json` | Registrado bajo `UserPromptSubmit`, sin `matcher` | Activarlo |
| `.gitignore` | Ignorado `.agents/hooks/.fts-index.db` | Índice no se versiona |
| `.agents/templates/template-instruction-formats.md` | `TEMPLATE-INSTRUCTION-FORMATS-4` documenta el mecanismo ya resuelto | Cerraba la tercera categoría de determinismo pendiente |

### Comprobado

| Qué revisé | Resultado |
|---|---|
| FTS5 disponible en el `sqlite3` de Python | Sí, con `remove_diacritics 2` |
| Reconstrucción por `mtime` | No reconstruye si nada cambió; sí si se toca un archivo fuente |
| Prompt real contra el corpus real | Rankea y devuelve documentos coherentes con el tema |

### Decisiones y por qué

- **Verificar contra la doc oficial en vez de confiar en el subagente:** el dato que trajo (matcher de `SessionStart` aplicado a `UserPromptSubmit`) era observablemente sospechoso — los valores eran literalmente los de otro evento. Verificación directa lo confirmó falso.

---

## Turno 12 — Mecánica de FTS5/bm25 + intento de subir a 10.000 + conflicto con H5

### Qué pediste

Explicación de "sin matcher", cómo indexa FTS5, qué es `bm25`, si conviene subir de 6.000 a 10.000 caracteres, por qué no es semántico, y qué otra categoría de lectura determinista podría faltar.

### Qué hice

Expliqué los cuatro conceptos. Al ejecutar la subida a 10.000, encontré que `haz-el-plan-de-agile-sketch.md` (Sección H5) ya fijaba ese presupuesto en 6.000 **a propósito**, con una razón explícita del propio propietario ("lee bajo demanda, nunca en masa") — revertí el cambio de inmediato y pregunté antes de proceder, en vez de pisar una decisión ya registrada.

### Archivos tocados

Ninguno permanente — el cambio a `inject-context.py` se hizo y se revirtió en el mismo turno.

### Decisiones y por qué

- **Revertir antes de preguntar, no después:** encontrar un conflicto con una decisión ya documentada del propietario amerita pausar la ejecución, no solo señalarlo en el chat mientras el cambio queda aplicado.

---

## Turno 13 — 4 preguntas + embeddings vs. router LLM + decisión de diseño (H3)

### Qué pediste

Cuatro preguntas puntuales (¿se reconstruye el índice solo? ¿hibridizar con tags o quedarnos con uno? confirmación de subir a 10.000 — revocas tu propia decisión de H5 — ¿y si solo pasamos wikilinks y el agente decide?), más una pregunta interjectada a media ejecución sobre si embeddings/vectores baten a un router LLM en costo y latencia.

### Qué hice

Confirmé la reconstrucción automática (ya probada). Sobre hibridizar: el plan (`H3`) ya lo definía como híbrido — "extiende B1, no lo reemplaza" — antes de que preguntaras. Subí el presupuesto a 10.000 de verdad esta vez (confirmación explícita tuya de revocar H5) y corregí el texto de H5 en el plan para reflejar la reversión. Expliqué el trade-off de "solo wikilinks" (pierde la garantía de entrega, gana precisión) y el de embeddings (necesita modelo — local o de API, dependencia nueva — pero es mucho más barato/rápido que un router LLM completo, ya documentado como Fase 2 opcional en B3, sin medición real que lo justifique todavía).

Usé `AskUserQuestion` para dos decisiones reales: modo de entrega (elegiste **wikilinks siempre**) y si construir H3 ya (elegiste **sí**). Con eso, construí H3: probé en vivo el diseño original del plan (columna `tags` separada, ponderada vía `bm25(tabla, w0, w1)`) y lo descarté — verificado contra `sqlite.org/fts5.html`, la normalización de longitud de BM25 es compartida por **todo el corpus**, así que subir el peso de una columna movía el score de documentos sin ningún match ahí, y el resultado no era monótono (el margen a favor del documento etiquetado se encogía, no crecía, al subir el peso). Lo reemplacé por boost de frecuencia clásico en una sola columna — repetir las palabras del tag `TAG_REPEAT = 4` veces al inicio del texto indexado — probado monótono y predecible con K de 1 a 12. Reescribí `inject-context.py` a wikilinks-siempre (quité `MAX_COMBINED_CHARS` y la rama de contenido completo) y creé `.agents/knowledge/tags-vocabulary.md` (H2, 4 tags semilla: `ponytail`, `modos-acompanamiento`, `reporting`, `memoria`). Probado en fixture aislado (no el corpus real, para no forzar un tag donde no aplica) y luego contra el corpus real.

### Archivos tocados

| Archivo | Qué cambió | Por qué |
|---|---|---|
| `.agents/hooks/inject-context.py` | Reescrito: wikilinks siempre, boost de tags por repetición | Decisión de diseño + H3 |
| `.agents/knowledge/tags-vocabulary.md` *(nuevo)* | Vocabulario semilla, 4 tags | H2 del plan |
| `.agents/templates/template-knowledge.md` | Nota de H1 actualizada (vocabulario ya existe) | Reflejar H2 hecho |
| `.agents/plans/haz-el-plan-de-agile-sketch.md` | H1, H3, H5 reescritos con el diseño real y su historial | Documentar la corrección de diseño y la reversión de presupuesto |

### Comprobado

| Qué revisé | Resultado |
|---|---|
| Peso multi-columna en SQLite real (4 y 20 documentos sintéticos) | No monótono, confirmado con números |
| Boost por repetición, K de 1 a 12 | Monótono y predecible |
| Fixture con `Tags: ponytail` real vs. mención de pasada | El documento con tag rankeó primero |
| `--audit` sobre todo `.agents/` | 0 violaciones duras |

### Decisiones y por qué

- **Descartar el diseño original de H3 tras evidencia empírica, no de memoria:** el plan mismo decía "peso exacto a calibrar en implementación, no se fija aquí sin haberlo probado" — probarlo reveló que el enfoque entero (no solo el número) no era viable, y el plan se corrigió en consecuencia.

---

## Turno 14 — Pinecone/pgvector y análisis de indexado por Sección

### Qué pediste

Si Pinecone o pgvector resuelven la necesidad de un modelo de embeddings, y análisis de una propuesta: llevar el mismo indexado a nivel de Sección con FTS5 y tags por Sección — ¿requeriría partir archivos?, ¿qué tal la precisión?

### Qué hice

Aclaré que ninguno de los dos genera embeddings — son motores de almacenamiento/búsqueda de vectores, el modelo que produce el vector sigue siendo obligatorio con cualquiera de los dos, y a este tamaño de corpus (~50-200 documentos) ninguno aporta nada sobre SQLite (además, Pinecone en la nube choca con el invariante Local-First del proyecto). Analicé la propuesta de indexado por Sección: viable sin partir archivos físicamente (el split se hace en memoria al indexar, no en disco), mejora la precisión de forma directa, con el límite de que no habilita lectura parcial (`Read` sigue abriendo el archivo completo) — solo mejora la señal de ranking. Sin archivos tocados, solo análisis.

---

## Turno 15 — Indexado por Sección + Tags por Sección construidos

### Qué pediste

"Quiero también los tags por sección, aun no hemos migrado mucho, es algo que podemos absorber desde ya."

### Qué hice

Construí ambas piezas. `inject-context.py`: nueva función `split_sections()` parte cada documento en formato ligero por sus encabezados `## <DOC>-N: `, insertando una fila por Sección (más una "cabecera" con todo lo previo a la primera Sección) en vez de una fila por archivo — un documento legado, sin ese patrón, cae completo en un solo segmento, sin cambiar su comportamiento previo. `Tags:` ahora también es válido dentro de una Sección (`TEMPLATE-INSTRUCTION-FORMATS-5`) — los de la cabecera se heredan a toda Sección, los propios de una se agregan encima. El manifiesto ahora cita la Sección exacta que matcheó.

Encontré y corregí un bug real antes de darlo por bueno: la primera regex de partición (`[^:\n]+` antes de `: `) hacía falso-positivo con encabezados legados que casualmente traen un `:` en el subtítulo (ej. `ORDER-COMPLIANCE-FILTER-1 — Postura del Filtro: Contraste Bidireccional`). Corregido exigiendo un ID contiguo sin espacio ni guion largo antes del `: `.

### Archivos tocados

| Archivo | Qué cambió | Por qué |
|---|---|---|
| `.agents/hooks/inject-context.py` | `split_sections()`, indexado por Sección, esquema con columna `section` | Pedido explícito, absorbido temprano en la migración |
| `.agents/templates/template-instruction-formats.md` | `TEMPLATE-INSTRUCTION-FORMATS-5` permite `Tags:` dentro de una Sección | Formalizar el campo a nivel de bloque |
| `.agents/plans/haz-el-plan-de-agile-sketch.md` | H1/H3 actualizados con el indexado por Sección y el bug encontrado | Registrar la extensión y su verificación |

### Comprobado

| Qué revisé | Resultado |
|---|---|
| `order-compliance-filter.md` (legado, `:` en subtítulo) | Antes del fix: 1 falso split. Después: 1 segmento completo, correcto |
| `context-engineer/SKILL.md` (ligero) | 7 segmentos: cabecera + P + 5 Secciones, correcto |
| Fixture con Sección tageada vs. mención de pasada en otra Sección del mismo archivo | La Sección con `Tags:` real rankeó primero |
| Prompt real contra corpus real ("citas huérfanas") | Apunta a `context-engineer/SKILL.md — sección CONTEXT-ENGINEER-6`, la sección correcta |

### Decisiones y por qué

- **No partir los archivos físicamente:** contradiría la filosofía del formato ligero ("se lee completo") — el split vive solo en el indexado, en memoria.

---

## Turno 16 — Verificación: Pipeline y Ejemplos ya funcionaban

### Qué pediste

Qué pasa con el resto de bloques válidos de un documento — Secciones de Ejemplos, Pipelines — con este mecanismo de partición.

### Qué hice

Verifiqué contra el corpus real: `CONTEXT-ENGINEER-P` (Pipeline) y `WRITE-LIKE-GEMINI-EJ` (Ejemplos) ya quedan como su propio segmento, sin código adicional — el separador no distingue tipo de bloque, solo el patrón `## <ID>: `. Aclaré un límite conocido: los casos individuales dentro de Ejemplos (`<DOC>-EJ-K`) no son encabezados `##` propios, quedan agrupados en un solo segmento junto con sus vecinos. Sin archivos tocados, solo verificación.

---

## Turno 17 — Presupuesto de inyección obsoleto + switch `FORCE_FULL_CONTENT`

### Qué pediste

Si con la granularidad por Sección seguíamos alimentando el archivo entero o la Sección, dado el presupuesto de 10.000 caracteres.

### Qué hice

Encontré que la pregunta destapaba una inconsistencia real: el código ya no tiene ningún presupuesto de caracteres (se eliminó al pasar a "wikilinks siempre" en el Turno 13), pero el plan todavía mencionaba "~10.000 caracteres combinados" en 4 lugares como si siguiera vigente. Corregí las 4 menciones, dejando el historial completo en H5 (dos revocaciones el mismo día: 6.000→10.000, luego eliminado del todo).

Con `AskUserQuestion` propuse reintroducir contenido completo para el match top si su Sección es chica, dado que ahora las Secciones miden 200-2.000 caracteres, no 9.000 — pediste algo más flexible: un switch de una sola propiedad que se pueda activar si en la práctica los wikilinks no se leen lo suficiente. Agregué `FORCE_FULL_CONTENT` (booleano, `False` por defecto) e hice que el esquema FTS5 guarde también el texto original sin boost (columna `raw`, para no inyectar el texto repetido de tags si se activa el switch), con un tope de seguridad `FORCE_MAX_CHARS = 12.000` incluso con el switch activo.

### Archivos tocados

| Archivo | Qué cambió | Por qué |
|---|---|---|
| `.agents/hooks/inject-context.py` | Switch `FORCE_FULL_CONTENT` + `FORCE_MAX_CHARS`, columna `raw` nueva | Palanca de troubleshooting pedida |
| `.agents/plans/haz-el-plan-de-agile-sketch.md` | 4 menciones obsoletas del presupuesto corregidas; H5 reescrito con el historial completo; nuevo bullet del switch | Citas huérfanas detectadas al tocar el tema, corregidas en el momento |

### Comprobado

| Qué revisé | Resultado |
|---|---|
| `grep` de "10.000"/"6.000"/"MAX_COMBINED" en el código | Sin resultados — confirma que el presupuesto ya no existe |
| Ambos modos del switch contra el corpus real | `False`: wikilinks. `True`: contenido real, sin el prefijo de boost de tags |
| `--audit` sobre todo `.agents/` | 0 violaciones duras |

### Decisiones y por qué

- **Corregir las 4 menciones de inmediato, no solo señalarlas:** exactamente el caso que cubre `CONTEXT-ENGINEER-6` — cita huérfana detectada al tocar el tema en el mismo turno, se corrige ahí mismo.

---

## Turno 18 — Reporte de sesión actualizado + orden de migración de skills

### Qué pediste

Guardar el trabajo de la sesión en el reporte, actualizar el plan, y proceder con la migración de los 14 skills restantes uno por uno, validando cada uno — empezando por los más importantes. Pregunta puntual: ¿primero `architect` o `tech-lead`?

### Qué hice

Reabrí este reporte (estaba marcado `⚪ cerrada` desde el Turno 9, pero la sesión siguió) y agregué los Turnos 10-17 con el trabajo real hecho desde entonces. Recomendé migrar `tech-lead` antes que `architect`: es el skill más usado en el día a día (gobierna todo el ciclo de vida de una Orden de Trabajo), y el plan ya lo agrupa con los 12 skills de extracción a `skill-common.md` — mientras que `architect` está deliberadamente al final porque no comparte esa extracción (es una decisión estructural distinta, no de importancia). Verifiqué que `.agents/knowledge/skill-common.md` (prerequisito de Sección E, paso 4) todavía no existe — es el primer paso técnico obligatorio antes de tocar cualquiera de los 12 skills del lote, `tech-lead` incluido.

### Archivos tocados

| Archivo | Qué cambió | Por qué |
|---|---|---|
| `.agents/sessions/2026-09-04-aligeramiento-templates-y-hooks-de-lecturas.md` | Turnos 10-18 agregados, cabecera reabierta a `🟢 en curso` | Pedido explícito |

### Pendientes y bloqueos

- **`skill-common.md` no existe todavía** — bloquea el inicio de la migración de cualquiera de los 12 skills del lote (incluido `tech-lead`) hasta construirlo.

---

## Turno 19 — `skill-common.md` construido con ayuda de un subagente

### Qué pediste

Confirmaste el orden (`tech-lead` antes que `architect`, ver Turno 18) y diste dos decisiones sobre cómo consolidar los bloques compartidos: un solo bloque de Reporte al CEO extendido por Auditor/Adversary, y el principio general de que todo lo que no encaje en la plantilla se adapta o se pregunta, sin dejar excepciones sin justificar.

### Qué hice

Antes de escribir, delegué a un subagente (`general-purpose`) la lectura completa de los 12 `SKILL.md` del lote de extracción, para traer la versión más completa de cada uno de los bloques que el plan (D2) daba por repetidos — barrido de muchos archivos, exactamente el tipo de tarea que `CLAUDE.md` pide no hacer en la ventana principal. El subagente trajo dos correcciones reales al plan, no solo el texto pedido: el Protocolo de Lecciones ya vivía centralizado en `mentorship-modes.md` (no hacía falta consolidarlo de nuevo, solo corregir que 2 skills apuntaban por error a `governance.md`), y la fuente asumida para "Formato de Reporte al CEO" (`auditor`) era incorrecta — `AUDITOR-6` es un formato de informe de auditoría con veredictos, no un reporte de estado genérico; la plantilla real y reusable es `TECH-LEAD-3`.

Con esas dos correcciones confirmadas por ti vía `AskUserQuestion`, escribí `.agents/knowledge/skill-common.md`: 4 secciones (no 6) — Filtro de Arranque, Modos de Acompañamiento (esqueleto mecánico tomado de `rust-engineer`, el más completo de los 7 que lo tienen; cada skill conserva su ejemplo de dominio), Demostración Roja/Verde (de `rust-engineer`), Formato de Reporte al CEO (plantilla de `tech-lead`, con nota de que Auditor/Adversary la extienden con su propio esquema de veredicto). Corregí sobre la marcha un error propio: había incluido un quinto bloque, "Cierre del Filtro de Verificación", que contradecía una decisión ya tomada esta misma sesión (Turno 1) — el aparato `<DOC>-FV` completo ya está retirado del catálogo, sin reemplazo hasta que exista el juez `Stop`. Lo quité antes de que se propagara a ningún skill. Actualicé el plan (D2) con las 3 correcciones y con la decisión sobre el único hueco real que sigue en pie en `tech-lead` (sin Filtro de Arranque formal) — lo adopta al migrarse.

### Archivos tocados

| Archivo | Qué cambió | Por qué |
|---|---|---|
| `.agents/knowledge/skill-common.md` *(nuevo)* | 5 bloques comunes, prerrequisito de la Sección E paso 4 | Migrar los 12 skills sin duplicar contenido |
| `.agents/plans/haz-el-plan-de-agile-sketch.md` | D2 corregido (5 bloques no 6, fuente real del Bloque 6); "Estado de ejecución real" con el hallazgo del subagente y la decisión sobre `tech-lead` | Registrar la corrección real sobre lo que el plan asumía |

### Comprobado

| Qué revisé | Resultado |
|---|---|
| `--audit` sobre `skill-common.md` | 0 violaciones duras, 1 aviso blando (FV retirado a propósito) |

### Decisiones y por qué

- **Delegar la lectura de 12 archivos a un subagente en vez de leerlos yo mismo:** exactamente el criterio de `CLAUDE.md` §3.4 — barrido de muchos archivos, conclusión aislada en su propia ventana de contexto.
- **Corregir el plan en vez de solo ejecutar lo que decía:** el plan asumía dos cosas que la investigación real refutó (fuente del Bloque 6, necesidad de un bloque nuevo para Lecciones) — ejecutar la versión original habría producido un documento con una fuente equivocada.

### Pendientes y bloqueos

- **`tech-lead` es el siguiente a migrar**, ya con `skill-common.md` disponible como prerrequisito.

---

## Turno 20 — `tech-lead/SKILL.md` migrado, primer skill del lote

### Qué pediste

Migrar `tech-lead` (orden ya confirmado en el Turno 18), con `skill-common.md` ya disponible.

### Qué hice

Antes de escribir, revisé si algo fuera de `tech-lead/SKILL.md` cita uno de sus sub-IDs (`grep -rln`) — 1 archivo (`analiza-los-cambios-en-curried-oasis.md`, cita `TECH-LEAD-7.1`), fuera del alcance actual, registrado como `CONTEXT-DEBT-012`.

Reescribí el archivo completo: cabecera a bullets (`SLUG-ID`, `Tamaño`, `Lecturas` con flechas — incluye `skill-common.md` como nueva fila `SIEMPRE`), `TECH-LEAD-0` (tabla de Lecturas) y `TECH-LEAD-FV` retirados, headings de Sección con `:`, sub-IDs colapsados a bullets planos. `TECH-LEAD-3` (Formato de Reporte al CEO, la fuente original del bloque) pasó a referencia a `skill-common.md`. Fusioné `TECH-LEAD-2.5` (Presentación Obligatoria) dentro del nuevo bullet de Filtro de Arranque en `TECH-LEAD-2`, en vez de dejarlo como bullet aparte redundante. Dejé `TECH-LEAD-5` (Modo de Acompañamiento de Implementación) íntegro, sin referenciar el bloque común de Modos — el subagente del turno anterior ya había confirmado que es contenido genuinamente distinto (cómo el Tech-Lead despacha a otros, no cómo él opera bajo un Modo) — y agregué una nota explícita en el propio bloque para que quede claro por qué no se colapsó, previniendo que un futuro paso lo intente fusionar por error.

### Archivos tocados

| Archivo | Qué cambió | Por qué |
|---|---|---|
| `.agents/skills/tech-lead/SKILL.md` | Reescrito completo a formato ligero: 18.431 → 14.148 caracteres (23% menos) | Primer skill del lote de 12, orden confirmado |
| `.agents/debt/CONTEXT-DEBT-012.md` *(nuevo)* | Cita huérfana a `TECH-LEAD-7.1` registrada | Fuera de alcance actual, regla de `CONTEXT-ENGINEER-6` |
| `.agents/CONTEXT-DEBT.md` | Fila nueva para `CONTEXT-DEBT-012` | Registro en el índice |
| `.agents/plans/haz-el-plan-de-agile-sketch.md` | Nuevo bullet en "Estado de ejecución real" con el detalle de la migración | Dejar registrado qué cambió y por qué |

### Comprobado

| Qué revisé | Resultado |
|---|---|
| `grep` de citas externas a sub-IDs de `TECH-LEAD` | 1 resultado, registrado como deuda |
| `--audit` sobre `tech-lead/SKILL.md` | 0 violaciones duras, 1 aviso blando (FV retirado a propósito) |
| `--audit` sobre todo `.agents/` | 50 archivos, 0 violaciones duras |

### Pendientes y bloqueos

- **Esperando validación del propietario sobre `tech-lead/SKILL.md`** antes de continuar con el siguiente skill del lote.

---

## Cierre de sesión

**Estado:** sesión sigue abierta — el propietario pidió continuar con la migración de los 14 skills restantes, validando uno por uno.

**Resuelto hoy:** las tres categorías de determinismo de "Lecturas" quedaron cerradas (universal, por skill, por palabra clave vía `inject-context.py`); el mecanismo de recuperación por FTS5 evolucionó de indexado por archivo a indexado por Sección con tags heredables, con un switch de troubleshooting (`FORCE_FULL_CONTENT`) para forzar contenido completo si hiciera falta.

**Sin resolver:**
- Migración de los 13 skills restantes al formato ligero (`architect`, `social-strategist` y los 11 que faltan del lote de extracción — `tech-lead` ya migrado).
- `CONTEXT-DEBT-010` (7 citas huérfanas de `context-engineer`) y `CONTEXT-DEBT-012` (1 cita huérfana de `tech-lead`) siguen abiertas.
- Remapeo de las ~1123 citas a sub-IDs retirados por el aligeramiento.
- Sección H4/H6/H7 del plan (índice de `docs/`, validadores de tags, aplicación progresiva sobre el corpus real).

**Siguiente paso:** el propietario valida `tech-lead/SKILL.md`; con luz verde, sigue el siguiente skill del lote de 12.
