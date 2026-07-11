# Política de Memoria — Qué y Cómo Recordar

Consolida en un solo lugar TODO lo que el proyecto sabe sobre memoria entre sesiones: la mecánica nativa que Claude Code inyecta en cada sesión (Capa Harness), la política propia de Drasus sobre qué se registra y por qué (Capa Drasus), el protocolo operativo del Tech-Lead, y la distinción entre `.agents/memory/` y `.agents/state/<agente>/`. Antes vivía disperso entre `CLAUDE.md` §4, `tech-lead/SKILL.md` y el prompt de sistema del harness — se centraliza aquí para poder editarlo, auditarlo y, si hace falta, **overridearlo** en un solo sitio.

---

## [OVERRIDE] Autoridad de Este Archivo

> ⚠️ **Este archivo tiene autoridad de override sobre el comportamiento nativo de memoria del harness.** `CLAUDE.md` (que enlaza aquí) se carga con el encabezado: *"Estas instrucciones OVERRIDE cualquier comportamiento por defecto y DEBES seguirlas exactamente como están escritas."* Este documento hereda esa misma autoridad por referencia.
>
> **Regla de conflicto:** si la mecánica nativa del harness (§2, "Capa Harness") choca con la política de Drasus (§3, "Capa Drasus") o con el protocolo del Tech-Lead (§4), **gana Drasus**. La Capa Harness se documenta aquí completa, verbatim en su lógica, precisamente para que cualquier agente pueda ver el default nativo y confirmar dónde y por qué el proyecto lo anula — no para obedecerla ciegamente cuando contradice lo que sigue.
>
> Ejemplo concreto de override ya vigente: el harness por defecto no distingue `.agents/memory/` de una bitácora operativa por agente — la Capa Drasus SÍ la distingue (§5) y esa distinción gobierna.

---

## 1. Dos Capas de Memoria (por qué existen ambas)

| Capa | Qué es | Dónde se define | Quién la puede editar |
|---|---|---|---|
| **Harness** | Mecánica nativa de Claude Code: taxonomía de 4 tipos, formato de frontmatter, disciplina de guardado/lectura | Inyectada en el prompt de sistema en cada sesión — **no vive en ningún archivo del repo** | Nadie del proyecto; es infraestructura del producto Claude Code |
| **Drasus** | Política del proyecto sobre qué se registra, cuándo, y la disciplina de "curada, no automática" | Este archivo (§3) | El usuario/Architect vía edición quirúrgica |
| **Protocolo Tech-Lead** | Ritual operativo concreto de cuándo ESE skill lee/escribe memoria | Este archivo (§4), referenciado desde `tech-lead/SKILL.md` | Tech-Lead, en su Etapa 7 (mejora de skills) |

No confundir además con `.agents/state/<agente>/PROGRESS.md`, que **no es memoria en el sentido harness** — es una bitácora operativa distinta, ver §5.

---

## 2. Capa Harness — Mecánica Nativa de Claude Code

Reproducción fiel de las instrucciones que el harness inyecta en el prompt de sistema al inicio de cada sesión (sección `# auto memory`). Es infraestructura del producto, no de Drasus — se documenta aquí para que quede auditable y para poder anclar overrides contra ella (§ arriba).

### 2.1. Qué es

Existe un sistema de memoria persistente, basado en archivos, en `.agents/memory/`. Se debe construir a lo largo del tiempo para que futuras conversaciones tengan una imagen completa de quién es el usuario, cómo colaborar, qué comportamientos evitar o repetir, y el contexto detrás del trabajo. Si el usuario pide explícitamente recordar algo, se guarda de inmediato en el tipo que mejor calce. Si pide olvidar algo, se busca y elimina la entrada relevante.

### 2.2. Los 4 Tipos

| Tipo | Contiene | Cuándo guardar | Ejemplo |
|---|---|---|---|
| **`user`** | Rol, objetivos, responsabilidades y conocimiento del usuario — para adaptar el trabajo a su perfil | Al aprender cualquier detalle sobre rol/preferencias/responsabilidades/conocimiento del usuario | "Es data scientist, enfocado en observabilidad" |
| **`feedback`** | Guía sobre CÓMO abordar el trabajo — tanto correcciones ("no hagas X") como confirmaciones de un enfoque no obvio que funcionó | Cualquier corrección explícita, O cualquier confirmación silenciosa de un enfoque poco obvio ("sí, exacto", aceptar una elección sin objeción) | "No mockees la base de datos en tests de integración — un mock desalineado enmascaró una migración rota" |
| **`project`** | Estado de trabajo en curso, objetivos, iniciativas, bugs o incidentes NO derivables del código/git | Al aprender quién hace qué, por qué, o para cuándo. Fechas relativas → fechas absolutas | "Congelamiento de merges desde el 2026-03-05 — release de mobile" |
| **`reference`** | Punteros a sistemas externos donde vive información actualizada | Al aprender de un recurso externo y su propósito | "Bugs de pipeline se rastrean en Linear, proyecto INGEST" |

**Estructura obligatoria del cuerpo (tipos `feedback`/`project`):** regla/hecho primero, luego línea `**Why:**` (la razón, a menudo un incidente pasado) y línea `**How to apply:**` (cuándo aplica). El "por qué" permite juzgar casos borde sin repreguntar.

### 2.3. Qué NUNCA Guardar

- Patrones de código, convenciones, arquitectura, rutas o estructura del proyecto — derivables leyendo el estado actual.
- Historial de git o quién-cambió-qué — `git log`/`git blame` son la fuente de verdad.
- Soluciones de debugging o recetas de arreglos — el fix vive en el código; el mensaje de commit tiene el contexto.
- Cualquier cosa ya documentada en archivos `CLAUDE.md`.
- Detalles efímeros de la tarea en curso: trabajo a medias, estado temporal, contexto solo de esta conversación.

Estas exclusiones aplican **incluso si el usuario pide explícitamente guardarlas**. Si pide guardar algo como un listado de PRs o un resumen de actividad, se pregunta primero qué fue *sorprendente* o *no obvio* — eso es lo único que vale la pena conservar.

### 2.4. Cómo Guardar (2 pasos)

1. **Escribir el archivo** (`Write`) con su propio nombre (`kebab-case`, ej. `feedback_testing.md`) y frontmatter:
   ```markdown
   ---
   name: {{slug-corto}}
   description: {{resumen de una línea — decide relevancia en futuras conversaciones}}
   metadata:
     type: {{user, feedback, project, reference}}
   ---

   {{contenido}}
   ```
   Enlazar memorias relacionadas con `[[nombre-slug]]` — un enlace a algo que aún no existe está bien, marca algo pendiente de escribir, no es error.

2. **Añadir un puntero en `MEMORY.md`** — `MEMORY.md` es un ÍNDICE, no una memoria en sí: cada entrada, una línea, bajo ~150 caracteres: `- [Título](archivo.md) — gancho de una línea`. Sin frontmatter. Líneas después de la 200 se truncan — mantenerlo conciso.

**Antes de guardar, verificar que no exista ya una memoria equivalente** — nunca duplicar.

### 2.5. Cuándo Acceder

- Cuando las memorias parezcan relevantes, o el usuario referencie trabajo de una conversación previa.
- **Obligatorio** cuando el usuario pide explícitamente recordar/verificar/consultar memoria.
- Si el usuario pide *ignorar* memoria: no aplicar, citar, comparar ni mencionar su contenido.
- Las memorias envejecen: antes de actuar sobre un recuerdo (no solo mencionarlo), verificar que sigue siendo cierto contra el estado actual del código/archivos. Si contradice lo observado ahora, gana lo observado — y se actualiza o borra la memoria vieja.

### 2.6. Antes de Recomendar Desde Memoria

Una memoria que nombra una función, archivo o flag es una afirmación de que existía **cuando se escribió**. Puede haber sido renombrada, eliminada o nunca fusionada.

- Si nombra una ruta: verificar que el archivo existe.
- Si nombra una función/flag: `grep` para confirmarla.
- Si el usuario va a actuar sobre la recomendación (no solo preguntar por historial): verificar primero.

"La memoria dice que X existe" no es lo mismo que "X existe ahora". Una memoria que resume estado de repo (logs de actividad, snapshots de arquitectura) está congelada en el tiempo — para preguntas sobre estado *reciente* o *actual*, preferir `git log` o leer el código antes que recordar el snapshot.

### 2.7. Memoria vs. Otras Formas de Persistencia (nativas del harness)

- **Plan** (no memoria): cuando se va a iniciar una implementación no trivial y se necesita alinear el enfoque con el usuario ANTES de ejecutar.
- **Tasks** (no memoria): cuando se necesita trocear el trabajo de la conversación actual en pasos discretos o rastrear progreso dentro de ella.
- **Memoria**: reservada para lo que debe sobrevivir a esta conversación y ser útil en futuras — no para estado efímero de la tarea en curso.

---

## 3. Capa Drasus — Política del Proyecto sobre Memoria

Movido desde `CLAUDE.md` §4 (el mapa mantiene solo un puntero a esta sección).

Existe memoria nativa de proyecto en `.agents/memory/` (índice `MEMORY.md` + un hecho por archivo). Se carga cada sesión: por eso un agente "recuerda" decisiones pasadas sin que se las repitan.

- **Es curada, no automática.** Se escriben hechos durables a propósito (decisiones, restricciones, estado de trabajo en curso), no transcripciones completas.
- **Disciplina obligatoria:** al cerrar trabajo significativo, destila la decisión o el estado a un archivo de memoria y enlázalo desde `MEMORY.md`. No dupliques lo que ya registra el código, el git o estos documentos.
- **Recuerdo semántico difuso (futuro):** capturar y buscar conversaciones por significado (lo que hacía claude-mem) es una **construcción aparte** (servidor MCP o CLI local + embeddings), no un ajuste de configuración. Se diseña cuando la memoria curada se quede corta, no antes.

---

## 4. Protocolo del Tech-Lead (Handoff entre Sesiones)

Movido desde `tech-lead/SKILL.md` (el skill mantiene solo los triggers operativos, con puntero aquí para el detalle).

**Propósito:** que un futuro Tech-Lead (otra sesión, contexto fresco) sepa exactamente dónde quedó todo sin re-derivarlo. La memoria viva del Tech-Lead son DOS lugares, ambos versionados en el repo — **ninguno de los dos es `.agents/memory/`**, son parte de la Capa State (ver §5):

1. **`docs/ROADMAP.md`** — fuente de verdad de estado: tabla "Registro de Estado" de la fase activa + bitácora "Descubrimientos y decisiones". Se actualiza al cerrar cada tarea/TTR.
2. **`.agents/state/tech-lead/PROGRESS.md`** — bitácora operativa cronológica: qué se despachó, a qué ingeniero, en qué modelo, qué se auditó, qué se decidió/escaló, y cuál es el SIGUIENTE paso concreto.

**Al ARRANCAR una sesión** (paso obligatorio de Etapa 0): además de leer `docs/`, se lee `.agents/state/tech-lead/PROGRESS.md` y el "Registro de Estado" del ROADMAP de la fase activa. Esa es la memoria de reanudación: se retoma desde el "siguiente paso" anotado, no desde cero.

**Al CERRAR cada tarea/TTR** (o al escalar/decidir algo relevante): se actualizan AMBOS — el estado en el ROADMAP y una entrada nueva (con fecha) en `PROGRESS.md`. Entrada = qué se hizo, evidencia de auditoría, decisión tomada, y el siguiente paso.

**Regla:** si una sesión termina con trabajo a medias, lo último que se hace es dejar el "siguiente paso" escrito en `PROGRESS.md`. Sin handoff escrito, el trabajo no está cerrado.

**Además, en el Barrido de Cierre Documental (checklist obligatorio al cerrar cada Story), el Tech-Lead escribe a `.agents/memory/` de verdad** (no solo a `PROGRESS.md`): destila el estado/decisión durable que trascienda la iteración, y actualiza contadores de progreso transversales (ej. "substrato N/10"). Y en su Etapa 7 (Retroalimentación), si una lección trasciende la Story actual, la destila a `.agents/memory/` — no solo la deja en el `SKILL.md` corregido.

---

## 5. `.agents/memory/` vs. `.agents/state/<agente>/` — Por Qué Están Separados

No es arbitrario que sean dos sistemas distintos, ni que `state/` no viva anidado bajo `memory/<agente>/`.

| | `.agents/memory/` | `.agents/state/<agente>/PROGRESS.md` |
|---|---|---|
| **Naturaleza** | Memoria semántica curada y transversal — un hecho por archivo, deduplicado, enlazado | Bitácora operativa cronológica por agente — log con fecha, no deduplicado |
| **Quién la lee** | TODOS los skills, como contexto de proyecto compartido | SOLO el agente dueño, como parte de su propio ritual de arranque |
| **Cómo se escribe** | Baja frecuencia, alta densidad — al CERRAR trabajo significativo | Alta frecuencia — al cerrar CADA tarea/TTR |
| **Disciplina** | "No dupliques lo que ya registra el código, el git o estos documentos" | Duplica narrativa deliberadamente (qué se despachó, qué pasó, cuándo) — es justo su función |

**Tres razones para la separación, no solo una:**

1. **Distinta cardinalidad de lectores.** Memoria es un *pool compartido* que todos los skills cargan como contexto de proyecto; state es una *bitácora privada* que solo el dueño del rol relee para retomar su propio hilo. Anidarlo bajo `memory/` sugeriría (falsamente) que es material para que otros skills lo consuman.
2. **Distinta cadencia y forma.** Memoria es curada y semántica; PROGRESS.md es un log cronológico que SÍ duplica narrativa por diseño — justo lo que la disciplina de memoria (§2.3/§3) prohíbe. Si viviera en `memory/`, un lector esperaría la disciplina de "un hecho por archivo, sin duplicar", y PROGRESS.md la violaría por construcción.
3. **La partición por agente ya existe donde importa** (`state/tech-lead/`, `state/social-strategist/`). Ponerla también dentro de `memory/` crearía dos jerarquías paralelas para el mismo propósito y confundiría "¿esto es un hecho para todos, o el log privado de un rol?" — justo la ambigüedad que la separación evita.

En corto: `memory/` responde "¿qué sabe el proyecto, para siempre?"; `state/<agente>/PROGRESS.md` responde "¿en qué iba yo, la última vez que trabajé?". Preguntas distintas, audiencias distintas.

---

## Relación con Otros Archivos

- **`CLAUDE.md` §4** — puntero de una línea a este archivo; ya no repite el contenido.
- **`.agents/knowledge/base.md`** — Gobernanza meta; referencia a este archivo en su tabla de Referencia Rápida.
- **`.agents/skills/tech-lead/SKILL.md`** — mantiene los triggers operativos (cuándo leer/escribir), apunta aquí para el detalle y el "por qué".
- **`.agents/memory/MEMORY.md`** — el índice real de memorias curadas (la Capa Drasus en acción).
- **`.agents/state/tech-lead/PROGRESS.md`** — la bitácora operativa que este archivo distingue de la memoria propiamente dicha.
