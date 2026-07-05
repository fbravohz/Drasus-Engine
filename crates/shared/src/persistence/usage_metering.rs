//! [SHELL] Repositorio de persistencia APPEND-ONLY para Usage Metering /
//! Libro de Nocional (`docs/features/usage-metering.md`, ADR-0143,
//! ADR-0144, ADR-0141, ADR-0020, migración `0010_usage_metering.sql`,
//! STORY-030).
//!
//! Envuelve la tabla `usage_records`. Dueño del único I/O de este cimiento:
//! lecturas/escrituras en SQLite, generación de UUIDv7 (ADR-0141) y la
//! lectura del puerto [`Clock`]. La lógica pura (cálculo de nocional,
//! acumulación, cruce de umbral, hash encadenado) vive en
//! [`crate::domain::usage_metering`] -- este módulo solo le da entradas
//! inyectadas y persiste el resultado, reflejando el patrón de
//! [`crate::persistence::audit_log::AuditLogRepository`] (misma
//! naturaleza APPEND-ONLY: `event_sequence_id UNIQUE`, sin `row_version`).
//!
//! ## Por qué NO existe `update`/`delete` en esta API
//!
//! A propósito: la única operación que este repositorio expone es
//! [`UsageRepository::record_operation`] (un INSERT). No hay ningún método
//! de actualización o borrado -- ni falta, porque los triggers
//! `trg_usage_records_no_update`/`trg_usage_records_no_delete` de la
//! migración los rechazarían de cualquier forma. La ausencia del método en
//! Rust es la primera línea de defensa; el trigger de SQLite es la
//! segunda (defensa en profundidad).

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::audit_log::GENESIS_PREVIOUS_HASH;
use crate::domain::clock::Clock;
use crate::domain::usage_metering::{
    accumulate, compute_notional, compute_usage_audit_hash, detect_quota_crossing, NotionalError,
    QuotaVerdict,
};

/// Errores que devuelven las operaciones de [`UsageRepository`].
#[derive(Debug, thiserror::Error)]
pub enum UsageRepositoryError {
    /// La operación subyacente de SQLite falló.
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// El cálculo de nocional o la acumulación fallaron (tamaño/precio
    /// negativos, o desborde de `i64`) -- ninguna fila llega a persistirse
    /// si esto ocurre.
    #[error("error de cálculo de nocional: {0}")]
    Notional(#[from] NotionalError),
    /// Una fila de `usage_records` tenía un `quota_verdict` fuera de las
    /// dos cadenas canónicas -- error de integridad de datos.
    #[error("quota_verdict desconocido en la fila '{0}' de usage_records")]
    UnknownQuotaVerdict(String),
}

/// Entrada para [`UsageRepository::record_operation`] -- todo lo que la
/// Shell necesita para registrar UNA operación medida: identidad del
/// dueño/máquina, el ciclo vigente, la operación en sí (tamaño/precio/
/// instrumento, ver [`crate::domain::usage_metering::MeteredOperation`]) y
/// el `notional_limit` REAL ya resuelto por `plan-tier-quota` (#3).
#[derive(Debug, Clone)]
pub struct RecordOperationInput {
    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,
    /// Estado de cumplimiento vigente al momento de la operación
    /// (Grupo V, subset -- nullable: no toda operación lo trae).
    pub compliance_status_id: Option<String>,
    pub billing_cycle_id: String,
    pub instrument_id: String,
    pub size: i64,
    pub price: i64,
    /// `notional_limit` REAL de `plan_tier_quota::PlanLimits` (#3, YA
    /// CONSTRUIDO) -- resuelto por quien llama ANTES de invocar este
    /// método (la Shell de `usage-metering` no depende del crate de
    /// `plan-tier-quota" -- ambos viven en `shared`, se consumen vía
    /// función pública, no vía acoplamiento de crates).
    pub notional_limit: i64,
}

/// Una fila de `usage_records` ya persistida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageRecordRow {
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

    pub notional_per_op: i64,
    pub cycle_accumulated: i64,
    pub billing_cycle_id: String,
    pub instrument_id: String,
    pub quota_verdict: QuotaVerdict,
}

/// Repositorio APPEND-ONLY para `usage_records`.
///
/// Constrúyelo con un [`SqlitePool`] ya migrado y cualquier implementación
/// de [`Clock`] -- mismo patrón que
/// [`crate::persistence::audit_log::AuditLogRepository`] /
/// [`crate::persistence::plan_tier_quota::PlanRepository`].
pub struct UsageRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> UsageRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`. Ambos se toman
    /// prestados por la vida del repositorio -- no se toma ownership.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Carga la fila más reciente de la cadena GLOBAL (el
    /// `event_sequence_id` más alto de TODA la tabla, no por dueño ni por
    /// ciclo) -- necesaria para encadenar el `audit_hash` de la siguiente
    /// fila, exactamente como
    /// [`crate::persistence::audit_log::AuditLogRepository::load_tail`].
    async fn load_tail(&self) -> Result<Option<UsageRecordRow>, UsageRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, node_id, compliance_status_id, \
                    notional_per_op, cycle_accumulated, billing_cycle_id, instrument_id, quota_verdict \
             FROM usage_records \
             ORDER BY event_sequence_id DESC \
             LIMIT 1",
        )
        .fetch_optional(self.pool)
        .await?;

        row.map(row_to_usage_record).transpose()
    }

    /// Suma el `notional_per_op` de todas las filas YA PERSISTIDAS de un
    /// mismo `(owner_id, billing_cycle_id)` -- el acumulado del ciclo
    /// ANTES de sumar la operación que se va a registrar ahora
    /// (`docs/features/usage-metering.md` "Ciclo de Vida" - "Proceso":
    /// "lo acumula en el ciclo vigente"). Un `billing_cycle_id` nuevo
    /// (ciclo distinto) siempre arranca en cero -- así es como "el
    /// acumulado se reinicia" sin borrar ninguna fila histórica.
    async fn cycle_accumulated_so_far(
        &self,
        owner_id: &str,
        billing_cycle_id: &str,
    ) -> Result<i64, UsageRepositoryError> {
        let row = sqlx::query(
            "SELECT COALESCE(SUM(notional_per_op), 0) AS total \
             FROM usage_records \
             WHERE owner_id = ? AND billing_cycle_id = ?",
        )
        .bind(owner_id)
        .bind(billing_cycle_id)
        .fetch_one(self.pool)
        .await?;

        Ok(row.get::<i64, _>("total"))
    }

    /// Registra UNA operación medida: calcula su nocional
    /// ([`compute_notional`]), lo suma al acumulado del ciclo vigente
    /// ([`accumulate`]), compara el nuevo acumulado contra `notional_limit`
    /// ([`detect_quota_crossing`]) y persiste la fila APPEND-ONLY
    /// resultante, encadenada por hash a la fila anterior de la secuencia
    /// GLOBAL.
    ///
    /// Es la ÚNICA forma de escribir en `usage_records` -- no existe
    /// `update`/`delete` en esta API (ver doc-comment del módulo).
    pub async fn record_operation(
        &self,
        input: RecordOperationInput,
    ) -> Result<UsageRecordRow, UsageRepositoryError> {
        // Núcleo puro: nocional de esta operación (puede fallar por
        // tamaño/precio negativos o desborde -- ninguna fila se persiste
        // si esto falla, `?` propaga antes de tocar disco).
        let notional_per_op = compute_notional(input.size, input.price)?;

        // Acumulado del ciclo ANTES de esta operación + esta operación.
        let previous_cumulative = self
            .cycle_accumulated_so_far(&input.owner_id, &input.billing_cycle_id)
            .await?;
        let cycle_accumulated = accumulate(previous_cumulative, notional_per_op)?;

        // Veredicto de cuota usando el notional_limit REAL de plan-tier-quota.
        let quota_verdict = detect_quota_crossing(cycle_accumulated, input.notional_limit);

        // Posición en la cadena GLOBAL (mismo patrón que audit_log::append).
        let previous = self.load_tail().await?;
        let (event_sequence_id, audit_chain_hash, previous_audit_hash) = match &previous {
            Some(prev) => (prev.event_sequence_id + 1, Some(prev.audit_hash.clone()), prev.audit_hash.clone()),
            None => (1, None, GENESIS_PREVIOUS_HASH.to_string()),
        };

        let id = Uuid::now_v7().to_string();
        let now_ns = self.clock.timestamp_ns();

        let audit_hash = compute_usage_audit_hash(
            &id,
            now_ns,
            event_sequence_id,
            &previous_audit_hash,
            &input.owner_id,
            &input.institutional_tag,
            &input.node_id,
            &input.billing_cycle_id,
            &input.instrument_id,
            notional_per_op,
            cycle_accumulated,
            quota_verdict,
        );

        sqlx::query(
            "INSERT INTO usage_records (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, compliance_status_id, \
                notional_per_op, cycle_accumulated, billing_cycle_id, instrument_id, quota_verdict\
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
        .bind(&input.compliance_status_id)
        .bind(notional_per_op)
        .bind(cycle_accumulated)
        .bind(&input.billing_cycle_id)
        .bind(&input.instrument_id)
        .bind(quota_verdict.as_str())
        .execute(self.pool)
        .await?;

        Ok(UsageRecordRow {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash,
            event_sequence_id,
            owner_id: input.owner_id,
            institutional_tag: input.institutional_tag,
            node_id: input.node_id,
            compliance_status_id: input.compliance_status_id,
            notional_per_op,
            cycle_accumulated,
            billing_cycle_id: input.billing_cycle_id,
            instrument_id: input.instrument_id,
            quota_verdict,
        })
    }

    /// Carga la cadena completa, ordenada por `event_sequence_id`
    /// ascendente -- usada por los tests de integridad de la cadena
    /// (génesis con `audit_chain_hash = NULL`, resto encadenado).
    pub async fn load_chain(&self) -> Result<Vec<UsageRecordRow>, UsageRepositoryError> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                    owner_id, institutional_tag, node_id, compliance_status_id, \
                    notional_per_op, cycle_accumulated, billing_cycle_id, instrument_id, quota_verdict \
             FROM usage_records \
             ORDER BY event_sequence_id ASC",
        )
        .fetch_all(self.pool)
        .await?;

        rows.into_iter().map(row_to_usage_record).collect()
    }
}

/// Convierte una fila de `usage_records` al tipo [`UsageRecordRow`].
fn row_to_usage_record(row: sqlx::sqlite::SqliteRow) -> Result<UsageRecordRow, UsageRepositoryError> {
    let quota_verdict_value: String = row.get("quota_verdict");
    let quota_verdict = QuotaVerdict::from_str_value(&quota_verdict_value)
        .ok_or(UsageRepositoryError::UnknownQuotaVerdict(quota_verdict_value))?;

    Ok(UsageRecordRow {
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
        notional_per_op: row.get("notional_per_op"),
        cycle_accumulated: row.get("cycle_accumulated"),
        billing_cycle_id: row.get("billing_cycle_id"),
        instrument_id: row.get("instrument_id"),
        quota_verdict,
    })
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

    fn sample_input(billing_cycle_id: &str, size: i64, price: i64, notional_limit: i64) -> RecordOperationInput {
        RecordOperationInput {
            owner_id: "owner-1".to_string(),
            institutional_tag: "DRASUS_LOCAL".to_string(),
            node_id: "node-1".to_string(),
            compliance_status_id: None,
            billing_cycle_id: billing_cycle_id.to_string(),
            instrument_id: "BTCUSDT".to_string(),
            size,
            price,
            notional_limit,
        }
    }

    // ── CRITERIO #1 (Orden §5): esquema STRICT append-only + Grupo I ────────

    #[tokio::test]
    async fn migration_creates_usage_records_table_with_group_i_and_event_sequence_id() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('usage_records')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();

        for expected in [
            "id", "created_at", "updated_at", "audit_hash", "audit_chain_hash", "event_sequence_id",
            "owner_id", "institutional_tag", "node_id", "compliance_status_id",
            "notional_per_op", "cycle_accumulated", "billing_cycle_id", "instrument_id", "quota_verdict",
        ] {
            assert!(column_names.contains(&expected.to_string()), "falta la columna: {expected}");
        }

        assert!(
            !column_names.contains(&"row_version".to_string()),
            "usage_records es APPEND-ONLY (ADR-0141): no debe tener row_version, solo event_sequence_id"
        );

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'usage_records'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(sql.contains("STRICT"), "la tabla usage_records debe declararse STRICT");
    }

    /// CRITERIO DE CIERRE (Orden §5, criterio #6): ninguna columna de
    /// monto es `REAL`.
    #[tokio::test]
    async fn amount_columns_are_never_real() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name, type FROM pragma_table_info('usage_records')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        for row in columns {
            let name: String = row.get("name");
            if name == "notional_per_op" || name == "cycle_accumulated" {
                let column_type: String = row.get("type");
                assert_eq!(column_type, "INTEGER", "la columna '{name}' nunca debe ser REAL");
            }
        }
    }

    // ── CRITERIO #1 (Orden §5): append-only -- UPDATE/DELETE rechazados ─────

    /// CRITERIO DE CIERRE: un `UPDATE` sobre `usage_records` es rechazado
    /// por el trigger de la migración -- si el trigger no existiera (o la
    /// tabla permitiera mutar), esta prueba fallaría con `Ok`.
    #[tokio::test]
    async fn update_is_rejected_by_trigger() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = UsageRepository::new(&pool, &clock);

        let row = repo
            .record_operation(sample_input("2026-07", 250_000_000, 4_000_000_000_000, 1_000_000_000_000))
            .await
            .expect("registrar operación");

        let result = sqlx::query("UPDATE usage_records SET cycle_accumulated = 0 WHERE id = ?")
            .bind(&row.id)
            .execute(&pool)
            .await;

        assert!(result.is_err(), "UPDATE sobre usage_records debe ser rechazado por el trigger");
    }

    /// CRITERIO DE CIERRE: un `DELETE` sobre `usage_records` es rechazado
    /// por el trigger de la migración.
    #[tokio::test]
    async fn delete_is_rejected_by_trigger() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = UsageRepository::new(&pool, &clock);

        let row = repo
            .record_operation(sample_input("2026-07", 250_000_000, 4_000_000_000_000, 1_000_000_000_000))
            .await
            .expect("registrar operación");

        let result = sqlx::query("DELETE FROM usage_records WHERE id = ?")
            .bind(&row.id)
            .execute(&pool)
            .await;

        assert!(result.is_err(), "DELETE sobre usage_records debe ser rechazado por el trigger");
    }

    // ── CRITERIO #5 (Orden §5): event_sequence_id monótono y UNIQUE ─────────

    /// CRITERIO DE CIERRE: inserciones consecutivas asignan
    /// `event_sequence_id` 1, 2, 3... -- si la asignación no fuera
    /// monótona, esta prueba vería posiciones repetidas o desordenadas.
    #[tokio::test]
    async fn event_sequence_id_is_monotonic_across_inserts() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = UsageRepository::new(&pool, &clock);

        let first = repo
            .record_operation(sample_input("2026-07", 100_000_000, 100_000_000, 1_000_000_000_000))
            .await
            .expect("primera operación");
        clock.tick();
        let second = repo
            .record_operation(sample_input("2026-07", 100_000_000, 100_000_000, 1_000_000_000_000))
            .await
            .expect("segunda operación");
        clock.tick();
        let third = repo
            .record_operation(sample_input("2026-07", 100_000_000, 100_000_000, 1_000_000_000_000))
            .await
            .expect("tercera operación");

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
        let repo = UsageRepository::new(&pool, &clock);

        repo.record_operation(sample_input("2026-07", 100_000_000, 100_000_000, 1_000_000_000_000))
            .await
            .expect("primera operación (event_sequence_id = 1)");

        let duplicate = sqlx::query(
            "INSERT INTO usage_records (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, compliance_status_id, \
                notional_per_op, cycle_accumulated, billing_cycle_id, instrument_id, quota_verdict\
            ) VALUES ('id-dup', 0, 0, 'hash', NULL, 1, 'owner-1', 'DRASUS_LOCAL', 'node-1', NULL, \
                       0, 0, '2026-07', 'BTCUSDT', 'WITHIN')",
        )
        .execute(&pool)
        .await;

        assert!(duplicate.is_err(), "duplicar event_sequence_id=1 debe ser rechazado por UNIQUE");
    }

    // ── CRITERIO #3 (Orden §5): acumulación por ciclo + reinicio ────────────

    /// CRITERIO DE CIERRE: varias operaciones en el MISMO ciclo acumulan
    /// suma exacta -- si `cycle_accumulated_so_far` no sumara
    /// correctamente, el segundo/tercer acumulado no reflejaría la suma.
    #[tokio::test]
    async fn multiple_operations_in_the_same_cycle_accumulate_exactly() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = UsageRepository::new(&pool, &clock);

        // Tres operaciones de $1,000.00 cada una ($100_000_000_00 en ×1e8:
        // size=1e8 (1.0), price=100_000_000_000 ($1,000.00) -> nocional $1,000.00.
        let notional_limit = 1_000_000_000_000_000; // límite alto, no cruza en esta prueba
        let first = repo
            .record_operation(sample_input("2026-07", 100_000_000, 100_000_000_000, notional_limit))
            .await
            .expect("primera operación");
        let second = repo
            .record_operation(sample_input("2026-07", 100_000_000, 100_000_000_000, notional_limit))
            .await
            .expect("segunda operación");
        let third = repo
            .record_operation(sample_input("2026-07", 100_000_000, 100_000_000_000, notional_limit))
            .await
            .expect("tercera operación");

        assert_eq!(first.notional_per_op, 100_000_000_000); // $1,000.00
        assert_eq!(first.cycle_accumulated, 100_000_000_000);
        assert_eq!(second.cycle_accumulated, 200_000_000_000);
        assert_eq!(third.cycle_accumulated, 300_000_000_000, "la suma del ciclo debe ser exacta");
    }

    /// CRITERIO DE CIERRE: cambiar de `billing_cycle_id` reinicia el
    /// acumulado a partir de cero para el ciclo NUEVO -- pero las filas
    /// del ciclo viejo siguen intactas en la tabla (histórico conservado).
    #[tokio::test]
    async fn changing_billing_cycle_resets_accumulation_but_keeps_history() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = UsageRepository::new(&pool, &clock);

        let notional_limit = 1_000_000_000_000_000;
        repo.record_operation(sample_input("2026-07", 100_000_000, 100_000_000_000, notional_limit))
            .await
            .expect("operación del ciclo de julio");
        clock.tick();
        repo.record_operation(sample_input("2026-07", 100_000_000, 100_000_000_000, notional_limit))
            .await
            .expect("segunda operación del ciclo de julio");

        clock.tick();
        // Ciclo nuevo (agosto) -- el acumulado debe arrancar en cero, no en 200_000_000_000.
        let august_first = repo
            .record_operation(sample_input("2026-08", 100_000_000, 100_000_000_000, notional_limit))
            .await
            .expect("primera operación del ciclo de agosto");

        assert_eq!(august_first.cycle_accumulated, 100_000_000_000, "el ciclo nuevo arranca en cero + esta operación");

        // El histórico de julio sigue completo (2 filas), no se borró nada.
        let chain = repo.load_chain().await.expect("cargar cadena completa");
        assert_eq!(chain.len(), 3, "las 2 filas de julio + la de agosto deben seguir todas presentes");
        let july_rows: Vec<_> = chain.iter().filter(|r| r.billing_cycle_id == "2026-07").collect();
        assert_eq!(july_rows.len(), 2, "el histórico de julio se conserva intacto");
    }

    // ── CRITERIO #4 (Orden §5): cruce de umbral (con notional_limit ya resuelto) ──

    /// CRITERIO DE CIERRE: por debajo del límite, el veredicto es
    /// `Within`; al cruzarlo, `Crossed`. Usa un `notional_limit` ya
    /// resuelto (simula el valor REAL que produciría `plan_tier_quota`
    /// para el tier FREE -- ver `orchestrator::usage_metering` para el
    /// cableado real contra el puerto).
    #[tokio::test]
    async fn quota_verdict_transitions_from_within_to_crossed() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = UsageRepository::new(&pool, &clock);

        // Límite FREE real de plan-tier-quota: $10,000.00 * 1e8 = 1_000_000_000_000.
        let notional_limit = 1_000_000_000_000;

        // Primera operación: nocional $1,000.00 -- sigue dentro.
        let first = repo
            .record_operation(sample_input("2026-07", 100_000_000, 100_000_000_000, notional_limit))
            .await
            .expect("primera operación");
        assert_eq!(first.quota_verdict, QuotaVerdict::Within);

        // Segunda operación: nocional $100,000.00 -- el acumulado ($101,000.00) cruza el límite ($10,000.00).
        let second = repo
            .record_operation(sample_input("2026-07", 250_000_000, 4_000_000_000_000, notional_limit))
            .await
            .expect("segunda operación");
        assert_eq!(second.quota_verdict, QuotaVerdict::Crossed, "el acumulado debe haber cruzado el límite FREE");
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
        let repo = UsageRepository::new(&pool, &clock);

        let notional_limit = 1_000_000_000_000_000;
        let first = repo
            .record_operation(sample_input("2026-07", 100_000_000, 100_000_000_000, notional_limit))
            .await
            .expect("primera operación (génesis)");
        clock.tick();
        let second = repo
            .record_operation(sample_input("2026-07", 100_000_000, 100_000_000_000, notional_limit))
            .await
            .expect("segunda operación");
        clock.tick();
        let third = repo
            .record_operation(sample_input("2026-07", 100_000_000, 100_000_000_000, notional_limit))
            .await
            .expect("tercera operación");

        assert_eq!(first.audit_chain_hash, None, "la fila génesis debe tener audit_chain_hash NULL");
        assert_eq!(
            second.audit_chain_hash,
            Some(first.audit_hash.clone()),
            "la segunda fila debe encadenar al audit_hash de la primera"
        );
        assert_eq!(
            third.audit_chain_hash,
            Some(second.audit_hash.clone()),
            "la tercera fila debe encadenar al audit_hash de la segunda"
        );
    }

    // ── CHECK de quota_verdict en la BD ──────────────────────────────────────

    #[tokio::test]
    async fn database_check_rejects_unknown_quota_verdict() {
        let pool = migrated_pool().await;

        let result = sqlx::query(
            "INSERT INTO usage_records (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id, \
                owner_id, institutional_tag, node_id, compliance_status_id, \
                notional_per_op, cycle_accumulated, billing_cycle_id, instrument_id, quota_verdict\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', 'DRASUS_LOCAL', 'node-1', NULL, \
                       0, 0, '2026-07', 'BTCUSDT', 'UNKNOWN')",
        )
        .execute(&pool)
        .await;

        assert!(result.is_err(), "un quota_verdict fuera de ('WITHIN','CROSSED') debe ser rechazado por el CHECK de la BD");
    }

    /// Propaga `NotionalError` sin persistir nada si el input es inválido
    /// (tamaño negativo) -- ninguna fila debe llegar a disco.
    #[tokio::test]
    async fn record_operation_propagates_notional_error_without_persisting() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = UsageRepository::new(&pool, &clock);

        let result = repo
            .record_operation(sample_input("2026-07", -1, 100_000_000, 1_000_000_000_000))
            .await;
        assert!(matches!(result, Err(UsageRepositoryError::Notional(NotionalError::NegativeSize))));

        let count: i64 = sqlx::query("SELECT COUNT(*) FROM usage_records")
            .fetch_one(&pool)
            .await
            .expect("contar filas")
            .get(0);
        assert_eq!(count, 0, "ninguna fila debe persistirse si el nocional falla");
    }
}
