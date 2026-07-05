//! [CORE] Lógica pura de Enriched Domain Events / Eventos de Dominio
//! Enriquecidos (`docs/features/enriched-domain-events.md`, ADR-0144
//! cimiento #6, ADR-0145 enriquecimiento, ADR-0143, ADR-0141, ADR-0020,
//! ADR-0093, STORY-033).
//!
//! Sin I/O, sin reloj de sistema, sin aleatoriedad sin semilla
//! (ADR-0002/0004). Este módulo es un **event-store heterogéneo**: en vez
//! de una tabla/tipo por cada clase de evento, hay UN enum
//! ([`EnrichedDomainEvent`]) con una variante por clase, y cada variante
//! serializa a un payload JSON canónico y determinista (mismo patrón de
//! `BTreeMap` ordenado que `domain::consent_registry`). La Shell
//! (`persistence::enriched_domain_events`) persiste `event_type()` +
//! ese payload en una sola tabla `domain_events`.
//!
//! Piezas de lógica pura que pide la Feature en su "Estructura Interna
//! (FCIS)" y la Orden STORY-033 §4.2:
//! - [`EnrichedDomainEvent`]: el catálogo completo -- orden ejecutada
//!   (reforzada con ADR-0145: `account_id`, PnL, MAE, MFE, duración),
//!   flujo de capital (ADR-0145), snapshot de cuenta (ADR-0145), backtest
//!   completado, régimen detectado, drawdown detectado, estrés de
//!   liquidez, cambio de correlación.
//! - [`EnrichedDomainEvent::event_type`]: el string canónico de la
//!   variante (el que acepta el `CHECK` de la migración).
//! - [`EnrichedDomainEvent::canonical_payload_json`]: el payload
//!   determinista de la variante.
//! - [`compute_event_audit_hash`]: hash de auditoría encadenado por
//!   `event_sequence_id` (mismo patrón que `usage_metering::compute_usage_audit_hash`
//!   -- esta tabla es APPEND-ONLY, no `row_version`).
//! - [`decide_replication`]: la decisión de supresión por tier (ADR-0143),
//!   consumiendo el `ExecutionGate` REAL de `licensing-system` (#2).
//!
//! ## Todos los montos monetarios son `i64` escalados ×10⁸ (ADR-0141)
//!
//! Ningún campo de este módulo usa `f64`/`REAL` -- ni los montos
//! explícitamente monetarios (nocional, PnL, flujo de capital, equity)
//! ni las métricas de coma flotante que en otros sistemas serían `f64`
//! (Sharpe, drawdown porcentual, probabilidad PBO, correlación). Se
//! escalan TODAS ×10⁸ por la misma razón que un precio: un evento de
//! dominio es un hecho histórico que se re-serializa y se re-hashea
//! (auditoría, replay); un `f64` puede perder precisión de los últimos
//! dígitos o serializar de forma distinta entre plataformas, lo cual
//! rompería tanto el hash de auditoría como la reconstrucción exacta de
//! curvas de equity/drawdown que este cimiento existe para habilitar.

use std::collections::BTreeMap;

use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};

use crate::domain::licensing_system::ExecutionGate;

/// Codifica bytes crudos a su representación hexadecimal en minúsculas
/// (mismo patrón que `licensing_system::encode_hex` / `usage_metering::encode_hex`).
fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ── Lado (BUY/SELL) de una orden ejecutada ──────────────────────────────────

/// Dirección de una orden ejecutada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    /// Representación canónica en texto -- la que entra en el payload JSON.
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        }
    }

    /// Reconstruye el lado desde su representación en texto, o `None` si no
    /// es ninguno de los dos reconocidos (integridad de datos).
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "BUY" => Some(OrderSide::Buy),
            "SELL" => Some(OrderSide::Sell),
            _ => None,
        }
    }
}

// ── Signo de un flujo de capital (ADR-0145) ─────────────────────────────────

/// Signo de un movimiento de capital -- depósito, retiro o transferencia
/// (`docs/features/enriched-domain-events.md` "Comportamientos Observables",
/// ADR-0145).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapitalFlowSign {
    Deposit,
    Withdrawal,
    Transfer,
}

impl CapitalFlowSign {
    /// Representación canónica en texto -- la que entra en el payload JSON.
    pub fn as_str(&self) -> &'static str {
        match self {
            CapitalFlowSign::Deposit => "DEPOSIT",
            CapitalFlowSign::Withdrawal => "WITHDRAWAL",
            CapitalFlowSign::Transfer => "TRANSFER",
        }
    }

    /// Reconstruye el signo desde su representación en texto, o `None` si no
    /// es ninguno de los tres reconocidos (integridad de datos).
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "DEPOSIT" => Some(CapitalFlowSign::Deposit),
            "WITHDRAWAL" => Some(CapitalFlowSign::Withdrawal),
            "TRANSFER" => Some(CapitalFlowSign::Transfer),
            _ => None,
        }
    }
}

// ── Payloads por variante (todos los montos i64 ×10⁸, CERO f64) ────────────

/// Orden ejecutada, reforzada con los campos de ADR-0145 (`account_id`, PnL
/// realizado, MAE, MFE, duración del trade) para que los eventos se agrupen
/// por cuenta y se deriven % de trades rentables, tiempo medio de espera y
/// días de trading.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderExecutedPayload {
    pub instrument_id: String,
    pub side: OrderSide,
    /// Cantidad operada, `i64` escalado ×10⁸.
    pub quantity: i64,
    /// Precio de ejecución, `i64` escalado ×10⁸.
    pub price: i64,
    /// Slippage (precio esperado - precio real), `i64` escalado ×10⁸, con
    /// signo -- positivo o negativo según si benefició o perjudicó al fill.
    pub slippage: i64,
    /// Instante del fill (nanosegundos UTC, puerto Clock).
    pub fill_time_ns: i64,
    pub broker: String,
    /// Nocional de la operación (cantidad × precio), `i64` escalado ×10⁸.
    pub notional: i64,
    /// Refuerzo ADR-0145: cuenta de bróker a la que pertenece esta orden.
    pub account_id: String,
    /// Refuerzo ADR-0145: PnL realizado de este trade, `i64` escalado ×10⁸.
    pub realized_pnl: i64,
    /// Refuerzo ADR-0145: Maximum Adverse Excursion, `i64` escalado ×10⁸.
    pub mae: i64,
    /// Refuerzo ADR-0145: Maximum Favorable Excursion, `i64` escalado ×10⁸.
    pub mfe: i64,
    /// Refuerzo ADR-0145: duración del trade en nanosegundos.
    pub duration_ns: i64,
}

/// Flujo de capital -- depósito, retiro o transferencia (ADR-0145). Sin
/// esto, el gain% de una cuenta es incalculable: no se puede separar el
/// beneficio del capital aportado.
#[derive(Debug, Clone, PartialEq)]
pub struct CapitalFlowPayload {
    pub account_id: String,
    pub sign: CapitalFlowSign,
    /// Monto del movimiento, `i64` escalado ×10⁸ -- SIEMPRE positivo; el
    /// signo semántico (entra/sale) lo da [`CapitalFlowSign`], no el signo
    /// aritmético de este campo.
    pub amount: i64,
    pub currency: String,
    pub timestamp_ns: i64,
}

/// Snapshot de estado de cuenta -- equity, balance y margen
/// disponible/requerido (ADR-0145). Alimenta las curvas de equidad y
/// balance del track record.
#[derive(Debug, Clone, PartialEq)]
pub struct AccountSnapshotPayload {
    pub account_id: String,
    /// `i64` escalado ×10⁸.
    pub equity: i64,
    /// `i64` escalado ×10⁸.
    pub balance: i64,
    /// `i64` escalado ×10⁸.
    pub margin_available: i64,
    /// `i64` escalado ×10⁸.
    pub margin_required: i64,
    pub timestamp_ns: i64,
}

/// Backtest completado -- métricas completas de desempeño.
#[derive(Debug, Clone, PartialEq)]
pub struct BacktestCompletedPayload {
    /// Sharpe ratio, `i64` escalado ×10⁸ (con signo -- puede ser negativo).
    pub sharpe: i64,
    /// Drawdown máximo (fracción de 0 a 1), `i64` escalado ×10⁸.
    pub drawdown: i64,
    /// Probability of Backtest Overfitting (fracción de 0 a 1), `i64`
    /// escalado ×10⁸.
    pub pbo: i64,
    pub regime: String,
}

/// Régimen de mercado detectado.
#[derive(Debug, Clone, PartialEq)]
pub struct RegimeDetectedPayload {
    pub instrument_id: String,
    pub regime_label: String,
    pub timestamp_ns: i64,
}

/// Drawdown detectado sobre una cuenta.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawdownDetectedPayload {
    pub account_id: String,
    /// Fracción de drawdown (0 a 1), `i64` escalado ×10⁸.
    pub drawdown_pct: i64,
    pub timestamp_ns: i64,
}

/// Estrés de liquidez detectado en un instrumento.
#[derive(Debug, Clone, PartialEq)]
pub struct LiquidityStressPayload {
    pub instrument_id: String,
    pub severity: String,
    pub timestamp_ns: i64,
}

/// Cambio de correlación entre dos instrumentos/estrategias.
#[derive(Debug, Clone, PartialEq)]
pub struct CorrelationChangePayload {
    pub instrument_a: String,
    pub instrument_b: String,
    /// Coeficiente de correlación (-1 a 1), `i64` escalado ×10⁸, con signo.
    pub correlation: i64,
    pub timestamp_ns: i64,
}

// ── Catálogo completo (Core, event-sourcing con enum + payload JSON) ───────

/// El catálogo completo de eventos de dominio enriquecidos -- un event-store
/// HETEROGÉNEO: cada variante es una clase de evento distinta, pero todas
/// comparten la misma tabla física (`domain_events`) vía `event_type()` +
/// [`Self::canonical_payload_json`].
#[derive(Debug, Clone, PartialEq)]
pub enum EnrichedDomainEvent {
    OrderExecuted(OrderExecutedPayload),
    CapitalFlow(CapitalFlowPayload),
    AccountSnapshot(AccountSnapshotPayload),
    BacktestCompleted(BacktestCompletedPayload),
    RegimeDetected(RegimeDetectedPayload),
    DrawdownDetected(DrawdownDetectedPayload),
    LiquidityStress(LiquidityStressPayload),
    CorrelationChange(CorrelationChangePayload),
}

impl EnrichedDomainEvent {
    /// El string canónico de esta variante -- exactamente los valores que
    /// acepta el `CHECK (event_type IN (...))` de la migración
    /// `0012_domain_events.sql`.
    pub fn event_type(&self) -> &'static str {
        match self {
            EnrichedDomainEvent::OrderExecuted(_) => "ORDER_EXECUTED",
            EnrichedDomainEvent::CapitalFlow(_) => "CAPITAL_FLOW",
            EnrichedDomainEvent::AccountSnapshot(_) => "ACCOUNT_SNAPSHOT",
            EnrichedDomainEvent::BacktestCompleted(_) => "BACKTEST_COMPLETED",
            EnrichedDomainEvent::RegimeDetected(_) => "REGIME_DETECTED",
            EnrichedDomainEvent::DrawdownDetected(_) => "DRAWDOWN_DETECTED",
            EnrichedDomainEvent::LiquidityStress(_) => "LIQUIDITY_STRESS",
            EnrichedDomainEvent::CorrelationChange(_) => "CORRELATION_CHANGE",
        }
    }

    /// Construye el payload canónico y determinista de esta variante como
    /// un `BTreeMap<String, JsonValue>` -- las claves de un `BTreeMap`
    /// siempre serializan en orden alfabético (mismo patrón que
    /// `domain::consent_registry::apply_consent_action` con su
    /// `optout_map`), así que el MISMO evento lógico siempre produce
    /// EXACTAMENTE el mismo string JSON, sin importar el orden en que se
    /// construyeron los campos en memoria.
    fn to_canonical_map(&self) -> BTreeMap<String, JsonValue> {
        let mut map = BTreeMap::new();

        // Macro local minúscula para no repetir `.insert(key.to_string(), json!(value))`
        // en cada rama -- reduce el ruido visual sin ocultar lógica.
        macro_rules! put {
            ($key:literal, $value:expr) => {
                map.insert($key.to_string(), serde_json::json!($value));
            };
        }

        match self {
            EnrichedDomainEvent::OrderExecuted(p) => {
                put!("instrument_id", p.instrument_id);
                put!("side", p.side.as_str());
                put!("quantity", p.quantity);
                put!("price", p.price);
                put!("slippage", p.slippage);
                put!("fill_time_ns", p.fill_time_ns);
                put!("broker", p.broker);
                put!("notional", p.notional);
                put!("account_id", p.account_id);
                put!("realized_pnl", p.realized_pnl);
                put!("mae", p.mae);
                put!("mfe", p.mfe);
                put!("duration_ns", p.duration_ns);
            }
            EnrichedDomainEvent::CapitalFlow(p) => {
                put!("account_id", p.account_id);
                put!("sign", p.sign.as_str());
                put!("amount", p.amount);
                put!("currency", p.currency);
                put!("timestamp_ns", p.timestamp_ns);
            }
            EnrichedDomainEvent::AccountSnapshot(p) => {
                put!("account_id", p.account_id);
                put!("equity", p.equity);
                put!("balance", p.balance);
                put!("margin_available", p.margin_available);
                put!("margin_required", p.margin_required);
                put!("timestamp_ns", p.timestamp_ns);
            }
            EnrichedDomainEvent::BacktestCompleted(p) => {
                put!("sharpe", p.sharpe);
                put!("drawdown", p.drawdown);
                put!("pbo", p.pbo);
                put!("regime", p.regime);
            }
            EnrichedDomainEvent::RegimeDetected(p) => {
                put!("instrument_id", p.instrument_id);
                put!("regime_label", p.regime_label);
                put!("timestamp_ns", p.timestamp_ns);
            }
            EnrichedDomainEvent::DrawdownDetected(p) => {
                put!("account_id", p.account_id);
                put!("drawdown_pct", p.drawdown_pct);
                put!("timestamp_ns", p.timestamp_ns);
            }
            EnrichedDomainEvent::LiquidityStress(p) => {
                put!("instrument_id", p.instrument_id);
                put!("severity", p.severity);
                put!("timestamp_ns", p.timestamp_ns);
            }
            EnrichedDomainEvent::CorrelationChange(p) => {
                put!("instrument_a", p.instrument_a);
                put!("instrument_b", p.instrument_b);
                put!("correlation", p.correlation);
                put!("timestamp_ns", p.timestamp_ns);
            }
        }

        map
    }

    /// Serializa el payload canónico a JSON (claves ordenadas
    /// alfabéticamente vía `BTreeMap`). Determinista: el mismo evento
    /// lógico siempre produce el mismo string, en cualquier ejecución
    /// (ADR-0002/0004).
    ///
    /// El `.expect` es seguro: el mapa solo contiene claves `String` y
    /// valores `serde_json::Value` construidos a partir de `String`/`i64`
    /// (nunca `f64`/`NaN`/`Infinity`, los únicos casos que hacen fallar la
    /// serialización de `serde_json`), así que esta llamada nunca falla en
    /// la práctica.
    pub fn canonical_payload_json(&self) -> String {
        let map = self.to_canonical_map();
        serde_json::to_string(&map)
            .expect("BTreeMap<String, JsonValue> de solo strings/enteros siempre serializa")
    }
}

// ── Decisión de replicación (ADR-0143, consumiendo el ExecutionGate real) ──

/// Decide si este evento se replica hacia la Cabina de Mando del proveedor,
/// a partir del `ExecutionGate` REAL de `licensing-system` (#2) --
/// `docs/features/enriched-domain-events.md` "Comportamientos Observables":
/// "Cuando la licencia ordena supresión (tier de pago, ADR-0143) -> el
/// emisor hacia la Cabina de Mando se apaga en origen, pero el evento sigue
/// disponible LOCALMENTE para el propio usuario".
///
/// Pura y determinista: no vuelve a evaluar nada del gate (huella, firma,
/// heartbeat) -- ya se evaluó en `derive_execution_gate`. Esta función solo
/// invierte el campo `suppress_work_telemetry` que ese veredicto ya trae:
/// si el gate suprime telemetría de trabajo, este evento NO se replica
/// (`false`); si no suprime, sí se replica (`true`). La persistencia LOCAL
/// del evento ocurre SIEMPRE, sin importar este resultado -- lo que decide
/// es únicamente el envío hacia el proveedor (adaptador futuro, diferido).
pub fn decide_replication(gate: &ExecutionGate) -> bool {
    !gate.suppress_work_telemetry
}

// ── Hash de auditoría encadenado (event_sequence_id, APPEND-ONLY) ──────────

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de una fila de
/// `domain_events`, encadenado al `audit_hash` de la fila anterior en la
/// secuencia GLOBAL (o [`crate::domain::audit_log::GENESIS_PREVIOUS_HASH`]
/// si es la fila génesis, `event_sequence_id == 1`). Mismo patrón que
/// `usage_metering::compute_usage_audit_hash` / `consent_registry::compute_consent_audit_hash`
/// -- la cadena es GLOBAL sobre toda la tabla porque `domain_events` es
/// APPEND-ONLY (ADR-0141: `event_sequence_id UNIQUE`).
#[allow(clippy::too_many_arguments)]
pub fn compute_event_audit_hash(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    previous_audit_hash: &str,
    owner_id: &str,
    institutional_tag: &str,
    node_id: &str,
    process_id: &str,
    session_id: Option<&str>,
    event_type: &str,
    payload_json: &str,
    replicate: bool,
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
    push(process_id);
    push(session_id.unwrap_or(""));
    push(event_type);
    push(payload_json);
    push(if replicate { "1" } else { "0" });

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    let digest = hasher.finalize();

    encode_hex(&digest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::licensing_system::{GateVerdict, LicenseTier};

    // ── CRITERIO #3 (Orden §5): catálogo + serialización determinista ──────

    fn sample_order_executed() -> EnrichedDomainEvent {
        EnrichedDomainEvent::OrderExecuted(OrderExecutedPayload {
            instrument_id: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            quantity: 250_000_000,
            price: 4_000_000_000_000,
            slippage: 1_000_000,
            fill_time_ns: 1_000,
            broker: "IBKR".to_string(),
            notional: 10_000_000_000_000,
            account_id: "acc-1".to_string(),
            realized_pnl: 500_000_000,
            mae: -200_000_000,
            mfe: 800_000_000,
            duration_ns: 3_600_000_000_000,
        })
    }

    /// CRITERIO DE CIERRE: el mismo evento lógico (construido dos veces de
    /// forma independiente) produce EXACTAMENTE el mismo string JSON --
    /// determinismo de la serialización canónica (ADR-0002/0004). Si el
    /// payload usara un `HashMap` en vez de `BTreeMap`, esta prueba podría
    /// fallar de forma intermitente entre ejecuciones del proceso.
    #[test]
    fn canonical_payload_json_is_deterministic_across_independent_constructions() {
        let event_a = sample_order_executed();
        let event_b = sample_order_executed();

        assert_eq!(event_a.canonical_payload_json(), event_b.canonical_payload_json());
    }

    /// CRITERIO DE CIERRE: `event_type()` devuelve exactamente el string
    /// que acepta el `CHECK` de la migración para cada variante del
    /// catálogo.
    #[test]
    fn event_type_matches_migration_check_catalog_for_every_variant() {
        let events = [
            sample_order_executed(),
            EnrichedDomainEvent::CapitalFlow(CapitalFlowPayload {
                account_id: "acc-1".to_string(),
                sign: CapitalFlowSign::Deposit,
                amount: 100_000_000_000,
                currency: "USD".to_string(),
                timestamp_ns: 1_000,
            }),
            EnrichedDomainEvent::AccountSnapshot(AccountSnapshotPayload {
                account_id: "acc-1".to_string(),
                equity: 1_000_000_000_000,
                balance: 1_000_000_000_000,
                margin_available: 500_000_000_000,
                margin_required: 100_000_000_000,
                timestamp_ns: 1_000,
            }),
            EnrichedDomainEvent::BacktestCompleted(BacktestCompletedPayload {
                sharpe: 150_000_000,
                drawdown: 20_000_000,
                pbo: 5_000_000,
                regime: "TRENDING".to_string(),
            }),
            EnrichedDomainEvent::RegimeDetected(RegimeDetectedPayload {
                instrument_id: "BTCUSDT".to_string(),
                regime_label: "TRENDING".to_string(),
                timestamp_ns: 1_000,
            }),
            EnrichedDomainEvent::DrawdownDetected(DrawdownDetectedPayload {
                account_id: "acc-1".to_string(),
                drawdown_pct: 15_000_000,
                timestamp_ns: 1_000,
            }),
            EnrichedDomainEvent::LiquidityStress(LiquidityStressPayload {
                instrument_id: "BTCUSDT".to_string(),
                severity: "HIGH".to_string(),
                timestamp_ns: 1_000,
            }),
            EnrichedDomainEvent::CorrelationChange(CorrelationChangePayload {
                instrument_a: "BTCUSDT".to_string(),
                instrument_b: "ETHUSDT".to_string(),
                correlation: 85_000_000,
                timestamp_ns: 1_000,
            }),
        ];
        // Array (no `vec!`): tamaño conocido en compilación, sin asignación
        // en heap -- clippy::useless_vec.

        let expected_types = [
            "ORDER_EXECUTED",
            "CAPITAL_FLOW",
            "ACCOUNT_SNAPSHOT",
            "BACKTEST_COMPLETED",
            "REGIME_DETECTED",
            "DRAWDOWN_DETECTED",
            "LIQUIDITY_STRESS",
            "CORRELATION_CHANGE",
        ];

        for (event, expected) in events.iter().zip(expected_types.iter()) {
            assert_eq!(event.event_type(), *expected);
            // Cada payload también debe ser JSON válido -- lo verifica el
            // parseo de vuelta a `serde_json::Value`.
            assert!(serde_json::from_str::<JsonValue>(&event.canonical_payload_json()).is_ok());
        }
    }

    /// CRITERIO DE CIERRE: las claves del payload quedan en orden
    /// alfabético en el JSON serializado -- si se usara un mapa sin orden
    /// garantizado, esta prueba podría fallar de forma intermitente.
    #[test]
    fn canonical_payload_json_keys_are_alphabetically_sorted() {
        let event = EnrichedDomainEvent::CapitalFlow(CapitalFlowPayload {
            account_id: "acc-1".to_string(),
            sign: CapitalFlowSign::Deposit,
            amount: 100_000_000_000,
            currency: "USD".to_string(),
            timestamp_ns: 1_000,
        });

        let json = event.canonical_payload_json();
        // Orden alfabético esperado: account_id, amount, currency, sign, timestamp_ns.
        let expected = r#"{"account_id":"acc-1","amount":100000000000,"currency":"USD","sign":"DEPOSIT","timestamp_ns":1000}"#;
        assert_eq!(json, expected);
    }

    // ── CRITERIO #4 (Orden §5): los 3 de ADR-0145 -- montos i64 ×10⁸ ───────

    /// CRITERIO DE CIERRE: `CapitalFlow` con valores conocidos redondos --
    /// el monto es exactamente el entero ×10⁸ que se pasó, sin
    /// recotización ni pérdida de precisión (imposible con `f64` en montos
    /// grandes).
    #[test]
    fn capital_flow_preserves_exact_integer_amount() {
        let event = EnrichedDomainEvent::CapitalFlow(CapitalFlowPayload {
            account_id: "acc-1".to_string(),
            sign: CapitalFlowSign::Deposit,
            amount: 100_000_000_000, // $1,000.00 * 1e8
            currency: "USD".to_string(),
            timestamp_ns: 1_000,
        });

        let json = event.canonical_payload_json();
        let parsed: JsonValue = serde_json::from_str(&json).expect("JSON válido");
        assert_eq!(parsed["amount"], serde_json::json!(100_000_000_000i64));
        // Ningún valor en el JSON debe llevar punto decimal (nunca f64).
        assert!(!json.contains('.'), "CapitalFlow no debe serializar montos como coma flotante");
    }

    /// CRITERIO DE CIERRE: `AccountSnapshot` -- equity/balance/márgenes
    /// preservan el entero ×10⁸ exacto.
    #[test]
    fn account_snapshot_preserves_exact_integer_amounts() {
        let event = EnrichedDomainEvent::AccountSnapshot(AccountSnapshotPayload {
            account_id: "acc-1".to_string(),
            equity: 1_050_000_000_000,   // $10,500.00 * 1e8
            balance: 1_000_000_000_000,  // $10,000.00 * 1e8
            margin_available: 900_000_000_000,
            margin_required: 100_000_000_000,
            timestamp_ns: 1_000,
        });

        let json = event.canonical_payload_json();
        let parsed: JsonValue = serde_json::from_str(&json).expect("JSON válido");
        assert_eq!(parsed["equity"], serde_json::json!(1_050_000_000_000i64));
        assert_eq!(parsed["balance"], serde_json::json!(1_000_000_000_000i64));
        assert!(!json.contains('.'), "AccountSnapshot no debe serializar montos como coma flotante");
    }

    /// CRITERIO DE CIERRE: `OrderExecuted` reforzado (ADR-0145) -- PnL, MAE,
    /// MFE preservan el entero ×10⁸ exacto, incluidos valores negativos
    /// (MAE representa la peor excursión adversa).
    #[test]
    fn order_executed_reinforced_fields_preserve_exact_integer_amounts() {
        let event = sample_order_executed();
        let json = event.canonical_payload_json();
        let parsed: JsonValue = serde_json::from_str(&json).expect("JSON válido");

        assert_eq!(parsed["account_id"], serde_json::json!("acc-1"));
        assert_eq!(parsed["realized_pnl"], serde_json::json!(500_000_000i64));
        assert_eq!(parsed["mae"], serde_json::json!(-200_000_000i64));
        assert_eq!(parsed["mfe"], serde_json::json!(800_000_000i64));
        assert!(!json.contains('.'), "OrderExecuted no debe serializar montos como coma flotante");
    }

    // ── CRITERIO #5 (Orden §5): decisión de replicación, gate real ─────────

    fn gate_with_suppression(suppress: bool) -> ExecutionGate {
        ExecutionGate {
            verdict: GateVerdict::Allow,
            suppress_work_telemetry: suppress,
            tier: LicenseTier::Sovereign,
            activations: 1,
            reason: "licencia válida dentro de los límites del plan".to_string(),
        }
    }

    /// CRITERIO DE CIERRE: un gate que suprime telemetría de trabajo
    /// (Sovereign al corriente) produce `replicate = false`.
    #[test]
    fn decide_replication_is_false_when_gate_suppresses_telemetry() {
        assert!(!decide_replication(&gate_with_suppression(true)));
    }

    /// CRITERIO DE CIERRE: un gate que NO suprime (Explorer, o Sovereign
    /// vencido) produce `replicate = true`.
    #[test]
    fn decide_replication_is_true_when_gate_does_not_suppress_telemetry() {
        assert!(decide_replication(&gate_with_suppression(false)));
    }

    // ── CRITERIO #7 (Orden §5): sin secretos (ADR-0093) ─────────────────────

    /// CRITERIO DE CIERRE (guardarraíl ADR-0093): ningún payload de ninguna
    /// variante del catálogo puede contener una credencial de bróker, una
    /// IP de servidor live, ni una clave de firma -- assert sobre el JSON
    /// serializado de cada variante de ejemplo.
    #[test]
    fn no_payload_variant_leaks_secret_looking_fields() {
        let events = [
            sample_order_executed(),
            EnrichedDomainEvent::CapitalFlow(CapitalFlowPayload {
                account_id: "acc-1".to_string(),
                sign: CapitalFlowSign::Withdrawal,
                amount: 47_698_000_000,
                currency: "USD".to_string(),
                timestamp_ns: 1_000,
            }),
            EnrichedDomainEvent::AccountSnapshot(AccountSnapshotPayload {
                account_id: "acc-1".to_string(),
                equity: 1_000_000_000_000,
                balance: 1_000_000_000_000,
                margin_available: 500_000_000_000,
                margin_required: 100_000_000_000,
                timestamp_ns: 1_000,
            }),
        ];

        for event in &events {
            let json = event.canonical_payload_json().to_lowercase();
            for forbidden in [
                "password", "api_key", "api-key", "broker_secret", "private_key",
                "signing_key", "investor_password", "192.168.", "10.0.0.",
            ] {
                assert!(!json.contains(forbidden), "el payload de {} no debe contener '{forbidden}'", event.event_type());
            }
        }
    }

    // ── Hash de auditoría encadenado ─────────────────────────────────────────

    #[test]
    fn compute_event_audit_hash_is_deterministic() {
        let hash_a = compute_event_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1", "process-1", None,
            "CAPITAL_FLOW", "{}", true,
        );
        let hash_b = compute_event_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1", "process-1", None,
            "CAPITAL_FLOW", "{}", true,
        );
        assert_eq!(hash_a, hash_b);
    }

    /// CRITERIO DE CIERRE: cambiar `replicate` cambia el hash -- si el
    /// campo no entrara en el hash, esta prueba fallaría con hashes
    /// iguales.
    #[test]
    fn compute_event_audit_hash_changes_when_replicate_changes() {
        let replicating = compute_event_audit_hash(
            "id-1", 2_000, 2, "prev-hash", "owner-1", "DRASUS_LOCAL", "node-1", "process-1",
            Some("session-1"), "ORDER_EXECUTED", "{\"a\":1}", true,
        );
        let suppressed = compute_event_audit_hash(
            "id-1", 2_000, 2, "prev-hash", "owner-1", "DRASUS_LOCAL", "node-1", "process-1",
            Some("session-1"), "ORDER_EXECUTED", "{\"a\":1}", false,
        );
        assert_ne!(replicating, suppressed, "cambiar replicate debe cambiar el hash de auditoría");
    }

    #[test]
    fn compute_event_audit_hash_changes_when_payload_changes() {
        let original = compute_event_audit_hash(
            "id-1", 2_000, 2, "prev-hash", "owner-1", "DRASUS_LOCAL", "node-1", "process-1", None,
            "CAPITAL_FLOW", "{\"amount\":100}", true,
        );
        let changed = compute_event_audit_hash(
            "id-1", 2_000, 2, "prev-hash", "owner-1", "DRASUS_LOCAL", "node-1", "process-1", None,
            "CAPITAL_FLOW", "{\"amount\":200}", true,
        );
        assert_ne!(original, changed, "cambiar el payload debe cambiar el hash de auditoría");
    }

    // ── OrderSide / CapitalFlowSign: round-trip de representación en texto ──

    #[test]
    fn order_side_round_trips_through_its_string_representation() {
        for side in [OrderSide::Buy, OrderSide::Sell] {
            assert_eq!(OrderSide::from_str_value(side.as_str()), Some(side));
        }
        assert_eq!(OrderSide::from_str_value("UNKNOWN"), None);
    }

    #[test]
    fn capital_flow_sign_round_trips_through_its_string_representation() {
        for sign in [CapitalFlowSign::Deposit, CapitalFlowSign::Withdrawal, CapitalFlowSign::Transfer] {
            assert_eq!(CapitalFlowSign::from_str_value(sign.as_str()), Some(sign));
        }
        assert_eq!(CapitalFlowSign::from_str_value("UNKNOWN"), None);
    }
}
