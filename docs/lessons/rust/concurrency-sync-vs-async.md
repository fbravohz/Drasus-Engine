# `async`/`await`, `std::sync::Mutex` vs. `tokio::sync::Mutex`, y canales `mpsc`

## Concepto

### `async fn` y `.await`: pausar sin bloquear el hilo

Una función normal, cuando hace algo lento (leer disco, esperar la red), bloquea el hilo entero hasta que termina — ese hilo no puede hacer nada más mientras tanto. Una función `async fn` puede **pausarse** en un punto de espera (`.await`) y devolverle el control al runtime de Tokio, que mientras tanto usa ese mismo hilo para avanzar OTRA tarea. Cuando lo que se esperaba está listo, la tarea pausada se reanuda donde quedó.

```rust
pub async fn purge_older_than(&self, cutoff_ns: i64) -> Result<u64, TelemetryError> {
    let result = sqlx::query(/* ... */).execute(self.pool).await?;
    Ok(result.rows_affected())
}
```

### Por qué `record_latency`/`record_heartbeat` NO son `async fn`

Esta es la decisión de diseño más importante de `orchestrator/telemetry.rs`. El requisito de la Feature es que registrar una muestra tarde menos de 50 microsegundos — un tiempo donde ni vale la pena pausar y reanudar una tarea async (ese mecanismo tiene su propio costo). Como encolar en un canal `mpsc` y tomar un `Mutex` síncrono NUNCA esperan a nada (ni disco, ni red, ni otra tarea), la función puede ser una función normal, sin `async`, sin `.await` — y por lo tanto sin ese costo. El vaciado a disco (lo lento) vive en una tarea de fondo separada (`spawn_flush_task`) que el que llama nunca espera.

### `std::sync::Mutex` vs. `tokio::sync::Mutex`: ¿cuál usar y cuándo?

Ambos sirven para lo mismo — proteger un dato para que solo una tarea lo toque a la vez — pero tienen una diferencia importante:

- **`std::sync::Mutex`** (usado en `chain_state` de `TelemetryBuffer`): es **bloqueante**. Si está ocupado, el hilo que lo pide se queda esperando *sin devolverle el control a Tokio*. Por eso solo es seguro usarlo dentro de código async cuando la sección protegida es **muy breve y nunca hace `.await`** mientras lo tiene tomado — exactamente el caso de `chain_state`: se toma, se lee/escribe una variable, se suelta, todo sin esperar a nada.
- **`tokio::sync::Mutex`** (usado en `queue_rx` de `TelemetryBuffer` y en `cancel_tokens` de `JobExecutor`): es **async-aware**. Si está ocupado, la tarea que lo pide se pausa (como cualquier `.await`) y le devuelve el control a Tokio, en vez de congelar el hilo entero. Hace falta cuando la sección protegida sí puede hacer `.await` mientras tiene el lock tomado.

Regla práctica: si nunca vas a hacer `.await` mientras tienes el lock, usa `std::sync::Mutex` (más barato). Si necesitas hacer `.await` con el lock tomado, usa `tokio::sync::Mutex` — usar el síncrono ahí sí sería un error real (podría congelar el runtime entero).

### Canales `mpsc`: pasar datos entre tareas sin que ninguna espere a la otra

`mpsc` = "multi-producer, single-consumer" — varios productores pueden enviar datos al mismo canal, un solo consumidor los recibe en orden.

```rust
let (queue_tx, queue_rx) = mpsc::unbounded_channel();
```

La versión **`unbounded`** (sin límite de tamaño) es la pieza clave de "no bloqueante": `queue_tx.send(valor)` nunca espera a que el receptor esté listo — simplemente agrega el valor a una cola en memoria y devuelve el control de inmediato. (Un canal *acotado*, `bounded`, sí pausaría al emisor si la cola se llenó — por eso aquí se eligió el no acotado a propósito.) Del lado receptor, `receiver.try_recv()` intenta tomar un valor sin esperar: devuelve `Ok(valor)` si había algo, o un error `Empty`/`Disconnected` si no.

## Trucos de Senior

- Nunca mezcles un `std::sync::Mutex` con un `.await` adentro de la sección crítica — es un error sutil que a veces ni el compilador detecta, y puede congelar el runtime de Tokio entero, no solo esa tarea. Si dudas, usa `tokio::sync::Mutex`; solo bájate al síncrono cuando estés seguro de que la sección nunca espera a nada (como aquí).
- `mpsc::error::TryRecvError` tiene dos variantes (`Empty` y `Disconnected`) — tratarlas igual (como "no hay nada que hacer") es un bug sutil: `Disconnected` significa que YA NO QUEDA NINGÚN EMISOR VIVO, así que seguir reintentando para siempre es un bucle inútil. `spawn_flush_task` distingue ambos casos a propósito para poder salir del bucle cuando corresponde.
