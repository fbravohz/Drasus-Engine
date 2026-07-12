//! [SHELL] Composición del flujo completo de Data Anonymization &
//! Aggregation (`docs/features/data-aggregation.md`, ADR-0144 cimiento
//! #9, ADR-0102, ADR-0143, ADR-0141, ADR-0020, ADR-0093, STORY-036).
//!
//! Orquesta, por cada evento candidato: (1) consulta el `consent_out`
//! REAL de `consent-registry` (#5) vía
//! [`crate::orchestrator::consent_registry::resolve_consent_verdict`] --
//! nunca un stub -- y excluye los eventos sin cobertura/opt-out; (2)
//! hashea cualquier topología cruda de inmediato (ADR-0102) y descarta el
//! texto crudo; (3) delega al Core ([`aggregate_index`]) la suma de los
//! valores cubiertos, el ruido de privacidad diferencial y la
//! verificación de k-anonimato; (4) si el canal pedido es `EXTERNAL` y la
//! venta externa está deshabilitada, NUNCA produce ese agregado (regla
//! obligatoria #7, ADR-0143); (5) si el Core publica (no suprime por
//! cohorte insuficiente), persiste el snapshot append-only atómico vía
//! [`AggregatedIndexRepository`].
//!
//! Es el mismo rol que cumple
//! `orchestrator::third_party_api_gateway::handle_gateway_request` para el
//! cimiento #8: la composición completa detrás de UNA función, para que
//! `public_interface` tenga un único punto de entrada estable.

use sqlx::SqlitePool;

use crate::domain::clock::Clock;
use crate::domain::data_aggregation::{aggregate_index, hash_strategy_topology, Channel, IndexType};
use crate::orchestrator::consent_registry::resolve_consent_verdict;
use crate::persistence::consent_registry::ConsentRepositoryError;
use crate::persistence::data_aggregation::{
    AggregatedIndexRepository, AggregatedIndexRepositoryError, AggregatedIndexRow, RecordAggregatedIndexInput,
};

/// Tipo de dato consultado en `consent-registry` (#5) para el gate de esta
/// feature -- el mismo vocabulario que usa el `consent_data_type` en los
/// tests de `domain::consent_registry` y en el harness CLI.
pub const DATA_AGGREGATION_CONSENT_DATA_TYPE: &str = "aggregation";

/// Error de orquestación de esta feature -- envuelve los dos puntos de
/// fallo posibles: resolver el veredicto de consentimiento (I/O contra
/// `consent_records`) y persistir el índice agregado (I/O contra
/// `aggregated_indexes`).
#[derive(Debug, thiserror::Error)]
pub enum DataAggregationError {
    #[error("error al resolver el veredicto de consentimiento: {0}")]
    Consent(#[from] ConsentRepositoryError),
    #[error("error al persistir el índice agregado: {0}")]
    Persistence(#[from] AggregatedIndexRepositoryError),
}

/// Un evento candidato a un agregado -- lo que la Shell/CLI arma por cada
/// evento de ejecución enriquecido (#6) + el `owner_id` de su
/// contribuyente, ANTES de resolver el consentimiento real.
#[derive(Debug, Clone)]
pub struct AggregationEventInput {
    /// Dueño del evento fuente -- se usa SOLO para consultar
    /// `consent_out`; nunca se persiste en la fila del agregado (ver
    /// `RecordAggregatedIndexInput::owner_id`, que identifica al
    /// AGREGADOR, no al contribuyente).
    pub owner_id: String,
    /// Métrica cruda de este evento, entero ×10⁸ (ADR-0141).
    pub metric_e8: i64,
    /// Firma cruda de topología de estrategia (fórmula/parámetros) que
    /// pudo participar en este evento -- NUNCA se persiste tal cual; se
    /// hashea de inmediato (ADR-0102) y el texto crudo se descarta apenas
    /// se usa (ver [`Self`] uso en [`run_aggregation`]).
    pub raw_topology: Option<String>,
}

/// Parámetros de configuración de UNA corrida de agregación (`docs/features/
/// data-aggregation.md` "Parámetros Configurables" + "Persistencia").
#[derive(Debug, Clone)]
pub struct AggregationRunConfig {
    pub index_type: IndexType,
    pub time_window: String,
    pub channel: Channel,
    /// `MIN_COHORT_SIZE` -- FIJO en el diseño de la Feature, pero se
    /// inyecta aquí para que el Core sea puro (`meets_k_anonymity` no lee
    /// ninguna constante global).
    pub min_cohort: i64,
    /// `DP_NOISE_LEVEL`, entero ×10⁸.
    pub noise_level_e8: i64,
    /// Semilla del RNG de privacidad diferencial -- INYECTADA, nunca
    /// generada por este módulo (ADR-0002/0004).
    pub seed: u64,
    /// Versión de ToS vigente contra la que se resuelve la cobertura de
    /// cada evento.
    pub consent_version: String,
    /// `EXTERNAL_SALE_ENABLED` -- si `false`, NUNCA se produce un
    /// agregado de canal `EXTERNAL` (regla obligatoria #7).
    pub external_sale_enabled: bool,
    /// Dueño del artefacto derivado (el proceso/agregador), NUNCA un
    /// contribuyente individual.
    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,
    pub data_snapshot_id: Option<String>,
}

/// Resultado observable de UNA corrida de agregación (`docs/features/
/// data-aggregation.md` "Comportamientos Observables").
#[derive(Debug, Clone)]
pub enum AggregationOutcome {
    /// El índice se publicó y persistió como fila append-only. En `Box`
    /// porque `AggregatedIndexRow` es sensiblemente más grande que las
    /// otras dos variantes (que no cargan datos) -- sin la indirección,
    /// clippy señala el desperdicio de espacio que impondría ese tamaño a
    /// TODAS las instancias del enum, incluidas las variantes vacías.
    Published(Box<AggregatedIndexRow>),
    /// La cohorte CUBIERTA (tras excluir no-consentidos/opt-out) no
    /// alcanzó `MIN_COHORT_SIZE` -- supresión por k-anonimato, ninguna
    /// fila se persiste.
    SuppressedByCohortSize,
    /// El canal pedido era `EXTERNAL` pero `EXTERNAL_SALE_ENABLED=false`
    /// -- NUNCA se produce un agregado de venta externa en ese caso
    /// (regla obligatoria #7, ADR-0143). El canal `INTERNAL` no pasa por
    /// esta puerta.
    ExternalChannelDisabled,
}

/// Ejecuta el flujo completo de agregación descrito en el doc-comment del
/// módulo. `events` puede venir de CUALQUIER mezcla de contribuyentes
/// cubiertos y no cubiertos -- esta función es la que decide, evento por
/// evento, cuáles entran a la suma.
pub async fn run_aggregation(
    pool: &SqlitePool,
    clock: &dyn Clock,
    events: &[AggregationEventInput],
    config: &AggregationRunConfig,
) -> Result<AggregationOutcome, DataAggregationError> {
    // Separación de canales (regla obligatoria #7, ADR-0143) -- PRIMERO,
    // antes de gastar ninguna consulta de consentimiento: un canal
    // EXTERNAL pedido con la venta externa apagada nunca se calcula.
    if matches!(config.channel, Channel::External) && !config.external_sale_enabled {
        return Ok(AggregationOutcome::ExternalChannelDisabled);
    }

    let mut covered_values_e8 = Vec::new();
    for event in events {
        // Hash unidireccional de topología (ADR-0102) -- el texto crudo
        // (`raw`) sale de alcance apenas termina esta línea; solo el hash
        // sobrevive (aquí no se usa el hash más allá de sanear el dato de
        // entrada, porque esta corrida no persiste ninguna columna de
        // topología -- el guardarraíl de "datos crudos nunca salen" exige
        // que el crudo nunca SOBREVIVA, no que su hash se use en un lugar
        // específico).
        if let Some(raw) = &event.raw_topology {
            let _ = hash_strategy_topology(raw);
        }

        // Gate de consentimiento REAL de #5 (default-deny GDPR) -- NUNCA
        // un stub. Un evento sin cobertura vigente, o con opt-out
        // explícito del tipo de dato de agregación, se EXCLUYE de la
        // suma.
        let verdict = resolve_consent_verdict(
            pool,
            clock,
            &event.owner_id,
            DATA_AGGREGATION_CONSENT_DATA_TYPE,
            &config.consent_version,
        )
        .await?;

        if verdict.is_covered() {
            covered_values_e8.push(event.metric_e8);
        }
    }

    // El Core puro decide: suma + ruido + verificación de k-anonimato.
    let aggregated = aggregate_index(
        &covered_values_e8,
        config.index_type,
        &config.time_window,
        config.channel,
        config.min_cohort,
        config.noise_level_e8,
        config.seed,
    );

    match aggregated {
        None => Ok(AggregationOutcome::SuppressedByCohortSize),
        Some(index) => {
            let repo = AggregatedIndexRepository::new(pool, clock);
            let row = repo
                .record_index(RecordAggregatedIndexInput {
                    owner_id: config.owner_id.clone(),
                    institutional_tag: config.institutional_tag.clone(),
                    node_id: config.node_id.clone(),
                    data_snapshot_id: config.data_snapshot_id.clone(),
                    index,
                })
                .await?;
            Ok(AggregationOutcome::Published(Box::new(row)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::domain::consent_registry::ConsentAction;
    use crate::orchestrator::consent_registry::record_consent_action;
    use crate::persistence::central_identity::test_support::seed_account;
    use crate::persistence::consent_registry::RecordConsentActionInput;
    use crate::persistence::pool::{connect, migrate};
    use std::collections::BTreeMap;

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    /// Siembra tres cuentas reales y devuelve sus `owner_id` (== `id`) --
    /// ADR-0141 enmienda 2026-07-11: `consent_records.owner_id` ahora tiene
    /// FK física a `accounts(id)`, así que los contribuyentes de la cohorte
    /// que registran consentimiento necesitan una cuenta real, no un
    /// literal como "owner-1".
    async fn seed_three_owners(pool: &SqlitePool, clock: &dyn Clock) -> (String, String, String) {
        let owner_1 = seed_account(pool, clock, "owner1@example.com").await;
        let owner_2 = seed_account(pool, clock, "owner2@example.com").await;
        let owner_3 = seed_account(pool, clock, "owner3@example.com").await;
        (owner_1, owner_2, owner_3)
    }

    fn base_config(channel: Channel, external_sale_enabled: bool) -> AggregationRunConfig {
        AggregationRunConfig {
            index_type: IndexType::Sentiment,
            time_window: "2026-W27".to_string(),
            channel,
            min_cohort: 3,
            noise_level_e8: 1_000_000,
            seed: 42,
            consent_version: "v2".to_string(),
            external_sale_enabled,
            owner_id: "drasus-aggregator".to_string(),
            institutional_tag: "DRASUS_LOCAL".to_string(),
            node_id: "node-1".to_string(),
            data_snapshot_id: Some("snapshot-1".to_string()),
        }
    }

    /// Registra una cobertura `Accept` para `owner_id` sobre
    /// `DATA_AGGREGATION_CONSENT_DATA_TYPE`, con `opted_out` decidiendo si
    /// el tipo de dato queda en opt-out.
    async fn cover_owner(pool: &SqlitePool, clock: &dyn Clock, owner_id: &str, version: &str, opted_out: bool) {
        let mut optout_changes = BTreeMap::new();
        optout_changes.insert(DATA_AGGREGATION_CONSENT_DATA_TYPE.to_string(), opted_out);

        record_consent_action(
            pool,
            clock,
            RecordConsentActionInput {
                owner_id: owner_id.to_string(),
                institutional_tag: "DRASUS_LOCAL".to_string(),
                node_id: "node-1".to_string(),
                compliance_status_id: None,
                action: ConsentAction::Accept,
                tos_version: Some(version.to_string()),
                optout_changes,
            },
        )
        .await
        .expect("registrar consentimiento");
    }

    // ── CRITERIO #4 (Orden §5): gate de consentimiento REAL de #5 ───────────

    /// CRITERIO DE CIERRE: un evento sin NINGÚN consentimiento registrado
    /// se EXCLUYE de la cohorte -- debe fallar si el veredicto real no se
    /// consultara (ej. con un stub que siempre cubre).
    #[tokio::test]
    async fn run_aggregation_excludes_events_without_any_consent() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);

        // Ningún owner tiene consentimiento registrado -- todos deben
        // excluirse, la cohorte cubierta queda en 0.
        let events = vec![
            AggregationEventInput { owner_id: "owner-1".to_string(), metric_e8: 100, raw_topology: None },
            AggregationEventInput { owner_id: "owner-2".to_string(), metric_e8: 200, raw_topology: None },
            AggregationEventInput { owner_id: "owner-3".to_string(), metric_e8: 300, raw_topology: None },
        ];

        let outcome = run_aggregation(&pool, &clock, &events, &base_config(Channel::Internal, false))
            .await
            .expect("la corrida debe tener éxito");

        assert!(
            matches!(outcome, AggregationOutcome::SuppressedByCohortSize),
            "sin ningún consentimiento, la cohorte cubierta es 0 -- debe suprimirse por k-anonimato"
        );
    }

    /// CRITERIO DE CIERRE: un evento con opt-out explícito se EXCLUYE,
    /// aunque haya aceptado la versión vigente del ToS -- debe fallar si
    /// el opt-out no se respetara.
    #[tokio::test]
    async fn run_aggregation_excludes_events_with_explicit_optout() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let (owner_1, owner_2, owner_3) = seed_three_owners(&pool, &clock).await;

        // owner-1 y owner-2 cubiertos; owner-3 en opt-out explícito.
        cover_owner(&pool, &clock, &owner_1, "v2", false).await;
        cover_owner(&pool, &clock, &owner_2, "v2", false).await;
        cover_owner(&pool, &clock, &owner_3, "v2", true).await;

        let events = vec![
            AggregationEventInput { owner_id: owner_1.clone(), metric_e8: 100_000_000, raw_topology: None },
            AggregationEventInput { owner_id: owner_2.clone(), metric_e8: 200_000_000, raw_topology: None },
            AggregationEventInput { owner_id: owner_3.clone(), metric_e8: 999_999_999, raw_topology: None },
        ];

        // min_cohort = 3, pero owner-3 queda excluido -> cohorte cubierta
        // real = 2 -> se suprime. Esto también demuestra que el opt-out
        // realmente redujo la cohorte (si no se respetara, cohort_size
        // sería 3 y publicaría).
        let outcome = run_aggregation(&pool, &clock, &events, &base_config(Channel::Internal, false))
            .await
            .expect("la corrida debe tener éxito");

        assert!(
            matches!(outcome, AggregationOutcome::SuppressedByCohortSize),
            "con el opt-out de owner-3 respetado, la cohorte cubierta cae a 2 (< min_cohort=3) -- debe suprimirse"
        );
    }

    /// CRITERIO DE CIERRE: eventos TODOS cubiertos (versión vigente
    /// aceptada, sin opt-out) se INCLUYEN y, con cohorte suficiente,
    /// publican -- camino feliz completo usando el veredicto real.
    #[tokio::test]
    async fn run_aggregation_includes_covered_events_and_publishes_with_sufficient_cohort() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let (owner_1, owner_2, owner_3) = seed_three_owners(&pool, &clock).await;

        for owner in [&owner_1, &owner_2, &owner_3] {
            cover_owner(&pool, &clock, owner, "v2", false).await;
        }

        let events = vec![
            AggregationEventInput { owner_id: owner_1.clone(), metric_e8: 100_000_000, raw_topology: None },
            AggregationEventInput { owner_id: owner_2.clone(), metric_e8: 200_000_000, raw_topology: None },
            AggregationEventInput { owner_id: owner_3.clone(), metric_e8: 300_000_000, raw_topology: None },
        ];

        let outcome = run_aggregation(&pool, &clock, &events, &base_config(Channel::Internal, false))
            .await
            .expect("la corrida debe tener éxito");

        match outcome {
            AggregationOutcome::Published(row) => {
                assert_eq!(row.cohort_size, 3, "los tres eventos cubiertos deben entrar a la cohorte");
                assert_eq!(row.channel, Channel::Internal);
            }
            other => panic!("se esperaba Published, se obtuvo {other:?}"),
        }
    }

    // ── CRITERIO #6 (Orden §5): separación de canales ────────────────────────

    /// CRITERIO DE CIERRE: `EXTERNAL_SALE_ENABLED=false` -- NUNCA se
    /// produce un agregado de canal `EXTERNAL`, ni siquiera con cohorte
    /// de sobra y todos cubiertos.
    #[tokio::test]
    async fn run_aggregation_never_produces_external_channel_when_sale_disabled() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let (owner_1, owner_2, owner_3) = seed_three_owners(&pool, &clock).await;

        for owner in [&owner_1, &owner_2, &owner_3] {
            cover_owner(&pool, &clock, owner, "v2", false).await;
        }

        let events = vec![
            AggregationEventInput { owner_id: owner_1.clone(), metric_e8: 100_000_000, raw_topology: None },
            AggregationEventInput { owner_id: owner_2.clone(), metric_e8: 200_000_000, raw_topology: None },
            AggregationEventInput { owner_id: owner_3.clone(), metric_e8: 300_000_000, raw_topology: None },
        ];

        let outcome = run_aggregation(&pool, &clock, &events, &base_config(Channel::External, false))
            .await
            .expect("la corrida debe tener éxito");

        assert!(
            matches!(outcome, AggregationOutcome::ExternalChannelDisabled),
            "EXTERNAL_SALE_ENABLED=false debe impedir CUALQUIER agregado EXTERNAL"
        );
    }

    /// El canal `INTERNAL` NUNCA pasa por la puerta de venta externa --
    /// se produce normalmente aunque `external_sale_enabled=false`.
    #[tokio::test]
    async fn run_aggregation_produces_internal_channel_regardless_of_external_sale_flag() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let (owner_1, owner_2, owner_3) = seed_three_owners(&pool, &clock).await;

        for owner in [&owner_1, &owner_2, &owner_3] {
            cover_owner(&pool, &clock, owner, "v2", false).await;
        }

        let events = vec![
            AggregationEventInput { owner_id: owner_1.clone(), metric_e8: 100_000_000, raw_topology: None },
            AggregationEventInput { owner_id: owner_2.clone(), metric_e8: 200_000_000, raw_topology: None },
            AggregationEventInput { owner_id: owner_3.clone(), metric_e8: 300_000_000, raw_topology: None },
        ];

        let outcome = run_aggregation(&pool, &clock, &events, &base_config(Channel::Internal, false))
            .await
            .expect("la corrida debe tener éxito");

        assert!(
            matches!(outcome, AggregationOutcome::Published(_)),
            "el canal INTERNAL debe producirse sin importar EXTERNAL_SALE_ENABLED"
        );
    }

    /// Con `EXTERNAL_SALE_ENABLED=true` y cohorte cubierta suficiente, el
    /// canal `EXTERNAL` sí se persiste con `channel = EXTERNAL`.
    #[tokio::test]
    async fn run_aggregation_produces_external_channel_when_sale_enabled_and_covered() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let (owner_1, owner_2, owner_3) = seed_three_owners(&pool, &clock).await;

        for owner in [&owner_1, &owner_2, &owner_3] {
            cover_owner(&pool, &clock, owner, "v2", false).await;
        }

        let events = vec![
            AggregationEventInput { owner_id: owner_1.clone(), metric_e8: 100_000_000, raw_topology: None },
            AggregationEventInput { owner_id: owner_2.clone(), metric_e8: 200_000_000, raw_topology: None },
            AggregationEventInput { owner_id: owner_3.clone(), metric_e8: 300_000_000, raw_topology: None },
        ];

        let outcome = run_aggregation(&pool, &clock, &events, &base_config(Channel::External, true))
            .await
            .expect("la corrida debe tener éxito");

        match outcome {
            AggregationOutcome::Published(row) => assert_eq!(row.channel, Channel::External),
            other => panic!("se esperaba Published con channel EXTERNAL, se obtuvo {other:?}"),
        }
    }

    // ── Topología cruda: se hashea, nunca sobrevive al alcance del bucle ────

    /// La topología cruda que trae un evento no impide el flujo normal --
    /// se hashea y se descarta dentro del mismo bucle, sin afectar el
    /// resultado de la agregación (regresión de forma: pasar
    /// `raw_topology` no debe cambiar el resto del comportamiento).
    #[tokio::test]
    async fn run_aggregation_accepts_raw_topology_without_changing_the_outcome() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let (owner_1, owner_2, owner_3) = seed_three_owners(&pool, &clock).await;

        for owner in [&owner_1, &owner_2, &owner_3] {
            cover_owner(&pool, &clock, owner, "v2", false).await;
        }

        let events = vec![
            AggregationEventInput {
                owner_id: owner_1.clone(),
                metric_e8: 100_000_000,
                raw_topology: Some("RSI(14)+MACD(12,26,9)".to_string()),
            },
            AggregationEventInput { owner_id: owner_2.clone(), metric_e8: 200_000_000, raw_topology: None },
            AggregationEventInput { owner_id: owner_3.clone(), metric_e8: 300_000_000, raw_topology: None },
        ];

        let outcome = run_aggregation(&pool, &clock, &events, &base_config(Channel::Internal, false))
            .await
            .expect("la corrida debe tener éxito");

        assert!(matches!(outcome, AggregationOutcome::Published(_)));
    }
}
