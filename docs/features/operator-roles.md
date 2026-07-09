# Operator Roles (Roles de Operador a la Carta)

> 🟡 **Parcial** 2026-07-08 · Orden de trabajo [STORY-044](../execution/STORY-044-operator-roles.md). Catálogo de roles a la carta (`operator_roles`) + asignaciones operador↔rol (`operator_assignments`) + ledger append-only de cambios (`operator_role_events`) + Core (matriz de capacidades, gate compuesto rol+ADR-0123, invariante "último admin en pie", gate de cuota de cuentas hijas) + orquestador + CLI de verificación, construidos e implementados. **Pendiente:** el transporte de red de la cascada de autoridad del fondo (relé ADR-0143), la integración cross-máquina completa de la doble atestación de #12, y la Cáscara Visual — diferidos a la construcción de la Cabina de Mando / Etapa 0.5.

**Carpeta:** `./features/operator-roles/`
**Estado:** 🟡 Parcial (cimiento #14 — Core/persistencia/orquestador/CLI cerrados, transporte de red de la cascada + UI diferidos)
**Última actualización:** 2026-07-08
**Decisión Arquitectónica Asociada:** ADR-0149 (cimiento #14) · ADR-0144 (substrato) · ADR-0137 (catálogo de puertos como universo de capacidades) · ADR-0123 (evaluador de permisos que este cimiento extiende) · ADR-0147 (cascada de autoridad, #12) · ADR-0020 (`access_token_id` Grupo II)

## ¿Qué es esta feature?

Dentro de **una sola** cuenta maestra, el mecanismo que permite al dueño crear **roles de operador a la carta** — no un catálogo fijo, sino roles con nombre libre y una matriz de capacidades con switches permitido/denegado, activables/desactivables en cualquier momento. Un operador (login humano o conexión MCP de un agente LLM) recibe uno de esos roles y queda limitado a lo que permite, sin que la cuenta deje de ser una cuenta maestra completa.

- **Problema:** hoy, dentro de una cuenta, quien tiene la contraseña tiene acceso total — no hay forma de dar a un becario acceso a 1-2 cosas, o a un CTO/CFO acceso a casi todo excepto lo que el dueño decida excluir, sin compartir la cuenta entera. Tampoco hay forma de que un agente LLM conectado vía MCP tenga un permiso distinto al de otro agente conectado a la misma cuenta.
- **Comportamiento observable:** el dueño crea un rol con nombre libre, marca capacidades permitidas/denegadas feature por feature, y lo asigna a un login humano o a una conexión LLM. Esa asignación es revocable/editable en cualquier momento.
- **Por qué:** es la palanca de venta a equipos (prop firms, fondos con analistas/risk managers) y la pieza que falta para delegar trabajo a agentes LLM con matices, en vez de un interruptor binario todo-o-nada.

## Comportamientos Observables

- Cuando el dueño (ADMIN-raíz de la cuenta) crea un rol → le da un nombre libre y marca, capacidad por capacidad (puerto de Feature, no módulo), permitido/denegado.
- Cuando el dueño asigna un rol a un login humano → ese usuario, al entrar, solo ve/opera lo que su rol permite; lo denegado ni siquiera se muestra como opción bloqueada confusa, se oculta o se marca claramente como fuera de alcance.
- Cuando el dueño asigna un rol a una conexión MCP de un agente LLM → esa conexión queda limitada exactamente igual que un humano con ese rol — mismo evaluador, mismo catálogo.
- Cuando el dueño edita o revoca un rol → el cambio aplica de inmediato a todos los operadores que lo tienen asignado.
- Cuando un cambio (reasignar, editar matriz, revocar) dejaría a la cuenta con **cero** operadores con la capacidad "gestionar operadores y roles" → se rechaza antes de comprometerse, con el motivo explícito ("esta cuenta se quedaría sin ningún admin"). Cualquier admin individual, incluido el original, se puede reasignar libremente **mientras quede al menos uno más**.
- Cuando la cuenta es una hija de una jerarquía de fondo (`master-account-hierarchy` #12) → la cuenta maestra raíz del fondo puede ver, cambiar o revocar cualquier asignación de rol de la hija (incluidas las de sus LLMs) vía el mismo canal de override + doble atestación de #12.
- Cuando el ADMIN-raíz de un fondo crea una cuenta maestra hija nueva → la cuota de cuentas hijas disponibles (`MAX_CHILD_ACCOUNTS`, `plan-tier-quota` #3) se descuenta; al llegar al límite, la creación se rechaza hasta que Drasus (el proveedor) ajuste la cuota de la suscripción.

## Restricciones

- **NUNCA (FIJO, corregido 2026-07-07)** un cambio de rol/matriz/asignación se completa si deja a la cuenta con **cero operadores** con la capacidad "gestionar operadores y roles" — invariante de "último admin en pie". Esto NO congela a una persona: el `owner_id` raíz es el primer admin por defecto, puede designar otros admins, y cualquier admin individual (incluido el original) es reasignable a otro rol siempre que quede al menos uno más con esa capacidad tras el cambio. El guardarraíl protege la capacidad, no a la persona — también bloquea editar la matriz del rol ADMIN para quitarle esa capacidad si eso deja la cuenta sin nadie que la tenga.
- **NUNCA** un operador sin la capacidad "gestionar operadores y roles" (ADMIN) vigente crea una cuenta maestra hija nueva bajo un fondo.
- **NUNCA** se exceden las cuentas hijas permitidas por la cuota de suscripción (`plan-tier-quota` #3); el límite lo fija el proveedor, nunca el fondo mismo.
- **NUNCA** la cascada de autoridad del fondo sobre asignaciones de rol de una hija viaja por un canal distinto al relé genérico + doble atestación de `master-account-hierarchy` (#12).
- **NUNCA** se gatea a nivel de módulo — la unidad real es el puerto de Frontera Pública de cada Feature (ADR-0137); el módulo es solo una plantilla de conveniencia al crear un rol, nunca el mecanismo de aplicación.
- **NUNCA** un agente LLM opera sin un rol explícito asignado — no hereda automáticamente ningún permiso por el solo hecho de conectarse vía MCP (refuerza, no relaja, la regla de bloqueo por defecto de ADR-0123).

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| ROLE_TEMPLATES_ENABLED | true | true/false | Ofrece plantillas de rol sugeridas (ej. "Analyst") como atajo de creación | CONFIG |
| MAX_CHILD_ACCOUNTS | (definido en `plan-tier-quota` #3) | según tier | Cuántas cuentas maestras hijas puede crear un fondo — fijado por Drasus, no por el fondo | CONFIG (proveedor) |
| ROLE_CHANGE_AUDIT_REQUIRED | true | true/false | Toda creación/edición/revocación de rol queda en el audit-log | FIJO |
| ROLE_CACHE_TTL | 1 h | 5 min – 24 h | Cuánto vale en local el veredicto de rol/capacidades cacheado antes de revalidar contra la Cabina de Mando (mismo patrón que `IDENTITY_CACHE_TTL` de `central-identity`) | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** evaluador de permisos extendido — dado `(operador, rol asignado, puerto de Feature invocado, pipeline/institutional_tag)`, decide conceder o denegar. Extiende la función pura de `agentic-mcp-gateway`/ADR-0123 con el insumo de rol; resolución de si una capacidad está en la matriz del rol. Segunda función pura, invocada antes de comprometer cualquier cambio de matriz/asignación: dado el catálogo de roles y asignaciones vigente más el cambio propuesto, calcula si el resultado deja ≥1 operador con la capacidad "gestionar operadores y roles" — invariante "último admin en pie".
- **Shell (Infraestructura):** persistencia central del catálogo de roles/matrices y de asignaciones operador↔rol (Cabina de Mando), caché local con `ROLE_CACHE_TTL` para operación offline, verificación de la cuota de cuentas hijas contra `plan-tier-quota`, envío/recepción de comandos de cascada de autoridad vía el relé genérico.
- **Frontera Pública:** puerto de evaluación de permisos que cualquier módulo/feature puede consultar antes de aceptar una llamada de un operador con rol asignado — mismo punto de consulta que ya usa ADR-0123, con un insumo más.

## Ciclo de Vida de la Feature — Operator Roles

### Entrada
Definición de rol (nombre + matriz de capacidades) del dueño, asignación operador↔rol, y la llamada entrante de un operador a un puerto de Feature.

### Proceso
Resuelve si el operador tiene un rol asignado, si ese rol permite la capacidad invocada, y si es una cuenta hija, si el fondo no la ha overrideado.

### Salida
Un veredicto permitir/denegar por llamada, y el catálogo de roles/asignaciones vigente de la cuenta, auditable.

## Tareas (TTRs)

- **TTR-001:** Catálogo de roles a la carta por cuenta maestra (Core: matriz de capacidades; Shell: persistencia).
- **TTR-002:** Asignación operador↔rol (humano y agente LLM bajo el mismo mecanismo) y evaluador de permisos extendido sobre ADR-0123.
- **TTR-003:** Ancla ADMIN-raíz inmutable + gate de creación de cuentas hijas contra la cuota de `plan-tier-quota` (#3).
- **TTR-004:** Cascada de autoridad del fondo sobre asignaciones de rol de cuentas hijas, vía el relé genérico + doble atestación de #12.

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `identity_in` | `AccountIdentity` (plomería, ADR-0144) | Input | `1` | Identidad de cuenta dueña del catálogo de roles, producida por `central-identity`. |
| `role_catalog_out` | Catálogo de roles y matrices (tipo técnico nuevo — plomería, ADR-0149) | Output | `1..N` | Roles custom definidos por la cuenta con su matriz de capacidades. |
| `permission_verdict_out` | Veredicto de permiso (tipo técnico nuevo — plomería, ADR-0149) | Output | `1..N` | Permitir/denegar por llamada de operador; consumido por el evaluador de `agentic-mcp-gateway` y por el Shell de la UI. |
| `authority_override_in` | Comando de cascada de autoridad (reutiliza el tipo de `master-account-hierarchy` #12) | Input | `0..1` | Cambio/revocación de asignación de rol ordenado por la cuenta maestra raíz del fondo. |

> Tipos técnicos nuevos del cimiento #14, registrados en el catálogo de ADR-0137 vía la enmienda de ADR-0149. Nombres canónicos de `struct` los fija el ingeniero.

## Cáscara Visual (Thin Shell)

> Pendiente de la Etapa 0.5 (UI-Designer). Superficie prevista: panel "roles y operadores" en el cajón de ajustes de cada cuenta maestra, con creación de rol (matriz de switches por Feature, agrupable por plantilla de módulo), asignación a logins/conexiones MCP, y — para cuentas hija — el panel de cascada visible desde la cuenta del fondo. El Architect NO rellena esta sección.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016), re-escopado por ADR-0143:** el catálogo de roles y las asignaciones **viven** en la Cabina de Mando (fuente de verdad — es donde se resuelve la identidad de operador y la cascada de autoridad, que cruza máquinas de distintas personas). El motor local nunca es fuente de verdad: cachea el veredicto de rol/capacidades vigente (`ROLE_CACHE_TTL`, mismo patrón que `IDENTITY_CACHE_TTL` de `central-identity`) y el **evaluador de permisos corre en local** contra esa caché — ninguna acción del usuario espera un viaje de ida y vuelta al servidor, y la app sigue operando si se corta la conexión hasta que vence la caché.
- **Inundación de Fundaciones (ADR-0020):** Grupo I completo + **Perfil D (Ops/Auditoría)**: Identidad(I) + Soberanía(II: `owner_id`, `access_token_id` como ancla de atribución de operador) + subset V (`compliance_status_id` si aplica a auditoría de cambios de rol).

## Persistencia (Inundación de Fundamentos — ADR-0020)

Tres piezas: (1) catálogo de roles por cuenta — tabla **mutable** (`row_version`) con Grupo I + Perfil D; campos propios (marcados): nombre del rol, matriz de capacidades (JSON, `json_valid`) — **sin flag de inmutabilidad por rol** (corregido 2026-07-07: la protección no es un flag estático sobre un rol/persona, es una validación dinámica en el Core — antes de comprometer cualquier UPDATE de matriz o de asignación, se recalcula si queda ≥1 operador con la capacidad "gestionar operadores y roles"; si no, se rechaza la transacción). (2) asignaciones operador↔rol — tabla mutable, campos propios: `access_token_id` del operador, tipo (`HUMAN`/`AGENT`), rol asignado. (3) registro append-only de cambios de rol/asignación (auditoría, `event_sequence_id UNIQUE`), mismo patrón que `attested_track_records` de #10. `STRICT`, UUIDv7. Multi-tenancy real solo en la Cabina de Mando: se reutiliza `owner_id`, prohibido calcar `tenant_id` (ADR-0144).

## Dependencias y Bloqueantes

- **Depende de:** `central-identity` (#1, `owner_id`/`access_token_id`), `agentic-mcp-gateway` (evaluador de permisos que se extiende), `plan-tier-quota` (#3, cuota `MAX_CHILD_ACCOUNTS`), `master-account-hierarchy` (#12, canal de override + doble atestación reutilizado para la cascada).
- **Distinto de (no confundir):** `master-account-hierarchy` (#12, gatea entre cuentas maestras separadas) — este cimiento gatea **dentro** de una sola cuenta. `saas-gateway`/ADR-0123 (RBAC por tier de suscripción o riesgo de pipeline, un solo operador asumido) — este cimiento añade el operador como dimensión nueva, no lo reemplaza.
- **Bloquea a:** ninguna feature de dominio directamente; es una compuerta adicional sobre features ya existentes vía su puerto público.
- **Contrato de Integración UI (ADR-0117) — Superficie propia:** panel "roles y operadores" en ajustes; SVF: crear un rol y asignarlo dispara el gate real vía `public_interface`, un operador con rol restringido ve la denegación real al intentar una acción fuera de su matriz; tras recargar, el catálogo y las asignaciones persisten.
