# DEBT-021 · Deflación de métricas compuestas (PBO/CSCV) — política diferida
- **Severidad:** 🟡 Baja (cobertura de puerta, no correctitud).
- **Origen:** corrección del modelo DSR (ADR-0151, 2026-07-11).
- **Descripción:** el DSR es específico del Sharpe; para selección por **fitness compuesta / Ret-DD** (matriz #2/#10 de ADR-0151) la puerta correcta es **PBO/CSCV** (ADR-0063), que necesita la distribución del estadístico compuesto, no la varianza del Sharpe. Cablear la deflación de compuestas vía PBO en los puntos de decisión correspondientes queda diferido.
- **Disparador de pago:** primera Story de `generate`/`validate` que seleccione por fitness compuesta (EPIC-3/4).
- **Estado:** Abierta.
