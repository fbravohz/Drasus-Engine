# Manejo de errores: `Result`, enums de error propios y el operador `?`

## Concepto

Rust no tiene excepciones (`try`/`catch`). Una función que puede fallar devuelve `Result<T, E>` — un enum con dos formas, igual de estricto que `Option`:

```rust
enum Result<T, E> {
    Ok(T),   // la operación funcionó, aquí está el valor
    Err(E),  // la operación falló, aquí está el error
}
```

`T` es el tipo del valor exitoso, `E` es el tipo del error. El compilador, otra vez, te obliga a manejar ambos casos antes de poder usar el valor de adentro.

### Por qué cada módulo define su propio enum de error

En `persistence/telemetry.rs`:

```rust
#[derive(Debug)]
pub enum TelemetryError {
    Database(sqlx::Error),
}
```

¿Por qué no usar `sqlx::Error` directamente? Porque el código que llama a este repositorio no debería tener que saber que por debajo se usa SQLx — ADR-0003 (FCIS) exige que cada módulo tenga su propio tipo de error en la frontera, así el día de mañana se puede cambiar de motor de base de datos sin que el error se filtre por todo el código. El mismo patrón ya existía en `AuditLogError` y `JobRepositoryError`.

### Las tres piezas que hacen que un enum de error sea "un error de verdad"

```rust
impl std::fmt::Display for TelemetryError { /* cómo se imprime como texto */ }
impl std::error::Error for TelemetryError { /* lo marca como "soy un error" para el ecosistema */ }
impl From<sqlx::Error> for TelemetryError { /* conversión automática */ }
```

La tercera (`From`) es la que habilita el truco más importante de Rust para errores: el operador `?`.

### El operador `?`: "si esto falla, devuelve el error ya mismo"

```rust
pub async fn purge_older_than(&self, cutoff_ns: i64) -> Result<u64, TelemetryError> {
    let result = sqlx::query("DELETE FROM telemetry_samples WHERE created_at < ?")
        .bind(cutoff_ns)
        .execute(self.pool)
        .await?;   // <- aquí

    Ok(result.rows_affected())
}
```

`.execute(...).await` devuelve `Result<_, sqlx::Error>`. El `?` al final dice: "si es `Err(e)`, conviértelo a `TelemetryError` (usando el `From` que escribimos arriba) y devuélvelo de inmediato desde esta función — sin seguir ejecutando el resto". Si es `Ok(valor)`, simplemente continúa con `valor` desenvuelto. Sin el `?`, habría que escribir un `match` en cada línea que pueda fallar.

## Trucos de Senior

- Nunca escribas `.unwrap()` en código de producción (sí está bien en tests, donde un panic es la forma correcta de fallar ruidosamente) — `.unwrap()` sobre un `Err`/`None` hace `panic!` y tira abajo el proceso. El operador `?` es la forma idiomática de propagar el error hacia quien sepa qué hacer con él.
- Cuando un enum de error solo tiene una variante (como `TelemetryError::Database`), puede parecer que "no hacía falta un enum" — pero deja la puerta abierta a agregar más variantes después (por ejemplo, una validación de negocio) sin romper la firma pública de la función. Diseñarlo así desde el principio es más barato que migrarlo después.
