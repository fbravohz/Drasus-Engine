---
name: context-engineer-rename-y-campo-tamano
description: Prompt Engineer se renombró a Context Engineer (2026-07-19); campo Tamaño en knowledge docs; y (2026-08-03) capacidad de clasificar/crear los 5 mecanismos nativos de Claude Code (Rules/Skills/Agents/Hooks/MCP) vía CONTEXT-ENGINEER-5 + claude-code-mechanisms.md.
metadata: 
  node_type: memory
  type: project
  originSessionId: 9048bc2f-606b-4389-a5cf-c5dabe692f1c
  modified: 2026-08-04T04:16:00.747Z
---

El skill `prompt-engineer` se renombró a `context-engineer` (slug `PROMPT_ENGINEER` → `CONTEXT-ENGINEER`; directorios `.claude/skills/` y `.agents/skills/` movidos con `git mv`). Se invoca ahora con `/context-engineer`.

**Por qué:** el rol ya cubría más que redactar prompts — refactorizar knowledge docs, diseñar procesos para agentes y estandarizar fuentes de verdad — que es lo que la industria llama "context engineering". El usuario pidió opinión honesta antes de decidir y confirmó el cambio.

**Alcance ampliado:** además de refactorizar redacción existente, ahora también diseña/crea procesos y arquitectura de archivos de contexto nuevos cuando el usuario lo pide explícitamente (`CONTEXT-ENGINEER-1.1`). La restricción de "prohibido inventar sin pedido" (`CONTEXT-ENGINEER-3.1`) sigue vigente para el modo refactorización.

**Campo Tamaño en KNOWLEDGE.md:** todo documento de knowledge nuevo o reescrito con esta plantilla lleva, debajo del Slug ID, `Tamaño: N caracteres (~M tokens aprox., ratio 4:1)`. Caracteres es medición real (`wc -m`); tokens es estimado declarado, nunca cifra exacta. Se recalcula en cada edición que cambie contenido — no en ediciones puramente de formato.

**Cómo aplicar:** al reescribir o crear cualquier documento de `.agents/knowledge/`, seguir la plantilla actualizada en `.agents/templates/knowledge.md`. El campo Tamaño no se aplicó retroactivamente a documentos de knowledge existentes — solo a los que se toquen de aquí en adelante.

**Alcance ampliado 2 (2026-08-03) — mecanismos nativos de Claude Code:** el usuario pidió que Context Engineer sepa identificar cuándo un documento/instrucción de `.agents/` calza mejor como Rule, Skill, Agent, Hook o conexión MCP nativos de Claude Code (en vez de quedarse bespoke), y sepa crearlos con la sintaxis oficial correcta. Se agregó `CONTEXT-ENGINEER-5` (clasificación post-refactor + creación verificada contra doc oficial, nunca migra sin confirmar con el usuario) y el documento satélite nuevo `.agents/knowledge/claude-code-mechanisms.md` (qué es/cuándo usar/ubicación/plantilla de cada uno de los 5, más matriz de decisión `CLAUDE-CODE-MECHANISMS-6`). Criterio clave de la matriz: la base de conocimiento actual (`governance.md`, etc.) se queda en `.agents/` porque es contexto de rol/tarea, no atable a una ruta de archivo ni a un evento del ciclo de vida — las Rules nativas con `paths:` solo son candidatas cuando el contenido debe regir incluso sin ningún skill activo (caso que el gate actual de `.agents/knowledge/README.md` no cubre, porque solo dispara dentro de un skill).

**Alcance ampliado 3 (mismo día) — prioridad de Hooks + test Rule vs. Knowledge:** el usuario pidió explícitamente que Context Engineer priorice Hooks deterministas sobre Rules/Skills/knowledge docs advisory al crear o refactorizar cualquier conjunto de instrucciones — razón: un Hook no depende de que el modelo recuerde la regla, la verifica con script (formato, ID, campo `Tamaño` vs. `wc -m` real, checklist) y bloquea si falla. Se reordenó `CLAUDE-CODE-MECHANISMS-6` para que la pregunta de Hooks (`6.1`) se resuelva SIEMPRE primero — no es excluyente con los demás mecanismos, un mismo conjunto normalmente combina un Hook (parte verificable) con una Rule/Skill (parte que requiere juicio). Se agregó `CONTEXT-ENGINEER-5.3` (prioridad de Hooks) y `-5.4` (test explícito Rule-vs-knowledge-doc: ¿debe regir el contenido incluso sin ningún skill invocado? → Rule; si solo importa cuando el skill correcto está activo → knowledge doc).
