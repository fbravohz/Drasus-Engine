# Traits, objetos `dyn` y `impl Trait` como parámetro

## Concepto

Un **trait** es un contrato: dice "cualquier tipo que implemente este trait promete tener estos métodos", sin decir nada de cómo están implementados. Es parecido a una interfaz en otros lenguajes.

```rust
pub trait Clock: Send + Sync {
    fn timestamp_ns(&self) -> i64;
}
```

Cualquier tipo que implemente `fn timestamp_ns(&self) -> i64` puede "ser un `Clock`". En este código hay dos: `SystemClock` (lee el reloj real del sistema operativo) y `DeterministicClock` (un reloj falso, controlado a mano, para que los tests sean reproducibles bit a bit). El resto del código nunca necesita saber cuál de los dos está usando — solo le importa que "sea un `Clock`".

### `dyn Trait`: decidir el tipo concreto en tiempo de ejecución

```rust
clock: Arc<dyn Clock>,
```

`dyn Clock` significa "algún tipo que implementa `Clock`, no sé cuál exactamente en tiempo de compilación". Esto permite que `TelemetryBuffer` se construya una vez con `SystemClock` en producción y otra vez con `DeterministicClock` en los tests, sin que `TelemetryBuffer` necesite dos versiones de su código — el mismo patrón que ya usaba `JobExecutor`.

### `impl Trait` como tipo de un parámetro: "acepto cualquier cosa que se pueda convertir en X"

```rust
pub fn record_heartbeat(&self, metric_name: impl Into<String>) {
    self.enqueue(metric_name.into(), None, None);
}
```

`impl Into<String>` significa: "acepto cualquier valor que tenga una conversión hacia `String`". Esto permite llamar a la función tanto con un `&str` (`"job_executor.heartbeat"`) como con un `String` ya construido, sin que quien llama tenga que escribir `.to_string()` por su cuenta — la conversión la hace `.into()` adentro de la función. Es azúcar sintáctica para no obligar a la persona que llama a adivinar qué tipo exacto se espera.

## Trucos de Senior

- `Send + Sync` en `pub trait Clock: Send + Sync` no son métodos — son **trait bounds** que dicen "cualquier `Clock` debe poder moverse entre hilos (`Send`) y compartirse entre hilos a través de una referencia (`Sync`)". Sin esto, no podrías meter un `Arc<dyn Clock>` dentro de una tarea de Tokio que corre en otro hilo — el compilador te lo rechazaría con un error de tipos, no en tiempo de ejecución.
- Prefiere `impl Trait` como parámetro (`metric_name: impl Into<String>`) sobre pedir directamente `String` cuando la función solo necesita "algo convertible" — es una mejora de ergonomía sin costo: la conversión ocurre una sola vez, dentro de la función, no en cada sitio donde se la llama.
