//! [SHELL] Repositorio de persistencia APPEND-ONLY para el Registro de
//! Consentimiento / ToS (`docs/features/consent-registry.md`, ADR-0143,
//! ADR-0144, ADR-0141, ADR-0020, migración `0011_consent_registry.sql`,
//! STORY-031).
//!
//! Envuelve la tabla `consent_records`. Dueño del único I/O de este
//! cimiento: lecturas/escrituras en SQLite, generación de UUIDv7
//! (ADR-0141) y la lectura del puerto [`Clock`]. La lógica pura (fusión de
//! una acción sobre el estado vigente, resolución de cobertura, hash
//! encadenado) vive en [`crate::domain::consent_registry`] -- este módulo
//! solo le da entradas inyectadas y persiste el resultado, reflejando el
//! patrón de [`crate::persistence::usage_metering::UsageRepository`]
//! (misma naturaleza APPEND-ONLY: `event_sequence_id UNIQUE`, sin
//! `row_version`).
//!
//! ## Por qué NO existe `update`/`delete` en esta API
//!
//! A propósito: la única operación de escritura que este repositorio
//! expone es [`ConsentRepository::record_action`] (un INSERT). No hay
//! ningún método de actualización o borrado -- ni falta, porque los
//! triggers `trg_consent_records_no_update`/`trg_consent_records_no_delete`
//! de la migración los rechazarían de cualquier forma. La ausencia del
//! método en Rust es la primera línea de defensa; el trigger de SQLite es
//! la segunda (defensa en profundidad).
//!
//! ## Cómo se lee el "estado vigente" con una tabla append-only
//!
//! [`ConsentRepository::load_latest_for_owner`] es la ÚNICA forma de leer
//! el estado lógico actual de un usuario: la fila con `event_sequence_id`
//! máximo para su `owner_id`. Ninguna consulta reconstruye el estado
//! recorriendo el historial completo (fold) -- cada fila ya trae el
//! snapshot completo (ver `domain::consent_registry::apply_consent_action`),
//! así que "leer la última fila" basta.

use std::collections::BTreeMap;

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::audit_log::GENESIS_PREVIOUS_HASH;
use crate::domain::clock::Clock;
use crate::domain::consent_registry::{
    apply_consent_action, compute_consent_audit_hash, ConsentAction, ConsentActionInput,
    ConsentState, OptoutMapError,
};

/// Errores que devuelven las operaciones de [`ConsentRepository`].
#[derive(Debug, thiserror::Error)]
pub enum ConsentRepositoryError {
    /// La operación subyacente de SQLite falló.
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// Una fila de `consent_records` tenía un `consent_action` fuera de
    /// las tres cadenas canónicas -- error de integridad de datos.
    #[error("consent_action desconocido en la fila '{0}' de consent_records")]
    UnknownConsentAction(String),
    /// El `optout_map` persistido no pudo parsearse de vuelta a
    /// `BTreeMap<String, bool>` -- no debería ocurrir nunca (el `CHECK
    /// (json_valid(...))` de la migración ya filtra JSON sintácticamente
    /// inválido), pero se propaga explícitamente en vez de hacer panic.
    #[error("optout_map corrupto en una fila persistida: {0}")]
    Optout(#[from] OptoutMapError),
    /// Falló la serialización del `optout_map` fusionado hacia JSON antes
    /// de persistirlo -- prácticamente imposible con `BTreeMap<String,
    /// bool>` (no hay claves no-string ni valores no serializables), pero
    /// se propaga en vez de usar `unwrap`.
    #[error("error al serializar optout_map a JSON: {0}")]
    Serialize(#[from] serde_json::Error),
    /// El append no pudo completarse tras agotar los reintentos ante
    /// contención de escritura transitoria (otro escritor mantuvo el lock
    /// de la base de datos más allá del `busy_timeout`, o hubo colisión
    /// repetida al derivar `event_sequence_id`). El evento NO se descartó
    /// en silencio -- se propaga este error tipado para que el llamador
    /// decida reintentar a un nivel superior o alertar (`docs/features/
    /// consent-registry.md`, regla "Atomicidad de ledgers append-only").
    #[error("no se pudo registrar el consentimiento tras {attempts} intentos por contención de escritura")]
    WriteContention { attempts: u32 },
    /// DEBT-007: una acción `OPTOUT_CHANGE` llegó como la PRIMERA acción de
    /// un `owner_id` (sin `ACCEPT`/estado previo). Antes, esta secuencia
    /// caía al efecto colateral `accepted_version=""` -> se resolvía luego
    /// como `StaleVersion` en `resolve_coverage`, sin dejar rastro explícito
    /// del problema real (falta un `ACCEPT`). Se rechaza aquí, antes de
    /// fusionar ni persistir nada -- la Shell debe registrar primero un
    /// `ACCEPT` para este dueño.
    #[error("OPTOUT_CHANGE no puede ser la primera acción registrada para el dueño '{owner_id}': falta un ACCEPT previo")]
    OptoutBeforeAccept { owner_id: String },
}

/// Número máximo de intentos del append ante contención de escritura
/// transitoria antes de rendirse con [`ConsentRepositoryError::WriteContention`].
/// Cinco es holgado: con `busy_timeout` de 5s (ADR-0141 R2) el lock casi
/// siempre se obtiene sin reintentar; el bucle solo actúa si el
/// `busy_timeout` expira bajo una contención extrema.
const MAX_RECORD_ATTEMPTS: u32 = 5;

/// Decide si un error del repositorio es una contención de escritura
/// TRANSITORIA -- es decir, algo que reintentar (re-derivando el
/// `event_sequence_id` y reinsertando) puede resolver, sin descartar el
/// evento.
///
/// Dos causas transitorias:
/// - `SQLITE_BUSY` / `SQLITE_LOCKED`: otro escritor tenía el lock de la
///   base de datos cuando esta conexión intentó tomarlo. El driver de
///   SQLite reporta estos con los mensajes canónicos "database is locked"
///   / "database table is locked" (ver el `Display` de `SqliteError` en
///   sqlx) -- son el criterio robusto, independiente del código primario
///   vs. extendido.
/// - Violación de UNIQUE sobre `event_sequence_id`: dos escritores
///   derivaron la misma posición de secuencia. Con `BEGIN IMMEDIATE` esto
///   no debería ocurrir (los escritores se serializan), pero se trata como
///   transitorio de cinturón-y-tirantes: re-derivar el MAX y reinsertar lo
///   resuelve. Cualquier OTRA violación de UNIQUE (p. ej. el `id`) NO es
///   transitoria y NO se reintenta.
fn is_transient_write_conflict(error: &ConsentRepositoryError) -> bool {
    let ConsentRepositoryError::Database(sqlx::Error::Database(db)) = error else {
        return false;
    };

    let message = db.message().to_lowercase();
    // Lock ocupado: otro escritor tenía el lock de la BD / de la tabla.
    if message.contains("database is locked") || message.contains("database table is locked") {
        return true;
    }

    // Colisión de secuencia: mismo event_sequence_id derivado por dos
    // escritores -- transitorio, re-derivar y reinsertar lo arregla.
    db.is_unique_violation() && message.contains("event_sequence_id")
}

/// Entrada para [`ConsentRepository::record_action`] -- todo lo que la
/// Shell necesita para registrar UN evento de consentimiento: identidad
/// del dueño/máquina y la acción en sí (ver
/// [`crate::domain::consent_registry::ConsentActionInput`]).
#[derive(Debug, Clone)]
pub struct RecordConsentActionInput {
    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,
    /// Estado de cumplimiento vigente al momento del evento (Grupo V,
    /// subset -- nullable: no todo evento lo trae).
    pub compliance_status_id: Option<String>,
    pub action: ConsentAction,
    pub tos_version: Option<String>,
    pub optout_changes: BTreeMap<String, bool>,
}

/// Una fila de `consent_records` ya persistida, con el `optout_map` ya
/// parseado de vuelta a `BTreeMap` (nunca se expone el JSON crudo fuera de
/// este módulo).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsentRecordRow {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub event_sequence_id: i64,

    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,
    pub compliance_status_id: Option<String>,

    pub tos_version: String,
    pub consent_action: ConsentAction,
    pub optout_map: BTreeMap<String, bool>,
    pub accepted_at_ns: i64,
}

/// Repositorio APPEND-ONLY para `consent_records`.
///
/// Constrúyelo con un [`SqlitePool`] ya migrado y cualquier implementación
/// de [`Clock`] -- mismo patrón que
/// [`crate::persistence::usage_metering::UsageRepository`].
pub struct ConsentRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> ConsentRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`. Ambos se toman
    /// prestados por la vida del repositorio -- no se toma ownership.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Carga el estado VIGENTE de un usuario: la fila con
    /// `event_sequence_id` MÁXIMO para su `owner_id` (`docs/features/
    /// consent-registry.md` "EL punto de modelado crítico"). Devuelve
    /// `None` si el usuario no tiene ningún evento registrado -- la Shell
    /// y el Core tratan eso como "sin consentimiento" (default: negar).
    pub async fn load_latest_for_owner(
        &self,
        owner_id: &str,
    ) -> Result<Option<ConsentRecordRow>, ConsentRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, node_id, compliance_status_id, \
                    tos_version, consent_action, optout_map, accepted_at \
             FROM consent_records \
             WHERE owner_id = ? \
             ORDER BY event_sequence_id DESC \
             LIMIT 1",
        )
        .bind(owner_id)
        .fetch_optional(self.pool)
        .await?;

        row.map(row_to_consent_record).transpose()
    }

    /// Registra UN evento de consentimiento: fusiona el estado vigente del
    /// dueño (o ninguno, si es su primer evento) con la acción entrante
    /// ([`apply_consent_action`]) y persiste el snapshot COMPLETO
    /// resultante como fila-evento nueva, encadenada por hash a la fila
    /// anterior de la secuencia GLOBAL.
    ///
    /// Es la ÚNICA forma de escribir en `consent_records` -- no existe
    /// `update`/`delete` en esta API (ver doc-comment del módulo).
    ///
    /// ## Atomicidad bajo concurrencia (regla "Atomicidad de ledgers append-only")
    ///
    /// Todo el *read-then-write* (leer el estado previo del dueño, leer el
    /// MAX(`event_sequence_id`) y el `audit_hash` previo para encadenar, y
    /// el `INSERT` final) ocurre dentro de UNA sola transacción
    /// `BEGIN IMMEDIATE` -- ver [`Self::try_record_action_once`]. Sin esa
    /// transacción, dos escritores concurrentes derivarían el mismo
    /// `event_sequence_id`, el `UNIQUE` rechazaría a uno y su evento se
    /// PERDERÍA. Ante contención transitoria (`SQLITE_BUSY` tras expirar el
    /// `busy_timeout`, o colisión de secuencia), se reintenta hasta
    /// [`MAX_RECORD_ATTEMPTS`] veces re-derivando la secuencia; el evento
    /// NUNCA se descarta en silencio (si se agotan los reintentos se
    /// devuelve [`ConsentRepositoryError::WriteContention`]).
    pub async fn record_action(
        &self,
        input: RecordConsentActionInput,
    ) -> Result<ConsentRecordRow, ConsentRepositoryError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_record_action_once(&input).await {
                Ok(row) => return Ok(row),
                Err(error) => {
                    // Solo se reintenta ante contención de escritura
                    // transitoria; cualquier otro error (input inválido,
                    // JSON corrupto, etc.) se propaga de inmediato.
                    if is_transient_write_conflict(&error) {
                        if attempt < MAX_RECORD_ATTEMPTS {
                            continue;
                        }
                        // Agotados los reintentos: error tipado, NUNCA
                        // pérdida silenciosa del evento.
                        return Err(ConsentRepositoryError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    /// Un intento único del append, dentro de una transacción
    /// `BEGIN IMMEDIATE`. Devuelve el error de SQLite tal cual si algo falla
    /// -- el bucle de [`Self::record_action`] decide si es transitorio y
    /// hay que reintentar. La transacción se abre con `BEGIN IMMEDIATE`
    /// (no el `BEGIN` DEFERRED por defecto de SQLx) para tomar el lock de
    /// escritura de ENTRADA: así ningún otro escritor puede intercalar
    /// entre la lectura del MAX(`event_sequence_id`) y el `INSERT`, y se
    /// evita además el interbloqueo de upgrade que ocurriría si dos
    /// transacciones DEFERRED intentaran subir de lectura a escritura a la
    /// vez.
    async fn try_record_action_once(
        &self,
        input: &RecordConsentActionInput,
    ) -> Result<ConsentRecordRow, ConsentRepositoryError> {
        // Abre la transacción tomando el lock de escritura de inmediato.
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        // Lectura 1 (DENTRO de la transacción) -- estado vigente previo de
        // este dueño (o None si es su primer evento). Al leerse dentro del
        // lock de escritura, dos OPTOUT_CHANGE concurrentes del mismo dueño
        // se serializan: el segundo ve el snapshot que dejó el primero, sin
        // perder su cambio.
        let previous_owner_row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, node_id, compliance_status_id, \
                    tos_version, consent_action, optout_map, accepted_at \
             FROM consent_records \
             WHERE owner_id = ? \
             ORDER BY event_sequence_id DESC \
             LIMIT 1",
        )
        .bind(&input.owner_id)
        .fetch_optional(&mut *tx)
        .await?;
        let previous_owner_row = previous_owner_row.map(row_to_consent_record).transpose()?;
        let previous_state = previous_owner_row.as_ref().map(|row| ConsentState {
            accepted_version: row.tos_version.clone(),
            optout_map: row.optout_map.clone(),
        });

        // DEBT-007 -- guarda explícita: OPTOUT_CHANGE como PRIMERA acción de
        // este owner_id (sin ACCEPT/estado previo) es una secuencia
        // inválida. Se rechaza aquí, ANTES de fusionar nada con
        // apply_consent_action, para no depender del efecto colateral
        // accepted_version="" -> StaleVersion (que no explica la causa
        // real: falta un ACCEPT).
        if input.action == ConsentAction::OptoutChange && previous_state.is_none() {
            return Err(ConsentRepositoryError::OptoutBeforeAccept {
                owner_id: input.owner_id.clone(),
            });
        }

        // Núcleo puro: fusiona el estado previo con la acción entrante.
        let action_input = ConsentActionInput {
            action: input.action,
            tos_version: input.tos_version.clone(),
            optout_changes: input.optout_changes.clone(),
        };
        let next_state = apply_consent_action(previous_state.as_ref(), &action_input);

        // JSON canónico: BTreeMap serializa SIEMPRE en orden alfabético de
        // claves -- el mismo estado lógico produce el mismo string en
        // cualquier ejecución (ver la nota de determinismo en
        // domain::consent_registry sobre BTreeMap vs HashMap).
        let optout_map_json = serde_json::to_string(&next_state.optout_map)?;

        // Lectura 2 (DENTRO de la transacción) -- posición en la cadena
        // GLOBAL: la fila con el event_sequence_id más alto de TODA la
        // tabla, para asignar la siguiente y encadenar su audit_hash.
        let tail_row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, node_id, compliance_status_id, \
                    tos_version, consent_action, optout_map, accepted_at \
             FROM consent_records \
             ORDER BY event_sequence_id DESC \
             LIMIT 1",
        )
        .fetch_optional(&mut *tx)
        .await?;
        let previous_tail = tail_row.map(row_to_consent_record).transpose()?;
        let (event_sequence_id, audit_chain_hash, previous_audit_hash) = match &previous_tail {
            Some(prev) => (prev.event_sequence_id + 1, Some(prev.audit_hash.clone()), prev.audit_hash.clone()),
            None => (1, None, GENESIS_PREVIOUS_HASH.to_string()),
        };

        let id = Uuid::now_v7().to_string();
        // Reloj INYECTADO -- nunca SystemTime::now() directo (ADR-0002/0004).
        // accepted_at (instante de dominio de la acción) y created_at
        // (instante de persistencia) usan la misma lectura del reloj: en
        // este repositorio ambos eventos ocurren en el mismo instante
        // lógico de la llamada.
        let now_ns = self.clock.timestamp_ns();

        let audit_hash = compute_consent_audit_hash(
            &id,
            now_ns,
            event_sequence_id,
            &previous_audit_hash,
            &input.owner_id,
            &input.institutional_tag,
            &input.node_id,
            &next_state.accepted_version,
            input.action,
            &optout_map_json,
            now_ns,
        );

        // Escritura (DENTRO de la transacción) -- el INSERT que cierra el
        // read-then-write atómico.
        sqlx::query(
            "INSERT INTO consent_records (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, compliance_status_id, \
                tos_version, consent_action, optout_map, accepted_at\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
        .bind(&input.compliance_status_id)
        .bind(&next_state.accepted_version)
        .bind(input.action.as_str())
        .bind(&optout_map_json)
        .bind(now_ns)
        .execute(&mut *tx)
        .await?;

        // Confirma la transacción: recién aquí el lock de escritura se
        // libera y la fila se hace visible a otros escritores.
        tx.commit().await?;

        Ok(ConsentRecordRow {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash,
            event_sequence_id,
            owner_id: input.owner_id.clone(),
            institutional_tag: input.institutional_tag.clone(),
            node_id: input.node_id.clone(),
            compliance_status_id: input.compliance_status_id.clone(),
            tos_version: next_state.accepted_version,
            consent_action: input.action,
            optout_map: next_state.optout_map,
            accepted_at_ns: now_ns,
        })
    }

    /// Carga la cadena completa, ordenada por `event_sequence_id`
    /// ascendente -- usada por los tests de integridad de la cadena
    /// (génesis con `audit_chain_hash = NULL`, resto encadenado) y por el
    /// test de snapshot event-sourced (verificar que la fila anterior
    /// queda intacta tras insertar una nueva).
    pub async fn load_chain(&self) -> Result<Vec<ConsentRecordRow>, ConsentRepositoryError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, node_id, compliance_status_id, \
                    tos_version, consent_action, optout_map, accepted_at \
             FROM consent_records \
             ORDER BY event_sequence_id ASC",
        )
        .fetch_all(self.pool)
        .await?;

        rows.into_iter().map(row_to_consent_record).collect()
    }
}

/// Convierte una fila de `consent_records` al tipo [`ConsentRecordRow`],
/// parseando `consent_action` y `optout_map` de vuelta a sus tipos Rust.
fn row_to_consent_record(row: sqlx::sqlite::SqliteRow) -> Result<ConsentRecordRow, ConsentRepositoryError> {
    let consent_action_value: String = row.get("consent_action");
    let consent_action = ConsentAction::from_str_value(&consent_action_value)
        .ok_or(ConsentRepositoryError::UnknownConsentAction(consent_action_value))?;

    let optout_map_json: String = row.get("optout_map");
    let optout_map = crate::domain::consent_registry::parse_optout_map(&optout_map_json)?;

    Ok(ConsentRecordRow {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        event_sequence_id: row.get("event_sequence_id"),
        owner_id: row.get("owner_id"),
        institutional_tag: row.get("institutional_tag"),
        node_id: row.get("node_id"),
        compliance_status_id: row.get("compliance_status_id"),
        tos_version: row.get("tos_version"),
        consent_action,
        optout_map,
        accepted_at_ns: row.get("accepted_at"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::persistence::central_identity::test_support::seed_account;
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    fn accept_input(owner_id: &str, tos_version: &str, optout_changes: BTreeMap<String, bool>) -> RecordConsentActionInput {
        RecordConsentActionInput {
            owner_id: owner_id.to_string(),
            institutional_tag: "DRASUS_LOCAL".to_string(),
            node_id: "node-1".to_string(),
            compliance_status_id: None,
            action: ConsentAction::Accept,
            tos_version: Some(tos_version.to_string()),
            optout_changes,
        }
    }

    // ── CRITERIO #1 (Orden §5): esquema STRICT append-only + Grupo I ────────

    #[tokio::test]
    async fn migration_creates_consent_records_table_with_group_i_and_event_sequence_id() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('consent_records')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();

        for expected in [
            "id", "created_at", "updated_at", "audit_hash", "audit_chain_hash", "event_sequence_id",
            "owner_id", "institutional_tag", "node_id", "compliance_status_id",
            "tos_version", "consent_action", "optout_map", "accepted_at",
        ] {
            assert!(column_names.contains(&expected.to_string()), "falta la columna: {expected}");
        }

        assert!(
            !column_names.contains(&"row_version".to_string()),
            "consent_records es APPEND-ONLY (ADR-0141): no debe tener row_version, solo event_sequence_id"
        );

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'consent_records'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(sql.contains("STRICT"), "la tabla consent_records debe declararse STRICT");
    }

    // ── CRITERIO #1 (Orden §5): append-only -- UPDATE/DELETE rechazados ─────

    /// CRITERIO DE CIERRE: un `UPDATE` sobre `consent_records` es
    /// rechazado por el trigger de la migración -- si el trigger no
    /// existiera (o la tabla permitiera mutar), esta prueba fallaría con
    /// `Ok`.
    #[tokio::test]
    async fn update_is_rejected_by_trigger() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let repo = ConsentRepository::new(&pool, &clock);

        let mut changes = BTreeMap::new();
        changes.insert("aggregation".to_string(), false);
        let row = repo
            .record_action(accept_input(&owner_id, "v2", changes))
            .await
            .expect("registrar aceptación");

        let result = sqlx::query("UPDATE consent_records SET tos_version = 'v99' WHERE id = ?")
            .bind(&row.id)
            .execute(&pool)
            .await;

        assert!(result.is_err(), "UPDATE sobre consent_records debe ser rechazado por el trigger");
    }

    /// CRITERIO DE CIERRE: un `DELETE` sobre `consent_records` es
    /// rechazado por el trigger de la migración.
    #[tokio::test]
    async fn delete_is_rejected_by_trigger() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let repo = ConsentRepository::new(&pool, &clock);

        let mut changes = BTreeMap::new();
        changes.insert("aggregation".to_string(), false);
        let row = repo
            .record_action(accept_input(&owner_id, "v2", changes))
            .await
            .expect("registrar aceptación");

        let result = sqlx::query("DELETE FROM consent_records WHERE id = ?")
            .bind(&row.id)
            .execute(&pool)
            .await;

        assert!(result.is_err(), "DELETE sobre consent_records debe ser rechazado por el trigger");
    }

    // ── CRITERIO #5 (Orden §5): event_sequence_id monótono y UNIQUE ─────────

    /// CRITERIO DE CIERRE: inserciones consecutivas asignan
    /// `event_sequence_id` 1, 2, 3... -- si la asignación no fuera
    /// monótona, esta prueba vería posiciones repetidas o desordenadas.
    #[tokio::test]
    async fn event_sequence_id_is_monotonic_across_inserts() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_1 = seed_account(&pool, &clock, "owner1@example.com").await;
        let owner_2 = seed_account(&pool, &clock, "owner2@example.com").await;
        let repo = ConsentRepository::new(&pool, &clock);

        let first = repo.record_action(accept_input(&owner_1, "v2", BTreeMap::new())).await.expect("primera acción");
        clock.tick();
        let second = repo.record_action(accept_input(&owner_2, "v2", BTreeMap::new())).await.expect("segunda acción");
        clock.tick();
        let third = repo.record_action(accept_input(&owner_1, "v2", BTreeMap::new())).await.expect("tercera acción");

        assert_eq!(first.event_sequence_id, 1);
        assert_eq!(second.event_sequence_id, 2);
        assert_eq!(third.event_sequence_id, 3);
    }

    /// CRITERIO DE CIERRE: duplicar una posición ya usada es rechazado por
    /// el `UNIQUE` de la migración -- se inserta directamente con SQL
    /// crudo para ejercitar el guardarraíl de la BD en sí mismo.
    #[tokio::test]
    async fn duplicating_event_sequence_id_is_rejected_by_unique_constraint() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let repo = ConsentRepository::new(&pool, &clock);

        repo.record_action(accept_input(&owner_id, "v2", BTreeMap::new()))
            .await
            .expect("primera acción (event_sequence_id = 1)");

        let duplicate = sqlx::query(
            "INSERT INTO consent_records (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, compliance_status_id, \
                tos_version, consent_action, optout_map, accepted_at\
            ) VALUES ('id-dup', 0, 0, 'hash', NULL, 1, ?, 'DRASUS_LOCAL', 'node-1', NULL, \
                       'v2', 'ACCEPT', '{}', 0)",
        )
        .bind(&owner_id)
        .execute(&pool)
        .await;

        assert!(duplicate.is_err(), "duplicar event_sequence_id=1 debe ser rechazado por UNIQUE");
    }

    // ── CRITERIO #3 (Orden §5): snapshot event-sourced ───────────────────────

    /// CRITERIO DE CIERRE: cambiar un opt-out inserta una fila NUEVA
    /// (`event_sequence_id` incrementado); la fila anterior queda intacta
    /// en la tabla; la resolución del estado vigente lee la ÚLTIMA fila.
    /// Debe fallar si el cambio mutara la fila anterior o si
    /// `load_latest_for_owner` leyera una fila vieja.
    #[tokio::test]
    async fn changing_an_optout_inserts_a_new_row_and_keeps_the_previous_one_intact() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let repo = ConsentRepository::new(&pool, &clock);

        let mut initial_optouts = BTreeMap::new();
        initial_optouts.insert("aggregation".to_string(), false);
        let first = repo
            .record_action(accept_input(&owner_id, "v2", initial_optouts))
            .await
            .expect("aceptación inicial");

        clock.tick();
        let mut changes = BTreeMap::new();
        changes.insert("aggregation".to_string(), true);
        let second = repo
            .record_action(RecordConsentActionInput {
                owner_id: owner_id.clone(),
                institutional_tag: "DRASUS_LOCAL".to_string(),
                node_id: "node-1".to_string(),
                compliance_status_id: None,
                action: ConsentAction::OptoutChange,
                tos_version: None,
                optout_changes: changes,
            })
            .await
            .expect("cambio de opt-out");

        assert_eq!(second.event_sequence_id, first.event_sequence_id + 1, "el cambio debe insertar una fila NUEVA");
        assert_ne!(second.id, first.id, "no debe reusar el id de la fila anterior");

        // La fila anterior sigue en la tabla, con su contenido original intacto.
        let chain = repo.load_chain().await.expect("cargar cadena completa");
        assert_eq!(chain.len(), 2, "ambas filas deben seguir presentes -- append-only");
        assert_eq!(chain[0].optout_map.get("aggregation"), Some(&false), "la fila original no debe mutar");
        assert_eq!(chain[1].optout_map.get("aggregation"), Some(&true), "la fila nueva trae el cambio");

        // El estado vigente debe reflejar la ÚLTIMA fila, no la primera.
        let latest = repo.load_latest_for_owner(&owner_id).await.expect("cargar vigente").expect("debe existir");
        assert_eq!(latest.id, second.id, "el estado vigente debe ser la fila con event_sequence_id máximo");
        assert_eq!(latest.optout_map.get("aggregation"), Some(&true));
    }

    // ── CRITERIO #7 (Orden §5): audit_chain_hash encadenado, NULL solo génesis ──

    /// CRITERIO DE CIERRE: la primera fila (génesis) tiene
    /// `audit_chain_hash = NULL`; las siguientes encadenan al `audit_hash`
    /// de la fila anterior -- si la cadena se rompiera, el segundo
    /// `audit_chain_hash` no coincidiría con el primer `audit_hash`.
    #[tokio::test]
    async fn audit_chain_hash_is_null_only_in_genesis_row_and_chains_afterwards() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_1 = seed_account(&pool, &clock, "owner1@example.com").await;
        let owner_2 = seed_account(&pool, &clock, "owner2@example.com").await;
        let repo = ConsentRepository::new(&pool, &clock);

        let first = repo.record_action(accept_input(&owner_1, "v2", BTreeMap::new())).await.expect("primera (génesis)");
        clock.tick();
        let second = repo.record_action(accept_input(&owner_2, "v2", BTreeMap::new())).await.expect("segunda");
        clock.tick();
        let third = repo.record_action(accept_input(&owner_1, "v2", BTreeMap::new())).await.expect("tercera");

        assert_eq!(first.audit_chain_hash, None, "la fila génesis debe tener audit_chain_hash NULL");
        assert_eq!(second.audit_chain_hash, Some(first.audit_hash.clone()), "debe encadenar a la primera");
        assert_eq!(third.audit_chain_hash, Some(second.audit_hash.clone()), "debe encadenar a la segunda");
    }

    // ── CHECK de consent_action y json_valid en la BD ────────────────────────

    #[tokio::test]
    async fn database_check_rejects_unknown_consent_action() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;

        let result = sqlx::query(
            "INSERT INTO consent_records (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, compliance_status_id, \
                tos_version, consent_action, optout_map, accepted_at\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, ?, 'DRASUS_LOCAL', 'node-1', NULL, \
                       'v2', 'UNKNOWN_ACTION', '{}', 0)",
        )
        .bind(&owner_id)
        .execute(&pool)
        .await;

        assert!(result.is_err(), "un consent_action fuera de ('ACCEPT','REACCEPT','OPTOUT_CHANGE') debe ser rechazado por el CHECK de la BD");
    }

    /// CRITERIO DE CIERRE: `CHECK (json_valid(optout_map))` rechaza JSON
    /// corrupto -- si el CHECK no existiera, el INSERT tendría éxito con
    /// basura en la columna.
    #[tokio::test]
    async fn database_check_rejects_invalid_json_in_optout_map() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;

        let result = sqlx::query(
            "INSERT INTO consent_records (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, compliance_status_id, \
                tos_version, consent_action, optout_map, accepted_at\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, ?, 'DRASUS_LOCAL', 'node-1', NULL, \
                       'v2', 'ACCEPT', '{not valid json', 0)",
        )
        .bind(&owner_id)
        .execute(&pool)
        .await;

        assert!(result.is_err(), "optout_map con JSON corrupto debe ser rechazado por el CHECK(json_valid) de la BD");
    }

    // ── CRITERIO DE CIERRE (ADR-0141 enmienda 2026-07-11, M6) ────────────────

    /// La FK física `consent_records.owner_id -> accounts(id)` rechaza un
    /// `owner_id` que no corresponde a ninguna cuenta -- un huérfano ya no
    /// es un bug silencioso, la base de datos lo atrapa.
    #[tokio::test]
    async fn record_action_with_nonexistent_owner_id_is_rejected_by_foreign_key() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = ConsentRepository::new(&pool, &clock);

        let result = repo
            .record_action(accept_input("cuenta-que-no-existe", "v2", BTreeMap::new()))
            .await;

        assert!(
            matches!(result, Err(ConsentRepositoryError::Database(_))),
            "un owner_id huérfano debe rechazarse por la FK, no persistirse: {result:?}"
        );
    }

    // ── Estado vigente ausente ────────────────────────────────────────────────

    /// Sin ningún evento registrado para un `owner_id`, el estado vigente
    /// es `None` -- consistente con el default "negar" del Core.
    #[tokio::test]
    async fn load_latest_for_owner_returns_none_without_any_event() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = ConsentRepository::new(&pool, &clock);

        let latest = repo.load_latest_for_owner("owner-sin-eventos").await.expect("consulta debe tener éxito");
        assert!(latest.is_none());
    }

    // ── DEBT-007: OPTOUT_CHANGE como primera acción de un owner_id ──────────

    /// CRITERIO DE CIERRE (DEBT-007): un `OPTOUT_CHANGE` como PRIMER evento
    /// registrado para un `owner_id` (sin `ACCEPT` previo) debe rechazarse
    /// con el error tipado [`ConsentRepositoryError::OptoutBeforeAccept`] --
    /// NO debe caer al efecto colateral `accepted_version=""` ->
    /// `StaleVersion`. Debe caerse (falso positivo de "pasa") si la guarda
    /// se quitara: en ese caso `record_action` tendría éxito con
    /// `accepted_version=""`, y este `assert!(matches!(...))` fallaría
    /// porque el resultado sería `Ok`.
    #[tokio::test]
    async fn optout_change_as_first_action_for_owner_is_rejected_with_typed_error() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = ConsentRepository::new(&pool, &clock);

        let mut changes = BTreeMap::new();
        changes.insert("aggregation".to_string(), true);
        let result = repo
            .record_action(RecordConsentActionInput {
                owner_id: "owner-sin-accept".to_string(),
                institutional_tag: "DRASUS_LOCAL".to_string(),
                node_id: "node-1".to_string(),
                compliance_status_id: None,
                action: ConsentAction::OptoutChange,
                tos_version: None,
                optout_changes: changes,
            })
            .await;

        assert!(
            matches!(
                result,
                Err(ConsentRepositoryError::OptoutBeforeAccept { owner_id }) if owner_id == "owner-sin-accept"
            ),
            "OPTOUT_CHANGE sin ACCEPT previo debe rechazarse con OptoutBeforeAccept, no persistir con StaleVersion"
        );

        // Ninguna fila debe haberse persistido: el rechazo ocurre ANTES del
        // INSERT (la transacción se abre pero jamás se confirma).
        let chain = repo.load_chain().await.expect("cargar cadena completa");
        assert!(chain.is_empty(), "la secuencia inválida no debe dejar ninguna fila persistida");
    }

    /// Confirma que el camino EXISTENTE (OPTOUT_CHANGE tras un ACCEPT
    /// previo) sigue funcionando sin cambios -- no regresiona la guarda de
    /// DEBT-007 sobre el caso válido.
    #[tokio::test]
    async fn optout_change_after_accept_still_succeeds() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let repo = ConsentRepository::new(&pool, &clock);

        let mut initial = BTreeMap::new();
        initial.insert("aggregation".to_string(), false);
        repo.record_action(accept_input(&owner_id, "v2", initial))
            .await
            .expect("aceptación inicial debe tener éxito");

        clock.tick();
        let mut changes = BTreeMap::new();
        changes.insert("aggregation".to_string(), true);
        let result = repo
            .record_action(RecordConsentActionInput {
                owner_id: owner_id.clone(),
                institutional_tag: "DRASUS_LOCAL".to_string(),
                node_id: "node-1".to_string(),
                compliance_status_id: None,
                action: ConsentAction::OptoutChange,
                tos_version: None,
                optout_changes: changes,
            })
            .await;

        assert!(result.is_ok(), "OPTOUT_CHANGE tras un ACCEPT previo debe seguir funcionando");
    }

    // ── Atomicidad bajo concurrencia (regla "Atomicidad de ledgers append-only") ──

    /// CRITERIO DE CIERRE (DEBT-001): N escritores concurrentes sobre el
    /// MISMO pool/ledger. La transacción `BEGIN IMMEDIATE` + reintento
    /// acotado debe garantizar que NINGÚN evento se pierde y que la
    /// secuencia queda densa (1..=N sin huecos ni duplicados) con la cadena
    /// de hashes íntegra.
    ///
    /// Esta prueba DEBE poder caerse si se quita la transacción: con el
    /// `SELECT MAX(...)` y el `INSERT` en sentencias sueltas, dos tareas
    /// leen el mismo MAX, derivan el mismo `event_sequence_id`, el `UNIQUE`
    /// rechaza a una y su fila se pierde -> la aserción (a) `chain.len()==N`
    /// o la (b) `1..=N` fallaría. Se usa una BD en ARCHIVO temporal (nunca
    /// `:memory:`, donde cada conexión sería una base distinta) para que la
    /// concurrencia entre conexiones sea real.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_record_actions_persist_every_event_without_gaps_or_lost_rows() {
        use std::sync::Arc;

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("consent_concurrency.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());
        let pool = connect(&database_url).await.expect("conectar");
        migrate(&pool).await.expect("migrar");

        // Reloj compartido (atómico, thread-safe). No se hace `tick`: todas
        // las filas comparten timestamp, lo cual es válido -- el orden lo
        // fija `event_sequence_id`, no el reloj.
        let clock: Arc<DeterministicClock> = Arc::new(DeterministicClock::new(1_000, 100));
        let owner_id = seed_account(&pool, clock.as_ref(), "owner-concurrente@example.com").await;

        const N: i64 = 16;

        // Lanza N tareas en paralelo, cada una registrando un ACCEPT con una
        // clave de opt-out DISTINTA sobre el mismo dueño.
        let mut handles = Vec::new();
        for i in 0..N {
            let pool_c = pool.clone(); // SqlitePool es un Arc interno: clonar es barato.
            let clock_c = clock.clone();
            let owner = owner_id.clone();
            handles.push(tokio::spawn(async move {
                let repo = ConsentRepository::new(&pool_c, clock_c.as_ref());
                let mut changes = BTreeMap::new();
                changes.insert(format!("type_{i}"), true);
                repo.record_action(RecordConsentActionInput {
                    owner_id: owner,
                    institutional_tag: "DRASUS_LOCAL".to_string(),
                    node_id: "node-1".to_string(),
                    compliance_status_id: None,
                    action: ConsentAction::Accept,
                    tos_version: Some("v2".to_string()),
                    optout_changes: changes,
                })
                .await
            }));
        }

        // (a) TODAS las tareas terminaron OK -- ningún evento se perdió por
        // colisión de secuencia (una tarea que perdiera la carrera y no
        // reintentara devolvería Err aquí).
        for handle in handles {
            handle
                .await
                .expect("la tarea no debe entrar en panic")
                .expect("record_action debe tener éxito para cada escritor concurrente");
        }

        let repo = ConsentRepository::new(&pool, clock.as_ref());
        let chain = repo.load_chain().await.expect("cargar la cadena completa");

        // (a) se persistieron TODAS las N filas.
        assert_eq!(chain.len() as i64, N, "deben persistirse las N filas, sin ninguna pérdida");

        // (b) los event_sequence_id son exactamente 1..=N (densa, sin huecos
        // ni duplicados). `load_chain` ya ordena ascendente por la columna.
        let sequence_ids: Vec<i64> = chain.iter().map(|row| row.event_sequence_id).collect();
        let expected: Vec<i64> = (1..=N).collect();
        assert_eq!(sequence_ids, expected, "los event_sequence_id deben ser 1..=N sin huecos ni duplicados");

        // (c) la cadena audit_chain_hash queda íntegra: génesis con NULL,
        // cada fila encadenada al audit_hash de la anterior, y cada
        // audit_hash recomputable (integridad de contenido completa).
        for (index, row) in chain.iter().enumerate() {
            let previous_audit_hash = if index == 0 {
                assert_eq!(row.audit_chain_hash, None, "la fila génesis debe tener audit_chain_hash NULL");
                GENESIS_PREVIOUS_HASH.to_string()
            } else {
                let prev = &chain[index - 1];
                assert_eq!(
                    row.audit_chain_hash.as_deref(),
                    Some(prev.audit_hash.as_str()),
                    "cada fila debe encadenar al audit_hash de la anterior"
                );
                prev.audit_hash.clone()
            };

            let optout_json = serde_json::to_string(&row.optout_map).expect("serializar optout_map");
            let recomputed = compute_consent_audit_hash(
                &row.id,
                row.created_at_ns,
                row.event_sequence_id,
                &previous_audit_hash,
                &row.owner_id,
                &row.institutional_tag,
                &row.node_id,
                &row.tos_version,
                row.consent_action,
                &optout_json,
                row.accepted_at_ns,
            );
            assert_eq!(recomputed, row.audit_hash, "el audit_hash de cada fila debe ser recomputable (integridad de la cadena)");
        }

        // (bonus) el snapshot vigente acumuló las N claves de opt-out: prueba
        // que los eventos se serializaron y cada uno fusionó su cambio sobre
        // el snapshot previo (si dos hubieran leído el mismo estado base
        // fuera de la transacción, se perdería alguna clave).
        let latest = repo
            .load_latest_for_owner(&owner_id)
            .await
            .expect("cargar vigente")
            .expect("el dueño concurrente debe tener estado vigente");
        assert_eq!(latest.optout_map.len() as i64, N, "el snapshot final debe acumular las N claves de opt-out");
        for i in 0..N {
            assert_eq!(
                latest.optout_map.get(&format!("type_{i}")),
                Some(&true),
                "cada clave aportada por un escritor concurrente debe estar en el snapshot final"
            );
        }
    }

    // ── CRITERIO (QA por mutación, DEBT-018): reintento acotado hasta AGOTAR ──

    /// CRITERIO DE CIERRE (QA por mutación): bajo contención de escritura
    /// SOSTENIDA (otro escritor retiene el lock de `BEGIN IMMEDIATE` y no lo
    /// suelta), el bucle de reintento debe agotar EXACTAMENTE
    /// `MAX_RECORD_ATTEMPTS` intentos y rendirse con
    /// `WriteContention { attempts: MAX }` -- nunca descartar el evento en
    /// silencio, ni rendirse un intento antes o después. Patrón de
    /// referencia: `persistence/data_portability.rs` (STORY-043, DEBT-018).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn record_action_exhausts_exactly_max_attempts_when_write_lock_is_held() {
        use std::str::FromStr;
        use std::time::Duration;

        use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("consent_registry_forced_contention.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());

        // Migrar con el pool normal (busy_timeout de 5s).
        let pool = connect(&database_url).await.expect("conectar");
        migrate(&pool).await.expect("migrar");

        // Sembrar la cuenta ANTES de que el escritor A tome el lock -- la FK
        // owner_id->accounts(id) exige que la cuenta ya exista; sembrarla
        // aquí evita interferir con el escenario de contención de abajo.
        let seed_clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &seed_clock, "owner-contention@example.com").await;

        // Opciones con busy_timeout=0: un lock ocupado falla de INMEDIATO con
        // "database is locked" en vez de esperar 5s -- hace la contención
        // determinista y rápida.
        let immediate_opts = || {
            SqliteConnectOptions::from_str(&database_url)
                .expect("parsear opciones")
                .journal_mode(SqliteJournalMode::Wal)
                .busy_timeout(Duration::from_millis(0))
        };

        // Escritor A: toma el lock de escritura con `BEGIN IMMEDIATE` y NO lo
        // suelta mientras B intenta escribir.
        let lock_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(immediate_opts())
            .await
            .expect("pool que retiene el lock");
        let lock_tx = lock_pool
            .begin_with("BEGIN IMMEDIATE")
            .await
            .expect("tomar el lock de escritura reservado");

        // Escritor B: intenta registrar una acción mientras A retiene el
        // lock. Cada `try_record_action_once` abre `BEGIN IMMEDIATE`, choca
        // con el lock de A, falla con "database is locked" (transitorio) y
        // reintenta, hasta agotar MAX_RECORD_ATTEMPTS.
        let repo_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(immediate_opts())
            .await
            .expect("pool del repositorio");
        let clock = DeterministicClock::new(1_000, 100);
        let repo = ConsentRepository::new(&repo_pool, &clock);

        let result = repo.record_action(accept_input(&owner_id, "v2", BTreeMap::new())).await;

        drop(lock_tx); // libera el lock (limpieza; el resultado ya está tomado)

        match result {
            Err(ConsentRepositoryError::WriteContention { attempts }) => {
                assert_eq!(
                    attempts, MAX_RECORD_ATTEMPTS,
                    "bajo contención sostenida debe agotar EXACTAMENTE MAX_RECORD_ATTEMPTS intentos"
                );
            }
            other => panic!(
                "se esperaba WriteContention {{ attempts: {MAX_RECORD_ATTEMPTS} }} bajo contención sostenida, se obtuvo: {other:?}"
            ),
        }
    }

    // ── CRITERIO (QA por mutación, DEBT-018): clasificador de contención ──────

    /// CRITERIO DE CIERRE (QA por mutación): `is_transient_write_conflict`
    /// distingue una violación UNIQUE PERMANENTE (la PK `id`, que NO se debe
    /// reintentar) de la contención transitoria. Fija que exige AMBAS
    /// condiciones (es violación UNIQUE **y** menciona `event_sequence_id`),
    /// no una sola, y que no clasifica cualquier cosa como transitoria.
    #[tokio::test]
    async fn is_transient_is_false_for_a_permanent_non_sequence_unique_violation() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;

        // Inserta una fila válida y luego otra con el MISMO `id`: viola la
        // PRIMARY KEY `id`, NO el UNIQUE de `event_sequence_id`. Error UNIQUE
        // PERMANENTE cuyo mensaje NO menciona `event_sequence_id`.
        let insert_with_id_dup = |event_sequence_id: i64| {
            sqlx::query(
                "INSERT INTO consent_records (\
                    id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, node_id, compliance_status_id, \
                    tos_version, consent_action, optout_map, accepted_at\
                ) VALUES ('dup-id', 0, 0, 'hash', NULL, ?, ?, 'DRASUS_LOCAL', 'node-1', NULL, \
                           'v2', 'ACCEPT', '{}', 0)",
            )
            .bind(event_sequence_id)
            .bind(&owner_id)
            .execute(&pool)
        };
        insert_with_id_dup(1).await.expect("primera fila válida");
        let err = insert_with_id_dup(2)
            .await
            .expect_err("la segunda fila debe violar la PRIMARY KEY id");

        let permanent = ConsentRepositoryError::Database(err);
        assert!(
            !is_transient_write_conflict(&permanent),
            "una violación UNIQUE de la PK (no de event_sequence_id) es PERMANENTE, no transitoria"
        );

        // Control: un error que ni siquiera es de base de datos jamás es
        // transitorio (fija la rama temprana `let ... else`).
        let non_database = ConsentRepositoryError::UnknownConsentAction("X".to_string());
        assert!(
            !is_transient_write_conflict(&non_database),
            "un error no-Database nunca es contención transitoria"
        );
    }

    // ── CRITERIO (QA por mutación, DEBT-018): fidelidad de la fila devuelta ───

    /// CRITERIO DE CIERRE (QA por mutación): la fila que DEVUELVE
    /// `record_action` es bit-a-bit idéntica a la fila persistida en disco
    /// -- si el literal de retorno de `try_record_action_once` sustituyera
    /// algún campo (`audit_hash`, `event_sequence_id`, timestamps...) por un
    /// valor por defecto en vez del recién calculado, esta comparación de
    /// igualdad completa lo detectaría.
    #[tokio::test]
    async fn record_action_returned_row_matches_the_persisted_row_exactly() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        let repo = ConsentRepository::new(&pool, &clock);

        let first = repo.record_action(accept_input(&owner_id, "v2", BTreeMap::new())).await.expect("primera acción");
        clock.tick();
        let second = repo
            .record_action(accept_input(&owner_id, "v3", BTreeMap::new()))
            .await
            .expect("segunda acción");

        let chain = repo.load_chain().await.expect("cargar cadena");
        assert_eq!(
            chain.first(),
            Some(&first),
            "la primera fila devuelta debe ser idéntica a la persistida en disco"
        );
        assert_eq!(
            chain.get(1),
            Some(&second),
            "la segunda fila devuelta debe ser idéntica a la persistida en disco"
        );
        assert_ne!(
            second.audit_hash, first.audit_hash,
            "el audit_hash devuelto debe ser recomputado, no copiado del intento anterior"
        );
        assert_eq!(second.updated_at_ns, 1_100, "el updated_at devuelto debe reflejar el now del reloj tras el tick");
    }
}
