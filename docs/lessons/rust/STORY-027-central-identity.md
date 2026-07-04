# STORY-027 — Central Identity: lecciones de Rust

> **Story:** [STORY-027 — Central Identity (cimiento #1 del substrato de monetización)](../../execution/STORY-027-central-identity.md).
> **Archivos que esta Story produjo y que se citan abajo:** `migrations/0007_central_identity.sql`, `crates/shared/src/domain/central_identity.rs`, `crates/shared/src/persistence/central_identity.rs`, `crates/shared/src/orchestrator/central_identity.rs`, `crates/shared/src/types/mod.rs`, `crates/shared/src/public_interface.rs`, `crates/app/src/main.rs`.
> **Modo de Acompañamiento:** Docente (ADR-0122) — el ingeniero implementó cada bloque por su cuenta y este archivo consolida lo enseñado, con profundidad cero-conocimiento (ADR-0124).

## Concepto

### ¿Qué es una "huella de hardware" y por qué tiene que ser determinista?

Una huella de hardware (`node_id` en el esquema, "huella de hardware" en la Feature) es un identificador que representa "esta máquina física", derivado de datos que casi nunca cambian: el UUID de la placa madre, el serial de un disco, la MAC de la tarjeta de red. Sirve para dos cosas en `central-identity`: (1) vincular una cuenta a la instancia que la usa, y (2) detectar abuso — "¿por qué desde el mismo hardware se registraron 20 cuentas?".

**Determinista** significa: la misma entrada produce siempre la misma salida, sin importar cuántas veces se ejecute ni en qué momento. Esto es distinto de, por ejemplo, generar un número aleatorio o leer la hora del reloj — ahí cada ejecución da un resultado distinto. Si la huella de hardware no fuera determinista, cada vez que el motor arrancara generaría un `node_id` distinto para la MISMA máquina física, y el sistema jamás podría reconocerla como "la misma" entre un arranque y el siguiente.

`crates/shared/src/domain/central_identity.rs`:

```rust
pub fn compute_hardware_fingerprint(machine_identifiers: &[String]) -> String {
    const SEP: char = '\u{1F}';

    let mut buffer = String::new();
    for identifier in machine_identifiers {
        buffer.push_str(identifier);
        buffer.push(SEP);
    }

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    let digest = hasher.finalize();

    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
```

Esta función NO llama al sistema operativo para leer el hardware real — recibe la lista de identificadores YA recolectados como parámetro (`&[String]`). Esa separación es la misma disciplina FCIS del resto del proyecto: recolectar datos del hardware real es I/O (le corresponde a la cáscara, cuando exista ese adaptador); calcular el hash a partir de esos datos es lógica pura (Core), y por eso vive en `domain/`. `SHA-256` es una función de hash criptográfico: toma cualquier cantidad de bytes de entrada y produce siempre una salida de 256 bits (64 caracteres en hexadecimal), y la MISMA entrada produce SIEMPRE la MISMA salida — esa es justamente la propiedad de determinismo que necesitamos.

El separador `\u{1F}` (un carácter invisible, "Unit Separator" de la tabla ASCII) se inserta entre cada identificador antes de hashear. ¿Por qué? Sin separador, la lista `["ab", "c"]` y la lista `["a", "bc"]` producirían el mismo string concatenado (`"abc"`) y por tanto el mismo hash — dos huellas de hardware distintas colisionando por accidente. Con el separador, `"ab\u{1F}c\u{1F}"` y `"a\u{1F}bc\u{1F}"` son strings distintos. Este es el MISMO patrón que ya usaba `audit_log.rs` y `telemetry.rs` en Stories anteriores — no se inventó nada nuevo, se reusó la técnica.

La prueba discriminante que exige la Orden vive en el mismo archivo:

```rust
#[test]
fn hardware_fingerprint_is_deterministic_for_same_identifiers() {
    let identifiers = vec![
        "motherboard-uuid-ABC123".to_string(),
        "disk-serial-XYZ789".to_string(),
    ];
    let first_boot = compute_hardware_fingerprint(&identifiers);
    let second_boot = compute_hardware_fingerprint(&identifiers);
    assert_eq!(first_boot, second_boot, "...");
}

#[test]
fn hardware_fingerprint_differs_when_an_identifier_changes() {
    // ...cambia un solo identificador y assert_ne! contra el original...
}
```

Sin la propiedad de determinismo, el primer test fallaría (dos llamadas con la misma entrada darían hashes distintos); sin sensibilidad a cambios, el segundo test fallaría (identificadores distintos darían el mismo hash) — exactamente el rojo→verde que pide la Orden.

### `row_version` vs. tabla append-only (`event_sequence_id`)

El proyecto ya tenía un patrón de "cadena de hashes" para detectar manipulación de datos históricos (`audit_events`, `job_results`, `telemetry_samples`): cada fila nueva incluye el hash de la fila anterior, y el campo `event_sequence_id` es un contador GLOBAL, único en toda la tabla, que nunca se repite ni se reutiliza — así se sabe el orden exacto en que las filas se insertaron. Esto funciona perfecto para tablas **append-only** (solo se inserta, nunca se actualiza ni se borra): cada fila es un evento nuevo e inmutable.

`accounts` es distinta: el estado de verificación de correo de UNA MISMA cuenta CAMBIA con el tiempo (`PENDING` → `VERIFIED`). No es "un evento nuevo por cada cambio" — es "la misma fila, actualizada". Para este caso, ADR-0141 define un contador distinto: `row_version`, que NO es global a la tabla, sino que cuenta las versiones de **una fila en particular**: arranca en 1 cuando la fila se crea, y sube a 2, 3, 4... cada vez que ESA fila se actualiza. Dos cuentas distintas pueden tener ambas `row_version = 1` al mismo tiempo — no hay conflicto, porque cada una cuenta sus propias versiones, no una posición en una cola compartida.

`migrations/0007_central_identity.sql` documenta la regla en el propio SQL:

```sql
audit_chain_hash   TEXT,                         -- audit_hash de la versión anterior de esta fila (NULL solo en la fila génesis)
row_version        INTEGER NOT NULL,             -- Contador de versión de esta fila; arranca en 1, +1 en cada UPDATE
```

Y `crates/shared/src/persistence/central_identity.rs` implementa el incremento en `update_email_verification_status`:

```rust
pub async fn update_email_verification_status(
    &self,
    account: &Account,
    new_status: EmailVerificationStatus,
) -> Result<Account, AccountRepositoryError> {
    let now_ns = self.clock.timestamp_ns();
    let row_version = account.row_version + 1;
    // ...UPDATE que fija row_version = row_version, audit_chain_hash = account.audit_hash (el hash ANTERIOR)...
}
```

La prueba discriminante crea una cuenta (`row_version == 1`), la actualiza (`row_version` debe ser `2`), y verifica que `audit_chain_hash` de la versión 2 apunte exactamente al `audit_hash` que tenía la versión 1 — y además RELEE la fila desde SQLite (no solo compara el struct en memoria) para confirmar que el cambio quedó grabado en disco:

```rust
let verified = repo.update_email_verification_status(&account, EmailVerificationStatus::Verified).await?;
assert_eq!(verified.row_version, 2, "row_version debe incrementar en cada UPDATE");
assert_eq!(verified.audit_chain_hash, Some(account.audit_hash.clone()));

let reloaded = repo.find_by_id(&account.id).await?.expect("la cuenta debe existir");
assert_eq!(reloaded.row_version, 2);
```

Si alguien "arreglara" el código para que `update_email_verification_status` no incrementara `row_version` (o lo dejara fijo en 1), este test se rompería de inmediato — es la definición misma de "prueba discriminante": puede fallar si el comportamiento no está.

### Concurrencia optimista: `row_version` como guarda, no como adorno (corrección tras revisión de QA)

La primera versión de este código incrementaba `row_version` en cada UPDATE — pero el `WHERE` filtraba solo por `id`. Eso hacía `row_version` **cosmético**: dos actualizaciones que arrancan de la MISMA versión en memoria (por ejemplo, dos partes del sistema que leyeron la cuenta al mismo tiempo) ambas tenían éxito y la segunda pisaba a la primera en silencio ("last-write-wins"), bifurcando la cadena de `audit_hash` sin que nadie se enterara. El QA lo marcó como defecto bloqueante.

La técnica que lo corrige se llama **concurrencia optimista** (optimistic concurrency). "Optimista" porque asume que los conflictos son raros y no bloquea la fila por adelantado (eso sería "pesimista", con un lock); en su lugar, al momento de escribir comprueba que la fila siga en la versión que se leyó. Si otra escritura la adelantó, la comprobación falla y se rechaza — nadie pisa a nadie. En SQL se expresa metiendo la versión esperada dentro del propio `WHERE`:

`crates/shared/src/persistence/central_identity.rs`, `update_email_verification_status`:

```rust
let result = sqlx::query(
    "UPDATE accounts SET \
        updated_at = ?, audit_hash = ?, audit_chain_hash = ?, row_version = ?, \
        email_verification_status = ? \
     WHERE id = ? AND row_version = ?",   // <- la guarda optimista
)
// ...binds..., incluido .bind(account.row_version) al final...
.execute(self.pool)
.await?;

if result.rows_affected() == 0 {
    return Err(AccountRepositoryError::VersionConflict {
        id: account.id.clone(),
        expected: account.row_version,
    });
}
```

La clave está en `result.rows_affected()`. En SQL, un `UPDATE` cuyo `WHERE` no encuentra ninguna fila NO es un error — simplemente actualiza cero filas y devuelve `Ok`. Por eso hay que preguntarle explícitamente al resultado "¿cuántas filas tocaste?". Si la respuesta es 0, significa que ninguna fila tenía a la vez ese `id` Y ese `row_version` — es decir, la fila existe pero ya avanzó a otra versión. Ahí devolvemos `VersionConflict` en lugar de fingir éxito.

Cero-conocimiento sobre `rows_affected()`: es un número que el motor de base de datos reporta después de cada `UPDATE`/`DELETE`/`INSERT`, contando cuántas filas cambió realmente. Es distinto de "¿hubo error?": una consulta puede ejecutarse perfectamente (sin error) y aun así no tocar ninguna fila porque nada encajó con el `WHERE`. Ignorar ese número es justamente el bug que tenía la primera versión.

El test discriminante simula los dos escritores partiendo de la misma versión:

```rust
let first_writer_view = account.clone();
let second_writer_view = account;      // ambos en row_version == 1

let updated = repo.update_email_verification_status(&first_writer_view, Verified).await
    .expect("el primer update debe tener éxito");     // 1 -> 2, OK
let conflict = repo.update_email_verification_status(&second_writer_view, Rejected).await;
assert!(matches!(conflict, Err(AccountRepositoryError::VersionConflict { expected: 1, .. })));
```

Se comprobó que este test es genuinamente discriminante quitando el `AND row_version = ?` del SQL: el test falla (el segundo update devuelve `Ok` en vez de conflicto). Con la guarda puesta, pasa. Ese es el ciclo rojo→verde que exige el protocolo de pruebas — una prueba que no puede fallar cuando el comportamiento está ausente no prueba nada.

### El caso degenerado de una función de hash: no todo input es válido (corrección tras revisión de QA)

La primera versión de `compute_hardware_fingerprint` devolvía siempre un `String`. El QA encontró el agujero: con una lista de identificadores vacía (`[]`), el buffer que se hashea también queda vacío, y el SHA-256 del vacío es una constante conocida (`e3b0c44...b855`). Resultado: TODA máquina sin identificadores utilizables produciría el MISMO `node_id` — y la señal anti-abuso "N cuentas desde el mismo hardware" se volvería falsa (todas colisionan en la misma huella "vacía").

La lección de diseño cero-conocimiento: una función pura y determinista **puede, aun así, tener entradas que no debería aceptar**. Determinismo (misma entrada → misma salida) no implica "toda entrada produce una salida útil". Cuando existe un input degenerado que rompe el propósito de la función, lo correcto es cambiar el tipo de retorno de `String` a `Result<String, Error>` y rechazarlo explícitamente:

```rust
pub fn compute_hardware_fingerprint(
    machine_identifiers: &[String],
) -> Result<String, HardwareFingerprintError> {
    // Exige al menos un identificador con contenido real (no vacío ni solo espacios).
    if !machine_identifiers.iter().any(|id| !id.trim().is_empty()) {
        return Err(HardwareFingerprintError::NoUsableIdentifiers);
    }
    // ...resto del hash igual que antes...
}
```

`iter().any(|id| !id.trim().is_empty())` se lee como: "¿existe AL MENOS UN identificador que, tras quitarle los espacios, no quede vacío?". `.any(closure)` recorre la lista y devuelve `true` en cuanto encuentra un elemento que cumple la condición (y `false` si la lista está vacía o ningún elemento la cumple — exactamente los dos casos degenerados que queremos rechazar). El cambio de firma obliga a que TODO el que llame maneje el fallo: el orquestador ahora usa el operador `?` (`let node_id = compute_hardware_fingerprint(&request.machine_identifiers)?;`), que propaga el error hacia arriba en lugar de crear una cuenta con huella basura. El compilador no deja compilar si te olvidas de manejar el `Result` — el tipo mismo te fuerza a considerar el caso de error.

### Normalizar en la frontera: unicidad case-insensitive (corrección tras revisión de QA)

Tercer defecto del QA: ni el Core ni el Shell normalizaban el correo. El índice único de SQLite compara bytes exactos (BINARY por defecto), así que `Case@Example.com` y `case@example.com` son dos claves distintas — se crearían dos cuentas para lo que es el mismo correo, rompiendo "una cuenta por correo" y abriendo una evasión trivial de los límites por-cuenta (justo lo que el substrato anti-abuso quiere impedir).

La solución es **normalizar en la frontera**: convertir el dato a una forma canónica única en el punto exacto donde entra al sistema, antes de validarlo o persistirlo. La función es pura y trivial, pero el concepto es importante:

```rust
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}
```

`trim()` quita espacios en blanco al principio y al final; `to_lowercase()` pasa todo a minúsculas. Aplicada tanto en `create` (antes de insertar) como en `find_by_email` (antes de buscar), garantiza que la unicidad sea efectivamente case-insensitive aunque el índice de la base compare bytes exactos: todas las variantes de mayúsculas/espacios del mismo correo colapsan a la misma cadena antes de tocar SQLite. El principio general — "normaliza en un solo lugar, en la entrada, no en cada uso" — evita que una parte del código busque `case@...` y otra guarde `Case@...` y nunca se encuentren. (La alternativa `COLLATE NOCASE` en el índice también funcionaría, pero normalizar en la frontera además deja el dato ya limpio en disco, no solo comparable.)

### Puerto hexagonal + adaptador stub (ADR-0144: "puerto ahora, adaptador después")

Un **puerto** en arquitectura hexagonal es un contrato — en Rust, un `trait` — que describe QUÉ operación se necesita, sin decir CÓMO se implementa. Un **adaptador** es una implementación concreta de ese contrato. La ventaja: el resto del sistema depende del contrato (el `trait`), nunca de una implementación concreta — así se puede cambiar la implementación real sin tocar ni una línea de quien la usa.

Este cimiento necesita "verificar la identidad contra la Cabina de Mando Central del proveedor" — pero esa Cabina de Mando todavía no existe (es un servidor que aún no se ha construido). ADR-0144 resuelve esto con la regla "puerto ahora, adaptador después": se define el contrato completo HOY, y se construye un adaptador STUB (una implementación de relleno, con comportamiento honesto y documentado) que cumple el contrato sin depender de ningún servidor real. El día que la Cabina de Mando exista, se escribe un SEGUNDO adaptador (uno que sí hace la llamada de red real) y se sustituye — el resto del sistema no cambia.

`crates/shared/src/orchestrator/central_identity.rs` declara el puerto:

```rust
#[async_trait]
pub trait CentralIdentityVerifier: Send + Sync {
    async fn verify_identity(
        &self,
        request: IdentityVerificationRequest,
    ) -> Result<AccountIdentity, CentralIdentityError>;
}
```

`Send + Sync` son marcadores que Rust exige para que un valor pueda cruzar entre hilos/tareas asíncronas de forma segura — cualquier cosa que se use dentro de `tokio` (el runtime asíncrono del proyecto) necesita cumplirlos. `async_trait` es una macro que permite declarar métodos `async fn` dentro de un `trait` — Rust todavía no soporta esto de forma nativa sin la macro (limitación conocida del lenguaje al momento de escribir esto).

Y el stub, en el mismo archivo:

```rust
pub struct LocalStubCentralIdentityVerifier<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

#[async_trait]
impl<'a> CentralIdentityVerifier for LocalStubCentralIdentityVerifier<'a> {
    async fn verify_identity(&self, request: IdentityVerificationRequest) -> Result<AccountIdentity, CentralIdentityError> {
        let repo = AccountRepository::new(self.pool, self.clock);
        if let Some(existing) = repo.find_by_email(&request.email).await? {
            return Ok(AccountIdentity::from(&existing));
        }
        let node_id = compute_hardware_fingerprint(&request.machine_identifiers);
        let created = repo.create(NewAccount { /* ... */ }).await?;
        Ok(AccountIdentity::from(&created))
    }
}
```

Nótese la honestidad del stub: NO finge contactar ningún servidor. Busca la cuenta localmente por correo (si ya existe, la reusa — así verificar dos veces el mismo correo no duplica la cuenta) y, si no existe, la crea con estado `PENDING`. Esto está documentado explícitamente en el doc-comment del struct: "ninguna verificación real contra un servidor central ocurrió — es trabajo diferido". Esta misma idea de puerto+stub se repite dentro del propio Core para la firma OAuth (`verify_oauth_signature` en `domain/central_identity.rs`): un login OAuth real usa criptografía asimétrica (RS256/ES256) contra la clave pública JWKS del proveedor, que requeriría un crate de criptografía fuera de alcance de este cimiento; la función implementa el MISMO contrato observable (dado un payload y un material público, decide si la firma es válida, de forma pura) usando SHA-256 como sustituto verificable, documentado como tal en el propio código.

### Caché con TTL (Time To Live) y por qué el reloj se inyecta

Una caché guarda un resultado ya calculado para no tener que recalcularlo (o, en este caso, no tener que volver a verificar contra la Cabina de Mando) en cada llamada. El problema de cachear para siempre: si la cuenta cambia de estado en el servidor central, la instancia local seguiría creyendo el dato viejo indefinidamente. La solución es un **TTL** ("Time To Live", tiempo de vida): la caché guarda TAMBIÉN el instante en que se guardó, y cada lectura compara "¿cuánto tiempo pasó desde entonces?" contra un límite configurado (`IDENTITY_CACHE_TTL`, 24 horas por defecto). Pasado ese límite, la caché "expira" — deja de servir el valor guardado y exige revalidar.

`crates/shared/src/orchestrator/central_identity.rs`:

```rust
pub struct IdentityCache {
    clock: Arc<dyn Clock>,
    config: IdentityCacheConfig,
    entry: StdMutex<Option<CachedEntry>>,
}

impl IdentityCache {
    pub fn get(&self) -> Option<AccountIdentity> {
        let now_ns = self.clock.timestamp_ns();
        let guard = self.entry.lock().expect("mutex de caché de identidad envenenado");
        match guard.as_ref() {
            Some(cached) if now_ns - cached.cached_at_ns < self.config.ttl_ns => Some(cached.identity.clone()),
            _ => None,
        }
    }
}
```

El punto cero-conocimiento importante aquí: `self.clock.timestamp_ns()` NO es `std::time::SystemTime::now()`. Es el puerto `Clock` que el proyecto ya usa en todas partes (`clock.md`, Stories anteriores) — en producción es el reloj real del sistema (`SystemClock`), pero en tests se inyecta un `DeterministicClock`, que solo avanza cuando el test se lo pide explícitamente (`clock.advance(ns)`). Si esta caché hubiera usado `SystemTime::now()` directamente, la única forma de probar "el TTL expira" sería hacer que el test literalmente ESPERE 24 horas de reloj real (con `std::thread::sleep`) — inviable. Con el reloj inyectado, el test simplemente le ordena al reloj "avanza 1000 nanosegundos" sin esperar nada:

```rust
#[test]
fn get_returns_none_after_ttl_expires() {
    let det_clock = Arc::new(DeterministicClock::new(0, 0));
    let clock: Arc<dyn Clock> = det_clock.clone();
    let cache = IdentityCache::new(clock, IdentityCacheConfig { ttl_ns: 1_000 });

    cache.set(sample_identity());
    det_clock.advance(1_000); // avanza el reloj VIRTUAL, no espera nada real

    assert_eq!(cache.get(), None, "pasado el TTL, la caché debe exigir revalidación");
}
```

Este test corre en microsegundos reales y prueba, con la misma certeza que si hubiera esperado 24 horas de verdad, que la lógica de expiración es correcta.

### Guardarraíl ADR-0093 hecho estructural, no solo por convención

ADR-0093 exige que ningún dato sensible (contraseñas, credenciales de bróker, IPs de servidores live) salga del motor local. La forma más débil de cumplir esto sería "acordarse de no incluir esos campos" — un acuerdo humano que cualquier cambio futuro puede romper sin querer. `AccountIdentity` (`domain/central_identity.rs`) lo hace estructural: el tipo que se expone por el puerto `identity_out` SOLO tiene cinco campos.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AccountIdentity {
    pub owner_id: String,
    pub email: String,
    pub email_verification_status: EmailVerificationStatus,
    pub node_id: String,
    pub institutional_tag: String,
}
```

Y el test lo blinda contra regresión futura, comparando la lista EXACTA de claves del JSON serializado:

```rust
#[test]
fn account_identity_json_never_leaks_secret_fields() {
    let identity = AccountIdentity { /* ... */ };
    let json = serde_json::to_value(&identity).expect("...");
    let mut keys: Vec<&str> = json.as_object().unwrap().keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, vec!["email", "email_verification_status", "institutional_tag", "node_id", "owner_id"], "...");
}
```

Si alguien, en una Story futura, agrega un campo `broker_api_key` a `AccountIdentity` "por conveniencia", este test se rompe INMEDIATAMENTE — no hace falta que un revisor humano lo note leyendo el diff. Esa es la diferencia entre un guardarraíl "de convención" (un comentario que dice "no hagas esto") y uno "estructural" (el código mismo se niega a compilar/pasar tests si se viola la regla). Hay una segunda capa de defensa en el mismo test: una búsqueda de substrings prohibidos (`"password"`, `"api_key"`, direcciones IP privadas típicas) sobre el JSON completo, por si algún valor de un campo existente terminara conteniendo un secreto colado por error.

## Trucos de Senior

- `#[serde(default)]` en un campo `Option<T>` de un struct que se deserializa desde JSON permite que la clave venga AUSENTE del JSON de entrada (no solo `null`) — sin esa anotación, según la versión de `serde`, una clave ausente puede fallar la deserialización en vez de asumir `None`. Se usó en `CentralIdentityVerifyInput` (`public_interface.rs`) para que `cargo run -p app -- verify central-identity --input '{"email":"a@b.com"}'` funcione sin tener que mandar `oauth_provider`/`machine_identifiers` explícitamente.
- `#[serde(default = "nombre_de_funcion")]` es la variante para un default que NO es `Default::default()` del tipo — se usó para que `institutional_tag` caiga en `"DRASUS_LOCAL_VERIFY"` en vez de un string vacío cuando el usuario no lo pasa.
- `#[serde(serialize_with = "funcion")]` permite serializar un campo con una función custom sin tener que envolver el tipo en un wrapper nuevo — se usó para que `EmailVerificationStatus` (un enum interno) se serialice siempre como su string canónico (`"PENDING"`), en vez de exponer la forma en que Rust representa el enum internamente.
- `Arc<dyn Clock>` compartido entre dos consumidores (el verificador y la caché, en `verify_central_identity`) se construye UNA vez y se clona (barato: `Arc::clone` solo incrementa un contador de referencias, no copia el reloj) — evita crear dos relojes de producción independientes que podrían, en teoría, leer instantes ligeramente distintos.
- Cuando una función toma un préstamo (`&'a dyn Clock`) y ese préstamo ya no se necesita después de su último uso, el compilador (gracias a NLL — "Non-Lexical Lifetimes") permite mover el valor original después, sin declarar el bloque de scope manualmente. Esto evitó tener que envolver el reloj en un `Arc` desde el principio solo para esquivar un conflicto de préstamo que en la práctica no ocurre.
- `impl From<&Account> for AccountIdentity` (`persistence/central_identity.rs`) centraliza la proyección "fila completa de la base de datos" → "tipo de puerto público sin secretos" en UN solo lugar. Cualquier código que necesite exponer una cuenta como `AccountIdentity` llama `AccountIdentity::from(&account)` — no hay una segunda copia de esa lógica de proyección en otro archivo que alguien pueda olvidar actualizar.
- Cuando dos tipos distintos necesitan el mismo nombre porque representan la misma idea a dos niveles distintos (el marcador de tipo de puerto en `types::AccountIdentity` vs. el struct con los campos reales en `domain::central_identity::AccountIdentity`), Rust no obliga a inventar un nombre distinto — el path del módulo ya los distingue. Es el mismo patrón que el proyecto ya usaba con `types::AuditEvent` vs. `domain::audit_log::AuditEvent`.
- Un `UPDATE`/`DELETE` de SQL cuyo `WHERE` no encaja con ninguna fila NO devuelve error — devuelve `Ok` con `rows_affected() == 0`. Siempre que "la fila tiene que existir para que esto tenga sentido" sea parte de la lógica (concurrencia optimista, borrado condicional), comprueba `rows_affected()`: confiar solo en "¿hubo error?" deja pasar el caso silencioso de "no tocó nada".
- Meter la versión esperada dentro del propio `WHERE` (`... AND row_version = ?`) es la forma más barata de concurrencia optimista — un solo `UPDATE` atómico que actualiza-o-no-hace-nada, sin transacción explícita ni lock. La base de datos garantiza la atomicidad de esa comparación-y-escritura por ti.
- Cambiar el retorno de una función pura de `T` a `Result<T, E>` cuando descubres un input degenerado no es "complicar la firma": es hacer que el compilador FUERCE a cada llamador a decidir qué hacer con ese caso. El `?` en el llamador es una línea; el bug de aceptar el input inválido en silencio es invisible hasta producción.
- `iter().any(|x| condición)` es el atajo idiomático para "¿existe al menos uno que cumpla?" — devuelve `false` sobre una lista vacía sin caso especial, lo que lo hace perfecto para vetar a la vez "lista vacía" y "ningún elemento válido" en una sola expresión.
