# Lección: Agentic MCP Gateway

> Story: [STORY-010 — Agentic MCP Gateway](../../execution/STORY-010-agentic-mcp-gateway.md)

---

## Concepto

### 1. El Protocolo MCP y el transporte stdio

**Qué es MCP.** El Model Context Protocol (MCP) es un protocolo estándar creado por Anthropic para que los modelos de lenguaje (LLMs) puedan invocar herramientas externas de forma estructurada. Un cliente MCP (por ejemplo, Claude en Claude Desktop) se conecta a un servidor MCP y le pregunta qué herramientas tiene disponibles; luego puede llamar a cualquiera de ellas con parámetros tipados.

**Por qué stdio para EPIC-0.** MCP admite dos transportes: stdio (stdin/stdout) y HTTP+SSE. El transporte stdio es el más simple: el cliente lanza el servidor como un proceso hijo, y la comunicación ocurre a través de los pipes estándar — sin networking, sin TLS, sin autenticación de transporte. Es el modo local de facto y el que todos los clientes MCP soportan de fábrica. HTTP+SSE se reserva para el modo SaaS (EPIC-8+, ADR-0123).

**JSON-RPC 2.0 por debajo.** Aunque `rmcp` abstrae el protocolo, conviene saber que MCP usa JSON-RPC 2.0. Cada mensaje tiene la forma:
```json
// Petición del cliente:
{ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": { "name": "drasus_clock_now", "arguments": {} } }

// Respuesta del servidor:
{ "jsonrpc": "2.0", "id": 1, "result": { "content": [{ "type": "text", "text": "1750000000000000000" }] } }
```
Los tres métodos centrales son `initialize` (handshake), `tools/list` (catálogo de herramientas) y `tools/call` (invocación).

**Crate elegido: `rmcp` 1.7.0.** Es el SDK oficial de `modelcontextprotocol` para Rust, con 13 M de descargas y versión estable. Features activadas:
- `server` → `ServerHandler`, tipos del protocolo, `transport-async-rw`.
- `transport-io` → `stdio()`, el transporte sobre stdin/stdout de Tokio.
- `macros` → `#[tool]`, `#[tool_router]`, `#[tool_handler]`.
- `schemars` → generación automática del JSON Schema de los parámetros de cada herramienta.

Código real en `crates/shared/src/orchestrator/mcp_server.rs`:
```rust
let server = service.serve(stdio()).await?;
server.waiting().await?;
```

---

### 2. `#[tool_router(server_handler)]` — cómo el macro genera el servidor

**El problema que resuelve.** Sin el macro, implementar un servidor MCP exige escribir a mano el `ServerHandler` (el trait que define `list_tools` y `call_tool`), parsear los parámetros de cada herramienta desde JSON, y despachar a la función correcta. El macro `#[tool_router(server_handler)]` genera todo eso a partir de las anotaciones en los métodos.

**Cómo funciona.** Se aplica al bloque `impl` del struct del servidor. Dentro, cada método anotado con `#[tool(...)]` se convierte en una herramienta MCP. El macro:
1. Registra el nombre de la herramienta (por defecto = nombre del método).
2. Genera el JSON Schema de los parámetros a partir del struct `Parameters<T>`, donde `T` implementa `schemars::JsonSchema` y `serde::Deserialize`.
3. Genera el `ServerHandler` que implementa `list_tools` (devuelve todas las herramientas registradas) y `call_tool` (despacha al método correcto).

Código real en `crates/shared/src/orchestrator/mcp_server.rs`:
```rust
#[tool_router(server_handler)]
impl DrasusGateway {
    #[tool(description = "Devuelve el timestamp actual...")]
    async fn drasus_clock_now(&self, _ctx: RequestContext<RoleServer>) -> Result<CallToolResult, McpError> {
        // ...
    }

    #[tool(description = "Lista los jobs activos...")]
    async fn drasus_jobs_list(&self, _ctx: RequestContext<RoleServer>) -> Result<CallToolResult, McpError> {
        // ...
    }
}
```

**`Parameters<T>` como wrapper de deserialización.** MCP envía los argumentos de una herramienta como un objeto JSON arbitrario. `Parameters<T>` hace que el macro deserialice ese JSON en el tipo `T` antes de pasar el control al método. El patrón `Parameters(SubmitJobParams { job_type, payload_json, user_id })` es destructuring directo en la firma — Rust lo resuelve en el stack, sin heap extra.

---

### 3. Función pura `evaluate_permission` — determinismo y exhaustividad del `match`

**Por qué una función pura.** `evaluate_permission` no hace I/O, no tiene estado mutable y no lee el reloj del sistema. El mismo `PermissionRequest` siempre produce el mismo `PermissionOutcome`. Esto da tres ventajas inmediatas:

1. **Testing exhaustivo sin mocks:** basta llamar a la función directamente. No hay que simular BD ni red.
2. **Determinismo (ADR-0002/0004):** la lógica de autorización no depende del momento en que se ejecuta.
3. **Composición limpia (FCIS):** el Shell puede llamarla, leer el resultado y luego hacer el I/O (persistir la decisión). La lógica nunca mezcla los dos pasos.

**Exhaustividad del `match` sobre enums.** En Rust, si un `match` sobre un enum no cubre todos los variantes, el compilador rechaza el código. No hay un `else` colgante que pueda olvidar un pipeline nuevo:

```rust
// crates/shared/src/domain/mcp_gateway.rs
match req.pipeline {
    Pipeline::Ingest | Pipeline::Generate | Pipeline::Validate
    | Pipeline::Incubate | Pipeline::Feedback => PermissionOutcome::Granted,
    Pipeline::Manage => { /* ... */ },
    Pipeline::Execute | Pipeline::Withdraw => { /* ... */ },
}
```

Si alguien añade `Pipeline::Settlement` en el futuro, el compilador forzará a cubrir ese caso antes de que el código compile. Es la seguridad del tipo, no la del test.

**El patrón `guard` en `match`.** El case de `Manage` usa un `if` guard dentro del `match`:
```rust
Pipeline::Manage => match &req.institutional_tag {
    Some(InstitutionalTag::Live) if !req.production_override_active =>
        PermissionOutcome::Denied { reason: "...".into() },
    _ => PermissionOutcome::Granted,
},
```
El `if !req.production_override_active` es el guard: el brazo solo coincide si ambas condiciones son verdaderas (Live Y sin interruptor). El `_` captura todos los demás casos (Demo, None, Live con interruptor).

---

### 4. Por qué `audit_hash` se calcula sobre los campos de dominio propio, no sobre el Grupo I

El Grupo I incluye `audit_hash` mismo. Si el hash se calculara sobre el Grupo I completo, entrarías en una circularidad: necesitarías el valor del hash para calcularlo.

La solución: calcular `audit_hash` solo sobre los campos **que llegan como input** (lo que se auditó), no sobre los campos que el sistema genera como output (el propio hash y los metadatos del Grupo I). Los campos de input son:
- `agent_session_id` — quién pidió el permiso.
- `requested_scope` — qué frontera quiso invocar.
- `permission_outcome` — qué decidió el evaluador.
- `production_override_active` — en qué estado estaba el interruptor.

Se añade también `prev_hash` y `sequence_id` para que el hash cubra el encadenamiento: si alguien reordena filas en la BD, los hashes dejan de coincidir y la verificación lo detecta.

Código real en `crates/shared/src/domain/mcp_gateway.rs`:
```rust
pub fn compute_audit_hash(
    agent_session_id: &str, requested_scope: &str, outcome_str: &str,
    production_override_active: bool, prev_hash: &str, sequence_id: i64,
) -> String {
    let payload = format!(
        "{agent_session_id}|{requested_scope}|{outcome_str}|{production_override_active}|{prev_hash}|{sequence_id}"
    );
    // SHA-256 del payload concatenado.
}
```

---

### 5. Tabla de hechos vs tabla de estado — `permission_decisions` vs `mcp_gateway_config`

**Tabla de hechos (append-only).** `permission_decisions` es un log forense: registra todo lo que el agente intentó. No existe UPDATE ni DELETE sobre ella. Si el agente intentó invocar `execute` 50 veces y fue denegado 50 veces, hay 50 filas — ninguna se puede borrar para ocultar el intento.

**Tabla de estado (mutable).** `mcp_gateway_config` tiene exactamente una fila por clave. El interruptor de producción (`production_override_active`) se actualiza cuando el propietario lo activa o desactiva. Aquí sí existe un UPDATE (implementado como `INSERT OR REPLACE` para idempotencia).

**Por qué el interruptor no vive en `permission_decisions`.** Porque no es un hecho — es el estado actual del sistema. Si viviera como columna en `permission_decisions`, habría que leer la última fila para saber el estado actual; y no habría forma limpia de "desactivarlo" sin insertar una fila artificial. La separación clarifica la intención: `permission_decisions` = historial inmutable; `mcp_gateway_config` = configuración mutable.

Migración: `migrations/0005_mcp_gateway.sql`.

---

### 6. `SqlitePool` como `Arc` — por qué clonar es barato

`SqlitePool` de SQLx es internamente un `Arc<PoolInner>`. Cuando escribes:
```rust
// crates/app/src/main.rs
let mcp_pool = pool.clone();
tokio::spawn(async move { run_mcp_server(mcp_pool).await });
```
No estás copiando el pool de conexiones — estás incrementando un contador de referencia atómico. El clon comparte el mismo pool de conexiones subyacente con el resto del motor. Cuando todos los clones se sueltan, el pool se cierra.

---

### 7. `tokio::spawn` — concurrencia sin bloqueo

`tokio::spawn` lanza una tarea async en el runtime de Tokio y devuelve de inmediato un `JoinHandle`. El código que sigue a `spawn` se ejecuta sin esperar a que la tarea termine. Comparación:

```rust
// ❌ Esto bloquearía: esperaría a que el servidor MCP terminara antes de continuar.
run_mcp_server(pool.clone()).await;

// ✅ Esto lanza el servidor en background y sigue adelante.
tokio::spawn(async move { run_mcp_server(pool.clone()).await });
println!("Motor arrancado."); // se ejecuta inmediatamente
```

El servidor MCP corre concurrentemente con el motor. Cuando el proceso recibe SIGTERM o SIGINT (el `tokio::select!` del `run_start`), el proceso termina, los handles de stdin/stdout del servidor se cierran, y el loop MCP finaliza limpiamente. No hace falta coordinación explícita de shutdown.

---

### 8. `Arc<str>` vs `String` para identificadores inmutables compartidos

En `DrasusGateway`, `agent_session_id` y `node_id` son `Arc<str>`:

```rust
// crates/shared/src/orchestrator/mcp_server.rs
pub struct DrasusGateway {
    pool: SqlitePool,
    agent_session_id: StdArc<str>,
    node_id: StdArc<str>,
}
```

`Arc<str>` ocupa el mismo espacio que `Arc<String>` pero es más eficiente: no tiene la indirección extra de `String` (que internamente es un `Vec<u8>` con puntero, longitud y capacidad). Para identificadores que se crean una vez y se leen muchas veces (en cada llamada a `check_and_record`), `Arc<str>` es el tipo idiomático.

La conversión desde `String`:
```rust
StdArc::from(agent_session_id.as_str())
```
Copia el contenido de la `String` al heap y devuelve el `Arc<str>`. No existe un `.into()` directo de `String` a `Arc<str>` porque el compilador necesita saber que el destino es `Arc<str>` (no `Arc<String>`).

---

## Trucos de Senior

**`ON CONFLICT(key) DO UPDATE SET value = excluded.value`** (SQLite upsert).  
En lugar de un `SELECT` seguido de un `INSERT` o `UPDATE` condicional (dos roundtrips), el upsert es atómico: si la clave ya existe la actualiza; si no, la inserta. `excluded` es una pseudotabla que referencia los valores que se intentaron insertar. Se usa en `set_production_override` para que la operación sea idempotente sin condición manual.

**`tool_router(server_handler)` combina dos macros en uno.**  
`#[tool_router]` solo genera el enrutador de herramientas. `#[tool_router(server_handler)]` también implementa `ServerHandler` automáticamente. Si solo necesitas herramientas (sin prompts ni recursos custom), el flag `server_handler` elimina la necesidad de un bloque `impl ServerHandler for ...` separado.

**Archivo temporal vs `:memory:` en pruebas de persistencia.**  
Una BD `:memory:` desaparece cuando el pool se cierra. Para probar durabilidad (que los datos sobreviven a cerrar y reabrir el pool), siempre usa `NamedTempFile` de la crate `tempfile`. El archivo se borra automáticamente al salir del test cuando `_file` cae fuera de scope (RAII).
