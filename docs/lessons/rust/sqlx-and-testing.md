# SQLx (transacciones, binds) y pruebas en Rust

## Concepto

### Transacciones: agrupar varias escrituras en un solo "todo o nada"

```rust
let mut tx = self.pool.begin().await?;

for sample in samples {
    sqlx::query("INSERT INTO telemetry_samples (...) VALUES (...)")
        .bind(&sample.id)
        // ...más .bind(...)...
        .execute(&mut *tx)
        .await?;
}

tx.commit().await?;
```

`pool.begin()` saca una conexión del pool y abre una transacción sobre ella. Cada `INSERT` dentro del bucle se ejecuta sobre `&mut *tx` (esa misma conexión), no sobre el pool — así todos los `INSERT` quedan agrupados. `tx.commit()` los confirma todos juntos. Si algo falla a mitad del bucle (el `?` propaga el error), la transacción nunca llega a `commit()` y SQLite descarta todo lo que se había insertado — no quedan filas a medias.

¿Por qué importa para telemetría? Cada `commit()` implica un `fsync` real al disco (el paso más lento de cualquier escritura). Insertar 50 muestras en 50 transacciones separadas paga ese costo 50 veces; insertarlas en una sola transacción lo paga una sola vez — exactamente el ahorro que necesita un "vaciado por lotes".

### `.bind(...)`: pasar valores a una consulta sin concatenar strings

```rust
sqlx::query("DELETE FROM telemetry_samples WHERE created_at < ?")
    .bind(cutoff_ns)
```

El `?` en el SQL es un placeholder; `.bind(cutoff_ns)` le asigna el valor real. Esto es lo que evita la inyección SQL — nunca se construye el SQL pegando texto del usuario, SQLx separa "la forma de la consulta" de "los datos".

### Pruebas en Rust: `#[test]` vs. `#[tokio::test]`

- `#[test]`: marca una función como una prueba normal, síncrona. `cargo test` la descubre y la corre sola.
- `#[tokio::test]`: lo mismo, pero para una función `async fn` — le envuelve un runtime de Tokio mínimo alrededor, así adentro sí se puede hacer `.await` (`connect(...).await`, etc.).

### Por qué algunas pruebas usan un archivo temporal en vez de `sqlite::memory:`

```rust
let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
let db_path = temp_dir.path().join("telemetry_durability.sqlite");
```

Una base `sqlite::memory:` vive solo mientras esa conexión específica esté abierta — cerrarla y "reabrirla" no recupera nada, porque no hay ningún archivo real detrás. Para demostrar que algo sobrevive a un reinicio del proceso (criterio de durabilidad), hace falta un archivo de verdad en disco: se abre un pool, se escribe, se cierra (`pool.close()`), y se abre un pool NUEVO sobre el MISMO archivo — eso sí es equivalente a reiniciar el proceso. `tempfile::tempdir()` crea un directorio temporal que Rust borra automáticamente cuando termina la prueba (cuando `temp_dir` sale de su ámbito — ver `ownership-borrowing-lifetimes.md`).

### Medir tiempo real con `std::time::Instant`

```rust
let start = std::time::Instant::now();
for _ in 0..ITERATIONS {
    buffer.record_heartbeat("bench.heartbeat");
}
let elapsed = start.elapsed();
```

`Instant::now()` toma una marca de tiempo monótona (no se ve afectada por correcciones del reloj del sistema); `.elapsed()` calcula cuánto pasó desde esa marca. Medir sobre muchas iteraciones (1000, en el benchmark de telemetría) y promediar evita que el ruido de una sola llamada (una pausa del sistema operativo, por ejemplo) arruine la medición.

## Trucos de Senior

- Forzar contención real (como sostener el lock de escritura de SQLite con `BEGIN IMMEDIATE` desde una conexión separada, en el test de no-bloqueo de `orchestrator/telemetry.rs`) demuestra la garantía de verdad, en vez de simularla con un `tokio::time::sleep` que no prueba nada sobre el recurso real que está en juego.
- Una lista vacía como entrada de `insert_batch` se trata como no-op (`if samples.is_empty() { return Ok(()); }`) — evita abrir una transacción que no va a escribir nada, un detalle barato que ahorra una ida y vuelta a la base de datos.
