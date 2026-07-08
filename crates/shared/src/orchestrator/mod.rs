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
//! - `central_identity`: caché de identidad con TTL + puerto de
//!   verificación central con stub local (`docs/features/central-identity.md`,
//!   ADR-0143, ADR-0144). STORY-027.
//! - `consent_registry`: composición del puerto `consent_out` -- registrar
//!   un evento de consentimiento y resolver el veredicto de cobertura por
//!   tipo de dato (`docs/features/consent-registry.md`, ADR-0143,
//!   ADR-0144). STORY-031.
//! - `data_aggregation`: composición del flujo completo del cimiento #9 --
//!   gate de consentimiento REAL de `consent-registry` (#5) evento por
//!   evento, delegación al Core (ruido + k-anonimato), separación de
//!   canales interno/externo y persistencia append-only atómica
//!   (`docs/features/data-aggregation.md`, ADR-0144, ADR-0102, ADR-0143).
//!   STORY-036.
//! - `licensing_system`: emisor de licencias de desarrollo (stub Ed25519),
//!   proveedor de límites de plan (stub) y caché con TTL del veredicto de
//!   ejecución (`docs/features/licensing-system.md`, ADR-0143, ADR-0144).
//!   STORY-028.
//! - `instance_continuity`: composición del cimiento #11 -- filtra secretos
//!   del delta a respaldar, cifra con la clave derivada + el nonce
//!   inyectado, persiste el registro de respaldos append-only atómico y el
//!   gate de titularidad exclusiva por `custody_epoch`
//!   (`docs/features/instance-continuity.md`, ADR-0146, ADR-0093).
//!   STORY-039.
//! - `institutional_report_engine`: composición del puerto `report_out` --
//!   lee el reloj inyectado, ensambla el reporte (Core), calcula su firma
//!   reproducible y lo persiste append-only atómico
//!   (`docs/features/institutional-report-engine.md`, ADR-0144). STORY-034.
//! - `job_executor`: la cáscara del Async Job Executor -- pool de workers
//!   de Tokio, cola en memoria, generación de UUID, lecturas de [`Clock`]
//!   y recuperación en startup (`docs/features/async-job-executor.md`
//!   TTR-ASYNC-EXECUTOR-001/002/004/005/006, ADR-0011).
//! - `master_account_hierarchy`: composición del cimiento #12 -- vincula
//!   hija a fondo, emite el override desde el fondo (resuelve el
//!   `consent_out` REAL de `consent-registry` #5, decide y encadena la
//!   fila ISSUER) y lo recibe/ejecuta en la hija (re-valida localmente,
//!   aplica el efecto "eliminar = archivar" y encadena la fila EXECUTOR)
//!   (`docs/features/master-account-hierarchy.md`, ADR-0147, ADR-0093).
//!   STORY-040.
//! - `mcp_server`: servidor MCP sobre stdio (ADR-0123, STORY-010) — expone
//!   las operaciones de `shared` como herramientas MCP y registra cada
//!   decisión de permiso en `permission_decisions`.
//! - `plan_tier_quota`: catálogo de desarrollo (stub, Free/Paid) + caché con
//!   TTL de límites resueltos por tier + composición del puerto
//!   `plan_limits_out` (`docs/features/plan-tier-quota.md`, ADR-0143,
//!   ADR-0144). STORY-029.
//! - `telemetry`: el buffer de alta velocidad -- cola en memoria no
//!   bloqueante + tarea de fondo que vacía a SQLite por lotes
//!   (`docs/features/telemetry.md` TTR-001, ADR-0015).
//! - `third_party_api_gateway`: composición del flujo completo del gateway
//!   -- autentica, cuenta el uso en la ventana vigente, resuelve el
//!   `consent_out` REAL de `consent-registry` (#5) y persiste el registro
//!   de uso (`docs/features/third-party-api-gateway.md`, ADR-0143,
//!   ADR-0144). STORY-035.
//! - `usage_metering`: composición del puerto `usage_out` -- consume el
//!   `PlanLimits` REAL de `plan_tier_quota` (#3) para resolver el
//!   veredicto de cuota de cada operación medida (`docs/features/usage-metering.md`,
//!   ADR-0143, ADR-0144). STORY-030.
//! - `verified_account_registry`: composición del flujo completo del
//!   cimiento #10 -- registrar cuenta (default PRIVATE), calcular y firmar
//!   el track por ámbito de atestación, y el gate de publicación con el
//!   `consent_out` REAL de `consent-registry` (#5)
//!   (`docs/features/verified-account-registry.md`, ADR-0145, ADR-0093).
//!   STORY-037.

pub mod central_identity;
pub mod consent_registry;
pub mod data_aggregation;
pub mod enriched_domain_events;
pub mod instance_continuity;
pub mod institutional_report_engine;
pub mod job_executor;
pub mod licensing_system;
pub mod master_account_hierarchy;
pub mod mcp_server;
pub mod plan_tier_quota;
pub mod telemetry;
pub mod third_party_api_gateway;
pub mod usage_metering;
pub mod verified_account_registry;
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
