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

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
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
    /// maestros (ADR-0020 V2) sobre una base de datos SQLite en modo WAL,
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
        // maestros (ADR-0020 V2) más el rowid implícito -> verifica el
        // conteo de columnas y una muestra representativa de nombres de
        // columna de cada uno de los 5 grupos de ADR-0020 V2.
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
}
