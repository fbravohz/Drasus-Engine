# Prohibiciones Duras Globales

Prohibiciones absolutas que aplican a todo rol, todo documento y todo turno, sin excepción de dominio. Par de `governance.md` — no un bloque suyo, otro documento con la misma prioridad `SIEMPRE`.

Slug ID: `HARD-PROHIBITIONS`.

Tamaño: `5698 caracteres (~1424 tokens aprox., ratio 4:1)`.

---

## HARD-PROHIBITIONS-0 — Lecturas Obligatorias

| Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
|---|---|---|---|
| [[.agents/knowledge/governance.md]] | Gobernanza meta, supremacía absoluta. | `SIEMPRE` al iniciar cualquier skill. | Todos los agentes |

---

## HARD-PROHIBITIONS-FV — Filtro de Verificación

**Aplica a:** todos los roles, como último paso antes de enviar cualquier respuesta o edición.

* **HARD-PROHIBITIONS-FV-1:** ¿alguna regla nueva que escribí lleva fecha, nombre de sesión o caso histórico en vez de vivir en `PROGRESS.md`/git (`HARD-PROHIBITIONS-1`)?
* **HARD-PROHIBITIONS-FV-2:** ¿algún nombre que propuse cae en el campo semántico de la muerte (`HARD-PROHIBITIONS-2`)?
* **HARD-PROHIBITIONS-FV-3:** ¿referencié una sección de documento por su numeral `§N` en vez del criterio vigente (`HARD-PROHIBITIONS-3`)?
* **HARD-PROHIBITIONS-FV-4:** ¿usé "Gate"/"gate" para nombrar un proceso o flujo de validación en vez de "Filtro" (`HARD-PROHIBITIONS-4`)?

De ser satisfactoria cada validación de la lista, declara `HARD-PROHIBITIONS - FILTRO DE VERIFICACION APROBADO`.

---

## HARD-PROHIBITIONS-1 — Sin Lenguaje de Bitácora Dentro de una Regla

Ninguna fecha, referencia a una versión previa del texto, nombre de sesión ni caso histórico dentro de una regla normativa — esa historia vive en git o en `PROGRESS.md`, nunca en el documento que rige el comportamiento futuro. Preserva la regla; corta la anécdota. Cortar la regla o el dato técnico que sostiene la regla para cumplir esto es el error inverso — la purga quita la fecha/sesión, nunca el contenido normativo (`CONTEXT-ENGINEER-3.2`).

**Aplica a:** todos los roles, al escribir o editar cualquier regla de `.agents/knowledge/` o `.agents/skills/`.

* ❌ *Incorrecto:* "regla del usuario, RECTORA" con fecha, o "(caso STORY-XXX, con fecha de sesión)" dentro del cuerpo de una regla.
* ✅ *Correcto:* la regla enunciada sin fecha ni sesión; el caso que la motivó, si vale la pena preservarlo, vive en `PROGRESS.md` o en la Orden de Trabajo que lo originó.

---

## HARD-PROHIBITIONS-2 — Prohibición de Lenguaje Mortuorio

Estrictamente prohibido todo término del campo semántico de la muerte y sus estados (muerte, morir, difunto, cadáver, tumba, entierro, cementerio, autopsia, forense, `post-mortem`, necro-, luto, etc.) en nombres de features, módulos, nodos de canvas, ADRs, plantillas o cualquier documento — aunque "suene temático" o técnico. No es una excepción aplazable a lista de ejemplos: aplica al campo semántico completo, no a una lista corta de palabras prohibidas.

**Aplica a:** todos los roles, en todo nombramiento o redacción.

* **HARD-PROHIBITIONS-2.1 — Saneamiento Inmediato, No Deuda:**

  al detectar una de estas palabras, se reemplaza en el momento — no se abre un `DEBT-XXX` ni se difiere. Una violación de nombres no es deuda aplazable, es saneamiento proactivo.

* **HARD-PROHIBITIONS-2.2 — Reemplazos Fijados:**

  `Forense` / `Post-mortem` / `Simulación Forense` → `Repetición de Corrida` / `Reproducción` / `Revisión de Corrida`.

  * ❌ *Incorrecto:* sanear solo "Autopsia"/"Cementerio" y dejar pasar "forense" o "post-mortem" por no estar en la lista corta.
  * ✅ *Correcto:* aplicar el campo semántico completo, no una lista de ejemplos.

**Lectura bajo demanda:** [[.agents/memory/prohibicion-lenguaje-mortuorio.md]] — caso de origen que motivó fijar esta regla como permanente.

---

## HARD-PROHIBITIONS-3 — Prohibido el Numeral `§N` Como Referencia a Sección

Prohibido citar una sección de cualquier documento por su numeral posicional (`§7`, `§9`, "sección 9") — un numeral es frágil ante reordenamientos y no es descubrible por búsqueda. El criterio vigente del proyecto es `SLUG-ID` (`<DOC>-N`) para reglas normativas.

**Aplica a:** todos los roles, al referenciar cualquier sección de cualquier documento.

* **HARD-PROHIBITIONS-3.1 — Criterio de Reemplazo Pendiente para Documentos-Formulario:**

  las Órdenes de Trabajo (`docs/execution/<ID>.md`) y su plantilla ([[docs/execution/_TEMPLATE.md]]) no siguen `SLUG-ID` — son documentos-formulario con secciones numeradas. El criterio de referencia que las reemplace (nombre estable en vez de número, o slug propio por sección) todavía no está definido.

  **Lectura bajo demanda:** [[.agents/CONTEXT-DEBT.md]] `CONTEXT-DEBT-009` — hueco abierto en `brief.md`, pendiente de este criterio.

---

## HARD-PROHIBITIONS-4 — Prohibido "Gate" Para Nombrar un Proceso o Flujo de Validación

El término correcto es **Filtro** (español) / **Filter** (en `SLUG-ID`, identificadores en inglés) — ya es el patrón canónico del proyecto: `Quality Gate` se traduce como `Filtro de Calidad` (`GOVERNANCE-3.4`) y `-FV` ya significa `Filtro de Verificación` en todo documento normativo.

**Aplica a:** todos los roles, al nombrar cualquier proceso, flujo de validación, archivo o identificador nuevo.

* **HARD-PROHIBITIONS-4.1 — No Retroactivo a Código ni a Documentos de Dominio Ajeno:**

  esta prohibición rige el dominio del Context Engineer (`.agents/knowledge/`, `.agents/skills/`, `.claude/skills/`) — ya renombrado. `docs/` (ADR, features, SAD, módulos, dominio del Architect) y `scripts/mutation-gate.sh` (código) quedan como deuda pendiente, no renombrados por esta regla.

  **Lectura bajo demanda:** [[.agents/CONTEXT-DEBT.md]] y [[docs/DEBT.md]] — fichas de la deuda de rename pendiente.
