# DEBT.md — Registro Canónico de Deuda Técnica Rastreada

> **Propósito:** el **único lugar descubrible** donde vive la lista de deudas técnicas conocidas y **deliberadamente aplazadas**. Una deuda rastreada es lo opuesto a deuda oculta: está escrita con su causa, su impacto y su disparador de pago. En greenfield es sano — permite avanzar en el camino crítico sin frenar por cosas que aún no muerden, **siempre que** queden registradas aquí.
> **Relación con `PROGRESS.md`:** la bitácora del Tech-Lead (`.claude/state/tech-lead/PROGRESS.md`) *narra* cuándo se halló una deuda; **este archivo es el registro canónico**. Las entradas de PROGRESS apuntan a los `DEBT-XXX` de aquí. Si una deuda no está aquí, no está rastreada.
> **Mantenimiento:** el Tech-Lead añade/cierra filas al abrir o pagar una deuda. Estado: `Abierta` · `En pago` · `Pagada` (con enlace a la Story que la saldó).

## Convención de severidad

| Severidad | Significado |
|---|---|
| 🔴 Alta | Puede corromper datos o violar un invariante bajo condiciones alcanzables; pagar pronto. |
| 🟠 Media | Fallo seguro (sin corrupción) pero con pérdida de función o correctitud bajo condiciones aún no presentes; pagar antes de que la condición llegue. |
| 🟡 Baja | Cosmético / diferido por decisión de secuenciación; sin riesgo de correctitud. |

---

## Deudas abiertas

### DEBT-001 · Ledgers append-only sin transacción atómica ni reintento
- **Severidad:** 🟠 Media
- **Origen:** observación de QA en STORY-030 (`usage-metering`); patrón preexistente desde `audit_log` (EPIC-0).
- **Descripción:** los ledgers append-only asignan `event_sequence_id` con `SELECT MAX(...)+1` e `INSERT` en **sentencias separadas**, sin envolverlas en una transacción `BEGIN IMMEDIATE`. Bajo escritura concurrente, dos escritores pueden derivar el mismo `event_sequence_id`; el `UNIQUE` rechaza a uno (fallo seguro) pero **el evento perdedor se pierde** si no hay reintento.
- **Impacto actual:** nulo — SQLite serializa escritores a nivel de archivo, el motor es local/monoproceso, y los tests corren monohilo sobre `:memory:` (nunca ejercen concurrencia). Se vuelve real con jobs concurrentes (`async-job-executor`, ejecución de varias estrategias).
- **Causa raíz (instrucciones):** el skill `rust-engineer` exigía los invariantes de tamper-evidence (UNIQUE, triggers, hash chain) pero **no** exigía (a) atomicidad transaccional en *read-then-write*, ni (b) prueba de 2 escritores. Vacío de plantilla, no descuido del agente.
- **Disparador de pago / plan:**
  1. Regla permanente en skills `rust-engineer` + `qa-engineer` (transacción `BEGIN IMMEDIATE` + `busy_timeout` + reintento acotado; prueba de 2 escritores obligatoria en todo ledger). → **hecho 2026-07-04**.
  2. `consent-registry` (#5) nace correcto (arreglado en STORY-031 antes de cerrar).
  3. **STORY-032 de endurecimiento** para los ledgers ya commiteados (`audit_log` #0002, `usage_records` #0010), con su propio QA. Recomendado: entre #5 y #6.
- **Estado:** En pago.

### DEBT-002 · `PlanLimits` duplicado (stub sellado de #2 vs. real de #3)
- **Severidad:** 🟡 Baja
- **Origen:** STORY-029 (`plan-tier-quota`).
- **Descripción:** conviven dos `PlanLimits` en namespaces distintos — el stub sellado dentro de `licensing-system` (#2) y el real de `plan-tier-quota` (#3).
- **Impacto actual:** ninguno de correctitud (namespaces separados); es deuda de unificación.
- **Disparador de pago:** mini-Story de re-cableado de #2 (toca código sellado → exige su propio QA).
- **Estado:** Abierta.

### DEBT-003 · Gaps de backend de `sovereign-data-fetcher`
- **Severidad:** 🟡 Baja
- **Origen:** STORY-024.
- **Descripción:** (G1) no existe submit de job en background → la SVF usa await/spinner; (G2) `sovereign_download_records` (migr. 0006) no guarda `symbol`/`bytes_total`/`status`; (G3) el estado `retrying` no existe en `JobState`.
- **Impacto actual:** limita la SVF del fetcher, no la correctitud del backend.
- **Disparador de pago:** editar el baseline SQL es barato en greenfield → se pagan en la auditoría retroactiva o al cablear el job en background.
- **Estado:** Abierta.

### DEBT-004 · Nodo Canvas del `sovereign-data-fetcher` no construido
- **Severidad:** 🟡 Baja
- **Origen:** STORY-024 (Opción B, manifestación #3).
- **Descripción:** el nodo Canvas + inspector lateral del fetcher no se construyó porque la infraestructura Canvas aún no existe.
- **Disparador de pago:** al construir la infra Canvas (decisión del usuario: al inicio de la auditoría retroactiva). De ahí en adelante toda feature entrega sus 3 manifestaciones. Reordena el ROADMAP → escalar al Architect (ADR-0117/0136).
- **Estado:** Abierta.

### DEBT-005 · Tanda de UI final del substrato (SVF + galería + harness genérico)
- **Severidad:** 🟡 Baja (autorizada, backend-first)
- **Origen:** decisión del usuario 2026-07-04.
- **Descripción:** los backends de los cimientos #1–#9 se verifican por CLI (Canal #2); su **SVF (Canal #1) + componentes de galería con mocks** se construyen en UNA tanda al final, que incluye: (a) el **harness SVF genérico** (una vez, obligatorio — nadie arma SVF a medida); (b) la SVF retroactiva de #1–#9; (c) el **arreglo de la SVF del `sovereign-data-fetcher`** (hoy dice "descargado" pero no muestra la respuesta del servidor).
- **Disparador de pago:** al cerrar los backends del substrato.
- **Estado:** Abierta.

### DEBT-006 · Auditoría de Inundación de Fundaciones en los 41 moonshots
- **Severidad:** 🟡 Baja
- **Origen:** cierre de la auditoría de features.
- **Descripción:** falta aplicar la misma auditoría de perfiles ADR-0020 a los 41 moonshots.
- **Disparador de pago:** TASK futura, fuera del camino crítico.
- **Estado:** Abierta.

### DEBT-007 · `OPTOUT_CHANGE` como primera acción sin guarda explícita
- **Severidad:** 🟡 Baja (falla-seguro)
- **Origen:** observación de QA en STORY-031 (`consent-registry`).
- **Descripción:** `apply_consent_action`/`try_record_action_once` no impiden explícitamente que una `OPTOUT_CHANGE` sea el **primer** evento de un `owner_id` (sin `ACCEPT` previo). Si ocurriera, `accepted_version` queda `""`.
- **Impacto actual:** inofensivo — `needs_reacceptance("", vigente)` es siempre `true` → el veredicto cae a `StaleVersion` (niega), nunca a `Covered`. Falla-seguro, no viola GDPR.
- **Disparador de pago:** añadir una guarda explícita con error tipado (en vez de depender del efecto colateral) → plegado al alcance de **STORY-032**.
- **Estado:** Abierta.

---

## Deudas pagadas

_(ninguna aún — se moverán aquí con enlace a la Story que las saldó)_
