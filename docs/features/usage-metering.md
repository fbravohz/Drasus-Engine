# Usage Metering (Libro de Nocional)

> 🟡 **Parcial** 2026-07-04 · Orden de trabajo [STORY-030](../execution/STORY-030-usage-metering.md) · Cimiento local completo: migración `0010_usage_metering.sql` (Grupo I append-only con `event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE + Perfil D acotado), Core puro (`domain/usage_metering.rs`: `compute_notional` con reescalado ×10¹⁶→×10⁸ en `i128` y redondeo explícito, `accumulate`, `detect_quota_crossing`, `derive_billing_cycle_id` sin dependencia de calendario externa, hash de auditoría encadenado por `event_sequence_id`), Shell (`persistence/usage_metering.rs`: repositorio APPEND-ONLY sin `update`/`delete`, acumulación por ciclo vía `SUM` filtrado; `orchestrator/usage_metering.rs`: consumo REAL de `PlanLimits` de `plan-tier-quota` #3 — primer cableado real entre cimientos del substrato), puerto `usage_out` → `UsageRecord` en `public_interface::usage_metering` (submódulo, más re-exports planos), CLI `verify usage-metering` (ADR-0142). Crate: `crates/shared` (excepción bendecida ADR-0137). Pendiente: mapeo del `Order` real (módulo `execute`/EPIC-5, hoy placeholder → se modela `MeteredOperation` mínimo), emisión real a `feedback`/telemetría, SVF (Canal #1) + galería del panel de consumo (deuda rastreada, backend-first).

**Carpeta:** `./features/usage-metering/`
**Estado:** 🟡 Parcial (cimiento local completo; mapeo de `Order` real, emisión a `feedback` y UI diferidos)
**Última actualización:** 2026-07-04
**Decisión Arquitectónica Asociada:** ADR-0144 (cimiento #4) · ADR-0143 (tiers)

## ¿Qué es esta feature?

El **contador de valor nocional en USD** por ciclo de facturación. Cada operación ejecutada suma su nocional (tamaño × precio) a un libro append-only por cuenta y ciclo. Es la métrica universal de uso — sirve igual a un cobro por volumen escalonado que a un flat-fee (el adaptador de billing elige cómo leerla).

- **Problema:** medir uso en lotes/contratos/acciones es inconsistente entre instrumentos. El nocional en USD es universal y el motor ya lo calcula para ejecutar.
- **Comportamiento observable:** al ejecutar una orden, su nocional se acumula; el usuario ve cuánto ha operado en el ciclo y cuánto le queda antes del siguiente tier.
- **Por qué:** es la base de cualquier modelo de cobro por uso y del gate de cuota del tier gratuito.

## Comportamientos Observables

- Cuando se ejecuta una orden → su nocional en USD se registra en el libro del ciclo vigente (append-only).
- Cuando el acumulado del ciclo cruza el límite del plan → se emite un evento de cuota alcanzada (upsell / gate).
- Cuando inicia un nuevo ciclo de facturación → el acumulado se reinicia (el histórico se conserva).
- Cuando el usuario abre su panel de consumo → ve el nocional acumulado y el límite de su tier.

## Restricciones

- NUNCA se mide el margen ni el apalancamiento: se mide el **nocional** (ADR-0143/0144).
- NUNCA se modifica un registro del libro: es append-only (`event_sequence_id`, sin `row_version`).
- El nocional se guarda como entero escalado (×10⁸), nunca `REAL` (ADR-0141).
- **Nocional en instrumentos no cotizados en USD (FX):** el nocional en USD se deriva de la **misma valoración que el motor ya produce** para ejecutar (precio de marca en la divisa de cotización → USD con la referencia FX presente en los datos de mercado al momento de ejecución). NUNCA se introduce un proveedor FX aparte ni se recotiza a posteriori: el tipo de cambio queda congelado con la operación (el libro es un hecho histórico inmutable).
- **Autoridad de reconciliación (Gap de sync):** el libro se acumula local, pero la **Cabina de Mando es la fuente autoritativa de facturación** (ADR-0143). Ante conflicto entre instancias del mismo dueño, el veredicto del servidor gana y el libro local obedece — el local nunca sobrescribe al central. El mecanismo de envío es el adaptador de red diferido (puerto ahora, adaptador después, ADR-0144).

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| BILLING_CYCLE | mensual | mensual/anual | Duración del ciclo de acumulación | CONFIG |
| QUOTA_ENFORCEMENT | soft | soft/hard | Si al cruzar el límite se bloquea (hard) o solo se avisa (soft) | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** cálculo de nocional (tamaño × precio, entero escalado), acumulación por ciclo, detección de cruce de umbral.
- **Shell (Infraestructura):** persistencia append-only del libro, emisión del evento de cuota, lectura para el panel.
- **Frontera Pública:** puerto que expone el acumulado del ciclo y el veredicto de cuota; consumido por `licensing-system` (gate) y por el billing futuro (adaptador `monetization-stripe`).

## Ciclo de Vida de la Feature — Usage Metering

### Entrada
Cada orden ejecutada (tamaño, precio, instrumento) y el límite de nocional del plan.

### Proceso
Calcula el nocional, lo acumula en el ciclo vigente y compara contra el límite.

### Salida
El acumulado del ciclo + un veredicto de cuota (dentro / cruzada), persistidos append-only.

## Tareas (TTRs)

- **TTR-001:** Registro append-only de nocional por orden (Core: cálculo entero escalado).
- **TTR-002:** Acumulación por ciclo y detección de cruce de umbral (Core puro).
- **TTR-003:** Reinicio de ciclo con conservación del histórico.

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `order_in` | `Order` | Input | `0..N` | Órdenes ejecutadas de las que se deriva el nocional. |
| `usage_out` | `UsageRecord` (tipo técnico nuevo — plomería, ADR-0144) | Output | `1..N` | Acumulado de nocional por ciclo + veredicto de cuota. |

## Cáscara Visual (Thin Shell)

> Pendiente Etapa 0.5 (UI-Designer). Superficie prevista: panel de consumo (nocional acumulado del ciclo vs. límite del tier). El Architect NO rellena esta sección.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016 enmendado por ADR-0143):** el libro se acumula local y se sincroniza a la Cabina de Mando por telemetría.
- **Inundación de Fundaciones (ADR-0020):** Grupo I completo + **Perfil D (Ops/Auditoría/Cumplimiento)**: Identidad(I) + Soberanía(II: `owner_id`, `institutional_tag`) + Hardware(IV: `node_id`) + subset V de gobernanza (`compliance_status_id` si aplica).

## Persistencia (Inundación de Fundamentos — ADR-0020)

Libro append-only (`event_sequence_id UNIQUE`, sin `row_version`) con Grupo I + Perfil D. Campos propios fuera del catálogo (marcados): nocional por operación (entero ×10⁸), acumulado del ciclo, identificador de ciclo, veredicto de cuota. `STRICT`, UUIDv7, `audit_chain_hash` encadenado (ADR-0141).

**Rastro de Evidencia:** emite el acumulado de nocional y los cruces de umbral al módulo `feedback` y a la telemetría.

## Dependencias y Bloqueantes

- **Depende de:** `plan-tier-quota` (límite de nocional), motor de ejecución (órdenes).
- **Bloquea a:** el gate de cuota del tier gratuito y el billing por volumen.
- **Contrato de Integración UI (ADR-0117) — Superficie propia:** panel de consumo. SVF: tras ejecutar una operación real, el panel muestra el nocional acumulado devuelto por el Core; tras recargar, el acumulado persiste.
