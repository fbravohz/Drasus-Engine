### CONTEXT-DEBT-012 · Cita huérfana a sub-ID de `TECH-LEAD` retirado por la migración al formato ligero
- **Severidad:** 🟡 Baja
- **Origen:** migración de `.agents/skills/tech-lead/SKILL.md` al formato ligero — el nuevo formato no admite sub-ID de bullet (`<DOC>-N.M`) ni bloque `<DOC>-FV`, así que la cita a `TECH-LEAD-7.1` quedó apuntando a un ID que ya no existe como tal (el contenido sigue vivo, fusionado en prosa dentro de `TECH-LEAD-7`).
- **Descripción:** cita detectada por `grep -rn` fuera del propio `SKILL.md`:
  - `.agents/plans/analiza-los-cambios-en-curried-oasis.md:45` → `TECH-LEAD-7.1` ("la memoria viva son ROADMAP + PROGRESS.md")
- **Impacto actual:** ninguno en gobernanza — la regla citada sigue vigente en `TECH-LEAD-7` completa; el hueco es que la cita ya no resuelve a un bloque citable exacto, solo a la Sección que lo contiene.
- **Causa raíz:** mismo patrón ya aceptado en `CONTEXT-DEBT-010`: aligerar el formato de un skill sin remapear en la misma tarea las citas externas que apuntaban a sus sub-IDs retirados.
- **Disparador de pago:** la próxima vez que `analiza-los-cambios-en-curried-oasis.md` se toque por otro motivo, corregir ahí mismo la cita para que apunte a `TECH-LEAD-7`.
- **Plan:** al tocar el archivo, reemplazar `TECH-LEAD-7.1` por `TECH-LEAD-7` o por una remisión en prosa si el punto exacto ya no es citable por separado.
- **Estado:** Abierta.
