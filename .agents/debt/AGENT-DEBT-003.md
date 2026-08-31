### AGENT-DEBT-003 · `ui-designer/SKILL.md` (worktree) — salvaguarda FFI perdida, prohibición debilitada, ADRs perdidos

- **Severidad:** 🟠 Media
- **Origen:** Auditoría ad hoc del worktree sin commitear de `.agents/`, 2026-08-20.
- **Descripción:** en el diff aún no commiteado de `ui-designer/SKILL.md` contra `HEAD` hay tres hallazgos independientes en el mismo archivo: (1) desapareció el ítem del antiguo Paso 5 que exigía verificar que las "Notas de implementación" calzaran con el binding FFI real (`ui/lib/src/rust/api/<feature>.dart`) antes de prometer polling o campos que el DTO no expone — sin reemplazo en `UI-DESIGNER-5.1`; (2) `UI-DESIGNER-7.3` cambió una prohibición absoluta ("Nunca modificas [...], solo el Architect define") por una condicional ("Nunca [...] sin visto bueno del Architect"), abriendo una vía donde el propio agente podría autoasignarse el visto bueno sin escalar de verdad; (3) la tabla de ADRs vinculantes (`UI-DESIGNER-8.1`) perdió la cita directa a ADR-0131, ADR-0116 y ADR-0038 sin sustituto.
- **Impacto actual:** el disparador se incumplió — el archivo se comiteó sin pagar la deuda, así que los tres hallazgos llegaron a `HEAD` y estuvieron vivos hasta su pago. Mientras lo estuvieron, `ui-designer` operó sin la salvaguarda de corrección funcional y con una vía de autoaprobación abierta.
- **Causa raíz:** reescritura del skill como parte del reset del sistema de diseño (ver `DEBT-046` en `docs/debt/`); el reset priorizó el vocabulario visual y no arrastró estas tres reglas, independientes del vocabulario.
- **Disparador de pago:** antes de commitear este archivo.
- **Plan:** restaurar el ítem de verificación FFI en `UI-DESIGNER-5.1`; revertir `UI-DESIGNER-7.3` a prohibición absoluta, o declarar explícitamente el mecanismo de "visto bueno" (quién lo da, cómo se registra); reponer las 3 citas de ADR en `UI-DESIGNER-8.1`.
- **Estado:** ✅ Pagada. `UI-DESIGNER-5.1` recupera la verificación del binding FFI (descrita por comportamiento exigido, ya que `ui/` fue archivado a `ui-old/`); `UI-DESIGNER-7.3` vuelve a prohibición absoluta, cerrando la vía de autoaprobación; `UI-DESIGNER-8.1` repone ADR-0116, ADR-0038 y ADR-0131 como wikilinks.

- **Lección:** un disparador "antes de comitear" no se cumple solo. Si la ficha no se revisa en el momento del commit, la deuda entra al historial y su impacto deja de ser hipotético.
