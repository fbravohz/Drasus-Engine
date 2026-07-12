//! [SHELL] Composición del cimiento #13 (`docs/features/data-portability.md`,
//! ADR-0148, ADR-0093, ADR-0141, STORY-043).
//!
//! Capa delgada sobre [`crate::persistence::data_portability`]: traduce las
//! operaciones que el resto del substrato necesita -- "declara esta tabla en
//! el catálogo", "siembra el catálogo conocido", "un titular pide exportar
//! sus datos" y "un titular pide el olvido" -- sin que el llamador tenga que
//! conocer los repositorios ni el esquema de las tablas. Mismo rol que
//! `orchestrator::master_account_hierarchy` para el cimiento #12.
//!
//! **El generador de archivo real (recorrer el esquema y volcar el dato) y
//! la UI quedan diferidos** (STORY-043 §1): este módulo arma el MANIFIESTO
//! (qué tablas aplican) y el DETALLE DE DISPOSICIÓN (qué le pasa a cada
//! tabla), pero no vuelca ni un byte de dato real de ninguna tabla ajena --
//! eso lo hace el adaptador diferido sobre este mismo puerto.

use sqlx::SqlitePool;

use crate::domain::clock::Clock;
use crate::domain::data_portability::{
    build_export_manifest, build_forget_disposition_detail, disposition_detail_to_json, CatalogEntry,
    ExportManifest, RequestStatus, RequestType, TableDispositionEntry,
};
use crate::persistence::data_portability::{
    DataPortabilityRequestRepository, DataPortabilityRequestRepositoryError, DataPortabilityRequestRow,
    ExportableDataCatalogRepository, ExportableDataCatalogRepositoryError, NewCatalogEntry,
    RecordDataPortabilityRequestInput,
};

/// Error de orquestación de esta feature -- envuelve los dos puntos de
/// fallo posibles: el catálogo (declarar/sembrar/cargar) y el ledger de
/// solicitudes (registrar un evento).
#[derive(Debug, thiserror::Error)]
pub enum DataPortabilityError {
    #[error("error en el catálogo de tablas exportables: {0}")]
    Catalog(#[from] ExportableDataCatalogRepositoryError),
    #[error("error al registrar la solicitud de portabilidad: {0}")]
    Request(#[from] DataPortabilityRequestRepositoryError),
}

/// La identidad Grupo II/IV que acompaña cualquier solicitud de
/// portabilidad -- mismo rol que `InstanceContinuityIdentity` (#11):
/// `owner_id`/`institutional_tag` SIEMPRE salen de `central-identity` (#1),
/// nunca se inventan sueltos.
#[derive(Debug, Clone)]
pub struct DataPortabilityIdentity {
    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,
}

/// Declara una tabla nueva en el catálogo -- delgado a propósito: existe
/// como punto de orquestación estable para que `public_interface` no
/// dependa directamente del repositorio. IDEMPOTENTE (ver
/// `ExportableDataCatalogRepository::declare_table`): cualquier Feature
/// puede llamar esto en su propio arranque sin arriesgar duplicados.
pub async fn declare_exportable_table(
    pool: &SqlitePool,
    clock: &dyn Clock,
    table_name: &str,
    feature_name: &str,
    owner_id_column: &str,
    retention_exempt: bool,
) -> Result<(), DataPortabilityError> {
    let repo = ExportableDataCatalogRepository::new(pool, clock);
    repo.declare_table(NewCatalogEntry {
        table_name: table_name.to_string(),
        feature_name: feature_name.to_string(),
        owner_id_column: owner_id_column.to_string(),
        retention_exempt,
    })
    .await?;
    Ok(())
}

/// Las tablas del substrato que YA portan `owner_id`, con su clasificación
/// de retención -- stub que demuestra el mecanismo de auto-declaración
/// (STORY-043 §6: "equivalente a `seed_default_catalog` de #3"). NO es un
/// recorrido real del esquema (eso es el adaptador diferido); es la lista
/// conocida en el momento de escribir este cimiento. Las tres marcadas
/// `retention_exempt = true` son ledgers de auditoría/cumplimiento con
/// obligación de retención legal (STORY-043 §6): `audit_events`
/// (audit-log), `usage_records` (#4 usage-metering) y
/// `attested_track_records` (#10 verified-account-registry).
const KNOWN_CATALOG_TABLES: [(&str, &str, &str, bool); 19] = [
    ("accounts", "central-identity", "owner_id", false),
    ("licenses", "licensing-system", "owner_id", false),
    ("plans", "plan-tier-quota", "owner_id", false),
    ("usage_records", "usage-metering", "owner_id", true),
    ("consent_records", "consent-registry", "owner_id", false),
    ("domain_events", "enriched-domain-events", "owner_id", false),
    ("generated_reports", "institutional-report-engine", "owner_id", false),
    ("api_credentials", "third-party-api-gateway", "owner_id", false),
    ("api_usage_records", "third-party-api-gateway", "owner_id", false),
    ("aggregated_indexes", "data-aggregation", "owner_id", false),
    ("verified_accounts", "verified-account-registry", "owner_id", false),
    ("attested_track_records", "verified-account-registry", "owner_id", true),
    ("instance_backups", "instance-continuity", "owner_id", false),
    ("custody_state", "instance-continuity", "owner_id", false),
    ("account_hierarchy", "master-account-hierarchy", "owner_id", false),
    ("override_attestations", "master-account-hierarchy", "owner_id", false),
    ("audit_events", "audit-log", "owner_id", true),
    ("jobs", "async-job-executor", "owner_id", false),
    ("permission_decisions", "agentic-mcp-gateway", "owner_id", false),
];

/// Siembra el catálogo con [`KNOWN_CATALOG_TABLES`] -- IDEMPOTENTE (cada
/// tabla se declara vía [`declare_exportable_table`], que no duplica si ya
/// existe). Análogo de `plan_tier_quota::seed_default_catalog` /
/// `consent_registry::seed_default_catalog`: demuestra el mecanismo con las
/// tablas ya conocidas del substrato, sin recorrer el esquema real.
pub async fn seed_known_catalog(pool: &SqlitePool, clock: &dyn Clock) -> Result<(), DataPortabilityError> {
    for (table_name, feature_name, owner_id_column, retention_exempt) in KNOWN_CATALOG_TABLES {
        declare_exportable_table(pool, clock, table_name, feature_name, owner_id_column, retention_exempt).await?;
    }
    Ok(())
}

/// Carga TODO el catálogo declarado y lo proyecta al tipo de dominio
/// [`CatalogEntry`] que consume el Core -- paso común de
/// [`request_export`]/[`request_forget`].
async fn load_catalog_entries(pool: &SqlitePool, clock: &dyn Clock) -> Result<Vec<CatalogEntry>, DataPortabilityError> {
    let repo = ExportableDataCatalogRepository::new(pool, clock);
    let rows = repo.load_all().await?;
    Ok(rows.iter().map(CatalogEntry::from).collect())
}

/// Resultado de [`request_export`]: la fila registrada (evento `RECEIVED`)
/// más el manifiesto de exportación resuelto en el mismo momento.
#[derive(Debug, Clone)]
pub struct ExportRequestResult {
    pub request: DataPortabilityRequestRow,
    pub manifest: ExportManifest,
}

/// Un titular pide exportar sus datos (Art. 15/20 GDPR): carga el catálogo
/// vigente, arma el manifiesto vía el Core ([`build_export_manifest`],
/// filtrando SIEMPRE los secretos) y registra el evento `RECEIVED` --
/// append-only atómico, con un `request_group_id` fresco (una solicitud
/// lógica nueva).
pub async fn request_export(
    pool: &SqlitePool,
    clock: &dyn Clock,
    identity: &DataPortabilityIdentity,
) -> Result<ExportRequestResult, DataPortabilityError> {
    let catalog_entries = load_catalog_entries(pool, clock).await?;
    let manifest = build_export_manifest(&identity.owner_id, &catalog_entries);

    let request_group_id = uuid::Uuid::now_v7().to_string();
    let repo = DataPortabilityRequestRepository::new(pool, clock);
    let request = repo
        .record_event(RecordDataPortabilityRequestInput {
            owner_id: identity.owner_id.clone(),
            institutional_tag: identity.institutional_tag.clone(),
            node_id: identity.node_id.clone(),
            compliance_status_id: None,
            request_type: RequestType::Export,
            status: RequestStatus::Received,
            request_group_id,
            disposition_detail: None,
        })
        .await?;

    Ok(ExportRequestResult { request, manifest })
}

/// Resultado de [`request_forget`]: la fila registrada (evento `RECEIVED`)
/// más el detalle de disposición por tabla (para inspección directa, sin
/// tener que re-parsear el JSON persistido).
#[derive(Debug, Clone)]
pub struct ForgetRequestResult {
    pub request: DataPortabilityRequestRow,
    pub disposition: Vec<TableDispositionEntry>,
}

/// Un titular pide el olvido (Art. 17 GDPR): carga el catálogo vigente,
/// aplica `decide_forget_disposition` a CADA tabla vía el Core
/// ([`build_forget_disposition_detail`] -- SIEMPRE pseudonimización, NUNCA
/// DELETE, ADR-0141) y registra el evento `RECEIVED` con el detalle
/// serializado en `disposition_detail` -- append-only atómico, con un
/// `request_group_id` fresco. El recorrido/pseudonimización REAL del dato
/// queda diferido (adaptador); aquí se registra la decisión auditable.
pub async fn request_forget(
    pool: &SqlitePool,
    clock: &dyn Clock,
    identity: &DataPortabilityIdentity,
) -> Result<ForgetRequestResult, DataPortabilityError> {
    let catalog_entries = load_catalog_entries(pool, clock).await?;
    let disposition = build_forget_disposition_detail(&catalog_entries);
    let disposition_json = disposition_detail_to_json(&disposition);

    let request_group_id = uuid::Uuid::now_v7().to_string();
    let repo = DataPortabilityRequestRepository::new(pool, clock);
    let request = repo
        .record_event(RecordDataPortabilityRequestInput {
            owner_id: identity.owner_id.clone(),
            institutional_tag: identity.institutional_tag.clone(),
            node_id: identity.node_id.clone(),
            compliance_status_id: None,
            request_type: RequestType::Forget,
            status: RequestStatus::Received,
            request_group_id,
            disposition_detail: Some(disposition_json),
        })
        .await?;

    Ok(ForgetRequestResult { request, disposition })
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

    fn sample_identity(owner_id: &str) -> DataPortabilityIdentity {
        DataPortabilityIdentity {
            owner_id: owner_id.to_string(),
            institutional_tag: "LIVE".to_string(),
            node_id: "node-A".to_string(),
        }
    }

    // ── seed_known_catalog: idempotente, cubre las tres tablas exentas ─────

    #[tokio::test]
    async fn seed_known_catalog_declares_all_known_tables_and_is_idempotent() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);

        seed_known_catalog(&pool, &clock).await.expect("sembrar catálogo");
        clock.tick();
        seed_known_catalog(&pool, &clock).await.expect("sembrar de nuevo, no debe fallar ni duplicar");

        let repo = ExportableDataCatalogRepository::new(&pool, &clock);
        let all = repo.load_all().await.expect("cargar catálogo");
        assert_eq!(all.len(), KNOWN_CATALOG_TABLES.len(), "sembrar dos veces no debe duplicar ninguna fila");

        let exempt: Vec<&str> =
            all.iter().filter(|row| row.retention_exempt).map(|row| row.table_name.as_str()).collect();
        let mut exempt_sorted = exempt.clone();
        exempt_sorted.sort_unstable();
        assert_eq!(
            exempt_sorted,
            vec!["attested_track_records", "audit_events", "usage_records"],
            "exactamente las tres tablas de retención legal declaradas por STORY-043 §6"
        );
    }

    // ── request_export: manifiesto + evento RECEIVED ────────────────────────

    #[tokio::test]
    async fn request_export_records_received_event_and_excludes_secret_tables_from_manifest() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        seed_known_catalog(&pool, &clock).await.expect("sembrar catálogo");

        let result = request_export(&pool, &clock, &sample_identity(&owner_id)).await.expect("solicitar export");

        assert_eq!(result.request.request_type, RequestType::Export);
        assert_eq!(result.request.status, RequestStatus::Received);
        assert_eq!(result.request.disposition_detail, None, "un EXPORT no lleva disposition_detail");
        assert_eq!(result.manifest.owner_id, owner_id);

        let table_names: Vec<&str> = result.manifest.tables.iter().map(|t| t.table_name.as_str()).collect();
        assert!(
            !table_names.contains(&"api_credentials"),
            "api_credentials porta secretos (ADR-0093) -- nunca debe entrar al manifiesto"
        );
        assert!(table_names.contains(&"verified_accounts"), "una tabla normal sí debe entrar al manifiesto");

        // El estado vigente de la solicitud recién creada es RECEIVED.
        let request_repo = DataPortabilityRequestRepository::new(&pool, &clock);
        let status = request_repo
            .latest_status_for(&result.request.request_group_id)
            .await
            .expect("consultar estado vigente");
        assert_eq!(status, Some(RequestStatus::Received));
    }

    // ── request_forget: disposición por tabla, NUNCA delete ─────────────────

    /// CRITERIO DE CIERRE: un FORGET produce un `disposition_detail`
    /// coherente con `retention_exempt` de cada tabla sembrada -- las tres
    /// tablas de retención legal quedan `PSEUDONYMIZE_AND_RETAIN`, el resto
    /// `PSEUDONYMIZE_AND_PURGE`, NINGUNA con una variante de borrado.
    #[tokio::test]
    async fn request_forget_records_disposition_consistent_with_retention_flags() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        seed_known_catalog(&pool, &clock).await.expect("sembrar catálogo");

        let result = request_forget(&pool, &clock, &sample_identity(&owner_id)).await.expect("solicitar olvido");

        assert_eq!(result.request.request_type, RequestType::Forget);
        assert_eq!(result.request.status, RequestStatus::Received);
        assert_eq!(result.disposition.len(), KNOWN_CATALOG_TABLES.len());

        let retained: Vec<&str> = result
            .disposition
            .iter()
            .filter(|entry| entry.disposition == crate::domain::data_portability::ForgetDisposition::PseudonymizeAndRetain)
            .map(|entry| entry.table_name.as_str())
            .collect();
        let mut retained_sorted = retained.clone();
        retained_sorted.sort_unstable();
        assert_eq!(retained_sorted, vec!["attested_track_records", "audit_events", "usage_records"]);

        // El JSON persistido debe reflejar EXACTAMENTE el mismo detalle.
        let stored_json = result.request.disposition_detail.as_deref().expect("debe llevar disposition_detail");
        let parsed: serde_json::Value = serde_json::from_str(stored_json).expect("debe ser JSON válido");
        assert_eq!(parsed.as_array().expect("debe ser array").len(), KNOWN_CATALOG_TABLES.len());
    }

    /// Dos solicitudes (una EXPORT, una FORGET) del mismo `owner_id`
    /// producen DOS `request_group_id` distintos -- cada solicitud lógica
    /// es independiente, aunque comparta titular.
    #[tokio::test]
    async fn export_and_forget_requests_get_independent_request_group_ids() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "owner1@example.com").await;
        seed_known_catalog(&pool, &clock).await.expect("sembrar catálogo");

        let export_result = request_export(&pool, &clock, &sample_identity(&owner_id)).await.expect("export");
        clock.tick();
        let forget_result = request_forget(&pool, &clock, &sample_identity(&owner_id)).await.expect("forget");

        assert_ne!(export_result.request.request_group_id, forget_result.request.request_group_id);

        let repo = DataPortabilityRequestRepository::new(&pool, &clock);
        let chain = repo.load_chain().await.expect("cargar cadena");
        assert_eq!(chain.len(), 2, "dos solicitudes lógicas distintas -> dos eventos en la cadena global");
    }
}
