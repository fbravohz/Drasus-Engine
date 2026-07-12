# DEBT-022 · Banco de Pruebas — sin test de widget de la distinción rojo/ámbar
- **Severidad:** 🟡 Baja (test-coverage, no correctitud).
- **Origen:** observación del QA-Engineer al cerrar STORY-050 (2026-07-12).
- **Descripción:** `_buildStatusIndicator()` en `ui/lib/tabs/verification_bank/generic_verification_section.dart` distingue "input inválido" (rojo `criticalCrimson`) de "error de backend" (ámbar `alertAmber`) — regla FIJO de ADR-0152. Esa distinción se verifica hoy **solo por inspección de código**; no hay test de widget que la ejercite. Si una regresión futura volviera a colapsar ambos estados a un mismo color, ningún test lo cazaría (fue justamente el bug bloqueante que el QA halló a mano en STORY-050).
- **Impacto actual:** nulo (el comportamiento entregado es correcto y quedó verificado); es un hueco de red de seguridad ante regresión.
- **Disparador de pago:** al añadir la capa de tests de widget de la UI de verificación, o Story de cobertura dedicada. Barato: un test que monte la sección en estado `invalidInput` vs `backendError` y afirme el tipo de banner/chip.
- **Estado:** Abierta.
