//! [SHELL] SQLite connection pool factory + embedded migration runner.
//!
//! ADR-0006 mandates a single centralized migration path: migration files
//! live in `/migrations` at the workspace root and are embedded into the
//! binary at compile time via `sqlx::migrate!`. ADR-0107/SAD mandates
//! SQLite 3 in WAL mode for all OLTP persistence.
//!
//! This module provides the only two primitives pipeline modules need:
//! - [`connect`]: open a WAL-mode SQLite pool at the given path (or
//!   `:memory:` for tests).
//! - [`migrate`]: apply all embedded migrations. Idempotent — running it
//!   against an already-migrated database is a no-op (ADR-0006).

use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;

/// Opens a SQLite connection pool with WAL (Write-Ahead Log) journal mode
/// enabled, as mandated by the project stack decision (SAD: "SQLite 3 con
/// WAL").
///
/// `database_url` accepts any connection string SQLx's SQLite driver
/// understands (e.g. `sqlite://path/to/db.sqlite`, `sqlite::memory:`).
/// The database file (and parent directories, if any) is created if it
/// does not already exist.
pub async fn connect(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);

    SqlitePoolOptions::new().connect_with(options).await
}

/// Applies all embedded migrations from `/migrations` (workspace root) to
/// the given pool.
///
/// Determinism & idempotency (ADR-0006): SQLx records applied migrations
/// in the `_sqlx_migrations` table. Calling this twice against the same
/// database is a no-op on the second call — no error, no duplicate
/// schema objects.
pub async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("../../migrations").run(pool).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

    /// W2 closing criterion: migration 0001 applies the 25 master fields
    /// (ADR-0020 V2) to a WAL-mode SQLite database, and re-running it is
    /// idempotent.
    #[tokio::test]
    async fn migration_0001_applies_in_wal_mode_and_is_idempotent() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("foundation.sqlite");
        let database_url = format!("sqlite://{}", db_path.display());

        // --- First application -------------------------------------------------
        let pool = connect(&database_url).await.expect("connect (1st)");

        // Confirm WAL mode is active.
        let journal_mode: String = sqlx::query("PRAGMA journal_mode")
            .fetch_one(&pool)
            .await
            .expect("read journal_mode")
            .get(0);
        assert_eq!(journal_mode.to_lowercase(), "wal");

        migrate(&pool).await.expect("migrate (1st run)");

        // Confirm the foundation table exists with all 25 master fields
        // (ADR-0020 V2) plus the implicit rowid -> verify column count and
        // a representative sample of column names from each of the 5
        // ADR-0020 V2 groups.
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

        // --- Second application (idempotency check) ----------------------------
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
