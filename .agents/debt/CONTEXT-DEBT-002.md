### CONTEXT-DEBT-002 · `debt-management.md` referencia una lectura que `memory-policy.md` prohíbe

- **Severidad:** 🔴 Alta
- **Origen:** Auditoría ad hoc del commit `972bab5f31d64cbdd28e70aee8962cd0198cd36b`, 2026-08-20.
- **Descripción:** `DEBT-MANAGEMENT-5` cierra con `**Lectura bajo demanda:** ... [[.agents/memory/comentarios-ponytail-reconciliacion.md]] — si necesitas el historial de reconciliación`, instruyendo a leer `.agents/memory/` como fuente activa. `MEMORY-POLICY-14.3` (reescrito en el mismo commit, con precedencia declarada sobre todo el documento) fija: "no se lee `.agents/memory/` para fundamentar una decisión". `DEBT-MANAGEMENT-8.4`, en el mismo archivo, sí está alineado ("Escribir en `.agents/memory/` está prohibido"), lo que hace más visible la inconsistencia de `-5`.
- **Impacto actual:** un agente que siga `debt-management.md` al pie de la letra tiene una ruta escrita para violar `MEMORY-POLICY-14.3` — contradicción activa entre dos documentos de lectura frecuente.
- **Causa raíz:** el refactor que introdujo la suspensión de memoria (`memory-policy.md`) no barrió las referencias salientes hacia `.agents/memory/` en otros documentos del mismo commit.
- **Disparador de pago:** inmediato — es contradicción de gobernanza vigente, no una condición futura. Se registra como ficha (en vez de corregirse en el acto) por decisión explícita del propietario de auditar primero y aplicar en un pase posterior.
- **Plan:** quitar el puntero de `DEBT-MANAGEMENT-5` o marcarlo explícitamente como histórico/suspendido, igual que `MEMORY-POLICY-14` hace consigo mismo.
- **Estado:** ✅ Pagada. El puntero de `DEBT-MANAGEMENT-5` quedó marcado explícitamente como histórico con lectura suspendida (`MEMORY-POLICY-14.3`), mismo tratamiento que `MEMORY-POLICY-14` se aplica a sí mismo. La contradicción de gobernanza ya no existe.
