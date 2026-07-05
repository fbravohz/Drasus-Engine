# Lección Rust — STORY-033: Enriched Domain Events

> Story: [STORY-033 — Enriched Domain Events](../../execution/STORY-033-enriched-domain-events.md)
> Modo: Docente (ADR-0122). Profundidad cero-conocimiento: no se da por sabido nada de Rust ni de bases de datos.

Este archivo explica, desde la base, los conceptos que produjo la construcción del **event-store de eventos de dominio enriquecidos** — el cimiento #6 del substrato de monetización (ADR-0144), raíz del pilar de Cuentas Verificadas (ADR-0145). Cada concepto cita el código real que esta Story creó.

---

## Concepto

### 1. Qué es un "event-store heterogéneo" (event-sourcing con enum + payload JSON)

**El problema.** El motor de trading, cuando exista, va a producir muchas clases de hecho distintas: una orden se ejecutó, entró un depósito de capital, se tomó una foto del estado de una cuenta, terminó un backtest, se detectó un régimen de mercado… Cada clase tiene campos diferentes. La pregunta de diseño es: ¿una tabla de base de datos por clase de evento (`orders_executed`, `capital_flows`, `account_snapshots`…), o una sola tabla para todas?

**La decisión (event-sourcing heterogéneo).** Una sola tabla, `domain_events`. Cada fila lleva dos columnas clave:
- `event_type` — un texto que dice **qué clase** de evento es (`"ORDER_EXECUTED"`, `"CAPITAL_FLOW"`…).
- `payload` — un texto JSON con **el contenido específico** de esa clase.

Así el esquema no cambia cada vez que aparece una clase de evento nueva. La tabla es un libro contable append-only (solo se agrega, nunca se borra ni edita) donde cada renglón es un hecho histórico inmutable. Reconstruir el pasado = leer los renglones en orden. Eso es *event-sourcing*: el estado del sistema es la suma de sus eventos, no una foto mutable.

**Cómo se modela en Rust.** El "qué clase + qué contenido" se representa con un **enum con datos**. En la mayoría de lenguajes un `enum` es solo una lista de nombres (RED, GREEN, BLUE). En Rust cada variante del enum puede **cargar su propio dato**, con una forma distinta por variante. Eso es exactamente un event-store heterogéneo tipado:

```rust
// crates/shared/src/domain/enriched_domain_events.rs
pub enum EnrichedDomainEvent {
    OrderExecuted(OrderExecutedPayload),
    CapitalFlow(CapitalFlowPayload),
    AccountSnapshot(AccountSnapshotPayload),
    BacktestCompleted(BacktestCompletedPayload),
    RegimeDetected(RegimeDetectedPayload),
    DrawdownDetected(DrawdownDetectedPayload),
    LiquidityStress(LiquidityStressPayload),
    CorrelationChange(CorrelationChangePayload),
}
```

`OrderExecutedPayload` tiene `instrument_id`, `side`, `price`, `notional`, `realized_pnl`… mientras que `CapitalFlowPayload` tiene `sign`, `amount`, `currency`. Son structs distintos. El enum los une bajo un mismo tipo para que el resto del código pueda decir "dame cualquier evento de dominio" sin saber de antemano cuál es.

El puente entre el enum (Rust) y la tabla (SQL) son dos métodos:

```rust
// El string que va a la columna event_type (y que el CHECK de la BD valida).
pub fn event_type(&self) -> &'static str {
    match self {
        EnrichedDomainEvent::OrderExecuted(_) => "ORDER_EXECUTED",
        EnrichedDomainEvent::CapitalFlow(_) => "CAPITAL_FLOW",
        // ...
    }
}
```

`match` es la construcción de Rust que obliga a cubrir **todas** las variantes: si mañana se agrega una variante nueva al enum y se olvida un brazo aquí, el compilador **no deja compilar**. Es un seguro contra catálogos desincronizados. El `_` dentro de `OrderExecuted(_)` significa "no me importa el dato que carga, solo la variante".

La columna `event_type` de la migración tiene un `CHECK` que enumera exactamente esos ocho strings:

```sql
-- migrations/0012_domain_events.sql
event_type TEXT NOT NULL CHECK (event_type IN (
    'ORDER_EXECUTED', 'CAPITAL_FLOW', 'ACCOUNT_SNAPSHOT', 'BACKTEST_COMPLETED',
    'REGIME_DETECTED', 'DRAWDOWN_DETECTED', 'LIQUIDITY_STRESS', 'CORRELATION_CHANGE'
)),
```

Doble defensa: el `match` de Rust garantiza que solo salgan esos strings; el `CHECK` de SQLite rechaza cualquier otro que intente entrar por una ruta que no pase por Rust (por eso el test `database_check_rejects_unknown_event_type` inserta SQL crudo con `'UNKNOWN_EVENT'` y verifica que la BD lo rechaza).

### 2. Serialización canónica determinista (por qué `BTreeMap` y no `HashMap`)

**El problema.** El payload es un JSON. Pero el mismo evento lógico debe producir **exactamente el mismo string JSON** cada vez — byte por byte. ¿Por qué tanto rigor? Porque ese string entra en el hash de auditoría (concepto 4). Si el mismo evento serializara `{"amount":100,"sign":"DEPOSIT"}` una vez y `{"sign":"DEPOSIT","amount":100}` otra, el hash cambiaría y la cadena de auditoría se rompería falsamente. También rompería la reconstrucción exacta (replay) que este cimiento existe para habilitar.

**La causa del no-determinismo.** El tipo estándar de "diccionario" en Rust es `HashMap`. Un `HashMap` **no garantiza orden** al recorrer sus claves — de hecho lo aleatoriza a propósito entre ejecuciones del programa (defensa contra ciertos ataques). Serializar un `HashMap` a JSON produciría claves en orden impredecible.

**La solución.** `BTreeMap` — un diccionario que mantiene sus claves **siempre en orden alfabético** (internamente es un árbol balanceado ordenado). Serializar un `BTreeMap` siempre emite las claves en el mismo orden. Este es el mismo patrón que ya usa `domain::consent_registry` con su `optout_map`.

```rust
// crates/shared/src/domain/enriched_domain_events.rs
fn to_canonical_map(&self) -> BTreeMap<String, JsonValue> {
    let mut map = BTreeMap::new();
    // ... put!("amount", p.amount); put!("sign", p.sign.as_str()); ...
    map
}

pub fn canonical_payload_json(&self) -> String {
    let map = self.to_canonical_map();
    serde_json::to_string(&map)
        .expect("BTreeMap<String, JsonValue> de solo strings/enteros siempre serializa")
}
```

El test `canonical_payload_json_keys_are_alphabetically_sorted` fija el string exacto esperado:
`{"account_id":"acc-1","amount":100000000000,"currency":"USD","sign":"DEPOSIT","timestamp_ns":1000}` — nótese el orden `account_id, amount, currency, sign, timestamp_ns`: alfabético, no el orden en que se escribieron en el código.

**Sobre el `.expect(...)`.** `serde_json::to_string` devuelve un `Result` (puede fallar). En Rust, `Result` es un tipo que es `Ok(valor)` o `Err(error)` — obliga a decidir qué hacer con el error. `.expect("mensaje")` dice "estoy seguro de que aquí no falla; si por algún imposible fallara, revienta el programa con este mensaje". La política del proyecto exige justificar cada `expect` en producción: aquí la justificación es que `serde_json` solo falla al serializar un mapa si contiene un `f64` infinito o `NaN` — y este mapa solo tiene strings y enteros `i64`, nunca coma flotante (ver concepto 3). Por eso es imposible que falle.

### 3. Por qué los montos son enteros `i64` ×10⁸ y qué reconstruyen (CERO `f64`)

**La regla (ADR-0141).** Ninguna cantidad monetaria del sistema se guarda como número de coma flotante (`f64`/`REAL`). Todas se guardan como enteros `i64` multiplicados por 100 000 000 (10⁸). Un depósito de $1 000.00 se guarda como `100_000_000_000` (mil por cien millones). Ocho decimales de precisión, fijos para todo el sistema.

**Por qué NO `f64`.** Un `f64` (número de coma flotante de 64 bits) solo representa **enteros exactos hasta 2⁵³** (≈9×10¹⁵). Por encima de eso empieza a perder los últimos dígitos. Peor: valores "redondos" en decimal como 0.1 **no tienen representación exacta** en binario de coma flotante — `0.1 + 0.2` no da exactamente `0.3`. En un cálculo de facturación o de track record, ese error de redondeo es dinero real perdido o cobrado de más. Y como el payload se re-serializa y se re-hashea, un `f64` podría además serializar distinto entre plataformas y romper el hash de auditoría.

**Por qué enteros lo resuelven.** Un `i64` representa todo entero exacto hasta ≈9.2×10¹⁸ sin ninguna pérdida. Trabajar en la escala ×10⁸ convierte "dólares con 8 decimales" en "unidades enteras", y la aritmética entera es exacta y determinista.

En esta Story, **todos** los campos numéricos son `i64`, incluidos los que en otros sistemas serían coma flotante — no solo los montos obvios:

```rust
// crates/shared/src/domain/enriched_domain_events.rs
pub struct CapitalFlowPayload {
    pub amount: i64,        // monto ×10⁸
    // ...
}
pub struct BacktestCompletedPayload {
    pub sharpe: i64,        // Sharpe ratio ×10⁸ (¡también entero!)
    pub drawdown: i64,      // fracción 0..1 ×10⁸
    pub pbo: i64,           // probabilidad 0..1 ×10⁸
    // ...
}
```

Incluso el Sharpe, el drawdown porcentual y la probabilidad PBO — que "naturalmente" serían decimales — van escalados ×10⁸ por la misma razón de determinismo del hash.

**Qué reconstruyen estos campos (el porqué de negocio, ADR-0145).** Los tres eventos reforzados de ADR-0145 existen para hacer un track record verificable:
- **`CapitalFlow`** (depósito/retiro/transferencia con signo y monto) permite calcular el **gain% correcto, que excluye el capital aportado**. Si una cuenta pasa de $1 000 a $2 000, ¿ganó 100%? Solo si no hubo depósitos. Si en medio entraron $500 de depósito, el gain real es menor. Sin registrar los flujos de capital, el gain% es incalculable — el ADR-0145 cita el ejemplo del "441%" que solo es correcto si se descuentan el depósito de 350 y el retiro de 476.98.
- **`AccountSnapshot`** (equity/balance/margen por foto) alimenta las **curvas de equidad y de balance** del track record.
- **`OrderExecuted` reforzada** (con `account_id`, `realized_pnl`, `mae`, `mfe`, `duration_ns`) permite derivar **% de trades rentables, tiempo medio de espera y días de trading por cuenta**. MAE (Maximum Adverse Excursion) y MFE (Maximum Favorable Excursion) miden cuánto se movió el precio en contra y a favor durante el trade — el test `order_executed_reinforced_fields_preserve_exact_integer_amounts` confirma que incluso valores negativos (MAE = `-200_000_000`) se preservan exactos.

Los tests `capital_flow_preserves_exact_integer_amount` y `account_snapshot_preserves_exact_integer_amounts` no solo comparan el número: también hacen `assert!(!json.contains('.'))` — ningún punto decimal en el JSON serializado, prueba de que nunca hubo un `f64` en el camino.

### 4. Hash de auditoría encadenado sobre una tabla append-only

**La idea.** Cada fila guarda un `audit_hash` (una huella SHA-256 de su contenido) que **incluye el `audit_hash` de la fila anterior**. Es una cadena tipo blockchain-lite: si alguien altera una fila histórica, su huella cambia, pero la fila siguiente todavía apunta a la huella vieja — la cadena se rompe y la manipulación queda detectable.

```rust
// crates/shared/src/domain/enriched_domain_events.rs
pub fn compute_event_audit_hash(
    id: &str, created_at_ns: i64, event_sequence_id: i64,
    previous_audit_hash: &str, /* ...campos... */,
    payload_json: &str, replicate: bool,
) -> String {
    const SEP: char = '\u{1F}';  // separador que no aparece en texto normal
    let mut buffer = String::new();
    let mut push = |field: &str| { buffer.push_str(field); buffer.push(SEP); };
    push(id);
    push(&event_sequence_id.to_string());
    push(previous_audit_hash);   // <- el eslabón con la fila anterior
    // ... todos los campos que deben quedar a prueba de manipulación ...
    // SHA-256 del buffer -> hex
}
```

El `\u{1F}` (carácter ASCII "Unit Separator") separa los campos en el buffer que se hashea. ¿Por qué? Para que dos combinaciones distintas de campos no colisionen en el mismo flujo de bytes: sin separador, `"ab" + "c"` y `"a" + "bc"` producirían el mismo texto `"abc"` y el mismo hash. Con separador, `"ab\u1Fc\u1F"` ≠ `"aᾼ\u1F"`. Es el mismo patrón que `audit_log`, `usage_metering` y `consent_registry`.

La primera fila (génesis) no tiene fila anterior: su `audit_chain_hash` es `NULL` en la BD (no un texto centinela `"genesis"` — ADR-0141 lo prohíbe). Para calcular su hash se usa la constante `GENESIS_PREVIOUS_HASH` (`"GENESIS"`) como eslabón previo ficticio. El test `audit_chain_hash_is_null_only_in_genesis_row_and_chains_afterwards` verifica que la fila 1 tiene `None` y la 2 encadena al `audit_hash` de la 1.

El campo `replicate` **entra en el hash** — el test `compute_event_audit_hash_changes_when_replicate_changes` lo prueba: cambiar solo ese flag cambia toda la huella, así que la decisión de replicación queda registrada de forma inmutable junto al evento.

### 5. El append atómico bajo concurrencia (`BEGIN IMMEDIATE`) — el corazón de la Story

**El peligro.** Asignar `event_sequence_id` (la posición en la cadena) es un patrón *read-then-write*: leer el máximo actual (`SELECT MAX(event_sequence_id)`), sumarle 1, e insertar. Si dos escritores hacen esto **a la vez**, ambos leen el mismo máximo (digamos 5), ambos derivan 6, ambos intentan insertar la fila 6. La restricción `UNIQUE` rechaza a uno de los dos — **y su evento se pierde en silencio**. En un event-store, perder un evento es un defecto grave: es un hecho histórico que desaparece.

**Por qué una transacción normal no basta.** SQLite abre transacciones en modo `DEFERRED` por defecto: no toma el lock de escritura hasta el primer `INSERT`/`UPDATE`. Dos transacciones `DEFERRED` pueden ambas leer, y luego ambas intentar subir a escritura — lo que produce un interbloqueo (deadlock) o el rechazo de una.

**La solución (regla "Atomicidad de ledgers append-only", causa raíz DEBT-001).** Envolver la lectura de la cola **y** el INSERT en una sola transacción abierta con `BEGIN IMMEDIATE`, que toma el lock de escritura **de entrada**. Así el segundo escritor espera su turno; cuando entra, ve la fila que dejó el primero y deriva la posición siguiente correcta.

```rust
// crates/shared/src/persistence/enriched_domain_events.rs
async fn try_record_event_once(&self, input: &RecordDomainEventInput)
    -> Result<DomainEventRow, DomainEventRepositoryError>
{
    // Toma el lock de escritura de INMEDIATO (no el BEGIN DEFERRED por defecto).
    let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

    // Lectura DENTRO de la transacción: la cola de la cadena.
    let tail_row = sqlx::query("SELECT audit_hash, event_sequence_id FROM domain_events \
                                ORDER BY event_sequence_id DESC LIMIT 1")
        .fetch_optional(&mut *tx).await?;
    // ... deriva event_sequence_id = previous + 1, computa el hash ...

    // Escritura DENTRO de la MISMA transacción.
    sqlx::query("INSERT INTO domain_events (...) VALUES (...)")
        // ... binds ...
        .execute(&mut *tx).await?;

    tx.commit().await?;  // recién aquí se libera el lock y la fila se hace visible
    Ok(row)
}
```

**Cinturón y tirantes.** El `UNIQUE` sobre `event_sequence_id` sigue estando en la migración, pero ya no es el guardián primario: es la red de seguridad por si algo se escapara. El guardián real es el `BEGIN IMMEDIATE`.

**Reintento acotado (nunca perder el evento).** Si aun así hay contención transitoria (`SQLITE_BUSY` por lock ocupado, o una colisión de secuencia), no se tira el evento: se reintenta hasta 5 veces re-derivando la posición. Solo si se agotan los 5 intentos se devuelve un error **tipado** `WriteContention { attempts }` para que el llamador decida — nunca una pérdida silenciosa.

```rust
pub async fn record_event(&self, input: RecordDomainEventInput) -> Result<DomainEventRow, DomainEventRepositoryError> {
    let mut attempt = 0;
    loop {
        attempt += 1;
        match self.try_record_event_once(&input).await {
            Ok(row) => return Ok(row),
            Err(error) => {
                if is_transient_write_conflict(&error) {
                    if attempt < MAX_RECORD_ATTEMPTS { continue; }
                    return Err(DomainEventRepositoryError::WriteContention { attempts: attempt });
                }
                return Err(error);  // error no transitorio: propagar de inmediato
            }
        }
    }
}
```

`is_transient_write_conflict` distingue lo reintentable (lock ocupado, colisión de `event_sequence_id`) de lo que no lo es (input inválido, otro error) — copiado exacto de `consent_registry.rs`.

**La prueba que puede caerse (discriminante).** El test `concurrent_record_events_persist_every_event_without_gaps_or_lost_rows` lanza **16 escritores en paralelo** con `#[tokio::test(flavor = "multi_thread", worker_threads = 4)]` (un runtime de verdad con varios hilos del sistema operativo) contra una **base en archivo temporal** (no `:memory:`, porque en memoria cada conexión sería una base distinta y no habría concurrencia real). Luego afirma tres cosas: (a) las 16 filas se persistieron, (b) los `event_sequence_id` son exactamente `1..=16` sin huecos ni repetidos, (c) la cadena de hashes es íntegra y **recomputable** (recalcula cada `audit_hash` y lo compara). Si se quitara el `BEGIN IMMEDIATE`, dos escritores colisionarían, una fila se perdería y la aserción (a) o (b) fallaría. Una prueba que solo puede pasar cuando el comportamiento existe: eso es una prueba discriminante.

### 6. Supresión por tier: separar "emitir local" de "replicar al proveedor"

**El modelo (ADR-0143).** El evento **siempre se persiste localmente** — es del usuario. Lo que el tier de pago gobierna es si además se **replica hacia la Cabina de Mando** del proveedor:
- Tier **gratuito** (Explorer): el trabajo fluye al proveedor → `replicate = true`.
- Tier **de pago al corriente** (Sovereign): privacidad real, la telemetría de trabajo se suprime en origen → `replicate = false`.

Esa decisión no se recalcula aquí: viene ya masticada en el `ExecutionGate` **real** de `licensing-system` (#2), en su campo `suppress_work_telemetry`. La función pura del Core solo lo invierte:

```rust
// crates/shared/src/domain/enriched_domain_events.rs
pub fn decide_replication(gate: &ExecutionGate) -> bool {
    !gate.suppress_work_telemetry
}
```

**Referencia compartida (`&ExecutionGate`).** El `&` significa que la función **toma prestado** el gate sin adueñarse de él — lo lee, no lo consume. Es el concepto de *borrowing* de Rust: quien llama sigue siendo dueño del gate y puede usarlo después. Se toma por referencia porque solo se necesita leer un campo booleano; copiar el struct entero sería desperdicio.

**La composición completa** (Shell/orchestrator) recibe el evento + el gate real, deriva `replicate`, y delega la persistencia atómica:

```rust
// crates/shared/src/orchestrator/enriched_domain_events.rs
pub async fn record_domain_event(
    pool: &SqlitePool, clock: &dyn Clock,
    identity: EventEmissionIdentity, gate: &ExecutionGate, event: EnrichedDomainEvent,
) -> Result<DomainEventRow, DomainEventRepositoryError> {
    let replicate = decide_replication(gate);   // gate_in -> flag
    let repo = DomainEventRepository::new(pool, clock);
    repo.record_event(RecordDomainEventInput { /* ...identidad..., */ event, replicate }).await
}
```

**Lo que NO se hace (diferido).** No hay envío por red. El `replicate` es solo un flag que se persiste junto al evento; el adaptador que efectivamente empuja los eventos marcados a la Cabina de Mando, y el fan-out al bus de eventos (ADR-0085), son trabajo futuro registrado en la Orden (§8). Este es el patrón "puerto + esquema ahora, adaptador después" de ADR-0144: se construye el contrato completo hoy para no reabrir la capa de ejecución mañana.

Los tests `suppressing_gate_persists_event_with_replicate_false` y `non_suppressing_gate_persists_event_with_replicate_true` ejercitan la composición completa (orchestrator → Core → repositorio → BD), no solo la función pura. Y el CLI lo confirma end-to-end: `verify enriched-domain-events` con `tier:"FREE"` reporta `replicate:true`; con `tier:"PAID"`, `replicate:false`.

### 7. Reloj inyectado (determinismo, sin `SystemTime::now()`)

El Core nunca lee el reloj del sistema. El tiempo se **inyecta** por un puerto `Clock`:

```rust
// crates/shared/src/persistence/enriched_domain_events.rs
pub struct DomainEventRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,   // puerto inyectado, no SystemTime::now()
}
// ...
let now_ns = self.clock.timestamp_ns();   // en tests: DeterministicClock
```

`&'a dyn Clock` merece desglose: `dyn Clock` es *dynamic dispatch* — acepta **cualquier** tipo que implemente el trait `Clock` (el `SystemClock` real en producción, o un `DeterministicClock` que avanza solo cuando se le dice, en tests). El `'a` es un *lifetime*: una anotación que le promete al compilador que el reloj prestado vive al menos tanto como el repositorio que lo usa, para que Rust garantice en compilación que no queda una referencia colgando a algo ya liberado. Gracias a esto, los tests corren con un reloj falso y determinista: mismas entradas → mismas salidas, bit por bit (ADR-0002/0004).

---

## Trucos de Senior

- **`match` exhaustivo como catálogo autovalidado.** Al mapear el enum a `event_type()` con un `match` sin brazo `_ =>` comodín, el compilador obliga a actualizar el mapeo cada vez que se agrega una variante. El catálogo de la BD y el del código no pueden desincronizarse sin un error de compilación. Es gratis y más fuerte que cualquier test.

- **Macro local `macro_rules!` para quitar ruido sin ocultar lógica.** En `to_canonical_map` se define una macro minúscula `put!("clave", valor)` en vez de repetir `map.insert("clave".to_string(), serde_json::json!(valor))` trece veces. La macro es local a la función (no contamina el módulo) y hace el código legible sin esconder qué se inserta.

- **`i64::from(bool)` para persistir un booleano como 0/1.** SQLite no tiene tipo booleano nativo; se guarda como entero. `i64::from(input.replicate)` convierte `true`→1, `false`→0 de forma explícita y sin `if`. Al leer de vuelta, `replicate_int != 0` reconstruye el `bool`.

- **`BEGIN IMMEDIATE` se copia verbatim, no se reinventa.** El patrón atómico (transacción de escritura de entrada + reintento acotado + `WriteContention` tipado + `is_transient_write_conflict`) es idéntico al de `consent_registry.rs` y `usage_metering.rs`. Copiarlo exacto — no "mejorarlo" — es lo correcto: es un patrón ya probado bajo la prueba de concurrencia, y divergir introduciría el riesgo de reintroducir DEBT-001.

- **Prueba de concurrencia sobre archivo temporal, nunca `:memory:`.** Una base SQLite `:memory:` es privada por conexión: 16 tareas concurrentes tendrían 16 bases distintas y la prueba pasaría trivialmente sin probar nada. El `tempfile::tempdir()` + ruta a archivo real es lo que hace que las 16 tareas compitan por el mismo lock — la única forma de que la prueba pueda **caerse** si el `BEGIN IMMEDIATE` se quita.

- **`assert!(!json.contains('.'))` como prueba de "cero f64".** Una forma barata y directa de demostrar que ningún monto se coló como coma flotante: si hubiera un `f64`, su serialización JSON llevaría un punto decimal. Ausencia de punto = todo entero.
