---
name: feedback-commit-cadence
description: "El usuario eligió 'Autorizar cada commit' (2026-07-05): el TL propone los commits agrupados por tipo y ESPERA el OK explícito del turno actual — nunca auto-commitea por cimiento."
metadata:
  node_type: memory
  type: feedback
  originSessionId: 121318b9-21ba-4d1e-a76b-a3648b955c7b
---

Ante la disyuntiva auto-commit-por-cimiento vs. autorizar-cada-commit, el usuario eligió explícitamente **"Autorizar cada commit"** (2026-07-05).

**Why:** control humano sobre el historial; el usuario quiere ver y aprobar cada grupo antes de que entre a git. Refuerza la regla de CLAUDE.md §1 ("nunca commitees en automático sin que el usuario lo pida explícitamente en el turno actual").

**How to apply:** al cerrar un cimiento/Story, prepara los commits **agrupados por tipo** (`feat`/`docs`/`chore`/`fix`/`test`, nunca un commit masivo) y **propónlos**; espera el OK explícito del turno actual antes de ejecutar `git commit`. Una autorización de un turno no se extiende al siguiente. Enlaza con [[roles-explicitos-y-subagentes]].
