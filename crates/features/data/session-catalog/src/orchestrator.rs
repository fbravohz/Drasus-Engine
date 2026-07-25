//! Imperative Shell — adaptadores de infraestructura.
//!
//! Implementa la Frontera Pública declarada en
//! `docs/features/session-catalog.md`: `ResolveSession(instrument_id, venue)
//! → SessionCalendar` y `ExpectedCalendarFingerprint(template_id) →
//! CalendarFingerprint` (TTR-002/TTR-003). También compone la semilla
//! `SESSION_TEMPLATE_SEED_SET` (Forex 24/5, Cripto 24/7, Futuros CME
//! Globex representativo -- confirmado por el propietario 2026-07-25).

use sqlx::SqlitePool;

use shared::public_interface::Clock;

use crate::domain::{cme_globex_representative_windows, compute_expected_fingerprint, CalendarFingerprint, SessionCatalogDomainError, Weekday};
use crate::persistence::{SessionRepository, SessionRepositoryError};
use crate::schemas::{NewSessionTemplate, SessionCalendar};

/// Errores que puede producir la composición Shell de `session-catalog`.
#[derive(Debug, thiserror::Error)]
pub enum SessionOrchestratorError {
    #[error("error de persistencia: {0}")]
    Repository(#[from] SessionRepositoryError),
    #[error("error de dominio: {0}")]
    Domain(#[from] SessionCatalogDomainError),
    /// No existe ninguna plantilla con ese `template_id`/`template_code`.
    #[error("plantilla de sesión no encontrada: '{0}'")]
    TemplateNotFound(String),
}

/// Carga la plantilla de sesión resuelta por `template_id` (clave
/// técnica) -- implementación de `ResolveSession` una vez que el
/// `template_id` YA se resolvió por la cadena override->default
/// (`domain::resolve_session_template_id`, ejecutada por el llamador con
/// los IDs que le entregó `instrument-catalog`).
pub async fn resolve_session(pool: &SqlitePool, clock: &dyn Clock, template_id: &str) -> Result<SessionCalendar, SessionOrchestratorError> {
    let repo = SessionRepository::new(pool, clock);
    repo.find_template_by_id(template_id)
        .await?
        .ok_or_else(|| SessionOrchestratorError::TemplateNotFound(template_id.to_string()))
}

/// Deriva la huella de calendario esperada de una plantilla, por su
/// `template_code` (clave de negocio, ej. `"FOREX_245"`) -- implementación
/// de `ExpectedCalendarFingerprint`, el payload que consume el Gate de
/// Coherencia de `pit-data-validator`.
pub async fn expected_fingerprint(
    pool: &SqlitePool,
    clock: &dyn Clock,
    template_code: &str,
) -> Result<CalendarFingerprint, SessionOrchestratorError> {
    let repo = SessionRepository::new(pool, clock);
    let calendar = repo
        .find_template_by_code(template_code)
        .await?
        .ok_or_else(|| SessionOrchestratorError::TemplateNotFound(template_code.to_string()))?;

    let windows: Vec<_> = calendar.windows.iter().map(|w| w.window).collect();
    Ok(compute_expected_fingerprint(&windows)?)
}

// ── SESSION_TEMPLATE_SEED_SET (confirmado por el propietario 2026-07-25) ──

/// Las tres plantillas semilla que el sistema ofrece al arrancar
/// (`SESSION_TEMPLATE_SEED_SET`, CONFIG -- `docs/features/session-catalog.md`
/// "Parámetros Configurables"). Forex y Cripto son formas de mercado
/// universales; Futuros es un default REPRESENTATIVO y EDITABLE (ver
/// `domain::cme_globex_representative_windows`).
pub fn session_template_seed_set() -> Vec<NewSessionTemplate> {
    vec![
        NewSessionTemplate {
            template_code: "FOREX_245".to_string(),
            display_name: "Forex 24/5".to_string(),
            market_timezone_ref: "America/New_York".to_string(),
            windows: vec![crate::domain::SessionWindow {
                start_day: Weekday::Sun,
                start_time_minutes: 1_320, // 22:00 domingo
                end_day: Weekday::Fri,
                end_time_minutes: 1_320, // 22:00 viernes
            }],
            node_id: None,
            process_id: None,
        },
        NewSessionTemplate {
            template_code: "CRYPTO_247".to_string(),
            display_name: "Cripto 24/7".to_string(),
            market_timezone_ref: "UTC".to_string(),
            windows: vec![crate::domain::SessionWindow {
                start_day: Weekday::Mon,
                start_time_minutes: 0,
                end_day: Weekday::Sun,
                end_time_minutes: 1_439,
            }],
            node_id: None,
            process_id: None,
        },
        NewSessionTemplate {
            template_code: "FUTURES_CME_GLOBEX".to_string(),
            display_name: "Futuros (CME Globex, representativo)".to_string(),
            market_timezone_ref: "America/Chicago".to_string(),
            windows: cme_globex_representative_windows(),
            node_id: None,
            process_id: None,
        },
    ]
}

/// Da de alta las plantillas semilla que todavía no existan en el
/// catálogo -- idempotente: si una ya existe (`template_code` duplicado),
/// se omite sin fallar, para que correr esta función varias veces (ej. en
/// cada arranque) sea un no-op sobre lo ya sembrado.
pub async fn seed_default_templates(pool: &SqlitePool, clock: &dyn Clock) -> Result<Vec<SessionCalendar>, SessionOrchestratorError> {
    let repo = SessionRepository::new(pool, clock);
    let mut seeded = Vec::new();

    for template in session_template_seed_set() {
        match repo.create_template(template).await {
            Ok(calendar) => seeded.push(calendar),
            Err(SessionRepositoryError::DuplicateTemplateCode(_)) => {
                // Ya sembrada en una corrida anterior -- no-op, no es error.
                continue;
            }
            Err(other) => return Err(other.into()),
        }
    }

    Ok(seeded)
}

// ── Pruebas del Shell (composición completa) ─────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;
    use shared::public_interface::{create_pool, run_migrations, DeterministicClock};

    async fn migrated_pool() -> (SqlitePool, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("crear directorio temporal");
        let db_path = dir.path().join("test.sqlite");
        let url = format!("sqlite://{}", db_path.display());
        let pool = create_pool(&url).await.expect("abrir base de datos temporal");
        run_migrations(&pool).await.expect("aplicar migraciones");
        (pool, dir)
    }

    /// Sembrar produce EXACTAMENTE las tres plantillas confirmadas por el
    /// propietario, con sus `template_code` esperados.
    #[tokio::test]
    async fn seed_default_templates_creates_all_three_confirmed_templates() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);

        let seeded = seed_default_templates(&pool, &clock).await.expect("sembrar debe tener éxito");
        assert_eq!(seeded.len(), 3, "la semilla debe crear EXACTAMENTE 3 plantillas (Forex, Cripto, Futuros)");

        let mut codes: Vec<&str> = seeded.iter().map(|c| c.template.template_code.as_str()).collect();
        codes.sort_unstable();
        assert_eq!(codes, vec!["CRYPTO_247", "FOREX_245", "FUTURES_CME_GLOBEX"]);
    }

    /// Sembrar dos veces es idempotente: la segunda corrida no falla y no
    /// duplica filas.
    #[tokio::test]
    async fn seed_default_templates_is_idempotent() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);

        seed_default_templates(&pool, &clock).await.expect("primera siembra");
        let second = seed_default_templates(&pool, &clock).await.expect("segunda siembra no debe fallar");
        assert_eq!(second.len(), 0, "la segunda corrida no debe reportar ninguna plantilla nueva creada");

        let count: i64 = sqlx::query("SELECT COUNT(*) FROM session_template")
            .fetch_one(&pool)
            .await
            .expect("contar")
            .get::<i64, _>(0);
        assert_eq!(count, 3, "no debe haber plantillas duplicadas tras sembrar dos veces");
    }

    /// `expected_fingerprint` sobre las plantillas YA sembradas reproduce
    /// las tres formas discriminadas por `domain::compute_expected_fingerprint`
    /// -- cierra el camino completo (persistencia real + Núcleo).
    #[tokio::test]
    async fn expected_fingerprint_matches_seeded_templates_shapes() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        seed_default_templates(&pool, &clock).await.expect("sembrar");

        let crypto = expected_fingerprint(&pool, &clock, "CRYPTO_247").await.expect("cripto");
        assert_eq!(crypto, CalendarFingerprint::Continuous);

        let forex = expected_fingerprint(&pool, &clock, "FOREX_245").await.expect("forex");
        assert_eq!(forex, CalendarFingerprint::WeeklyGap);

        let futures = expected_fingerprint(&pool, &clock, "FUTURES_CME_GLOBEX").await.expect("futuros");
        assert_eq!(futures, CalendarFingerprint::WeeklyGapPlusDailyMaintenance);
    }

    /// Un `template_id` inexistente produce un error explícito.
    #[tokio::test]
    async fn resolve_session_missing_template_returns_explicit_error() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);

        let result = resolve_session(&pool, &clock, "does-not-exist").await;
        assert!(matches!(result, Err(SessionOrchestratorError::TemplateNotFound(_))));
    }

    // ── CRITERIO #6 (Orden §5): resolve_session_follows_override_chain ────

    /// `ResolveSession` (a través de `resolve_session`, ya con el
    /// `template_id` resuelto por la cadena pura
    /// `domain::resolve_session_template_id`) devuelve la plantilla
    /// correcta tanto cuando el ID viene del override de bróker como
    /// cuando viene del default del instrumento -- ejercita la cadena
    /// completa: resolución pura -> carga real de la plantilla resultante.
    #[tokio::test]
    async fn resolve_session_follows_override_chain() {
        let (pool, _dir) = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        seed_default_templates(&pool, &clock).await.expect("sembrar");

        let repo = SessionRepository::new(&pool, &clock);
        let forex_template = repo.find_template_by_code("FOREX_245").await.expect("buscar").expect("debe existir");
        let crypto_template = repo.find_template_by_code("CRYPTO_247").await.expect("buscar").expect("debe existir");

        // Caso A: el bróker declaró un override de calendario -- debe
        // ganar sobre el default del instrumento (aunque sean distintos).
        let resolved_id_a = crate::domain::resolve_session_template_id(
            Some(forex_template.template.id.as_str()),
            Some(crypto_template.template.id.as_str()),
        )
        .expect("debe resolver algún id");
        let calendar_a = resolve_session(&pool, &clock, &resolved_id_a).await.expect("debe cargar la plantilla resuelta");
        assert_eq!(calendar_a.template.template_code, "FOREX_245", "el override de bróker debe ganar");

        // Caso B: sin override de bróker, cae al default del instrumento.
        let resolved_id_b = crate::domain::resolve_session_template_id(None, Some(crypto_template.template.id.as_str()))
            .expect("debe resolver algún id");
        let calendar_b = resolve_session(&pool, &clock, &resolved_id_b).await.expect("debe cargar la plantilla resuelta");
        assert_eq!(calendar_b.template.template_code, "CRYPTO_247", "sin override, debe caer al default del instrumento");
    }
}
