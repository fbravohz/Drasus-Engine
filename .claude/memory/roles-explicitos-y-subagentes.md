---
name: roles-explicitos-y-subagentes
description: El usuario decide cuándo se asume un rol/skill y cuándo se despacha un subagente; no auto-asumir roles.
metadata: 
  node_type: memory
  type: feedback
  originSessionId: cd158787-0796-4910-892c-ca56cc35eebe
---

El usuario controla explícitamente la orquestación de roles. NO debo auto-asumir un rol/skill (architect, flutter-engineer, etc.) por iniciativa propia ni quedarme en él de más; solo cuando él lo pide. Para trabajo de implementación pesado prefiere que **despache un subagente** en lugar de ejecutarlo yo en la conversación principal, y lo pide de forma explícita.

**Why:** mantiene la conversación principal en el rol que él eligió (p. ej. Architect, que es diseño/orquestación, pasivo) y aísla la implementación en una ventana de contexto separada.

**How to apply:** quédate en el rol pedido; cuando pida cambios de código/implementación, invoca un subagente (Sonnet por defecto — opus solo con su autorización, [[politica-de-pruebas-y-validacion]]) con un brief autocontenido. No cambies de skill sin que lo pida.
