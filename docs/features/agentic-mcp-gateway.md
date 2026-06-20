> 🟡 **Parcial** 2026-06-20 · TTR-001 (parcial), TTR-002, TTR-003 (storage), TTR-005 implementados · TTR-001 UI herramientas reales (EPIC-1+) y TTR-004 (SaaS terms) pendientes · Orden [STORY-010](../execution/STORY-010-agentic-mcp-gateway.md)

# Agentic MCP Gateway (Cabina Dual)

**Carpeta:** `./features/agentic-mcp-gateway/`
**Estado:** En Diseño
**Última actualización:** 2026-06-17
**Decisión Arquitectónica Asociada:** ADR-0123 (Cabina Dual — Acceso Agéntico vía MCP con Permisos Graduados por Riesgo de Pipeline)

## 1. ¿Qué es esta feature?
Es la puerta de control que expone la `public_interface` de los 8 módulos del pipeline a un agente LLM externo (ej. Claude) conectado vía MCP, con la misma autoridad que hoy tiene el cliente Flutter o el cliente Headless gRPC — pero con permisos graduados por defecto según el riesgo del pipeline invocado.

**Problema que resuelve:** Hoy, automatizar el trabajo tedioso de descubrimiento (lanzar barridos, iterar diseños de estrategia, revisar reportes) exige que el usuario opere manualmente la interfaz paso a paso. El usuario quiere poder delegar ese trabajo a un agente — su propio "copiloto de trading" — sin que eso abra una puerta sin control hacia el capital real.

**Por qué la necesitamos:** "Software por resultados": el usuario decide, tarea por tarea, si la hace él desde la interfaz o la delega a un agente. Ninguna de las dos vías sustituye a la otra — es Cabina Dual, no migración a un solo canal.

## 2. Comportamientos Observables
- [ ] El usuario conecta un agente LLM a Drasus Engine vía MCP. El agente ve disponibles, sin pedir permiso adicional, las herramientas de `ingest`, `generate`, `validate`, `incubate` y `feedback`.
- [ ] El agente pide a `manage` rebalancear un portafolio con `institutional_tag = Demo`. La operación se ejecuta sin fricción.
- [ ] El agente pide a `manage` rebalancear un portafolio con `institutional_tag = Live` mientras el interruptor de producción está apagado. La llamada se rechaza con el mismo código de denegación que usa el Gateway SaaS para un token sin permisos.
- [ ] El agente intenta invocar cualquier frontera de `execute` o `withdraw` mientras el interruptor está apagado. La llamada se rechaza incondicionalmente.
- [ ] El propietario activa el interruptor de producción desde la interfaz local. A partir de ese momento, las mismas llamadas de `manage(Live)`, `execute` y `withdraw` se conceden.
- [ ] En modo SaaS, el propietario intenta activar el interruptor sin haber aceptado los términos de riesgo de delegar producción a un LLM. El sistema bloquea la activación y muestra el texto de aceptación pendiente.
- [ ] El propietario cierra la conexión MCP en cualquier momento. El agente pierde acceso de inmediato; la interfaz Flutter sigue operando con control 100% manual, sin degradación.
- [ ] Cada acción ejecutada por el agente queda registrada en el log de auditoría con un origen identificado como "agente", distinguible de una acción humana en el mismo registro.

## 3. Restricciones
- PROHIBIDO crear un camino de validación o de lógica de negocio exclusivo para el canal MCP: consume la misma `public_interface` que ya usan Flutter y el gRPC Headless (FCIS, Soberanía de Datos).
- NUNCA una conexión MCP nace con permiso sobre `execute` o `withdraw` por defecto.
- NUNCA se concede una llamada de `manage` sobre un portafolio `institutional_tag = Live` sin que el interruptor de producción esté activo.
- NUNCA el interruptor de producción se activa en modo SaaS sin que el usuario haya aceptado expresamente los términos de riesgo correspondientes.
- NUNCA una acción ejecutada por el agente queda indistinguible de una acción humana en el registro de auditoría.
- El cierre del canal MCP por el propietario es inmediato y no requiere reinicio de la interfaz local.

## 4. Parámetros Configurables
| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| PRODUCTION_OVERRIDE | desactivado | activado/desactivado | Interruptor que habilita al agente sobre `manage(Live)`, `execute`, `withdraw` | CONFIG |
| TERMS_ACCEPTED_SAAS | false | true/false | Aceptación expresa de los términos de riesgo en modo SaaS; precondición para `PRODUCTION_OVERRIDE` en esa cuenta | [FIJO] una vez aceptado, no se reescribe — se revoca por separado |
| MCP_RATE_LIMIT | 120 req/min | 10-1000 | Límite de llamadas del agente por ventana temporal, reutiliza el mecanismo del Gateway SaaS | CONFIG |
| AGENT_SESSION_EXPIRY | 60 minutos | 5-1440 minutos | Vigencia de la sesión MCP antes de requerir reautenticación | CONFIG |

## 5. Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** Evaluador de permisos: dado el pipeline invocado, el `institutional_tag` del objetivo (si aplica) y el estado de `PRODUCTION_OVERRIDE`, decide conceder o denegar. Función pura, sin IO.
- **Shell (Infraestructura):** Servidor MCP que traduce las herramientas invocables por el agente hacia la `public_interface` real de cada módulo; persistencia del estado del interruptor y del log de auditoría; reutiliza el servidor de autenticación/RBAC del Gateway SaaS (`saas-gateway`).
- **Frontera Pública:** Puerto de evaluación de permisos que cualquier módulo puede consultar antes de aceptar una llamada proveniente del canal MCP.

## 6. Ciclo de Vida de la Feature — Agentic MCP Gateway

### Entrada
- Llamada entrante del agente vía MCP: pipeline objetivo, frontera invocada, parámetros.
- Estado vigente de `PRODUCTION_OVERRIDE` y de `TERMS_ACCEPTED_SAAS` (si aplica).
- `institutional_tag` del objeto afectado, cuando la llamada es sobre `manage`.

### Proceso
- Identifica a qué pipeline pertenece la frontera invocada.
- Si el pipeline está en la lista abierta (`ingest`, `generate`, `validate`, `incubate`, `feedback`), concede.
- Si es `manage`, consulta el `institutional_tag` del objetivo: Demo concede, Live exige `PRODUCTION_OVERRIDE` activo.
- Si es `execute` o `withdraw`, exige `PRODUCTION_OVERRIDE` activo sin excepción.
- Registra el resultado (concedido/denegado) y la procedencia agente en el log de auditoría.

### Salida
- La llamada se enruta hacia el módulo real y su resultado vuelve al agente, o se devuelve un rechazo con el motivo (pipeline bloqueado, interruptor apagado, términos no aceptados).
- Registro inmutable de la decisión de permiso.

### Contextos de Uso
**Contexto 1: Descubrimiento delegado**
- El usuario delega a su agente todo el ciclo de `ingest` → `feedback`, dedicando su atención solo a las decisiones que de verdad quiere tomar él.
**Contexto 2: Producción supervisada**
- El propietario activa el interruptor de forma temporal para que el agente ejecute una tarea puntual sobre producción, y lo desactiva al terminar.

## 7. Tareas (TTRs)

### TTR-001: Servidor MCP expone la frontera pública de los módulos como herramientas
* **¿Cuál es el problema?** Un agente LLM no puede invocar directamente la `public_interface` interna de Rust; necesita un protocolo estándar de herramientas.
* **¿Qué tiene que pasar?** El agente, al conectarse vía MCP, ve listadas las operaciones disponibles de cada módulo según la matriz de permisos vigente en ese momento.
* **¿Cómo sé que está hecho?**
  - [ ] Conecto un agente MCP y veo las herramientas de `ingest`/`generate`/`validate`/`incubate`/`feedback` disponibles de inmediato.
  - [ ] Las herramientas de `execute`/`withdraw` no aparecen listadas mientras el interruptor está apagado.
* **¿Qué no puede pasar?** El servidor MCP no puede exponer una operación que no exista ya en la `public_interface` real del módulo.

### TTR-002: Evaluador de permisos por pipeline e `institutional_tag`
* **¿Cuál es el problema?** Hay que decidir, en cada llamada, si el agente tiene permiso, sin duplicar lógica de negocio.
* **¿Qué tiene que pasar?** Antes de enrutar cualquier llamada del canal MCP, el sistema evalúa el pipeline de destino y, si es `manage`, el `institutional_tag` del objetivo.
* **¿Cómo sé que está hecho?**
  - [ ] Una llamada a `manage` sobre un portafolio Demo se concede sin el interruptor activo.
  - [ ] La misma llamada sobre un portafolio Live se rechaza sin el interruptor activo.
* **¿Qué no puede pasar?** No puede existir una ruta de `manage` que evite esta evaluación.

### TTR-003: Interruptor de activación de producción
* **¿Cuál es el problema?** El propietario necesita una forma explícita y reversible de delegar autoridad sobre producción, sin que sea el comportamiento de fábrica.
* **¿Qué tiene que pasar?** El interruptor está oculto/apagado por defecto; el propietario lo activa desde la interfaz local, y puede desactivarlo en cualquier momento.
* **¿Cómo sé que está hecho?**
  - [ ] Activo el interruptor y una llamada antes rechazada ahora se concede.
  - [ ] Lo desactivo y la misma llamada vuelve a rechazarse de inmediato.
* **¿Qué no puede pasar?** El interruptor no puede activarse por una llamada del propio agente — solo el propietario, desde su interfaz, lo controla.

### TTR-004: Aceptación de términos de riesgo en modo SaaS
* **¿Cuál es el problema?** En modo SaaS, delegar producción a un LLM no puede ser una decisión tomada a la ligera ni revertible solo por el operador de la plataforma.
* **¿Qué tiene que pasar?** El interruptor de producción no existe para una cuenta SaaS hasta que el usuario acepta expresamente el texto de riesgo correspondiente.
* **¿Cómo sé que está hecho?**
  - [ ] Intento activar el interruptor sin haber aceptado los términos y el sistema lo bloquea, mostrando el texto pendiente.
  - [ ] Tras aceptar, el interruptor queda disponible para esa cuenta.
* **¿Qué no puede pasar?** No puede inferirse una aceptación implícita (ej. por seguir usando la plataforma).

### TTR-005: Auditoría de procedencia agente vs humano
* **¿Cuál es el problema?** Sin trazabilidad, una acción del agente y una acción humana serían indistinguibles en el historial.
* **¿Qué tiene que pasar?** Cada entrada de auditoría originada por el canal MCP queda marcada con su procedencia y la sesión del agente que la disparó.
* **¿Cómo sé que está hecho?**
  - [ ] Reviso el log de auditoría y cada acción del agente está marcada como tal, con su `agent_session_id`.
* **¿Qué no puede pasar?** No puede existir una entrada de auditoría sin procedencia identificada.

## 8. Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** El canal MCP corre contra el mismo Core local; en modo SaaS hereda las garantías de `saas-gateway`.
- **Inundación de Fundaciones (ADR-0020 V2): Perfil D (Ops / Auditoría)** — Identidad (I) + Soberanía (II) + Hardware (IV).

  | Categoría | Campo | Descripción |
  | :--- | :--- | :--- |
  | **I. Identidad** | `id` | Identificador único de la decisión de permiso |
  | | `created_at` | Timestamp de la llamada |
  | | `updated_at` | Timestamp de última modificación del registro |
  | | `audit_hash` | Hash forense de la llamada (pipeline + parámetros + resultado) |
  | | `audit_chain_hash` | Hash encadenado del historial de decisiones de permiso |
  | | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
  | **II. Soberanía** | `owner_id` | Propietario que controla el interruptor de producción |
  | | `institutional_tag` | Entorno del objeto afectado (Live / Demo) cuando aplica |
  | **IV. Hardware** | `node_id` | Host donde corre el Gateway MCP |
  | | `process_id` | Proceso de la sesión del agente |

  Campos propios de la feature (fuera del catálogo canónico, específicos de este dominio): `agent_session_id` (sesión MCP del agente conectado), `requested_scope` (pipeline/frontera invocada), `permission_outcome` (concedido/denegado), `production_override_active` (estado del interruptor en el momento de la llamada).

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador de la decisión de permiso |
| | `created_at` | Timestamp de la llamada |
| | `updated_at` | Timestamp de última modificación |
| | `audit_hash` | Hash forense de la llamada |
| | `audit_chain_hash` | Hash encadenado del historial |
| | `event_sequence_id` | Secuencia de recuperación |
| **II. Soberanía** | `owner_id` | Propietario del interruptor |
| | `institutional_tag` | Entorno del objeto afectado (Live/Demo) |
| **IV. Hardware** | `node_id` | Host del Gateway MCP |
| | `process_id` | Proceso de la sesión del agente |
| **Dominio propio** | `agent_session_id` | Sesión MCP que originó la llamada |
| | `requested_scope` | Pipeline/frontera invocada |
| | `permission_outcome` | Concedido / Denegado |
| | `production_override_active` | Estado del interruptor en el momento de la llamada |

**Rastro de Evidencia:** Emite a `feedback` cada denegación y cada activación/desactivación del interruptor de producción, para detectar patrones de uso anómalo del canal agéntico.

## 9. Dependencias y Bloqueantes
**Consumido por:** `ingest`, `generate`, `validate`, `incubate`, `manage`, `execute`, `feedback`, `withdraw` (los 8 módulos del pipeline exponen su `public_interface` a través de este Gateway).
**Depende de:** [`saas-gateway`](../moonshots/saas-gateway.md) (reutiliza autenticación, RBAC y rate limiting), [`remote-portfolio-access-protocol`](./remote-portfolio-access-protocol.md) (patrón de auditoría de acceso de referencia), [`audit-log`](./audit-log.md).
**Bloqueantes:** Ninguno — la `public_interface` de los 8 módulos ya existe; esta feature añade la capa de permisos sobre ella.

## Contrato de Integración UI
**Superficie propia:** sección "Cabina Dual" en el Panel Operativo Fundacional, donde el propietario ve el estado de la conexión MCP, activa/desactiva `PRODUCTION_OVERRIDE`, y revisa el log de procedencia agente vs humano.
- SVF: el interruptor de producción dispara un cambio real de permiso vía `public_interface`; la sección muestra en vivo si hay un agente conectado y su última acción; tras recargar, el estado del interruptor y el historial de decisiones siguen visibles (persistidos).
