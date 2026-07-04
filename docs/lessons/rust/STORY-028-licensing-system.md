# STORY-028 — Licensing System: lecciones de Rust

> **Story:** [STORY-028 — Licensing System (cimiento #2 del substrato de monetización)](../../execution/STORY-028-licensing-system.md).
> **Archivos que esta Story produjo y que se citan abajo:** `migrations/0008_licensing_system.sql`, `crates/shared/src/domain/licensing_system.rs`, `crates/shared/src/persistence/licensing_system.rs`, `crates/shared/src/orchestrator/licensing_system.rs`, `crates/shared/src/public_interface.rs`, `crates/app/src/main.rs`.
> **Modo de Acompañamiento:** Docente (ADR-0122) — el ingeniero implementó cada bloque por su cuenta y este archivo consolida lo enseñado, con profundidad cero-conocimiento (ADR-0124).

## Concepto

### ¿Qué es una firma asimétrica y por qué el cliente solo tiene la clave pública?

Firmar un documento digitalmente sirve para probar dos cosas a la vez: "esto lo produjo quien dice haberlo producido" y "nadie lo alteró después de firmarlo". Hay dos familias de algoritmos para lograrlo:

- **Simétrica (HMAC):** una ÚNICA clave sirve para firmar Y para verificar. Es rápida, pero tiene un problema fatal para este caso de uso: si el cliente necesita verificar la licencia SIN conexión a internet (licensing-system.md: "Local-First... la red solo se utiliza asíncronamente"), esa clave tiene que estar incrustada en el binario del cliente. Y si la clave que verifica es la MISMA que firma, cualquiera que extraiga la clave del binario (trivial con un desensamblador) puede firmar sus propias licencias falsas — el candado y la llave son la misma cosa.
- **Asimétrica (Ed25519, RSA...):** existen DOS claves matemáticamente relacionadas pero distintas: una PRIVADA (firma) y una PÚBLICA (verifica). Conocer la pública no permite reconstruir la privada, y una firma hecha con la privada solo se puede confirmar como válida usando la pública correspondiente — pero la pública NO sirve para firmar nada. Esto resuelve el problema: el emisor (la Cabina de Mando, o en esta Story su stub de desarrollo) se queda con la clave privada, que NUNCA sale de ahí; el cliente solo recibe la pública, incrustada en el binario, y con ella puede verificar pero jamás falsificar.

Esta es la corrección obligatoria #3 del Gate de Coherencia de la Orden: el documento-semilla original de la Feature (§6 "Proceso") decía `HMAC-SHA256`, pero eso viola ADR-0093 §3 ("PROHIBIDO almacenar claves privadas de firma... en el cliente") en cuanto se piensa qué clave tendría que llevar el binario para verificar sin red. `crates/shared/src/domain/licensing_system.rs` implementa la verificación con el crate `ed25519-dalek`:

```rust
pub fn verify_license_signature(
    payload: &LicensePayload<'_>,
    signature_hex: &str,
    public_key_hex: &str,
) -> Result<(), LicenseSignatureError> {
    let public_key_bytes = decode_hex(public_key_hex).ok_or(LicenseSignatureError::InvalidPublicKeyEncoding)?;
    let public_key_array: [u8; 32] = public_key_bytes.as_slice().try_into()
        .map_err(|_| LicenseSignatureError::InvalidPublicKeyEncoding)?;
    let verifying_key = VerifyingKey::from_bytes(&public_key_array)
        .map_err(|_| LicenseSignatureError::InvalidPublicKeyEncoding)?;

    let signature_bytes = decode_hex(signature_hex).ok_or(LicenseSignatureError::InvalidSignatureEncoding)?;
    let signature_array: [u8; 64] = signature_bytes.as_slice().try_into()
        .map_err(|_| LicenseSignatureError::InvalidSignatureEncoding)?;
    let signature = Signature::from_bytes(&signature_array);

    let message = canonical_license_bytes(payload);
    verifying_key.verify_strict(&message, &signature).map_err(|_| LicenseSignatureError::SignatureMismatch)
}
```

Nótese que esta función NUNCA toma una clave privada como argumento — estructuralmente no PUEDE firmar, solo verificar. `VerifyingKey` (32 bytes) y `Signature` (64 bytes) son los tamaños fijos que exige el algoritmo Ed25519; por eso `try_into::<[u8; 32]>()`/`[u8; 64]>()` rechaza cualquier dato que no tenga exactamente esa longitud, ANTES de intentar interpretarlo como una clave o una firma.

Quien SÍ tiene la clave privada es el emisor de desarrollo, en la Shell (nunca en el Core — genera la clave con aleatoriedad real del sistema operativo, algo que el Core tiene prohibido por ADR-0002/0004):

```rust
// crates/shared/src/orchestrator/licensing_system.rs
struct DevKeypair {
    signing_key: SigningKey,       // PRIVADA -- nunca se serializa ni sale de este struct
    verifying_key_hex: String,     // PÚBLICA -- esto sí se entrega al "cliente"
}

impl DevKeypair {
    fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);   // azar REAL del SO
        // ...
    }
}
```

La prueba discriminante del criterio #2 de la Orden verifica ambos lados del contrato: una firma válida se acepta, y un solo byte alterado (del payload O de la firma) la rechaza:

```rust
#[test]
fn verify_license_signature_rejects_tampered_payload() {
    let original = sample_payload("lic-1", "owner-1", "node-A");
    let (signature_hex, public_key_hex) = signed_sample(&original);
    let tampered = sample_payload("lic-1", "owner-1", "node-B-ATTACKER"); // node_id cambiado
    assert_eq!(
        verify_license_signature(&tampered, &signature_hex, &public_key_hex),
        Err(LicenseSignatureError::SignatureMismatch)
    );
}
```

Se comprobó que es genuinamente discriminante: si `verify_license_signature` no comparara de verdad la firma (por ejemplo, si solo revisara que los hex tuvieran el largo correcto), este test pasaría con `Ok(())` en vez de fallar — exactamente lo que el protocolo de pruebas de la Orden exige evitar.

### Reutilizar una huella de hardware ya calculada, en vez de volver a derivarla

`central-identity` (STORY-027, cimiento #1) ya resuelve el problema de "¿cómo identifico esta máquina de forma estable?" con `compute_hardware_fingerprint`, y lo expone en el puerto `identity_out` como `AccountIdentity.node_id`. La corrección obligatoria #2 del Gate de esta Orden es NO volver a resolver ese mismo problema aquí — sería lógica duplicada, y peor: si algún día cambia el algoritmo de huella, dos implementaciones podrían divergir silenciosamente y un mismo hardware físico terminaría con dos `node_id` distintos según qué feature lo calculó.

La solución en `domain/licensing_system.rs` es deliberadamente una función de una línea:

```rust
pub fn hardware_matches(license_node_id: &str, instance_node_id: &str) -> bool {
    license_node_id == instance_node_id
}
```

No hay ningún `Sha256`, ningún acceso a hardware — solo una comparación de dos strings que YA llegaron calculados desde otro lado: uno del archivo de licencia (`license_node_id`, persistido en la migración `0008`), y otro de `AccountIdentity.node_id` (el puerto `identity_in`, producido por `central-identity`). Esta es la esencia de "puerto tipado" en la arquitectura hexagonal (ADR-0137): esta feature no necesita saber CÓMO se deriva una huella de hardware, solo necesita CONSUMIR el resultado ya calculado a través del puerto.

### Heartbeat con período de gracia: una máquina de estados sobre el tiempo

Un "heartbeat" (latido) es una señal periódica que confirma "sigo vivo/vigente". Aquí, en vez de exigir conexión constante a un servidor (lo cual violaría Local-First), la licencia guarda una fecha de vencimiento (`heartbeat_expires_at`) que se extiende cada vez que la instancia logra revalidar en línea. El problema de diseño es: ¿qué pasa exactamente en el instante en que esa fecha se acerca, y qué pasa justo después de pasarla?

`docs/features/licensing-system.md` describe tres comportamientos distintos según qué tan lejos esté el vencimiento, y `evaluate_heartbeat_status` los modela como cuatro estados con tres fronteras:

```rust
pub enum HeartbeatStatus { Fresh, RecheckWindow, WithinGrace, Expired }

pub fn evaluate_heartbeat_status(
    now_ns: i64,
    heartbeat_expires_at_ns: i64,
    config: &HeartbeatConfig,
) -> HeartbeatStatus {
    let recheck_starts_at = heartbeat_expires_at_ns.saturating_sub(config.recheck_window_ns);
    let grace_ends_at = heartbeat_expires_at_ns.saturating_add(config.grace_period_ns);

    if now_ns < recheck_starts_at { HeartbeatStatus::Fresh }
    else if now_ns < heartbeat_expires_at_ns { HeartbeatStatus::RecheckWindow }
    else if now_ns < grace_ends_at { HeartbeatStatus::WithinGrace }
    else { HeartbeatStatus::Expired }
}
```

Las tres fronteras dibujan una línea de tiempo: `[Fresh] --(recheck_starts_at)--> [RecheckWindow] --(expires_at)--> [WithinGrace] --(grace_ends_at)--> [Expired]`. `Fresh` y `RecheckWindow` operan normal (la única diferencia es que `RecheckWindow` dispara una alerta visual en la UI); `WithinGrace` sigue operando pero ya venció el heartbeat formal (el usuario honesto que perdió internet un rato no queda bloqueado de inmediato); `Expired` es lo único que de verdad restringe.

`now_ns` viene del reloj INYECTADO (`Clock`), nunca de `SystemTime::now()` directamente — el mismo patrón que ya usaba `IdentityCache` en STORY-027. Esto es lo que permite probar las cuatro ventanas sin esperar tiempo real:

```rust
#[test]
fn heartbeat_status_is_within_grace_just_after_expiry() {
    let config = HeartbeatConfig { recheck_window_ns: 100, grace_period_ns: 50 };
    assert_eq!(evaluate_heartbeat_status(1_020, 1_000, &config), HeartbeatStatus::WithinGrace);
}
```

`saturating_sub`/`saturating_add` (en vez de `-`/`+` directos) son la variante de las operaciones aritméticas que, en vez de hacer *overflow* silencioso o entrar en pánico cuando el resultado se sale del rango de `i64`, se detiene en el valor mínimo o máximo representable. Con una configuración de ventanas absurdamente grandes (alguien mete `grace_period_ns = i64::MAX` por error), `heartbeat_expires_at_ns.saturating_add(...)` da `i64::MAX` en vez de "envolver" hacia un número negativo — que rompería la comparación `now_ns < grace_ends_at` de formas muy difíciles de depurar.

### Cómo un gate decide sin llamar a la red: separar "refrescar" de "consultar"

ADR-0039 prohíbe una llamada de red SÍNCRONA en el hot-path (la ruta que se ejecuta antes de cada operación sensible, con presupuesto de latencia de milisegundos). Pero saber "¿la licencia sigue vigente?" en principio SÍ requiere, tarde o temprano, hablar con la Cabina de Mando. La solución, igual que `IdentityCache` en STORY-027, es partir el problema en dos piezas con responsabilidades distintas:

1. **`build_execution_gate`** (`orchestrator/licensing_system.rs`): SÍ hace I/O (lee SQLite local para contar activaciones, y en el futuro llamaría a la red para revalidar). Se ejecuta de forma ASÍNCRONA, fuera del hot-path — por ejemplo, en un job periódico de fondo.
2. **`ExecutionGateCache`**: guarda el ÚLTIMO resultado calculado por `build_execution_gate`, junto con el instante en que se guardó. Su método `get()` es síncrono, no hace I/O, y es la ÚNICA superficie que el hot-path real (`execute`, `telemetry`) puede permitirse consultar:

```rust
pub fn get(&self) -> Option<ExecutionGate> {
    let now_ns = self.clock.timestamp_ns();          // lectura de memoria, no de red
    let guard = self.entry.lock().expect("...");     // un Mutex en memoria del proceso
    match guard.as_ref() {
        Some(cached) if now_ns - cached.cached_at_ns < self.config.ttl_ns => Some(cached.gate.clone()),
        _ => None,
    }
}
```

La prueba de "hot-path sin I/O" de esta Story no simula una red y comprueba que no se llamó — hace algo más fuerte, estructural: llama a `derive_execution_gate` (la función que decide el veredicto) dentro de un `#[test]` SÍNCRONO normal, sin runtime `async`, sin pool de base de datos, sin ningún mock de red:

```rust
#[test]
fn derive_execution_gate_has_no_network_io_dependency() {
    let gate = derive_execution_gate(GateEvaluationInput { /* ... */ });
    assert_eq!(gate.verdict, GateVerdict::Allow);
}
```

Si `derive_execution_gate` necesitara de verdad tocar la red, esta prueba ni siquiera COMPILARÍA (necesitaría `#[tokio::test]`, un pool, credenciales...) — el hecho de que compile y corra como test síncrono trivial es, en sí mismo, la demostración de que la función no depende de I/O.

### Supresión de telemetría por tier: una tabla de verdad de tres casos

ADR-0143 introduce el modelo de negocio: el tier de pago (`Sovereign`) al corriente obtiene privacidad real (la telemetría de trabajo se suprime en origen); el tier gratuito (`Explorer`) nunca la suprime; y un `Sovereign` que dejó de pagar (`heartbeat` vencido más allá de la gracia) PIERDE el privilegio — la emisión se reactiva, sin borrar nada del entorno del usuario. Esto se modela con una función que combina DOS entradas (el tier, y el estado de heartbeat) en una tabla de verdad de tres casos reales (el cuarto — Explorer vencido — colapsa al mismo resultado que Explorer vigente: nunca suprime):

```rust
pub fn should_suppress_work_telemetry(tier: LicenseTier, heartbeat_status: HeartbeatStatus) -> bool {
    match tier {
        LicenseTier::Sovereign => !matches!(heartbeat_status, HeartbeatStatus::Expired),
        LicenseTier::Explorer => false,
    }
}
```

`!matches!(heartbeat_status, HeartbeatStatus::Expired)` se lee "verdadero para cualquier estado que NO sea `Expired`" — es decir, `Fresh`, `RecheckWindow` y `WithinGrace` cuentan los tres como "al corriente" para efectos de supresión (el usuario sigue en su período de gracia sin conexión, así que su privacidad no se revoca de golpe apenas se le vence el heartbeat formal). Las pruebas cubren los tres casos de negocio explícitamente:

```rust
#[test]
fn sovereign_current_suppresses_work_telemetry() {
    assert!(should_suppress_work_telemetry(LicenseTier::Sovereign, HeartbeatStatus::Fresh));
    assert!(should_suppress_work_telemetry(LicenseTier::Sovereign, HeartbeatStatus::RecheckWindow));
    assert!(should_suppress_work_telemetry(LicenseTier::Sovereign, HeartbeatStatus::WithinGrace));
}
#[test]
fn sovereign_expired_reactivates_telemetry() {
    assert!(!should_suppress_work_telemetry(LicenseTier::Sovereign, HeartbeatStatus::Expired));
}
```

### Por qué las activaciones cuentan máquinas, no procesos

La Feature (`licensing-system.md` §3, FIJO) exige: "corre una instancia de Drasus por máquina... un segundo arranque en la misma máquina comparte la huella y NO cuenta como una segunda activación". La forma ingenua de contar activaciones sería un contador que sube cada vez que el proceso arranca — pero eso confundiría "reinicié mi laptop" con "instalé Drasus en una laptop nueva". La unidad correcta de conteo es la MÁQUINA (identificada por su huella de hardware, `node_id`), no el PROCESO (que nace y muere en cada arranque).

Esto se resuelve en dos capas que se refuerzan:

1. **El esquema** (`migrations/0008_licensing_system.sql`) tiene un índice único sobre `(owner_id, node_id)` — a nivel de base de datos, es FÍSICAMENTE imposible tener dos filas para la misma máquina del mismo dueño:

```sql
CREATE UNIQUE INDEX IF NOT EXISTS idx_licenses_owner_node
    ON licenses (owner_id, node_id);
```

2. **El repositorio** (`persistence/licensing_system.rs`, `LicenseRepository::activate`) busca primero antes de insertar — el mismo patrón de idempotencia que `LocalStubCentralIdentityVerifier::verify_identity` en STORY-027:

```rust
pub async fn activate(&self, new_activation: NewLicenseActivation) -> Result<LicenseRecord, LicenseRepositoryError> {
    if let Some(existing) = self.find_by_owner_and_node(&new_activation.owner_id, &new_activation.node_id).await? {
        return Ok(existing);   // reactivar la misma máquina reusa la fila, no inserta otra
    }
    // ...INSERT solo si de verdad es una máquina nueva...
}
```

La prueba discriminante activa la MISMA máquina dos veces y comprueba que el conteo (`COUNT(DISTINCT node_id)`) no sube, y luego activa tres máquinas distintas y comprueba que sí sube a 3 — y que un "reinicio" (reactivar una de las tres ya vistas) sigue en 3:

```rust
#[tokio::test]
async fn three_distinct_machines_count_as_three_activations() {
    // activa node-A, node-B, node-C -> count == 3
    repo.activate(sample_activation(&owner_id, "node-B")).await.expect("reactivar node-B");
    let count_after_reboot = repo.count_distinct_activations(&owner_id).await.expect("...");
    assert_eq!(count_after_reboot, 3, "un segundo arranque en una máquina ya vista no debe sumar activación");
}
```

### El bug real que atrapó esta batería de pruebas: "reconstruir exactamente lo que se firmó"

Al integrar el emisor de licencias con el gate (`build_execution_gate`), una prueba de integración falló con "firma de licencia inválida" a pesar de que la firma en sí era perfectamente válida. La causa raíz es una lección de diseño importante sobre firmas digitales: **para verificar una firma, hay que reconstruir BYTE POR BYTE el mismo mensaje que se firmó — cualquier diferencia, aunque sea de un campo que "debería dar igual", invalida la verificación.**

El error concreto: la migración original NO tenía una columna separada para el `license_id` que va DENTRO del payload firmado — reutilizaba por accidente el `id` de la fila de activación (la PK, generada fresca en cada `INSERT` con `Uuid::now_v7()`). Pero el `license_id` firmado por el emisor y el `id` de la fila de activación son dos conceptos DISTINTOS: una misma licencia (`license_id`) puede tener varias filas de activación (una por máquina, hasta el límite del tier) — cada fila con su PROPIO `id` de fila, pero las tres compartiendo el MISMO `license_id`. Al reconstruir el payload para verificar con `license.id` en vez de `license.license_id`, el mensaje reconstruido nunca coincidía con el que de verdad se había firmado.

La corrección fue añadir la columna que faltaba y usarla explícitamente en la reconstrucción:

```sql
-- migrations/0008_licensing_system.sql
license_id         TEXT    NOT NULL,   -- el payload firmado; DISTINTO de `id` (PK de la fila de activación)
```

```rust
// orchestrator/licensing_system.rs, build_execution_gate
let payload = LicensePayload {
    license_id: &license.license_id,   // NO license.id
    owner_id: &license.owner_id,
    // ...
};
```

La lección cero-conocimiento general: cuando una función de verificación criptográfica falla de forma inesperada, el primer sospechoso NO es el algoritmo (Ed25519 es matemáticamente correcto) — es la reconstrucción del mensaje. Cualquier campo que participó en la firma original (incluido un identificador que "parece" redundante con otro) tiene que reconstruirse EXACTAMENTE igual en el momento de verificar, y eso obliga a persistir cada uno de esos campos tal cual, sin derivarlos ni sustituirlos por un valor "equivalente".

Una segunda variante del mismo problema apareció con `issued_at_ns`: al principio se reconstruía usando `created_at_ns` (cuándo se guardó la FILA en SQLite) en vez de un campo `issued_at` propio (cuándo el EMISOR firmó el payload) — con un reloj determinista que avanza en cada llamada, ambos instantes casi nunca coinciden exactamente. La migración terminó con dos columnas de tiempo con significados distintos y ambas necesarias:

```sql
issued_at             INTEGER NOT NULL,  -- cuándo el EMISOR firmó (parte del payload firmado, inmutable)
heartbeat_expires_at  INTEGER NOT NULL,  -- cuándo vence el heartbeat vigente (cambia en cada refresco, y SÍ se re-firma)
```

## Trucos de Senior

- `try_into::<[u8; N]>()` sobre un `&[u8]` (por ejemplo `bytes.as_slice().try_into()`) es la forma idiomática de pasar de "un slice de longitud dinámica" a "un array de longitud FIJA conocida en tiempo de compilación" — falla limpio (`Err`) si la longitud no coincide exactamente, en vez de entrar en pánico o truncar en silencio. Se usó dos veces en `verify_license_signature` para validar que la clave pública mide EXACTAMENTE 32 bytes y la firma EXACTAMENTE 64, antes de dárselas a `ed25519-dalek`.
- `#[allow(clippy::too_many_arguments)]` es preferible a envolver un montón de parámetros en un struct "de una sola vez" cuando la función (aquí, `compute_license_audit_hash`) es un cálculo interno de una sola llamada, sin API pública que se beneficie de agrupar los campos — el struct extra sería una capa de indirección sin ganancia real de legibilidad.
- Cuando dos structs distintos necesitan compartir "cómo se ve un mismo dato lógico" (aquí, `LicensePayload` se usa tanto para firmar en `LocalStubLicenseIssuer::issue_license` como para verificar en `verify_license_signature`), conviene que AMBOS pasen por una única función de serialización canónica (`canonical_license_bytes`) — así "firmar" y "verificar" comparten literalmente el mismo código de construcción del mensaje, y un cambio futuro en el formato solo se edita en un lugar.
- `matches!(valor, Patrón)` dentro de un `!matches!(...)` es más legible que escribir la negación de una comparación de enum campo por campo (`estado != HeartbeatStatus::Expired` habría funcionado igual aquí porque el enum implementa `PartialEq`, pero `matches!` escala mejor cuando el patrón necesita capturar variantes con datos, o varias variantes a la vez con `|`).
- Separar una operación en "la parte que hace I/O y refresca un caché" (`build_execution_gate`, async) de "la parte que solo lee el caché" (`ExecutionGateCache::get`, síncrona) es el mismo patrón general que resuelve CUALQUIER restricción de "sin llamadas de red en el hot-path" — no es específico de licencias. Vale la pena reconocerlo como plantilla reusable.
