# Lección STORY-051: Hybrid Data Transformer

> Story: [STORY-051 — Transformador de datos de mercado (motor Polars)](../../execution/STORY-051-hybrid-data-transformer.md)
>
> Conceptos cubiertos: por qué existe la escala entera ×10⁸ y qué es "aritmética exacta"; acumuladores `i128` y `checked_*` contra el desbordamiento silencioso; redondeo half-to-even (bankers' rounding) y por qué NO es lo mismo que "redondear normal"; `div_euclid` vs. el operador `/` de Rust; `BTreeMap` como agrupador ordenado determinista; Polars `LazyFrame`/`DataFrame`/evaluación perezosa; el puente Zero-Copy Polars → `ndarray` vía Arrow; el mismo patrón de ledger atómico (`BEGIN IMMEDIATE` + reintento) ya visto en `sovereign-data-fetcher`, ahora replicado; pruebas discriminantes (rojo→verde) contra datos reales congelados.

---

## Concepto

### 1. Por qué el dinero no vive en `f64` — la escala entera ×10⁸

**El problema que resuelve:** las computadoras representan números decimales (`f64`) en base 2, no en base 10. `0.1 + 0.2` en casi cualquier lenguaje da `0.30000000000000004`, no `0.3`, porque `0.1` no tiene una representación exacta en binario (es periódica, como `1/3` en decimal). Para mostrar un precio en pantalla eso no importa. Para sumar millones de operaciones de trading y que el centavo cuadre, es un desastre: el error se acumula.

**La solución del proyecto (ADR-0141):** nunca se guarda "42283.58 dólares" como un número con punto decimal. Se guarda el entero `4228358000000` y una convención fija: "divide entre 10⁸ para volver a ver el decimal". Un entero de 64 bits representa exactamente cualquier valor hasta ~9.2 × 10¹⁸ — sin aproximación, sin error acumulado.

**Cómo se ve en el código real** (`crates/features/data/hybrid-data-transformer/src/domain.rs`):

```rust
pub struct RawTick {
    pub event_timestamp_ns: i64,
    pub price_scaled: i64,   // 42283.58 se guarda como 4228358000000
    pub volume_scaled: i64,
    pub origin_seq: u64,
}
```

Ningún campo es `f64`. La fixture congelada de Binance (`tests/fixtures/binance_btcusdt_sample.csv`) convierte el precio `"42283.58000000"` (texto, tal como lo entrega la API real) a `4228358000000` una sola vez, a mano, al congelar el archivo — exactamente el trabajo que en producción hace la Cáscara antes de que un tick le llegue a este transformador.

---

### 2. El acumulador `i128` — por qué `i64` no alcanza para el VWAP

**El problema:** el VWAP (precio promedio ponderado por volumen) se calcula como `sum(precio_i × volumen_i) / sum(volumen_i)`. Un precio típico escalado es del orden de 10¹³ (`42283.58 × 10⁸`). Un volumen agregado de un par muy líquido puede ser del orden de 10¹⁵. Multiplicarlos da ~10²⁸ — muy por encima de lo que un entero de 64 bits puede representar (`i64::MAX` ≈ 9.2 × 10¹⁸). Si se multiplica en `i64` sin darse cuenta, Rust en modo `release` **envuelve** el resultado (overflow silencioso: el número da la vuelta y sale un valor absurdo pero sintácticamente válido) — el peor tipo de bug, porque no truena, solo miente.

**La solución:** acumular en `i128` (128 bits, hasta ~1.7 × 10³⁸), que sí tiene margen de sobra:

```rust
// crates/features/data/hybrid-data-transformer/src/domain.rs
pub fn checked_scaled_product(a: i64, b: i64) -> Option<i128> {
    (a as i128).checked_mul(b as i128)
}
```

`checked_mul` es un método que TODOS los enteros de Rust tienen: en vez de envolver silenciosamente en overflow, devuelve `None`. Esto convierte un bug invisible en un `Option` que el código está OBLIGADO a manejar explícitamente (Rust no compila si ignoras un `Option` sin querer). La prueba `scaled_product_overflows_i64_but_i128_holds_exact_value` (en `domain.rs`) demuestra las dos caras: primero confirma que el MISMO producto SÍ desborda en `i64` (`checked_mul` devuelve `None`), y luego confirma que en `i128` el mismo producto es exacto — la prueba se caería si alguien "simplificara" el acumulador de vuelta a `i64`.

---

### 3. Redondeo half-to-even (bankers' rounding) — la regla FIJA de la feature

**El problema:** al dividir `numerator / denominator` para recuperar la escala ×10⁸, casi siempre sobra un resto. ¿Hacia dónde se redondea? La intuición de "redondear siempre hacia arriba en el .5" (como enseñan en la primaria) tiene un defecto sutil: sobre millones de barras, ese sesgo se ACUMULA sistemáticamente hacia arriba. El redondeo half-to-even (también llamado "banker's rounding") resuelve esto: en un empate exacto, redondea hacia el número PAR más cercano — la mitad de las veces sube, la mitad baja, así que el sesgo se cancela en el agregado.

**Cómo se ve en el código:**

```rust
// crates/features/data/hybrid-data-transformer/src/domain.rs
pub fn round_half_to_even_div_i128(numerator: i128, denominator: i128) -> Option<i64> {
    // ...
    let quotient = num_abs / den_abs;
    let remainder = num_abs % den_abs;
    let twice_remainder = remainder.checked_mul(2)?;

    let round_up = twice_remainder > den_abs || (twice_remainder == den_abs && quotient % 2 != 0);
    // ...
}
```

La clave está en `twice_remainder == den_abs && quotient % 2 != 0`: eso detecta un empate EXACTO (el resto es justo la mitad del divisor) y solo entonces mira si el cociente es impar. Si es impar, sube (para llegar al par); si ya es par, se queda.

**Las tres pruebas de empate exacto** que exige el criterio 3 de la Orden (`domain.rs`):

```rust
assert_eq!(round_half_to_even_div_i128(5, 2), Some(2));  // 2.5 -> 2 (el par más cercano)
assert_eq!(round_half_to_even_div_i128(7, 2), Some(4));  // 3.5 -> 4 (el par más cercano)
assert_eq!(round_half_to_even_div_i128(-5, 2), Some(-2)); // -2.5 -> -2 (respeta el signo)
```

Fíjate que `5/2` da 2 (hacia abajo) y `7/2` da 4 (hacia arriba) — ambos son casos de ".5 exacto", pero el resultado depende de si el cociente entero (2 y 3 respectivamente) es par o impar. Eso es precisamente lo que NO es "redondeo normal".

---

### 4. `div_euclid` — por qué la barra no se calcula con el operador `/`

**El problema:** el operador `/` de Rust, para enteros, **trunca hacia cero**. `-5 / 3` en Rust da `-1` (no `-2`). Para el "inicio de la barra que contiene este instante" (`floor(t / interval) * interval`) eso importaría si algún día hay timestamps negativos (eventos antes de 1970) — con `/` normal, un instante `t = -1` con `interval = 100` calcularía la barra `0` (incorrecto: `-1` debería caer en la barra `[-100, 0)`, no en `[0, 100)`).

**La solución:**

```rust
// crates/features/data/hybrid-data-transformer/src/domain.rs
pub fn floor_bar_timestamp(event_timestamp_ns: i64, interval_ns: i64) -> Result<i64, TransformError> {
    // ...
    Ok(event_timestamp_ns.div_euclid(interval_ns) * interval_ns)
}
```

`div_euclid` es la "división euclidiana": siempre redondea hacia el infinito negativo (el floor matemático real), sin importar el signo. En datos de mercado reales los timestamps son siempre positivos, así que hoy `/` y `div_euclid` darían el mismo resultado — pero usar `div_euclid` documenta la intención exacta ("quiero floor, no truncamiento") y blinda el código contra un caso borde que hoy no ocurre pero que un cambio futuro (backtesting con datos sintéticos anteriores a 1970, por ejemplo) podría introducir sin que nadie lo note.

**La prueba anti look-ahead** (`event_at_exact_boundary_belongs_to_next_bar_not_current`, criterio 6) demuestra el intervalo semiabierto `[inicio, inicio+interval)`: un tick exactamente en `t=100` con `interval=100` cae en la barra `[100, 200)`, NUNCA en `[0, 100)` — si cayera en la barra que está cerrando, esa barra "sabría" del futuro en el instante exacto de su propio cierre.

---

### 5. `BTreeMap` como agrupador determinista — la alternativa a un `HashMap`

**El problema:** para agregar ticks en barras hace falta agruparlos por su timestamp de inicio. La opción obvia en muchos lenguajes es un "diccionario"/"mapa hash". En Rust eso es `HashMap`. Pero un `HashMap` NO garantiza ningún orden al iterarlo — dos ejecuciones del mismo programa pueden recorrer sus claves en orden distinto (por diseño, para prevenir ciertos ataques y permitir optimizaciones internas).

**La solución:**

```rust
// crates/features/data/hybrid-data-transformer/src/domain.rs
let mut groups: BTreeMap<i64, Vec<RawTick>> = BTreeMap::new();
for tick in ticks {
    let bar_start = floor_bar_timestamp(tick.event_timestamp_ns, interval_ns)?;
    groups.entry(bar_start).or_default().push(*tick);
}

let mut bars = Vec::with_capacity(groups.len());
for (bar_timestamp_ns, group_ticks) in groups {
    // ...
}
```

`BTreeMap` es un mapa ordenado (internamente, un árbol balanceado por clave). Iterar sobre un `BTreeMap<i64, _>` SIEMPRE devuelve las claves en orden ascendente — así, las barras de salida ya vienen ordenadas cronológicamente sin necesitar un `.sort()` extra al final, y el resultado es 100% reproducible entre corridas (criterio 4 y 5: determinismo y auditoría reproducible dependen exactamente de esto).

---

### 6. Polars: `DataFrame`, `LazyFrame` y evaluación perezosa

**El problema que resuelve Polars:** procesar millones de filas fila-por-fila en Rust puro (un `for` sobre un `Vec`) es lento porque el procesador no puede aprovechar SIMD (instrucciones que operan sobre varios números a la vez) ni varios núcleos automáticamente. Polars almacena los datos por COLUMNA (no por fila) en memoria contigua — eso es lo que permite operaciones vectorizadas rapidísimas (ordenar una columna de un millón de enteros es mucho más rápido que ordenar un millón de structs dispersos en memoria).

**Cómo se ve en el código** (`crates/features/data/hybrid-data-transformer/src/orchestrator.rs`):

```rust
let df = df![
    "event_timestamp_ns" => event_timestamp_ns,
    "price_scaled" => price_scaled,
    "volume_scaled" => volume_scaled,
    "origin_seq" => origin_seq,
]
.map_err(|e| TransformError::Polars(e.to_string()))?;

let sorted = df
    .lazy()
    .sort(["event_timestamp_ns", "origin_seq"], SortMultipleOptions::default())
    .collect()
    .map_err(|e| TransformError::Polars(e.to_string()))?;
```

`df![...]` es un macro que construye un `DataFrame` (una tabla en memoria, columna por columna) a partir de varios `Vec`. `.lazy()` lo convierte en un `LazyFrame`: en vez de ejecutar el `.sort(...)` inmediatamente, Polars construye un PLAN de lo que hay que hacer. Solo cuando llamas a `.collect()` Polars mira el plan completo, lo optimiza (por ejemplo, si hubiera un filtro después del sort, podría reordenar las operaciones para filtrar primero y ordenar menos datos) y lo ejecuta de una vez, en paralelo sobre varios núcleos.

**Por qué la agregación OHLCV NO se hizo con Polars puro:** Polars es excelente para ordenar/filtrar/columnas, pero no tiene un acumulador `i128` nativo ni redondeo half-to-even — el §2 y §3 de esta lección son matemática que Polars no puede expresar de forma segura. Por eso la frontera exacta de esta feature es: **Polars ordena y columnariza (la parte "80" del 80/20 de la feature), Rust puro agrega con aritmética exacta (la parte "20" delicada)**. Es la misma idea de FCIS de siempre, aplicada a un motor de DataFrame en vez de a HTTP: la Cáscara (`orchestrator.rs`) columnariza; el Núcleo (`domain.rs`) nunca importa `polars`.

---

### 7. El puente Zero-Copy Polars → `ndarray` (TTR-002)

**El problema:** una vez que tienes las barras OHLCV, algunos consumidores (regresiones, álgebra lineal estadística) necesitan una matriz densa en memoria contigua — el formato que la librería `ndarray` (y todo el ecosistema de álgebra lineal en Rust) espera. Convertir "a mano" (iterar fila por fila y copiar cada valor a un nuevo buffer) funciona, pero duplica la memoria y es lento para datasets grandes.

**La solución — el método `to_ndarray` de Polars:**

```rust
// crates/features/data/hybrid-data-transformer/src/orchestrator.rs
let matrix_df = df![
    "open" => open, "high" => high, "low" => low, "close" => close, "volume" => volume,
].ok()?;

matrix_df.to_ndarray::<Float64Type>(IndexOrder::C).ok()
```

Polars almacena sus columnas en formato Apache Arrow (un estándar de memoria columnar compartido por muchas herramientas de datos). Cuando todas las columnas ya son del mismo tipo (`f64` en este caso), `to_ndarray` puede copiar los buffers de Arrow DIRECTAMENTE al layout de `ndarray::Array2`, sin pasar por una serialización intermedia genérica fila-por-fila — de ahí "Zero-Copy vía Arrow" (ADR-0105/restricción de la feature).

**Por qué la conversión a `f64` ocurre AQUÍ y no antes:** ADR-0141 prohíbe `f64` para precio/volumen en cualquier persistencia o en el Núcleo. Este puente es la ÚNICA frontera donde `f64` existe, y solo en memoria, solo para el consumidor estadístico — nunca se guarda en disco. Antes de construir la matriz, `try_build_matrix` verifica CADA valor con `verify_exact_f64_representable` (criterio 7): un entero por encima de 2⁵³ (9 007 199 254 740 992) no cabe exacto en un `f64` de doble precisión (que solo tiene 52 bits de mantisa + 1 implícito = 53 bits de precisión entera). Si algún valor lo excede, la función devuelve `None` en vez de construir una matriz mentirosa.

---

### 8. El mismo patrón de ledger atómico, ahora replicado

**El problema (ya resuelto una vez en `sovereign-data-fetcher`, replicado aquí sin reinventarlo):** cuando dos escrituras concurrentes intentan insertar en una tabla append-only, ambas necesitan LEER "¿cuál es el último `event_sequence_id`?" y luego ESCRIBIR "el mío es ese +1". Si esas dos operaciones no son atómicas, dos escritores pueden leer el mismo número y un `INSERT` fallará por violar el `UNIQUE` — perdiendo ese registro si nadie reintenta.

**La solución, calcada del patrón ya existente** (`crates/features/data/hybrid-data-transformer/src/persistence.rs`):

```rust
async fn try_record_once(&self, new: &NewTransformationRecord) -> Result<TransformationRecord, TransformationRepositoryError> {
    let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
    // SELECT del último event_sequence_id... y el INSERT...
    // ambos DENTRO de la misma transacción.
    tx.commit().await?;
    // ...
}
```

`BEGIN IMMEDIATE` toma el lock de escritura de SQLite desde el primer instante de la transacción (no espera a que alguien intente escribir para "subir de nivel" un lock de lectura — eso evitaría un interbloqueo clásico entre dos transacciones que compiten por subir al mismo tiempo). El método público `record()` reintenta hasta 5 veces si la contención es transitoria (`is_transient_write_conflict`, que reconoce "database is locked" o una colisión específica de `event_sequence_id`, pero NUNCA reintenta una violación de PK genuina).

**Diferencia de semántica con `sovereign-data-fetcher` (vale la pena notarla):** allá, `audit_hash` se calculaba DENTRO del repositorio, sobre los campos de la fila. Aquí, `audit_hash` llega YA CALCULADO desde el Núcleo (`compute_bars_audit_hash`, sobre el contenido de las barras ya ordenadas) — el repositorio solo lo persiste y lo encadena. La razón: el criterio 5 exige que el hash sea reproducible entre corridas del MISMO dataset sin importar el número de hilos de Polars, y esa garantía solo la puede dar el Núcleo puro, no la Cáscara de persistencia.

---

## Trucos de Senior

- **`include_str!("fixtures/binance_btcusdt_sample.csv")`** — embebe el CONTENIDO de un archivo como un `&'static str` en tiempo de COMPILACIÓN (no en tiempo de ejecución). Así, el binario de test no necesita "encontrar" el archivo en disco en runtime (que fallaría si el test corre desde otro directorio de trabajo) — el contenido ya está dentro del ejecutable.

- **`unsigned_abs()` en vez de `.abs()` para evitar un panic de borde:** `i64::MIN.abs()` hace panic en Rust (`-i64::MIN` no cabe en `i64`, porque el rango de un entero con signo es asimétrico: hay un negativo más que positivos). `unsigned_abs()` devuelve un `u64` sin ese problema — se usa en `verify_exact_f64_representable` y en `round_half_to_even_div_i128` precisamente para no introducir un panic en un caso borde de magnitud extrema.

- **`cargo add <crate> --dry-run --features a,b`** — antes de escribir un `Cargo.toml` a ciegas, este comando le pregunta a `crates.io` qué features existen y cuáles activaría tu combinación, sin modificar nada. Así se confirmó que Polars 0.54 llama a su feature de interoperabilidad `ndarray` (no `to-ndarray` ni otro nombre parecido) y que el feature de lectura Parquet se llama `parquet` (no `polars-parquet`, que es el nombre del CRATE interno, no el del FEATURE flag).

- **Cuando el compilador dice "no encontré esta función", a veces el problema es un `feature` de Cargo, no un typo.** El primer intento de compilar esta Story falló con `no associated function... scan_parquet`. La causa real no era un nombre incorrecto: era que el módulo entero que CONTIENE `scan_parquet` está detrás de `#[cfg(feature = "parquet")]` en el código fuente de Polars, y ese feature no estaba activado en el `Cargo.toml` de este crate. `grep -rn "pub fn scan_parquet"` sobre el código fuente descargado en `~/.cargo/registry/src/` (Cargo SIEMPRE descarga el código fuente completo, no solo los binarios) fue lo que reveló el `#[cfg(...)]` que faltaba.

- **Dos versiones del mismo crate pueden convivir en el árbol de dependencias, y el compilador te avisa con un mensaje muy específico:** `error[E0308]: mismatched types ... note: there are multiple different versions of crate 'ndarray'`. Pasó aquí porque este Cargo.toml pedía `ndarray = "0.16"` mientras Polars, internamente, ya traía `ndarray 0.17` para su propio feature `ndarray`. Cargo no fusiona versiones incompatibles (0.16 y 0.17 cuentan como "distintas" a efectos de tipos, aunque semver diga que 0.17 es compatible hacia adelante) — hay que fijar la MISMA versión mayor que la dependencia transitiva para que ambos "hablen del mismo tipo".

- **`#[tokio::test]` usa un runtime de UN SOLO hilo por defecto — algunas librerías (como Polars, que usa `spawn_blocking` internamente para ciertas operaciones de E/S) exigen un runtime multi-hilo.** El síntoma es un panic en runtime (no un error de compilación) que dice literalmente `can call blocking only when running on the multi-threaded runtime`. La solución es `#[tokio::test(flavor = "multi_thread")]`. Vale la pena recordar el síntoma exacto porque no es intuitivo relacionarlo con "mi test usa Polars".

---

## Retrofit ADR-0155 (2026-07-18): esquema canónico real

> Se retoma esta lección tras el congelamiento de la Story: el motor entregado agregaba asumiendo un solo precio y un solo "volumen"; ADR-0155 canonizó después el esquema real de mercado (cotización/ejecución como tipos físicos distintos, volumen en cuatro magnitudes, campos de Futuros/Opciones esquema-listo). Lo que sigue es lo nuevo que enseña este retrofit, además de lo de arriba (que sigue vigente sin cambios: escala entera, `i128`, half-to-even, `div_euclid`, `BTreeMap`, Polars Lazy, puente `ndarray`, ledger atómico).

### 9. Separación de tipos como garantía de compilador — por qué `RawTrade` no tiene `bid`/`ask`

**El problema:** ADR-0155 exige que "una cotización nunca contribuya a una barra agregada". Hay dos formas de garantizar esto: (a) un chequeo en tiempo de ejecución (`if es_cotización { rechazar }`), o (b) hacer que sea IMPOSIBLE construir el valor prohibido en primer lugar. La opción (b) es estrictamente más fuerte: un chequeo en runtime puede tener un caso borde que alguien olvidó cubrir; un tipo que no tiene los campos necesarios para representar el dato prohibido no necesita chequeo — el compilador lo rechaza antes de que el programa exista.

**Cómo se ve en el código** (`crates/features/data/hybrid-data-transformer/src/domain.rs`):

```rust
pub struct RawTrade {
    pub event_timestamp_ns: i64,
    pub price_scaled: i64,
    pub trade_volume_scaled: i64,
    pub aggressor_side: Option<AggressorSide>,
    pub origin_seq: u64,
}
```

Fíjate en lo que NO está: no hay `bid`, no hay `ask`. Una cotización (`QuoteTick` en el vocabulario de ADR-0155) se define precisamente por tener esos dos campos. Como `RawTrade` no los tiene, no existe ninguna función en este crate que pueda recibir "una cotización disfrazada de ejecución" — el tipo del parámetro (`&[RawTrade]`) ya excluye esa posibilidad antes de que el programa corra. Es la misma idea que "no se puede pasar un `String` donde se espera un `u32`", aplicada a un concepto de negocio en vez de a un tipo primitivo.

**Dónde se demuestra en runtime, no solo en la firma del tipo:** la garantía de tipo es sólida DENTRO de Rust, pero el dato entra al sistema como JSON (por el CLI o el futuro puente FFI), que no tiene tipos en tiempo de compilación. Por eso el retrofit añade una prueba en la FRONTERA de deserialización (`crates/features/data/hybrid-data-transformer/tests/integration_tests.rs`):

```rust
let quote_shaped_json = r#"{
    "event_timestamp_ns": 1700000000000000000,
    "bid": 4300000000000,
    "ask": 4300100000000,
    "origin_seq": 0
}"#;
let result: Result<TradeTickInput, _> = serde_json::from_str(quote_shaped_json);
assert!(result.is_err(), "...");
```

`TradeTickInput` (la versión serializable de `RawTrade`, en `schemas.rs`) declara `price_scaled`/`trade_volume_scaled` como campos SIN `#[serde(default)]` — es decir, obligatorios. Un JSON que solo trae `bid`/`ask` (sin esos dos campos) falla al deserializar con "missing field `price_scaled`", ANTES de que el dato llegue a ninguna función de agregación. Se puede comprobar en la terminal real (no solo en el test):

```
$ cargo run -p app -- verify hybrid-data-transformer --input '{"ticks":[{"event_timestamp_ns":1700000000000000000,"bid":100,"ask":101,"origin_seq":0}]}'
Error al parsear --input JSON: missing field `price_scaled` at line 1 column 113
```

Esa es la prueba discriminante completa: garantía de tipo en Rust + prueba ejecutable en la frontera real de entrada (JSON).

### 10. El patrón "todas conocidas o `None`" — cómo se codifica "nunca rellenar por defecto"

**El problema:** ADR-0155 exige que las cuatro magnitudes de volumen (`volume_traded`, `trade_count`, `volume_buy`, `volume_ticks`) nunca se rellenen a partir de otra. En particular, `volume_buy` (volumen del lado comprador) solo tiene sentido si la fuente distingue el lado (`aggressor_side`) para las ejecuciones agregadas. Si una sola ejecución del grupo no lo distingue, ¿qué se hace? Sumar solo las conocidas y fingir que eso es "el total" sería mentir (subestimaría el volumen comprador real). La respuesta correcta es: si no se puede calcular el TOTAL con certeza, no se calcula nada — se deja `None`.

**Cómo se ve en el código:**

```rust
// crates/features/data/hybrid-data-transformer/src/domain.rs
fn sum_if_all_known<I: IntoIterator<Item = Option<i64>>>(values: I) -> Result<Option<i64>, TransformError> {
    let mut total: i64 = 0;
    for v in values {
        match v {
            Some(x) => total = total.checked_add(x).ok_or(TransformError::Overflow)?,
            None => return Ok(None),
        }
    }
    Ok(Some(total))
}
```

El patrón es: recorre una secuencia de `Option<i64>`; en el momento en que encuentras UN SOLO `None`, abortas devolviendo `Ok(None)` de inmediato (con `return` dentro del `for` — corta el resto del recorrido, no hace falta seguir sumando algo que ya sabes que será descartado). Solo si TODOS los elementos fueron `Some` llegas al final del bucle y devuelves `Some(total)`.

Para `volume_buy`, cada ejecución se traduce a un `Option<i64>` así:

```rust
let buy_contributions = group_trades.iter().map(|t| match t.aggressor_side {
    Some(AggressorSide::Buy) => Some(t.trade_volume_scaled),  // aporta su volumen
    Some(AggressorSide::Sell) | Some(AggressorSide::Unknown) => Some(0),  // se conoce el lado, aporta 0
    None => None,  // NO se conoce el lado -- invalida todo el grupo
});
let volume_buy = sum_if_all_known(buy_contributions)?;
```

Fíjate en la diferencia entre `Some(AggressorSide::Unknown)` (la fuente SÍ tiene el concepto de "lado", pero no pudo clasificar esta operación puntual — aporta `Some(0)`, sigue siendo "conocido") y `None` (la fuente NI SIQUIERA tiene el concepto — invalida todo). Es una distinción de tres niveles ("sí, es compra" / "sí, pero no es compra" / "no sé") que un simple `bool` o un valor por defecto no puede expresar — por eso `aggressor_side` es `Option<AggressorSide>` con tres variantes adentro, no un `bool` con `Option` por fuera nada más.

**La prueba discriminante que demuestra esto** (`domain.rs`):

```rust
#[test]
fn single_trade_with_unknown_side_nullifies_volume_buy_for_the_whole_bar() {
    let trades = vec![
        RawTrade { /* ... */ aggressor_side: Some(AggressorSide::Buy), /* ... */ },
        RawTrade { /* ... */ aggressor_side: None, /* ... */ },
    ];
    let bars = aggregate_trades_to_bars(&trades, 1_000).unwrap();
    assert_eq!(bars[0].volume_buy, None, "una sola ejecución sin lado conocido invalida volume_buy de toda la barra");
}
```

Una sola ejecución sin dato conocido basta para anular la magnitud completa de la barra — el test se cae si alguien "simplificara" la función para ignorar los `None` y sumar solo lo que sí conoce (ese es exactamente el defecto que ADR-0155 prohíbe: "nunca se asume por defecto").

### 11. Columnas Polars nulas (`Option<String>`) — cómo transportar "no sé" a través del motor columnar

**El problema:** `aggressor_side` puede estar ausente por ejecución. Polars necesita representar eso en una columna real (no es un `Vec<AggressorSide>` sin más, porque `AggressorSide` no tiene un cuarto valor para "ausente" -- ese es justo el trabajo de `Option`).

**La solución:** construir la columna como `Vec<Option<String>>` -- el macro `df![...]` de Polars entiende ese tipo y crea una columna `Utf8`/`String` con nulos reales (Arrow tiene un bitmap de validez nativo para esto, no hace falta un valor centinela como `"N/A"`):

```rust
// crates/features/data/hybrid-data-transformer/src/orchestrator.rs
let aggressor_side: Vec<Option<String>> =
    trades.iter().map(|t| aggressor_side_to_column_value(t.aggressor_side.map(Into::into))).collect();

let df = df![
    // ...
    "aggressor_side" => aggressor_side,
    // ...
]
```

Y para leerla de vuelta después del `sort()`, el accessor de Polars 0.54 para una columna de texto es `.str()` (en versiones más viejas de Polars se llamaba `.utf8()` -- el nombre cambió cuando el proyecto renombró internamente `Utf8Chunked` a `StringChunked`; si ves un error de "no encontré el método `utf8`" en una versión reciente, es exactamente este cambio de nombre, no un typo):

```rust
let aggressor_side = df.column("aggressor_side")?.str()?;
// .get(i) devuelve Option<&str> de forma natural -- sin .expect(), el nulo es legítimo aquí
// (a diferencia de las otras columnas, que sí usan .expect() porque nunca tienen huecos)
```

**Por qué esto importa más allá de este campo puntual:** es el patrón general para transportar CUALQUIER magnitud opcional-por-fuente a través de Polars sin inventar un valor centinela. Cuando esta feature (u otra) necesite `spread_reported`, `open_interest` u otro campo esquema-listo de verdad (no solo `None` fijo como hoy), el mismo patrón `Vec<Option<T>>` + `.str()`/`.i64()` con `.get(i)` aplica sin cambios.

### 12. Cuando la instrucción explícita y el ADR nuevo chocan — el valor de PARAR en vez de decidir en silencio

**El problema (no técnico, de disciplina de ingeniería):** ADR-0155 dice explícitamente que un `vwap` calculado por Drasus NUNCA se persiste — solo el que la fuente reporta con su propia convención. Pero el código de esta feature SÍ calcula `vwap` desde `TradeTick` (con la misma maquinaria `i128`/half-to-even de siempre), y la Orden de este retrofit pedía EXPLÍCITAMENTE conservar ese invariante numérico y sus pruebas sin tocarlos. Dos instrucciones correctas, en conflicto real.

**La tentación (mala):** resolver el conflicto en silencio -- o borrar `vwap` (rompiendo pruebas que la Orden dijo que no se tocaran) o dejarlo como si no pasara nada (ignorando una regla explícita de un ADR recién canonizado).

**La disciplina correcta, aplicada aquí:** cuando dos fuentes de autoridad legítimas (una Orden de trabajo y un ADR) chocan de verdad, no le toca al ingeniero que está escribiendo código decidir cuál gana -- eso es una decisión de gobernanza, no de implementación. Lo correcto es: (1) identificar el conflicto con precisión (¿en qué línea, bajo qué condición se manifiesta?), (2) verificar si hoy existe una violación REAL o solo LATENTE (aquí: `Bar` todavía no se escribe a Parquet en esta Story, así que no hay persistencia real todavía -- la violación es LATENTE, no activa), y (3) dejar la decisión documentada y explícita para quien tiene la autoridad de resolverla (aquí, el Tech-Lead, en `docs/execution/STORY-051-hybrid-data-transformer.md` §8), en vez de enterrarla en un comentario de código que nadie audita a tiempo.

Este patrón (no soy yo quien decide, dejo la decisión visible y accionable para quien sí puede) es tan importante como cualquier invariante numérico de esta Story -- es lo que evita que un ADR de un día se aplique a medias sin que nadie se entere.

---

## Corrección DEBT-028 (2026-07-18): la frontera materia prima / indicador, y por qué un `None` ambiguo es un antipatrón

> El punto que quedó abierto en el §12 anterior (¿gana la Orden que pedía conservar `vwap`, o gana ADR-0155 que prohíbe persistirlo?) lo falló el Tech-Lead: se anula el campo. Esta sección explica, desde cero, el porqué de fondo -- no es solo "el ADR ganó", es una distinción de arquitectura que aplica a cualquier feature de ingesta futura. Ver la resolución completa en [`docs/debt/DEBT-028.md`](../../debt/DEBT-028.md) y en [STORY-051 §8 punto 1](../../execution/STORY-051-hybrid-data-transformer.md#8-deudas-decisiones-y-constancia-del-gate-de-coherencia).

### 13. Materia prima vs. indicador -- por qué el VWAP no es un dato, es un cálculo

**El problema, explicado desde cero:** en un sistema de trading, los datos de mercado se dividen en dos categorías con reglas MUY distintas.

- **Materia prima (primitivos):** lo que el mercado literalmente imprimió. Cuando alguien compra 2 BTC a 100 000 dólares, ese hecho -- ese precio, ese volumen, ese instante -- ya ocurrió. Nadie lo "calcula": se observa y se guarda tal cual. Apertura, máximo, mínimo, cierre de una barra, el volumen negociado, el lado comprador si la fuente lo distingue -- todo esto es materia prima, aunque ya esté "agregado" en una barra (agregar 100 ejecuciones en un OHLCV sigue siendo un resumen mecánico de hechos, no una interpretación).
- **Indicador (derivado):** cualquier número que se OBTIENE aplicando una fórmula sobre la materia prima, con el propósito de ayudar a decidir (comprar/vender/esperar). El precio promedio ponderado por volumen (VWAP) es exactamente esto: `sum(precio_i × volumen_i) / sum(volumen_i)`. Es una fórmula aplicada sobre datos ya persistidos -- ni más ni menos que el ATR (rango promedio verdadero), el RSI (índice de fuerza relativa) o una media móvil. Nadie discutiría que el RSI debería vivir en la tabla de ingesta de ticks; el VWAP calculado por Drasus es exactamente la misma categoría de cosa, aunque "se sienta" más cerca del precio por estar en la misma escala.

**Por qué esto es una regla de ARQUITECTURA y no una preferencia de estilo:** el proyecto separa el pipeline en fases (`docs/README.md`, pipeline `ingest → generate → validate → incubate → manage → execute → feedback → withdraw`). La fase `ingest` (donde vive esta feature) tiene un trabajo: convertir datos crudos de proveedores externos en una tabla limpia y tipada. El motor de indicadores (EPIC-2/EPIC-3, todavía no construido) tiene otro trabajo: calcular estadísticas derivadas que las estrategias van a consumir para decidir. Si `ingest` empieza a calcular indicadores "porque ya tiene los números a la mano", dos problemas aparecen con el tiempo: (1) cada indicador nuevo que alguien necesite (ATR, RSI, Bollinger, VWAP con otra convención de precio...) se vuelve una razón para tocar la feature de ingesta, que debería ser estable y aburrida; (2) el motor de indicadores termina reimplementando la misma fórmula dos veces (una vez "de regalo" en `ingest`, otra vez "de verdad" en el motor) sin que nadie decida cuál es la fuente de verdad.

**La distinción fina que el propio DEBT-028 señala (para no confundirla con otra regla que se parece):** ADR-0155 tiene un principio distinto, "guarda el hecho, calcula la interpretación", que prohíbe persistir columnas como `adjusted_close` (precio ajustado por split) o el contrato continuo de futuros. Ese principio es sobre **mutabilidad retroactiva**: un split que ocurre HOY cambia el valor "correcto" de una barra de HACE 5 AÑOS -- si esa reinterpretación se persistiera como columna, cada split futuro forzaría reescribir particiones históricas completas (prohibido en un sistema append-only, ADR-0141). El VWAP de una barra NO tiene ese problema: una vez que la barra cierra, su VWAP nunca cambia por un evento futuro -- es un cálculo estable, no una reinterpretación. El motivo de sacarlo del `Bar` es distinto: es una **frontera de fase** (¿en qué módulo vive este cálculo?), no un problema de mutabilidad. Dos reglas parecidas ("no persistas esto"), dos causas raíz completamente distintas -- confundirlas llevaría a "arreglar" el problema equivocado (por ejemplo, intentando versionar el VWAP en vez de simplemente sacarlo de la ingesta).

**Cómo se ve la corrección en el código real** (`crates/features/data/hybrid-data-transformer/src/domain.rs`): el `struct Bar` ya no tiene el campo `vwap: Option<i64>`. Lo que SÍ se conservó -- porque es aritmética correcta y reutilizable, no el campo en sí -- son las dos funciones que lo habrían calculado:

```rust
// Siguen existiendo, pub, con sus pruebas -- el motor de indicadores las
// reutilizará cuando exista (EPIC-2/EPIC-3), en vez de reescribirlas.
pub fn checked_scaled_product(a: i64, b: i64) -> Option<i128> { /* ... */ }
pub fn round_half_to_even_div_i128(numerator: i128, denominator: i128) -> Option<i64> { /* ... */ }
```

La prueba que antes ejercitaba esta aritmética A TRAVÉS de `aggregate_trades_to_bars` (construyendo una barra completa solo para leer `bar.vwap`) ahora la ejercita DIRECTAMENTE:

```rust
// domain.rs -- antes: vwap_computation_succeeds_at_realistic_upper_bound_magnitude
// ahora: wide_accumulator_division_succeeds_at_realistic_upper_bound_magnitude
#[test]
fn wide_accumulator_division_succeeds_at_realistic_upper_bound_magnitude() {
    let price_scaled = 100_000 * 100_000_000i64;
    let volume_scaled = 10_000_000 * 100_000_000i64;

    let product = checked_scaled_product(price_scaled, volume_scaled).expect("no debe desbordar con i128");
    let result = round_half_to_even_div_i128(product, volume_scaled as i128).expect("no debe desbordar al redondear");

    assert_eq!(result, price_scaled);
}
```

El truco de senior aquí: cuando un campo se retira pero la MAQUINARIA detrás de ese campo sigue siendo válida, la prueba correcta no es "borrar el test" -- es "reapuntar el test a la maquinaria, no al campo que la envolvía". El criterio de aceptación (2: sin desbordamiento silencioso) sigue exactamente igual de cubierto; solo cambió el camino por el que se llega a probarlo.

### 14. Por qué un `None` que confunde "falló el cálculo" con "no hay dato" es un antipatrón

**El problema, explicado desde cero:** en Rust, `Option<T>` tiene exactamente dos posibilidades: `Some(valor)` o `None`. Eso es perfecto cuando `None` significa UNA sola cosa. El defecto que la auditoría del Tech-Lead encontró en el código retirado es que el `vwap: Option<i64>` de `Bar` usaba `None` para representar DOS situaciones completamente distintas, sin forma de distinguirlas después:

1. **"La fuente no reporta VWAP"** -- una ausencia legítima de dato, esperada y sana (la mayoría de las fuentes de ejecuciones no traen un VWAP propio).
2. **"El cálculo del VWAP falló"** -- `round_half_to_even_div_i128` devuelve `None` si el divisor es cero o si el resultado, tras redondear, no cabe en `i64`. Esto NO es "no hay dato": es un error aritmético real que alguien debería poder ver.

El código que existía antes de esta corrección hacía, en esencia, `let vwap = round_half_to_even_div_i128(numerator, denominator);` y guardaba ese `Option<i64>` directo en `Bar.vwap` sin ninguna distinción. Si mañana una barra con datos corruptos producía un desbordamiento real, el resultado en `Bar` era IDÉNTICO -- bit a bit -- al de una barra sana cuya fuente simplemente no reporta VWAP. Nadie que leyera esa barra podría saber, sin ir a leer el código fuente de la función, cuál de las dos situaciones ocurrió.

**Por qué esto importa en la práctica (no es un purismo teórico):** un fallo aritmético silencioso es exactamente el tipo de defecto que este proyecto existe para prevenir (`.agents/knowledge/base.md` -- "no inventes", "no alucines"; el motor de indicadores está construido para NUNCA fingir certeza donde no la hay). Si un `None` por error de cálculo se ve igual que un `None` por ausencia de fuente, un desarrollador que audite datos raros meses después va a asumir "ah, esta fuente no trae VWAP" cuando en realidad la aritmética falló y nadie se enteró. El patrón correcto -- que esta feature YA aplica en otro campo, `volume_buy`, y que sirvió de referencia para reconocer el defecto -- es que un campo derivado necesita DOS canales separados: un `Result` (o un log/métrica) para el fallo de cálculo, y un `Option` para la ausencia legítima de dato. Nunca el mismo `None` para ambos.

**El segundo defecto relacionado, en `reaggregate_bars` (barra→barra, ej. 1m→1h):** el código retirado calculaba el "VWAP compuesto" de una barra mayor ponderando el VWAP de cada sub-barra por su propio volumen -- pero si una sub-barra no tenía VWAP (`None`), el código simplemente la SALTABA del numerador y del denominador, en vez de propagar `None` para toda la barra combinada:

```rust
// Código RETIRADO (ya no existe) -- se muestra aquí solo como referencia
// histórica del defecto que esta corrección elimina, NO como código vigente.
for b in &sub_bars {
    if let Some(sub_vwap) = b.vwap {
        // sub-barras SIN vwap simplemente se excluían de la suma...
        numerator += checked_scaled_product(sub_vwap, b.volume_traded)?;
        denominator += b.volume_traded as i128;
    }
    // ...y el resultado final se presentaba como "el VWAP de la barra
    // completa", cuando en realidad solo promediaba las sub-barras que SÍ
    // tenían el dato -- un promedio parcial disfrazado de total.
}
```

Esto es EXACTAMENTE el mismo error que la regla "todas conocidas o `None`" de `volume_buy` existe para impedir (concepto 10 de esta lección, `sum_if_all_known`): si no puedes calcular el TOTAL con certeza porque a algún componente le falta el dato, la respuesta correcta es `None` para el conjunto completo, nunca un resultado parcial que se presenta como si fuera completo. El código de `volume_buy` ya tenía la disciplina correcta (una sola sub-barra sin dato invalida toda la barra agregada); el código de `vwap` tenía la disciplina incorrecta (ignoraba silenciosamente lo que faltaba) -- una inconsistencia entre dos campos del mismo `struct`, que es exactamente el tipo de defecto que una auditoría de otro par de ojos (aquí, el Tech-Lead) está para atrapar.

**Ambos defectos desaparecen al retirar el campo -- no se "arreglaron" en el sentido de escribir código nuevo que los resuelva.** Es la lección más importante de esta corrección: a veces la solución correcta a un defecto no es una función más cuidadosa, es reconocer que el dato no debería estar ahí en absoluto. Intentar arreglar la ambigüedad del `None` (por ejemplo, cambiando `Option<i64>` por un `Result<Option<i64>, TransformError>` para `vwap`) habría sido resolver el síntoma sin tocar la causa raíz -- el campo seguiría en la fase equivocada del pipeline.
