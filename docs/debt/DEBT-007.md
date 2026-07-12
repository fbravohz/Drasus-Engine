# DEBT-007 · `OPTOUT_CHANGE` como primera acción sin guarda explícita
- **Severidad:** 🟡 Baja (falla-seguro)
- **Origen:** observación de QA en STORY-031 (`consent-registry`).
- **Descripción:** `apply_consent_action`/`try_record_action_once` no impiden explícitamente que una `OPTOUT_CHANGE` sea el **primer** evento de un `owner_id` (sin `ACCEPT` previo). Si ocurriera, `accepted_version` queda `""`.
- **Impacto actual:** inofensivo — `needs_reacceptance("", vigente)` es siempre `true` → el veredicto cae a `StaleVersion` (niega), nunca a `Covered`. Falla-seguro, no viola GDPR.
- **Disparador de pago:** añadir una guarda explícita con error tipado (en vez de depender del efecto colateral) → plegado al alcance de **STORY-032**.
- **Estado:** ✅ **Pagada** — [STORY-032](../execution/STORY-032-ledger-atomicity-hardening.md) (2026-07-05): guarda tipada `ConsentRepositoryError::OptoutBeforeAccept` que rechaza `OPTOUT_CHANGE` como primer evento antes de fusionar/persistir.
