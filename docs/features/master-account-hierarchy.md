# Master Account Hierarchy (Jerarquía de Cuentas Maestras)

> 🟡 **Parcial** 2026-07-07 · Orden [STORY-040](../execution/STORY-040-master-account-hierarchy.md) · Core (`OverrideCommandKind` + gate puro `decide_override_authorization` que consume el `ConsentVerdict` **real** de #5 — override solo con consentimiento `Covered` vigente; "eliminar = archivar" vía `LocalEffect` enum cerrado sin borrado físico; hashes deterministas) + esquema (`migrations/0018_master_account_hierarchy.sql`: `account_hierarchy` MUTABLE con `parent_owner_id` nullable — puntero anti-`tenant_id`, no árbol; `override_attestations` APPEND-ONLY atómica) + puertos `identity_in`/`consent_in`/`override_command_out`/`override_attestation_out` + CLI `verify master-account-hierarchy`. **Doble atestación** ISSUER (fondo) + EXECUTOR (hija) con re-consulta de consentimiento por lado. Consume `AccountIdentity` real de #1 y `ConsentVerdict` real de #5. **Auditoría TL independiente aprobada + QA APTO por mutación** (23/29 cazados, 6 inviables, **0 sobrevivientes** — cobertura total, sin deuda nueva). Pendiente (diferido, disparador "venta a fondos"): relé cifrado genérico como adaptador de red (ADR-0143), panel consolidado del fondo y el indicador "reporta a `<Fondo>`" de la hija (Cáscara Visual).

**Carpeta:** `./features/master-account-hierarchy/`
**Estado:** Core + esquema + puertos implementados (relé de red y UI diferidos)
**Última actualización:** 2026-07-07
**Decisión Arquitectónica Asociada:** ADR-0147 (cimiento #12) · ADR-0119 (Plano de Control, generalizado un nivel arriba) · ADR-0143 (relé cifrado genérico) · ADR-0141 (append-only, "eliminar" nunca es DELETE) · ADR-0144 (anti-`tenant_id`)

## ¿Qué es esta feature?

Una **cuenta maestra raíz** (ej. un fondo de inversión) agrupa **N cuentas maestras hijas** (ej. traders/desks bajo su paraguas). Cada cuenta hija es un maestro completo por derecho propio — conserva su propio Plano de Control (ADR-0119) con sus propios nodos satélite VPS reportándole a ella — pero la raíz tiene autoridad de **auditoría total** (ver datos, resultados, informes, performance) y de **override total** (parar, archivar, mover, modificar estrategias/portafolios/clústeres/pipelines/parámetros) sobre cada hija.

- **Problema:** vender Drasus a un fondo de inversión exige que el fondo supervise y controle múltiples cuentas de trader bajo su mando, sin que cada trader pierda su autonomía operativa del día a día, y sin inventar una segunda arquitectura de multi-tenancy paralela a la ya decidida (ADR-0144).
- **Comportamiento observable:** el operador de una cuenta hija ve "esta cuenta reporta a `<Fondo X>`"; el fondo ve un panel consolidado de sus cuentas hijas y puede emitir comandos de override que la hija ejecuta localmente.
- **Por qué:** generaliza el patrón de mando que ya existe (ADR-0119: un Plano de Control administra N satélites sin estar en la ruta crítica) un nivel arriba, donde el "satélite" ya no es un nodo de ejecución sin criterio propio, sino otra cuenta maestra completa.

## Comportamientos Observables

- Cuando una cuenta hija se vincula a un fondo → registra un consentimiento versionado y fechado que autoriza la autoridad de auditoría/override del fondo (`consent-registry`, #5) — término contractual, no un backdoor impuesto.
- Cuando el fondo emite un comando de override (ej. "archivar esta estrategia") → viaja cifrado por el relé genérico (ADR-0143) hacia la hija, quien lo ejecuta localmente sobre su propia base de datos.
- Cuando la hija ejecuta un comando de override → encadena en su propia auditoría "recibí esta orden, firmada por mi padre, y la ejecuté"; el fondo encadena en la suya "emití esta orden" — ambos lados quedan atestados, nunca una mutación silenciosa.
- Cuando el fondo "elimina" una estrategia/portafolio de una hija → la fila se archiva/desactiva, nunca se borra físicamente ni rompe la cadena de hash (ADR-0141).
- Cuando el fondo intenta un override sobre una cuenta que revocó su consentimiento → el comando es rechazado y ambos lados lo registran como intento denegado.
- Cuando una cuenta hija opera su día a día → sigue orquestando sus propios satélites VPS (ADR-0119) sin depender del fondo; el fondo nunca está en la ruta crítica de sus decisiones operativas.

## Restricciones

- **NUNCA** la jerarquía se calca localmente como tabla de árbol completo (anti-`tenant_id`, ADR-0144 FIJO). Cada cuenta hija solo cachea un `parent_owner_id` — el árbol vive en el plano central (`central-identity`, #1).
- **NUNCA** el fondo escribe directo en la base de datos de la hija. Todo mando viaja como comando administrativo cifrado sobre el relé genérico (ADR-0143); la hija ejecuta localmente.
- **NUNCA** la autoridad del fondo es un backdoor técnico impuesto en silencio — se establece vía consentimiento versionado y fechado (`consent-registry`, #5).
- **NUNCA** un override queda sin atestación en ambos lados (hija y fondo) — de lo contrario se contradice la promesa de atestación soberana de `verified-account-registry` (#10).
- **NUNCA** "eliminar" es un DELETE físico — siempre archivar/desactivar (append-only, ADR-0141).
- **NUNCA** la hija pierde su autonomía de Plano de Control (ADR-0119) por tener un padre — la jerarquía es una capa encima, nunca un reemplazo.
- **NUNCA** Drasus se convierte en gestor/asesor regulado de facto — el fondo administra infraestructura propia (detener, archivar, auditar), no gestiona activos de terceros (esa idea ya está descartada como moonshot-zizaña, ADR-0144, "Capital Allocation Platform").

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| OVERRIDE_COMMANDS | Archive, Modify, RequestAuditReport | conjunto | Catálogo de comandos administrativos que el fondo puede emitir sobre una hija | CONFIG |
| CHILD_CONSENT_REQUIRED | true | true/false | Exige consentimiento vigente de la hija antes de aceptar cualquier override del fondo | FIJO |
| OVERRIDE_DOUBLE_ATTESTATION | true | true/false | Exige que todo override quede encadenado en ambos lados (hija y fondo) | FIJO |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** validación de que un comando de override está autorizado por el consentimiento vigente de la hija; cálculo de la firma/encadenado del evento de override en ambos extremos.
- **Shell (Infraestructura):** envío/recepción del comando cifrado sobre el relé genérico (ADR-0143); ejecución local del comando (archivar/modificar) sobre las tablas propias de la hija; panel consolidado de auditoría del lado del fondo.
- **Frontera Pública:** puerto que expone "emitir override" (lado fondo) y "recibir y atestar override" (lado hija); puerto de consulta "¿quién es mi padre / quiénes son mis hijas?".

## Ciclo de Vida de la Feature — Master Account Hierarchy

### Entrada
El comando de override del fondo (tipo + objetivo + justificación), el consentimiento vigente de la hija, y el estado local de la hija (estrategias/portafolios/parámetros existentes).

### Proceso
Verifica el consentimiento, cifra y envía el comando por el relé, la hija lo recibe y lo ejecuta localmente (archivar/modificar), ambos lados encadenan el evento en su propia auditoría.

### Salida
Un estado modificado (archivado/desactivado, nunca borrado) en la hija, y dos eventos atestados — uno en cada extremo — que prueban quién ordenó qué y quién lo ejecutó.

## Tareas (TTRs)

- **TTR-001:** Registro de jerarquía (`parent_owner_id`) y consentimiento contractual de la autoridad del fondo sobre la hija.
- **TTR-002:** Catálogo de comandos de override transportados sobre el relé genérico (ADR-0143).
- **TTR-003:** Doble atestación del override — encadenado en ambos extremos, "eliminar" siempre archiva.
- **TTR-004:** Panel consolidado de auditoría/reportes/performance del lado del fondo (Superficie propia, diferido junto con la UI).

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `identity_in` | `AccountIdentity` (plomería, ADR-0144) | Input | `1` | Identidad de la cuenta (fondo o hija), producida por `central-identity`. |
| `consent_in` | `ConsentVerdict` (plomería, ADR-0144) | Input | `1` | Vigencia del consentimiento de la hija a la autoridad del fondo, de `consent-registry` (#5). |
| `override_command_out` | Comando de override (tipo técnico nuevo — plomería, ADR-0147) | Output | `0..N` | Emitido por el fondo, viaja por el relé genérico hacia la hija. |
| `override_attestation_out` | Evento de override atestado (tipo técnico nuevo — plomería, ADR-0147) | Output | `1..N` | Encadenado en ambos extremos (fondo y hija) tras ejecutar un override. |

> Tipos técnicos nuevos del cimiento #12, registrados en el catálogo de ADR-0137 vía la enmienda de ADR-0147. Nombres canónicos de `struct` los fija el ingeniero.

## Cáscara Visual (Thin Shell)

> Pendiente de la Etapa 0.5 (UI-Designer). Superficie prevista: panel "cuentas hijas" para el fondo (auditoría + emisión de override) y un indicador "reporta a `<Fondo>`" en el panel de cuenta de la hija. El Architect NO rellena esta sección.

## Gobernanza y Estándares (Fijos)

- **Anti-`tenant_id` (ADR-0144):** la jerarquía reutiliza `owner_id`/`parent_owner_id`; prohibido calcar una tabla de árbol completo local.
- **Inundación de Fundaciones (ADR-0020):** Grupo I completo + **Perfil D (Ops/Auditoría)**: Identidad(I) + Soberanía(II: `owner_id`, `parent_owner_id`) + Hardware(IV: `node_id`) + subset V (`signature_hash`/`audit_chain_hash` del evento de override en ambos extremos).

## Persistencia (Inundación de Fundamentos — ADR-0020)

Registro de jerarquía con Grupo I + Perfil D; campos propios fuera del catálogo (marcados): `parent_owner_id` (referencia a la cuenta maestra raíz, nullable si la cuenta no tiene padre), catálogo de comandos permitidos, referencia al consentimiento vigente. Los eventos de override se persisten como filas **append-only** (mismo patrón que `attested_track_records` de #10) en ambos extremos — nunca UPDATE/DELETE. `STRICT`, UUIDv7.

## Dependencias y Bloqueantes

- **Depende de:** `central-identity` (#1, `parent_owner_id`), `licensing-system` (#2, activaciones por cuenta hija), `plan-tier-quota` (#3, cuota `MAX_CHILD_ACCOUNTS` — cuántas hijas puede crear un fondo), `consent-registry` (#5, autorización contractual), `verified-account-registry` (#10, la doble atestación preserva su garantía de integridad), ADR-0119 (Plano de Control de cada hija), el relé genérico de ADR-0143.
- **Consumido por:** [`operator-roles`](operator-roles.md) (#14) reutiliza este mismo canal de override + doble atestación para la cascada de autoridad del fondo sobre las asignaciones de rol dentro de sus cuentas hijas — sin abrir un canal nuevo.
- **Bloquea a:** el adaptador de venta a fondos de inversión (diferido, sin fecha).
- **Enlazado, no reimplementado:** `marketplace-cajas-negras` (ADR-0099) + `fit-to-portfolio-search` para el funnel opcional de exploración de terceros — no forma parte de este cimiento.
- **Contrato de Integración UI (ADR-0117) — Superficie propia:** panel de cuentas hijas del fondo + indicador de jerarquía en el panel de cuenta de la hija.
