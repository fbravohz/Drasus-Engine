# Lección — STORY-042: extender un catálogo de cuotas (`MAX_CHILD_ACCOUNTS` en `plan-tier-quota` #3)

> Retrabajo de extensión sobre código sellado del cimiento #3, por ADR-0149 (#14 `operator-roles`). Paga DEBT-017.

## Contexto no obvio

Añadir una cuota nueva a `plan-tier-quota` **no es** inventar infraestructura: es replicar el mecanismo que `notional_limit`/`max_activations` ya tienen, en **todos** sus puntos de contacto. El valor de la lección está en el inventario completo de esos puntos — omitir uno rompe la compilación o, peor, deja un campo que no se sella en la cadena de auditoría.

## Inventario de puntos de contacto de una cuota (calca `notional_limit`)

Para un campo de cuota `x: i64` en el catálogo de planes, hay que tocarlo en **todos** estos sitios, o el cambio queda incompleto:

1. **Migración** (`0009`): columna `x INTEGER NOT NULL DEFAULT 0 CHECK (x >= 0)`.
2. **Domain — `PlanCandidate`** (entrada de validación) + **`validate_plan`** (regla) + **`PlanSnapshot`** (entrada de resolución) + **`PlanLimits`** (salida del puerto) + **`resolve_limits`** (mapea snapshot→limits) + **`compute_plan_audit_hash`** (firma la cuota).
3. **Persistence — `NewPlan`** + **`Plan`** structs; **`create`** (PlanCandidate + audit_hash + INSERT columnas/VALUES/bind + `Ok(Plan)`); **`update_limits`** (audit_hash con el valor vigente); **`row_to_plan`** (lee la columna); las **SELECT** (lista de columnas); **`seed_default_catalog`** (siembra por tier).
4. **Orchestrator — `LocalStubPlanCatalogConfig`** (config del stub) + su `Default` + `seed_default_catalog` (pasa config→NewPlan) + cualquier `PlanSnapshot`/`PlanLimits` construido.
5. **public_interface — `PlanTierQuotaVerifyOutput`** (struct de salida del CLI) + `from_error`/`from_limits`. **Un struct de salida propio NO hereda campos por serialización** — si el `verify` arma su propio DTO, hay que añadir el campo a mano o el CLI no lo expone (fallo silencioso que compila).
6. **Tests**: los helpers (`valid_candidate`, `sample_free_plan`, `sample_paid_plan`), los literales de `PlanLimits`/`PlanSnapshot`, los call sites de `compute_plan_audit_hash` (¡su aridad cambió!), y **la allowlist de claves JSON** del test anti-fuga de secretos (`plan_limits_json_never_leaks_secret_fields`) — añadir un campo al struct **rompe** ese test hasta que se actualiza la lista esperada.

## Dos decisiones de diseño que importaron

- **`0` es un valor válido, no "sin cuota".** A diferencia de la regla "`notional_limit == 0 && max_activations == 0` ⇒ plan inválido" (un plan debe tener alguna cuota), `max_child_accounts == 0` es legítimo: significa "este tier no puede crear cuentas hijas" (el caso FREE). La validación solo rechaza `< 0` (`NegativeMaxChildAccounts`); **no** se mete la cuota nueva en la regla de "cuota mínima".
- **La cuota entra en el hash de auditoría aunque `update_limits` no la revise hoy.** Cualquier campo de cuota debe sellarse en `compute_plan_audit_hash`; si quedara fuera, una alteración directa en disco (o una futura revisión en sitio) no rompería la cadena y pasaría inadvertida. En `update_limits` se pasa el valor **vigente** del plan (`plan.max_child_accounts`) — la ruta revisa nocional/activaciones/precio, la cuota de hijas se conserva pero se re-sella con el resto.

## Nota de proceso

El primer intento del subagente se estancó (watchdog de streaming) a mitad de `compute_plan_audit_hash`, dejando el árbol sin compilar (5 errores). El Tech-Lead completó el retrabajo directamente: el trabajo restante era mecánico (replicar el patrón en los ~14 sitios del inventario de arriba) y estaba completamente determinado por el patrón existente, más rápido y confiable que reanudar un agente colgado. El gate de QA por mutación se aplicó igual — el código lo escriba quien lo escriba, no se cierra sin APTO.
