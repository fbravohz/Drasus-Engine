# STORY-068 — Lector de barras de MetaTrader: lecciones de Rust

Doce conceptos de Rust, explicados desde cero, usando el código real que se
escribió para leer el archivo de 9.2 millones de barras del propietario.
Cada sección cita la ruta y el fragmento exacto — nada de ejemplos de manual.
Los Conceptos 11 y 12 se añadieron tras la auditoría de mutación del
Tech-Lead y la observación del propietario sobre la unidad del spread.

---

## Concepto 1: `&str` vs `String` — quién es dueño del texto

Rust tiene DOS tipos de texto, y la diferencia es la primera pregunta que
cualquier función debe responder: "¿esta función va a QUEDARSE con este
texto, o solo lo va a MIRAR un instante?".

- `String` es dueña de su memoria — como fotocopiar un documento para
  quedártelo. Puedes modificarlo, guardarlo, pasarlo a otra parte del
  programa que lo conserve.
- `&str` es una VENTANA hacia texto que vive en otro lado — como leer un
  documento por encima del hombro de alguien sin llevártelo. Es gratis
  (no copia nada), pero solo dura mientras el original siga vivo.

Código real, `crates/features/data/data-import-wizard/src/domain.rs`:

```rust
pub fn parse_mt5_bar_line(line: &str) -> Result<Mt5RawBarFields, Mt5LineParseError> {
    let stripped = strip_line_ending(line);
    let fields: Vec<&str> = stripped.split('\t').collect();
```

`parse_mt5_bar_line` recibe `&str` (una ventana), no `String` (una copia).
El archivo tiene 9.2 millones de líneas — si cada llamada a esta función
copiara la línea completa, el programa gastaría memoria multiplicándola
sin necesidad. Como la función solo necesita LEER la línea, mirarla un
instante y decidir qué números contiene, una ventana alcanza. `fields` es
un `Vec<&str>`: un vector de ventanas hacia pedazos de la línea original,
no nueve copias de texto.

**Qué pasaría distinto:** si `parse_mt5_bar_line` recibiera `String` en vez
de `&str`, cada llamada a esta función OBLIGARÍA a quien la invoca a
entregarle su copia (o a hacer una copia nueva solo para dársela) — sobre
9.2 millones de líneas, eso son 9.2 millones de copias de string
innecesarias antes de siquiera empezar a parsear.

---

## Concepto 2: `Result<T, E>` y el operador `?` — errores sin excepciones

Otros lenguajes usan "excepciones": una función puede lanzar un error que
interrumpe el programa a menos que alguien lo atrape. Rust no tiene eso.
En Rust, toda función que puede fallar declara EN SU FIRMA que puede
fallar, devolviendo `Result<T, E>`: o el resultado bueno (`Ok(T)`) o el
motivo del fallo (`Err(E)`). El compilador te OBLIGA a decidir qué hacer
con cada posible fallo — no hay forma de "olvidarte" de un error.

Código real, `crates/features/data/data-import-wizard/src/domain.rs`:

```rust
pub fn resolve_event_timestamp_ns(
    date: NaiveDate,
    time: NaiveTime,
    source_timezone: Option<&str>,
) -> Result<i64, TimestampResolutionError> {
    let tz_name = source_timezone.ok_or(TimestampResolutionError::MissingSourceTimezone)?;
    let tz: Tz = Tz::from_str(tz_name).map_err(|_| TimestampResolutionError::UnknownTimezone(tz_name.to_string()))?;
```

El signo `?` al final de una línea significa: "si esto salió mal, DETENTE
AQUÍ MISMO y devuelve el error hacia quien me llamó — no sigas ejecutando
con datos rotos". `source_timezone.ok_or(...)` convierte "no hay zona
horaria declarada" (`None`) en un error tipado; el `?` hace que, si eso
pasa, la función entera se corte ahí, sin llegar nunca a calcular un
timestamp con una zona inventada. Esto es, en código, la regla de negocio
del criterio de aceptación #3: **una declaración ausente hace fallar la
importación, nunca asume UTC en silencio.**

---

## Concepto 3: enums con datos + `thiserror` — errores que explican el motivo exacto

Un enum en Rust no es solo una lista de nombres (como "rojo, verde, azul")
— cada variante puede CARGAR datos distintos. Esto permite modelar "todas
las formas en que algo puede salir mal", cada una con la información
exacta que necesita para explicarse.

Código real, `crates/features/data/data-import-wizard/src/domain.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FieldParseError {
    #[error("el campo de precio {field:?} no es un decimal válido: {raw:?}")]
    InvalidPrice { field: &'static str, raw: String },
    #[error(
        "el campo de precio {field:?} trae más de 8 decimales ({raw:?}) — se rechaza en vez de \
         truncar en silencio y perder precisión"
    )]
    ExcessPricePrecision { field: &'static str, raw: String },
    #[error("el campo entero {field:?} no es un entero válido: {raw:?}")]
    InvalidInteger { field: &'static str, raw: String },
}
```

Cada variante de `FieldParseError` guarda EXACTAMENTE lo que hace falta
para que el mensaje de rechazo sea accionable: qué campo falló (`<OPEN>`,
`<SPREAD>`...) y qué texto crudo trajo el problema. `thiserror::Error` es
una herramienta (una "derivación de código") que, a partir de la anotación
`#[error("...")]` sobre cada variante, escribe automáticamente el código
repetitivo de "cómo convertir este error en un mensaje de texto" — sin
`thiserror`, habría que escribir esa conversión a mano para cada una de las
tres variantes.

**Por qué importa aquí:** ADR-0156 exige "rechazo con motivo explícito,
nunca un error de parseo genérico". Un `String` de error genérico
("parseo fallido") no distingue un precio con demasiados decimales de un
entero corrupto; este enum sí.

---

## Concepto 4: por qué los precios se calculan como texto, no como `f64`

Un número de punto flotante (`f64`, el tipo de "decimal" que casi todo
lenguaje usa por defecto) NO representa exactamente la mayoría de los
decimales que un humano escribe. Es una limitación física de cómo la
computadora guarda fracciones en binario, no un descuido de Rust.

Código real, `crates/features/data/data-import-wizard/src/domain.rs`:

```rust
/// Implementación por texto, NUNCA por `f64::parse` + multiplicación: un
/// `f64` no representa exactamente la mayoría de los decimales financieros
/// (`0.1_f64 * 1e8` ya no es `10_000_000` exacto en binario) — el redondeo
/// resultante rompería la ida y vuelta exacta que el criterio de aceptación
/// #2 de STORY-068 exige.
pub fn parse_price_to_scaled(field: &'static str, raw: &str) -> Result<i64, FieldParseError> {
```

En vez de convertir `"1.11488"` a `f64` y multiplicar por 100 millones (lo
obvio, y lo que casi cualquier tutorial haría), la función separa la parte
entera (`"1"`) de la fraccionaria (`"11488"`) COMO TEXTO, y hace la
aritmética con enteros (`i64`) desde el principio. Los enteros en binario
SÍ son exactos — no hay redondeo posible.

**La prueba que lo demuestra**, en el mismo archivo:

```rust
fn price_scale_round_trip_preserves_real_observed_values_including_extremes() {
    for raw in ["1.00730", "1.11488", "0.82310", "1.60380", "1.11670"] {
        let scaled = parse_price_to_scaled("<OPEN>", raw).expect("precio real debe parsear");
        let back = scaled_to_price_string(scaled, 5);
        assert_eq!(back, raw, "la ida y vuelta debe reconstruir exactamente el valor original: {raw}");
    }
}
```

Convierte un precio a entero y de vuelta a texto, y exige que el resultado
sea el MISMO texto exacto que entró — sobre los extremos reales observados
en 25 años de archivo (0.82310 el mínimo, 1.60380 el máximo). Con `f64`,
esta prueba habría fallado silenciosamente en algún punto del archivo, con
un dígito de más o de menos que nadie habría notado a simple vista.

---

## Concepto 5: zonas horarias — `LocalResult` y por qué una hora local puede no existir (o existir dos veces)

Cuando el reloj de una zona horaria "brinca" por el cambio de horario de
verano, pasan dos cosas raras que un programa DEBE manejar explícitamente:

- **En primavera**, el reloj salta de 02:00 directo a 03:00 — la hora
  "02:30" literalmente NO EXISTE ese día en esa zona.
- **En otoño**, el reloj retrocede de 03:00 a 02:00 — la hora "02:30"
  OCURRE DOS VECES ese día (antes y después del retroceso).

`chrono-tz` (la librería que sabe las reglas de cada zona horaria del
mundo) modela esto con un tipo que tiene tres resultados posibles, no dos:

Código real, `crates/features/data/data-import-wizard/src/domain.rs`:

```rust
let local_result = tz.from_local_datetime(&naive_datetime);
let localized = match local_result {
    LocalResult::Single(dt) => dt,
    // Hora ambigua (cambio de horario de otoño): se toma la primera
    // ocurrencia -- ver doc-comment de la función.
    LocalResult::Ambiguous(earlier, _later) => earlier,
    LocalResult::None => {
        return Err(TimestampResolutionError::NonexistentLocalTime { date, time, timezone: tz_name.to_string() })
    }
};
```

`match` es la forma que tiene Rust de decir "manejo TODOS los casos
posibles de este valor, uno por uno" — el compilador rechaza el programa
si te olvidas de alguno. Aquí hay tres: hora normal (`Single`), hora
duplicada (`Ambiguous` — se elige la primera ocurrencia, una decisión
documentada, no una adivinanza), y hora inexistente (`None` — se rechaza
con un error explícito en vez de inventar un instante que nunca pasó).

---

## Concepto 6: traits — cómo Rust hace que dos cosas distintas se comporten "igual"

Un `trait` es una promesa: "cualquier tipo que implemente este trait sabe
hacer estas operaciones concretas". Es lo que permite que el código de
persistencia funcione igual sin importar si el reloj es el reloj real del
sistema o un reloj de prueba totalmente controlado.

`crates/shared/src/domain/clock.rs` define el trait:

```rust
pub trait Clock: Send + Sync {
    fn timestamp_ns(&self) -> i64;
}
```

Y `crates/features/data/data-import-wizard/src/persistence.rs` lo usa sin
saber ni importarle CUÁL implementación recibirá:

```rust
pub struct MappingDeclarationRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}
```

`&dyn Clock` significa "una referencia a CUALQUIER COSA que cumpla la
promesa del trait `Clock`, no me importa cuál en concreto". En producción
se le pasa `SystemClock` (el reloj real); en las pruebas de este Story se
le pasa `DeterministicClock` (un reloj de mentira que arranca en `0` y solo
avanza cuando el test se lo pide explícitamente). El repositorio del ledger
nunca se entera de la diferencia — y por eso los tests de concurrencia y
contención (que necesitan timestamps predecibles) pueden usar exactamente
el mismo código que corre en producción.

---

## Concepto 7: closures como "puertos" — inyectar comportamiento sin acoplar crates

Una "closure" es una función que se puede guardar en una variable y pasar
como argumento, como cualquier otro dato. STORY-068 la usa para resolver
un problema de arquitectura real: este lector NUNCA debe depender del
código de `hybrid-data-transformer` (la feature que de verdad escribe los
datos en disco) — esa regla viene de ADR-0137 ("prohibido acoplar features
entre sí").

Código real, `crates/features/data/data-import-wizard/src/orchestrator.rs`:

```rust
pub async fn import_mt5_bar_file<F>(
    pool: &SqlitePool,
    clock: &dyn Clock,
    input: ImportBarFileInput,
    bar_sink: F,
) -> Result<ImportBarFileOutcome, ImportBarFileError>
where
    F: FnOnce(&[CanonicalBarRow]) -> Result<(), String>,
```

`bar_sink` es un HUECO en la función: "dame cualquier función que reciba
las barras ya mapeadas y me diga si logró guardarlas". El lector no conoce
ni necesita conocer qué hay del otro lado de ese hueco. En producción, algo
compondrá ese hueco con una llamada real a `hybrid-data-transformer`; en
las pruebas de este Story, se le pasa una closure que simula un fallo:

```rust
import_mt5_bar_file(&pool, &clock, sample_input(REAL_SAMPLE_CSV.to_vec()), |_rows| {
    Err("fallo simulado de la persistencia de datos aguas abajo".to_string())
})
```

Esto es lo que permite probar el criterio de aceptación #5 ("si la
persistencia de datos falla, el ledger ya quedó sellado") SIN necesitar que
`hybrid-data-transformer` exista siquiera en la prueba — se simula el
fallo con una línea.

---

## Concepto 8: concurrencia real, no fingida — la prueba de los 16 escritores

Un error clásico al probar concurrencia es escribir un test que EJECUTA
varias tareas, pero de una en una, disfrazado de prueba paralela — pasa en
verde sin haber probado nada. La regla del proyecto es que una prueba de
concurrencia debe MEDIR comportamiento real bajo hilos de verdad.

Código real, `crates/features/data/data-import-wizard/src/persistence.rs`:

```rust
#[tokio::test(flavor = "multi_thread")]
async fn sixteen_concurrent_writers_never_lose_or_duplicate_sequence() {
    let (pool, _dir) = temp_pool().await;
    let clock = DeterministicClock::new(0, 1);
    let repo = Arc::new((pool, clock));

    let mut handles = Vec::new();
    for i in 0..16 {
        let repo = Arc::clone(&repo);
        handles.push(tokio::spawn(async move {
            let (pool, clock) = &*repo;
            let repository = MappingDeclarationRepository::new(pool, clock);
            repository.record(sample_new_declaration(&format!("writer-{i}"))).await
        }));
    }
```

`#[tokio::test(flavor = "multi_thread")]` arranca un runtime con VARIOS
hilos de sistema operativo de verdad, no uno solo fingiendo turnos.
`tokio::spawn` lanza 16 tareas que corren EN PARALELO, todas compitiendo
por escribir en la misma base de datos al mismo tiempo. `Arc` (Atomically
Reference Counted — "contador de referencias atómico") es lo que permite
que las 16 tareas compartan el mismo pool de conexión y el mismo reloj sin
que cada una necesite su propia copia — es un puntero compartido seguro
entre hilos, con un contador que sabe cuándo ya nadie lo usa para liberar
la memoria. La prueba exige que las 16 escrituras terminen con la secuencia
`1, 2, 3... 16` exacta, sin huecos ni números repetidos — eso es lo que
demuestra que el candado de escritura (`BEGIN IMMEDIATE`, concepto 9)
funciona bajo carga real.

---

## Concepto 9: `BEGIN IMMEDIATE` — el candado que evita perder una fila del ledger

Cuando dos escritores intentan escribir a la vez en la misma tabla, hay una
secuencia de pasos peligrosa: "leer cuál es el último número usado" y
"escribir el siguiente número" NO son una sola operación atómica por
defecto — entre esos dos pasos, otro escritor puede colarse y usar el mismo
número, y una fila del ledger se perdería o duplicaría en silencio.

Código real, `crates/features/data/data-import-wizard/src/persistence.rs`:

```rust
async fn try_record_once(
    &self,
    new: &NewMappingDeclaration,
) -> Result<MappingDeclarationRow, MappingDeclarationRepositoryError> {
    let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

    let tail_row = sqlx::query(
        "SELECT audit_hash, event_sequence_id FROM import_mapping_declarations \
         ORDER BY event_sequence_id DESC LIMIT 1",
    )
    .fetch_optional(&mut *tx)
    .await?;
    // ... (deriva el siguiente número) ...
    // ... INSERT de la fila nueva ...
    tx.commit().await?;
```

`BEGIN IMMEDIATE` toma el candado de escritura de la base de datos DESDE EL
PRIMER INSTANTE de la transacción — nadie más puede escribir hasta que esta
transacción termine (`tx.commit()`). Leer el último número Y escribir el
siguiente ocurren como un solo bloque indivisible. Si otro escritor ya
tiene el candado, esta transacción espera (o, si espera demasiado, falla
con un error de "contención" que el código reintenta un número acotado de
veces — la prueba `sustained_contention_exhausts_retries_and_returns_typed_error_with_exact_attempt_count`
demuestra exactamente ese límite).

---

## Concepto 10: la prueba que no probaba nada — el hallazgo real de esta Story

Durante la implementación, `parse_counter_field` (la función que convierte
`<SPREAD>`/`<TICKVOL>` a número) usaba `raw.trim()` antes de parsear —
`.trim()` quita espacios en blanco de los bordes del texto. El problema:
en Rust (y en Unicode en general), el carácter `\r` (retorno de carro, el
que Windows agrega al final de cada línea) **cuenta como espacio en
blanco**. Eso significaba que `.trim()` ya estaba borrando el `\r` sospechoso
ANTES de que la función de parseo lo viera — la prueba original pasaba en
verde, pero no porque `strip_line_ending` (la función que se supone debe
encargarse de esto) funcionara: pasaba porque `.trim()` tapaba el problema
por otro lado. Una prueba "verde" que pasa por la razón equivocada es
exactamente el tipo de defecto que este proyecto exige cazar (regla de
"demostración obligatoria de discriminación" del Rust-Engineer).

**Cómo se destapó:** al romper deliberadamente `strip_line_ending` para
demostrar el ciclo rojo→verde de la prueba (paso obligatorio antes de
entregar cualquier criterio de aceptación), la prueba SIGUIÓ EN VERDE — la
señal de que algo estaba mal, no de que todo estaba bien. La corrección fue
quitar el `.trim()` de los tres parseadores de campo (`parse_price_to_scaled`,
`parse_counter_field`, `parse_volume_field`), dejando que la limpieza del
`\r` sea responsabilidad EXCLUSIVA de `strip_line_ending` sobre la línea
completa. Recién ahí, romper `strip_line_ending` hizo que la prueba fallara
de verdad — la prueba pasó a discriminar lo que decía discriminar.

**La lección general:** que una prueba pase no basta. Hay que preguntarse
"¿por qué razón exacta pasó?" — y la única forma confiable de responder esa
pregunta es romper a propósito el código que la prueba dice cubrir y
verificar que, en efecto, se cae.

---

## Concepto 11: el gate de mutación — un robot que rompe tu código a propósito

Después de entregar la primera versión de este lector, el Tech-Lead corrió
una herramienta llamada `cargo-mutants` sobre el código. Su trabajo es
literalmente el Concepto 10 llevado al extremo y automatizado: en vez de
que un humano rompa una línea a mano para ver si una prueba se entera, el
programa rompe ÉL MISMO cada línea, una por una, de decenas de formas
distintas (cambia un `+` por un `*`, invierte un `>` a `>=`, borra un
signo negativo...) y corre TODA la batería de pruebas después de cada
rotura. Si las pruebas siguen en verde con el código roto, esa rotura
"sobrevivió" — es la prueba de que ninguna prueba realmente vigila esa
línea.

El resultado sobre este lector: **21 supervivientes de 101 mutaciones.**
Ejemplo real, `crates/features/data/data-import-wizard/src/domain.rs`:

```rust
pub fn mt5_bar_column_mapping() -> Vec<(String, String)> {
    [
        ("<DATE>", "bar_date_source"),
        ("<TIME>", "bar_time_source"),
        // ... siete pares más ...
```

Antes de esta ronda, NINGUNA prueba comparaba el contenido real de esta
plantilla contra algo esperado. El robot de mutación cambió la función
entera por `vec![]` (una plantilla VACÍA) y las pruebas existentes
siguieron en verde — porque ninguna leía el resultado de esta función, solo
confiaban en que "seguramente" tenía los datos correctos. La corrección:

```rust
#[test]
fn mt5_bar_column_mapping_declares_exactly_the_nine_canonical_correspondences() {
    let mapping = mt5_bar_column_mapping();
    let expected = vec![
        ("<DATE>".to_string(), "bar_date_source".to_string()),
        // ... las nueve, una por una ...
    ];
    assert_eq!(mapping, expected, "la plantilla debe declarar exactamente estas nueve correspondencias, en este orden");
}
```

Un caso más sutil, el mismo archivo, línea del escalado de precio:

```rust
Ok(sign * (integer_value * PRICE_SCALE + fractional_value))
```

El robot cambió el PRIMER `*` por un `+`: `sign + (...)` en vez de
`sign * (...)`. Para un precio POSITIVO (`sign = 1`), `1 * X` y `1 + X`
solo difieren en exactamente 1 unidad de 100 millones — un error minúsculo
que la prueba de ida y vuelta (que solo mostraba 5 decimales) nunca podía
ver, porque esos 5 decimales visibles ni siquiera llegan al dígito donde
vive el error. La corrección fue una prueba que compara el número ENTERO
exacto, sin pasar por el recorte de decimales:

```rust
assert_eq!(parse_price_to_scaled("<OPEN>", "1.00000000").unwrap(), 100_000_000);
assert_eq!(parse_price_to_scaled("<OPEN>", "-1.23000000").unwrap(), -123_000_000);
```

El segundo caso (precio NEGATIVO) es el que de verdad mata este mutante:
con `sign = -1`, `-1 * X` y `-1 + X` son radicalmente distintos (`-X` contra
`X - 1`), así que cualquier prueba con un número negativo lo atrapa sin
ambigüedad.

**Por qué esto importa más que "otra prueba más":** un mutante que
sobrevive no es un defecto en el CÓDIGO — el código de este lector estaba
correcto todo el tiempo. Es un defecto en la RED que debería detectar si
algún día alguien (una persona, o un cambio automatizado, o el propio
autor meses después) rompe esa línea sin querer. El gate de mutación no
pregunta "¿funciona hoy?" — pregunta "¿alguien se enteraría si dejara de
funcionar?".

---

## Concepto 12: un número sin unidad no es un dato — es una trampa

El propietario señaló algo que ninguna prueba, ni la implementación
original, capturaba: el campo `<SPREAD>` de MetaTrader (`spread_reported_raw`
en el código) se guardaba como el entero crudo que el archivo trae, con un
comentario que decía honestamente "esto es crudo, sin escalar" — pero
NADA en el dato mismo, ni en el registro sellado, decía en QUÉ UNIDAD está
ese entero.

¿Por qué es grave? Porque `10` significa cosas completamente distintas
según el instrumento: en EURUSD (5 decimales), 10 "puntos" son 1 pip —
un spread normal. Si alguien interpretara ese mismo `10` como "10 pips"
directamente, estaría leyendo un spread diez veces más grande de lo real
— y ese error de interpretación viviría escondido en el código de quien lo
consuma después, no en el dato. El dato, sin la unidad viajando con él, es
ambiguo por diseño.

La corrección — código real, `crates/features/data/data-import-wizard/src/domain.rs`:

```rust
pub const MT5_SPREAD_REPORTED_UNIT: &str = "POINTS_MT5";
```

Y en la fila canónica, un campo nuevo que viaja SIEMPRE junto al número:

```rust
pub struct CanonicalBarRow {
    // ...
    pub spread_reported_raw: i64,
    pub spread_reported_unit: String,
    // ...
}
```

La regla de negocio detrás de este campo reutiliza exactamente el
`Result`/`Option` del Concepto 2: si la plantilla no declara esta unidad,
`map_to_canonical_bar` RECHAZA la importación en vez de adivinar —

```rust
let spread_reported_unit =
    ctx.spread_reported_unit_declared.ok_or(CanonicalMappingError::MissingSpreadUnitDeclaration)?.to_string();
```

— el mismo patrón exacto que ya protegía la zona horaria (Concepto 5) y la
unidad del volumen (criterio de aceptación #8), aplicado ahora a un tercer
campo que sufre el mismo problema de fondo: **un número sin la unidad en
la que está expresado no es información, es una adivinanza pendiente.**

---

## Trucos de Senior

- **Nunca confíes en que un helper "de cortesía" (`.trim()`, `.unwrap_or_default()`,
  cualquier función que "arregla" datos silenciosamente) no esté enmascarando
  un bug en otra capa.** Si dos mecanismos distintos podrían resolver el
  mismo problema, hay que verificar CUÁL de los dos es el que realmente
  actúa — no asumir que es el que se diseñó a propósito para eso.
- **El signo `?` y `match` sobre enums con datos son la forma en que Rust
  reemplaza "acordarse de manejar los errores"** por "el compilador no te
  deja compilar si te falta un caso". Cuanta más lógica de negocio se
  exprese como variantes de un enum en vez de banderas sueltas (`bool`,
  `Option<String>` genérico), menos posible es olvidar un caso borde.
- **Cuando dos features no deben depender la una de la otra en código
  (regla de arquitectura de este proyecto), una closure/`trait` como
  parámetro genérico es el mecanismo correcto** para cablear su
  colaboración sin que ninguna importe el crate de la otra — la
  composición real ocurre en una capa aparte que sí conoce a ambas.
- **Antes de escribir la prueba de un comportamiento concurrente o de
  contención, pregúntate: "¿esto pasaría igual si el código corriera un
  hilo a la vez?"** Si la respuesta es sí, la prueba no está probando
  concurrencia — está fingiendo que lo hace.
- **Toda función que devuelva un valor "resumen" de una entrada más grande
  (un hash, un mapeo, una plantilla) necesita una prueba que compare el
  CONTENIDO real, no solo "no truena" o "no está vacío".** Una función que
  siempre devuelve la misma constante compila, corre y no truena — y sigue
  siendo completamente inútil.
- **Un número sin su unidad es una interpretación pendiente disfrazada de
  dato.** Si un campo puede significar cosas físicamente distintas según
  el contexto (puntos vs. pips, lotes vs. unidades base), la unidad va
  sellada JUNTO al valor — nunca solo en un comentario o en el nombre de
  la variable, que no viajan con el dato cuando alguien más lo lee.
