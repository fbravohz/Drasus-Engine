# Structs, `impl` y los `derive` automáticos

## Concepto

Un **struct** es la forma que tiene Rust de agrupar varios datos relacionados bajo un solo nombre — el equivalente a una clase sin métodos (todavía):

```rust
pub struct TelemetrySampleContent {
    pub metric_name: String,
    pub execution_latency_ms: Option<i64>,
    // ...
}
```

Esto crea un *tipo* nuevo. Para construir un valor de ese tipo, se usa una **struct literal** — el nombre del tipo seguido de `{ campo: valor, ... }`:

```rust
let content = TelemetrySampleContent {
    metric_name: "ingest.hot_path_latency".to_string(),
    execution_latency_ms: Some(7),
    // ... el resto de los campos
};
```

### Composición: un struct puede envolver a otro

`TelemetrySample` no repite los campos de `TelemetrySampleContent` — los contiene como un campo más:

```rust
pub struct TelemetrySample {
    pub id: String,
    pub created_at_ns: i64,
    // ...campos "universales" (Grupo I)...
    pub content: TelemetrySampleContent,  // <- aquí "envuelve" al otro struct
}
```

Esto es exactamente el mismo patrón que ya existía en `audit_log.rs` con `AuditEvent` envolviendo a `AuditEventContent` — separa "lo que es igual para cualquier muestra" (identidad, hash) de "lo que es específico de esta Feature" (qué se midió). Para acceder al campo de adentro: `sample.content.metric_name`.

### `impl`: cómo se le agregan métodos a un struct

Un struct por sí solo no tiene comportamiento. Los métodos se agregan en un bloque `impl` separado:

```rust
impl TelemetryBuffer {
    pub fn record_heartbeat(&self, metric_name: impl Into<String>) {
        // ...
    }
}
```

`&self` es el primer parámetro de cualquier método que necesite leer (sin tomar ownership) la instancia sobre la que se llama — es lo que permite escribir `buffer.record_heartbeat(...)` en vez de pasar `buffer` como un argumento más.

### `#[derive(...)]`: pedirle al compilador que escriba código repetitivo por ti

Encima de casi todos los structs de este código aparece algo como:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetrySample { /* ... */ }
```

Cada uno de estos nombres es un *trait* (ver `traits-and-generics.md`) que el compilador puede implementar automáticamente, campo por campo, si tú lo pides con `derive`:
- `Debug`: te permite imprimir el valor con `{:?}` (para depurar).
- `Clone`: te permite duplicar el valor con `.clone()`.
- `PartialEq` / `Eq`: te permite comparar dos valores con `==`.

Sin `derive`, tendrías que escribir esa lógica a mano para cada struct — sería ruido puro.

## Trucos de Senior

- `..request` (la "struct update syntax") permite construir un valor nuevo copiando todos los campos de otro excepto los que listas explícitamente. Se usa en `job_executor.rs`:
  ```rust
  let request = NewJob {
      session_id: request.session_id.or_else(|| /* ... */),
      ..request   // el resto de los campos vienen de `request` tal cual
  };
  ```
  Sin esto, habría que repetir cada campo a mano.
- No le pongas `derive(Clone)` a un struct enorme que cargue datos pesados solo "por si acaso" — cada `.clone()` después tendrá ese costo real. En este código, `TelemetrySample` es pequeño (unos cuantos `String`/`i64`/`Option`), así que clonarlo para guardarlo en `chain_state` es barato a propósito.
