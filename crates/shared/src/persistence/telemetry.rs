//! [SHELL] Repositorio para `telemetry_samples`
//! (`docs/features/telemetry.md` TTR-001, migración `0004_telemetry.sql`).
//!
//! A diferencia de [`super::audit_log::AuditLogRepository`], este
//! repositorio SÍ expone una operación de borrado (`purge_older_than`) —
//! `telemetry_samples` no es append-only, la poda automática es un
//! requisito explícito de la Feature, no una violación de invariante.
//!
//! `insert_batch` inserta varias muestras en una sola transacción SQLx: es
//! la mitad "cáscara" del buffer de alta velocidad — la otra mitad (la cola
//! en memoria que nunca espera al disco) vive en
//! `crate::orchestrator::telemetry`.

use sqlx::{Row, SqlitePool};

use crate::domain::telemetry::{TelemetrySample, TelemetrySampleContent};

/// Errores devueltos por las operaciones de [`TelemetryRepository`].
#[derive(Debug)]
pub enum TelemetryError {
    /// La operación de SQLite subyacente falló.
    Database(sqlx::Error),
}

impl std::fmt::Display for TelemetryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TelemetryError::Database(err) => write!(f, "error de base de datos de telemetría: {err}"),
        }
    }
}

impl std::error::Error for TelemetryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TelemetryError::Database(err) => Some(err),
        }
    }
}

impl From<sqlx::Error> for TelemetryError {
    fn from(err: sqlx::Error) -> Self {
        TelemetryError::Database(err)
    }
}

/// Repositorio para `telemetry_samples`.
///
/// Constrúyelo con un [`SqlitePool`] ya migrado (ver
/// [`crate::persistence::pool::connect`] + [`crate::persistence::pool::migrate`]).
/// A diferencia de [`super::audit_log::AuditLogRepository`], no necesita un
/// [`crate::domain::clock::Clock`] propio: las muestras ya llegan
/// completamente construidas (con su `id`/`created_at_ns`/cadena de hash ya
/// calculados por [`crate::domain::telemetry::build_sample`]) — este
/// repositorio solo las persiste, las purga o las consulta.
pub struct TelemetryRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> TelemetryRepository<'a> {
    /// Crea un repositorio ligado a `pool`. El pool se toma prestado, no se
    /// posee — el mismo pool puede compartirse con otros repositorios.
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Carga la muestra más reciente (mayor `event_sequence_id`), o `None`
    /// si la tabla está vacía.
    ///
    /// Se llama UNA SOLA VEZ, al iniciar el proceso — sirve para sembrar el
    /// estado de cadena en memoria de `orchestrator::telemetry::TelemetryBuffer`
    /// y así evitar que el `event_sequence_id` colisione con filas ya
    /// persistidas por una corrida anterior del proceso.
    pub async fn load_tail(&self) -> Result<Option<TelemetrySample>, TelemetryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    institutional_tag, logic_hash, session_id, node_id, process_id, \
                    execution_latency_ms, metric_name, details_json \
             FROM telemetry_samples \
             ORDER BY event_sequence_id DESC \
             LIMIT 1",
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(row.map(row_to_sample))
    }

    /// Inserta `samples` en una sola transacción SQLx — un lote, un commit,
    /// para que el flush de muchas muestras a la vez pague un solo costo de
    /// `fsync` en vez de uno por fila. Una lista vacía es un no-op (no abre
    /// transacción).
    pub async fn insert_batch(&self, samples: &[TelemetrySample]) -> Result<(), TelemetryError> {
        if samples.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        for sample in samples {
            sqlx::query(
                "INSERT INTO telemetry_samples (\
                    id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    institutional_tag, logic_hash, session_id, node_id, process_id, \
                    execution_latency_ms, metric_name, details_json\
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&sample.id)
            .bind(sample.created_at_ns)
            .bind(sample.updated_at_ns)
            .bind(&sample.audit_hash)
            .bind(&sample.audit_chain_hash)
            .bind(sample.event_sequence_id)
            .bind(&sample.content.institutional_tag)
            .bind(&sample.content.logic_hash)
            .bind(&sample.content.session_id)
            .bind(&sample.content.node_id)
            .bind(&sample.content.process_id)
            .bind(sample.content.execution_latency_ms)
            .bind(&sample.content.metric_name)
            .bind(&sample.content.details_json)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Purga (`DELETE`) las muestras con `created_at < cutoff_ns`
    /// (`docs/features/telemetry.md` "PODA AUTOMÁTICA"). Devuelve cuántas
    /// filas se borraron.
    ///
    /// `cutoff_ns` ya viene calculado por quien llama (la cáscara, a partir
    /// del `Clock` inyectado y `RETENTION_DAYS`) — esta función no decide
    /// qué es "ahora", solo ejecuta el corte.
    pub async fn purge_older_than(&self, cutoff_ns: i64) -> Result<u64, TelemetryError> {
        let result = sqlx::query("DELETE FROM telemetry_samples WHERE created_at < ?")
            .bind(cutoff_ns)
            .execute(self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// Consulta la serie temporal de `metric_name` en el rango
    /// `[from_ns, to_ns]` (ambos extremos inclusivos), ordenada por
    /// `created_at` ascendente.
    pub async fn query_by_metric(
        &self,
        metric_name: &str,
        from_ns: i64,
        to_ns: i64,
    ) -> Result<Vec<TelemetrySample>, TelemetryError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    institutional_tag, logic_hash, session_id, node_id, process_id, \
                    execution_latency_ms, metric_name, details_json \
             FROM telemetry_samples \
             WHERE metric_name = ? AND created_at >= ? AND created_at <= ? \
             ORDER BY created_at ASC",
        )
        .bind(metric_name)
        .bind(from_ns)
        .bind(to_ns)
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter().map(row_to_sample).collect())
    }
}

/// Convierte una fila de `telemetry_samples` en el tipo de núcleo
/// [`TelemetrySample`].
fn row_to_sample(row: sqlx::sqlite::SqliteRow) -> TelemetrySample {
    TelemetrySample {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        event_sequence_id: row.get("event_sequence_id"),
        content: TelemetrySampleContent {
            metric_name: row.get("metric_name"),
            details_json: row.get("details_json"),
            institutional_tag: row.get("institutional_tag"),
            logic_hash: row.get("logic_hash"),
            session_id: row.get("session_id"),
            node_id: row.get("node_id"),
            process_id: row.get("process_id"),
            execution_latency_ms: row.get("execution_latency_ms"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::telemetry::build_sample;
    use crate::persistence::pool::{connect, migrate};

    fn latency_content(metric_name: &str, execution_latency_ms: i64) -> TelemetrySampleContent {
        TelemetrySampleContent {
            metric_name: metric_name.to_string(),
            details_json: None,
            institutional_tag: "BACKTEST".to_string(),
            logic_hash: Some("shared-v1".to_string()),
            session_id: Some("session-1".to_string()),
            node_id: Some("node-1".to_string()),
            process_id: "process-1".to_string(),
            execution_latency_ms: Some(execution_latency_ms),
        }
    }

    fn heartbeat_content(metric_name: &str) -> TelemetrySampleContent {
        TelemetrySampleContent {
            metric_name: metric_name.to_string(),
            details_json: None,
            institutional_tag: "BACKTEST".to_string(),
            logic_hash: Some("shared-v1".to_string()),
            session_id: Some("session-1".to_string()),
            node_id: Some("node-1".to_string()),
            process_id: "process-1".to_string(),
            execution_latency_ms: None,
        }
    }

    /// CRITERIO #2 (persistencia del heartbeat): una muestra sin valor de
    /// latencia se persiste y se relee con `execution_latency_ms == None`
    /// — la columna NULL de SQLite hace el viaje completo de ida y vuelta
    /// hacia `Option<i64>`, no solo se construye en memoria.
    #[tokio::test]
    async fn heartbeat_sample_persists_with_null_execution_latency() {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        let repo = TelemetryRepository::new(&pool);

        let sample = build_sample("hb-1".to_string(), 1_000, heartbeat_content("job_executor.heartbeat"), None);
        repo.insert_batch(std::slice::from_ref(&sample)).await.expect("insertar heartbeat");

        let reloaded = repo.load_tail().await.expect("cargar la cola").expect("debe existir una muestra");
        assert_eq!(reloaded.content.execution_latency_ms, None);
        assert_eq!(reloaded.content.metric_name, "job_executor.heartbeat");
    }

    /// CRITERIO #5 (durabilidad): las muestras persisten tras reabrir la
    /// base de datos. Usa un archivo temporal real, NO `sqlite::memory:` —
    /// una DB en memoria no sobrevive a cerrar y reabrir el pool, así que
    /// no demostraría nada sobre durabilidad.
    #[tokio::test]
    async fn samples_persist_after_reopening_the_database() {
        let temp_dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = temp_dir.path().join("telemetry_durability.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());

        let sample_id = {
            let pool = connect(&database_url).await.expect("conectar (antes de cerrar)");
            migrate(&pool).await.expect("migrar (antes de cerrar)");

            let repo = TelemetryRepository::new(&pool);
            let sample = build_sample("id-1".to_string(), 1_000, latency_content("ingest.hot_path_latency", 12), None);
            repo.insert_batch(std::slice::from_ref(&sample)).await.expect("insertar lote");

            pool.close().await;
            sample.id
        };

        // Pool completamente nuevo sobre el MISMO archivo — equivalente a
        // reiniciar el proceso.
        let pool = connect(&database_url).await.expect("conectar (después de reabrir)");
        migrate(&pool).await.expect("migrar (después de reabrir) debe ser un no-op");

        let repo = TelemetryRepository::new(&pool);
        let tail = repo.load_tail().await.expect("cargar la cola tras reabrir");

        assert_eq!(tail.expect("debe existir una muestra").id, sample_id);

        pool.close().await;
    }

    /// CRITERIO #6 (poda): `purge_older_than` borra solo las muestras más
    /// viejas que el corte, conserva el resto.
    #[tokio::test]
    async fn purge_older_than_deletes_only_samples_before_the_cutoff() {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        let repo = TelemetryRepository::new(&pool);

        let old = build_sample("old".to_string(), 1_000, latency_content("m", 1), None);
        let recent = build_sample("recent".to_string(), 10_000, latency_content("m", 2), Some(&old));
        repo.insert_batch(&[old, recent]).await.expect("insertar lote");

        let deleted = repo.purge_older_than(5_000).await.expect("purgar");
        assert_eq!(deleted, 1, "solo la muestra vieja debe borrarse");

        let survivors = repo.query_by_metric("m", 0, i64::MAX).await.expect("consultar supervivientes");
        assert_eq!(survivors.len(), 1);
        assert_eq!(survivors[0].id, "recent");
    }

    /// CRITERIO #7 (consulta): `query_by_metric` devuelve solo la serie del
    /// `metric_name` pedido, dentro del rango, ordenada por tiempo.
    #[tokio::test]
    async fn query_by_metric_filters_by_name_and_range_and_orders_by_time() {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        let repo = TelemetryRepository::new(&pool);

        let first = build_sample("a-1000".to_string(), 1_000, latency_content("metric.a", 1), None);
        let other_metric = build_sample("b-2000".to_string(), 2_000, latency_content("metric.b", 9), Some(&first));
        let second = build_sample("a-3000".to_string(), 3_000, latency_content("metric.a", 2), Some(&other_metric));
        let outside_range = build_sample("a-9000".to_string(), 9_000, latency_content("metric.a", 3), Some(&second));

        repo.insert_batch(&[first, other_metric, second, outside_range])
            .await
            .expect("insertar lote");

        let series = repo.query_by_metric("metric.a", 0, 5_000).await.expect("consultar serie");

        assert_eq!(series.len(), 2, "solo metric.a dentro del rango, fuera queda metric.b y la muestra en 9000");
        assert_eq!(series[0].id, "a-1000");
        assert_eq!(series[1].id, "a-3000");
        assert!(series[0].created_at_ns < series[1].created_at_ns);
    }

    /// `load_tail` en una tabla vacía es `None` — el próximo `build_sample`
    /// debe tratarse como génesis.
    #[tokio::test]
    async fn load_tail_on_empty_table_is_none() {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        let repo = TelemetryRepository::new(&pool);

        assert!(repo.load_tail().await.expect("cargar la cola").is_none());
    }
}
