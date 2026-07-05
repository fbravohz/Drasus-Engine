//! [SHELL] Repositorio de persistencia para Licensing System
//! (`docs/features/licensing-system.md`, ADR-0143, ADR-0144, ADR-0141,
//! ADR-0020, migración `0008_licensing_system.sql`, STORY-028).
//!
//! Envuelve la tabla `licenses`. Dueño del único I/O para activaciones de
//! licencia: lecturas/escrituras en SQLite, generación de UUIDv7 (ADR-0141)
//! y la lectura del puerto [`Clock`]. La lógica pura (verificación de firma,
//! comparación de huella, heartbeat/gracia, derivación del veredicto) vive
//! en [`crate::domain::licensing_system`] -- este módulo solo le da entradas
//! inyectadas y persiste/carga el resultado, reflejando el patrón de
//! [`crate::persistence::central_identity::AccountRepository`].
//!
//! ## `row_version` en vez de `event_sequence_id` (ADR-0141)
//!
//! `licenses` es una tabla MUTABLE (el heartbeat refresca la validez en
//! sitio) -- por eso [`LicenseRepository::refresh_heartbeat`] incrementa
//! `row_version` en vez de generar una posición en una secuencia global,
//! exactamente como `AccountRepository::update_email_verification_status`.

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::clock::Clock;
use crate::domain::licensing_system::{compute_license_audit_hash, LicenseTier};

/// Errores que devuelven las operaciones de [`LicenseRepository`].
#[derive(Debug, thiserror::Error)]
pub enum LicenseRepositoryError {
    /// La operación subyacente de SQLite falló.
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// Una fila de `licenses` tenía un valor de `tier` fuera de las dos
    /// cadenas canónicas -- error de integridad de datos.
    #[error("tier desconocido en la tabla licenses: '{0}'")]
    UnknownTier(String),
    /// Concurrencia optimista (ADR-0141): el refresco de heartbeat partió
    /// de un `row_version` que ya no es el vigente en disco -- otra
    /// escritura actualizó la fila en el ínterin. La operación NO pisa el
    /// cambio ajeno; quien llama debe releer la fila y reintentar.
    #[error("conflicto de versión en la licencia '{id}': se esperaba row_version {expected}, la fila ya avanzó")]
    VersionConflict { id: String, expected: i64 },
}

/// Una activación nueva para persistir -- la primera vez que un `owner_id`
/// (de `central-identity`) activa la licencia en una máquina concreta
/// (`node_id`, REUTILIZADO de `AccountIdentity.node_id`, nunca recalculado
/// aquí -- ADR-0144 FIJO).
#[derive(Debug, Clone)]
pub struct NewLicenseActivation {
    pub owner_id: String,
    pub institutional_tag: String,
    pub access_token_id: Option<String>,
    /// Huella de hardware de la instancia que activa -- viene de
    /// `AccountIdentity.node_id` (puerto `identity_in`).
    pub node_id: String,
    /// Identificador de la LICENCIA firmada (`SignedLicenseFile::license_id`)
    /// -- distinto del `id` de esta fila de activación. Varias activaciones
    /// (máquinas) del mismo dueño comparten el mismo `license_id`.
    pub license_id: String,
    pub process_id: Option<String>,
    /// Firma Ed25519 (hex) del archivo de licencia -- dato público
    /// verificable, NUNCA la clave privada (ADR-0093).
    pub signature_hash: String,
    pub tier: LicenseTier,
    /// Instante (ns UTC) en que el emisor firmó este payload -- parte del
    /// contenido firmado, se persiste tal cual para poder re-verificar la
    /// firma después (ver comentario de la columna `issued_at` en la
    /// migración).
    pub issued_at_ns: i64,
    pub heartbeat_expires_at_ns: i64,
    /// Estado de cumplimiento inicial (normalmente `"ACTIVE"` -- la
    /// licencia recién activada arranca vigente).
    pub compliance_status_id: String,
}

/// Una fila de licencia persistida (tabla `licenses`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenseRecord {
    pub id: String,
    pub created_at_ns: i64,
    pub updated_at_ns: i64,
    pub audit_hash: String,
    pub audit_chain_hash: Option<String>,
    pub row_version: i64,

    pub owner_id: String,
    pub institutional_tag: String,
    pub access_token_id: Option<String>,

    pub node_id: String,
    pub license_id: String,
    pub process_id: Option<String>,

    pub signature_hash: String,
    pub compliance_status_id: String,

    pub tier: LicenseTier,
    pub issued_at_ns: i64,
    pub heartbeat_expires_at_ns: i64,
}

/// Repositorio para `licenses`.
///
/// Constrúyelo con un [`SqlitePool`] ya migrado y cualquier implementación
/// de [`Clock`] -- mismo patrón que
/// [`crate::persistence::central_identity::AccountRepository`].
pub struct LicenseRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> LicenseRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`. Ambos se toman
    /// prestados por la vida del repositorio -- no se toma ownership.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Activa una licencia para `owner_id` en la máquina `node_id`.
    ///
    /// **Idempotencia (licensing-system.md §3, "Una sola instancia por
    /// máquina, FIJO"):** si ya existe una fila para (`owner_id`, `node_id`)
    /// -- el índice único `idx_licenses_owner_node` de la migración lo
    /// garantiza -- devuelve la fila EXISTENTE en vez de duplicarla. Un
    /// segundo arranque en la misma máquina reutiliza su activación, nunca
    /// cuenta como una segunda.
    pub async fn activate(
        &self,
        new_activation: NewLicenseActivation,
    ) -> Result<LicenseRecord, LicenseRepositoryError> {
        if let Some(existing) = self
            .find_by_owner_and_node(&new_activation.owner_id, &new_activation.node_id)
            .await?
        {
            return Ok(existing);
        }

        let id = Uuid::now_v7().to_string();
        let now_ns = self.clock.timestamp_ns();
        let row_version: i64 = 1;

        let audit_hash = compute_license_audit_hash(
            &id,
            now_ns,
            row_version,
            None,
            &new_activation.owner_id,
            &new_activation.node_id,
            new_activation.tier,
            new_activation.heartbeat_expires_at_ns,
            &new_activation.compliance_status_id,
            &new_activation.signature_hash,
        );

        sqlx::query(
            "INSERT INTO licenses (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                owner_id, institutional_tag, access_token_id, \
                node_id, license_id, process_id, \
                signature_hash, compliance_status_id, \
                tier, issued_at, heartbeat_expires_at\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(Option::<String>::None)
        .bind(row_version)
        .bind(&new_activation.owner_id)
        .bind(&new_activation.institutional_tag)
        .bind(&new_activation.access_token_id)
        .bind(&new_activation.node_id)
        .bind(&new_activation.license_id)
        .bind(&new_activation.process_id)
        .bind(&new_activation.signature_hash)
        .bind(&new_activation.compliance_status_id)
        .bind(new_activation.tier.as_str())
        .bind(new_activation.issued_at_ns)
        .bind(new_activation.heartbeat_expires_at_ns)
        .execute(self.pool)
        .await?;

        Ok(LicenseRecord {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: None,
            row_version,
            owner_id: new_activation.owner_id,
            institutional_tag: new_activation.institutional_tag,
            access_token_id: new_activation.access_token_id,
            node_id: new_activation.node_id,
            license_id: new_activation.license_id,
            process_id: new_activation.process_id,
            signature_hash: new_activation.signature_hash,
            compliance_status_id: new_activation.compliance_status_id,
            tier: new_activation.tier,
            issued_at_ns: new_activation.issued_at_ns,
            heartbeat_expires_at_ns: new_activation.heartbeat_expires_at_ns,
        })
    }

    /// Carga la activación de `owner_id` en `node_id`, o `None` si esa
    /// máquina nunca activó esta licencia.
    pub async fn find_by_owner_and_node(
        &self,
        owner_id: &str,
        node_id: &str,
    ) -> Result<Option<LicenseRecord>, LicenseRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    owner_id, institutional_tag, access_token_id, \
                    node_id, license_id, process_id, \
                    signature_hash, compliance_status_id, \
                    tier, issued_at, heartbeat_expires_at \
             FROM licenses WHERE owner_id = ? AND node_id = ?",
        )
        .bind(owner_id)
        .bind(node_id)
        .fetch_optional(self.pool)
        .await?;

        row.map(row_to_license).transpose()
    }

    /// Cuenta las **máquinas distintas** (activaciones) que tiene `owner_id`
    /// -- licensing-system.md §3: "las activaciones del tier cuentan
    /// máquinas distintas... no procesos". Como el índice único
    /// `(owner_id, node_id)` de la migración ya impide dos filas para la
    /// misma máquina, un `COUNT(*)` y un `COUNT(DISTINCT node_id)` dan lo
    /// mismo en la práctica -- se usa `DISTINCT` de todas formas para que la
    /// invariante quede explícita en la consulta, no solo en el esquema.
    pub async fn count_distinct_activations(
        &self,
        owner_id: &str,
    ) -> Result<i64, LicenseRepositoryError> {
        let count: i64 = sqlx::query("SELECT COUNT(DISTINCT node_id) FROM licenses WHERE owner_id = ?")
            .bind(owner_id)
            .fetch_one(self.pool)
            .await?
            .get(0);
        Ok(count)
    }

    /// Refresca el heartbeat de una licencia ya activada: fija un nuevo
    /// `heartbeat_expires_at`, `compliance_status_id` **y** `signature_hash`
    /// (el emisor firma un payload NUEVO cada vez que extiende el
    /// heartbeat -- la firma vieja solo cubría el `heartbeat_expires_at`
    /// anterior, ya no el nuevo), incrementa `row_version`, y encadena
    /// `audit_hash`/`audit_chain_hash`.
    ///
    /// `issued_at` NO cambia en un refresco de heartbeat -- sigue siendo el
    /// instante de la emisión ORIGINAL de la licencia; solo el heartbeat
    /// (la vigencia operativa) se extiende.
    ///
    /// ## Concurrencia optimista (ADR-0141)
    ///
    /// El UPDATE filtra por `id` **y** `row_version = <el de `license`>`. Si
    /// otro refresco ya avanzó la fila desde que se leyó `license`, el
    /// `WHERE` no encuentra ninguna fila (`rows_affected() == 0`) y se
    /// devuelve [`LicenseRepositoryError::VersionConflict`] en vez de pisar
    /// el cambio ajeno -- mismo patrón que
    /// [`crate::persistence::central_identity::AccountRepository::update_email_verification_status`].
    pub async fn refresh_heartbeat(
        &self,
        license: &LicenseRecord,
        new_heartbeat_expires_at_ns: i64,
        new_compliance_status_id: &str,
        new_signature_hash: &str,
    ) -> Result<LicenseRecord, LicenseRepositoryError> {
        let now_ns = self.clock.timestamp_ns();
        let row_version = license.row_version + 1;

        let audit_hash = compute_license_audit_hash(
            &license.id,
            now_ns,
            row_version,
            Some(&license.audit_hash),
            &license.owner_id,
            &license.node_id,
            license.tier,
            new_heartbeat_expires_at_ns,
            new_compliance_status_id,
            new_signature_hash,
        );

        // La guarda `row_version = ?` es la comparación optimista: solo
        // actualiza si la fila en disco sigue en la versión que leímos.
        let result = sqlx::query(
            "UPDATE licenses SET \
                updated_at = ?, audit_hash = ?, audit_chain_hash = ?, row_version = ?, \
                heartbeat_expires_at = ?, compliance_status_id = ?, signature_hash = ? \
             WHERE id = ? AND row_version = ?",
        )
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&license.audit_hash)
        .bind(row_version)
        .bind(new_heartbeat_expires_at_ns)
        .bind(new_compliance_status_id)
        .bind(new_signature_hash)
        .bind(&license.id)
        .bind(license.row_version)
        .execute(self.pool)
        .await?;

        // Cero filas afectadas => la fila ya no está en `license.row_version`
        // (otro refresco la adelantó). No pisamos: reportamos el conflicto.
        if result.rows_affected() == 0 {
            return Err(LicenseRepositoryError::VersionConflict {
                id: license.id.clone(),
                expected: license.row_version,
            });
        }

        Ok(LicenseRecord {
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: Some(license.audit_hash.clone()),
            row_version,
            heartbeat_expires_at_ns: new_heartbeat_expires_at_ns,
            compliance_status_id: new_compliance_status_id.to_string(),
            signature_hash: new_signature_hash.to_string(),
            ..license.clone()
        })
    }
}

/// Convierte una fila de `licenses` al tipo [`LicenseRecord`].
fn row_to_license(row: sqlx::sqlite::SqliteRow) -> Result<LicenseRecord, LicenseRepositoryError> {
    let tier_value: String = row.get("tier");
    let tier = LicenseTier::from_str_value(&tier_value)
        .ok_or(LicenseRepositoryError::UnknownTier(tier_value))?;

    Ok(LicenseRecord {
        id: row.get("id"),
        created_at_ns: row.get("created_at"),
        updated_at_ns: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        row_version: row.get("row_version"),
        owner_id: row.get("owner_id"),
        institutional_tag: row.get("institutional_tag"),
        access_token_id: row.get("access_token_id"),
        node_id: row.get("node_id"),
        license_id: row.get("license_id"),
        process_id: row.get("process_id"),
        signature_hash: row.get("signature_hash"),
        compliance_status_id: row.get("compliance_status_id"),
        issued_at_ns: row.get("issued_at"),
        tier,
        heartbeat_expires_at_ns: row.get("heartbeat_expires_at"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::central_identity::EmailVerificationStatus;
    use crate::domain::clock::DeterministicClock;
    use crate::persistence::central_identity::{AccountRepository, NewAccount};
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    /// Crea una cuenta real en `accounts` (la FK de `licenses.owner_id` la
    /// exige) y devuelve su `id` para usar como `owner_id` en los tests de
    /// licencias.
    async fn seed_account(pool: &SqlitePool, clock: &dyn Clock, email: &str) -> String {
        let repo = AccountRepository::new(pool, clock);
        let account = repo
            .create(NewAccount {
                email: email.to_string(),
                oauth_provider: None,
                institutional_tag: "DRASUS_LOCAL".to_string(),
                access_token_id: None,
                node_id: "seed-node".to_string(),
                owner_id: None,
            })
            .await
            .expect("crear cuenta semilla");
        assert_eq!(account.email_verification_status, EmailVerificationStatus::Pending);
        account.owner_id
    }

    fn sample_activation(owner_id: &str, node_id: &str) -> NewLicenseActivation {
        NewLicenseActivation {
            owner_id: owner_id.to_string(),
            institutional_tag: "DRASUS_LOCAL".to_string(),
            access_token_id: None,
            node_id: node_id.to_string(),
            license_id: "license-fixture-1".to_string(),
            process_id: Some("pid-1".to_string()),
            signature_hash: "deadbeef".to_string(),
            tier: LicenseTier::Sovereign,
            issued_at_ns: 1_000,
            heartbeat_expires_at_ns: 10_000,
            compliance_status_id: "ACTIVE".to_string(),
        }
    }

    // ── CRITERIO #1 (Orden §5): esquema STRICT + Grupo I + Perfil D + row_version ──

    #[tokio::test]
    async fn migration_creates_licenses_table_with_group_i_profile_d_and_row_version() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('licenses')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info de licenses");
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
            "access_token_id",
            "node_id",
            "license_id",
            "process_id",
            "signature_hash",
            "compliance_status_id",
            "tier",
            "issued_at",
            "heartbeat_expires_at",
        ] {
            assert!(column_names.contains(&expected.to_string()), "falta la columna: {expected}");
        }

        assert!(
            !column_names.contains(&"event_sequence_id".to_string()),
            "licenses es una tabla MUTABLE (ADR-0141): no debe tener event_sequence_id, solo row_version"
        );

        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'licenses'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(sql.contains("STRICT"), "la tabla licenses debe declararse STRICT");
    }

    // ── Activación + idempotencia por máquina ────────────────────────────────

    #[tokio::test]
    async fn activate_persists_a_new_license_row_with_row_version_one() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "user@example.com").await;
        let repo = LicenseRepository::new(&pool, &clock);

        let license = repo.activate(sample_activation(&owner_id, "node-A")).await.expect("activar licencia");
        assert_eq!(license.row_version, 1);
        assert_eq!(license.compliance_status_id, "ACTIVE");
        assert_eq!(license.audit_chain_hash, None);
    }

    /// CRITERIO DE CIERRE (Orden §5, criterio #6 -- "un segundo arranque con
    /// una huella ya vista sigue contando 3"): activar dos veces la MISMA
    /// máquina para el MISMO dueño no duplica la fila -- reutiliza la
    /// existente y `count_distinct_activations` no sube.
    #[tokio::test]
    async fn activating_the_same_machine_twice_is_idempotent() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "user@example.com").await;
        let repo = LicenseRepository::new(&pool, &clock);

        let first = repo.activate(sample_activation(&owner_id, "node-A")).await.expect("primera activación");
        let second = repo.activate(sample_activation(&owner_id, "node-A")).await.expect("segunda activación, misma máquina");

        assert_eq!(first.id, second.id, "reactivar la misma máquina debe reusar la fila, no crear otra");

        let count = repo.count_distinct_activations(&owner_id).await.expect("contar activaciones");
        assert_eq!(count, 1, "la misma máquina activada dos veces cuenta como UNA sola activación");
    }

    /// CRITERIO DE CIERRE (Orden §5, criterio #6 -- "3 huellas distintas ->
    /// 3 activaciones"): tres máquinas distintas para el mismo dueño cuentan
    /// tres activaciones.
    #[tokio::test]
    async fn three_distinct_machines_count_as_three_activations() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "user@example.com").await;
        let repo = LicenseRepository::new(&pool, &clock);

        repo.activate(sample_activation(&owner_id, "node-A")).await.expect("activar node-A");
        repo.activate(sample_activation(&owner_id, "node-B")).await.expect("activar node-B");
        repo.activate(sample_activation(&owner_id, "node-C")).await.expect("activar node-C");

        let count = repo.count_distinct_activations(&owner_id).await.expect("contar activaciones");
        assert_eq!(count, 3);

        // Un segundo arranque en una de las tres máquinas ya vistas NO sube el conteo.
        repo.activate(sample_activation(&owner_id, "node-B")).await.expect("reactivar node-B");
        let count_after_reboot = repo.count_distinct_activations(&owner_id).await.expect("contar de nuevo");
        assert_eq!(count_after_reboot, 3, "un segundo arranque en una máquina ya vista no debe sumar activación");
    }

    // ── CRITERIO #7 (Orden §5): concurrencia optimista real en refresco ─────

    #[tokio::test]
    async fn refresh_heartbeat_increments_row_version_and_persists() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "user@example.com").await;
        let repo = LicenseRepository::new(&pool, &clock);

        let license = repo.activate(sample_activation(&owner_id, "node-A")).await.expect("activar");
        clock.tick();
        let refreshed = repo
            .refresh_heartbeat(&license, 20_000, "ACTIVE", "resigned-hex-1")
            .await
            .expect("refrescar heartbeat");

        assert_eq!(refreshed.row_version, 2);
        assert_eq!(refreshed.heartbeat_expires_at_ns, 20_000);
        assert_eq!(refreshed.audit_chain_hash, Some(license.audit_hash.clone()));
        assert_ne!(refreshed.audit_hash, license.audit_hash);

        let reloaded = repo
            .find_by_owner_and_node(&owner_id, "node-A")
            .await
            .expect("releer")
            .expect("debe existir");
        assert_eq!(reloaded.row_version, 2);
        assert_eq!(reloaded.heartbeat_expires_at_ns, 20_000);
    }

    /// CRITERIO DE CIERRE (Orden §5, criterio #7 -- concurrencia optimista):
    /// dos refrescos que parten del MISMO `row_version` en memoria no pueden
    /// ambos tener éxito. El primero pasa; el segundo, que sigue creyendo
    /// estar en la versión vieja, devuelve `VersionConflict` (`rows_affected
    /// == 0`) en vez de pisar el cambio del primero en silencio.
    ///
    /// Esta prueba FALLA si se quita la guarda `AND row_version = ?` del
    /// UPDATE: sin ella, el segundo refresco también afectaría 1 fila y
    /// devolvería `Ok`, bifurcando la cadena `audit_hash`.
    #[tokio::test]
    async fn concurrent_heartbeat_refreshes_from_same_version_conflict_instead_of_overwriting() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "user@example.com").await;
        let repo = LicenseRepository::new(&pool, &clock);

        let license = repo.activate(sample_activation(&owner_id, "node-A")).await.expect("activar");
        assert_eq!(license.row_version, 1);

        // Dos "actores" leyeron la MISMA licencia en la versión 1.
        let first_writer_view = license.clone();
        let second_writer_view = license;

        clock.tick();
        let updated = repo
            .refresh_heartbeat(&first_writer_view, 20_000, "ACTIVE", "resigned-hex-1")
            .await
            .expect("el primer refresco debe tener éxito");
        assert_eq!(updated.row_version, 2);

        clock.tick();
        let conflict = repo.refresh_heartbeat(&second_writer_view, 30_000, "ACTIVE", "resigned-hex-2").await;
        assert!(
            matches!(conflict, Err(LicenseRepositoryError::VersionConflict { expected: 1, .. })),
            "el segundo refresco desde la versión 1 debe dar VersionConflict, no éxito silencioso; fue: {conflict:?}"
        );

        // La fila en disco conserva el cambio del PRIMER writer (20_000), no el del segundo (30_000).
        let reloaded = repo
            .find_by_owner_and_node(&owner_id, "node-A")
            .await
            .expect("releer")
            .expect("existe");
        assert_eq!(reloaded.row_version, 2);
        assert_eq!(reloaded.heartbeat_expires_at_ns, 20_000);
    }

    #[tokio::test]
    async fn find_by_owner_and_node_returns_none_for_unknown_machine() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let owner_id = seed_account(&pool, &clock, "user@example.com").await;
        let repo = LicenseRepository::new(&pool, &clock);

        let missing = repo
            .find_by_owner_and_node(&owner_id, "node-nunca-activada")
            .await
            .expect("buscar máquina inexistente");
        assert_eq!(missing, None);
    }
}
