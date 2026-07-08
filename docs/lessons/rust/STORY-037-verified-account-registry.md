# STORY-037 — Verified Account Registry: lecciones de Rust

> **Story:** [STORY-037 — Verified Account Registry (cimiento #10 y último del substrato de monetización)](../../execution/STORY-037-verified-account-registry.md).
> **Archivos que esta Story produjo y que se citan abajo:** `migrations/0016_verified_account_registry.sql`, `crates/shared/src/domain/verified_account_registry.rs`, `crates/shared/src/persistence/verified_account_registry.rs`, `crates/shared/src/orchestrator/verified_account_registry.rs`, `crates/shared/src/public_interface.rs`, `crates/app/src/main.rs`.
> **Modo de Acompañamiento:** Docente (ADR-0122) — el ingeniero implementó cada bloque por su cuenta y este archivo consolida lo enseñado, con profundidad cero-conocimiento (ADR-0124).

## Concepto

### Qué es la atestación soberana y por qué una cadena de hash prueba "lo ejecutó Drasus" (vs. myFXbook)

myFXbook y MetaTrader 5 Signals publican un track record conectándose **read-only** al bróker: leen el balance/equity que el bróker reporta y confían en que ese número es correcto. Su promesa es "conectamos y no tocamos tu cuenta" — pero NO pueden probar quién generó cada operación ni que el historial no se editó después.

Drasus puede prometer algo más fuerte porque ya tiene, de cimientos anteriores de este mismo substrato, una **cadena de hash encadenada** (el patrón `audit_chain_hash` que aparece en CADA tabla append-only del proyecto: `domain_events`, `generated_reports`, y ahora `attested_track_records`). La idea de una cadena de hash es simple pero potente: cada fila nueva incluye, como parte de lo que se hashea, el hash de la fila anterior. Si alguien intentara editar una fila del medio del historial, su hash cambiaría, y entonces el hash de la fila SIGUIENTE (que depende del anterior) dejaría de coincidir con lo que está grabado — el manipuleo se detecta en cascada, sin tener que confiar en la palabra de nadie.

En este cimiento, la pieza que hace esa promesa es `compute_track_record_audit_hash` (`crates/shared/src/domain/verified_account_registry.rs`):

```rust
pub fn compute_track_record_audit_hash(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    previous_audit_hash: &str,
    owner_id: &str,
    institutional_tag: &str,
    node_id: &str,
    verified_account_id: &str,
    scope: &str,
    time_window: &str,
    signature_hash: &str,
) -> String {
    const SEP: char = '\u{1F}';
    let mut buffer = String::new();
    let mut push = |field: &str| { buffer.push_str(field); buffer.push(SEP); };
    push(id);
    push(&created_at_ns.to_string());
    // ... (resto de campos, incluido previous_audit_hash)
    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    encode_hex(&hasher.finalize())
}
```

El parámetro `previous_audit_hash` es la bisagra: como entra al hash de ESTA fila, cualquier alteración retroactiva de una fila vieja invalida toda la cadena hacia adelante. Es la misma construcción que ya usaba `enriched_domain_events::compute_event_audit_hash` (#6) — este cimiento no inventó la técnica, la reutilizó, porque "cadena de hash" es exactamente lo que myFXbook no tiene: ellos confían en el bróker, Drasus prueba matemáticamente que su propio motor generó y no alteró el historial.

Ojo con una distinción sutil que aparece dos veces en este módulo: `audit_hash`/`audit_chain_hash` protegen la **fila del ledger** (¿se editó esta fila después de escribirse?); `signature_hash` protege el **contenido del track** (¿este track, calculado de nuevo sobre los mismos datos, da lo mismo?). Son preguntas distintas y por eso son dos columnas distintas — se explica en el siguiente concepto.

### Por qué el gain% debe EXCLUIR el flujo de capital (EL diferenciador de esta feature)

Imagina una cuenta con $10,000 de capital propio que en un mes gana $500 de trading, y en ese mismo mes el dueño deposita otros $5,000 (para operar con más margen) y retira $200. Si sumas TODO lo que "entró" a la cuenta ($500 + $5,000 - $200 = $5,300) y lo comparas contra el capital inicial, el resultado es una "ganancia" de 53% — un número absurdo, porque $5,000 no fue ganancia: fue el dueño metiendo SU PROPIO dinero. El track record estaría mintiendo.

La solución no es un `if` que reste el flujo de capital al final — eso sería fácil de romper por accidente. La solución de esta Story es **estructural**: en el catálogo de eventos de dominio (`crates/shared/src/domain/enriched_domain_events.rs`, cimiento #6) el PnL de trading y el movimiento de capital viven en variantes de `enum` COMPLETAMENTE SEPARADAS:

```rust
pub enum EnrichedDomainEvent {
    OrderExecuted(OrderExecutedPayload),   // trae `realized_pnl`
    CapitalFlow(CapitalFlowPayload),        // trae `amount` + `sign` (DEPOSIT/WITHDRAWAL/TRANSFER)
    AccountSnapshot(AccountSnapshotPayload),
    // ...
}
```

Un `enum` en Rust es una unión etiquetada: un valor de tipo `EnrichedDomainEvent` es *o* un `OrderExecuted` *o* un `CapitalFlow`, nunca ambos a la vez. Esto significa que en `compute_track_record` (`crates/shared/src/domain/verified_account_registry.rs`), el `match` que recorre los eventos tiene una rama para cada variante, y es IMPOSIBLE escribir código donde un depósito termine sumándose donde va el PnL — el compilador no te deja mezclar los campos de dos variantes distintas del mismo `match`:

```rust
match event {
    EnrichedDomainEvent::OrderExecuted(p) if p.account_id == account_id => {
        realized_pnls.push(p.realized_pnl);           // SOLO PnL entra aquí
        durations_ns.push(p.duration_ns);
        fill_days.push(p.fill_time_ns.div_euclid(NS_PER_DAY));
    }
    EnrichedDomainEvent::CapitalFlow(p) if p.account_id == account_id => {
        match p.sign {                                  // rama TOTALMENTE separada
            CapitalFlowSign::Deposit => total_deposits += i128::from(p.amount),
            CapitalFlowSign::Withdrawal => total_withdrawals += i128::from(p.amount),
            CapitalFlowSign::Transfer => {}
        }
    }
    _ => {}
}
```

`total_deposits`/`total_withdrawals` se acumulan para REPORTARSE (transparencia: el usuario puede ver cuánto capital metió), pero nunca entran a `total_realized_pnl_e8`, que es la única fuente del numerador del gain%:

```rust
let gain_pct_e8: i64 = if capital_base <= 0 { 0 } else {
    ((total_realized_pnl * 100_000_000) / capital_base) as i64
};
```

El único lugar donde el flujo de capital SÍ participa es como aproximación del **denominador** (`capital_base`) cuando no hay ningún snapshot de cuenta disponible — y ahí participa como "cuánto capital había para trabajar", no como "cuánta ganancia hubo". La prueba discriminante que blinda esto (`crates/shared/src/domain/verified_account_registry.rs`, test `gain_pct_excludes_capital_flow_matching_adr_0145_example_proportions`) calcula el gain% dos veces — una sin eventos de flujo de capital, otra con un depósito de $350 y un retiro de $476.98 añadidos — y exige que el resultado sea EXACTAMENTE el mismo (441%). Si un futuro cambio de código mezclara las dos ramas por error, esta prueba se caería inmediatamente.

### La diferencia entre ámbito SOBERANO y READ-ONLY, y por qué NUNCA se confunden

ADR-0145 exige que cada track quede etiquetado con su **ámbito de atestación**: `SOVEREIGN` (la porción de actividad que ejecutó el propio motor Drasus, con cadena de hash de por medio) o `BROKER_READONLY` (el balance que el bróker reporta, que puede incluir trades manuales u otros programas — Drasus no puede probar eso). La regla de negocio es tajante: JAMÁS se presenta un dato `BROKER_READONLY` con el sello "Ejecución Verificada por Drasus".

La lección de Rust aquí es que una regla de negocio "inviolable" se defiende mejor con **un solo punto de decisión**, no con un booleano que cualquier función pueda fijar a mano. Por eso `AttestationScope` (el `enum` de dos variantes) trae su propio método:

```rust
impl AttestationScope {
    pub fn is_sovereign_attestation(&self) -> bool {
        matches!(self, AttestationScope::Sovereign)
    }
}
```

`matches!` es una macro de Rust que compara un valor contra un patrón y devuelve `true`/`false` — aquí es más corto y más claro que escribir `match self { AttestationScope::Sovereign => true, _ => false }`. Pero lo importante no es la sintaxis: es que esta función es la ÚNICA fuente de verdad. En la proyección hacia el puerto público (`crates/shared/src/persistence/verified_account_registry.rs`, `impl From<&AttestedTrackRecordRow> for AttestedTrackRecord`), el campo visible `is_attested_by_drasus` se deriva SIEMPRE de aquí:

```rust
is_attested_by_drasus: row.scope.is_sovereign_attestation(),
```

Nunca hay un segundo lugar del código que decida "¿esto es soberano?" por su cuenta — si mañana se agregara un tercer ámbito, solo hay que tocar `is_sovereign_attestation` una vez y toda la superficie pública queda correcta. La prueba `attested_track_record_projection_marks_is_attested_only_for_sovereign_scope` construye la MISMA fila con los dos ámbitos posibles y verifica que solo `Sovereign` puede reclamar el sello.

La firma reproducible también refuerza la distinción: `compute_track_record_signature` incluye el `scope` como parte de lo que se hashea, así que el MISMO contenido numérico firmado bajo `SOVEREIGN` da una firma distinta que bajo `BROKER_READONLY` (test `compute_track_record_signature_differs_by_scope`) — ni siquiera dos firmas podrían confundirse por accidente.

### Por qué la publicación es opt-in y consulta el consentimiento REAL (no un stub)

El default de publicación es `PRIVATE`, siempre — ninguna cuenta se hace pública "por accidente". Publicar exige un veredicto de `consent-registry` (#5) que diga `Covered` para el tipo de dato `"verified_account_publication"`. La parte interesante de Rust/diseño aquí es que la decisión SÍ y el efecto secundario de "leer el consentimiento" están en dos funciones distintas — Core puro vs. Shell:

```rust
// Core puro (crates/shared/src/domain/verified_account_registry.rs) -- sin I/O
pub fn decide_publication(
    current_status: PublicationStatus,
    requested_status: PublicationStatus,
    consent: &ConsentVerdict,
) -> PublicationStatus {
    match requested_status {
        PublicationStatus::Private => PublicationStatus::Private,
        PublicationStatus::Public => {
            if consent.is_covered() { PublicationStatus::Public } else { current_status }
        }
    }
}
```

`decide_publication` NUNCA toca la base de datos — recibe el veredicto ya resuelto como parámetro (`&ConsentVerdict`), lo cual la hace 100% testeable sin SQLite (los tests de este módulo la llaman directo con `ConsentVerdict::Covered` o `ConsentVerdict::NotCovered(...)` construidos a mano). El que SÍ hace I/O es el orquestador (`crates/shared/src/orchestrator/verified_account_registry.rs`, `request_publication`), que primero resuelve el veredicto REAL contra la base de datos y luego llama a la función pura:

```rust
let verdict = resolve_consent_verdict(
    pool, clock, &account.owner_id,
    VERIFIED_ACCOUNT_PUBLICATION_CONSENT_DATA_TYPE,
    consent_version,
).await?;
let new_status = decide_publication(account.publication_status, requested_status, &verdict);
```

`resolve_consent_verdict` es la MISMA función que usa `consent-registry` (#5) y que ya usó `data-aggregation` (#9) y `third-party-api-gateway` (#8) — no se reimplementó un chequeo de consentimiento propio para esta feature, se reutilizó el puerto real. La prueba `request_publication_denies_without_any_real_consent` no siembra NINGÚN evento de consentimiento y confirma que, sin stub que "cubra por defecto", el resultado es `Private` — el default-deny de GDPR se respeta de punta a punta.

## Trucos de Senior

- **`i128` como zona de tránsito para evitar overflow, nunca como tipo persistido.** Todas las sumas y multiplicaciones intermedias de `compute_track_record` (sumar PnL de muchos trades, multiplicar por `100_000_000` para reescalar el gain%) se hacen en `i128`, que tiene rango de sobra para no desbordarse aunque los montos en `i64` ya sean grandes, y solo al final se convierte de vuelta a `i64` con `as i64`. Es el mismo patrón que `usage_metering::compute_notional` (#4) usaba para su reescalado ×10¹⁶→×10⁸: usar un tipo más grande *solo* durante el cálculo, y volver al tipo de columna real (`i64`) al guardar.
- **`BTreeSet<&str>` para deduplicar Y ordenar en una sola pasada.** `canonical_attestation_scopes_json` construye un `BTreeSet<&'static str>` a partir de la lista de ámbitos: un `BTreeSet` no permite duplicados (colapsan solos) y siempre itera en orden alfabético — dos beneficios con una sola estructura, sin escribir un `sort()` + `dedup()` manual.
- **`unwrap_or(AttestationScope::BrokerReadonly)` como reconstrucción defensiva, no como atajo perezoso.** Al releer una fila de `attested_track_records`, `row_to_track_record` usa `AttestationScope::from_str_value(&scope_value).unwrap_or(AttestationScope::BrokerReadonly)`: si alguna vez apareciera un valor corrupto en la columna `scope` (algo que el `CHECK` de la migración ya debería impedir), la reconstrucción cae del lado MENOS privilegiado (read-only, nunca soberano) en vez de hacer panic o, peor, asumir `Sovereign` por accidente — un ejemplo de que hasta el "valor por defecto de emergencia" debe respetar la regla de negocio inviolable.

---

# STORY-041 — Retrabajo del Eje B: por qué `institutional_tag` reemplazó a `capital_reality`

> **Story:** [STORY-041 — Consolidación del Eje B en `institutional_tag` (retrabajo de #10, paga DEBT-016)](../../execution/STORY-041-verified-account-eje-b-consolidation.md).
> **Archivos que este retrabajo tocó:** `migrations/0016_verified_account_registry.sql`, `crates/shared/src/domain/verified_account_registry.rs`, `crates/shared/src/persistence/verified_account_registry.rs`, `crates/shared/src/orchestrator/verified_account_registry.rs`, `crates/shared/src/public_interface.rs`.
> **Modo de Acompañamiento:** Autónomo — el retrabajo llegó como Orden de implementación directa del Tech-Lead (auditoría independiente + QA por mutación obligatorios por tocar código sellado). Esta sección documenta el "por qué" de la decisión de diseño para quien la audite después.

## Concepto

### Por qué dos columnas con el mismo vocabulario de valores es un defecto, no solo "deuda estética"

STORY-038 había añadido una columna nueva, `capital_reality` (`LIVE`/`PAPER`/`DEMO`/`CHALLENGE`), a `verified_accounts` y `attested_track_records` para modelar el Eje B (si el capital arriesgado era real o virtual). El problema: esas dos tablas YA tenían `institutional_tag` — un campo del Grupo II ("Soberanía & Propiedad") que el contrato lógico de 25 campos (ADR-0020) exige en su Perfil D, y que en el resto del substrato se usa como una etiqueta de entorno genérica (el placeholder `"DRASUS_LOCAL"` que aparece en casi todas las demás tablas del proyecto).

Cuando dos columnas de la MISMA fila pueden portar el mismo tipo de información — un vocabulario cerrado de valores que clasifica la fila —, el sistema queda con una pregunta sin respuesta única: "¿cuál de las dos es la verdad?". Nada en el código lo impedía, pero era una trampa esperando a pasar: alguien podía escribir `institutional_tag = "PAPER"` y `capital_reality = "LIVE"` en la misma fila sin que ningún `CHECK` lo detectara, porque cada columna se valida por separado. Por eso ADR-0144 (FIJO) exige "reutilización antes que creación": antes de agregar una columna nueva, hay que preguntar si el dato YA tiene un lugar canónico. Aquí lo tenía.

La corrección (ADR-0145, 2026-07-07) fue literal: `institutional_tag`, en ESTAS DOS tablas específicamente, ES el Eje B. Su vocabulario se extendió (`PROD`/`PAPER`/`CHALLENGE` → `LIVE`/`PAPER`/`DEMO`/`CHALLENGE`) y se le agregó el `CHECK` que antes tenía la columna duplicada:

```sql
institutional_tag      TEXT    NOT NULL
    CHECK (institutional_tag IN ('LIVE', 'PAPER', 'DEMO', 'CHALLENGE')),
```

La columna `capital_reality` desapareció de la migración (`migrations/0016_verified_account_registry.sql`) — en fase GREENFIELD (ADR-0006) esto se hace editando el `CREATE TABLE` in situ, sin migración incremental, porque ningún usuario final corre aún una build distribuida.

### Cómo un tipo de dominio puede "interpretar" un campo sin duplicarlo

Lo interesante en Rust es que esta consolidación NO obligó a borrar el tipo `CapitalReality` (`crates/shared/src/domain/verified_account_registry.rs`). El `enum` (`Live`/`Paper`/`Demo`/`Challenge`) sigue existiendo exactamente igual, con sus mismos métodos `as_str()`/`from_str_value()`/`is_real_capital()`. Lo que cambió es de DÓNDE viene el `String` que ese tipo interpreta:

```rust
// Antes (STORY-038): un campo separado en el struct de persistencia.
pub struct VerifiedAccountRow {
    pub institutional_tag: String,     // Grupo II genérico
    pub capital_reality: CapitalReality, // Eje B, columna aparte
    // ...
}

// Ahora (STORY-041): institutional_tag hace las dos cosas a la vez.
pub struct VerifiedAccountRow {
    pub institutional_tag: String, // Grupo II Y Eje B en esta tabla
    // ...
}
```

`CapitalReality` pasó de ser "el tipo de una columna" a ser "una lente tipada sobre `institutional_tag`": en cualquier punto donde el código necesita razonar con la semántica del Eje B (por ejemplo, decidir `is_real_capital`), se llama `CapitalReality::from_str_value(&row.institutional_tag)` en vez de leer un campo `capital_reality` que ya no existe. Esto es lo mismo que hacer un `impl` sobre un tipo prestado: el dato en memoria es un `String` plano, pero el código que lo consume nunca trabaja con strings sueltos — siempre pasa por el `enum` tipado, que sigue dando la misma garantía de "solo cuatro valores válidos, cero strings mágicos sueltos por el código".

La validación de escritura refuerza esta idea. Antes de insertar una cuenta nueva, `VerifiedAccountRepository::create` valida que el `institutional_tag` entrante SÍ pertenezca al vocabulario del Eje B:

```rust
let capital_reality = CapitalReality::from_str_value(&new.institutional_tag)
    .ok_or_else(|| VerifiedAccountRepositoryError::UnknownInstitutionalTag(new.institutional_tag.clone()))?;
```

Esto es defensa en profundidad: el `CHECK` de la migración ya rechazaría un valor inválido a nivel de base de datos, pero fallar ANTES de tocar la BD, con un error tipado y un mensaje claro, es mejor experiencia para quien llama que un error genérico de SQLite. La regla general (que aplica más allá de este cimiento): cuando un campo `String` se reinterpreta como un tipo con vocabulario cerrado, la función que lo escribe debe validar, no asumir.

### Por qué NO fue necesario tocar las funciones de hash/firma

Las funciones puras `compute_track_record_signature`, `compute_track_record_audit_hash` y `compute_verified_account_audit_hash` siguen recibiendo un parámetro `capital_reality` (tipado o como `&str`) exactamente como antes. Esto parece contradecir la idea de "ya no hay columna separada" — pero es intencional: cambiar la firma de una función pura que ya tiene pruebas de determinismo (`compute_track_record_signature_is_reproducible_for_the_same_content`, etc.) habría sido un cambio innecesariamente amplio para un retrabajo que solo debía mover la FUENTE del dato, no su forma. En el sitio de la llamada, `institutional_tag` e `capital_reality` ahora reciben literalmente el MISMO valor de texto:

```rust
push(institutional_tag);   // Grupo II, en el buffer del hash
// ... otros campos ...
push(capital_reality);     // Eje B, MISMO valor que institutional_tag arriba
```

Es una redundancia deliberada y documentada (con comentario explicando la equivalencia), no un descuido: preserva el `audit_hash`/`signature_hash` bit-a-bit para filas ya calculadas con la lógica anterior, y evita reescribir doce sitios de prueba que ya cubrían el comportamiento correcto. La lección de ingeniería: no todo cambio de "de dónde viene el dato" tiene que propagarse a la forma de cada función que lo consume — a veces la frontera correcta para el cambio es el llamador (Shell), no la función pura (Core).

## Trucos de Senior

- **Guardarraíl anti-regresión con `pragma_table_info` + `sqlite_master`.** Para probar "esta columna NO debe existir nunca más", la prueba más directa en SQLite no es tratar de insertar y esperar un error — es preguntarle al catálogo del propio motor. `SELECT name FROM pragma_table_info('tabla')` da la lista real de columnas después de aplicar la migración; comparar esa lista contra lo que se espera (`assert!(!column_names.contains(...))`) hace que la prueba se caiga automáticamente si alguien reintroduce la columna por accidente en el futuro — no depende de que alguien recuerde escribir un test negativo para ESE caso específico.
- **Verificar el `CHECK` leyendo el SQL crudo de `sqlite_master`.** `SELECT sql FROM sqlite_master WHERE name = 'tabla'` devuelve el `CREATE TABLE` completo tal como SQLite lo guardó — sirve para confirmar con un `sql.contains("'LIVE'")` que el `CHECK` correcto está en la columna correcta, sin tener que insertar filas de prueba para cada valor del vocabulario.
