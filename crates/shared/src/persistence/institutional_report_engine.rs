//! [SHELL] Repositorio de persistencia APPEND-ONLY ATÓMICO para el Motor
//! de Reportes Institucionales (`docs/features/institutional-report-engine.md`,
//! ADR-0144 cimiento #7, ADR-0027, ADR-0141, ADR-0020, ADR-0093, migración
//! `0013_generated_reports.sql`, STORY-034).
//!
//! Envuelve la tabla `generated_reports`. Dueño del único I/O de este
//! cimiento: lecturas/escrituras en SQLite, generación de UUIDv7
//! (ADR-0141) y la lectura del puerto [`Clock`]. La lógica pura (ensamblado
//! del reporte, serialización canónica, firma reproducible, hash
//! encadenado) vive en [`crate::domain::institutional_report_engine`] --
//! este módulo solo le da entradas inyectadas y persiste el resultado,
//! reflejando el patrón de
//! [`crate::persistence::enriched_domain_events::DomainEventRepository`]
//! (misma naturaleza APPEND-ONLY: `event_sequence_id UNIQUE`, sin
//! `row_version`, transacción `BEGIN IMMEDIATE` + reintento acotado).
//!
//! ## Por qué NO existe `update`/`delete` en esta API
//!
//! A propósito: la única operación de escritura que este repositorio
//! expone es [`GeneratedReportRepository::record_report`] (un INSERT). No
//! hay ningún método de actualización o borrado -- ni falta, porque los
//! triggers `trg_generated_reports_no_update`/`trg_generated_reports_no_delete`
//! de la migración los rechazarían de cualquier forma. La ausencia del
//! método en Rust es la primera línea de defensa; el trigger de SQLite es
//! la segunda (defensa en profundidad).

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::audit_log::GENESIS_PREVIOUS_HASH;
use crate::domain::clock::Clock;
use crate::domain::institutional_report_engine::{compute_report_audit_hash, InstitutionalReport};

/// Errores que devuelven las operaciones de [`GeneratedReportRepository`].
#[derive(Debug, thiserror::Error)]
pub enum GeneratedReportRepositoryError {
    /// La operación subyacente de SQLite falló.
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// El append no pudo completarse tras agotar los reintentos ante
    /// contención de escritura transitoria (otro escritor mantuvo el lock
    /// de la base de datos más allá del `busy_timeout`, o hubo colisión
    /// repetida al derivar `event_sequence_id`). El reporte NO se descartó
    /// en silencio -- se propaga este error tipado para que el llamador
    /// decida reintentar a un nivel superior o alertar (regla "Atomicidad
    /// de ledgers append-only", rust-engineer/SKILL.md §4).
    #[error("no se pudo registrar el reporte tras {attempts} intentos por contención de escritura")]
    WriteContention { attempts: u32 },
}

/// Número máximo de intentos del append ante contención de escritura
/// transitoria antes de rendirse con
/// [`GeneratedReportRepositoryError::WriteContention`]. Cinco es holgado:
/// con `busy_timeout` de 5s (ADR-0141 R2) el lock casi siempre se obtiene
/// sin reintentar; el bucle solo actúa si el `busy_timeout` expira bajo una
/// contención extrema.
const MAX_RECORD_ATTEMPTS: u32 = 5;

/// Decide si un error del repositorio es una contención de escritura
/// TRANSITORIA -- es decir, algo que reintentar (re-derivando el
/// `event_sequence_id` y reinsertando) puede resolver, sin descartar el
/// reporte. Mismo criterio que
/// `enriched_domain_events::is_transient_write_conflict`.
fn is_transient_write_conflict(error: &GeneratedReportRepositoryError) -> bool {
    let GeneratedReportRepositoryError::Database(sqlx::Error::Database(db)) = error else {
        return false;
    };

    let message = db.message().to_lowercase();
    // Lock ocupado: otro escritor tenía el lock de la BD / de la tabla.
    if message.contains("database is locked") || message.contains("database table is locked") {
        return true;
    }

    // Colisión de secuencia: mismo event_sequence_id derivado por dos
    // escritores -- transitorio, re-derivar y reinsertar lo resuelve.
    db.is_unique_violation() && message.contains("event_sequence_id")
}

/// Entrada para [`GeneratedReportRepository::record_report`] -- todo lo que
/// la Shell necesita para registrar UN reporte generado: el reporte en sí
/// (ya ensamblado y firmado por el Core), la identidad del
/// dueño/máquina (Perfil D de ADR-0020) y el veredicto de cumplimiento
/// vigente (nullable -- no todo reporte trae uno anotado).
#[derive(Debug, Clone)]
pub struct RecordGeneratedReportInput {
    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,
    pub compliance_status_id: Option<String>,
    /// El reporte ya ensamblado por el Core.
    pub report: InstitutionalReport,
    /// La firma reproducible del contenido del reporte, ya calculada por
    /// `domain::institutional_report_engine::compute_report_signature`
    /// ANTES de llamar aquí -- este repositorio no la recalcula, solo la
    /// persiste (separación Core/Shell: la Shell no decide contenido).
    pub signature_hash: String,
}

/// Una fila de `generated_reports` ya persistida.
#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedReportRow {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub event_sequence_id: i64,

    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,

    pub signature_hash: String,
    pub compliance_status_id: Option<String>,

    pub report_type: String,
    pub source_result_ref: Option<String>,
    pub source_event_refs: String,
    pub report_body: String,
}

/// Repositorio APPEND-ONLY para `generated_reports`.
///
/// Constrúyelo con un [`SqlitePool`] ya migrado y cualquier implementación
/// de [`Clock`] -- mismo patrón que
/// [`crate::persistence::enriched_domain_events::DomainEventRepository`].
pub struct GeneratedReportRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> GeneratedReportRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`. Ambos se toman
    /// prestados por la vida del repositorio -- no se toma ownership.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Registra UN reporte generado: deriva su posición en la cadena
    /// GLOBAL, computa su `audit_hash` encadenado y lo persiste como fila
    /// nueva.
    ///
    /// Es la ÚNICA forma de escribir en `generated_reports` -- no existe
    /// `update`/`delete` en esta API (ver doc-comment del módulo).
    ///
    /// ## Atomicidad bajo concurrencia (regla "Atomicidad de ledgers append-only")
    ///
    /// Todo el *read-then-write* (leer el MAX(`event_sequence_id`) y el
    /// `audit_hash` previo para encadenar, y el `INSERT` final) ocurre
    /// dentro de UNA sola transacción `BEGIN IMMEDIATE` -- ver
    /// [`Self::try_record_report_once`]. Sin esa transacción, dos
    /// escritores concurrentes derivarían el mismo `event_sequence_id`, el
    /// `UNIQUE` rechazaría a uno y su reporte se PERDERÍA. Ante contención
    /// transitoria (`SQLITE_BUSY` tras expirar el `busy_timeout`, o
    /// colisión de secuencia), se reintenta hasta [`MAX_RECORD_ATTEMPTS`]
    /// veces re-derivando la secuencia; el reporte NUNCA se descarta en
    /// silencio (si se agotan los reintentos se devuelve
    /// [`GeneratedReportRepositoryError::WriteContention`]).
    pub async fn record_report(
        &self,
        input: RecordGeneratedReportInput,
    ) -> Result<GeneratedReportRow, GeneratedReportRepositoryError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_record_report_once(&input).await {
                Ok(row) => return Ok(row),
                Err(error) => {
                    // Solo se reintenta ante contención de escritura
                    // transitoria; cualquier otro error se propaga de
                    // inmediato.
                    if is_transient_write_conflict(&error) {
                        if attempt < MAX_RECORD_ATTEMPTS {
                            continue;
                        }
                        // Agotados los reintentos: error tipado, NUNCA
                        // pérdida silenciosa del reporte.
                        return Err(GeneratedReportRepositoryError::WriteContention { attempts: attempt });
                    }
                    return Err(error);
                }
            }
        }
    }

    /// Un intento único del append, dentro de una transacción
    /// `BEGIN IMMEDIATE`. Devuelve el error de SQLite tal cual si algo
    /// falla -- el bucle de [`Self::record_report`] decide si es
    /// transitorio y hay que reintentar. La transacción se abre con
    /// `BEGIN IMMEDIATE` (no el `BEGIN` DEFERRED por defecto de SQLx) para
    /// tomar el lock de escritura de ENTRADA: así ningún otro escritor
    /// puede intercalar entre la lectura del MAX(`event_sequence_id`) y el
    /// `INSERT`, y se evita además el interbloqueo de upgrade que
    /// ocurriría si dos transacciones DEFERRED intentaran subir de lectura
    /// a escritura a la vez.
    async fn try_record_report_once(
        &self,
        input: &RecordGeneratedReportInput,
    ) -> Result<GeneratedReportRow, GeneratedReportRepositoryError> {
        // Abre la transacción tomando el lock de escritura de inmediato.
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        // Lectura (DENTRO de la transacción) -- posición en la cadena
        // GLOBAL: la fila con el event_sequence_id más alto de TODA la
        // tabla, para asignar la siguiente y encadenar su audit_hash.
        let tail_row = sqlx::query(
            "SELECT audit_hash, event_sequence_id \
             FROM generated_reports \
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

        // Núcleo puro: serialización canónica del reporte + sus columnas
        // propias derivadas del reporte ya ensamblado.
        let report_type = input.report.report_type();
        let report_body = input.report.canonical_report_json();
        let source_event_refs = serde_json::to_string(&input.report.source_event_refs)
            // Un Vec<String> siempre serializa -- nunca falla en la práctica.
            .expect("Vec<String> de source_event_refs siempre serializa");

        let id = Uuid::now_v7().to_string();
        // Reloj INYECTADO -- nunca SystemTime::now() directo (ADR-0002/0004).
        // Esta tabla es append-only: created_at y updated_at comparten la
        // misma lectura del reloj, la fila nunca se modifica.
        let now_ns = self.clock.timestamp_ns();

        let audit_hash = compute_report_audit_hash(
            &id,
            now_ns,
            event_sequence_id,
            &previous_audit_hash,
            &input.owner_id,
            &input.institutional_tag,
            &input.node_id,
            report_type,
            input.report.source_result_ref.as_deref(),
            &source_event_refs,
            &report_body,
            &input.signature_hash,
            input.compliance_status_id.as_deref(),
        );

        // Escritura (DENTRO de la transacción) -- el INSERT que cierra el
        // read-then-write atómico.
        sqlx::query(
            "INSERT INTO generated_reports (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, \
                signature_hash, compliance_status_id, \
                report_type, source_result_ref, source_event_refs, report_body\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
        .bind(&input.compliance_status_id)
        .bind(report_type)
        .bind(&input.report.source_result_ref)
        .bind(&source_event_refs)
        .bind(&report_body)
        .execute(&mut *tx)
        .await?;

        // Confirma la transacción: recién aquí el lock de escritura se
        // libera y la fila se hace visible a otros escritores.
        tx.commit().await?;

        Ok(GeneratedReportRow {
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
            compliance_status_id: input.compliance_status_id.clone(),
            report_type: report_type.to_string(),
            source_result_ref: input.report.source_result_ref.clone(),
            source_event_refs,
            report_body,
        })
    }

    /// Carga la cadena completa, ordenada por `event_sequence_id`
    /// ascendente -- usada por los tests de integridad de la cadena
    /// (génesis con `audit_chain_hash = NULL`, resto encadenado) y por
    /// cualquier consumidor futuro que reconstruya el historial de
    /// reportes generados.
    pub async fn load_chain(&self) -> Result<Vec<GeneratedReportRow>, GeneratedReportRepositoryError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, node_id, \
                    signature_hash, compliance_status_id, \
                    report_type, source_result_ref, source_event_refs, report_body \
             FROM generated_reports \
             ORDER BY event_sequence_id ASC",
        )
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter().map(row_to_generated_report).collect())
    }
}

/// Convierte una fila cruda de `generated_reports` al tipo
/// [`GeneratedReportRow`].
fn row_to_generated_report(row: sqlx::sqlite::SqliteRow) -> GeneratedReportRow {
    GeneratedReportRow {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        event_sequence_id: row.get("event_sequence_id"),
        owner_id: row.get("owner_id"),
        institutional_tag: row.get("institutional_tag"),
        node_id: row.get("node_id"),
        signature_hash: row.get("signature_hash"),
        compliance_status_id: row.get("compliance_status_id"),
        report_type: row.get("report_type"),
        source_result_ref: row.get("source_result_ref"),
        source_event_refs: row.get("source_event_refs"),
        report_body: row.get("report_body"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::domain::institutional_report_engine::{assemble_report, compute_report_signature, AssembleReportInput, ReportType};
    use crate::persistence::pool::{connect, migrate};
    use std::collections::BTreeMap;

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    fn sample_report_input(metric_value: i64) -> AssembleReportInput {
        let mut metrics = BTreeMap::new();
        metrics.insert("sharpe_e8".to_string(), metric_value);
        AssembleReportInput {
            report_type: ReportType::Validation,
            metrics,
            source_result_ref: Some("run-1".to_string()),
            source_event_refs: vec!["evt-1".to_string(), "evt-2".to_string()],
            generated_at_ns: 1_000,
        }
    }

    fn record_input(metric_value: i64) -> RecordGeneratedReportInput {
        let report = assemble_report(sample_report_input(metric_value));
        let signature_hash = compute_report_signature(&report);
        RecordGeneratedReportInput {
            owner_id: "owner-1".to_string(),
            institutional_tag: "DRASUS_LOCAL".to_string(),
            node_id: "node-1".to_string(),
            compliance_status_id: None,
            report,
            signature_hash,
        }
    }

    // ── CRITERIO #1 (Orden §5): esquema STRICT append-only + Grupo I ────────

    #[tokio::test]
    async fn migration_creates_generated_reports_table_strict_with_group_i_and_event_sequence_id() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('generated_reports')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();

        for expected in [
            "id", "created_at", "updated_at", "audit_hash", "audit_chain_hash", "event_sequence_id",
            "owner_id", "institutional_tag", "node_id",
            "signature_hash", "compliance_status_id",
            "report_type", "source_result_ref", "source_event_refs", "report_body",
        ] {
            assert!(column_names.contains(&expected.to_string()), "falta la columna: {expected}");
        }

        assert!(
            !column_names.contains(&"row_version".to_string()),
            "generated_reports es APPEND-ONLY (ADR-0141): no debe tener row_version, solo event_sequence_id"
        );

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'generated_reports'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(sql.contains("STRICT"), "la tabla generated_reports debe declararse STRICT");
    }

    // ── CRITERIO #3 (Orden §5): append atómico + concurrencia ────────────────

    /// CRITERIO DE CIERRE (DEBT-001): N escritores concurrentes sobre el
    /// MISMO pool/ledger. La transacción `BEGIN IMMEDIATE` + reintento
    /// acotado debe garantizar que NINGÚN reporte se pierde y que la
    /// secuencia queda densa (1..=N sin huecos ni duplicados) con la cadena
    /// de hashes íntegra y recomputable. BD en ARCHIVO temporal (nunca
    /// `:memory:`, donde cada conexión sería una base distinta) para que
    /// la concurrencia entre conexiones sea real.
    ///
    /// Esta prueba DEBE poder caerse si se quita la transacción: con el
    /// `SELECT MAX(...)` y el `INSERT` en sentencias sueltas, dos tareas
    /// leen el mismo MAX, derivan el mismo `event_sequence_id`, el `UNIQUE`
    /// rechaza a una y su fila se pierde -> la aserción (a) `chain.len()==N`
    /// o la (b) `1..=N` fallaría.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_record_reports_persist_every_report_without_gaps_or_lost_rows() {
        use std::sync::Arc;

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("generated_reports_concurrency.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());
        let pool = connect(&database_url).await.expect("conectar");
        migrate(&pool).await.expect("migrar");

        // Reloj compartido (atómico, thread-safe). No se hace `tick`: todas
        // las filas comparten timestamp, lo cual es válido -- el orden lo
        // fija `event_sequence_id`, no el reloj.
        let clock: Arc<DeterministicClock> = Arc::new(DeterministicClock::new(1_000, 100));

        const N: i64 = 16;

        // Lanza N tareas en paralelo, cada una registrando un reporte
        // distinto (métrica distinta para que cada firma sea única).
        let mut handles = Vec::new();
        for i in 0..N {
            let pool_c = pool.clone(); // SqlitePool es un Arc interno: clonar es barato.
            let clock_c = clock.clone();
            handles.push(tokio::spawn(async move {
                let repo = GeneratedReportRepository::new(&pool_c, clock_c.as_ref());
                repo.record_report(record_input((i + 1) * 100_000_000)).await
            }));
        }

        // (a) TODAS las tareas terminaron OK -- ningún reporte se perdió
        // por colisión de secuencia.
        for handle in handles {
            handle
                .await
                .expect("la tarea no debe entrar en panic")
                .expect("record_report debe tener éxito para cada escritor concurrente");
        }

        let repo = GeneratedReportRepository::new(&pool, clock.as_ref());
        let chain = repo.load_chain().await.expect("cargar la cadena completa");

        // (a) se persistieron TODAS las N filas.
        assert_eq!(chain.len() as i64, N, "deben persistirse las N filas, sin ninguna pérdida");

        // (b) los event_sequence_id son exactamente 1..=N (densa, sin
        // huecos ni duplicados). `load_chain` ya ordena ascendente por la
        // columna.
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

            let recomputed = compute_report_audit_hash(
                &row.id,
                row.created_at_ns,
                row.event_sequence_id,
                &previous_audit_hash,
                &row.owner_id,
                &row.institutional_tag,
                &row.node_id,
                &row.report_type,
                row.source_result_ref.as_deref(),
                &row.source_event_refs,
                &row.report_body,
                &row.signature_hash,
                row.compliance_status_id.as_deref(),
            );
            assert_eq!(recomputed, row.audit_hash, "el audit_hash de cada fila debe ser recomputable (integridad de la cadena)");
        }
    }

    // ── CRITERIO #4 (Orden §5): trazabilidad -- source_event_refs persistidos ──

    #[tokio::test]
    async fn source_event_refs_are_persisted_as_valid_json_array() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = GeneratedReportRepository::new(&pool, &clock);

        let row = repo.record_report(record_input(150_000_000)).await.expect("registrar reporte");

        let parsed: serde_json::Value = serde_json::from_str(&row.source_event_refs).expect("JSON válido");
        assert_eq!(parsed, serde_json::json!(["evt-1", "evt-2"]));
    }

    // ── CRITERIO #6 (Orden §5): audit_chain_hash génesis NULL + encadenado ──

    #[tokio::test]
    async fn audit_chain_hash_is_null_only_in_genesis_row_and_chains_afterwards() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = GeneratedReportRepository::new(&pool, &clock);

        let first = repo.record_report(record_input(100_000_000)).await.expect("génesis");
        clock.tick();
        let second = repo.record_report(record_input(200_000_000)).await.expect("segundo");

        assert_eq!(first.audit_chain_hash, None, "la fila génesis debe tener audit_chain_hash NULL");
        assert_eq!(second.audit_chain_hash, Some(first.audit_hash.clone()), "debe encadenar a la primera");
    }

    // ── CRITERIO #5 (Orden §5): signature_hash != audit_hash ────────────────

    #[tokio::test]
    async fn signature_hash_and_audit_hash_are_present_and_distinct() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = GeneratedReportRepository::new(&pool, &clock);

        let row = repo.record_report(record_input(150_000_000)).await.expect("registrar reporte");

        assert_ne!(row.signature_hash, row.audit_hash, "signature_hash (contenido) y audit_hash (fila) deben ser distintos");
        assert!(!row.signature_hash.is_empty());
        assert!(!row.audit_hash.is_empty());
    }

    // ── CRITERIO #7 (Orden §5): sin secretos + cero f64 ─────────────────────

    #[tokio::test]
    async fn persisted_report_body_has_no_secrets_and_no_floating_point() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = GeneratedReportRepository::new(&pool, &clock);

        let row = repo.record_report(record_input(150_000_000)).await.expect("registrar reporte");

        assert!(!row.report_body.contains('.'), "report_body no debe contener coma flotante");
        let lowercase_body = row.report_body.to_lowercase();
        for forbidden in ["password", "api_key", "private_key", "signing_key"] {
            assert!(!lowercase_body.contains(forbidden), "report_body no debe contener '{forbidden}'");
        }
    }

    // ── Append-only: UPDATE/DELETE rechazados por trigger ───────────────────

    #[tokio::test]
    async fn update_is_rejected_by_trigger() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = GeneratedReportRepository::new(&pool, &clock);

        let row = repo.record_report(record_input(150_000_000)).await.expect("registrar reporte");

        let result = sqlx::query("UPDATE generated_reports SET signature_hash = 'tampered' WHERE id = ?")
            .bind(&row.id)
            .execute(&pool)
            .await;

        assert!(result.is_err(), "UPDATE sobre generated_reports debe ser rechazado por el trigger");
    }

    #[tokio::test]
    async fn delete_is_rejected_by_trigger() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = GeneratedReportRepository::new(&pool, &clock);

        let row = repo.record_report(record_input(150_000_000)).await.expect("registrar reporte");

        let result = sqlx::query("DELETE FROM generated_reports WHERE id = ?")
            .bind(&row.id)
            .execute(&pool)
            .await;

        assert!(result.is_err(), "DELETE sobre generated_reports debe ser rechazado por el trigger");
    }

    /// CRITERIO DE CIERRE: duplicar una posición ya usada es rechazado por
    /// el `UNIQUE` de la migración -- se inserta directamente con SQL crudo
    /// para ejercitar el guardarraíl de la BD en sí mismo.
    #[tokio::test]
    async fn duplicating_event_sequence_id_is_rejected_by_unique_constraint() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = GeneratedReportRepository::new(&pool, &clock);

        repo.record_report(record_input(150_000_000))
            .await
            .expect("primer reporte (event_sequence_id = 1)");

        let duplicate = sqlx::query(
            "INSERT INTO generated_reports (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, \
                signature_hash, compliance_status_id, \
                report_type, source_result_ref, source_event_refs, report_body\
            ) VALUES ('id-dup', 0, 0, 'hash', NULL, 1, 'owner-1', 'DRASUS_LOCAL', 'node-1', \
                       'sig', NULL, 'VALIDATION', NULL, '[]', '{}')",
        )
        .execute(&pool)
        .await;

        assert!(duplicate.is_err(), "duplicar event_sequence_id=1 debe ser rechazado por UNIQUE");
    }

    /// CRITERIO DE CIERRE: un `report_type` fuera del catálogo es rechazado
    /// por el `CHECK` de la BD.
    #[tokio::test]
    async fn database_check_rejects_unknown_report_type() {
        let pool = migrated_pool().await;

        let result = sqlx::query(
            "INSERT INTO generated_reports (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, \
                signature_hash, compliance_status_id, \
                report_type, source_result_ref, source_event_refs, report_body\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', 'DRASUS_LOCAL', 'node-1', \
                       'sig', NULL, 'UNKNOWN_TYPE', NULL, '[]', '{}')",
        )
        .execute(&pool)
        .await;

        assert!(result.is_err(), "un report_type fuera del catálogo debe ser rechazado por el CHECK de la BD");
    }

    /// CRITERIO DE CIERRE: `CHECK(json_valid(source_event_refs))` rechaza
    /// JSON corrupto -- si el CHECK no existiera, el INSERT tendría éxito
    /// con basura en la columna.
    #[tokio::test]
    async fn database_check_rejects_invalid_json_in_source_event_refs() {
        let pool = migrated_pool().await;

        let result = sqlx::query(
            "INSERT INTO generated_reports (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, \
                signature_hash, compliance_status_id, \
                report_type, source_result_ref, source_event_refs, report_body\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', 'DRASUS_LOCAL', 'node-1', \
                       'sig', NULL, 'VALIDATION', NULL, '{not valid json', '{}')",
        )
        .execute(&pool)
        .await;

        assert!(result.is_err(), "source_event_refs con JSON corrupto debe ser rechazado por el CHECK(json_valid)");
    }

    /// CRITERIO DE CIERRE: `CHECK(json_valid(report_body))` rechaza JSON
    /// corrupto en el cuerpo del reporte.
    #[tokio::test]
    async fn database_check_rejects_invalid_json_in_report_body() {
        let pool = migrated_pool().await;

        let result = sqlx::query(
            "INSERT INTO generated_reports (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, \
                signature_hash, compliance_status_id, \
                report_type, source_result_ref, source_event_refs, report_body\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', 'DRASUS_LOCAL', 'node-1', \
                       'sig', NULL, 'VALIDATION', NULL, '[]', '{not valid json')",
        )
        .execute(&pool)
        .await;

        assert!(result.is_err(), "report_body con JSON corrupto debe ser rechazado por el CHECK(json_valid)");
    }

    // ── CRITERIO (QA por mutación, DEBT-018): reintento acotado hasta AGOTAR ──

    /// CRITERIO DE CIERRE (QA por mutación): bajo contención de escritura
    /// SOSTENIDA (otro escritor retiene el lock de `BEGIN IMMEDIATE` y no lo
    /// suelta), el bucle de reintento debe agotar EXACTAMENTE
    /// `MAX_RECORD_ATTEMPTS` intentos y rendirse con
    /// `WriteContention { attempts: MAX }` -- nunca descartar el reporte en
    /// silencio, ni rendirse un intento antes o después. Patrón de
    /// referencia: `persistence/data_portability.rs` (STORY-043, DEBT-018).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn record_report_exhausts_exactly_max_attempts_when_write_lock_is_held() {
        use std::str::FromStr;
        use std::time::Duration;

        use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("institutional_report_engine_forced_contention.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());

        // Migrar con el pool normal (busy_timeout de 5s).
        let pool = connect(&database_url).await.expect("conectar");
        migrate(&pool).await.expect("migrar");

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

        // Escritor B: intenta registrar un reporte mientras A retiene el
        // lock. Cada `try_record_report_once` abre `BEGIN IMMEDIATE`, choca
        // con el lock de A, falla con "database is locked" (transitorio) y
        // reintenta, hasta agotar MAX_RECORD_ATTEMPTS.
        let repo_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(immediate_opts())
            .await
            .expect("pool del repositorio");
        let clock = DeterministicClock::new(1_000, 100);
        let repo = GeneratedReportRepository::new(&repo_pool, &clock);

        let result = repo.record_report(record_input(150_000_000)).await;

        drop(lock_tx); // libera el lock (limpieza; el resultado ya está tomado)

        match result {
            Err(GeneratedReportRepositoryError::WriteContention { attempts }) => {
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

        // Inserta una fila válida y luego otra con el MISMO `id`: viola la
        // PRIMARY KEY `id`, NO el UNIQUE de `event_sequence_id`. Error UNIQUE
        // PERMANENTE cuyo mensaje NO menciona `event_sequence_id`.
        let insert_with_id_dup = |event_sequence_id: i64| {
            sqlx::query(
                "INSERT INTO generated_reports (\
                    id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, node_id, \
                    signature_hash, compliance_status_id, \
                    report_type, source_result_ref, source_event_refs, report_body\
                ) VALUES ('dup-id', 0, 0, 'hash', NULL, ?, 'owner-1', 'DRASUS_LOCAL', 'node-1', \
                           'sig', NULL, 'VALIDATION', NULL, '[]', '{}')",
            )
            .bind(event_sequence_id)
            .execute(&pool)
        };
        insert_with_id_dup(1).await.expect("primera fila válida");
        let err = insert_with_id_dup(2)
            .await
            .expect_err("la segunda fila debe violar la PRIMARY KEY id");

        let permanent = GeneratedReportRepositoryError::Database(err);
        assert!(
            !is_transient_write_conflict(&permanent),
            "una violación UNIQUE de la PK (no de event_sequence_id) es PERMANENTE, no transitoria"
        );

        // Control: un error que ni siquiera es de base de datos jamás es
        // transitorio (fija la rama temprana `let ... else`).
        let non_database = GeneratedReportRepositoryError::WriteContention { attempts: 5 };
        assert!(
            !is_transient_write_conflict(&non_database),
            "un error no-Database nunca es contención transitoria"
        );
    }

    // ── CRITERIO (QA por mutación, DEBT-018): fidelidad de la fila devuelta ───

    /// CRITERIO DE CIERRE (QA por mutación): la fila que DEVUELVE
    /// `record_report` es bit-a-bit idéntica a la fila persistida en disco
    /// -- si el literal de retorno de `try_record_report_once` sustituyera
    /// algún campo (`audit_hash`, `event_sequence_id`, timestamps...) por un
    /// valor por defecto en vez del recién calculado, esta comparación de
    /// igualdad completa lo detectaría.
    #[tokio::test]
    async fn record_report_returned_row_matches_the_persisted_row_exactly() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = GeneratedReportRepository::new(&pool, &clock);

        let first = repo.record_report(record_input(150_000_000)).await.expect("primer reporte");
        clock.tick();
        let second = repo.record_report(record_input(250_000_000)).await.expect("segundo reporte");

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
