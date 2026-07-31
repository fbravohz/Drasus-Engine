//! [SHELL] Repositorio de persistencia para `instrument_master` y
//! `broker_instrument_terms` (migración `0022_instrument_session_catalog.sql`,
//! ADR-0162).
//!
//! Usa el pool de `shared` -- este módulo nunca crea su propio pool;
//! siempre recibe uno ya inicializado y migrado desde el exterior. Mismo
//! patrón de concurrencia optimista (`row_version` + `VersionConflict`) que
//! `shared::persistence::plan_tier_quota::PlanRepository` y
//! `shared::persistence::central_identity::AccountRepository` -- ambas
//! tablas de este ADR son Referencia/Mutable, NO event-stores append-only
//! (ADR-0141 "Semántica de secuencias", corrección del Architect en
//! ADR-0162), así que un `INSERT`/`UPDATE` de una sola sentencia ya es
//! atómico por sí mismo: no hace falta envolverlo en `BEGIN IMMEDIATE`
//! (esa transacción explícita solo es necesaria cuando hay una derivación
//! *read-then-write* como la posición de un event-store append-only, que
//! esta tabla no tiene).

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use shared::public_interface::Clock;

use crate::domain::{
    compute_broker_server_clock_audit_hash, compute_broker_terms_audit_hash, compute_instrument_master_audit_hash,
    AssetClass, CommissionModel,
};
use crate::schemas::{
    BrokerInstrumentTermsRow, BrokerServerClockRevision, BrokerServerClockRow, InstrumentMasterRevision,
    InstrumentMasterRow, NewBrokerInstrumentTerms, NewBrokerServerClock, NewInstrumentMaster,
};

/// Errores que devuelven las operaciones de [`InstrumentRepository`].
#[derive(Debug, thiserror::Error)]
pub enum InstrumentRepositoryError {
    /// La operación subyacente de SQLite falló.
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// [`InstrumentRepository::create_master`] con un `instrument_id` que
    /// ya existe (criterio de aceptación #1: la clave de negocio es
    /// única) -- NUNCA se pisa el instrumento existente.
    #[error("el instrument_id '{0}' ya existe en el catálogo")]
    DuplicateInstrumentId(String),
    /// [`InstrumentRepository::create_broker_terms`] con un
    /// `(instrument_id, venue)` que ya tiene override -- cada bróker tiene
    /// UNA sola fila de override por instrumento.
    #[error("ya existe un override de '{venue}' para el instrumento '{instrument_id}'")]
    DuplicateBrokerOverride { instrument_id: String, venue: String },
    /// Concurrencia optimista (ADR-0141): el `UPDATE` partió de un
    /// `row_version` que ya no es el vigente en disco -- otra escritura
    /// actualizó la fila en el ínterin. La operación NO pisa el cambio
    /// ajeno; quien llama debe releer la fila y reintentar.
    #[error("conflicto de versión en '{id}': se esperaba row_version {expected}, la fila ya avanzó")]
    VersionConflict { id: String, expected: i64 },
    /// Una fila tenía un valor de `asset_class` fuera del vocabulario fijo
    /// -- error de integridad de datos, no debería ocurrir dado el `CHECK`
    /// de la migración.
    #[error("asset_class desconocido en instrument_master: '{0}'")]
    UnknownAssetClass(String),
    /// Igual que arriba, para `commission_model`.
    #[error("commission_model desconocido en broker_instrument_terms: '{0}'")]
    UnknownCommissionModel(String),
    /// [`InstrumentRepository::create_broker_server_clock`] con un `venue`
    /// que ya tiene un reloj de servidor declarado -- cada bróker tiene UN
    /// solo reloj (retrofit STORY-070/TASK-071).
    #[error("ya existe un reloj de servidor declarado para el venue '{0}'")]
    DuplicateVenue(String),
}

/// Repositorio para `instrument_master` y `broker_instrument_terms`.
///
/// Constrúyelo con un [`SqlitePool`] ya migrado y cualquier implementación
/// de [`Clock`] -- mismo patrón que
/// `shared::persistence::plan_tier_quota::PlanRepository`.
pub struct InstrumentRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> InstrumentRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`. Ambos se toman
    /// prestados por la vida del repositorio -- no se toma ownership.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    // ── instrument_master ────────────────────────────────────────────

    /// Da de alta un instrumento canónico nuevo con `row_version = 1`.
    ///
    /// Un `instrument_id` duplicado se rechaza como
    /// [`InstrumentRepositoryError::DuplicateInstrumentId`] -- el `UNIQUE`
    /// de la migración es el guardián primario; esta función solo traduce
    /// el error crudo de SQLite a un tipo de dominio legible.
    pub async fn create_master(&self, new: NewInstrumentMaster) -> Result<InstrumentMasterRow, InstrumentRepositoryError> {
        let id = Uuid::now_v7().to_string();
        let now_ns = self.clock.timestamp_ns();
        let row_version: i64 = 1;

        let audit_hash = compute_instrument_master_audit_hash(
            &id,
            now_ns,
            row_version,
            None,
            &new.instrument_id,
            new.asset_class,
            new.canonical_pip_size,
            new.canonical_tick_size,
            new.default_session_template_id.as_deref(),
        );

        let result = sqlx::query(
            "INSERT INTO instrument_master (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                node_id, process_id, \
                instrument_id, asset_class, base_currency, quote_currency, \
                canonical_pip_size, canonical_tick_size, default_session_template_id\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(Option::<String>::None)
        .bind(row_version)
        .bind(&new.node_id)
        .bind(&new.process_id)
        .bind(&new.instrument_id)
        .bind(new.asset_class.as_str())
        .bind(&new.base_currency)
        .bind(&new.quote_currency)
        .bind(new.canonical_pip_size)
        .bind(new.canonical_tick_size)
        .bind(&new.default_session_template_id)
        .execute(self.pool)
        .await;

        if let Err(sqlx::Error::Database(db_err)) = &result {
            if is_duplicate_instrument_id_violation(&**db_err) {
                return Err(InstrumentRepositoryError::DuplicateInstrumentId(new.instrument_id));
            }
        }
        result?;

        Ok(InstrumentMasterRow {
            id,
            created_at: now_ns,
            updated_at: now_ns,
            audit_hash,
            audit_chain_hash: None,
            row_version,
            node_id: new.node_id,
            process_id: new.process_id,
            instrument_id: new.instrument_id,
            asset_class: new.asset_class,
            base_currency: new.base_currency,
            quote_currency: new.quote_currency,
            canonical_pip_size: new.canonical_pip_size,
            canonical_tick_size: new.canonical_tick_size,
            default_session_template_id: new.default_session_template_id,
        })
    }

    /// Carga un instrumento canónico por su clave de NEGOCIO
    /// (`instrument_id`, ej. "EURUSD") -- la búsqueda que usan tanto la
    /// resolución de términos como cualquier consumidor externo (ADR-0162
    /// Corrección del Architect #1: `broker_instrument_terms` referencia
    /// esta misma columna, nunca el UUID técnico).
    pub async fn find_master_by_instrument_id(
        &self,
        instrument_id: &str,
    ) -> Result<Option<InstrumentMasterRow>, InstrumentRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    node_id, process_id, \
                    instrument_id, asset_class, base_currency, quote_currency, \
                    canonical_pip_size, canonical_tick_size, default_session_template_id \
             FROM instrument_master WHERE instrument_id = ?",
        )
        .bind(instrument_id)
        .fetch_optional(self.pool)
        .await?;

        row.map(row_to_master).transpose()
    }

    /// Revisa los campos canónicos editables de un instrumento (TTR-001).
    ///
    /// ## Concurrencia optimista (ADR-0141)
    ///
    /// El `UPDATE` filtra por `id` **y** `row_version = <el de `current`>`.
    /// Si otra edición ya avanzó la fila desde que se leyó `current`, el
    /// `WHERE` no encuentra ninguna fila (`rows_affected() == 0`) y se
    /// devuelve [`InstrumentRepositoryError::VersionConflict`] en vez de
    /// pisar el cambio ajeno.
    pub async fn update_master(
        &self,
        current: &InstrumentMasterRow,
        revision: InstrumentMasterRevision,
    ) -> Result<InstrumentMasterRow, InstrumentRepositoryError> {
        let now_ns = self.clock.timestamp_ns();
        let row_version = current.row_version + 1;

        let audit_hash = compute_instrument_master_audit_hash(
            &current.id,
            now_ns,
            row_version,
            Some(&current.audit_hash),
            &current.instrument_id,
            current.asset_class,
            revision.canonical_pip_size,
            revision.canonical_tick_size,
            revision.default_session_template_id.as_deref(),
        );

        // La guarda `row_version = ?` es la comparación optimista: solo
        // actualiza si la fila en disco sigue en la versión que leímos.
        let result = sqlx::query(
            "UPDATE instrument_master SET \
                updated_at = ?, audit_hash = ?, audit_chain_hash = ?, row_version = ?, \
                canonical_pip_size = ?, canonical_tick_size = ?, default_session_template_id = ? \
             WHERE id = ? AND row_version = ?",
        )
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&current.audit_hash)
        .bind(row_version)
        .bind(revision.canonical_pip_size)
        .bind(revision.canonical_tick_size)
        .bind(&revision.default_session_template_id)
        .bind(&current.id)
        .bind(current.row_version)
        .execute(self.pool)
        .await?;

        // Cero filas afectadas => la fila ya no está en `current.row_version`
        // (otra edición la adelantó). No pisamos: reportamos el conflicto.
        if result.rows_affected() == 0 {
            return Err(InstrumentRepositoryError::VersionConflict { id: current.id.clone(), expected: current.row_version });
        }

        Ok(InstrumentMasterRow {
            updated_at: now_ns,
            audit_hash,
            audit_chain_hash: Some(current.audit_hash.clone()),
            row_version,
            canonical_pip_size: revision.canonical_pip_size,
            canonical_tick_size: revision.canonical_tick_size,
            default_session_template_id: revision.default_session_template_id,
            ..current.clone()
        })
    }

    // ── broker_instrument_terms ──────────────────────────────────────

    /// Da de alta el override de un bróker sobre un instrumento, con
    /// `row_version = 1`. Un `(instrument_id, venue)` duplicado se
    /// rechaza (cada bróker tiene UNA sola fila de override).
    pub async fn create_broker_terms(
        &self,
        new: NewBrokerInstrumentTerms,
    ) -> Result<BrokerInstrumentTermsRow, InstrumentRepositoryError> {
        let id = Uuid::now_v7().to_string();
        let now_ns = self.clock.timestamp_ns();
        let row_version: i64 = 1;

        let commission_model_str = new.commission_model.map(CommissionModel::as_str);
        let audit_hash = compute_broker_terms_audit_hash(
            &id,
            now_ns,
            row_version,
            None,
            &new.instrument_id,
            &new.venue,
            new.default_spread,
            new.session_template_id_override.as_deref(),
        );

        let result = sqlx::query(
            "INSERT INTO broker_instrument_terms (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                node_id, process_id, \
                instrument_id, venue, venue_symbol, \
                default_spread, default_slippage, pip_size_override, tick_size_override, point_value, \
                commission_model, commission_value, session_template_id_override\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(Option::<String>::None)
        .bind(row_version)
        .bind(&new.node_id)
        .bind(&new.process_id)
        .bind(&new.instrument_id)
        .bind(&new.venue)
        .bind(&new.venue_symbol)
        .bind(new.default_spread)
        .bind(new.default_slippage)
        .bind(new.pip_size_override)
        .bind(new.tick_size_override)
        .bind(new.point_value)
        .bind(commission_model_str)
        .bind(new.commission_value)
        .bind(&new.session_template_id_override)
        .execute(self.pool)
        .await;

        if let Err(sqlx::Error::Database(db_err)) = &result {
            if db_err.is_unique_violation() {
                return Err(InstrumentRepositoryError::DuplicateBrokerOverride {
                    instrument_id: new.instrument_id,
                    venue: new.venue,
                });
            }
        }
        result?;

        Ok(BrokerInstrumentTermsRow {
            id,
            created_at: now_ns,
            updated_at: now_ns,
            audit_hash,
            audit_chain_hash: None,
            row_version,
            node_id: new.node_id,
            process_id: new.process_id,
            instrument_id: new.instrument_id,
            venue: new.venue,
            venue_symbol: new.venue_symbol,
            default_spread: new.default_spread,
            default_slippage: new.default_slippage,
            pip_size_override: new.pip_size_override,
            tick_size_override: new.tick_size_override,
            point_value: new.point_value,
            commission_model: new.commission_model,
            commission_value: new.commission_value,
            session_template_id_override: new.session_template_id_override,
        })
    }

    /// Carga el override de un bróker concreto sobre un instrumento, o
    /// `None` si ese bróker no tiene override declarado (resuelve 100%
    /// desde el canónico).
    pub async fn find_broker_terms(
        &self,
        instrument_id: &str,
        venue: &str,
    ) -> Result<Option<BrokerInstrumentTermsRow>, InstrumentRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    node_id, process_id, \
                    instrument_id, venue, venue_symbol, \
                    default_spread, default_slippage, pip_size_override, tick_size_override, point_value, \
                    commission_model, commission_value, session_template_id_override \
             FROM broker_instrument_terms WHERE instrument_id = ? AND venue = ?",
        )
        .bind(instrument_id)
        .bind(venue)
        .fetch_optional(self.pool)
        .await?;

        row.map(row_to_broker_terms).transpose()
    }

    /// Borra el override de un bróker sobre un instrumento (criterio de
    /// aceptación #3: "el usuario borra el override de un bróker"). Tras
    /// esto, [`Self::find_broker_terms`] devuelve `None` y la resolución
    /// (`domain::resolve_terms`) cae 100% al canónico -- sin tocar
    /// `instrument_master`. NUNCA es soft-delete (ADR-0141 lo prohíbe para
    /// tablas de estado, pero esta fila no es un estado -- es una
    /// declaración sparse que el usuario puede retirar por completo).
    ///
    /// Devuelve `true` si había una fila que borrar, `false` si ya no
    /// existía (idempotente).
    pub async fn delete_broker_terms(&self, instrument_id: &str, venue: &str) -> Result<bool, InstrumentRepositoryError> {
        let result = sqlx::query("DELETE FROM broker_instrument_terms WHERE instrument_id = ? AND venue = ?")
            .bind(instrument_id)
            .bind(venue)
            .execute(self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // ── broker_server_clock (retrofit STORY-070/TASK-071) ────────────

    /// Da de alta el reloj de servidor declarado de un bróker/venue, con
    /// `row_version = 1`. Un `venue` duplicado se rechaza (cada bróker
    /// tiene UN solo reloj declarado).
    pub async fn create_broker_server_clock(
        &self,
        new: NewBrokerServerClock,
    ) -> Result<BrokerServerClockRow, InstrumentRepositoryError> {
        let id = Uuid::now_v7().to_string();
        let now_ns = self.clock.timestamp_ns();
        let row_version: i64 = 1;

        let audit_hash = compute_broker_server_clock_audit_hash(
            &id,
            now_ns,
            row_version,
            None,
            &new.venue,
            &new.clock_anchor_timezone_ref,
            new.clock_fixed_offset_minutes,
            new.venue_operational_since_ns,
        );

        let result = sqlx::query(
            "INSERT INTO broker_server_clock (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                node_id, process_id, \
                venue, clock_anchor_timezone_ref, clock_fixed_offset_minutes, venue_operational_since_ns\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(Option::<String>::None)
        .bind(row_version)
        .bind(&new.node_id)
        .bind(&new.process_id)
        .bind(&new.venue)
        .bind(&new.clock_anchor_timezone_ref)
        .bind(new.clock_fixed_offset_minutes)
        .bind(new.venue_operational_since_ns)
        .execute(self.pool)
        .await;

        if let Err(sqlx::Error::Database(db_err)) = &result {
            if is_duplicate_venue_violation(&**db_err) {
                return Err(InstrumentRepositoryError::DuplicateVenue(new.venue));
            }
        }
        result?;

        Ok(BrokerServerClockRow {
            id,
            created_at: now_ns,
            updated_at: now_ns,
            audit_hash,
            audit_chain_hash: None,
            row_version,
            node_id: new.node_id,
            process_id: new.process_id,
            venue: new.venue,
            clock_anchor_timezone_ref: new.clock_anchor_timezone_ref,
            clock_fixed_offset_minutes: new.clock_fixed_offset_minutes,
            venue_operational_since_ns: new.venue_operational_since_ns,
        })
    }

    /// Carga el reloj de servidor declarado de un bróker por su clave de
    /// NEGOCIO (`venue`) -- la búsqueda que usa quien orquesta un import
    /// para inyectar `venue_operational_since_ns` en `data-import-wizard`
    /// (ADR-0137: esa feature nunca consulta esta tabla directamente).
    pub async fn find_broker_server_clock_by_venue(
        &self,
        venue: &str,
    ) -> Result<Option<BrokerServerClockRow>, InstrumentRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    node_id, process_id, \
                    venue, clock_anchor_timezone_ref, clock_fixed_offset_minutes, venue_operational_since_ns \
             FROM broker_server_clock WHERE venue = ?",
        )
        .bind(venue)
        .fetch_optional(self.pool)
        .await?;

        Ok(row.map(row_to_broker_server_clock))
    }

    /// Revisa el reloj de servidor declarado de un bróker.
    ///
    /// ## Concurrencia optimista (ADR-0141)
    ///
    /// Mismo mecanismo que [`Self::update_master`]: el `UPDATE` filtra por
    /// `id` **y** `row_version = <el de `current`>`. Esta fila guarda el
    /// reloj del bróker -- el borrador desde el que se pre-rellena la
    /// declaración que el usuario sella como evidencia (criterio de
    /// aceptación #13 de STORY-070) -- así que dos ediciones concurrentes
    /// que partan del mismo `row_version` NUNCA deben poder tener éxito
    /// ambas: la segunda debe recibir [`InstrumentRepositoryError::VersionConflict`]
    /// en vez de pisar en silencio el cambio de la primera.
    pub async fn update_broker_server_clock(
        &self,
        current: &BrokerServerClockRow,
        revision: BrokerServerClockRevision,
    ) -> Result<BrokerServerClockRow, InstrumentRepositoryError> {
        let now_ns = self.clock.timestamp_ns();
        let row_version = current.row_version + 1;

        let audit_hash = compute_broker_server_clock_audit_hash(
            &current.id,
            now_ns,
            row_version,
            Some(&current.audit_hash),
            &current.venue,
            &revision.clock_anchor_timezone_ref,
            revision.clock_fixed_offset_minutes,
            revision.venue_operational_since_ns,
        );

        // La guarda `row_version = ?` es la comparación optimista: solo
        // actualiza si la fila en disco sigue en la versión que leímos.
        let result = sqlx::query(
            "UPDATE broker_server_clock SET \
                updated_at = ?, audit_hash = ?, audit_chain_hash = ?, row_version = ?, \
                clock_anchor_timezone_ref = ?, clock_fixed_offset_minutes = ?, venue_operational_since_ns = ? \
             WHERE id = ? AND row_version = ?",
        )
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&current.audit_hash)
        .bind(row_version)
        .bind(&revision.clock_anchor_timezone_ref)
        .bind(revision.clock_fixed_offset_minutes)
        .bind(revision.venue_operational_since_ns)
        .bind(&current.id)
        .bind(current.row_version)
        .execute(self.pool)
        .await?;

        // Cero filas afectadas => la fila ya no está en `current.row_version`
        // (otra edición la adelantó). No pisamos: reportamos el conflicto.
        if result.rows_affected() == 0 {
            return Err(InstrumentRepositoryError::VersionConflict { id: current.id.clone(), expected: current.row_version });
        }

        Ok(BrokerServerClockRow {
            updated_at: now_ns,
            audit_hash,
            audit_chain_hash: Some(current.audit_hash.clone()),
            row_version,
            clock_anchor_timezone_ref: revision.clock_anchor_timezone_ref,
            clock_fixed_offset_minutes: revision.clock_fixed_offset_minutes,
            venue_operational_since_ns: revision.venue_operational_since_ns,
            ..current.clone()
        })
    }
}

/// Mapea una fila cruda de `instrument_master` al tipo [`InstrumentMasterRow`].
/// Decide si un error de base de datos es una violación UNIQUE de la
/// clave de NEGOCIO `instrument_id` -- transformable a
/// [`InstrumentRepositoryError::DuplicateInstrumentId`]. Una violación
/// UNIQUE de OTRA columna (en particular la PK técnica `id`) NO cuenta:
/// debe propagarse como el error genérico de base de datos, nunca
/// camuflarse como duplicado de negocio (mismo criterio que
/// `is_transient_write_conflict` en los ledgers append-only del
/// substrato -- clasificar mal aquí bloquearía un alta legítima con un
/// mensaje de error que apunta a la columna equivocada).
fn is_duplicate_instrument_id_violation(db_err: &(dyn sqlx::error::DatabaseError + 'static)) -> bool {
    db_err.is_unique_violation() && db_err.message().to_lowercase().contains("instrument_id")
}

/// Mismo criterio que [`is_duplicate_instrument_id_violation`], para la
/// clave de negocio `venue` de `broker_server_clock` -- distingue una
/// violación UNIQUE de esa columna (duplicado de negocio) de una violación
/// UNIQUE de la PK técnica `id` (que NUNCA debe camuflarse como duplicado).
fn is_duplicate_venue_violation(db_err: &(dyn sqlx::error::DatabaseError + 'static)) -> bool {
    db_err.is_unique_violation() && db_err.message().to_lowercase().contains("venue")
}

fn row_to_master(row: sqlx::sqlite::SqliteRow) -> Result<InstrumentMasterRow, InstrumentRepositoryError> {
    let asset_class_value: String = row.get("asset_class");
    let asset_class =
        AssetClass::from_str_value(&asset_class_value).ok_or(InstrumentRepositoryError::UnknownAssetClass(asset_class_value))?;

    Ok(InstrumentMasterRow {
        id: row.get("id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        row_version: row.get("row_version"),
        node_id: row.get("node_id"),
        process_id: row.get("process_id"),
        instrument_id: row.get("instrument_id"),
        asset_class,
        base_currency: row.get("base_currency"),
        quote_currency: row.get("quote_currency"),
        canonical_pip_size: row.get("canonical_pip_size"),
        canonical_tick_size: row.get("canonical_tick_size"),
        default_session_template_id: row.get("default_session_template_id"),
    })
}

/// Mapea una fila cruda de `broker_instrument_terms` al tipo
/// [`BrokerInstrumentTermsRow`].
fn row_to_broker_terms(row: sqlx::sqlite::SqliteRow) -> Result<BrokerInstrumentTermsRow, InstrumentRepositoryError> {
    let commission_model_value: Option<String> = row.get("commission_model");
    let commission_model = commission_model_value
        .map(|v| CommissionModel::from_str_value(&v).ok_or(InstrumentRepositoryError::UnknownCommissionModel(v)))
        .transpose()?;

    Ok(BrokerInstrumentTermsRow {
        id: row.get("id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        row_version: row.get("row_version"),
        node_id: row.get("node_id"),
        process_id: row.get("process_id"),
        instrument_id: row.get("instrument_id"),
        venue: row.get("venue"),
        venue_symbol: row.get("venue_symbol"),
        default_spread: row.get("default_spread"),
        default_slippage: row.get("default_slippage"),
        pip_size_override: row.get("pip_size_override"),
        tick_size_override: row.get("tick_size_override"),
        point_value: row.get("point_value"),
        commission_model,
        commission_value: row.get("commission_value"),
        session_template_id_override: row.get("session_template_id_override"),
    })
}

/// Mapea una fila cruda de `broker_server_clock` al tipo
/// [`BrokerServerClockRow`]. Sin decodificación de enum -- a diferencia de
/// `instrument_master`/`broker_instrument_terms`, ninguna columna de esta
/// tabla tiene vocabulario fijo, así que no puede fallar.
fn row_to_broker_server_clock(row: sqlx::sqlite::SqliteRow) -> BrokerServerClockRow {
    BrokerServerClockRow {
        id: row.get("id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        row_version: row.get("row_version"),
        node_id: row.get("node_id"),
        process_id: row.get("process_id"),
        venue: row.get("venue"),
        clock_anchor_timezone_ref: row.get("clock_anchor_timezone_ref"),
        clock_fixed_offset_minutes: row.get("clock_fixed_offset_minutes"),
        venue_operational_since_ns: row.get("venue_operational_since_ns"),
    }
}

// ── Pruebas de la capa de persistencia ───────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use shared::public_interface::{create_pool, run_migrations, DeterministicClock};

    /// Crea una base de datos SQLite en un archivo temporal y aplica todas
    /// las migraciones -- nunca `:memory:` (durabilidad real, mismo patrón
    /// que `hybrid_data_transformer::persistence::tests::migrated_pool`).
    async fn migrated_pool() -> (SqlitePool, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = dir.path().join("test.sqlite");
        let url = format!("sqlite://{}", db_path.display());
        let pool = create_pool(&url).await.expect("abrir base de datos temporal");
        run_migrations(&pool).await.expect("aplicar migraciones");
        (pool, dir)
    }

    fn sample_new_master() -> NewInstrumentMaster {
        NewInstrumentMaster {
            instrument_id: "EURUSD".to_string(),
            asset_class: AssetClass::Forex,
            base_currency: Some("EUR".to_string()),
            quote_currency: Some("USD".to_string()),
            canonical_pip_size: 10_000,
            canonical_tick_size: 1_000,
            default_session_template_id: None,
            node_id: Some("node-A".to_string()),
            process_id: Some("pid-1".to_string()),
        }
    }

    // ── CRITERIO #1 (Orden §5): instrument_master_rejects_duplicate_business_key ──

    /// Alta de instrumento canónico persiste con `id` UUIDv7 +
    /// `instrument_id` de negocio único; un `instrument_id` duplicado se
    /// rechaza explícitamente -- NUNCA pisa la fila existente en silencio.
    #[tokio::test]
    async fn instrument_master_rejects_duplicate_business_key() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = InstrumentRepository::new(&pool, &clock);

        let created = repo.create_master(sample_new_master()).await.expect("primera alta debe tener éxito");
        assert_eq!(created.row_version, 1);
        assert_eq!(created.instrument_id, "EURUSD");
        // UUIDv7 es un string no vacío con guiones -- verificación mínima
        // de forma sin acoplar la prueba al formato exacto de la librería.
        assert!(created.id.contains('-'), "id debe ser un UUID con guiones: {}", created.id);

        let duplicate = repo.create_master(sample_new_master()).await;
        assert!(
            matches!(duplicate, Err(InstrumentRepositoryError::DuplicateInstrumentId(ref id)) if id == "EURUSD"),
            "un instrument_id duplicado debe rechazarse explícitamente, no pisar la fila existente; fue: {duplicate:?}"
        );

        // Solo debe existir UNA fila -- el intento fallido no dejó un
        // segundo registro a medias.
        let count: i64 = sqlx::query("SELECT COUNT(*) FROM instrument_master")
            .fetch_one(&pool)
            .await
            .expect("contar filas")
            .get(0);
        assert_eq!(count, 1);
    }

    // ── FIX B (gate de mutación, hallazgo Tech-Lead 2026-07-25): el
    // clasificador de duplicado distingue PK de clave de negocio ──────────

    /// CRITERIO DE CIERRE (mutación `&&`→`||`): la prueba anterior
    /// (`instrument_master_rejects_duplicate_business_key`) solo ejercita
    /// el caso donde AMBAS mitades de la guarda (`is_unique_violation()`
    /// Y `message().contains("instrument_id")`) son verdaderas -- ahí
    /// `&&` y `||` dan el MISMO resultado, así que un mutante que cambie
    /// el operador sobrevive sin que esa prueba se entere. Esta prueba
    /// fuerza el caso donde SOLO la primera mitad es verdadera: una
    /// violación UNIQUE de la PK técnica `id` (mismo `id`, `instrument_id`
    /// DISTINTO) -- con `&&` correcto, NO debe clasificarse como
    /// duplicado de negocio; con `||`, sí lo haría (mapearía mal a
    /// `DuplicateInstrumentId` aunque el mensaje real hable de `id`, no de
    /// `instrument_id`). Ejercita `is_duplicate_instrument_id_violation`
    /// DIRECTAMENTE con errores reales de SQLite -- mismo patrón que
    /// `hybrid_data_transformer::persistence::is_transient_write_conflict`.
    #[tokio::test]
    async fn is_duplicate_instrument_id_violation_distinguishes_business_key_from_primary_key() {
        let (pool, _dir) = migrated_pool().await;

        let insert = |id: &'static str, instrument_id: &'static str| {
            let pool = pool.clone();
            async move {
                sqlx::query(
                    "INSERT INTO instrument_master (id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, instrument_id, asset_class, canonical_pip_size, canonical_tick_size) \
                     VALUES (?, 0, 0, 'h', NULL, 1, ?, 'FOREX', 0, 0)",
                )
                .bind(id)
                .bind(instrument_id)
                .execute(&pool)
                .await
            }
        };

        insert("id-a", "EURUSD").await.expect("primera fila válida");

        // Violación de la CLAVE DE NEGOCIO: mismo instrument_id, id distinto
        // -- el mensaje de SQLite SÍ menciona "instrument_id".
        let business_key_err = insert("id-b", "EURUSD").await.expect_err("debe violar el UNIQUE de instrument_id");
        let sqlx::Error::Database(business_key_db_err) = business_key_err else {
            panic!("se esperaba un error de base de datos")
        };
        assert!(
            is_duplicate_instrument_id_violation(&*business_key_db_err),
            "una violación UNIQUE de instrument_id SÍ es un duplicado de clave de negocio"
        );

        // Violación de la PK `id`: MISMO id, instrument_id DISTINTO -- el
        // mensaje de SQLite habla de "instrument_master.id", NUNCA de
        // "instrument_id" -- NUNCA debe clasificarse como duplicado de negocio.
        let pk_err = insert("id-a", "GBPUSD").await.expect_err("debe violar la PRIMARY KEY id");
        let sqlx::Error::Database(pk_db_err) = pk_err else { panic!("se esperaba un error de base de datos") };
        assert!(
            !is_duplicate_instrument_id_violation(&*pk_db_err),
            "una violación UNIQUE de la PK id NUNCA debe clasificarse como duplicado de instrument_id -- fue clasificada como duplicado, lo que mapearía mal el error al usuario"
        );
    }

    // ── CRITERIO #3 (Orden §5, capa de persistencia): borrar override ────

    #[tokio::test]
    async fn deleting_broker_terms_row_makes_find_return_none() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = InstrumentRepository::new(&pool, &clock);

        repo.create_master(sample_new_master()).await.expect("crear canónico");
        repo.create_broker_terms(NewBrokerInstrumentTerms {
            instrument_id: "EURUSD".to_string(),
            venue: "OANDA".to_string(),
            default_spread: Some(150_000_000),
            ..Default::default()
        })
        .await
        .expect("crear override");

        let before = repo.find_broker_terms("EURUSD", "OANDA").await.expect("buscar override");
        assert!(before.is_some(), "el override debe existir antes de borrarlo");

        let deleted = repo.delete_broker_terms("EURUSD", "OANDA").await.expect("borrar override");
        assert!(deleted, "delete_broker_terms debe reportar que sí había una fila");

        let after = repo.find_broker_terms("EURUSD", "OANDA").await.expect("buscar override tras borrar");
        assert!(after.is_none(), "tras borrar, find_broker_terms debe devolver None -- sin fila fantasma");

        // instrument_master NUNCA se tocó -- el canónico sigue intacto.
        let canonical = repo.find_master_by_instrument_id("EURUSD").await.expect("buscar canónico").expect("debe existir");
        assert_eq!(canonical.row_version, 1, "borrar el override no debe tocar instrument_master");
    }

    /// Borrar un override que no existe es idempotente: devuelve `false`,
    /// no un error.
    #[tokio::test]
    async fn deleting_nonexistent_broker_terms_returns_false_not_error() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = InstrumentRepository::new(&pool, &clock);

        let deleted = repo.delete_broker_terms("EURUSD", "OANDA").await.expect("no debe fallar");
        assert!(!deleted, "borrar un override inexistente debe devolver false");
    }

    // ── CRITERIO #7 (Orden §5): stale_row_version_yields_version_conflict ──

    /// Concurrencia optimista real: dos ediciones que parten del MISMO
    /// `row_version` en memoria no pueden ambas tener éxito. La primera
    /// pasa; la segunda, que sigue creyendo estar en la versión vieja,
    /// devuelve `VersionConflict` (`rows_affected == 0`) en vez de pisar
    /// el cambio de la primera en silencio.
    ///
    /// Esta prueba FALLA si se quita la guarda `AND row_version = ?` del
    /// `UPDATE`: sin ella, la segunda edición también afectaría 1 fila y
    /// devolvería `Ok`, bifurcando la cadena `audit_hash`.
    #[tokio::test]
    async fn stale_row_version_yields_version_conflict() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = InstrumentRepository::new(&pool, &clock);

        let created = repo.create_master(sample_new_master()).await.expect("crear canónico");
        assert_eq!(created.row_version, 1);

        // Dos "actores" leyeron el MISMO instrumento en la versión 1.
        let first_writer_view = created.clone();
        let second_writer_view = created;

        clock.tick();
        let updated = repo
            .update_master(
                &first_writer_view,
                InstrumentMasterRevision {
                    canonical_pip_size: 20_000,
                    canonical_tick_size: 2_000,
                    default_session_template_id: None,
                },
            )
            .await
            .expect("la primera edición debe tener éxito");
        assert_eq!(updated.row_version, 2);
        assert_eq!(updated.canonical_pip_size, 20_000);
        assert_ne!(updated.audit_hash, first_writer_view.audit_hash, "el audit_hash debe recalcularse en la edición");
        assert_eq!(
            updated.audit_chain_hash,
            Some(first_writer_view.audit_hash.clone()),
            "audit_chain_hash debe encadenar al audit_hash de la versión anterior"
        );

        clock.tick();
        let conflict = repo
            .update_master(
                &second_writer_view,
                InstrumentMasterRevision {
                    canonical_pip_size: 99_999,
                    canonical_tick_size: 9_999,
                    default_session_template_id: None,
                },
            )
            .await;
        assert!(
            matches!(conflict, Err(InstrumentRepositoryError::VersionConflict { expected: 1, .. })),
            "la segunda edición desde la versión 1 debe dar VersionConflict, no éxito silencioso; fue: {conflict:?}"
        );

        // La fila en disco conserva el cambio de la PRIMERA edición.
        let reloaded = repo.find_master_by_instrument_id("EURUSD").await.expect("releer").expect("debe existir");
        assert_eq!(reloaded.row_version, 2);
        assert_eq!(reloaded.canonical_pip_size, 20_000, "el valor en disco debe ser el de la primera edición, no el de la segunda");
    }

    // ── FIX A (gate de mutación, hallazgo Tech-Lead 2026-07-25): fidelidad
    // del struct DEVUELTO por update_master ─────────────────────────────

    /// `update_master` construye el struct de retorno con `..current.clone()`
    /// como base y sobreescribe los campos que cambiaron -- si se borrara
    /// alguna de esas sobreescrituras, el campo caería silenciosamente al
    /// valor VIEJO (el de `current`) en vez de al nuevo, y ninguna prueba
    /// anterior lo detectaría porque ninguna comparaba el struct DEVUELTO
    /// campo a campo contra la revisión (solo comparaban `row_version` y
    /// `canonical_pip_size`). Esta prueba cambia TRES campos a la vez
    /// (`updated_at` vía el tick del reloj, `canonical_tick_size` y
    /// `default_session_template_id` -- este último de `None` a `Some`,
    /// para que un mutante que borre esa línea sea detectable) y afirma
    /// sobre el valor que `update_master` DEVUELVE, no sobre una relectura.
    #[tokio::test]
    async fn update_master_return_value_reflects_every_changed_field() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = InstrumentRepository::new(&pool, &clock);

        let created = repo.create_master(sample_new_master()).await.expect("crear canónico");
        assert_eq!(created.default_session_template_id, None, "el canónico de la muestra arranca sin plantilla de sesión");

        // La FK `default_session_template_id -> session_template(id)` exige
        // que la plantilla exista de verdad -- `instrument-catalog` no
        // depende de `session-catalog` (ADR-0137), así que se inserta
        // directamente por SQL crudo solo para satisfacer la FK de esta
        // prueba.
        sqlx::query(
            "INSERT INTO session_template (id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, template_code, display_name, market_timezone_ref) \
             VALUES ('tpl-new-session', 0, 0, 'h', NULL, 1, 'TEST_TEMPLATE', 'Test Template', 'UTC')",
        )
        .execute(&pool)
        .await
        .expect("insertar plantilla de sesión de prueba");

        clock.tick();
        let updated = repo
            .update_master(
                &created,
                InstrumentMasterRevision {
                    canonical_pip_size: created.canonical_pip_size, // sin cambio -- aísla los campos bajo prueba
                    canonical_tick_size: 55_555,
                    default_session_template_id: Some("tpl-new-session".to_string()),
                },
            )
            .await
            .expect("la edición debe tener éxito");

        assert_ne!(updated.updated_at, created.updated_at, "updated_at DEVUELTO debe ser el nuevo timestamp, no el viejo");
        assert_eq!(updated.canonical_tick_size, 55_555, "canonical_tick_size DEVUELTO debe ser el nuevo valor, no el viejo (1000 -- ver sample_new_master)");
        assert_ne!(updated.canonical_tick_size, created.canonical_tick_size, "el valor nuevo debe ser DISTINTO del viejo para que la prueba no sea trivial");
        assert_eq!(
            updated.default_session_template_id,
            Some("tpl-new-session".to_string()),
            "default_session_template_id DEVUELTO debe ser el nuevo valor, no el None original"
        );
    }

    // ── CRITERIO #8 (Orden §5): check_constraints_reject_invalid_enum ────

    /// Un `asset_class` fuera del vocabulario fijo es rechazado por la BD
    /// (el `CHECK` de la migración), no por la capa Rust -- inserta
    /// directamente con SQL crudo, sin pasar por `create_master`, para
    /// ejercitar el guardarraíl de la BD en sí mismo.
    #[tokio::test]
    async fn check_constraints_reject_invalid_enum() {
        let (pool, _dir) = migrated_pool().await;

        let result = sqlx::query(
            "INSERT INTO instrument_master (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                instrument_id, asset_class, canonical_pip_size, canonical_tick_size\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'XAUUSD', 'BOND', 0, 0)",
        )
        .execute(&pool)
        .await;

        assert!(result.is_err(), "un asset_class fuera de ('FOREX','FUTURE','CFD','CRYPTO','EQUITY') debe ser rechazado por el CHECK de la BD");
    }

    /// Mismo criterio, para `commission_model` en `broker_instrument_terms`.
    #[tokio::test]
    async fn check_constraints_reject_invalid_commission_model() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = InstrumentRepository::new(&pool, &clock);
        repo.create_master(sample_new_master()).await.expect("crear canónico");

        let result = sqlx::query(
            "INSERT INTO broker_instrument_terms (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                instrument_id, venue, commission_model\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'EURUSD', 'OANDA', 'FLAT_FEE')",
        )
        .execute(&pool)
        .await;

        assert!(result.is_err(), "un commission_model fuera del vocabulario fijo debe ser rechazado por el CHECK de la BD");
    }

    // ── CRITERIO #2 (Orden §5, verificado también a nivel de persistencia real) ──

    /// La resolución vía el Núcleo (`domain::resolve_terms`) usando datos
    /// REALMENTE persistidos y releídos -- no solo structs construidos a
    /// mano en el test de dominio. Cierra el camino completo: escribir,
    /// leer, resolver.
    #[tokio::test]
    async fn resolved_terms_from_persisted_rows_apply_null_inheritance() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = InstrumentRepository::new(&pool, &clock);

        repo.create_master(sample_new_master()).await.expect("crear canónico");
        repo.create_broker_terms(NewBrokerInstrumentTerms {
            instrument_id: "EURUSD".to_string(),
            venue: "OANDA".to_string(),
            default_spread: Some(150_000_000),
            pip_size_override: None, // hereda
            ..Default::default()
        })
        .await
        .expect("crear override");

        let canonical = repo.find_master_by_instrument_id("EURUSD").await.expect("buscar").expect("debe existir");
        let override_row = repo.find_broker_terms("EURUSD", "OANDA").await.expect("buscar override").expect("debe existir");

        let resolved = crate::domain::resolve_terms(
            &crate::domain::CanonicalTerms {
                instrument_id: canonical.instrument_id.clone(),
                asset_class: canonical.asset_class,
                canonical_pip_size: canonical.canonical_pip_size,
                canonical_tick_size: canonical.canonical_tick_size,
                default_session_template_id: canonical.default_session_template_id.clone(),
            },
            Some(&crate::domain::BrokerOverride {
                venue: override_row.venue.clone(),
                default_spread: override_row.default_spread,
                pip_size_override: override_row.pip_size_override,
                ..Default::default()
            }),
        );

        assert_eq!(resolved.pip_size, canonical.canonical_pip_size, "pip_size_override NULL debe heredar el canónico persistido");
        assert_eq!(resolved.spread, Some(150_000_000), "default_spread no-NULL debe ganar");
    }

    // ═══════════════════════════════════════════════════════════════════
    // broker_server_clock (retrofit STORY-070/TASK-071, criterio de
    // aceptación #13 de STORY-070) -- mismo Grupo I + concurrencia
    // optimista que instrument_master, probado con el mismo molde.
    // ═══════════════════════════════════════════════════════════════════

    fn sample_new_broker_server_clock() -> NewBrokerServerClock {
        NewBrokerServerClock {
            venue: "IC_MARKETS".to_string(),
            clock_anchor_timezone_ref: "America/New_York".to_string(),
            clock_fixed_offset_minutes: 420,
            venue_operational_since_ns: Some(1_167_609_600_000_000_000), // 2007-01-01T00:00:00Z, fundación declarada
            node_id: Some("node-A".to_string()),
            process_id: Some("pid-1".to_string()),
        }
    }

    /// Alta del reloj de servidor: persiste con `row_version = 1`; un
    /// `venue` duplicado se rechaza explícitamente -- NUNCA pisa la fila
    /// existente en silencio (mismo criterio que
    /// `instrument_master_rejects_duplicate_business_key`).
    #[tokio::test]
    async fn broker_server_clock_rejects_duplicate_venue() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = InstrumentRepository::new(&pool, &clock);

        let created = repo.create_broker_server_clock(sample_new_broker_server_clock()).await.expect("primera alta debe tener éxito");
        assert_eq!(created.row_version, 1);
        assert_eq!(created.venue, "IC_MARKETS");
        assert_eq!(created.clock_fixed_offset_minutes, 420);
        assert_eq!(created.venue_operational_since_ns, Some(1_167_609_600_000_000_000));

        let duplicate = repo.create_broker_server_clock(sample_new_broker_server_clock()).await;
        assert!(
            matches!(duplicate, Err(InstrumentRepositoryError::DuplicateVenue(ref v)) if v == "IC_MARKETS"),
            "un venue duplicado debe rechazarse explícitamente, no pisar la fila existente; fue: {duplicate:?}"
        );

        let count: i64 =
            sqlx::query("SELECT COUNT(*) FROM broker_server_clock").fetch_one(&pool).await.expect("contar filas").get(0);
        assert_eq!(count, 1, "el intento fallido no debe dejar una segunda fila a medias");
    }

    /// El clasificador de duplicado distingue la clave de NEGOCIO (`venue`)
    /// de la PK técnica (`id`) -- mismo hueco histórico que
    /// `is_duplicate_instrument_id_violation_distinguishes_business_key_from_primary_key`
    /// (hallazgo del Tech-Lead 2026-07-25 sobre `instrument_master`, mismo
    /// patrón de mutación `&&`→`||` posible aquí): ejercita
    /// `is_duplicate_venue_violation` DIRECTAMENTE con sus dos casos.
    #[tokio::test]
    async fn is_duplicate_venue_violation_distinguishes_business_key_from_primary_key() {
        let (pool, _dir) = migrated_pool().await;

        let insert = |id: &'static str, venue: &'static str| {
            let pool = pool.clone();
            async move {
                sqlx::query(
                    "INSERT INTO broker_server_clock (id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, venue, clock_anchor_timezone_ref) \
                     VALUES (?, 0, 0, 'h', NULL, 1, ?, 'UTC')",
                )
                .bind(id)
                .bind(venue)
                .execute(&pool)
                .await
            }
        };

        insert("id-a", "IC_MARKETS").await.expect("primera fila válida");

        // Violación de la CLAVE DE NEGOCIO: mismo venue, id distinto -- el
        // mensaje de SQLite SÍ menciona "venue".
        let business_key_err = insert("id-b", "IC_MARKETS").await.expect_err("debe violar el UNIQUE de venue");
        let sqlx::Error::Database(business_key_db_err) = business_key_err else { panic!("se esperaba un error de base de datos") };
        assert!(
            is_duplicate_venue_violation(&*business_key_db_err),
            "una violación UNIQUE de venue SÍ es un duplicado de clave de negocio"
        );

        // Violación de la PK `id`: MISMO id, venue DISTINTO -- el mensaje de
        // SQLite habla de "broker_server_clock.id", NUNCA de "venue".
        let pk_err = insert("id-a", "PEPPERSTONE").await.expect_err("debe violar la PRIMARY KEY id");
        let sqlx::Error::Database(pk_db_err) = pk_err else { panic!("se esperaba un error de base de datos") };
        assert!(
            !is_duplicate_venue_violation(&*pk_db_err),
            "una violación UNIQUE de la PK id NUNCA debe clasificarse como duplicado de venue -- fue clasificada como duplicado"
        );
    }

    /// Buscar por `venue`: `None` cuando no existe, `Some` tras sellar.
    #[tokio::test]
    async fn find_broker_server_clock_by_venue_returns_none_when_absent_and_some_when_sealed() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = InstrumentRepository::new(&pool, &clock);

        assert!(repo.find_broker_server_clock_by_venue("IC_MARKETS").await.expect("buscar").is_none());

        repo.create_broker_server_clock(sample_new_broker_server_clock()).await.expect("crear reloj");
        let found = repo.find_broker_server_clock_by_venue("IC_MARKETS").await.expect("buscar").expect("debe existir");
        assert_eq!(found.clock_anchor_timezone_ref, "America/New_York");
    }

    // ── CRITERIO #13 (STORY-070 §5): concurrencia optimista real ─────────

    /// El núcleo del criterio 13: dos ediciones que parten del MISMO
    /// `row_version` en memoria no pueden ambas tener éxito. La primera
    /// pasa; la segunda, que sigue creyendo estar en la versión vieja,
    /// devuelve `VersionConflict` (`rows_affected == 0`) en vez de pisar el
    /// cambio de la primera en silencio -- mismo molde que
    /// `stale_row_version_yields_version_conflict` sobre `instrument_master`.
    ///
    /// Por qué esta fila importa más que la mecánica de la columna: guarda
    /// el reloj del bróker, el BORRADOR desde el que se pre-rellena la
    /// declaración que el usuario sella como evidencia (LDR-0008) -- si dos
    /// ediciones concurrentes se pisaran en silencio, el usuario podría
    /// sellar una declaración basada en un borrador que ya no es el vigente,
    /// sin que nada se lo advirtiera.
    ///
    /// Esta prueba FALLA si se quita la guarda `AND row_version = ?` del
    /// `UPDATE`: sin ella, la segunda edición también afectaría 1 fila y
    /// devolvería `Ok`, bifurcando la cadena `audit_hash` en silencio.
    #[tokio::test]
    async fn broker_server_clock_stale_row_version_yields_version_conflict() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = InstrumentRepository::new(&pool, &clock);

        let created = repo.create_broker_server_clock(sample_new_broker_server_clock()).await.expect("crear reloj");
        assert_eq!(created.row_version, 1);

        // Dos "actores" leyeron el MISMO reloj de bróker en la versión 1 --
        // ej. dos sesiones del asistente de importación abiertas a la vez,
        // ambas pre-rellenando su borrador desde la misma lectura.
        let first_editor_view = created.clone();
        let second_editor_view = created;

        clock.tick();
        let updated = repo
            .update_broker_server_clock(
                &first_editor_view,
                BrokerServerClockRevision {
                    clock_anchor_timezone_ref: "America/New_York".to_string(),
                    clock_fixed_offset_minutes: 480, // corrige el offset declarado
                    venue_operational_since_ns: first_editor_view.venue_operational_since_ns,
                },
            )
            .await
            .expect("la primera edición debe tener éxito");
        assert_eq!(updated.row_version, 2);
        assert_eq!(updated.clock_fixed_offset_minutes, 480);
        assert_ne!(updated.audit_hash, first_editor_view.audit_hash, "el audit_hash debe recalcularse en la edición");
        assert_eq!(
            updated.audit_chain_hash,
            Some(first_editor_view.audit_hash.clone()),
            "audit_chain_hash debe encadenar al audit_hash de la versión anterior"
        );

        clock.tick();
        let conflict = repo
            .update_broker_server_clock(
                &second_editor_view,
                BrokerServerClockRevision {
                    clock_anchor_timezone_ref: "America/New_York".to_string(),
                    clock_fixed_offset_minutes: 9_999, // valor claramente distinto -- si esto "ganara" en silencio, se notaría
                    venue_operational_since_ns: second_editor_view.venue_operational_since_ns,
                },
            )
            .await;
        assert!(
            matches!(conflict, Err(InstrumentRepositoryError::VersionConflict { expected: 1, .. })),
            "la segunda edición desde la versión 1 debe dar VersionConflict, no éxito silencioso; fue: {conflict:?}"
        );

        // La fila en disco conserva el cambio de la PRIMERA edición -- la
        // segunda NUNCA se aplicó, ni parcial ni completa.
        let reloaded = repo.find_broker_server_clock_by_venue("IC_MARKETS").await.expect("releer").expect("debe existir");
        assert_eq!(reloaded.row_version, 2);
        assert_eq!(reloaded.clock_fixed_offset_minutes, 480, "el valor en disco debe ser el de la primera edición, no el de la segunda (9999)");
    }

    /// Fidelidad del struct DEVUELTO por `update_broker_server_clock` --
    /// mismo criterio que `update_master_return_value_reflects_every_changed_field`
    /// (hallazgo del gate de mutación sobre `..current.clone()`): cambia
    /// TRES campos a la vez (`updated_at` vía el tick del reloj,
    /// `clock_anchor_timezone_ref` y `venue_operational_since_ns` -- este
    /// último de `Some` a `None`, para que un mutante que borre esa
    /// sobreescritura sea detectable) y afirma sobre el valor DEVUELTO, no
    /// sobre una relectura.
    #[tokio::test]
    async fn update_broker_server_clock_return_value_reflects_every_changed_field() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = InstrumentRepository::new(&pool, &clock);

        let created = repo.create_broker_server_clock(sample_new_broker_server_clock()).await.expect("crear reloj");
        assert_eq!(created.venue_operational_since_ns, Some(1_167_609_600_000_000_000));

        clock.tick();
        let updated = repo
            .update_broker_server_clock(
                &created,
                BrokerServerClockRevision {
                    clock_anchor_timezone_ref: "Europe/London".to_string(),
                    clock_fixed_offset_minutes: created.clock_fixed_offset_minutes, // sin cambio -- aísla los campos bajo prueba
                    venue_operational_since_ns: None, // de Some a None -- se retira la verificación declarada
                },
            )
            .await
            .expect("la edición debe tener éxito");

        assert_ne!(updated.updated_at, created.updated_at, "updated_at DEVUELTO debe ser el nuevo timestamp, no el viejo");
        assert_eq!(updated.clock_anchor_timezone_ref, "Europe/London", "clock_anchor_timezone_ref DEVUELTO debe ser el nuevo valor");
        assert_ne!(updated.clock_anchor_timezone_ref, created.clock_anchor_timezone_ref, "el valor nuevo debe ser DISTINTO del viejo");
        assert_eq!(updated.venue_operational_since_ns, None, "venue_operational_since_ns DEVUELTO debe reflejar el retiro (Some -> None), no quedarse en el valor viejo");
        assert_eq!(updated.venue, "IC_MARKETS", "venue NUNCA cambia -- no es parte de la revisión");
    }
}
