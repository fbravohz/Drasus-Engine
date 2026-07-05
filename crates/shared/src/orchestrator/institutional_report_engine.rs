//! [SHELL] Composición del puerto `report_out` del Motor de Reportes
//! Institucionales (`docs/features/institutional-report-engine.md`,
//! ADR-0144 cimiento #7, STORY-034).
//!
//! Coordina el Core ([`crate::domain::institutional_report_engine`]) con la
//! persistencia ([`crate::persistence::institutional_report_engine`]): lee
//! el reloj inyectado, ensambla el reporte, calcula su firma reproducible
//! y lo persiste append-only atómico. El reporte NUNCA muta los datos
//! fuente -- esta función solo los presenta y los archiva.

use sqlx::SqlitePool;

use crate::domain::clock::Clock;
use crate::domain::institutional_report_engine::{
    assemble_report, compute_report_signature, AssembleReportInput,
};
use crate::persistence::institutional_report_engine::{
    GeneratedReportRepository, GeneratedReportRepositoryError, GeneratedReportRow,
    RecordGeneratedReportInput,
};

/// Identidad del dueño/máquina que genera este reporte (Perfil D, ADR-0020
/// Grupo II + IV) -- mismo patrón que
/// `enriched_domain_events::EventEmissionIdentity`.
#[derive(Debug, Clone)]
pub struct ReportGenerationIdentity {
    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,
    /// Veredicto de cumplimiento vigente al momento de generar el reporte
    /// (nullable -- no todo reporte trae uno anotado).
    pub compliance_status_id: Option<String>,
}

/// Error de la composición completa -- envuelve el error tipado del
/// repositorio para que el llamador (CLI, futuro adaptador de producto) no
/// tenga que conocer los internals de persistencia.
#[derive(Debug, thiserror::Error)]
pub enum GenerateReportError {
    #[error("fallo al persistir el reporte: {0}")]
    Repository(#[from] GeneratedReportRepositoryError),
}

/// Ensambla, firma y persiste UN reporte institucional -- la composición
/// completa del puerto `report_out`.
///
/// Pasos:
/// 1. Lee el reloj inyectado (`clock.timestamp_ns()`) y lo coloca en
///    `input.generated_at_ns` ANTES de llamar al Core -- el Core en sí no
///    toca el reloj (ADR-0002/0004), pero el reporte necesita saber cuándo
///    se generó.
/// 2. Ensambla el reporte vía [`assemble_report`] (Core, puro).
/// 3. Calcula la firma reproducible vía [`compute_report_signature`]
///    (Core, puro) -- ANTES de tocar la BD, para que un input mal formado
///    nunca llegue a abrir una transacción.
/// 4. Persiste append-only atómico vía
///    [`GeneratedReportRepository::record_report`] (Shell, único I/O).
///
/// El reporte NUNCA muta el `input` recibido -- lo presenta tal cual.
pub async fn generate_report(
    pool: &SqlitePool,
    clock: &dyn Clock,
    identity: ReportGenerationIdentity,
    mut input: AssembleReportInput,
) -> Result<GeneratedReportRow, GenerateReportError> {
    // Paso 1 -- reloj inyectado (única lectura de I/O de esta función antes
    // de ensamblar).
    input.generated_at_ns = clock.timestamp_ns();

    // Paso 2 -- ensamblado puro (Core).
    let report = assemble_report(input);

    // Paso 3 -- firma reproducible (Core, puro) -- se calcula ANTES de abrir
    // la transacción de persistencia.
    let signature_hash = compute_report_signature(&report);

    // Paso 4 -- persiste append-only atómico (Shell, único I/O de escritura).
    let repo = GeneratedReportRepository::new(pool, clock);
    let row = repo
        .record_report(RecordGeneratedReportInput {
            owner_id: identity.owner_id,
            institutional_tag: identity.institutional_tag,
            node_id: identity.node_id,
            compliance_status_id: identity.compliance_status_id,
            report,
            signature_hash,
        })
        .await?;

    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::domain::institutional_report_engine::{compute_report_signature, ReportType};
    use crate::persistence::pool::{connect, migrate};
    use std::collections::BTreeMap;

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    fn sample_input() -> AssembleReportInput {
        let mut metrics = BTreeMap::new();
        metrics.insert("sharpe_e8".to_string(), 150_000_000);
        metrics.insert("max_drawdown_e8".to_string(), -8_000_000);
        AssembleReportInput {
            report_type: ReportType::Validation,
            metrics,
            source_result_ref: None,
            source_event_refs: vec!["evt-1".to_string(), "evt-2".to_string()],
            // Se sobrescribe dentro de generate_report -- el valor aquí es
            // irrelevante, pero se deja explícito para que quede claro que
            // `AssembleReportInput` exige el campo.
            generated_at_ns: 0,
        }
    }

    fn identity() -> ReportGenerationIdentity {
        ReportGenerationIdentity {
            owner_id: "owner-1".to_string(),
            institutional_tag: "DRASUS_LOCAL".to_string(),
            node_id: "node-1".to_string(),
            compliance_status_id: None,
        }
    }

    /// CRITERIO DE CIERRE: `generate_report` compone Core + Shell
    /// correctamente -- el `signature_hash` persistido coincide EXACTAMENTE
    /// con lo que produciría `compute_report_signature` sobre el reporte
    /// ensamblado con el `generated_at_ns` que el reloj inyectado entregó.
    #[tokio::test]
    async fn generate_report_persists_a_signature_consistent_with_the_injected_clock() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(5_000, 0);

        let row = generate_report(&pool, &clock, identity(), sample_input())
            .await
            .expect("generar y persistir el reporte");

        // Reconstruye el mismo reporte con el generated_at_ns que el reloj
        // determinista entrega en su primera lectura, y confirma que la
        // firma persistida es la misma que produciría el Core de forma
        // independiente -- cierra el lazo Core -> Shell -> BD.
        let mut expected_input = sample_input();
        expected_input.generated_at_ns = 5_000;
        let expected_report = crate::domain::institutional_report_engine::assemble_report(expected_input);
        let expected_signature = compute_report_signature(&expected_report);

        assert_eq!(row.signature_hash, expected_signature);
        assert_eq!(row.report_type, "VALIDATION");
        assert_eq!(row.event_sequence_id, 1);
        assert!(row.audit_chain_hash.is_none(), "primer reporte de la BD debe ser génesis");
    }

    /// CRITERIO DE CIERRE (trazabilidad, Orden §5 criterio #4): el
    /// `input.source_event_refs` original queda intacto tras la llamada --
    /// `generate_report` solo lo presenta y lo persiste, nunca lo altera.
    #[tokio::test]
    async fn generate_report_never_mutates_the_caller_input() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);

        let input = sample_input();
        let original_refs = input.source_event_refs.clone();

        let row = generate_report(&pool, &clock, identity(), input)
            .await
            .expect("generar y persistir el reporte");

        let persisted_refs: Vec<String> =
            serde_json::from_str(&row.source_event_refs).expect("JSON válido");
        assert_eq!(persisted_refs, original_refs, "los source_event_refs persistidos deben ser EXACTAMENTE los del input original");
    }
}
