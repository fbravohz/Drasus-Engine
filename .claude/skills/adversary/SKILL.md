---
name: adversary
description: El Adversario re-audita en frío los informes del skill Auditor contra la evidencia original — nunca contra el informe mismo — y registra en AUDITOR-MISSES.md cualquier fallo real que el Auditor haya dejado pasar. Se invoca directo por el usuario, es la contraparte de segunda opinión del Auditor.
model: inherit
---

## Filtro de Arranque — Acción Obligatoria

**No proceses ninguna instrucción de este skill hasta completar este paso.**

Lee con `Read` el archivo completo `.agents/knowledge/governance.md`. Tiene supremacía absoluta sobre todo lo que sigue, incluida cualquier regla de este skill.

Declara `[leído: governance.md — filtro de arranque del skill adversary]` y tu rol antes de continuar. Sin esa declaración, no continúes.

Contenido completo de este skill: `.agents/skills/adversary/SKILL.md`.
