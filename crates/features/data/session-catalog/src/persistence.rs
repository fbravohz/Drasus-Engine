//! [SHELL] Repositorio de persistencia para `session_template` y
//! `session_template_window` (migración `0022_instrument_session_catalog.sql`,
//! ADR-0162).
//!
//! Usa el pool de `shared` -- este módulo nunca crea su propio pool;
//! siempre recibe uno ya inicializado y migrado desde el exterior.
//!
//! ## Por qué SÍ usa `BEGIN IMMEDIATE` (a diferencia de `instrument-catalog`)
//!
//! Dar de alta una plantilla implica DOS tipos de escritura que deben
//! tener éxito o fallar juntas: el `INSERT` de `session_template` y el
//! `INSERT` de cada una de sus `session_template_window` hijas. Si el
//! proceso muriera entre el primero y el segundo, quedaría una plantilla
//! sin ninguna ventana -- un estado a medias que ninguna lectura posterior
//! podría distinguir de "la plantilla nunca tuvo ventanas". Por eso
//! [`SessionRepository::create_template`] envuelve TODAS las escrituras en
//! una única transacción `BEGIN IMMEDIATE`: o se persiste la plantilla
//! completa con sus ventanas, o no se persiste nada.

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use shared::public_interface::Clock;

use crate::domain::{compute_session_template_audit_hash, compute_session_window_audit_hash, SessionWindow, Weekday};
use crate::schemas::{NewSessionTemplate, SessionCalendar, SessionTemplateRow, SessionTemplateWindowRow};

/// Errores que devuelven las operaciones de [`SessionRepository`].
#[derive(Debug, thiserror::Error)]
pub enum SessionRepositoryError {
    /// La operación subyacente de SQLite falló.
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    /// [`SessionRepository::create_template`] con un `template_code` que
    /// ya existe -- cada forma de mercado se declara UNA sola vez
    /// (ADR-0162 Restricción #2: nunca se copia por bróker×símbolo).
    #[error("el template_code '{0}' ya existe en el catálogo")]
    DuplicateTemplateCode(String),
    /// Un `NewSessionTemplate` sin ventanas -- una plantilla sin ninguna
    /// ventana no tiene calendario que ofrecer (mismo criterio de
    /// [`crate::domain::SessionCatalogDomainError::NoWindows`], aplicado
    /// aquí en la Shell antes de tocar disco).
    #[error("una plantilla de sesión debe declarar al menos una ventana semanal")]
    NoWindows,
    /// Una fila de `session_template_window` tenía un valor de
    /// `start_day`/`end_day` fuera del vocabulario fijo -- error de
    /// integridad de datos.
    #[error("día de la semana desconocido en session_template_window: '{0}'")]
    UnknownWeekday(String),
}

/// Repositorio para `session_template` y `session_template_window`.
pub struct SessionRepository<'a> {
    pool: &'a SqlitePool,
    clock: &'a dyn Clock,
}

impl<'a> SessionRepository<'a> {
    /// Crea un repositorio asociado a `pool` y `clock`.
    pub fn new(pool: &'a SqlitePool, clock: &'a dyn Clock) -> Self {
        Self { pool, clock }
    }

    /// Da de alta una plantilla de sesión nueva con sus ventanas
    /// semanales, con `row_version = 1` en ambas tablas. Atómico: la
    /// plantilla y TODAS sus ventanas se persisten dentro de una única
    /// transacción `BEGIN IMMEDIATE` (ver comentario de módulo).
    ///
    /// Un `template_code` duplicado se rechaza como
    /// [`SessionRepositoryError::DuplicateTemplateCode`] -- el `UNIQUE` de
    /// la migración es el guardián primario; esta función solo traduce el
    /// error crudo de SQLite a un tipo de dominio legible.
    pub async fn create_template(&self, new: NewSessionTemplate) -> Result<SessionCalendar, SessionRepositoryError> {
        if new.windows.is_empty() {
            return Err(SessionRepositoryError::NoWindows);
        }

        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        let template_id = Uuid::now_v7().to_string();
        let now_ns = self.clock.timestamp_ns();
        let template_row_version: i64 = 1;

        let template_audit_hash = compute_session_template_audit_hash(
            &template_id,
            now_ns,
            template_row_version,
            None,
            &new.template_code,
            &new.display_name,
            &new.market_timezone_ref,
        );

        let insert_template = sqlx::query(
            "INSERT INTO session_template (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                node_id, process_id, template_code, display_name, market_timezone_ref\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&template_id)
        .bind(now_ns)
        .bind(now_ns)
        .bind(&template_audit_hash)
        .bind(Option::<String>::None)
        .bind(template_row_version)
        .bind(&new.node_id)
        .bind(&new.process_id)
        .bind(&new.template_code)
        .bind(&new.display_name)
        .bind(&new.market_timezone_ref)
        .execute(&mut *tx)
        .await;

        if let Err(sqlx::Error::Database(db_err)) = &insert_template {
            if is_duplicate_template_code_violation(&**db_err) {
                // El rollback ocurre automáticamente al soltar `tx` sin
                // `commit()` -- no se persiste nada.
                return Err(SessionRepositoryError::DuplicateTemplateCode(new.template_code));
            }
        }
        insert_template?;

        let mut window_rows = Vec::with_capacity(new.windows.len());
        for window in &new.windows {
            let window_id = Uuid::now_v7().to_string();
            let window_row_version: i64 = 1;
            let window_audit_hash =
                compute_session_window_audit_hash(&window_id, now_ns, window_row_version, None, &template_id, *window);

            sqlx::query(
                "INSERT INTO session_template_window (\
                    id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    node_id, process_id, session_template_id, \
                    start_day, end_day, start_time_minutes, end_time_minutes\
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&window_id)
            .bind(now_ns)
            .bind(now_ns)
            .bind(&window_audit_hash)
            .bind(Option::<String>::None)
            .bind(window_row_version)
            .bind(&new.node_id)
            .bind(&new.process_id)
            .bind(&template_id)
            .bind(window.start_day.as_str())
            .bind(window.end_day.as_str())
            .bind(window.start_time_minutes)
            .bind(window.end_time_minutes)
            .execute(&mut *tx)
            .await?;

            window_rows.push(SessionTemplateWindowRow {
                id: window_id,
                created_at: now_ns,
                updated_at: now_ns,
                audit_hash: window_audit_hash,
                audit_chain_hash: None,
                row_version: window_row_version,
                node_id: new.node_id.clone(),
                process_id: new.process_id.clone(),
                session_template_id: template_id.clone(),
                window: *window,
            });
        }

        tx.commit().await?;

        Ok(SessionCalendar {
            template: SessionTemplateRow {
                id: template_id,
                created_at: now_ns,
                updated_at: now_ns,
                audit_hash: template_audit_hash,
                audit_chain_hash: None,
                row_version: template_row_version,
                node_id: new.node_id,
                process_id: new.process_id,
                template_code: new.template_code,
                display_name: new.display_name,
                market_timezone_ref: new.market_timezone_ref,
            },
            windows: window_rows,
        })
    }

    /// Carga una plantilla de sesión por su clave de negocio
    /// (`template_code`, ej. `"FOREX_245"`), con TODAS sus ventanas.
    pub async fn find_template_by_code(&self, template_code: &str) -> Result<Option<SessionCalendar>, SessionRepositoryError> {
        let template_row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    node_id, process_id, template_code, display_name, market_timezone_ref \
             FROM session_template WHERE template_code = ?",
        )
        .bind(template_code)
        .fetch_optional(self.pool)
        .await?;

        let Some(template_row) = template_row else {
            return Ok(None);
        };

        self.load_calendar(row_to_template(template_row)?).await.map(Some)
    }

    /// Carga una plantilla de sesión por su clave TÉCNICA (`id`), con
    /// todas sus ventanas -- la búsqueda que usa la resolución de
    /// `default_session_template_id`/`session_template_id_override` de
    /// `instrument-catalog` (ambos son este mismo `id`).
    pub async fn find_template_by_id(&self, id: &str) -> Result<Option<SessionCalendar>, SessionRepositoryError> {
        let template_row = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    node_id, process_id, template_code, display_name, market_timezone_ref \
             FROM session_template WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        let Some(template_row) = template_row else {
            return Ok(None);
        };

        self.load_calendar(row_to_template(template_row)?).await.map(Some)
    }

    /// Carga las ventanas de una plantilla ya resuelta y arma el
    /// [`SessionCalendar`] completo. Factor común entre
    /// [`Self::find_template_by_code`] y [`Self::find_template_by_id`].
    async fn load_calendar(&self, template: SessionTemplateRow) -> Result<SessionCalendar, SessionRepositoryError> {
        let window_rows = sqlx::query(
            "SELECT id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                    node_id, process_id, session_template_id, \
                    start_day, end_day, start_time_minutes, end_time_minutes \
             FROM session_template_window WHERE session_template_id = ? ORDER BY start_day, start_time_minutes",
        )
        .bind(&template.id)
        .fetch_all(self.pool)
        .await?;

        let windows = window_rows.into_iter().map(row_to_window).collect::<Result<Vec<_>, _>>()?;

        Ok(SessionCalendar { template, windows })
    }
}

/// Decide si un error de base de datos es una violación UNIQUE de la
/// clave de NEGOCIO `template_code` -- transformable a
/// [`SessionRepositoryError::DuplicateTemplateCode`]. Una violación
/// UNIQUE de OTRA columna (en particular la PK técnica `id`) NO cuenta:
/// debe propagarse como el error genérico de base de datos. Mismo
/// criterio que `instrument_catalog::persistence::is_duplicate_instrument_id_violation`.
fn is_duplicate_template_code_violation(db_err: &(dyn sqlx::error::DatabaseError + 'static)) -> bool {
    db_err.is_unique_violation() && db_err.message().to_lowercase().contains("template_code")
}

fn row_to_template(row: sqlx::sqlite::SqliteRow) -> Result<SessionTemplateRow, SessionRepositoryError> {
    Ok(SessionTemplateRow {
        id: row.get("id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        row_version: row.get("row_version"),
        node_id: row.get("node_id"),
        process_id: row.get("process_id"),
        template_code: row.get("template_code"),
        display_name: row.get("display_name"),
        market_timezone_ref: row.get("market_timezone_ref"),
    })
}

fn row_to_window(row: sqlx::sqlite::SqliteRow) -> Result<SessionTemplateWindowRow, SessionRepositoryError> {
    let start_day_value: String = row.get("start_day");
    let end_day_value: String = row.get("end_day");
    let start_day = Weekday::from_str_value(&start_day_value).ok_or(SessionRepositoryError::UnknownWeekday(start_day_value))?;
    let end_day = Weekday::from_str_value(&end_day_value).ok_or(SessionRepositoryError::UnknownWeekday(end_day_value))?;

    Ok(SessionTemplateWindowRow {
        id: row.get("id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        audit_hash: row.get("audit_hash"),
        audit_chain_hash: row.get("audit_chain_hash"),
        row_version: row.get("row_version"),
        node_id: row.get("node_id"),
        process_id: row.get("process_id"),
        session_template_id: row.get("session_template_id"),
        window: SessionWindow {
            start_day,
            start_time_minutes: row.get("start_time_minutes"),
            end_day,
            end_time_minutes: row.get("end_time_minutes"),
        },
    })
}

// ── Pruebas de la capa de persistencia ───────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use shared::public_interface::{create_pool, run_migrations, DeterministicClock};

    async fn migrated_pool() -> (SqlitePool, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = dir.path().join("test.sqlite");
        let url = format!("sqlite://{}", db_path.display());
        let pool = create_pool(&url).await.expect("abrir base de datos temporal");
        run_migrations(&pool).await.expect("aplicar migraciones");
        (pool, dir)
    }

    fn sample_forex_template() -> NewSessionTemplate {
        NewSessionTemplate {
            template_code: "FOREX_245".to_string(),
            display_name: "Forex 24/5".to_string(),
            market_timezone_ref: "America/New_York".to_string(),
            windows: vec![SessionWindow {
                start_day: Weekday::Sun,
                start_time_minutes: 1_320,
                end_day: Weekday::Fri,
                end_time_minutes: 1_320,
            }],
            node_id: Some("node-A".to_string()),
            process_id: Some("pid-1".to_string()),
        }
    }

    // ── CRITERIO #4 (Orden §5): session_template_persists_iana_timezone ─────

    /// La plantilla + sus ventanas persisten; la zona horaria se guarda
    /// COMO IDENTIFICADOR IANA (no como convención de desplazamiento) --
    /// verifica el valor exacto, no solo que "algo" se guardó.
    #[tokio::test]
    async fn session_template_persists_iana_timezone() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = SessionRepository::new(&pool, &clock);

        let calendar = repo.create_template(sample_forex_template()).await.expect("crear plantilla");
        assert_eq!(calendar.template.market_timezone_ref, "America/New_York");
        assert_ne!(calendar.template.market_timezone_ref, "EST+07", "NUNCA una convención de desplazamiento inventada");
        assert_eq!(calendar.template.row_version, 1);
        assert_eq!(calendar.windows.len(), 1);

        let reloaded = repo.find_template_by_code("FOREX_245").await.expect("releer").expect("debe existir");
        assert_eq!(reloaded.template.market_timezone_ref, "America/New_York");
        assert_eq!(reloaded.windows.len(), 1);
        assert_eq!(reloaded.windows[0].window.start_day, Weekday::Sun);
        assert_eq!(reloaded.windows[0].window.start_time_minutes, 1_320);
    }

    /// `find_template_by_id` debe devolver la MISMA plantilla que
    /// `find_template_by_code` -- ambas búsquedas son caminos válidos
    /// hacia la misma fila (clave técnica vs. clave de negocio).
    #[tokio::test]
    async fn find_template_by_id_matches_find_template_by_code() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = SessionRepository::new(&pool, &clock);

        let created = repo.create_template(sample_forex_template()).await.expect("crear plantilla");
        let by_id = repo.find_template_by_id(&created.template.id).await.expect("buscar por id").expect("debe existir");
        assert_eq!(by_id.template.template_code, "FOREX_245");
    }

    // ── Duplicados y atomicidad ───────────────────────────────────────────

    /// Un `template_code` duplicado se rechaza -- y NO deja ventanas
    /// huérfanas del intento fallido (la transacción completa hizo rollback).
    #[tokio::test]
    async fn create_template_rejects_duplicate_template_code_without_orphan_windows() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = SessionRepository::new(&pool, &clock);

        repo.create_template(sample_forex_template()).await.expect("primera alta debe tener éxito");

        let mut duplicate = sample_forex_template();
        duplicate.display_name = "Forex 24/5 (intento duplicado)".to_string();
        let result = repo.create_template(duplicate).await;
        assert!(
            matches!(result, Err(SessionRepositoryError::DuplicateTemplateCode(ref code)) if code == "FOREX_245"),
            "un template_code duplicado debe rechazarse explícitamente; fue: {result:?}"
        );

        let template_count: i64 =
            sqlx::query("SELECT COUNT(*) FROM session_template").fetch_one(&pool).await.expect("contar").get(0);
        assert_eq!(template_count, 1, "el intento fallido no debe dejar una segunda plantilla");

        let window_count: i64 =
            sqlx::query("SELECT COUNT(*) FROM session_template_window").fetch_one(&pool).await.expect("contar").get(0);
        assert_eq!(window_count, 1, "el intento fallido NO debe dejar ventanas huérfanas -- la transacción hizo rollback completo");
    }

    // ── FIX B (gate de mutación, hallazgo Tech-Lead 2026-07-25): el
    // clasificador de duplicado distingue PK de clave de negocio ──────────

    /// CRITERIO DE CIERRE (mutación `&&`→`||`): la prueba anterior solo
    /// ejercita el caso donde AMBAS mitades de la guarda son verdaderas --
    /// `&&` y `||` dan el mismo resultado ahí. Esta prueba fuerza el caso
    /// donde SOLO la primera mitad es verdadera: una violación UNIQUE de
    /// la PK técnica `id` (mismo `id`, `template_code` DISTINTO) -- con
    /// `&&` correcto, NO debe clasificarse como duplicado de negocio.
    /// Ejercita `is_duplicate_template_code_violation` DIRECTAMENTE con
    /// errores reales de SQLite.
    #[tokio::test]
    async fn is_duplicate_template_code_violation_distinguishes_business_key_from_primary_key() {
        let (pool, _dir) = migrated_pool().await;

        let insert = |id: &'static str, template_code: &'static str| {
            let pool = pool.clone();
            async move {
                sqlx::query(
                    "INSERT INTO session_template (id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, template_code, display_name, market_timezone_ref) \
                     VALUES (?, 0, 0, 'h', NULL, 1, ?, 'Test', 'UTC')",
                )
                .bind(id)
                .bind(template_code)
                .execute(&pool)
                .await
            }
        };

        insert("id-a", "FOREX_245").await.expect("primera fila válida");

        // Violación de la CLAVE DE NEGOCIO: mismo template_code, id distinto.
        let business_key_err = insert("id-b", "FOREX_245").await.expect_err("debe violar el UNIQUE de template_code");
        let sqlx::Error::Database(business_key_db_err) = business_key_err else {
            panic!("se esperaba un error de base de datos")
        };
        assert!(
            is_duplicate_template_code_violation(&*business_key_db_err),
            "una violación UNIQUE de template_code SÍ es un duplicado de clave de negocio"
        );

        // Violación de la PK `id`: MISMO id, template_code DISTINTO --
        // NUNCA debe clasificarse como duplicado de negocio.
        let pk_err = insert("id-a", "CRYPTO_247").await.expect_err("debe violar la PRIMARY KEY id");
        let sqlx::Error::Database(pk_db_err) = pk_err else { panic!("se esperaba un error de base de datos") };
        assert!(
            !is_duplicate_template_code_violation(&*pk_db_err),
            "una violación UNIQUE de la PK id NUNCA debe clasificarse como duplicado de template_code"
        );
    }

    /// Una plantilla sin ventanas se rechaza ANTES de tocar disco.
    #[tokio::test]
    async fn create_template_rejects_empty_windows() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = SessionRepository::new(&pool, &clock);

        let mut empty = sample_forex_template();
        empty.windows = vec![];
        let result = repo.create_template(empty).await;
        assert!(matches!(result, Err(SessionRepositoryError::NoWindows)));

        let count: i64 = sqlx::query("SELECT COUNT(*) FROM session_template").fetch_one(&pool).await.expect("contar").get(0);
        assert_eq!(count, 0, "nada debe persistirse si se rechaza antes de tocar disco");
    }

    // ── CRITERIO #8 (Orden §5, para session_template_window): CHECK de día ──

    #[tokio::test]
    async fn check_constraints_reject_invalid_weekday() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let repo = SessionRepository::new(&pool, &clock);
        let calendar = repo.create_template(sample_forex_template()).await.expect("crear plantilla");

        let result = sqlx::query(
            "INSERT INTO session_template_window (\
                id, created_at, updated_at, audit_hash, audit_chain_hash, row_version, \
                session_template_id, start_day, end_day, start_time_minutes, end_time_minutes\
            ) VALUES ('win-1', 0, 0, 'hash', NULL, 1, ?, 'FUNDAY', 'MON', 0, 100)",
        )
        .bind(&calendar.template.id)
        .execute(&pool)
        .await;

        assert!(result.is_err(), "un start_day fuera de MON..SUN debe ser rechazado por el CHECK de la BD");
    }
}
