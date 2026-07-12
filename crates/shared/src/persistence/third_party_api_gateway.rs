//! [SHELL] Repositorios de persistencia del Third-Party API Gateway
//! (`docs/features/third-party-api-gateway.md`, ADR-0144 cimiento #8,
//! ADR-0142, ADR-0093, ADR-0137, ADR-0141, ADR-0020, migración
//! `0014_api_gateway.sql`, STORY-035).
//!
//! Envuelve DOS tablas con naturaleza opuesta -- dueño del único I/O de
//! este cimiento (lecturas/escrituras en SQLite, generación de UUIDv7
//! (ADR-0141) y la lectura del puerto [`Clock`]):
//!
//! - [`ApiCredentialRepository`]: `api_credentials`, MUTABLE, con
//!   `row_version` (ADR-0141) -- mismo patrón de concurrencia optimista que
//!   [`crate::persistence::central_identity::AccountRepository`].
//! - [`ApiUsageRepository`]: `api_usage_records`, APPEND-ONLY, con
//!   `event_sequence_id` (ADR-0141) -- mismo patrón de append atómico bajo
//!   `BEGIN IMMEDIATE` + reintento acotado que
//!   [`crate::persistence::usage_metering::UsageRepository`] /
//!   [`crate::persistence::enriched_domain_events::DomainEventRepository`]
//!   (regla "Atomicidad de ledgers append-only", causa raíz DEBT-001).
//!
//! La lógica pura (hash de credencial, autenticación, rate-limit, hash de
//! auditoría encadenado) vive en
//! [`crate::domain::third_party_api_gateway`] -- este módulo solo le da
//! entradas inyectadas y persiste/carga el resultado.

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::audit_log::GENESIS_PREVIOUS_HASH;
use crate::domain::clock::Clock;
use crate::domain::third_party_api_gateway::{
    compute_api_credential_audit_hash, compute_api_usage_audit_hash, CredentialStatus,
    GatewayOutcome,
};

// ── api_credentials (MUTABLE, row_version) ──────────────────────────────────

/// Errores que devuelven las operaciones de [`ApiCredentialRepository`].
#[derive(Debug, thiserror::Error)]
pub enum ApiCredentialRepositoryError {
    /// La operación subyacente de SQLite falló.
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// Una fila de `api_credentials` tenía un `status` fuera de las dos
    /// cadenas canónicas -- error de integridad de datos.
    #[error("status desconocido en la fila '{0}' de api_credentials")]
    UnknownStatus(String),
    /// El JSON de `endpoints_enabled` persistido no pudo parsearse como
    /// `Vec<String>` -- el `CHECK(json_valid(...))` de la migración solo
    /// garantiza JSON sintácticamente válido, no la FORMA esperada.
    #[error("endpoints_enabled no es un JSON válido de tipo array de strings: {0}")]
    InvalidEndpointsJson(String),
    /// Concurrencia optimista (ADR-0141): el UPDATE (revocación) partió de
    /// un `row_version` que ya no es el vigente en disco -- otra escritura
    /// actualizó la fila en el ínterin. La operación NO pisa el cambio
    /// ajeno; quien llama debe releer la fila y reintentar sobre la
    /// versión actual.
    #[error("conflicto de versión en la credencial '{id}': se esperaba row_version {expected}, la fila ya avanzó")]
    VersionConflict { id: String, expected: i64 },
}

/// Una credencial de API nueva para persistir.
#[derive(Debug, Clone)]
pub struct NewApiCredential {
    pub owner_id: String,
    pub access_token_id: Option<String>,
    pub node_id: String,
    /// Hash SHA-256 (hex) ya calculado por
    /// [`crate::domain::third_party_api_gateway::hash_api_credential`] --
    /// este repositorio nunca ve ni calcula el secreto en claro.
    pub credential_hash: String,
    pub rate_limit_per_window: i64,
    pub window_seconds: i64,
    pub endpoints_enabled: Vec<String>,
}

/// Una fila de `api_credentials` ya persistida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiCredentialRow {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub row_version: i64,

    pub owner_id: String,
    pub access_token_id: Option<String>,
    pub node_id: String,

    pub credential_hash: String,
    pub status: CredentialStatus,
    pub rate_limit_per_window: i64,
    pub window_seconds: i64,
    pub endpoints_enabled: Vec<String>,
}

/// Repositorio MUTABLE para `api_credentials`.
///
/// Constrúyelo con un [`SqlitePool`] ya migrado y cualquier implementación
/// de [`Clock`] -- mismo patrón que
/// [`crate::persistence::central_identity::AccountRepository`].
pub struct ApiCredentialRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> ApiCredentialRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`. Ambos se toman
    /// prestados por la vida del repositorio -- no se toma ownership.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Persiste una credencial nueva con `row_version = 1` y `status =
    /// ACTIVE` (una credencial recién emitida nace activa -- no existe un
    /// estado "pendiente" para credenciales de API, a diferencia de la
    /// verificación de correo de `central-identity`).
    pub async fn create(&self, new_credential: NewApiCredential) -> Result<ApiCredentialRow, ApiCredentialRepositoryError> {
        let id = Uuid::now_v7().to_string();
        let now_ns = self.clock.timestamp_ns();
        let row_version: i64 = 1;
        let status = CredentialStatus::Active;
        // Serializa el JSON UNA sola vez -- el mismo string se persiste y
        // entra al hash de auditoría (reproducible desde la fila en disco).
        let endpoints_enabled_json = serde_json::to_string(&new_credential.endpoints_enabled)
            // `Vec<String>` siempre serializa -- no hay forma de que este
            // `to_string` falle con un tipo tan simple.
            .expect("Vec<String> siempre es serializable a JSON");

        let audit_hash = compute_api_credential_audit_hash(
            &id,
            now_ns,
            row_version,
            None,
            &new_credential.owner_id,
            new_credential.access_token_id.as_deref(),
            &new_credential.node_id,
            &new_credential.credential_hash,
            status,
            new_credential.rate_limit_per_window,
            new_credential.window_seconds,
            &endpoints_enabled_json,
        );

        sqlx::query(
            "INSERT INTO api_credentials (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                owner_id, access_token_id, node_id, \
                credential_hash, status, rate_limit_per_window, window_seconds, endpoints_enabled\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(Option::<String>::None)
        .bind(row_version)
        .bind(&new_credential.owner_id)
        .bind(&new_credential.access_token_id)
        .bind(&new_credential.node_id)
        .bind(&new_credential.credential_hash)
        .bind(status.as_str())
        .bind(new_credential.rate_limit_per_window)
        .bind(new_credential.window_seconds)
        .bind(&endpoints_enabled_json)
        .execute(self.pool)
        .await?;

        Ok(ApiCredentialRow {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: None,
            row_version,
            owner_id: new_credential.owner_id,
            access_token_id: new_credential.access_token_id,
            node_id: new_credential.node_id,
            credential_hash: new_credential.credential_hash,
            status,
            rate_limit_per_window: new_credential.rate_limit_per_window,
            window_seconds: new_credential.window_seconds,
            endpoints_enabled: new_credential.endpoints_enabled,
        })
    }

    /// Carga una única credencial por `id`, o `None` si no existe.
    pub async fn find_by_id(&self, id: &str) -> Result<Option<ApiCredentialRow>, ApiCredentialRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    owner_id, access_token_id, node_id, \
                    credential_hash, status, rate_limit_per_window, window_seconds, endpoints_enabled \
             FROM api_credentials WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        row.map(row_to_api_credential).transpose()
    }

    /// Carga una única credencial por su `credential_hash` (índice único
    /// `idx_api_credentials_credential_hash`), o `None` si ninguna
    /// credencial tiene ese hash. Es la vía de autenticación: la Shell
    /// hashea el secreto presentado por el tercero
    /// ([`crate::domain::third_party_api_gateway::hash_api_credential`]) y
    /// busca la fila -- sin este método, autenticar exigiría recorrer TODA
    /// la tabla comparando hashes uno por uno.
    pub async fn find_by_credential_hash(
        &self,
        credential_hash: &str,
    ) -> Result<Option<ApiCredentialRow>, ApiCredentialRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    owner_id, access_token_id, node_id, \
                    credential_hash, status, rate_limit_per_window, window_seconds, endpoints_enabled \
             FROM api_credentials WHERE credential_hash = ?",
        )
        .bind(credential_hash)
        .fetch_optional(self.pool)
        .await?;

        row.map(row_to_api_credential).transpose()
    }

    /// Revoca una credencial (`docs/features/third-party-api-gateway.md`
    /// "Comportamientos Observables": "Cuando la credencial se revoca → el
    /// acceso cesa de inmediato"). Incrementa `row_version` (+1), fija
    /// `status = REVOKED` y encadena `audit_hash`/`audit_chain_hash` --
    /// mismo patrón que
    /// [`crate::persistence::central_identity::AccountRepository::update_email_verification_status`].
    ///
    /// ## Concurrencia optimista (ADR-0141)
    ///
    /// El UPDATE filtra por `id` **y** `row_version = <el de `credential`>`.
    /// Si otra escritura ya avanzó la fila desde que se leyó `credential`
    /// (ej. dos revocaciones concurrentes de la misma credencial), el
    /// `WHERE` no encuentra ninguna fila (`rows_affected() == 0`) y se
    /// devuelve [`ApiCredentialRepositoryError::VersionConflict`] en vez de
    /// pisar el cambio ajeno.
    pub async fn revoke(&self, credential: &ApiCredentialRow) -> Result<ApiCredentialRow, ApiCredentialRepositoryError> {
        let now_ns = self.clock.timestamp_ns();
        let row_version = credential.row_version + 1;
        let new_status = CredentialStatus::Revoked;
        let endpoints_enabled_json = serde_json::to_string(&credential.endpoints_enabled)
            .expect("Vec<String> siempre es serializable a JSON");

        let audit_hash = compute_api_credential_audit_hash(
            &credential.id,
            now_ns,
            row_version,
            Some(&credential.audit_hash),
            &credential.owner_id,
            credential.access_token_id.as_deref(),
            &credential.node_id,
            &credential.credential_hash,
            new_status,
            credential.rate_limit_per_window,
            credential.window_seconds,
            &endpoints_enabled_json,
        );

        // La guarda `row_version = ?` es la comparación optimista: solo
        // actualiza si la fila en disco sigue en la versión que leímos.
        let result = sqlx::query(
            "UPDATE api_credentials SET \
                updated_at = ?, audit_hash = ?, audit_chain_hash = ?, row_version = ?, status = ? \
             WHERE id = ? AND row_version = ?",
        )
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&credential.audit_hash)
        .bind(row_version)
        .bind(new_status.as_str())
        .bind(&credential.id)
        .bind(credential.row_version)
        .execute(self.pool)
        .await?;

        // Cero filas afectadas => la fila ya no está en
        // `credential.row_version` (otra escritura la adelantó, ej. otra
        // revocación concurrente ganó primero). No pisamos: reportamos el
        // conflicto.
        if result.rows_affected() == 0 {
            return Err(ApiCredentialRepositoryError::VersionConflict {
                id: credential.id.clone(),
                expected: credential.row_version,
            });
        }

        Ok(ApiCredentialRow {
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: Some(credential.audit_hash.clone()),
            row_version,
            status: new_status,
            ..credential.clone()
        })
    }
}

/// Convierte una fila de `api_credentials` al tipo [`ApiCredentialRow`].
fn row_to_api_credential(row: sqlx::sqlite::SqliteRow) -> Result<ApiCredentialRow, ApiCredentialRepositoryError> {
    let status_value: String = row.get("status");
    let status = CredentialStatus::from_str_value(&status_value)
        .ok_or_else(|| ApiCredentialRepositoryError::UnknownStatus(status_value))?;

    let endpoints_enabled_json: String = row.get("endpoints_enabled");
    let endpoints_enabled: Vec<String> = serde_json::from_str(&endpoints_enabled_json)
        .map_err(|error| ApiCredentialRepositoryError::InvalidEndpointsJson(error.to_string()))?;

    Ok(ApiCredentialRow {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        row_version: row.get("row_version"),
        owner_id: row.get("owner_id"),
        access_token_id: row.get("access_token_id"),
        node_id: row.get("node_id"),
        credential_hash: row.get("credential_hash"),
        status,
        rate_limit_per_window: row.get("rate_limit_per_window"),
        window_seconds: row.get("window_seconds"),
        endpoints_enabled,
    })
}

// ── api_usage_records (APPEND-ONLY, event_sequence_id) ──────────────────────

/// Errores que devuelven las operaciones de [`ApiUsageRepository`].
#[derive(Debug, thiserror::Error)]
pub enum ApiUsageRepositoryError {
    /// La operación subyacente de SQLite falló.
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// Una fila de `api_usage_records` tenía un `outcome` fuera de las
    /// tres cadenas canónicas -- error de integridad de datos.
    #[error("outcome desconocido en la fila '{0}' de api_usage_records")]
    UnknownOutcome(String),
    /// El registro de uso no pudo completarse tras agotar los reintentos
    /// ante contención de escritura transitoria -- el evento NO se
    /// descartó en silencio (regla "Atomicidad de ledgers append-only",
    /// causa raíz DEBT-001).
    #[error("no se pudo registrar el uso tras {attempts} intentos por contención de escritura")]
    WriteContention { attempts: u32 },
}

/// Número máximo de intentos del append ante contención de escritura
/// transitoria antes de rendirse con
/// [`ApiUsageRepositoryError::WriteContention`]. Mismo valor y misma
/// justificación que
/// [`crate::persistence::usage_metering::MAX_RECORD_OPERATION_ATTEMPTS`] /
/// [`crate::persistence::enriched_domain_events::MAX_RECORD_ATTEMPTS`].
const MAX_RECORD_USAGE_ATTEMPTS: u32 = 5;

/// Decide si un error de [`ApiUsageRepository::record_usage`] es una
/// contención de escritura TRANSITORIA -- mismo criterio que
/// `crate::persistence::usage_metering::is_transient_write_conflict`.
fn is_transient_write_conflict(error: &ApiUsageRepositoryError) -> bool {
    let ApiUsageRepositoryError::Database(sqlx::Error::Database(db)) = error else {
        return false;
    };

    let message = db.message().to_lowercase();
    if message.contains("database is locked") || message.contains("database table is locked") {
        return true;
    }

    db.is_unique_violation() && message.contains("event_sequence_id")
}

/// Entrada para [`ApiUsageRepository::record_usage`] -- todo lo que la
/// Shell necesita para registrar UNA solicitud procesada por el gateway.
/// `owner_id`/`access_token_id`/`node_id` se denormalizan desde la
/// credencial en el momento de la solicitud (ver doc-comment de la
/// migración `0014_api_gateway.sql`).
#[derive(Debug, Clone)]
pub struct RecordApiUsageInput {
    pub owner_id: String,
    pub access_token_id: Option<String>,
    pub node_id: String,
    pub credential_id: String,
    pub endpoint: String,
    pub outcome: GatewayOutcome,
}

/// Una fila de `api_usage_records` ya persistida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiUsageRow {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub event_sequence_id: i64,

    pub owner_id: String,
    pub access_token_id: Option<String>,
    pub node_id: String,

    pub credential_id: String,
    pub endpoint: String,
    pub outcome: GatewayOutcome,
}

/// Repositorio APPEND-ONLY para `api_usage_records`.
///
/// Constrúyelo con un [`SqlitePool`] ya migrado y cualquier implementación
/// de [`Clock`] -- mismo patrón que
/// [`crate::persistence::usage_metering::UsageRepository`].
pub struct ApiUsageRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> ApiUsageRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`. Ambos se toman
    /// prestados por la vida del repositorio -- no se toma ownership.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Cuenta cuántas solicitudes `ALLOWED` de `credential_id` caen en la
    /// ventana `[since_ns, +inf)` (`docs/features/third-party-api-gateway.md`
    /// "Ciclo de Vida" - "Proceso": "verifica rate-limit"). El resultado
    /// alimenta [`crate::domain::third_party_api_gateway::compute_rate_limit`]
    /// como `requests_in_window` -- lectura pura, sin transacción, porque
    /// un conteo levemente desfasado bajo concurrencia extrema solo relaja
    /// el límite en el peor caso (no pierde ni corrompe ningún dato,
    /// distinto del *read-then-write* de `record_usage`).
    pub async fn count_allowed_in_window(
        &self,
        credential_id: &str,
        since_ns: i64,
    ) -> Result<i64, ApiUsageRepositoryError> {
        let row = sqlx::query(
            "SELECT COUNT(*) AS total FROM api_usage_records \
             WHERE credential_id = ? AND outcome = 'ALLOWED' AND created_at >= ?",
        )
        .bind(credential_id)
        .bind(since_ns)
        .fetch_one(self.pool)
        .await?;

        Ok(row.get("total"))
    }

    /// Registra UNA solicitud procesada por el gateway: deriva su posición
    /// en la cadena GLOBAL, computa su hash encadenado y la persiste como
    /// fila nueva.
    ///
    /// Es la ÚNICA forma de escribir en `api_usage_records` -- no existe
    /// `update`/`delete` en esta API; los triggers
    /// `trg_api_usage_records_no_update`/`trg_api_usage_records_no_delete`
    /// de la migración los rechazarían de cualquier forma.
    ///
    /// ## Atomicidad bajo concurrencia (regla "Atomicidad de ledgers append-only")
    ///
    /// El *read-then-write* (leer el MAX(`event_sequence_id`) y el
    /// `audit_hash` previo para encadenar, y el `INSERT` final) ocurre
    /// dentro de UNA sola transacción `BEGIN IMMEDIATE` -- ver
    /// [`Self::try_record_usage_once`]. Ante contención transitoria se
    /// reintenta hasta [`MAX_RECORD_USAGE_ATTEMPTS`] veces re-derivando la
    /// secuencia; el evento NUNCA se descarta en silencio.
    pub async fn record_usage(&self, input: RecordApiUsageInput) -> Result<ApiUsageRow, ApiUsageRepositoryError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_record_usage_once(&input).await {
                Ok(row) => return Ok(row),
                Err(error) => {
                    if is_transient_write_conflict(&error) {
                        if attempt < MAX_RECORD_USAGE_ATTEMPTS {
                            continue;
                        }
                        return Err(ApiUsageRepositoryError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    /// Un intento único del append, dentro de una transacción
    /// `BEGIN IMMEDIATE`. `BEGIN IMMEDIATE` toma el lock de escritura de
    /// ENTRADA: así ningún otro escritor puede intercalar entre la lectura
    /// del MAX(`event_sequence_id`) y el `INSERT`.
    async fn try_record_usage_once(&self, input: &RecordApiUsageInput) -> Result<ApiUsageRow, ApiUsageRepositoryError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        let tail_row = sqlx::query(
            "SELECT audit_hash, event_sequence_id FROM api_usage_records \
             ORDER BY event_sequence_id DESC LIMIT 1",
        )
        .fetch_optional(&mut *tx)
        .await?;

        let (event_sequence_id, audit_chain_hash, previous_audit_hash) = match tail_row {
            Some(row) => {
                let previous_seq: i64 = row.get("event_sequence_id");
                let previous_hash: String = row.get("audit_hash");
                (previous_seq + 1, Some(previous_hash.clone()), previous_hash)
            }
            None => (1, None, GENESIS_PREVIOUS_HASH.to_string()),
        };

        let id = Uuid::now_v7().to_string();
        let now_ns = self.clock.timestamp_ns();

        let audit_hash = compute_api_usage_audit_hash(
            &id,
            now_ns,
            event_sequence_id,
            &previous_audit_hash,
            &input.owner_id,
            input.access_token_id.as_deref(),
            &input.node_id,
            &input.credential_id,
            &input.endpoint,
            input.outcome,
        );

        sqlx::query(
            "INSERT INTO api_usage_records (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, access_token_id, node_id, \
                credential_id, endpoint, outcome\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&audit_chain_hash)
        .bind(event_sequence_id)
        .bind(&input.owner_id)
        .bind(&input.access_token_id)
        .bind(&input.node_id)
        .bind(&input.credential_id)
        .bind(&input.endpoint)
        .bind(input.outcome.as_str())
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(ApiUsageRow {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash,
            event_sequence_id,
            owner_id: input.owner_id.clone(),
            access_token_id: input.access_token_id.clone(),
            node_id: input.node_id.clone(),
            credential_id: input.credential_id.clone(),
            endpoint: input.endpoint.clone(),
            outcome: input.outcome,
        })
    }

    /// Carga la cadena completa, ordenada por `event_sequence_id`
    /// ascendente -- usada por los tests de integridad de la cadena.
    pub async fn load_chain(&self) -> Result<Vec<ApiUsageRow>, ApiUsageRepositoryError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, access_token_id, node_id, credential_id, endpoint, outcome \
             FROM api_usage_records \
             ORDER BY event_sequence_id ASC",
        )
        .fetch_all(self.pool)
        .await?;

        rows.into_iter().map(row_to_api_usage).collect()
    }
}

/// Convierte una fila de `api_usage_records` al tipo [`ApiUsageRow`].
fn row_to_api_usage(row: sqlx::sqlite::SqliteRow) -> Result<ApiUsageRow, ApiUsageRepositoryError> {
    let outcome_value: String = row.get("outcome");
    let outcome = GatewayOutcome::from_str_value(&outcome_value)
        .ok_or_else(|| ApiUsageRepositoryError::UnknownOutcome(outcome_value))?;

    Ok(ApiUsageRow {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        event_sequence_id: row.get("event_sequence_id"),
        owner_id: row.get("owner_id"),
        access_token_id: row.get("access_token_id"),
        node_id: row.get("node_id"),
        credential_id: row.get("credential_id"),
        endpoint: row.get("endpoint"),
        outcome,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::domain::third_party_api_gateway::hash_api_credential;
    use crate::persistence::central_identity::test_support::seed_account;
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    fn sample_new_credential(owner_id: &str, credential_hash: &str) -> NewApiCredential {
        NewApiCredential {
            owner_id: owner_id.to_string(),
            access_token_id: None,
            node_id: "node-1".to_string(),
            credential_hash: credential_hash.to_string(),
            rate_limit_per_window: 100,
            window_seconds: 60,
            endpoints_enabled: vec!["CERTIFY".to_string()],
        }
    }

    // ── CRITERIO #1 (Orden §5): esquema STRICT + Grupo I + row_version ──────

    #[tokio::test]
    async fn migration_creates_api_credentials_table_with_group_i_and_row_version() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('api_credentials')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();

        for expected in [
            "id", "created_at", "updated_at", "audit_hash", "audit_chain_hash", "row_version",
            "owner_id", "access_token_id", "node_id",
            "credential_hash", "status", "rate_limit_per_window", "window_seconds", "endpoints_enabled",
        ] {
            assert!(column_names.contains(&expected.to_string()), "falta la columna: {expected}");
        }

        assert!(
            !column_names.contains(&"event_sequence_id".to_string()),
            "api_credentials es MUTABLE (ADR-0141): no debe tener event_sequence_id, solo row_version"
        );

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'api_credentials'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(sql.contains("STRICT"), "la tabla api_credentials debe declararse STRICT");
    }

    #[tokio::test]
    async fn migration_creates_api_usage_records_table_with_group_i_and_event_sequence_id() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('api_usage_records')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();

        for expected in [
            "id", "created_at", "updated_at", "audit_hash", "audit_chain_hash", "event_sequence_id",
            "owner_id", "access_token_id", "node_id",
            "credential_id", "endpoint", "outcome",
        ] {
            assert!(column_names.contains(&expected.to_string()), "falta la columna: {expected}");
        }

        assert!(
            !column_names.contains(&"row_version".to_string()),
            "api_usage_records es APPEND-ONLY (ADR-0141): no debe tener row_version, solo event_sequence_id"
        );

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'api_usage_records'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(sql.contains("STRICT"), "la tabla api_usage_records debe declararse STRICT");
    }

    // ── CRITERIO #2 (Orden §5): credencial hasheada, nunca en claro ─────────

    #[tokio::test]
    async fn create_persists_only_the_hash_never_the_plaintext_secret() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = ApiCredentialRepository::new(&pool, &clock);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let secret = "sk-super-secret-do-not-leak";
        let stored_hash = hash_api_credential(secret);

        repo.create(sample_new_credential(&owner_id, &stored_hash)).await.expect("crear credencial");

        let raw: String = sqlx::query("SELECT credential_hash FROM api_credentials LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("leer fila cruda")
            .get(0);
        assert_eq!(raw, stored_hash);
        assert_ne!(raw, secret, "la columna jamás debe contener el secreto en claro");
    }

    #[tokio::test]
    async fn find_by_credential_hash_finds_existing_and_returns_none_for_missing() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = ApiCredentialRepository::new(&pool, &clock);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let stored_hash = hash_api_credential("sk-demo-123");

        let created = repo.create(sample_new_credential(&owner_id, &stored_hash)).await.expect("crear credencial");

        let found = repo
            .find_by_credential_hash(&stored_hash)
            .await
            .expect("buscar por hash")
            .expect("debe existir");
        assert_eq!(found.id, created.id);

        let missing = repo
            .find_by_credential_hash("hash-inexistente")
            .await
            .expect("buscar hash inexistente");
        assert_eq!(missing, None);
    }

    // ── CRITERIO #4 (Orden §5): revocación con row_version ──────────────────

    #[tokio::test]
    async fn revoke_increments_row_version_and_sets_status_revoked() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = ApiCredentialRepository::new(&pool, &clock);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let stored_hash = hash_api_credential("sk-demo-123");

        let credential = repo.create(sample_new_credential(&owner_id, &stored_hash)).await.expect("crear credencial");
        assert_eq!(credential.status, CredentialStatus::Active);

        clock.tick();
        let revoked = repo.revoke(&credential).await.expect("revocar credencial");
        assert_eq!(revoked.row_version, 2);
        assert_eq!(revoked.status, CredentialStatus::Revoked);
        assert_eq!(revoked.audit_chain_hash, Some(credential.audit_hash.clone()));

        let reloaded = repo.find_by_id(&credential.id).await.expect("releer").expect("existe");
        assert_eq!(reloaded.status, CredentialStatus::Revoked);
        assert_eq!(reloaded.row_version, 2);
    }

    /// CRITERIO DE CIERRE (Orden §5, criterio #4): dos revocaciones
    /// concurrentes de la MISMA credencial (misma `row_version` en
    /// memoria) -- una gana, la otra `VersionConflict`.
    #[tokio::test]
    async fn concurrent_revocations_from_same_version_conflict_instead_of_overwriting() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = ApiCredentialRepository::new(&pool, &clock);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let stored_hash = hash_api_credential("sk-demo-123");

        let credential = repo.create(sample_new_credential(&owner_id, &stored_hash)).await.expect("crear credencial");
        let first_view = credential.clone();
        let second_view = credential;

        clock.tick();
        let first_result = repo.revoke(&first_view).await.expect("la primera revocación debe tener éxito");
        assert_eq!(first_result.row_version, 2);

        clock.tick();
        let second_result = repo.revoke(&second_view).await;
        assert!(
            matches!(second_result, Err(ApiCredentialRepositoryError::VersionConflict { expected: 1, .. })),
            "la segunda revocación desde la versión 1 debe dar VersionConflict; fue: {second_result:?}"
        );
    }

    // ── CRITERIO #7 (Orden §5): append-only -- UPDATE/DELETE rechazados ─────

    #[tokio::test]
    async fn usage_update_is_rejected_by_trigger() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = ApiUsageRepository::new(&pool, &clock);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;

        let row = repo
            .record_usage(RecordApiUsageInput {
                owner_id,
                access_token_id: None,
                node_id: "node-1".to_string(),
                credential_id: "cred-1".to_string(),
                endpoint: "CERTIFY".to_string(),
                outcome: GatewayOutcome::Allowed,
            })
            .await
            .expect("registrar uso");

        let result = sqlx::query("UPDATE api_usage_records SET outcome = 'DENIED' WHERE id = ?")
            .bind(&row.id)
            .execute(&pool)
            .await;
        assert!(result.is_err(), "UPDATE sobre api_usage_records debe ser rechazado por el trigger");
    }

    #[tokio::test]
    async fn usage_delete_is_rejected_by_trigger() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = ApiUsageRepository::new(&pool, &clock);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;

        let row = repo
            .record_usage(RecordApiUsageInput {
                owner_id,
                access_token_id: None,
                node_id: "node-1".to_string(),
                credential_id: "cred-1".to_string(),
                endpoint: "CERTIFY".to_string(),
                outcome: GatewayOutcome::Allowed,
            })
            .await
            .expect("registrar uso");

        let result = sqlx::query("DELETE FROM api_usage_records WHERE id = ?")
            .bind(&row.id)
            .execute(&pool)
            .await;
        assert!(result.is_err(), "DELETE sobre api_usage_records debe ser rechazado por el trigger");
    }

    // ── CRITERIO #7 (Orden §5): event_sequence_id UNIQUE + audit_chain_hash ─

    #[tokio::test]
    async fn event_sequence_id_is_monotonic_and_chain_is_linked() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = ApiUsageRepository::new(&pool, &clock);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;

        let make_input = |endpoint: &str| RecordApiUsageInput {
            owner_id: owner_id.clone(),
            access_token_id: None,
            node_id: "node-1".to_string(),
            credential_id: "cred-1".to_string(),
            endpoint: endpoint.to_string(),
            outcome: GatewayOutcome::Allowed,
        };

        let first = repo.record_usage(make_input("CERTIFY")).await.expect("primero");
        clock.tick();
        let second = repo.record_usage(make_input("FEED")).await.expect("segundo");

        assert_eq!(first.event_sequence_id, 1);
        assert_eq!(second.event_sequence_id, 2);
        assert_eq!(first.audit_chain_hash, None);
        assert_eq!(second.audit_chain_hash, Some(first.audit_hash));
    }

    #[tokio::test]
    async fn duplicating_event_sequence_id_is_rejected_by_unique_constraint() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = ApiUsageRepository::new(&pool, &clock);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;

        repo.record_usage(RecordApiUsageInput {
            owner_id: owner_id.clone(),
            access_token_id: None,
            node_id: "node-1".to_string(),
            credential_id: "cred-1".to_string(),
            endpoint: "CERTIFY".to_string(),
            outcome: GatewayOutcome::Allowed,
        })
        .await
        .expect("primer registro (event_sequence_id = 1)");

        // owner_id sembrado (FK válida) bindeado vía `?` para que la ÚNICA
        // violación sea el UNIQUE de event_sequence_id, no la FK.
        let duplicate = sqlx::query(
            "INSERT INTO api_usage_records (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, access_token_id, node_id, credential_id, endpoint, outcome\
            ) VALUES ('id-dup', 0, 0, 'hash', NULL, 1, ?, NULL, 'node-1', 'cred-1', 'CERTIFY', 'ALLOWED')",
        )
        .bind(&owner_id)
        .execute(&pool)
        .await;

        assert!(duplicate.is_err(), "duplicar event_sequence_id=1 debe ser rechazado por UNIQUE");
    }

    // ── json_valid ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn database_check_rejects_invalid_json_in_endpoints_enabled() {
        let pool = migrated_pool().await;

        let result = sqlx::query(
            "INSERT INTO api_credentials (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                owner_id, access_token_id, node_id, \
                credential_hash, status, rate_limit_per_window, window_seconds, endpoints_enabled\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', NULL, 'node-1', \
                       'hash-abc', 'ACTIVE', 100, 60, '{not valid json')",
        )
        .execute(&pool)
        .await;

        assert!(result.is_err(), "endpoints_enabled con JSON corrupto debe ser rechazado por el CHECK de la BD");
    }

    #[tokio::test]
    async fn database_check_rejects_unknown_status() {
        let pool = migrated_pool().await;

        let result = sqlx::query(
            "INSERT INTO api_credentials (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                owner_id, access_token_id, node_id, \
                credential_hash, status, rate_limit_per_window, window_seconds, endpoints_enabled\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', NULL, 'node-1', \
                       'hash-abc', 'UNKNOWN', 100, 60, '[]')",
        )
        .execute(&pool)
        .await;

        assert!(result.is_err(), "un status fuera de (ACTIVE, REVOKED) debe ser rechazado por el CHECK de la BD");
    }

    #[tokio::test]
    async fn database_check_rejects_unknown_outcome() {
        let pool = migrated_pool().await;

        let result = sqlx::query(
            "INSERT INTO api_usage_records (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, access_token_id, node_id, credential_id, endpoint, outcome\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', NULL, 'node-1', 'cred-1', 'CERTIFY', 'UNKNOWN')",
        )
        .execute(&pool)
        .await;

        assert!(result.is_err(), "un outcome fuera de (ALLOWED, RATE_LIMITED, DENIED) debe ser rechazado por el CHECK de la BD");
    }

    // ── count_allowed_in_window ──────────────────────────────────────────────

    #[tokio::test]
    async fn count_allowed_in_window_counts_only_allowed_and_only_within_window() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(10_000_000_000, 1); // 10s en ns, paso de 1ns
        let repo = ApiUsageRepository::new(&pool, &clock);
        let owner_id_1 = seed_account(&pool, &clock, "owner1@example.com").await;
        let owner_id_2 = seed_account(&pool, &clock, "owner2@example.com").await;

        // Una solicitud DENIED no debe contar para el rate-limit.
        repo.record_usage(RecordApiUsageInput {
            owner_id: owner_id_1.clone(),
            access_token_id: None,
            node_id: "node-1".to_string(),
            credential_id: "cred-1".to_string(),
            endpoint: "CERTIFY".to_string(),
            outcome: GatewayOutcome::Denied,
        })
        .await
        .expect("registrar denegada");

        // Dos solicitudes ALLOWED de la MISMA credencial.
        for _ in 0..2 {
            repo.record_usage(RecordApiUsageInput {
                owner_id: owner_id_1.clone(),
                access_token_id: None,
                node_id: "node-1".to_string(),
                credential_id: "cred-1".to_string(),
                endpoint: "CERTIFY".to_string(),
                outcome: GatewayOutcome::Allowed,
            })
            .await
            .expect("registrar permitida");
        }

        // Una solicitud ALLOWED de OTRA credencial no debe contarse.
        repo.record_usage(RecordApiUsageInput {
            owner_id: owner_id_2,
            access_token_id: None,
            node_id: "node-1".to_string(),
            credential_id: "cred-2".to_string(),
            endpoint: "CERTIFY".to_string(),
            outcome: GatewayOutcome::Allowed,
        })
        .await
        .expect("registrar permitida de otra credencial");

        let count = repo
            .count_allowed_in_window("cred-1", 0)
            .await
            .expect("contar en ventana");
        assert_eq!(count, 2, "solo las ALLOWED de cred-1 deben contarse");
    }

    // ── Atomicidad bajo concurrencia (regla "Atomicidad de ledgers append-only", DEBT-001) ──

    /// CRITERIO DE CIERRE (Orden §5, criterio #5): 16 escritores
    /// concurrentes sobre `api_usage_records` (archivo temporal, nunca
    /// `:memory:`) -- ninguna fila se pierde y `event_sequence_id` queda
    /// denso 1..=N. Esta prueba DEBE poder caerse si se quita la
    /// transacción `BEGIN IMMEDIATE` (mismo argumento que
    /// `usage_metering`/`enriched_domain_events`).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_record_usage_persists_every_row_without_gaps_or_lost_rows() {
        use std::sync::Arc;

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("api_gateway_concurrency.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());
        let pool = connect(&database_url).await.expect("conectar");
        migrate(&pool).await.expect("migrar");

        let clock: Arc<DeterministicClock> = Arc::new(DeterministicClock::new(1_000, 100));
        // Una sola cuenta sembrada (FK owner_id->accounts): los 16 escritores
        // comparten owner_id -- el ledger es append-only y admite owner
        // repetido.
        let owner_id = seed_account(&pool, clock.as_ref(), "owner-concurrente@example.com").await;

        const N: i64 = 16;

        let mut handles = Vec::new();
        for i in 0..N {
            let pool_c = pool.clone();
            let clock_c = clock.clone();
            let owner_id_c = owner_id.clone();
            handles.push(tokio::spawn(async move {
                let repo = ApiUsageRepository::new(&pool_c, clock_c.as_ref());
                repo.record_usage(RecordApiUsageInput {
                    owner_id: owner_id_c,
                    access_token_id: None,
                    node_id: "node-1".to_string(),
                    credential_id: "cred-1".to_string(),
                    endpoint: format!("ENDPOINT-{i}"),
                    outcome: GatewayOutcome::Allowed,
                })
                .await
            }));
        }

        for handle in handles {
            handle
                .await
                .expect("la tarea no debe entrar en panic")
                .expect("record_usage debe tener éxito para cada escritor concurrente");
        }

        let repo = ApiUsageRepository::new(&pool, clock.as_ref());
        let chain = repo.load_chain().await.expect("cargar la cadena completa");

        assert_eq!(chain.len() as i64, N, "deben persistirse las N filas, sin ninguna pérdida");

        let sequence_ids: Vec<i64> = chain.iter().map(|row| row.event_sequence_id).collect();
        let expected: Vec<i64> = (1..=N).collect();
        assert_eq!(sequence_ids, expected, "los event_sequence_id deben ser 1..=N sin huecos ni duplicados");
    }
}
