# STORY-010 · Agentic MCP Gateway — núcleo MCP + evaluador de permisos

| Campo | Valor |
|---|---|
| **ID** | STORY-010 |
| **Título** | `agentic-mcp-gateway` — servidor MCP, evaluador de permisos y auditoría de procedencia |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | 1 |
| **Estado** | Completada |
| **Responsable** | Rust-Engineer (Sonnet) · Modo Docente · auditará Tech-Lead |
| **Creada** | 2026-06-20 |
| **Completada** | 2026-06-20 |

## 1. Especificación de origen (qué specs implementa)

- **Feature:** [`agentic-mcp-gateway`](../features/agentic-mcp-gateway.md)
- **ADR-0123** — Cabina Dual: MCP como Shell-cliente más sobre la misma `public_interface`.
- **TTRs en scope para EPIC-0:**
  - **TTR-001 (parcial):** servidor MCP activo al arrancar `drasus start`; expone como herramientas las operaciones de `shared` ya implementadas (clock, telemetría, estado de jobs).
  - **TTR-002 (completo):** evaluador de permisos puro por pipeline + `institutional_tag`.
  - **TTR-003 (storage only):** persistencia del estado `production_override_active` en SQLite; sin UI (llega con SPIKE-006/EPIC-0 Flutter).
  - **TTR-005 (completo):** cada decisión de permiso queda registrada en tabla `permission_decisions` con procedencia agente + campos de dominio propio.
- **TTR-004 diferido:** aceptación de términos SaaS depende de `saas-gateway` → EPIC-9+.
- **ADR-0020 V2 — Perfil D (Ops/Auditoría):** Grupo I (6 campos universales) + Grupo II (Soberanía: `owner_id`, `institutional_tag`) + Grupo IV (Hardware: `node_id`, `process_id`) + 4 campos de dominio propio documentados en la spec (`agent_session_id`, `requested_scope`, `permission_outcome`, `production_override_active`).

## 2. Objetivo (una frase llana)

Construir el núcleo del Gateway MCP de Drasus: un servidor MCP que arranca junto al motor, expone como herramientas las operaciones de `shared`, evalúa permisos por pipeline antes de ejecutar cualquier llamada, y registra cada decisión en un log de auditoría inmutable con procedencia de agente identificada.

## 3. Agentes y Modo de Acompañamiento (ADR-0120)

| Agente | Etapa del pipeline | Depende de | Modo |
|---|---|---|---|
| Rust-Engineer | Etapa 2 — Implementación Core | ninguno | Docente |

**Modo Docente (ADR-0122):** implementa cada bloque completo, luego explica decisiones de diseño con profundidad cero-conocimiento antes de avanzar. Documenta TODO en `docs/lessons/rust/STORY-010-agentic-mcp-gateway.md` (ADR-0124).

## 4. Instrucciones de despacho por agente (la spec ejecutable)

### 4.1 Rust-Engineer

```
Eres el Rust-Engineer de Drasus Engine.

PASO OBLIGATORIO ANTES DE ACTUAR:
1. Lee `/var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine/.claude/skills/base/SKILL.md` completo. Declara "[base/SKILL.md leído]".
2. Lee `/var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine/.claude/skills/rust-engineer/SKILL.md` completo. Declara "[rust-engineer/SKILL.md leído]".
3. Tu Modo de Acompañamiento es **Docente** (ADR-0122): implementas cada bloque completo, luego explicas las decisiones de diseño con profundidad cero-conocimiento. Documenta TODO en `docs/lessons/rust/STORY-010-agentic-mcp-gateway.md`.

DIRECTORIO DE TRABAJO: /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine

Lee antes de empezar:
- La Orden de Trabajo completa: `docs/execution/STORY-010-agentic-mcp-gateway.md`
- La spec: `docs/features/agentic-mcp-gateway.md`
- ADR-0123: `docs/adr/ADR-0123.md`
- El public_interface actual de shared: `crates/shared/src/public_interface.rs`
- main.rs del binario app: `crates/app/src/main.rs`

---

## Contexto del proyecto

Drasus Engine — sistema de trading algorítmico en Rust + Flutter.
- Pipeline de 8 módulos. Patrón FCIS: Core (lógica pura, sin IO) + Shell (infraestructura).
- Todos los módulos se comunican SOLO a través de sus `public_interface`. El Gateway MCP es un Shell-cliente que llama a `public_interface`, nunca a internals.
- El crate `shared` ya tiene: clock, audit-log, telemetría, async-job-executor, worker-isolation-orchestrator.
- El binario `drasus start` ya arranca el motor. STORY-010 añade el MCP server como otra tarea async que `drasus start` lanza en background.

---

## Bloque 1 — Selección del crate MCP y scaffolding

**Primero: evalúa qué crate Rust usar para el servidor MCP.**

El protocolo MCP (Model Context Protocol) usa JSON-RPC 2.0 sobre stdio o HTTP+SSE. Opciones a evaluar:
- `rmcp` (crate oficial del proyecto modelcontextprotocol para Rust): si existe en crates.io con versión estable, úsalo.
- Si `rmcp` no existe o no tiene API estable: implementa el protocolo directamente sobre stdio con `serde_json` + `tokio::io` (MCP sobre stdio es el modo más simple: el servidor lee de stdin y escribe en stdout con mensajes JSON-RPC 2.0 delimitados por newline).

**Para EPIC-0, usa el transporte stdio** (el modo más simple del protocolo MCP). El servidor se inicia como un proceso hijo con stdin/stdout, que es exactamente cómo Claude Desktop y otros clientes MCP conectan servidores locales.

**Estructura de archivos en `crates/shared/src/`:**

```
domain/
  mcp_gateway.rs          ← Core puro: tipos + evaluador de permisos
orchestrator/
  mcp_server.rs           ← Shell: servidor MCP stdio, despacho de herramientas
persistence/
  mcp_gateway.rs          ← Shell: repositorio de permission_decisions
```

Añade los módulos al `lib.rs` de `shared` y re-exporta desde `public_interface.rs`.

La dependencia del crate MCP elegido va en `crates/shared/Cargo.toml`.

**Enseñanza después de este bloque:**
- Qué es el protocolo MCP y cómo funciona el transporte stdio (vs HTTP+SSE).
- Por qué EPIC-0 usa stdio (simplicidad, sin servidor HTTP).
- Cómo JSON-RPC 2.0 estructura las llamadas de herramienta (id, method, params, result/error).

---

## Bloque 2 — Core: evaluador de permisos (función pura)

Implementa `crates/shared/src/domain/mcp_gateway.rs`.

**Tipos del dominio:**

```rust
// Los 8 pipelines del sistema
pub enum Pipeline { Ingest, Generate, Validate, Incubate, Manage, Execute, Feedback, Withdraw }

// Etiqueta del objeto afectado (solo aplica a manage)
pub enum InstitutionalTag { Live, Demo }

// Resultado de la evaluación
pub enum PermissionOutcome { Granted, Denied { reason: String } }

// Solicitud de permiso completa
pub struct PermissionRequest {
    pub pipeline: Pipeline,
    pub institutional_tag: Option<InstitutionalTag>, // solo para Manage
    pub production_override_active: bool,
    pub agent_session_id: String,
    pub requested_scope: String, // ej. "ingest.submit_bar"
}

// Decisión registrable
pub struct PermissionDecision {
    // Grupo I (6 campos universales ADR-0020 V2)
    pub id: Uuid,
    pub created_at: i64,
    pub updated_at: i64,
    pub audit_hash: String,
    pub audit_chain_hash: String,
    pub event_sequence_id: i64,
    // Grupo II (Soberanía)
    pub owner_id: Option<String>,
    pub institutional_tag: Option<String>,
    // Grupo IV (Hardware)
    pub node_id: String,
    pub process_id: i64,
    // Dominio propio (fuera del catálogo canónico)
    pub agent_session_id: String,
    pub requested_scope: String,
    pub permission_outcome: String, // "granted" | "denied:<razón>"
    pub production_override_active: bool,
}
```

**Función pura del evaluador** (implementa la matriz de ADR-0123):

```rust
pub fn evaluate_permission(req: &PermissionRequest) -> PermissionOutcome {
    match req.pipeline {
        // Abiertos por defecto sin ningún gate
        Pipeline::Ingest | Pipeline::Generate | Pipeline::Validate
        | Pipeline::Incubate | Pipeline::Feedback => PermissionOutcome::Granted,

        // Condicionado por institutional_tag: Demo libre, Live exige interruptor
        Pipeline::Manage => {
            match req.institutional_tag {
                Some(InstitutionalTag::Live) if !req.production_override_active =>
                    PermissionOutcome::Denied { reason: "manage/live requiere production_override activo".into() },
                _ => PermissionOutcome::Granted,
            }
        },

        // Bloqueados por defecto salvo interruptor activo
        Pipeline::Execute | Pipeline::Withdraw => {
            if req.production_override_active {
                PermissionOutcome::Granted
            } else {
                PermissionOutcome::Denied {
                    reason: format!("{:?} bloqueado por defecto; activa production_override", req.pipeline)
                }
            }
        }
    }
}
```

Implementa también:
- `PermissionDecision::build(req, outcome, prev_hash, sequence_id, node_id, pid)` — construye la decisión con el Grupo I completo (incluyendo `audit_hash` sobre los campos del dominio propio).

**Enseñanza después de este bloque:**
- Por qué `evaluate_permission` es una función pura (sin IO, sin estado mutable): facilita testing exhaustivo, determinismo, y composición.
- Cómo el patrón `match` de Rust sobre enums cubre todos los casos sin `else` colgante (exhaustividad del compilador).
- Por qué `audit_hash` se calcula sobre los campos de dominio propio (agent_session_id + requested_scope + permission_outcome) y no sobre el Grupo I (que incluiría el hash mismo, creando circularidad).

---

## Bloque 3 — Shell: persistencia de permission_decisions

Implementa `crates/shared/src/persistence/mcp_gateway.rs` y la migración SQLite.

**Migración `migrations/0005_mcp_gateway.sql`:**

```sql
-- Tabla de decisiones de permiso del Gateway MCP (Perfil D — ADR-0020 V2)
CREATE TABLE IF NOT EXISTS permission_decisions (
    -- Grupo I: Identidad & Integridad (universal)
    id                       TEXT    PRIMARY KEY NOT NULL,
    created_at               INTEGER NOT NULL,
    updated_at               INTEGER NOT NULL,
    audit_hash               TEXT    NOT NULL,
    audit_chain_hash         TEXT    NOT NULL,
    event_sequence_id        INTEGER NOT NULL,
    -- Grupo II: Soberanía
    owner_id                 TEXT,
    institutional_tag        TEXT,
    -- Grupo IV: Hardware
    node_id                  TEXT    NOT NULL,
    process_id               INTEGER NOT NULL,
    -- Dominio propio (fuera del catálogo canónico — documentados en agentic-mcp-gateway.md)
    agent_session_id         TEXT    NOT NULL,
    requested_scope          TEXT    NOT NULL,
    permission_outcome       TEXT    NOT NULL, -- "granted" | "denied:<razón>"
    production_override_active INTEGER NOT NULL DEFAULT 0
);

-- Tabla de configuración del interruptor de producción (TTR-003 storage)
CREATE TABLE IF NOT EXISTS mcp_gateway_config (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
-- Valor inicial del interruptor (apagado por defecto, ADR-0123)
INSERT OR IGNORE INTO mcp_gateway_config (key, value) VALUES ('production_override_active', '0');
```

**Repositorio** con métodos:
- `append(pool, decision)` — inserta una decisión (append-only: sin UPDATE/DELETE, igual que audit_events).
- `get_production_override(pool)` → `bool` — lee el interruptor desde `mcp_gateway_config`.
- `set_production_override(pool, active: bool)` — el propietario activa/desactiva.
- `chain_tip(pool)` → `Option<(String, i64)>` — último `audit_chain_hash` + `event_sequence_id` para encadenar la siguiente decisión.

**Enseñanza después de este bloque:**
- Por qué `permission_decisions` es append-only igual que `audit_events` (las decisiones son forenses: nadie puede borrarlas para ocultar que el agente intentó acceder a producción).
- Por qué el interruptor de producción vive en una tabla de configuración clave-valor y no como una columna en `permission_decisions`.
- Diferencia entre una tabla de hechos (append-only, `permission_decisions`) y una tabla de estado (mutable, `mcp_gateway_config`).

---

## Bloque 4 — Shell: servidor MCP sobre stdio

Implementa `crates/shared/src/orchestrator/mcp_server.rs`.

El servidor MCP stdio:
1. Lee mensajes JSON-RPC 2.0 de stdin (un mensaje por línea).
2. Despacha según el `method`:
   - `initialize` → responde con capacidades del servidor + lista de herramientas disponibles.
   - `tools/list` → devuelve la lista de herramientas con sus esquemas JSON.
   - `tools/call` → evalúa permisos, ejecuta si `Granted`, registra la decisión.
3. Escribe la respuesta en stdout (JSON-RPC 2.0, una línea por respuesta).

**Herramientas expuestas en EPIC-0** (las operaciones de `shared` ya implementadas):
- `drasus.clock.now` — devuelve el timestamp actual del reloj del sistema.
- `drasus.jobs.list` — devuelve la lista de jobs en la cola (estado y progreso).
- `drasus.jobs.submit` — encola un nuevo job.
- `drasus.telemetry.latest` — devuelve las últimas N muestras de telemetría.

Para cada llamada:
1. Identifica el pipeline de la herramienta invocada (todas las de EPIC-0 son `Ingest`/`Feedback` en términos de permisos → siempre `Granted`).
2. Llama a `evaluate_permission` con el pipeline, el `institutional_tag` del objeto si aplica, y el estado actual de `production_override_active` (leído de DB).
3. Si `Granted`, ejecuta la herramienta vía `public_interface` de `shared`.
4. Registra la decisión en `permission_decisions` vía `append`.
5. Devuelve el resultado (o el motivo de denegación si `Denied`).

**Función pública a re-exportar:**
```rust
pub async fn run_mcp_server(pool: SqlitePool) -> Result<()>
```
Esta función es la que `drasus start` llama en un `tokio::spawn`.

Si usas el crate `rmcp`: implementa el servidor usando su API. Si implementas el protocolo directamente: usa `tokio::io::BufReader<tokio::io::Stdin>` + `tokio::io::stdout()` con `serde_json`.

**Enseñanza después de este bloque:**
- Cómo funciona JSON-RPC 2.0: estructura de petición (id, method, params) y respuesta (id, result/error).
- Por qué MCP usa stdio en modo local (es lo más simple: el cliente inicia el servidor como proceso hijo y lee/escribe sus pipes estándar — sin networking, sin TLS, sin auth de transporte).
- Cómo `tokio::spawn` permite que el servidor MCP y el motor corran concurrentemente en el mismo proceso sin bloquearse mutuamente.

---

## Bloque 5 — Integración en `drasus start`

Modifica `crates/app/src/main.rs` para que el subcomando `start` también lance el servidor MCP:

```rust
// Después de inicializar el pool y correr migraciones:
let mcp_pool = pool.clone();
tokio::spawn(async move {
    if let Err(e) = shared::public_interface::run_mcp_server(mcp_pool).await {
        eprintln!("MCP server error: {e}");
    }
});
println!("Servidor MCP activo (stdio).");
```

El servidor MCP corre en background; el shutdown por SIGTERM/SIGINT del proceso principal también detiene el servidor (al cerrar el proceso, los handles de stdin/stdout se cierran y el loop del servidor termina).

**Enseñanza después de este bloque:**
- Por qué clonamos el pool (SqlitePool es un pool de conexiones compartidas, barato de clonar: solo incrementa un contador de referencia).
- Por qué NO usamos `tokio::spawn` con `await` (eso bloquearía; `spawn` lanza la tarea en background y devuelve inmediatamente un `JoinHandle` que ignoramos aquí).
- Por qué el shutdown del proceso padre limpia el servidor MCP sin necesidad de coordinación explícita.

---

## Bloque 6 — Pruebas y lección formal

**Pruebas a escribir** (en `crates/shared/src/`):

| Test | Qué demuestra |
|---|---|
| `ingest_pipeline_is_always_granted` | Todos los pipelines abiertos devuelven Granted sin importar el interruptor |
| `manage_demo_is_granted_without_override` | manage + Demo → Granted |
| `manage_live_is_denied_without_override` | manage + Live sin interruptor → Denied |
| `manage_live_is_granted_with_override` | manage + Live con interruptor → Granted |
| `execute_is_denied_without_override` | execute sin interruptor → Denied |
| `execute_is_granted_with_override` | execute con interruptor → Granted |
| `permission_decision_persists_and_is_retrievable` | Append + query |
| `audit_chain_links_sequential_decisions` | La cadena hash encadena correctamente |
| `production_override_toggle_persists` | set_production_override → get_production_override |

**Lección formal:** `docs/lessons/rust/STORY-010-agentic-mcp-gateway.md` (ADR-0124).
Estructura: enlace a la Orden + sección `## Concepto` (una subsección por cada concepto enseñado en los bloques 1-5) + `## Trucos de Senior`.

---

## Criterio de cierre

Al terminar, reporta al Tech-Lead:
1. Artefactos creados (rutas).
2. Resultado de CADA criterio de §5 (tabla criterio → test → resultado).
3. Cobertura por archivo nuevo.
4. Decisiones de diseño no especificadas (ej. crate MCP elegido + justificación).
```

**Plan de Implementación / Revisión** (Rust-Engineer, Modo Docente, 2026-06-20):

| Bloque | Archivo(s) creados/modificados | Estado |
|---|---|---|
| 1 — Crate MCP + scaffolding | `Cargo.toml` shared, `domain/mod.rs`, `orchestrator/mod.rs`, `persistence/mod.rs` | ✅ |
| 2 — Core: evaluador de permisos | `domain/mcp_gateway.rs` | ✅ |
| 3 — Shell: persistencia | `persistence/mcp_gateway.rs`, `migrations/0005_mcp_gateway.sql` | ✅ |
| 4 — Shell: servidor MCP stdio | `orchestrator/mcp_server.rs` | ✅ |
| 5 — Integración drasus start | `crates/app/src/main.rs` | ✅ |
| 6 — Pruebas + lección | pruebas embebidas en dominio y persistencia, `docs/lessons/rust/STORY-010-agentic-mcp-gateway.md` | ✅ |

**Crate MCP elegido:** `rmcp` 1.7.0 (SDK oficial, estable, 13 M de descargas). Features: `server`, `transport-io`, `macros`, `schemars`. Se descartó implementación manual sobre JSON-RPC porque el SDK oficial existe, es estable y reduce el código boilerplate a cero.

## 5. Criterio de aceptación (cada criterio ↔ su prueba)

| # | Criterio verificable | Prueba que lo demuestra |
|---|---|---|
| 1 | `cargo build --workspace` limpio | — (build) |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` 0 warnings | — (clippy) |
| 3 | Evaluador: pipelines abiertos → siempre Granted | `ingest_pipeline_is_always_granted` |
| 4 | Evaluador: manage+Demo → Granted sin interruptor | `manage_demo_is_granted_without_override` |
| 5 | Evaluador: manage+Live → Denied sin interruptor | `manage_live_is_denied_without_override` |
| 6 | Evaluador: manage+Live → Granted con interruptor | `manage_live_is_granted_with_override` |
| 7 | Evaluador: execute → Denied sin interruptor | `execute_is_denied_without_override` |
| 8 | Evaluador: execute → Granted con interruptor | `execute_is_granted_with_override` |
| 9 | Decisión persiste y es recuperable | `permission_decision_persists_and_is_retrievable` |
| 10 | Cadena de auditoría encadena decisiones correctamente | `audit_chain_links_sequential_decisions` |
| 11 | Interruptor de producción persiste en DB | `production_override_toggle_persists` |
| 12 | FCIS: `domain/mcp_gateway.rs` sin imports de IO/DB | grep de verificación |
| 13 | Tests previos (STORY-001-009) siguen verdes | `cargo test --workspace` |
| 14 | Lección en `docs/lessons/rust/STORY-010-agentic-mcp-gateway.md` | — (inspección) |

## 6. Comandos de validación (para el usuario — copy/paste)

```bash
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p shared -- mcp --nocapture
cargo llvm-cov --workspace --summary-only
grep -n "sqlx\|tokio\|std::io" crates/shared/src/domain/mcp_gateway.rs
```

## 7. Registro de ejecución (bitácora cronológica)

| Fecha | Evento |
|---|---|
| 2026-06-20 | Sesión Docente completada. 6 bloques implementados. 12 pruebas MCP en verde. `cargo clippy -D warnings` limpio. Cobertura: domain 96.15%, persistence 100%. Lección formal creada. |

## 8. Pendientes derivados / decisiones

- **TTR-003 UI (interruptor de producción):** el estado persiste desde STORY-010; el control visual (botón en "Cabina Dual") llega con SPIKE-006/ADR-0117 (Panel Operativo Fundacional).
- **TTR-004 (SaaS terms):** diferido a EPIC-9+ junto con `saas-gateway`.
- **Herramientas de módulos reales (EPIC-1+):** cuando `ingest` tenga su `public_interface` real, se añaden herramientas MCP de ingest. El servidor MCP ya está listo para recibirlas.
- **Transporte HTTP+SSE (EPIC-8+):** para el modo SaaS, el servidor MCP añade transporte HTTP además de stdio. No es trabajo de EPIC-0.
