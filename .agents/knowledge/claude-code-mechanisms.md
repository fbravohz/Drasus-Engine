# Mecanismos Nativos de Claude Code

Qué son y cuándo usar cada uno de los cinco mecanismos de extensión de Claude Code (Rules, Skills, Agents, Hooks, MCP) frente a contenido bespoke en `.agents/`, con la sintaxis exacta de creación según la documentación oficial.

Slug ID: `CLAUDE-CODE-MECHANISMS`.

Tamaño: `13229 caracteres (~3307 tokens aprox., ratio 4:1)`.

---

## CLAUDE-CODE-MECHANISMS-FV — Filtro de Verificación

**Aplica a:** Context Engineer, al clasificar contenido para un mecanismo nativo o al crear uno.

* **CLAUDE-CODE-MECHANISMS-FV-1:** ¿el mecanismo elegido responde a la fila de la matriz de decisión que de verdad aplica (`CLAUDE-CODE-MECHANISMS-6`), no a una elección por costumbre?
* **CLAUDE-CODE-MECHANISMS-FV-2:** ¿la sintaxis de creación (frontmatter, ubicación, campos) coincide con la plantilla de este documento o, si el caso no está cubierto, fue verificada contra la documentación oficial en vez de improvisada?
* **CLAUDE-CODE-MECHANISMS-FV-3:** si el contenido hoy vive en `.agents/knowledge/` o `.claude/skills/`, ¿se señaló la migración al usuario en vez de ejecutarla sin confirmación?

De ser satisfactoria cada validación de la lista, declara `CLAUDE-CODE-MECHANISMS - FILTRO DE VERIFICACION APROBADO`.

---

## CLAUDE-CODE-MECHANISMS-1 — RULES

Contexto de texto que Claude Code inyecta automáticamente al inicio de cada sesión, o solo cuando la ruta de un archivo tocado coincide con un patrón declarado. Es *advisory*: Claude puede ignorarlo si la instrucción es ambigua.

**Aplica a:** Context Engineer, al clasificar o crear un Rule.

* **CLAUDE-CODE-MECHANISMS-1.1 — Qué Es:**

  contexto estático (markdown), sin lógica ni ejecución — el mismo mecanismo detrás de `CLAUDE.md`.

* **CLAUDE-CODE-MECHANISMS-1.2 — Cuándo Usar:**

  la instrucción debe estar presente incluso sin que ningún skill la dispare explícitamente — sobre todo si es específica de una ruta, carpeta o lenguaje (ej. convenciones de un crate de Rust).

* **CLAUDE-CODE-MECHANISMS-1.3 — Ubicación:**

  `./CLAUDE.md` (raíz, todo el proyecto, siempre carga) · `./.claude/rules/<nombre>.md` (con frontmatter `paths:` para acotar por glob, solo carga si la ruta matchea) · `~/.claude/CLAUDE.md` o `~/.claude/rules/` (usuario, todos los proyectos).

**Filtro de verificación:** `CLAUDE-CODE-MECHANISMS-FV-2`.

**Plantilla de referencia:**
```
---
paths:
  - "crates/bridge/**/*.rs"
---

# Convenciones FFI del Bridge

- Todo tipo expuesto a Dart implementa `#[frb(...)]` explícito.
- Nunca exponer un tipo `Option<T>` directo — usar wrapper documentado.
```

---

## CLAUDE-CODE-MECHANISMS-2 — SKILLS

Procedimiento empaquetado, invocable con `/nombre` o descubierto automáticamente por su `description` cuando Claude juzga que aplica. Corre inline en el hilo actual — no en contexto aislado.

**Aplica a:** Context Engineer, al clasificar o crear un Skill.

* **CLAUDE-CODE-MECHANISMS-2.1 — Qué Es:**

  markdown con frontmatter (`name`, `description`, opcionalmente `disable-model-invocation`, `arguments`, `allowed-tools`), más archivos de soporte opcionales (scripts, plantillas) en el mismo directorio. Soporta inyección dinámica de shell con `` !`comando` `` antes de que Claude vea el resto del skill.

* **CLAUDE-CODE-MECHANISMS-2.2 — Cuándo Usar:**

  procedimiento multi-paso que se invoca puntualmente (deploy, review, generación de código), sobre todo si trae archivos de soporte o necesita ser invocable manualmente por el usuario.

* **CLAUDE-CODE-MECHANISMS-2.3 — Ubicación:**

  `./.claude/skills/<nombre>/SKILL.md` (proyecto) · `~/.claude/skills/<nombre>/SKILL.md` (usuario, todos los proyectos).

**Filtro de verificación:** `CLAUDE-CODE-MECHANISMS-FV-2`.

**Plantilla de referencia:**
```
---
name: deploy
description: Deploy the application to production. Use when the user asks to deploy or ship a release.
disable-model-invocation: true
---

Deploy the application:
1. Run the test suite
2. Build the application
3. Push to the deployment target
```

---

## CLAUDE-CODE-MECHANISMS-3 — AGENTS (Subagents)

Contexto aislado con su propio prompt, modelo y lista de herramientas permitidas — protege el hilo principal de resultados intermedios ruidosos (búsquedas, logs, iteraciones).

**Aplica a:** Context Engineer, al clasificar o crear un Agent.

* **CLAUDE-CODE-MECHANISMS-3.1 — Qué Es:**

  markdown con frontmatter (`name`, `description`, `tools`, `model`, opcionalmente `permissionMode`), invocado por el tool `Agent` — Claude decide delegar según la `description`, o el usuario lo invoca directo.

* **CLAUDE-CODE-MECHANISMS-3.2 — Cuándo Usar:**

  la tarea generaría ruido de contexto (muchas búsquedas/resultados intermedios) o necesita un modelo distinto al del hilo principal (ej. un modelo más económico para research) o permisos más restrictos (solo lectura).

* **CLAUDE-CODE-MECHANISMS-3.3 — Ubicación:**

  `./.claude/agents/<nombre>.md` (proyecto) · `~/.claude/agents/` (usuario, todos los proyectos).

**Filtro de verificación:** `CLAUDE-CODE-MECHANISMS-FV-2`.

**Plantilla de referencia:**
```
---
name: code-reviewer
description: Scans code and suggests improvements. Use after writing or modifying code.
tools: Read, Glob, Grep
model: sonnet
---

You are a code reviewer. Analyze the code and provide specific,
actionable feedback on quality, security, and best practices.
```

---

## CLAUDE-CODE-MECHANISMS-4 — HOOKS

Comando shell disparado por el harness (no por decisión del modelo) en un evento del ciclo de vida — `PreToolUse`, `PostToolUse`, `SessionStart`, entre otros. Es el único de los cinco mecanismos que no es *advisory*: se ejecuta siempre que su `matcher` coincide.

**Aplica a:** Context Engineer, al clasificar o crear un Hook.

* **CLAUDE-CODE-MECHANISMS-4.1 — Qué Es:**

  entrada JSON declarada en `settings.json` (evento + `matcher` + comando) que ejecuta un script; el script recibe el contexto del evento por stdin y controla el resultado por código de salida (`exit 0` = continúa, `exit 2` = bloquea la acción, en los eventos que lo permiten).

* **CLAUDE-CODE-MECHANISMS-4.2 — Cuándo Usar:**

  algo debe pasar siempre, sin que el modelo decida — bloquear un archivo protegido, formatear código tras editar, auditar una acción.

* **CLAUDE-CODE-MECHANISMS-4.3 — Ubicación:**

  la entrada del hook vive en `.claude/settings.json` (proyecto) o `~/.claude/settings.json` (usuario) — el script que referencia puede vivir en **cualquier ruta del repo** (ver `CLAUDE-CODE-MECHANISMS-6.4`), `settings.json` solo necesita apuntar a la ruta correcta.

**Filtro de verificación:** `CLAUDE-CODE-MECHANISMS-FV-2`.

**Plantilla de referencia:**
```
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Edit|Write",
        "hooks": [
          {
            "type": "command",
            "command": "\"$CLAUDE_PROJECT_DIR\"/.agents/hooks/protect-files.sh"
          }
        ]
      }
    ]
  }
}
```

---

## CLAUDE-CODE-MECHANISMS-5 — MCP

Conexión a un servicio externo (API, base de datos, navegador) que expone herramientas nuevas. A diferencia de los otros cuatro mecanismos, no es instrucción ni contexto — es una herramienta más que Claude puede invocar.

**Aplica a:** Context Engineer, al clasificar o crear una conexión MCP.

* **CLAUDE-CODE-MECHANISMS-5.1 — Qué Es:**

  servidor que habla el protocolo MCP, vía `stdio` (proceso local que Claude Code arranca) o `http` (servicio ya hosteado); cada herramienta que expone consume tokens de contexto en cada sesión conectada.

* **CLAUDE-CODE-MECHANISMS-5.2 — Cuándo Usar:**

  se necesita conectar con algo fuera del filesystem local del usuario — un servicio, una API, un navegador, una base de datos remota.

* **CLAUDE-CODE-MECHANISMS-5.3 — Ubicación:**

  `.mcp.json` (raíz del proyecto, compartido con quien clona el repo) · `~/.claude.json` (usuario, todos los proyectos).

**Filtro de verificación:** `CLAUDE-CODE-MECHANISMS-FV-2`.

**Plantilla de referencia:**
```
claude mcp add playwright -- npx -y @playwright/mcp@latest
```
equivalente en `.mcp.json`:
```
{
  "mcpServers": {
    "playwright": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@playwright/mcp@latest"]
    }
  }
}
```

---

## CLAUDE-CODE-MECHANISMS-6 — Matriz de Decisión: Cuándo Refactorizar y Cuándo Mantener en `.agents/`

Antes de proponer migrar contenido de `.agents/knowledge/` o `.claude/skills/` a un mecanismo nativo, o de crear uno nuevo, resuelve estas preguntas en orden — la primera que responde "sí" gana. **La pregunta de Hooks va primero siempre**, incluso al crear o refactorizar un conjunto de instrucciones completo (skill, knowledge doc, plantilla): un Hook no compite con Rule/Skill/Agent, los complementa — un mismo conjunto normalmente termina con un Hook para la parte verificable y un Rule o Skill para la parte que sí requiere juicio.

**Aplica a:** Context Engineer.

* **CLAUDE-CODE-MECHANISMS-6.1 — ¿Hay una Parte Mecánicamente Verificable? (se resuelve primero):**

  antes de clasificar el resto, pregunta si una porción del contenido se puede comprobar con un script (grep de un patrón, conteo de caracteres, comparación de un campo declarado contra su valor real) en vez de confiar en que el modelo la recuerde bajo presión de contexto → esa porción va a Hook (`CLAUDE-CODE-MECHANISMS-4.2`), sin importar qué mecanismo termine cubriendo el resto. Un Rule o un Skill son *advisory*: el modelo puede leerlos y aun así olvidarlos; un Hook no depende de que el modelo se acuerde — se ejecuta o bloquea la acción, siempre. Prioriza esta pregunta **sobre todo** al crear o refactorizar un conjunto de instrucciones para agentes, porque ahí es donde más se paga el olvido (formato roto, ID reciclado, campo `Tamaño` desactualizado, checklist saltado).

* **CLAUDE-CODE-MECHANISMS-6.2 — ¿Aplica Sin Ningún Skill Activo?:**

  para lo que quede fuera de un Hook: si el contenido debe regir incluso cuando Claude edita un archivo directamente, fuera de cualquier flujo de skill → Rule con `paths:` (`CLAUDE-CODE-MECHANISMS-1.2`). El gate actual de `.agents/knowledge/README.md` solo dispara **dentro** de un skill que declare la lectura — si el criterio real es la ruta del archivo, no el rol activo, un Rule nativo lo garantiza sin depender de que el skill correcto haya sido invocado.

* **CLAUDE-CODE-MECHANISMS-6.3 — ¿Es Un Procedimiento Invocable?:**

  si es multi-paso, repetible, y el usuario lo dispara puntualmente → Skill (`CLAUDE-CODE-MECHANISMS-2.2`).

* **CLAUDE-CODE-MECHANISMS-6.4 — ¿Necesita Aislamiento?:**

  si generaría ruido de contexto o necesita modelo/permisos distintos al hilo principal → Agent (`CLAUDE-CODE-MECHANISMS-3.2`).

* **CLAUDE-CODE-MECHANISMS-6.5 — ¿Necesita un Servicio Externo?:**

  si requiere conectar con algo fuera del filesystem local → MCP (`CLAUDE-CODE-MECHANISMS-5.2`).

* **CLAUDE-CODE-MECHANISMS-6.6 — Ninguna Aplica — Se Queda en `.agents/`:**

  si nada de lo anterior aplica, el contenido es doctrina o proceso específico de Drasus, consultado solo cuando el skill correcto lo dispara — el caso por defecto de `governance.md`, `writing-efficiency.md` y el resto de la base de conocimiento actual. No son atables a una ruta de archivo ni a un evento del ciclo de vida: son contexto de rol/tarea. El patrón "lee bajo demanda" (`.agents/knowledge/README.md`) es la elección correcta para este caso, no un sustituto pendiente de las Rules nativas.

**Filtro de verificación:** `CLAUDE-CODE-MECHANISMS-FV-1`.

---

## CLAUDE-CODE-MECHANISMS-EJ — Ejemplos

### CLAUDE-CODE-MECHANISMS-EJ-1 — Aplicando la Matriz a `.agents/knowledge/` Real

Caso real: ¿`commenting-policy.md` debería migrar a un mecanismo nativo? — demuestra `CLAUDE-CODE-MECHANISMS-6.1`, `-6.2` y `-6.6` actuando juntas, no de forma binaria.

* **Pregunta 6.1 (se resuelve primero):** ¿hay una parte mecánicamente verificable? Sí — la Capa 3 (Ponytail) exige un formato exacto de comentario inline (`ponytail: <razón>, escalar si <condición>`) que un Hook `PostToolUse` puede validar por regex sobre cualquier `Edit`/`Write` a `crates/**/*.rs` o `ui/**/*.dart`, y bloquear (`exit 2`) si el comentario no matchea. Esa porción va a Hook, independiente de lo que pase con el resto.
* **Pregunta 6.2:** ¿el resto aplica sin ningún skill activo? Sí — ocurre cada vez que Claude edita código directamente sin pasar por un skill de rol de ingeniero. Punto a favor de un Rule con `paths:` para el resumen de las 4 capas.
* **Pero:** la política completa tiene matices de precedencia entre capas — no cabe entera en un Rule de texto plano sin volverlo largo y caro de inyectar en cada archivo tocado, incluso cuando el trabajo no necesita esa profundidad.
* **Conclusión:** tres piezas, no una — un Hook que valida el formato de Ponytail mecánicamente, un Rule corto (`.claude/rules/comments.md`, `paths: **/*.rs, **/*.dart`) con el resumen de 3 líneas de las capas 1-2, y `commenting-policy.md` intacto para el detalle completo de las 4 capas y sus matices de precedencia — que sigue exigiendo juicio, no verificación mecánica.

---

## Documentos Relacionados

* `.agents/knowledge/context-window-budget.md` — costo de inyectar contexto por sesión; relevante al decidir si un Rule (siempre carga) vale su costo frente a un knowledge doc (bajo demanda).
