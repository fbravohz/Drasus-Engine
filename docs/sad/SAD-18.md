## 18. Plan de Lanzamiento (Rollout Strategy v3.0)

El orden de implementación —qué módulo se construye y cuándo— es competencia **exclusiva** del **[`ROADMAP.md`](../ROADMAP.md)**, fuente única por ADR-0118. Esta sección **no** duplica ese orden (hacerlo provocó la divergencia que la v2.0 arrastraba); fija solo los **principios de arquitectura** que gobiernan cualquier despliegue, independientes de la fase.

### Principios de despliegue (FIJOS)

- **Unidad de entrega = módulo completo (ADR-0118):** cada fase libera el 100% del núcleo de su módulo, no una selección P0/P1 dispersa entre fases. Una Feature se construye una sola vez, en el primer módulo que la consume; los módulos posteriores solo la integran (TTR de Integración). El criterio Alpha-First ordena los módulos y justifica los splits por dependencia dura; no fragmenta el núcleo. El estado y los resultados viven en [`docs/execution/`](../execution/) y en los sellos de implementación, nunca en el plan.
- **Entrega progresiva de UI (ADR-0117):** la interfaz no se acumula para el final. Cada Feature con superficie UI declarada entrega su Cáscara Delgada (Techo Fijo) en el Panel Operativo Fundacional dentro de la misma Story que su backend. EPIC-8 deja de "construir la UI desde cero" y pasa a unificar en el Dashboard + Canvas [Forge/Reactor] (ADR-0136) y pulir.
- **Separación cómputo/visualización (ADR-0033):** la UI Flutter es 100% State-Driven y puede desconectarse sin que el motor Rust detenga su ciclo (backtest o live). Esto habilita los tres modos del despliegue trimodal: LocalPowerUser (FFI, default), VpsMonolithic (sin GPU, shaders apagados) y SaaSCloudEngine (daemon headless + UI remota por gRPC).
- **Determinismo y reproducibilidad (ADR-0005 / ADR-0020 V2):** el versionado y los 25 campos de fundación nacen en EPIC-0/EPIC-2; retrofitearlos cuesta 10x. Toda entrega debe poder reproducirse bit-a-bit con los comandos de validación de su Orden de Trabajo.

### Validación por fase

Cada fase tiene su criterio de salida en el ROADMAP y su gate de QA documentado en [`docs/execution/`](../execution/). Una fase no se considera "terminada" mientras su módulo tenga TTRs de construcción del núcleo pendientes sin una dependencia dura documentada (ADR-0118).

---
