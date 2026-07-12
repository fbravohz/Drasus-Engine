# DEBT-009 · Placeholders de tipos del guantelete en #7 (`BacktestResult`/`RobustnessScore`)
- **Severidad:** 🟡 Baja
- **Origen:** STORY-034 (`institutional-report-engine`).
- **Descripción:** `institutional-report-engine` (#7) consume `BacktestResult`/`RobustnessScore` que hoy son **placeholders** (`pub struct X;` en `types/mod.rs`); el reporte se arma con un input mínimo (`metrics: BTreeMap<String,i64>`). La firma es reproducible y correcta, pero el mapeo desde los tipos **reales** del guantelete de validación/ejecución no existe todavía porque esos tipos aún no están construidos.
- **Impacto actual:** ninguno de correctitud — el puerto y la firma son estables; es un mapeo pendiente, no un bug.
- **Disparador de pago:** cuando el guantelete produzca los tipos reales (EPIC de validación/ejecución), mapear `result_in` → `metrics` sin tocar la firma.
- **Estado:** Abierta.
