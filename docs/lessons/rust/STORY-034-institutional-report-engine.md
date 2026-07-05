# Lección Rust — STORY-034: Institutional Report Engine

> Story: [STORY-034 — Institutional Report Engine](../../execution/STORY-034-institutional-report-engine.md)
> Modo: Docente (ADR-0122). Profundidad cero-conocimiento: no se da por sabido nada de Rust, de bases de datos ni de criptografía.

Este archivo explica, desde la base, los conceptos que produjo la construcción del **motor de reportes institucionales** — el séptimo cimiento del substrato de monetización (ADR-0144). Cada concepto cita el código real que esta Story creó.

---

## Concepto

### 1. Qué es una "firma reproducible" y por qué el determinismo la hace verificable por terceros

**El problema de negocio.** El día que Drasus le entregue a un fondo un reporte de validación de estrategia, ese fondo necesita poder confiar en que el documento no fue alterado después de generarse — ni por un error, ni por manipulación deliberada (por ejemplo, alguien que suaviza un drawdown antes de enviarlo). La forma clásica de resolver esto en papel es un sello notarial. En software, el equivalente es una **firma de integridad**: un número (en la práctica, un texto hexadecimal largo) que se calcula a partir del contenido exacto del documento.

**Por qué "reproducible" es la palabra clave.** Una firma cualquiera no sirve si solo el que la generó puede verificarla. La propiedad que la hace útil para un tercero es esta: **si yo tomo el mismo contenido y aplico el mismo procedimiento de cálculo, obtengo EXACTAMENTE la misma firma, bit a bit** — sin importar en qué máquina, en qué momento, ni cuántas veces se repita. Esto se llama **determinismo**: misma entrada → misma salida, siempre. Es la misma idea de "reproducibilidad científica" aplicada a una función de software.

En este proyecto el determinismo no es un detalle de implementación — es un invariante fijo (ADR-0002/0004: "sin I/O, sin reloj de sistema, sin aleatoriedad sin semilla"). La función que ensambla el reporte es un ejemplo puro de esta regla:

```rust
// crates/shared/src/domain/institutional_report_engine.rs
pub fn assemble_report(input: AssembleReportInput) -> InstitutionalReport {
    InstitutionalReport {
        report_type: input.report_type,
        metrics: input.metrics,
        source_result_ref: input.source_result_ref,
        source_event_refs: input.source_event_refs,
        generated_at_ns: input.generated_at_ns,
    }
}
```

Fíjate en lo que **no** hace esta función: no lee el reloj del sistema, no abre un archivo, no genera un número aleatorio. Solo toma los datos que le llegan (`input`) y los reorganiza en la forma del reporte. Si la llamas dos veces con el mismo `input`, el resultado es idéntico byte a byte — no puede ser de otra forma, porque no hay ninguna fuente de variación oculta.

**Qué es exactamente la firma.** Es un **hash criptográfico** (SHA-256 en este caso) del contenido del reporte. Un hash es una función matemática que convierte cualquier texto, sin importar su tamaño, en una huella de longitud fija (64 caracteres hexadecimales para SHA-256). Dos propiedades le dan su utilidad de "sello":
1. **Determinista**: el mismo texto de entrada siempre produce el mismo hash.
2. **Sensible a cualquier cambio**: cambiar UN solo carácter del texto de entrada produce un hash completamente distinto, sin ningún patrón que lo relacione con el original.

La función que calcula la firma en este cimiento:

```rust
// crates/shared/src/domain/institutional_report_engine.rs
pub fn compute_report_signature(report: &InstitutionalReport) -> String {
    let mut hasher = Sha256::new();
    hasher.update(report.canonical_report_json().as_bytes());
    encode_hex(&hasher.finalize())
}
```

`Sha256::new()` crea un "acumulador" del algoritmo SHA-256. `hasher.update(...)` le alimenta los bytes del reporte serializado. `hasher.finalize()` cierra el cálculo y devuelve el hash en bruto (una secuencia de bytes); `encode_hex` lo convierte a un texto legible en hexadecimal (`"a3f9..."`).

**Cómo se prueba que es reproducible.** La prueba discriminante de esta Story es literalmente la definición de determinismo aplicada dos veces, con datos construidos de forma completamente independiente:

```rust
// crates/shared/src/domain/institutional_report_engine.rs (tests)
#[test]
fn compute_report_signature_is_reproducible_across_independent_assemblies() {
    let report_a = assemble_report(sample_input());
    let report_b = assemble_report(sample_input());

    let signature_a = compute_report_signature(&report_a);
    let signature_b = compute_report_signature(&report_b);

    assert_eq!(signature_a, signature_b, "el mismo input debe producir la MISMA firma");
}
```

`sample_input()` se llama dos veces, construyendo dos structs `AssembleReportInput` en memoria de forma separada (no se reutiliza la misma variable). Si la firma dependiera de algo no determinista — el reloj, el orden de un `HashMap`, un `f64` con redondeo distinto entre plataformas — esta prueba fallaría de forma intermitente. Que pase de forma consistente es la evidencia de que la firma es reproducible.

La otra mitad de la prueba discriminante confirma que la firma SÍ reacciona a un cambio real:

```rust
#[test]
fn compute_report_signature_changes_when_a_metric_changes() {
    let mut input = sample_input();
    let original = compute_report_signature(&assemble_report(input.clone()));

    input.metrics.insert("sharpe_e8".to_string(), 200_000_000);
    let changed = compute_report_signature(&assemble_report(input));

    assert_ne!(original, changed, "cambiar una métrica debe cambiar la firma");
}
```

Sin esta segunda prueba, una función que simplemente devolviera siempre el mismo string fijo (`"firma-constante"`) pasaría la primera prueba (sería "determinista" por definición trivial) sin firmar realmente nada. Las dos pruebas juntas son las que demuestran una firma de verdad: estable ante la repetición, sensible al contenido.

---

### 2. Por qué la serialización canónica es la clave de todo lo anterior

**El problema oculto.** `compute_report_signature` no hashea el struct `InstitutionalReport` directamente — un struct en memoria es una región de bytes cuya disposición exacta depende del compilador, la versión de Rust, la arquitectura del procesador, y el orden en que el compilador decidió acomodar los campos. Esa representación en memoria **no es estable entre ejecuciones ni entre máquinas**. Para hashear algo de forma reproducible, primero hay que convertirlo a una forma de texto que sea idéntica sin importar dónde se ejecute: eso es **serializar**.

Pero serializar tampoco alcanza por sí solo. Si el reporte tuviera, por ejemplo, tres métricas (`sharpe_e8`, `max_drawdown_e8`, `pbo_e8`), un serializador ingenuo podría escribirlas en el orden en que se insertaron en memoria — y ese orden puede variar entre dos ejecuciones si internamente se usa un `HashMap` (una estructura de "diccionario" cuyo orden de iteración NO está garantizado en Rust, a propósito, por razones de seguridad y rendimiento). Si el orden varía, el texto JSON resultante varía (`{"a":1,"b":2}` vs `{"b":2,"a":1}` son textos distintos aunque el contenido lógico sea el mismo), y por lo tanto el hash varía — rompiendo la reproducibilidad aunque el CONTENIDO no haya cambiado en absoluto.

**La solución: `BTreeMap` en vez de `HashMap`.** Un `BTreeMap` es un mapa clave-valor que, a diferencia de `HashMap`, **garantiza** que al recorrerlo (y por lo tanto al serializarlo) las claves salen siempre en orden alfabético estricto. Esa garantía viene de cómo está implementado internamente (un árbol balanceado ordenado por clave), no es una casualidad de una versión particular de Rust.

Este cimiento reutiliza exactamente el patrón que `enriched_domain_events` (cimiento #6, STORY-033) ya había establecido:

```rust
// crates/shared/src/domain/institutional_report_engine.rs
fn to_canonical_map(&self) -> BTreeMap<String, JsonValue> {
    let mut map = BTreeMap::new();

    map.insert("generated_at_ns".to_string(), serde_json::json!(self.generated_at_ns));
    map.insert("metrics".to_string(), serde_json::json!(self.metrics));
    map.insert("report_type".to_string(), serde_json::json!(self.report_type.as_str()));
    map.insert("source_event_refs".to_string(), serde_json::json!(self.source_event_refs));
    map.insert("source_result_ref".to_string(), serde_json::json!(self.source_result_ref));

    map
}
```

Nota que `self.metrics` (el campo de métricas del reporte) YA es un `BTreeMap<String, i64>` desde que se definió el struct `InstitutionalReport` — así que la garantía de orden se hereda también en el nivel anidado, sin trabajo adicional. Cuando `serde_json::json!(self.metrics)` convierte ese mapa a JSON, las claves de las métricas (`max_drawdown_e8`, `sharpe_e8`...) también salen ordenadas alfabéticamente.

`to_canonical_map` es una función privada (sin `pub`) porque es un detalle interno de CÓMO se logra la canonicalización — lo público es el resultado:

```rust
pub fn canonical_report_json(&self) -> String {
    let map = self.to_canonical_map();
    serde_json::to_string(&map)
        .expect("BTreeMap<String, JsonValue> de solo strings/enteros siempre serializa")
}
```

Este es el string EXACTO que `compute_report_signature` hashea, y también el string exacto que se persiste en la columna `report_body` de la base de datos (ver más abajo). Eso significa que cualquiera que tenga acceso a la fila persistida puede recalcular la firma de forma independiente y comparar — es literalmente la mecánica de verificación por un tercero que motivó todo el diseño.

**Por qué el `.expect(...)` aquí es seguro (y no una señal de alarma).** La política de comentarios del proyecto exige justificar cada `unwrap()`/`expect()` en producción. Este es uno de los casos donde SÍ se puede probar que nunca falla: `serde_json::to_string` solo puede fallar si el valor contiene un `f64` que sea `NaN` o `Infinity` (esos dos casos no tienen representación válida en JSON). Como todos los campos del reporte son `String`, `i64`, `Option<String>`, `Vec<String>` o `BTreeMap<String, i64>` — nunca `f64` — esa condición de fallo es matemáticamente imposible aquí. Ese es precisamente el motivo de la regla "cero `f64` en montos" (ADR-0141): no es solo por precisión numérica, es también lo que hace posible este `.expect()` seguro.

**Prueba discriminante de la canonicalización.** Para blindar contra una regresión futura (alguien que cambia `BTreeMap` por `HashMap` sin darse cuenta de la consecuencia), existe una prueba que verifica el orden alfabético directamente sobre el JSON producido:

```rust
#[test]
fn canonical_report_json_top_level_keys_are_alphabetically_sorted() {
    let report = assemble_report(sample_input());
    let json = report.canonical_report_json();
    let parsed: JsonValue = serde_json::from_str(&json).expect("JSON válido");
    let keys: Vec<&String> = match &parsed {
        JsonValue::Object(map) => map.keys().collect(),
        _ => panic!("se esperaba un objeto JSON"),
    };
    let mut sorted_keys = keys.clone();
    sorted_keys.sort();
    assert_eq!(keys, sorted_keys, "las claves de nivel superior deben quedar en orden alfabético");
}
```

---

### 3. La diferencia entre `audit_hash` (integridad de la fila) y `signature_hash` (integridad del contenido del reporte)

Esta es la distinción de modelado más importante de la Story, y es fácil confundirla porque ambos son "hashes SHA-256 en hexadecimal" — pero protegen cosas distintas, en capas distintas.

**`signature_hash` protege el CONTENIDO del reporte.** Responde a la pregunta: *"¿este documento que tengo en la mano es exactamente el que se generó, sin alteraciones?"* Se calcula SOLO a partir de los campos de negocio del reporte (tipo, métricas, referencias de trazabilidad, momento de generación) — es la firma reproducible que se explicó arriba. Un tercero puede recalcularla sin tener acceso a la base de datos completa, solo necesita el contenido del reporte.

**`audit_hash` protege la FILA del ledger.** Responde a una pregunta distinta: *"¿el historial completo de reportes generados en esta instancia fue manipulado?"* — por ejemplo, ¿alguien insertó una fila falsa a mano directamente en la base de datos, saltándose el código de la aplicación? Para eso, `audit_hash` no solo cubre el contenido del reporte: cubre TODOS los campos de la fila, incluidos los metadatos de identidad (`owner_id`, `node_id`), la posición en la secuencia (`event_sequence_id`) y, crucialmente, **el hash de la fila anterior** — formando una cadena:

```rust
// crates/shared/src/domain/institutional_report_engine.rs
pub fn compute_report_audit_hash(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    previous_audit_hash: &str,
    owner_id: &str,
    institutional_tag: &str,
    node_id: &str,
    report_type: &str,
    source_result_ref: Option<&str>,
    source_event_refs_json: &str,
    report_body_json: &str,
    signature_hash: &str,
    compliance_status_id: Option<&str>,
) -> String {
    // ... concatena todos los campos con un separador y aplica SHA-256
}
```

Fíjate que `signature_hash` es uno de los ARGUMENTOS de `compute_report_audit_hash` — la firma del contenido entra como un ingrediente más del hash de la fila. Esto significa que si alguien lograra cambiar la firma persistida sin recalcular el `audit_hash`, la cadena de auditoría lo delataría de inmediato. La prueba que demuestra esta relación:

```rust
#[test]
fn compute_report_audit_hash_changes_when_signature_hash_changes() {
    let with_sig_a = compute_report_audit_hash(
        "id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1", "VALIDATION",
        None, "[]", "{}", "sig-aaa", None,
    );
    let with_sig_b = compute_report_audit_hash(
        "id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1", "VALIDATION",
        None, "[]", "{}", "sig-bbb", None,
    );
    assert_ne!(with_sig_a, with_sig_b, "cambiar signature_hash debe cambiar audit_hash");
}
```

**"Encadenado" quiere decir que cada fila apunta a la anterior.** `previous_audit_hash` es el `audit_hash` de la fila que se generó justo antes. Esto es lo mismo que hace una blockchain simplificada: si alguien intentara alterar el reporte #5 del historial después de generado, tendría que recalcular su `audit_hash` — pero el reporte #6 ya guardó (encadenado) el `audit_hash` ORIGINAL del #5, así que la alteración rompe la cadena y se detecta comparando. La primera fila de la tabla (la "génesis") no tiene fila anterior, así que su `audit_chain_hash` es `NULL` — nunca un valor de relleno como el texto `"genesis"` (regla explícita de ADR-0141, para que "es la primera fila" se verifique con `IS NULL`, no comparando strings).

La Shell (`persistence/institutional_report_engine.rs`) es quien orquesta este encadenamiento, leyendo la fila anterior ANTES de insertar la nueva — ver la sección de atomicidad más abajo para el porqué de hacerlo dentro de una transacción.

**Ambos hashes coexisten en la misma fila, con roles distintos** — la migración los declara como columnas separadas:

```sql
-- migrations/0013_generated_reports.sql
audit_hash            TEXT    NOT NULL,             -- SHA-256 del contenido de ESTA FILA + enlace previo (integridad del ledger)
audit_chain_hash      TEXT,                         -- audit_hash de la fila anterior (NULL solo en la fila génesis)
...
signature_hash        TEXT    NOT NULL,             -- Firma de integridad REPRODUCIBLE del CONTENIDO del reporte (distinta de audit_hash)
```

Y la prueba que cierra el criterio de aceptación #5 de la Story confirma, sobre datos reales persistidos, que son valores DISTINTOS:

```rust
// crates/shared/src/persistence/institutional_report_engine.rs (tests)
#[tokio::test]
async fn signature_hash_and_audit_hash_are_present_and_distinct() {
    let pool = migrated_pool().await;
    let clock = DeterministicClock::new(1_000, 100);
    let repo = GeneratedReportRepository::new(&pool, &clock);

    let row = repo.record_report(record_input(150_000_000)).await.expect("registrar reporte");

    assert_ne!(row.signature_hash, row.audit_hash, "signature_hash (contenido) y audit_hash (fila) deben ser distintos");
    assert!(!row.signature_hash.is_empty());
    assert!(!row.audit_hash.is_empty());
}
```

**Regla mnemónica para no confundirlos nunca más:** `signature_hash` es la firma de UN DOCUMENTO (viaja con el reporte si se exporta a PDF/JSON fuera de la base de datos); `audit_hash`/`audit_chain_hash` son la integridad de UNA FILA dentro de UN LIBRO CONTABLE (solo tienen sentido dentro de la base de datos, como parte de una cadena).

---

### 4. Qué es la trazabilidad al audit-log y cómo se modela sin acoplar features

**El requisito (ADR-0027).** Un reporte institucional no puede ser una afirmación de fe — cada métrica que presenta debe poder rastrearse hasta los hechos concretos (eventos) que la originaron. Si un reporte dice "Sharpe = 1.5", un auditor debe poder preguntar "¿de qué eventos de ejecución sale ese número?" y obtener una respuesta verificable, no una caja negra.

**Cómo se modela sin violar la arquitectura hexagonal (ADR-0137).** La regla del proyecto es que ninguna feature accede directamente a las tablas de otra feature — el acceso cruzado ocurre siempre a través de un puerto tipado. `institutional-report-engine` NO lee la tabla `domain_events` (del cimiento #6) para "ir a buscar" los eventos citados. En cambio, el reporte simplemente **guarda los IDS** de los eventos que cita, como texto:

```rust
// crates/shared/src/domain/institutional_report_engine.rs
/// Ids de eventos del event-store (#6) / audit-log que este reporte
/// cita, para trazabilidad (ADR-0027). NUNCA se leen ni modifican los
/// eventos referenciados -- son solo punteros de texto.
pub source_event_refs: Vec<String>,
```

Esto es deliberadamente "débil" en el sentido de que Rust no verifica en tiempo de compilación que esos IDs existan de verdad en `domain_events` — es responsabilidad de quien construye el reporte (el futuro adaptador de producto) pasar IDs reales. Lo que SÍ garantiza el sistema es que, una vez que el reporte se generó con esas referencias, **quedan persistidas de forma inmutable** junto con el reporte, en su propia columna dedicada para hacer consultas ("dame todos los reportes que citan el evento X"):

```sql
-- migrations/0013_generated_reports.sql
-- Lista JSON de ids de eventos del event-store (#6) / audit-log que
-- este reporte cita -- trazabilidad (ADR-0027).
source_event_refs     TEXT    NOT NULL CHECK (json_valid(source_event_refs)),
```

**Por qué también viven dentro de `report_body`, duplicadas.** `source_event_refs` aparece DOS veces en la fila persistida: una vez como columna dedicada (para poder hacer `WHERE source_event_refs LIKE '%evt-1%'` o, mejor, parsear el JSON en una consulta), y otra vez dentro del `report_body` completo (porque forman parte del contenido que la firma protege). No es un error de diseño — es intencional: la columna dedicada es para CONSULTAR, el campo dentro de `report_body` es para VERIFICAR la firma. Si algún día se quisiera desnormalizar (dejar solo una copia), habría que recalcular la firma de forma distinta, así que por ahora conviven.

**Regla de no-alteración, verificada con una prueba.** El reporte "presenta, nunca altera" — la Story lo prueba de dos formas: a nivel del Core (que el struct de salida conserva los mismos valores que el struct de entrada) y a nivel de la composición completa Shell (que lo persistido coincide con lo que se pasó originalmente, sin pasar por ninguna transformación oculta):

```rust
// crates/shared/src/domain/institutional_report_engine.rs (tests)
#[test]
fn assemble_report_preserves_source_event_refs_and_result_ref_unaltered() {
    let input = sample_input();
    let expected_refs = input.source_event_refs.clone();
    let expected_result_ref = input.source_result_ref.clone();

    let report = assemble_report(input);

    assert_eq!(report.source_event_refs, expected_refs, "los source_event_refs no deben alterarse");
    assert_eq!(report.source_result_ref, expected_result_ref, "el source_result_ref no debe alterarse");
}
```

```rust
// crates/shared/src/orchestrator/institutional_report_engine.rs (tests)
#[tokio::test]
async fn generate_report_never_mutates_the_caller_input() {
    let pool = migrated_pool().await;
    let clock = DeterministicClock::new(1_000, 100);

    let input = sample_input();
    let original_refs = input.source_event_refs.clone();

    let row = generate_report(&pool, &clock, identity(), input)
        .await
        .expect("generar y persistir el reporte");

    let persisted_refs: Vec<String> =
        serde_json::from_str(&row.source_event_refs).expect("JSON válido");
    assert_eq!(persisted_refs, original_refs, "los source_event_refs persistidos deben ser EXACTAMENTE los del input original");
}
```

---

### 5. Por qué el "reloj inyectado" es parte del reporte, y no una llamada directa al sistema

El Core (`domain::institutional_report_engine`) tiene prohibido leer la hora del sistema — es una lógica pura (ADR-0002/0004). Pero un reporte necesita saber CUÁNDO se generó. La solución es que el campo `generated_at_ns` viaja como un dato de ENTRADA plano (`i64`), no como una llamada:

```rust
// crates/shared/src/domain/institutional_report_engine.rs
/// Instante de generación del reporte, en nanosegundos UTC (puerto
/// Clock, inyectado por la Shell).
pub generated_at_ns: i64,
```

Quien SÍ tiene permiso de tocar el reloj es el orchestrator (la Shell), que lo hace justo antes de llamar al Core:

```rust
// crates/shared/src/orchestrator/institutional_report_engine.rs
pub async fn generate_report(
    pool: &SqlitePool,
    clock: &dyn Clock,
    identity: ReportGenerationIdentity,
    mut input: AssembleReportInput,
) -> Result<GeneratedReportRow, GenerateReportError> {
    // Paso 1 -- reloj inyectado (única lectura de I/O de esta función antes
    // de ensamblar).
    input.generated_at_ns = clock.timestamp_ns();

    // Paso 2 -- ensamblado puro (Core).
    let report = assemble_report(input);
    ...
```

`clock: &dyn Clock` es un **puerto**: en producción se pasa un `SystemClock` (que sí lee la hora real), y en las pruebas se pasa un `DeterministicClock` (que devuelve un valor fijo o avanza solo cuando se le pide explícitamente con `.tick()`). Esto es lo que permite que el mismo código de composición sea determinista bajo prueba y realista en producción, sin condicionales (`if test { ... } else { ... }`) esparcidos por el código — el patrón se llama **inyección de dependencias** y aquí se aplica al concepto "tiempo actual".

---

### 6. Atomicidad de ledgers append-only bajo concurrencia (reutilizando el patrón de STORY-033)

**El problema.** Dos generaciones de reporte que ocurren "al mismo tiempo" (dos hilos, dos procesos) podrían leer el mismo `event_sequence_id` máximo actual y ambas intentar insertar la siguiente posición — provocando que una de las dos pierda su fila (rechazada por la restricción `UNIQUE`) sin que nadie se entere, salvo que el código maneje ese caso explícitamente.

**La solución: leer y escribir dentro de la MISMA transacción, con el lock tomado desde el inicio.** SQLite ofrece tres modos de iniciar una transacción; el que se usa aquí es `BEGIN IMMEDIATE`, que toma el lock de escritura de la base de datos DESDE EL PRIMER COMANDO, no solo cuando llega el `INSERT`:

```rust
// crates/shared/src/persistence/institutional_report_engine.rs
async fn try_record_report_once(
    &self,
    input: &RecordGeneratedReportInput,
) -> Result<GeneratedReportRow, GeneratedReportRepositoryError> {
    // Abre la transacción tomando el lock de escritura de inmediato.
    let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

    // Lectura (DENTRO de la transacción) -- posición en la cadena GLOBAL.
    let tail_row = sqlx::query(
        "SELECT audit_hash, event_sequence_id FROM generated_reports \
         ORDER BY event_sequence_id DESC LIMIT 1",
    )
    .fetch_optional(&mut *tx)
    .await?;

    // ... deriva event_sequence_id = anterior + 1 ...

    // Escritura (DENTRO de la misma transacción) -- el INSERT.
    sqlx::query("INSERT INTO generated_reports (...) VALUES (...)")
        .execute(&mut *tx)
        .await?;

    // Confirma: recién aquí se libera el lock y la fila se hace visible.
    tx.commit().await?;
    ...
}
```

Con `BEGIN IMMEDIATE`, si dos tareas intentan generar un reporte al mismo tiempo, la SEGUNDA se bloquea esperando a que la primera termine su transacción completa (lectura + escritura + commit) — nunca pueden intercalarse. Esto es exactamente lo contrario de dos sentencias sueltas (`SELECT MAX(...)` y luego, en otra llamada, `INSERT`), donde SÍ podrían intercalarse y perder una fila.

**Por qué hay un reintento, y por qué no es una contradicción con "atómico".** Aun con `BEGIN IMMEDIATE`, existe una ventana de espera (`busy_timeout`, configurado en 5 segundos) durante la cual SQLite reporta "database is locked" si el lock tarda demasiado en liberarse bajo carga extrema. Ese error es TRANSITORIO — no significa que algo esté mal, solo que hay que esperar el turno. El repositorio reintenta hasta 5 veces, re-derivando la posición desde cero cada vez (nunca reutiliza un `event_sequence_id` ya calculado de un intento fallido):

```rust
pub async fn record_report(
    &self,
    input: RecordGeneratedReportInput,
) -> Result<GeneratedReportRow, GeneratedReportRepositoryError> {
    let mut attempt = 0;
    loop {
        attempt += 1;
        match self.try_record_report_once(&input).await {
            Ok(row) => return Ok(row),
            Err(error) => {
                if is_transient_write_conflict(&error) {
                    if attempt < MAX_RECORD_ATTEMPTS {
                        continue;
                    }
                    return Err(GeneratedReportRepositoryError::WriteContention { attempts: attempt });
                }
                return Err(error);
            }
        }
    }
}
```

Si se agotan los 5 intentos, el reporte NUNCA se descarta en silencio — se devuelve un error tipado (`WriteContention`) para que quien llamó decida qué hacer (reintentar más tarde, alertar). "Nunca perder el evento en silencio" es la regla rectora de esta parte del diseño (rust-engineer/SKILL.md §4).

**La prueba que demuestra que esto funciona bajo concurrencia real.** No basta con probarlo con un solo escritor — hay que lanzar varios A LA VEZ y verificar que ninguno se pierde. La prueba usa una base de datos en un ARCHIVO temporal (nunca `:memory:`, porque cada conexión a `:memory:` sería una base de datos distinta y aislada — no habría contención real que probar) y 16 tareas de Tokio corriendo en un runtime de VERDAD multi-hilo:

```rust
// crates/shared/src/persistence/institutional_report_engine.rs (tests)
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_record_reports_persist_every_report_without_gaps_or_lost_rows() {
    let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
    let db_path = temp_dir.path().join("generated_reports_concurrency.sqlite");
    let database_url = format!("sqlite://{}", db_path.display());
    let pool = connect(&database_url).await.expect("conectar");
    migrate(&pool).await.expect("migrar");

    const N: i64 = 16;
    let mut handles = Vec::new();
    for i in 0..N {
        let pool_c = pool.clone();
        let clock_c = clock.clone();
        handles.push(tokio::spawn(async move {
            let repo = GeneratedReportRepository::new(&pool_c, clock_c.as_ref());
            repo.record_report(record_input((i + 1) * 100_000_000)).await
        }));
    }

    for handle in handles {
        handle.await.expect("la tarea no debe entrar en panic")
              .expect("record_report debe tener éxito para cada escritor concurrente");
    }

    let chain = repo.load_chain().await.expect("cargar la cadena completa");
    assert_eq!(chain.len() as i64, N, "deben persistirse las N filas, sin ninguna pérdida");

    let sequence_ids: Vec<i64> = chain.iter().map(|row| row.event_sequence_id).collect();
    assert_eq!(sequence_ids, (1..=N).collect::<Vec<i64>>(), "los event_sequence_id deben ser 1..=N sin huecos ni duplicados");
    // ... y además recalcula cada audit_hash para confirmar que la cadena es íntegra.
}
```

`#[tokio::test(flavor = "multi_thread", worker_threads = 4)]` es lo que hace que las 16 tareas corran genuinamente en paralelo (en threads del sistema operativo distintos), no solo intercaladas cooperativamente en un solo thread — sin esto, la prueba podría dar falso verde aunque el código no fuera realmente seguro bajo concurrencia (la misma lección que dejó STORY-024 con el `Semaphore` decorativo). Si alguien quitara `BEGIN IMMEDIATE` del código de producción, esta prueba es la que se cae: dos tareas leerían el mismo `event_sequence_id` máximo, competirían por la misma posición, y `chain.len()` daría menos de 16.

---

## Trucos de Senior

- **`.clone()` en un test no es "hacer trampa" — es aislar la variable que se está probando.** En `compute_report_signature_changes_when_a_metric_changes`, se hace `input.clone()` antes de calcular la firma "original" para poder seguir usando `input` después de mutarlo (`input.metrics.insert(...)`) sin pelear con el checker de ownership de Rust. Es un patrón habitual en pruebas: clonar el estado "antes" para comparar contra el estado "después" sin que ambos compartan memoria.

- **Reutilizar una función auxiliar de pruebas entre capas (`sample_input`/`record_input`) documenta la composición mejor que un comentario.** El archivo `persistence/institutional_report_engine.rs` construye su `record_input(...)` LLAMANDO a `assemble_report` y `compute_report_signature` del Core real (no un mock inventado) — así, si la firma del Core cambiara de forma, los tests de persistencia lo detectarían de inmediato sin que nadie tenga que actualizar dos lugares a mano.

- **Separar "qué reintentar" de "por qué reintentar" en dos funciones pequeñas (`is_transient_write_conflict` + el `loop` de `record_report`) hace que la política de reintento sea legible de un vistazo**, sin tener que leer los detalles de mensajes de error de SQLite mezclados con la lógica de control de flujo.
