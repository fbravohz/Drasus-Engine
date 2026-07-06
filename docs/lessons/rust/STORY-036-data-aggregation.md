# STORY-036 — Data Anonymization & Aggregation: lecciones de Rust

> **Story:** [STORY-036 — Data Anonymization & Aggregation (cimiento #9 del substrato de monetización)](../../execution/STORY-036-data-aggregation.md).
> **Archivos que esta Story produjo y que se citan abajo:** `migrations/0015_data_aggregation.sql`, `crates/shared/src/domain/data_aggregation.rs`, `crates/shared/src/persistence/data_aggregation.rs`, `crates/shared/src/orchestrator/data_aggregation.rs`, `crates/shared/src/public_interface.rs`, `crates/app/src/main.rs`.
> **Modo de Acompañamiento:** Docente (ADR-0122) — el ingeniero implementó cada bloque por su cuenta y este archivo consolida lo enseñado, con profundidad cero-conocimiento (ADR-0124).

## Concepto

### Qué es la privacidad diferencial y por qué el ruido debe ser DETERMINISTA con semilla inyectada

El negocio de este cimiento es vender **índices agregados** (sentimiento, régimen, fricción de bróker, correlación) que resumen a miles de usuarios sin que ninguno sea reconocible. El problema: si publicas la suma exacta de una métrica sobre una cohorte, un atacante que conozca a todos los contribuyentes menos uno puede restar y despejar el dato de ese uno. La suma "exacta" filtra al individuo.

La **privacidad diferencial** (differential privacy, ADR-0102) resuelve esto perturbando la respuesta agregada con una pizca de ruido aleatorio calibrado: el número que publicas está *cerca* del verdadero, lo bastante para ser útil estadísticamente, pero lo bastante *incierto* para que nadie pueda despejar a un individuo con certeza. El "mecanismo gaussiano" clásico suma a la respuesta una muestra de una distribución normal (la campana de Gauss) con media 0.

Aquí viene la lección de Rust y de FCIS (ADR-0002). Un "número aleatorio" en la mayoría de lenguajes se obtiene de una fuente de entropía del sistema operativo (`rand::thread_rng()` en Rust) — impredecible por diseño. Eso es exactamente lo que un Core **puro** NO puede hacer. Un Core FCIS obedece "mismo input → mismo output, bit a bit". Si `apply_differential_privacy` leyera entropía del sistema, la MISMA llamada con los MISMOS argumentos daría un resultado distinto cada vez, y eso rompe dos cosas a la vez:

- **No se puede probar.** Ningún `assert_eq!` sobrevive: no sabrías qué valor esperar.
- **No se puede auditar.** Nadie podría reproducir qué ruido se aplicó a un dato histórico para verificar que la anonimización fue correcta.

La solución es la misma que el proyecto ya usa para el tiempo: así como el `Clock` se inyecta (`DeterministicClock` en backtests), el azar se **siembra e inyecta**. Se recibe un `seed: u64` como parámetro y se construye un RNG determinista a partir de él:

```rust
// crates/shared/src/domain/data_aggregation.rs
pub fn apply_differential_privacy(raw_value_e8: i64, noise_level_e8: i64, seed: u64) -> i64 {
    let mut rng = StdRng::seed_from_u64(seed);

    // Dos uniformes independientes en [0, 1) -- u1 acotado lejos de 0 para
    // que ln(u1) nunca sea -infinito.
    let u1: f64 = rng.gen::<f64>().max(f64::MIN_POSITIVE);
    let u2: f64 = rng.gen::<f64>();

    // Box-Muller: transforma (u1, u2) uniformes en UNA muestra de una
    // normal estándar (media 0, desviación 1).
    let standard_normal = (-2.0_f64 * u1.ln()).sqrt() * (2.0_f64 * std::f64::consts::PI * u2).cos();

    let noise = standard_normal * noise_level_e8 as f64;
    let noisy_value = raw_value_e8 as f64 + noise;

    noisy_value.round() as i64
}
```

`StdRng::seed_from_u64(seed)` es la clave: es un generador **pseudo**aleatorio — sus números *parecen* aleatorios, pero están completamente determinados por la semilla. Con la misma semilla, produce siempre la misma secuencia. Por eso `let mut rng = ...` necesita `mut`: cada `rng.gen()` avanza el estado interno del generador (consume un número y prepara el siguiente), y mutar ese estado exige que la variable sea mutable en Rust — el compilador te obliga a declarar explícitamente que algo va a cambiar.

**Box-Muller** es el truco matemático que convierte lo que el RNG *sí* sabe hacer (números uniformes entre 0 y 1) en lo que necesitamos (una muestra de la campana de Gauss). Toma dos uniformes independientes y los combina con `sqrt(-2·ln(u1))·cos(2π·u2)`. El `u1.max(f64::MIN_POSITIVE)` es una guarda de borde: `ln(0)` es `-infinito` y arruinaría todo el cálculo produciendo `NaN`; acotar `u1` lejos de cero hace la función *total* (nunca explota) sin cambiar el resultado de ningún caso real, porque la probabilidad de que un uniforme continuo caiga exactamente en `0.0` es nula.

Sobre los `f64`: aquí SÍ se usa coma flotante, pero solo como cálculo **transitorio**. El logaritmo, el coseno y la raíz cuadrada viven en el mundo de los reales. Pero lo que sale por la puerta — lo que se persiste — se redondea a entero con `.round() as i64`, respetando la regla de enteros ×10⁸ (ADR-0141): ninguna columna de la base de datos es `REAL`. El `f64` es una herramienta de paso, nunca el producto final.

Las pruebas discriminantes que blindan el determinismo (`crates/shared/src/domain/data_aggregation.rs`):

```rust
#[test]
fn apply_differential_privacy_is_deterministic_for_the_same_seed() {
    let result_a = apply_differential_privacy(100_000_000_000, 5_000_000_000, 42);
    let result_b = apply_differential_privacy(100_000_000_000, 5_000_000_000, 42);
    assert_eq!(result_a, result_b);
}

#[test]
fn apply_differential_privacy_differs_from_the_raw_value() {
    let raw = 100_000_000_000;
    let noisy = apply_differential_privacy(raw, 5_000_000_000, 42);
    assert_ne!(noisy, raw); // privacidad real: el ruido sí alteró el valor
}
```

La primera prueba SÓLO puede pasar si el RNG está sembrado — con `thread_rng()` los dos resultados diferirían y `assert_eq!` fallaría. La segunda garantiza que el ruido no es cosmético: el valor publicado difiere del crudo, que es todo el punto de la privacidad. Hay una tercera (`apply_differential_privacy_differs_across_different_seeds`) que confirma que la semilla realmente participa del cálculo y no es un parámetro decorativo ignorado.

### Qué es k-anonimato y por qué una cohorte pequeña se SUPRIME

El ruido protege el valor, pero hay un segundo agujero: el **tamaño de la cohorte**. Si publicas un agregado que resume a 2 personas, por más ruido que le pongas, un agregado "de 2" es casi un dato individual disfrazado. El **k-anonimato** exige que cada agregado publicado esté respaldado por al menos `k` contribuyentes (`MIN_COHORT_SIZE`) — así ningún individuo destaca dentro del grupo. Es un invariante FIJO, no configurable por el operador: es una promesa de privacidad, no una perilla de negocio.

La regla vive en dos funciones puras. La primera decide el borde; la segunda la aplica para no construir jamás un agregado prohibido:

```rust
// crates/shared/src/domain/data_aggregation.rs
pub fn meets_k_anonymity(cohort_size: i64, min_cohort: i64) -> bool {
    cohort_size >= min_cohort
}

pub fn aggregate_index(
    covered_values_e8: &[i64],
    index_type: IndexType,
    time_window: &str,
    channel: Channel,
    min_cohort: i64,
    noise_level_e8: i64,
    seed: u64,
) -> Option<AggregatedIndex> {
    let cohort_size = covered_values_e8.len() as i64;

    // Guardarraíl de k-anonimato -- PRIMERO, antes de calcular nada más.
    if !meets_k_anonymity(cohort_size, min_cohort) {
        return None;
    }

    let raw_sum_e8: i64 = covered_values_e8.iter().sum();
    let metric_value_e8 = apply_differential_privacy(raw_sum_e8, noise_level_e8, seed);

    Some(AggregatedIndex { index_type, time_window: time_window.to_string(), cohort_size, noise_level_e8, metric_value_e8, channel })
}
```

Dos detalles de Rust que enseñan cómo el lenguaje hace *imposible* el error, no solo improbable:

**`Option<AggregatedIndex>` como valor de retorno.** En muchos lenguajes "suprimir" se modelaría devolviendo un objeto con un flag `suppressed = true`, o `null`, o lanzando una excepción — todas formas en las que un llamador distraído puede olvidar el chequeo y persistir basura. Rust ofrece `Option<T>`: el tipo dice, en la firma misma, "esta función a veces no produce un índice". El `None` no es un objeto vacío que se pueda persistir por accidente — es la ausencia literal del `AggregatedIndex`. Para sacar el valor, el llamador está *obligado* por el compilador a manejar el caso `None`. La supresión no es un dato que viaja: es la nada. Un índice con cohorte insuficiente ni siquiera se llega a construir.

**El orden importa: la guarda va PRIMERO.** El `if !meets_k_anonymity(...) { return None; }` está antes de sumar y antes de aplicar ruido. No se hace trabajo sobre una cohorte que de todas formas se va a rechazar, y — más importante — es imposible que un refactor futuro "se salte" la verificación, porque es la puerta de entrada de la función.

Las pruebas de borde exacto (`crates/shared/src/domain/data_aggregation.rs`) fijan la frontera con precisión quirúrgica: en el mínimo publica, uno menos suprime.

```rust
#[test]
fn aggregate_index_publishes_at_the_exact_cohort_boundary() {
    let covered = vec![100_000_000_000_i64; 5];
    let result = aggregate_index(&covered, IndexType::Sentiment, "2026-W27", Channel::Internal, 5, 1_000_000, 42);
    assert!(result.is_some()); // cohorte de exactamente 5, min_cohort 5 -> publica
}

#[test]
fn aggregate_index_suppresses_one_below_the_cohort_boundary() {
    let covered = vec![100_000_000_000_i64; 4];
    let result = aggregate_index(&covered, IndexType::Sentiment, "2026-W27", Channel::Internal, 5, 1_000_000, 42);
    assert!(result.is_none()); // cohorte de 4 -> None, no una fila
}
```

La comparación es `>=`, no `>`: por eso "en el mínimo" (5 ≥ 5) publica y "uno menos" (4 ≥ 5 es falso) suprime. Un `>` en lugar de `>=` habría movido la frontera una posición y estas dos pruebas lo cazarían de inmediato. Como refuerzo de cinturón-y-tirantes, la migración `0015_data_aggregation.sql` añade un `CHECK (cohort_size > 0)` a nivel de base de datos, y hay una prueba (`persistence::data_aggregation::tests::database_check_rejects_non_positive_cohort_size`) que lo verifica: aunque el Core fallara, la BD rechazaría una fila con cohorte no positiva.

### Por qué la topología se guarda hasheada (SHA-256) y nunca cruda

La "topología" de una estrategia es su fórmula/firma — por ejemplo `RSI(14)+MACD(12,26,9)`. Es propiedad intelectual del usuario: si Drasus la transmitiera en claro hacia un agregado o hacia un tercero, cualquiera podría hacer ingeniería inversa de la ventaja competitiva del trader (ADR-0102). Pero a veces la topología SÍ se necesita para *agrupar* — juntar en una misma cohorte a todos los que usan la misma familia de estrategia. La tensión es: necesito comparar topologías sin revelar ninguna.

El **hash unidireccional** resuelve exactamente eso. SHA-256 convierte cualquier texto en una huella de 64 caracteres hex de la que es inviable volver al original. Dos topologías idénticas producen el mismo hash (sirven para agrupar); del hash no se puede reconstruir la fórmula (no revela nada):

```rust
// crates/shared/src/domain/data_aggregation.rs
pub fn hash_strategy_topology(topology: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(topology.as_bytes());
    encode_hex(&hasher.finalize())
}
```

Es exactamente el mismo patrón de `hash_api_credential` del cimiento #8: la única operación que se hace con el dato sensible es convertirlo de inmediato en algo irreversible. El texto crudo entra por el parámetro `topology: &str` (una referencia prestada — la función *lee* el texto pero no toma posesión de él ni lo guarda) y sale de alcance apenas termina la línea. En el orquestador, el crudo se hashea y se descarta dentro del mismo bucle:

```rust
// crates/shared/src/orchestrator/data_aggregation.rs
if let Some(raw) = &event.raw_topology {
    let _ = hash_strategy_topology(raw);
}
```

El `let _ =` es idiomático de Rust: "ejecuta esto por su efecto, pero descarta explícitamente el resultado". Aquí el punto es que el texto crudo (`raw`) *sobreviva lo menos posible* — se hashea y se suelta. El guardarraíl del proyecto (ADR-0093/0102, "datos crudos nunca salen") exige que el crudo nunca SOBREVIVA hasta la persistencia, no que su hash se use en un lugar concreto. Por eso el struct `AggregatedIndex` que se persiste ni siquiera tiene un campo de topología: la prueba `aggregated_index_json_never_leaks_raw_identifiable_data` fija que el JSON de salida solo lleva seis claves (`channel`, `cohort_size`, `index_type`, `metric_value_e8`, `noise_level_e8`, `time_window`) — nada de balances crudos, topología ni IDs de usuario.

### Por qué el agregado consulta el consentimiento REAL de #5 antes de sumar cada dato

La ley (GDPR) y el diseño (ADR-0143) coinciden: NUNCA se agrega un dato de un usuario sin un consentimiento vigente que lo cubra. Un usuario puede no haber aceptado la versión actual del contrato, o haber marcado un opt-out explícito para la agregación. Ese dato debe quedar *fuera* de la suma — no basta con un stub que "casi siempre" dice que sí.

El orquestador consume el `consent_out` **real** del cimiento #5 (`consent-registry`) evento por evento, con default-deny: si no hay cobertura, se excluye.

```rust
// crates/shared/src/orchestrator/data_aggregation.rs
let mut covered_values_e8 = Vec::new();
for event in events {
    if let Some(raw) = &event.raw_topology {
        let _ = hash_strategy_topology(raw);
    }

    let verdict = resolve_consent_verdict(
        pool,
        clock,
        &event.owner_id,
        DATA_AGGREGATION_CONSENT_DATA_TYPE,
        &config.consent_version,
    )
    .await?;

    if verdict.is_covered() {
        covered_values_e8.push(event.metric_e8);
    }
}
```

Aquí conviven varias piezas importantes:

- **`resolve_consent_verdict` es la función real de #5**, no una copia local. El orquestador de #9 depende del `consent_out` que ya existe en `crates/shared/src/orchestrator/consent_registry.rs`. Reusar el veredicto real (en vez de reimplementar la lógica de cobertura) es lo que hace que el gate sea de verdad y no decorativo — si mañana cambia la regla de consentimiento, #9 la hereda sin tocar nada.
- **`.await?` — dos operadores en dos caracteres.** `resolve_consent_verdict` es `async` porque lee la base de datos (I/O); `.await` suspende el hilo hasta que la consulta responde, sin bloquear el runtime de Tokio. El `?` es la propagación de errores de Rust: si la consulta falla, la función entera retorna ese error de inmediato (envuelto en `DataAggregationError::Consent` por el `#[from]` del `thiserror`), en vez de continuar con datos corruptos. Una línea que en otros lenguajes serían cinco de `try/catch`.
- **Solo los cubiertos entran al `Vec`.** El `if verdict.is_covered()` es el gate: un evento sin cobertura simplemente no se hace `push`. Cuando el bucle termina, `covered_values_e8` contiene EXACTAMENTE los valores permitidos — y su `.len()` es la cohorte real que verá `aggregate_index`. Esto encadena las tres protecciones: el consentimiento filtra *quién* entra, el k-anonimato exige que los que quedan sean suficientes, y el ruido protege el valor final.

La prueba que demuestra que el gate es real y no un stub (`crates/shared/src/orchestrator/data_aggregation.rs`) muestra el opt-out reduciendo la cohorte hasta hacerla insuficiente:

```rust
#[tokio::test]
async fn run_aggregation_excludes_events_with_explicit_optout() {
    // owner-1 y owner-2 cubiertos; owner-3 con opt-out explícito.
    cover_owner(&pool, &clock, "owner-1", "v2", false).await;
    cover_owner(&pool, &clock, "owner-2", "v2", false).await;
    cover_owner(&pool, &clock, "owner-3", "v2", true).await; // opt-out

    // min_cohort = 3, pero owner-3 queda fuera -> cohorte cubierta real = 2 -> se suprime.
    let outcome = run_aggregation(&pool, &clock, &events, &base_config(Channel::Internal, false))
        .await.expect("la corrida debe tener éxito");

    assert!(matches!(outcome, AggregationOutcome::SuppressedByCohortSize));
}
```

Si el opt-out se ignorara (o si el gate fuera un stub que siempre cubre), la cohorte sería 3, alcanzaría el mínimo y publicaría — y esta prueba fallaría. Que suprima *demuestra* que el opt-out real recortó la cohorte. Hay pruebas hermanas para el caso sin ningún consentimiento (`run_aggregation_excludes_events_without_any_consent`, el default-deny) y para el camino feliz con todos cubiertos (`run_aggregation_includes_covered_events_and_publishes_with_sufficient_cohort`).

## Trucos de Senior

- **`Option<T>` como "el resultado a veces no existe", en vez de un flag booleano.** `aggregate_index` devuelve `Option<AggregatedIndex>`: la supresión por k-anonimato es un `None` que el compilador obliga a manejar, no un objeto-con-flag que un llamador distraído puede persistir. El tipo *es* la documentación de que la operación puede no producir nada.

- **`let _ = f(x);` para "ejecuta por el efecto, descarta el valor".** En `run_aggregation`, `let _ = hash_strategy_topology(raw);` deja claro al lector (y al `clippy`) que el resultado se descarta a propósito. El objetivo real es que el texto crudo se consuma y muera cuanto antes.

- **`#[from]` de `thiserror` + `?` = plomería de errores gratis.** `DataAggregationError` declara `Consent(#[from] ConsentRepositoryError)` y `Persistence(#[from] AggregatedIndexRepositoryError)`; gracias a eso, un simple `?` sobre cualquier operación de esos dos subsistemas se envuelve solo en la variante correcta del error del orquestador, sin `match` ni `map_err` manuales.

- **`f64` transitorio + `.round() as i64` como frontera.** El cálculo del ruido gaussiano vive en coma flotante (logaritmos, cosenos), pero el redondeo a entero ×10⁸ en la última línea es la frontera dura: dentro, reales; fuera (persistido), enteros exactos. El `f64` es una herramienta de paso, nunca cruza a la base de datos (ADR-0141).

- **`Box<AggregatedIndexRow>` en una variante de enum grande.** `AggregationOutcome::Published` envuelve la fila en un `Box` (puntero al heap) porque `AggregatedIndexRow` es mucho más grande que las otras dos variantes vacías. Sin la indirección, `clippy::large_enum_variant` señala que *toda* instancia del enum — incluidas `SuppressedByCohortSize` y `ExternalChannelDisabled`, que no cargan datos — pagaría el tamaño de la variante más gorda. El `Box` deja el enum pequeño y mueve el peso al heap solo cuando de verdad hay una fila.

- **`StdRng::seed_from_u64(seed)` es el gemelo de `DeterministicClock`.** El patrón de "inyectar la fuente de no-determinismo en vez de leerla del sistema" es idéntico para el tiempo (Clock) y para el azar (RNG sembrado). Cuando veas un Core que necesita algo impredecible, la respuesta FCIS siempre es: recíbelo por parámetro, no lo generes.
