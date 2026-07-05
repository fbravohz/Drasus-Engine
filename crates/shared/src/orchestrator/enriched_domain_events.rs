//! [SHELL] Composición del puerto `event_out` para los Eventos de Dominio
//! Enriquecidos (`docs/features/enriched-domain-events.md`, ADR-0144
//! cimiento #6, ADR-0145, ADR-0143, STORY-033).
//!
//! Capa delgada sobre [`crate::persistence::enriched_domain_events::DomainEventRepository`]:
//! recibe un evento del Core + el `ExecutionGate` REAL de `licensing-system`
//! (#2), deriva la decisión de replicación (`decide_replication`) y persiste
//! el evento append-only. Es el mismo rol que cumple
//! `orchestrator::consent_registry::record_consent_action` para el cimiento
//! #5: la composición completa detrás de UNA función.
//!
//! ## Qué es y qué NO es el `replicate` que este orquestador deriva
//!
//! El puerto `gate_in` consume el `ExecutionGate` real (no un stub): su
//! campo `suppress_work_telemetry` ya fue evaluado por
//! `licensing_system::derive_execution_gate` a partir del tier y el estado
//! de heartbeat (ADR-0143). Aquí SOLO se traduce ese veredicto a un flag
//! `replicate` por evento (`decide_replication`) y se persiste junto al
//! evento. **NO hay envío por red**: el fan-out al bus (ADR-0085) y el
//! adaptador de red hacia la Cabina de Mando son futuros diferidos (Orden
//! STORY-033 §8). El evento se persiste SIEMPRE localmente, replique o no.

use sqlx::SqlitePool;

use crate::domain::clock::Clock;
use crate::domain::enriched_domain_events::{decide_replication, EnrichedDomainEvent};
use crate::domain::licensing_system::ExecutionGate;
use crate::persistence::enriched_domain_events::{
    DomainEventRepository, DomainEventRepositoryError, DomainEventRow, RecordDomainEventInput,
};

/// Identidad de emisión de un evento -- el Perfil D de ADR-0020 que la Shell
/// adjunta a cada fila (`owner_id`/`institutional_tag`/`node_id`/`process_id`
/// obligatorios; `session_id` nullable). Se pasa aparte del evento porque el
/// Core (el `EnrichedDomainEvent`) modela solo el CONTENIDO de negocio, no la
/// procedencia de auditoría.
#[derive(Debug, Clone)]
pub struct EventEmissionIdentity {
    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,
    pub process_id: String,
    pub session_id: Option<String>,
}

/// Registra UN evento de dominio enriquecido, derivando la decisión de
/// replicación del `ExecutionGate` REAL de #2.
///
/// Compone la Story completa: recibe el evento (Core) + la identidad de
/// emisión (Perfil D) + el `gate` real (`gate_in`), deriva `replicate` con
/// [`decide_replication`] y delega la persistencia atómica al repositorio.
/// Existe como función de orquestación (en vez de que el llamador use
/// [`DomainEventRepository`] directamente) para que `public_interface` tenga
/// UN punto de entrada estable, igual que
/// `orchestrator::consent_registry::record_consent_action`.
pub async fn record_domain_event(
    pool: &SqlitePool,
    clock: &dyn Clock,
    identity: EventEmissionIdentity,
    gate: &ExecutionGate,
    event: EnrichedDomainEvent,
) -> Result<DomainEventRow, DomainEventRepositoryError> {
    // gate_in: deriva la decisión de replicación del veredicto real de
    // licencia (suprime -> no replica; no suprime -> replica).
    let replicate = decide_replication(gate);

    let repo = DomainEventRepository::new(pool, clock);
    repo.record_event(RecordDomainEventInput {
        owner_id: identity.owner_id,
        institutional_tag: identity.institutional_tag,
        node_id: identity.node_id,
        process_id: identity.process_id,
        session_id: identity.session_id,
        event,
        replicate,
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::domain::enriched_domain_events::{CapitalFlowPayload, CapitalFlowSign};
    use crate::domain::licensing_system::{GateVerdict, LicenseTier};
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    fn sample_identity() -> EventEmissionIdentity {
        EventEmissionIdentity {
            owner_id: "owner-1".to_string(),
            institutional_tag: "DRASUS_LOCAL".to_string(),
            node_id: "node-1".to_string(),
            process_id: "process-1".to_string(),
            session_id: None,
        }
    }

    fn sample_gate(suppress: bool) -> ExecutionGate {
        ExecutionGate {
            verdict: GateVerdict::Allow,
            suppress_work_telemetry: suppress,
            tier: if suppress { LicenseTier::Sovereign } else { LicenseTier::Explorer },
            activations: 1,
            reason: "licencia válida dentro de los límites del plan".to_string(),
        }
    }

    fn sample_event() -> EnrichedDomainEvent {
        EnrichedDomainEvent::CapitalFlow(CapitalFlowPayload {
            account_id: "acc-1".to_string(),
            sign: CapitalFlowSign::Deposit,
            amount: 100_000_000_000,
            currency: "USD".to_string(),
            timestamp_ns: 1_000,
        })
    }

    // ── CRITERIO #5 (Orden §5): decisión de replicación con el gate real ────

    /// CRITERIO DE CIERRE: un gate que suprime telemetría de trabajo
    /// (Sovereign al corriente) hace que el evento se persista con
    /// `replicate = false` -- ejercitando la composición completa
    /// (orchestrator -> Core `decide_replication` -> repo), no solo la
    /// función pura.
    #[tokio::test]
    async fn suppressing_gate_persists_event_with_replicate_false() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);

        let row = record_domain_event(&pool, &clock, sample_identity(), &sample_gate(true), sample_event())
            .await
            .expect("registrar evento");

        assert!(!row.replicate, "gate que suprime -> replicate=false (solo local)");
    }

    /// CRITERIO DE CIERRE: un gate que NO suprime (Explorer/gratuito) hace
    /// que el evento se persista con `replicate = true`.
    #[tokio::test]
    async fn non_suppressing_gate_persists_event_with_replicate_true() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);

        let row = record_domain_event(&pool, &clock, sample_identity(), &sample_gate(false), sample_event())
            .await
            .expect("registrar evento");

        assert!(row.replicate, "gate que no suprime -> replicate=true (replica al proveedor)");
    }
}
