# STORY-031 — Consent Registry / Registro de Consentimiento ToS: lecciones de Rust

> **Story:** [STORY-031 — Consent Registry / Registro de Consentimiento ToS (cimiento #5 del substrato de monetización)](../../execution/STORY-031-consent-registry.md).
> **Archivos que esta Story produjo y que se citan abajo:** `migrations/0011_consent_registry.sql`, `crates/shared/src/domain/consent_registry.rs`, `crates/shared/src/persistence/consent_registry.rs`, `crates/shared/src/orchestrator/consent_registry.rs`, `crates/shared/src/public_interface.rs`, `crates/app/src/main.rs`.
> **Modo de Acompañamiento:** Docente (ADR-0122) — el ingeniero implementó cada bloque por su cuenta y este archivo consolida lo enseñado, con profundidad cero-conocimiento (ADR-0124).

## Concepto

### Qué es un "registro de consentimiento" y por qué es la columna vertebral legal (GDPR)

Cuando un producto de software usa o vende los datos de una persona, la ley (GDPR en Europa, y equivalentes en otras jurisdicciones) exige poder demostrar, con fecha y versión exactas, que esa persona **aceptó explícitamente** que sus datos se usaran de esa forma. No basta con que el usuario "probablemente estuvo de acuerdo" — la carga de la prueba es del proveedor del software: si no puedes mostrar el evento de aceptación, legalmente es como si nunca hubiera ocurrido.

Un **registro de consentimiento** es exactamente ese archivo de pruebas: cada vez que un usuario acepta un Término de Servicio (ToS) o cambia una preferencia de privacidad ("no quieras que mis datos se usen para X"), se graba un evento con fecha. Ese registro es lo único que hace legal el resto del negocio de datos de Drasus Engine: el firehose gratuito (todo el trabajo del usuario fluye al proveedor, ADR-0143) y la venta de índices agregados anónimos (`data-aggregation`, #9) solo pueden operar sobre un usuario concreto si su registro de consentimiento dice "cubierto" para ese tipo de dato. Por eso la Feature lo describe como "columna vertebral legal": si este cimiento falla o se salta, todo lo que depende de él se vuelve ilegal, sin importar qué tan bien funcione el resto del sistema.

En código, el corazón de la Story es la función [`resolve_coverage`](../../../crates/shared/src/domain/consent_registry.rs):

```rust
// crates/shared/src/domain/consent_registry.rs
pub fn resolve_coverage(
    consent_state: Option<&ConsentState>,
    data_type: &str,
    current_version: &str,
) -> ConsentVerdict {
    let state = match consent_state {
        Some(state) => state,
        None => return ConsentVerdict::NotCovered(NotCoveredReason::NoConsent),
    };

    if needs_reacceptance(&state.accepted_version, current_version) {
        return ConsentVerdict::NotCovered(NotCoveredReason::StaleVersion);
    }

    if state.optout_map.get(data_type).copied().unwrap_or(false) {
        return ConsentVerdict::NotCovered(NotCoveredReason::OptedOut);
    }

    ConsentVerdict::Covered
}
```

Es una función pura: no toca la base de datos, no lee el reloj, no tiene aleatoriedad. Recibe el estado ya cargado (`consent_state`) y devuelve un veredicto. Cualquier otra feature del substrato (`data-aggregation`, el firehose) que quiera saber "¿puedo usar este dato de este usuario?" pasa por aquí — nunca decide por su cuenta ni accede directo a la tabla `consent_records` (ADR-0137: acceso cross-feature solo por puerto tipado).

### Por qué es append-only, y cómo se modela un estado MUTABLE (los opt-outs) sobre una tabla INMUTABLE

"Append-only" ("solo agregar") significa que ninguna fila de la tabla `consent_records` se edita ni se borra jamás — solo se insertan filas nuevas. Esto es deliberado por una razón legal, no solo técnica: si el registro de consentimiento pudiera editarse, un proveedor deshonesto podría "arreglar" retroactivamente su historial para simular que siempre tuvo el consentimiento correcto. Un registro append-only, con cada fila encadenada por hash a la anterior (`audit_chain_hash`), hace esa manipulación detectable: cambiar cualquier fila histórica rompería la cadena de hashes de todas las filas posteriores.

Pero aquí aparece una tensión real: **los opt-outs del usuario SÍ cambian con el tiempo** (hoy acepta compartir su dato de "agregación", mañana se arrepiente y lo desactiva). ¿Cómo se modela algo que cambia sobre una tabla donde nada se edita?

La respuesta es **event-sourcing con snapshot completo**: en vez de guardar solo "lo que cambió" (un delta), cada fila nueva guarda el **estado COMPLETO** resultante después del cambio — la versión de ToS aceptada Y el mapa entero de opt-outs, no solo la clave que se tocó. El "estado vigente" de un usuario deja de ser algo que hay que reconstruir recorriendo todo su historial (un *fold*): es, literalmente, la última fila que tiene para ese `owner_id`. "Última fila gana."

La función que hace esta fusión es [`apply_consent_action`](../../../crates/shared/src/domain/consent_registry.rs) — es EL punto de modelado crítico de esta Story:

```rust
// crates/shared/src/domain/consent_registry.rs
pub fn apply_consent_action(previous: Option<&ConsentState>, input: &ConsentActionInput) -> ConsentState {
    // Punto de partida: el mapa previo completo, o vacío si es la primera acción.
    let mut optout_map = previous
        .map(|state| state.optout_map.clone())
        .unwrap_or_default();

    // Sobrescribe SOLO las claves que trae esta acción -- el resto del
    // mapa previo queda intacto (snapshot completo, no delta parcial).
    for (data_type, opted_out) in &input.optout_changes {
        optout_map.insert(data_type.clone(), *opted_out);
    }

    let accepted_version = input.tos_version.clone().unwrap_or_else(|| {
        previous.map(|state| state.accepted_version.clone()).unwrap_or_default()
    });

    ConsentState { accepted_version, optout_map }
}
```

Nótese que esta función es pura: toma el estado anterior (o `None`) y la acción nueva, y devuelve el estado siguiente — sin tocar la base de datos. Quien SÍ toca la base de datos es [`ConsentRepository::record_action`](../../../crates/shared/src/persistence/consent_registry.rs), que hace tres cosas en orden: (1) carga la última fila del dueño con `load_latest_for_owner`, (2) le pide a `apply_consent_action` el snapshot siguiente, y (3) `INSERT` esa fila nueva — nunca un `UPDATE`.

La prueba discriminante que ejercita exactamente este comportamiento:

```rust
// crates/shared/src/persistence/consent_registry.rs
#[tokio::test]
async fn changing_an_optout_inserts_a_new_row_and_keeps_the_previous_one_intact() {
    // ... acepta v2 con "aggregation": false ...
    // ... cambia "aggregation" a true ...

    assert_eq!(second.event_sequence_id, first.event_sequence_id + 1, "el cambio debe insertar una fila NUEVA");
    assert_ne!(second.id, first.id, "no debe reusar el id de la fila anterior");

    let chain = repo.load_chain().await.expect("cargar cadena completa");
    assert_eq!(chain.len(), 2, "ambas filas deben seguir presentes -- append-only");
    assert_eq!(chain[0].optout_map.get("aggregation"), Some(&false), "la fila original no debe mutar");
    assert_eq!(chain[1].optout_map.get("aggregation"), Some(&true), "la fila nueva trae el cambio");
}
```

Y a nivel de base de datos, la garantía es doble (defensa en profundidad, mismo patrón que `audit_events`/`usage_records`): aunque el código Rust nunca ofrece un método de `update`, la migración además pone un guardián en el motor:

```sql
-- migrations/0011_consent_registry.sql
CREATE TRIGGER IF NOT EXISTS trg_consent_records_no_update
BEFORE UPDATE ON consent_records
BEGIN
    SELECT RAISE(ABORT, 'consent_records is append-only: UPDATE is forbidden');
END;
```

### Por qué el default es negar (nunca se asume consentimiento)

`resolve_coverage` tiene tres "puertas" en orden, y la primera es la más importante: si `consent_state` es `None` (el usuario no tiene NINGÚN evento de consentimiento registrado), el veredicto es `NotCovered(NoConsent)` — nunca `Covered`. Esto parece obvio escrito así, pero es la clase de bug que en un sistema real se cuela fácilmente: si alguien escribiera `consent_state.map(|s| resolve(...)).unwrap_or(Covered)` en vez de `unwrap_or(NotCovered(NoConsent))`, un usuario nuevo sin ningún registro quedaría "cubierto" por accidente, y el firehose podría empezar a usar sus datos antes de que aceptara nada. La regla de negocio (`docs/features/consent-registry.md`: "NUNCA se asume consentimiento por defecto") se traduce literalmente en la primera línea de la función: el camino de "no tengo información" y el camino de "tengo información y dice que no" terminan en el mismo lugar (`NotCovered`), y el único camino a `Covered` exige pasar TODAS las puertas explícitamente.

La prueba que lo cierra:

```rust
// crates/shared/src/domain/consent_registry.rs
#[test]
fn resolve_coverage_denies_by_default_without_any_consent() {
    assert_eq!(
        resolve_coverage(None, "aggregation", "v2"),
        ConsentVerdict::NotCovered(NotCoveredReason::NoConsent)
    );
}
```

### Qué es la re-aceptación forzada por versión

Cuando el proveedor cambia los Términos de Servicio (por ejemplo, añade un nuevo uso de los datos que antes no existía), el consentimiento que el usuario dio para la versión VIEJA no puede seguir cubriendo la versión NUEVA — legalmente, aceptar "v1" no es lo mismo que aceptar "v2" aunque el texto se parezca. `REACCEPT_ON_VERSION_CHANGE` es un parámetro **FIJO** (no configurable, `docs/features/consent-registry.md` "Parámetros Configurables"): siempre que la versión aceptada difiera de la vigente, se exige re-aceptación, sin excepción ni umbral de tolerancia.

La función que lo decide es deliberadamente la comparación más simple posible:

```rust
// crates/shared/src/domain/consent_registry.rs
pub fn needs_reacceptance(accepted_version: &str, current_version: &str) -> bool {
    accepted_version != current_version
}
```

Comparación de texto exacta — `"v2"` y `"v2.0"` son versiones DISTINTAS a propósito. Esta función no intenta ser "inteligente" (no interpreta números de versión ni hace comparaciones semánticas) porque cualquier heurística de "casi igual" introduciría un vector para que una versión con más recolección de datos se cuele como "la misma" bajo un consentimiento viejo. Dentro de `resolve_coverage`, una versión obsoleta invalida la cobertura para **TODO** tipo de dato consultado, no solo para el tipo que cambió en el nuevo ToS — la prueba `resolve_coverage_stale_version_denies_every_data_type` ejercita justamente eso.

### Por qué el append va en una transacción con reintento (y por qué el `UNIQUE` no basta) — reapertura DEBT-001

La primera versión de `record_action` hacía tres cosas en sentencias sueltas: leía el `MAX(event_sequence_id)` de la tabla, calculaba "el siguiente es MAX+1", y luego insertaba esa fila. Con un solo escritor esto es correcto. Con DOS escritores a la vez, es un defecto latente. Imagina dos tareas A y B corriendo en paralelo: ambas leen `MAX = 4` (todavía nadie insertó el 5), ambas calculan "me toca el 5", ambas intentan insertar la fila con `event_sequence_id = 5`. SQLite deja pasar la primera; la segunda choca contra la restricción `UNIQUE` de esa columna y su `INSERT` falla. En la versión vieja, ese fallo se propagaba como error y **el evento de B se perdía** — un consentimiento que el usuario dio y que legalmente teníamos que registrar, desaparecido por una carrera.

Es tentador pensar "pero el `UNIQUE` atrapó el problema, ¿no?". No: el `UNIQUE` es **cinturón-y-tirantes, no el guardián primario**. Un cinturón de seguridad no evita el choque, solo evita que salgas volando; el `UNIQUE` no evita la carrera, solo evita que se persista una secuencia duplicada corrupta. El evento perdedor sigue perdido. El guardián de verdad tiene que impedir que la carrera ocurra en primer lugar. Ese guardián es la transacción `BEGIN IMMEDIATE`:

```rust
// crates/shared/src/persistence/consent_registry.rs — try_record_action_once
let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
// ... leer estado del dueño, leer MAX(event_sequence_id), INSERT ...
tx.commit().await?;
```

Una transacción normal en SQLite empieza con `BEGIN` "DEFERRED": no toma ningún lock hasta que realmente escribe. Si dos transacciones DEFERRED leen primero y luego intentan subir a escritura a la vez, chocan (o incluso se interbloquean). `BEGIN IMMEDIATE` es distinto: toma el **lock de escritura de la base de datos en el instante de abrir**, antes de leer nada. Eso significa que mientras la tarea A tiene su transacción abierta, la tarea B no puede ni empezar la suya — espera. Cuando A hace `commit()` (y recién ahí su fila `event_sequence_id = 5` se vuelve visible), B por fin abre la suya, lee `MAX = 5`, calcula 6, e inserta 6. Sin colisión. El "leer el MAX y luego insertar" pasa de ser dos operaciones separadas (donde alguien puede colarse en medio) a ser un bloque atómico e indivisible frente a otros escritores.

Dos piezas acompañan a la transacción:

1. **`busy_timeout` (ADR-0141 R2)**, configurado en `crates/shared/src/persistence/pool.rs`: cuando B intenta abrir su `BEGIN IMMEDIATE` y A todavía tiene el lock, SQLite no falla de inmediato con "database is locked" — espera hasta 5 segundos a que el lock se libere. Como cada transacción de este ledger solo hace un par de lecturas y un `INSERT`, el lock se libera en microsegundos y B casi nunca tiene que esperar de verdad.

2. **Reintento acotado**, en `record_action`: si aun con `busy_timeout` la contención fuera tan extrema que el lock no se libera a tiempo (o si, por cinturón-y-tirantes, ocurriera una colisión de `event_sequence_id`), se reintenta hasta `MAX_RECORD_ATTEMPTS` (5) veces, **re-derivando** el MAX en cada intento. Lo que jamás se hace es tragarse el error en silencio: si se agotan los reintentos, se devuelve un error tipado explícito (`ConsentRepositoryError::WriteContention { attempts }`) para que quien llame decida — nunca se pierde el evento sin que nadie se entere.

```rust
// crates/shared/src/persistence/consent_registry.rs — is_transient_write_conflict
let message = db.message().to_lowercase();
if message.contains("database is locked") || message.contains("database table is locked") {
    return true; // reintentar: otro escritor tenía el lock
}
db.is_unique_violation() && message.contains("event_sequence_id") // colisión de secuencia: reintentar
```

La prueba que blinda todo esto (`concurrent_record_actions_persist_every_event_without_gaps_or_lost_rows`) lanza 16 tareas en paralelo con `#[tokio::test(flavor = "multi_thread")]` sobre una BD en archivo (nunca `:memory:`, porque ahí cada conexión sería una base de datos distinta y no habría concurrencia real). Afirma tres cosas que solo se cumplen si el read-then-write es atómico: (a) las 16 filas se persistieron —ninguna se perdió—, (b) los `event_sequence_id` son exactamente `1..=16` sin huecos ni duplicados, y (c) la cadena de `audit_chain_hash` queda íntegra y cada `audit_hash` es recomputable. Se verificó empíricamente que la prueba SÍ se cae si se quita la transacción: la variante sin transacción falla con `UNIQUE constraint failed: consent_records.event_sequence_id` — exactamente el evento perdido que la regla de "Atomicidad de ledgers append-only" (DEBT-001) existe para prevenir.

## Trucos de Senior

### `BTreeMap` en vez de `HashMap` cuando el orden de serialización debe ser reproducible

El mapa de opt-outs (`{tipo_dato: bool}`) se serializa a JSON dos veces con implicaciones distintas: una vez para persistirlo en la columna `optout_map`, y otra vez porque ese mismo JSON entra al cálculo del `audit_hash` encadenado. Si el mapa fuera un `HashMap<String, bool>`, el orden en que Rust recorre sus claves al serializar NO está garantizado entre ejecuciones distintas del proceso — `HashMap` usa una semilla aleatoria (`RandomState`) como protección contra ataques de colisión de hash, así que el mismo contenido lógico (`{"aggregation": false, "firehose": true}`) podría serializar como `{"aggregation":false,"firehose":true}` en una ejecución y `{"firehose":true,"aggregation":false}` en otra. Dos strings JSON distintos para el MISMO estado producirían dos `audit_hash` distintos — rompiendo el invariante de determinismo del proyecto (ADR-0002/0004: "mismo input → mismo output, bit a bit").

`BTreeMap<String, bool>` resuelve esto gratis: ordena sus claves siempre alfabéticamente, así que `serde_json::to_string` produce el mismo JSON sin importar cuántas veces corra el proceso. El cambio de tipo (`HashMap` → `BTreeMap`) es, en este caso, la decisión de diseño completa — no hace falta escribir ningún serializador ordenado a mano.

### Enums con datos usando `#[serde(tag = "...", content = "...")]` ("adjacently tagged")

`ConsentVerdict` tiene una variante sin datos (`Covered`) y una variante con datos (`NotCovered(NotCoveredReason)`). Serde ofrece varias formas de serializar un enum así; la usada aquí es "adjacently tagged":

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "verdict", content = "reason", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConsentVerdict {
    Covered,
    NotCovered(NotCoveredReason),
}
```

Esto produce `{"verdict":"COVERED"}` para la variante sin datos (la clave `content` se omite del todo cuando no hay nada que poner ahí) y `{"verdict":"NOT_COVERED","reason":"OPTED_OUT"}` para la variante con datos. Es la forma idiomática de Serde de resolver "un enum donde algunas variantes traen carga y otras no" sin escribir un `impl Serialize` manual — y es exactamente el shape de JSON que el CLI de verificación necesita exponer.

### Guardarraíl ADR-0093 como prueba explícita, no como confianza en el diseño

Que un tipo "no debería" tener secretos no es una garantía verificable — un campo se puede añadir por error en un refactor futuro sin que nadie lo note hasta que ya se filtró algo. Por eso el test `consent_verdict_json_never_leaks_secret_fields` no se limita a comprobar los dos casos felices: además congela la lista EXACTA de claves permitidas (`["verdict"]` / `["reason", "verdict"]`) y recorre una lista de patrones prohibidos (`"password"`, `"api_key"`, IPs privadas, etc.) contra el JSON serializado. Si alguien añadiera un campo nuevo a `ConsentVerdict` mañana, esta prueba se caería inmediatamente con un mensaje que dice exactamente qué clave nueva apareció — convirtiendo un guardarraíl de diseño en un guardarraíl ejecutable.
