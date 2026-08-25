### AGENT-DEBT-004 · 5 knowledge docs nuevos huérfanos — nadie los cita, la copia vieja sigue siendo la que se lee

- **Severidad:** 🟡 Baja
- **Origen:** Auditoría ad hoc del commit `972bab5f31d64cbdd28e70aee8962cd0198cd36b`, 2026-08-20.
- **Descripción:** `coherence-sweep.md`, `cross-platform-rust.md`, `execution-vocabulary.md`, `test-discrimination.md` y `ui-verification-model.md` son documentos nuevos, bien formateados y correctamente indexados en `.agents/knowledge/README.md`, pero ningún `SKILL.md` los cita por wikilink. El contenido que deberían reemplazar sigue duplicado inline dentro de `tech-lead/SKILL.md`, `architect/SKILL.md`, `qa-engineer/SKILL.md`, `rust-engineer/SKILL.md`, `ui-designer/SKILL.md` y `flutter-engineer/SKILL.md` (el caso más grave es `ui-verification-model.md`, triplicado en 3 skills distintos). La copia vieja retiene además lenguaje de bitácora ("STORY-008/009... detectado 2026-06-20") que el doc nuevo correctamente omite.
- **Impacto actual:** nulo hoy — ambas copias dicen lo mismo. El riesgo es de desincronización futura: una edición al doc nuevo no se propaga a los 6+ puntos donde el texto sigue duplicado inline.
- **Causa raíz:** el refactor extrajo el contenido a un documento canónico pero no completó el segundo paso (reemplazar el bloque inline del skill de origen por un wikilink).
- **Disparador de pago:** antes de la próxima edición de cualquiera de los 5 temas, o en un pase dedicado de `context-engineer`.
- **Plan:** `context-engineer` reemplaza cada bloque inline duplicado por un wikilink al doc canónico correspondiente, en los 6 skills señalados.
- **Estado:** Abierta.
