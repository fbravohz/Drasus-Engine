# STORY-043 — Data Portability: lecciones de Rust

> **Story:** [STORY-043 — Data Portability (cimiento #13 del substrato de monetización)](../../execution/STORY-043-data-portability.md).
> **Archivos que esta Story produjo y que se citan abajo:** `migrations/0019_data_portability.sql`, `crates/shared/src/domain/data_portability.rs`, `crates/shared/src/persistence/data_portability.rs`, `crates/shared/src/orchestrator/data_portability.rs`, `crates/shared/src/public_interface.rs`, `crates/app/src/main.rs`.
> **Modo de Acompañamiento:** Docente (STORY-043 declara "Rust-Engineer (Sonnet, Docente)" en su cabecera) — este archivo consolida, siguiendo el protocolo de Lecciones (ADR-0122/ADR-0124), lo no obvio de cada bloque que se implementó.

## Concepto

### Por qué el catálogo es metadato declarativo AUTO-POBLADO, no una auditoría central recurrente

La forma "obvia" de resolver "¿qué tablas del sistema tienen datos de un usuario?" sería un proceso central que, cada cierto tiempo, recorre TODO el esquema SQL (`sqlite_master`), detecta qué tablas tienen una columna `owner_id`, y arma una lista. Ese diseño tiene un problema de acoplamiento invertido: el catálogo terminaría sabiendo más sobre cada Feature que la Feature misma, y cualquier tabla nueva quedaría invisible al catálogo hasta la próxima corrida del auditor.

`exportable_data_catalog` (`migrations/0019_data_portability.sql`) resuelve esto al revés: es una tabla MUTABLE donde CADA Feature se auto-declara, la misma idea que `foundation_master_fields` de la migración `0001` (metadato de esquema, no hecho de negocio). `ExportableDataCatalogRepository::declare_table` (`crates/shared/src/persistence/data_portability.rs`) es la operación que una Feature invoca sobre sí misma:

```rust
pub async fn declare_table(
    &self,
    new: NewCatalogEntry,
) -> Result<ExportableDataCatalogRow, ExportableDataCatalogRepositoryError> {
    if let Some(existing) = self.find_by_table_name(&new.table_name).await? {
        return Ok(existing);
    }
    // ... INSERT con row_version = 1
}
```

El `if let Some(existing) = ...` al inicio es la pieza que hace la auto-declaración SEGURA de invocar tantas veces como se quiera: como cada Feature (o el arranque de la app) puede llamar `declare_table` en cualquier momento sin coordinarse con nadie más, la operación tiene que ser IDEMPOTENTE — si ya existe una fila con ese `table_name`, la segunda llamada no debe duplicar nada, solo devolver lo que ya había. El test `declare_table_is_idempotent_by_table_name` (`crates/shared/src/persistence/data_portability.rs`) verifica exactamente esto: llamar dos veces con el mismo `NewCatalogEntry` deja UNA sola fila en la tabla.

`seed_known_catalog` (`crates/shared/src/orchestrator/data_portability.rs`) es el stub que demuestra el mecanismo con las tablas ya conocidas del substrato — es un `for` que llama `declare_exportable_table` (que envuelve `declare_table`) por cada entrada de una lista fija `KNOWN_CATALOG_TABLES`. El día que exista un mecanismo real de auto-registro (cada Feature llamando `declare_table` en su propio arranque), este `seed` desaparece sin que el resto del cimiento cambie — la interfaz (`declare_table` idempotente) ya es la correcta.

### El olvido como pseudonimización sobre DELETE — la MISMA técnica de "catálogo cerrado sin variante de borrado" que #12, aplicada a un dominio distinto

STORY-040 (#12, `master-account-hierarchy`) ya enseñó la técnica: modelar "no debe existir un DELETE" no como una disciplina de convención ("recuerda no escribir `DELETE FROM`"), sino como un tipo de Rust cuyo catálogo de variantes NO tiene la opción prohibida. Este cimiento reutiliza la MISMA técnica para el olvido GDPR (Art. 17):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ForgetDisposition {
    PseudonymizeAndRetain,
    PseudonymizeAndPurge,
}

pub fn decide_forget_disposition(retention_exempt: bool) -> ForgetDisposition {
    if retention_exempt {
        ForgetDisposition::PseudonymizeAndRetain
    } else {
        ForgetDisposition::PseudonymizeAndPurge
    }
}
```

(`crates/shared/src/domain/data_portability.rs`). `ForgetDisposition` tiene EXACTAMENTE dos variantes, y las dos son transiciones de estado ("retener" o "purgar contenido no-esencial"), nunca un borrado de fila. Igual que `LocalEffect` en #12, si alguien intentara añadir una tercera variante `Delete` al `enum`, todo el código que hace `match` sobre `ForgetDisposition` (incluida la serialización JSON del `disposition_detail`) tendría que actualizarse explícitamente — el compilador no permite "olvidarse" de un caso nuevo en un `match` exhaustivo, así que la regla queda protegida estructuralmente, no solo documentada.

La diferencia interesante frente a #12 es que aquí la decisión NO es binaria a nivel de UNA fila de negocio, sino que se aplica a CADA tabla del catálogo — `build_forget_disposition_detail` recorre TODO el catálogo y aplica `decide_forget_disposition` tabla por tabla:

```rust
pub fn build_forget_disposition_detail(catalog_entries: &[CatalogEntry]) -> Vec<TableDispositionEntry> {
    let mut entries: Vec<TableDispositionEntry> = catalog_entries
        .iter()
        .map(|entry| TableDispositionEntry {
            table_name: entry.table_name.clone(),
            feature_name: entry.feature_name.clone(),
            disposition: decide_forget_disposition(entry.retention_exempt),
        })
        .collect();
    entries.sort_by(|a, b| a.table_name.cmp(&b.table_name));
    entries
}
```

El `.sort_by(...)` al final NO es cosmético: es lo que hace que el JSON resultante (`disposition_detail_to_json`) sea DETERMINISTA — el mismo catálogo, recorrido en cualquier orden de lectura de la base de datos, produce SIEMPRE el mismo texto JSON. Esto importa porque `disposition_detail` entra al cálculo del `audit_hash` de la solicitud (`compute_request_audit_hash`): si el orden no fuera determinista, dos ejecuciones idénticas del mismo olvido podrían producir `audit_hash` distintos, rompiendo la promesa de reproducibilidad de ADR-0002.

### Reuso del patrón append-only atómico de #10/#12, aplicado a un ledger de SOLICITUDES en vez de atestaciones o tracks

`DataPortabilityRequestRepository::record_event` (`crates/shared/src/persistence/data_portability.rs`) copia, sin ninguna variación, el patrón que ya cerraron #10 (`AttestedTrackRecordRepository`) y #12 (`OverrideAttestationRepository`): un *read-then-write* (leer el `MAX(event_sequence_id)`/`audit_hash` de la cola, e insertar la fila nueva) envuelto en una única transacción `BEGIN IMMEDIATE`, con reintento acotado (`MAX_RECORD_ATTEMPTS = 5`) ante contención transitoria (`database is locked` o una violación del `UNIQUE` sobre `event_sequence_id`):

```rust
pub async fn record_event(
    &self,
    input: RecordDataPortabilityRequestInput,
) -> Result<DataPortabilityRequestRow, DataPortabilityRequestRepositoryError> {
    let mut attempt = 0;
    loop {
        attempt += 1;
        match self.try_record_once(&input).await {
            Ok(row) => return Ok(row),
            Err(error) => {
                if is_transient_write_conflict(&error) {
                    if attempt < MAX_RECORD_ATTEMPTS { continue; }
                    return Err(DataPortabilityRequestRepositoryError::WriteContention { attempts: attempt });
                }
                return Err(error);
            }
        }
    }
}
```

Lo que SÍ es distinto de #10/#12 es cómo el orquestador usa `request_group_id`: cada solicitud lógica (una llamada a `request_export` o `request_forget`) genera un `Uuid::now_v7()` FRESCO como `request_group_id`, y ese mismo valor viajaría en los eventos POSTERIORES de esa misma solicitud (RECEIVED → PROCESSING → COMPLETED) el día que el adaptador diferido emita esos avances. El `event_sequence_id` sigue siendo GLOBAL (monótono a través de TODAS las solicitudes de TODOS los titulares — es la posición en el ledger completo), mientras que `request_group_id` es la clave que agrupa "los eventos de ESTA solicitud en particular". `latest_status_for` (`crates/shared/src/persistence/data_portability.rs`) es la consulta que aprovecha esa distinción:

```rust
pub async fn latest_status_for(
    &self,
    request_group_id: &str,
) -> Result<Option<RequestStatus>, DataPortabilityRequestRepositoryError> {
    let row = sqlx::query(
        "SELECT status FROM data_portability_requests \
         WHERE request_group_id = ? \
         ORDER BY event_sequence_id DESC LIMIT 1",
    )
    // ...
}
```

`ORDER BY event_sequence_id DESC LIMIT 1` filtrado por `request_group_id` es la única forma correcta de leer "en qué estado está esta solicitud AHORA" en un modelo donde el avance de estado NUNCA es un `UPDATE` (el trigger `trg_data_portability_requests_no_update` de la migración lo rechazaría) sino SIEMPRE una fila nueva — el mismo patrón de "estado vigente = el evento con mayor secuencia" que `consent-registry` (#5) usa para resolver el consentimiento vigente de un `owner_id`.

## Trucos de Senior

### Convertir la fila persistida al tipo del Core con `impl From<&Row> for CoreType`, para que el Core nunca vea SQL

`ExportableDataCatalogRow` (persistencia, con `id`/`row_version`/`audit_hash`) y `CatalogEntry` (dominio, solo `table_name`/`feature_name`/`owner_id_column`/`retention_exempt`) son deliberadamente tipos DISTINTOS, con una conversión explícita:

```rust
impl From<&ExportableDataCatalogRow> for CatalogEntry {
    fn from(row: &ExportableDataCatalogRow) -> Self {
        CatalogEntry {
            table_name: row.table_name.clone(),
            feature_name: row.feature_name.clone(),
            owner_id_column: row.owner_id_column.clone(),
            retention_exempt: row.retention_exempt,
        }
    }
}
```

(`crates/shared/src/persistence/data_portability.rs`). Esto permite que `load_catalog_entries` (`crates/shared/src/orchestrator/data_portability.rs`) escriba `rows.iter().map(CatalogEntry::from).collect()` en vez de un `.map(|r| CatalogEntry { table_name: r.table_name.clone(), ... })` repetido en cada punto donde el orquestador necesita pasarle el catálogo al Core. La ventaja de fondo (no solo menos código): el Core (`build_export_manifest`, `build_forget_disposition_detail`) queda estructuralmente incapaz de leer `audit_hash` o `row_version` de una fila — ese tipo ni siquiera existe en su firma — reforzando en el compilador la regla FCIS de que la lógica pura no conoce el esquema SQL, solo conoce su propio vocabulario de dominio.

### Reutilizar `#[allow(clippy::too_many_arguments)]` con criterio, no como supresión automática

`compute_request_audit_hash` y `compute_catalog_audit_hash` (`crates/shared/src/domain/data_portability.rs`) llevan `#[allow(clippy::too_many_arguments)]` porque el hash de auditoría, por diseño (ADR-0020/ADR-0141), tiene que incorporar CADA campo persistido de la fila — reducir la cuenta de argumentos agrupándolos en un `struct` intermedio solo movería el problema (el `struct` tendría los mismos campos) sin ganar nada, y el resto del substrato (`compute_override_audit_hash` de #12, `compute_track_record_audit_hash` de #10) ya toma la misma decisión documentada. La lección no es "usa `#[allow]` para silenciar clippy" — es reconocer CUÁNDO una advertencia de estilo (número de argumentos) choca con una restricción de dominio real (todos los campos deben entrar al hash) y documentar la excepción en vez de retorcer el diseño para complacer al linter.

## Endurecimiento por mutación: 3 tests que matan lo que "verde + cobertura" no vigila (cierre TL, 2026-07-08)

La primera entrega pasaba 33 tests con cobertura alta, pero `cargo-mutants` dejó **11 sobrevivientes**: líneas de correctitud que ninguna prueba vigilaba. Todos vivían en el patrón de ledger append-only (no en el dominio puro, que ya estaba bien cubierto). El Tech-Lead los mató con tres pruebas deterministas — el patrón vale para CUALQUIER ledger del substrato (por eso quedó como regla en ADR-0133 y en los skills, y como DEBT-018 para los cimientos previos).

### 1. Contención sostenida hasta agotar reintentos (mata el bucle `record_event`)

El bucle `attempt += 1` / `if attempt < MAX { continue }` solo es observable en la ruta de **agotamiento**: un test que asume "todas las escrituras tienen éxito" nunca lo ejerce, porque el número de intentos jamás se lee en el camino feliz. La clave es forzar contención **determinista** (no depender del azar del scheduler de 16 escritores): un segundo escritor retiene el lock de escritura y NO lo suelta, y ambas conexiones usan `busy_timeout(Duration::from_millis(0))` para que el choque falle de INMEDIATO en vez de esperar 5s×5.

```rust
let immediate_opts = || SqliteConnectOptions::from_str(&database_url).unwrap()
    .journal_mode(SqliteJournalMode::Wal)
    .busy_timeout(Duration::from_millis(0)); // choque inmediato, no espera

let lock_pool = SqlitePoolOptions::new().max_connections(1).connect_with(immediate_opts()).await.unwrap();
let lock_tx = lock_pool.begin_with("BEGIN IMMEDIATE").await.unwrap(); // retiene el write-lock

// El repo (otra conexión) intenta escribir mientras el lock está tomado:
let result = repo.record_event(...).await;
drop(lock_tx);
assert!(matches!(result, Err(WriteContention { attempts }) if attempts == MAX_RECORD_ATTEMPTS));
```

Esto mata cuatro mutantes de golpe: `attempt += 1`→`*=` (el contador se congela en 0 → bucle infinito → `cargo-mutants` lo captura por **timeout**), y `attempt < MAX`→`==`/`>`/`<=` (cambian el punto de corte, y la aserción `attempts == MAX` los delata). De paso mata `is_transient→false` y el `||→&&` de la rama de lock, porque si el clasificador dejara de reconocer "database is locked" el resultado sería el error crudo, no `WriteContention`.

### 2. Clasificador directo con una violación UNIQUE PERMANENTE (mata `is_transient_write_conflict`)

`is_transient_write_conflict` distingue una colisión transitoria (UNIQUE sobre `event_sequence_id`, se reintenta) de un error permanente (cualquier otra violación UNIQUE, NO se reintenta). Para fijar que exige **ambas** condiciones (`is_unique() && contains("event_sequence_id")`, no una sola), hay que pasarle un error real cuyo mensaje UNIQUE NO mencione `event_sequence_id` — se consigue violando la PK `id`:

```rust
// dos INSERT con el MISMO id (viola PRIMARY KEY id) pero distinto event_sequence_id:
insert_with_id_dup(1).await.unwrap();
let err = insert_with_id_dup(2).await.unwrap_err();
assert!(!is_transient_write_conflict(&DataPortabilityRequestRepositoryError::Database(err)));
```

Mata `is_transient→true` (clasificaría este error permanente como transitorio) y `&&→||` (con `||`, `is_unique() || ...` daría `true`). Como `is_transient_*` es privada, el test vive en el mismo módulo (`mod tests` con `use super::*`) — no hace falta exponerla.

### 3. Fidelidad de la fila DEVUELTA en tablas mutables (mata el borrado de campo en la proyección)

`reclassify` persiste bien en la base, pero además DEVUELVE una fila en memoria construida con `..current.clone()`. Un mutante que borra `audit_hash:` de esa proyección hace que la fila devuelta traiga el hash **viejo** — invisible para un test que solo verifica lo persistido (releyendo de la base). La cura es afirmar sobre lo que la función retorna:

```rust
let updated = repo.reclassify(&entry, true).await.unwrap();
assert_ne!(updated.audit_hash, entry.audit_hash);                     // recomputado, no el viejo
assert_eq!(updated.audit_chain_hash, Some(entry.audit_hash.clone())); // encadena a la versión previa
assert_eq!(updated.updated_at_ns, 1_100);                             // now del reloj tras el tick
```

Regla general que deja este episodio: **si una función construye su valor de retorno con `..base.clone()`, cada campo que sobrescribe necesita una aserción sobre el valor devuelto** — releer de la base no basta, porque la base y la proyección en memoria son dos caminos distintos y el mutante vive en el segundo.
