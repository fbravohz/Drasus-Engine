# STORY-009 · CLI con Clap + binario raíz `app`

| Campo | Valor |
|---|---|
| **ID** | STORY-009 |
| **Título** | CLI con Clap + binario raíz `app` — arranque del motor, shutdown graceful y gate `kill -9` |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | 1 |
| **Estado** | Completada |
| **Responsable** | Rust-Engineer (Sonnet) · Modo Docente · auditará Tech-Lead |
| **Creada** | 2026-06-20 |
| **Completada** | 2026-06-20 |

## 1. Especificación de origen (qué specs implementa)

- **SAD §4.2** — Nivel 2 del modelo C4: binario raíz que orquesta los 8 módulos + `shared`.
- **ROADMAP EPIC-0 alcance:** "CLI con Clap + binario raíz `app`".
- **ROADMAP EPIC-0 criterio de salida (parcial):** "un job asíncrono sobrevive a un `kill -9` y se recupera" — este es el gate EPIC-0 pendiente.
- **ADR-0003** — el crate `app` sigue la misma regla de frontera: accede a los demás crates solo a través de sus `public_interface`.
- **ADR-0033** — tres modos de despliegue. Para EPIC-0 solo se implementa el modo `local` (arranque estándar). Los otros modos se activan en épicas futuras.
- **No tiene feature propia**: el binario `app` es infraestructura de orquestación, no una Feature de dominio.

## 2. Objetivo (una frase llana)

Crear el binario ejecutable `app` con una CLI mínima (Clap) que inicialice la base de datos, arranque el motor y se apague limpiamente; y demostrar con una prueba de integración real que los jobs sobreviven a un `kill -9` (SIGKILL) y se recuperan al reiniciar.

## 3. Agentes y Modo de Acompañamiento (ADR-0120)

| Agente | Etapa del pipeline | Depende de | Modo |
|---|---|---|---|
| Rust-Engineer | Etapa 2 — Implementación Core | ninguno | Docente |

**Modo Docente (ADR-0122):** el Rust-Engineer implementa cada bloque completo (escribe el código con `Edit`/`Write` sin esperar al usuario). Antes de avanzar al siguiente bloque, explica cada decisión de diseño con profundidad cero-conocimiento (qué es, por qué existe, qué problema resuelve), luego avanza. Documenta TODO lo enseñado en `docs/lessons/rust/STORY-009-cli-app.md` (ADR-0124: un archivo por Story, con código real de esta Story como ejemplo).

## 4. Instrucciones de despacho por agente (la spec ejecutable)

### 4.1 Rust-Engineer

```
Eres el Rust-Engineer de Drasus Engine.

PASO OBLIGATORIO ANTES DE ACTUAR:
1. Lee `.claude/skills/base/SKILL.md` completo. Declara "[base/SKILL.md leído]".
2. Lee `.claude/skills/rust-engineer/SKILL.md` completo. Declara "[rust-engineer/SKILL.md leído]".
3. Declara tu Modo de Acompañamiento leyendo §3 de esta Orden: es **Docente** (ADR-0122).

DIRECTORIO DE TRABAJO: /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine

---

## Contexto del proyecto

Drasus Engine — sistema de trading algorítmico en Rust + Flutter.
- Pipeline de 8 módulos (crates de biblioteca): `ingest`, `generate`, `validate`, `incubate`, `manage`, `execute`, `feedback`, `withdraw` + crate `shared` con features transversales.
- Workspace actual: `Cargo.toml` en la raíz con los 9 crates de biblioteca. Aún no existe un crate binario.
- Patrón FCIS (Functional Core / Imperative Shell): la lógica pura vive en `domain/`, la orquestación en la capa Shell.
- Todos los módulos se comunican SOLO a través de sus `public_interface.rs`. Prohibido importar internos de otro crate.

---

## Tu tarea

Crea el crate binario `app` (STORY-009). Sigue la estructura de bloques que se describe abajo. En Modo Docente: implementa cada bloque completamente, luego explica las decisiones de diseño antes de avanzar al siguiente.

---

## Bloque 1 — Estructura del crate `app`

Crea el crate en `crates/app/`:

```
crates/app/
├── Cargo.toml          ← binario, no librería; depende de shared + 8 módulos
└── src/
    └── main.rs         ← Shell puro: wiring y arranque, cero lógica de dominio
```

**`crates/app/Cargo.toml`:**
- `[package]` con `name = "app"`, `edition`/`version`/`license` heredadas del workspace.
- `[[bin]]` con `name = "drasus"` y `path = "src/main.rs"`.
- Dependencias:
  - `clap = { version = "4", features = ["derive"] }` — CLI declarativa.
  - `tokio = { version = "1", features = ["rt-multi-thread", "macros", "signal"] }` — runtime async + señales del OS.
  - `shared` (path local).
  - Los 8 crates de módulo (paths locales). Aunque en EPIC-0 solo se usan indirectamente vía `shared`, declararlos como dependencias garantiza que el workspace los compile como un bloque monolítico desde el binario raíz.

Añade `"crates/app"` al array `members` del `Cargo.toml` raíz.

**Enseñanza esperada después de este bloque:**
- Qué es un crate binario vs librería en Cargo (`[[bin]]` vs `[lib]`).
- Por qué `edition`/`version`/`license` se heredan del workspace (DRY en monorepos).
- Por qué `clap` con `features = ["derive"]` (macro proc vs builder API).
- Por qué declarar los 8 módulos como dependencias del binario aunque no los use directamente en EPIC-0.

---

## Bloque 2 — CLI con Clap (subcomandos mínimos para EPIC-0)

Implementa `src/main.rs`:

```rust
// Shell puro: wiring, arranque, y espera de señal. Cero lógica de dominio.
```

Estructura mínima para EPIC-0:

```
drasus start [--db <ruta>]   → arranca el motor
drasus version               → imprime la versión
```

Subcomando `start`:
- Argumento `--db <ruta>` (opcional, default: `"drasus.db"`): ruta al archivo SQLite.
- Inicializa el pool de conexiones (`shared::public_interface::create_pool`).
- Corre las migraciones (`shared::public_interface::run_migrations`).
- Imprime "Motor Drasus arrancado. Presiona Ctrl+C para detener."
- Espera señal de cierre: `tokio::signal::ctrl_c()` (SIGINT) **y** `tokio::signal::unix::signal(SignalKind::terminate())` (SIGTERM). Al recibir cualquiera de las dos: imprime "Apagado limpio." y sale con código 0.

Subcomando `version`:
- Imprime `drasus v<version del Cargo.toml>` (usa `env!("CARGO_PKG_VERSION")`).

**Restricción FCIS (obligatoria):**
- `main.rs` NO puede contener lógica de dominio.
- `main.rs` NO puede importar internals de ningún módulo — solo `public_interface`.

**Enseñanza esperada después de este bloque:**
- Cómo funciona `#[derive(Parser)]` y `#[derive(Subcommand)]` en Clap 4 (el proc-macro genera el parser sin código manual).
- Por qué esperar TANTO SIGINT como SIGTERM (SIGINT = Ctrl+C interactivo; SIGTERM = señal del OS en producción/deploy).
- Cómo `tokio::select!` une ambas señales sin bloquear.
- Por qué `main.rs` es la única pieza de Shell que puede importar todos los módulos (es el punto de composición del sistema).

---

## Bloque 3 — Test de integración: gate `kill -9` (SIGKILL)

Este es el gate de EPIC-0 pendiente. El test demuestra que los jobs sobreviven a un `kill -9` real.

Crea un test de integración en `crates/app/tests/kill9_recovery.rs`:

El test:
1. Prepara una base de datos temporal en archivo (NO `:memory:` — debe sobrevivir al reinicio del proceso).
2. Lanza el binario `drasus start --db <ruta_temporal>` como subproceso real usando `std::process::Command`.
   - Usa `env!("CARGO_BIN_EXE_drasus")` para obtener la ruta del binario compilado.
3. Espera 300ms para que el motor arranque y corra las migraciones.
4. Inserta un job en estado `QUEUED` directamente en la DB (usando `sqlx` o SQL raw con `sqlite3` via Command).
5. Envía `SIGKILL` al subproceso con `nix::sys::signal::kill(Pid::from_raw(pid), Signal::SIGKILL)`.
6. Verifica que el subproceso murió (sin código de salida limpio — SIGKILL no da chance de cleanup).
7. Relanza el binario `drasus start --db <misma_ruta>` (reinicio tras crash).
8. Espera 500ms para que corra la recuperación de startup.
9. Consulta la DB: el job debe estar en estado `QUEUED` (si estaba `RUNNING` al morir, debe haberse reseteado a `QUEUED`; si estaba `QUEUED`, debe seguir `QUEUED`). Verifica también que se emitió el evento `JOB_RECOVERED_AT_STARTUP` en `audit_events`.
10. Apaga el segundo subproceso con `SIGTERM` (shutdown limpio).
11. Limpia el archivo temporal.

Nombre del test: `job_survives_kill9_and_recovers_on_restart`.

**Enseñanza esperada después de este bloque:**
- Qué es `env!("CARGO_BIN_EXE_<nombre>")`: cómo Cargo expone la ruta del binario compilado en tests de integración.
- Por qué los tests de integración van en `crates/app/tests/` y no en `src/` (son tests de caja negra sobre el binario, no de internals).
- Por qué SIGKILL no puede interceptarse (diferencia con SIGTERM): el kernel mata el proceso inmediatamente, sin dar tiempo al proceso a correr cleanup handlers.
- Por qué el test usa un archivo real en lugar de `:memory:` (la durabilidad es la postcondición; `:memory:` desaparece al morir el proceso).
- Cómo `nix::sys::signal::kill` envía señales a procesos externos desde Rust.

---

## Bloque 4 — Lección formal

Escribe `docs/lessons/rust/STORY-009-cli-app.md` (ADR-0124).

Estructura:
- Encabezado: `> Story: [STORY-009](../../execution/STORY-009-cli-app.md)`
- Sección `## Concepto` con una subsección por cada concepto enseñado en los bloques 1-3 (usa el código real de esta Story como ejemplo, con ruta de archivo + fragmento).
- Sección `## Trucos de Senior` solo si apareció azúcar sintáctica real que valga destacar.

---

## Criterio de cierre

| # | Criterio | Prueba |
|---|---|---|
| 1 | El workspace compila limpio (`cargo build --workspace`) | — |
| 2 | Clippy sin warnings (`cargo clippy --workspace --all-targets -- -D warnings`) | — |
| 3 | El binario `drasus version` imprime la versión | — |
| 4 | El binario `drasus start` arranca, imprime el mensaje y se apaga limpio con Ctrl+C / SIGTERM | — |
| 5 | Gate EPIC-0: job sobrevive a `kill -9` y se recupera al reiniciar | `job_survives_kill9_and_recovers_on_restart` |
| 6 | FCIS: `main.rs` sin lógica de dominio ni imports de internals | grep de verificación |
| 7 | Tests previos (STORY-001-008) siguen en verde | `cargo test --workspace` |
| 8 | Lección en `docs/lessons/rust/STORY-009-cli-app.md` con código real de esta Story | — |

---

## Comandos de validación

```bash
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p app -- --nocapture     # incluye el test de integración kill -9
cargo llvm-cov --workspace --summary-only
./target/debug/drasus version
./target/debug/drasus start          # arranca; Ctrl+C para parar
```

Al terminar, reporta al Tech-Lead:
1. Artefactos creados (rutas).
2. Resultado de cada criterio de cierre (tabla criterio → test → resultado).
3. Cobertura por archivo nuevo.
4. Cualquier decisión de diseño que tomaste y no estaba especificada.
```

**Plan de Implementación / Revisión** (lo llena el Agente al ser invocado):

> ✅ **Implementado** 2026-06-20 · Orden de trabajo [STORY-009](../execution/STORY-009-cli-app.md)
>
> Lección: [docs/lessons/rust/STORY-009-cli-app.md](../lessons/rust/STORY-009-cli-app.md)

Bloques ejecutados en orden (Modo Docente):
1. Crate `app` creado en `crates/app/`; añadido al workspace.
2. `src/main.rs` con Clap 4 derive, subcomandos `start`/`version`, `recover_at_startup`, señales SIGINT+SIGTERM.
3. Test de integración `tests/kill9_recovery.rs` — gate EPIC-0 en verde.
4. Lección formal en `docs/lessons/rust/STORY-009-cli-app.md` (9 conceptos, 3 trucos de senior).

Decisión no especificada: se añadieron las re-exportaciones `create_pool` y `run_migrations` a `shared/src/public_interface.rs` (la Orden las referenciaba con esos nombres pero no existían; se añadieron como aliases de `persistence::pool::connect` y `persistence::pool::migrate`).

## 5. Criterio de aceptación (cada criterio ↔ su prueba)

| # | Criterio verificable | Prueba que lo demuestra |
|---|---|---|
| 1 | `cargo build --workspace` limpio — crate `app` compila | — (build) |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` sin warnings | — (clippy) |
| 3 | `drasus version` imprime `drasus vX.Y.Z` | — (ejecución manual / test de salida) |
| 4 | `drasus start` arranca, inicializa DB, espera SIGTERM/SIGINT y sale con código 0 | — (ejecución manual) |
| 5 | Gate EPIC-0: job en QUEUED sobrevive a SIGKILL y se recupera en el reinicio; evento `JOB_RECOVERED_AT_STARTUP` emitido | `job_survives_kill9_and_recovers_on_restart` |
| 6 | FCIS: `main.rs` no contiene lógica de dominio ni imports de internals de módulos | `grep -n "domain::\|persistence::\|orchestrator::" crates/app/src/main.rs` → 0 resultados |
| 7 | Tests anteriores (STORY-001-008) siguen verdes | `cargo test --workspace` |
| 8 | Lección en `docs/lessons/rust/STORY-009-cli-app.md` con código real referenciado | — (inspección) |

## 6. Comandos de validación (para el usuario — copy/paste)

```bash
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p app -- --nocapture
cargo llvm-cov --workspace --summary-only
./target/debug/drasus version
./target/debug/drasus start
grep -n "domain::\|persistence::\|orchestrator::" crates/app/src/main.rs
```

## 7. Registro de ejecución (bitácora cronológica)

- **2026-06-20** — Rust-Engineer (Sonnet, Modo Docente) ejecuta los 4 bloques. Todos los criterios en verde. 93 tests del workspace en verde. Cobertura `app/src/main.rs`: 89.33% de líneas (8 líneas no cubiertas = rama `#[cfg(not(unix))]`, inalcanzable en Linux).

## 8. Pendientes derivados / decisiones

- **STORY-010 (`agentic-mcp-gateway`):** siguiente en la secuencia de EPIC-0. Depende de que el binario raíz exista (este Story).
- **Subcomandos futuros:** `drasus ingest`, `drasus backtest`, etc. se añadirán en sus épicas respectivas — no en EPIC-0.
- **Modos de despliegue (ADR-0033):** LocalMode y SaaSCloudEngine se implementan en EPIC-3/EPIC-8 cuando existan la UI Flutter y el gRPC headless.
- **⚠️ Validación pendiente en Windows:** la rama `#[cfg(not(unix))]` de `main.rs` (SIGTERM fallback para Windows) no se puede ejercer en Linux. Pendiente de ejecutar en Windows:
  ```bash
  cargo test --workspace
  cargo test -p app -- --nocapture
  cargo llvm-cov --workspace --summary-only
  ```
  Cuando se confirme verde en Windows, cobertura de `main.rs` debería subir por encima del 95%.
