# Consent Registry (ToS)

> 🟡 **Parcial** 2026-07-04 · Orden de trabajo [STORY-031](../execution/STORY-031-consent-registry.md) · Cimiento local completo: migración `0011_consent_registry.sql` (Grupo I append-only con `event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE + `CHECK(json_valid(optout_map))` + Perfil D acotado), Core puro (`domain/consent_registry.rs`: `needs_reacceptance`, `resolve_coverage` con las tres puertas Covered/StaleVersion/OptedOut/NoConsent y default-niega, `apply_consent_action` — event-sourcing con snapshot completo sobre `BTreeMap` para serialización JSON determinista, `parse_optout_map`, hash de auditoría encadenado por `event_sequence_id`), Shell (`persistence/consent_registry.rs`: repositorio APPEND-ONLY sin `update`/`delete`, `load_latest_for_owner` para el estado vigente; `orchestrator/consent_registry.rs`: composición del puerto), puerto `consent_out` → `ConsentVerdict` en `public_interface` (más re-exports planos), CLI `verify consent-registry` (ADR-0142). Crate: `crates/shared` (excepción bendecida ADR-0137). Pendiente: sincronización con la Cabina de Mando Central (no existe aún), cableado real de consumidores (`data-aggregation` #9, firehose, opt-in del track record ADR-0145 #10), SVF (Canal #1) + galería del panel de ToS/opt-outs (deuda rastreada, backend-first).

**Carpeta:** `./features/consent-registry/`
**Estado:** 🟡 Parcial (cimiento local completo; sincronización central, cableado de consumidores y UI diferidos)
**Última actualización:** 2026-07-04
**Decisión Arquitectónica Asociada:** ADR-0143 (firehose gratuito) · ADR-0144 (cimiento #5)

## ¿Qué es esta feature?

El registro **versionado y fechado** de aceptación de Términos y Condiciones. Es la columna vertebral legal del modelo: el firehose del tier gratuito (ADR-0143) y toda venta de datos agregados (ADR-0102/0144) son legales **solo si** hay consentimiento registrado.

- **Problema:** usar o vender datos del usuario sin base legal probable es ilegal (GDPR y equivalentes).
- **Comportamiento observable:** el usuario acepta un ToS con versión concreta; queda registrado con fecha; puede ajustar opt-outs granulares **solo en las categorías genuinamente opcionales**.
- **Por qué:** sin este registro, el negocio de datos no existe legalmente.

> **Dos categorías de consentimiento, NO intercambiables (decisión del propietario 2026-07-07, base legal GDPR Art. 6/7):**
> 1. **Gate de tier (obligatorio, ToS, NO es "consentimiento" revocable en sentido GDPR):** el firehose de trabajo/PI del tier gratuito (Clase 1, ADR-0143) y el control/licencia/anti-abuso (Clase 3, todos los tiers) son la **contraprestación contractual** del tier elegido — base legal Art. 6(1)(b) "necesario para la ejecución del contrato", no Art. 6(1)(a) "consentimiento". Aceptar el ToS es binario: **si el usuario no acepta, no usa Drasus en ese tier** — la alternativa real es el tier de pago (que suprime el firehose). Esto NUNCA tiene un toggle de opt-out granular dentro del tier gratuito: desactivar el firehose sin pagar rompería tanto el modelo de negocio como el principio legal (sería dar gratis lo que el ToS declara como su contraprestación).
> 2. **Consentimiento genuino (opt-in real, siempre revocable, NUNCA condiciona el acceso al servicio):** categorías que NO son necesarias para prestar el servicio — hoy, la publicación del track record (Clase 5, `verified-account-registry` #10, opt-in independiente del tier, ADR-0145) y cualquier futura categoría de la misma naturaleza (ej. comunicaciones de marketing). Aquí SÍ aplica el opt-out/opt-in granular; forzarlo como condición de acceso violaría Art. 7(4) GDPR (prohibición de "bundling" de consentimiento).
>
> El campo `optout_map` de este registro (ver Persistencia) modela **solo la categoría 2**. La categoría 1 se resuelve en `licensing-system`/`plan-tier-quota` (qué tier tiene el usuario), no aquí.

## Comportamientos Observables

- Cuando el usuario acepta el ToS → se registra la versión aceptada con fecha; sin ello, no opera en ningún tier (gate obligatorio, no granular).
- Cuando cambia la versión del ToS → se exige re-aceptación antes de continuar.
- Cuando el usuario ajusta un opt-out granular en una categoría **genuinamente opcional** (ej. Clase 5, publicación del track record) → el pipeline correspondiente lo respeta. **Esto NUNCA incluye el firehose del tier gratuito** — ese no es ajustable por el usuario, es inherente al tier.
- Cuando se audita el consentimiento de un dato → se puede probar qué versión aceptó el usuario y cuándo.

## Restricciones

- NUNCA se procesa dato del usuario sin base legal vigente que lo cubra (ToS del tier, o consentimiento granular en categorías opcionales).
- El registro de consentimiento es append-only (inmutable, auditable).
- NUNCA se asume consentimiento por defecto para venta a terceros fuera del gate de tier: el opt-out granular de categorías opcionales manda.
- NUNCA se ofrece un opt-out granular sobre datos que son la contraprestación contractual del tier gratuito (categoría 1) — eso se resuelve cambiando de tier, no con un toggle.

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| TOS_VERSION_ACTUAL | (definida) | texto | Versión de ToS vigente | CONFIG |
| REACCEPT_ON_VERSION_CHANGE | true | true/false | Exigir re-aceptación al cambiar versión | FIJO |
| GRANULAR_OPTOUT_TYPES | (conjunto) | conjunto | Categorías de dato con opt-out independiente | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** comparación de versión aceptada vs. vigente, resolución de "¿este dato está cubierto por un consentimiento activo?".
- **Shell (Infraestructura):** persistencia append-only de aceptaciones y opt-outs, sincronización con la Cabina de Mando.
- **Frontera Pública:** puerto que responde "¿puedo procesar/vender este tipo de dato de este usuario?"; consumido por `data-aggregation` y el firehose.

## Ciclo de Vida de la Feature — Consent Registry

### Entrada
Aceptación del usuario (versión de ToS) y sus ajustes de opt-out granular.

### Proceso
Registra la aceptación con fecha, valida vigencia de versión, resuelve cobertura por tipo de dato.

### Salida
Un veredicto de consentimiento (cubierto / no cubierto) por tipo de dato, y el registro auditable de aceptación.

## Tareas (TTRs)

- **TTR-001:** Registro append-only de aceptación de ToS con versión y fecha.
- **TTR-002:** Opt-out granular por tipo de dato y resolución de cobertura (Core puro).
- **TTR-003:** Re-aceptación forzada al cambiar la versión vigente.

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `consent_out` | `ConsentVerdict` (tipo técnico nuevo — plomería, ADR-0144) | Output | `1..N` | Veredicto de cobertura de consentimiento por tipo de dato; consumido por `data-aggregation` y el firehose de telemetría. |

## Cáscara Visual (Thin Shell)

> Pendiente Etapa 0.5 (UI-Designer). Superficie prevista: pantalla de aceptación de ToS + panel de opt-outs granulares en ajustes. El Architect NO rellena esta sección.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016 enmendado por ADR-0143):** el consentimiento se captura local y se replica a la Cabina de Mando (es prueba legal central).
- **Inundación de Fundaciones (ADR-0020):** Grupo I completo + **Perfil D (Ops/Auditoría/Cumplimiento)**: Identidad(I) + Soberanía(II: `owner_id`, `institutional_tag`) + subset V (`compliance_status_id`).

## Persistencia (Inundación de Fundamentos — ADR-0020)

Registro append-only (`event_sequence_id UNIQUE`) con Grupo I + Perfil D. Campos propios fuera del catálogo (marcados): versión de ToS aceptada, timestamp de aceptación, mapa de opt-outs por tipo. `STRICT`, UUIDv7, `audit_chain_hash` encadenado (ADR-0141).

## Dependencias y Bloqueantes

- **Depende de:** `central-identity` (`owner_id`).
- **Bloquea a:** `data-aggregation` (no agrega sin consentimiento), el firehose de `enriched-domain-events`, y [`data-portability`](data-portability.md) (#13, el registro de aceptación de ToS/opt-outs forma parte de lo exportable).
- **Contrato de Integración UI (ADR-0117) — Superficie propia:** pantalla de ToS + panel de opt-outs. SVF: aceptar el ToS dispara el registro real vía `public_interface`; el panel muestra la versión aceptada y la fecha; tras recargar, persiste.
