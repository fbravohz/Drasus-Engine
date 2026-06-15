# CLAUDE.md — Mapa de Orientación y Protocolo de Contexto

Este archivo se carga automáticamente al inicio de cada sesión y se cachea. Es el lugar **más barato** para el contexto que siempre se necesita. Por eso es un **mapa**, no un volcado: orienta y dice dónde está cada cosa. Nadie debe copiar aquí el contenido de otros documentos.

**Regla rectora de tokens:** lee bajo demanda, nunca en masa. El conocimiento vive en archivos; este mapa dice cuál abrir y cómo abrirlo barato.

---

## 1. Invariantes de Arquitectura (FIJO — no se debaten)

- **Stack único:** Rust (Core/Backend) + Flutter con Dart/Impeller (Frontend). Prohibido Python, FastAPI, Tauri, React, TypeScript en el diseño. Residuos se purgan a FFI nativo (`flutter_rust_bridge`) y Rust.
- **Modo Headless (SaaS):** gRPC.
- **FCIS:** Functional Core (lógica pura) / Imperative Shell (orquestación). Los módulos son Shell delgada; **toda lógica vive en una Feature**.
- **Zero-Docker** y **Local-First / Soberanía de Datos:** sin contenedores en el core; el estado vive local. Prohibido acceso cross-module a tablas ajenas (se expone un puerto en la `public_interface` del dueño).
- **Foundation Inundation (ADR-0020 V2):** ante duda genuina, incluir. El detalle y la tabla canónica de 4 perfiles viven en `docs/ADR.md` (ADR-0020 V2) — el ADR gana sobre cualquier resumen.
- **Configurable vs Fijo:** "NUNCA/SIEMPRE" = invariante físico, fijo. "Umbral/Max/Min" = parámetro configurable.

---

## 2. Mapa Documental — Dónde Vive Cada Cosa

Todo el diseño vive bajo `docs/`. Tamaños aproximados para decidir cómo leer:

| Documento | Qué contiene | Tamaño | Cómo leerlo |
|---|---|---|---|
| `docs/README.md` | **Índice maestro**: tabla de módulos, ~138 features con "consumido por", moonshots, índice de 117 ADRs | ~356 líneas | Es la navegación. Léelo para localizar; no lo memorices entero. |
| `docs/ADR.md` | 117 decisiones de arquitectura | **~2.047 líneas** | NUNCA entero. Ver §3. |
| `docs/SAD.md` | Diseño de alto nivel | ~1.348 líneas | Por sección, no entero. Ver §3. |
| `docs/ROADMAP.md` | Épicas, sprints, spikes | ~365 líneas | Lee la sección de la fase activa. |
| `docs/TEMPLATES.md` | Plantillas maestras + "LO PROHIBIDO" (§4.0) | ~522 líneas | Lee la plantilla concreta que vas a usar. |
| `docs/modules/*.md` | 8 orquestadores: ingest, generate, validate, incubate, manage, execute, feedback, withdraw | 280–833 líneas c/u | Abre solo el módulo en juego. |
| `docs/features/*.md` | ~138 features (lógica pura / drivers) | variable | Abre solo la(s) feature(s) relevante(s). |
| `docs/moonshots/*.md` | ~41 proyectos experimentales | variable | Solo si el trabajo es R&D experimental. |
| `docs/execution/*.md` | Órdenes de trabajo (ejecución) | variable | La orden concreta que se está ejecutando. |

**Pipeline de módulos:** `ingest → generate → validate → incubate → manage → execute → feedback → withdraw`.

**Skills (agentes):** `.claude/skills/<rol>/SKILL.md`. `base/SKILL.md` tiene supremacía sobre todos.

---

## 3. Protocolo de Recuperación de Contexto (Eficiencia de Tokens)

El objetivo es traer **solo el fragmento exacto** que el trabajo necesita, no archivos completos "por si acaso".

1. **Localiza con el índice, no leyendo.** El `README.md` te dice qué módulo/feature/ADR toca. Empieza ahí.
2. **`grep` antes de `Read`.** Para encontrar una sección concreta (una regla, un ADR, un contrato), busca el patrón y lee solo alrededor del resultado.
   - Ejemplo ADR: `grep` del patrón `### **ADR-0117` en `docs/ADR.md` → `Read` con `offset`/`limit` ceñido a esa sección (≈30–80 líneas), nunca las 2.047.
3. **Lee por sección.** SAD y ROADMAP se leen por apartado, usando el offset del resultado de búsqueda.
4. **Delega los barridos a subagentes.** Cualquier tarea que obligue a recorrer muchos archivos (auditoría de integridad relacional sobre las ~138 features, "¿qué features consume el módulo X?", rastrear referencias huérfanas) va a un subagente de exploración: corre en su propia ventana de contexto y devuelve **solo la conclusión**, sin contaminar la principal. Este es el "RAG nativo" de este entorno.
5. **No releas lo que no cambió.** Si ya leíste un archivo en este turno y no se ha editado, no lo vuelvas a abrir.
6. **Lectura progresiva encadenada.** Para un archivo grande que sí debes recorrer entero, encadena `Read` por el `offset` exacto que indica el truncamiento; nunca desde 0 otra vez, nunca rangos repetidos.

---

## 4. Memoria entre Sesiones (Recuerdo tipo "persona")

Existe memoria nativa de proyecto en `.claude/projects/.../memory/` (índice `MEMORY.md` + un hecho por archivo). Se carga cada sesión: por eso un agente "recuerda" decisiones pasadas sin que se las repitan.

- **Es curada, no automática.** Se escriben hechos durables a propósito (decisiones, restricciones, estado de trabajo en curso), no transcripciones completas.
- **Disciplina obligatoria:** al cerrar trabajo significativo, destila la decisión o el estado a un archivo de memoria y enlázalo desde `MEMORY.md`. No dupliques lo que ya registra el código, el git o estos documentos.
- **Recuerdo semántico difuso (futuro):** capturar y buscar conversaciones por significado (lo que hacía claude-mem) es una **construcción aparte** (servidor MCP o CLI local + embeddings), no un ajuste de configuración. Se diseña cuando la memoria curada se quede corta, no antes.

---

## 5. Reglas de Governance (resumen — el detalle manda en sus archivos)

- **Lectura previa obligatoria:** antes de crear o editar diseño, lee `docs/README.md`.
- **Anti-obsolescencia:** prohibido inventar nombres de variables, funciones, clases o snippets en documentos. Describe el contrato y el comportamiento observable.
- **Gate de creación de documentos:** flujo permitido = `ADR.md`, `SAD.md`, `TEMPLATES.md`, `modules/*.md`, `features/*.md`, `moonshots/*.md`, `README.md`. Prohibido sin preguntar: `*-AUDIT.md`, `*-SUMMARY.md`, `*-PLAN.md`.
- **Edición quirúrgica:** usa `Edit` en bloques pequeños; nunca reescribas un documento entero (riesgo de perder densidad).
- **Idioma:** español con acentos para prosa; inglés para código.
- **Saneamiento terminológico:** nomenclatura institucional (Ingest/Generate, Validate, Feedback, Orchestrator, Execute, Withdraw), no alias gamificados.
- **Supremacía:** `.claude/skills/base/SKILL.md` gobierna a todos los skills; la instrucción explícita del usuario gana siempre.
