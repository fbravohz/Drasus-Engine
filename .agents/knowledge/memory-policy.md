# Política de Memoria — Qué y Cómo Recordar

Consolida en un solo lugar todo lo que el proyecto sabe sobre memoria entre sesiones: la mecánica nativa que Claude Code inyecta en cada sesión (Capa Harness), la política propia de Drasus sobre qué se registra y por qué (Capa Drasus), el protocolo operativo del Tech-Lead, y la distinción entre `.agents/memory/` y `.agents/state/<agente>/` — todo en un solo lugar para poder editarlo, auditarlo y overridearlo.

> ⛔ **ESCRITURA SUSPENDIDA — lee `MEMORY-POLICY-14` antes que cualquier otra regla de este documento.** Escribir en `.agents/memory/` está prohibido. Todo lo que sigue describe una mecánica **inactiva**: se conserva documentada, no vigente.

Slug ID: `MEMORY-POLICY`.

Tamaño: `21229 caracteres (~5307 tokens aprox., ratio 4:1)`.

---

## MEMORY-POLICY-0 — Lecturas Obligatorias

| Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
|---|---|---|---|
| [[.agents/knowledge/governance.md]] | Gobernanza meta, supremacía absoluta. | `SIEMPRE` al iniciar cualquier skill. | Todos los agentes |
| [[.agents/skills/tech-lead/SKILL.md]] | Triggers operativos concretos de cuándo el Tech-Lead lee/escribe memoria. | Al operar como Tech-Lead. | Tech-Lead |
| [[.agents/memory/MEMORY.md]] | Índice real de memorias curadas — la Capa Drasus en acción. | Al escribir o leer memoria curada. | Todos los agentes |
| [[.agents/state/tech-lead/PROGRESS.md]] | Bitácora operativa cronológica del Tech-Lead. | Al operar como Tech-Lead, para retomar el hilo. | Tech-Lead |

---

## MEMORY-POLICY-FV — Filtro de Verificación

**Aplica a:** todos los agentes, antes de guardar, leer o actuar sobre memoria.

* **MEMORY-POLICY-FV-1:** si la mecánica nativa del harness choca con la política de Drasus o el protocolo del Tech-Lead, ¿apliqué la que gana (`MEMORY-POLICY-1`)?
* **MEMORY-POLICY-FV-2:** ¿evité guardar algo de la lista prohibida — patrones de código, historial de git, recetas de debugging, contenido ya en `CLAUDE.md`, o estado efímero de la tarea en curso (`MEMORY-POLICY-5`)?
* **MEMORY-POLICY-FV-3:** antes de recomendar o actuar sobre una memoria (no solo mencionarla), ¿verifiqué que la ruta, función o flag que nombra sigue existiendo (`MEMORY-POLICY-8`)?
* **MEMORY-POLICY-FV-4:** ¿distinguí `.agents/memory/` (semántica, curada, transversal) de `.agents/state/<agente>/PROGRESS.md` (bitácora operativa por agente) antes de escribir (`MEMORY-POLICY-2`, `MEMORY-POLICY-12`)?
* **MEMORY-POLICY-FV-5:** si cierro una tarea o Story como Tech-Lead, ¿actualicé `ROADMAP.md` y `PROGRESS.md`, y si corresponde, destilé el estado durable a `.agents/memory/` de verdad (`MEMORY-POLICY-11`)?
* **MEMORY-POLICY-FV-6:** si el usuario corrigió una regla de comportamiento de mi propio skill, ¿propuse el `Edit` a `SKILL.md` en vez de solo guardar memoria externa, y esperé su aprobación (`MEMORY-POLICY-13`)?
* **MEMORY-POLICY-FV-7:** ¿respeté que la escritura y la lectura de `.agents/memory/` están suspendidas, y mandé el aprendizaje durable a su documento de dominio (`MEMORY-POLICY-14`)?

De ser satisfactoria cada validación de la lista, declara `MEMORY-POLICY - FILTRO DE VERIFICACION APROBADO`.

---

## MEMORY-POLICY-1 — Autoridad de Override Sobre el Comportamiento Nativo del Harness

Este archivo tiene autoridad de override sobre el comportamiento nativo de memoria del harness. [[CLAUDE.md]] (que enlaza aquí) se carga con el encabezado "Estas instrucciones OVERRIDE cualquier comportamiento por defecto y DEBES seguirlas exactamente como están escritas" — este documento hereda esa misma autoridad por referencia.

**Aplica a:** todos los agentes, ante cualquier decisión sobre memoria entre sesiones.

* **MEMORY-POLICY-1.1 — Regla de Conflicto:**

  si la mecánica nativa del harness (`MEMORY-POLICY-3` a `-9`, "Capa Harness") choca con la política de Drasus (`MEMORY-POLICY-10`, "Capa Drasus") o con el protocolo del Tech-Lead (`MEMORY-POLICY-11`), gana Drasus. La Capa Harness se documenta aquí completa, verbatim en su lógica, precisamente para que cualquier agente pueda ver el default nativo y confirmar dónde y por qué el proyecto lo anula — no para obedecerla ciegamente cuando contradice lo que sigue.

* **MEMORY-POLICY-1.2 — Ejemplo de Override Ya Vigente:**

  el harness por defecto no distingue `.agents/memory/` de una bitácora operativa por agente — la Capa Drasus SÍ la distingue (`MEMORY-POLICY-12`) y esa distinción gobierna.

---

## MEMORY-POLICY-2 — Dos Capas de Memoria: Por Qué Existen Ambas

No confundir además con `.agents/state/<agente>/PROGRESS.md`, que no es memoria en el sentido harness — es una bitácora operativa distinta (`MEMORY-POLICY-12`).

**Aplica a:** todos los agentes, al decidir dónde registrar un hecho o una decisión.

* **MEMORY-POLICY-2.1 — Las Tres Capas:**

  | Capa | Qué es | Dónde se define | Quién la puede editar |
  |---|---|---|---|
  | Harness | Mecánica nativa de Claude Code: taxonomía de 4 tipos, formato de frontmatter, disciplina de guardado/lectura. | Inyectada en el prompt de sistema en cada sesión — no vive en ningún archivo del repo. | Nadie del proyecto; es infraestructura del producto Claude Code. |
  | Drasus | Política del proyecto sobre qué se registra, cuándo, y la disciplina de "curada, no automática". | `MEMORY-POLICY-10`. | El usuario/Architect vía edición quirúrgica. |
  | Protocolo Tech-Lead | Ritual operativo concreto de cuándo ese skill lee/escribe memoria. | `MEMORY-POLICY-11`, referenciado desde [[.agents/skills/tech-lead/SKILL.md]]. | Tech-Lead, en su Etapa 7 (mejora de skills). |

---

## MEMORY-POLICY-3 — Capa Harness: Qué Es la Memoria Nativa

Reproducción fiel de las instrucciones que el harness inyecta en el prompt de sistema al inicio de cada sesión (sección "auto memory"). Es infraestructura del producto, no de Drasus — se documenta aquí para que quede auditable y para anclar overrides contra ella (`MEMORY-POLICY-1`).

**Aplica a:** todos los agentes, en cualquier sesión.

* **MEMORY-POLICY-3.1 — Mecánica Base:**

  existe un sistema de memoria persistente, basado en archivos, en `.agents/memory/`. Se construye a lo largo del tiempo para que futuras conversaciones tengan una imagen completa de quién es el usuario, cómo colaborar, qué comportamientos evitar o repetir, y el contexto detrás del trabajo. Si el usuario pide explícitamente recordar algo, se guarda de inmediato en el tipo que mejor calce (`MEMORY-POLICY-4`). Si pide olvidar algo, se busca y elimina la entrada relevante.

---

## MEMORY-POLICY-4 — Capa Harness: Los Cuatro Tipos de Memoria

**Aplica a:** todos los agentes, al guardar un hecho en `.agents/memory/`.

* **MEMORY-POLICY-4.1 — Taxonomía:**

  | Tipo | Contiene | Cuándo guardar | Ejemplo |
  |---|---|---|---|
  | `user` | Rol, objetivos, responsabilidades y conocimiento del usuario — para adaptar el trabajo a su perfil. | Al aprender cualquier detalle sobre rol/preferencias/responsabilidades/conocimiento del usuario. | "Es data scientist, enfocado en observabilidad". |
  | `feedback` | Guía sobre CÓMO abordar el trabajo — correcciones ("no hagas X") y confirmaciones de un enfoque no obvio que funcionó. | Cualquier corrección explícita, o cualquier confirmación silenciosa de un enfoque poco obvio ("sí, exacto", aceptar una elección sin objeción). | "No mockees la base de datos en tests de integración — un mock desalineado enmascaró una migración rota". |
  | `project` | Estado de trabajo en curso, objetivos, iniciativas, bugs o incidentes NO derivables del código/git. | Al aprender quién hace qué, por qué, o para cuándo. Fechas relativas → fechas absolutas. | "Congelamiento de merges desde el 2026-03-05 — release de mobile". |
  | `reference` | Punteros a sistemas externos donde vive información actualizada. | Al aprender de un recurso externo y su propósito. | "Bugs de pipeline se rastrean en Linear, proyecto INGEST". |

* **MEMORY-POLICY-4.2 — Estructura Obligatoria del Cuerpo (Tipos feedback/project):**

  regla o hecho primero, luego línea `**Why:**` (la razón, a menudo un incidente pasado) y línea `**How to apply:**` (cuándo aplica). El "por qué" permite juzgar casos borde sin repreguntar.

**Filtro de verificación:** `MEMORY-POLICY-FV-4`.

---

## MEMORY-POLICY-5 — Capa Harness: Qué Nunca Guardar

**Aplica a:** todos los agentes, al evaluar si un hecho merece guardarse en `.agents/memory/`.

* **MEMORY-POLICY-5.1 — Lista de Exclusión:**

  patrones de código, convenciones, arquitectura, rutas o estructura del proyecto (derivables leyendo el estado actual); historial de git o quién-cambió-qué (`git log`/`git blame` son la fuente de verdad); soluciones de debugging o recetas de arreglos (el fix vive en el código; el mensaje de commit tiene el contexto); cualquier cosa ya documentada en archivos `CLAUDE.md`; detalles efímeros de la tarea en curso (trabajo a medias, estado temporal, contexto solo de esta conversación).

* **MEMORY-POLICY-5.2 — Excepción No Aplica ni Bajo Pedido Explícito:**

  estas exclusiones aplican incluso si el usuario pide explícitamente guardarlas. Si pide guardar algo como un listado de PRs o un resumen de actividad, se pregunta primero qué fue sorprendente o no obvio — eso es lo único que vale la pena conservar.

**Filtro de verificación:** `MEMORY-POLICY-FV-2`.

---

## MEMORY-POLICY-6 — Capa Harness: Cómo Guardar (Dos Pasos)

**Aplica a:** todos los agentes, al escribir una memoria nueva.

* **MEMORY-POLICY-6.1 — Paso 1: Escribir el Archivo:**

  con `Write`, nombre propio en `kebab-case` (ej. `feedback_testing.md`) y frontmatter. Enlazar memorias relacionadas con `[[nombre-slug]]` — un enlace a algo que aún no existe está bien, marca algo pendiente de escribir, no es error.

  ```markdown
  ---
  name: {{slug-corto}}
  description: {{resumen de una línea — decide relevancia en futuras conversaciones}}
  metadata:
    type: {{user, feedback, project, reference}}
  ---

  {{contenido}}
  ```

* **MEMORY-POLICY-6.2 — Paso 2: Añadir un Puntero en MEMORY.md:**

  [[.agents/memory/MEMORY.md]] es un ÍNDICE, no una memoria en sí: cada entrada, una línea, bajo ~150 caracteres — `- [Título](archivo.md) — gancho de una línea`, sin frontmatter. Líneas después de la 200 se truncan; mantenerlo conciso.

* **MEMORY-POLICY-6.3 — Verificar Antes de Guardar:**

  confirmar que no exista ya una memoria equivalente — nunca duplicar.

**Lectura bajo demanda:** [[.agents/memory/MEMORY.md]] — siempre que vayas a guardar una memoria nueva, para verificar duplicados y añadir el puntero.

---

## MEMORY-POLICY-7 — Capa Harness: Cuándo Acceder a la Memoria

**Aplica a:** todos los agentes, al inicio o durante cualquier sesión.

* **MEMORY-POLICY-7.1 — Disparadores de Lectura:**

  cuando las memorias parezcan relevantes, o el usuario referencie trabajo de una conversación previa. Obligatorio cuando el usuario pide explícitamente recordar/verificar/consultar memoria.

* **MEMORY-POLICY-7.2 — Si el Usuario Pide Ignorar Memoria:**

  no aplicar, citar, comparar ni mencionar su contenido.

* **MEMORY-POLICY-7.3 — Las Memorias Envejecen:**

  antes de actuar sobre un recuerdo (no solo mencionarlo), verificar que sigue siendo cierto contra el estado actual del código/archivos. Si contradice lo observado ahora, gana lo observado — y se actualiza o borra la memoria vieja.

---

## MEMORY-POLICY-8 — Capa Harness: Antes de Recomendar Desde Memoria

Una memoria que nombra una función, archivo o flag es una afirmación de que existía cuando se escribió. Puede haber sido renombrada, eliminada o nunca fusionada.

**Aplica a:** todos los agentes, antes de recomendar o actuar sobre una memoria.

* **MEMORY-POLICY-8.1 — Verificación Obligatoria:**

  si nombra una ruta, verificar que el archivo existe. Si nombra una función/flag, `grep` para confirmarla. Si el usuario va a actuar sobre la recomendación (no solo preguntar por historial), verificar primero.
  * ❌ *Incorrecto:* tratar "la memoria dice que X existe" como lo mismo que "X existe ahora".
  * ✅ *Correcto:* verificar antes de recomendar; para preguntas sobre estado reciente o actual, preferir `git log` o leer el código antes que recordar el snapshot congelado.

**Filtro de verificación:** `MEMORY-POLICY-FV-3`.

---

## MEMORY-POLICY-9 — Capa Harness: Memoria vs. Otras Formas de Persistencia

**Aplica a:** todos los agentes, al decidir si algo va a memoria, a un Plan o a Tasks.

* **MEMORY-POLICY-9.1 — Plan (No Es Memoria):**

  cuando se va a iniciar una implementación no trivial y se necesita alinear el enfoque con el usuario ANTES de ejecutar.

* **MEMORY-POLICY-9.2 — Tasks (No Es Memoria):**

  cuando se necesita trocear el trabajo de la conversación actual en pasos discretos o rastrear progreso dentro de ella.

* **MEMORY-POLICY-9.3 — Memoria:**

  reservada para lo que debe sobrevivir a esta conversación y ser útil en futuras — no para estado efímero de la tarea en curso.

---

## MEMORY-POLICY-10 — Capa Drasus: Política del Proyecto sobre Memoria

[[CLAUDE.md]] mantiene solo un puntero de una línea a esta sección. Existe memoria nativa de proyecto en `.agents/memory/` (índice `MEMORY.md` + un hecho por archivo). Se carga cada sesión: por eso un agente "recuerda" decisiones pasadas sin que se las repitan.

**Aplica a:** todos los agentes, al decidir qué registrar como memoria de proyecto.

* **MEMORY-POLICY-10.1 — Curada, No Automática:**

  se escriben hechos durables a propósito (decisiones, restricciones, estado de trabajo en curso), no transcripciones completas.

* **MEMORY-POLICY-10.2 — Disciplina Obligatoria:**

  al cerrar trabajo significativo, destila la decisión o el estado a un archivo de memoria y enlázalo desde `MEMORY.md`. No dupliques lo que ya registra el código, el git o estos documentos.

* **MEMORY-POLICY-10.3 — Recuerdo Semántico Difuso (Futuro):**

  capturar y buscar conversaciones por significado es una construcción aparte (servidor MCP o CLI local + embeddings), no un ajuste de configuración. Se diseña cuando la memoria curada se quede corta, no antes.

---

## MEMORY-POLICY-11 — Protocolo del Tech-Lead: Handoff Entre Sesiones

[[.agents/skills/tech-lead/SKILL.md]] mantiene solo los triggers operativos, con puntero aquí para el detalle y el porqué. Propósito: que un futuro Tech-Lead (otra sesión, contexto fresco) sepa exactamente dónde quedó todo sin re-derivarlo.

**Aplica a:** Tech-Lead, al arrancar o cerrar cualquier sesión de trabajo.

* **MEMORY-POLICY-11.1 — Dos Lugares, Ninguno Es memory/:**

  la memoria viva del Tech-Lead son DOS lugares, ambos versionados en el repo, ninguno de los dos `.agents/memory/` — son parte de la Capa State (`MEMORY-POLICY-12`): [[docs/ROADMAP.md]] (fuente de verdad de estado: tabla "Registro de Estado" de la fase activa + bitácora "Descubrimientos y decisiones"; se actualiza al cerrar cada tarea/TTR) y [[.agents/state/tech-lead/PROGRESS.md]] (bitácora operativa cronológica: qué se despachó, a qué ingeniero, en qué modelo, qué se auditó, qué se decidió/escaló, y cuál es el SIGUIENTE paso concreto).

* **MEMORY-POLICY-11.2 — Al Arrancar una Sesión:**

  además de leer `docs/`, se lee [[.agents/state/tech-lead/PROGRESS.md]] y el "Registro de Estado" del ROADMAP de la fase activa — es la memoria de reanudación: se retoma desde el "siguiente paso" anotado, no desde cero.

* **MEMORY-POLICY-11.3 — Al Cerrar Cada Tarea/TTR:**

  (o al escalar/decidir algo relevante) se actualizan AMBOS: el estado en el ROADMAP y una entrada nueva (con fecha) en `PROGRESS.md`. Entrada = qué se hizo, evidencia de auditoría, decisión tomada, y el siguiente paso.

* **MEMORY-POLICY-11.4 — Sin Handoff Escrito, el Trabajo No Está Cerrado:**

  si una sesión termina con trabajo a medias, lo último que se hace es dejar el "siguiente paso" escrito en `PROGRESS.md`.

* **MEMORY-POLICY-11.5 — Barrido de Cierre Documental:**

  en el checklist obligatorio al cerrar cada Story, el Tech-Lead escribe a `.agents/memory/` de verdad (no solo a `PROGRESS.md`): destila el estado/decisión durable que trascienda la iteración, y actualiza contadores de progreso transversales (ej. "substrato N/10"). En su Etapa 7 (Retroalimentación), si una lección trasciende la Story actual, la destila a `.agents/memory/` — no solo la deja en el `SKILL.md` corregido.

**Filtro de verificación:** `MEMORY-POLICY-FV-5`.

---

## MEMORY-POLICY-12 — `.agents/memory/` vs. `.agents/state/<agente>/`: Por Qué Están Separados

No es arbitrario que sean dos sistemas distintos, ni que `state/` no viva anidado bajo `memory/<agente>/`.

**Aplica a:** todos los agentes, al decidir dónde escribir un hecho o una entrada de bitácora.

* **MEMORY-POLICY-12.1 — Comparativa:**

  | | `.agents/memory/` | `.agents/state/<agente>/PROGRESS.md` |
  |---|---|---|
  | Naturaleza | Memoria semántica curada y transversal — un hecho por archivo, deduplicado, enlazado. | Bitácora operativa cronológica por agente — log con fecha, no deduplicado. |
  | Quién la lee | TODOS los skills, como contexto de proyecto compartido. | SOLO el agente dueño, como parte de su propio ritual de arranque. |
  | Cómo se escribe | Baja frecuencia, alta densidad — al CERRAR trabajo significativo. | Alta frecuencia — al cerrar CADA tarea/TTR. |
  | Disciplina | "No dupliques lo que ya registra el código, el git o estos documentos". | Duplica narrativa deliberadamente (qué se despachó, qué pasó, cuándo) — es justo su función. |

* **MEMORY-POLICY-12.2 — Tres Razones para la Separación:**

  distinta cardinalidad de lectores (memoria es un pool compartido que todos los skills cargan como contexto de proyecto; state es una bitácora privada que solo el dueño del rol relee para retomar su propio hilo — anidarlo bajo `memory/` sugeriría falsamente que es material para que otros skills lo consuman); distinta cadencia y forma (memoria es curada y semántica; `PROGRESS.md` es un log cronológico que SÍ duplica narrativa por diseño, justo lo que la disciplina de memoria — `MEMORY-POLICY-5`/`-10` — prohíbe; si viviera en `memory/`, un lector esperaría la disciplina de "un hecho por archivo, sin duplicar", y `PROGRESS.md` la violaría por construcción); la partición por agente ya existe donde importa (`state/tech-lead/`, `state/social-strategist/` — ponerla también dentro de `memory/` crearía dos jerarquías paralelas para el mismo propósito). En corto: `memory/` responde "¿qué sabe el proyecto, para siempre?"; `state/<agente>/PROGRESS.md` responde "¿en qué iba yo, la última vez que trabajé?" — preguntas distintas, audiencias distintas.

---

## MEMORY-POLICY-13 — Auto-Edición de Skill ante Corrección

Distinto de guardar una memoria: cuando el usuario corrige tu enfoque de una forma que revela una regla de comportamiento de tu propio skill — no un hecho puntual del proyecto — propón un `Edit` quirúrgico a tu propio `SKILL.md` incorporando la corrección.

**Aplica a:** todos los agentes, cuando una corrección del usuario describe una regla de comportamiento del propio rol, no un hecho del proyecto.

* **MEMORY-POLICY-13.1 — Nunca Sin Aprobación:**

  nunca apliques el `Edit` sin que el usuario lo apruebe primero en el turno actual. La memoria (`MEMORY-POLICY-3` a `-10`) registra hechos y decisiones del proyecto; esto edita la regla de comportamiento que regirá la próxima vez que el skill se invoque. Son complementarios, no sustitutos.
  * ❌ *Incorrecto:* guardar la corrección solo como memoria externa cuando en realidad describe una regla de comportamiento del rol.
  * ✅ *Correcto:* proponer el `Edit` al `SKILL.md` con el texto exacto del cambio, y esperar aprobación antes de aplicarlo.

**Filtro de verificación:** `MEMORY-POLICY-FV-6`.

---

## MEMORY-POLICY-14 — Escritura de Memoria Suspendida

La memoria automática está apagada (`autoMemoryEnabled: false`). Todo lo que este documento describe de `MEMORY-POLICY-2` a `-13` queda documentado pero **inactivo**: se conserva para poder reactivarlo con criterio, no para ejecutarlo hoy.

**Aplica a:** todos los agentes, en toda decisión sobre dónde registrar un aprendizaje.

* **MEMORY-POLICY-14.1 — Prohibido Escribir en `.agents/memory/`:**

  ninguna entrada nueva, por ninguna razón, ni siquiera si el usuario dice "recuérdalo". Los archivos ya existentes se dejan intactos: el propietario los depura en una tarea aparte.
  * ❌ *Incorrecto:* crear `.agents/memory/<tema>.md` porque el usuario pidió recordar algo.
  * ✅ *Correcto:* escribir esa regla en el skill, el knowledge doc o el ADR que la consume, y citar ahí dónde quedó.

* **MEMORY-POLICY-14.2 — Destino Real de un Aprendizaje Durable:**

  el documento de dominio que lo consume, elegido por quién necesita la regla:
  - Regla de comportamiento de un rol → su `SKILL.md` (`MEMORY-POLICY-13`).
  - Regla transversal a varios roles → un documento de `.agents/knowledge/`.
  - Decisión de arquitectura o de producto → un ADR en `docs/adr/`.

* **MEMORY-POLICY-14.3 — Lectura También Suspendida:**

  no se lee `.agents/memory/` para fundamentar una decisión. Un contenido que siga siendo válido se migra primero a su documento de dominio y se cita desde ahí.

**Precedencia:** gana sobre todo este documento, incluido `MEMORY-POLICY-1`. Su fuente es [[CLAUDE.md]] §4, que tiene autoridad sobre esta base de conocimiento.

**Filtro de verificación:** `MEMORY-POLICY-FV-7`.
