//! [SHELL] Repositorio de decisiones de permiso del Gateway MCP.
//!
//! Implementa la persistencia de la tabla `permission_decisions`
//! (append-only, forense) y de la tabla de configuración `mcp_gateway_config`
//! (estado mutable del interruptor de producción).
//!
//! ADR-0020, Perfil D (Ops/Auditoría): Grupo I + Grupo II + Grupo IV +
//! 4 campos de dominio propio. Migración: `0005_mcp_gateway.sql`.

use sqlx::{Row, SqlitePool};

use crate::domain::mcp_gateway::PermissionDecision;

// ────────────────────────────────────────────────────────────────────────────
// Tipo de error
// ────────────────────────────────────────────────────────────────────────────

/// Errores del repositorio de decisiones de permiso.
#[derive(Debug, thiserror::Error)]
pub enum McpGatewayError {
    /// Error de la capa SQLite (SQLx).
    #[error("SQLite: {0}")]
    Sqlx(#[from] sqlx::Error),
    /// [`McpGatewayRepository::record_decision`] no pudo completarse tras
    /// agotar los reintentos ante contención de escritura transitoria --
    /// la decisión NO se descartó en silencio (regla "Atomicidad de
    /// ledgers append-only", rust-engineer/SKILL.md §4).
    #[error("no se pudo registrar la decisión de permiso tras {attempts} intentos por contención de escritura")]
    WriteContention { attempts: u32 },
}

/// Número máximo de intentos de [`McpGatewayRepository::record_decision`]
/// ante contención de escritura transitoria antes de rendirse con
/// [`McpGatewayError::WriteContention`]. Mismo valor y misma justificación
/// que [`crate::persistence::audit_log::MAX_APPEND_ATTEMPTS`].
const MAX_APPEND_ATTEMPTS: u32 = 5;

/// Decide si un error de [`McpGatewayRepository::record_decision`] es una
/// contención de escritura TRANSITORIA -- mismo criterio que
/// `crate::persistence::audit_log::is_transient_write_conflict`.
fn is_transient_write_conflict(error: &McpGatewayError) -> bool {
    let McpGatewayError::Sqlx(sqlx::Error::Database(db)) = error else {
        return false;
    };

    let message = db.message().to_lowercase();
    if message.contains("database is locked") || message.contains("database table is locked") {
        return true;
    }

    db.is_unique_violation() && message.contains("event_sequence_id")
}

// ────────────────────────────────────────────────────────────────────────────
// Repositorio
// ────────────────────────────────────────────────────────────────────────────

/// Repositorio de decisiones de permiso y del interruptor de producción.
///
/// Todas las operaciones sobre `permission_decisions` son append-only:
/// no existe UPDATE ni DELETE sobre esa tabla. El propósito es forense:
/// una vez registrada, una decisión no puede borrarse ni modificarse.
///
/// `mcp_gateway_config` sí es mutable: el interruptor puede
/// activarse y desactivarse en cualquier momento por el propietario.
pub struct McpGatewayRepository;

impl McpGatewayRepository {
    // ── Decisiones de permiso (append-only) ──────────────────────────────

    /// Inserta una decisión de permiso en la tabla `permission_decisions`.
    ///
    /// La tabla es append-only (nunca UPDATE ni DELETE). El camino de
    /// producción con reintento atómico es `record_decision`; este `append`
    /// se conserva para setup de pruebas.
    pub async fn append(
        pool: &SqlitePool,
        d: &PermissionDecision,
    ) -> Result<(), McpGatewayError> {
        let production_override_i64 = if d.production_override_active { 1i64 } else { 0i64 };
        let id_str = d.id.to_string();

        sqlx::query(
            r#"
            INSERT INTO permission_decisions (
                id, created_at, updated_at,
                audit_hash, audit_chain_hash, event_sequence_id,
                owner_id, institutional_tag,
                node_id, process_id,
                agent_session_id, requested_scope,
                permission_outcome, production_override_active
            ) VALUES (
                ?, ?, ?,
                ?, ?, ?,
                ?, ?,
                ?, ?,
                ?, ?,
                ?, ?
            )
            "#,
        )
        .bind(&id_str)
        .bind(d.created_at)
        .bind(d.updated_at)
        .bind(&d.audit_hash)
        .bind(&d.audit_chain_hash)
        .bind(d.event_sequence_id)
        .bind(&d.owner_id)
        .bind(&d.institutional_tag)
        .bind(&d.node_id)
        .bind(d.process_id)
        .bind(&d.agent_session_id)
        .bind(&d.requested_scope)
        .bind(&d.permission_outcome)
        .bind(production_override_i64)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Registra una nueva decisión de permiso de forma ATÓMICA: deriva su
    /// posición en la cadena (extremo actual + 1) y hace el `INSERT` dentro
    /// de UNA sola transacción `BEGIN IMMEDIATE`, con reintento acotado ante
    /// contención transitoria.
    ///
    /// ## Por qué existe (fusiona `chain_tip` + `append`, STORY-046 M1)
    ///
    /// El camino viejo -- leer [`Self::chain_tip`] y luego llamar a
    /// [`Self::append`] en sentencias sueltas -- es un *read-then-write* sin
    /// transacción: dos llamadas concurrentes pueden leer el mismo extremo,
    /// derivar el mismo `event_sequence_id`, y el `UNIQUE` de la migración
    /// rechazaría a una, perdiendo esa decisión de permiso en silencio
    /// (regla "Atomicidad de ledgers append-only", causa raíz DEBT-001).
    /// Este método reemplaza ese camino para escrituras: lee el extremo Y
    /// escribe la fila nueva dentro de la MISMA transacción.
    ///
    /// `build_decision` recibe `(prev_hash, next_event_sequence_id)` leídos
    /// DENTRO de la transacción y devuelve la [`PermissionDecision`] ya
    /// calculada (típicamente delegando en
    /// [`crate::domain::mcp_gateway::PermissionDecision::build`]). Se llama
    /// una vez por intento -- puede invocarse más de una vez si hay
    /// reintentos, por eso es `Fn` y no `FnOnce`.
    pub async fn record_decision(
        pool: &SqlitePool,
        build_decision: impl Fn(Option<String>, i64) -> PermissionDecision,
    ) -> Result<PermissionDecision, McpGatewayError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match Self::try_record_decision_once(pool, &build_decision).await {
                Ok(decision) => return Ok(decision),
                Err(error) => {
                    // Solo se reintenta ante contención de escritura
                    // transitoria; cualquier otro error se propaga de
                    // inmediato.
                    if is_transient_write_conflict(&error) {
                        if attempt < MAX_APPEND_ATTEMPTS {
                            continue;
                        }
                        // Agotados los reintentos: error tipado, NUNCA
                        // pérdida silenciosa de la decisión.
                        return Err(McpGatewayError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    /// Un intento único de [`Self::record_decision`], dentro de una
    /// transacción `BEGIN IMMEDIATE` -- toma el lock de escritura de
    /// ENTRADA, evitando tanto la intercalación de otro escritor entre la
    /// lectura del extremo y el `INSERT` como el interbloqueo de upgrade de
    /// dos transacciones DEFERRED.
    async fn try_record_decision_once(
        pool: &SqlitePool,
        build_decision: &impl Fn(Option<String>, i64) -> PermissionDecision,
    ) -> Result<PermissionDecision, McpGatewayError> {
        let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

        // Lectura (DENTRO de la transacción) -- el extremo actual de la
        // cadena, para derivar el próximo event_sequence_id y el prev_hash.
        let tail_row = sqlx::query(
            "SELECT audit_hash, event_sequence_id FROM permission_decisions \
             ORDER BY event_sequence_id DESC LIMIT 1",
        )
        .fetch_optional(&mut *tx)
        .await?;

        let (prev_hash, next_sequence_id) = match tail_row {
            Some(row) => {
                let hash: String = row.get("audit_hash");
                let seq: i64 = row.get("event_sequence_id");
                (Some(hash), seq + 1)
            }
            None => (None, 1),
        };

        let decision = build_decision(prev_hash, next_sequence_id);
        let production_override_i64 = if decision.production_override_active { 1i64 } else { 0i64 };
        let id_str = decision.id.to_string();

        // Escritura (DENTRO de la transacción) -- el INSERT que cierra el
        // read-then-write atómico.
        sqlx::query(
            r#"
            INSERT INTO permission_decisions (
                id, created_at, updated_at,
                audit_hash, audit_chain_hash, event_sequence_id,
                owner_id, institutional_tag,
                node_id, process_id,
                agent_session_id, requested_scope,
                permission_outcome, production_override_active
            ) VALUES (
                ?, ?, ?,
                ?, ?, ?,
                ?, ?,
                ?, ?,
                ?, ?,
                ?, ?
            )
            "#,
        )
        .bind(&id_str)
        .bind(decision.created_at)
        .bind(decision.updated_at)
        .bind(&decision.audit_hash)
        .bind(&decision.audit_chain_hash)
        .bind(decision.event_sequence_id)
        .bind(&decision.owner_id)
        .bind(&decision.institutional_tag)
        .bind(&decision.node_id)
        .bind(decision.process_id)
        .bind(&decision.agent_session_id)
        .bind(&decision.requested_scope)
        .bind(&decision.permission_outcome)
        .bind(production_override_i64)
        .execute(&mut *tx)
        .await?;

        // Confirma la transacción: recién aquí el lock de escritura se
        // libera y la fila se hace visible a otros escritores.
        tx.commit().await?;

        Ok(decision)
    }

    /// Devuelve el `audit_hash` y el `event_sequence_id` de la última
    /// decisión de la cadena (el extremo donde encadenar la siguiente).
    /// Lectura informativa, sin garantías transaccionales -- para
    /// registrar una decisión nueva usa [`Self::record_decision`], que lee
    /// el mismo extremo DENTRO de la transacción de escritura.
    ///
    /// Devuelve `None` si la tabla está vacía.
    pub async fn chain_tip(
        pool: &SqlitePool,
    ) -> Result<Option<(String, i64)>, McpGatewayError> {
        let row = sqlx::query(
            r#"
            SELECT audit_hash, event_sequence_id
            FROM permission_decisions
            ORDER BY event_sequence_id DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| {
            let hash: String = r.get("audit_hash");
            let seq: i64 = r.get("event_sequence_id");
            (hash, seq)
        }))
    }

    // ── Interruptor de producción (estado mutable) ────────────────────────

    /// Lee el estado actual del interruptor de producción desde la BD.
    ///
    /// `true` = activo (el agente puede invocar `execute`/`withdraw`/`manage(Live)`).
    /// `false` = inactivo (comportamiento de fábrica, ADR-0123).
    pub async fn get_production_override(pool: &SqlitePool) -> Result<bool, McpGatewayError> {
        let row = sqlx::query(
            r#"
            SELECT value FROM mcp_gateway_config
            WHERE key = 'production_override_active'
            "#,
        )
        .fetch_one(pool)
        .await?;

        let value: String = row.get("value");
        Ok(value == "1")
    }

    /// Activa o desactiva el interruptor de producción.
    ///
    /// Solo el propietario del despliegue puede llamar a este método.
    /// La API del agente MCP NO expone esta operación como herramienta
    /// (ADR-0123: "El interruptor no puede activarse por una llamada del propio agente").
    pub async fn set_production_override(
        pool: &SqlitePool,
        active: bool,
    ) -> Result<(), McpGatewayError> {
        let value = if active { "1" } else { "0" };

        sqlx::query(
            r#"
            INSERT INTO mcp_gateway_config (key, value)
            VALUES ('production_override_active', ?)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
            "#,
        )
        .bind(value)
        .execute(pool)
        .await?;

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Pruebas de integración (Capa 2 — ADR-0133)
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;
    use crate::domain::mcp_gateway::{
        PermissionDecision, PermissionOutcome, PermissionRequest, Pipeline,
        evaluate_permission,
    };
    use crate::persistence::pool::{connect, migrate};

    /// Abre una BD en archivo temporal y aplica las migraciones.
    ///
    /// Usamos archivo temporal, no `:memory:`, porque la Capa 2 de la pirámide
    /// ADR-0133 exige verificar durabilidad/recuperación: la BD debe sobrevivir
    /// al cierre del pool. Una BD `:memory:` desaparece al cerrar el pool,
    /// haciendo inútil cualquier prueba de persistencia real.
    async fn setup_db() -> (SqlitePool, NamedTempFile) {
        let file = NamedTempFile::new().unwrap();
        let url = format!("sqlite://{}", file.path().display());
        let pool = connect(&url).await.unwrap();
        migrate(&pool).await.unwrap();
        (pool, file)
    }

    fn make_decision(
        req: &PermissionRequest,
        outcome: &PermissionOutcome,
        prev_hash: Option<&str>,
        seq: i64,
    ) -> PermissionDecision {
        PermissionDecision::build(
            req,
            outcome,
            1_700_000_000_000_000_000i64,
            prev_hash.map(str::to_string),
            seq,
            "node-test".to_string(),
            12345,
        )
    }

    // Criterio 9: una decisión persistida es recuperable vía chain_tip.
    #[tokio::test]
    async fn permission_decision_persists_and_is_retrievable() {
        let (pool, _file) = setup_db().await;

        let req = PermissionRequest {
            pipeline: Pipeline::Ingest,
            institutional_tag: None,
            production_override_active: false,
            agent_session_id: "session-abc".into(),
            requested_scope: "ingest.submit_bar".into(),
        };
        let outcome = evaluate_permission(&req);
        let decision = make_decision(&req, &outcome, None, 1);

        McpGatewayRepository::append(&pool, &decision).await.unwrap();

        let tip = McpGatewayRepository::chain_tip(&pool).await.unwrap();
        assert!(tip.is_some(), "chain_tip debe devolver Some tras insertar");
        let (hash, seq) = tip.unwrap();
        assert_eq!(seq, 1);
        // El hash recuperado coincide con el que se insertó.
        assert_eq!(hash, decision.audit_hash);
    }

    // Criterio 10: la cadena de auditoría encadena decisiones secuencialmente.
    #[tokio::test]
    async fn audit_chain_links_sequential_decisions() {
        let (pool, _file) = setup_db().await;

        let req = PermissionRequest {
            pipeline: Pipeline::Ingest,
            institutional_tag: None,
            production_override_active: false,
            agent_session_id: "session-chain".into(),
            requested_scope: "ingest.submit_bar".into(),
        };
        let outcome = evaluate_permission(&req);

        // Primera decisión: prev_hash = None (génesis), seq = 1.
        let d1 = make_decision(&req, &outcome, None, 1);
        let hash_d1 = d1.audit_hash.clone();
        McpGatewayRepository::append(&pool, &d1).await.unwrap();

        // Segunda decisión: prev_hash = audit_hash de d1, seq = 2.
        let d2 = make_decision(&req, &outcome, Some(&hash_d1), 2);
        // El audit_chain_hash de d2 apunta al audit_hash de d1.
        assert_eq!(d2.audit_chain_hash, Some(hash_d1));
        McpGatewayRepository::append(&pool, &d2).await.unwrap();

        // chain_tip devuelve la última (d2).
        let tip = McpGatewayRepository::chain_tip(&pool).await.unwrap().unwrap();
        assert_eq!(tip.1, 2, "sequence_id debe ser 2");
        assert_eq!(tip.0, d2.audit_hash, "hash del tip debe ser el de d2");
    }

    /// CRITERIO DE CIERRE (hallazgo M4): `permission_decisions` rechaza
    /// `UPDATE` a nivel de base de datos -- hasta ahora solo la disciplina
    /// del repositorio (sin método `update`) lo protegía; esta prueba
    /// ataca la tabla directo por SQL, sin pasar por el repositorio.
    #[tokio::test]
    async fn update_on_permission_decisions_is_rejected_by_append_only_trigger() {
        let (pool, _file) = setup_db().await;
        let req = sample_req("session-trigger-update");
        let outcome = evaluate_permission(&req);
        let decision = make_decision(&req, &outcome, None, 1);
        McpGatewayRepository::append(&pool, &decision).await.unwrap();

        let result = sqlx::query("UPDATE permission_decisions SET permission_outcome = ? WHERE id = ?")
            .bind("tampered")
            .bind(decision.id.to_string())
            .execute(&pool)
            .await;

        assert!(result.is_err(), "UPDATE sobre permission_decisions debe ser rechazado por el trigger append-only");
    }

    /// CRITERIO DE CIERRE (hallazgo M4): `permission_decisions` rechaza
    /// `DELETE` a nivel de base de datos.
    #[tokio::test]
    async fn delete_on_permission_decisions_is_rejected_by_append_only_trigger() {
        let (pool, _file) = setup_db().await;
        let req = sample_req("session-trigger-delete");
        let outcome = evaluate_permission(&req);
        let decision = make_decision(&req, &outcome, None, 1);
        McpGatewayRepository::append(&pool, &decision).await.unwrap();

        let result = sqlx::query("DELETE FROM permission_decisions WHERE id = ?")
            .bind(decision.id.to_string())
            .execute(&pool)
            .await;

        assert!(result.is_err(), "DELETE sobre permission_decisions debe ser rechazado por el trigger append-only");
    }

    // Criterio 11: el interruptor de producción persiste en la BD.
    #[tokio::test]
    async fn production_override_toggle_persists() {
        let (pool, _file) = setup_db().await;

        // Valor inicial debe ser false (la migración lo inicializa a '0').
        let initial = McpGatewayRepository::get_production_override(&pool).await.unwrap();
        assert!(!initial, "valor inicial debe ser false");

        // Activar.
        McpGatewayRepository::set_production_override(&pool, true).await.unwrap();
        let after_enable = McpGatewayRepository::get_production_override(&pool).await.unwrap();
        assert!(after_enable, "debe ser true tras activar");

        // Desactivar.
        McpGatewayRepository::set_production_override(&pool, false).await.unwrap();
        let after_disable = McpGatewayRepository::get_production_override(&pool).await.unwrap();
        assert!(!after_disable, "debe ser false tras desactivar");
    }

    // ── Atomicidad bajo concurrencia (STORY-046 M1: chain_tip + append fusionados) ──

    fn sample_req(session: &str) -> PermissionRequest {
        PermissionRequest {
            pipeline: Pipeline::Ingest,
            institutional_tag: None,
            production_override_active: false,
            agent_session_id: session.to_string(),
            requested_scope: "ingest.submit_bar".into(),
        }
    }

    /// `record_decision` deriva la secuencia y encadena el hash DENTRO de la
    /// transacción -- dos llamadas secuenciales producen seq 1/2 con el
    /// mismo encadenamiento que el camino viejo `chain_tip` + `append`.
    #[tokio::test]
    async fn record_decision_persists_and_chains_across_calls() {
        let (pool, _file) = setup_db().await;
        let req = sample_req("session-record-decision");
        let outcome = evaluate_permission(&req);

        let first = McpGatewayRepository::record_decision(&pool, |prev_hash, seq| {
            make_decision(&req, &outcome, prev_hash.as_deref(), seq)
        })
        .await
        .expect("registrar primera decisión");
        assert_eq!(first.event_sequence_id, 1);
        assert_eq!(first.audit_chain_hash, None, "la fila génesis no debe encadenar");

        let second = McpGatewayRepository::record_decision(&pool, |prev_hash, seq| {
            make_decision(&req, &outcome, prev_hash.as_deref(), seq)
        })
        .await
        .expect("registrar segunda decisión");
        assert_eq!(second.event_sequence_id, 2);
        assert_eq!(second.audit_chain_hash, Some(first.audit_hash.clone()));

        let tip = McpGatewayRepository::chain_tip(&pool).await.unwrap().unwrap();
        assert_eq!(tip.1, 2);
        assert_eq!(tip.0, second.audit_hash);
    }

    /// CRITERIO DE CIERRE (QA por mutación): bajo contención de escritura
    /// SOSTENIDA (otro escritor retiene el lock de `BEGIN IMMEDIATE` y no lo
    /// suelta), el bucle de reintento de `record_decision` debe agotar
    /// EXACTAMENTE `MAX_APPEND_ATTEMPTS` intentos y rendirse con
    /// `WriteContention { attempts: MAX }` -- nunca descartar la decisión en
    /// silencio.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn record_decision_exhausts_exactly_max_attempts_when_write_lock_is_held() {
        use std::str::FromStr;
        use std::time::Duration;

        use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("mcp_gateway_forced_contention.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());

        let setup_pool = connect(&database_url).await.expect("conectar (setup)");
        migrate(&setup_pool).await.expect("migrar");
        setup_pool.close().await;

        // Opciones con busy_timeout=0: un lock ocupado falla de INMEDIATO con
        // "database is locked" en vez de esperar 5s -- contención
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

        let repo_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(immediate_opts())
            .await
            .expect("pool del repositorio");

        let req = sample_req("session-contention");
        let outcome = evaluate_permission(&req);

        let result = McpGatewayRepository::record_decision(&repo_pool, |prev_hash, seq| {
            make_decision(&req, &outcome, prev_hash.as_deref(), seq)
        })
        .await;

        drop(lock_tx); // libera el lock (limpieza; el resultado ya está tomado)

        match result {
            Err(McpGatewayError::WriteContention { attempts }) => {
                assert_eq!(
                    attempts, MAX_APPEND_ATTEMPTS,
                    "bajo contención sostenida debe agotar EXACTAMENTE MAX_APPEND_ATTEMPTS intentos"
                );
            }
            other => panic!(
                "se esperaba WriteContention {{ attempts: {MAX_APPEND_ATTEMPTS} }} bajo contención sostenida, se obtuvo: {other:?}"
            ),
        }
    }

    /// CRITERIO DE CIERRE (QA por mutación): `is_transient_write_conflict`
    /// distingue una violación UNIQUE PERMANENTE (la PK `id`, que NO se debe
    /// reintentar) de la contención transitoria. Fija que exige AMBAS
    /// condiciones (es violación UNIQUE **y** menciona `event_sequence_id`),
    /// no una sola (`&&` != `||`), y que no clasifica cualquier cosa como
    /// transitoria (`is_transient` != `true`).
    #[tokio::test]
    async fn is_transient_is_false_for_a_permanent_non_sequence_unique_violation() {
        let (pool, _file) = setup_db().await;

        // Inserta una fila válida y luego otra con el MISMO `id`: viola la
        // PRIMARY KEY `id`, NO el UNIQUE de `event_sequence_id`. Error UNIQUE
        // PERMANENTE cuyo mensaje NO menciona `event_sequence_id`.
        let insert_with_id_dup = |event_sequence_id: i64| {
            sqlx::query(
                "INSERT INTO permission_decisions \
                 (id, created_at, updated_at, audit_hash, event_sequence_id, node_id, process_id, \
                  agent_session_id, requested_scope, permission_outcome) \
                 VALUES ('dup-decision-id', 1, 1, 'h', ?, 'node-1', 1, 'session-x', 'scope.x', 'granted')",
            )
            .bind(event_sequence_id)
            .execute(&pool)
        };
        insert_with_id_dup(1).await.expect("primera fila válida");
        let err = insert_with_id_dup(2)
            .await
            .expect_err("la segunda fila debe violar la PRIMARY KEY id");

        let permanent = McpGatewayError::Sqlx(err);
        assert!(
            !is_transient_write_conflict(&permanent),
            "una violación UNIQUE de la PK (no de event_sequence_id) es PERMANENTE, no transitoria"
        );
    }
}
