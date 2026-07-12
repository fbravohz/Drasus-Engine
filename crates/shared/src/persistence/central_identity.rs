//! [SHELL] Repositorio de persistencia para Central Identity
//! (`docs/features/central-identity.md`, ADR-0143, ADR-0144, ADR-0141,
//! ADR-0020, migración `0007_central_identity.sql`).
//!
//! Envuelve la tabla `accounts`. Dueño del único I/O para cuentas:
//! lecturas/escrituras en SQLite, generación de UUIDv7 (ADR-0141: "PK
//! universal ... UUIDv7 generado con `Uuid::now_v7()`") y la lectura del
//! puerto [`Clock`]. La lógica pura (validación de correo, hash de
//! auditoría encadenado) vive en [`crate::domain::central_identity`] --
//! este módulo solo le da entradas inyectadas y persiste/carga el
//! resultado, reflejando el patrón de
//! [`crate::persistence::job::JobRepository`].
//!
//! ## `row_version` en vez de `event_sequence_id` (ADR-0141)
//!
//! `accounts` es una tabla MUTABLE (el estado de verificación de correo
//! cambia). ADR-0141 prohíbe usar `event_sequence_id` para lo que es en
//! realidad un contador de versión por fila -- por eso
//! [`AccountRepository::update_email_verification_status`] incrementa
//! `row_version` en vez de generar una posición en una secuencia global.

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::central_identity::{
    compute_account_audit_hash, normalize_email, validate_email_format, AccountIdentity,
    EmailFormatError, EmailVerificationStatus,
};
use crate::domain::clock::Clock;

/// Errores que devuelven las operaciones de [`AccountRepository`].
#[derive(Debug, thiserror::Error)]
pub enum AccountRepositoryError {
    /// La operación subyacente de SQLite falló.
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// El correo de la cuenta nueva no pasó [`validate_email_format`].
    #[error("correo inválido: {0}")]
    InvalidEmail(#[from] EmailFormatError),
    /// Una fila de `accounts` tenía un valor de `email_verification_status`
    /// fuera de las tres cadenas canónicas -- un error de integridad de
    /// datos, no un error de la operación solicitada.
    #[error("estado de verificación de correo desconocido en la tabla accounts: '{0}'")]
    UnknownVerificationStatus(String),
    /// Concurrencia optimista (ADR-0141): el UPDATE partió de un
    /// `row_version` que ya no es el vigente en disco -- otra escritura
    /// actualizó la fila en el ínterin. La operación NO pisa el cambio
    /// ajeno; quien llama debe releer la fila y reintentar sobre la versión
    /// actual.
    #[error("conflicto de versión en la cuenta '{id}': se esperaba row_version {expected}, la fila ya avanzó")]
    VersionConflict { id: String, expected: i64 },
}

/// Una cuenta nueva para persistir (`docs/features/central-identity.md`
/// "Ciclo de Vida": "Entrada": credenciales del usuario + identificadores de
/// máquina), más los metadatos de ADR-0020 que provee quien llama al
/// momento de crearla.
#[derive(Debug, Clone)]
pub struct NewAccount {
    pub email: String,
    /// Proveedor de identidad federada vinculado, si el registro fue vía
    /// OAuth (`None` si es solo correo).
    pub oauth_provider: Option<String>,
    pub institutional_tag: String,
    pub access_token_id: Option<String>,
    /// Huella de hardware ya calculada (ver
    /// [`crate::domain::central_identity::compute_hardware_fingerprint`]) --
    /// este repositorio no la calcula, solo la persiste.
    pub node_id: String,
    /// Dueño explícito, si aplica (ej. sub-cuenta institucional). `None` =>
    /// la cuenta es dueña de sí misma (retail individual): `owner_id` se
    /// fija al propio `id` recién generado.
    pub owner_id: Option<String>,
}

/// Una fila de cuenta persistida (tabla `accounts`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
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

    pub email: String,
    pub email_verification_status: EmailVerificationStatus,
    pub oauth_provider: Option<String>,
}

/// Repositorio para `accounts`.
///
/// Constrúyelo con un [`SqlitePool`] ya migrado (ver
/// [`crate::persistence::pool::connect`] +
/// [`crate::persistence::pool::migrate`]) y cualquier implementación de
/// [`Clock`] (producción: [`crate::orchestrator::SystemClock`];
/// tests/backtests: [`crate::domain::clock::DeterministicClock`]).
pub struct AccountRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> AccountRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`. Ambos se toman
    /// prestados por la vida del repositorio -- no se toma ownership.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Persiste una cuenta nueva con `row_version = 1` y estado de
    /// verificación `PENDING` (`EMAIL_VERIFICATION_REQUIRED` es FIJO --
    /// central-identity.md: "no activa la cuenta hasta confirmarla").
    ///
    /// Genera un UUIDv7 fresco (`id`, ADR-0141: `Uuid::now_v7()` -- a
    /// diferencia de `jobs`/`audit_events`, que usan v4 desde antes de este
    /// ADR) y lee el [`Clock`] actual (`created_at_ns` == `updated_at_ns`
    /// para una fila recién creada). `audit_chain_hash` es `None` para la
    /// versión génesis de la fila (`row_version == 1`).
    pub async fn create(&self, new_account: NewAccount) -> Result<Account, AccountRepositoryError> {
        // Normaliza el correo en la frontera (trim + minúsculas) antes de
        // validar y persistir: la unicidad de correo es case-insensitive
        // aunque el índice SQLite compare bytes exactos (Defecto 3 del QA).
        let email = normalize_email(&new_account.email);
        validate_email_format(&email)?;

        let id = Uuid::now_v7().to_string();
        let now_ns = self.clock.timestamp_ns();
        let row_version: i64 = 1;
        let status = EmailVerificationStatus::Pending;
        // Una cuenta retail individual es dueña de sí misma; una sub-cuenta
        // institucional puede declarar un owner_id explícito distinto.
        let owner_id = new_account.owner_id.clone().unwrap_or_else(|| id.clone());

        let audit_hash = compute_account_audit_hash(
            &id,
            now_ns,
            row_version,
            None,
            &email,
            status,
            new_account.oauth_provider.as_deref(),
            &new_account.node_id,
        );

        sqlx::query(
            "INSERT INTO accounts (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                owner_id, institutional_tag, access_token_id, \
                node_id, \
                email, email_verification_status, oauth_provider\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(Option::<String>::None)
        .bind(row_version)
        .bind(&owner_id)
        .bind(&new_account.institutional_tag)
        .bind(&new_account.access_token_id)
        .bind(&new_account.node_id)
        .bind(&email)
        .bind(status.as_str())
        .bind(&new_account.oauth_provider)
        .execute(self.pool)
        .await?;

        Ok(Account {
            id,
            created_at_ns: now_ns,
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: None,
            row_version,
            owner_id,
            institutional_tag: new_account.institutional_tag,
            access_token_id: new_account.access_token_id,
            node_id: new_account.node_id,
            email,
            email_verification_status: status,
            oauth_provider: new_account.oauth_provider,
        })
    }

    /// Carga una única cuenta por `id`, o `None` si no existe.
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Account>, AccountRepositoryError> {
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    owner_id, institutional_tag, access_token_id, \
                    node_id, \
                    email, email_verification_status, oauth_provider \
             FROM accounts WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        row.map(row_to_account).transpose()
    }

    /// Carga una única cuenta por `email` (correo es único -- índice
    /// `idx_accounts_email` en la migración), o `None` si no existe.
    ///
    /// Normaliza el correo de búsqueda igual que [`Self::create`] (trim +
    /// minúsculas) para que el lookup sea case-insensitive: buscar
    /// `Case@Example.com` encuentra la fila persistida como
    /// `case@example.com` (Defecto 3 del QA). Es la vía de idempotencia del
    /// registro: antes de crear una cuenta nueva, quien llama busca primero
    /// por correo (ver `LocalStubCentralIdentityVerifier::verify_identity`).
    pub async fn find_by_email(&self, email: &str) -> Result<Option<Account>, AccountRepositoryError> {
        let normalized = normalize_email(email);
        let row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    owner_id, institutional_tag, access_token_id, \
                    node_id, \
                    email, email_verification_status, oauth_provider \
             FROM accounts WHERE email = ?",
        )
        .bind(&normalized)
        .fetch_optional(self.pool)
        .await?;

        row.map(row_to_account).transpose()
    }

    /// Actualiza `email_verification_status` para `account` (TTR-001/002 de
    /// la Feature: la cuenta pasa de PENDING a VERIFIED/REJECTED).
    ///
    /// Incrementa `row_version` (+1), fija `updated_at` a la lectura actual
    /// de [`Clock`], y encadena `audit_hash`/`audit_chain_hash` igual que
    /// [`crate::persistence::job::JobRepository::transition`] -- el
    /// `audit_chain_hash` nuevo es el `audit_hash` que tenía la fila ANTES
    /// de esta actualización.
    ///
    /// ## Concurrencia optimista (ADR-0141) — Defecto 1 del QA
    ///
    /// El UPDATE filtra por `id` **y** `row_version = <el de `account`>`. Si
    /// otra escritura ya avanzó la fila desde que se leyó `account`, el
    /// `WHERE` no encuentra ninguna fila (`rows_affected() == 0`) y se
    /// devuelve [`AccountRepositoryError::VersionConflict`] en vez de pisar
    /// el cambio ajeno. Sin la comparación de `row_version`, dos updates que
    /// parten de la misma versión en memoria tendrían ambos éxito
    /// (last-write-wins) y bifurcarían la cadena `audit_hash` en silencio.
    pub async fn update_email_verification_status(
        &self,
        account: &Account,
        new_status: EmailVerificationStatus,
    ) -> Result<Account, AccountRepositoryError> {
        let now_ns = self.clock.timestamp_ns();
        let row_version = account.row_version + 1;

        let audit_hash = compute_account_audit_hash(
            &account.id,
            now_ns,
            row_version,
            Some(&account.audit_hash),
            &account.email,
            new_status,
            account.oauth_provider.as_deref(),
            &account.node_id,
        );

        // La guarda `row_version = ?` es la comparación optimista: solo
        // actualiza si la fila en disco sigue en la versión que leímos.
        let result = sqlx::query(
            "UPDATE accounts SET \
                updated_at = ?, audit_hash = ?, audit_chain_hash = ?, row_version = ?, \
                email_verification_status = ? \
             WHERE id = ? AND row_version = ?",
        )
        .bind(now_ns)
        .bind(&audit_hash)
        .bind(&account.audit_hash)
        .bind(row_version)
        .bind(new_status.as_str())
        .bind(&account.id)
        .bind(account.row_version)
        .execute(self.pool)
        .await?;

        // Cero filas afectadas => la fila ya no está en `account.row_version`
        // (otra escritura la adelantó). No pisamos: reportamos el conflicto.
        if result.rows_affected() == 0 {
            return Err(AccountRepositoryError::VersionConflict {
                id: account.id.clone(),
                expected: account.row_version,
            });
        }

        Ok(Account {
            updated_at_ns: now_ns,
            audit_hash,
            audit_chain_hash: Some(account.audit_hash.clone()),
            row_version,
            email_verification_status: new_status,
            ..account.clone()
        })
    }
}

/// Convierte una fila de `accounts` al tipo [`Account`].
fn row_to_account(row: sqlx::sqlite::SqliteRow) -> Result<Account, AccountRepositoryError> {
    let status_value: String = row.get("email_verification_status");
    let email_verification_status = EmailVerificationStatus::from_str_value(&status_value)
        .ok_or(AccountRepositoryError::UnknownVerificationStatus(status_value))?;

    Ok(Account {
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
        email: row.get("email"),
        email_verification_status,
        oauth_provider: row.get("oauth_provider"),
    })
}

impl From<&Account> for AccountIdentity {
    /// Proyecta la fila persistida (`Account`, con metadatos internos como
    /// `access_token_id`) al tipo de puerto público `AccountIdentity`
    /// (ADR-0137: `identity_out`) -- deliberadamente NO copia
    /// `access_token_id` ni ningún otro campo fuera de los cinco que
    /// `AccountIdentity` declara (ADR-0093: sin secretos en el puerto
    /// público).
    fn from(account: &Account) -> Self {
        AccountIdentity {
            owner_id: account.owner_id.clone(),
            email: account.email.clone(),
            email_verification_status: account.email_verification_status,
            node_id: account.node_id.clone(),
            institutional_tag: account.institutional_tag.clone(),
        }
    }
}

/// Utilidad de pruebas compartida por TODO el crate (ADR-0141 enmienda
/// 2026-07-11, M6): con la FK física `owner_id -> accounts(id)` activa en
/// `PRAGMA foreign_keys=ON`, cualquier test que inserte una fila con
/// `owner_id` en otra tabla del substrato necesita una cuenta real
/// preexistente cuyo `id` use como `owner_id` -- un literal como
/// `"owner-1"` ya no basta, la FK lo rechaza. `seed_account` crea esa
/// cuenta mínima y devuelve su `id`. Vive aquí (no duplicado por archivo)
/// porque `AccountRepository`/`NewAccount` son de este módulo.
#[cfg(test)]
pub(crate) mod test_support {
    use super::{AccountRepository, NewAccount};
    use crate::domain::clock::Clock;
    use sqlx::SqlitePool;

    /// Crea una cuenta semilla con `email` dado y devuelve su `owner_id`
    /// (== su propio `id`, una cuenta retail es dueña de sí misma). Falla
    /// el test con `.expect(...)` si la inserción falla -- una cuenta
    /// semilla que no se pudo crear invalida cualquier aserción posterior.
    pub(crate) async fn seed_account(pool: &SqlitePool, clock: &dyn Clock, email: &str) -> String {
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
        account.owner_id
    }
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

    fn sample_new_account() -> NewAccount {
        NewAccount {
            email: "user@example.com".to_string(),
            oauth_provider: None,
            institutional_tag: "DRASUS_LOCAL".to_string(),
            access_token_id: None,
            node_id: "fingerprint-hash-1".to_string(),
            owner_id: None,
        }
    }

    // ── CRITERIO DE CIERRE #1: esquema STRICT + Grupo I + Perfil D + row_version ──

    /// La migración crea la tabla `accounts` en modo STRICT, con las
    /// columnas del Grupo I (sustituyendo `event_sequence_id` por
    /// `row_version`, ADR-0141) + Perfil D (`owner_id`, `institutional_tag`,
    /// `access_token_id`, `node_id`) + columnas propias de la Feature. NO
    /// existe una columna `event_sequence_id` en esta tabla (append-only es
    /// el patrón equivocado para una tabla mutable).
    #[tokio::test]
    async fn migration_creates_accounts_table_with_group_i_profile_d_and_row_version() {
        let pool = migrated_pool().await;

        let columns = sqlx::query("SELECT name FROM pragma_table_info('accounts')")
            .fetch_all(&pool)
            .await
            .expect("leer table_info de accounts");
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
            "email",
            "email_verification_status",
            "oauth_provider",
        ] {
            assert!(column_names.contains(&expected.to_string()), "falta la columna: {expected}");
        }

        assert!(
            !column_names.contains(&"event_sequence_id".to_string()),
            "accounts es una tabla MUTABLE (ADR-0141): no debe tener event_sequence_id, solo row_version"
        );

        // STRICT mode: sqlite_master.sql debe contener la palabra "STRICT".
        let sql: String = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'accounts'")
            .fetch_one(&pool)
            .await
            .expect("leer sqlite_master")
            .get(0);
        assert!(sql.contains("STRICT"), "la tabla accounts debe declararse STRICT");
    }

    // ── CRITERIO DE CIERRE #persistencia: row_version incrementa al actualizar ──

    /// Crear una cuenta la persiste con `row_version == 1` y estado
    /// `PENDING`; actualizarla incrementa `row_version` a 2, cambia
    /// `audit_hash` y encadena `audit_chain_hash` al `audit_hash` anterior.
    /// Releer la fila desde SQLite confirma que el cambio quedó en disco
    /// (no solo en el struct devuelto en memoria).
    #[tokio::test]
    async fn update_email_verification_status_increments_row_version() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AccountRepository::new(&pool, &clock);

        let account = repo.create(sample_new_account()).await.expect("crear cuenta");
        assert_eq!(account.row_version, 1);
        assert_eq!(account.email_verification_status, EmailVerificationStatus::Pending);
        assert_eq!(account.audit_chain_hash, None);

        clock.tick();
        let verified = repo
            .update_email_verification_status(&account, EmailVerificationStatus::Verified)
            .await
            .expect("actualizar estado de verificación");

        assert_eq!(verified.row_version, 2, "row_version debe incrementar en cada UPDATE");
        assert_eq!(verified.email_verification_status, EmailVerificationStatus::Verified);
        assert_eq!(verified.audit_chain_hash, Some(account.audit_hash.clone()));
        assert_ne!(verified.audit_hash, account.audit_hash);

        // Releer desde disco: el cambio debe estar persistido, no solo en memoria.
        let reloaded = repo
            .find_by_id(&account.id)
            .await
            .expect("releer cuenta")
            .expect("la cuenta debe existir");
        assert_eq!(reloaded.row_version, 2);
        assert_eq!(reloaded.email_verification_status, EmailVerificationStatus::Verified);
        assert_eq!(reloaded, verified);
    }

    /// Un segundo UPDATE encadena sobre el primero: `row_version` llega a
    /// 3, y `audit_chain_hash` apunta al `audit_hash` de la versión 2 (no a
    /// la génesis).
    #[tokio::test]
    async fn successive_updates_chain_row_versions() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AccountRepository::new(&pool, &clock);

        let account = repo.create(sample_new_account()).await.expect("crear cuenta");
        clock.tick();
        let v2 = repo
            .update_email_verification_status(&account, EmailVerificationStatus::Verified)
            .await
            .expect("actualizar a VERIFIED");
        clock.tick();
        let v3 = repo
            .update_email_verification_status(&v2, EmailVerificationStatus::Rejected)
            .await
            .expect("actualizar a REJECTED");

        assert_eq!(v3.row_version, 3);
        assert_eq!(v3.audit_chain_hash, Some(v2.audit_hash.clone()));
    }

    /// CRITERIO DE CIERRE (Defecto 1 del QA — concurrencia optimista):
    /// dos updates que parten del MISMO `account` (misma `row_version` en
    /// memoria) no pueden ambos tener éxito. El primero pasa (row_version
    /// 1 -> 2); el segundo, que sigue creyendo estar en la versión 1,
    /// devuelve `VersionConflict` en vez de pisar el cambio del primero.
    ///
    /// Esta prueba FALLA si se quita la guarda `AND row_version = ?` del
    /// UPDATE: sin ella, el segundo update también afectaría 1 fila y
    /// devolvería `Ok`, bifurcando la cadena `audit_hash` en silencio.
    #[tokio::test]
    async fn concurrent_updates_from_same_version_conflict_instead_of_overwriting() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AccountRepository::new(&pool, &clock);

        let account = repo.create(sample_new_account()).await.expect("crear cuenta");
        assert_eq!(account.row_version, 1);

        // Dos "actores" leyeron la MISMA cuenta en la versión 1.
        let first_writer_view = account.clone();
        let second_writer_view = account;

        // El primer writer gana: 1 -> 2.
        clock.tick();
        let updated = repo
            .update_email_verification_status(&first_writer_view, EmailVerificationStatus::Verified)
            .await
            .expect("el primer update debe tener éxito");
        assert_eq!(updated.row_version, 2);

        // El segundo writer sigue creyendo estar en la versión 1 -> conflicto.
        clock.tick();
        let conflict = repo
            .update_email_verification_status(&second_writer_view, EmailVerificationStatus::Rejected)
            .await;
        assert!(
            matches!(conflict, Err(AccountRepositoryError::VersionConflict { expected: 1, .. })),
            "el segundo update desde la versión 1 debe dar VersionConflict, no éxito silencioso; fue: {conflict:?}"
        );

        // La fila en disco conserva el cambio del PRIMER writer (VERIFIED),
        // no el del segundo (REJECTED) que se rechazó.
        let reloaded = repo
            .find_by_id(&updated.id)
            .await
            .expect("releer")
            .expect("existe");
        assert_eq!(reloaded.row_version, 2);
        assert_eq!(reloaded.email_verification_status, EmailVerificationStatus::Verified);
    }

    /// CRITERIO DE CIERRE (Defecto 3 del QA — unicidad case-insensitive):
    /// una cuenta creada con `Case@Example.com` se encuentra buscando
    /// `case@example.com`, y crear la segunda variante es rechazado por el
    /// índice único (misma cuenta, no dos). El correo se persiste ya
    /// normalizado a minúsculas.
    #[tokio::test]
    async fn email_uniqueness_is_case_insensitive() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AccountRepository::new(&pool, &clock);

        let mut first = sample_new_account();
        first.email = "  Case@Example.COM ".to_string();
        let account = repo.create(first).await.expect("crear cuenta con mayúsculas y espacios");

        // Se persiste normalizado (trim + minúsculas).
        assert_eq!(account.email, "case@example.com");

        // Buscar la variante en minúsculas encuentra la misma fila.
        let found = repo
            .find_by_email("case@example.com")
            .await
            .expect("buscar en minúsculas")
            .expect("debe encontrar la cuenta creada con mayúsculas");
        assert_eq!(found.id, account.id);

        // Intentar crear la variante en minúsculas la rechaza el índice único
        // (una cuenta por correo), no crea una segunda.
        let mut second = sample_new_account();
        second.email = "case@example.com".to_string();
        let dup = repo.create(second).await;
        assert!(
            matches!(dup, Err(AccountRepositoryError::Database(_))),
            "crear la misma cuenta en otra caja de mayúsculas debe violar el índice único; fue: {dup:?}"
        );

        let count: i64 = sqlx::query("SELECT COUNT(*) FROM accounts")
            .fetch_one(&pool)
            .await
            .expect("contar filas")
            .get(0);
        assert_eq!(count, 1, "no deben existir dos cuentas para el mismo correo en distinta caja");
    }

    /// `find_by_email` recupera la cuenta creada; un correo inexistente
    /// devuelve `None`.
    #[tokio::test]
    async fn find_by_email_finds_existing_and_returns_none_for_missing() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AccountRepository::new(&pool, &clock);

        let account = repo.create(sample_new_account()).await.expect("crear cuenta");

        let found = repo
            .find_by_email("user@example.com")
            .await
            .expect("buscar por correo")
            .expect("debe existir");
        assert_eq!(found.id, account.id);

        let missing = repo.find_by_email("nadie@example.com").await.expect("buscar correo inexistente");
        assert_eq!(missing, None);
    }

    /// Crear una cuenta con correo malformado se rechaza ANTES de tocar la
    /// base de datos (ninguna fila se inserta).
    #[tokio::test]
    async fn create_rejects_malformed_email_without_inserting() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AccountRepository::new(&pool, &clock);

        let mut request = sample_new_account();
        request.email = "correo-sin-arroba".to_string();

        let result = repo.create(request).await;
        assert!(matches!(result, Err(AccountRepositoryError::InvalidEmail(_))));

        let count: i64 = sqlx::query("SELECT COUNT(*) FROM accounts")
            .fetch_one(&pool)
            .await
            .expect("contar filas")
            .get(0);
        assert_eq!(count, 0, "un correo inválido no debe insertar ninguna fila");
    }

    /// Una cuenta retail (sin `owner_id` explícito) es dueña de sí misma.
    #[tokio::test]
    async fn create_without_explicit_owner_defaults_owner_id_to_self() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AccountRepository::new(&pool, &clock);

        let account = repo.create(sample_new_account()).await.expect("crear cuenta");
        assert_eq!(account.owner_id, account.id);
    }

    /// La proyección `From<&Account> for AccountIdentity` nunca copia
    /// `access_token_id` (guardarraíl ADR-0093 aplicado en el punto de
    /// conversión, no solo en el struct destino).
    #[tokio::test]
    async fn account_identity_projection_omits_access_token_id() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = AccountRepository::new(&pool, &clock);

        let mut request = sample_new_account();
        request.access_token_id = Some("secret-session-token-do-not-leak".to_string());
        let account = repo.create(request).await.expect("crear cuenta");

        let identity = AccountIdentity::from(&account);
        let json = serde_json::to_string(&identity).expect("serializar AccountIdentity");
        assert!(
            !json.contains("secret-session-token-do-not-leak"),
            "AccountIdentity jamás debe incluir access_token_id"
        );
    }
}
