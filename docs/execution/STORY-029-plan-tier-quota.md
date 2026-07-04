# STORY-029 — Plan / Tier / Quota (cimiento #3 del substrato de monetización)

| Campo | Valor |
|---|---|
| **ID** | STORY-029 |
| **Tipo** | Story (código — tercer cimiento del substrato de pricing/SaaS) |
| **Épica (Fase)** | Cimientos de Monetización (transversal, greenfield — va antes de la auditoría retroactiva) |
| **Sprint** | substrato-monetizacion |
| **Estado** | ✅ Completada (catálogo local; sincronización central, re-cableado #2 y UI diferidos) |
| **Creada** | 2026-07-04 |
| **Feature** | [`plan-tier-quota`](../features/plan-tier-quota.md) |
| **ADRs** | ADR-0144 (cimiento #3) · ADR-0143 (tiers) · ADR-0137 (puerto `PlanLimits`) · ADR-0141 (esquema/precios ×10⁸) · ADR-0020 V2 (Perfil D) · ADR-0008 (configurabilidad) · ADR-0093 (secretos) · ADR-0142 (CLI verify) |

## 1. Objetivo llano

Construir el **catálogo configurable de planes** y su resolvedor de límites: la migración de la tabla de planes, la lógica pura que valida un plan (tier + cuotas + precio coherentes) y resuelve "¿qué límites aplican a esta licencia?", y el puerto `plan_limits_out` que produce `PlanLimits`. Es el cimiento #3 — produce justo el tipo que `licensing-system` (#2) hoy consume por **stub** y que `usage-metering` (#4) necesitará.

**Alcance ahora vs. después (ADR-0144 "puerto + esquema ahora, adaptador después"):**
- **Ahora (esta Story):** esquema + Core (validación de plan + resolución de límites) + puerto `plan_limits_out` + caché local + un **stub local** que siembra un catálogo de planes de desarrollo (los planes reales los define la **Cabina de Mando**, que aún no existe) + CLI verify.
- **Después (diferidos):** la sincronización real del catálogo con la Cabina de Mando; el re-cableado del `plan_limits_in` de `licensing-system` (#2) a este puerto real; la UI de planes/precios.

## 2. Agentes y Modo de Acompañamiento (ADR-0120/0122)

| Agente | Modelo | Modo |
|---|---|---|
| Rust-Engineer | Sonnet | **Docente** |

En Docente el ingeniero implementa el bloque completo por su cuenta y además escribe la lección cero-conocimiento en `docs/lessons/rust/STORY-029-plan-tier-quota.md` (ADR-0124).

## 3. Gate de Coherencia — resultado (contraste bidireccional)

- **Stack:** limpio (Rust + CLI; sin tecnologías rechazadas).
- **Esquema (ADR-0141) — regla obligatoria #1:** la tabla de planes es **mutable** (un plan cambia límite/precio en sitio; "en la siguiente revalidación se refleja"). Lleva **`row_version`** (concurrencia optimista), NO `event_sequence_id UNIQUE`. El historial de cambios de plan va al `audit-log`, no a esta tabla.
- **Precios y nocional (ADR-0141) — regla obligatoria #2:** todo monto (precio del plan, límite de volumen nocional) se persiste como **entero escalado ×10⁸** (`INTEGER`), **NUNCA `REAL`**. Round-trip sin deriva de punto flotante.
- **Perfil ADR-0020 V2:** Perfil D — Grupo I completo (con `row_version`) + II (`owner_id` del creador del plan, `institutional_tag`) + IV (`node_id`). Campos propios fuera del catálogo, marcados: `tier` (`TEXT` + `CHECK`), `notional_limit` (`INTEGER` ×10⁸), `max_activations` (`INTEGER`), `price` (`INTEGER` ×10⁸), `pricing_model` (`TEXT` + `CHECK`), conjunto de features habilitadas (codificación determinista — lista `TEXT` ordenada o tabla hija con FK; elige y justifica bajo ADR-0141, sin `REAL`).
- **Puerto (ADR-0137):** `plan_limits_out` → `PlanLimits` (cardinalidad `1..N`), ya en el catálogo vía enmienda ADR-0144. Consumido por `licensing-system` y `usage-metering`.
- **Ubicación del crate — Gate de Lectura del ingeniero:** `PlanLimits` es **tipo técnico de plomería** (`textLabel`, ≥2 consumidores, sin puerto de Alpha en el canvas) → `crates/shared` (mismo criterio que `central-identity`/`licensing-system`). Confirma leyendo el patrón ya construido. Si tu lectura dicta lo contrario de forma clara → **párate y escálame**.
- **Coherencia del plan (Core):** NUNCA un plan sin tier ni sin cuota declarada (feature §Restricciones) → la validación pura debe rechazarlo.
- **Clasificación UI (ADR-0117) + backend-first (decisión del usuario 2026-07-04):** la feature tiene Superficie prevista (vista de planes, read-only) pero su UI + la sincronización real son adaptador diferido. Para ESTA Story el observable se verifica por **CLI (Canal #2, ADR-0142)**. La **SVF (Canal #1) + galería con mocks** de este cimiento se entregan en la **tanda de UI final del substrato** (harness SVF genérico) — es **deuda rastreada y autorizada** por el usuario (backend-first), NO diferición silenciosa. Registrada en `PROGRESS.md`.
- **SAD:** SAD-22 ya cubre el substrato. Si detectas desalineamiento → escala.

## 4. Instrucciones de despacho (prompt exacto)

> Eres el **Rust-Engineer** en **Modo Docente**. Lee primero: `CLAUDE.md`, `.claude/skills/base/SKILL.md`, `.claude/skills/rust-engineer/SKILL.md`, esta Orden completa, la feature `docs/features/plan-tier-quota.md`, las features ya construidas `docs/features/central-identity.md` y `docs/features/licensing-system.md` (patrón de referencia), y los ADR-0144, ADR-0143, ADR-0137, ADR-0141, ADR-0020 (§ADR.md perfiles), ADR-0008, ADR-0093, ADR-0142. Declara que los leíste.
>
> **Gate de Lectura Pre-Código:** confirma la ubicación del crate (`crates/shared`, ver §3) leyendo el `licensing-system` ya construido (`domain/licensing_system.rs`, `orchestrator/licensing_system.rs`, `persistence/licensing_system.rs`, `public_interface.rs`) como plantilla de puerto+stub+caché+concurrencia optimista.
>
> **Construye (catálogo local — la sincronización real con la Cabina de Mando es un stub):**
> 1. **Migración greenfield 0009** de la tabla de planes: Grupo I completo con **`row_version`** (mutable, ADR-0141), Perfil D + `owner_id`/`institutional_tag`/`node_id`; campos propios marcados (`tier TEXT CHECK`, `notional_limit INTEGER` ×10⁸, `max_activations INTEGER`, `price INTEGER` ×10⁸, `pricing_model TEXT CHECK`, conjunto de features con codificación determinista sin `REAL`). `STRICT`, PK `TEXT` UUIDv7, timestamps `INTEGER` ns UTC. Baseline editable in-situ (greenfield).
> 2. **Core (lógica pura, sin I/O):** (a) `validate_plan`: rechaza plan sin tier o sin cuota; verifica coherencia (tier ∈ conjunto, montos ≥ 0 como enteros escalados, pricing_model válido); (b) `resolve_limits`: dado un plan (o el tier/plan_id de una licencia) → `PlanLimits` (nocional, activaciones, features). Determinismo bit-a-bit; sin `REAL` en ningún cálculo de monto.
> 3. **Shell:** persistencia del catálogo (CRUD con **concurrencia optimista** en update: `WHERE id=? AND row_version=?` + chequeo `rows_affected`→conflicto, patrón de `licensing-system`); caché local de límites resueltos con TTL (reloj inyectado); **stub local** que siembra un catálogo de planes de desarrollo (Free/Paid con sus cuotas) — el catálogo real lo define la Cabina de Mando (futuro), coméntalo como tal.
> 4. **`public_interface`:** el puerto `plan_limits_out` que devuelve `PlanLimits` para un tier/plan. **Sin secretos** (ADR-0093).
> 5. **CLI `verify` (Canal #2, ADR-0142):** subcomando que reproduce el observable (límites vigentes de un plan/tier) en JSON, ejecutable por `cargo run -p app -- verify plan-tier-quota --input '<json>'`.
>
> **Pruebas discriminantes (rojo→verde, deben poder fallar):**
> - **Validación de plan:** un plan sin tier o sin cuota → rechazado (assert). Debe fallar si `validate_plan` acepta cualquier cosa.
> - **Precios como entero escalado:** persistir `price`/`notional_limit` y releer sin deriva (p. ej. $10,000.00 → `1_000_000_000_000` y de vuelta exacto). Debe fallar si algo usa `REAL`/float. Inspección de esquema: columnas de monto son `INTEGER`.
> - **Resolución de límites:** para el tier Free → `PlanLimits` con la cuota Free; para Paid → la cuota Paid (assert por tier).
> - **Cambio de plan reflejado:** actualizar el `notional_limit` de un plan → `resolve_limits` devuelve el nuevo valor, y `row_version` incrementó. Debe fallar si el cambio no se refleja.
> - **Concurrencia optimista (lección STORY-027/028):** dos updates del mismo plan desde el mismo `row_version` → el segundo da conflicto (`rows_affected == 0`), NO pisa en silencio (assert).
> - **`CHECK` de enums:** insertar `tier`/`pricing_model` inválido → rechazado por la BD.
> - **Caché TTL:** límites válidos dentro del TTL; revalida pasado el TTL (reloj determinista, no `SystemTime`).
> - **Guardarraíl ADR-0093:** el payload de `PlanLimits` NO contiene secretos (assert explícito).
> - Cobertura del criterio con `cargo llvm-cov --workspace --summary-only`.
>
> **Comentarios en español, identificadores en inglés (ADR-0121).** Entrega en verde con mapeo criterio→prueba.
>
> **Docente:** escribe `docs/lessons/rust/STORY-029-plan-tier-quota.md` (enlace a esta Orden al inicio) explicando cero-conocimiento: qué es un catálogo configurable (dato, no código), por qué los montos son enteros escalados y no floats, qué es resolver límites por licencia, y por qué la tabla es mutable (`row_version`). Cita el código real que produjiste.
>
> **NO hagas commits** (los hace el Tech-Lead). Al terminar reporta: archivos creados, salida de `cargo test` + `cargo llvm-cov`, salida del `cargo run -p app -- verify plan-tier-quota`, y tu decisión de ubicación del crate con su justificación.

## 5. Criterio de aceptación (verificable)

| # | Criterio | Prueba |
|---|---|---|
| 1 | Migración `STRICT` con Grupo I + Perfil D + `row_version` | inspección del `.sql` + test de esquema |
| 2 | Montos como `INTEGER` ×10⁸ (nunca `REAL`), round-trip exacto | test discriminante + inspección de esquema |
| 3 | `validate_plan` rechaza plan sin tier/cuota | test discriminante |
| 4 | `resolve_limits` devuelve `PlanLimits` correcto por tier | test por tier |
| 5 | Cambio de límite reflejado + `row_version` incrementa | test |
| 6 | Concurrencia optimista real en update | test de conflicto (`rows_affected == 0`) |
| 7 | `CHECK` de `tier`/`pricing_model` | test de rechazo |
| 8 | `PlanLimits` sin secretos (ADR-0093) | test + assert |
| 9 | Caché con TTL usando reloj determinista | test de expiración |
| 10 | CLI `verify plan-tier-quota` devuelve el JSON correcto | `cargo run -p app -- verify plan-tier-quota --input '…'` |
| 11 | Lección Docente escrita | existe `docs/lessons/rust/STORY-029-plan-tier-quota.md` |
| 12 | Verde + cobertura de cada criterio | `cargo test` + `cargo llvm-cov` |

## 6. Comandos de validación (usuario)

```bash
cargo test -p shared
cargo llvm-cov --workspace --summary-only
cargo run -p app -- verify plan-tier-quota --input '{"tier":"FREE"}'
```

## 7. Registro de ejecución

- 2026-07-04 · Tech-Lead · Gate corrido (contraste bidireccional). Reglas: tabla mutable→`row_version`; montos `INTEGER` ×10⁸ (no `REAL`); puerto `plan_limits_out`→`PlanLimits` (ya en catálogo ADR-0137); crate `crates/shared`; SVF/galería a la tanda de UI final (deuda rastreada). Orden creada, pendiente de despacho a Rust-Engineer (Sonnet, **Docente**).
- 2026-07-04 · Rust-Engineer (Docente) · Entregado en verde. Migración `0009_plan_tier_quota.sql` (tabla `plans`, Grupo I + `row_version` + Perfil D, montos `INTEGER` ×10⁸ con `CHECK >= 0`, `tier`/`pricing_model` con `CHECK`, `features_enabled` JSON con `json_valid`); Core `domain/plan_tier_quota.rs` (`validate_plan`, `resolve_limits`, features JSON determinista, `PlanLimits`); Shell `persistence/plan_tier_quota.rs` (repo con concurrencia optimista) y `orchestrator/plan_tier_quota.rs` (`seed_default_catalog` stub Free/Paid, `PlanLimitsCache` keyed por tier); puerto en `public_interface::plan_tier_quota` + CLI `verify plan-tier-quota`. **Decisión de diseño:** el `PlanLimits` real vive bajo submódulo `public_interface::plan_tier_quota::*` para no colisionar con el stub sellado de #2 (`E0255`). 35 tests nuevos (220 workspace), clippy limpio, cobertura 98–100%. Lección Docente escrita.
- 2026-07-04 · Tech-Lead · Auditoría independiente (reproducida): 220 tests / 35 de plan-tier-quota verdes; clippy limpio; FCIS 0 violaciones; esquema `STRICT`+UUIDv7+`row_version`+CHECK+montos `INTEGER` (ningún `REAL`); `UPDATE ... WHERE id=? AND row_version=?`+`rows_affected()==0`→conflicto; CLI Free/Paid con enteros escalados correctos. Verde.
- 2026-07-04 · QA-Engineer (Sonnet) · **APTO.** Lógica línea por línea + reproducción + **pruebas de mutación**: neutralizó la concurrencia optimista (`WHERE ... AND 1=?`) → cayó el test de refresco concurrente; neutralizó la guarda `MissingQuota` (`if false && ...`) → cayeron 3 tests. Verificó montos ×10⁸ sin deriva (round-trip $10K/$1M), tier desconocido falla seguro (exit 1, sin fallback a Paid), `CHECK` a nivel BD con inserción cruda, caché TTL con reloj determinista + keying por tier, `audit_chain_hash` NULL solo en génesis, `PlanLimits` sin secretos. Los dos `PlanLimits` = observación no bloqueante (deuda de re-cableado ya en §8). Mutaciones restauradas byte a byte. Sin regresiones.
- 2026-07-04 · Tech-Lead · Gate QA cerrado con APTO. **STORY-029 completada** (catálogo local). Feature `plan-tier-quota` 🟡 Parcial por diseño (sincronización central + re-cableado de #2 + UI diferidos).

## 8. Deudas / diferidos registrados

- **Sincronización real del catálogo con la Cabina de Mando (diferido, ADR-0144):** ahora es un stub que siembra planes de desarrollo. La sincronización real llega con la Cabina de Mando.
- **Re-cableado de `licensing-system` (#2):** su `plan_limits_in` hoy consume un stub de `PlanLimits`; una vez cerrado este cimiento, se puede enchufar al `plan_limits_out` real. Es un **follow-up de integración** (toca código sellado de #2 → nueva mini-Story con su propio QA), NO parte de esta Orden.
- **SVF (Canal #1) + galería con mocks:** se entregan en la tanda de UI final del substrato (harness SVF genérico). Deuda rastreada y autorizada (backend-first).
- **UI de planes/precios (Superficie propia, diferida):** vista read-only; parte del adaptador. Verificación de esta Story vía CLI Canal #2.
