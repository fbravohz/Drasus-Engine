//! [CORE] Lógica pura del Registro de Cuentas Verificadas Drasus
//! (`docs/features/verified-account-registry.md`, ADR-0145 cimiento #10 --
//! rector, ADR-0143, ADR-0093, ADR-0141, ADR-0020, ADR-0137, ADR-0002,
//! STORY-037).
//!
//! Sin I/O, sin reloj de sistema, sin aleatoriedad sin semilla
//! (ADR-0002/0004). Este módulo calcula el track record verificado de una
//! cuenta de trading a partir de los eventos de dominio enriquecidos de #6
//! (`crate::domain::enriched_domain_events`), y decide si una cuenta puede
//! publicarse.
//!
//! ## EL diferenciador: el gain% EXCLUYE el flujo de capital (regla obligatoria #2)
//!
//! Un depósito o un retiro NUNCA es ganancia. [`compute_track_record`] suma
//! el `realized_pnl` de los eventos `OrderExecuted` -- que es un campo
//! COMPLETAMENTE SEPARADO de los eventos `CapitalFlow` en el catálogo de #6
//! -- y jamás mezcla ambos. `total_deposits_e8`/`total_withdrawals_e8` se
//! reportan por transparencia, pero nunca entran a la suma de
//! `total_realized_pnl_e8` ni al numerador de `gain_pct_e8`.
//!
//! ## Dos ámbitos de atestación, distinción INVIOLABLE (regla obligatoria #1)
//!
//! [`AttestationScope::Sovereign`] (ejecución propia atestada por la cadena
//! de hash del audit-log) y [`AttestationScope::BrokerReadonly`] (cuenta-
//! completa reportada, computada localmente) NUNCA se confunden. Solo un
//! track `Sovereign` puede reclamar "Ejecución Verificada por Drasus"
//! ([`AttestedTrackRecord::is_attested_by_drasus`]).
//!
//! ## `signature_hash` vs. `audit_hash` (ADR-0020 Perfil D, subset V)
//!
//! Igual que `institutional_report_engine` (#7): [`compute_track_record_signature`]
//! firma REPRODUCIBLE del CONTENIDO del track (mismo contenido, mismo hash,
//! sin importar cuándo se recalcule); [`compute_track_record_audit_hash`]
//! protege la integridad de LA FILA del ledger `attested_track_records`
//! (encadenado por `event_sequence_id`, mismo patrón que
//! `enriched_domain_events::compute_event_audit_hash`).
//!
//! ## Todos los montos son `i64` escalados ×10⁸ (ADR-0141)
//!
//! Equity, balance, drawdown, gain%, PnL, depósitos y retiros son enteros
//! ×10⁸. Ninguna función de este módulo devuelve ni persiste un `f64`.

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};

use crate::domain::consent_registry::ConsentVerdict;
use crate::domain::enriched_domain_events::{CapitalFlowSign, EnrichedDomainEvent};

/// Codifica bytes crudos a su representación hexadecimal en minúsculas
/// (mismo patrón que `enriched_domain_events::encode_hex` /
/// `institutional_report_engine::encode_hex`).
fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Nanosegundos en un día -- usado para agrupar `fill_time_ns` en días de
/// trading distintos ([`compute_track_record`]).
pub const NS_PER_DAY: i64 = 86_400_000_000_000;

// ── Tipo de cuenta (columna `account_type`) ─────────────────────────────────

/// Tipo de cuenta de trading (`docs/features/verified-account-registry.md`
/// "Comportamientos Observables": "bróker, apalancamiento, divisa y tipo
/// (fondeo/prop/propio)").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountType {
    /// Cuenta de fondeo (prop firm que financia con capital de un tercero).
    Funded,
    /// Cuenta prop (capital propio de una firma, distinto de fondeo retail).
    Prop,
    /// Capital propio del usuario (retail).
    Own,
}

impl AccountType {
    /// Representación canónica en texto -- la que acepta el
    /// `CHECK (account_type IN (...))` de la migración.
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountType::Funded => "FUNDED",
            AccountType::Prop => "PROP",
            AccountType::Own => "OWN",
        }
    }

    /// Reconstruye el tipo desde su representación en texto, o `None` si no
    /// es ninguno de los tres reconocidos (integridad de datos).
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "FUNDED" => Some(AccountType::Funded),
            "PROP" => Some(AccountType::Prop),
            "OWN" => Some(AccountType::Own),
            _ => None,
        }
    }
}

// ── Estado de publicación (columna `publication_status`) ───────────────────

/// Estado de publicación de una cuenta verificada
/// (`docs/features/verified-account-registry.md` "Restricciones": "el
/// default es privado"). `PUBLICATION_DEFAULT = privado` es FIJO.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationStatus {
    Private,
    Public,
}

impl PublicationStatus {
    /// Representación canónica en texto -- la que acepta el
    /// `CHECK (publication_status IN (...))` de la migración.
    pub fn as_str(&self) -> &'static str {
        match self {
            PublicationStatus::Private => "PRIVATE",
            PublicationStatus::Public => "PUBLIC",
        }
    }

    /// Reconstruye el estado desde su representación en texto, o `None` si
    /// no es ninguno de los dos reconocidos.
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "PRIVATE" => Some(PublicationStatus::Private),
            "PUBLIC" => Some(PublicationStatus::Public),
            _ => None,
        }
    }
}

// ── Ámbito de atestación (columna `scope`) -- regla obligatoria #1, FIJO ────

/// Ámbito de atestación de UN track record (`docs/adr/ADR-0145.md` "Modelo
/// de confianza"): [`AttestationScope::Sovereign`] es la porción de
/// actividad que fluyó por el motor Drasus y queda atestada por la cadena
/// de hash del audit-log; [`AttestationScope::BrokerReadonly`] es el
/// balance/equity que el bróker reporta (incluye actividad ajena a Drasus,
/// como trades manuales u otros EAs) -- Drasus NO puede atestiguar esto por
/// sí solo. La distinción es INVIOLABLE: solo un track `Sovereign` puede
/// reclamar "Ejecución Verificada por Drasus".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AttestationScope {
    Sovereign,
    BrokerReadonly,
}

impl AttestationScope {
    /// Representación canónica en texto -- la que acepta el
    /// `CHECK (scope IN (...))` de la migración.
    pub fn as_str(&self) -> &'static str {
        match self {
            AttestationScope::Sovereign => "SOVEREIGN",
            AttestationScope::BrokerReadonly => "BROKER_READONLY",
        }
    }

    /// Reconstruye el ámbito desde su representación en texto, o `None` si
    /// no es ninguno de los dos reconocidos.
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "SOVEREIGN" => Some(AttestationScope::Sovereign),
            "BROKER_READONLY" => Some(AttestationScope::BrokerReadonly),
            _ => None,
        }
    }

    /// `true` SOLO para [`AttestationScope::Sovereign`] -- EL punto
    /// estructural que impide presentar un track `BrokerReadonly` como
    /// atestado por Drasus (regla obligatoria #1, ADR-0145: "NUNCA se
    /// presenta un dato 'reportado por el bróker' como 'verificado por
    /// Drasus'"). Este método es la ÚNICA fuente de verdad sobre si un
    /// track puede llevar el sello "Ejecución Verificada por Drasus"; nada
    /// en este módulo decide esa etiqueta por otra vía.
    pub fn is_sovereign_attestation(&self) -> bool {
        matches!(self, AttestationScope::Sovereign)
    }
}

/// Serializa un conjunto de ámbitos de atestación a JSON canónico y
/// determinista -- usa un `BTreeSet<&str>` para deduplicar y ordenar
/// alfabéticamente ANTES de serializar (mismo motivo que
/// `consent_registry::apply_consent_action` con `BTreeMap`: dos ejecuciones
/// con el mismo conjunto lógico deben producir EXACTAMENTE el mismo string).
pub fn canonical_attestation_scopes_json(scopes: &[AttestationScope]) -> String {
    let unique_sorted: std::collections::BTreeSet<&'static str> =
        scopes.iter().map(AttestationScope::as_str).collect();
    serde_json::to_string(&unique_sorted)
        // Un BTreeSet<&str> siempre serializa -- nunca falla en la práctica.
        .expect("BTreeSet<&str> de ámbitos siempre serializa")
}

/// Error al decodificar la columna `attestation_scopes` -- una fila
/// persistida con JSON corrupto o un ámbito fuera del catálogo (integridad
/// de datos, no debería ocurrir si el `CHECK(json_valid)` y el catálogo de
/// escritura se respetan, pero se maneja explícitamente en vez de con
/// `panic!`).
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum AttestationScopeDecodeError {
    #[error("attestation_scopes no es JSON válido: {0}")]
    InvalidJson(String),
    #[error("ámbito de atestación desconocido en attestation_scopes: '{0}'")]
    UnknownScope(String),
}

/// Reconstruye la lista de ámbitos desde el JSON canónico persistido, o
/// `Err` si el contenido está corrupto o trae un ámbito desconocido.
pub fn decode_attestation_scopes_json(json: &str) -> Result<Vec<AttestationScope>, AttestationScopeDecodeError> {
    let raw: Vec<String> = serde_json::from_str(json)
        .map_err(|e| AttestationScopeDecodeError::InvalidJson(e.to_string()))?;

    raw.iter()
        .map(|value| {
            AttestationScope::from_str_value(value)
                .ok_or_else(|| AttestationScopeDecodeError::UnknownScope(value.clone()))
        })
        .collect()
}

// ── Realidad de capital (interpreta la columna `institutional_tag`) -- Eje B, ──
// ── ORTOGONAL al Eje A (`scope`), corrección ADR-0145 2026-07-07 ────────────

/// Realidad del capital arriesgado por UNA cuenta/track (`docs/adr/ADR-0145.md`
/// "Modelo de confianza", corregido 2026-07-07 -- STORY-041/DEBT-016): el Eje
/// B, ORTOGONAL al ámbito de atestación ([`AttestationScope`], Eje A -- quién
/// ejecutó). Una cuenta en [`CapitalReality::Paper`]/[`CapitalReality::Demo`]/
/// [`CapitalReality::Challenge`] corre en el MISMO entorno determinista de
/// ejecución que [`CapitalReality::Live`] (NO es backtesting) -- por eso es
/// igualmente atestiguable por Drasus (el Eje A puede ser `Sovereign`), solo
/// que arriesga capital virtual en vez de real. Un track `Sovereign` +
/// `Paper` es perfectamente válido: atestado (Eje A) pero de capital virtual
/// (Eje B) -- los dos ejes nunca se condicionan entre sí.
///
/// ## Este tipo NO tiene columna propia -- interpreta `institutional_tag`
///
/// `verified_accounts`/`attested_track_records` YA tenían `institutional_tag`
/// (Grupo II, ADR-0020, obligatorio en su Perfil D). Guardar el Eje B en una
/// columna nueva `capital_reality` habría duplicado ese dominio de valores en
/// la misma fila -- violación de "reutilización antes que creación"
/// (ADR-0144 FIJO). La corrección (ADR-0145 2026-07-07): `institutional_tag`
/// ES el Eje B en estas dos tablas, con su vocabulario extendido a
/// `LIVE`/`PAPER`/`DEMO`/`CHALLENGE`. [`CapitalReality`] sigue existiendo
/// como tipo de dominio porque le da tipado fuerte y `is_real_capital()` a un
/// `String` -- pero la fuente de ese `String` es siempre `institutional_tag`,
/// nunca una columna aparte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapitalReality {
    Live,
    Paper,
    Demo,
    Challenge,
}

impl CapitalReality {
    /// Representación canónica en texto -- la que acepta el
    /// `CHECK (capital_reality IN (...))` de la migración.
    pub fn as_str(&self) -> &'static str {
        match self {
            CapitalReality::Live => "LIVE",
            CapitalReality::Paper => "PAPER",
            CapitalReality::Demo => "DEMO",
            CapitalReality::Challenge => "CHALLENGE",
        }
    }

    /// Reconstruye la realidad de capital desde su representación en texto,
    /// o `None` si no es ninguna de las cuatro reconocidas (integridad de
    /// datos).
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "LIVE" => Some(CapitalReality::Live),
            "PAPER" => Some(CapitalReality::Paper),
            "DEMO" => Some(CapitalReality::Demo),
            "CHALLENGE" => Some(CapitalReality::Challenge),
            _ => None,
        }
    }

    /// `true` SOLO para [`CapitalReality::Live`] -- EL punto estructural que
    /// impide presentar un track `PAPER`/`DEMO`/`CHALLENGE` como si arriesgó
    /// capital real (ADR-0145 corregido: "nunca se omite ni se confunde con
    /// LIVE"). Este método es la ÚNICA fuente de verdad sobre si un track
    /// puede llevar la etiqueta "Cuenta LIVE (capital real)"; es
    /// COMPLETAMENTE INDEPENDIENTE de
    /// [`AttestationScope::is_sovereign_attestation`] (Eje A) -- un track
    /// puede ser atestado y de capital virtual a la vez, y este método NUNCA
    /// consulta el `scope` para decidir su resultado.
    pub fn is_real_capital(&self) -> bool {
        matches!(self, CapitalReality::Live)
    }
}

// ── Track record calculado (Core puro) ──────────────────────────────────────

/// Las métricas de un track record calculado a partir de una lista de
/// eventos de #6 -- salida de [`compute_track_record`], entrada de
/// [`compute_track_record_signature`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrackRecordMetrics {
    /// Curva de equidad: pares `(timestamp_ns, equity_e8)`, ordenados
    /// cronológicamente (de los eventos `AccountSnapshot`).
    pub equity_curve: Vec<(i64, i64)>,
    /// Curva de balance: pares `(timestamp_ns, balance_e8)`.
    pub balance_curve: Vec<(i64, i64)>,
    /// Drawdown máximo (fracción de 0 a 1) sobre la curva de equidad,
    /// entero ×10⁸.
    pub max_drawdown_e8: i64,
    /// Ganancia porcentual, fracción ×10⁸ (ej. 4.41 -> 441000000 = 441%).
    /// EXCLUYE depósitos/retiros -- regla obligatoria #2.
    pub gain_pct_e8: i64,
    /// Fracción de trades rentables (0 a 1), entero ×10⁸.
    pub win_rate_e8: i64,
    /// Tiempo medio de espera de un trade, nanosegundos.
    pub avg_holding_time_ns: i64,
    /// Número de días de trading distintos (por `fill_time_ns`).
    pub trading_days: i64,
    /// Suma de `realized_pnl` de todas las órdenes de la cuenta, ×10⁸.
    pub total_realized_pnl_e8: i64,
    /// Suma de depósitos (`CapitalFlowSign::Deposit`), ×10⁸ -- NUNCA entra
    /// a `total_realized_pnl_e8` ni a `gain_pct_e8`.
    pub total_deposits_e8: i64,
    /// Suma de retiros (`CapitalFlowSign::Withdrawal`), ×10⁸ -- NUNCA entra
    /// a `total_realized_pnl_e8` ni a `gain_pct_e8`.
    pub total_withdrawals_e8: i64,
}

/// Calcula el track record de UNA cuenta (`account_id`, el campo que
/// `EnrichedDomainEvent::OrderExecuted`/`CapitalFlow`/`AccountSnapshot`
/// llevan) a partir de una lista de eventos de #6 -- pura, determinista,
/// sin I/O. Ignora cualquier evento de otra cuenta o de otra variante
/// (`BacktestCompleted`, `RegimeDetected`, etc. -- no aportan al track).
///
/// ## Por qué el gain% nunca puede incluir el flujo de capital
///
/// `total_realized_pnl_e8` se suma EXCLUSIVAMENTE de la variante
/// `OrderExecuted` (campo `realized_pnl`); los depósitos/retiros viven en
/// la variante SEPARADA `CapitalFlow` y se acumulan en columnas propias
/// (`total_deposits_e8`/`total_withdrawals_e8`) que jamás se suman al PnL.
/// Es estructuralmente imposible que un depósito infle `gain_pct_e8`,
/// porque el `match` de abajo nunca mezcla las dos ramas.
pub fn compute_track_record(events: &[EnrichedDomainEvent], account_id: &str) -> TrackRecordMetrics {
    let mut snapshot_points: Vec<(i64, i64, i64)> = Vec::new(); // (timestamp_ns, equity, balance)
    let mut realized_pnls: Vec<i64> = Vec::new();
    let mut durations_ns: Vec<i64> = Vec::new();
    let mut fill_days: Vec<i64> = Vec::new();
    let mut total_deposits: i128 = 0;
    let mut total_withdrawals: i128 = 0;

    for event in events {
        match event {
            EnrichedDomainEvent::AccountSnapshot(p) if p.account_id == account_id => {
                snapshot_points.push((p.timestamp_ns, p.equity, p.balance));
            }
            EnrichedDomainEvent::OrderExecuted(p) if p.account_id == account_id => {
                realized_pnls.push(p.realized_pnl);
                durations_ns.push(p.duration_ns);
                fill_days.push(p.fill_time_ns.div_euclid(NS_PER_DAY));
            }
            EnrichedDomainEvent::CapitalFlow(p) if p.account_id == account_id => {
                // Rama COMPLETAMENTE SEPARADA de OrderExecuted -- lo único
                // que hace es acumular en las columnas de transparencia;
                // NUNCA toca `realized_pnls`.
                match p.sign {
                    CapitalFlowSign::Deposit => total_deposits += i128::from(p.amount),
                    CapitalFlowSign::Withdrawal => total_withdrawals += i128::from(p.amount),
                    // Una transferencia interna no es ni depósito externo
                    // ni retiro externo -- no se cuenta en ninguno de los
                    // dos totales (ADR-0145 no la trata como capital
                    // externo aportado/retirado).
                    CapitalFlowSign::Transfer => {}
                }
            }
            // Cualquier otra variante, o un evento de OTRA cuenta: no
            // aporta a este track.
            _ => {}
        }
    }

    // Curvas cronológicas -- orden ascendente por timestamp.
    snapshot_points.sort_by_key(|&(ts, _, _)| ts);
    let equity_curve: Vec<(i64, i64)> = snapshot_points.iter().map(|&(ts, eq, _)| (ts, eq)).collect();
    let balance_curve: Vec<(i64, i64)> = snapshot_points.iter().map(|&(ts, _, bal)| (ts, bal)).collect();

    // Drawdown máximo: pico-a-valle sobre la curva de equidad, fracción ×10⁸.
    let mut peak: i64 = i64::MIN;
    let mut max_drawdown_e8: i64 = 0;
    for &(_, equity) in &equity_curve {
        if equity > peak {
            peak = equity;
        }
        if peak > 0 {
            let drawdown_e8 = (i128::from(peak - equity) * 100_000_000) / i128::from(peak);
            if drawdown_e8 > i128::from(max_drawdown_e8) {
                max_drawdown_e8 = drawdown_e8 as i64;
            }
        }
    }

    let total_realized_pnl: i128 = realized_pnls.iter().map(|&pnl| i128::from(pnl)).sum();
    let winning_trades = realized_pnls.iter().filter(|&&pnl| pnl > 0).count();
    let win_rate_e8: i64 = if realized_pnls.is_empty() {
        0
    } else {
        ((winning_trades as i128 * 100_000_000) / realized_pnls.len() as i128) as i64
    };
    let avg_holding_time_ns: i64 = if durations_ns.is_empty() {
        0
    } else {
        let sum_ns: i128 = durations_ns.iter().map(|&d| i128::from(d)).sum();
        (sum_ns / durations_ns.len() as i128) as i64
    };

    fill_days.sort_unstable();
    fill_days.dedup();
    let trading_days = fill_days.len() as i64;

    // Capital base para el gain%: el balance del PRIMER snapshot
    // cronológico (el capital con el que la cuenta arrancó a operar). Sin
    // ningún snapshot, aproxima con el capital neto aportado
    // (depósitos - retiros, nunca negativo) -- ESTE es el único lugar
    // donde el flujo de capital participa del cálculo, y SOLO como
    // denominador (cuánto capital había para trabajar), jamás como
    // numerador (cuánta ganancia hubo).
    let capital_base: i128 = balance_curve
        .first()
        .map(|&(_, balance)| i128::from(balance))
        .unwrap_or_else(|| (total_deposits - total_withdrawals).max(1));

    let gain_pct_e8: i64 = if capital_base <= 0 {
        0
    } else {
        ((total_realized_pnl * 100_000_000) / capital_base) as i64
    };

    TrackRecordMetrics {
        equity_curve,
        balance_curve,
        max_drawdown_e8,
        gain_pct_e8,
        win_rate_e8,
        avg_holding_time_ns,
        trading_days,
        total_realized_pnl_e8: total_realized_pnl as i64,
        total_deposits_e8: total_deposits as i64,
        total_withdrawals_e8: total_withdrawals as i64,
    }
}

/// Serializa `metrics` a un `BTreeMap<String, JsonValue>` canónico -- claves
/// ordenadas alfabéticamente (mismo patrón que
/// `institutional_report_engine::InstitutionalReport::to_canonical_map`).
fn metrics_to_canonical_map(
    metrics: &TrackRecordMetrics,
    scope: AttestationScope,
    capital_reality: CapitalReality,
    verified_account_id: &str,
    time_window: &str,
) -> BTreeMap<String, JsonValue> {
    let mut map = BTreeMap::new();
    map.insert("account_id".to_string(), serde_json::json!(verified_account_id));
    map.insert("avg_holding_time_ns".to_string(), serde_json::json!(metrics.avg_holding_time_ns));
    map.insert("balance_curve".to_string(), serde_json::json!(metrics.balance_curve));
    // Eje B en la firma: el MISMO contenido de métricas con `capital_reality`
    // distinto (ej. LIVE vs PAPER) debe producir una firma distinta -- de lo
    // contrario un track PAPER podría falsificarse como LIVE sin que la
    // firma lo detecte. La clave canónica sigue llamándose "capital_reality"
    // (nombre lógico del dato firmado, preserva la firma existente) aunque
    // el valor proviene de la columna `institutional_tag` (STORY-041): quien
    // llama parsea `institutional_tag` a `CapitalReality` ANTES de invocar
    // esta función, nunca lee una columna `capital_reality` separada.
    map.insert("capital_reality".to_string(), serde_json::json!(capital_reality.as_str()));
    map.insert("equity_curve".to_string(), serde_json::json!(metrics.equity_curve));
    map.insert("gain_pct_e8".to_string(), serde_json::json!(metrics.gain_pct_e8));
    map.insert("max_drawdown_e8".to_string(), serde_json::json!(metrics.max_drawdown_e8));
    map.insert("scope".to_string(), serde_json::json!(scope.as_str()));
    map.insert("time_window".to_string(), serde_json::json!(time_window));
    map.insert("total_deposits_e8".to_string(), serde_json::json!(metrics.total_deposits_e8));
    map.insert("total_realized_pnl_e8".to_string(), serde_json::json!(metrics.total_realized_pnl_e8));
    map.insert("total_withdrawals_e8".to_string(), serde_json::json!(metrics.total_withdrawals_e8));
    map.insert("trading_days".to_string(), serde_json::json!(metrics.trading_days));
    map.insert("win_rate_e8".to_string(), serde_json::json!(metrics.win_rate_e8));
    map
}

/// Calcula la firma de integridad SHA-256 (hex, minúsculas) REPRODUCIBLE
/// del CONTENIDO de un track record -- regla obligatoria #5 (subset V,
/// ADR-0020): el mismo track, recalculado sobre los MISMOS eventos, produce
/// EXACTAMENTE la misma firma; cambiar una sola métrica cambia la firma.
/// Mismo rol que `institutional_report_engine::compute_report_signature`,
/// aplicado al track record en vez del reporte.
pub fn compute_track_record_signature(
    metrics: &TrackRecordMetrics,
    scope: AttestationScope,
    capital_reality: CapitalReality,
    verified_account_id: &str,
    time_window: &str,
) -> String {
    let map = metrics_to_canonical_map(metrics, scope, capital_reality, verified_account_id, time_window);
    let json = serde_json::to_string(&map)
        // El mapa solo contiene String/i64/Vec<(i64,i64)> -- nunca f64/NaN,
        // los únicos casos que hacen fallar la serialización de serde_json.
        .expect("BTreeMap<String, JsonValue> del track siempre serializa");

    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    encode_hex(&hasher.finalize())
}

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de una fila de
/// `attested_track_records`, encadenado al `audit_hash` de la fila anterior
/// en la secuencia GLOBAL (o [`crate::domain::audit_log::GENESIS_PREVIOUS_HASH`]
/// si es la fila génesis). Mismo patrón que
/// `enriched_domain_events::compute_event_audit_hash` -- protege la
/// integridad DE LA FILA en el ledger, distinto de `signature_hash`, que
/// protege el CONTENIDO del track.
///
/// `institutional_tag` y `capital_reality` reciben el MISMO valor de texto
/// en quien llama (STORY-041): en esta tabla `institutional_tag` ES el Eje B
/// (ver [`CapitalReality`]) -- no hay una columna `capital_reality` aparte
/// en la migración. Se conservan como dos parámetros para no romper la
/// posición de los campos ya encadenados en el `audit_hash` de filas
/// existentes ni las pruebas de determinismo que ya cubren esta función.
#[allow(clippy::too_many_arguments)]
pub fn compute_track_record_audit_hash(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    previous_audit_hash: &str,
    owner_id: &str,
    institutional_tag: &str,
    node_id: &str,
    verified_account_id: &str,
    scope: &str,
    time_window: &str,
    signature_hash: &str,
    capital_reality: &str,
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
    push(verified_account_id);
    push(scope);
    push(time_window);
    push(signature_hash);
    // Eje B, posición fija al final del buffer -- encadena la realidad de
    // capital dentro del audit_hash de la fila (integridad del ledger). El
    // valor es el mismo `institutional_tag` de arriba (STORY-041): no existe
    // una columna `capital_reality` separada en la migración.
    push(capital_reality);

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    encode_hex(&hasher.finalize())
}

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de una VERSIÓN de fila
/// de `verified_accounts`, encadenado a la versión anterior de la MISMA
/// fila (`previous_audit_hash: None` en la versión génesis, `row_version ==
/// 1`). Mismo patrón que `central_identity::compute_account_audit_hash` --
/// tabla MUTABLE, se encadena por versión de fila, no por secuencia global.
///
/// `institutional_tag` y `capital_reality` reciben el MISMO valor de texto
/// en quien llama (STORY-041): en esta tabla `institutional_tag` ES el Eje B
/// -- no hay columna `capital_reality` aparte en la migración. `capital_reality`
/// se mantiene como parámetro tipado ([`CapitalReality`]) para no romper las
/// pruebas de determinismo existentes ni la posición de los campos ya
/// encadenados.
#[allow(clippy::too_many_arguments)]
pub fn compute_verified_account_audit_hash(
    id: &str,
    created_at_ns: i64,
    row_version: i64,
    previous_audit_hash: Option<&str>,
    owner_id: &str,
    institutional_tag: &str,
    node_id: &str,
    broker: &str,
    currency: &str,
    account_type: AccountType,
    publication_status: PublicationStatus,
    attestation_scopes_json: &str,
    broker_connection_ref: Option<&str>,
    capital_reality: CapitalReality,
) -> String {
    const SEP: char = '\u{1F}';

    let mut buffer = String::new();
    let mut push = |field: &str| {
        buffer.push_str(field);
        buffer.push(SEP);
    };

    push(id);
    push(&created_at_ns.to_string());
    push(&row_version.to_string());
    push(previous_audit_hash.unwrap_or(""));
    push(owner_id);
    push(institutional_tag);
    push(node_id);
    push(broker);
    push(currency);
    push(account_type.as_str());
    push(publication_status.as_str());
    push(attestation_scopes_json);
    push(broker_connection_ref.unwrap_or(""));
    // Eje B, posición fija al final del buffer -- encadena la realidad de
    // capital dentro del audit_hash de esta versión de fila.
    push(capital_reality.as_str());

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    encode_hex(&hasher.finalize())
}

// ── Gate de publicación (regla obligatoria #4) ──────────────────────────────

/// Decide el nuevo [`PublicationStatus`] de una cuenta -- EL punto de
/// correctitud legal de esta feature (`docs/adr/ADR-0145.md`: "NUNCA se
/// publica sin consentimiento explícito, versionado y fechado").
///
/// - Pedir `Private` SIEMPRE se concede (una cuenta puede volver a privado
///   en cualquier momento, sin necesitar consentimiento para ESO).
/// - Pedir `Public` SOLO se concede si `consent.is_covered()` -- el
///   veredicto REAL de `consent-registry` (#5), resuelto por quien llama
///   ANTES de invocar esta función pura. Sin opt-in vigente, el estado se
///   queda EXACTAMENTE como estaba (`current_status`) -- nunca avanza a
///   `Public` "a medias" ni por defecto.
pub fn decide_publication(
    current_status: PublicationStatus,
    requested_status: PublicationStatus,
    consent: &ConsentVerdict,
) -> PublicationStatus {
    match requested_status {
        PublicationStatus::Private => PublicationStatus::Private,
        PublicationStatus::Public => {
            if consent.is_covered() {
                PublicationStatus::Public
            } else {
                current_status
            }
        }
    }
}

// ── Tipos de puerto (ADR-0137: `registry_out` / `track_record_out`) ────────

/// El tipo de puerto `registry_out` -- una cuenta verificada tal como se
/// expone hacia afuera (panel de cuentas / contrato de reporte futuro).
/// **Guardarraíl ADR-0093 (estructural):** ningún campo de este struct es ni
/// puede ser una credencial -- `broker_connection_ref` es una referencia de
/// texto libre NO secreta.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VerifiedAccountRecord {
    pub id: String,
    pub owner_id: String,
    pub broker: String,
    pub leverage: i64,
    pub currency: String,
    pub account_type: String,
    pub publication_status: String,
    pub attestation_scopes: Vec<String>,
    /// Eje B (`docs/adr/ADR-0145.md` corregido 2026-07-07, STORY-041):
    /// `"LIVE"`, `"PAPER"`, `"DEMO"` o `"CHALLENGE"` -- SIEMPRE presente,
    /// junto al Eje A (`attestation_scopes`), nunca omitido. Copiado
    /// directamente de la columna `institutional_tag` de la fila (que en
    /// esta tabla ES el Eje B, no un campo separado).
    pub capital_reality: String,
    pub broker_connection_ref: Option<String>,
}

/// El tipo de puerto `track_record_out` -- un track record atestado tal
/// como se expone hacia afuera (panel de cuentas / contrato de reporte
/// futuro). `is_attested_by_drasus` es la ÚNICA fuente de la etiqueta
/// visible ("Ejecución Verificada por Drasus" vs. "Reportado por el
/// Bróker") -- se deriva de [`AttestationScope::is_sovereign_attestation`],
/// nunca de un booleano aparte que pudiera desincronizarse del `scope` real.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AttestedTrackRecord {
    pub id: String,
    pub verified_account_id: String,
    pub scope: String,
    /// Eje B (`docs/adr/ADR-0145.md` corregido 2026-07-07, STORY-041):
    /// `"LIVE"`, `"PAPER"`, `"DEMO"` o `"CHALLENGE"` -- estructuralmente
    /// SIEMPRE presente junto al `scope` (Eje A); esta proyección nunca
    /// emite un track con un eje pero no el otro. Copiado directamente de
    /// la columna `institutional_tag` de la fila (que en esta tabla ES el
    /// Eje B, no un campo separado).
    pub capital_reality: String,
    pub time_window: String,
    pub signature_hash: String,
    pub equity_curve: Vec<(i64, i64)>,
    pub balance_curve: Vec<(i64, i64)>,
    pub max_drawdown_e8: i64,
    pub gain_pct_e8: i64,
    pub win_rate_e8: i64,
    pub avg_holding_time_ns: i64,
    pub trading_days: i64,
    pub total_realized_pnl_e8: i64,
    pub total_deposits_e8: i64,
    pub total_withdrawals_e8: i64,
    pub is_attested_by_drasus: bool,
    /// Derivado SOLO de [`CapitalReality::is_real_capital`] -- NUNCA de
    /// `is_attested_by_drasus`. Un track puede ser `is_attested_by_drasus ==
    /// true` (Eje A = Sovereign) y `is_real_capital == false` (Eje B =
    /// Paper/Demo/Challenge) simultáneamente -- son ejes independientes.
    pub is_real_capital: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::enriched_domain_events::{
        AccountSnapshotPayload, CapitalFlowPayload, OrderExecutedPayload, OrderSide,
    };

    // ── Enums: round-trip de representación en texto ────────────────────────

    #[test]
    fn account_type_round_trips_through_its_string_representation() {
        for variant in [AccountType::Funded, AccountType::Prop, AccountType::Own] {
            assert_eq!(AccountType::from_str_value(variant.as_str()), Some(variant));
        }
        assert_eq!(AccountType::from_str_value("UNKNOWN"), None);
    }

    #[test]
    fn publication_status_round_trips_through_its_string_representation() {
        for variant in [PublicationStatus::Private, PublicationStatus::Public] {
            assert_eq!(PublicationStatus::from_str_value(variant.as_str()), Some(variant));
        }
        assert_eq!(PublicationStatus::from_str_value("UNKNOWN"), None);
    }

    #[test]
    fn attestation_scope_round_trips_through_its_string_representation() {
        for variant in [AttestationScope::Sovereign, AttestationScope::BrokerReadonly] {
            assert_eq!(AttestationScope::from_str_value(variant.as_str()), Some(variant));
        }
        assert_eq!(AttestationScope::from_str_value("UNKNOWN"), None);
    }

    #[test]
    fn capital_reality_round_trips_through_its_string_representation() {
        for variant in [
            CapitalReality::Live,
            CapitalReality::Paper,
            CapitalReality::Demo,
            CapitalReality::Challenge,
        ] {
            assert_eq!(CapitalReality::from_str_value(variant.as_str()), Some(variant));
        }
        assert_eq!(CapitalReality::from_str_value("UNKNOWN"), None);
    }

    // ── CRITERIO #3 (Orden §5): ámbito inviolable, assert estructural ───────

    /// CRITERIO DE CIERRE: SOLO `Sovereign` es una atestación de Drasus --
    /// `BrokerReadonly` nunca puede reclamarlo, sin importar nada más.
    #[test]
    fn only_sovereign_scope_is_a_drasus_attestation() {
        assert!(AttestationScope::Sovereign.is_sovereign_attestation());
        assert!(!AttestationScope::BrokerReadonly.is_sovereign_attestation());
    }

    /// CRITERIO DE CIERRE (Eje B): SOLO `Live` es capital real -- las tres
    /// variantes virtuales (`Paper`/`Demo`/`Challenge`) nunca lo reclaman.
    #[test]
    fn only_live_capital_reality_is_real_capital() {
        assert!(CapitalReality::Live.is_real_capital());
        assert!(!CapitalReality::Paper.is_real_capital());
        assert!(!CapitalReality::Demo.is_real_capital());
        assert!(!CapitalReality::Challenge.is_real_capital());
    }

    /// CRITERIO DE CIERRE (EL punto de DEBT-014, Orden §4 punto 14): un
    /// track `SOVEREIGN` + `PAPER` demuestra que los dos ejes son
    /// ORTOGONALES -- atestable (Eje A) NO implica capital real (Eje B).
    /// `is_attested_by_drasus` debe ser `true` (el motor Drasus lo ejecutó,
    /// en el mismo entorno determinista que producción) mientras
    /// `is_real_capital` debe ser `false` y `capital_reality` debe seguir
    /// siendo `"PAPER"` -- jamás se presenta como si fuera LIVE.
    #[test]
    fn sovereign_paper_track_is_attested_but_not_real_capital() {
        let row = AttestedTrackRecord {
            id: "track-sovereign-paper".to_string(),
            verified_account_id: "acc-1".to_string(),
            scope: AttestationScope::Sovereign.as_str().to_string(),
            capital_reality: CapitalReality::Paper.as_str().to_string(),
            time_window: "2026-W27".to_string(),
            signature_hash: "sig-paper".to_string(),
            equity_curve: vec![],
            balance_curve: vec![],
            max_drawdown_e8: 0,
            gain_pct_e8: 0,
            win_rate_e8: 0,
            avg_holding_time_ns: 0,
            trading_days: 0,
            total_realized_pnl_e8: 0,
            total_deposits_e8: 0,
            total_withdrawals_e8: 0,
            is_attested_by_drasus: AttestationScope::Sovereign.is_sovereign_attestation(),
            is_real_capital: CapitalReality::Paper.is_real_capital(),
        };

        assert!(row.is_attested_by_drasus, "SOVEREIGN debe seguir siendo atestado -- el Eje A no cambia");
        assert!(!row.is_real_capital, "PAPER nunca es capital real -- el Eje B es independiente del Eje A");
        assert_eq!(row.capital_reality, "PAPER", "la etiqueta de capital virtual debe seguir visible, nunca LIVE");
    }

    // ── attestation_scopes JSON: canónico, determinista, con round-trip ─────

    #[test]
    fn canonical_attestation_scopes_json_is_deterministic_and_deduplicated() {
        let json_a = canonical_attestation_scopes_json(&[
            AttestationScope::BrokerReadonly,
            AttestationScope::Sovereign,
            AttestationScope::Sovereign, // duplicado a propósito
        ]);
        let json_b = canonical_attestation_scopes_json(&[
            AttestationScope::Sovereign,
            AttestationScope::BrokerReadonly,
        ]);
        // Mismo conjunto lógico (deduplicado), en cualquier orden de
        // construcción -> el MISMO string JSON.
        assert_eq!(json_a, json_b);

        let decoded = decode_attestation_scopes_json(&json_a).expect("debe decodificar");
        assert_eq!(decoded.len(), 2, "los duplicados deben colapsar a un único elemento");
        assert!(decoded.contains(&AttestationScope::Sovereign));
        assert!(decoded.contains(&AttestationScope::BrokerReadonly));
    }

    #[test]
    fn decode_attestation_scopes_json_rejects_invalid_json_and_unknown_scope() {
        assert!(matches!(
            decode_attestation_scopes_json("{not valid"),
            Err(AttestationScopeDecodeError::InvalidJson(_))
        ));
        assert!(matches!(
            decode_attestation_scopes_json(r#"["UNKNOWN_SCOPE"]"#),
            Err(AttestationScopeDecodeError::UnknownScope(_))
        ));
    }

    // ── CRITERIO #2 (Orden §5): gain% EXCLUYE el flujo de capital ───────────

    /// Construye un evento `OrderExecuted` mínimo para `account_id` con el
    /// PnL realizado y la duración dados -- el resto de campos son valores
    /// de relleno sin efecto en el track (instrumento/lado/precio no entran
    /// al cálculo de `compute_track_record`).
    fn order(account_id: &str, realized_pnl: i64, fill_time_ns: i64, duration_ns: i64) -> EnrichedDomainEvent {
        EnrichedDomainEvent::OrderExecuted(OrderExecutedPayload {
            instrument_id: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            quantity: 100_000_000,
            price: 100_000_000_000,
            slippage: 0,
            fill_time_ns,
            broker: "ICMarkets".to_string(),
            notional: 100_000_000_000,
            account_id: account_id.to_string(),
            realized_pnl,
            mae: 0,
            mfe: 0,
            duration_ns,
        })
    }

    fn capital_flow(account_id: &str, sign: CapitalFlowSign, amount: i64) -> EnrichedDomainEvent {
        EnrichedDomainEvent::CapitalFlow(CapitalFlowPayload {
            account_id: account_id.to_string(),
            sign,
            amount,
            currency: "USD".to_string(),
            timestamp_ns: 0,
        })
    }

    fn snapshot(account_id: &str, timestamp_ns: i64, equity: i64, balance: i64) -> EnrichedDomainEvent {
        EnrichedDomainEvent::AccountSnapshot(AccountSnapshotPayload {
            account_id: account_id.to_string(),
            equity,
            balance,
            margin_available: balance,
            margin_required: 0,
            timestamp_ns,
        })
    }

    /// CRITERIO DE CIERRE (EL diferenciador de ADR-0145): con un capital
    /// base fijo de $10,000 (balance del primer snapshot) y trades que
    /// suman exactamente $44,100 de PnL realizado, el gain% es 441%
    /// (441000000 ×10⁸) -- reconstrucción ilustrativa consistente con las
    /// cifras que cita ADR-0145 (depósito de 350, retiro de 476.98, gain
    /// 441%; el ADR no trae el dataset completo original, así que este test
    /// usa un capital base y un PnL propios que reproducen la MISMA
    /// proporción). Añadir el depósito de $350 y el retiro de $476.98 NO
    /// cambia el gain% -- si un defecto sumara el flujo de capital al PnL,
    /// esta aserción fallaría.
    #[test]
    fn gain_pct_excludes_capital_flow_matching_adr_0145_example_proportions() {
        let account_id = "acc-adr-example";
        let deposit_e8 = 35_000_000_000i64; // $350.00 * 1e8
        let withdrawal_e8 = 47_698_000_000i64; // $476.98 * 1e8
        let capital_base_e8 = 1_000_000_000_000i64; // $10,000.00 * 1e8
        let profit_e8 = 4_410_000_000_000i64; // $44,100.00 * 1e8 -> 441% de $10,000

        let events_without_flows = vec![
            snapshot(account_id, 0, capital_base_e8, capital_base_e8),
            order(account_id, profit_e8, NS_PER_DAY, 3_600_000_000_000),
        ];
        let metrics_without_flows = compute_track_record(&events_without_flows, account_id);
        assert_eq!(metrics_without_flows.gain_pct_e8, 441_000_000, "441% esperado sin flujo de capital");

        let mut events_with_flows = events_without_flows.clone();
        events_with_flows.push(capital_flow(account_id, CapitalFlowSign::Deposit, deposit_e8));
        events_with_flows.push(capital_flow(account_id, CapitalFlowSign::Withdrawal, withdrawal_e8));
        let metrics_with_flows = compute_track_record(&events_with_flows, account_id);

        assert_eq!(
            metrics_with_flows.gain_pct_e8, 441_000_000,
            "el depósito y el retiro NUNCA deben cambiar el gain% -- de lo contrario se estaría \
             contando capital aportado/retirado como ganancia"
        );
        assert_eq!(metrics_with_flows.total_deposits_e8, deposit_e8, "el depósito se reporta por transparencia");
        assert_eq!(metrics_with_flows.total_withdrawals_e8, withdrawal_e8, "el retiro se reporta por transparencia");
        assert_eq!(
            metrics_with_flows.total_realized_pnl_e8, metrics_without_flows.total_realized_pnl_e8,
            "el PnL realizado nunca debe incorporar el flujo de capital"
        );
    }

    /// CRITERIO DE CIERRE (contraprueba del anterior): si un defecto SUMARA
    /// el depósito y el retiro al PnL antes de calcular el gain%, el
    /// resultado sería distinto de 441% -- este test demuestra que la suma
    /// ingenua (que la implementación correcta rechaza) SÍ habría cambiado
    /// el número, confirmando que la exclusión es la pieza que hace la
    /// diferencia.
    #[test]
    fn naive_inclusion_of_capital_flow_would_have_produced_a_different_gain_pct() {
        let account_id = "acc-naive-contraprueba";
        let capital_base_e8 = 1_000_000_000_000i64;
        let profit_e8 = 4_410_000_000_000i64;
        let deposit_e8 = 35_000_000_000i64;
        let withdrawal_e8 = 47_698_000_000i64;

        let events = vec![
            snapshot(account_id, 0, capital_base_e8, capital_base_e8),
            order(account_id, profit_e8, NS_PER_DAY, 3_600_000_000_000),
            capital_flow(account_id, CapitalFlowSign::Deposit, deposit_e8),
            capital_flow(account_id, CapitalFlowSign::Withdrawal, withdrawal_e8),
        ];
        let metrics = compute_track_record(&events, account_id);

        // La suma "ingenua" (defecto hipotético): PnL + depósito - retiro.
        let naive_profit_e8 = profit_e8 + deposit_e8 - withdrawal_e8;
        let naive_gain_pct_e8 = ((i128::from(naive_profit_e8) * 100_000_000) / i128::from(capital_base_e8)) as i64;

        assert_ne!(
            metrics.gain_pct_e8, naive_gain_pct_e8,
            "la implementación correcta debe diferir de la suma ingenua que mezclaría capital con ganancia"
        );
        assert_eq!(metrics.gain_pct_e8, 441_000_000);
    }

    // ── Curvas, drawdown, win rate, holding time, trading days ──────────────

    #[test]
    fn compute_track_record_builds_chronological_curves_and_max_drawdown() {
        let account_id = "acc-curves";
        let events = vec![
            snapshot(account_id, 3_000, 900_000_000_000, 900_000_000_000),
            snapshot(account_id, 1_000, 1_000_000_000_000, 1_000_000_000_000),
            snapshot(account_id, 2_000, 1_200_000_000_000, 1_200_000_000_000),
        ];
        let metrics = compute_track_record(&events, account_id);

        // Las curvas deben quedar ORDENADAS por timestamp, sin importar el
        // orden de inserción en el Vec de entrada.
        assert_eq!(
            metrics.equity_curve,
            vec![(1_000, 1_000_000_000_000), (2_000, 1_200_000_000_000), (3_000, 900_000_000_000)]
        );

        // Pico = 1,200,000,000,000 (en t=2000); valle posterior = 900,000,000,000 (t=3000).
        // Drawdown = (1200 - 900) / 1200 = 0.25 -> 25_000_000 ×10⁸.
        assert_eq!(metrics.max_drawdown_e8, 25_000_000);
    }

    #[test]
    fn compute_track_record_computes_win_rate_holding_time_and_trading_days() {
        let account_id = "acc-stats";
        let events = vec![
            order(account_id, 100_000_000, 0, 1_000_000_000),
            order(account_id, -50_000_000, NS_PER_DAY, 3_000_000_000),
            order(account_id, 200_000_000, NS_PER_DAY, 2_000_000_000),
        ];
        let metrics = compute_track_record(&events, account_id);

        // 2 de 3 trades ganadores -> 66.666...% -> 66_666_666 (redondeo hacia abajo, entero).
        assert_eq!(metrics.win_rate_e8, 66_666_666);
        // Promedio de duraciones: (1_000_000_000 + 3_000_000_000 + 2_000_000_000) / 3 = 2_000_000_000.
        assert_eq!(metrics.avg_holding_time_ns, 2_000_000_000);
        // Dos días distintos: día 0 y día 1 (NS_PER_DAY).
        assert_eq!(metrics.trading_days, 2);
    }

    #[test]
    fn compute_track_record_ignores_events_from_other_accounts() {
        let events = vec![
            order("acc-a", 100_000_000, 0, 1_000_000_000),
            order("acc-b", 999_000_000_000, 0, 1_000_000_000),
            capital_flow("acc-b", CapitalFlowSign::Deposit, 1_000_000_000_000),
        ];
        let metrics = compute_track_record(&events, "acc-a");

        assert_eq!(metrics.total_realized_pnl_e8, 100_000_000, "solo debe contar el evento de acc-a");
        assert_eq!(metrics.total_deposits_e8, 0, "el depósito de otra cuenta no debe contarse");
    }

    // ── CRITERIO #6 (Orden §5): firma reproducible ──────────────────────────

    #[test]
    fn compute_track_record_signature_is_reproducible_for_the_same_content() {
        let account_id = "acc-sig";
        let events = vec![
            snapshot(account_id, 0, 1_000_000_000_000, 1_000_000_000_000),
            order(account_id, 100_000_000_000, NS_PER_DAY, 3_600_000_000_000),
        ];
        let metrics_a = compute_track_record(&events, account_id);
        let metrics_b = compute_track_record(&events, account_id);

        let sig_a = compute_track_record_signature(&metrics_a, AttestationScope::Sovereign, CapitalReality::Live, account_id, "2026-W27");
        let sig_b = compute_track_record_signature(&metrics_b, AttestationScope::Sovereign, CapitalReality::Live, account_id, "2026-W27");
        assert_eq!(sig_a, sig_b, "el mismo contenido debe producir la MISMA firma");
    }

    #[test]
    fn compute_track_record_signature_changes_when_a_metric_changes() {
        let account_id = "acc-sig-change";
        let events = vec![order(account_id, 100_000_000_000, NS_PER_DAY, 3_600_000_000_000)];
        let metrics = compute_track_record(&events, account_id);
        let original = compute_track_record_signature(&metrics, AttestationScope::Sovereign, CapitalReality::Live, account_id, "2026-W27");

        let mut changed_metrics = metrics.clone();
        changed_metrics.gain_pct_e8 += 1;
        let changed = compute_track_record_signature(&changed_metrics, AttestationScope::Sovereign, CapitalReality::Live, account_id, "2026-W27");

        assert_ne!(original, changed, "cambiar una métrica debe cambiar la firma");
    }

    /// CRITERIO DE CIERRE: la firma también distingue el ámbito -- el MISMO
    /// contenido de métricas, pero un `scope` distinto, produce una firma
    /// distinta (refuerza que `Sovereign`/`BrokerReadonly` nunca colisionan).
    #[test]
    fn compute_track_record_signature_differs_by_scope() {
        let account_id = "acc-scope-sig";
        let events = vec![order(account_id, 100_000_000_000, NS_PER_DAY, 3_600_000_000_000)];
        let metrics = compute_track_record(&events, account_id);

        let sovereign_sig = compute_track_record_signature(&metrics, AttestationScope::Sovereign, CapitalReality::Live, account_id, "2026-W27");
        let readonly_sig = compute_track_record_signature(&metrics, AttestationScope::BrokerReadonly, CapitalReality::Live, account_id, "2026-W27");

        assert_ne!(sovereign_sig, readonly_sig, "el ámbito debe formar parte de la firma");
    }

    /// CRITERIO #4 (Orden §4 punto 15) DE CIERRE: la firma también distingue
    /// el Eje B -- las MISMAS métricas y el MISMO `scope`, pero una
    /// `capital_reality` distinta (LIVE vs PAPER), produce una firma
    /// distinta. Sin esto, un track PAPER podría falsificarse como LIVE sin
    /// que la firma lo detectara.
    #[test]
    fn compute_track_record_signature_differs_by_capital_reality() {
        let account_id = "acc-capital-reality-sig";
        let events = vec![order(account_id, 100_000_000_000, NS_PER_DAY, 3_600_000_000_000)];
        let metrics = compute_track_record(&events, account_id);

        let live_sig = compute_track_record_signature(&metrics, AttestationScope::Sovereign, CapitalReality::Live, account_id, "2026-W27");
        let paper_sig = compute_track_record_signature(&metrics, AttestationScope::Sovereign, CapitalReality::Paper, account_id, "2026-W27");

        assert_ne!(live_sig, paper_sig, "la realidad de capital (Eje B) debe formar parte de la firma");
    }

    // ── audit_hash: determinismo + sensibilidad a cambios (fila del ledger) ─

    #[test]
    fn compute_track_record_audit_hash_is_deterministic() {
        // `institutional_tag` y `capital_reality` llevan el MISMO valor
        // (STORY-041): en esta tabla `institutional_tag` ES el Eje B.
        let hash_a = compute_track_record_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "LIVE", "node-1", "acc-1", "SOVEREIGN", "2026-W27", "sig-abc", "LIVE",
        );
        let hash_b = compute_track_record_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "LIVE", "node-1", "acc-1", "SOVEREIGN", "2026-W27", "sig-abc", "LIVE",
        );
        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn compute_track_record_audit_hash_changes_when_signature_hash_changes() {
        let base = compute_track_record_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "LIVE", "node-1", "acc-1", "SOVEREIGN", "2026-W27", "sig-aaa", "LIVE",
        );
        let changed = compute_track_record_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "LIVE", "node-1", "acc-1", "SOVEREIGN", "2026-W27", "sig-bbb", "LIVE",
        );
        assert_ne!(base, changed, "cambiar signature_hash debe cambiar audit_hash -- son campos distintos");
    }

    #[test]
    fn compute_track_record_audit_hash_changes_when_capital_reality_changes() {
        let live = compute_track_record_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "LIVE", "node-1", "acc-1", "SOVEREIGN", "2026-W27", "sig-abc", "LIVE",
        );
        let paper = compute_track_record_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "PAPER", "node-1", "acc-1", "SOVEREIGN", "2026-W27", "sig-abc", "PAPER",
        );
        assert_ne!(live, paper, "cambiar capital_reality (Eje B) debe cambiar el audit_hash");
    }

    #[test]
    fn compute_verified_account_audit_hash_is_deterministic_and_chains_by_row_version() {
        // `institutional_tag` y `capital_reality` llevan el MISMO valor
        // (STORY-041): en esta tabla `institutional_tag` ES el Eje B.
        let genesis = compute_verified_account_audit_hash(
            "id-1", 1_000, 1, None, "owner-1", "LIVE", "node-1", "ICMarkets", "USD",
            AccountType::Own, PublicationStatus::Private, "[]", None, CapitalReality::Live,
        );
        let genesis_again = compute_verified_account_audit_hash(
            "id-1", 1_000, 1, None, "owner-1", "LIVE", "node-1", "ICMarkets", "USD",
            AccountType::Own, PublicationStatus::Private, "[]", None, CapitalReality::Live,
        );
        assert_eq!(genesis, genesis_again, "misma versión, mismo contenido -> mismo hash");

        let updated = compute_verified_account_audit_hash(
            "id-1", 2_000, 2, Some(&genesis), "owner-1", "LIVE", "node-1", "ICMarkets", "USD",
            AccountType::Own, PublicationStatus::Public, "[]", None, CapitalReality::Live,
        );
        assert_ne!(updated, genesis, "cambiar publication_status debe cambiar el hash");

        let paper = compute_verified_account_audit_hash(
            "id-1", 1_000, 1, None, "owner-1", "PAPER", "node-1", "ICMarkets", "USD",
            AccountType::Own, PublicationStatus::Private, "[]", None, CapitalReality::Paper,
        );
        assert_ne!(paper, genesis, "cambiar capital_reality (Eje B) debe cambiar el hash");
    }

    // ── CRITERIO #5 (Orden §5): gate de publicación puro ────────────────────

    #[test]
    fn decide_publication_defaults_to_private_and_never_advances_without_covered_consent() {
        use crate::domain::consent_registry::NotCoveredReason;

        let not_covered = ConsentVerdict::NotCovered(NotCoveredReason::NoConsent);
        let result = decide_publication(PublicationStatus::Private, PublicationStatus::Public, &not_covered);
        assert_eq!(result, PublicationStatus::Private, "sin opt-in vigente, NUNCA debe avanzar a Public");
    }

    #[test]
    fn decide_publication_advances_to_public_only_with_covered_consent() {
        let covered = ConsentVerdict::Covered;
        let result = decide_publication(PublicationStatus::Private, PublicationStatus::Public, &covered);
        assert_eq!(result, PublicationStatus::Public, "con opt-in vigente real, debe publicar");
    }

    #[test]
    fn decide_publication_always_allows_reverting_to_private() {
        use crate::domain::consent_registry::NotCoveredReason;
        let not_covered = ConsentVerdict::NotCovered(NotCoveredReason::NoConsent);
        let result = decide_publication(PublicationStatus::Public, PublicationStatus::Private, &not_covered);
        assert_eq!(result, PublicationStatus::Private, "volver a privado nunca requiere consentimiento");
    }

    // ── CRITERIO #4 (Orden §5): sin secretos (ADR-0093), assert estructural ─

    #[test]
    fn verified_account_record_json_never_leaks_secret_looking_fields() {
        let record = VerifiedAccountRecord {
            id: "acc-1".to_string(),
            owner_id: "owner-1".to_string(),
            broker: "ICMarkets".to_string(),
            leverage: 100,
            currency: "USD".to_string(),
            account_type: "OWN".to_string(),
            publication_status: "PRIVATE".to_string(),
            attestation_scopes: vec!["SOVEREIGN".to_string()],
            capital_reality: "LIVE".to_string(),
            broker_connection_ref: Some("conn-ref-not-a-secret".to_string()),
        };
        let json = serde_json::to_string(&record).expect("serializar").to_lowercase();
        for forbidden in [
            "password", "api_key", "api-key", "broker_secret", "private_key",
            "signing_key", "investor_password", "192.168.", "10.0.0.",
        ] {
            assert!(!json.contains(forbidden), "VerifiedAccountRecord no debe contener '{forbidden}'");
        }
    }

    #[test]
    fn attested_track_record_json_never_leaks_secret_looking_fields() {
        let track = AttestedTrackRecord {
            id: "track-1".to_string(),
            verified_account_id: "acc-1".to_string(),
            scope: "SOVEREIGN".to_string(),
            capital_reality: "LIVE".to_string(),
            time_window: "2026-W27".to_string(),
            signature_hash: "abc123".to_string(),
            equity_curve: vec![(0, 1_000_000_000_000)],
            balance_curve: vec![(0, 1_000_000_000_000)],
            max_drawdown_e8: 0,
            gain_pct_e8: 441_000_000,
            win_rate_e8: 66_666_666,
            avg_holding_time_ns: 3_600_000_000_000,
            trading_days: 2,
            total_realized_pnl_e8: 4_410_000_000_000,
            total_deposits_e8: 35_000_000_000,
            total_withdrawals_e8: 47_698_000_000,
            is_attested_by_drasus: true,
            is_real_capital: true,
        };
        let json = serde_json::to_string(&track).expect("serializar").to_lowercase();
        for forbidden in [
            "password", "api_key", "api-key", "broker_secret", "private_key",
            "signing_key", "investor_password", "192.168.", "10.0.0.",
        ] {
            assert!(!json.contains(forbidden), "AttestedTrackRecord no debe contener '{forbidden}'");
        }
    }
}
