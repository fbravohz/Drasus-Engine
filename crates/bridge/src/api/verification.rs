//! Funciones FFI del "Banco de Pruebas" -- despacho GENÉRICO de verificación
//! de features hacia Flutter/Dart (Canal #2, ADR-0142).
//!
//! ## Qué resuelve este módulo
//!
//! Cada feature verificable ya expone su propia función `verify_<feature>`
//! en `shared::public_interface` (o, para el Sovereign Data Fetcher, en su
//! propio crate). Hasta ahora el ÚNICO cliente de esas funciones era el CLI
//! (`crates/app/src/main.rs`, subcomando `verify`), con un `match` de 15
//! brazos escrito a mano. El Banco de Pruebas necesita la MISMA capacidad
//! desde Flutter, pero con una sola función FFI genérica (`verify_feature`)
//! en vez de 15 bindings distintos -- así el selector de feature en la UI
//! no exige un binding nuevo cada vez que se añade una feature.
//!
//! ## Por qué la validación de input ocurre ANTES de tocar el backend
//!
//! El propietario pidió poder distinguir "mi JSON está mal" de "mi JSON
//! está bien pero la feature rechazó la operación". `serde_json` ya hace
//! las dos comprobaciones en un solo paso al deserializar contra el tipo
//! `Input` tipado de la feature: si el JSON no es JSON válido, o le falta
//! un campo obligatorio, o un campo tiene el tipo incorrecto, `from_str`
//! falla ANTES de que se ejecute una sola línea del backend. Por eso
//! `input_status = Invalid` nunca coexiste con una llamada real al backend
//! (ver `dispatch()` más abajo).
//!
//! ## Despacho genérico: `dispatch()`
//!
//! Los 15 backends comparten una forma: `async fn(Input) -> Output`, donde
//! `Output` SIEMPRE tiene los campos `ok: bool` y `error: Option<String>`
//! (contrato ADR-0142 "JSON estructurado en el CLI, FIJO" -- los 15
//! `pub struct *VerifyOutput` de origen lo cumplen). En vez de repetir el
//! `match` + serializar + comprobar `ok` 15 veces (como hace `main.rs`),
//! `dispatch()` es genérico sobre `Input`/`Output`/la función backend, y
//! extrae `ok` y `error` desde el `serde_json::Value` ya serializado -- sin
//! necesitar un trait nuevo para cada uno de los 15 tipos `Output`.

use std::future::Future;

use serde::Serialize;
use serde::de::DeserializeOwned;

use shared::public_interface::{
    CentralIdentityVerifyInput, ConsentRegistryVerifyInput, DataAggregationVerifyInput,
    DataPortabilityVerifyInput, EnrichedDomainEventsVerifyInput, InstanceContinuityVerifyInput,
    InstitutionalReportEngineVerifyInput, LicensingSystemVerifyInput,
    MasterAccountHierarchyVerifyInput, OperatorRolesVerifyInput, PlanTierQuotaVerifyInput,
    ThirdPartyApiGatewayVerifyInput, UsageMeteringVerifyInput, VerifiedAccountRegistryVerifyInput,
    verify_central_identity, verify_consent_registry, verify_data_aggregation,
    verify_data_portability, verify_enriched_domain_events, verify_institutional_report_engine,
    verify_instance_continuity, verify_licensing_system, verify_master_account_hierarchy,
    verify_operator_roles, verify_plan_tier_quota, verify_third_party_api_gateway,
    verify_usage_metering, verify_verified_account_registry,
};
use sovereign_data_fetcher::public_interface::{
    VerifyInput as SovereignDataFetcherVerifyInput, verify as verify_sovereign_data_fetcher,
};

// ── Tipos FFI-safe ────────────────────────────────────────────────────────

/// Resultado de validar `input_json` contra el contrato tipado de la
/// feature, ANTES de ejecutar el backend.
///
/// | Variante | Rust | Dart | Cuándo ocurre |
/// |---|---|---|---|
/// | `Valid` | unidad | `InputStatus.valid()` | El JSON deserializó correctamente contra el tipo esperado. |
/// | `Invalid { reason }` | `String` | `InputStatus.invalid(reason: String)` | JSON malformado, campo obligatorio ausente/con tipo incorrecto, o `feature_id` fuera del catálogo. |
pub enum InputStatus {
    /// El `input_json` es sintácticamente válido y su estructura coincide
    /// exactamente con lo que la feature `feature_id` espera.
    Valid,
    /// El `input_json` no es JSON válido, o es JSON válido que no coincide
    /// con la estructura esperada (campo obligatorio ausente, tipo
    /// incorrecto), o `feature_id` no está en el catálogo. `reason` es el
    /// mensaje de `serde_json` (incluye línea/columna cuando aplica) o,
    /// para `feature_id` desconocido, la indicación de consultar el
    /// catálogo vigente.
    Invalid { reason: String },
}

/// Resultado estructurado de una verificación despachada de forma genérica
/// por [`verify_feature`]. Es la ÚNICA forma de respuesta del Banco de
/// Pruebas -- el mismo shape para las 15 features, sin importar cuál se
/// invoque.
///
/// | Campo | Rust | Dart | Nullable en Dart |
/// |---|---|---|---|
/// | `input_status` | `InputStatus` | `InputStatus` | No |
/// | `ok` | `bool` | `bool` | No |
/// | `output_json` | `String` | `String` | No |
/// | `error` | `Option<String>` | `String?` | Sí |
///
/// Propietario de memoria: Rust crea esta estructura; flutter_rust_bridge la
/// serializa y la libera. Dart recibe su propia copia -- sin memoria
/// compartida entre los dos lados.
pub struct VerificationOutcome {
    /// Si el input fue rechazado ANTES de llamar al backend, esta variante
    /// trae el motivo. Cuando es `Invalid`, `ok` siempre es `false`,
    /// `output_json` siempre es `""` y `error` siempre es `None` -- el
    /// problema ya quedó descrito aquí, no hace falta duplicarlo.
    pub input_status: InputStatus,
    /// Copia de `output.ok` del backend cuando `input_status == Valid`.
    /// Cuando `input_status == Invalid`, vale `false` (nunca se llegó a
    /// ejecutar el backend).
    pub ok: bool,
    /// El `Output` completo del backend, serializado con
    /// `serde_json::to_string`. Cadena vacía si `input_status == Invalid`.
    /// El propietario puede inspeccionar cualquier campo del backend real
    /// (no solo `ok`/`error`) sin que el Bridge declare un DTO nuevo por
    /// cada una de las 15 features.
    pub output_json: String,
    /// Mensaje de error de negocio cuando `ok == false` y
    /// `input_status == Valid` (el backend corrió pero rechazó la
    /// operación). Viene del campo `error` que TODO `*VerifyOutput` expone
    /// (contrato ADR-0142). `None` si `ok == true` o si
    /// `input_status == Invalid`.
    pub error: Option<String>,
}

impl VerificationOutcome {
    /// Construye el resultado para un `input_json` (o un `feature_id`) que
    /// no pasó la validación -- nunca se llega a llamar al backend.
    fn invalid(reason: String) -> Self {
        Self {
            input_status: InputStatus::Invalid { reason },
            ok: false,
            output_json: String::new(),
            error: None,
        }
    }
}

/// Un elemento del catálogo de features verificables -- usado por la UI del
/// Banco de Pruebas para poblar el selector y precargar un input de ejemplo.
///
/// | Campo | Rust | Dart | Nullable en Dart |
/// |---|---|---|---|
/// | `id` | `String` | `String` | No |
/// | `display_name` | `String` | `String` | No |
/// | `example_input_json` | `String` | `String` | No |
pub struct FeatureDescriptor {
    /// Identificador en kebab-case: el mismo que espera [`verify_feature`]
    /// y el subcomando `drasus verify <feature_id>` del CLI (ADR-0142).
    pub id: String,
    /// Nombre legible para el selector de la UI (nunca el `id` crudo).
    pub display_name: String,
    /// JSON de ejemplo que deserializa correctamente contra el `Input`
    /// tipado de la feature -- listo para precargar el campo de texto del
    /// Banco de Pruebas. Reutiliza los ejemplos ya documentados en
    /// `crates/app/src/main.rs` (subcomando `verify`) -- una sola fuente de
    /// verdad para "cómo se ve un input válido de humo".
    pub example_input_json: String,
}

// ── Despacho genérico ────────────────────────────────────────────────────

/// Deserializa `input_json` contra `Input`, ejecuta `backend` y empaqueta el
/// resultado. Genérico sobre las 15 features: todas comparten la forma
/// `async fn(Input) -> Output` con `Output` conteniendo `ok: bool` y
/// `error: Option<String>` (ver comentario de módulo). `backend` NUNCA se
/// invoca si la deserialización falla -- ese es el requisito de "validar
/// antes de ejecutar".
async fn dispatch<Input, Output, Backend, Fut>(
    input_json: &str,
    backend: Backend,
) -> VerificationOutcome
where
    Input: DeserializeOwned,
    Output: Serialize,
    Backend: FnOnce(Input) -> Fut,
    Fut: Future<Output = Output>,
{
    // Paso 1: valida forma ANTES de tocar el backend. `serde_json::from_str`
    // contra el tipo tipado de la feature cubre a la vez "¿es JSON?" y
    // "¿tiene la forma correcta?" en un solo fallo con un solo mensaje.
    let input: Input = match serde_json::from_str(input_json) {
        Ok(v) => v,
        Err(e) => return VerificationOutcome::invalid(e.to_string()),
    };

    // Paso 2: ejecuta el round-trip real contra el backend (BD SQLite
    // efímera propia + migraciones + feature real -- ver doc de cada
    // `verify_*` en `shared::public_interface`).
    let output = backend(input).await;

    // Extrae `ok` y `error` sin necesitar un trait nuevo por cada uno de
    // los 15 tipos `Output` -- todos siguen el mismo contrato ADR-0142.
    let value = serde_json::to_value(&output)
        // Los 15 `Output` son structs planos serializables (mismo supuesto
        // que usa `main.rs` con `to_string_pretty`); esto no falla en la
        // práctica.
        .expect("VerifyOutput siempre es serializable a JSON");
    let ok = value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    let error = value
        .get("error")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let output_json =
        serde_json::to_string(&value).expect("serde_json::Value ya validado siempre serializa");

    VerificationOutcome {
        input_status: InputStatus::Valid,
        ok,
        output_json,
        error,
    }
}

// ── Funciones FFI ────────────────────────────────────────────────────────

/// Despacha la verificación de `feature_id` con `input_json` hacia el
/// backend real y devuelve un resultado estructurado.
///
/// ## Contrato (FIJO -- el Flutter-Engineer construye la UI del Banco de
/// Pruebas contra esta firma exacta)
/// 1. Si `input_json` no es JSON válido, o no coincide con la estructura
///    que `feature_id` espera, o `feature_id` no está en el catálogo de
///    [`list_verifiable_features`]: devuelve `input_status = Invalid` SIN
///    ejecutar el backend.
/// 2. Si el input es válido: ejecuta la `verify_*` real de la feature
///    (round-trip completo -- BD efímera propia, migraciones, feature
///    ejercitada de punta a punta) y devuelve `ok` + `output_json` con la
///    respuesta completa del backend serializada, o `error` con el motivo
///    si el backend rechazó la operación.
///
/// ## Ownership al cruzar la frontera FFI
/// `VerificationOutcome` se serializa por flutter_rust_bridge en un objeto
/// Dart. Rust libera la memoria; Dart recibe su propia copia -- sin memoria
/// compartida entre los dos lados.
///
/// ## Error handling
/// Nunca hace panic ante input malformado o `feature_id` desconocido -- eso
/// se captura como `input_status = Invalid`. Los fallos del backend (BD
/// inaccesible, validación de dominio rechazada, etc.) ya vienen
/// capturados dentro de cada `Output.error` (contrato ADR-0142); `dispatch`
/// solo los promueve al nivel superior de `VerificationOutcome.error`.
pub async fn verify_feature(feature_id: String, input_json: String) -> VerificationOutcome {
    match feature_id.as_str() {
        "sovereign-data-fetcher" => {
            dispatch::<SovereignDataFetcherVerifyInput, _, _, _>(
                &input_json,
                verify_sovereign_data_fetcher,
            )
            .await
        }
        "central-identity" => {
            dispatch::<CentralIdentityVerifyInput, _, _, _>(&input_json, verify_central_identity)
                .await
        }
        "licensing-system" => {
            dispatch::<LicensingSystemVerifyInput, _, _, _>(&input_json, verify_licensing_system)
                .await
        }
        "plan-tier-quota" => {
            dispatch::<PlanTierQuotaVerifyInput, _, _, _>(&input_json, verify_plan_tier_quota)
                .await
        }
        "usage-metering" => {
            dispatch::<UsageMeteringVerifyInput, _, _, _>(&input_json, verify_usage_metering)
                .await
        }
        "consent-registry" => {
            dispatch::<ConsentRegistryVerifyInput, _, _, _>(&input_json, verify_consent_registry)
                .await
        }
        "enriched-domain-events" => {
            dispatch::<EnrichedDomainEventsVerifyInput, _, _, _>(
                &input_json,
                verify_enriched_domain_events,
            )
            .await
        }
        "institutional-report-engine" => {
            dispatch::<InstitutionalReportEngineVerifyInput, _, _, _>(
                &input_json,
                verify_institutional_report_engine,
            )
            .await
        }
        "third-party-api-gateway" => {
            dispatch::<ThirdPartyApiGatewayVerifyInput, _, _, _>(
                &input_json,
                verify_third_party_api_gateway,
            )
            .await
        }
        "data-aggregation" => {
            dispatch::<DataAggregationVerifyInput, _, _, _>(&input_json, verify_data_aggregation)
                .await
        }
        "verified-account-registry" => {
            dispatch::<VerifiedAccountRegistryVerifyInput, _, _, _>(
                &input_json,
                verify_verified_account_registry,
            )
            .await
        }
        "instance-continuity" => {
            dispatch::<InstanceContinuityVerifyInput, _, _, _>(
                &input_json,
                verify_instance_continuity,
            )
            .await
        }
        "master-account-hierarchy" => {
            dispatch::<MasterAccountHierarchyVerifyInput, _, _, _>(
                &input_json,
                verify_master_account_hierarchy,
            )
            .await
        }
        "data-portability" => {
            dispatch::<DataPortabilityVerifyInput, _, _, _>(&input_json, verify_data_portability)
                .await
        }
        "operator-roles" => {
            dispatch::<OperatorRolesVerifyInput, _, _, _>(&input_json, verify_operator_roles)
                .await
        }
        unknown => VerificationOutcome::invalid(format!(
            "feature_id no reconocido: '{unknown}'. Usa list_verifiable_features() para ver el catálogo vigente."
        )),
    }
}

/// Devuelve el catálogo completo de features verificables, en el mismo
/// orden en que aparecen los brazos de [`verify_feature`]. La UI del Banco
/// de Pruebas usa este catálogo para poblar el selector y el input por
/// defecto -- añadir una feature nueva solo exige un elemento más aquí y un
/// brazo más en `verify_feature`, sin tocar ningún otro binding FFI.
///
/// ## Ownership al cruzar la frontera FFI
/// `Vec<FeatureDescriptor>` se serializa en una `List<FeatureDescriptor>`
/// Dart. Rust libera el vector; Dart recibe copias de los datos. Sin
/// memoria compartida.
#[flutter_rust_bridge::frb(sync)]
pub fn list_verifiable_features() -> Vec<FeatureDescriptor> {
    vec![
        FeatureDescriptor {
            id: "sovereign-data-fetcher".to_string(),
            display_name: "Sovereign Data Fetcher".to_string(),
            example_input_json: r#"{"symbol":"BTCUSDT","interval":"1h"}"#.to_string(),
        },
        FeatureDescriptor {
            id: "central-identity".to_string(),
            display_name: "Identidad Central".to_string(),
            example_input_json: r#"{"email":"a@b.com"}"#.to_string(),
        },
        FeatureDescriptor {
            id: "licensing-system".to_string(),
            display_name: "Sistema de Licenciamiento".to_string(),
            example_input_json: r#"{"tier":"SOVEREIGN"}"#.to_string(),
        },
        FeatureDescriptor {
            id: "plan-tier-quota".to_string(),
            display_name: "Cuota por Plan/Tier".to_string(),
            example_input_json: r#"{"tier":"FREE"}"#.to_string(),
        },
        FeatureDescriptor {
            id: "usage-metering".to_string(),
            display_name: "Medición de Uso".to_string(),
            example_input_json:
                r#"{"tier":"FREE","operations":[{"size":250000000,"price":4000000000000}]}"#
                    .to_string(),
        },
        FeatureDescriptor {
            id: "consent-registry".to_string(),
            display_name: "Registro de Consentimiento".to_string(),
            example_input_json: r#"{"current_version":"v2","actions":[{"action":"ACCEPT","tos_version":"v2","optout_map":{"aggregation":false}}],"query":{"data_type":"aggregation"}}"#.to_string(),
        },
        FeatureDescriptor {
            id: "enriched-domain-events".to_string(),
            display_name: "Eventos de Dominio Enriquecidos".to_string(),
            example_input_json: r#"{"tier":"FREE","event":{"type":"CapitalFlow","account_id":"acc-1","sign":"DEPOSIT","amount":100000000000,"currency":"USD"}}"#.to_string(),
        },
        FeatureDescriptor {
            id: "institutional-report-engine".to_string(),
            display_name: "Motor de Reportes Institucionales".to_string(),
            example_input_json: r#"{"report_type":"VALIDATION","metrics":{"sharpe_e8":150000000,"max_drawdown_e8":-8000000},"source_event_refs":["evt-1","evt-2"]}"#.to_string(),
        },
        FeatureDescriptor {
            id: "third-party-api-gateway".to_string(),
            display_name: "Pasarela de API de Terceros".to_string(),
            example_input_json: r#"{"credential":"sk-demo-123","endpoint":"CERTIFY","rate_limit_per_window":100,"requests_in_window":100}"#.to_string(),
        },
        FeatureDescriptor {
            id: "data-aggregation".to_string(),
            display_name: "Agregación de Datos".to_string(),
            example_input_json: r#"{"seed":42,"min_cohort":5,"external_sale_enabled":false,"events":[{"metric_e8":150000000,"consent":"COVERED"}]}"#.to_string(),
        },
        FeatureDescriptor {
            id: "verified-account-registry".to_string(),
            display_name: "Registro de Cuentas Verificadas".to_string(),
            example_input_json: r#"{"account":{"broker":"ICMarkets","currency":"USD","account_type":"OWN"},"consent":"COVERED","events":[{"type":"CapitalFlow","sign":"DEPOSIT","amount_e8":35000000000},{"type":"OrderExecuted","pnl_e8":15000000000}]}"#.to_string(),
        },
        FeatureDescriptor {
            id: "instance-continuity".to_string(),
            display_name: "Continuidad de Instancia".to_string(),
            example_input_json: r#"{"master_secret":"correct horse battery staple","plaintext":"snapshot-bytes","nonce_seed":42,"custody":{"titular_node_id":"node-A","custody_epoch":3},"my_node_id":"node-A"}"#.to_string(),
        },
        FeatureDescriptor {
            id: "master-account-hierarchy".to_string(),
            display_name: "Jerarquía de Cuentas Maestras".to_string(),
            example_input_json: r#"{"parent_owner_id":"fund-X","child_owner_id":"trader-7","node_id":"node-A","consent":"COVERED","command_kind":"ARCHIVE","target_ref":"strategy-42","justification":"riesgo excedido"}"#.to_string(),
        },
        FeatureDescriptor {
            id: "data-portability".to_string(),
            display_name: "Portabilidad de Datos".to_string(),
            example_input_json: r#"{"owner_id":"user-42","institutional_tag":"LIVE","node_id":"node-A","request_type":"FORGET"}"#.to_string(),
        },
        FeatureDescriptor {
            id: "operator-roles".to_string(),
            display_name: "Roles de Operador".to_string(),
            example_input_json: r#"{"owner_id":"acc-1","institutional_tag":"LIVE","node_id":"node-A","access_token_id":"tok-owner","capability_key":"generate.run_search","pipeline":"GENERATE"}"#.to_string(),
        },
    ]
}

// ── Pruebas ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Los 15 ids del catálogo, en el mismo orden que los brazos de
    /// `verify_feature`. Fuente única para las pruebas de este módulo --
    /// si alguien añade una feature al catálogo sin añadir el brazo (o
    /// viceversa), esta prueba lo detecta.
    const EXPECTED_FEATURE_IDS: [&str; 15] = [
        "sovereign-data-fetcher",
        "central-identity",
        "licensing-system",
        "plan-tier-quota",
        "usage-metering",
        "consent-registry",
        "enriched-domain-events",
        "institutional-report-engine",
        "third-party-api-gateway",
        "data-aggregation",
        "verified-account-registry",
        "instance-continuity",
        "master-account-hierarchy",
        "data-portability",
        "operator-roles",
    ];

    /// El catálogo debe traer exactamente los 15 ids esperados, en el mismo
    /// orden en que aparecen en `verify_feature`. Prueba discriminante: si
    /// se borra o se duplica un `FeatureDescriptor`, o se cambia un `id` sin
    /// avisar, esta prueba falla.
    #[test]
    fn catalog_has_exactly_the_fifteen_expected_feature_ids() {
        let ids: Vec<String> = list_verifiable_features()
            .into_iter()
            .map(|f| f.id)
            .collect();
        assert_eq!(ids, EXPECTED_FEATURE_IDS.to_vec());
    }

    /// Cada `example_input_json` del catálogo debe ser al menos JSON
    /// sintácticamente válido -- si alguien rompe una comilla o una coma al
    /// editar un ejemplo, esta prueba lo detecta sin necesitar ejecutar
    /// ningún backend (ni red, ni BD).
    #[test]
    fn catalog_examples_are_syntactically_valid_json() {
        for feature in list_verifiable_features() {
            let parsed: Result<serde_json::Value, _> =
                serde_json::from_str(&feature.example_input_json);
            assert!(
                parsed.is_ok(),
                "el ejemplo de '{}' no es JSON válido: {:?}",
                feature.id,
                parsed.err()
            );
        }
    }

    /// JSON sintácticamente inválido debe rechazarse ANTES de tocar el
    /// backend: `input_status = Invalid`, `ok = false`, `output_json`
    /// vacío. Prueba discriminante de la Regla 1 del contrato.
    #[tokio::test]
    async fn malformed_json_is_rejected_without_calling_backend() {
        let outcome = verify_feature("central-identity".to_string(), "{esto no es json".to_string()).await;

        match outcome.input_status {
            InputStatus::Invalid { reason } => {
                assert!(!reason.is_empty(), "el motivo de rechazo no debe venir vacío");
            }
            InputStatus::Valid => panic!("JSON malformado no debe pasar la validación"),
        }
        assert!(!outcome.ok);
        assert_eq!(outcome.output_json, "");
    }

    /// JSON sintácticamente válido pero con la forma equivocada (le falta
    /// el único campo obligatorio, `email`, de `central-identity`) también
    /// debe rechazarse ANTES de tocar el backend. Distingue "JSON inválido"
    /// de "JSON válido con forma incorrecta" -- ambos caen en
    /// `input_status = Invalid`, pero por motivos de deserialización
    /// distintos.
    #[tokio::test]
    async fn well_formed_json_with_wrong_shape_is_rejected_without_calling_backend() {
        // "{}" es JSON válido, pero CentralIdentityVerifyInput exige "email"
        // sin valor por defecto (ver public_interface.rs).
        let outcome = verify_feature("central-identity".to_string(), "{}".to_string()).await;

        match outcome.input_status {
            InputStatus::Invalid { reason } => {
                assert!(
                    reason.contains("email"),
                    "el motivo debe señalar el campo faltante 'email', llegó: {reason}"
                );
            }
            InputStatus::Valid => panic!("input sin 'email' no debe pasar la validación"),
        }
        assert!(!outcome.ok);
        assert_eq!(outcome.output_json, "");
    }

    /// Un `feature_id` fuera del catálogo se trata como input inválido
    /// (nunca se llega a ejecutar ningún backend porque no hay backend al
    /// que despachar).
    #[tokio::test]
    async fn unknown_feature_id_is_rejected_without_calling_backend() {
        let outcome = verify_feature("feature-que-no-existe".to_string(), "{}".to_string()).await;

        match outcome.input_status {
            InputStatus::Invalid { reason } => {
                assert!(reason.contains("feature-que-no-existe"));
            }
            InputStatus::Valid => panic!("un feature_id desconocido no debe pasar la validación"),
        }
        assert!(!outcome.ok);
    }

    /// Round-trip real de extremo a extremo con un input válido: input
    /// estructuralmente correcto (`licensing-system` con `{}`, todos sus
    /// campos tienen default -- ver `LicensingSystemVerifyInput`) debe
    /// ejecutar el backend de verdad (BD SQLite efímera + migraciones +
    /// feature real, sin red) y devolver `input_status = Valid` con un
    /// `output_json` no vacío que trae el veredicto real de la feature.
    #[tokio::test]
    async fn valid_input_executes_backend_and_returns_its_real_output() {
        let outcome = verify_feature("licensing-system".to_string(), "{}".to_string()).await;

        match outcome.input_status {
            InputStatus::Valid => {}
            InputStatus::Invalid { reason } => {
                panic!("un input con todos los campos por defecto debe ser válido, llegó: {reason}")
            }
        }
        // El round-trip real corrió: el backend devuelve `ok = true` con
        // tier SOVEREIGN por defecto y adaptadores stub locales (sin red).
        assert!(outcome.ok, "licensing-system con defaults debe verificar en verde");
        assert!(!outcome.output_json.is_empty());
        assert!(outcome.output_json.contains("\"ok\":true"));
        assert!(outcome.error.is_none());
    }
}
