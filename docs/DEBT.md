# DEBT.md — Índice del Registro Canónico de Deuda Técnica

> **Propósito:** el **único lugar descubrible** donde vive la lista de deudas técnicas conocidas y **deliberadamente aplazadas**. Una deuda rastreada es lo opuesto a deuda oculta: está escrita con su causa, su impacto y su disparador de pago. En greenfield es sano — permite avanzar en el camino crítico sin frenar por cosas que aún no muerden, **siempre que** queden registradas aquí.
> **Estructura (patrón ADR):** este archivo es el **índice**; la ficha completa de cada deuda vive en `docs/debt/DEBT-XXX.md`. Para leer una deuda concreta, abre su archivo (≈10–30 líneas) — no hace falta cargar todo el registro.
> **Relación con `PROGRESS.md`:** la bitácora del Tech-Lead (`.agents/state/tech-lead/PROGRESS.md`) *narra* cuándo se halló una deuda; **este registro es el canónico**. Las entradas de PROGRESS apuntan a los `DEBT-XXX` de aquí. Si una deuda no está aquí, no está rastreada.
> **Mantenimiento:** el Tech-Lead crea/cierra la ficha en `docs/debt/` al abrir o pagar una deuda y actualiza la fila del índice. Estado: `Abierta` · `En pago` · `Pagada` (con enlace a la Story que la saldó). Al cerrar un hito se **barre** el índice y se promueve toda deuda cuyo disparador se cumplió (ver `.agents/knowledge/debt-management.md` §Regla de Promoción).

## Convención de severidad

| Severidad | Significado |
|---|---|
| 🔴 Alta | Puede corromper datos o violar un invariante bajo condiciones alcanzables; pagar pronto. |
| 🟠 Media | Fallo seguro (sin corrupción) pero con pérdida de función o correctitud bajo condiciones aún no presentes; pagar antes de que la condición llegue. |
| 🟡 Baja | Cosmético / diferido por decisión de secuenciación; sin riesgo de correctitud. |

---

## Índice

| ID | Severidad | Resumen | Estado | Ficha |
|---|---|---|---|---|
| DEBT-001 | 🟠 Media | Ledgers append-only sin transacción atómica ni reintento | ✅ Pagada (STORY-032) | [detalle](./debt/DEBT-001.md) |
| DEBT-002 | 🟡 Baja | `PlanLimits` duplicado (stub de #2 vs. real de #3) | Abierta | [detalle](./debt/DEBT-002.md) |
| DEBT-003 | 🟡 Baja | Gaps de backend de `sovereign-data-fetcher` (G1/G2/G3) | Abierta | [detalle](./debt/DEBT-003.md) |
| DEBT-004 | 🟡 Baja | Nodo Canvas por-feature del fetcher no construido | Abierta | [detalle](./debt/DEBT-004.md) |
| DEBT-005 | 🟠 Media | Tanda de UI del substrato (SVF + harness genérico) — gap de DoD | ✅ Pagada (STORY-050) | [detalle](./debt/DEBT-005.md) |
| DEBT-006 | 🟡 Baja | Auditoría de Inundación de Fundaciones en los 41 moonshots | Abierta | [detalle](./debt/DEBT-006.md) |
| DEBT-007 | 🟡 Baja | `OPTOUT_CHANGE`-primera sin guarda explícita (#5) | ✅ Pagada (STORY-032) | [detalle](./debt/DEBT-007.md) |
| DEBT-008 | 🟡 Baja | `enriched-domain-events` (#6) sin fan-out al bus (ADR-0085) | Abierta | [detalle](./debt/DEBT-008.md) |
| DEBT-009 | 🟡 Baja | Placeholders de tipos del guantelete en #7 | Abierta | [detalle](./debt/DEBT-009.md) |
| DEBT-010 | 🟡 Baja | Render Tera→PDF/HTML no cableado en #7 | Abierta | [detalle](./debt/DEBT-010.md) |
| DEBT-011 | 🟡 Baja | Huecos de cobertura en `third-party-api-gateway` (#8) | Abierta | [detalle](./debt/DEBT-011.md) |
| DEBT-012 | 🟡 Baja | Huecos de cobertura en `data-aggregation` (#9) | Abierta | [detalle](./debt/DEBT-012.md) |
| DEBT-013 | 🟡 Baja | Huecos de cobertura en `verified-account-registry` (#10) | Abierta | [detalle](./debt/DEBT-013.md) |
| DEBT-014 | 🟠 Media | Retrabajo de #10 — faltaba el Eje B (realidad de capital) | ✅ Pagada (STORY-038) | [detalle](./debt/DEBT-014.md) |
| DEBT-015 | 🟡 Media | #11 `canonical_delta_bytes` sin test de valor-dorado | Abierta | [detalle](./debt/DEBT-015.md) |
| DEBT-016 | 🔴 Alta | #10 columna `capital_reality` duplica `institutional_tag` | ✅ Pagada (STORY-041) | [detalle](./debt/DEBT-016.md) |
| DEBT-017 | 🔶 Media | #3 falta la cuota `MAX_CHILD_ACCOUNTS` | ✅ Pagada (STORY-042) | [detalle](./debt/DEBT-017.md) |
| DEBT-018 | 🟠 Media | Cobertura de mutación del patrón append-only en #4–#12 | ✅ Pagada (STORY-047) | [detalle](./debt/DEBT-018.md) |
| DEBT-019 | 🟡 Baja | Cobertura de mutación de la plomería EPIC-0 (`job`/`mcp_*`) | Abierta | [detalle](./debt/DEBT-019.md) |
| DEBT-020 | 🟡 Baja | `N_eff` por clustering (ONC) — política diferida del DSR | Abierta | [detalle](./debt/DEBT-020.md) |
| DEBT-021 | 🟡 Baja | Deflación de compuestas (PBO/CSCV) — política diferida | Abierta | [detalle](./debt/DEBT-021.md) |
| DEBT-022 | 🟡 Baja | Banco de Pruebas — sin test de widget de la distinción rojo/ámbar | ✅ Pagada (2026-07-12) | [detalle](./debt/DEBT-022.md) |
| DEBT-023 | 🟠 Media | `proptest` ausente en funciones numéricas de dominio no acotado (E4/TASK-049) | Abierta | [detalle](./debt/DEBT-023.md) |
| DEBT-024 | 🟠 Media | Pipeline como proceso recurrente/en bucle — diseño cerrado, falta implementación | Abierta | [detalle](./debt/DEBT-024.md) |

---

## Deudas pagadas (resumen)

- **DEBT-001** (ledgers append-only sin transacción atómica) → [STORY-032](./execution/STORY-032-ledger-atomicity-hardening.md), 2026-07-05.
- **DEBT-007** (`OPTOUT_CHANGE`-primera sin guarda) → [STORY-032](./execution/STORY-032-ledger-atomicity-hardening.md), 2026-07-05.
- **DEBT-014** (Eje B ausente en #10) → [STORY-038](./execution/STORY-038-verified-account-capital-reality.md), 2026-07-06.
- **DEBT-016** (columna `capital_reality` duplica `institutional_tag` en #10) → [STORY-041](./execution/STORY-041-verified-account-eje-b-consolidation.md), 2026-07-07.
- **DEBT-017** (falta cuota `MAX_CHILD_ACCOUNTS` en #3) → [STORY-042](./execution/STORY-042-plan-tier-quota-max-child-accounts.md), 2026-07-07.
- **DEBT-018** (cobertura de mutación del patrón append-only en cimientos #4–#12) → [STORY-047](./execution/STORY-045-foundation-audit-remediation.md), 2026-07-10 (gate de mutación 0 `missed`).
- **DEBT-005** (tanda de UI del substrato — SVF + harness genérico, gap de DoD) → [STORY-050](./execution/STORY-050-verification-bench.md), 2026-07-12 (QA APTO; Banco de Pruebas con 15 features enchufadas; residual de cobertura → DEBT-022).
- **DEBT-022** (test de widget de la distinción rojo/ámbar del Banco) → saldada 2026-07-12 (Flutter-Engineer; `ui/test/verification_bench_status_test.dart`, 3 casos, prueba discriminante verificada por el TL).

> Las fichas de las deudas pagadas se conservan en `docs/debt/` con su historia completa y Estado ✅ Pagada.
