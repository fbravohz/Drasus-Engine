# Vocabulario de Tags — `.agents/`

- Vocabulario controlado y append-only del campo `Tags:` para `.agents/knowledge/*.md` y `.agents/skills/*/SKILL.md` — eje único: tema. El eje dominio/módulo ya lo cubre [[docs/README.md]] (columna "Consumido por"), un tag de dominio aquí sería redundante.
- SLUG-ID: `TAGS-VOCABULARY`
- Tamaño: `1270 caracteres (~318 tokens aprox., ratio 4:1)`

## TAGS-VOCABULARY-1: Vocabulario vigente

- **ponytail:** simplificación deliberada marcada en comentario — [[.agents/knowledge/ponytail.md]].
- **modos-acompanamiento:** modos de acompañamiento del agente (mentor/revisión/docente) — [[.agents/knowledge/mentorship-modes.md]].
- **reporting:** cómo reportar estado o cierre de sesión al propietario — [[.agents/knowledge/session-reporting.md]].
- **memoria:** persistencia y memoria entre sesiones — [[.agents/knowledge/memory-policy.md]].

## TAGS-VOCABULARY-2: Alta de un tag nuevo

- `grep` contra este archivo antes de usar un tag no listado — evita sinónimo duplicado de uno ya vigente.
- Si no existe, se propone al dueño (Context Engineer) antes de usarlo en un documento — nunca se inventa ad hoc durante trabajo normal.
- Alta = bullet nuevo en `TAGS-VOCABULARY-1`, append-only — un tag publicado nunca se renombra ni se retira.
