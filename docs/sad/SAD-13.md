## 13. Estándares de Implementación (Gobernanza)

Para mantener la integridad del monolito modular, se aplican los siguientes estándares obligatorios:

*   **Contratos Inyectables:** Todo acceso a infraestructura (DB, tiempo, red) se realiza a través de interfaces (Ports).
*   **Evolución Incremental:** Las nuevas funcionalidades no crean tareas paralelas, sino que refinan los contratos y TTRs existentes. Ver **ADR-0014**.
*   **Causalidad Obligatoria:** Todo módulo debe emitir evidencia para el módulo de Feedback (Consumidor Maestro). Ver **ADR-0015**.
*   **Local-First Processing:** El cómputo pesado reside en la infraestructura del usuario; el cloud es solo un overlay de soporte (Auth, Flags, P2P). Ver **ADR-0016**.
*   **Fidelidad Extrema:** La simulación debe replicar la fricción institucional (4-ticks, triple swap, límite de Pardo). Ver **ADR-0017**.
*   **Cero Lógica en el Shell (Soberanía de Features):** Los módulos son orquestadores puros (Thin Shell); toda lógica algorítmica reside en Features reutilizables.

---

