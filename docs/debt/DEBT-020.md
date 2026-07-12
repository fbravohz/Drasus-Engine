# DEBT-020 · `N_eff` por clustering no supervisado (ONC) — política diferida sobre el sketch de Sharpe
- **Severidad:** 🟡 Baja (refinamiento, no correctitud) — el DSR canónico ya funciona sin esto.
- **Origen:** corrección del modelo DSR (ADR-0151, 2026-07-11). El Architect decidió persistir el **sketch del vector de Sharpe** en el `expedition-ledger` desde el arranque (primitivo), pero **diferir la política que lo consume**.
- **Descripción:** el término `V[{SR_n}]` (Welford) basta para el DSR canónico y absorbe la correlación. El refinamiento `N_eff` por clustering no supervisado (ONC, López de Prado 2019) —reducir el número efectivo de ensayos independientes agrupando ensayos correlacionados— queda **diferido**: requiere consumir el sketch/estructura de correlación, no solo el escalar σ².
- **Disparador de pago:** cuando la minería masiva (EPIC-3) muestre que el castigo por `V` sola es insuficiente, o Story dedicada de `dsr-tracking-engine`/`expedition-ledger`.
- **Estado:** Abierta.
