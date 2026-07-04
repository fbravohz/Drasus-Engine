//! Catálogo canónico de tipos de puerto (ADR-0137).
//!
//! Cada tipo de dato que circula entre features en el Canvas [Forge/Reactor]
//! tiene un struct Rust + un identificador único. El trait `TypedPort` asigna
//! a cada tipo su id canónico, color semántico en el canvas y cardinalidad
//! esperada.
//!
//! Para añadir un tipo nuevo: crear el struct, implementar `TypedPort`,
//! añadirlo al enum `PortType`, y enmendar ADR-0137.

use std::fmt;

// ── Trait canónico de puerto tipado ─────────────────────────────────────────

/// Todo tipo que circula por el canvas implementa este trait.
/// Define cómo se identifica, con qué color se pinta y cuántos valores
/// espera en cada extremo de una conexión.
pub trait TypedPort: fmt::Debug + Clone + Send + Sync + 'static {
    /// Identificador canónico del tipo (ej. `"Bars"`). Coincide con ADR-0137.
    fn type_id() -> &'static str
    where
        Self: Sized;
    /// Color semántico en el canvas (hex string, ej. `"#56A8FF"`).
    fn canvas_color() -> &'static str
    where
        Self: Sized;
    /// Cardinalidad por defecto: `1` (exactamente uno), `0..1` (opcional),
    /// `0..N` (múltiple), `1..N` (al menos uno).
    fn cardinality() -> &'static str
    where
        Self: Sized;
}

// ── Categorías de dominio ───────────────────────────────────────────────────

/// Categoría de dominio a la que pertenece un tipo (agrupa en el catálogo).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDomain {
    DatosDeMercado,
    EstrategiaYGenoma,
    BacktestYValidacion,
    ScoresYVeredictos,
    OrdenesYEjecucion,
    Portafolio,
    Infraestructura,
}

// ── Tipos de datos de mercado ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Tick;
impl TypedPort for Tick {
    fn type_id() -> &'static str { "Tick" }
    fn canvas_color() -> &'static str { "#56A8FF" }
    fn cardinality() -> &'static str { "0..N" }
}

#[derive(Debug, Clone)]
pub struct Bars;
impl TypedPort for Bars {
    fn type_id() -> &'static str { "Bars" }
    fn canvas_color() -> &'static str { "#56A8FF" }
    fn cardinality() -> &'static str { "1" }
}

#[derive(Debug, Clone)]
pub struct AlgoBar;
impl TypedPort for AlgoBar {
    fn type_id() -> &'static str { "AlgoBar" }
    fn canvas_color() -> &'static str { "#56A8FF" }
    fn cardinality() -> &'static str { "0..N" }
}

#[derive(Debug, Clone)]
pub struct SanitizedDataframe;
impl TypedPort for SanitizedDataframe {
    fn type_id() -> &'static str { "SanitizedDataframe" }
    fn canvas_color() -> &'static str { "#2DD4BF" }
    fn cardinality() -> &'static str { "1" }
}

#[derive(Debug, Clone)]
pub struct FundamentalEvent;
impl TypedPort for FundamentalEvent {
    fn type_id() -> &'static str { "FundamentalEvent" }
    fn canvas_color() -> &'static str { "#9A8CFF" }
    fn cardinality() -> &'static str { "0..N" }
}

#[derive(Debug, Clone)]
pub struct FundamentalIndicatorSeries;
impl TypedPort for FundamentalIndicatorSeries {
    fn type_id() -> &'static str { "FundamentalIndicatorSeries" }
    fn canvas_color() -> &'static str { "#9A8CFF" }
    fn cardinality() -> &'static str { "0..1" }
}

#[derive(Debug, Clone)]
pub struct RegimeLabel;
impl TypedPort for RegimeLabel {
    fn type_id() -> &'static str { "RegimeLabel" }
    fn canvas_color() -> &'static str { "#FFC94D" }
    fn cardinality() -> &'static str { "1" }
}

#[derive(Debug, Clone)]
pub struct ArrowStream;
impl TypedPort for ArrowStream {
    fn type_id() -> &'static str { "ArrowStream" }
    fn canvas_color() -> &'static str { "#8492B0" }
    fn cardinality() -> &'static str { "1" }
}

// ── Tipos de estrategia y genoma ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ExecutableContainer;
impl TypedPort for ExecutableContainer {
    fn type_id() -> &'static str { "ExecutableContainer" }
    fn canvas_color() -> &'static str { "#9A8CFF" }
    fn cardinality() -> &'static str { "0..N" }
}

#[derive(Debug, Clone)]
pub struct StrategyManifest;
impl TypedPort for StrategyManifest {
    fn type_id() -> &'static str { "StrategyManifest" }
    fn canvas_color() -> &'static str { "#9A8CFF" }
    fn cardinality() -> &'static str { "0..1" }
}

#[derive(Debug, Clone)]
pub struct CandidateGenome;
impl TypedPort for CandidateGenome {
    fn type_id() -> &'static str { "CandidateGenome" }
    fn canvas_color() -> &'static str { "#8B83E8" }
    fn cardinality() -> &'static str { "0..N" }
}

#[derive(Debug, Clone)]
pub struct Signal;
impl TypedPort for Signal {
    fn type_id() -> &'static str { "Signal" }
    fn canvas_color() -> &'static str { "#7CF06A" }
    fn cardinality() -> &'static str { "0..N" }
}

#[derive(Debug, Clone)]
pub struct ParetoFront;
impl TypedPort for ParetoFront {
    fn type_id() -> &'static str { "ParetoFront" }
    fn canvas_color() -> &'static str { "#54E8D0" }
    fn cardinality() -> &'static str { "0..1" }
}

#[derive(Debug, Clone)]
pub struct StrategyVersionNode;
impl TypedPort for StrategyVersionNode {
    fn type_id() -> &'static str { "StrategyVersionNode" }
    fn canvas_color() -> &'static str { "#8492B0" }
    fn cardinality() -> &'static str { "0..1" }
}

// ── Tipos de backtest y validación ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BacktestResult;
impl TypedPort for BacktestResult {
    fn type_id() -> &'static str { "BacktestResult" }
    fn canvas_color() -> &'static str { "#9A8CFF" }
    fn cardinality() -> &'static str { "0..N" }
}

#[derive(Debug, Clone)]
pub struct TradeLog;
impl TypedPort for TradeLog {
    fn type_id() -> &'static str { "TradeLog" }
    fn canvas_color() -> &'static str { "#9A8CFF" }
    fn cardinality() -> &'static str { "0..N" }
}

#[derive(Debug, Clone)]
pub struct EquityCurve;
impl TypedPort for EquityCurve {
    fn type_id() -> &'static str { "EquityCurve" }
    fn canvas_color() -> &'static str { "#54E8D0" }
    fn cardinality() -> &'static str { "0..1" }
}

#[derive(Debug, Clone)]
pub struct DrawdownCurve;
impl TypedPort for DrawdownCurve {
    fn type_id() -> &'static str { "DrawdownCurve" }
    fn canvas_color() -> &'static str { "#FF8A8A" }
    fn cardinality() -> &'static str { "0..1" }
}

#[derive(Debug, Clone)]
pub struct MetricsDict;
impl TypedPort for MetricsDict {
    fn type_id() -> &'static str { "MetricsDict" }
    fn canvas_color() -> &'static str { "#54E8D0" }
    fn cardinality() -> &'static str { "0..1" }
}

#[derive(Debug, Clone)]
pub struct WfaMatrix;
impl TypedPort for WfaMatrix {
    fn type_id() -> &'static str { "WFAMatrix" }
    fn canvas_color() -> &'static str { "#9A8CFF" }
    fn cardinality() -> &'static str { "0..1" }
}

#[derive(Debug, Clone)]
pub struct MonteCarloResult;
impl TypedPort for MonteCarloResult {
    fn type_id() -> &'static str { "MonteCarloResult" }
    fn canvas_color() -> &'static str { "#9A8CFF" }
    fn cardinality() -> &'static str { "0..1" }
}

#[derive(Debug, Clone)]
pub struct PboCpcvReport;
impl TypedPort for PboCpcvReport {
    fn type_id() -> &'static str { "PBOCPCVReport" }
    fn canvas_color() -> &'static str { "#9A8CFF" }
    fn cardinality() -> &'static str { "0..1" }
}

// ── Tipos de scores y veredictos ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RobustnessScore;
impl TypedPort for RobustnessScore {
    fn type_id() -> &'static str { "RobustnessScore" }
    fn canvas_color() -> &'static str { "#54E8D0" }
    fn cardinality() -> &'static str { "0..1" }
}

#[derive(Debug, Clone)]
pub struct RobustnessVerdict;
impl TypedPort for RobustnessVerdict {
    fn type_id() -> &'static str { "RobustnessVerdict" }
    fn canvas_color() -> &'static str { "#54E8D0" }
    fn cardinality() -> &'static str { "0..1" }
}

#[derive(Debug, Clone)]
pub struct DriftScore;
impl TypedPort for DriftScore {
    fn type_id() -> &'static str { "DriftScore" }
    fn canvas_color() -> &'static str { "#FFC94D" }
    fn cardinality() -> &'static str { "0..1" }
}

#[derive(Debug, Clone)]
pub struct IncubationVerdict;
impl TypedPort for IncubationVerdict {
    fn type_id() -> &'static str { "IncubationVerdict" }
    fn canvas_color() -> &'static str { "#54E8D0" }
    fn cardinality() -> &'static str { "0..1" }
}

#[derive(Debug, Clone)]
pub struct HealthStatus;
impl TypedPort for HealthStatus {
    fn type_id() -> &'static str { "HealthStatus" }
    fn canvas_color() -> &'static str { "#54E8D0" }
    fn cardinality() -> &'static str { "0..1" }
}

#[derive(Debug, Clone)]
pub struct ContextualFitnessScore;
impl TypedPort for ContextualFitnessScore {
    fn type_id() -> &'static str { "ContextualFitnessScore" }
    fn canvas_color() -> &'static str { "#9A8CFF" }
    fn cardinality() -> &'static str { "0..1" }
}

// ── Tipos de órdenes y ejecución ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Order;
impl TypedPort for Order {
    fn type_id() -> &'static str { "Order" }
    fn canvas_color() -> &'static str { "#7CF06A" }
    fn cardinality() -> &'static str { "0..N" }
}

#[derive(Debug, Clone)]
pub struct PreTradeVerdict;
impl TypedPort for PreTradeVerdict {
    fn type_id() -> &'static str { "PreTradeVerdict" }
    fn canvas_color() -> &'static str { "#54E8D0" }
    fn cardinality() -> &'static str { "1" }
}

#[derive(Debug, Clone)]
pub struct FillEvent;
impl TypedPort for FillEvent {
    fn type_id() -> &'static str { "FillEvent" }
    fn canvas_color() -> &'static str { "#7CF06A" }
    fn cardinality() -> &'static str { "0..N" }
}

#[derive(Debug, Clone)]
pub struct PositionSize;
impl TypedPort for PositionSize {
    fn type_id() -> &'static str { "PositionSize" }
    fn canvas_color() -> &'static str { "#2DD4BF" }
    fn cardinality() -> &'static str { "0..1" }
}

#[derive(Debug, Clone)]
pub struct AccountState;
impl TypedPort for AccountState {
    fn type_id() -> &'static str { "AccountState" }
    fn canvas_color() -> &'static str { "#8492B0" }
    fn cardinality() -> &'static str { "0..N" }
}

#[derive(Debug, Clone)]
pub struct ReconciliationReport;
impl TypedPort for ReconciliationReport {
    fn type_id() -> &'static str { "ReconciliationReport" }
    fn canvas_color() -> &'static str { "#FFC94D" }
    fn cardinality() -> &'static str { "0..1" }
}

// ── Tipos de portafolio ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PortfolioWeights;
impl TypedPort for PortfolioWeights {
    fn type_id() -> &'static str { "PortfolioWeights" }
    fn canvas_color() -> &'static str { "#8B83E8" }
    fn cardinality() -> &'static str { "1" }
}

#[derive(Debug, Clone)]
pub struct PortfolioBacktestResult;
impl TypedPort for PortfolioBacktestResult {
    fn type_id() -> &'static str { "PortfolioBacktestResult" }
    fn canvas_color() -> &'static str { "#9A8CFF" }
    fn cardinality() -> &'static str { "0..1" }
}

#[derive(Debug, Clone)]
pub struct CorrelationMatrix;
impl TypedPort for CorrelationMatrix {
    fn type_id() -> &'static str { "CorrelationMatrix" }
    fn canvas_color() -> &'static str { "#FFC94D" }
    fn cardinality() -> &'static str { "0..1" }
}

#[derive(Debug, Clone)]
pub struct RuleVerdict;
impl TypedPort for RuleVerdict {
    fn type_id() -> &'static str { "RuleVerdict" }
    fn canvas_color() -> &'static str { "#54E8D0" }
    fn cardinality() -> &'static str { "0..N" }
}

// ── Tipos de infraestructura ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AuditEvent;
impl TypedPort for AuditEvent {
    fn type_id() -> &'static str { "AuditEvent" }
    fn canvas_color() -> &'static str { "#8492B0" }
    fn cardinality() -> &'static str { "0..N" }
}

#[derive(Debug, Clone)]
pub struct TelemetrySample;
impl TypedPort for TelemetrySample {
    fn type_id() -> &'static str { "TelemetrySample" }
    fn canvas_color() -> &'static str { "#8492B0" }
    fn cardinality() -> &'static str { "0..N" }
}

#[derive(Debug, Clone)]
pub struct Job;
impl TypedPort for Job {
    fn type_id() -> &'static str { "Job" }
    fn canvas_color() -> &'static str { "#8492B0" }
    fn cardinality() -> &'static str { "0..N" }
}

#[derive(Debug, Clone)]
pub struct TimestampNs;
impl TypedPort for TimestampNs {
    fn type_id() -> &'static str { "timestamp_ns" }
    fn canvas_color() -> &'static str { "#8492B0" }
    fn cardinality() -> &'static str { "0..1" }
}

// ── Tipos del substrato de monetización (ADR-0137 enmienda 2026-07-03, ADR-0144) ─

/// Marcador de tipo de puerto para el catálogo (ADR-0137). El dato real que
/// circula por este puerto es `crate::domain::central_identity::AccountIdentity`
/// (mismo patrón de nombres duplicados-por-módulo que `AuditEvent`/
/// `TelemetrySample`/`Job` arriba: el marcador vive en `types`, el struct
/// con los campos reales vive en `domain`/`persistence`).
#[derive(Debug, Clone)]
pub struct AccountIdentity;
impl TypedPort for AccountIdentity {
    fn type_id() -> &'static str { "AccountIdentity" }
    fn canvas_color() -> &'static str { "#8492B0" }
    fn cardinality() -> &'static str { "1" }
}
