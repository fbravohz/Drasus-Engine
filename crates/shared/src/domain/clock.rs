//! [CORE] Primitivas puras de reloj — el puerto `Clock` y la implementación
//! de reloj determinista (lista para backtest).
//!
//! Sin I/O, sin acceso al reloj de sistema, sin azar sin semilla
//! (ADR-0002/0004). `docs/features/clock.md`:
//! - TTR-001: puerto de timestamp con precisión de nanosegundos
//!   (`Clock::timestamp_ns`).
//! - TTR-002: reloj determinista para simulaciones reproducibles
//!   (`DeterministicClock`), que solo avanza mediante llamadas explícitas
//!   a `advance(ns)`.

use std::sync::atomic::{AtomicI64, Ordering};

/// El puerto Clock (TTR-001).
///
/// Cualquier módulo que necesite la hora actual llama a este puerto en vez
/// de ir directo al reloj de sistema (`docs/features/clock.md`: "NUNCA un
/// módulo llama a `datetime.now()` o equivalente directo. Siempre a través
/// de Clock.").
///
/// Implementaciones:
/// - [`super::super::orchestrator::SystemClock`] (cáscara): envuelve el
///   reloj real del sistema para producción (`request_type = REAL`).
/// - [`DeterministicClock`] (core, este módulo): reloj totalmente
///   controlado y reproducible para backtests y tests (`request_type =
///   FAKE`).
///
/// Invariante (clock.md "Restricciones"): una implementación de `Clock`
/// NUNCA devuelve un `timestamp_ns` menor a uno ya devuelto antes en la
/// misma instancia — el tiempo es monótono no decreciente.
pub trait Clock: Send + Sync {
    /// Devuelve el timestamp actual en nanosegundos desde el Unix epoch
    /// (TTR-001: "Expone el Unix timestamp actual con precisión de
    /// nanosegundos").
    fn timestamp_ns(&self) -> i64;
}

/// Reloj determinista, listo para backtest (TTR-002).
///
/// El reloj arranca en `initial_timestamp_ns` y solo avanza mediante
/// llamadas explícitas a [`DeterministicClock::advance`]. Llamadas
/// repetidas a [`Clock::timestamp_ns`] entre dos `advance` devuelven
/// exactamente el mismo valor — esto es lo que hace que un backtest sea
/// reproducible bit a bit (clock.md: "Todas las llamadas a Clock dentro
/// de la barra devuelven el mismo timestamp").
///
/// `step_ns` es el paso por defecto configurado (clock.md
/// `ADVANCE_PER_STEP`, en nanosegundos) que usa [`DeterministicClock::tick`]
/// para avanzar un paso de simulación (ej. una barra) sin que quien llama
/// tenga que repetir el tamaño del paso en cada llamada.
pub struct DeterministicClock {
    /// Timestamp virtual actual, en nanosegundos desde el Unix epoch.
    virtual_timestamp_ns: AtomicI64,
    /// Avance por defecto por paso, en nanosegundos (clock.md `ADVANCE_PER_STEP`).
    step_ns: i64,
}

impl DeterministicClock {
    /// Crea un reloj determinista que arranca en `initial_timestamp_ns`
    /// (clock.md `INITIAL_TIMESTAMP` / TTR-002 entrada) con un avance por
    /// defecto de `step_ns` por paso (clock.md `ADVANCE_PER_STEP`).
    ///
    /// `step_ns` debe ser `>= 0` (clock.md: "NUNCA Clock devuelve un valor
    /// menor al anterior" — un paso por defecto negativo haría que `tick`
    /// mueva el tiempo hacia atrás).
    pub fn new(initial_timestamp_ns: i64, step_ns: i64) -> Self {
        assert!(
            step_ns >= 0,
            "DeterministicClock::new: step_ns must be >= 0 (got {step_ns}); \
             a negative step would violate the monotonic-time invariant"
        );

        Self {
            virtual_timestamp_ns: AtomicI64::new(initial_timestamp_ns),
            step_ns,
        }
    }

    /// Avanza el reloj virtual exactamente `delta_ns` nanosegundos
    /// (TTR-002: "El reloj solo avanza mediante llamadas explícitas
    /// `advance(ns)`."). Devuelve el nuevo `virtual_timestamp_ns`.
    ///
    /// `delta_ns` debe ser `>= 0` (clock.md: "NUNCA Clock devuelve un valor
    /// menor al anterior. El tiempo es monótono creciente dentro de una
    /// sesión.").
    pub fn advance(&self, delta_ns: i64) -> i64 {
        assert!(
            delta_ns >= 0,
            "DeterministicClock::advance: delta_ns must be >= 0 (got \
             {delta_ns}); time must be monotonically non-decreasing"
        );

        self.virtual_timestamp_ns
            .fetch_add(delta_ns, Ordering::SeqCst)
            + delta_ns
    }

    /// Avanza el reloj virtual el `step_ns` configurado (clock.md
    /// `ADVANCE_PER_STEP`), ej. una vez por barra simulada. Devuelve el
    /// nuevo `virtual_timestamp_ns`.
    pub fn tick(&self) -> i64 {
        self.advance(self.step_ns)
    }

    /// El avance por paso configurado, en nanosegundos (clock.md
    /// `ADVANCE_PER_STEP`).
    pub fn step_ns(&self) -> i64 {
        self.step_ns
    }
}

impl Clock for DeterministicClock {
    fn timestamp_ns(&self) -> i64 {
        self.virtual_timestamp_ns.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Criterio de cierre de TTR-002: determinismo bit a bit. Dos
    /// instancias independientes de `DeterministicClock`, construidas con
    /// la misma semilla (`initial_timestamp_ns`, `step_ns`) y conducidas
    /// por la misma secuencia de llamadas `advance`/`tick`, deben producir
    /// la EXACTA misma secuencia de valores `timestamp_ns()`, en cada
    /// ejecución.
    #[test]
    fn deterministic_clock_same_seed_produces_identical_sequence() {
        const INITIAL_TIMESTAMP_NS: i64 = 1_577_869_800_000_000_000; // 2020-01-01 09:30:00 UTC
        const STEP_NS: i64 = 60_000_000_000; // 60 seconds, in nanoseconds (1m timeframe)

        let run = || {
            let clock = DeterministicClock::new(INITIAL_TIMESTAMP_NS, STEP_NS);
            let mut sequence = Vec::new();

            // Lectura inicial, antes de cualquier advance.
            sequence.push(clock.timestamp_ns());

            // Simula 5 barras: cada `tick()` avanza exactamente `step_ns`,
            // y lecturas repetidas dentro de una "barra" devuelven el
            // mismo valor.
            for _ in 0..5 {
                clock.tick();
                sequence.push(clock.timestamp_ns());
                sequence.push(clock.timestamp_ns()); // misma barra, mismo timestamp
            }

            // Un advance explícito y no uniforme (ej. un salto manual).
            clock.advance(3_600_000_000_000); // +1 hora
            sequence.push(clock.timestamp_ns());

            sequence
        };

        let sequence_a = run();
        let sequence_b = run();

        assert_eq!(
            sequence_a, sequence_b,
            "same seed (initial_timestamp_ns, step_ns) and same call \
             sequence must produce an identical timestamp sequence, bit \
             for bit"
        );

        // Verificación de cordura sobre los valores esperados, para evitar
        // una comparación vacuamente verdadera (ej. ambas secuencias
        // vacías).
        assert_eq!(sequence_a.len(), 1 + 5 * 2 + 1);
        assert_eq!(sequence_a[0], INITIAL_TIMESTAMP_NS);
        assert_eq!(
            sequence_a[1],
            INITIAL_TIMESTAMP_NS + STEP_NS,
            "first tick must advance by exactly step_ns"
        );
        assert_eq!(
            sequence_a[1], sequence_a[2],
            "repeated reads within the same simulated bar must be identical"
        );
        let last_tick_value = INITIAL_TIMESTAMP_NS + STEP_NS * 5;
        assert_eq!(
            sequence_a[11],
            last_tick_value + 3_600_000_000_000,
            "final explicit advance must apply on top of the last tick"
        );
    }

    #[test]
    fn timestamp_ns_does_not_change_without_advance() {
        let clock = DeterministicClock::new(1_000, 100);

        let first = clock.timestamp_ns();
        let second = clock.timestamp_ns();
        let third = clock.timestamp_ns();

        assert_eq!(first, 1_000);
        assert_eq!(first, second);
        assert_eq!(second, third);
    }

    #[test]
    fn advance_is_monotonically_non_decreasing() {
        let clock = DeterministicClock::new(0, 0);

        let t1 = clock.advance(500);
        let t2 = clock.advance(0); // zero-delta advance is allowed
        let t3 = clock.advance(250);

        assert_eq!(t1, 500);
        assert_eq!(t2, 500);
        assert_eq!(t3, 750);
        assert!(t2 >= t1);
        assert!(t3 >= t2);
    }

    #[test]
    #[should_panic(expected = "delta_ns must be >= 0")]
    fn advance_rejects_negative_delta() {
        let clock = DeterministicClock::new(0, 0);
        clock.advance(-1);
    }

    #[test]
    #[should_panic(expected = "step_ns must be >= 0")]
    fn new_rejects_negative_step() {
        let _ = DeterministicClock::new(0, -1);
    }

    #[test]
    fn tick_advances_by_configured_step() {
        let clock = DeterministicClock::new(1_000, 60);

        assert_eq!(clock.tick(), 1_060);
        assert_eq!(clock.tick(), 1_120);
        assert_eq!(clock.step_ns(), 60);
    }
}
