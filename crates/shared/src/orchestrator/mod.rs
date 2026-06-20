//! [SHELL] Orquestación para `shared`.
//!
//! Coordina la lógica de `domain` para los componentes reutilizables
//! (FCIS, ADR-0003).
//!
//! `SystemClock` es la única pieza de `shared` que toca I/O real (el reloj
//! del sistema operativo). Implementa el puerto `Clock` (TTR-001,
//! `docs/features/clock.md`) para uso en producción (`request_type =
//! REAL`).
//!
//! - `job_executor`: la cáscara del Async Job Executor -- pool de workers
//!   de Tokio, cola en memoria, generación de UUID, lecturas de [`Clock`]
//!   y recuperación en startup (`docs/features/async-job-executor.md`
//!   TTR-ASYNC-EXECUTOR-001/002/004/005/006, ADR-0011).
//! - `telemetry`: el buffer de alta velocidad -- cola en memoria no
//!   bloqueante + tarea de fondo que vacía a SQLite por lotes
//!   (`docs/features/telemetry.md` TTR-001, ADR-0015).

pub mod job_executor;
pub mod telemetry;
pub mod worker_runner;

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::clock::Clock;

/// Implementación de producción del puerto [`Clock`] (TTR-001).
///
/// Envuelve `SystemTime::now()` y lo convierte a nanosegundos desde el
/// Unix epoch (TTR-001: "En producción, utiliza `time.time_ns()` para
/// evitar errores de precisión de punto flotante.").
///
/// `SystemTime` en sí NO garantiza ser monótono entre llamadas (un ajuste
/// NTP puede mover el reloj de pared hacia atrás). Para sostener el
/// invariante del puerto Clock — "NUNCA Clock devuelve un valor menor al
/// anterior" — esta implementación recuerda el último timestamp que
/// devolvió y clampea cualquier lectura nueva para que sea estrictamente
/// mayor que esa.
pub struct SystemClock {
    last_timestamp_ns: AtomicI64,
}

impl SystemClock {
    /// Crea un nuevo `SystemClock`. La primera llamada a
    /// [`Clock::timestamp_ns`] devuelve la hora de pared actual.
    pub fn new() -> Self {
        Self {
            last_timestamp_ns: AtomicI64::new(i64::MIN),
        }
    }

    /// Lee la hora de pared actual como nanosegundos desde el Unix epoch.
    /// Solo entra en panic si el reloj del sistema está fijado antes del
    /// Unix epoch (1970-01-01), un despliegue que no se soporta.
    fn read_system_time_ns() -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is set before the Unix epoch");

        now.as_nanos()
            .try_into()
            .expect("system time in nanoseconds overflows i64")
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn timestamp_ns(&self) -> i64 {
        let observed_ns = Self::read_system_time_ns();

        // Fuerza tiempo monótono no decreciente incluso si el reloj del SO
        // salta hacia atrás (ej. corrección NTP): nunca devuelve un valor
        // menor (o igual, entre llamadas consecutivas) al anterior.
        let mut previous = self.last_timestamp_ns.load(Ordering::SeqCst);
        loop {
            // Estrictamente mayor que el valor anterior, pero por lo
            // demás la hora real observada (sin drift artificial cuando
            // el reloj del SO ya va adelante).
            let next = observed_ns.max(previous + 1);

            match self.last_timestamp_ns.compare_exchange(
                previous,
                next,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return next,
                Err(actual) => previous = actual,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_ns_is_monotonically_increasing() {
        let clock = SystemClock::new();

        let first = clock.timestamp_ns();
        let second = clock.timestamp_ns();
        let third = clock.timestamp_ns();

        assert!(second >= first);
        assert!(third >= second);
    }

    #[test]
    fn timestamp_ns_is_positive_and_plausible() {
        let clock = SystemClock::new();
        let ts = clock.timestamp_ns();

        // Cota de cordura: cualquier timestamp posterior a 2020-01-01 (en nanosegundos).
        assert!(ts > 1_577_836_800_000_000_000);
    }
}
