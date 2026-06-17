//! [CORE] Máquina de estados pura para jobs asíncronos
//! (`docs/features/async-job-executor.md` TTR-ASYNC-EXECUTOR-001..006,
//! ADR-0004, ADR-0011).
//!
//! Sin I/O, sin reloj de sistema, sin azar sin semilla (ADR-0002/0004).
//! El `id` (UUID) y los timestamps los inyecta la cáscara (capa de
//! persistencia / orquestador) — el mismo patrón que
//! [`super::audit_log::chain_event`] — para que, dadas las mismas
//! entradas, cada función de este módulo siempre produzca la misma
//! salida, bit a bit.
//!
//! ## Estados (FSM de ADR-0004)
//!
//! - [`JobState::Queued`]: esperando a un worker.
//! - [`JobState::Running`]: un worker lo tomó.
//! - [`JobState::Completed`]: terminó con éxito (terminal).
//! - [`JobState::Failed`]: terminó con error (terminal).
//! - [`JobState::Cancelled`]: cancelado por el usuario (terminal).
//!
//! ## Transiciones válidas (async-job-executor.md TTRs 002/004/006)
//!
//! | De | A | Cuándo |
//! |---|---|---|
//! | `QUEUED` | `RUNNING` | Un worker toma el job (TTR-002) |
//! | `RUNNING` | `COMPLETED` | El job termina con éxito (TTR-002/003) |
//! | `RUNNING` | `FAILED` | El callback del job devuelve o lanza un error (TTR-002) |
//! | `QUEUED` | `CANCELLED` | El usuario cancela un job que no había arrancado (TTR-006) |
//! | `RUNNING` | `CANCELLED` | El usuario cancela un job en ejecución; el worker observa el token de cancelación (TTR-006) |
//! | `RUNNING` | `QUEUED` | Recuperación en startup: un job que estaba `RUNNING` cuando el proceso murió se reencola, porque no se sabe si terminó (TTR-004) |
//!
//! Cualquier otro par `(de, a)` — incluida cualquier transición que salga
//! de un estado terminal — es rechazado por [`validate_transition`].

use std::fmt;

/// Los cinco estados posibles de un job (ADR-0004: estados representados
/// como un conjunto fijo y finito; aquí como un enum de Rust en vez de
/// enteros crudos, porque esta tabla no está en el camino caliente de
/// trading).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JobState {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl JobState {
    /// La cadena exacta que se persiste en la columna `jobs.state`
    /// (migración `0003_jobs.sql`).
    pub fn as_str(&self) -> &'static str {
        match self {
            JobState::Queued => "QUEUED",
            JobState::Running => "RUNNING",
            JobState::Completed => "COMPLETED",
            JobState::Failed => "FAILED",
            JobState::Cancelled => "CANCELLED",
        }
    }

    /// Parsea un valor de la columna `jobs.state` de vuelta a [`JobState`].
    ///
    /// Devuelve `None` para cualquier valor que no sea una de las cinco
    /// cadenas canónicas — la capa de persistencia trata eso como un
    /// error de integridad de datos (una fila escrita fuera de esta
    /// máquina de estados), no como un valor por defecto silencioso.
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "QUEUED" => Some(JobState::Queued),
            "RUNNING" => Some(JobState::Running),
            "COMPLETED" => Some(JobState::Completed),
            "FAILED" => Some(JobState::Failed),
            "CANCELLED" => Some(JobState::Cancelled),
            _ => None,
        }
    }

    /// Un estado terminal nunca es el origen de una transición válida
    /// (async-job-executor.md "Restricciones": "Una vez CANCELLED, no se
    /// puede reanudar"; lo mismo aplica a COMPLETED/FAILED).
    pub fn is_terminal(&self) -> bool {
        matches!(self, JobState::Completed | JobState::Failed | JobState::Cancelled)
    }
}

impl fmt::Display for JobState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Una transición `(from, to)` que [`validate_transition`] rechazó.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTransition {
    pub from: JobState,
    pub to: JobState,
}

impl fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid job state transition: {} -> {}", self.from, self.to)
    }
}

impl std::error::Error for InvalidTransition {}

/// Valida una transición de estado propuesta `(from, to)` contra la tabla
/// del doc comment de este módulo.
///
/// Devuelve `Ok(to)` si la transición está permitida, o
/// [`InvalidTransition`] en caso contrario. Pura: sin I/O, determinista.
pub fn validate_transition(from: JobState, to: JobState) -> Result<JobState, InvalidTransition> {
    let allowed = matches!(
        (from, to),
        (JobState::Queued, JobState::Running)
            | (JobState::Running, JobState::Completed)
            | (JobState::Running, JobState::Failed)
            | (JobState::Queued, JobState::Cancelled)
            | (JobState::Running, JobState::Cancelled)
            | (JobState::Running, JobState::Queued)
    );

    if allowed {
        Ok(to)
    } else {
        Err(InvalidTransition { from, to })
    }
}

/// Porcentaje de progreso, clampeado al rango `0..=100` que exige
/// async-job-executor.md TTR-005 ("Progreso es 0-100%").
///
/// La construcción siempre tiene éxito: los valores fuera de rango se
/// clampean en vez de rechazarse, porque un worker que reporta
/// `progress = 104` por una rareza de redondeo no debería abortar el
/// job — debería quedar registrado como `100`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Progress(u8);

impl Progress {
    /// Clampea `percent` al rango `0..=100`.
    pub fn new(percent: u8) -> Self {
        Progress(percent.min(100))
    }

    pub fn value(&self) -> u8 {
        self.0
    }

    /// `0%` — el valor con el que arranca un job cuando transiciona a
    /// `RUNNING` (TTR-002 "Worker cambia estado a RUNNING, inicializa
    /// progreso=0").
    pub fn zero() -> Self {
        Progress(0)
    }

    /// `100%` — el valor que alcanza un job cuando transiciona a
    /// `COMPLETED`.
    pub fn complete() -> Self {
        Progress(100)
    }
}

impl fmt::Display for Progress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Estima el tiempo restante de un job en ejecución (TTR-005 "Reglas de
/// Negocio": `estimación = (elapsed_time / progress) * (100 - progress)`).
///
/// Devuelve `None` cuando la estimación no se puede calcular:
/// - `progress == 0`: todavía no se observó trabajo, así que el ratio
///   elapsed/progress no está definido (división por cero).
///
/// Devuelve `Some(0)` cuando `progress >= 100` (no queda nada por hacer —
/// la fórmula de TTR-005 da `(elapsed/100) * 0 = 0`, devuelto
/// directamente para evitar cualquier redondeo de punto flotante en el
/// límite).
///
/// `elapsed_seconds` y el resultado se expresan ambos en segundos enteros
/// (ejemplo de TTR-005: `"estimated_time_remaining": "2 minutes"`).
pub fn estimate_remaining_seconds(progress: Progress, elapsed_seconds: u64) -> Option<u64> {
    let percent = progress.value();

    if percent == 0 {
        return None;
    }

    if percent >= 100 {
        return Some(0);
    }

    // remaining = elapsed * (100 - percent) / percent
    let percent = percent as u128;
    let remaining = (elapsed_seconds as u128) * (100 - percent) / percent;

    Some(remaining as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- validate_transition: transiciones permitidas ---------------------

    #[test]
    fn queued_to_running_is_allowed() {
        assert_eq!(
            validate_transition(JobState::Queued, JobState::Running),
            Ok(JobState::Running)
        );
    }

    #[test]
    fn running_to_completed_is_allowed() {
        assert_eq!(
            validate_transition(JobState::Running, JobState::Completed),
            Ok(JobState::Completed)
        );
    }

    #[test]
    fn running_to_failed_is_allowed() {
        assert_eq!(
            validate_transition(JobState::Running, JobState::Failed),
            Ok(JobState::Failed)
        );
    }

    #[test]
    fn queued_to_cancelled_is_allowed() {
        assert_eq!(
            validate_transition(JobState::Queued, JobState::Cancelled),
            Ok(JobState::Cancelled)
        );
    }

    #[test]
    fn running_to_cancelled_is_allowed() {
        assert_eq!(
            validate_transition(JobState::Running, JobState::Cancelled),
            Ok(JobState::Cancelled)
        );
    }

    /// TTR-004: la recuperación en startup reencola un job que estaba
    /// `RUNNING` cuando el proceso murió, porque no se sabe si terminó.
    #[test]
    fn running_to_queued_is_allowed_for_recovery() {
        assert_eq!(
            validate_transition(JobState::Running, JobState::Queued),
            Ok(JobState::Queued)
        );
    }

    // --- validate_transition: transiciones rechazadas ---------------------

    /// Los estados terminales nunca transicionan a ningún lado
    /// (async-job-executor.md: "Una vez CANCELLED, no se puede reanudar").
    #[test]
    fn terminal_states_reject_every_transition() {
        for terminal in [JobState::Completed, JobState::Failed, JobState::Cancelled] {
            for target in [
                JobState::Queued,
                JobState::Running,
                JobState::Completed,
                JobState::Failed,
                JobState::Cancelled,
            ] {
                let result = validate_transition(terminal, target);
                assert!(
                    result.is_err(),
                    "expected {terminal} -> {target} to be rejected, got {result:?}"
                );
            }
        }
    }

    #[test]
    fn queued_to_completed_is_rejected() {
        let result = validate_transition(JobState::Queued, JobState::Completed);
        assert_eq!(
            result,
            Err(InvalidTransition {
                from: JobState::Queued,
                to: JobState::Completed
            })
        );
    }

    #[test]
    fn queued_to_failed_is_rejected() {
        assert!(validate_transition(JobState::Queued, JobState::Failed).is_err());
    }

    #[test]
    fn queued_to_queued_is_rejected() {
        assert!(validate_transition(JobState::Queued, JobState::Queued).is_err());
    }

    #[test]
    fn running_to_running_is_rejected() {
        assert!(validate_transition(JobState::Running, JobState::Running).is_err());
    }

    #[test]
    fn invalid_transition_display_is_human_readable() {
        let err = InvalidTransition {
            from: JobState::Queued,
            to: JobState::Completed,
        };
        assert_eq!(err.to_string(), "invalid job state transition: QUEUED -> COMPLETED");
    }

    // --- JobState: ida y vuelta por su representación en string ----------

    #[test]
    fn job_state_round_trips_through_its_string_representation() {
        for state in [
            JobState::Queued,
            JobState::Running,
            JobState::Completed,
            JobState::Failed,
            JobState::Cancelled,
        ] {
            let s = state.as_str();
            assert_eq!(JobState::from_str_value(s), Some(state));
        }
    }

    #[test]
    fn from_str_value_rejects_unknown_strings() {
        assert_eq!(JobState::from_str_value("BOGUS"), None);
        assert_eq!(JobState::from_str_value(""), None);
        assert_eq!(JobState::from_str_value("queued"), None); // sensible a mayúsculas
    }

    #[test]
    fn is_terminal_matches_the_three_terminal_states() {
        assert!(!JobState::Queued.is_terminal());
        assert!(!JobState::Running.is_terminal());
        assert!(JobState::Completed.is_terminal());
        assert!(JobState::Failed.is_terminal());
        assert!(JobState::Cancelled.is_terminal());
    }

    // --- Progress -----------------------------------------------------------

    #[test]
    fn progress_clamps_values_above_100() {
        assert_eq!(Progress::new(150).value(), 100);
        assert_eq!(Progress::new(255).value(), 100);
    }

    #[test]
    fn progress_accepts_values_within_range() {
        assert_eq!(Progress::new(0).value(), 0);
        assert_eq!(Progress::new(45).value(), 45);
        assert_eq!(Progress::new(100).value(), 100);
    }

    #[test]
    fn progress_zero_and_complete_constants() {
        assert_eq!(Progress::zero().value(), 0);
        assert_eq!(Progress::complete().value(), 100);
    }

    // --- estimate_remaining_seconds (TTR-005) --------------------------------

    /// Ejemplo resuelto de TTR-005: 45% hecho tras cierto tiempo transcurrido
    /// debe dar una estimación restante proporcional a `(100 - 45) / 45`.
    #[test]
    fn estimate_matches_ttr_005_formula() {
        // elapsed=90s al 45% => remaining = 90 * (100-45)/45 = 90 * 55/45 = 110
        let remaining = estimate_remaining_seconds(Progress::new(45), 90);
        assert_eq!(remaining, Some(110));
    }

    #[test]
    fn estimate_is_none_when_progress_is_zero() {
        assert_eq!(estimate_remaining_seconds(Progress::zero(), 100), None);
    }

    #[test]
    fn estimate_is_zero_when_progress_is_complete() {
        assert_eq!(estimate_remaining_seconds(Progress::complete(), 1_000), Some(0));
    }

    #[test]
    fn estimate_at_50_percent_is_equal_to_elapsed() {
        // 50% hecho => remaining == elapsed (punto medio simétrico).
        let remaining = estimate_remaining_seconds(Progress::new(50), 60);
        assert_eq!(remaining, Some(60));
    }

    #[test]
    fn estimate_with_zero_elapsed_is_zero() {
        let remaining = estimate_remaining_seconds(Progress::new(10), 0);
        assert_eq!(remaining, Some(0));
    }
}
