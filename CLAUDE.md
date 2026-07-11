# CLAUDE.md — Mapa de Orientación y Protocolo de Contexto

Este archivo se carga automáticamente al inicio de cada sesión y se cachea. Es el lugar **más barato** para el contexto que siempre se necesita. Por eso es un **mapa**, no un volcado: orienta y dice dónde está cada cosa. Nadie debe copiar aquí el contenido de otros documentos.

**Regla rectora de tokens:** lee bajo demanda, nunca en masa. El conocimiento vive en archivos; este mapa dice cuál abrir y cómo abrirlo barato.

---

## 1. Invariantes de Arquitectura (FIJO — no se debaten)

- **Stack único:** Rust (Core/Backend) + Flutter con Dart/Impeller (Frontend). Prohibido Python, FastAPI, Tauri, React, TypeScript en el diseño. Residuos se purgan a FFI nativo (`flutter_rust_bridge`) y Rust.
- **Modo Headless (SaaS):** gRPC.
- **FCIS:** Functional Core (lógica pura) / Imperative Shell (orquestación). Los módulos son Shell delgada; **toda lógica vive en una Feature**.
- **Arquitectura Hexagonal (ADR-0137):** cada feature es un hexágono con puertos tipados (`InputPorts` / `OutputPorts`). Los módulos son presets de composición, no dueños. Cada feature-crate depende solo de `shared`; prohibido acoplar features entre sí.
- **Workspace de crates:** `shared/` (tipos ADR-0137 + plumbing), `features/<dominio>/<feature>/` (un crate hexagonal por feature), `presets/` (cableado sin lógica), `app/` (binario), `bridge/` (FFI). **Excepción bendecida (ADR-0137, enmienda 2026-06-23 y sucesivas hasta 2026-07-11):** las features de infraestructura crosscutting viven en `crates/shared`, NO como crate propio — solo si producen un tipo `textLabel` del catálogo, son consumidas por ≥2 dominios y no exponen puerto de Alpha en el canvas. El conjunto fundacional fueron seis (`clock`, `audit-log`, `async-job-executor`, `telemetry`, `worker-isolation-orchestrator`, `agentic-mcp-gateway`); las enmiendas de ADR-0137 (2026-07-03/07-04/07-11) extendieron la excepción a los **14 cimientos del substrato de monetización** (#1–#14, ADR-0144..0149), que el código correctamente coloca en `shared`. Toda feature que produzca un tipo de dominio va a su crate hexagonal. **La lista vigente y el criterio canónico viven en ADR-0137** (este mapa no la duplica para no derivar).
- **Zero-Docker** y **cómputo Local-First:** sin contenedores en el core; el motor (backtesting/ejecución) corre siempre en el hardware del usuario (su PC o su VPS headless), nunca en servidores del proveedor. Prohibido acceso cross-module a tablas ajenas (se expone un puerto en la `public_interface` del dueño).
- **Tres Planos + Soberanía Condicionada por Tier (ADR-0143 — enmienda a Local-First/Zero-Telemetry):** el sistema tiene tres planos: UI (local), Ejecución (hardware del usuario), y **Cabina de Mando Central del proveedor** (servidor de Drasus que NUNCA computa: solo autentica, licencia, ingiere telemetría y agrega). **Zero-Telemetry queda derogado:** toda instancia mantiene canal de control. La soberanía de datos depende del tier: **gratis** = el trabajo del usuario (estrategias, backtests, portafolios, resultados, instrumentos) fluye al proveedor y es suyo por ToS; **pago al corriente** = supresión de telemetría de trabajo en origen (privacidad real, vendible); **pago vencido** = se reactiva la emisión (el entorno no se borra). **Secretos (credenciales de bróker, IPs live) jamás salen, en ningún tier** (ADR-0093). **La Cabina de Mando (Plano #3) se materializa en el repo hermano `../drasus-portal/`** (servidor central del proveedor + cara web pública; **stack libre**, NO hereda Rust+Flutter — ADR-0001 acotado por ADR-0145). Es un repo aparte que **referencia** este diseño (no lo redefine); su `README.md` fija su alcance y consume los 10 cimientos del substrato (ADR-0144/0145). El detalle canónico vive en ADR-0143/ADR-0144; el ADR gana sobre este resumen.
- **Foundation Inundation (ADR-0020):** ante duda genuina, incluir. El detalle y la tabla canónica de 4 perfiles viven en `docs/ADR.md` (ADR-0020) — el ADR gana sobre cualquier resumen.
- **Configurable vs Fijo:** "NUNCA/SIEMPRE" = invariante físico, fijo. "Umbral/Max/Min" = parámetro configurable.
- **FASE DEL PROYECTO: GREENFIELD (pre-release).** Ningún usuario final ejecuta aún una build distribuida. Implica: el baseline de migraciones SQL es editable in-situ (recrear tablas con STRICT, corregir tipos, renombrar columnas, sin migration incremental). El congelamiento a **BROWNFIELD** se dispara con el primer release distribuido; a partir de ahí las migraciones son forward-only y robustas a saltos de versión. Detalle canónico en ADR-0006 (enmienda 2026-06-28).
* **Agrupación de commits por tipo:** cuando el usuario autoriza commitear, agrupa los cambios pendientes en commits separados por tipo (`feat`, `docs`, `chore`, `fix`, `test`). Nunca un commit masivo de todo. Nunca commitees en automático sin que el usuario lo pida explícitamente en el turno actual.

---

## 2. Mapa Documental — Dónde Vive Cada Cosa

Todo el diseño vive bajo `docs/`. Tamaños aproximados para decidir cómo leer:

| Documento | Qué contiene | Tamaño | Cómo leerlo |
|---|---|---|---|
| `docs/README.md` | **Índice maestro**: tabla de módulos, ~138 features con "consumido por", moonshots, índice de 117 ADRs | ~356 líneas | Es la navegación. Léelo para localizar; no lo memorices entero. |
| `docs/ADR.md` | Índice de los 117 ADR | ~123 líneas | Para un ADR concreto, abre `docs/adr/ADR-XXXX.md` (≈10–55 líneas). |
| `docs/adr/ADR-XXXX.md` | Un ADR por archivo (0001–0117) | ≈10–55 líneas | Abre solo el ADR que necesitas. |
| `docs/SAD.md` | Índice de las 20 secciones | ~26 líneas | Para una sección, abre `docs/sad/SAD-NN.md`. |
| `docs/sad/SAD-NN.md` | Una sección del SAD por archivo (01–20) | variable | Abre solo la sección relevante. |
| `docs/ROADMAP.md` | Épicas, sprints, spikes | ~365 líneas | Lee la sección de la fase activa. |
| `docs/templates/TEMPLATES.md` | Índice + reglas transversales (Lo Prohibido, Regla de Oro, Checklist) | ~100 líneas | Para una plantilla concreta, abre `docs/templates/<NOMBRE>.md`. |
| `docs/templates/<NOMBRE>.md` | Una plantilla por archivo: `ADR.md`, `SAD.md`, `FEATURE.md`, `TTR.md` | ≈40–150 líneas | Abre solo la plantilla que vas a usar. |
| `docs/modules/*.md` | 8 orquestadores: ingest, generate, validate, incubate, manage, execute, feedback, withdraw | 280–833 líneas c/u | Abre solo el módulo en juego. |
| `docs/features/*.md` | ~138 features (lógica pura / drivers) | variable | Abre solo la(s) feature(s) relevante(s). |
| `docs/moonshots/*.md` | ~41 proyectos experimentales | variable | Solo si el trabajo es R&D experimental. |
| `docs/execution/*.md` | Órdenes de trabajo (ejecución) | variable | La orden concreta que se está ejecutando. |
| `docs/lessons/<dominio>/*.md` | Lecciones de aprendizaje acumuladas por tema (Modos Mentor/Revisión/Docente, ADR-0122) — no por tarea | variable | Abre solo el tema relevante; índice de carpetas en `docs/lessons/README.md`. |

**Este archivo y el ROADMAP son editables por el Architect.** `CLAUDE.md` (este mapa) y `docs/ROADMAP.md` (la guía de orden de entregas) no son de solo lectura: el Architect los actualiza cuando el mapa documental cambia (nueva sección, ADR, feature o módulo que altera la navegación) o cuando una decisión arquitectónica nueva reordena o añade una fase/entrega al ROADMAP. Edición quirúrgica igual que el resto (`Edit` en bloques pequeños, nunca reescritura completa).

**Pipeline de módulos:** `ingest → generate → validate → incubate → manage → execute → feedback → withdraw`.

**Skills (agentes):** `.claude/skills/<rol>/SKILL.md` (frontmatter de descubrimiento; el contenido completo vive en `.agents/skills/<rol>/SKILL.md`, referenciado desde ahí). 

**Base de Conocimiento (obligatoria para skills):** índice con disparadores en **`.agents/knowledge/README.md`** (qué documento abrir y cuándo).
- `.agents/knowledge/base.md` — Gobernanza meta + mapa de referencias (supremacía absoluta).
- `.agents/knowledge/commenting-policy.md` — HOW TO: escribir comentarios (4 capas jerárquicas).
- `.agents/knowledge/debt-management.md` — HOW TO: registrar deuda técnica (`ponytail:` vs. DEBT-XXX).
- `.agents/knowledge/ponytail.md` — HOW TO: simplificación deliberada (Capa 3 de comentarios).
- `.agents/knowledge/memory-policy.md` — HOW TO: qué y cómo recordar entre sesiones (mecánica del harness + disciplina de Drasus + `.agents/memory/` vs. `.agents/state/`).
- `.agents/knowledge/critical-domain-reasoning.md` — HOW TO: ojo crítico experto antes de sellar decisiones de dominio (7 comprobaciones + gate de especialista); obligatorio para Architect/TL/ingenieros al canonizar reglas cuant/estadística/cripto/legal.

Lee bajo demanda, nunca en masa. Si necesitas comentarios, lee `commenting-policy.md` (~40 líneas). Si necesitas deuda, lee `debt-management.md` (~50 líneas). Si necesitas memoria, lee `memory-policy.md` (~130 líneas). Si necesitas gobernanza meta, lee `base.md` (~234 líneas). Si vas a **sellar una decisión de dominio** (cuant/estadística/cripto/fiscal/legal), lee `critical-domain-reasoning.md`.

**Gate de Reportaje de Conocimiento (OBLIGATORIO):** cuando detectes que el concepto en el que trabajas toca el área de un documento de knowledge (los del índice), léelo bajo demanda y **declara explícitamente al invocador** qué documento consultaste y por qué (`[knowledge consultado: <doc> — <motivo>]`). `base.md` se declara siempre al arrancar; los demás, cuando se consultan. El invocador (usuario / Architect / Tech-Lead) debe saber qué conocimiento se usó para resolver cada conflicto real.

---

## 3. Protocolo de Recuperación de Contexto (Eficiencia de Tokens)

El objetivo es traer **solo el fragmento exacto** que el trabajo necesita, no archivos completos "por si acaso".

1. **Localiza con el índice, no leyendo.** El `README.md` te dice qué módulo/feature/ADR toca. Empieza ahí.
2. **`grep` antes de `Read`.** Para encontrar una sección concreta (una regla, un ADR, un contrato), busca el patrón y lee solo alrededor del resultado.
   - Ejemplo ADR: abre directamente `docs/adr/ADR-0117.md` (≈25 líneas). No hace falta cargar el índice ni recorrer un monolito.
3. **Lee por sección.** SAD y ROADMAP se leen por apartado, usando el offset del resultado de búsqueda.
4. **Delega los barridos a subagentes.** Cualquier tarea que obligue a recorrer muchos archivos (auditoría de integridad relacional sobre las ~138 features, "¿qué features consume el módulo X?", rastrear referencias huérfanas) va a un subagente de exploración: este por defecto deber ser *Sonnet o un equivalente si cambia de nombre*, *opus y agentes se reservan para tareas extremas como subagentes y bajo previa autorizacion del usuario*. *Jamas despaches un subagente opus si el usuario no te lo pidio explicitamente o le preguntaste segun tu inteligencia y determinaste que la tarea necesitaba opus o equivalente* corre en su propia ventana de contexto y devuelve **solo la conclusión**, sin contaminar la principal. Este es el "RAG nativo" de este entorno.
5. **No releas lo que no cambió.** Si ya leíste un archivo en este turno y no se ha editado, no lo vuelvas a abrir.
6. **Lectura progresiva encadenada.** Para un archivo grande que sí debes recorrer entero, encadena `Read` por el `offset` exacto que indica el truncamiento; nunca desde 0 otra vez, nunca rangos repetidos.

---

## 4. Memoria entre Sesiones (Recuerdo tipo "persona")

Existe memoria nativa de proyecto en `~/Drasus-Engine/.agents/memory/` (índice `MEMORY.md` + un hecho por archivo). Se carga cada sesión: por eso un agente "recuerda" decisiones pasadas sin que se las repitan. Es curada, no automática — nunca dupliques lo que ya registra el código, el git o estos documentos.

**📖 Política completa (mecánica del harness + disciplina de Drasus + protocolo del Tech-Lead + por qué `.agents/state/` es un sistema distinto):** `.agents/knowledge/memory-policy.md`.

---

## 5. Governance — Fuente Única de Verdad

Las reglas operativas canónicas (rigor, anti-alucinación, anti-obsolescencia, gate de creación de documentos, edición quirúrgica, idioma, saneamiento terminológico) viven en **`.agents/knowledge/base.md`**, que gobierna a todos los skills. **No es una skill invocable** — es conocimiento base de lectura obligatoria, referenciado por cada skill al iniciar. **No se replican aquí** para evitar deriva: ante cualquier duda de governance, ve a `base`; si algo de aquí contradice a `base`, gana `base`. La instrucción explícita del usuario gana siempre.

Recordatorio mínimo siempre activo (el detalle está en `base`): no inventes nombres, rutas ni snippets; edita con `Edit` en bloques pequeños, nunca reescribas un documento entero; lee bajo demanda (§3); español con acentos en prosa; en código, identificadores en inglés y comentarios en español (ADR-0121).
