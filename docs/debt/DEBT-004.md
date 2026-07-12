# DEBT-004 · Nodo Canvas por-feature del `sovereign-data-fetcher` no construido
- **Severidad:** 🟡 Baja
- **Origen:** STORY-024 (Opción B, manifestación #3). **Causa raíz corregida por la auditoría retroactiva (2026-07-10, Lote 5).**
- **Descripción:** la causa original ("la infra Canvas no existe") **ya no es cierta**: la auditoría confirmó que la **infraestructura genérica de Canvas SÍ existe** (`ui/lib/tabs/canvas_tab.dart`: drag-drop, nodos, breadcrumb). Lo que falta es el **nodo específico por-feature + inspector panel** del fetcher (y del resto de features con superficie).
- **Disparador de pago:** definido el contrato del nodo por-feature por el Architect (ver TASK-049 §E5, reconciliación ADR-0117/0136). Entonces toda feature entrega su nodo Canvas.
- **Estado:** Abierta (alcance reformulado).
