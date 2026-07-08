//! [CORE] Lógica pura de Data Portability (`docs/features/data-portability.md`,
//! ADR-0148 -- cimiento #13, ADR-0141, ADR-0093, ADR-0020, ADR-0002,
//! ADR-0137, STORY-043).
//!
//! Sin I/O, sin reloj de sistema, sin aleatoriedad sin semilla
//! (ADR-0002/0004). Infraestructura de cumplimiento transversal (mismo
//! nivel que `consent_registry`, #5), NO dominio de trading: da a un
//! `owner_id` autenticado dos derechos GDPR -- exportar sus datos (Art.
//! 15/20) y pedir el olvido (Art. 17), con excepciones de retención legal.
//!
//! Cinco piezas de lógica pura:
//! - [`RequestType`] / [`RequestStatus`]: el vocabulario cerrado que acepta
//!   el `CHECK` de la migración para `data_portability_requests`.
//! - [`decide_forget_disposition`]: la ÚNICA puerta que decide qué le pasa a
//!   una tabla del catálogo cuando alguien pide el olvido -- SIEMPRE
//!   pseudonimización, NUNCA un DELETE físico (ADR-0141). El catálogo de
//!   salida ([`ForgetDisposition`]) es estructuralmente incapaz de expresar
//!   un borrado: solo tiene dos variantes, ambas transiciones de estado.
//! - [`is_excluded_from_export`]: el filtro de exclusión de secretos
//!   (ADR-0093) -- mismo espíritu que
//!   [`crate::domain::instance_continuity::compute_backup_delta`], aplicado
//!   a nombres de tabla/columna del catálogo en vez de a campos de un
//!   snapshot. Corre ANTES de que cualquier tabla entre al manifiesto.
//! - [`build_export_manifest`]: resuelve, de forma determinista (ordenada
//!   por `table_name`), qué tablas del catálogo declarado aplican a un
//!   `owner_id` -- es el manifiesto (la ESTRUCTURA de qué se exportaría),
//!   nunca el dato real (eso lo trae el adaptador diferido que recorre el
//!   esquema real).
//! - [`build_forget_disposition_detail`]: por cada tabla del catálogo,
//!   aplica [`decide_forget_disposition`] y arma el detalle auditable de la
//!   decisión (qué se pseudonimiza-y-retiene vs. pseudonimiza-y-purga).
//! - [`compute_request_audit_hash`] / [`compute_catalog_audit_hash`]: los
//!   hashes de auditoría de ambas tablas -- encadenado por
//!   `event_sequence_id` (ledger APPEND-ONLY de solicitudes) y por
//!   `row_version` (catálogo MUTABLE), mismo estilo que el resto del
//!   substrato.

use serde::Serialize;
use sha2::{Digest, Sha256};

/// Codifica bytes crudos a su representación hexadecimal en minúsculas
/// (mismo patrón que el resto del substrato:
/// `master_account_hierarchy::encode_hex` / `instance_continuity::encode_hex`).
fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ── Vocabulario de `data_portability_requests` (columnas `request_type`/`status`) ──

/// El tipo de solicitud GDPR -- catálogo CERRADO de dos valores, el que
/// acepta el `CHECK (request_type IN ('EXPORT','FORGET'))` de la migración.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestType {
    /// Art. 15/20 GDPR -- el titular pide una copia legible de sus datos.
    Export,
    /// Art. 17 GDPR -- el titular pide el olvido (con excepciones de
    /// retención legal, ver [`ForgetDisposition`]).
    Forget,
}

impl RequestType {
    /// Representación canónica en texto -- la que acepta el `CHECK` de la
    /// migración.
    pub fn as_str(&self) -> &'static str {
        match self {
            RequestType::Export => "EXPORT",
            RequestType::Forget => "FORGET",
        }
    }

    /// Reconstruye el tipo desde su representación en texto, o `None` si no
    /// es ninguno de los dos reconocidos.
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "EXPORT" => Some(RequestType::Export),
            "FORGET" => Some(RequestType::Forget),
            _ => None,
        }
    }
}

/// El estado de UN evento de la solicitud -- catálogo CERRADO de tres
/// valores, el que acepta el `CHECK (status IN (...))` de la migración. El
/// estado VIGENTE de una solicitud lógica es el del evento con
/// `event_sequence_id` más alto para su `request_group_id` (ver
/// `persistence::data_portability::DataPortabilityRequestRepository::latest_status_for`)
/// -- nunca se corrige el estado de un evento anterior in-place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestStatus {
    Received,
    Processing,
    Completed,
}

impl RequestStatus {
    /// Representación canónica en texto -- la que acepta el `CHECK` de la
    /// migración.
    pub fn as_str(&self) -> &'static str {
        match self {
            RequestStatus::Received => "RECEIVED",
            RequestStatus::Processing => "PROCESSING",
            RequestStatus::Completed => "COMPLETED",
        }
    }

    /// Reconstruye el estado desde su representación en texto, o `None` si
    /// no es ninguno de los tres reconocidos.
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "RECEIVED" => Some(RequestStatus::Received),
            "PROCESSING" => Some(RequestStatus::Processing),
            "COMPLETED" => Some(RequestStatus::Completed),
            _ => None,
        }
    }
}

// ── "Olvido" = pseudonimización, NUNCA DELETE (regla fija #3, ADR-0141) ────

/// El efecto de aplicar un olvido (Art. 17) sobre UNA tabla del catálogo --
/// deliberadamente un catálogo CERRADO de dos valores, ninguno de los
/// cuales es un borrado físico de fila. En AMBOS casos el `owner_id` de la
/// fila se desvincula (pseudonimización); lo único que cambia es si el
/// CONTENIDO no-esencial sobrevive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ForgetDisposition {
    /// La tabla tiene una obligación de retención legal
    /// (`retention_exempt = true`, ej. un ledger de auditoría): el
    /// `owner_id` se desvincula, pero la fila y su contenido se CONSERVAN
    /// -- la integridad del ledger es la obligación que prevalece.
    PseudonymizeAndRetain,
    /// La tabla NO tiene retención legal (`retention_exempt = false`): el
    /// `owner_id` se desvincula y el contenido no-esencial PUEDE purgarse.
    /// Sigue sin ser un DELETE de la fila -- el recorrido/purga real del
    /// dato es responsabilidad del adaptador diferido que ejecuta esta
    /// decisión.
    PseudonymizeAndPurge,
}

impl ForgetDisposition {
    /// Representación canónica en texto -- usada al serializar
    /// `disposition_detail` (columna `TEXT` con `CHECK (json_valid(...))`).
    pub fn as_str(&self) -> &'static str {
        match self {
            ForgetDisposition::PseudonymizeAndRetain => "PSEUDONYMIZE_AND_RETAIN",
            ForgetDisposition::PseudonymizeAndPurge => "PSEUDONYMIZE_AND_PURGE",
        }
    }
}

/// Decide el efecto de un olvido sobre UNA tabla del catálogo, dado si esa
/// tabla porta una obligación de retención legal (`retention_exempt`).
///
/// Pura y determinista (ADR-0002): mismo input, mismo output. Regla fija #3
/// (ADR-0148/ADR-0141): estructuralmente IMPOSIBLE que devuelva algo
/// distinto de las dos variantes de [`ForgetDisposition`] -- ninguna de las
/// cuales es un DELETE físico. Retención legal -> se retiene el contenido;
/// sin retención -> el contenido no-esencial puede purgarse, pero la fila
/// en sí nunca desaparece del ledger.
pub fn decide_forget_disposition(retention_exempt: bool) -> ForgetDisposition {
    if retention_exempt {
        ForgetDisposition::PseudonymizeAndRetain
    } else {
        ForgetDisposition::PseudonymizeAndPurge
    }
}

// ── Filtro de exclusión de secretos (regla fija #1, ADR-0093) ──────────────

/// Subcadenas que marcan una tabla o columna del catálogo como portadora de
/// un secreto de bróker, una clave de cifrado o una IP de servidor live --
/// mismo espíritu que `EXCLUDED_BACKUP_KEY_SUBSTRINGS` de
/// `instance_continuity` (#11), con un conjunto propio documentado (STORY-043
/// §4 punto 4: "aceptable un conjunto propio si no se puede referenciar la
/// lista privada"). La comparación es *case-insensitive*.
const EXCLUDED_EXPORT_KEY_SUBSTRINGS: [&str; 9] = [
    "credential",
    "broker_password",
    "investor_password",
    "broker_api_key",
    "api_key",
    "encryption_key",
    "private_key",
    "live_server_ip",
    "live_ip",
];

/// Decide si un nombre de tabla o columna del catálogo pertenece a una de
/// las clases de secreto que ADR-0093 excluye SIEMPRE del export -- ninguna
/// credencial de bróker, clave de cifrado ni IP de servidor live puede
/// llegar al manifiesto de exportación, sin excepción.
pub fn is_excluded_from_export(column_or_table: &str) -> bool {
    let lower = column_or_table.to_lowercase();
    EXCLUDED_EXPORT_KEY_SUBSTRINGS.iter().any(|pattern| lower.contains(pattern))
}

// ── Manifiesto de exportación (Art. 15/20) ──────────────────────────────────

/// Una tabla candidata del catálogo declarativo -- proyección liviana de
/// `persistence::data_portability::ExportableDataCatalogRow` que el Core
/// consume sin conocer el esquema SQL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogEntry {
    pub table_name: String,
    pub feature_name: String,
    pub owner_id_column: String,
    pub retention_exempt: bool,
}

/// UNA tabla incluida en el manifiesto de exportación -- la ESTRUCTURA (qué
/// tabla, qué columna identifica al dueño), nunca el dato real de esa
/// tabla.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestTableEntry {
    pub table_name: String,
    pub feature_name: String,
    pub owner_id_column: String,
}

/// El manifiesto de exportación completo para un `owner_id` -- el tipo de
/// puerto `export_manifest_out` (ADR-0137, catálogo, cimiento #13). NUNCA
/// contiene el dato real de ninguna tabla (eso lo trae el adaptador
/// diferido que recorre el esquema); solo la lista ordenada de qué se
/// exportaría.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExportManifest {
    pub owner_id: String,
    pub tables: Vec<ManifestTableEntry>,
}

/// Resuelve, de forma DETERMINISTA (ordenada por `table_name`), qué tablas
/// del catálogo declarado aplican al `owner_id` dado -- excluyendo SIEMPRE
/// las que [`is_excluded_from_export`] marca como secreto (regla fija #1,
/// ADR-0093). Como el catálogo es metadato de ESQUEMA (no está ligado a un
/// dueño concreto), "aplican a `owner_id`" significa: toda tabla declarada
/// que no esté excluida por secreto -- el filtro por dueño real del dato
/// (qué FILAS de esa tabla son de este `owner_id`) lo aplica el adaptador
/// diferido que recorre el esquema, no este manifiesto.
pub fn build_export_manifest(owner_id: &str, catalog_entries: &[CatalogEntry]) -> ExportManifest {
    let mut tables: Vec<ManifestTableEntry> = catalog_entries
        .iter()
        .filter(|entry| !is_excluded_from_export(&entry.table_name) && !is_excluded_from_export(&entry.owner_id_column))
        .map(|entry| ManifestTableEntry {
            table_name: entry.table_name.clone(),
            feature_name: entry.feature_name.clone(),
            owner_id_column: entry.owner_id_column.clone(),
        })
        .collect();

    // Orden determinista por table_name -- dos llamadas con el mismo
    // catálogo, en cualquier orden de entrada, producen el MISMO manifiesto.
    tables.sort_by(|a, b| a.table_name.cmp(&b.table_name));

    ExportManifest { owner_id: owner_id.to_string(), tables }
}

// ── Detalle auditable del olvido (Art. 17) ──────────────────────────────────

/// La decisión de disposición para UNA tabla del catálogo, ya resuelta --
/// lo que compone `disposition_detail` (columna JSON de
/// `data_portability_requests`) para una solicitud FORGET.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TableDispositionEntry {
    pub table_name: String,
    pub feature_name: String,
    pub disposition: ForgetDisposition,
}

/// Aplica [`decide_forget_disposition`] a CADA tabla del catálogo declarado
/// y arma el detalle auditable completo -- determinista (ordenado por
/// `table_name`), sin excluir ninguna tabla (a diferencia del export, el
/// olvido SÍ debe decidir la disposición de las tablas de secretos también:
/// su `owner_id` se desvincula igual que cualquier otra, aunque su
/// contenido nunca se exporte).
pub fn build_forget_disposition_detail(catalog_entries: &[CatalogEntry]) -> Vec<TableDispositionEntry> {
    let mut entries: Vec<TableDispositionEntry> = catalog_entries
        .iter()
        .map(|entry| TableDispositionEntry {
            table_name: entry.table_name.clone(),
            feature_name: entry.feature_name.clone(),
            disposition: decide_forget_disposition(entry.retention_exempt),
        })
        .collect();

    entries.sort_by(|a, b| a.table_name.cmp(&b.table_name));
    entries
}

/// Serializa el detalle de disposición ya resuelto (y ya ordenado por
/// [`build_forget_disposition_detail`]) a JSON canónico -- listo para la
/// columna `disposition_detail` (`CHECK (json_valid(...))` de la
/// migración). Determinista: el mismo detalle produce SIEMPRE el mismo
/// texto.
pub fn disposition_detail_to_json(entries: &[TableDispositionEntry]) -> String {
    serde_json::to_string(entries)
        // Vec<TableDispositionEntry> con solo String/enum-a-texto siempre
        // serializa -- no hay floats ni claves de mapa no-string en juego.
        .expect("TableDispositionEntry siempre serializa a JSON")
}

// ── Hash de auditoría de `data_portability_requests` (APPEND-ONLY) ─────────

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de UNA fila de
/// `data_portability_requests`, encadenado al `audit_hash` de la fila
/// anterior en la secuencia GLOBAL (o
/// [`crate::domain::audit_log::GENESIS_PREVIOUS_HASH`] si es la fila
/// génesis) -- mismo estilo (buffer separado por bytes de control, nunca
/// JSON) que
/// [`crate::domain::master_account_hierarchy::compute_override_audit_hash`].
#[allow(clippy::too_many_arguments)]
pub fn compute_request_audit_hash(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    previous_audit_hash: &str,
    owner_id: &str,
    institutional_tag: &str,
    node_id: &str,
    request_group_id: &str,
    request_type: &str,
    status: &str,
    disposition_detail: Option<&str>,
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
    push(request_group_id);
    push(request_type);
    push(status);
    // NULL (EXPORT, o un FORGET aún sin detalle) se encadena como cadena
    // vacía -- idéntico criterio que `parent_owner_id`/`broker_connection_ref`
    // en el resto del substrato.
    push(disposition_detail.unwrap_or(""));

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    encode_hex(&hasher.finalize())
}

// ── Hash de auditoría de `exportable_data_catalog` (MUTABLE) ───────────────

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de una VERSIÓN de fila
/// de `exportable_data_catalog`, encadenado a la versión anterior de la
/// MISMA fila (`previous_audit_hash: None` en la versión génesis,
/// `row_version == 1`) -- mismo patrón que
/// [`crate::domain::master_account_hierarchy::compute_hierarchy_audit_hash`].
#[allow(clippy::too_many_arguments)]
pub fn compute_catalog_audit_hash(
    id: &str,
    created_at_ns: i64,
    row_version: i64,
    previous_audit_hash: Option<&str>,
    table_name: &str,
    feature_name: &str,
    owner_id_column: &str,
    retention_exempt: bool,
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
    push(table_name);
    push(feature_name);
    push(owner_id_column);
    push(if retention_exempt { "1" } else { "0" });

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    encode_hex(&hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Enums: round-trip de representación en texto ────────────────────────

    #[test]
    fn request_type_round_trips_through_its_string_representation() {
        for variant in [RequestType::Export, RequestType::Forget] {
            assert_eq!(RequestType::from_str_value(variant.as_str()), Some(variant));
        }
        assert_eq!(RequestType::from_str_value("UNKNOWN"), None);
    }

    #[test]
    fn request_status_round_trips_through_its_string_representation() {
        for variant in [RequestStatus::Received, RequestStatus::Processing, RequestStatus::Completed] {
            assert_eq!(RequestStatus::from_str_value(variant.as_str()), Some(variant));
        }
        assert_eq!(RequestStatus::from_str_value("UNKNOWN"), None);
    }

    // ── CRITERIO (Orden §8): decide_forget_disposition -- NUNCA hay DELETE ──

    /// CRITERIO DE CIERRE: `retention_exempt = true` -> SIEMPRE
    /// `PseudonymizeAndRetain`.
    #[test]
    fn decide_forget_disposition_retains_when_retention_exempt() {
        assert_eq!(decide_forget_disposition(true), ForgetDisposition::PseudonymizeAndRetain);
    }

    /// CRITERIO DE CIERRE: `retention_exempt = false` -> SIEMPRE
    /// `PseudonymizeAndPurge`.
    #[test]
    fn decide_forget_disposition_purges_when_not_retention_exempt() {
        assert_eq!(decide_forget_disposition(false), ForgetDisposition::PseudonymizeAndPurge);
    }

    /// Guardarraíl estructural: el catálogo de salida no tiene una tercera
    /// variante de borrado que un defecto pudiera alcanzar -- se confirma
    /// enumerando las dos únicas variantes existentes.
    #[test]
    fn forget_disposition_has_exactly_two_variants_neither_a_delete() {
        let retain = ForgetDisposition::PseudonymizeAndRetain;
        let purge = ForgetDisposition::PseudonymizeAndPurge;
        assert_ne!(retain, purge);
        assert_eq!(retain.as_str(), "PSEUDONYMIZE_AND_RETAIN");
        assert_eq!(purge.as_str(), "PSEUDONYMIZE_AND_PURGE");
    }

    // ── CRITERIO (Orden §8): filtro de secretos ──────────────────────────────

    /// CRITERIO DE CIERRE: `build_export_manifest` NUNCA incluye tablas
    /// cuyo nombre o columna de dueño delate un secreto de bróker, una
    /// clave de cifrado o una IP live -- las tablas normales sí sobreviven.
    #[test]
    fn build_export_manifest_excludes_secret_tables_and_keeps_normal_ones() {
        let catalog = vec![
            CatalogEntry {
                table_name: "verified_accounts".to_string(),
                feature_name: "verified-account-registry".to_string(),
                owner_id_column: "owner_id".to_string(),
                retention_exempt: false,
            },
            CatalogEntry {
                table_name: "api_credentials".to_string(),
                feature_name: "third-party-api-gateway".to_string(),
                owner_id_column: "owner_id".to_string(),
                retention_exempt: false,
            },
            CatalogEntry {
                table_name: "accounts".to_string(),
                feature_name: "central-identity".to_string(),
                owner_id_column: "owner_id".to_string(),
                retention_exempt: false,
            },
        ];

        let manifest = build_export_manifest("owner-1", &catalog);
        let table_names: Vec<&str> = manifest.tables.iter().map(|t| t.table_name.as_str()).collect();

        assert_eq!(
            table_names,
            vec!["accounts", "verified_accounts"],
            "api_credentials debe quedar excluida del manifiesto -- porta secretos (ADR-0093)"
        );
        assert_eq!(manifest.owner_id, "owner-1");
    }

    #[test]
    fn is_excluded_from_export_flags_all_secret_classes() {
        for secret in [
            "broker_credential_secret",
            "investor_password",
            "broker_api_key",
            "live_server_ip",
            "encryption_key_material",
            "PRIVATE_KEY", // case-insensitive
        ] {
            assert!(is_excluded_from_export(secret), "'{secret}' debe marcarse como secreto");
        }
        for normal in ["strategy_name", "account_balance_e8", "owner_id", "table_name"] {
            assert!(!is_excluded_from_export(normal), "'{normal}' NO debe marcarse como secreto");
        }
    }

    /// Determinismo: el manifiesto no depende del orden de entrada del
    /// catálogo.
    #[test]
    fn build_export_manifest_is_order_independent() {
        let a = vec![
            CatalogEntry { table_name: "zeta".to_string(), feature_name: "f".to_string(), owner_id_column: "owner_id".to_string(), retention_exempt: false },
            CatalogEntry { table_name: "alpha".to_string(), feature_name: "f".to_string(), owner_id_column: "owner_id".to_string(), retention_exempt: false },
        ];
        let b = vec![a[1].clone(), a[0].clone()];

        assert_eq!(build_export_manifest("owner-1", &a), build_export_manifest("owner-1", &b));
    }

    // ── CRITERIO (Orden §8): detalle de disposición determinista ────────────

    #[test]
    fn build_forget_disposition_detail_matches_retention_flag_per_table() {
        let catalog = vec![
            CatalogEntry { table_name: "usage_records".to_string(), feature_name: "usage-metering".to_string(), owner_id_column: "owner_id".to_string(), retention_exempt: true },
            CatalogEntry { table_name: "consent_records".to_string(), feature_name: "consent-registry".to_string(), owner_id_column: "owner_id".to_string(), retention_exempt: false },
        ];

        let detail = build_forget_disposition_detail(&catalog);
        assert_eq!(detail.len(), 2);
        // Ordenado por table_name: "consent_records" antes que "usage_records".
        assert_eq!(detail[0].table_name, "consent_records");
        assert_eq!(detail[0].disposition, ForgetDisposition::PseudonymizeAndPurge);
        assert_eq!(detail[1].table_name, "usage_records");
        assert_eq!(detail[1].disposition, ForgetDisposition::PseudonymizeAndRetain);
    }

    #[test]
    fn disposition_detail_to_json_produces_valid_json() {
        let catalog = vec![CatalogEntry {
            table_name: "audit_events".to_string(),
            feature_name: "audit-log".to_string(),
            owner_id_column: "owner_id".to_string(),
            retention_exempt: true,
        }];
        let detail = build_forget_disposition_detail(&catalog);
        let json = disposition_detail_to_json(&detail);

        let parsed: serde_json::Value = serde_json::from_str(&json).expect("debe ser JSON válido");
        assert!(parsed.is_array());
        assert_eq!(parsed[0]["table_name"], "audit_events");
        assert_eq!(parsed[0]["disposition"], "PSEUDONYMIZE_AND_RETAIN");
    }

    // ── Hash determinista ─────────────────────────────────────────────────

    #[test]
    fn compute_request_audit_hash_is_deterministic() {
        let a = compute_request_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "LIVE", "node-A", "grp-1", "EXPORT", "RECEIVED", None,
        );
        let b = compute_request_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "LIVE", "node-A", "grp-1", "EXPORT", "RECEIVED", None,
        );
        assert_eq!(a, b, "mismo evento, mismo hash");
    }

    #[test]
    fn compute_request_audit_hash_changes_when_status_changes() {
        let received = compute_request_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "LIVE", "node-A", "grp-1", "EXPORT", "RECEIVED", None,
        );
        let processing = compute_request_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "LIVE", "node-A", "grp-1", "EXPORT", "PROCESSING", None,
        );
        assert_ne!(received, processing, "cambiar el status debe cambiar el audit_hash");
    }

    #[test]
    fn compute_request_audit_hash_changes_when_disposition_detail_changes() {
        let without = compute_request_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "LIVE", "node-A", "grp-1", "FORGET", "RECEIVED", None,
        );
        let with = compute_request_audit_hash(
            "id-1", 1_000, 1, "GENESIS", "owner-1", "LIVE", "node-A", "grp-1", "FORGET", "RECEIVED",
            Some("[{\"table_name\":\"x\"}]"),
        );
        assert_ne!(without, with, "el disposition_detail debe formar parte del hash");
    }

    #[test]
    fn compute_catalog_audit_hash_is_deterministic_and_chains_by_row_version() {
        let genesis = compute_catalog_audit_hash("id-1", 1_000, 1, None, "usage_records", "usage-metering", "owner_id", true);
        let genesis_again = compute_catalog_audit_hash("id-1", 1_000, 1, None, "usage_records", "usage-metering", "owner_id", true);
        assert_eq!(genesis, genesis_again, "misma versión, mismo contenido -> mismo hash");

        let reclassified = compute_catalog_audit_hash("id-1", 2_000, 2, Some(&genesis), "usage_records", "usage-metering", "owner_id", false);
        assert_ne!(reclassified, genesis, "cambiar retention_exempt debe cambiar el hash");
    }
}
