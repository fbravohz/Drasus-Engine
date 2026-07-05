# STORY-030 — Usage Metering / Libro de Nocional (cimiento #4 del substrato de monetización)

| Campo | Valor |
|---|---|
| **ID** | STORY-030 |
| **Tipo** | Story (código — cuarto cimiento del substrato de pricing/SaaS) |
| **Épica (Fase)** | Cimientos de Monetización (transversal, greenfield — va antes de la auditoría retroactiva) |
| **Sprint** | substrato-monetizacion |
| **Estado** | ✅ Completada (libro append-only local; mapeo Order real, emisión feedback y UI diferidos) |
| **Creada** | 2026-07-04 |
| **Feature** | [`usage-metering`](../features/usage-metering.md) |
| **ADRs** | ADR-0144 (cimiento #4) · ADR-0143 (tiers) · ADR-0137 (puertos `Order`/`UsageRecord`) · ADR-0141 (append-only + montos ×10⁸) · ADR-0020 V2 (Perfil D) · ADR-0093 (secretos) · ADR-0142 (CLI verify) |

## 1. Objetivo llano

Construir el **libro append-only de nocional en USD**: la migración de la tabla de consumo (inmutable, por cuenta y ciclo), la lógica pura que calcula el nocional de cada operación (tamaño × precio, entero escalado) y lo acumula por ciclo detectando el cruce de umbral, y el puerto `usage_out` que produce `UsageRecord`. Es el cimiento #4 — **primer cimiento que consume un puerto REAL de otro cimiento** (el `PlanLimits` de `plan-tier-quota`/#3, ya construido, NO stub) para el veredicto de cuota.

**Alcance ahora vs. después (ADR-0144 "puerto + esquema ahora, adaptador después"):**
- **Ahora (esta Story):** esquema append-only + Core (cálculo de nocional + acumulación + cruce de umbral) + puerto `usage_out` + consumo real de `PlanLimits` de #3 + CLI verify.
- **Después (diferidos):** el mapeo del tipo `Order` real (módulo `execute`/EPIC-5, hoy un placeholder vacío) a la entrada de metering; la emisión real a `feedback`/telemetría; la UI del panel de consumo.

## 2. Agentes y Modo de Acompañamiento (ADR-0120/0122)

| Agente | Modelo | Modo |
|---|---|---|
| Rust-Engineer | Sonnet | **Docente** |

En Docente el ingeniero implementa el bloque completo y escribe la lección cero-conocimiento en `docs/lessons/rust/STORY-030-usage-metering.md` (ADR-0124).

## 3. Gate de Coherencia — resultado (contraste bidireccional)

- **Stack:** limpio (Rust + CLI).
- **Esquema APPEND-ONLY (ADR-0141) — regla obligatoria #1:** la tabla es **append-only** (un registro de consumo NUNCA se modifica; feature §Restricciones). Lleva **`event_sequence_id INTEGER NOT NULL UNIQUE`** (posición monótona en la secuencia), **NO `row_version`**, + **triggers `BEFORE UPDATE`/`BEFORE DELETE` que abortan** (patrón de `migrations/0002_audit_log.sql`). `audit_chain_hash` encadenado (NULL solo en génesis). El reinicio de ciclo NO borra: abre un `billing_cycle_id` nuevo; el histórico se conserva.
- **Nocional como entero escalado — regla obligatoria #2 (EL punto de correctitud crítico):** el nocional se persiste como **`INTEGER` ×10⁸**, NUNCA `REAL`. El cálculo `nocional = tamaño × precio` multiplica **dos** cantidades ×10⁸ → el producto crudo está en ×10¹⁶: hay que **reescalar dividiendo por 10⁸** para volver a ×10⁸, con **política de redondeo explícita y determinista** y **sin overflow** (usa `i128` para el producto intermedio; documenta el redondeo). Un error aquí factura mal. Prueba discriminante obligatoria con valores conocidos + valores grandes.
- **Consumo del puerto REAL de #3 — nota de integración:** el veredicto de cuota compara el acumulado del ciclo contra `PlanLimits.notional_limit` de `plan-tier-quota` (#3, ya construido). Consúmelo **real** vía su `public_interface::plan_tier_quota` (`resolve_limits`/catálogo), NO un stub. Este es el primer cableado real entre cimientos.
- **Tipo `Order` de entrada (placeholder):** `order_in ← Order`, pero `Order` es hoy un `pub struct Order;` vacío en `crates/shared/src/types/mod.rs` (el tipo real es del módulo `execute`/EPIC-5, no construido). Modela la **entrada mínima de metering** (tamaño, precio, instrumento) suficiente para derivar el nocional; NO inventes un `Order` completo. Deja nota de que el mapeo `Order`→entrada es futuro.
- **Perfil ADR-0020 V2:** Perfil D — Grupo I completo (con `event_sequence_id`) + II (`owner_id`, `institutional_tag`) + IV (`node_id`) + subset V (`compliance_status_id` si aplica). Campos propios marcados: nocional por operación (`INTEGER` ×10⁸), acumulado del ciclo (`INTEGER` ×10⁸), `billing_cycle_id` (`TEXT`), `instrument_id` (`TEXT`), veredicto de cuota (`TEXT` + `CHECK`).
- **Puerto (ADR-0137):** `usage_out` → `UsageRecord` (acumulado + veredicto), ya en el catálogo. Consumido por `licensing-system` (gate) y el billing futuro. Crate `crates/shared` (plomería). Confirma la ubicación leyendo el patrón ya construido; si tu lectura contradice → **párate y escálame**.
- **Restricción de dominio:** NUNCA se mide margen ni apalancamiento — solo **nocional** (feature §Restricciones, ADR-0143/0144).
- **Clasificación UI (ADR-0117) + backend-first (decisión del usuario 2026-07-04):** la feature tiene Superficie propia (panel de consumo), pero por backend-first su **SVF (Canal #1) + galería** van a la **tanda de UI final del substrato** (harness SVF genérico) — deuda **rastreada y autorizada**, NO silenciosa. Para ESTA Story el observable se verifica por **CLI (Canal #2, ADR-0142)**.
- **SAD:** SAD-22 ya cubre el substrato. Desalineamiento → escala.

## 4. Instrucciones de despacho (prompt exacto)

> Eres el **Rust-Engineer** en **Modo Docente**. Lee primero: `CLAUDE.md`, `.claude/skills/base/SKILL.md`, `.claude/skills/rust-engineer/SKILL.md`, esta Orden completa, la feature `docs/features/usage-metering.md`, las features ya construidas `docs/features/plan-tier-quota.md` (produce el `PlanLimits` que consumes) y `docs/features/licensing-system.md` (patrón), la migración `migrations/0002_audit_log.sql` (patrón append-only + triggers), y los ADR-0144, ADR-0143, ADR-0137, ADR-0141, ADR-0020 (§ADR.md perfiles), ADR-0093, ADR-0142. Declara que los leíste.
>
> **Gate de Lectura Pre-Código:** (a) confirma la ubicación del crate (`crates/shared`); (b) lee cómo `plan-tier-quota` expone `PlanLimits`/`resolve_limits` en `crates/shared/src/{domain,orchestrator,public_interface}` — lo consumes REAL; (c) lee el placeholder `pub struct Order;` en `crates/shared/src/types/mod.rs` y confirma que modelarás la entrada mínima de metering, no un `Order` completo.
>
> **Construye (libro local append-only — el mapeo de `Order` real y la emisión a feedback son futuros):**
> 1. **Migración greenfield 0010** de la tabla de consumo: **append-only** — `event_sequence_id INTEGER NOT NULL UNIQUE` (NO `row_version`) + triggers `BEFORE UPDATE`/`BEFORE DELETE` que abortan (patrón `0002_audit_log.sql`). Grupo I completo + Perfil D + `owner_id`/`institutional_tag`/`node_id`; campos propios marcados (`notional_per_op INTEGER` ×10⁸, `cycle_accumulated INTEGER` ×10⁸, `billing_cycle_id TEXT`, `instrument_id TEXT`, `quota_verdict TEXT CHECK`). `STRICT`, PK `TEXT` UUIDv7, timestamps `INTEGER` ns UTC, `audit_chain_hash` encadenado.
> 2. **Core (lógica pura, sin I/O):** (a) `compute_notional(size, price)` → `INTEGER` ×10⁸: producto en `i128` (×10¹⁶) reescalado dividiendo por 10⁸, con redondeo explícito y documentado, sin overflow; (b) `accumulate(previous_cumulative, notional)` → nuevo acumulado; (c) `detect_quota_crossing(cumulative, notional_limit)` → veredicto (dentro / cruzada). Determinismo bit-a-bit; CERO `REAL`/`f64` en cualquier cálculo de monto.
> 3. **Shell:** repositorio **append-only** (solo INSERT; nada de update/delete — los triggers lo garantizan a nivel BD); derivación del `billing_cycle_id` desde el timestamp (reloj inyectado, no `SystemTime`); reinicio de ciclo = nuevo `billing_cycle_id` sin borrar histórico; **consumo real** de `PlanLimits` de `plan-tier-quota` (#3) para el veredicto.
> 4. **`public_interface`:** el puerto `usage_out` que devuelve `UsageRecord` (acumulado del ciclo + veredicto). **Sin secretos** (ADR-0093).
> 5. **CLI `verify` (Canal #2, ADR-0142):** subcomando que, dado un conjunto de operaciones y un tier, reproduce el observable (acumulado + veredicto) en JSON, ejecutable por `cargo run -p app -- verify usage-metering --input '<json>'`.
>
> **Pruebas discriminantes (rojo→verde, deben poder fallar):**
> - **Reescalado del nocional (CRÍTICO):** `compute_notional` con valores conocidos (p. ej. tamaño 2.5 y precio $40,000.00 → nocional $100,000.00 = `10_000_000_000_000`); verifica el reescalado ×10¹⁶→×10⁸ exacto y el redondeo en el borde; valores grandes NO hacen overflow (usa `i128`). Debe fallar si multiplica sin reescalar o usa `f64`.
> - **Append-only (patrón audit_log):** un `UPDATE` y un `DELETE` sobre el libro → **rechazados por trigger** (assert de error). Debe fallar si la tabla permite mutar.
> - **`event_sequence_id` monótono y UNIQUE:** inserciones consecutivas → posiciones 1,2,3…; duplicar una posición → rechazado.
> - **Acumulación por ciclo:** varias operaciones en el mismo ciclo → acumulado = SUMA exacta. Cambio de ciclo → acumulado reinicia, filas viejas intactas.
> - **Cruce de umbral con `PlanLimits` REAL de #3:** acumulado por debajo del `notional_limit` del tier → veredicto "dentro"; al cruzarlo → "cruzada". Usa el `PlanLimits` real, no un stub. Debe fallar si el umbral se ignora.
> - **Sin `REAL`:** inspección de esquema — columnas de monto son `INTEGER`.
> - **Guardarraíl ADR-0093:** el payload de `UsageRecord` NO contiene secretos (assert explícito).
> - **`audit_chain_hash`:** encadenado entre filas; NULL solo en la fila génesis.
> - Cobertura del criterio con `cargo llvm-cov --workspace --summary-only`.
>
> **Comentarios en español, identificadores en inglés (ADR-0121).** Entrega en verde con mapeo criterio→prueba.
>
> **Docente:** escribe `docs/lessons/rust/STORY-030-usage-metering.md` (enlace a esta Orden al inicio) explicando cero-conocimiento: qué es un libro append-only y por qué NO se edita, qué es `event_sequence_id` vs `row_version` (contrasta con #1/#2/#3), por qué el nocional se reescala al multiplicar dos enteros ×10⁸ (y qué pasa con `f64`), qué es acumular por ciclo, y cómo consumes el puerto real de #3. Cita el código real.
>
> **NO hagas commits** (los hace el Tech-Lead). Al terminar reporta: archivos creados, salida de `cargo test` + `cargo llvm-cov`, salida del `cargo run -p app -- verify usage-metering`, y tu decisión de ubicación del crate con su justificación.

## 5. Criterio de aceptación (verificable)

| # | Criterio | Prueba |
|---|---|---|
| 1 | Migración `STRICT` append-only: `event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE | inspección del `.sql` + test de rechazo |
| 2 | `compute_notional` reescala ×10¹⁶→×10⁸ exacto, sin overflow, sin `f64` | test discriminante con valores conocidos + grandes |
| 3 | Acumulación por ciclo correcta; reinicio conserva histórico | test |
| 4 | Cruce de umbral usando `PlanLimits` REAL de #3 | test dentro/cruzada |
| 5 | `event_sequence_id` monótono y UNIQUE | test |
| 6 | `UsageRecord` sin secretos (ADR-0093) | test + assert |
| 7 | `audit_chain_hash` encadenado (NULL solo génesis) | test |
| 8 | CLI `verify usage-metering` devuelve el JSON correcto | `cargo run -p app -- verify usage-metering --input '…'` |
| 9 | Lección Docente escrita | existe `docs/lessons/rust/STORY-030-usage-metering.md` |
| 10 | Verde + cobertura de cada criterio | `cargo test` + `cargo llvm-cov` |

## 6. Comandos de validación (usuario)

```bash
cargo test -p shared
cargo llvm-cov --workspace --summary-only
cargo run -p app -- verify usage-metering --input '{"tier":"FREE","operations":[{"size":250000000,"price":4000000000000}]}'
```

## 7. Registro de ejecución

- 2026-07-04 · Tech-Lead · Gate corrido (contraste bidireccional). Reglas: tabla **append-only** (`event_sequence_id UNIQUE` + triggers, NO `row_version`); nocional `INTEGER` ×10⁸ con **reescalado ×10¹⁶→×10⁸** (i128, redondeo explícito, sin overflow); consumo **real** de `PlanLimits` de #3 (primer cableado entre cimientos); `Order` es placeholder → entrada mínima de metering; SVF/galería a la tanda de UI final (deuda rastreada). Orden creada, pendiente de despacho a Rust-Engineer (Sonnet, **Docente**).
- 2026-07-04 · Rust-Engineer (Docente) · Implementado EN VERDE: migración `migrations/0010_usage_metering.sql` (tabla `usage_records` STRICT, `event_sequence_id UNIQUE` + triggers `trg_usage_records_no_update`/`trg_usage_records_no_delete`, Grupo I + Perfil D acotado + subset V `compliance_status_id`); Core `crates/shared/src/domain/usage_metering.rs` (`compute_notional` con `i128` + redondeo "half up" explícito, `accumulate`, `detect_quota_crossing`, `derive_billing_cycle_id` vía `civil_from_days` sin dependencia de calendario, `compute_usage_audit_hash`); Shell `crates/shared/src/persistence/usage_metering.rs` (`UsageRepository::record_operation`, append-only sin `update`/`delete`) y `crates/shared/src/orchestrator/usage_metering.rs` (`record_metered_operation`, consumo REAL de `build_plan_limits_for_tier` de `plan-tier-quota` #3); puerto `usage_out` expuesto en `public_interface.rs` (submódulo `usage_metering` + re-exports planos) con harness `verify_usage_metering` cableado en `crates/app/src/main.rs` (`cargo run -p app -- verify usage-metering`). 35 pruebas nuevas, las 255 de `shared` en verde, cobertura de línea del módulo nuevo: domain 100.00%, orchestrator 100.00%, persistence 100.00%. Lección Docente en `docs/lessons/rust/STORY-030-usage-metering.md`. Pendiente (diferido, fuera de esta Story): mapeo de `Order` real, emisión a `feedback`/telemetría, SVF + galería.
- 2026-07-04 · Tech-Lead · Auditoría independiente (reproducida): 255 tests / 35 de usage-metering verdes; clippy `-D warnings` limpio; FCIS 0 violaciones; esquema append-only (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE, sin `row_version`, montos `INTEGER`); `compute_notional` con producto `i128`, redondeo half-up `(raw+half)/scale`, `checked`/`try_from` anti-overflow, cero `f64`; consumo REAL de `build_plan_limits_for_tier` de #3; CLI 2.5×$40K=$100K → `CROSSED`. Verde.
- 2026-07-04 · QA-Engineer (Sonnet) · **APTO.** Lógica línea por línea + **pruebas de mutación** sobre las 4 guardas críticas: quitó el `+ half` del redondeo → cayó `compute_notional_rounds_up_at_exact_half`; neutralizó el trigger de UPDATE (`WHEN 0`) → cayó `update_is_rejected_by_trigger`; reemplazó el límite por `i64::MAX` → cayó el test de cruce con PlanLimits real; cambió `>`→`>=` en `detect_quota_crossing` → cayó el test del borde exacto. Verificó append-only por SQL crudo, `event_sequence_id` UNIQUE, acumulación/reinicio por ciclo con reloj inyectado, `audit_chain_hash` génesis, `UsageRecord` sin secretos, sin margen/apalancamiento, cero `REAL`. Mutaciones restauradas byte a byte. **Observación no bloqueante:** `record_operation` no envuelve leer-cola+acumular+INSERT en transacción → dos escrituras concurrentes podrían calcular el mismo `event_sequence_id` y una fallar limpio por `UNIQUE` (sin corrupción ni doble-cobro); es **patrón preexistente del substrato append-only** (audit_log, etc.), NO introducido por #030 → deuda rastreada en PROGRESS.
- 2026-07-04 · Tech-Lead · Gate QA cerrado con APTO. **STORY-030 completada** (libro local append-only). Feature `usage-metering` 🟡 Parcial por diseño (mapeo `Order` real + emisión a feedback + UI diferidos).

## 8. Deudas / diferidos registrados

- **Mapeo del `Order` real (módulo `execute`/EPIC-5, diferido):** hoy `Order` es un placeholder vacío; se modela la entrada mínima de metering. Cuando `execute` construya el `Order` real, se mapea a esta entrada.
- **Emisión a `feedback`/telemetría (diferida):** el acumulado y los cruces de umbral se emitirán a `feedback` y a la telemetría cuando esos consumidores estén cableados.
- **SVF (Canal #1) + galería con mocks:** en la tanda de UI final del substrato (harness SVF genérico). Deuda rastreada y autorizada (backend-first).
- **UI del panel de consumo (Superficie propia, diferida):** parte del adaptador. Verificación de esta Story vía CLI Canal #2.
