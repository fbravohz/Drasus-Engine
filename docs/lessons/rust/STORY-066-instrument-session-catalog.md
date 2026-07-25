# STORY-066 — Catálogo de Instrumento y Sesión de mercado: lecciones de Rust

> **Story:** [STORY-066 — Catálogo de Instrumento y Sesión de mercado](../../execution/STORY-066-instrument-session-catalog.md).
> **Archivos que esta Story produjo y que se citan abajo:** `migrations/0022_instrument_session_catalog.sql`, `crates/features/data/instrument-catalog/src/{domain,persistence,orchestrator,public_interface}.rs`, `crates/features/data/session-catalog/src/{domain,persistence,orchestrator,public_interface}.rs`, `crates/app/src/main.rs`.
> **Modo de Acompañamiento:** Docente (ADR-0122) — el ingeniero implementó cada bloque por su cuenta y este archivo consolida lo enseñado, con profundidad cero-conocimiento (ADR-0124).

## Concepto

### Dos "cosas que suenan como una sola" son dos tablas con roles distintos de identidad

Cuando el propietario comparó nuestro modelo contra StrategyQuant X, encontró que ese sistema mezcla "identidad del símbolo" (EURUSD existe, tiene 5 decimales) con "condiciones de trading de ese símbolo en un bróker concreto" (el spread de OANDA para EURUSD) en una sola fila repetida por cada bróker. ADR-0162 separa eso en dos tablas con una regla de identidad muy concreta que vale la pena entender bien porque aparece en casi todo el código de esta Story:

```rust
// crates/features/data/instrument-catalog/src/schemas.rs
pub struct InstrumentMasterRow {
    pub id: String,             // clave TÉCNICA — UUIDv7, nunca la usa nadie fuera de esta fila
    pub instrument_id: String,  // clave de NEGOCIO — "EURUSD", la misma que ya circula en Bars/QuoteTick
    // ...
}
```

`id` cumple la regla universal de ADR-0141 ("toda tabla tiene una PK `TEXT` con UUIDv7") — nadie del exterior del sistema conoce ni le importa este valor. `instrument_id` es la cadena legible que YA existe en el resto del sistema (`"EURUSD"`) desde antes de que esta Story se escribiera (ADR-0155). La tabla hija (`broker_instrument_terms`) referencia el SEGUNDO, no el primero:

```sql
-- migrations/0022_instrument_session_catalog.sql
instrument_id  TEXT NOT NULL
    REFERENCES instrument_master (instrument_id) ON DELETE RESTRICT,
```

Esto es inusual: la mayoría de las FK del proyecto (`owner_id → accounts(id)`) referencian la clave técnica. Aquí se referencia una columna `UNIQUE` que NO es la PK (`instrument_id`, no `id`). SQLite lo permite siempre que la columna referenciada tenga una restricción `UNIQUE` o sea PK — y es la elección correcta aquí porque `broker_instrument_terms` nunca necesita el UUID técnico: solo necesita el mismo string que ya usan `Bars`/`QuoteTick`, para no obligar a nadie a "traducir" entre dos identificadores del mismo instrumento. La lección general: cuando un concepto de negocio ya tiene una clave legible circulando por todo el sistema, colapsarla con la PK técnica universal (forzando que TODO sea UUID) rompe la compatibilidad con lo que ya existe; mantener las dos, cada una con su rol, es más código pero cero ambigüedad.

### Herencia por `NULL`: `Option<T>` como vocabulario de negocio, no solo como "tal vez falta un dato"

En la mayoría del código, un campo `Option<i64>` en `None` normalmente significa "no sabemos este valor" o "hubo un error de captura". Aquí significa exactamente lo contrario: "el usuario declaró, a propósito, que este bróker no necesita un valor propio — usa el canónico." Es una decisión de diseño, no una ausencia accidental. La función que resuelve esto es pura (sin tocar la base de datos) y cabe en pocas líneas:

```rust
// crates/features/data/instrument-catalog/src/domain.rs
pub fn resolve_terms(canonical: &CanonicalTerms, override_terms: Option<&BrokerOverride>) -> ResolvedInstrumentTerms {
    let pip_size = override_terms
        .and_then(|o| o.pip_size_override)   // Option<Option<i64>> -> Option<i64>: aplana "hay override Y trae valor"
        .unwrap_or(canonical.canonical_pip_size); // si cualquiera de los dos niveles fue None, usa el canónico
    // ...
}
```

`and_then` es el operador de Rust para "si hay un valor, transforma; si no, propaga `None` sin ejecutar el cierre". Aquí lo usamos dos veces encadenado: `override_terms` (¿existe una fila de override para este bróker?) y, dentro de ella, `pip_size_override` (¿ese bróker concreto declaró un valor para ESTE campo?). Si CUALQUIERA de los dos es `None` — no hay override en absoluto, o hay override pero este campo específico quedó sin declarar — el resultado final es `None`, y `unwrap_or` cae al valor canónico. Es el mismo patrón que produce el criterio de aceptación #3 ("borrar el override resuelve 100% desde el canónico"): con `override_terms = None`, TODOS los `and_then` de la función devuelven `None` sin excepción, sin que haga falta un `if` especial para el caso "sin bróker".

La prueba que demuestra esto no compara el resultado consigo mismo — construye DOS entradas por separado (una con el campo `NULL`, otra con el campo declarado) y exige que salgan valores DISTINTOS:

```rust
// crates/features/data/instrument-catalog/src/domain.rs
#[test]
fn resolve_terms_null_inherits_nonnull_overrides() {
    let override_terms = BrokerOverride { pip_size_override: None, default_spread: Some(150_000_000), ..Default::default() };
    let resolved = resolve_terms(&canonical, Some(&override_terms));
    assert_eq!(resolved.pip_size, canonical.canonical_pip_size, "NULL en el override debe heredar el canónico");
    assert_eq!(resolved.spread, Some(150_000_000), "un override no-NULL debe ganar sobre el canónico");
}
```

Si alguien invirtiera la prioridad por error (canónico siempre gana, o override siempre gana sin mirar si es `None`), esta única prueba ya lo atraparía porque mezcla los dos casos en la misma llamada.

### Por qué `session-catalog` nunca importa `instrument-catalog` (y qué hicimos en su lugar)

La especificación original de esta Story describía "sigue la cadena override de bróker → default de instrumento" como si `session-catalog` pudiera simplemente leer las tablas `broker_instrument_terms`/`instrument_master`. El problema: esas tablas son propiedad de OTRO crate (`instrument-catalog`), y ADR-0137 prohíbe que un feature crate dependa de otro — cada uno depende SOLO de `shared`. Si `session-catalog` importara `instrument-catalog`, cualquier cambio futuro en el esquema de instrumentos obligaría a recompilar y revisar session-catalog aunque su propio dominio (calendarios) no haya cambiado en nada — el acoplamiento se propaga en cascada.

La resolución (documentada como "premisa retada" en `crates/features/data/session-catalog/src/lib.rs`) es que la función de cadena de resolución reciba los DOS IDs YA resueltos como parámetros simples, en vez de ir a buscarlos ella misma:

```rust
// crates/features/data/session-catalog/src/domain.rs
pub fn resolve_session_template_id(
    broker_override_template_id: Option<&str>,
    instrument_default_template_id: Option<&str>,
) -> Option<String> {
    broker_override_template_id
        .map(str::to_string)
        .or_else(|| instrument_default_template_id.map(str::to_string))
}
```

`.or_else(...)` es el complemento de `.unwrap_or(...)`: en vez de devolver un valor por defecto ya calculado, ejecuta un cierre SOLO si el primer `Option` fue `None` — evita calcular `instrument_default_template_id.map(...)` cuando ni hace falta (aunque aquí el costo sea mínimo, es el idiomatismo correcto para "un segundo intento perezoso"). Quien SÍ tiene permiso de leer ambas tablas es `instrument-catalog` (porque las posee), así que la responsabilidad de "consultar y pasar los dos IDs" queda en la capa de composición (el futuro `data-import-wizard`, o el Bridge), nunca dentro de `session-catalog`. Esta función pura, sola, ya es 100% comprobable sin tocar ninguna base de datos — y así se prueba: con strings literales, sin ningún `SqlitePool` de por medio.

### Derivar una "forma de calendario" a partir de ventanas sueltas: un mapa de bits de una semana

El corazón cuantitativo de `session-catalog` es responder "¿esta plantilla es continua, tiene un hueco semanal, o tiene huecos diarios además del semanal?" a partir de una lista de ventanas (día/hora de inicio, día/hora de fin). La técnica es la misma que usaría cualquier calendario: representar la semana entera como una fila de casillas (una por minuto) y marcar cuáles están "abiertas":

```rust
// crates/features/data/session-catalog/src/domain.rs
const MINUTES_PER_WEEK: i32 = 1_440 * 7; // 10 080 minutos en una semana

let mut covered = vec![false; MINUTES_PER_WEEK as usize];
for window in windows {
    let start = week_offset_minutes(window.start_day, window.start_time_minutes);
    let end = week_offset_minutes(window.end_day, window.end_time_minutes);
    if end >= start {
        for minute in covered.iter_mut().take((end + 1) as usize).skip(start as usize) {
            *minute = true;
        }
    }
    // ... rama de "cruza la frontera de la semana", ver abajo
}
```

`week_offset_minutes` convierte "miércoles, 14:30" en un único número entero (posición 0 = lunes 00:00, posición 10079 = domingo 23:59) — así toda la aritmética de "¿qué está cubierto?" se reduce a operaciones sobre un `Vec<bool>` plano, sin tener que razonar sobre días y horas por separado. `covered.iter_mut().skip(start).take(end + 1 - start)` es la forma idiomática de Rust de decir "dame un sub-rango mutable del vector, del índice `start` al `end` inclusive" sin usar índices manuales (`covered[i] = true` en un bucle `for i in start..=end`) — ambas formas son válidas; esta evita un `usize`/`i32` mixto en cada iteración.

El caso interesante es Forex 24/5: la ventana declara "abre domingo 22:00, cierra el viernes SIGUIENTE a las 22:00" — en la semana lineal (lunes a domingo), el `end` (viernes ≈ minuto 7080) es MENOR que el `start` (domingo ≈ minuto 9660), porque la ventana atraviesa la frontera artificial "domingo 23:59 → lunes 00:00" que el array introduce (el calendario real es cíclico; el array, no). Ese caso se cubre en DOS tramos:

```rust
} else {
    // La ventana cruza la frontera lunes/domingo -- dos tramos.
    for minute in covered.iter_mut().skip(start as usize) { *minute = true; }       // domingo 22:00 -> fin del array
    for minute in covered.iter_mut().take((end + 1) as usize) { *minute = true; }   // inicio del array -> viernes 22:00
}
```

Una vez marcado el array, encontrar los "huecos" es contar corridas consecutivas de `false` (un bucle `while` que avanza el índice mientras la casilla esté cerrada, y anota cuántas casillas duró cada corrida). El número y tamaño de esas corridas es lo que clasifica la forma: cero huecos es `Continuous` (Cripto); exactamente un hueco es `WeeklyGap` (Forex); varios huecos, uno grande (el fin de semana) y varios cortos (una pausa diaria de mantenimiento), es `WeeklyGapPlusDailyMaintenance` (Futuros). El umbral que distingue "hueco grande" de "hueco corto" (`LARGE_GAP_THRESHOLD_MINUTES = 20 * 60`, 20 horas) es una elección deliberada y documentada en el código, no un número mágico sin explicar.

Una sutileza que casi se cuela como bug: si el HUECO MISMO cruza la frontera del array (toca tanto la posición 0 como la última), el algoritmo de conteo lo vería como DOS corridas separadas (una al final del array, otra al principio) cuando en realidad es UN SOLO hueco continuo en el calendario real (cíclico). La corrección:

```rust
// Fusiona el primer y último gap si el array "wrappea".
if gaps.len() >= 2 && !covered[0] && !covered[covered.len() - 1] {
    let first = gaps.remove(0);
    let last = gaps.pop().expect("gaps.len() >= 2 tras el remove anterior garantiza al menos un elemento");
    gaps.push(first + last);
}
```

Esta rama no la ejercita ningún criterio de aceptación directamente (ninguna de las tres plantillas semilla produce un hueco que toque ambos extremos del array), pero se dejó implementada y comentada a propósito: es la clase de bug que aparece meses después, con una plantilla nueva, si nadie lo hubiera anticipado — mejor pagarlo una vez, aquí, con el contexto fresco.

### Un default de dominio que NO es una verdad sellada (GOVERNANCE-4.7 en la práctica)

La plantilla semilla de Futuros necesitaba horas concretas de apertura/cierre/pausa de mantenimiento. El riesgo real: los calendarios de futuros varían por producto y mercado (un contrato de energía no cierra a la misma hora que un índice bursátil) — inventar un horario "porque hay que poner algo" y dejarlo sin marcar como tal habría quedado, con el tiempo, leído como un hecho verificado. La función que construye esas ventanas lo dice explícitamente, con constantes nombradas en vez de números sueltos:

```rust
// crates/features/data/session-catalog/src/domain.rs
/// **GOVERNANCE-4.7 -- valor representativo, NO verdad sellada:** ...
pub fn cme_globex_representative_windows() -> Vec<SessionWindow> {
    const DAILY_CLOSE_MINUTES: i32 = 959;   // 15:59 -- último minuto antes de la pausa de las 16:00.
    const DAILY_REOPEN_MINUTES: i32 = 975;  // 16:15 -- primer minuto tras la pausa de 15 minutos.
    // ...
}
```

La lección de proceso, más allá del código: cuando una tarea pide un dato de dominio específico (una hora, un porcentaje, un umbral) y no hay una fuente verificada a mano, la alternativa correcta no es "elegir un número razonable y seguir" en silencio — es declarar el número COMO default editable, con su razonamiento, para que quien lo audite después sepa que es un punto de partida, no una cita.

### Cuándo SÍ hace falta envolver varias escrituras en una transacción explícita

El resto del substrato (`plan_tier_quota`, `central_identity`) hace un solo `INSERT`/`UPDATE` por operación — una sola sentencia SQL ya es atómica por sí misma en SQLite, sin necesitar una transacción explícita. `session-catalog::create_template` es distinto: dar de alta una plantilla implica un `INSERT` en `session_template` MÁS un `INSERT` por cada una de sus ventanas semanales. Si el proceso muriera entre el primero y el tercero de cinco `INSERT`s de ventanas (el caso de Futuros), quedaría una plantilla con solo algunas de sus ventanas — un estado que ninguna lectura posterior podría distinguir de "a propósito tiene menos ventanas".

```rust
// crates/features/data/session-catalog/src/persistence.rs
let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
// ... INSERT session_template ...
for window in &new.windows {
    // ... INSERT session_template_window ...
}
tx.commit().await?;
```

Si cualquier `INSERT` dentro del bloque falla (por ejemplo, el `template_code` ya existe), la función retorna temprano CON el error, sin llamar `tx.commit()` — y Rust suelta (`drop`) la transacción sin confirmar, lo que SQLite interpreta como un `ROLLBACK` automático. La prueba que lo demuestra no solo verifica que el segundo intento falla — cuenta las filas de la tabla HIJA después del fallo, para confirmar que no quedó ninguna ventana huérfana del intento que no llegó a completarse:

```rust
// crates/features/data/session-catalog/src/persistence.rs
let window_count: i64 = sqlx::query("SELECT COUNT(*) FROM session_template_window").fetch_one(&pool).await?.get(0);
assert_eq!(window_count, 1, "el intento fallido NO debe dejar ventanas huérfanas -- la transacción hizo rollback completo");
```

`BEGIN IMMEDIATE` (en vez de `BEGIN DEFERRED`, el default de SQLite) toma el lock de ESCRITURA desde el primer momento de la transacción, no en la primera escritura real — esto evita que dos transacciones concurrentes, ambas `DEFERRED`, intenten "subir de categoría" de lectura a escritura al mismo tiempo y se bloqueen mutuamente (el mismo patrón que ya usan los ledgers append-only del proyecto, aunque aquí el motivo es distinto: no es una derivación *read-then-write* de una secuencia, es simple atomicidad multi-fila).

### El vocabulario JSON debe ser IDÉNTICO al vocabulario `CHECK` de la migración

Los enums que representan columnas con `CHECK (col IN (...))` (`AssetClass`, `CommissionModel`, `Weekday`, `CalendarFingerprint`) usan un atributo de `serde` para controlar exactamente cómo se ven en JSON:

```rust
// crates/features/data/instrument-catalog/src/domain.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssetClass { Forex, Future, Cfd, Crypto, Equity }
```

Sin ese atributo, `serde` serializaría la variante `Forex` como el string `"Forex"` (el nombre Rust tal cual) — pero el `CHECK` de la migración exige exactamente `'FOREX'` (mayúsculas). `rename_all = "SCREAMING_SNAKE_CASE"` le dice a `serde` que transforme CADA variante a mayúsculas con guiones bajos antes de serializar/deserializar, así que `AssetClass::Forex` se convierte en `"FOREX"` automáticamente, sin escribir una tabla de conversión a mano. Esto importa en dos direcciones a la vez: el JSON que llega por `--input` del harness CLI (`{"asset_class":"FOREX"}`) deserializa directo al enum, y el `.as_str()` que persiste en SQLite usa el mismo texto — las tres representaciones (Rust, JSON, SQL) quedan sincronizadas por construcción, en vez de por disciplina manual.

### Probar que un hash "depende del contenido" y no solo de sí mismo

La política del equipo prohíbe una prueba que compare `f(x)` contra `f(x)` (la misma llamada dos veces) para afirmar "el hash es estable" — eso es cierto de cualquier función pura, hasta una que siempre devolviera la cadena vacía. La prueba correcta compara DOS llamadas construidas por separado con las MISMAS entradas (confirma que es determinista) y una tercera con UNA entrada distinta (confirma que el resultado SÍ depende del contenido, no es un valor fijo disfrazado):

```rust
// crates/features/data/instrument-catalog/src/domain.rs
#[test]
fn instrument_master_audit_hash_depends_on_content_not_just_identity() {
    let hash_a = compute_instrument_master_audit_hash("id-1", 1_000, 1, None, "EURUSD", AssetClass::Forex, 10_000, 1_000, None);
    let hash_a_again = compute_instrument_master_audit_hash("id-1", 1_000, 1, None, "EURUSD", AssetClass::Forex, 10_000, 1_000, None);
    assert_eq!(hash_a, hash_a_again, "las mismas entradas, construidas por separado, deben producir el mismo hash");

    let hash_b = compute_instrument_master_audit_hash("id-1", 1_000, 1, None, "EURUSD", AssetClass::Forex, 20_000, 1_000, None);
    assert_ne!(hash_a, hash_b, "cambiar canonical_pip_size debe cambiar el hash resultante");
}
```

Si alguien reemplazara la implementación real por `"constante-fija".to_string()`, la primera aserción (`hash_a == hash_a_again`) seguiría pasando —  pero la segunda (`hash_a != hash_b`) fallaría de inmediato, porque una constante nunca cambia aunque el `canonical_pip_size` sí. Esa es la propiedad que de verdad importa proteger: que el sello de auditoría refleje el contenido real de la fila.

### Cuando el gate de mutación encuentra supervivientes: diseñar la prueba PARA la línea, no para el criterio

El primer corte de pruebas de esta Story pasaba en verde y cubría los 10 criterios de aceptación -- y aun así el gate de mutación (`cargo-mutants`, que corre el Tech-Lead, nunca el ingeniero) encontró 21 supervivientes entre las dos features. La lección es sobre la DIFERENCIA entre "probar el comportamiento que me pidieron" y "probar cada línea que decide ese comportamiento":

**Caso 1 — el struct de retorno con `..current.clone()`.** `update_master` construye su valor de retorno así:

```rust
// crates/features/data/instrument-catalog/src/persistence.rs
Ok(InstrumentMasterRow {
    updated_at: now_ns,
    canonical_tick_size: revision.canonical_tick_size,
    default_session_template_id: revision.default_session_template_id,
    ..current.clone()  // el resto de los campos viene de la fila VIEJA
})
```

Si un mutante borra la línea `updated_at: now_ns,`, el struct sigue compilando perfecto -- `..current.clone()` simplemente rellena ese campo con el valor VIEJO. La única prueba que existía comparaba `updated.row_version` y `updated.canonical_pip_size` contra el nuevo valor, pero NUNCA leía `updated.updated_at` ni `updated.default_session_template_id` del struct que la función ACABA de devolver (solo los verificaba tras releer de la base de datos, que sí tenía el valor correcto porque el `UPDATE` SQL en sí estaba bien). Dos verificaciones distintas -- "¿el SQL que se ejecutó fue correcto?" y "¿el struct que la función devuelve en memoria refleja ese mismo cambio?" -- y la Story original solo tenía la primera. La prueba nueva (`update_master_return_value_reflects_every_changed_field`) lee el valor DEVUELTO, no una relectura, y cambia TRES campos a la vez (incluyendo uno que pasa de `None` a `Some`, para que "borrar la línea" sea detectable incluso si el valor viejo también era razonable).

**Caso 2 — una guarda `&&` que ninguna prueba fuerza a distinguir de `||`.** El código que detecta un `instrument_id` duplicado es:

```rust
fn is_duplicate_instrument_id_violation(db_err: &(dyn sqlx::error::DatabaseError + 'static)) -> bool {
    db_err.is_unique_violation() && db_err.message().to_lowercase().contains("instrument_id")
}
```

La prueba original de "instrumento duplicado" SÍ dispara este código -- pero lo hace con una violación donde AMBAS mitades son verdaderas (es una violación UNIQUE, Y el mensaje sí menciona `instrument_id`). Con datos así, `&&` y `||` devuelven exactamente el mismo `true`: la prueba no puede distinguir entre los dos operadores, así que un mutante que cambiara uno por el otro sobrevive sin que nada lo note. La forma de matarlo es forzar el caso donde SOLO una mitad es verdadera: una violación UNIQUE de la PK `id` (que SÍ es `is_unique_violation() == true`) pero cuyo mensaje habla de `instrument_master.id`, NUNCA de `instrument_id` como substring. Con `&&`, esa combinación da `false` (correcto -- no es un duplicado de negocio); con `||`, daría `true` (incorrecto). Para fabricar esa violación de verdad (no simulada), hubo que extraer la lógica a una función standalone y provocar el error REAL insertando dos filas con el mismo `id` por SQL crudo -- exactamente el patrón que ya usaba `hybrid_data_transformer::persistence::is_transient_write_conflict` para su propia clasificación de errores.

**Caso 3 — aritmética interna que las pruebas "de forma" nunca ejercitan con precisión.** `fingerprint_distinguishes_crypto_forex_futures` prueba las TRES formas de las plantillas semilla -- pero son solo 3 puntos de un espacio mucho más grande de comportamiento interno (dónde exactamente empieza/termina un hueco, qué pasa en la frontera del umbral, qué pasa si dos huecos son igual de grandes). Cada una de esas preguntas necesita su PROPIO caso fabricado a mano, diseñado específicamente para que la respuesta CAMBIE si esa línea concreta estuviera mal -- por ejemplo, un hueco de EXACTAMENTE 1200 minutos (el valor del umbral) para forzar que `>=` se comporte distinto de `>` (con `>` el hueco no contaría como "grande" justo en ese punto). La lección general: una batería de pruebas que cubre bien los CASOS DE USO puede dejar la ARITMÉTICA interna casi sin ejercitar en sus bordes exactos -- son dos preguntas distintas ("¿funciona para los casos reales?" vs. "¿cada operador/constante de la fórmula importa de verdad?"), y el gate de mutación existe específicamente para hacer la segunda pregunta cuando la primera ya está satisfecha.

### Un mutante "que sobrevive" no siempre es una prueba faltante -- a veces es imposible de matar, y eso también hay que demostrarlo

El primer intento de matar el mutante de la línea 187 (`gaps.push(first + last)` mutado a `first * last`) usó la MISMA prueba que ya existía para la fusión cíclica (dos mitades de 700 minutos cada una). Esa prueba pasó -- pero el gate de mutación siguió reportando el mutante como VIVO. La razón: `700 + 700 = 1400` y `700 * 700 = 490 000` caen los DOS del mismo lado del umbral de 1200 minutos ("grande") -- el conteo de huecos grandes es idéntico en ambos casos, así que el `CalendarFingerprint` final tampoco cambia. Una prueba puede EJERCITAR una línea (pasar por ella durante la ejecución) sin DISCRIMINARLA (producir un resultado distinto si esa línea cambiara) -- son dos propiedades distintas, y el gate de mutación mide la segunda, no la primera. La corrección fue elegir números donde suma y producto caen en LADOS OPUESTOS del umbral (500+600=1100, corto; 500×600=300 000, grande) -- ahí sí, cambiar el operador cambia el conteo de huecos grandes y, con él, la clasificación final.

El segundo mutante de esa misma línea general (`>=` mutado a `<` en el filtro de huecos grandes, línea 196) resultó ser distinto en naturaleza: el Tech-Lead demostró matemáticamente que es un **mutante equivalente** -- ningún conjunto de huecos, por más que se fabrique, puede distinguir el código original del mutado. La prueba es una identidad algebraica: volteando `>=` a `<`, el conteo que antes contaba "huecos grandes" ahora cuenta "huecos cortos" (`len - large_gap_count`, porque cada hueco es una cosa o la otra, nunca ambas), y la condición completa de clasificación (`c >= 1 && c < len`) resulta ser SIMÉTRICA bajo ese cambio de variable -- da el mismo resultado para cualquier entrada. Cuando eso ocurre, la respuesta correcta NO es seguir intentando pruebas cada vez más creativas (ninguna puede funcionar, por definición) -- es demostrar la equivalencia por escrito, con el razonamiento completo, en un comentario junto a la línea. Esa nota es la evidencia de que el hueco de cobertura fue investigado y descartado con rigor, no simplemente ignorado.

## Trucos de Senior

- `Option<T>.and_then(...).unwrap_or(default)` es el modismo de una sola línea para "cadena de herencia campo a campo" — evita un `if let Some(x) = ... { x } else { default }` explícito, y encadena limpio cuando hay dos niveles de opcionalidad (¿existe el override? ¿ese campo del override trae valor?).
- Cuando una FK debe apuntar a una clave de negocio (`instrument_id`) en vez de a la PK técnica (`id`), SQLite lo permite siempre que la columna referenciada tenga `UNIQUE` — no hace falta que sea la PK. Es una herramienta legítima cuando el resto del sistema ya circula esa clave de negocio y forzar todo a UUID solo añadiría un paso de traducción innecesario en cada consulta.
- Antes de envolver una escritura en `BEGIN IMMEDIATE`, pregúntate si el riesgo real es "otro escritor concurrente" (ahí hace falta el lock de escritura inmediato, como en los ledgers append-only) o "varias sentencias que deben tener éxito juntas o fallar juntas" (ahí lo que importa es que el `commit()` sea el único punto de confirmación — el motivo es distinto aunque la sintaxis se parezca).
- Un algoritmo de "detección de huecos en un ciclo representado como array lineal" (aquí, una semana) casi siempre necesita una fusión especial en los dos extremos del array — es fácil olvidarla porque ningún caso de prueba "típico" la dispara, y solo aparece cuando el hueco real cruza exactamente la frontera artificial que introdujo la linealización.
- Cuando una tarea pide fijar un valor de dominio (una hora, un umbral, una convención) sin una fuente verificada a la mano, nombrar la constante y comentar explícitamente "default representativo, editable" cuesta una línea y evita que, meses después, alguien lo lea como un hecho contrastado en vez de un punto de partida razonable.
