//! [CORE] Lógica pura de Institutional Report Engine / Motor de Reportes
//! Institucionales (`docs/features/institutional-report-engine.md`,
//! ADR-0144 cimiento #7, ADR-0101 plantillas Tera diferidas, ADR-0027
//! trazabilidad al audit-log, ADR-0141, ADR-0020, ADR-0093, STORY-034).
//!
//! Sin I/O, sin reloj de sistema, sin aleatoriedad sin semilla
//! (ADR-0002/0004). Este módulo ensambla un reporte institucional a partir
//! de un puñado de métricas nombradas + referencias de trazabilidad, y
//! calcula su **firma de integridad reproducible**: el mismo contenido de
//! reporte, ensamblado dos veces, produce el mismo hash bit a bit -- esta
//! propiedad es la que le permite a un tercero (auditor, regulador, fondo
//! cliente) verificar que un reporte no fue alterado, sin tener que confiar
//! en la palabra de quien lo entrega.
//!
//! ## Por qué NO se modela el `BacktestResult`/`RobustnessScore` reales
//!
//! `crate::types::BacktestResult` y `crate::types::RobustnessScore` son hoy
//! placeholders vacíos (`pub struct X;`) en el catálogo de tipos de puerto
//! (ADR-0137) -- el guantelete de validación/backtest todavía no los
//! produce con campos reales. Modelar aquí su forma completa sería
//! alucinar un contrato que no existe (`base/SKILL.md`: "sin
//! especulación técnica"). En su lugar, [`AssembleReportInput`] modela la
//! **entrada mínima de reporte** que SÍ se puede construir hoy: un
//! conjunto de métricas nombradas (`BTreeMap<String, i64>`, todas enteras
//! ×10⁸ -- ADR-0141, cero `f64`) + los metadatos de trazabilidad. El
//! mapeo real al `BacktestResult`/`RobustnessScore` completos queda como
//! trabajo futuro, cuando el guantelete los produzca (ver Orden STORY-034
//! §8 "Deudas / diferidos registrados").
//!
//! ## `signature_hash` vs. `audit_hash` (distinción clave, ADR-0020 Perfil D)
//!
//! Este módulo produce DOS hashes con roles distintos, que la Shell
//! persiste en columnas separadas:
//! - [`compute_report_signature`]: firma REPRODUCIBLE del **contenido** del
//!   reporte (`InstitutionalReport`) -- mismo contenido, mismo hash, sin
//!   importar cuándo o cuántas veces se recalcule. Es lo que un tercero
//!   verifica para confirmar que el reporte no fue alterado.
//! - [`compute_report_audit_hash`]: hash de integridad de **la fila del
//!   ledger** (`generated_reports`), encadenado al `audit_hash` de la fila
//!   anterior en la secuencia GLOBAL -- mismo patrón que
//!   `enriched_domain_events::compute_event_audit_hash`. Protege el
//!   historial de generaciones, no el contenido de un reporte en
//!   particular.

use std::collections::BTreeMap;

use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};

/// Codifica bytes crudos a su representación hexadecimal en minúsculas
/// (mismo patrón que `enriched_domain_events::encode_hex` /
/// `licensing_system::encode_hex`).
fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ── Catálogo de tipos de reporte ────────────────────────────────────────────

/// Qué clase de reporte institucional es este. El catálogo cubre los tres
/// tipos que el guantelete YA produce hoy (validación, backtest, ejecución)
/// más los cuatro tipos de producto anticipados por ADR-0144 punto 7 --
/// esos cuatro solo tienen aquí su nombre canónico; su adaptador de negocio
/// (plantilla, branding) queda diferido.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportType {
    Validation,
    Backtest,
    Execution,
    StressTest,
    ModelValidation,
    BacktestCertification,
    DrawdownForensics,
}

impl ReportType {
    /// Representación canónica en texto -- la que acepta el
    /// `CHECK (report_type IN (...))` de la migración `0013_generated_reports.sql`.
    pub fn as_str(&self) -> &'static str {
        match self {
            ReportType::Validation => "VALIDATION",
            ReportType::Backtest => "BACKTEST",
            ReportType::Execution => "EXECUTION",
            ReportType::StressTest => "STRESS_TEST",
            ReportType::ModelValidation => "MODEL_VALIDATION",
            ReportType::BacktestCertification => "BACKTEST_CERTIFICATION",
            ReportType::DrawdownForensics => "DRAWDOWN_FORENSICS",
        }
    }

    /// Reconstruye el tipo desde su representación en texto, o `None` si no
    /// es ninguno de los reconocidos (integridad de datos -- mismo patrón
    /// que `enriched_domain_events::OrderSide::from_str_value`).
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "VALIDATION" => Some(ReportType::Validation),
            "BACKTEST" => Some(ReportType::Backtest),
            "EXECUTION" => Some(ReportType::Execution),
            "STRESS_TEST" => Some(ReportType::StressTest),
            "MODEL_VALIDATION" => Some(ReportType::ModelValidation),
            "BACKTEST_CERTIFICATION" => Some(ReportType::BacktestCertification),
            "DRAWDOWN_FORENSICS" => Some(ReportType::DrawdownForensics),
            _ => None,
        }
    }
}

// ── Entrada mínima de ensamblado (Core, sin I/O) ────────────────────────────

/// Entrada mínima para [`assemble_report`] -- todo lo que el Core necesita
/// para construir un [`InstitutionalReport`], sin tocar disco ni reloj.
///
/// `generated_at_ns` viaja como campo plano (no se lee del reloj del
/// sistema aquí): la Shell/orchestrator lo obtiene del puerto [`Clock`]
/// inyectado y lo pasa ya resuelto (`docs/features/clock.md`,
/// ADR-0002/0004) -- así `assemble_report` sigue siendo una función pura:
/// la MISMA entrada (incluido el mismo `generated_at_ns`) siempre produce
/// el MISMO reporte.
///
/// [`Clock`]: crate::domain::clock::Clock
#[derive(Debug, Clone, PartialEq)]
pub struct AssembleReportInput {
    pub report_type: ReportType,
    /// Métricas nombradas del resultado a reportar -- TODAS enteras
    /// escaladas ×10⁸ (ADR-0141: "PROHIBIDO `REAL`/`f64` en columnas de
    /// precio o volumen"; aquí se extiende a cualquier métrica cuantitativa
    /// del reporte, por la misma razón que `enriched_domain_events`: un
    /// reporte es un documento que se re-serializa y se re-firma, y un
    /// `f64` puede perder precisión o serializar distinto entre
    /// plataformas, lo cual rompería la firma reproducible). Claves en
    /// `BTreeMap` para que el orden de inserción en memoria nunca afecte
    /// la serialización canónica.
    pub metrics: BTreeMap<String, i64>,
    /// Referencia de texto libre al resultado fuente del guantelete
    /// (`BacktestResult`/`RobustnessScore` placeholder) -- `None` si el
    /// reporte no está atado a un único id de resultado.
    pub source_result_ref: Option<String>,
    /// Ids de eventos del event-store (#6) / audit-log que este reporte
    /// cita, para trazabilidad (ADR-0027). NUNCA se leen ni modifican los
    /// eventos referenciados -- son solo punteros de texto.
    pub source_event_refs: Vec<String>,
    /// Instante de generación del reporte, en nanosegundos UTC (puerto
    /// Clock, inyectado por la Shell).
    pub generated_at_ns: i64,
}

// ── El reporte ensamblado ───────────────────────────────────────────────────

/// Un reporte institucional ensamblado -- el tipo de puerto `report_out`
/// (ADR-0137, catálogo, `InstitutionalReport`). Presenta las métricas del
/// resultado fuente sin alterarlas (`docs/features/institutional-report-engine.md`
/// "Restricciones": "NUNCA un reporte altera los datos fuente: solo los
/// presenta").
#[derive(Debug, Clone, PartialEq)]
pub struct InstitutionalReport {
    pub report_type: ReportType,
    pub metrics: BTreeMap<String, i64>,
    pub source_result_ref: Option<String>,
    pub source_event_refs: Vec<String>,
    pub generated_at_ns: i64,
}

impl InstitutionalReport {
    /// El string canónico del tipo de este reporte -- exactamente el valor
    /// que acepta el `CHECK` de la migración `0013_generated_reports.sql`.
    pub fn report_type(&self) -> &'static str {
        self.report_type.as_str()
    }

    /// Construye el mapa canónico y determinista de este reporte como un
    /// `BTreeMap<String, JsonValue>` -- las claves de un `BTreeMap` siempre
    /// serializan en orden alfabético (mismo patrón que
    /// `enriched_domain_events::EnrichedDomainEvent::to_canonical_map`), así
    /// que el MISMO reporte lógico siempre produce EXACTAMENTE el mismo
    /// string JSON, sin importar el orden en que se construyeron los campos
    /// en memoria. Las `metrics` anidadas también son un `BTreeMap`, así que
    /// heredan la misma garantía de orden.
    fn to_canonical_map(&self) -> BTreeMap<String, JsonValue> {
        let mut map = BTreeMap::new();

        map.insert("generated_at_ns".to_string(), serde_json::json!(self.generated_at_ns));
        map.insert("metrics".to_string(), serde_json::json!(self.metrics));
        map.insert("report_type".to_string(), serde_json::json!(self.report_type.as_str()));
        map.insert(
            "source_event_refs".to_string(),
            serde_json::json!(self.source_event_refs),
        );
        map.insert(
            "source_result_ref".to_string(),
            serde_json::json!(self.source_result_ref),
        );

        map
    }

    /// Serializa el reporte a JSON canónico (claves ordenadas
    /// alfabéticamente vía `BTreeMap`). Determinista: el mismo reporte
    /// lógico siempre produce el mismo string, en cualquier ejecución
    /// (ADR-0002/0004). Este string es EXACTAMENTE lo que persiste la
    /// columna `report_body` y lo que [`compute_report_signature`] hashea.
    ///
    /// El `.expect` es seguro: el mapa solo contiene claves `String` y
    /// valores `serde_json::Value` construidos a partir de
    /// `String`/`i64`/`Option<String>`/`Vec<String>`/`BTreeMap<String, i64>`
    /// (nunca `f64`/`NaN`/`Infinity`, los únicos casos que hacen fallar la
    /// serialización de `serde_json`), así que esta llamada nunca falla en
    /// la práctica.
    pub fn canonical_report_json(&self) -> String {
        let map = self.to_canonical_map();
        serde_json::to_string(&map)
            .expect("BTreeMap<String, JsonValue> de solo strings/enteros siempre serializa")
    }
}

/// Ensambla un [`InstitutionalReport`] a partir de la entrada mínima --
/// pura, determinista, sin I/O. Presenta las métricas y referencias tal
/// cual llegaron, sin recalcularlas ni mutarlas (el reporte NUNCA altera
/// los datos fuente).
pub fn assemble_report(input: AssembleReportInput) -> InstitutionalReport {
    InstitutionalReport {
        report_type: input.report_type,
        metrics: input.metrics,
        source_result_ref: input.source_result_ref,
        source_event_refs: input.source_event_refs,
        generated_at_ns: input.generated_at_ns,
    }
}

// ── Firma de integridad reproducible (EL punto de correctitud crítico) ─────

/// Calcula la firma de integridad SHA-256 (hex, minúsculas) del
/// **contenido** de `report` -- DETERMINISTA: el mismo reporte, ensamblado
/// de forma independiente las veces que sea, produce EXACTAMENTE la misma
/// firma bit a bit. Es la propiedad que le permite a un tercero (auditor,
/// regulador, cliente institucional) verificar que un reporte no fue
/// alterado, sin tener que confiar en la palabra de quien lo entrega --
/// solo tiene que re-ensamblar el mismo input y comparar la firma.
///
/// La firma se calcula sobre [`InstitutionalReport::canonical_report_json`]
/// (serialización canónica `BTreeMap`, claves ordenadas) -- NUNCA sobre una
/// representación en memoria no determinista (`HashMap`, orden de
/// construcción de structs), que produciría firmas distintas para el mismo
/// contenido lógico entre ejecuciones.
///
/// Distinto en rol de [`compute_report_audit_hash`]: esta función firma el
/// CONTENIDO del reporte (columna `signature_hash`); aquella firma la FILA
/// del ledger (columna `audit_hash`, Grupo I).
pub fn compute_report_signature(report: &InstitutionalReport) -> String {
    let mut hasher = Sha256::new();
    hasher.update(report.canonical_report_json().as_bytes());
    encode_hex(&hasher.finalize())
}

// ── Hash de auditoría de la fila del ledger (encadenado, APPEND-ONLY) ──────

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de una fila de
/// `generated_reports`, encadenado al `audit_hash` de la fila anterior en
/// la secuencia GLOBAL (o [`crate::domain::audit_log::GENESIS_PREVIOUS_HASH`]
/// si es la fila génesis, `event_sequence_id == 1`). Mismo patrón que
/// `enriched_domain_events::compute_event_audit_hash` -- protege la
/// INTEGRIDAD DE LA FILA en el ledger (quién generó qué reporte y cuándo),
/// no el contenido del reporte en sí (eso lo cubre `signature_hash`, vía
/// [`compute_report_signature`]).
#[allow(clippy::too_many_arguments)]
pub fn compute_report_audit_hash(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    previous_audit_hash: &str,
    owner_id: &str,
    institutional_tag: &str,
    node_id: &str,
    report_type: &str,
    source_result_ref: Option<&str>,
    source_event_refs_json: &str,
    report_body_json: &str,
    signature_hash: &str,
    compliance_status_id: Option<&str>,
) -> String {
    const SEP: char = '\u{1F}';

    let mut buffer = String::new();
    let mut push = |field: &str| {
        buffer.push_str(field);
        buffer.push(SEP);
    };

    push(id);
    push(&created_at_ns.to_string());
    push(&event_sequence_id.to_string());
    push(previous_audit_hash);
    push(owner_id);
    push(institutional_tag);
    push(node_id);
    push(report_type);
    push(source_result_ref.unwrap_or(""));
    push(source_event_refs_json);
    push(report_body_json);
    push(signature_hash);
    push(compliance_status_id.unwrap_or(""));

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    encode_hex(&hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_input() -> AssembleReportInput {
        let mut metrics = BTreeMap::new();
        metrics.insert("sharpe_e8".to_string(), 150_000_000);
        metrics.insert("max_drawdown_e8".to_string(), -8_000_000);

        AssembleReportInput {
            report_type: ReportType::Validation,
            metrics,
            source_result_ref: Some("backtest-run-42".to_string()),
            source_event_refs: vec!["evt-1".to_string(), "evt-2".to_string()],
            generated_at_ns: 1_000,
        }
    }

    // ── CRITERIO #2 (Orden §5): firma REPRODUCIBLE ──────────────────────────

    /// CRITERIO DE CIERRE: dos ensamblados independientes sobre el MISMO
    /// input producen la MISMA firma -- determinismo de la serialización
    /// canónica (ADR-0002/0004). Si `to_canonical_map` usara un `HashMap`
    /// en vez de `BTreeMap`, esta prueba podría fallar de forma
    /// intermitente entre ejecuciones del proceso.
    #[test]
    fn compute_report_signature_is_reproducible_across_independent_assemblies() {
        let report_a = assemble_report(sample_input());
        let report_b = assemble_report(sample_input());

        let signature_a = compute_report_signature(&report_a);
        let signature_b = compute_report_signature(&report_b);

        assert_eq!(signature_a, signature_b, "el mismo input debe producir la MISMA firma");
    }

    /// CRITERIO DE CIERRE: cambiar UN dato (una métrica) cambia la firma --
    /// si la firma no cubriera el contenido real, esta prueba fallaría con
    /// firmas iguales pese al cambio.
    #[test]
    fn compute_report_signature_changes_when_a_metric_changes() {
        let mut input = sample_input();
        let original = compute_report_signature(&assemble_report(input.clone()));

        input.metrics.insert("sharpe_e8".to_string(), 200_000_000);
        let changed = compute_report_signature(&assemble_report(input));

        assert_ne!(original, changed, "cambiar una métrica debe cambiar la firma");
    }

    /// CRITERIO DE CIERRE: cambiar el tipo de reporte cambia la firma.
    #[test]
    fn compute_report_signature_changes_when_report_type_changes() {
        let mut input = sample_input();
        let original = compute_report_signature(&assemble_report(input.clone()));

        input.report_type = ReportType::Backtest;
        let changed = compute_report_signature(&assemble_report(input));

        assert_ne!(original, changed, "cambiar el report_type debe cambiar la firma");
    }

    /// CRITERIO DE CIERRE: cambiar los `source_event_refs` cambia la firma
    /// -- la firma cubre también la trazabilidad, no solo las métricas.
    #[test]
    fn compute_report_signature_changes_when_source_event_refs_change() {
        let mut input = sample_input();
        let original = compute_report_signature(&assemble_report(input.clone()));

        input.source_event_refs.push("evt-3".to_string());
        let changed = compute_report_signature(&assemble_report(input));

        assert_ne!(original, changed, "cambiar source_event_refs debe cambiar la firma");
    }

    /// CRITERIO DE CIERRE: la serialización canónica nunca contiene un
    /// punto decimal -- todas las métricas son `i64`, nunca `f64`
    /// (inspección directa del JSON producido).
    #[test]
    fn canonical_report_json_never_contains_a_decimal_point() {
        let report = assemble_report(sample_input());
        let json = report.canonical_report_json();
        assert!(!json.contains('.'), "el reporte no debe serializar métricas como coma flotante");
    }

    /// CRITERIO DE CIERRE: las claves de nivel superior quedan en orden
    /// alfabético -- si se usara un mapa sin orden garantizado, esta
    /// prueba podría fallar de forma intermitente.
    #[test]
    fn canonical_report_json_top_level_keys_are_alphabetically_sorted() {
        let report = assemble_report(sample_input());
        let json = report.canonical_report_json();
        let parsed: JsonValue = serde_json::from_str(&json).expect("JSON válido");
        let keys: Vec<&String> = match &parsed {
            JsonValue::Object(map) => map.keys().collect(),
            _ => panic!("se esperaba un objeto JSON"),
        };
        let mut sorted_keys = keys.clone();
        sorted_keys.sort();
        assert_eq!(keys, sorted_keys, "las claves de nivel superior deben quedar en orden alfabético");
    }

    // ── Trazabilidad: assemble_report no altera el input ────────────────────

    /// CRITERIO DE CIERRE (Orden §5 criterio #4): el reporte presenta
    /// EXACTAMENTE las mismas referencias de trazabilidad que llegaron en
    /// el input -- `assemble_report` nunca las reescribe ni las filtra.
    #[test]
    fn assemble_report_preserves_source_event_refs_and_result_ref_unaltered() {
        let input = sample_input();
        let expected_refs = input.source_event_refs.clone();
        let expected_result_ref = input.source_result_ref.clone();

        let report = assemble_report(input);

        assert_eq!(report.source_event_refs, expected_refs, "los source_event_refs no deben alterarse");
        assert_eq!(report.source_result_ref, expected_result_ref, "el source_result_ref no debe alterarse");
    }

    // ── report_type() ────────────────────────────────────────────────────────

    #[test]
    fn report_type_matches_migration_check_catalog_for_every_variant() {
        let variants = [
            ReportType::Validation,
            ReportType::Backtest,
            ReportType::Execution,
            ReportType::StressTest,
            ReportType::ModelValidation,
            ReportType::BacktestCertification,
            ReportType::DrawdownForensics,
        ];
        let expected = [
            "VALIDATION",
            "BACKTEST",
            "EXECUTION",
            "STRESS_TEST",
            "MODEL_VALIDATION",
            "BACKTEST_CERTIFICATION",
            "DRAWDOWN_FORENSICS",
        ];

        for (variant, expected_str) in variants.iter().zip(expected.iter()) {
            assert_eq!(variant.as_str(), *expected_str);
            assert_eq!(ReportType::from_str_value(expected_str), Some(*variant));
        }
        assert_eq!(ReportType::from_str_value("UNKNOWN"), None);
    }

    // ── Sin secretos (ADR-0093) ──────────────────────────────────────────────

    /// CRITERIO DE CIERRE (guardarraíl ADR-0093): el reporte ensamblado no
    /// puede contener una credencial de bróker, una IP de servidor live, ni
    /// una clave de firma -- assert sobre el JSON canónico serializado.
    #[test]
    fn assembled_report_json_does_not_leak_secret_looking_fields() {
        let report = assemble_report(sample_input());
        let json = report.canonical_report_json().to_lowercase();
        for forbidden in [
            "password", "api_key", "api-key", "broker_secret", "private_key",
            "signing_key", "investor_password", "192.168.", "10.0.0.",
        ] {
            assert!(!json.contains(forbidden), "el reporte no debe contener '{forbidden}'");
        }
    }

    // ── Hash de auditoría de la fila (distinto en rol de signature_hash) ────

    #[test]
    fn compute_report_audit_hash_is_deterministic() {
        let hash_a = compute_report_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1", "VALIDATION",
            Some("run-1"), "[\"evt-1\"]", "{}", "sig-abc", None,
        );
        let hash_b = compute_report_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1", "VALIDATION",
            Some("run-1"), "[\"evt-1\"]", "{}", "sig-abc", None,
        );
        assert_eq!(hash_a, hash_b);
    }

    /// CRITERIO DE CIERRE (Orden §5 criterio #5): `signature_hash` y
    /// `audit_hash` son campos DISTINTOS con roles distintos -- cambiar
    /// `signature_hash` (mismo resto de campos) cambia el `audit_hash`
    /// resultante, lo que confirma que ambos hashes son independientes y
    /// que el `audit_hash` incorpora la firma como parte de la integridad
    /// de la fila, sin ser la misma cosa.
    #[test]
    fn compute_report_audit_hash_changes_when_signature_hash_changes() {
        let with_sig_a = compute_report_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1", "VALIDATION",
            None, "[]", "{}", "sig-aaa", None,
        );
        let with_sig_b = compute_report_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1", "VALIDATION",
            None, "[]", "{}", "sig-bbb", None,
        );
        assert_ne!(with_sig_a, with_sig_b, "cambiar signature_hash debe cambiar audit_hash");
    }

    #[test]
    fn compute_report_audit_hash_changes_when_report_body_changes() {
        let original = compute_report_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1", "VALIDATION",
            None, "[]", "{\"a\":1}", "sig-abc", None,
        );
        let changed = compute_report_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1", "VALIDATION",
            None, "[]", "{\"a\":2}", "sig-abc", None,
        );
        assert_ne!(original, changed, "cambiar report_body_json debe cambiar audit_hash");
    }
}
