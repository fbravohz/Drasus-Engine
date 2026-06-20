> Story: [STORY-009 — CLI con Clap + binario raíz `app`](../../execution/STORY-009-cli-app.md)

# Lección STORY-009 — CLI con Clap y binario `drasus`

Esta Story implementó el crate binario `app`, la CLI con Clap, el arranque del motor con recuperación de jobs y el gate EPIC-0 (`kill -9`). Cada concepto de Rust que apareció durante la implementación se documenta abajo con código real de esta Story.

---

## Concepto

### 1. Crate binario vs crate de biblioteca en Cargo

**Qué es.** Un workspace de Rust puede contener dos tipos de crates: bibliotecas (`[lib]`) y binarios (`[[bin]]`). Una biblioteca produce un archivo `.rlib` que otro crate puede importar. Un binario produce un ejecutable que el OS puede lanzar directamente.

**El problema que resuelve.** En Drasus Engine, todos los crates de dominio (`shared`, `ingest`, `generate`…) son bibliotecas: exponen lógica que el binario raíz reutiliza. El crate `app` es el único binario: es el punto de entrada del motor para el usuario final.

**Cómo se declara.** En `crates/app/Cargo.toml`:

```toml
# crates/app/Cargo.toml
[[bin]]
name = "drasus"       # nombre del ejecutable que produce Cargo
path = "src/main.rs"  # archivo con la función `fn main()`
```

La doble corchete `[[bin]]` indica que puede haber varios binarios en un crate (ej. `drasus` + una herramienta de migración). Para bibliotecas sería `[lib]` (una sola, sin corchetes dobles).

**Por qué `edition`/`version`/`license` son `.workspace = true`.** En un monorepo con 10+ crates, mantener la misma versión en cada `Cargo.toml` es propenso a errores. La directiva `.workspace = true` delega esos campos al `[workspace.package]` de la raíz. Un solo sitio para cambiarlos — DRY (Don't Repeat Yourself).

```toml
# crates/app/Cargo.toml
[package]
name = "app"
edition.workspace = true   # heredado de [workspace.package] en Cargo.toml raíz
version.workspace = true
license.workspace = true
```

---

### 2. Clap 4 con el macro `derive` — CLI declarativa

**Qué es Clap.** Clap (Command Line Argument Parser) es la librería estándar de facto para parsear argumentos CLI en Rust. Existen dos APIs: la builder API (encadenamiento de métodos) y la derive API (macros proc que generan el parser a partir de structs).

**Por qué `features = ["derive"]`.**  El parser generado por `#[derive(Parser)]` vive en un proc-macro que Cargo compila por separado. Está detrás de un feature flag para no incluirlo si el proyecto solo necesita la builder API.

```toml
# crates/app/Cargo.toml
clap = { version = "4", features = ["derive"] }
```

**Cómo funciona `#[derive(Parser)]`.** El compilador de Rust invoca el proc-macro de Clap en tiempo de compilación. El macro lee la definición del struct/enum y genera todo el código del parser (validación de argumentos, mensajes de ayuda, errores de formato). En tiempo de ejecución, `Cli::parse()` lee `std::env::args()` y devuelve una instancia del struct.

```rust
// crates/app/src/main.rs — fragmento real
#[derive(Parser)]
#[command(name = "drasus", about = "Motor de trading algorítmico", version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start {
        #[arg(long, default_value = "drasus.db")]
        db: String,
    },
    Version,
}
```

Clap infiere automáticamente:
- `--db` del nombre del campo `db` (con el atributo `#[arg(long)]`).
- El valor por defecto `"drasus.db"` de `default_value`.
- Los subcomandos `start` y `version` de las variantes del enum.
- El mensaje de ayuda `--help` de los doc-comments (`///`).

**Qué pasa si falta un argumento obligatorio.** Clap imprime un error claro y sale con código 2 — sin `panic!`, sin `unwrap`. Eso es lo que hace un CLI profesional.

---

### 3. `env!("CARGO_PKG_VERSION")` — versiones en tiempo de compilación

**El problema.** ¿Cómo sabe el binario su propia versión sin hardcodear un string? Si hardcodeas `"0.1.0"` en el código, tienes que recordar actualizarlo cada vez que subes la versión en `Cargo.toml`.

**La solución.** Cargo inyecta la versión del `Cargo.toml` como variable de entorno `CARGO_PKG_VERSION` durante la compilación. El macro `env!()` de Rust resuelve esa variable en tiempo de compilación (no en tiempo de ejecución):

```rust
// crates/app/src/main.rs
Commands::Version => {
    println!("drasus v{}", env!("CARGO_PKG_VERSION"));
}
```

El string `"0.1.0"` queda embebido en el binario en el momento del build. Si cambias la versión en `Cargo.toml` y recompila, el nuevo binario tiene la versión nueva — automáticamente.

---

### 4. `#[tokio::main]` — cómo Rust convierte async en síncrono

**El problema.** El OS no sabe qué es un `async fn`. El punto de entrada `main` del OS debe ser una función síncrona normal. Pero todo el motor de Drasus es asíncrono (I/O de SQLite, señales del OS, workers…).

**Cómo funciona `#[tokio::main]`.** El atributo es otro proc-macro. Transforma esto:

```rust
#[tokio::main]
async fn main() {
    run_start("drasus.db").await;
}
```

...en algo equivalente a esto (a grandes rasgos):

```rust
fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            run_start("drasus.db").await;
        });
}
```

Es decir, crea el runtime de Tokio, lo arranca, y bloquea el thread del OS hasta que el future `main` completa. La feature `rt-multi-thread` del `Cargo.toml` habilita el scheduler con múltiples threads del OS (necesario para `JobExecutor` que lanza tasks en paralelo).

---

### 5. `tokio::select!` — esperar varias señales async a la vez

**El problema.** El motor debe detenerse si llega SIGINT (Ctrl+C) O si llega SIGTERM (apagado del OS). No puedes `.await` las dos secuencialmente — eso esperaría primero una y luego la otra.

**Cómo funciona `select!`.** El macro `tokio::select!` toma N branches de la forma `_ = future => { ... }` y espera al que resuelva primero. Los demás branches se cancelan.

```rust
// crates/app/src/main.rs
tokio::select! {
    _ = tokio::signal::ctrl_c() => {
        // SIGINT: el usuario presionó Ctrl+C
    }
    _ = sigterm_received() => {
        // SIGTERM: el OS ordenó apagar
    }
}
```

**Por qué separamos `sigterm_received()` en su propia función.** Porque la API de señales de Unix de Tokio (`tokio::signal::unix::signal`) solo existe en Unix — no en Windows. Envolver la llamada en una función con `#[cfg(unix)]` permite que el código compile en ambas plataformas sin `#[cfg]` dentro del cuerpo del `select!`.

---

### 6. SIGINT vs SIGTERM vs SIGKILL — las tres señales que importan

**Por qué distinguirlas.** Son tres señales distintas con semántica completamente diferente:

| Señal | Número | Interceptable | Quién la envía | Qué hace el proceso |
|---|---|---|---|---|
| SIGINT | 2 | Sí | Ctrl+C en terminal | Termina si no hay handler |
| SIGTERM | 15 | Sí | `kill <pid>`, systemd, Docker stop | Termina si no hay handler |
| SIGKILL | 9 | NO — nunca | `kill -9 <pid>`, OOM killer | El kernel lo mata instantáneamente |

**Por qué SIGKILL no puede interceptarse.** SIGKILL es la única señal que el kernel envía directamente al scheduler, sin pasar por el proceso. El proceso no tiene ni un nanosegundo para correr código de limpieza. Por eso un `kill -9` simula exactamente lo que ocurre en un crash de hardware, un OOM killer o una pérdida de energía — el proceso muere sin cerrar nada.

**Implicación de diseño.** Para que los datos sobrevivan a SIGKILL, el motor usa SQLite con WAL y el patrón "persist-before-ack": el job se persiste en disco **antes** de encolarse en memoria. Si el proceso muere entre "encolado en memoria" y "tomado por un worker", la fila en disco garantiza la recuperación al reiniciar.

---

### 7. `env!("CARGO_BIN_EXE_<nombre>")` — ruta del binario en tests de integración

**El problema.** Un test de integración que quiere lanzar el binario `drasus` necesita saber dónde está el ejecutable compilado. La ruta varía según el perfil (`debug` vs `release`) y el OS.

**La solución.** Cargo expone la ruta del binario como variable de entorno `CARGO_BIN_EXE_<nombre>` (donde `<nombre>` es el campo `name` en `[[bin]]`) disponible solo dentro de los tests de integración:

```rust
// crates/app/tests/kill9_recovery.rs
let bin_path = env!("CARGO_BIN_EXE_drasus");
let mut child = Command::new(bin_path).args(["start", "--db", &db_url]).spawn()?;
```

Esto es más robusto que `"./target/debug/drasus"` — funciona en CI, en builds de release y sin hardcodear el perfil.

**Por qué el test va en `crates/app/tests/` y no en `src/`.** Los tests en `src/` (dentro de `#[cfg(test)]`) son tests de unidad: tienen acceso a los internals del módulo. Los tests en `tests/` son tests de integración de caja negra: solo ven la interfaz pública del crate. Para un test que lanza el binario como proceso externo, la caja negra es la única opción — el test ni siquiera tiene acceso al código de `main.rs` en tiempo de ejecución.

---

### 8. Por qué el test usa un archivo real en vez de `:memory:`

**El problema.** Una base de datos SQLite en `:memory:` existe solo mientras el pool que la abrió está vivo. Cuando el proceso muere (por SIGKILL), el pool se destruye y la BD desaparece con él. No hay nada que recuperar.

**La solución.** SQLite en modo WAL sobre un archivo real es durable: las páginas se sincronizan al disco (o al buffer de escritura del OS) antes de que el commit retorne. Cuando el proceso vuelve a arrancar, lee el mismo archivo — con todos los datos del estado anterior.

```rust
// crates/app/tests/kill9_recovery.rs
let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
let db_path = temp_dir.path().join("drasus_kill9_test.sqlite");
// db_path es un archivo real en /tmp — sobrevive al proceso que lo creó
```

`tempfile::tempdir()` crea un directorio en `/tmp` que se limpia automáticamente cuando el valor sale del scope (implementa `Drop`). Eso evita residuos en el sistema de archivos si el test falla a mitad.

---

### 9. `recover_at_startup` — el patrón de recuperación de crash

**El problema.** Si el proceso muere mientras hay jobs en estado `RUNNING`, al reiniciar esos jobs quedan "atascados": el worker que los estaba ejecutando ya no existe, pero la fila en `jobs` dice `RUNNING`. Sin recuperación, esos jobs nunca terminarían.

**La solución.** El `JobExecutor::recover_at_startup` escanea la tabla `jobs` buscando filas en `QUEUED` o `RUNNING`:
- `QUEUED` → se re-encolan tal cual (el motor aún no los tomó).
- `RUNNING` → se resetean a `QUEUED` (no se sabe si el worker completó la tarea — es más seguro reintentar que asumir éxito).

Por cada job recuperado, se emite un evento `JOB_RECOVERED_AT_STARTUP` en `audit_events`. Esto deja un rastro auditable de cada recovery, útil para diagnóstico en producción.

```rust
// crates/app/src/main.rs — en run_start()
let executor = JobExecutor::new(pool.clone(), clock, identity, config, HashMap::new());
let recovered = executor.recover_at_startup().await.expect("recuperación de startup");
if !recovered.is_empty() {
    println!("Recuperados {} jobs del crash anterior.", recovered.len());
}
```

**Por qué `recover_at_startup` debe llamarse ANTES de `spawn_workers`.** Si el dispatcher de workers empieza a tomar jobs antes de que `recover_at_startup` reencole los jobs residuales, los jobs `QUEUED` que quedaron en disco nunca llegarían a la cola en memoria. El orden es: 1) construir el executor, 2) llamar `recover_at_startup`, 3) llamar `spawn_workers`.

---

## Trucos de Senior

### `#[cfg(unix)]` para código específico de plataforma

En vez de `#[cfg(target_os = "linux")]`, se usa `#[cfg(unix)]` que cubre Linux, macOS y todos los BSD — cualquier OS que siga el estándar POSIX. Esto hace el código portable a macOS sin cambios, que es relevante para desarrollo local de ingenieros en Mac:

```rust
// crates/app/src/main.rs
#[cfg(unix)]
async fn sigterm_received() {
    use tokio::signal::unix::{signal, SignalKind};
    let mut stream = signal(SignalKind::terminate()).expect("registrar SIGTERM");
    stream.recv().await;
}

#[cfg(not(unix))]
async fn sigterm_received() {
    std::future::pending::<()>().await; // nunca resuelve — en Windows solo SIGINT
}
```

### `std::future::pending::<()>().await` — un future que nunca resuelve

`std::future::pending()` retorna un future que nunca entra en estado `Ready`. Usado en la rama no-Unix de `sigterm_received`, hace que `select!` nunca tome ese branch en Windows — exactamente la semántica correcta: "en Windows no hay SIGTERM, ignora ese branch para siempre".

### `format!("sqlite://{db_path}")` — interpolación de rutas como strings

SQLx necesita la URL en formato `sqlite://<ruta>`. Usar interpolación de strings en vez de concatenación con `+` evita la transferencia de ownership y es más legible. El prefijo `sqlite://` activa el driver correcto dentro de SQLx.
