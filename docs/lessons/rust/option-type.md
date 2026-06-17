# `Option<T>` — Expresar "esto puede no existir"

## Concepto

En muchos lenguajes, "este valor no existe" se representa con `null`, `None`, `nil` o un valor especial como `-1`. El problema: el compilador no te obliga a comprobarlo, así que es fácil olvidar el caso "no hay nada" y el programa explota en producción (el famoso "null pointer exception").

Rust no tiene `null`. En su lugar, cuando un valor puede estar ausente, su tipo se envuelve en `Option<T>`, que es un enum con exactamente dos formas:

```rust
enum Option<T> {
    Some(T),  // hay un valor, y es de tipo T
    None,     // no hay nada
}
```

`T` aquí es un **genérico** — un "tipo cualquiera" que se rellena según el caso: `Option<i64>`, `Option<String>`, `Option<TelemetrySample>`, etc.

La clave: para *usar* el valor de dentro de un `Option`, Rust te obliga a manejar los dos casos. No puedes "olvidarte" del `None` — el compilador no deja compilar el código si falta un caso en un `match` (ver `pattern-matching.md`).

### Dónde apareció en este código

En `domain/telemetry.rs`, `TelemetrySampleContent.execution_latency_ms` es `Option<i64>`:
- Una muestra de **latencia** trae `Some(7)` (7 milisegundos).
- Una muestra de **heartbeat** trae `None` (no se midió ninguna latencia).

El tipo mismo hace imposible representar un estado inválido como "heartbeat con latencia 7" por accidente — no hay un tercer valor especial como `-1` que alguien pueda mal-interpretar.

### Métodos útiles que aparecieron en el código

- `.as_deref()`: convierte `&Option<String>` en `Option<&str>` — útil cuando quieres "tomar prestado" el contenido sin clonar el `String`. Se usó en `canonical_bytes` para leer `details_json`, `logic_hash`, etc. sin copiarlos.
- `.unwrap_or_default()`: si es `Some(x)`, devuelve `x`; si es `None`, devuelve el valor por defecto del tipo (para `String` es `""`, para `i64` es `0`). Se usó para convertir `execution_latency_ms` a texto: `content.execution_latency_ms.map(|v| v.to_string()).unwrap_or_default()`.
- `.map(f)`: si es `Some(x)`, aplica la función `f` y devuelve `Some(f(x))`; si es `None`, devuelve `None` sin llamar a `f`. Es la forma de "transformar el valor de adentro, si existe, sin tener que escribir un `match` completo".
- `.clone()` sobre un `Option<T>`: si `T: Clone`, `Option<T>` también es clonable — clona el `Some(x)` clonando `x`, o simplemente copia el `None`.

## Trucos de Senior

- Encadenar `.map(...).unwrap_or_default()` (como arriba) evita escribir un `match` de 5 líneas para una transformación de una sola expresión. Un Junior tiende a escribir:
  ```rust
  let texto = match content.execution_latency_ms {
      Some(v) => v.to_string(),
      None => String::new(),
  };
  ```
  Un Senior escribe la versión encadenada de una línea — mismo resultado, menos ruido visual.
- Si solo necesitas *leer* el contenido de un `Option<String>` como texto (sin tomar ownership), `.as_deref().unwrap_or("")` es más barato que clonar el `String` solo para descartarlo después.
