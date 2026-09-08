### CONTEXT-DEBT-008 · `qa-engineer/SKILL.md` en formato legacy — avisos blandos del hook

- **Severidad:** 🟡 Baja
- **Origen:** salida del hook `validate-agent-docs.py` al editar el archivo (avisos, no bloqueo).
- **Descripción:** `qa-engineer/SKILL.md` tiene `Slug ID`/`Tamaño` correctos (0 violaciones duras) pero conserva formato legacy: sin bloque `## QA-ENGINEER-FV — Filtro de Verificación`; varias rutas sin wikilink (`.agents/knowledge/governance.md` en al menos 6 puntos, `.claude/skills/tech-lead.md`); y lenguaje de bitácora en el cuerpo de dos reglas (referencias a "regla del usuario 2026-07-19" y casos con fecha en la sección de mutación/gate de mutación).
- **Impacto actual:** nulo — son avisos, no bloquean el hook ni afectan la ejecución del skill.
- **Causa raíz:** el archivo no pasó por el refactor de formato que sí recibieron otros skills (`general-counsel`, `partner`, `refactoring-engineer`) ni por la reestructuración pendiente de `CONTEXT-DEBT-001`.
- **Disparador de pago:** próxima edición de fondo de `qa-engineer/SKILL.md`, o el mismo pase que salde `CONTEXT-DEBT-001`.
- **Plan:** `context-engineer` reestructura el archivo al formato canónico (`QA-ENGINEER-0/-FV/reglas/-EJ`), agrega wikilinks a las rutas sueltas, y retira el lenguaje de bitácora de las dos reglas señaladas.
- **Estado:** ✅ Pagada — `python3 .agents/hooks/validate-agent-docs.py --audit .agents/skills/qa-engineer/SKILL.md` → `0 violaciones duras, 0 avisos` (antes: 13 avisos). Reestructurado a `QA-ENGINEER-0/-FV/-P/1..10/-EJ` (24 257 caracteres), wikilinks corregidos (incluida la ruta rota a `tech-lead.md`), lenguaje de bitácora purgado sin perder el contenido normativo (estándar de 0 supervivientes, prueba de dos escritores, patrón verde-trivial). Recibió además `QA-ENGINEER-P` (Pipeline de Ejecución).
