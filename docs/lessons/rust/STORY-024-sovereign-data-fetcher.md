# Lección STORY-024: Sovereign Data Fetcher

> Story: [STORY-024 — Descarga híbrida soberana de datos de mercado](../../execution/STORY-024-sovereign-data-fetcher.md)
>
> Conceptos cubiertos: FCIS en Rust, traits como inyección de dependencias, `JoinSet` + `Arc<dyn Trait>` + `Semaphore` para concurrencia real, test honesto con `Arc<AtomicUsize>`, tipos `enum` para resultados, persistencia con SQLx, ciclo de vida de Job (ADR-0011), errores encadenados con `thiserror`.
>
> **Regresión resuelta (STORY-024):** la primera entrega usaba un bucle secuencial con `Semaphore` que nunca producía concurrencia. Esta lección documenta el patrón correcto (`JoinSet::spawn` + permiso DENTRO de la tarea) y explica por qué el patrón incorrecto parece funcionar pero no lo hace.

---

## Concepto

### 1. FCIS — Functional Core / Imperative Shell en Rust

**El problema que resuelve:** ¿cómo se prueba lógica de negocio sin tener que conectarse a internet o a una base de datos?

La respuesta del proyecto es FCIS: separar la lógica pura (que no toca nada externo) de la cáscara (que sí lo hace). En Rust, esto se traduce en una regla de imports muy estricta.

**Cómo se ve en el código:**

```rust
// crates/features/data/sovereign-data-fetcher/src/domain.rs
// -- no hay ningun `use reqwest`, `use tokio`, `use sqlx`, `use std::fs`

pub fn plan_downloads(requested: TimeRange, bulk_inventory: &[BulkFileInfo]) -> DownloadPlan {
    let mut relevant_files: Vec<&BulkFileInfo> = bulk_inventory
        .iter()
        .filter(|f| f.time_range().overlaps(&requested))
        .collect();
    relevant_files.sort_by_key(|f| f.start_ns);
    // ...
}
```

Esta función no sabe si los archivos vienen de internet, de disco o de un vector en memoria. Simplemente toma datos, aplica lógica, devuelve un resultado. Por eso se puede llamar en un test en microsegundos, sin red.

**Dónde está el I/O:** en `orchestrator.rs`, que sí importa `reqwest`, `tokio::fs`, etc. Esa es la cáscara — y los tests no la prueban directamente; la reemplazan con adaptadores falsos.

**La regla para recordarla:** si `domain.rs` importa `tokio`, hay un error de arquitectura. El criterio 9 de la Orden lo verifica con un `grep`.

---

### 2. Traits como inyección de dependencias (los "puertos" del hexágono)

**El problema:** el `fetch()` necesita descargar archivos de internet, pero en los tests no podemos contactar internet. ¿Cómo resolvemos esto sin duplicar código?

En Rust, un `trait` es como un contrato: defines qué operaciones debe tener un tipo sin decir cómo las implementa. Esto nos permite tener una implementación real (HTTP) y una implementación falsa (en memoria) que el test puede usar.

```rust
// crates/features/data/sovereign-data-fetcher/src/public_interface.rs

#[async_trait]
pub trait BulkSource: Send + Sync {
    async fn list_inventory(&self, range: &TimeRange) -> Result<Vec<BulkFileInfo>, FetchError>;
    async fn download_file(&self, file: &BulkFileInfo, dest_path: &Path) -> Result<u64, FetchError>;
}
```

`Send + Sync` son dos traits automáticos de Rust: `Send` significa "este valor se puede mover entre hilos" y `Sync` significa "se puede referenciar desde varios hilos a la vez". Son obligatorios porque el orquestador es `async` y puede ejecutarse en cualquier hilo del pool de Tokio.

**Cómo se usa en producción:**

```rust
// orchestrator.rs — adaptador real
pub struct ReqwestBulkSource { client: reqwest::Client, base_url: String }

#[async_trait]
impl BulkSource for ReqwestBulkSource { /* HTTP real */ }
```

**Cómo se usa en tests:**

```rust
// tests/integration_tests.rs — adaptador falso
struct FakeSuccessBulkSource { files: Vec<BulkFileInfo> }

#[async_trait]
impl BulkSource for FakeSuccessBulkSource {
    async fn download_file(&self, file: &BulkFileInfo, dest_path: &Path) -> Result<u64, FetchError> {
        tokio::fs::write(dest_path, format!("FAKE:{}", file.filename).as_bytes()).await.ok();
        Ok(file.estimated_size_bytes)
    }
}
```

El truco: `fetch()` acepta `&dyn BulkSource` (una referencia a cualquier cosa que implemente el trait). El test pasa el falso; producción pasa el real.

**Por qué `async_trait`:** en Rust estable, las funciones `async` en traits no funcionan directamente con `dyn Trait` (el compilador no sabe qué tamaño tiene el futuro devuelto). El crate `async-trait` lo resuelve automáticamente encapsulando el futuro en un `Box`.

---

### 3. Concurrencia real con `JoinSet` + `Arc<dyn Trait>` + `Semaphore`

**El problema:** necesitamos descargar N archivos con un máximo de K descargas simultáneas, y verificar con un test que la concurrencia es REAL, no aparente.

---

#### 3a. Por qué el bucle secuencial con `Semaphore` NO produce concurrencia

El patrón incorrecto (el que produjo el defecto de esta Story):

```rust
// INCORRECTO — orchestrator.rs (versión anterior, NO usar)
let semaphore = Arc::new(Semaphore::new(config.concurrent_downloads));

for file in &plan.bulk_files {
    // ❌ El permit se adquiere ANTES del download, en el hilo principal.
    let permit = semaphore.clone().acquire_owned().await.expect("...");

    // ❌ El download ocurre dentro del mismo hilo que adquirió el permit.
    // No hay otra tarea corriendo en paralelo.
    bulk_source.download_file(file, &dest_path).await?;

    drop(permit); // el permit se suelta DESPUÉS del download completo
    // → siguiente iteración del for
}
```

Qué pasa en la práctica:
1. El bucle adquiere el permit → hay 1 permit tomado.
2. El bucle hace `.await` del download → cede el control, pero no hay ninguna otra
   tarea esperando, porque no se ha lanzado ninguna.
3. El download termina → se suelta el permit → siguiente iteración.

Resultado: **una descarga a la vez**, siempre. El `Semaphore` nunca llega a limitar nada porque nunca hay más de 1 descarga intentando empezar.

---

#### 3b. El patrón correcto: `JoinSet` + `Arc<dyn BulkSource>`

El `Semaphore` solo produce concurrencia cuando hay **múltiples tareas compitiendo por él al mismo tiempo**. Para eso necesitamos `tokio::task::JoinSet::spawn`:

```rust
// CORRECTO — orchestrator.rs (versión actual)
use tokio::task::JoinSet;

// Arc porque las tareas concurrentes necesitan compartir la fuente.
// Una &dyn BulkSource no puede cruzar el límite de spawn ('static).
// Un Arc<dyn BulkSource> sí: cada tarea clona el Arc (clonar un Arc
// solo incrementa el contador de referencias — no clona el valor).
let semaphore = Arc::new(Semaphore::new(config.concurrent_downloads));
let mut join_set: JoinSet<Result<u64, FetchError>> = JoinSet::new();

for file in bulk_files.into_iter() {
    let sem = Arc::clone(&semaphore);
    let src = Arc::clone(&bulk_source);   // <-- Arc<dyn BulkSource>
    let dest_path = dest_dir.join(&file.filename);

    // ✅ Se lanza la tarea al runtime pero no espera a que termine.
    // El bucle continúa inmediatamente al siguiente archivo.
    join_set.spawn(async move {
        // ✅ El permit se adquiere DENTRO de la tarea, no fuera.
        // Varias tareas pueden estar esperando el permit simultáneamente.
        // Cuando una termina y lo suelta, la siguiente empieza de inmediato.
        let _permit = sem.acquire_owned().await.expect("semáforo cerrado");

        // El download ocurre aquí, en la tarea concurrente.
        src.download_file(&file, &dest_path).await
    });
}

// join_next().await cede el control al runtime, que empieza a ejecutar
// todas las tareas lanzadas. Recolecta resultados conforme terminan.
while let Some(result) = join_set.join_next().await {
    match result {
        Ok(Ok(bytes)) => { /* éxito */ }
        Ok(Err(e))    => return Err(e),    // fallo de descarga
        Err(join_err) => { /* tarea cancelada/pánico */ }
    }
}
```

Qué pasa ahora:
1. El bucle lanza 10 tareas sin esperar a ninguna.
2. `join_next().await` cede el control → el runtime empieza las tareas.
3. Las 3 primeras tareas adquieren permit (las 3 slots disponibles) y empiezan a descargar.
4. Las otras 7 quedan esperando el permit — pero **esperan en paralelo**.
5. En cuanto una de las 3 termina y suelta el permit, la siguiente empieza de inmediato.

**Regla para recordarlo:** el `Semaphore` limita cuántas tareas están en su sección crítica a la vez. Para que eso tenga sentido, primero necesitas **tareas** — y eso requiere `spawn`.

---

#### 3c. `Arc<AtomicUsize>` — cómo medir la concurrencia en el test

El test honesto del criterio 5 mide el pico de descargas simultáneas con un contador atómico compartido:

```rust
// tests/integration_tests.rs

struct FakeCountingBulkSource {
    active: Arc<AtomicUsize>,           // cuántas descargas hay activas ahora mismo
    peak_concurrent: Arc<AtomicUsize>,  // máximo observado en toda la ejecución
}

async fn download_file(&self, ...) -> Result<u64, FetchError> {
    // Incrementa el contador y registra el pico.
    let current = self.active.fetch_add(1, Ordering::SeqCst) + 1;
    self.peak_concurrent.fetch_max(current, Ordering::SeqCst);

    // Pausa de 10ms: da tiempo a que otras tareas lleguen a este punto
    // antes de que esta descarga termine. Sin la pausa, las tareas podrían
    // terminar tan rápido que nunca se solaparían.
    tokio::time::sleep(Duration::from_millis(10)).await;

    self.active.fetch_sub(1, Ordering::SeqCst);
    Ok(file.estimated_size_bytes)
}
```

- **`Arc`**: permite que el test y la fuente compartan el mismo contador. El test clona el `Arc` antes de mover `source` al `JoinSet`, de modo que puede leer el pico después de que terminen todas las descargas.
- **`AtomicUsize`**: entero sin `Mutex`. `fetch_add(1)` suma 1 y devuelve el valor anterior — en una sola operación indivisible de CPU.
- **`fetch_max`**: actualiza el máximo de forma atómica. Sin esto tendríamos una carrera de datos: dos tareas leyendo el máximo, comparando con su `current` local y escribiendo el mayor — podría perderse el pico real.
- **`Ordering::SeqCst`**: el modo más conservador. Garantiza que todas las operaciones atómicas de todos los hilos se ven en el mismo orden. Para tests de pico de concurrencia es el más seguro.

**La aserción que detecta el defecto:**

```rust
let observed_peak = peak.load(Ordering::SeqCst);

// Límite superior: el semáforo funcionó.
assert!(observed_peak <= 3);

// ⭐ Límite inferior: hubo concurrencia real.
// Con código secuencial (sin JoinSet), el pico siempre sería 1.
// Esta aserción falla con el código incorrecto y pasa con el correcto.
assert!(
    observed_peak >= 2,
    "la descarga debe ser concurrente (pico >= 2, observado: {observed_peak})"
);
```

Un test que solo verifica `peak > 0` o `peak <= límite` **siempre pasaría** aunque el código fuera completamente secuencial. La aserción `peak >= 2` es la que hace el test honesto.

---

### 4. `enum` para resultados de dominio — más que `Result`

**El problema:** ¿cómo devolver "suficiente" o "insuficiente" de forma que el compilador obligue a manejar ambos casos?

```rust
// domain.rs
pub enum DiskSpaceResult {
    Sufficient,
    Insufficient { required_bytes: u64, available_bytes: u64 },
}

pub fn check_disk_space(required_bytes: u64, available_bytes: u64) -> DiskSpaceResult {
    if available_bytes >= required_bytes {
        DiskSpaceResult::Sufficient
    } else {
        DiskSpaceResult::Insufficient { required_bytes, available_bytes }
    }
}
```

La ventaja sobre un simple `bool`: el resultado `Insufficient` lleva consigo los datos del error (cuánto se necesita, cuánto hay). El `match` del orquestador TIENE que manejar ambos casos — el compilador no deja olvidar el caso de error.

```rust
// orchestrator.rs
if let DiskSpaceResult::Insufficient { required_bytes, available_bytes } = check_disk_space(...) {
    return Err(FetchError::InsufficientDiskSpace { required_bytes, available_bytes });
}
```

La variante `Insufficient` "abre" sus campos con desestructuración — Rust saca `required_bytes` y `available_bytes` del enum en la misma línea.

---

### 5. Ciclo de vida de Job (ADR-0011) — durabilidad ante crashes

**El problema:** si el proceso se cae a mitad de una descarga de 5 GB, ¿cómo saber qué estaba haciendo para reanudarlo?

La respuesta es persistir el estado ANTES de empezar. El orquestador crea un `Job` en SQLite antes de descargar nada:

```rust
// orchestrator.rs
let job = job_repo.submit(NewJob {
    job_type: "SOVEREIGN_FETCH".to_string(),
    parameters: serde_json::json!({ "symbol": ..., "start_ns": ... }).to_string(),
    // ...
}).await?;

// Solo DESPUÉS de que el INSERT tiene éxito, empieza la descarga.
let job = job_repo.transition(&job, JobState::Running, Some("sovereign-fetcher")).await?;
```

Al reiniciar, `recover_interrupted_downloads` busca Jobs en estado `RUNNING`:

```rust
// orchestrator.rs
let running_jobs = job_repo.jobs_in_state(JobState::Running).await?;
for job in running_jobs {
    if job.job_type == "SOVEREIGN_FETCH" {
        job_repo.transition(&job, JobState::Queued, None).await?;
    }
}
```

**Lo importante:** RUNNING → QUEUED no es una transición "hacia atrás" anómala. El módulo `domain/job.rs` de `shared` la tiene explícitamente como válida precisamente para este caso de recovery:

```rust
// shared/src/domain/job.rs
(JobState::Running, JobState::Queued)  // recovery at startup
```

---

### 6. Errores tipados con `thiserror`

**El problema:** ¿cómo representar varios tipos de error de forma que el que llama pueda distinguirlos?

```rust
// public_interface.rs
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("disco insuficiente: se necesitan {required_bytes} bytes pero solo hay {available_bytes}")]
    InsufficientDiskSpace { required_bytes: u64, available_bytes: u64 },
    #[error("error en fuente Bulk: {0}")]
    BulkSourceFailed(String),
    #[error("error en fuente Delta: {0}")]
    DeltaSourceFailed(String),
    // ...
}
```

`thiserror` genera automáticamente la implementación de `std::error::Error` y `Display`. El atributo `#[error("...")]` define el mensaje de texto que se imprime. `{0}` referencia el primer campo de la variante tupla; `{required_bytes}` referencia el campo por nombre.

En los tests, `matches!` verifica la variante sin necesitar extraer los datos:

```rust
assert!(matches!(result, Err(FetchError::InsufficientDiskSpace { .. })));
```

El `..` ignora los campos — solo comprueba que es esa variante.

---

### 7. Encadenamiento de hashes para integridad (Perfil A)

**El problema:** ¿cómo detectar si alguien modifica un registro de descarga en la base de datos?

Cada `DownloadRecord` tiene un `audit_hash` (SHA-256 del contenido de la fila) y un `audit_chain_hash` (el `audit_hash` del registro anterior). Esto forma una cadena: si alguien modifica un registro antiguo, su `audit_hash` cambia, y ya no coincide con el `audit_chain_hash` del siguiente. La cadena queda rota.

```rust
// persistence.rs
fn compute_audit_hash(id: &str, created_at: i64, ..., source_endpoint: &str) -> String {
    const SEP: char = '\u{1F}';  // ASCII Unit Separator — no puede aparecer en datos normales
    let mut buf = String::new();
    buf.push_str(id); buf.push(SEP);
    buf.push_str(&created_at.to_string()); buf.push(SEP);
    // ... todos los campos
    let digest = Sha256::digest(buf.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}
```

El separador `\u{1F}` (ASCII Unit Separator) evita colisiones: si el contenido de un campo cambia, el hash resultante siempre cambia también. Sin separador, `"a"+"bc"` y `"ab"+"c"` producirían el mismo string antes de hashear.

---

## Trucos de Senior

### String continuation con `\` — cuidado con el espacio final

Rust permite continuar un string literal en la siguiente línea con `\` al final de la línea:

```rust
let sql = "SELECT id, created_at, \
           source_endpoint \
           FROM tabla WHERE id = ?";
// Resultado: "SELECT id, created_at, source_endpoint FROM tabla WHERE id = ?"
```

El `\` + newline elimina el newline Y todo el espacio inicial de la siguiente línea. Si olvidas el espacio ANTES del `\`, las palabras se pegan:

```rust
// BUG detectado en esta Story:
let sql = "...source_endpoint\
 FROM tabla";  // → "...source_endpointFROM tabla" ← ERROR SQL
```

La solución: siempre poner un espacio antes del `\` en el valor de la línea actual:

```rust
let sql = "...source_endpoint \
 FROM tabla";  // → "...source_endpoint FROM tabla" ✓
```

### `#[allow(clippy::too_many_arguments)]` para funciones internas

Cuando una función de ayuda interna acumula 8+ parámetros de inyección de dependencias, clippy advierte. Si crear un struct de contexto añadiría más boilerplate que claridad, el atributo puntual es la solución idiomática:

```rust
#[allow(clippy::too_many_arguments)]
async fn execute_download(config, request, bulk_source, delta_source, plan, job, pool, clock) { ... }
```

Esto NO aplica a funciones públicas o de dominio, donde muchos argumentos sí son una señal de diseño.

### `fetch_max` atómico para picos de concurrencia en tests

Para medir el pico de concurrencia sin `Mutex`, `AtomicUsize::fetch_max` actualiza el máximo de forma atómica en una sola instrucción de CPU:

```rust
// Actualiza peak_concurrent al máximo entre el valor actual y `current`.
// Sin carrera de datos, sin Mutex, sin lock.
self.peak_concurrent.fetch_max(current, Ordering::SeqCst);
```
