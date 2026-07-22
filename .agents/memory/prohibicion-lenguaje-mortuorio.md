---
name: prohibicion-lenguaje-mortuorio
description: "Regla permanente del propietario — prohibido todo término del campo semántico de la muerte en nombres/documentos; se sanea, no se registra como DEBT."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 08235ffa-9cd0-4500-be6e-1a0a6f2cedae
  modified: 2026-07-21T05:12:35.777Z
---

El propietario prohíbe **todo término del campo semántico de la muerte y sus estados** (muerte, morir, difunto, cadáver, tumba, entierro, cementerio, autopsia, forense, `post-mortem`, necro-, luto, etc.) en nombres de features, módulos, nodos de canvas, ADRs, plantillas o cualquier documento — aunque "suene temático" o técnico.

**Why:** es una regla permanente que tuvo que pelear repetidamente con agentes architect previos porque el saneamiento terminológico solo listaba `Autopsia`/`Cementerio` y no el campo semántico completo. Además, tratar esto como una deuda técnica (DEBT-XXX) es el vehículo equivocado y también lo rechazó: una violación de nombres no es deuda aplazable, es saneamiento proactivo inmediato.

**How to apply:** al detectar una de estas palabras, reemplázala en el momento (igual que un alias gamificado), no abras un ticket. Reemplazos fijados: `Forense` / `Post-mortem` / `Simulación Forense` → `Repetición de Corrida` / `Reproducción` / `Revisión de Corrida`. La regla vive canonizada en la sección de Saneamiento Terminológico de `.agents/skills/architect/SKILL.md`. Aplica a todos los roles, no solo al Architect.

Relacionado: la feature de visualización (ex "Time-Warp Debugger / Simulación Forense") se renombra bajo esta regla — ver [[arquitectura-visual-canvas]].
