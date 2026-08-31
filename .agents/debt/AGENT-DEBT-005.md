### AGENT-DEBT-005 · Referencias muertas a `ui/COMPONENTS.md` en 2 skills

- **Severidad:** 🟡 Baja
- **Origen:** Auditoría ad hoc del worktree sin commitear de `.agents/`, 2026-08-20.
- **Descripción:** `flutter-engineer/SKILL.md` (`FLUTTER-ENGINEER-0`, `-6.1`) y `ui-designer/SKILL.md` (Lecturas Obligatorias, `UI-DESIGNER-FV-3`, `-4.1`) referencian `[[ui/COMPONENTS.md]]`, pero `ui/` fue renombrado íntegro a `ui-old/` en este mismo árbol de trabajo (reset del sistema de diseño, ver `DEBT-046` en `docs/debt/`). El archivo no existe en la ruta citada.
- **Impacto actual:** nulo hasta que un agente intente seguir esa lectura obligatoria y falle. `ui/` arrancó vacío tras el reset, así que puede ser intencional (a la espera del `ui/` nuevo) — pero ninguno de los dos skills lo declara como transitorio.
- **Causa raíz:** el reset de `ui/` → `ui-old/` (Architect, 2026-08-14) no se propagó a estas referencias.
- **Disparador de pago:** cuando exista un `ui/COMPONENTS.md` real en el `ui/` nuevo, o antes si algún agente intenta seguir esa lectura y falla.
- **Plan:** declarar explícitamente en ambos skills que la ruta apunta al `ui/` nuevo aún por construir (nota transitoria), o apuntar temporalmente a `ui-old/COMPONENTS.md` si el catálogo viejo sigue siendo consultable como referencia histórica.
- **Estado:** ✅ Pagada. Decisión del propietario: **marcar como suspendida**, no redirigir a `ui-old/`. La fila de Lecturas Obligatorias de ambos skills declara ahora que el catálogo no existe todavía, por qué el viejo no sirve de sustituto (cita tokens que el `DESIGN.md` vigente ya no define), y qué hacer mientras tanto: tratar todo componente como "a construir" y escalar al Tech Lead. Se reactiva cuando exista el `ui/` nuevo.
