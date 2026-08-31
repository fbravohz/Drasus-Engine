### AGENT-DEBT-006 · IDs de ítem que reutilizan el ID del bloque padre

- **Severidad:** 🟡 Baja
- **Origen:** Auditoría ad hoc del commit `972bab5f31d64cbdd28e70aee8962cd0198cd36b`, 2026-08-20.
- **Descripción:** `commenting-policy.md` (`COMMENTING-POLICY-1 — Declaración de Lectura`) y `partner/SKILL.md` (`PARTNER-4 — Tono`) usan el ID del bloque de primer nivel como ID de la viñeta interna, en vez de `-1.1`/`-4.1` — rompe el patrón bloque→`N.1` que el resto de ambos archivos sí respeta. El hook `validate-agent-docs.py` no lo detecta (no valida colisión de ID entre nivel de bloque y de ítem).
- **Impacto actual:** nulo — cosmético; no genera ambigüedad práctica porque cada archivo solo lo hace una vez.
- **Causa raíz:** descuido de formato durante el refactor masivo.
- **Disparador de pago:** próxima edición de cualquiera de los dos archivos.
- **Plan:** renumerar el ítem a `-1.1`/`-4.1` respectivamente.
- **Estado:** ✅ Pagada en sus dos mitades. `commenting-policy.md` renumera su viñeta a `COMMENTING-POLICY-1.1` y `partner/SKILL.md` la suya a `PARTNER-4.1`. En ambos casos se verificó por `grep` que ninguna otra regla citaba el ID viejo como referencia normativa.

- **Pendiente derivado:** el hook sigue sin detectar esta colisión (bloque y viñeta con el mismo ID). Si vuelve a ocurrir, no hay quien avise — candidato a regla nueva en `validate-agent-docs.py`.
