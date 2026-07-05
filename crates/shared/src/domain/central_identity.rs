//! [CORE] Lógica pura de Central Identity (`docs/features/central-identity.md`,
//! ADR-0143, ADR-0144, ADR-0020).
//!
//! Sin I/O, sin reloj de sistema, sin azar sin semilla (ADR-0002/0004). Igual
//! que en `audit_log`/`telemetry`, el `id` y el timestamp los inyecta la
//! cáscara (persistencia) -- este módulo solo calcula.
//!
//! Tres piezas de lógica pura, tal como las pide la Feature en su sección
//! "Estructura Interna (FCIS)":
//! - [`compute_hardware_fingerprint`]: huella de hardware determinista.
//! - [`validate_email_format`]: validación de formato de correo.
//! - [`verify_oauth_signature`]: verificación de firma de un token OAuth
//!   dado el material público.
//!
//! Más el tipo de puerto público [`AccountIdentity`] (ADR-0137: `identity_out`,
//! catálogo de tipos, enmienda 2026-07-03) y el estado de verificación de
//! correo [`EmailVerificationStatus`].

use serde::Serialize;
use sha2::{Digest, Sha256};

// ── Huella de hardware determinista ──────────────────────────────────────────

/// Motivo por el que no se puede derivar una huella de hardware.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HardwareFingerprintError {
    /// La lista de identificadores estaba vacía, o todos sus elementos eran
    /// cadenas vacías/en blanco -- no hay material real de máquina que
    /// hashear. Si se dejara pasar, TODA máquina en ese estado produciría el
    /// MISMO `node_id` (el SHA-256 del buffer vacío), anulando la señal
    /// anti-abuso de `central-identity`.
    #[error("no hay identificadores de máquina no vacíos para derivar la huella de hardware")]
    NoUsableIdentifiers,
}

/// Calcula la huella de hardware (`node_id` de Grupo IV, ADR-0020) a
/// partir de una lista ordenada de identificadores de máquina (ej. UUID de
/// placa madre, serial de disco, MAC address -- los recolecta la cáscara,
/// nunca este módulo).
///
/// Determinismo (ADR-0002/0004): la MISMA lista de identificadores, en el
/// MISMO orden, produce SIEMPRE el mismo hash SHA-256 (hex, minúsculas) --
/// no hay reloj, no hay azar, no hay I/O. Cambiar un solo identificador (o
/// su orden) cambia el hash. El separador `\u{1F}` (Unit Separator ASCII,
/// mismo patrón que `audit_log::canonical_bytes`) evita que dos listas
/// distintas de identificadores puedan producir accidentalmente el mismo
/// stream de bytes al concatenarse.
///
/// **Rechaza el caso degenerado (guardarraíl anti-abuso):** una lista vacía
/// -- o una lista donde todos los identificadores son cadenas vacías o solo
/// espacios -- devuelve [`HardwareFingerprintError::NoUsableIdentifiers`] en
/// vez de un hash. Sin este guardarraíl, cualquier máquina sin
/// identificadores utilizables colapsaría al MISMO `node_id` (el SHA-256 del
/// buffer vacío, `e3b0c44...b855`), y la señal "N cuentas desde el mismo
/// hardware" (central-identity.md "Comportamientos Observables") sería falsa.
pub fn compute_hardware_fingerprint(
    machine_identifiers: &[String],
) -> Result<String, HardwareFingerprintError> {
    const SEP: char = '\u{1F}';

    // Exige al menos un identificador con contenido real (no vacío ni solo
    // espacios). El hash se sigue calculando sobre la lista COMPLETA (misma
    // representación canónica de siempre) -- esta comprobación solo veta el
    // caso degenerado, no altera qué se hashea cuando sí hay material válido.
    if !machine_identifiers.iter().any(|id| !id.trim().is_empty()) {
        return Err(HardwareFingerprintError::NoUsableIdentifiers);
    }

    let mut buffer = String::new();
    for identifier in machine_identifiers {
        buffer.push_str(identifier);
        buffer.push(SEP);
    }

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    let digest = hasher.finalize();

    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

// ── Validación de formato de correo ──────────────────────────────────────────

/// Motivo por el que un correo no pasa la validación de formato
/// (`docs/features/central-identity.md` "Comportamientos Observables":
/// "Cuando el usuario se registra con correo -> el sistema envía
/// verificación..." -- implica que el formato ya se validó antes).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EmailFormatError {
    #[error("el correo está vacío")]
    Empty,
    #[error("el correo contiene espacios en blanco")]
    ContainsWhitespace,
    #[error("el correo no tiene arroba (@)")]
    MissingAtSign,
    #[error("el correo tiene más de una arroba (@)")]
    MultipleAtSigns,
    #[error("la parte local (antes de la arroba) está vacía")]
    EmptyLocalPart,
    #[error("el dominio (después de la arroba) está vacío")]
    EmptyDomain,
    #[error("el dominio no tiene un punto (falta el TLD, ej. .com)")]
    DomainMissingDot,
}

/// Valida el formato de un correo con una regla simple y determinista:
/// exactamente una arroba, parte local no vacía, dominio no vacío con al
/// menos un punto que no sea el primer ni el último carácter, y sin
/// espacios en blanco en ningún lado.
///
/// No es una validación RFC 5322 completa (esa gramática admite casos
/// exóticos como comillas y comentarios que ningún proveedor de correo real
/// usa) -- es la regla práctica que basta para rechazar entradas mal
/// formadas y aceptar direcciones reales, exactamente lo que pide
/// `docs/features/central-identity.md`: "validación de formato de correo".
pub fn validate_email_format(email: &str) -> Result<(), EmailFormatError> {
    if email.is_empty() {
        return Err(EmailFormatError::Empty);
    }
    if email.chars().any(char::is_whitespace) {
        return Err(EmailFormatError::ContainsWhitespace);
    }

    let parts: Vec<&str> = email.split('@').collect();
    match parts.as_slice() {
        [_local_no_at] => Err(EmailFormatError::MissingAtSign),
        [local, domain] => {
            if local.is_empty() {
                return Err(EmailFormatError::EmptyLocalPart);
            }
            if domain.is_empty() {
                return Err(EmailFormatError::EmptyDomain);
            }
            // El punto del TLD no puede ser el primer ni el último carácter
            // del dominio (ej. rechaza "a@.com" y "a@b.").
            if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
                return Err(EmailFormatError::DomainMissingDot);
            }
            Ok(())
        }
        _ => Err(EmailFormatError::MultipleAtSigns),
    }
}

/// Normaliza un correo a su forma canónica: recorta espacios en los extremos
/// (`trim`) y lo pasa a minúsculas (`to_lowercase`).
///
/// Por qué (anti-abuso / unicidad case-insensitive): el índice único de la
/// tabla `accounts` compara bytes exactos (BINARY), así que sin normalizar,
/// `Case@Example.com` y `case@example.com` crearían DOS cuentas distintas --
/// rompiendo "una cuenta por correo" (central-identity.md) y abriendo una
/// evasión trivial de los límites por-cuenta que el substrato de licencias
/// quiere impedir. Normalizar en la frontera, antes de validar y persistir,
/// deja el dato limpio y la unicidad efectiva.
///
/// Pura (ADR-0002/0004): sin I/O, sin reloj, sin azar -- el mismo correo de
/// entrada siempre produce la misma forma canónica.
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

// ── Verificación de firma de token OAuth ─────────────────────────────────────

/// Material de un token OAuth a verificar: el payload firmado y la firma
/// que lo acompaña, codificada en hexadecimal.
#[derive(Debug, Clone)]
pub struct OAuthTokenMaterial<'a> {
    /// El contenido firmado del token (ej. el cuerpo del JWT ya decodificado
    /// de base64, antes de aplicar la firma).
    pub payload: &'a str,
    /// La firma que acompaña al payload, codificada en hexadecimal.
    pub signature_hex: &'a str,
}

/// Verifica la firma de un token OAuth dado el material público del
/// proveedor (`docs/features/central-identity.md` "Estructura Interna":
/// "verificación de firma de token OAuth (dado el material público)").
///
/// **Simplificación deliberada:** un proveedor OAuth real (Google/GitHub)
/// firma con una clave asimétrica (RS256/ES256) y publica su clave pública
/// en un JWKS; verificarla de verdad requiere un crate de criptografía
/// asimétrica, que está fuera de alcance de este cimiento local (el
/// adaptador real de login federado es trabajo diferido, igual que la
/// verificación central -- ADR-0144). Esta función implementa el MISMO
/// contrato observable -- dado un payload y un material de verificación
/// público, decide si la firma es válida, de forma pura y determinista --
/// usando SHA-256 sobre `payload + material` como sustituto verificable.
/// Cuando el adaptador OAuth real se construya, esta función se reemplaza
/// sin tocar el resto del sistema (mismo patrón puerto+stub que
/// [`super::super::orchestrator::central_identity::CentralIdentityVerifier`]).
pub fn verify_oauth_signature(material: &OAuthTokenMaterial<'_>, public_key_material: &str) -> bool {
    const SEP: char = '\u{1F}';

    let mut buffer = String::new();
    buffer.push_str(material.payload);
    buffer.push(SEP);
    buffer.push_str(public_key_material);

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    let digest = hasher.finalize();
    let expected: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();

    expected == material.signature_hex
}

// ── Hash de auditoría encadenado (row_version, ADR-0141) ─────────────────────

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de una VERSIÓN de fila
/// de `accounts`, encadenado al `audit_hash` de la versión anterior de esa
/// misma fila (o [`crate::domain::audit_log::GENESIS_PREVIOUS_HASH`] si es
/// la versión génesis, `row_version == 1`).
///
/// Mismo patrón que `job.rs::compute_job_audit_hash`: a diferencia de
/// `audit_events` (una cadena global entre TODAS las filas), aquí la cadena
/// es POR CUENTA -- cada fila de `accounts` encadena sus propias versiones
/// sucesivas (`row_version` 1, 2, 3...), no las versiones de otras cuentas.
///
/// Determinista: los mismos argumentos siempre producen el mismo digest
/// (ADR-0002/0004) -- sin I/O, sin reloj, sin azar dentro de esta función.
#[allow(clippy::too_many_arguments)]
pub fn compute_account_audit_hash(
    id: &str,
    updated_at_ns: i64,
    row_version: i64,
    previous_audit_hash: Option<&str>,
    email: &str,
    email_verification_status: EmailVerificationStatus,
    oauth_provider: Option<&str>,
    node_id: &str,
) -> String {
    const SEP: char = '\u{1F}';

    let mut buffer = String::new();
    let mut push = |field: &str| {
        buffer.push_str(field);
        buffer.push(SEP);
    };

    push(id);
    push(&updated_at_ns.to_string());
    push(&row_version.to_string());
    push(previous_audit_hash.unwrap_or(crate::domain::audit_log::GENESIS_PREVIOUS_HASH));
    push(email);
    push(email_verification_status.as_str());
    push(oauth_provider.unwrap_or(""));
    push(node_id);

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    let digest = hasher.finalize();

    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ── Estado de verificación de correo ─────────────────────────────────────────

/// Estado de verificación de correo de una cuenta (columna propia
/// `email_verification_status`, con `CHECK` en la migración).
///
/// `EMAIL_VERIFICATION_REQUIRED` es FIJO (central-identity.md "Parámetros
/// Configurables"): toda cuenta nace en `Pending` y solo pasa a `Verified`
/// tras confirmar el correo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailVerificationStatus {
    /// Registrada, esperando confirmación del correo.
    Pending,
    /// Correo confirmado.
    Verified,
    /// Verificación rechazada (ej. enlace expirado, correo denunciado).
    Rejected,
}

impl EmailVerificationStatus {
    /// Representación canónica en texto (la que se persiste en la columna
    /// `email_verification_status`).
    pub fn as_str(&self) -> &'static str {
        match self {
            EmailVerificationStatus::Pending => "PENDING",
            EmailVerificationStatus::Verified => "VERIFIED",
            EmailVerificationStatus::Rejected => "REJECTED",
        }
    }

    /// Reconstruye el estado desde el valor persistido, o `None` si el
    /// valor no es ninguno de los tres reconocidos (integridad de datos).
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "PENDING" => Some(EmailVerificationStatus::Pending),
            "VERIFIED" => Some(EmailVerificationStatus::Verified),
            "REJECTED" => Some(EmailVerificationStatus::Rejected),
            _ => None,
        }
    }
}

// ── Tipo de puerto público (ADR-0137: identity_out) ──────────────────────────

/// Identidad de cuenta vinculada a la instancia -- el tipo de puerto
/// `AccountIdentity` del catálogo (ADR-0137, enmienda 2026-07-03): "identidad
/// de cuenta vinculada a la instancia (`owner_id` + estado de verificación)".
///
/// **Guardarraíl ADR-0093 (estructural, no solo por convención):** este
/// struct SOLO tiene los cinco campos de abajo. No existe un campo para
/// contraseña, clave de bróker, IP de servidor live, ni ningún secreto --
/// quien intente añadir uno rompe el test
/// [`tests::account_identity_json_never_leaks_secret_fields`] de este mismo
/// módulo, que fija la lista exacta de claves permitidas en el JSON
/// serializado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AccountIdentity {
    /// Grupo II ADR-0020: dueño de la cuenta (capital/IP). Para una
    /// cuenta retail individual, es su propio `id`.
    pub owner_id: String,
    /// Correo de la cuenta (no es un secreto -- es el identificador de
    /// contacto, igual que en cualquier proveedor de correo).
    pub email: String,
    /// Estado de verificación de correo (serializado como su string
    /// canónico -- ver [`EmailVerificationStatus::as_str`]).
    #[serde(serialize_with = "serialize_email_verification_status")]
    pub email_verification_status: EmailVerificationStatus,
    /// Grupo IV ADR-0020: huella de hardware determinista de la
    /// instancia vinculada.
    pub node_id: String,
    /// Grupo II ADR-0020: entorno/etiqueta institucional.
    pub institutional_tag: String,
}

/// Serializa [`EmailVerificationStatus`] como su string canónico
/// (`"PENDING"`/`"VERIFIED"`/`"REJECTED"`) en vez de la representación
/// interna del enum -- el JSON que ve el usuario del CLI (ADR-0142) usa el
/// mismo vocabulario que la columna `email_verification_status` en SQLite.
fn serialize_email_verification_status<S>(
    status: &EmailVerificationStatus,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(status.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Huella de hardware: determinismo (criterio de aceptación #2) ────────

    /// CRITERIO DE CIERRE: los mismos identificadores de máquina, en el
    /// mismo orden, producen SIEMPRE el mismo hash -- probado con dos
    /// llamadas independientes (simula "entre arranques").
    #[test]
    fn hardware_fingerprint_is_deterministic_for_same_identifiers() {
        let identifiers = vec![
            "motherboard-uuid-ABC123".to_string(),
            "disk-serial-XYZ789".to_string(),
        ];

        let first_boot = compute_hardware_fingerprint(&identifiers).expect("huella válida");
        let second_boot = compute_hardware_fingerprint(&identifiers).expect("huella válida");

        assert_eq!(
            first_boot, second_boot,
            "la misma lista de identificadores debe producir el mismo hash entre arranques"
        );
        assert!(!first_boot.is_empty());
    }

    /// CRITERIO DE CIERRE: alterar un solo identificador cambia el hash.
    #[test]
    fn hardware_fingerprint_differs_when_an_identifier_changes() {
        let original = vec![
            "motherboard-uuid-ABC123".to_string(),
            "disk-serial-XYZ789".to_string(),
        ];
        let altered = vec![
            "motherboard-uuid-ABC123".to_string(),
            "disk-serial-CHANGED".to_string(),
        ];

        let hash_original = compute_hardware_fingerprint(&original).expect("huella válida");
        let hash_altered = compute_hardware_fingerprint(&altered).expect("huella válida");

        assert_ne!(
            hash_original, hash_altered,
            "cambiar un identificador de hardware debe cambiar la huella"
        );
    }

    /// El orden de los identificadores importa (dos listas con los mismos
    /// elementos en distinto orden no son "el mismo hardware" para esta
    /// función -- la cáscara es responsable de normalizar el orden antes de
    /// llamar).
    #[test]
    fn hardware_fingerprint_is_order_sensitive() {
        let order_a = vec!["a".to_string(), "b".to_string()];
        let order_b = vec!["b".to_string(), "a".to_string()];

        assert_ne!(
            compute_hardware_fingerprint(&order_a).expect("huella válida"),
            compute_hardware_fingerprint(&order_b).expect("huella válida")
        );
    }

    /// CRITERIO DE CIERRE (Defecto 2 del QA): una lista de identificadores
    /// VACÍA no produce un hash -- devuelve error. Sin este guardarraíl,
    /// toda máquina sin identificadores colapsaría al SHA-256 del buffer
    /// vacío (el mismo `node_id` para cualquiera), rompiendo la señal
    /// anti-abuso.
    #[test]
    fn hardware_fingerprint_rejects_empty_list() {
        let empty: Vec<String> = vec![];
        assert_eq!(
            compute_hardware_fingerprint(&empty),
            Err(HardwareFingerprintError::NoUsableIdentifiers),
            "una lista vacía debe fallar limpio, no devolver un hash constante"
        );
    }

    /// CRITERIO DE CIERRE (Defecto 2 del QA): una lista con solo cadenas
    /// vacías o en blanco tampoco tiene material real de máquina -> error.
    #[test]
    fn hardware_fingerprint_rejects_all_blank_identifiers() {
        let all_blank = vec!["".to_string(), "   ".to_string(), "\t".to_string()];
        assert_eq!(
            compute_hardware_fingerprint(&all_blank),
            Err(HardwareFingerprintError::NoUsableIdentifiers),
            "identificadores solo-espacios no cuentan como material de máquina"
        );
    }

    /// Con al menos un identificador no vacío, sí produce hash (aunque otros
    /// estén en blanco) -- el guardarraíl solo veta el caso totalmente
    /// degenerado, no altera el hash cuando hay material válido.
    #[test]
    fn hardware_fingerprint_accepts_at_least_one_nonblank_identifier() {
        let mixed = vec!["".to_string(), "real-disk-serial".to_string()];
        assert!(compute_hardware_fingerprint(&mixed).is_ok());
    }

    // ── Normalización de correo (Defecto 3 del QA) ──────────────────────────

    /// CRITERIO DE CIERRE (Defecto 3): `normalize_email` recorta espacios y
    /// baja a minúsculas, de modo que variantes de mayúsculas/espacios del
    /// mismo correo colapsan a la MISMA forma canónica.
    #[test]
    fn normalize_email_lowercases_and_trims() {
        assert_eq!(normalize_email("  Case@Example.COM  "), "case@example.com");
        assert_eq!(normalize_email("case@example.com"), "case@example.com");
        assert_eq!(
            normalize_email("Case@Example.com"),
            normalize_email("case@example.com"),
            "dos variantes de mayúsculas del mismo correo deben normalizar igual"
        );
    }

    // ── Validación de correo (criterio de aceptación #3 en la Orden) ────────

    #[test]
    fn validate_email_format_accepts_a_well_formed_email() {
        assert_eq!(validate_email_format("user@example.com"), Ok(()));
        assert_eq!(validate_email_format("a.b+tag@sub.example.co"), Ok(()));
    }

    #[test]
    fn validate_email_format_rejects_missing_at_sign() {
        assert_eq!(
            validate_email_format("user-example.com"),
            Err(EmailFormatError::MissingAtSign)
        );
    }

    #[test]
    fn validate_email_format_rejects_multiple_at_signs() {
        assert_eq!(
            validate_email_format("user@@example.com"),
            Err(EmailFormatError::MultipleAtSigns)
        );
        assert_eq!(
            validate_email_format("us@er@example.com"),
            Err(EmailFormatError::MultipleAtSigns)
        );
    }

    #[test]
    fn validate_email_format_rejects_empty_local_part() {
        assert_eq!(
            validate_email_format("@example.com"),
            Err(EmailFormatError::EmptyLocalPart)
        );
    }

    #[test]
    fn validate_email_format_rejects_domain_without_dot() {
        assert_eq!(
            validate_email_format("user@localhost"),
            Err(EmailFormatError::DomainMissingDot)
        );
    }

    #[test]
    fn validate_email_format_rejects_whitespace() {
        assert_eq!(
            validate_email_format("user @example.com"),
            Err(EmailFormatError::ContainsWhitespace)
        );
    }

    #[test]
    fn validate_email_format_rejects_empty_string() {
        assert_eq!(validate_email_format(""), Err(EmailFormatError::Empty));
    }

    // ── Verificación de firma OAuth ──────────────────────────────────────────

    #[test]
    fn verify_oauth_signature_accepts_a_valid_signature() {
        let public_key_material = "provider-public-key-material";
        let payload = r#"{"sub":"user-123","iss":"google"}"#;

        // Calcula la firma esperada con la misma función determinista para
        // construir un caso de prueba válido (espejo de cómo un proveedor
        // real firmaría el payload con su clave privada correspondiente).
        let mut hasher = Sha256::new();
        hasher.update(format!("{payload}\u{1F}{public_key_material}").as_bytes());
        let signature_hex: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();

        let material = OAuthTokenMaterial { payload, signature_hex: &signature_hex };
        assert!(verify_oauth_signature(&material, public_key_material));
    }

    #[test]
    fn verify_oauth_signature_rejects_a_tampered_payload() {
        let public_key_material = "provider-public-key-material";
        let original_payload = r#"{"sub":"user-123"}"#;

        let mut hasher = Sha256::new();
        hasher.update(format!("{original_payload}\u{1F}{public_key_material}").as_bytes());
        let signature_hex: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();

        // El atacante cambia el sub del payload sin volver a firmar.
        let tampered_material = OAuthTokenMaterial {
            payload: r#"{"sub":"user-999"}"#,
            signature_hex: &signature_hex,
        };

        assert!(!verify_oauth_signature(&tampered_material, public_key_material));
    }

    // ── Guardarraíl ADR-0093 (criterio de aceptación #3 en la Orden) ─────────

    /// CRITERIO DE CIERRE (guardarraíl ADR-0093): el JSON serializado de
    /// `AccountIdentity` contiene EXACTAMENTE estas cinco claves -- ninguna
    /// más. Si alguien añade un campo de credenciales de bróker o una IP de
    /// servidor live a este struct, este test se rompe de inmediato.
    #[test]
    fn account_identity_json_never_leaks_secret_fields() {
        let identity = AccountIdentity {
            owner_id: "owner-1".to_string(),
            email: "user@example.com".to_string(),
            email_verification_status: EmailVerificationStatus::Pending,
            node_id: "fingerprint-hash".to_string(),
            institutional_tag: "DRASUS_LOCAL".to_string(),
        };

        let json = serde_json::to_value(&identity).expect("AccountIdentity debe serializar a JSON");
        let object = json.as_object().expect("el JSON de AccountIdentity debe ser un objeto");

        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();

        assert_eq!(
            keys,
            vec![
                "email",
                "email_verification_status",
                "institutional_tag",
                "node_id",
                "owner_id",
            ],
            "AccountIdentity solo puede exponer estas cinco claves -- ninguna credencial de bróker, \
             IP de servidor live, contraseña ni token de sesión (ADR-0093)"
        );

        // Guardarraíl adicional: ninguno de los valores contiene los
        // literales típicos de un secreto que alguien pudiera colar dentro
        // de un campo de texto existente.
        let json_string = json.to_string();
        for forbidden in ["password", "api_key", "api-key", "broker_secret", "192.168.", "10.0.0."] {
            assert!(
                !json_string.to_lowercase().contains(forbidden),
                "el JSON de AccountIdentity no debe contener '{forbidden}'"
            );
        }
    }

    // ── Hash de auditoría encadenado por cuenta ─────────────────────────────

    #[test]
    fn compute_account_audit_hash_is_deterministic() {
        let hash_a = compute_account_audit_hash(
            "id-1", 1_000, 1, None, "a@b.com", EmailVerificationStatus::Pending, None, "node-1",
        );
        let hash_b = compute_account_audit_hash(
            "id-1", 1_000, 1, None, "a@b.com", EmailVerificationStatus::Pending, None, "node-1",
        );
        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn compute_account_audit_hash_changes_when_verification_status_changes() {
        let pending = compute_account_audit_hash(
            "id-1", 2_000, 2, Some("prev-hash"), "a@b.com", EmailVerificationStatus::Pending, None, "node-1",
        );
        let verified = compute_account_audit_hash(
            "id-1", 2_000, 2, Some("prev-hash"), "a@b.com", EmailVerificationStatus::Verified, None, "node-1",
        );
        assert_ne!(pending, verified);
    }

    #[test]
    fn email_verification_status_round_trips_through_its_string_representation() {
        for status in [
            EmailVerificationStatus::Pending,
            EmailVerificationStatus::Verified,
            EmailVerificationStatus::Rejected,
        ] {
            let as_str = status.as_str();
            assert_eq!(EmailVerificationStatus::from_str_value(as_str), Some(status));
        }

        assert_eq!(EmailVerificationStatus::from_str_value("UNKNOWN"), None);
    }
}
