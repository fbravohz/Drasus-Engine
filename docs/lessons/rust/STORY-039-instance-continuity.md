# STORY-039 — Instance Continuity: lecciones de Rust

> **Story:** [STORY-039 — Instance Continuity (cimiento #11 del substrato de monetización · respaldo cifrado + maestro itinerante)](../../execution/STORY-039-instance-continuity.md).
> **Archivos que esta Story produjo y que se citan abajo:** `migrations/0017_instance_continuity.sql`, `crates/shared/src/domain/instance_continuity.rs`, `crates/shared/src/persistence/instance_continuity.rs`, `crates/shared/src/orchestrator/instance_continuity.rs`, `crates/shared/src/public_interface.rs`, `crates/app/src/main.rs`, `crates/shared/Cargo.toml`.
> **Modo de Acompañamiento:** Docente (ADR-0122) — el ingeniero implementó cada bloque por su cuenta y este archivo consolida lo enseñado, con profundidad cero-conocimiento (ADR-0124).

## Concepto

### Qué es el cifrado autenticado y por qué el tag de GCM prueba integridad + confidencialidad a la vez

Cifrar con AES en modo clásico (ej. CBC) solo esconde el contenido: si alguien altera un byte del ciphertext y lo descifras igual, obtienes basura que PARECE un plaintext válido — no hay forma de saber si el dato que acabas de recuperar es el original o fue manipulado en tránsito. Eso es un problema real para un respaldo que viaja a un servidor de un tercero (el proveedor, en este cimiento): si el proveedor (o un atacante) corrompe un byte del blob guardado, el usuario que lo restaura no tendría ninguna señal de que su "backup restaurado" es basura.

AES-GCM resuelve esto siendo un modo de **cifrado autenticado** (AEAD — *Authenticated Encryption with Associated Data*): además del ciphertext, produce un **tag** de autenticación (16 bytes, calculado sobre el ciphertext completo) que se anexa al final. Al descifrar, la librería recalcula el tag esperado ANTES de devolver ni un solo byte de plaintext — si no coincide, se rechaza con un error, punto. No hay "plaintext parcial corrupto": o la operación tiene éxito con el contenido exacto, o falla limpio.

En el Core (`crates/shared/src/domain/instance_continuity.rs`) esto se ve en `decrypt_backup_blob`:

```rust
pub fn decrypt_backup_blob(blob: &EncryptedBackupBlob, key: &[u8; 32]) -> Result<Vec<u8>, EncryptionError> {
    // ...
    cipher
        .decrypt(nonce, ciphertext.as_slice())
        .map_err(|_| EncryptionError::AuthenticationFailed)
}
```

El tipo de retorno es `Result`, no `Vec<u8>` directo — el diseño de la API de `aes-gcm` obliga a manejar el caso "no pasó la autenticación" como un camino de error explícito, no como un valor sospechoso que el llamador tendría que inspeccionar por su cuenta. El test `tampering_a_single_byte_of_the_ciphertext_fails_authentication` demuestra esto: cambia UN carácter del ciphertext hexadecimal y verifica que `decrypt_backup_blob` devuelve `Err(EncryptionError::AuthenticationFailed)`, nunca un `Ok` con contenido corrupto.

Un detalle de diseño API: `aead::Error` (el tipo de error que devuelve la librería `aes-gcm`) es **opaco a propósito** — no dice SI falló por tag inválido, por longitud incorrecta, o por otra razón. Esto es deliberado en criptografía: si el error dijera "el tag no coincide" vs. "la longitud es inválida", un atacante podría usar esa diferencia (un *oracle* de error) para ir adivinando bytes de la clave con ataques de canal lateral. Por eso `decrypt_backup_blob` colapsa CUALQUIER fallo de la librería al mismo `EncryptionError::AuthenticationFailed` — no intenta ser más específico de lo que la librería permite con seguridad.

### Qué es un KDF y por qué la clave se deriva del secreto maestro (que nunca sale)

Un **KDF** (*Key Derivation Function*) toma un secreto de entrada (típicamente algo que un humano puede recordar, como una frase de paso) y produce una clave criptográfica de longitud y calidad fijas a partir de él. No es lo mismo que un hash cualquiera (como SHA-256 sola): un secreto maestro elegido por una persona tiene MUCHA menos entropía real que 256 bits aleatorios — alguien podría probar millones de frases comunes por segundo con hardware barato si el KDF fuera rápido. Argon2 (el ganador de la *Password Hashing Competition*) está diseñado a propósito para ser **lento y costoso en memoria**, así que probar millones de combinaciones se vuelve caro incluso para un atacante con GPUs.

`derive_encryption_key` (`crates/shared/src/domain/instance_continuity.rs`) es la función que hace esto:

```rust
pub fn derive_encryption_key(master_secret: &str, owner_id: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(owner_id.as_bytes());
    let salt = hasher.finalize();

    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(master_secret.as_bytes(), &salt, &mut key)
        .expect("salt de 32 bytes y clave de salida de 32 bytes siempre son válidos para Argon2");
    key
}
```

Dos decisiones concretas aquí:

1. **El *salt* es determinista, no aleatorio.** Un *salt* normalmente se genera al azar para que dos usuarios con la misma contraseña no terminen con la misma clave (defensa contra tablas precomputadas, *rainbow tables*). Pero en este cimiento hay un requisito distinto: **la MISMA cuenta, en OTRA máquina, debe poder derivar la MISMA clave** para poder descifrar el blob que la primera máquina subió — es justo el "maestro itinerante" del ADR-0146. Por eso el *salt* se deriva del `owner_id` (SHA-256 de un identificador de cuenta que ya es público, no un secreto) en vez de generarse al azar: determinismo por diseño, sin sacrificar seguridad, porque el `owner_id` es único por cuenta de todas formas.
2. **`.expect(...)` con justificación escrita.** La política de comentarios del proyecto (`base/SKILL.md` §"Sobre `unwrap()`/`expect()`") exige que todo `expect()` en producción explique por qué es imposible que falle. Aquí, `hash_password_into` solo falla si el *salt* o la clave de salida violan los límites de tamaño que Argon2 documenta — y ambos tamaños son **fijos** en este código (32 bytes, siempre), así que nunca entran en el rango de error. El comentario lo dice explícito, no solo "nunca falla".

**La clave nunca se persiste ni sale de la función que la usa.** No existe ningún `struct` en todo este cimiento con un campo `key` o `encryption_key` — se deriva, se usa en memoria dentro de `encrypt_backup_blob`/`decrypt_backup_blob`, y se descarta al salir de scope (Rust libera la memoria del `[u8; 32]` automáticamente). El test `encrypted_backup_blob_json_never_leaks_key_or_secret` verifica esto de forma estructural: serializa el tipo de puerto `EncryptedBackupBlob` a JSON y confirma que tiene EXACTAMENTE dos claves (`ciphertext_hex`, `nonce_hex`) — si alguien añadiera un tercer campo con la clave por error, ese test se rompe de inmediato.

### Por qué el nonce nunca se reutiliza, pero aun así se siembra en los tests (determinismo, ADR-0002)

El **nonce** (*number used once*) de AES-GCM es un valor de 12 bytes que se combina con la clave para cifrar. La regla de seguridad más importante de GCM es: **el mismo par (clave, nonce) JAMÁS puede usarse dos veces para cifrar mensajes distintos**. Si se reutiliza, un atacante que vea los dos ciphertexts puede recuperar información suficiente para romper la autenticación de mensajes futuros con esa misma clave — es un fallo catastrófico, no una debilidad menor.

Esto podría sugerir "entonces el nonce debe ser impredecible/aleatorio siempre" — y en producción, sí: la Shell debe alimentar una semilla nueva (de una fuente de entropía real) en cada snapshot. Pero el Core de este proyecto es FCIS (`base/SKILL.md`, ADR-0002/0004): **toda función pura debe ser determinista** para poder probarse y auditarse. Si `generate_nonce` leyera el reloj o `/dev/urandom` directamente, la MISMA llamada con los MISMOS argumentos produciría un nonce distinto cada vez, y ningún `assert_eq!` podría verificar nada.

La solución es el MISMO patrón que ya usaba `data_aggregation::apply_differential_privacy` (cimiento #9) para el ruido de privacidad diferencial: un RNG **sembrado explícitamente**.

```rust
pub fn generate_nonce(seed: u64) -> [u8; 12] {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut nonce = [0u8; 12];
    rng.fill(&mut nonce);
    nonce
}
```

`StdRng::seed_from_u64(seed)` construye un generador pseudoaleatorio cuya secuencia de bytes queda 100% determinada por `seed`: la MISMA semilla produce SIEMPRE el MISMO nonce (probado en `generate_nonce_is_deterministic_for_the_same_seed`). Esto **no contradice** la regla "nunca reutilizar el nonce" — la contradicción sería reutilizar la MISMA semilla en dos snapshots reales en producción, y de eso es responsable la Shell (nunca el Core): en producción, cada llamada a `take_encrypted_snapshot` debe recibir una semilla fresca derivada de una fuente de entropía real (nunca la del snapshot anterior). El Core solo garantiza: "dame una semilla, te doy el mismo nonce siempre" — la disciplina de NUNCA repetir la semilla en la vida real vive fuera de esta función, documentada en su doc-comment.

El nonce en sí **no es secreto** — viaja junto al blob (`EncryptedBackupBlob::nonce_hex`) porque hace falta para descifrar después, igual que la sal de un KDF. Lo que nunca se repite es el PAR (clave, nonce), no el nonce aislado.

### Qué es el gate de titularidad de custodia y por qué evita dos escritores de la cadena de auditoría

El "maestro itinerante" (ADR-0146) permite que la misma cuenta opere desde varias máquinas (laptop, PC de escritorio), pero **exactamente una a la vez** puede escribir la cadena de auditoría de esa cuenta. Si dos máquinas escribieran simultáneamente, dos historiales divergentes podrían coexistir sin que nadie lo note hasta que fuera demasiado tarde.

Este cimiento resuelve esto con **concurrencia optimista**, la misma técnica que `AccountRepository::update_email_verification_status` (cimiento #1) ya usaba con `row_version` — pero aplicada a un concepto de negocio distinto: no "¿esta fila de cuenta cambió?", sino "¿qué máquina es la titular AHORA MISMO?". El campo que hace ese papel se llama `custody_epoch` (no `row_version` — ADR-0146 fija ese nombre porque el concepto es "a nivel de instancia completa", no de una fila de negocio cualquiera).

La pieza pura, en el Core (`crates/shared/src/domain/instance_continuity.rs`):

```rust
pub fn decide_custody_claim(
    current: &CustodyState,
    claiming_node_id: &str,
    expected_epoch: i64,
) -> Result<CustodyState, CustodyClaimError> {
    if current.custody_epoch != expected_epoch {
        return Err(CustodyClaimError::CustodyConflict {
            owner_id: current.owner_id.clone(),
            expected_epoch,
        });
    }
    Ok(CustodyState {
        owner_id: current.owner_id.clone(),
        titular_node_id: claiming_node_id.to_string(),
        custody_epoch: current.custody_epoch + 1,
    })
}
```

Esta función es pura: no toca la base de datos, solo COMPARA el epoch que el reclamante cree vigente (`expected_epoch`) contra el epoch actual (`current.custody_epoch`). Si coinciden, el reclamo gana y el epoch avanza en +1 (nadie más puede reclamar desde ESE epoch otra vez). Si no coinciden, alguien más ya reclamó primero — `CustodyConflict`, sin excepciones.

Pero esta comparación en memoria **no basta por sí sola contra una carrera real** entre dos procesos que leen el mismo estado al mismo tiempo — ahí es donde entra la Shell (`crates/shared/src/persistence/instance_continuity.rs`, `CustodyRepository::claim_over_existing`), que aplica la MISMA guarda pero contra la fila real de SQLite, de forma atómica en una sola sentencia:

```rust
let result = sqlx::query(
    "UPDATE custody_state SET \
        updated_at = ?, audit_hash = ?, audit_chain_hash = ?, custody_epoch = ?, titular_node_id = ? \
     WHERE owner_id = ? AND custody_epoch = ?",
)
// ...
.execute(self.pool)
.await?;

if result.rows_affected() == 0 {
    return Err(CustodyRepositoryError::CustodyConflict { /* ... */ });
}
```

El `WHERE owner_id = ? AND custody_epoch = ?` es la clave: SQLite solo puede aplicar el `UPDATE` si la fila SIGUE en el epoch que el reclamante leyó. Si dos máquinas leen el mismo epoch (digamos, 3) y ambas intentan reclamar, la PRIMERA en llegar a SQLite gana (la fila pasa a epoch 4), y cuando la SEGUNDA ejecuta su `UPDATE ... WHERE custody_epoch = 3`, esa condición ya NO es cierta (la fila real está en epoch 4) — `rows_affected()` devuelve 0, y el código lo traduce a `CustodyConflict` en vez de fingir éxito. La base de datos, no la memoria del proceso, es el árbitro final de la carrera — por eso la decisión pura del Core (`decide_custody_claim`) y la guarda de la Shell (el `WHERE`) son DOS capas de la MISMA invariante: la del Core es rápida y testeable en aislamiento; la del `WHERE` es la que de verdad protege contra dos procesos del sistema operativo compitiendo en paralelo.

El test `two_claims_from_the_same_epoch_only_one_wins` (`crates/shared/src/persistence/instance_continuity.rs`) demuestra esto de punta a punta: siembra un estado inicial en epoch 3, hace que "node-B" reclame primero (gana, pasa a epoch 4), y luego hace que "node-C" reclame TAMBIÉN desde epoch 3 (el epoch que ya quedó viejo) — y verifica que ese segundo reclamo devuelve `CustodyConflict`, no un éxito silencioso que dejaría a dos máquinas creyéndose tituales.

### Por qué el harness CLI resuelve `owner_id` llamando a `central-identity` en vez de aceptarlo como texto suelto

La Orden exige que este cimiento consuma el tipo **REAL** de `AccountIdentity` (producido por `central-identity`, #1) como su puerto `identity_in` — no un `owner_id` inventado a mano. La tentación fácil sería aceptar `"owner_id": "..."` directo en el JSON de `--input` y usarlo tal cual; eso *compilaría* y hasta *pasaría* una prueba superficial, pero sería un placeholder disfrazado: cualquier string serviría, sin pasar por la cuenta real.

`verify_instance_continuity` (`crates/shared/src/public_interface.rs`) en cambio resuelve la identidad llamando al MISMO verificador que ya usan `verify_licensing_system`/`verify_enriched_domain_events`:

```rust
let identity_verifier =
    crate::orchestrator::central_identity::LocalStubCentralIdentityVerifier::new(&pool, clock.as_ref());
let account_identity = identity_verifier
    .verify_identity(crate::orchestrator::central_identity::IdentityVerificationRequest {
        email: format!("verify-instance-continuity-{}@drasus.local", input.my_node_id),
        oauth_provider: None,
        machine_identifiers: vec![input.my_node_id.clone()],
        institutional_tag: default_institutional_tag(),
        access_token_id: None,
    })
    .await?;
```

Esto crea (o encuentra) una fila real en la tabla `accounts` y devuelve un `AccountIdentity` con un `owner_id` generado de verdad (un UUIDv7, no un texto inventado) — se puede confirmar corriendo el CLI: el campo `owner_id` de la salida cambia de una ejecución a otra, porque cada corrida crea una BD temporal nueva con una cuenta nueva. De ahí en adelante, `identity.owner_id`/`identity.institutional_tag` (NUNCA un valor suelto del JSON de entrada) alimentan tanto el registro de respaldos como el estado de custodia — el "quién es el dueño de esta cuenta" viene siempre de `central-identity`, nunca se inventa dentro de este cimiento.

Nota aparte: el `node_id` de custodia (`my_node_id`, "qué máquina es titular") es un concepto DISTINTO del `node_id` de `AccountIdentity` (la huella de hardware de #1) — aunque ambos se llaman `node_id`, uno identifica "la máquina vinculada a la cuenta" (#1, una por cuenta) y el otro "cuál de varias máquinas activadas tiene la custodia ahora" (#11, cambia con el tiempo). Por eso `my_node_id` viaja como su propio campo, sin intentar reusar el `node_id` que produce el verificador de identidad.

## Trucos de Senior

### `let ... else` para "extraer o salir temprano" sin anidar un `match`

En `is_transient_write_conflict` (`crates/shared/src/persistence/instance_continuity.rs`), en vez de anidar un `match` de dos niveles para llegar al campo `db` de un error de SQLite, se usa la sintaxis `let ... else` (estabilizada en Rust 1.65):

```rust
fn is_transient_write_conflict(error: &BackupRegistryRepositoryError) -> bool {
    let BackupRegistryRepositoryError::Database(sqlx::Error::Database(db)) = error else {
        return false;
    };
    // a partir de aquí, `db` ya está desestructurado y disponible
    let message = db.message().to_lowercase();
    // ...
}
```

Se lee como: "intenta hacer *pattern match* de `error` contra esta forma exacta; si NO coincide, ejecuta el bloque `else` (que debe divergir — `return`, `continue`, `panic!`, etc.) y sal". Es la forma idiomática de Rust moderno para el patrón "si esto no es la variante que espero, no sigo" sin envolver el resto de la función en un `if let` con sangría extra — el mismo patrón que ya usaba `enriched_domain_events::is_transient_write_conflict`, reutilizado aquí tal cual.

### Reutilizar el mismo tipo de error del Core en la Shell vía `impl From<...>`

`decide_custody_claim` (Core) devuelve `CustodyClaimError`; `CustodyRepository::claim_titular` (Shell) devuelve `CustodyRepositoryError`. En vez de mapear el error a mano en cada punto de llamada con un `match` repetido, `crates/shared/src/persistence/instance_continuity.rs` implementa la conversión UNA vez:

```rust
impl From<CustodyClaimError> for CustodyRepositoryError {
    fn from(error: CustodyClaimError) -> Self {
        match error {
            CustodyClaimError::CustodyConflict { owner_id, expected_epoch } => {
                CustodyRepositoryError::CustodyConflict { owner_id, expected_epoch }
            }
        }
    }
}
```

Con este `impl From` en su lugar, el operador `?` (usado en `claim_over_existing` sobre `decide_custody_claim(...)?`) hace la conversión automáticamente — no hace falta un `.map_err(...)` explícito en cada sitio donde el error del Core cruza hacia la Shell. Es el mismo mecanismo que ya usan `#[from]` de `thiserror` para envolver `sqlx::Error`, aplicado aquí a mano porque son dos enums del propio crate (no un error de una librería externa).
