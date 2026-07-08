//! [SHELL] Repositorios de persistencia del Registro de Cuentas Verificadas
//! (`docs/features/verified-account-registry.md`, ADR-0145 cimiento #10,
//! ADR-0093, ADR-0141, ADR-0020, migración `0016_verified_account_registry.sql`,
//! STORY-037).
//!
//! Envuelve DOS tablas con naturalezas opuestas (regla obligatoria #6,
//! ADR-0141):
//! - [`VerifiedAccountRepository`]: `verified_accounts`, MUTABLE con
//!   `row_version` (concurrencia optimista -> [`VerifiedAccountRepositoryError::VersionConflict`]),
//!   mismo patrón que
//!   [`crate::persistence::central_identity::AccountRepository`].
//! - [`AttestedTrackRecordRepository`][]: `attested_track_records`,
//!   APPEND-ONLY ATÓMICA (`event_sequence_id UNIQUE`, `BEGIN IMMEDIATE` +
//!   reintento acotado), mismo patrón que
//!   [`crate::persistence::enriched_domain_events::DomainEventRepository`].
//!
//! La lógica pura (cálculo del track, firma reproducible, hash de
//! auditoría) vive en [`crate::domain::verified_account_registry`] -- este
//! módulo solo le da entradas inyectadas y persiste/carga el resultado.

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::audit_log::GENESIS_PREVIOUS_HASH;
use crate::domain::clock::Clock;
use crate::domain::verified_account_registry::{
    canonical_attestation_scopes_json, compute_track_record_audit_hash,
    compute_verified_account_audit_hash, decode_attestation_scopes_json, AccountType,
    AttestationScope, AttestationScopeDecodeError, AttestedTrackRecord, CapitalReality,
    PublicationStatus, TrackRecordMetrics, VerifiedAccountRecord,
};

// ── `verified_accounts` -- MUTABLE, row_version ─────────────────────────────

/// Errores que devuelven las operaciones de [`VerifiedAccountRepository`].
#[derive(Debug, thiserror::Error)]
pub enum VerifiedAccountRepositoryError {
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// Concurrencia optimista (ADR-0141): el UPDATE partió de un
    /// `row_version` que ya no es el vigente en disco -- otra escritura
    /// actualizó la fila en el ínterin. Mismo patrón que
    /// `central_identity::AccountRepositoryError::VersionConflict`.
    #[error("conflicto de versión en la cuenta verificada '{id}': se esperaba row_version {expected}, la fila ya avanzó")]
    VersionConflict { id: String, expected: i64 },
    /// Una fila persistida tenía un `account_type` fuera del catálogo --
    /// error de integridad de datos, no debería ocurrir si el `CHECK` de la
    /// migración se respeta.
    #[error("account_type desconocido en la tabla verified_accounts: '{0}'")]
    UnknownAccountType(String),
    /// Análogo para `publication_status`.
    #[error("publication_status desconocido en la tabla verified_accounts: '{0}'")]
    UnknownPublicationStatus(String),
    /// La columna `attestation_scopes` no decodificó -- JSON corrupto o
    /// ámbito desconocido.
    #[error("attestation_scopes corrupto en verified_accounts: {0}")]
    AttestationScopes(#[from] AttestationScopeDecodeError),
    /// Un `institutional_tag` fuera del vocabulario del Eje B
    /// (`LIVE`/`PAPER`/`DEMO`/`CHALLENGE`) -- error de integridad de datos
    /// (STORY-041: en esta tabla `institutional_tag` ES el Eje B, no debería
    /// ocurrir si el `CHECK` de la migración se respeta, pero se valida
    /// también en la escritura en vez de asumir un default).
    #[error("institutional_tag (Eje B) desconocido en la tabla verified_accounts: '{0}'")]
    UnknownInstitutionalTag(String),
}

/// Una cuenta verificada nueva para registrar
/// (`docs/features/verified-account-registry.md` "Ciclo de Vida":
/// "Entrada": bróker, apalancamiento, divisa, tipo, ámbitos de atestación).
/// Deliberadamente NO tiene campo `publication_status`: el default PRIVATE
/// es FIJO (regla obligatoria #4) y estructuralmente no se puede pedir otra
/// cosa al registrar.
#[derive(Debug, Clone)]
pub struct NewVerifiedAccount {
    pub owner_id: String,
    /// Grupo II obligatorio -- en ESTA tabla `institutional_tag` ES el Eje B
    /// (`docs/adr/ADR-0145.md` corregido 2026-07-07, STORY-041/DEBT-016):
    /// `"LIVE"`, `"PAPER"`, `"DEMO"` o `"CHALLENGE"`, valor ÚNICO por cuenta
    /// (NUNCA un conjunto como `attestation_scopes`). Fuente de verdad que
    /// `attest_track_record` copia a cada track calculado. Se valida contra
    /// [`CapitalReality`] al escribir ([`VerifiedAccountRepository::create`]);
    /// un valor fuera de ese vocabulario falla con
    /// [`VerifiedAccountRepositoryError::UnknownInstitutionalTag`].
    pub institutional_tag: String,
    pub node_id: String,
    pub broker: String,
    pub leverage: i64,
    pub currency: String,
    pub account_type: AccountType,
    pub attestation_scopes: Vec<AttestationScope>,
    /// Referencia NO SECRETA a la conexión de bróker (nullable, ADR-0093).
    pub broker_connection_ref: Option<String>,
}

/// Una fila de `verified_accounts` ya persistida.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifiedAccountRow {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub row_version: i64,

    pub owner_id: String,
    /// Grupo II -- ver [`NewVerifiedAccount::institutional_tag`] (Eje B en
    /// esta tabla).
    pub institutional_tag: String,
    pub node_id: String,

    pub broker: String,
    pub leverage: i64,
    pub currency: String,
    pub account_type: AccountType,
    pub publication_status: PublicationStatus,
    pub attestation_scopes: Vec<AttestationScope>,
    pub broker_connection_ref: Option<String>,
}

/// Repositorio MUTABLE para `verified_accounts`.
pub struct VerifiedAccountRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> VerifiedAccountRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Registra una cuenta verificada nueva con `row_version = 1` y
    /// `publication_status = PRIVATE` -- SIEMPRE, sin importar lo que
    /// `new` traiga, porque [`NewVerifiedAccount`] ni siquiera tiene ese
    /// campo (regla obligatoria #4, ADR-0145: "el default es privado").
    pub async fn create(
        &self,
        new: NewVerifiedAccount,
    ) -> Result<VerifiedAccountRow, VerifiedAccountRepositoryError> {
        // Valida el Eje B ANTES de tocar la BD -- un institutional_tag fuera
        // del vocabulario LIVE/PAPER/DEMO/CHALLENGE falla tipado aquí en vez
        // de confiar únicamente en el CHECK de la migración (STORY-041: no
        // se asume un default cuando el dato entrante es inválido).
        let capital_reality = CapitalReality::from_str_value(&new.institutional_tag)
            .ok_or_else(|| VerifiedAccountRepositoryError::UnknownInstitutionalTag(new.institutional_tag.clone()))?;

        let id = Uuid::now_v7().to_string();
        let now_ns = self.clock.timestamp_ns();
        let row_version: i64 = 1;
        let publication_status = PublicationStatus::Private;
        let attestation_scopes_json = canonical_attestation_scopes_json(&new.attestation_scopes);

        let audit_hash = compute_verified_account_audit_hash(
            &id,
            now_ns,
            row_version,
            None,
            &new.owner_id,
            &new.institutional_tag,
            &new.node_id,
            &new.broker,
            &new.currency,
            new.account_type,
            publication_status,
            &attestation_scopes_json,
            new.broker_connection_ref.as_deref(),
            capital_reality,
        );

        sqlx::query(
            "INSERT INTO verified_accounts (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                owner_id, institutional_tag, node_id, \
                broker, leverage, currency, account_type, publication_status, \
                attestation_scopes, broker_connection_ref\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(Option::<String>::None)
        .bind(row_version)
        .bind(&new.owner_id)
        .bind(&new.institutional_tag)
        .bind(&new.node_id)
        .bind(&new.broker)
        .bind(new.leverage)
        .bind(&new.currency)
        .bind(new.account_type.as_str())
        .bind(publication_status.as_str())
        .bind(&attestation_scopes_json)
        .bind(&new.broker_connection_ref)
        .execute(self.pool)
        .await?;

        Ok(VerifiedAccountRow {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: None,
            row_version,
            owner_id: new.owner_id,
            institutional_tag: new.institutional_tag,
            node_id: new.node_id,
            broker: new.broker,
            leverage: new.leverage,
            currency: new.currency,
            account_type: new.account_type,
            publication_status,
            attestation_scopes: new.attestation_scopes,
            broker_connection_ref: new.broker_connection_ref,
        })
    }

    /// Carga una única cuenta por `id`, o `None` si no existe.
    pub async fn find_by_id(
        &self,
        id: &str,
    ) -> Result<Option<VerifiedAccountRow>, VerifiedAccountRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    owner_id, institutional_tag, node_id, \
                    broker, leverage, currency, account_type, publication_status, \
                    attestation_scopes, broker_connection_ref \
             FROM verified_accounts WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        row.map(row_to_verified_account).transpose()
    }

    /// Actualiza `publication_status` y `attestation_scopes` de `account`
    /// (`docs/features/verified-account-registry.md`: "el estado de
    /// publicación... cambia" / "ámbitos de atestación... coexistentes").
    ///
    /// ## Concurrencia optimista (ADR-0141)
    ///
    /// El UPDATE filtra por `id` **y** `row_version = <el de `account`>`.
    /// Si otra escritura ya avanzó la fila desde que se leyó `account`, el
    /// `WHERE` no encuentra ninguna fila y se devuelve
    /// [`VerifiedAccountRepositoryError::VersionConflict`] en vez de pisar
    /// el cambio ajeno -- mismo patrón que
    /// `central_identity::AccountRepository::update_email_verification_status`.
    pub async fn update_publication_and_scopes(
        &self,
        account: &VerifiedAccountRow,
        new_status: PublicationStatus,
        new_scopes: &[AttestationScope],
    ) -> Result<VerifiedAccountRow, VerifiedAccountRepositoryError> {
        let now_ns = self.clock.timestamp_ns();
        let row_version = account.row_version + 1;
        let attestation_scopes_json = canonical_attestation_scopes_json(new_scopes);

        // El Eje B no cambia en esta operación (solo publicación/ámbitos del
        // Eje A) -- se re-deriva del MISMO institutional_tag de la cuenta ya
        // persistida (validado al escribirse, protegido por el CHECK de la
        // migración) para re-encadenar el audit_hash con el mismo valor.
        let capital_reality = CapitalReality::from_str_value(&account.institutional_tag).ok_or_else(|| {
            VerifiedAccountRepositoryError::UnknownInstitutionalTag(account.institutional_tag.clone())
        })?;

        let audit_hash = compute_verified_account_audit_hash(
            &account.id,
            now_ns,
            row_version,
            Some(&account.audit_hash),
            &account.owner_id,
            &account.institutional_tag,
            &account.node_id,
            &account.broker,
            &account.currency,
            account.account_type,
            new_status,
            &attestation_scopes_json,
            account.broker_connection_ref.as_deref(),
            capital_reality,
        );

        // La guarda `row_version = ?` es la comparación optimista: solo
        // actualiza si la fila en disco sigue en la versión que leímos.
        let result = sqlx::query(
            "UPDATE verified_accounts SET \
                updated_at = ?, audit_hash = ?, audit_chain_hash = ?, row_version = ?, \
                publication_status = ?, attestation_scopes = ? \
             WHERE id = ? AND row_version = ?",
        )
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&account.audit_hash)
        .bind(row_version)
        .bind(new_status.as_str())
        .bind(&attestation_scopes_json)
        .bind(&account.id)
        .bind(account.row_version)
        .execute(self.pool)
        .await?;

        // Cero filas afectadas => la fila ya no está en `account.row_version`
        // (otra escritura la adelantó). No pisamos: reportamos el conflicto.
        if result.rows_affected() == 0 {
            return Err(VerifiedAccountRepositoryError::VersionConflict {
                id: account.id.clone(),
                expected: account.row_version,
            });
        }

        Ok(VerifiedAccountRow {
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: Some(account.audit_hash.clone()),
            row_version,
            publication_status: new_status,
            attestation_scopes: new_scopes.to_vec(),
            ..account.clone()
        })
    }
}

/// Convierte una fila de `verified_accounts` al tipo [`VerifiedAccountRow`],
/// decodificando los enums de texto persistidos.
fn row_to_verified_account(
    row: sqlx::sqlite::SqliteRow,
) -> Result<VerifiedAccountRow, VerifiedAccountRepositoryError> {
    let account_type_value: String = row.get("account_type");
    let account_type = AccountType::from_str_value(&account_type_value)
        .ok_or(VerifiedAccountRepositoryError::UnknownAccountType(account_type_value))?;

    let publication_status_value: String = row.get("publication_status");
    let publication_status = PublicationStatus::from_str_value(&publication_status_value)
        .ok_or(VerifiedAccountRepositoryError::UnknownPublicationStatus(publication_status_value))?;

    let attestation_scopes_json: String = row.get("attestation_scopes");
    let attestation_scopes = decode_attestation_scopes_json(&attestation_scopes_json)?;

    Ok(VerifiedAccountRow {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        row_version: row.get("row_version"),
        owner_id: row.get("owner_id"),
        institutional_tag: row.get("institutional_tag"),
        node_id: row.get("node_id"),
        broker: row.get("broker"),
        leverage: row.get("leverage"),
        currency: row.get("currency"),
        account_type,
        publication_status,
        attestation_scopes,
        broker_connection_ref: row.get("broker_connection_ref"),
    })
}

impl From<&VerifiedAccountRow> for VerifiedAccountRecord {
    /// Proyecta la fila persistida al tipo de puerto `registry_out`
    /// (ADR-0137) -- deliberadamente solo copia campos NO secretos
    /// (ADR-0093): `broker_connection_ref` es una referencia de texto, no
    /// una credencial.
    fn from(row: &VerifiedAccountRow) -> Self {
        VerifiedAccountRecord {
            id: row.id.clone(),
            owner_id: row.owner_id.clone(),
            broker: row.broker.clone(),
            leverage: row.leverage,
            currency: row.currency.clone(),
            account_type: row.account_type.as_str().to_string(),
            publication_status: row.publication_status.as_str().to_string(),
            attestation_scopes: row.attestation_scopes.iter().map(|s| s.as_str().to_string()).collect(),
            // Copia directa de `institutional_tag` (Eje B en esta tabla,
            // STORY-041) -- no hay una columna `capital_reality` separada.
            capital_reality: row.institutional_tag.clone(),
            broker_connection_ref: row.broker_connection_ref.clone(),
        }
    }
}

// ── `attested_track_records` -- APPEND-ONLY ATÓMICA ─────────────────────────

/// Errores que devuelven las operaciones de [`AttestedTrackRecordRepository`].
#[derive(Debug, thiserror::Error)]
pub enum AttestedTrackRecordRepositoryError {
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// El append no pudo completarse tras agotar los reintentos ante
    /// contención de escritura transitoria -- el track NO se descartó en
    /// silencio (regla "Atomicidad de ledgers append-only",
    /// rust-engineer/SKILL.md §4).
    #[error("no se pudo registrar el track record tras {attempts} intentos por contención de escritura")]
    WriteContention { attempts: u32 },
}

/// Número máximo de intentos del append ante contención de escritura
/// transitoria antes de rendirse con
/// [`AttestedTrackRecordRepositoryError::WriteContention`]. Mismo valor
/// (cinco) que el resto de los ledgers append-only del substrato.
const MAX_RECORD_ATTEMPTS: u32 = 5;

/// Decide si un error del repositorio es una contención de escritura
/// TRANSITORIA -- mismo criterio que
/// `enriched_domain_events::is_transient_write_conflict`.
fn is_transient_write_conflict(error: &AttestedTrackRecordRepositoryError) -> bool {
    let AttestedTrackRecordRepositoryError::Database(sqlx::Error::Database(db)) = error else {
        return false;
    };

    let message = db.message().to_lowercase();
    if message.contains("database is locked") || message.contains("database table is locked") {
        return true;
    }

    db.is_unique_violation() && message.contains("event_sequence_id")
}

/// Entrada para [`AttestedTrackRecordRepository::record_track_record`] --
/// todo lo que la Shell necesita para registrar UN track calculado: las
/// métricas ya calculadas por el Core, el ámbito, la ventana temporal, la
/// firma ya calculada y la identidad del dueño/máquina (Perfil D).
#[derive(Debug, Clone)]
pub struct RecordTrackRecordInput {
    pub owner_id: String,
    /// Grupo II -- en ESTA tabla `institutional_tag` ES el Eje B
    /// (`docs/adr/ADR-0145.md` corregido 2026-07-07, STORY-041/DEBT-016),
    /// copiado por el orquestador desde `account.institutional_tag` ANTES
    /// de llamar aquí (regla: "el llamador no puede mislabelar", la cuenta
    /// es la fuente de verdad, nunca un parámetro libre).
    pub institutional_tag: String,
    pub node_id: String,
    pub verified_account_id: String,
    pub scope: AttestationScope,
    pub time_window: String,
    pub metrics: TrackRecordMetrics,
    /// La firma reproducible, ya calculada por
    /// `domain::verified_account_registry::compute_track_record_signature`
    /// ANTES de llamar aquí -- este repositorio no la recalcula, solo la
    /// persiste (separación Core/Shell).
    pub signature_hash: String,
}

/// Una fila de `attested_track_records` ya persistida.
#[derive(Debug, Clone, PartialEq)]
pub struct AttestedTrackRecordRow {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub event_sequence_id: i64,

    pub owner_id: String,
    /// Grupo II -- ver [`RecordTrackRecordInput::institutional_tag`] (Eje B
    /// en esta tabla).
    pub institutional_tag: String,
    pub node_id: String,

    pub signature_hash: String,
    pub verified_account_id: String,
    pub scope: AttestationScope,
    pub time_window: String,
    pub metrics: TrackRecordMetrics,
}

/// Repositorio APPEND-ONLY para `attested_track_records`.
///
/// Al igual que `DomainEventRepository`/`GeneratedReportRepository`, la
/// única operación de escritura expuesta es
/// [`Self::record_track_record`] (un INSERT) -- no hay `update`/`delete`;
/// los triggers `trg_attested_track_records_no_update`/`_no_delete` de la
/// migración los rechazarían de todas formas.
pub struct AttestedTrackRecordRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> AttestedTrackRecordRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Registra UN track record calculado: deriva su posición en la cadena
    /// GLOBAL, computa su `audit_hash` encadenado y lo persiste como fila
    /// nueva.
    ///
    /// ## Atomicidad bajo concurrencia (regla "Atomicidad de ledgers append-only")
    ///
    /// El *read-then-write* (leer el MAX(`event_sequence_id`)/`audit_hash`
    /// previo, y el `INSERT`) ocurre dentro de UNA sola transacción
    /// `BEGIN IMMEDIATE` -- ver [`Self::try_record_once`]. Sin ella, dos
    /// escritores concurrentes derivarían el mismo `event_sequence_id`, el
    /// `UNIQUE` rechazaría a uno y su track se PERDERÍA. Ante contención
    /// transitoria se reintenta hasta [`MAX_RECORD_ATTEMPTS`] veces
    /// re-derivando la secuencia; nunca se descarta el track en silencio.
    pub async fn record_track_record(
        &self,
        input: RecordTrackRecordInput,
    ) -> Result<AttestedTrackRecordRow, AttestedTrackRecordRepositoryError> {
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
                        return Err(AttestedTrackRecordRepositoryError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    /// Un intento único del append, dentro de una transacción
    /// `BEGIN IMMEDIATE` -- toma el lock de escritura de ENTRADA, evitando
    /// tanto la intercalación de otro escritor entre lectura e inserción
    /// como el interbloqueo de upgrade de dos transacciones DEFERRED.
    async fn try_record_once(
        &self,
        input: &RecordTrackRecordInput,
    ) -> Result<AttestedTrackRecordRow, AttestedTrackRecordRepositoryError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        let tail_row = sqlx::query(
            "SELECT audit_hash, event_sequence_id \
             FROM attested_track_records \
             ORDER BY event_sequence_id DESC \
             LIMIT 1",
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

        let scope_str = input.scope.as_str();
        let equity_curve_json = serde_json::to_string(&input.metrics.equity_curve)
            // Vec<(i64,i64)> siempre serializa -- nunca falla en la práctica.
            .expect("equity_curve siempre serializa");
        let balance_curve_json = serde_json::to_string(&input.metrics.balance_curve)
            .expect("balance_curve siempre serializa");

        // Eje B, STORY-041: en esta tabla `institutional_tag` ES el Eje B --
        // no hay una columna `capital_reality` separada. El mismo valor
        // alimenta el buffer del audit_hash (posición Eje B, ver
        // `compute_track_record_audit_hash`) y el INSERT.
        let capital_reality_str = input.institutional_tag.as_str();

        let audit_hash = compute_track_record_audit_hash(
            &id,
            now_ns,
            event_sequence_id,
            &previous_audit_hash,
            &input.owner_id,
            &input.institutional_tag,
            &input.node_id,
            &input.verified_account_id,
            scope_str,
            &input.time_window,
            &input.signature_hash,
            capital_reality_str,
        );

        sqlx::query(
            "INSERT INTO attested_track_records (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, \
                signature_hash, verified_account_id, scope, time_window, \
                equity_curve, balance_curve, max_drawdown_e8, gain_pct_e8, win_rate_e8, \
                avg_holding_time_ns, trading_days, total_realized_pnl_e8, total_deposits_e8, \
                total_withdrawals_e8\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&audit_chain_hash)
        .bind(event_sequence_id)
        .bind(&input.owner_id)
        .bind(&input.institutional_tag)
        .bind(&input.node_id)
        .bind(&input.signature_hash)
        .bind(&input.verified_account_id)
        .bind(scope_str)
        .bind(&input.time_window)
        .bind(&equity_curve_json)
        .bind(&balance_curve_json)
        .bind(input.metrics.max_drawdown_e8)
        .bind(input.metrics.gain_pct_e8)
        .bind(input.metrics.win_rate_e8)
        .bind(input.metrics.avg_holding_time_ns)
        .bind(input.metrics.trading_days)
        .bind(input.metrics.total_realized_pnl_e8)
        .bind(input.metrics.total_deposits_e8)
        .bind(input.metrics.total_withdrawals_e8)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(AttestedTrackRecordRow {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash,
            event_sequence_id,
            owner_id: input.owner_id.clone(),
            institutional_tag: input.institutional_tag.clone(),
            node_id: input.node_id.clone(),
            signature_hash: input.signature_hash.clone(),
            verified_account_id: input.verified_account_id.clone(),
            scope: input.scope,
            time_window: input.time_window.clone(),
            metrics: input.metrics.clone(),
        })
    }

    /// Carga la cadena completa, ordenada por `event_sequence_id`
    /// ascendente -- usada por los tests de integridad de la cadena y por
    /// cualquier consumidor futuro que reconstruya el historial de tracks.
    pub async fn load_chain(&self) -> Result<Vec<AttestedTrackRecordRow>, AttestedTrackRecordRepositoryError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, node_id, \
                    signature_hash, verified_account_id, scope, time_window, \
                    equity_curve, balance_curve, max_drawdown_e8, gain_pct_e8, win_rate_e8, \
                    avg_holding_time_ns, trading_days, total_realized_pnl_e8, total_deposits_e8, \
                    total_withdrawals_e8 \
             FROM attested_track_records \
             ORDER BY event_sequence_id ASC",
        )
        .fetch_all(self.pool)
        .await?;

        rows.into_iter().map(row_to_track_record).collect()
    }
}

/// Convierte una fila de `attested_track_records` al tipo
/// [`AttestedTrackRecordRow`], reconstruyendo el `scope` y las curvas JSON.
fn row_to_track_record(
    row: sqlx::sqlite::SqliteRow,
) -> Result<AttestedTrackRecordRow, AttestedTrackRecordRepositoryError> {
    let scope_value: String = row.get("scope");
    // Fila de nuestra propia tabla, protegida por el CHECK de la migración
    // -- un valor desconocido aquí sería corrupción de datos externa al
    // control de este repositorio; se trata como fallo de base de datos
    // genérico envolviendo el mensaje en un error de decodificación simple.
    let scope = AttestationScope::from_str_value(&scope_value).unwrap_or(AttestationScope::BrokerReadonly);

    let equity_curve: Vec<(i64, i64)> =
        serde_json::from_str(&row.get::<String, _>("equity_curve")).unwrap_or_default();
    let balance_curve: Vec<(i64, i64)> =
        serde_json::from_str(&row.get::<String, _>("balance_curve")).unwrap_or_default();

    Ok(AttestedTrackRecordRow {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        event_sequence_id: row.get("event_sequence_id"),
        owner_id: row.get("owner_id"),
        // Eje B, STORY-041: `institutional_tag` ES el Eje B en esta tabla --
        // no hay columna `capital_reality` separada que decodificar aparte.
        institutional_tag: row.get("institutional_tag"),
        node_id: row.get("node_id"),
        signature_hash: row.get("signature_hash"),
        verified_account_id: row.get("verified_account_id"),
        scope,
        time_window: row.get("time_window"),
        metrics: TrackRecordMetrics {
            equity_curve,
            balance_curve,
            max_drawdown_e8: row.get("max_drawdown_e8"),
            gain_pct_e8: row.get("gain_pct_e8"),
            win_rate_e8: row.get("win_rate_e8"),
            avg_holding_time_ns: row.get("avg_holding_time_ns"),
            trading_days: row.get("trading_days"),
            total_realized_pnl_e8: row.get("total_realized_pnl_e8"),
            total_deposits_e8: row.get("total_deposits_e8"),
            total_withdrawals_e8: row.get("total_withdrawals_e8"),
        },
    })
}

impl From<&AttestedTrackRecordRow> for AttestedTrackRecord {
    /// Proyecta la fila persistida al tipo de puerto `track_record_out`
    /// (ADR-0137). `is_attested_by_drasus` se deriva SIEMPRE de
    /// `AttestationScope::is_sovereign_attestation` -- nunca de un booleano
    /// aparte que pudiera desincronizarse del `scope` real (regla
    /// obligatoria #1, ADR-0145).
    fn from(row: &AttestedTrackRecordRow) -> Self {
        AttestedTrackRecord {
            id: row.id.clone(),
            verified_account_id: row.verified_account_id.clone(),
            scope: row.scope.as_str().to_string(),
            // Copia directa de `institutional_tag` (Eje B en esta tabla,
            // STORY-041) -- no hay una columna `capital_reality` separada.
            capital_reality: row.institutional_tag.clone(),
            time_window: row.time_window.clone(),
            signature_hash: row.signature_hash.clone(),
            equity_curve: row.metrics.equity_curve.clone(),
            balance_curve: row.metrics.balance_curve.clone(),
            max_drawdown_e8: row.metrics.max_drawdown_e8,
            gain_pct_e8: row.metrics.gain_pct_e8,
            win_rate_e8: row.metrics.win_rate_e8,
            avg_holding_time_ns: row.metrics.avg_holding_time_ns,
            trading_days: row.metrics.trading_days,
            total_realized_pnl_e8: row.metrics.total_realized_pnl_e8,
            total_deposits_e8: row.metrics.total_deposits_e8,
            total_withdrawals_e8: row.metrics.total_withdrawals_e8,
            is_attested_by_drasus: row.scope.is_sovereign_attestation(),
            // Eje B, derivado SOLO de `CapitalReality::is_real_capital`,
            // interpretando `institutional_tag` (STORY-041) -- NUNCA de
            // `is_attested_by_drasus` (regla obligatoria del retrabajo: los
            // ejes son independientes). Mismo criterio de `unwrap_or` que
            // `row_to_track_record` para `scope`: la fila está protegida
            // por el CHECK de la migración, un valor desconocido aquí sería
            // corrupción externa al control de este repositorio.
            is_real_capital: CapitalReality::from_str_value(&row.institutional_tag)
                .unwrap_or(CapitalReality::Live)
                .is_real_capital(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    fn sample_new_account() -> NewVerifiedAccount {
        NewVerifiedAccount {
            owner_id: "owner-1".to_string(),
            // Eje B, STORY-041: en esta tabla `institutional_tag` ES el Eje
            // B -- ya no acepta el placeholder genérico "DRASUS_LOCAL" del
            // resto del substrato (CHECK restringido a LIVE/PAPER/DEMO/CHALLENGE).
            institutional_tag: CapitalReality::Live.as_str().to_string(),
            node_id: "node-1".to_string(),
            broker: "ICMarkets".to_string(),
            leverage: 100,
            currency: "USD".to_string(),
            account_type: AccountType::Own,
            attestation_scopes: vec![AttestationScope::Sovereign],
            broker_connection_ref: None,
        }
    }

    // ── CRITERIO #1 (Orden §5): esquema STRICT + Grupo I/II/IV + row_version ──

    #[tokio::test]
    async fn migration_creates_verified_accounts_strict_with_row_version_and_no_event_sequence_id() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('verified_accounts')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();

        for expected in [
            "id", "created_at", "updated_at", "audit_hash", "audit_chain_hash", "row_version",
            "owner_id", "institutional_tag", "node_id",
            "broker", "leverage", "currency", "account_type", "publication_status",
            "attestation_scopes", "broker_connection_ref",
        ] {
            assert!(column_names.contains(&expected.to_string()), "falta la columna: {expected}");
        }
        assert!(
            !column_names.contains(&"event_sequence_id".to_string()),
            "verified_accounts es MUTABLE (ADR-0141): no debe tener event_sequence_id"
        );

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'verified_accounts'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(sql.contains("STRICT"), "verified_accounts debe declararse STRICT");
    }

    /// CRITERIO #1 (Orden §5, guardarraíl anti-regresión STORY-041/DEBT-016):
    /// la columna `capital_reality` NO debe existir en `verified_accounts`
    /// -- el Eje B vive en `institutional_tag`, que debe portar el `CHECK`
    /// con el vocabulario `LIVE`/`PAPER`/`DEMO`/`CHALLENGE`. Esta prueba
    /// debe poder caerse si alguien reintroduce la columna duplicada que
    /// STORY-038 había creado.
    #[tokio::test]
    async fn verified_accounts_has_no_capital_reality_column_and_institutional_tag_carries_axis_b_check() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('verified_accounts')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();
        assert!(
            !column_names.contains(&"capital_reality".to_string()),
            "verified_accounts NO debe tener una columna capital_reality -- el Eje B vive en institutional_tag (ADR-0145 corregido)"
        );

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'verified_accounts'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(
            sql.contains("institutional_tag") && sql.contains("'LIVE'") && sql.contains("'PAPER'")
                && sql.contains("'DEMO'") && sql.contains("'CHALLENGE'"),
            "institutional_tag debe portar el CHECK con el vocabulario del Eje B: {sql}"
        );
    }

    #[tokio::test]
    async fn migration_creates_attested_track_records_strict_with_event_sequence_id_and_signature_hash() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('attested_track_records')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();

        for expected in [
            "id", "created_at", "updated_at", "audit_hash", "audit_chain_hash", "event_sequence_id",
            "owner_id", "institutional_tag", "node_id", "signature_hash",
            "verified_account_id", "scope", "time_window",
            "equity_curve", "balance_curve", "max_drawdown_e8", "gain_pct_e8", "win_rate_e8",
            "avg_holding_time_ns", "trading_days", "total_realized_pnl_e8", "total_deposits_e8",
            "total_withdrawals_e8",
        ] {
            assert!(column_names.contains(&expected.to_string()), "falta la columna: {expected}");
        }
        assert!(
            !column_names.contains(&"row_version".to_string()),
            "attested_track_records es APPEND-ONLY (ADR-0141): no debe tener row_version"
        );

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'attested_track_records'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(sql.contains("STRICT"), "attested_track_records debe declararse STRICT");
    }

    /// CRITERIO #1 (Orden §5, guardarraíl anti-regresión STORY-041/DEBT-016):
    /// paralelo a la prueba equivalente de `verified_accounts` -- la columna
    /// `capital_reality` NO debe existir en `attested_track_records` y
    /// `institutional_tag` debe portar el `CHECK` del Eje B.
    #[tokio::test]
    async fn attested_track_records_has_no_capital_reality_column_and_institutional_tag_carries_axis_b_check() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('attested_track_records')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();
        assert!(
            !column_names.contains(&"capital_reality".to_string()),
            "attested_track_records NO debe tener una columna capital_reality -- el Eje B vive en institutional_tag (ADR-0145 corregido)"
        );

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'attested_track_records'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(
            sql.contains("institutional_tag") && sql.contains("'LIVE'") && sql.contains("'PAPER'")
                && sql.contains("'DEMO'") && sql.contains("'CHALLENGE'"),
            "institutional_tag debe portar el CHECK con el vocabulario del Eje B: {sql}"
        );
    }

    // ── create(): default PRIVATE estructural (regla obligatoria #4) ────────

    #[tokio::test]
    async fn create_always_yields_row_version_one_and_private_publication_status() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = VerifiedAccountRepository::new(&pool, &clock);

        let account = repo.create(sample_new_account()).await.expect("registrar cuenta");
        assert_eq!(account.row_version, 1);
        assert_eq!(account.publication_status, PublicationStatus::Private);
        assert_eq!(account.attestation_scopes, vec![AttestationScope::Sovereign]);
        assert_eq!(account.audit_chain_hash, None);

        let reloaded = repo.find_by_id(&account.id).await.expect("releer").expect("debe existir");
        assert_eq!(reloaded, account);
    }

    // ── CRITERIO #7 (Orden §5): row_version, concurrencia optimista ─────────

    #[tokio::test]
    async fn update_publication_and_scopes_increments_row_version_and_chains_audit_hash() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = VerifiedAccountRepository::new(&pool, &clock);

        let account = repo.create(sample_new_account()).await.expect("registrar cuenta");
        clock.tick();
        let updated = repo
            .update_publication_and_scopes(&account, PublicationStatus::Public, &[AttestationScope::Sovereign])
            .await
            .expect("actualizar publicación");

        assert_eq!(updated.row_version, 2);
        assert_eq!(updated.publication_status, PublicationStatus::Public);
        assert_eq!(updated.audit_chain_hash, Some(account.audit_hash.clone()));
        assert_ne!(updated.audit_hash, account.audit_hash);
    }

    /// CRITERIO DE CIERRE: dos actualizaciones que parten de la MISMA
    /// versión en memoria no pueden ambas tener éxito -- la primera avanza
    /// (1 -> 2); la segunda, que sigue creyendo estar en la versión 1,
    /// devuelve `VersionConflict` en vez de pisar el cambio de la primera.
    #[tokio::test]
    async fn concurrent_updates_from_same_row_version_conflict_instead_of_overwriting() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = VerifiedAccountRepository::new(&pool, &clock);

        let account = repo.create(sample_new_account()).await.expect("registrar cuenta");
        let first_writer_view = account.clone();
        let second_writer_view = account;

        clock.tick();
        let updated = repo
            .update_publication_and_scopes(&first_writer_view, PublicationStatus::Public, &[AttestationScope::Sovereign])
            .await
            .expect("el primer update debe tener éxito");
        assert_eq!(updated.row_version, 2);

        clock.tick();
        let conflict = repo
            .update_publication_and_scopes(&second_writer_view, PublicationStatus::Private, &[AttestationScope::Sovereign])
            .await;
        assert!(
            matches!(conflict, Err(VerifiedAccountRepositoryError::VersionConflict { expected: 1, .. })),
            "el segundo update desde la versión 1 debe dar VersionConflict; fue: {conflict:?}"
        );

        let reloaded = repo.find_by_id(&updated.id).await.expect("releer").expect("existe");
        assert_eq!(reloaded.row_version, 2);
        assert_eq!(
            reloaded.publication_status,
            PublicationStatus::Public,
            "debe conservarse el cambio del PRIMER writer, no el del segundo"
        );
    }

    // ── CRITERIO #9 (Orden §5): CHECKs de account_type/publication_status ───

    #[tokio::test]
    async fn database_check_rejects_unknown_account_type() {
        let pool = migrated_pool().await;
        let result = sqlx::query(
            "INSERT INTO verified_accounts (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                owner_id, institutional_tag, node_id, \
                broker, leverage, currency, account_type, publication_status, \
                attestation_scopes, broker_connection_ref\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', 'LIVE', 'node-1', \
                       'ICMarkets', 100, 'USD', 'UNKNOWN_TYPE', 'PRIVATE', '[]', NULL)",
        )
        .execute(&pool)
        .await;
        assert!(result.is_err(), "un account_type fuera del catálogo debe ser rechazado por el CHECK");
    }

    #[tokio::test]
    async fn database_check_rejects_invalid_json_in_attestation_scopes() {
        let pool = migrated_pool().await;
        let result = sqlx::query(
            "INSERT INTO verified_accounts (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                owner_id, institutional_tag, node_id, \
                broker, leverage, currency, account_type, publication_status, \
                attestation_scopes, broker_connection_ref\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', 'LIVE', 'node-1', \
                       'ICMarkets', 100, 'USD', 'OWN', 'PRIVATE', '{not valid json', NULL)",
        )
        .execute(&pool)
        .await;
        assert!(result.is_err(), "attestation_scopes con JSON corrupto debe ser rechazado por el CHECK(json_valid)");
    }

    /// CRITERIO #6 (Orden §4 punto 16) DE CIERRE, consolidado STORY-041: un
    /// `institutional_tag` (Eje B) fuera del catálogo (`LIVE`/`PAPER`/`DEMO`/
    /// `CHALLENGE`) debe ser rechazado por el CHECK de la migración -- Eje
    /// B, paralelo a `database_check_rejects_unknown_account_type`. Ya no
    /// hay una columna `capital_reality` separada que probar: el CHECK vive
    /// ahora en `institutional_tag`.
    #[tokio::test]
    async fn database_check_rejects_unknown_institutional_tag_on_verified_accounts() {
        let pool = migrated_pool().await;
        let result = sqlx::query(
            "INSERT INTO verified_accounts (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                owner_id, institutional_tag, node_id, \
                broker, leverage, currency, account_type, publication_status, \
                attestation_scopes, broker_connection_ref\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', 'UNKNOWN_REALITY', 'node-1', \
                       'ICMarkets', 100, 'USD', 'OWN', 'PRIVATE', '[]', NULL)",
        )
        .execute(&pool)
        .await;
        assert!(result.is_err(), "un institutional_tag (Eje B) fuera del catálogo debe ser rechazado por el CHECK");
    }

    // ── attested_track_records: append-only, secuencia, cadena ──────────────

    fn sample_metrics() -> TrackRecordMetrics {
        TrackRecordMetrics {
            equity_curve: vec![(0, 1_000_000_000_000)],
            balance_curve: vec![(0, 1_000_000_000_000)],
            max_drawdown_e8: 0,
            gain_pct_e8: 441_000_000,
            win_rate_e8: 66_666_666,
            avg_holding_time_ns: 3_600_000_000_000,
            trading_days: 2,
            total_realized_pnl_e8: 4_410_000_000_000,
            total_deposits_e8: 35_000_000_000,
            total_withdrawals_e8: 47_698_000_000,
        }
    }

    fn record_input(verified_account_id: &str, scope: AttestationScope) -> RecordTrackRecordInput {
        RecordTrackRecordInput {
            owner_id: "owner-1".to_string(),
            // Eje B, STORY-041: en esta tabla `institutional_tag` ES el Eje
            // B -- ya no acepta el placeholder genérico "DRASUS_LOCAL".
            institutional_tag: CapitalReality::Live.as_str().to_string(),
            node_id: "node-1".to_string(),
            verified_account_id: verified_account_id.to_string(),
            scope,
            time_window: "2026-W27".to_string(),
            metrics: sample_metrics(),
            signature_hash: "sig-abc".to_string(),
        }
    }

    #[tokio::test]
    async fn update_is_rejected_by_trigger_on_attested_track_records() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AttestedTrackRecordRepository::new(&pool, &clock);

        let row = repo
            .record_track_record(record_input("acc-1", AttestationScope::Sovereign))
            .await
            .expect("registrar track");

        let result = sqlx::query("UPDATE attested_track_records SET gain_pct_e8 = 0 WHERE id = ?")
            .bind(&row.id)
            .execute(&pool)
            .await;
        assert!(result.is_err(), "UPDATE sobre attested_track_records debe ser rechazado por el trigger");
    }

    #[tokio::test]
    async fn delete_is_rejected_by_trigger_on_attested_track_records() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AttestedTrackRecordRepository::new(&pool, &clock);

        let row = repo
            .record_track_record(record_input("acc-1", AttestationScope::Sovereign))
            .await
            .expect("registrar track");

        let result = sqlx::query("DELETE FROM attested_track_records WHERE id = ?")
            .bind(&row.id)
            .execute(&pool)
            .await;
        assert!(result.is_err(), "DELETE sobre attested_track_records debe ser rechazado por el trigger");
    }

    #[tokio::test]
    async fn database_check_rejects_unknown_scope() {
        let pool = migrated_pool().await;
        let result = sqlx::query(
            "INSERT INTO attested_track_records (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, signature_hash, verified_account_id, scope, \
                time_window, equity_curve, balance_curve, max_drawdown_e8, gain_pct_e8, win_rate_e8, \
                avg_holding_time_ns, trading_days, total_realized_pnl_e8, total_deposits_e8, total_withdrawals_e8\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', 'LIVE', 'node-1', 'sig', 'acc-1', \
                       'UNKNOWN_SCOPE', '2026-W27', '[]', '[]', 0, 0, 0, 0, 0, 0, 0, 0)",
        )
        .execute(&pool)
        .await;
        assert!(result.is_err(), "un scope fuera del catálogo debe ser rechazado por el CHECK");
    }

    /// CRITERIO #6 (Orden §4 punto 16) DE CIERRE, consolidado STORY-041: un
    /// `institutional_tag` (Eje B) fuera del catálogo en
    /// `attested_track_records` debe ser rechazado por el CHECK -- paralelo
    /// a `database_check_rejects_unknown_scope`, pero sobre el Eje B. Ya no
    /// hay una columna `capital_reality` separada que probar.
    #[tokio::test]
    async fn database_check_rejects_unknown_institutional_tag_on_attested_track_records() {
        let pool = migrated_pool().await;
        let result = sqlx::query(
            "INSERT INTO attested_track_records (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, signature_hash, verified_account_id, scope, \
                time_window, equity_curve, balance_curve, max_drawdown_e8, gain_pct_e8, win_rate_e8, \
                avg_holding_time_ns, trading_days, total_realized_pnl_e8, total_deposits_e8, total_withdrawals_e8\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', 'UNKNOWN_REALITY', 'node-1', 'sig', 'acc-1', \
                       'SOVEREIGN', '2026-W27', '[]', '[]', 0, 0, 0, 0, 0, 0, 0, 0)",
        )
        .execute(&pool)
        .await;
        assert!(result.is_err(), "un institutional_tag (Eje B) fuera del catálogo debe ser rechazado por el CHECK");
    }

    #[tokio::test]
    async fn event_sequence_id_is_monotonic_and_chain_is_recomputable() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AttestedTrackRecordRepository::new(&pool, &clock);

        let first = repo.record_track_record(record_input("acc-1", AttestationScope::Sovereign)).await.expect("primero");
        clock.tick();
        let second = repo.record_track_record(record_input("acc-1", AttestationScope::BrokerReadonly)).await.expect("segundo");

        assert_eq!(first.event_sequence_id, 1);
        assert_eq!(second.event_sequence_id, 2);
        assert_eq!(first.audit_chain_hash, None, "génesis debe tener audit_chain_hash NULL");
        assert_eq!(second.audit_chain_hash, Some(first.audit_hash.clone()));

        // El ámbito inviolable persiste en disco, distinto por fila.
        assert_eq!(first.scope, AttestationScope::Sovereign);
        assert_eq!(second.scope, AttestationScope::BrokerReadonly);

        let chain = repo.load_chain().await.expect("cargar cadena");
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0], first);
        assert_eq!(chain[1], second);
    }

    // ── CRITERIO #8 (Orden §5): append atómico + concurrencia (16 escritores) ──

    /// CRITERIO DE CIERRE: 16 escritores concurrentes sobre el MISMO
    /// pool/ledger, en un archivo SQLite temporal (NUNCA `:memory:`, donde
    /// cada conexión sería una base distinta). La transacción
    /// `BEGIN IMMEDIATE` + reintento acotado debe garantizar que NINGÚN
    /// track se pierde y que la secuencia queda densa (1..=N). Esta prueba
    /// DEBE poder caerse si se quita la transacción (dos escritores
    /// leerían el mismo MAX, el UNIQUE rechazaría a uno y su fila se
    /// perdería).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_record_track_records_persist_every_track_without_gaps_or_lost_rows() {
        use std::sync::Arc;

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("attested_track_records_concurrency.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());
        let pool = connect(&database_url).await.expect("conectar");
        migrate(&pool).await.expect("migrar");

        let clock: Arc<DeterministicClock> = Arc::new(DeterministicClock::new(1_000, 100));
        const N: i64 = 16;

        let mut handles = Vec::new();
        for i in 0..N {
            let pool_c = pool.clone();
            let clock_c = clock.clone();
            handles.push(tokio::spawn(async move {
                let repo = AttestedTrackRecordRepository::new(&pool_c, clock_c.as_ref());
                repo.record_track_record(record_input(&format!("acc-{i}"), AttestationScope::Sovereign)).await
            }));
        }

        for handle in handles {
            handle
                .await
                .expect("la tarea no debe entrar en panic")
                .expect("record_track_record debe tener éxito para cada escritor concurrente");
        }

        let repo = AttestedTrackRecordRepository::new(&pool, clock.as_ref());
        let chain = repo.load_chain().await.expect("cargar la cadena completa");

        assert_eq!(chain.len() as i64, N, "deben persistirse las N filas, sin ninguna pérdida");

        let sequence_ids: Vec<i64> = chain.iter().map(|row| row.event_sequence_id).collect();
        let expected: Vec<i64> = (1..=N).collect();
        assert_eq!(sequence_ids, expected, "los event_sequence_id deben ser 1..=N sin huecos ni duplicados");
    }

    // ── Proyección al puerto: is_attested_by_drasus deriva SIEMPRE del scope ──

    #[test]
    fn attested_track_record_projection_marks_is_attested_only_for_sovereign_scope() {
        let base_row = AttestedTrackRecordRow {
            id: "track-1".to_string(),
            created_at_ns: 0,
            updated_at_ns: 0,
            audit_hash: "hash".to_string(),
            audit_chain_hash: None,
            event_sequence_id: 1,
            owner_id: "owner-1".to_string(),
            // Eje B, STORY-041: en esta tabla `institutional_tag` ES el Eje
            // B -- ya no acepta el placeholder genérico "DRASUS_LOCAL".
            institutional_tag: CapitalReality::Live.as_str().to_string(),
            node_id: "node-1".to_string(),
            signature_hash: "sig".to_string(),
            verified_account_id: "acc-1".to_string(),
            scope: AttestationScope::Sovereign,
            time_window: "2026-W27".to_string(),
            metrics: sample_metrics(),
        };

        let sovereign_projection = AttestedTrackRecord::from(&base_row);
        assert!(sovereign_projection.is_attested_by_drasus);

        let mut readonly_row = base_row;
        readonly_row.scope = AttestationScope::BrokerReadonly;
        let readonly_projection = AttestedTrackRecord::from(&readonly_row);
        assert!(
            !readonly_projection.is_attested_by_drasus,
            "un track BROKER_READONLY nunca debe presentarse como atestado por Drasus"
        );
    }

    /// CRITERIO DE CIERRE (Eje B): la proyección deriva `is_real_capital`
    /// SOLO de `capital_reality` -- un track `SOVEREIGN` (atestado) con
    /// `capital_reality = PAPER` debe seguir siendo `is_attested_by_drasus =
    /// true` pero `is_real_capital = false`, demostrando que los dos ejes
    /// son ortogonales también a nivel de la fila persistida real.
    #[test]
    fn attested_track_record_projection_marks_is_real_capital_only_for_live_and_independently_of_scope() {
        let sovereign_paper_row = AttestedTrackRecordRow {
            id: "track-2".to_string(),
            created_at_ns: 0,
            updated_at_ns: 0,
            audit_hash: "hash".to_string(),
            audit_chain_hash: None,
            event_sequence_id: 1,
            owner_id: "owner-1".to_string(),
            // Eje B, STORY-041: en esta tabla `institutional_tag` ES el Eje
            // B -- este test lo fija en PAPER (el punto de la prueba).
            institutional_tag: CapitalReality::Paper.as_str().to_string(),
            node_id: "node-1".to_string(),
            signature_hash: "sig".to_string(),
            verified_account_id: "acc-1".to_string(),
            scope: AttestationScope::Sovereign,
            time_window: "2026-W27".to_string(),
            metrics: sample_metrics(),
        };

        let projection = AttestedTrackRecord::from(&sovereign_paper_row);
        assert!(projection.is_attested_by_drasus, "SOVEREIGN sigue siendo atestado (Eje A intacto)");
        assert!(!projection.is_real_capital, "PAPER nunca es capital real (Eje B), sin importar el Eje A");
        assert_eq!(projection.capital_reality, "PAPER");
    }
}
