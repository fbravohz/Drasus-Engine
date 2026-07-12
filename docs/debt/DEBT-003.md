# DEBT-003 · Gaps de backend de `sovereign-data-fetcher`
- **Severidad:** 🟡 Baja
- **Origen:** STORY-024.
- **Descripción:** (G1) no existe submit de job en background → la SVF usa await/spinner; (G2) `sovereign_download_records` (migr. 0006) no guarda `symbol`/`bytes_total`/`status`; (G3) el estado `retrying` no existe en `JobState`.
- **Impacto actual:** limita la SVF del fetcher, no la correctitud del backend.
- **Disparador de pago:** editar el baseline SQL es barato en greenfield → se pagan en la auditoría retroactiva o al cablear el job en background.
- **Estado:** Abierta.
