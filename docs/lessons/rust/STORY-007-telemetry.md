# STORY-007 — Telemetría: lecciones de Rust

> **Story:** [STORY-007 — Telemetría técnica](../../execution/STORY-007-telemetry.md) (buffer de alta velocidad + señal de vida).
> **Archivos que esta Story produjo y que se citan abajo:** `crates/shared/src/domain/telemetry.rs`, `crates/shared/src/persistence/telemetry.rs`, `crates/shared/src/orchestrator/telemetry.rs`, `migrations/0004_telemetry.sql`, `crates/shared/src/public_interface.rs`.
> **Modo de Acompañamiento:** Bloque 1 en Mentor (el usuario tecleó `TelemetrySampleContent`, con un defecto detectado en la relectura — `process_id` duplicado); el resto de la Story en Docente (ADR-0122/ADR-0124).

## Concepto

### `Option<T>` — expresar "esto puede no existir"

En muchos lenguajes, "este valor no existe" se representa con `null`. El problema: el compilador no te obliga a comprobarlo, así que es fácil olvidar el caso "no hay nada" y el programa explota en producción. Rust no tiene `null`. En su lugar, cuando un valor puede estar ausente, su tipo se envuelve en `Option<T>`:

```rust
enum Option<T> {
    Some(T),  // hay un valor, y es de tipo T
    None,     // no hay nada
}
```

En `domain/telemetry.rs`, `TelemetrySampleContent.execution_latency_ms` es `Option<i64>`:
- Una muestra de **latencia** trae `Some(7)` (7 milisegundos).
- Una muestra de **heartbeat** trae `None` (no se midió ninguna latencia).

El tipo mismo hace imposible representar un estado inválido como "heartbeat con latencia 7" por accidente — no hay un valor especial como `-1` que alguien pueda mal-interpretar. Esto fue, de hecho, el primer defecto real de la Story: en el Bloque 1 (Modo Mentor) el campo `process_id: String` quedó duplicado por error de tecleo — un error de otra naturaleza (dos campos con el mismo nombre, ni siquiera compila), pero ilustra por qué el Modo Docente relee con `Read` antes de avanzar al siguiente bloque.

Métodos de `Option` que aparecieron en el código:
- `.as_deref()`: convierte `&Option<String>` en `Option<&str>` — se usó en `canonical_bytes` (`domain/telemetry.rs`) para leer `details_json`, `logic_hash`, etc. sin clonarlos.
- `.map(f).unwrap_or_default()`: transforma el valor de adentro si existe, o devuelve el valor por defecto si no. Se usó para convertir `execution_latency_ms` a texto dentro del hash canónico:
  ```rust
  content.execution_latency_ms.map(|value| value.to_string()).unwrap_or_default()
  ```
  Un Junior escribiría un `match` de 5 líneas para esto — ver "Trucos de Senior" abajo.

### Structs, `impl` y los `derive` automáticos

Un **struct** agrupa varios datos relacionados bajo un solo nombre — el equivalente a una clase sin métodos (todavía). `TelemetrySample` (`domain/telemetry.rs`) no repite los campos de `TelemetrySampleContent`: lo envuelve como un campo más —

```rust
pub struct TelemetrySample {
    pub id: String,
    pub created_at_ns: i64,
    // ...campos "universales" (Grupo I, ADR-0020)...
    pub content: TelemetrySampleContent,  // <- envuelve al otro struct
}
```

Exactamente el mismo patrón que ya existía en `audit_log.rs` con `AuditEvent` envolviendo `AuditEventContent` — separa "lo que es igual para cualquier muestra" (identidad, hash) de "lo que es específico de esta Feature" (qué se midió).

Los métodos se agregan en un bloque `impl` separado, con `&self` como primer parámetro cuando solo necesitan leer la instancia:

```rust
impl TelemetryBuffer {
    pub fn record_heartbeat(&self, metric_name: impl Into<String>) { /* ... */ }
}
```

Encima de los structs de esta Story aparece `#[derive(Debug, Clone, PartialEq, Eq)]` — le pide al compilador que escriba, campo por campo, la lógica de imprimir (`Debug`), duplicar (`Clone`) y comparar (`PartialEq`/`Eq`) sin que tengamos que escribirla a mano.

### `match`, `if let` y desestructurar tuplas

`match` es el "switch" de Rust, pero más estricto: el compilador exige cubrir todos los casos. `build_sample` (`domain/telemetry.rs`) lo usa para decidir cómo encadenar la nueva muestra con la anterior:

```rust
let (event_sequence_id, audit_chain_hash, previous_audit_hash) = match previous {
    Some(previous_sample) => (
        previous_sample.event_sequence_id + 1,
        Some(previous_sample.audit_hash.clone()),
        previous_sample.audit_hash.clone(),
    ),
    None => (1, None, GENESIS_PREVIOUS_HASH.to_string()),
};
```

Dos cosas pasan a la vez: **desestructuración de tupla** (`(a, b, c) = ...` asigna tres valores en un solo paso — una tupla es "varios valores agrupados sin nombre de campo", se accede por posición) y **binding del valor interior** (en `Some(previous_sample) => ...`, `previous_sample` apunta al valor que estaba *adentro* del `Some`, sin desenvolverlo a mano).

### Manejo de errores: `Result`, enums de error propios y el operador `?`

Rust no tiene excepciones. Una función que puede fallar devuelve `Result<T, E>` — la misma idea de `Option`, pero con un motivo de fallo adentro del caso negativo. `persistence/telemetry.rs` define su propio enum de error en vez de exponer `sqlx::Error` directamente:

```rust
#[derive(Debug)]
pub enum TelemetryError {
    Database(sqlx::Error),
}
```

¿Por qué? FCIS (ADR-0003) exige que cada módulo tenga su propio tipo de error en la frontera — el día de mañana se puede cambiar de motor de base de datos sin que el error se filtre por todo el código. Tres impls lo convierten en "un error de verdad": `Display` (cómo se imprime), `std::error::Error` (lo marca como error para el ecosistema) y `From<sqlx::Error>` (conversión automática). Esa tercera pieza habilita el operador `?`:

```rust
pub async fn purge_older_than(&self, cutoff_ns: i64) -> Result<u64, TelemetryError> {
    let result = sqlx::query("DELETE FROM telemetry_samples WHERE created_at < ?")
        .bind(cutoff_ns)
        .execute(self.pool)
        .await?;   // si falla, convierte a TelemetryError y devuelve ya mismo

    Ok(result.rows_affected())
}
```

### Ownership, préstamos (`&`) y lifetimes (`'a`)

Cada valor en Rust tiene exactamente un dueño; cuando el dueño sale de su ámbito, el valor se destruye solo, sin recolector de basura. `build_sample` toma ownership de `id` y `content` (los "mueve" adentro de la función), pero solo **presta** la muestra anterior:

```rust
pub fn build_sample(
    id: String,
    created_at_ns: i64,
    content: TelemetrySampleContent,
    previous: Option<&TelemetrySample>,  // PRESTA, no se queda con la muestra anterior
) -> TelemetrySample { /* ... */ }
```

`TelemetryRepository<'a>` (`persistence/telemetry.rs`) guarda una referencia prestada al pool de conexiones:

```rust
pub struct TelemetryRepository<'a> {
    pool: &'a SqlitePool,
}
```

`'a` es una etiqueta de tiempo de compilación: dice "esta referencia prestada no puede sobrevivir más que el `SqlitePool` original" — el compilador rechaza cualquier código que intente usar el repositorio después de que el pool prestado ya fue destruido. Mismo patrón que ya usaban `AuditLogRepository<'a>` y `JobRepository<'a>`.

Cuando sí hace falta que varias tareas async compartan el mismo dato (no solo un préstamo temporal), se usa `Arc<T>`:

```rust
#[derive(Clone)]
pub struct TelemetryBuffer {
    shared: Arc<Shared>,  // cada .clone() comparte el MISMO Shared
}
```

Clonar un `Arc` es barato — no copia lo de adentro, solo incrementa un contador de referencias.

### Traits, objetos `dyn` y `impl Trait` como parámetro

Un **trait** es un contrato: cualquier tipo que lo implemente promete tener ciertos métodos. `Clock` (`domain/clock.rs`, reusado por esta Story) define `fn timestamp_ns(&self) -> i64`; tanto `SystemClock` (reloj real) como `DeterministicClock` (reloj de test) lo implementan, y `TelemetryBuffer` guarda `clock: Arc<dyn Clock>` — "algún tipo que es un `Clock`, decidido en tiempo de ejecución", sin que el resto del código necesite saber cuál.

`record_heartbeat` usa `impl Trait` como tipo de parámetro:

```rust
pub fn record_heartbeat(&self, metric_name: impl Into<String>) {
    self.enqueue(metric_name.into(), None, None);
}
```

`impl Into<String>` acepta tanto un `&str` literal (`"job_executor.heartbeat"`) como un `String` ya construido — la conversión la hace `.into()` adentro de la función, sin obligar a quien llama a adivinar el tipo exacto.

### `async`/`await`, `std::sync::Mutex` vs. `tokio::sync::Mutex`, y canales `mpsc`

Esta es la decisión de diseño más importante de `orchestrator/telemetry.rs`. El requisito de la Feature (criterio #3 de la Story) es que registrar una muestra tarde menos de 50 microsegundos. Como encolar en un canal y tomar un Mutex breve nunca esperan a nada, `record_latency`/`record_heartbeat` son funciones **síncronas** (sin `async`, sin `.await`) — el vaciado a disco (lo lento) vive en una tarea de fondo separada (`spawn_flush_task`) que el que llama nunca espera.

`std::sync::Mutex` (usado en `chain_state` de `Shared`) es **bloqueante**: si está ocupado, el hilo que lo pide se congela sin devolverle el control a Tokio. Solo es seguro cuando la sección protegida es muy breve y nunca hace `.await` mientras lo tiene tomado — exactamente el caso de `chain_state`: se toma, se lee/escribe una variable, se suelta. `tokio::sync::Mutex` (usado en `queue_rx`) es async-aware: si está ocupado, la tarea se pausa sin congelar el hilo — hace falta cuando sí se hace `.await` con el lock tomado (como al esperar `.lock().await` antes de tomar el receptor del canal en `spawn_flush_task`).

El canal `mpsc::unbounded_channel()` es la pieza clave de "no bloqueante": `queue_tx.send(valor)` nunca espera a que el receptor esté listo, solo agrega el valor a una cola en memoria y devuelve el control de inmediato. Del lado receptor, `spawn_flush_task` usa `receiver.try_recv()` en bucle para drenar lo acumulado, distinguiendo `TryRecvError::Empty` (no hay nada ahora, sigue vivo) de `TryRecvError::Disconnected` (ya no queda ningún emisor, hay que salir del bucle).

### SQLx (transacciones, binds) y pruebas en Rust

`insert_batch` (`persistence/telemetry.rs`) agrupa varias inserciones en una sola transacción:

```rust
let mut tx = self.pool.begin().await?;
for sample in samples {
    sqlx::query("INSERT INTO telemetry_samples (...) VALUES (...)")
        .bind(&sample.id)
        // ...
        .execute(&mut *tx)
        .await?;
}
tx.commit().await?;
```

Cada `tx.commit()` implica un `fsync` real al disco — el paso más lento de cualquier escritura. Insertar 50 muestras en 50 transacciones separadas paga ese costo 50 veces; en una sola transacción lo paga una sola vez. `.bind(...)` separa "la forma de la consulta" (el `?` en el SQL) de "los datos reales" — así se evita la inyección SQL sin concatenar texto.

Las pruebas de esta Story usan dos anotaciones: `#[test]` (síncrona) y `#[tokio::test]` (envuelve un runtime de Tokio mínimo para poder hacer `.await` adentro). El test de durabilidad (criterio #5) usa un archivo temporal real (`tempfile::tempdir()`), no `sqlite::memory:` — una base en memoria no sobrevive a cerrar y reabrir el pool, así que no demostraría nada. El benchmark del criterio #3 mide con `std::time::Instant` sobre 1000 iteraciones y promedia, para no depender del ruido de una sola llamada. El test de no-bloqueo (criterio #4) sostiene el lock real de escritura de SQLite (`BEGIN IMMEDIATE` desde una conexión separada) durante 150ms y demuestra que 100 llamadas a `record_heartbeat` mientras tanto tardan una fracción de eso — contención real, no un `sleep` simulado.

## Trucos de Senior

- Encadenar `.map(f).unwrap_or_default()` evita un `match` de 5 líneas para una transformación de una sola expresión — mismo resultado que escribirlo a mano, menos ruido visual.
- Si solo necesitas *leer* el contenido de un `Option<String>` como texto (sin tomar ownership), `.as_deref().unwrap_or("")` es más barato que clonar el `String` solo para descartarlo después.
- `..request` (struct update syntax, usada en `job_executor.rs` y aplicable al mismo patrón en telemetría) construye un valor nuevo copiando todos los campos de otro excepto los que listas explícitamente — evita repetir cada campo a mano.
- No le pongas `derive(Clone)` "por si acaso" a un struct que cargue datos pesados — cada `.clone()` después paga ese costo real. `TelemetrySample` es pequeño a propósito, así que clonarlo para guardarlo en `chain_state` es barato.
- `let ... else` es más legible que anidar un `match` con una rama vacía cuando solo te interesa el caso positivo de un `Option`.
- `matches!(valor, Patron)` es un atajo para "esto encaja con tal patrón" cuando solo te interesa un `bool`, sin extraer ningún valor de adentro (usado en `domain/job.rs` para `validate_transition`, mismo dominio de `shared` que telemetría).
- Nunca uses `.unwrap()` en código de producción — el operador `?` es la forma idiomática de propagar un error hacia quien sabe qué hacer con él. `.unwrap()` solo es aceptable en tests, donde un panic es la forma correcta de fallar ruidosamente.
- Si una función solo necesita *leer* un valor, pide `&T`, no `T` — así quien te llama no pierde su propio valor.
- `Arc<dyn Clock>` combina dos ideas en una línea: `Arc` para compartir el dueño entre tareas async, `dyn Clock` para que el tipo concreto (reloj real vs. reloj de test) se decida en producción sin que el resto del código lo sepa.
- Nunca mezcles un `std::sync::Mutex` con un `.await` adentro de la sección crítica — puede congelar el runtime de Tokio entero, no solo esa tarea. Si dudas, usa `tokio::sync::Mutex`; solo bájate al síncrono cuando estés seguro de que la sección nunca espera a nada.
- Tratar `TryRecvError::Empty` y `TryRecvError::Disconnected` como lo mismo es un bug sutil: `Disconnected` significa que ya no queda ningún emisor vivo, así que seguir reintentando para siempre es un bucle inútil.
- Una lista vacía como entrada de `insert_batch` se trata como no-op (`if samples.is_empty() { return Ok(()); }`) — evita abrir una transacción que no va a escribir nada.
- Forzar contención real (sostener el lock de escritura de SQLite desde una conexión separada) demuestra una garantía de concurrencia de verdad, en vez de simularla con un `sleep` que no prueba nada sobre el recurso real en juego.
