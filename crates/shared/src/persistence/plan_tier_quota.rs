//! [SHELL] Repositorio de persistencia para Plan / Tier / Quota
//! (`docs/features/plan-tier-quota.md`, ADR-0143, ADR-0144, ADR-0141,
//! ADR-0020, migración `0009_plan_tier_quota.sql`, STORY-029).
//!
//! Envuelve la tabla `plans`. Dueño del único I/O para el catálogo de
//! planes: lecturas/escrituras en SQLite, generación de UUIDv7 (ADR-0141) y
//! la lectura del puerto [`Clock`]. La lógica pura (validación de
//! coherencia, resolución de límites, hash de auditoría encadenado) vive en
//! [`crate::domain::plan_tier_quota`] -- este módulo solo le da entradas
//! inyectadas y persiste/carga el resultado, reflejando el patrón de
//! [`crate::persistence::licensing_system::LicenseRepository`].
//!
//! ## `row_version` en vez de `event_sequence_id` (ADR-0141)
//!
//! `plans` es una tabla MUTABLE (un plan cambia límite/precio en sitio) --
//! por eso [`PlanRepository::update_limits`] incrementa `row_version` en
//! vez de generar una posición en una secuencia global, exactamente como
//! [`crate::persistence::licensing_system::LicenseRepository::refresh_heartbeat`].

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::clock::Clock;
use crate::domain::plan_tier_quota::{
    canonical_features_json, compute_plan_audit_hash, decode_features_json, validate_plan,
    PlanCandidate, PlanTier, PlanValidationError, PricingModel,
};

/// Errores que devuelven las operaciones de [`PlanRepository`].
#[derive(Debug, thiserror::Error)]
pub enum PlanRepositoryError {
    /// La operación subyacente de SQLite falló.
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// El candidato de plan no pasó [`validate_plan`] -- ninguna escritura
    /// ocurre si esto falla (guarda de la Shell antes de tocar disco).
    #[error("plan inválido: {0}")]
    InvalidPlan(#[from] PlanValidationError),
    /// Una fila de `plans` tenía un valor de `tier` fuera de las dos
    /// cadenas canónicas -- error de integridad de datos.
    #[error("tier desconocido en la tabla plans: '{0}'")]
    UnknownTier(String),
    /// Una fila de `plans` tenía un valor de `pricing_model` fuera de las
    /// dos cadenas canónicas -- error de integridad de datos.
    #[error("pricing_model desconocido en la tabla plans: '{0}'")]
    UnknownPricingModel(String),
    /// Una fila de `plans` tenía `features_enabled` con JSON inválido --
    /// no debería ocurrir dado el `CHECK (json_valid(...))` de la
    /// migración, pero se maneja explícitamente en vez de entrar en pánico.
    #[error("features_enabled no es JSON válido en la fila '{0}' de plans")]
    MalformedFeaturesJson(String),
    /// Concurrencia optimista (ADR-0141): el UPDATE partió de un
    /// `row_version` que ya no es el vigente en disco -- otra escritura
    /// actualizó la fila en el ínterin. La operación NO pisa el cambio
    /// ajeno; quien llama debe releer la fila y reintentar.
    #[error("conflicto de versión en el plan '{id}': se esperaba row_version {expected}, la fila ya avanzó")]
    VersionConflict { id: String, expected: i64 },
}

/// Un plan nuevo para persistir -- `docs/features/plan-tier-quota.md`
/// "Ciclo de Vida": "Entrada": "Definición de plan (tier, cuotas, precio)
/// desde la Cabina de Mando" (o, en esta Story, desde el stub local de
/// desarrollo).
#[derive(Debug, Clone)]
pub struct NewPlan {
    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,
    pub tier: PlanTier,
    pub notional_limit: i64,
    pub max_activations: i64,
    pub price: i64,
    pub pricing_model: PricingModel,
    pub features_enabled: Vec<String>,
}

/// Una fila de plan persistida (tabla `plans`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub row_version: i64,

    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,

    pub tier: PlanTier,
    pub notional_limit: i64,
    pub max_activations: i64,
    pub price: i64,
    pub pricing_model: PricingModel,
    pub features_enabled: Vec<String>,
}

/// Repositorio para `plans`.
///
/// Constrúyelo con un [`SqlitePool`] ya migrado y cualquier implementación
/// de [`Clock`] -- mismo patrón que
/// [`crate::persistence::licensing_system::LicenseRepository`].
pub struct PlanRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> PlanRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`. Ambos se toman
    /// prestados por la vida del repositorio -- no se toma ownership.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Valida y persiste un plan nuevo con `row_version = 1`.
    ///
    /// Llama a [`validate_plan`] ANTES de tocar disco -- ninguna fila
    /// inválida ("sin tier", "sin cuota", montos negativos) llega siquiera
    /// a construir el INSERT (`docs/features/plan-tier-quota.md`
    /// "Restricciones").
    pub async fn create(&self, new_plan: NewPlan) -> Result<Plan, PlanRepositoryError> {
        validate_plan(&PlanCandidate {
            tier: Some(new_plan.tier),
            notional_limit: new_plan.notional_limit,
            max_activations: new_plan.max_activations,
            price: new_plan.price,
            pricing_model: new_plan.pricing_model,
            features_enabled: &new_plan.features_enabled,
        })?;

        let id = Uuid::now_v7().to_string();
        let now_ns = self.clock.timestamp_ns();
        let row_version: i64 = 1;
        let features_json = canonical_features_json(&new_plan.features_enabled);
        // Decodifica de vuelta la forma canónica (ordenada, sin duplicados)
        // -- el `Plan` devuelto debe reflejar EXACTAMENTE lo que quedó
        // persistido, no la lista cruda de entrada (que puede venir
        // desordenada o con duplicados).
        let canonical_features = decode_features_json(&features_json)
            .expect("canonical_features_json siempre produce JSON que decode_features_json puede leer");

        let audit_hash = compute_plan_audit_hash(
            &id,
            now_ns,
            row_version,
            None,
            &new_plan.owner_id,
            new_plan.tier,
            new_plan.notional_limit,
            new_plan.max_activations,
            new_plan.price,
            new_plan.pricing_model,
            &features_json,
        );

        sqlx::query(
            "INSERT INTO plans (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                owner_id, institutional_tag, node_id, \
                tier, notional_limit, max_activations, price, pricing_model, features_enabled\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(Option::<String>::None)
        .bind(row_version)
        .bind(&new_plan.owner_id)
        .bind(&new_plan.institutional_tag)
        .bind(&new_plan.node_id)
        .bind(new_plan.tier.as_str())
        .bind(new_plan.notional_limit)
        .bind(new_plan.max_activations)
        .bind(new_plan.price)
        .bind(new_plan.pricing_model.as_str())
        .bind(&features_json)
        .execute(self.pool)
        .await?;

        Ok(Plan {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: None,
            row_version,
            owner_id: new_plan.owner_id,
            institutional_tag: new_plan.institutional_tag,
            node_id: new_plan.node_id,
            tier: new_plan.tier,
            notional_limit: new_plan.notional_limit,
            max_activations: new_plan.max_activations,
            price: new_plan.price,
            pricing_model: new_plan.pricing_model,
            features_enabled: canonical_features,
        })
    }

    /// Carga un plan por su `id`, o `None` si no existe.
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Plan>, PlanRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    owner_id, institutional_tag, node_id, \
                    tier, notional_limit, max_activations, price, pricing_model, features_enabled \
             FROM plans WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        row.map(row_to_plan).transpose()
    }

    /// Carga el plan MÁS RECIENTE (por `created_at`) de un tier dado --
    /// query path principal de `resolve_limits`
    /// (`docs/features/plan-tier-quota.md` "Ciclo de Vida" - "Proceso":
    /// "resuelve los límites aplicables a una licencia dada", partiendo del
    /// tier). Si el catálogo llega a tener varios planes para el mismo
    /// tier (ej. mensual vs. anual), "el más reciente" es el default
    /// documentado -- desambiguar por plan_id explícito es responsabilidad
    /// de quien llama, vía [`Self::find_by_id`].
    pub async fn find_latest_by_tier(&self, tier: PlanTier) -> Result<Option<Plan>, PlanRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    owner_id, institutional_tag, node_id, \
                    tier, notional_limit, max_activations, price, pricing_model, features_enabled \
             FROM plans WHERE tier = ? ORDER BY created_at DESC LIMIT 1",
        )
        .bind(tier.as_str())
        .fetch_optional(self.pool)
        .await?;

        row.map(row_to_plan).transpose()
    }

    /// Revisa los límites (nocional/activaciones) y el precio de un plan ya
    /// existente -- "Cuando se cambia el límite de un plan -> las licencias
    /// de ese plan lo reflejan en la siguiente revalidación"
    /// (plan-tier-quota.md "Comportamientos Observables").
    ///
    /// Vuelve a correr [`validate_plan`] sobre los valores NUEVOS antes de
    /// escribir -- una revisión no puede introducir la misma incoherencia
    /// que una creación ("sin cuota", montos negativos).
    ///
    /// ## Concurrencia optimista (ADR-0141)
    ///
    /// El UPDATE filtra por `id` **y** `row_version = <el de `plan`>`. Si
    /// otra revisión ya avanzó la fila desde que se leyó `plan`, el `WHERE`
    /// no encuentra ninguna fila (`rows_affected() == 0`) y se devuelve
    /// [`PlanRepositoryError::VersionConflict`] en vez de pisar el cambio
    /// ajeno -- mismo patrón que
    /// [`crate::persistence::licensing_system::LicenseRepository::refresh_heartbeat`].
    pub async fn update_limits(
        &self,
        plan: &Plan,
        new_notional_limit: i64,
        new_max_activations: i64,
        new_price: i64,
    ) -> Result<Plan, PlanRepositoryError> {
        validate_plan(&PlanCandidate {
            tier: Some(plan.tier),
            notional_limit: new_notional_limit,
            max_activations: new_max_activations,
            price: new_price,
            pricing_model: plan.pricing_model,
            features_enabled: &plan.features_enabled,
        })?;

        let now_ns = self.clock.timestamp_ns();
        let row_version = plan.row_version + 1;
        let features_json = canonical_features_json(&plan.features_enabled);

        let audit_hash = compute_plan_audit_hash(
            &plan.id,
            now_ns,
            row_version,
            Some(&plan.audit_hash),
            &plan.owner_id,
            plan.tier,
            new_notional_limit,
            new_max_activations,
            new_price,
            plan.pricing_model,
            &features_json,
        );

        // La guarda `row_version = ?` es la comparación optimista: solo
        // actualiza si la fila en disco sigue en la versión que leímos.
        let result = sqlx::query(
            "UPDATE plans SET \
                updated_at = ?, audit_hash = ?, audit_chain_hash = ?, row_version = ?, \
                notional_limit = ?, max_activations = ?, price = ? \
             WHERE id = ? AND row_version = ?",
        )
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&plan.audit_hash)
        .bind(row_version)
        .bind(new_notional_limit)
        .bind(new_max_activations)
        .bind(new_price)
        .bind(&plan.id)
        .bind(plan.row_version)
        .execute(self.pool)
        .await?;

        // Cero filas afectadas => la fila ya no está en `plan.row_version`
        // (otra revisión la adelantó). No pisamos: reportamos el conflicto.
        if result.rows_affected() == 0 {
            return Err(PlanRepositoryError::VersionConflict {
                id: plan.id.clone(),
                expected: plan.row_version,
            });
        }

        Ok(Plan {
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: Some(plan.audit_hash.clone()),
            row_version,
            notional_limit: new_notional_limit,
            max_activations: new_max_activations,
            price: new_price,
            ..plan.clone()
        })
    }
}

/// Convierte una fila de `plans` al tipo [`Plan`].
fn row_to_plan(row: sqlx::sqlite::SqliteRow) -> Result<Plan, PlanRepositoryError> {
    let tier_value: String = row.get("tier");
    let tier = PlanTier::from_str_value(&tier_value).ok_or(PlanRepositoryError::UnknownTier(tier_value))?;

    let pricing_model_value: String = row.get("pricing_model");
    let pricing_model = PricingModel::from_str_value(&pricing_model_value)
        .ok_or(PlanRepositoryError::UnknownPricingModel(pricing_model_value))?;

    let id: String = row.get("id");
    let features_json: String = row.get("features_enabled");
    let features_enabled =
        decode_features_json(&features_json).ok_or_else(|| PlanRepositoryError::MalformedFeaturesJson(id.clone()))?;

    Ok(Plan {
        id,
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        row_version: row.get("row_version"),
        owner_id: row.get("owner_id"),
        institutional_tag: row.get("institutional_tag"),
        node_id: row.get("node_id"),
        tier,
        notional_limit: row.get("notional_limit"),
        max_activations: row.get("max_activations"),
        price: row.get("price"),
        pricing_model,
        features_enabled,
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

    fn sample_free_plan() -> NewPlan {
        NewPlan {
            owner_id: "drasus-system".to_string(),
            institutional_tag: "DRASUS_LOCAL".to_string(),
            node_id: "seed-node".to_string(),
            tier: PlanTier::Free,
            notional_limit: 1_000_000_000_000, // $10,000.00 * 1e8
            max_activations: 1,
            price: 0,
            pricing_model: PricingModel::Flat,
            features_enabled: vec![],
        }
    }

    fn sample_paid_plan() -> NewPlan {
        NewPlan {
            owner_id: "drasus-system".to_string(),
            institutional_tag: "DRASUS_LOCAL".to_string(),
            node_id: "seed-node".to_string(),
            tier: PlanTier::Paid,
            notional_limit: 100_000_000_000_000, // $1,000,000.00 * 1e8
            max_activations: 3,
            price: 4_900_000_000, // $49.00 * 1e8
            pricing_model: PricingModel::Flat,
            features_enabled: vec!["priority_support".to_string()],
        }
    }

    // ── CRITERIO #1 (Orden §5): esquema STRICT + Grupo I + Perfil D + row_version ──

    #[tokio::test]
    async fn migration_creates_plans_table_with_group_i_profile_d_and_row_version() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('plans')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info de plans");
        let column_names: Vec<String> = columns.iter().map(|row| row.get::<String, _>(0)).collect();

        for expected in [
            "id",
            "created_at",
            "updated_at",
            "audit_hash",
            "audit_chain_hash",
            "row_version",
            "owner_id",
            "institutional_tag",
            "node_id",
            "tier",
            "notional_limit",
            "max_activations",
            "price",
            "pricing_model",
            "features_enabled",
        ] {
            assert!(column_names.contains(&expected.to_string()), "falta la columna: {expected}");
        }

        assert!(
            !column_names.contains(&"event_sequence_id".to_string()),
            "plans es una tabla MUTABLE (ADR-0141): no debe tener event_sequence_id, solo row_version"
        );

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'plans'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(sql.contains("STRICT"), "la tabla plans debe declararse STRICT");
    }

    // ── CRITERIO #2 (Orden §5): montos INTEGER ×10⁸, round-trip exacto ──────

    /// CRITERIO DE CIERRE: un monto grande (equivalente a $10,000.00
    /// escalado ×10⁸) se persiste y se relee EXACTO -- sin la deriva de
    /// punto flotante que produciría una columna `REAL`. La inspección de
    /// tipo de columna confirma además que `pragma_table_info` reporta
    /// `INTEGER`, no `REAL`.
    #[tokio::test]
    async fn amounts_persist_and_reload_as_exact_integers_never_real() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = PlanRepository::new(&pool, &clock);

        let plan = repo.create(sample_free_plan()).await.expect("crear plan");
        assert_eq!(plan.notional_limit, 1_000_000_000_000);

        let reloaded = repo.find_by_id(&plan.id).await.expect("releer").expect("debe existir");
        assert_eq!(reloaded.notional_limit, 1_000_000_000_000, "round-trip debe ser exacto, sin deriva de float");
        assert_eq!(reloaded.price, 0);

        let columns = sqlx::query("SELECT name, type FROM pragma_table_info('plans')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info");
        for row in columns {
            let name: String = row.get("name");
            if name == "notional_limit" || name == "price" || name == "max_activations" {
                let column_type: String = row.get("type");
                assert_eq!(column_type, "INTEGER", "la columna '{name}' nunca debe ser REAL");
            }
        }
    }

    // ── CRITERIO #3 (Orden §5, a nivel Shell): create rechaza plan inválido ──

    #[tokio::test]
    async fn create_rejects_plan_without_any_quota() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = PlanRepository::new(&pool, &clock);

        let mut invalid = sample_free_plan();
        invalid.notional_limit = 0;
        invalid.max_activations = 0;

        let result = repo.create(invalid).await;
        assert!(matches!(result, Err(PlanRepositoryError::InvalidPlan(_))));

        // Nada se persistió -- ninguna fila inválida llega a disco.
        let count: i64 = sqlx::query("SELECT COUNT(*) FROM plans")
            .fetch_one(&pool)
            .await
            .expect("contar filas")
            .get(0);
        assert_eq!(count, 0);
    }

    // ── CRITERIO #4 (Orden §5): resolve_limits correcto por tier (vía repositorio) ──

    #[tokio::test]
    async fn find_latest_by_tier_returns_the_seeded_plan_for_each_tier() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = PlanRepository::new(&pool, &clock);

        repo.create(sample_free_plan()).await.expect("crear plan free");
        repo.create(sample_paid_plan()).await.expect("crear plan paid");

        let free = repo.find_latest_by_tier(PlanTier::Free).await.expect("buscar free").expect("debe existir");
        assert_eq!(free.tier, PlanTier::Free);
        assert_eq!(free.max_activations, 1);

        let paid = repo.find_latest_by_tier(PlanTier::Paid).await.expect("buscar paid").expect("debe existir");
        assert_eq!(paid.tier, PlanTier::Paid);
        assert_eq!(paid.max_activations, 3);
    }

    #[tokio::test]
    async fn find_latest_by_tier_returns_none_when_no_plan_exists() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = PlanRepository::new(&pool, &clock);

        let missing = repo.find_latest_by_tier(PlanTier::Paid).await.expect("buscar");
        assert_eq!(missing, None);
    }

    // ── CRITERIO #5 (Orden §5): cambio de límite reflejado + row_version incrementa ──

    /// CRITERIO DE CIERRE: actualizar el `notional_limit` de un plan hace
    /// que `find_by_id`/`find_latest_by_tier` reflejen el nuevo valor, y
    /// `row_version` incrementa -- si `update_limits` no persistiera el
    /// cambio, esta prueba fallaría comparando contra el valor viejo.
    #[tokio::test]
    async fn update_limits_reflects_new_notional_limit_and_increments_row_version() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = PlanRepository::new(&pool, &clock);

        let plan = repo.create(sample_free_plan()).await.expect("crear plan");
        assert_eq!(plan.row_version, 1);

        clock.tick();
        let updated = repo
            .update_limits(&plan, 2_000_000_000_000, 2, 0)
            .await
            .expect("revisar límites");

        assert_eq!(updated.row_version, 2);
        assert_eq!(updated.notional_limit, 2_000_000_000_000);
        assert_eq!(updated.max_activations, 2);
        assert_ne!(updated.audit_hash, plan.audit_hash);

        let reloaded = repo.find_by_id(&plan.id).await.expect("releer").expect("debe existir");
        assert_eq!(reloaded.row_version, 2);
        assert_eq!(reloaded.notional_limit, 2_000_000_000_000);

        let via_tier = repo
            .find_latest_by_tier(PlanTier::Free)
            .await
            .expect("buscar por tier")
            .expect("debe existir");
        assert_eq!(via_tier.notional_limit, 2_000_000_000_000);
    }

    /// La revisión también rechaza incoherencias -- no permite "arreglar"
    /// un plan quitándole toda cuota.
    #[tokio::test]
    async fn update_limits_rejects_removing_all_quota() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = PlanRepository::new(&pool, &clock);

        let plan = repo.create(sample_free_plan()).await.expect("crear plan");
        let result = repo.update_limits(&plan, 0, 0, 0).await;
        assert!(matches!(result, Err(PlanRepositoryError::InvalidPlan(_))));
    }

    // ── CRITERIO #6 (Orden §5): concurrencia optimista real en update ───────

    /// CRITERIO DE CIERRE (concurrencia optimista): dos revisiones que
    /// parten del MISMO `row_version` en memoria no pueden ambas tener
    /// éxito. El primero pasa; el segundo, que sigue creyendo estar en la
    /// versión vieja, devuelve `VersionConflict` (`rows_affected == 0`) en
    /// vez de pisar el cambio del primero en silencio.
    ///
    /// Esta prueba FALLA si se quita la guarda `AND row_version = ?` del
    /// UPDATE: sin ella, el segundo refresco también afectaría 1 fila y
    /// devolvería `Ok`, bifurcando la cadena `audit_hash`.
    #[tokio::test]
    async fn concurrent_updates_from_same_version_conflict_instead_of_overwriting() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = PlanRepository::new(&pool, &clock);

        let plan = repo.create(sample_free_plan()).await.expect("crear plan");
        assert_eq!(plan.row_version, 1);

        // Dos "actores" leyeron el MISMO plan en la versión 1.
        let first_writer_view = plan.clone();
        let second_writer_view = plan;

        clock.tick();
        let updated = repo
            .update_limits(&first_writer_view, 5_000_000_000_000, 1, 0)
            .await
            .expect("la primera revisión debe tener éxito");
        assert_eq!(updated.row_version, 2);

        clock.tick();
        let conflict = repo.update_limits(&second_writer_view, 9_000_000_000_000, 1, 0).await;
        assert!(
            matches!(conflict, Err(PlanRepositoryError::VersionConflict { expected: 1, .. })),
            "la segunda revisión desde la versión 1 debe dar VersionConflict, no éxito silencioso; fue: {conflict:?}"
        );

        // La fila en disco conserva el cambio del PRIMER writer, no el del segundo.
        let reloaded = repo.find_by_id(&updated.id).await.expect("releer").expect("existe");
        assert_eq!(reloaded.row_version, 2);
        assert_eq!(reloaded.notional_limit, 5_000_000_000_000);
    }

    // ── CRITERIO #7 (Orden §5): CHECK de tier/pricing_model ─────────────────

    /// CRITERIO DE CIERRE: un `tier` fuera de ('FREE','PAID') es rechazado
    /// por la BD (el `CHECK` de la migración), no por la capa Rust -- esta
    /// prueba inserta directamente con SQL crudo, sin pasar por
    /// [`PlanRepository::create`], para ejercitar el guardarraíl de la BD
    /// en sí mismo.
    #[tokio::test]
    async fn database_check_rejects_unknown_tier() {
        let pool = migrated_pool().await;

        let result = sqlx::query(
            "INSERT INTO plans (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                owner_id, institutional_tag, node_id, \
                tier, notional_limit, max_activations, price, pricing_model, features_enabled\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', 'DRASUS_LOCAL', 'node-1', \
                       'ENTERPRISE', 0, 1, 0, 'FLAT', '[]')",
        )
        .execute(&pool)
        .await;

        assert!(result.is_err(), "un tier fuera de ('FREE','PAID') debe ser rechazado por el CHECK de la BD");
    }

    #[tokio::test]
    async fn database_check_rejects_unknown_pricing_model() {
        let pool = migrated_pool().await;

        let result = sqlx::query(
            "INSERT INTO plans (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                owner_id, institutional_tag, node_id, \
                tier, notional_limit, max_activations, price, pricing_model, features_enabled\
            ) VALUES ('id-1', 0, 0, 'hash', NULL, 1, 'owner-1', 'DRASUS_LOCAL', 'node-1', \
                       'FREE', 0, 1, 0, 'SUBSCRIPTION', '[]')",
        )
        .execute(&pool)
        .await;

        assert!(result.is_err(), "un pricing_model fuera de ('FLAT','VOLUME') debe ser rechazado por el CHECK de la BD");
    }

    // ── Features habilitadas: persistencia determinista ─────────────────────

    #[tokio::test]
    async fn features_enabled_persist_sorted_and_deduplicated() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = PlanRepository::new(&pool, &clock);

        let mut plan_input = sample_paid_plan();
        plan_input.features_enabled =
            vec!["vps_headless".to_string(), "priority_support".to_string(), "priority_support".to_string()];

        let plan = repo.create(plan_input).await.expect("crear plan");
        assert_eq!(plan.features_enabled, vec!["priority_support".to_string(), "vps_headless".to_string()]);

        let reloaded = repo.find_by_id(&plan.id).await.expect("releer").expect("debe existir");
        assert_eq!(reloaded.features_enabled, vec!["priority_support".to_string(), "vps_headless".to_string()]);
    }
}
