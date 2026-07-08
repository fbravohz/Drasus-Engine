# STORY-042 — Retrabajo #3: añadir la cuota `MAX_CHILD_ACCOUNTS` al catálogo de planes

> **Orden de retrabajo del Tech-Lead** · Ingeniero: Rust-Engineer (Sonnet, Revisión) · Paga **DEBT-017** · Extiende `plan-tier-quota` (#3) por ADR-0149 (cimiento #14 `operator-roles`), banner 🔶 en `plan-tier-quota.md`.
>
> **Toca código SELLADO de #3** (`plan-tier-quota`, STORY-029) → auditoría TL independiente + QA por mutación **obligatorios**.

## 1. Objetivo

Añadir un límite nuevo al catálogo de planes: **`MAX_CHILD_ACCOUNTS`** — cuántas cuentas maestras hijas puede crear un fondo bajo `master-account-hierarchy` (#12). Mismo mecanismo que `notional_limit`/`max_activations`: un campo más del plan, dato configurable, **no infraestructura nueva**. La autoridad "solo el propietario de Drasus lo fija, nunca el fondo" es de la Cabina de Mando (diferida); aquí solo se añade el campo al catálogo.

## 2. Fase greenfield (ADR-0006)

`0009` es editable **in situ**. Edita la migración existente; no crees una nueva.

## 3. Mapa quirúrgico

### 3.1 `migrations/0009_plan_tier_quota.sql`
- Añade columna `max_child_accounts INTEGER NOT NULL DEFAULT 0 CHECK (max_child_accounts >= 0)` a la tabla `plans` (junto a `notional_limit`/`max_activations`). Documenta en comentario: cuota de cuentas hijas (#12), ×1 (conteo, no ×10⁸), `0` = plan sin derecho a cuentas hijas. Fijado por el proveedor, no por el fondo (autoridad en la Cabina, diferida).

### 3.2 `crates/shared/src/domain/plan_tier_quota.rs`
- Añade `max_child_accounts: i64` a `PlanCandidate` y a `PlanLimits`.
- `validate_plan`: rechaza `max_child_accounts < 0` (error tipado, como `notional_limit < 0`). **`max_child_accounts == 0` es VÁLIDO** (un plan sin cuentas hijas) — NO lo metas en la regla "notional_limit==0 && max_activations==0 es inválido"; esa regla queda igual.
- `resolve_limits`: incluye `max_child_accounts` en el `PlanLimits` que produce.

### 3.3 `crates/shared/src/persistence/plan_tier_quota.rs`
- INSERT/SELECT/UPDATE incluyen `max_child_accounts`.
- `seed_default_catalog`: puebla el nuevo campo — sugerencia coherente: `FREE` → `0` (sin cuentas hijas), `PAID` → un valor > 0 (ej. `5`). El ingeniero fija el default del stub; documenta el porqué.

### 3.4 `crates/shared/src/public_interface.rs` + `crates/app/src/main.rs`
- El `verify plan-tier-quota` expone `max_child_accounts` en el `PlanLimits` de salida. Si el input permite definir un plan, acepta `max_child_accounts` (default `0`).

## 4. NO tocar (crítico)
- **El stub `PlanLimits` de `domain/licensing_system.rs` (#2)** — es un struct DISTINTO en otro namespace (solo `max_activations` + `features_enabled`). NO le añadas `max_child_accounts`; el re-cableado de #2 es un follow-up aparte. Solo tocas el `PlanLimits` de `domain/plan_tier_quota.rs` (#3).
- Concurrencia optimista `row_version`/`VersionConflict` intacta.

## 5. Tests obligatorios (ADR-0133)
- Todos los tests de #3 existentes siguen pasando (adaptados al campo nuevo).
- `validate_plan` rechaza `max_child_accounts < 0` y ACEPTA `== 0`.
- `resolve_limits` propaga `max_child_accounts` correctamente por tier.
- `seed_default_catalog` puebla el campo (FREE=0, PAID>0).
- JSON no filtra secretos (test existente intacto).

## 6. Verificación antes de reportar
```bash
cargo test -p shared plan_tier_quota
cargo clippy -p shared --all-targets -- -D warnings
cargo run -p app -- verify plan-tier-quota --input '{"tier":"PAID"}'
```
El output debe mostrar `max_child_accounts` en los límites resueltos del plan PAID.

## 7. Docente (ADR-0122)
Documenta la lección (extender un catálogo de cuotas reutilizando el mecanismo existente, sin infraestructura nueva) en `docs/lessons/rust/`.

## 8. Prohibiciones
- **NO** commitees nada. **NO** toques archivos protegidos del Architect ni otras features (#10/#11/#12).
- **NO** uses modelos/agentes Opus.

---

## §9. Registro de cierre (lo llena el Tech-Lead al auditar)
- **Ingeniero:** 2026-07-07 · Rust-Engineer (Sonnet) — **entrega parcial**: alcanzó la migración `0009` (columna `max_child_accounts` + CHECK) y el domain parcial (`PlanCandidate`/`validate_plan`/`PlanLimits`/`PlanSnapshot` + firma en `compute_plan_audit_hash`), pero **se estancó** (watchdog de streaming) a mitad de `compute_plan_audit_hash`, dejando el árbol sin compilar (5 errores; persistence, orchestrator y public_interface sin tocar).
- **Completado por el Tech-Lead:** 2026-07-07 — el trabajo restante era mecánico (replicar el patrón de `notional_limit`/`max_activations` en los ~14 puntos de contacto: uso del parámetro en el hash, `NewPlan`/`Plan` structs, `create`/`update_limits`/`row_to_plan`/SELECT en persistence, `LocalStubPlanCatalogConfig`+`seed_default_catalog`+`PlanSnapshot` en orchestrator, `PlanTierQuotaVerifyOutput` en public_interface, y los helpers/allowlist-de-claves de tests). Decisión de diseño: `update_limits` conserva `max_child_accounts` (no lo revisa hoy) pero lo re-sella en el hash; `0` es válido, solo se rechaza `< 0` (`NegativeMaxChildAccounts`). Lección Docente escrita por el TL.
- **Auditoría / verificación:** 2026-07-07 · `cargo test -p shared` **528 verdes** (suite completa, cero regresión en #10/#11/#12) + `cargo clippy` limpio; CLI `verify plan-tier-quota --input '{"tier":"PAID"}'` expone `max_child_accounts: 5` (FREE → `0`).
- **QA por mutación (`cargo-mutants` sobre el Core, ejecutada por el TL):** 2026-07-07 · **APTO**. 39 mutantes: 36 cazados, 3 inviables, **0 sobrevivientes** — cobertura total del Core (incluida la validación de `max_child_accounts`). Sin deuda nueva. (El gate se aplicó igual pese a que el TL completó el código.)
- **Estado:** ✅ **CERRADO — DEBT-017 PAGADA.** Pendiente de autorización: commit agrupado.
