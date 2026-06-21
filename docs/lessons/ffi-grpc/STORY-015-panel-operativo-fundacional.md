# Lecciones — STORY-015: Panel Operativo Fundacional

> Story: [STORY-015](../../execution/STORY-015-panel-operativo-fundacional.md)
> Dominio: FFI / flutter_rust_bridge
> Ingeniero: Bridge-Engineer
> Modo: Docente (ADR-0122)

---

## Concepto

### 1. Qué es una librería nativa y por qué Flutter necesita dos formatos

Cuando Flutter quiere llamar código Rust, no puede importar un crate Rust directamente — Flutter es una aplicación Dart, y Dart no entiende Rust. Necesitas compilar Rust como una **librería nativa**: un archivo binario compilado que expone funciones con una interfaz estándar que cualquier lenguaje puede llamar (la interfaz C, que es el mínimo común denominador de la industria).

Hay dos formatos de librería nativa:

| Formato | Archivo en disco | Cuándo se usa |
|---|---|---|
| `cdylib` (dinámica) | `.so` (Linux), `.dylib` (macOS), `.dll` (Windows) | En Desktop y Android: Flutter carga la librería en tiempo de ejecución con `dlopen`/`LoadLibrary`. |
| `staticlib` (estática) | `.a` (Unix), `.lib` (Windows) | En iOS y algunos builds de Android: la librería se enlaza dentro del binario de la app en tiempo de compilación, no como archivo separado. |

En `crates/bridge/Cargo.toml` se declaran ambos:
```toml
[lib]
crate-type = ["cdylib", "staticlib"]
```

Rust compila ambas versiones. `flutter_rust_bridge` elige cuál usar según el target de compilación (el sistema operativo destino). No hay que elegir a mano.

**Qué significa "generar bindings Dart desde Rust":**

`flutter_rust_bridge_codegen` lee tu código Rust (las funciones y structs que expones) e infiere automáticamente el equivalente Dart. Por ejemplo, esta función en Rust:

```rust
// crates/bridge/src/api/clock.rs
#[frb(sync)]
pub fn get_clock_timestamp_ns() -> i64 { ... }
```

...produce automáticamente este código en Dart:

```dart
// ui/lib/src/rust/api/clock.dart
int getClockTimestampNs() { /* llama al símbolo FFI */ }
```

Nunca escribes el Dart a mano: si cambias la firma en Rust y vuelves a correr el codegen, el Dart se actualiza automáticamente. Esto garantiza que los tipos sean idénticos en ambos lados — es imposible que Dart espere un `String` si Rust devuelve un `i64`.

---

### 2. `#[frb(sync)]` — qué significa y cuándo usarlo

Cuando flutter_rust_bridge expone una función Rust a Dart, por defecto la convierte en una función **asíncrona** (`Future<T>`): Dart hace `await fn()` y el motor de UI sigue dibujando mientras Rust trabaja.

El atributo `#[frb(sync)]` le dice al codegen: "esta función es tan rápida que no necesita el mecanismo async — expónla como función síncrona normal en Dart".

En STORY-015, `get_clock_timestamp_ns` lleva `#[frb(sync)]` porque:
- Solo lee `SystemTime::now()` del sistema operativo (microsegundos, sin I/O a disco ni red).
- No bloquea nada.
- Desde Dart se llama en un `Timer.periodic` cada segundo — hacer `await` cada vez sería innecesario.

```rust
// crates/bridge/src/api/clock.rs
#[frb(sync)]
pub fn get_clock_timestamp_ns() -> i64 {
    SystemClock::new().timestamp_ns()
}
```

**Regla general:**
- `#[frb(sync)]` → solo para funciones que NO hacen I/O y retornan en microsegundos.
- Sin atributo (async) → para cualquier función que abre ficheros, consulta bases de datos o hace red.

**Tipos que pueden cruzar la frontera FFI con flutter_rust_bridge:**

| Rust | Dart | Notas |
|---|---|---|
| `i8`, `i16`, `i32`, `i64` | `int` | Dart usa int de 64 bits |
| `u8`, `u16`, `u32` | `int` | u64 puede perder precisión en Dart web |
| `f32`, `f64` | `double` | |
| `bool` | `bool` | |
| `String` | `String` | Se copia por valor al cruzar |
| `Vec<T>` (T primitivo) | `List<T>` | Se copia el contenido completo |
| struct con campos primitivos | clase Dart equivalente | El codegen la genera automáticamente |

`i64` es seguro: Dart (VM nativa) lo representa exactamente. No hay truncamiento.

**Nota sobre `u64`:**

La Orden original pedía `u64` para el timestamp del reloj. Se cambió a `i64` porque:
1. `SystemClock::timestamp_ns()` devuelve `i64` — este es el tipo real del Core.
2. SQLite almacena timestamps como `INTEGER` con signo (i64).
3. Mantener el mismo tipo en toda la pila evita conversiones con pérdida.

---

### 3. Por qué las funciones `async` se exponen como `Future<T>` en Dart

Flutter tiene un solo hilo principal (el "UI thread" o "Isolate raíz"). Si ese hilo se bloquea esperando datos de disco, la pantalla se congela: no se dibujan frames mientras el hilo está ocupado.

Rust tiene el mismo problema: las operaciones de I/O bloquean el hilo que las llama si no usas el patrón asíncrono.

flutter_rust_bridge resuelve esto así:
1. Rust expone funciones `async fn` que usan el runtime **Tokio** (el equivalente a los event loops de Node.js/Python asyncio, pero en Rust).
2. flutter_rust_bridge genera en Dart el equivalente como `Future<T>`: Dart hace `await getJobsSummary(dbPath)`, que cede el control al motor de Flutter mientras Rust trabaja en un hilo de fondo de Tokio.
3. Cuando Rust termina, le notifica a Flutter con el resultado.

En STORY-015, `get_jobs_summary` y `get_recent_audit_events` son `async fn`:

```rust
// crates/bridge/src/api/jobs.rs
pub async fn get_jobs_summary(db_path: String) -> Vec<JobSummary> {
    let pool = match create_pool(&url).await { Ok(p) => p, Err(_) => return Vec::new() };
    // ... query SQLite
}
```

Dart la ve como:
```dart
// ui/lib/src/rust/api/jobs.dart
Future<List<JobSummary>> getJobsSummary({required String dbPath}) async { ... }
```

**Por qué `Vec<JobSummary>` es seguro si `JobSummary` contiene solo `String` e `i64`:**

Al cruzar la frontera FFI, flutter_rust_bridge serializa el `Vec<JobSummary>` de Rust en una lista Dart. Cada `String` se copia por valor (Rust libera la suya, Dart tiene la propia). Cada `i64` se copia por valor (es un número, no hay heap). Una vez que el dato cruza, Rust y Dart gestionan sus copias de forma independiente — no hay punteros compartidos, no hay riesgo de use-after-free ni de double-free.

---

### 4. El hash de cadena en auditoría — qué es y por qué es `String` en la frontera

La bitácora de auditoría de Drasus es **append-only** (solo permite insertar, nunca modificar ni borrar) y cada evento incluye el hash SHA-256 del evento anterior. Esto forma una **cadena criptográfica**: si alguien modifica un evento histórico (aunque sea un solo byte), el hash de ese evento cambia, lo que rompe el hash del evento siguiente, y así sucesivamente. La cadena entera queda inválida, y `verify_chain()` lo detecta.

En el tipo Rust `AuditEvent`:
```rust
// shared/src/domain/audit_log.rs
pub struct AuditEvent {
    pub audit_chain_hash: Option<String>, // None para el evento génesis
    // ...
}
```

`audit_chain_hash` es `Option<String>`:
- `None` en el primer evento de la cadena (el "génesis") — no hay predecesor que hashear.
- `Some("abc123...")` en todos los demás — es el `audit_hash` del evento anterior.

Al cruzar la frontera FFI, se convierte a `String` con `unwrap_or_default()`:
```rust
// crates/bridge/src/api/audit.rs
audit_chain_hash: event.audit_chain_hash.unwrap_or_default(),
```

**Por qué `String` y no `Vec<u8>` (bytes crudos):**

El hash SHA-256 son 32 bytes. Se almacena en la BD como cadena hexadecimal de 64 caracteres ("a3f2b1..."). Si lo enviáramos a Dart como `Vec<u8>`, el Flutter-Engineer tendría que hacer la conversión a hex en Dart. Al mandarlo ya como `String` hexadecimal, Dart solo necesita mostrarlo (o tomar los últimos 8 caracteres para verificación visual). Cero lógica de conversión en Dart (ADR-0097).

**Throttling del ADR-0116 para actualizaciones periódicas de la UI:**

El Panel Operativo usa polling simple: Flutter llama `getRecentAuditEvents` cada N segundos. El ADR-0116 establece que el throttling se aplica **en Rust antes de cruzar la frontera**, no en Dart. Para el Panel (EPIC-0), el polling a 1-5 segundos es completamente aceptable:

- 1 llamada cada 1-5 segundos → ningún riesgo de saturar la BD ni el hilo de Flutter.
- Los streams de alta frecuencia (telemetría en tiempo real, velas cada 100ms) son otro caso: ahí sí se aplica throttling en Rust con `tokio::time::interval` antes de que el dato cruce la frontera. Ese mecanismo se implementará en EPIC-1+ cuando lleguen features con carga masiva.

`ZeroCopyBuffer` (buffers Arrow de memoria compartida) se reserva para cargas masivas puntales (cargar un dataset completo para inspección), no para el polling periódico del Panel.

---

## Trucos de Senior

### La ruta de módulo en `public_interface.rs` no es la raíz del crate

En Rust, cuando un crate tiene un `public_interface.rs`, los tipos re-exportados en él **no** están en la raíz del crate automáticamente. El acceso correcto es:

```rust
// INCORRECTO — no existe `shared::Clock`
use shared::{Clock, SystemClock};

// CORRECTO — el módulo se llama `public_interface`
use shared::public_interface::{Clock, SystemClock};
```

Esto no es evidente si eres nuevo en Rust: la re-exportación (`pub use`) en `public_interface.rs` pone los tipos en `shared::public_interface::*`, no en `shared::*`. Para que estuvieran en la raíz haría falta re-exportar también desde `lib.rs` (`pub use crate::public_interface::*;`), que en este proyecto no ocurre por diseño.

### El cfg `frb_expand` y cómo silenciarlo sin contaminar el código

flutter_rust_bridge usa un cfg interno `frb_expand` durante la fase de expansión de macros en el codegen. Cuando el codegen no ha corrido aún, Rust lanza un warning de cfg no declarado. Para silenciarlo limpiamente:

```toml
# crates/bridge/Cargo.toml — NO en el código fuente
[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = ['cfg(frb_expand)'] }
```

Alternativa sucia (no usar): poner `#[allow(unexpected_cfgs)]` en cada archivo. El `Cargo.toml` es el lugar correcto para declarar cfgs esperados.

### `unwrap_or_default()` vs `unwrap_or("")` para `Option<String>`

Ambas producen el mismo resultado (`""`), pero `unwrap_or_default()` es el idioma Rust: llama al método `Default::default()` del tipo interno (`String::default()` devuelve `""`). Es más expresivo y evita introducir el literal `""` en el código — si el tipo cambiara a `Option<u64>`, `unwrap_or_default()` daría `0` automáticamente, mientras que `unwrap_or("")` no compilaría.

---

> Sesión ampliada: 2026-06-21 — lecciones del primer lanzamiento real (post-STORY-015)

### El `ioDirectory` generado por FRB codegen es incorrecto en workspaces Cargo

**Contexto:** después de cerrar STORY-015 con `flutter build linux` verde, intentar `flutter run -d linux` produjo:

```
Failed to load dynamic library 'libbridge.so': cannot open shared object file: No such file or directory
```

La causa no fue que `libbridge.so` no existiera — existía en `target/release/libbridge.so`. El problema fue que el loader de FRB buscaba en el lugar equivocado.

**Cómo FRB encuentra la librería en desktop:**

`frb_generated.dart` contiene:
```dart
static const kDefaultExternalLibraryLoaderConfig = ExternalLibraryLoaderConfig(
  stem: 'bridge',
  ioDirectory: '../crates/bridge/target/release/',  // ← generado por codegen
  ...
);
```

El loader (`flutter_rust_bridge/src/loader/_io.dart`) resuelve el path así:
```dart
Directory.current.uri.resolve(ioDirectory)
// con CWD=ui/ → resuelve a: Drasus-Engine/crates/bridge/target/release/
```

**El problema:** en un workspace Cargo, el directorio `target/` vive en la **raíz del workspace** (`Drasus-Engine/target/`), no dentro del crate (`crates/bridge/target/`). Cargo consolida la salida de todos los crates en un único `target/` para evitar recompilaciones. El codegen no detecta este comportamiento de workspace y genera el path del crate individual.

**El fix:** cambiar `ioDirectory` en `frb_generated.dart` después de cada regeneración:

```dart
// Incorrecto (lo que genera el codegen):
ioDirectory: '../crates/bridge/target/release/',

// Correcto (donde realmente está la .so en un workspace):
ioDirectory: '../target/release/',
```

**Qué plataformas están afectadas:**

Sólo los tres desktops usan `ioDirectory`. El loader de iOS usa enlace estático vía Xcode, el de Android usa JNI desde el APK, el de Web usa WASM. En Drasus Engine, iOS/Android son clientes gRPC (ADR-0134) — no usan FFI en absoluto.

| Desktop | Archivo que busca |
|---|---|
| Linux | `libbridge.so` |
| macOS | `libbridge.dylib` |
| Windows | `bridge.dll` |

Los tres se resuelven con el mismo fix de `ioDirectory` porque comparten el mismo `target/release/` del workspace.

**Orden correcto de arranque en desarrollo:**

```bash
# Paso 1 — desde la raíz del workspace (Drasus-Engine/)
cargo build --release -p bridge
# Produce: target/release/libbridge.so (Linux) / .dylib (macOS) / .dll (Windows)

# Paso 2 — desde ui/ (para que Directory.current sea ui/)
cd ui && flutter run -d linux
```

**Por qué `flutter build` pasó pero `flutter run` no:**

`flutter build linux` compila el binario Dart (AOT) y produce el ejecutable. El compilador Dart no llama a `DynamicLibrary.open` — eso ocurre en **tiempo de ejecución**, cuando Dart inicializa el Bridge. Por eso un build puede ser verde y el lanzamiento puede crashear: son dos fases distintas, y la falta de la `.so` solo es visible en la segunda.
