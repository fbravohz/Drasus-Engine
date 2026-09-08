### DEBT-047 · `licensing-system` cerrada sin Cáscara Visual (Etapa 0.5 nunca ejecutada)
- **Severidad:** 🟠 Media
- **Origen:** auditoría de la bitácora de `tech-lead/SKILL.md` (cotejo con `PROGRESS.md`); el cierre original (STORY-028, 2026-07-04) declaró la feature 🟡 Parcial con "UI diferidos", pero nunca pasó por Etapa 0.5 (Diseño Visual) — a diferencia de `central-identity` (STORY-027, mismo cierre), que sí tiene `## Cáscara Visual` en su feature doc.
- **Descripción:** `docs/features/licensing-system.md` no tiene sección `## Cáscara Visual (Thin Shell)` — verificado por grep contra el archivo real. Sin ella, la feature no cumple la Definición de Terminado (Superficie de Verificación Funcional, `EXECUTION-PROTOCOL-27.2`) ni puede pasar por Etapas 3-4 cuando se despache su UI.
- **Impacto actual:** limitación — no bloquea el gate local (emisor Ed25519, firma, supresión por tier ya funcionan y están probados), pero impide verificar el ciclo completo de licenciamiento desde la interfaz sin leer código.
- **Causa raíz:** cierre acelerado con solo Canal #2 (CLI `verify`) el mismo día que `central-identity`, sin despachar UI-Designer.
- **Disparador de pago:** antes de despachar el emisor central real o `plan-tier-quota` real (los dos pendientes ya declarados en el estado de la feature) — ese trabajo futuro debería absorber la Etapa 0.5 diferida en vez de repetir el mismo hueco.
- **Plan:** despachar UI-Designer para completar `## Cáscara Visual` en `docs/features/licensing-system.md`, siguiendo el mismo patrón que `central-identity.md`.
- **Estado:** Abierta.
