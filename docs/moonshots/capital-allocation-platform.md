# Capital Allocation Platform

**Carpeta:** `./moonshots/`
**Estado:** Moonshot — ⚠️ **ZIZAÑA** (deja de ser software; carga regulatoria de gestor)
**Última actualización:** 2026-07-03
**Decisión Arquitectónica Asociada:** ADR-0144 (clasificación de modelos)

---

## ¿Qué es?

La idea (del documento-semilla): Drasus no solo ejecuta, sino que **conecta capital de inversores con estrategias certificadas**, cobrando management fee (1–2% sobre AUM) + un cut del performance fee. Referentes: Darwinex, Collective2, Numerai.

## ⚠️ Por qué es zizaña (no va al núcleo)

- **Ya no es software:** cobrar sobre AUM y gestionar/asignar capital de terceros convierte a Drasus en **asesor de inversión / gestor regulado** (SEC/CNBV/FCA). Es una entidad financiera con licencias, custodia, auditoría y responsabilidad fiduciaria — no una feature de un motor.
- **Ciclo largo y confianza:** requiere track record público, reputación y marco legal que no existen en greenfield pre-release.
- **Cambia la valuación pero también el riesgo:** es el "moonshot más grande" del documento precisamente porque deja de ser un negocio de tecnología.

## Qué SÍ se construye ahora (y habilita esto después)

El cimiento `institutional-report-engine` + `backtest-certification` (moonshot `institutional-report-products`) produce la **certificación** de estrategias que una plataforma de allocation necesitaría. Es decir: Drasus puede ser el **auditor tecnológico** de un ecosistema de allocation sin ser el gestor. La plataforma de allocation completa queda archivada como moonshot condicionado a estructura legal.

## Dependencias

- Requeriría: entidad financiera regulada, custodia de capital, marco fiduciario. Fuera del alcance del motor.
