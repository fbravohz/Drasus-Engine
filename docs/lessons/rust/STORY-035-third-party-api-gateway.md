# STORY-035 — Third-Party API Gateway: lecciones de Rust

> **Story:** [STORY-035 — Third-Party API Gateway (cimiento #8 del substrato de monetización)](../../execution/STORY-035-third-party-api-gateway.md).
> **Archivos que esta Story produjo y que se citan abajo:** `migrations/0014_api_gateway.sql`, `crates/shared/src/domain/third_party_api_gateway.rs`, `crates/shared/src/persistence/third_party_api_gateway.rs`, `crates/shared/src/orchestrator/third_party_api_gateway.rs`, `crates/shared/src/public_interface.rs`, `crates/app/src/main.rs`.
> **Modo de Acompañamiento:** Docente (ADR-0122) — el ingeniero implementó cada bloque por su cuenta y este archivo consolida lo enseñado, con profundidad cero-conocimiento (ADR-0124).

## Concepto

### Por qué una credencial se guarda hasheada y nunca en claro

Una "credencial de API" es, en el fondo, una contraseña que Drasus le entrega a un sistema externo (un fondo, una plataforma, un bot) para que se identifique en cada llamada. Si esa contraseña se guardara tal cual en la base de datos (`api_credentials.credential_hash = "sk-demo-123"`), cualquiera con acceso de lectura a esa tabla — un administrador deshonesto, una fuga de backup, un `SELECT *` mal protegido — podría copiarla y hacerse pasar por ese tercero indefinidamente. La regla del proyecto (ADR-0093) es tajante: un secreto NUNCA se persiste en claro.

La solución es un **hash criptográfico de un solo sentido**: una función que convierte el secreto en una cadena de longitud fija (aquí, SHA-256, 64 caracteres hexadecimales) de la que es computacionalmente inviable volver al secreto original. Autenticar deja de ser "comparar el secreto guardado contra el presentado" y pasa a ser "hashear lo presentado y comparar los dos HASHES":

```rust
// crates/shared/src/domain/third_party_api_gateway.rs
pub fn hash_api_credential(secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    encode_hex(&hasher.finalize())
}

pub fn authenticate(presented: &str, stored_hash: &str, status: CredentialStatus) -> AuthVerdict {
    if status == CredentialStatus::Revoked {
        return AuthVerdict::Denied(AuthDenialReason::Revoked);
    }
    if hash_api_credential(presented) != stored_hash {
        return AuthVerdict::Denied(AuthDenialReason::InvalidCredential);
    }
    AuthVerdict::Authenticated
}
```

Fíjate que `authenticate` recibe el secreto EN CLARO (`presented`) como parámetro, pero solo lo usa para hashearlo de inmediato — nunca lo guarda, nunca lo compara byte a byte contra nada persistido. Lo único que la base de datos conoce es `stored_hash`. Un atacante que robara la tabla completa `api_credentials` tendría una lista de hashes, no de secretos — y SHA-256 no tiene una operación inversa conocida que revierta eso.

Esto no es solo teoría: hay una prueba que lo verifica de forma ejecutable, no como una promesa de diseño:

```rust
// crates/shared/src/persistence/third_party_api_gateway.rs
#[tokio::test]
async fn create_persists_only_the_hash_never_the_plaintext_secret() {
    let secret = "sk-super-secret-do-not-leak";
    let stored_hash = hash_api_credential(secret);
    repo.create(sample_new_credential(&stored_hash)).await.expect("crear credencial");

    let raw: String = sqlx::query("SELECT credential_hash FROM api_credentials LIMIT 1")
        .fetch_one(&pool).await.expect("leer fila cruda").get(0);
    assert_eq!(raw, stored_hash);
    assert_ne!(raw, secret, "la columna jamás debe contener el secreto en claro");
}
```

Y en `domain::third_party_api_gateway::tests::third_party_response_json_never_leaks_the_presented_secret` se verifica lo mismo del lado de la RESPUESTA que el gateway le devuelve al tercero: ni siquiera el hash almacenado aparece ahí — lo único que un tercero legítimo necesita saber es si su solicitud fue `ALLOWED`, `RATE_LIMITED` o `DENIED`.

### Por qué la revocación gana SIEMPRE, incluso con el secreto correcto

Fíjate en el orden dentro de `authenticate`: primero se revisa `status`, y solo si NO está revocada se compara el hash. Esto es deliberado. Si el orden fuera al revés (comparar el hash primero, y "por si acaso" revisar el estado después), el código seguiría funcionando hoy — pero dejaría una trampa para el futuro: alguien que refactorice la función y borre "por error" el segundo chequeo (porque "ya comparamos el hash, ¿qué más hace falta?") volvería inservible la revocación sin que ningún test unitario de la ruta feliz lo detectara. Poniendo la revocación como la PRIMERA puerta, es imposible olvidarla sin que la función deje de compilar o de tener sentido leyéndola de arriba a abajo.

La prueba discriminante que blinda esto:

```rust
// crates/shared/src/domain/third_party_api_gateway.rs
#[test]
fn authenticate_denies_revoked_credential_even_with_correct_secret() {
    let stored_hash = hash_api_credential("sk-demo-123");
    let verdict = authenticate("sk-demo-123", &stored_hash, CredentialStatus::Revoked);
    assert_eq!(verdict, AuthVerdict::Denied(AuthDenialReason::Revoked));
}
```

### Qué es una ventana de rate-limit y cómo se computa determinísticamente

"Rate-limit" (límite de tasa) es la regla "esta credencial puede hacer como máximo N solicitudes exitosas cada X segundos". Existe para proteger el motor de un tercero que, por error o por abuso, dispara miles de solicitudes por segundo: sin este control, un solo cliente mal comportado podría saturar el servicio para todos los demás.

Computarlo de forma **determinista** significa que la decisión ("¿cabe esta solicitud o no?") depende ÚNICAMENTE de dos números — cuántas solicitudes ya se hicieron en la ventana vigente, y cuál es el límite — nunca de temporizadores en segundo plano, contadores que se resetean con un `sleep`, ni nada que dependa de CUÁNDO corre el código. Esa es la función pura, sin I/O:

```rust
// crates/shared/src/domain/third_party_api_gateway.rs
pub fn compute_rate_limit(requests_in_window: i64, limit: i64) -> RateLimitVerdict {
    if requests_in_window < limit {
        RateLimitVerdict::Allow
    } else {
        RateLimitVerdict::RateLimited
    }
}
```

`requests_in_window` es un número que la Shell YA calculó ANTES de llamar a esta función — contando cuántas filas `outcome = 'ALLOWED'` tiene esa credencial en `api_usage_records` desde el inicio de la ventana. La función en sí no sabe qué es una "ventana de tiempo", ni lee ningún reloj: solo compara dos enteros. Esa separación es FCIS en acción — el Core (`compute_rate_limit`) es una comparación trivial y 100% predecible; la parte que SÍ depende del tiempo y de la base de datos (contar filas en un rango de `created_at`) vive en la Shell:

```rust
// crates/shared/src/persistence/third_party_api_gateway.rs
pub async fn count_allowed_in_window(&self, credential_id: &str, since_ns: i64) -> Result<i64, ApiUsageRepositoryError> {
    let row = sqlx::query(
        "SELECT COUNT(*) AS total FROM api_usage_records \
         WHERE credential_id = ? AND outcome = 'ALLOWED' AND created_at >= ?",
    )
    .bind(credential_id).bind(since_ns).fetch_one(self.pool).await?;
    Ok(row.get("total"))
}
```

El borde exacto merece atención porque es donde los errores "por uno" (*off-by-one*) suelen esconderse. Con `limit = 100`: si YA hubo 99 solicitudes permitidas antes (`requests_in_window = 99`), la que se está evaluando ahora sería la centésima — justo en el límite — y `99 < 100` es verdadero, así que se permite. Si ya hubo 100 (`requests_in_window = 100`), la que se evalúa sería la 101ª — una de más — y `100 < 100` es falso, así que se rechaza. La comparación estricta `<` (no `<=`) es exactamente lo que separa "cabe justo en el límite" de "se pasó por una":

```rust
// crates/shared/src/domain/third_party_api_gateway.rs
#[test]
fn compute_rate_limit_allows_at_the_exact_boundary() {
    assert_eq!(compute_rate_limit(99, 100), RateLimitVerdict::Allow);
}

#[test]
fn compute_rate_limit_rejects_one_past_the_boundary() {
    assert_eq!(compute_rate_limit(100, 100), RateLimitVerdict::RateLimited);
}
```

Esto es observable de punta a punta desde la terminal, sin leer una línea de código: sembrando 99 usos previos, el comando `verify` responde `ALLOWED`; sembrando 100, responde `RATE_LIMITED`, con el mismo `rate_limit_per_window`:

```bash
$ cargo run -p app -- verify third-party-api-gateway --input \
  '{"credential":"sk-demo-abc","endpoint":"CERTIFY","rate_limit_per_window":100,"requests_in_window":99}'
{"ok": true, "outcome": "ALLOWED", "delegate_to": "CERTIFY", ...}

$ cargo run -p app -- verify third-party-api-gateway --input \
  '{"credential":"sk-demo-123","endpoint":"CERTIFY","rate_limit_per_window":100,"requests_in_window":100}'
{"ok": true, "outcome": "RATE_LIMITED", "delegate_to": null, ...}
```

### La diferencia entre tabla MUTABLE (`row_version`) y APPEND-ONLY (`event_sequence_id`) — por qué credenciales es una y uso es la otra

Esta Story crea DOS tablas con naturalezas OPUESTAS, y entender por qué cada una es lo que es (y no lo contrario) es el corazón de ADR-0141.

**`api_credentials` es MUTABLE.** Una credencial de API tiene un ciclo de vida real: nace activa, y en algún momento puede revocarse. "Revocar" ES una edición del estado de esa fila — no tiene sentido modelarlo como "insertar una fila nueva que diga REVOKED" porque entonces existirían DOS filas para la MISMA credencial (la activa y la revocada) y cualquier consulta ("¿esta credencial está activa?") tendría que decidir cuál de las dos filas es la vigente. Para esto, ADR-0141 exige `row_version`: cada `UPDATE` incrementa un contador entero, y ese contador es el mecanismo de **concurrencia optimista** — evita que dos revocaciones simultáneas de la misma credencial se pisen sin darse cuenta:

```rust
// crates/shared/src/persistence/third_party_api_gateway.rs
let result = sqlx::query(
    "UPDATE api_credentials SET updated_at = ?, audit_hash = ?, audit_chain_hash = ?, row_version = ?, status = ? \
     WHERE id = ? AND row_version = ?",
)
// ...
.execute(self.pool).await?;

if result.rows_affected() == 0 {
    return Err(ApiCredentialRepositoryError::VersionConflict { id: credential.id.clone(), expected: credential.row_version });
}
```

El truco está en el `WHERE id = ? AND row_version = ?`: el `UPDATE` solo tiene efecto si la fila SIGUE en la versión que leímos en memoria. Si otra revocación concurrente ya la adelantó a `row_version = 2`, nuestro `WHERE ... AND row_version = 1` no encuentra ninguna fila que actualizar — `rows_affected()` da cero, y devolvemos `VersionConflict` en vez de fingir éxito. La prueba `concurrent_revocations_from_same_version_conflict_instead_of_overwriting` ejercita justo esto: la primera revocación gana, la segunda (que partió de la misma versión) recibe el conflicto explícito, nunca un éxito silencioso que pisara el cambio ajeno.

**`api_usage_records` es APPEND-ONLY.** Cada fila es UN EVENTO histórico — "esta solicitud pasó, en este momento, con este desenlace" — y un evento del pasado NUNCA cambia. Editar una fila de este ledger sería literalmente reescribir la historia (¿cuántas solicitudes REALMENTE se hicieron el mes pasado, si alguien pudiera borrar las que le convenga?). Por eso usa `event_sequence_id` (una posición monótona en una cola global, `1, 2, 3, ...`) en vez de `row_version`, y la migración además pone triggers que rechazan cualquier intento de `UPDATE`/`DELETE` a nivel de motor — defensa en profundidad, no solo disciplina en el código Rust:

```sql
-- migrations/0014_api_gateway.sql
CREATE TRIGGER IF NOT EXISTS trg_api_usage_records_no_update
BEFORE UPDATE ON api_usage_records
BEGIN
    SELECT RAISE(ABORT, 'api_usage_records is append-only: UPDATE is forbidden');
END;
```

La regla de oro que separa los dos casos, aplicada aquí: **¿esta fila representa "el estado actual de una cosa" (mutable, usa `row_version`) o "un hecho que ocurrió en un instante" (inmutable, usa `event_sequence_id`)?** Una credencial es lo primero (tiene un estado: activa o revocada, AHORA). Una solicitud procesada es lo segundo (ocurrió, con un desenlace, en un momento del pasado que no se puede deshacer). ADR-0141 prohíbe explícitamente mezclar los dos nombres de columna para lo que le corresponde al otro — y esta Story es un ejemplo de manual de por qué la distinción no es arbitraria, sino que refleja qué tipo de dato es cada tabla.

### Por qué el gateway consulta consentimiento ANTES de delegar

El gateway no es solo un candado de autenticación — es la puerta por la que datos y capacidades internas de Drasus salen hacia sistemas de terceros. Aunque un tercero tenga una credencial válida, cupo de rate-limit disponible y el endpoint habilitado en su plan, eso NO basta para autorizar la delegación: falta una cuarta pregunta, completamente distinta de las tres anteriores — "¿el DUEÑO de esta credencial dio su consentimiento para que sus capacidades se expongan así?". Esa pregunta la resuelve `consent-registry` (cimiento #5), y el gateway la consulta usando el puerto REAL, no un valor inventado:

```rust
// crates/shared/src/orchestrator/third_party_api_gateway.rs
let consent_verdict = resolve_consent_verdict(
    pool, clock, &credential.owner_id, API_GATEWAY_CONSENT_DATA_TYPE, current_consent_version,
).await?;
(endpoint_enabled, rate_limit_verdict, consent_verdict.is_covered())
```

"Real" aquí importa mucho: es tentador, cuando una feature aún no existe o está en otra Story, escribir un stub que siempre devuelva `true` ("ya lo conectaremos después") para poder avanzar rápido. El problema es que un stub así esconde exactamente el bug que las pruebas deberían atrapar — todo se ve verde en desarrollo, y el día que se conecta el consentimiento real, aparecen sorpresas. Por eso esta Story consume `resolve_consent_verdict` de verdad, contra la tabla `consent_records` de verdad, y la prueba de cierre lo demuestra sin ningún atajo:

```rust
// crates/shared/src/orchestrator/third_party_api_gateway.rs
#[tokio::test]
async fn handle_gateway_request_denies_without_delegating_when_consent_not_covered() {
    seed_credential(&pool, &clock, "sk-demo-123", 100, &["CERTIFY"]).await;
    // Sin accept_gateway_consent: el dueño no tiene ningún consentimiento.

    let response = handle_gateway_request(&pool, &clock, "sk-demo-123", "CERTIFY", "v2").await.unwrap();

    assert_eq!(response.outcome, GatewayOutcome::Denied);
    assert_eq!(response.delegate_to, None, "sin consentimiento, NUNCA se delega");
}
```

Nótese que esta prueba NO registra ningún consentimiento a propósito — usa el comportamiento de "negar por defecto" que ya vive en `consent-registry` (`resolve_coverage`, STORY-031: sin ningún evento registrado, el veredicto es `NotCovered`). El gateway no reimplementa esa lógica ni la contradice: la HEREDA, consumiéndola por su puerto público. Es el mismo patrón que usó el cimiento #6 (`enriched-domain-events`) al consumir el `ExecutionGate` real de `licensing-system` (#2) en vez de un stub — cada cimiento nuevo consume los puertos REALES de los cimientos ya construidos, nunca los reinventa.

Por eso `decide_gateway_outcome` pone el chequeo de consentimiento como la ÚLTIMA de las cuatro puertas, justo antes de fijar `delegate_to`: es literalmente la última barrera antes de que el gateway decida "sí, voy a exponer esta capacidad hacia afuera".

```rust
// crates/shared/src/domain/third_party_api_gateway.rs
if !consent_covered {
    return ThirdPartyResponse::denied("CONSENT_NOT_COVERED");
}
ThirdPartyResponse { outcome: GatewayOutcome::Allowed, delegate_to: Some(endpoint.to_string()), denial_reason: None }
```

## Trucos de Senior

### Buscar por HASH indexado en vez de recorrer y comparar

Autenticar exige, dado un secreto presentado, encontrar la fila cuyo `credential_hash` coincide. La forma ingenua sería `SELECT * FROM api_credentials` y comparar el hash de cada fila en Rust — un recorrido completo de la tabla en cada autenticación, cada vez más lento a medida que crecen las credenciales emitidas. En cambio, esta Story declara un índice único sobre la columna:

```sql
-- migrations/0014_api_gateway.sql
CREATE UNIQUE INDEX IF NOT EXISTS idx_api_credentials_credential_hash
    ON api_credentials (credential_hash);
```

y la búsqueda se vuelve un `WHERE credential_hash = ?` directo (`find_by_credential_hash`), que SQLite resuelve con el índice en tiempo prácticamente constante, sin importar cuántas credenciales existan. La regla general: si vas a buscar por una columna con frecuencia (y más si esa búsqueda ocurre en cada solicitud, como aquí), esa columna necesita un índice — el hash ya es efectivamente una clave de búsqueda, así que el índice único cumple doble función: unicidad Y velocidad de búsqueda.

### Cortocircuitar trabajo de I/O cuando la primera puerta ya decidió el resultado

`decide_gateway_outcome` (el Core) evalúa las cuatro puertas en orden y la primera que falla decide el resultado — pero es una función PURA, así que evaluarla es barata sin importar cuántos argumentos reciba. El costo real está en CONSEGUIR esos argumentos: contar filas de uso (una consulta SQL) y resolver el consentimiento (otra consulta SQL) cuestan tiempo real de base de datos. Si la autenticación ya falló, seguir adelante y hacer esas dos consultas de todas formas sería trabajo desperdiciado — el resultado de `decide_gateway_outcome` va a ser `Denied` de cualquier forma, sin importar lo que esas consultas devuelvan. Por eso el orquestador solo las ejecuta cuando la autenticación pasó:

```rust
// crates/shared/src/orchestrator/third_party_api_gateway.rs
let (endpoint_enabled, rate_limit_verdict, consent_covered) = if matches!(auth, AuthVerdict::Authenticated) {
    // ... dos consultas a la base de datos ...
} else {
    (false, RateLimitVerdict::Allow, false) // valores irrelevantes: el Core ya va a negar en la puerta 1
};
```

Es una optimización pequeña pero con un principio grande detrás: en un pipeline de validaciones secuenciales, ordena el trabajo BARATO (comparar un hash en memoria) antes que el trabajo CARO (consultar la base de datos), y evita el caro por completo cuando el barato ya decidió el resultado.
