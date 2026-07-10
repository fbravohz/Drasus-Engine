---
name: roadmap-metodologia-modulo-completo
description: ADR-0118 — la unidad de entrega es el módulo completo; el ROADMAP es guía sin bitácora; las features se construyen en su primer consumidor.
metadata: 
  node_type: memory
  type: project
  originSessionId: 27d40f16-05ac-4dd4-baf0-cd220271bf50
---

Decisión de metodología (ADR-0118, 2026-06-16), tras detectar que el ROADMAP era un documento vivo ilegible y que la fragmentación P0/P1 por fase rompía módulos (caso STORY-006/crash-recovery puesto en Fundación sin soporte de diseño).

**Las tres reglas:**
1. **Unidad de entrega = módulo completo.** Cada fase libera el 100% del núcleo de su módulo, no una selección de TTRs. La fuente de verdad del alcance es la tabla "TTRs Etiquetados por Fase" de cada `docs/modules/<módulo>.md`.
2. **Construcción en el primer consumidor.** Una feature se construye una vez, en el primer módulo que la usa según el orden del pipeline; los módulos posteriores solo la integran (TTR de Integración, no re-build). Splits permitidos solo con dependencia dura documentada (`validate` núcleo/guantelete, `execute` bridge/nativo, `order-flow-microstructure` histórico/vivo, `fit-to-portfolio-search`).
3. **ROADMAP = guía sin bitácora.** Solo lleva estado simple (pendiente/en curso/terminado). El "cómo" y los resultados viven en `docs/execution/` (Órdenes de Trabajo) y en los sellos de implementación de cada documento fuente. `ROADMAP.md` quedó en v3.0 con esa estructura.

**Alpha-First sigue válido:** gobierna el ORDEN de los módulos y justifica los splits, ya no selecciona piezas dentro de un módulo. La vanidad (UI→EPIC-8 por ADR-0117; R&D→moonshots por ADR-0103) está externalizada del núcleo, así que "módulo completo" no obliga a construir adornos antes del dinero.

**Why:** la fragmentación dejaba módulos a fracciones de su diseño (ej. `ingest` 13/21 TTRs), rompía la reusabilidad real de features y causaba choques de secuenciación.

**How to apply:** al planear una fase, entrega el módulo entero; al diferir algo, que sea solo integración o vanidad ya externalizada, nunca construcción del núcleo. No escribas estado/bitácora en el ROADMAP.

Corolario de saneamiento aplicado: `crash-recovery` citaba mal ADR-0088 (es Incubación); el ancla correcta es ADR-0027 (Event Sourcing). Ver [[adr-0020-contrato-logico]] para el otro gran contrato de fundación.
