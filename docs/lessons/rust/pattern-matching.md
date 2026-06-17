# `match`, `if let` y desestructurar tuplas

## Concepto

`match` es el "switch" de Rust, pero más estricto: el compilador exige que cubras **todos** los casos posibles del valor, o que pongas un caso `_` que capture "cualquier otra cosa". Si te olvidas un caso, no compila — esta es la razón por la que `Option<T>` es seguro (ver `option-type.md`): no puedes leer un `Option` sin que el compilador te recuerde que existe el caso `None`.

En `domain/telemetry.rs`, `build_sample` usa `match` sobre un `Option<&TelemetrySample>` para decidir cómo encadenar la nueva muestra:

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

Dos cosas pasan aquí a la vez:
1. **Desestructuración de tupla:** el lado izquierdo `(a, b, c) = match { ... }` asigna los tres valores que devuelve cada rama del `match` a tres variables nuevas en un solo paso. Una tupla es simplemente "varios valores agrupados sin nombre de campo" — se accede por posición, no por nombre como en un struct.
2. **Binding del valor interior:** en la rama `Some(previous_sample) => ...`, `previous_sample` es una variable nueva que apunta al valor que estaba *adentro* del `Some` — no tienes que "desenvolverlo" a mano.

### `if let`: un `match` de un solo caso

Cuando solo te interesa UN caso de un `Option`/enum y quieres ignorar el resto, escribir un `match` completo es ruido de más. `if let` es el atajo:

```rust
if let Some(token) = tokens.get(job_id) {
    token.cancel();
}
```

Ese ejemplo viene de `job_executor.rs` — equivalente a un `match` con una rama útil y un `_ => {}` que no hace nada.

## Trucos de Senior

- Si necesitas el caso contrario de `if let` (actuar cuando NO matchea), `let ... else` es más legible que anidar un `match` con una rama vacía:
  ```rust
  let Some(value) = optional else {
      return; // o lo que corresponda hacer si no había valor
  };
  // a partir de aquí, `value` ya está desenvuelto
  ```
- `matches!(valor, Patron)` (usado en `domain/job.rs` para `validate_transition`) es un atajo para "esto encaja con tal patrón" cuando solo te interesa un `bool`, sin necesitar extraer ningún valor de adentro:
  ```rust
  let allowed = matches!(
      (from, to),
      (JobState::Queued, JobState::Running) | (JobState::Running, JobState::Completed)
  );
  ```
  El `|` dentro del patrón significa "o este patrón, o este otro".
