# Plan / Tier / Quota

> 🟡 **Parcial** 2026-07-04 · Orden de trabajo [STORY-029](../execution/STORY-029-plan-tier-quota.md) · Cimiento local completo: migración `0009_plan_tier_quota.sql` (Grupo I + Perfil D acotado + `row_version`), Core puro (`domain/plan_tier_quota.rs`: `validate_plan`, `resolve_limits`, codificación determinista de `features_enabled`, hash de auditoría encadenado), Shell (`persistence/plan_tier_quota.rs`: repositorio con concurrencia optimista; `orchestrator/plan_tier_quota.rs`: `seed_default_catalog` stub Free/Paid, `PlanLimitsCache` con TTL keyed por tier), puerto `plan_limits_out` → `PlanLimits` en `public_interface::plan_tier_quota`, CLI `verify plan-tier-quota` (ADR-0142). Crate: `crates/shared` (excepción bendecida ADR-0137). Pendiente: sincronización real del catálogo con la Cabina de Mando Central (no existe aún), re-cableado de `licensing-system` (#2) para consumir este `PlanLimits` real en vez de su stub (follow-up de integración diferido, fuera de esta Story), y la UI de planes/precios (Superficie propia, deuda de integración).

**Carpeta:** `./features/plan-tier-quota/`
**Estado:** 🟡 Parcial (cimiento local completo; sincronización central, re-cableado de licensing-system y UI diferidos)
**Última actualización:** 2026-07-04
**Decisión Arquitectónica Asociada:** ADR-0144 (cimiento #3) · ADR-0008 (configurabilidad)

## ¿Qué es esta feature?

El **catálogo configurable de planes** y sus límites. Un plan es dato, no código: define el tier, sus cuotas (volumen nocional permitido, activaciones simultáneas, features habilitadas) y su precio. Permite cambiar la estructura de precios sin recompilar.

- **Problema:** si los tiers están hardcodeados, cada cambio de pricing es un release. El negocio necesita mover precios y límites con agilidad.
- **Comportamiento observable:** existe un catálogo de planes; cada licencia apunta a un plan; los límites del plan alimentan el gate y la medición.
- **Por qué:** desacopla la política comercial (cambia seguido) de la mecánica de licencia (estable).

## Comportamientos Observables

- Cuando se define un plan → queda disponible con su tier, cuotas y precio.
- Cuando una licencia referencia un plan → hereda sus límites (activaciones, volumen nocional).
- Cuando se cambia el límite de un plan → las licencias de ese plan lo reflejan en la siguiente revalidación.
- Cuando se opera cobro por volumen o flat-fee → el mismo catálogo sirve a ambos (el adaptador de billing elige cómo leerlo).

## Restricciones

- NUNCA un plan sin tier ni sin cuota declarada.
- Los precios se guardan como enteros escalados (×10⁸), nunca `REAL` (ADR-0141).
- El catálogo es fuente de verdad central; el motor local lo cachea.

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| TIER_SET | Free, Paid | conjunto | Tiers disponibles (extensible) | CONFIG |
| NOTIONAL_LIMIT_FREE | $10K/mes | 0 – ∞ | Volumen nocional real permitido en gratuito | CONFIG |
| PRICING_MODEL | flat | flat / volumen | Cómo lee el billing el catálogo | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** validación de coherencia de un plan (tier + cuotas + precio), resolución de "¿qué límite aplica a esta licencia?".
- **Shell (Infraestructura):** persistencia del catálogo, sincronización con la Cabina de Mando, caché local.
- **Frontera Pública:** puerto que expone los límites de un plan; consumido por `licensing-system` y `usage-metering`.

## Ciclo de Vida de la Feature — Plan / Tier / Quota

### Entrada
Definición de plan (tier, cuotas, precio) desde la Cabina de Mando.

### Proceso
Valida y publica el plan; resuelve los límites aplicables a una licencia dada.

### Salida
El conjunto de límites vigentes (volumen nocional, activaciones, features) para el tier de la licencia.

## Tareas (TTRs)

- **TTR-001:** Catálogo de planes con cuotas y precio (entero escalado).
- **TTR-002:** Resolución de límites por licencia (Core puro) y caché local.

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `plan_limits_out` | `PlanLimits` (tipo técnico nuevo — plomería, ADR-0144) | Output | `1..N` | Límites del plan (nocional, activaciones, features); consumido por `licensing-system` y `usage-metering`. |

## Cáscara Visual (Thin Shell)

> Pendiente Etapa 0.5 (UI-Designer). Superficie prevista: vista de planes/precios (solo lectura para el usuario; administración en la Cabina de Mando). El Architect NO rellena esta sección.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016 enmendado por ADR-0143):** catálogo central, cacheado local.
- **Inundación de Fundaciones (ADR-0020):** Grupo I completo + **Perfil D (Ops/Auditoría)**: Identidad(I) + Soberanía(II: `owner_id` del creador del plan, `institutional_tag`) + Hardware(IV: `node_id`).

## Persistencia (Inundación de Fundamentos — ADR-0020)

Tabla de planes con Grupo I + Perfil D. Campos propios fuera del catálogo (marcados): tier, límite de volumen nocional (entero ×10⁸), activaciones máximas, precio (entero ×10⁸), conjunto de features. Enums (`tier`, `pricing_model`) con `CHECK`; `STRICT`; UUIDv7 (ADR-0141).

## Dependencias y Bloqueantes

- **Bloquea a:** `licensing-system` (necesita límites de activación) y `usage-metering` (necesita límite de nocional).
- **Contrato de Integración UI (ADR-0117) — Ventana de Verificación:** su observable (catálogo de planes y límites vigentes) queda visible en el panel de licencia de `licensing-system`; hasta entonces, deuda de integración registrada.
