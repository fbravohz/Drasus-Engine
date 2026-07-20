---
name: context-engineer-rename-y-campo-tamano
description: Prompt Engineer se renombró a Context Engineer (2026-07-19) con alcance ampliado a diseño de procesos/arquitectura de contexto; KNOWLEDGE.md ganó el campo Tamaño (caracteres + estimado de tokens) bajo el Slug ID.
metadata: 
  node_type: memory
  type: project
  originSessionId: 9048bc2f-606b-4389-a5cf-c5dabe692f1c
  modified: 2026-07-20T04:40:29.864Z
---

El skill `prompt-engineer` se renombró a `context-engineer` (slug `PROMPT_ENGINEER` → `CONTEXT-ENGINEER`; directorios `.claude/skills/` y `.agents/skills/` movidos con `git mv`). Se invoca ahora con `/context-engineer`.

**Por qué:** el rol ya cubría más que redactar prompts — refactorizar knowledge docs, diseñar procesos para agentes y estandarizar fuentes de verdad — que es lo que la industria llama "context engineering". El usuario pidió opinión honesta antes de decidir y confirmó el cambio.

**Alcance ampliado:** además de refactorizar redacción existente, ahora también diseña/crea procesos y arquitectura de archivos de contexto nuevos cuando el usuario lo pide explícitamente (`CONTEXT-ENGINEER-1.1`). La restricción de "prohibido inventar sin pedido" (`CONTEXT-ENGINEER-3.1`) sigue vigente para el modo refactorización.

**Campo Tamaño en KNOWLEDGE.md:** todo documento de knowledge nuevo o reescrito con esta plantilla lleva, debajo del Slug ID, `Tamaño: N caracteres (~M tokens aprox., ratio 4:1)`. Caracteres es medición real (`wc -m`); tokens es estimado declarado, nunca cifra exacta. Se recalcula en cada edición que cambie contenido — no en ediciones puramente de formato.

**Cómo aplicar:** al reescribir o crear cualquier documento de `.agents/knowledge/`, seguir la plantilla actualizada en `.agents/templates/knowledge.md`. El campo Tamaño no se aplicó retroactivamente a documentos de knowledge existentes — solo a los que se toquen de aquí en adelante.
