# Reportes de Sesión

Un archivo por sesión de trabajo, `YYYY-MM-DD-<slug>.md`. Contiene el texto que el agente escribiría en el chat: qué se pidió, qué se hizo, qué archivo cambió y por qué, qué quedó pendiente.

**No es documentación de dominio.** La orden de trabajo (`docs/execution/`), la bitácora (`.agents/state/`) y las fichas de deuda (`docs/debt/`) se siguen escribiendo igual — el reporte narra que se escribieron y repite su contenido tantas veces como haga falta para entenderse. La memoria automática (`.agents/memory/`) está desactivada desde 2026-08-07 (`autoMemoryEnabled: false`) — no se escribe más ahí.

- **Reglas de comportamiento:** `.agents/knowledge/session-reporting.md`
- **Plantilla copiable:** `.agents/templates/template-session-report.md`

Los archivos son append-only: una entrada publicada no se edita. Una rectificación se escribe como entrada nueva que nombra cuál corrige.
