//! [SHELL] Fábrica del pool de conexiones SQLite + runner de migraciones embebidas.
//!
//! ADR-0006 exige un único camino centralizado de migración: los archivos
//! de migración viven en `/migrations` en la raíz del workspace y se
//! embeben en el binario en tiempo de compilación vía `sqlx::migrate!`.
//! ADR-0107/SAD exige SQLite 3 en modo WAL para toda persistencia OLTP.
//!
//! Este módulo provee las únicas dos primitivas que necesitan los módulos
//! del pipeline:
//! - [`connect`]: abre un pool de SQLite en modo WAL en la ruta dada (o
//!   `:memory:` para tests).
//! - [`migrate`]: aplica todas las migraciones embebidas. Idempotente —
//!   correrlo contra una base de datos ya migrada es un no-op (ADR-0006).

use std::str::FromStr;
use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;

/// Abre un pool de conexiones SQLite con el modo de journal WAL
/// (Write-Ahead Log) habilitado, según exige la decisión de stack del
/// proyecto (SAD: "SQLite 3 con WAL").
///
/// `database_url` acepta cualquier cadena de conexión que entienda el
/// driver SQLite de SQLx (ej. `sqlite://path/to/db.sqlite`,
/// `sqlite::memory:`). El archivo de base de datos (y los directorios
/// padre, si hace falta) se crea si todavía no existe.
pub async fn connect(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .journal_mode(SqliteJournalMode::Wal)
        // Espera hasta 5s a que se libere el lock de escritura antes de
        // fallar con "database is locked" (ADR-0141 R2). Sin esto, dos
        // escritores concurrentes que compiten por el lock de un ledger
        // append-only (p. ej. abrir `BEGIN IMMEDIATE`) fallarían de
        // inmediato en vez de esperar su turno; con esto, se serializan
        // limpiamente y el reintento acotado de la capa de repositorio solo
        // actúa como red de seguridad, no como camino normal.
        .busy_timeout(Duration::from_secs(5))
        // Activa `PRAGMA foreign_keys=ON` (ADR-0141 R1, hallazgo C1 de la
        // auditoría retroactiva). Sin esto SQLite acepta silenciosamente
        // inserts huérfanos contra la única FK real del baseline
        // (`job_results.job_uuid -> jobs.id`) -- la restricción existe en el
        // esquema pero queda inerte hasta que este PRAGMA se activa por
        // conexión (SQLite no lo persiste en el archivo).
        .foreign_keys(true)
        // En modo WAL, `synchronous=NORMAL` es tan durable como `FULL` ante
        // caídas de proceso (el WAL es el registro de verdad) y 2-3x más
        // rápido -- solo `synchronous=OFF` arriesgaría corrupción ante un
        // corte de energía a mitad de escritura (ADR-0141).
        .synchronous(SqliteSynchronous::Normal)
        // Techo del archivo WAL: evita que crezca sin límite ante lectores
        // de larga duración que retrasan el checkpoint automático
        // (67 108 864 bytes = 64 MB, ADR-0141).
        .pragma("journal_size_limit", "67108864")
        // Dispara un checkpoint automático cada 1000 páginas del WAL, para
        // que el archivo -wal no acumule crecimiento entre checkpoints
        // manuales (ADR-0141).
        .pragma("wal_autocheckpoint", "1000")
        .create_if_missing(true);

    SqlitePoolOptions::new().connect_with(options).await
}

/// Aplica todas las migraciones embebidas desde `/migrations` (raíz del
/// workspace) sobre el pool dado.
///
/// Determinismo e idempotencia (ADR-0006): SQLx registra las migraciones
/// aplicadas en la tabla `_sqlx_migrations`. Llamar esto dos veces contra
/// la misma base de datos es un no-op en la segunda llamada — sin error,
/// sin objetos de esquema duplicados.
pub async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("../../migrations").run(pool).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

    /// Criterio de cierre de W2: la migración 0001 aplica los 25 campos
    /// maestros (ADR-0020) sobre una base de datos SQLite en modo WAL,
    /// y volver a correrla es idempotente.
    #[tokio::test]
    async fn migration_0001_applies_in_wal_mode_and_is_idempotent() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("foundation.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());

        // --- Primera aplicación -------------------------------------------------
        let pool = connect(&database_url).await.expect("connect (1st)");

        // Confirma que el modo WAL está activo.
        let journal_mode: String = sqlx::query("PRAGMA journal_mode")
            .fetch_one(&pool)
            .await
            .expect("read journal_mode")
            .get(0);
        assert_eq!(journal_mode.to_lowercase(), "wal");

        migrate(&pool).await.expect("migrate (1st run)");

        // Confirma que la tabla de fundación existe con los 25 campos
        // maestros (ADR-0020) más el rowid implícito -> verifica el
        // conteo de columnas y una muestra representativa de nombres de
        // columna de cada uno de los 5 grupos de ADR-0020.
        let columns = sqlx::query(
            "SELECT name FROM pragma_table_info('foundation_master_fields')",
        )
        .fetch_all(&pool)
        .await
        .expect("read table_info");

        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();
        assert_eq!(column_names.len(), 25, "expected exactly 25 master fields");

        for expected in [
            // I. Identidad & Integridad
            "id",
            "created_at",
            "updated_at",
            "audit_hash",
            "audit_chain_hash",
            "event_sequence_id",
            // II. Soberanía & Propiedad
            "owner_id",
            "institutional_tag",
            "manifest_id",
            "access_token_id",
            // III. Linaje Alpha & Datos
            "version_node_id",
            "parent_id",
            "logic_hash",
            "data_snapshot_id",
            "transformation_id",
            // IV. Infraestructura & Ops
            "process_id",
            "session_id",
            "node_id",
            // V. Forense & Ejecución
            "portfolio_container_id",
            "compliance_status_id",
            "risk_audit_id",
            "indicator_state_hash",
            "execution_latency_ms",
            "source_signal_id",
            "signature_hash",
        ] {
            assert!(
                column_names.contains(&expected.to_string()),
                "missing master field column: {expected}"
            );
        }

        pool.close().await;

        // --- Segunda aplicación (verificación de idempotencia) ------------------
        let pool = connect(&database_url).await.expect("connect (2nd)");
        migrate(&pool).await.expect("migrate (2nd run) must be a no-op, not an error");

        let row_count: i64 = sqlx::query("SELECT COUNT(*) FROM foundation_master_fields")
            .fetch_one(&pool)
            .await
            .expect("count rows")
            .get(0);
        assert_eq!(row_count, 0, "idempotent re-migration must not alter data");

        pool.close().await;
    }

    /// CRITERIO DE CIERRE (hallazgo C1): con `PRAGMA foreign_keys=ON`
    /// activo, insertar una fila de `job_results` cuyo `job_uuid` NO existe
    /// en `jobs` debe fallar por violación de FK. Antes de activar
    /// `.foreign_keys(true)` en [`connect`], este mismo INSERT pasaba en
    /// silencio (SQLite trata las FKs como no-op sin el PRAGMA) -- esta
    /// prueba se cae si se quita esa línea.
    #[tokio::test]
    async fn inserting_job_result_with_unknown_job_uuid_is_rejected_by_foreign_key() {
        let pool = connect("sqlite::memory:").await.expect("connect");
        migrate(&pool).await.expect("migrate");

        let result = sqlx::query(
            "INSERT INTO job_results (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                job_uuid, result_data, error_message, completed_at\
            ) VALUES ('result-orphan', 0, 0, 'hash', NULL, 1, 'job-that-does-not-exist', NULL, NULL, 0)",
        )
        .execute(&pool)
        .await;

        assert!(
            result.is_err(),
            "un job_results.job_uuid huérfano debe rechazarse por la FK job_results -> jobs"
        );
    }

    /// CRITERIO DE CIERRE (ADR-0141 "Configuración del pool SQLite"): los
    /// PRAGMAs `synchronous=NORMAL`, `journal_size_limit=64MB` y
    /// `wal_autocheckpoint=1000` páginas quedan activos por conexión.
    #[tokio::test]
    async fn connect_activates_the_pragmas_required_by_adr_0141() {
        let pool = connect("sqlite::memory:").await.expect("connect");

        // `synchronous`: 0=OFF, 1=NORMAL, 2=FULL, 3=EXTRA.
        let synchronous: i64 = sqlx::query("PRAGMA synchronous")
            .fetch_one(&pool)
            .await
            .expect("read synchronous")
            .get(0);
        assert_eq!(synchronous, 1, "synchronous debe ser NORMAL (1)");

        let journal_size_limit: i64 = sqlx::query("PRAGMA journal_size_limit")
            .fetch_one(&pool)
            .await
            .expect("read journal_size_limit")
            .get(0);
        assert_eq!(journal_size_limit, 67_108_864, "journal_size_limit debe ser 64MB");

        let wal_autocheckpoint: i64 = sqlx::query("PRAGMA wal_autocheckpoint")
            .fetch_one(&pool)
            .await
            .expect("read wal_autocheckpoint")
            .get(0);
        assert_eq!(wal_autocheckpoint, 1_000, "wal_autocheckpoint debe ser 1000 páginas");

        let foreign_keys: i64 = sqlx::query("PRAGMA foreign_keys")
            .fetch_one(&pool)
            .await
            .expect("read foreign_keys")
            .get(0);
        assert_eq!(foreign_keys, 1, "foreign_keys debe estar activo (ON)");
    }

    /// CRITERIO DE CIERRE (hallazgo C2): las 6 tablas del baseline
    /// (migraciones `0001`-`0006`) se declaran `STRICT`, igual que las
    /// tablas `0007`-`0020`.
    #[tokio::test]
    async fn all_six_baseline_tables_are_declared_strict() {
        let pool = connect("sqlite::memory:").await.expect("connect");
        migrate(&pool).await.expect("migrate");

        for table in [
            "foundation_master_fields",
            "audit_events",
            "jobs",
            "job_results",
            "telemetry_samples",
            "permission_decisions",
            "mcp_gateway_config",
            "sovereign_download_records",
        ] {
            let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?")
                .bind(table)
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|_| panic!("leer sqlite_master para {table}"))
                .get(0);
            assert!(sql.contains("STRICT"), "la tabla '{table}' debe declararse STRICT; DDL: {sql}");
        }
    }
}
