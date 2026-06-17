# Ownership, préstamos (`&`) y lifetimes (`'a`)

## Concepto

Esta es la regla más distintiva de Rust: **cada valor tiene exactamente un dueño**, y cuando el dueño sale de su ámbito (`{ }`), el valor se destruye automáticamente — sin recolector de basura, sin `free()` manual.

```rust
let id = "id-1".to_string(); // `id` es dueño de este String
let sample = build_sample(id, /* ... */); // la propiedad de `id` se MUEVE a build_sample
// usar `id` aquí abajo ya no compila: ya no es tuyo, se lo "regalaste" a build_sample
```

Esto se llama **mover** (`move`). Si quisieras seguir usando `id` después, tendrías que clonarlo (`id.clone()`) antes — pagas el costo de la copia a propósito, en vez de que pase desapercibido.

### Préstamos (`&T` y `&mut T`): usar un valor sin volverte su dueño

La mayoría de las veces no quieres mover un valor, solo "mirarlo" temporalmente. Para eso existen las referencias:

```rust
pub fn build_sample(
    id: String,                          // toma ownership de `id`
    created_at_ns: i64,
    content: TelemetrySampleContent,     // toma ownership de `content`
    previous: Option<&TelemetrySample>,  // SOLO PRESTA `previous` — no se lo queda
) -> TelemetrySample { /* ... */ }
```

`&TelemetrySample` es un préstamo de solo lectura: la función puede leer la muestra anterior, pero quien la llamó sigue siendo su dueño y puede seguir usándola después. El compilador verifica, en tiempo de compilación, que nunca exista una referencia a un valor que ya fue destruido — esa garantía es la que en otros lenguajes se rompe con punteros colgantes.

### Lifetimes (`'a`): cuánto tiempo es válido un préstamo

```rust
pub struct TelemetryRepository<'a> {
    pool: &'a SqlitePool,
}
```

`'a` es una etiqueta (no un valor en tiempo de ejecución, desaparece al compilar) que dice: "este struct guarda una referencia prestada, y esa referencia no puede sobrevivir más tiempo que el `SqlitePool` original". El compilador usa esa etiqueta para rechazar cualquier código que intente usar un `TelemetryRepository` después de que el `pool` que le prestaron ya fue destruido. Mismo patrón que ya existía en `AuditLogRepository<'a>` y `JobRepository<'a>` — el repositorio nunca es dueño del pool, varios repositorios pueden compartir el mismo pool prestado al mismo tiempo.

### `Arc<T>`: cuando SÍ necesitas que varios dueños compartan el mismo valor

Un préstamo (`&T`) no sirve cuando varias tareas async (que pueden vivir más tiempo que la función que las creó) necesitan acceso al mismo dato. Para eso existe `Arc<T>` ("Atomically Reference Counted") — varios `Arc` pueden apuntar al mismo valor en memoria, y el valor se destruye solo cuando el ÚLTIMO `Arc` se destruye:

```rust
#[derive(Clone)]
pub struct TelemetryBuffer {
    shared: Arc<Shared>,  // cada .clone() de TelemetryBuffer comparte el MISMO Shared
    // ...
}
```

Clonar un `Arc` es barato: NO copia el `Shared` de adentro, solo incrementa un contador. Por eso `TelemetryBuffer` se documenta como "barato de clonar" — cada clon es un handle distinto al mismo estado compartido (el mismo patrón que ya usaba `JobExecutor`).

## Trucos de Senior

- Si una función solo necesita *leer* un valor, pide `&T`, no `T` — así quien te llama no pierde su propio valor. Pedir ownership (`T`) cuando no lo necesitas obliga a quien te llama a clonar innecesariamente.
- `Arc<dyn Clock>` (en vez de `Arc<DeterministicClock>` o similar) combina dos ideas: `Arc` para compartir el dueño entre tareas async, `dyn Clock` para que el tipo concreto (reloj real vs. reloj de test) se decida en producción sin que el resto del código lo sepa — ver `traits-and-generics.md`.
