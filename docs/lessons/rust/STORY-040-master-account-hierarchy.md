# STORY-040 — Master Account Hierarchy: lecciones de Rust

> **Story:** [STORY-040 — Master Account Hierarchy (cimiento #12, ÚLTIMO del substrato de monetización · jerarquía fondo-hija con override doblemente atestado)](../../execution/STORY-040-master-account-hierarchy.md).
> **Archivos que esta Story produjo y que se citan abajo:** `migrations/0018_master_account_hierarchy.sql`, `crates/shared/src/domain/master_account_hierarchy.rs`, `crates/shared/src/persistence/master_account_hierarchy.rs`, `crates/shared/src/orchestrator/master_account_hierarchy.rs`, `crates/shared/src/public_interface.rs`, `crates/app/src/main.rs`.
> **Modo de Acompañamiento:** Autónomo (la Orden no declaró un Modo distinto en su §3) — este archivo consolida, de todas formas, lo no obvio de la implementación, siguiendo el protocolo de Lecciones (ADR-0122/ADR-0124).

## Concepto

### Reuso del patrón append-only atómico de #10, aplicado a DOS filas por evento en vez de una

Los diez cimientos previos que usan un ledger append-only (`attested_track_records` de #10, `instance_backups` de #11, etc.) resuelven todos el mismo problema: dos escritores concurrentes no pueden derivar el mismo `event_sequence_id` sin que uno pierda su fila. La solución bendecida es envolver el *read-then-write* (leer el `MAX(event_sequence_id)`/`audit_hash` de la cola, e insertar) dentro de una única transacción `BEGIN IMMEDIATE`, con reintento acotado ante contención transitoria.

Este cimiento (`OverrideAttestationRepository::record_attestation`, `crates/shared/src/persistence/master_account_hierarchy.rs`) copia ESE patrón exacto, sin ninguna variación:

```rust
pub async fn record_attestation(
    &self,
    input: RecordOverrideAttestationInput,
) -> Result<OverrideAttestationRow, OverrideAttestationRepositoryError> {
    let mut attempt = 0;
    loop {
        attempt += 1;
        match self.try_record_once(&input).await {
            Ok(row) => return Ok(row),
            Err(error) => {
                if is_transient_write_conflict(&error) {
                    if attempt < MAX_RECORD_ATTEMPTS {
                        continue;
                    }
                    return Err(OverrideAttestationRepositoryError::WriteContention { attempts: attempt });
                }
                return Err(error);
            }
        }
    }
}
```

Lo interesante de #12 no es el patrón en sí (idéntico a #10/#11), sino CÓMO lo usa el orquestador por encima: la "doble atestación" (regla fija #4, ADR-0147) no es una tabla nueva ni una columna nueva — es simplemente **llamar `record_attestation` DOS VECES** sobre la MISMA tabla, una vez con `attestation_side = ISSUER` (`orchestrator::master_account_hierarchy::issue_override`) y otra con `attestation_side = EXECUTOR` (`receive_override`). Como ambas llamadas pasan por el MISMO repositorio append-only atómico, la fila EXECUTOR queda encadenada sobre la ISSUER en el ledger GLOBAL (mismo `event_sequence_id` monótono, mismo `audit_chain_hash`) sin que el orquestador tenga que coordinar nada extra:

```rust
pub async fn execute_override(/* ... */) -> Result<OverrideExecutionResult, MasterAccountHierarchyError> {
    let (issuer, outcome) = issue_override(/* ... */).await?;
    let (executor, _executor_outcome, local_effect) = receive_override(/* ... */).await?;
    Ok(OverrideExecutionResult { issuer, executor, outcome, local_effect })
}
```

El test `execute_override_produces_exactly_one_issuer_and_one_executor_row_when_executed` (`crates/shared/src/orchestrator/master_account_hierarchy.rs`) verifica exactamente esto: `result.executor.audit_chain_hash == Some(result.issuer.audit_hash)` — la cadena de auditoría no distingue "quién" escribió la fila (fondo o hija), solo que cada escritura se encadena atómicamente a la anterior, sea cual sea su origen. Esta es la lección reutilizable: un ledger append-only atómico ya resuelve "doble atestación" gratis en cuanto dos llamadas lógicas distintas comparten el mismo repositorio — no hace falta ningún mecanismo adicional de sincronización entre ISSUER y EXECUTOR.

### El gate de consentimiento como función pura, y por qué NO decide nada por sí sola contra la base de datos

`decide_override_authorization` (`crates/shared/src/domain/master_account_hierarchy.rs`) es intencionalmente una función de una sola línea de lógica:

```rust
pub fn decide_override_authorization(consent: &ConsentVerdict) -> OverrideOutcome {
    match consent {
        ConsentVerdict::Covered => OverrideOutcome::Executed,
        ConsentVerdict::NotCovered(reason) => OverrideOutcome::Denied(format!("{reason:?}")),
    }
}
```

Nótese que esta función **no recibe** un `pool`, un `owner_id`, ni un `data_type` — recibe el `ConsentVerdict` YA resuelto. Esto es FCIS (`base/SKILL.md` §4) llevado a su forma más estricta: la pregunta legal "¿esta hija tiene consentimiento vigente?" es I/O (hay que leer `consent_records` de la base de datos) y por lo tanto es responsabilidad de la Shell (`orchestrator::master_account_hierarchy::issue_override`/`receive_override`, que llaman a `resolve_consent_verdict` de `consent-registry`, #5, ANTES de invocar el gate). La pregunta puramente lógica "dado que YA SÉ el veredicto, ¿qué hago?" es lo único que vive en el Core.

La ventaja práctica de esta separación se ve en los tests: `decide_override_authorization_denies_without_covered_consent` (`domain/master_account_hierarchy.rs`) construye un `ConsentVerdict::NotCovered(...)` A MANO, sin tocar ninguna base de datos, y verifica la decisión en microsegundos. Si el gate hiciera su propia consulta SQL, cada test de esta decisión necesitaría una BD SQLite completa solo para probar un `match` de dos ramas — un desperdicio de tiempo de test y de claridad (mezclaría "¿la lógica es correcta?" con "¿la consulta SQL es correcta?", dos preguntas distintas que merecen tests distintos).

### Por qué "eliminar = archivar" se modela como una función pura con un catálogo de SALIDA que no tiene variante de borrado

La regla fija #5 (ADR-0147/ADR-0141) dice que un `ARCHIVE` nunca puede traducirse a un `DELETE` físico. La forma más débil de cumplir esto sería "recordar no escribir un `DELETE FROM ...` en ningún lado" — una disciplina de convención, fácil de romper sin querer en un cambio futuro. Este cimiento lo hace estructural en cambio, con el catálogo de salida de `apply_local_command_effect`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LocalEffect {
    Archived,
    NoEffect,
}

pub fn apply_local_command_effect(command_kind: OverrideCommandKind, outcome: &OverrideOutcome) -> LocalEffect {
    match (command_kind, outcome) {
        (OverrideCommandKind::Archive, OverrideOutcome::Executed) => LocalEffect::Archived,
        _ => LocalEffect::NoEffect,
    }
}
```

`LocalEffect` solo tiene DOS variantes posibles, y ninguna es "Deleted" o similar. Aunque alguien añadiera código nuevo que intentara devolver un tercer efecto de borrado, **no compilaría** — el `enum` no tiene esa variante para devolver. Esta es la diferencia entre una regla de negocio documentada en un comentario ("no borres nada") y una regla de negocio codificada en el sistema de tipos (Rust literalmente no te deja construir el valor prohibido). El mismo principio protege la tabla `override_attestations` a nivel de esquema: los triggers `trg_override_attestations_no_update`/`_no_delete` (`migrations/0018_master_account_hierarchy.sql`) hacen que CUALQUIER intento de `UPDATE`/`DELETE` sobre esa tabla falle en tiempo de ejecución de SQLite, incluso si algún código futuro se saltara el repositorio y escribiera SQL crudo — dos capas independientes (tipos en Rust + triggers en SQL) para la MISMA invariante.

### Por qué la jerarquía es un puntero cacheado (`account_hierarchy.parent_owner_id`) y no un árbol

La tentación de diseño más común al modelar "un fondo tiene muchas hijas" sería una tabla `hierarchy_tree` con una fila por cada par (ancestro, descendiente) a cualquier profundidad, o peor, una columna `tenant_id`/`org_path` que cada tabla del sistema tendría que llevar para saber "a qué jerarquía pertenezco". La regla fija #1 (ADR-0147) prohíbe exactamente esto: "la hija SOLO cachea `parent_owner_id`, NUNCA el árbol completo — anti-`tenant_id`".

`account_hierarchy` (`migrations/0018_master_account_hierarchy.sql`) reflaja esto en su forma más literal posible: es una tabla de UNA fila por hija (`owner_id TEXT NOT NULL UNIQUE`), con una sola columna `parent_owner_id` que apunta hacia arriba. No existe ninguna columna, índice ni consulta en este cimiento que responda "dame TODAS las hijas de este fondo, recursivamente" — solo existe `idx_account_hierarchy_parent_owner_id`, que responde "dame las hijas DIRECTAS de este fondo" con una consulta explícita, nunca implícita. Si mañana hiciera falta un árbol de varios niveles (un fondo de fondos), esa sería una feature NUEVA que compondría sobre este puntero, no una que este cimiento debería anticipar construyendo de más (`base/SKILL.md` "Principio de Inclusión ante la Duda" se aplica a *campos dentro del perfil declarado*, no a estructuras de datos completas que la Orden no pidió).

La consecuencia práctica: cada owner (fondo o hija) sigue siendo la unidad de aislamiento de datos en TODO el resto del sistema (central-identity, consent-registry, verified-account-registry...) — ninguna tabla ajena a este cimiento necesita saber "a qué jerarquía pertenezco" para funcionar. Solo cuando el flujo de override específicamente lo necesita (`issue_override`/`receive_override`), se consulta el puntero — la jerarquía es una capa ENCIMA del resto del substrato, nunca una dimensión que el resto tuvo que adoptar (regla fija #6: "la hija conserva su Plano de Control").

### Por qué `issue_override` y `receive_override` resuelven el consentimiento CADA UNA por su cuenta, en vez de que una le pase el veredicto a la otra

Podría parecer más simple que `execute_override` resolviera el `ConsentVerdict` UNA sola vez y se lo pasara como parámetro a ambas funciones. Este cimiento decide NO hacer eso a propósito — `receive_override` (`crates/shared/src/orchestrator/master_account_hierarchy.rs`) vuelve a llamar `resolve_consent_verdict` por su cuenta:

```rust
pub async fn receive_override(/* ... */) -> Result<(OverrideAttestationRow, OverrideOutcome, LocalEffect), MasterAccountHierarchyError> {
    // Re-valida el consentimiento LOCALMENTE -- la hija nunca confía
    // ciegamente en el desenlace que el fondo declaró...
    let verdict = resolve_consent_verdict(pool, clock, child_owner_id, /* ... */).await?;
    let outcome = decide_override_authorization(&verdict);
    // ...
}
```

Esto modela la regla fija #6 de forma literal: en producción, el fondo y la hija son DOS procesos en DOS máquinas distintas, comunicadas por el adaptador de red del relé genérico (diferido). La hija JAMÁS podría confiar en un booleano que el fondo le manda diciendo "el consentimiento estaba cubierto, créeme" — sería un vector de falsificación trivial (un fondo malicioso o comprometido podría mentir). La hija tiene que resolver la MISMA pregunta contra SU PROPIA copia de `consent-registry`. En este harness de un solo proceso, ambas llamadas comparten el mismo `pool` (y por tanto ven la misma fila de `consent_records`), así que el resultado siempre coincide — pero la separación de llamadas en el código documenta la intención real, y es la forma correcta de extenderlo el día que el adaptador de red separe ambos lados en procesos de verdad.

## Trucos de Senior

### Derivar una etiqueta persistida SIEMPRE desde el tipo de decisión real, nunca construirla aparte

`OverrideOutcomeLabel` (la columna `outcome`, solo `EXECUTED`/`DENIED`) podría escribirse a mano en cada punto donde se llama a `record_attestation`. En cambio, este cimiento define una conversión `impl From<&OverrideOutcome> for OverrideOutcomeLabel` (`crates/shared/src/domain/master_account_hierarchy.rs`):

```rust
impl From<&OverrideOutcome> for OverrideOutcomeLabel {
    fn from(outcome: &OverrideOutcome) -> Self {
        match outcome {
            OverrideOutcome::Executed => OverrideOutcomeLabel::Executed,
            OverrideOutcome::Denied(_) => OverrideOutcomeLabel::Denied,
        }
    }
}
```

y el orquestador la usa con la sintaxis `.into()`/`OverrideOutcomeLabel::from(&outcome)` en vez de repetir un `match` idéntico en `issue_override` y `receive_override`. La ventaja no es solo evitar duplicación: es IMPOSIBLE que la etiqueta persistida se desincronice del desenlace real que la produjo, porque no hay ningún otro lugar del código donde se construya un `OverrideOutcomeLabel` desde cero — siempre nace de un `OverrideOutcome` ya decidido. Mismo principio que `AttestedTrackRecord::is_attested_by_drasus` de #10, que se deriva SIEMPRE de `AttestationScope::is_sovereign_attestation()` y nunca de un booleano aparte.

### Reutilizar una función `default_*` de otro tipo de verificación CLI cuando el vocabulario es idéntico

`MasterAccountHierarchyVerifyInput::consent` (`crates/shared/src/public_interface.rs`) usa `#[serde(default = "default_verify_consent_state")]` — la MISMA función que ya usa `VerifiedAccountRegistryVerifyInput::consent` un poco más arriba en el mismo archivo, sin redefinirla:

```rust
#[serde(default = "default_verify_consent_state")]
pub consent: String,
```

Como ambos tipos de input viven en el mismo módulo (`public_interface.rs`) y el vocabulario es idéntico (`"COVERED"` | `"OPTED_OUT"` | cualquier otro valor → `"NO_CONSENT"`), no hace falta declarar una segunda función `default_master_account_hierarchy_consent_state` que solo repetiría el mismo `"COVERED".to_string()`. Es un ejemplo pequeño pero real de la regla "no dupliques lo que ya existe" — antes de escribir un helper nuevo, vale la pena revisar si el archivo ya tiene uno con el mismo contrato.
