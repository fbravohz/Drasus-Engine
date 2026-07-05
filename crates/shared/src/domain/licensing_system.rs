//! [CORE] Lógica pura de Licensing System (`docs/features/licensing-system.md`,
//! ADR-0143, ADR-0144, ADR-0141, ADR-0093, ADR-0020, STORY-028).
//!
//! Sin I/O, sin reloj de sistema, sin aleatoriedad sin semilla
//! (ADR-0002/0004). El reloj lo inyecta quien llama (puerto `Clock`); la
//! generación de claves criptográficas (que sí necesita azar real) vive en
//! la Shell (`orchestrator::licensing_system::LocalStubLicenseIssuer`) --
//! este módulo solo VERIFICA firmas, nunca las genera con una clave privada.
//!
//! Piezas de lógica pura, tal como las pide la Feature en su sección
//! "Estructura Interna (FCIS)" y la Orden STORY-028 §4.2:
//! - [`verify_license_signature`]: verificación de firma **asimétrica**
//!   Ed25519 (NO HMAC -- ADR-0093 §3: la clave incrustada en el cliente es
//!   la PÚBLICA, que solo verifica; la privada firma en el servidor y jamás
//!   sale de ahí).
//! - [`hardware_matches`]: compara el `node_id` de la licencia contra el
//!   `node_id` que trae `AccountIdentity` (identity_in) -- NUNCA recalcula
//!   la huella (esa lógica ya vive en `central_identity`, ADR-0144 FIJO).
//! - [`evaluate_heartbeat_status`]: comparación determinista de
//!   heartbeat/gracia con el reloj inyectado.
//! - [`should_suppress_work_telemetry`]: orden de supresión de telemetría
//!   por tier (ADR-0143).
//! - [`derive_execution_gate`]: combina todo lo anterior en el veredicto
//!   final `ExecutionGate` (ADR-0137: catálogo de tipos, puerto
//!   `execution_gate_out`).

use ed25519_dalek::{Signature, VerifyingKey};
use serde::Serialize;
use sha2::{Digest, Sha256};

// ── Codificación hexadecimal (sin dependencia nueva -- mismo patrón que
//    central_identity::compute_hardware_fingerprint) ─────────────────────────

/// Codifica bytes crudos a su representación hexadecimal en minúsculas.
fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Decodifica una cadena hexadecimal a bytes crudos. Devuelve `None` si la
/// cadena tiene longitud impar o contiene un carácter que no es dígito
/// hexadecimal -- ambos casos son "no es hex válido", no un pánico.
fn decode_hex(hex_str: &str) -> Option<Vec<u8>> {
    if !hex_str.len().is_multiple_of(2) {
        return None;
    }
    (0..hex_str.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_str[i..i + 2], 16).ok())
        .collect()
}

// ── Tier de licencia ──────────────────────────────────────────────────────

/// Nivel de licencia (`docs/features/licensing-system.md` "Niveles de
/// Licencia (tiers de ADR-0143)").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LicenseTier {
    /// Pago al corriente: privacidad real, la telemetría de trabajo se
    /// suprime en origen (ADR-0143).
    Sovereign,
    /// Gratuito: el trabajo del usuario alimenta a la Cabina de Mando.
    Explorer,
}

impl LicenseTier {
    /// Representación canónica en texto (la que se persiste en la columna
    /// `tier` y la que acepta el CHECK de la migración).
    pub fn as_str(&self) -> &'static str {
        match self {
            LicenseTier::Sovereign => "SOVEREIGN",
            LicenseTier::Explorer => "EXPLORER",
        }
    }

    /// Reconstruye el tier desde el valor persistido/JSON, o `None` si no es
    /// ninguno de los dos reconocidos (integridad de datos).
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value {
            "SOVEREIGN" => Some(LicenseTier::Sovereign),
            "EXPLORER" => Some(LicenseTier::Explorer),
            _ => None,
        }
    }
}

// ── Payload canónico de licencia (lo que se firma/verifica) ─────────────────

/// El contenido de una licencia que el emisor firma y el cliente verifica.
/// Es el "documento" -- `license_id` + a quién y a qué máquina ata la
/// licencia + su tier + sus fechas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicensePayload<'a> {
    pub license_id: &'a str,
    pub owner_id: &'a str,
    /// Huella de hardware a la que esta licencia queda atada. Se compara
    /// (nunca se recalcula aquí) contra `AccountIdentity.node_id`.
    pub node_id: &'a str,
    pub tier: LicenseTier,
    pub issued_at_ns: i64,
    pub heartbeat_expires_at_ns: i64,
}

/// Construye la representación canónica en bytes de un [`LicensePayload`]
/// para firmar/verificar -- mismo patrón de separador `\u{1F}` (Unit
/// Separator ASCII) que `central_identity::compute_hardware_fingerprint`,
/// para que dos payloads con campos distintos nunca puedan colisionar en el
/// mismo stream de bytes al concatenarse.
///
/// Determinista (ADR-0002/0004): el mismo payload siempre produce los mismos
/// bytes -- sin esto, la MISMA licencia podría verificar distinto según el
/// orden de construcción, lo cual rompería firmar-una-vez-verificar-siempre.
pub fn canonical_license_bytes(payload: &LicensePayload<'_>) -> Vec<u8> {
    const SEP: char = '\u{1F}';

    let mut buffer = String::new();
    let mut push = |field: &str| {
        buffer.push_str(field);
        buffer.push(SEP);
    };

    push(payload.license_id);
    push(payload.owner_id);
    push(payload.node_id);
    push(payload.tier.as_str());
    push(&payload.issued_at_ns.to_string());
    push(&payload.heartbeat_expires_at_ns.to_string());

    buffer.into_bytes()
}

// ── Verificación de firma asimétrica (ADR-0093 §3 — NO HMAC) ────────────────

/// Motivo por el que una firma de licencia no verifica.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LicenseSignatureError {
    /// La clave pública incrustada no decodifica a los 32 bytes que exige
    /// Ed25519 (hex inválido, o longitud incorrecta).
    #[error("la clave pública no tiene el formato Ed25519 esperado (32 bytes en hex)")]
    InvalidPublicKeyEncoding,
    /// La firma no decodifica a los 64 bytes que exige Ed25519.
    #[error("la firma no tiene el formato Ed25519 esperado (64 bytes en hex)")]
    InvalidSignatureEncoding,
    /// La firma decodificó correctamente pero NO corresponde al payload con
    /// esa clave pública -- payload alterado, firma alterada, o clave
    /// equivocada. Ed25519 no distingue estos tres casos entre sí (por
    /// diseño: un atacante no debe poder distinguir "casi correcto" de
    /// "totalmente falso").
    #[error("la firma no corresponde al payload firmado con esta clave pública")]
    SignatureMismatch,
}

/// Verifica que `signature_hex` es una firma **asimétrica Ed25519** válida
/// de `payload`, producida por el poseedor de la clave privada
/// correspondiente a `public_key_hex`.
///
/// **Por qué asimétrica y no HMAC (ADR-0093 §3, corrección obligatoria del
/// Gate de Coherencia):** HMAC es simétrico -- firmar y verificar usan LA
/// MISMA clave. Si esa clave viviera incrustada en el cliente (necesario
/// para poder verificar sin red), cualquiera podría extraerla del binario y
/// FIRMAR sus propias licencias falsas. Con Ed25519 (asimétrico), el cliente
/// solo tiene la clave PÚBLICA (`public_key_hex`) -- basta para verificar,
/// pero es matemáticamente inútil para firmar. La clave PRIVADA de firma
/// vive solo en el emisor (Cabina de Mando real, o el stub de desarrollo) y
/// nunca sale de ahí.
///
/// Pura y determinista (ADR-0002/0004): sin I/O, sin reloj, sin azar --
/// dados los mismos tres argumentos, siempre devuelve el mismo resultado.
pub fn verify_license_signature(
    payload: &LicensePayload<'_>,
    signature_hex: &str,
    public_key_hex: &str,
) -> Result<(), LicenseSignatureError> {
    // Decodifica y valida el tamaño de la clave pública (32 bytes, Ed25519).
    let public_key_bytes =
        decode_hex(public_key_hex).ok_or(LicenseSignatureError::InvalidPublicKeyEncoding)?;
    let public_key_array: [u8; 32] = public_key_bytes
        .as_slice()
        .try_into()
        .map_err(|_| LicenseSignatureError::InvalidPublicKeyEncoding)?;
    let verifying_key = VerifyingKey::from_bytes(&public_key_array)
        .map_err(|_| LicenseSignatureError::InvalidPublicKeyEncoding)?;

    // Decodifica y valida el tamaño de la firma (64 bytes, Ed25519).
    let signature_bytes =
        decode_hex(signature_hex).ok_or(LicenseSignatureError::InvalidSignatureEncoding)?;
    let signature_array: [u8; 64] = signature_bytes
        .as_slice()
        .try_into()
        .map_err(|_| LicenseSignatureError::InvalidSignatureEncoding)?;
    let signature = Signature::from_bytes(&signature_array);

    let message = canonical_license_bytes(payload);

    // `verify_strict` (en vez de `verify`) rechaza firmas maleables --
    // la variante estricta recomendada por el propio crate para nuevos usos.
    verifying_key
        .verify_strict(&message, &signature)
        .map_err(|_| LicenseSignatureError::SignatureMismatch)
}

// ── Comparación de huella de hardware (reutilización, ADR-0144 FIJO) ────────

/// Compara el `node_id` grabado en la licencia contra el `node_id` que trae
/// la identidad de la instancia (`AccountIdentity.node_id`, producida por
/// `central-identity`).
///
/// **Por qué no recalcula la huella:** `central-identity` ya la deriva
/// (`compute_hardware_fingerprint`) y la expone vía el puerto `identity_in`.
/// Volver a calcularla aquí sería lógica duplicada -- exactamente lo que la
/// corrección obligatoria #2 del Gate de Coherencia de STORY-028 prohíbe.
/// Esta función es una simple comparación de strings.
pub fn hardware_matches(license_node_id: &str, instance_node_id: &str) -> bool {
    license_node_id == instance_node_id
}

// ── Heartbeat y período de gracia ────────────────────────────────────────────

/// Configuración de ventanas de heartbeat (`docs/features/licensing-system.md`
/// "Parámetros Configurables": `RECHECK_WINDOW`, `GRACE_PERIOD`).
#[derive(Debug, Clone, Copy)]
pub struct HeartbeatConfig {
    /// Ventana (ns) antes del vencimiento del heartbeat donde se activan
    /// alertas visuales (default 5 días).
    pub recheck_window_ns: i64,
    /// Días adicionales (ns) de ejecución permitida tras vencer el heartbeat
    /// antes del bloqueo funcional (default 7 días).
    pub grace_period_ns: i64,
}

impl Default for HeartbeatConfig {
    /// Defaults declarados por la Feature: `RECHECK_WINDOW` = 5 días,
    /// `GRACE_PERIOD` = 7 días.
    fn default() -> Self {
        const NANOS_PER_DAY: i64 = 24 * 60 * 60 * 1_000_000_000;
        Self { recheck_window_ns: 5 * NANOS_PER_DAY, grace_period_ns: 7 * NANOS_PER_DAY }
    }
}

/// Intervalo por defecto de heartbeat (`docs/features/licensing-system.md`
/// "Parámetros Configurables": `HEARTBEAT_INTERVAL`, default 90 días) --
/// cuánto dura la validez de una licencia recién emitida/refrescada antes de
/// exigir revalidación. Anclado aquí (Inundación de Fundaciones) para que el
/// emisor (Shell) y el harness de verificación (CLI) no dupliquen el número.
pub const DEFAULT_HEARTBEAT_INTERVAL_NS: i64 = 90 * 24 * 60 * 60 * 1_000_000_000;

/// El estado de vigencia del heartbeat de una licencia en un instante dado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeartbeatStatus {
    /// Lejos del vencimiento -- opera normal, sin alertas.
    Fresh,
    /// Dentro de la ventana de recheck (antes de vencer) -- opera normal,
    /// pero la interfaz debe mostrar alertas preventivas.
    RecheckWindow,
    /// Vencido, pero todavía dentro del período de gracia -- opera, sin
    /// bloquear al usuario honesto que perdió conexión momentáneamente.
    WithinGrace,
    /// Vencido y agotado el período de gracia -- restringe operaciones.
    Expired,
}

/// Compara determinísticamente `now_ns` (del reloj inyectado, NUNCA
/// `SystemTime::now()`) contra `heartbeat_expires_at_ns` y devuelve en cuál
/// de las cuatro ventanas cae.
///
/// Las tres fronteras son: `expires_at - recheck_window` (entra en
/// `RecheckWindow`), `expires_at` (entra en `WithinGrace`), y
/// `expires_at + grace_period` (entra en `Expired`). Usa `saturating_sub`/
/// `saturating_add` para que una configuración con ventanas absurdamente
/// grandes no desborde `i64` en vez de producir un resultado incorrecto.
pub fn evaluate_heartbeat_status(
    now_ns: i64,
    heartbeat_expires_at_ns: i64,
    config: &HeartbeatConfig,
) -> HeartbeatStatus {
    let recheck_starts_at = heartbeat_expires_at_ns.saturating_sub(config.recheck_window_ns);
    let grace_ends_at = heartbeat_expires_at_ns.saturating_add(config.grace_period_ns);

    if now_ns < recheck_starts_at {
        HeartbeatStatus::Fresh
    } else if now_ns < heartbeat_expires_at_ns {
        HeartbeatStatus::RecheckWindow
    } else if now_ns < grace_ends_at {
        HeartbeatStatus::WithinGrace
    } else {
        HeartbeatStatus::Expired
    }
}

/// Proyecta un [`HeartbeatStatus`] a su `compliance_status_id` persistido
/// (Grupo V, ADR-0020) -- `Fresh`/`RecheckWindow` cuentan como "al
/// corriente" (`ACTIVE`); solo `Expired` es lo que la Feature llama
/// "vencido".
pub fn heartbeat_status_to_compliance_status_id(status: HeartbeatStatus) -> &'static str {
    match status {
        HeartbeatStatus::Fresh | HeartbeatStatus::RecheckWindow => "ACTIVE",
        HeartbeatStatus::WithinGrace => "GRACE",
        HeartbeatStatus::Expired => "EXPIRED",
    }
}

// ── Supresión de telemetría por tier (ADR-0143) ──────────────────────────────

/// Decide si debe suprimirse en origen la telemetría de trabajo, según el
/// tier y si la licencia sigue "al corriente" (ADR-0143: "la supresión se
/// gobierna por el estado de licencia, evaluado localmente con licencia
/// cacheada y período de gracia").
///
/// - **Sovereign al corriente** (`Fresh`/`RecheckWindow`/`WithinGrace` --
///   cualquier estado que no sea `Expired`): suprime (`true`) -- privacidad
///   real mientras el pago siga vigente, incluida la ventana de gracia sin
///   conexión.
/// - **Sovereign vencido** (`Expired`): NO suprime (`false`) -- ADR-0143:
///   "si dejas de pagar... cesa el trato premium", la emisión se reactiva.
/// - **Explorer:** nunca suprime (`false`) -- el modelo gratuito exige el
///   firehose completo (ADR-0143).
pub fn should_suppress_work_telemetry(tier: LicenseTier, heartbeat_status: HeartbeatStatus) -> bool {
    match tier {
        LicenseTier::Sovereign => !matches!(heartbeat_status, HeartbeatStatus::Expired),
        LicenseTier::Explorer => false,
    }
}

// ── Límites de plan (puerto `plan_limits_in`, stub hasta plan-tier-quota) ────

/// Límites vigentes de un plan (tipo de puerto `PlanLimits`, ADR-0137
/// catálogo, enmienda 2026-07-03) -- **stub** hasta que exista
/// `plan-tier-quota` (cimiento #3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanLimits {
    /// Máximo de activaciones simultáneas (máquinas distintas) permitidas
    /// para este tier (`ACTIVATIONS_PER_TIER`: Explorer 1, Sovereign 3).
    pub max_activations: i64,
    /// Features del catálogo habilitadas para este plan (vacío = todas las
    /// básicas). No usado por el gate de esta Story; anclado para cuando
    /// `plan-tier-quota` real lo materialice (Foundation Inundation).
    pub features_enabled: Vec<String>,
}

// ── Veredicto de ejecución (puerto `execution_gate_out`, ADR-0137) ──────────

/// Veredicto de la puerta de licencia (`docs/features/licensing-system.md`
/// "Ciclo de Vida" - "Salida").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum GateVerdict {
    /// Puede ejecutar sin restricciones.
    Allow,
    /// No puede ejecutar (huella no coincide, firma inválida, o heartbeat
    /// agotó su período de gracia).
    Deny,
    /// Puede seguir operando en lo ya activo, pero excedió un límite de su
    /// plan (ej. activaciones) -- necesita subir de tier, no un bloqueo de
    /// seguridad.
    UpgradeRequired,
}

/// El tipo de puerto `ExecutionGate` (ADR-0137 catálogo, enmienda
/// 2026-07-03): "Veredicto de ejecución (Allow/Deny/UpgradeRequired) + orden
/// de supresión de telemetría por tier".
///
/// **Guardarraíl ADR-0093 (estructural):** este struct SOLO tiene los cinco
/// campos de abajo -- ninguna credencial de bróker, IP de servidor live, ni
/// la clave de firma. El test
/// [`tests::execution_gate_json_never_leaks_secret_fields`] fija la lista
/// exacta de claves permitidas en el JSON serializado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExecutionGate {
    pub verdict: GateVerdict,
    pub suppress_work_telemetry: bool,
    pub tier: LicenseTier,
    /// Activaciones (máquinas distintas) contadas para el dueño de esta
    /// licencia en el momento de evaluar el gate.
    pub activations: i64,
    /// Explicación en texto plano del veredicto -- para mostrar al usuario
    /// o para el JSON del CLI de verificación (ADR-0142). Nunca contiene un
    /// secreto: es una frase fija por rama de [`derive_execution_gate`].
    pub reason: String,
}

/// Entradas ya evaluadas que [`derive_execution_gate`] combina. Cada campo
/// es el resultado de una función pura de este mismo módulo (o del conteo de
/// activaciones que hace la Shell) -- `derive_execution_gate` en sí no
/// vuelve a calcular nada, solo decide el veredicto a partir de hechos ya
/// establecidos.
#[derive(Debug, Clone)]
pub struct GateEvaluationInput<'a> {
    /// Resultado de [`verify_license_signature`] (`Ok(())` => `true`).
    pub signature_valid: bool,
    /// Resultado de [`hardware_matches`].
    pub hardware_match: bool,
    /// Resultado de [`evaluate_heartbeat_status`].
    pub heartbeat_status: HeartbeatStatus,
    pub tier: LicenseTier,
    /// Activaciones (máquinas distintas) ya persistidas para este dueño.
    pub activations: i64,
    pub plan_limits: &'a PlanLimits,
}

/// Deriva el veredicto final `ExecutionGate` combinando huella, firma,
/// heartbeat y cuota -- en ese orden de prioridad. Es una función PURA:
/// ninguna de sus ramas toca disco, red ni reloj (ADR-0039 -- el hot-path de
/// `execute`/`telemetry` puede llamarla sin violar el límite de latencia).
///
/// **Orden de prioridad (de mayor a menor severidad):**
/// 1. Firma inválida -> `Deny` (el archivo de licencia no es de fiar).
/// 2. Huella no coincide -> `Deny` (licencia de otra máquina).
/// 3. Heartbeat `Expired` -> `Deny` (agotó incluso el período de gracia).
/// 4. Activaciones exceden el límite del plan -> `UpgradeRequired` (la
///    licencia en sí es válida; el problema es de cuota, no de seguridad).
/// 5. Ninguna de las anteriores -> `Allow`.
///
/// La supresión de telemetría ([`should_suppress_work_telemetry`]) se
/// calcula en TODAS las ramas -- incluso denegada la ejecución, la Feature
/// sigue debiendo decidir si telemetría de trabajo se suprime o no.
pub fn derive_execution_gate(input: GateEvaluationInput<'_>) -> ExecutionGate {
    let suppress = should_suppress_work_telemetry(input.tier, input.heartbeat_status);

    if !input.signature_valid {
        return ExecutionGate {
            verdict: GateVerdict::Deny,
            suppress_work_telemetry: suppress,
            tier: input.tier,
            activations: input.activations,
            reason: "firma de licencia inválida".to_string(),
        };
    }

    if !input.hardware_match {
        return ExecutionGate {
            verdict: GateVerdict::Deny,
            suppress_work_telemetry: suppress,
            tier: input.tier,
            activations: input.activations,
            reason: "la huella de hardware no coincide con la licencia".to_string(),
        };
    }

    if input.heartbeat_status == HeartbeatStatus::Expired {
        return ExecutionGate {
            verdict: GateVerdict::Deny,
            suppress_work_telemetry: suppress,
            tier: input.tier,
            activations: input.activations,
            reason: "heartbeat expirado más allá del período de gracia".to_string(),
        };
    }

    if input.activations > input.plan_limits.max_activations {
        return ExecutionGate {
            verdict: GateVerdict::UpgradeRequired,
            suppress_work_telemetry: suppress,
            tier: input.tier,
            activations: input.activations,
            reason: "activaciones exceden el límite del plan vigente".to_string(),
        };
    }

    ExecutionGate {
        verdict: GateVerdict::Allow,
        suppress_work_telemetry: suppress,
        tier: input.tier,
        activations: input.activations,
        reason: "licencia válida dentro de los límites del plan".to_string(),
    }
}

// ── Hash de auditoría encadenado (row_version, ADR-0141) ─────────────────────

/// Calcula el `audit_hash` SHA-256 (hex, minúsculas) de una VERSIÓN de fila
/// de `licenses`, encadenado al `audit_hash` de la versión anterior de esa
/// misma fila (o [`crate::domain::audit_log::GENESIS_PREVIOUS_HASH`] si es
/// la versión génesis, `row_version == 1`).
///
/// Mismo patrón que `central_identity::compute_account_audit_hash`: la
/// cadena es POR LICENCIA (cada fila de `licenses` encadena sus propias
/// versiones), no una cadena global entre todas las licencias.
#[allow(clippy::too_many_arguments)]
pub fn compute_license_audit_hash(
    id: &str,
    updated_at_ns: i64,
    row_version: i64,
    previous_audit_hash: Option<&str>,
    owner_id: &str,
    node_id: &str,
    tier: LicenseTier,
    heartbeat_expires_at_ns: i64,
    compliance_status_id: &str,
    signature_hash: &str,
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
    push(owner_id);
    push(node_id);
    push(tier.as_str());
    push(&heartbeat_expires_at_ns.to_string());
    push(compliance_status_id);
    // La firma vigente también entra en la cadena de auditoría: si el
    // refresco re-firma la licencia (nueva `signature_hash` tras extender el
    // heartbeat), ese cambio queda registrado igual que cualquier otro campo.
    push(signature_hash);

    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    let digest = hasher.finalize();

    encode_hex(&digest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;

    /// Genera un par de claves Ed25519 de prueba y firma un payload de
    /// muestra -- helper compartido por los tests de firma de este módulo.
    fn signed_sample(payload: &LicensePayload<'_>) -> (String, String) {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let message = canonical_license_bytes(payload);
        let signature = signing_key.sign(&message);
        (encode_hex(&signature.to_bytes()), encode_hex(verifying_key.as_bytes()))
    }

    fn sample_payload<'a>(license_id: &'a str, owner_id: &'a str, node_id: &'a str) -> LicensePayload<'a> {
        LicensePayload {
            license_id,
            owner_id,
            node_id,
            tier: LicenseTier::Sovereign,
            issued_at_ns: 1_000,
            heartbeat_expires_at_ns: 2_000,
        }
    }

    // ── CRITERIO #2 (Orden §5): firma asimétrica -- válida vs. byte alterado ──

    #[test]
    fn verify_license_signature_accepts_a_valid_signature() {
        let payload = sample_payload("lic-1", "owner-1", "node-A");
        let (signature_hex, public_key_hex) = signed_sample(&payload);

        assert_eq!(verify_license_signature(&payload, &signature_hex, &public_key_hex), Ok(()));
    }

    /// CRITERIO DE CIERRE: un byte alterado del PAYLOAD (después de firmar)
    /// hace que la verificación falle -- discriminante: si el módulo no
    /// verificara de verdad la firma, esta prueba pasaría igual.
    #[test]
    fn verify_license_signature_rejects_tampered_payload() {
        let original = sample_payload("lic-1", "owner-1", "node-A");
        let (signature_hex, public_key_hex) = signed_sample(&original);

        // El atacante cambia el node_id (intenta reatar la licencia a OTRA máquina)
        // sin volver a firmar con la clave privada real.
        let tampered = sample_payload("lic-1", "owner-1", "node-B-ATTACKER");

        assert_eq!(
            verify_license_signature(&tampered, &signature_hex, &public_key_hex),
            Err(LicenseSignatureError::SignatureMismatch)
        );
    }

    /// CRITERIO DE CIERRE: un byte alterado de la FIRMA (misma longitud, hex
    /// válido, pero no la firma real) también se rechaza.
    #[test]
    fn verify_license_signature_rejects_tampered_signature_bytes() {
        let payload = sample_payload("lic-1", "owner-1", "node-A");
        let (mut signature_hex, public_key_hex) = signed_sample(&payload);

        // Voltea el primer carácter hexadecimal de la firma -- sigue siendo
        // hex válido de 64 bytes, pero ya no es LA firma correcta.
        let first_char = signature_hex.chars().next().expect("firma no vacía");
        let replacement = if first_char == 'a' { 'b' } else { 'a' };
        signature_hex.replace_range(0..1, &replacement.to_string());

        assert_eq!(
            verify_license_signature(&payload, &signature_hex, &public_key_hex),
            Err(LicenseSignatureError::SignatureMismatch)
        );
    }

    #[test]
    fn verify_license_signature_rejects_malformed_public_key_encoding() {
        let payload = sample_payload("lic-1", "owner-1", "node-A");
        let (signature_hex, _) = signed_sample(&payload);

        assert_eq!(
            verify_license_signature(&payload, &signature_hex, "no-es-hex-zz"),
            Err(LicenseSignatureError::InvalidPublicKeyEncoding)
        );
    }

    #[test]
    fn verify_license_signature_rejects_malformed_signature_encoding() {
        let payload = sample_payload("lic-1", "owner-1", "node-A");
        let (_, public_key_hex) = signed_sample(&payload);

        assert_eq!(
            verify_license_signature(&payload, "no-es-hex-zz", &public_key_hex),
            Err(LicenseSignatureError::InvalidSignatureEncoding)
        );
    }

    // ── CRITERIO #3 (Orden §5): huella no coincide -> Deny ──────────────────

    #[test]
    fn hardware_matches_true_for_identical_node_ids() {
        assert!(hardware_matches("node-A", "node-A"));
    }

    #[test]
    fn hardware_matches_false_for_different_node_ids() {
        assert!(!hardware_matches("node-A", "node-B"));
    }

    // ── CRITERIO #4 (Orden §5): heartbeat/gracia con reloj determinista ─────

    #[test]
    fn heartbeat_status_is_fresh_well_before_expiry() {
        let config = HeartbeatConfig { recheck_window_ns: 100, grace_period_ns: 50 };
        // now = 0, expira en 1000, ventana de recheck empieza en 900 -> Fresh.
        assert_eq!(evaluate_heartbeat_status(0, 1_000, &config), HeartbeatStatus::Fresh);
    }

    #[test]
    fn heartbeat_status_is_recheck_window_just_before_expiry() {
        let config = HeartbeatConfig { recheck_window_ns: 100, grace_period_ns: 50 };
        // now = 950 está dentro de [900, 1000) -> RecheckWindow.
        assert_eq!(evaluate_heartbeat_status(950, 1_000, &config), HeartbeatStatus::RecheckWindow);
    }

    #[test]
    fn heartbeat_status_is_within_grace_just_after_expiry() {
        let config = HeartbeatConfig { recheck_window_ns: 100, grace_period_ns: 50 };
        // now = 1020 está dentro de [1000, 1050) -> WithinGrace.
        assert_eq!(evaluate_heartbeat_status(1_020, 1_000, &config), HeartbeatStatus::WithinGrace);
    }

    #[test]
    fn heartbeat_status_is_expired_past_grace_period() {
        let config = HeartbeatConfig { recheck_window_ns: 100, grace_period_ns: 50 };
        // now = 1050 es exactamente el límite de gracia -> ya no es < grace_ends_at (1050) -> Expired.
        assert_eq!(evaluate_heartbeat_status(1_050, 1_000, &config), HeartbeatStatus::Expired);
    }

    // ── CRITERIO #5 (Orden §5): supresión de telemetría por tier (ADR-0143) ──

    #[test]
    fn sovereign_current_suppresses_work_telemetry() {
        assert!(should_suppress_work_telemetry(LicenseTier::Sovereign, HeartbeatStatus::Fresh));
        assert!(should_suppress_work_telemetry(LicenseTier::Sovereign, HeartbeatStatus::RecheckWindow));
        assert!(should_suppress_work_telemetry(LicenseTier::Sovereign, HeartbeatStatus::WithinGrace));
    }

    #[test]
    fn sovereign_expired_reactivates_telemetry() {
        assert!(!should_suppress_work_telemetry(LicenseTier::Sovereign, HeartbeatStatus::Expired));
    }

    #[test]
    fn explorer_never_suppresses_work_telemetry() {
        for status in [
            HeartbeatStatus::Fresh,
            HeartbeatStatus::RecheckWindow,
            HeartbeatStatus::WithinGrace,
            HeartbeatStatus::Expired,
        ] {
            assert!(!should_suppress_work_telemetry(LicenseTier::Explorer, status));
        }
    }

    // ── CRITERIO #3/#4 vía derive_execution_gate: Deny por huella/heartbeat ──

    fn permissive_plan_limits() -> PlanLimits {
        PlanLimits { max_activations: 100, features_enabled: vec![] }
    }

    #[test]
    fn derive_execution_gate_denies_on_hardware_mismatch() {
        let gate = derive_execution_gate(GateEvaluationInput {
            signature_valid: true,
            hardware_match: false,
            heartbeat_status: HeartbeatStatus::Fresh,
            tier: LicenseTier::Sovereign,
            activations: 1,
            plan_limits: &permissive_plan_limits(),
        });
        assert_eq!(gate.verdict, GateVerdict::Deny);
    }

    #[test]
    fn derive_execution_gate_denies_on_invalid_signature() {
        let gate = derive_execution_gate(GateEvaluationInput {
            signature_valid: false,
            hardware_match: true,
            heartbeat_status: HeartbeatStatus::Fresh,
            tier: LicenseTier::Sovereign,
            activations: 1,
            plan_limits: &permissive_plan_limits(),
        });
        assert_eq!(gate.verdict, GateVerdict::Deny);
    }

    #[test]
    fn derive_execution_gate_denies_on_expired_heartbeat() {
        let gate = derive_execution_gate(GateEvaluationInput {
            signature_valid: true,
            hardware_match: true,
            heartbeat_status: HeartbeatStatus::Expired,
            tier: LicenseTier::Sovereign,
            activations: 1,
            plan_limits: &permissive_plan_limits(),
        });
        assert_eq!(gate.verdict, GateVerdict::Deny);
    }

    #[test]
    fn derive_execution_gate_allows_when_everything_checks_out() {
        let gate = derive_execution_gate(GateEvaluationInput {
            signature_valid: true,
            hardware_match: true,
            heartbeat_status: HeartbeatStatus::Fresh,
            tier: LicenseTier::Sovereign,
            activations: 1,
            plan_limits: &permissive_plan_limits(),
        });
        assert_eq!(gate.verdict, GateVerdict::Allow);
    }

    // ── CRITERIO #6 (Orden §5): UpgradeRequired por cuota excedida ──────────

    /// CRITERIO DE CIERRE: con un límite de plan de 1 activación y 2
    /// activaciones ya contadas, el veredicto debe ser `UpgradeRequired` --
    /// no `Deny` (la licencia en sí es válida) ni `Allow` (excede la cuota).
    #[test]
    fn derive_execution_gate_requires_upgrade_when_activations_exceed_plan_limit() {
        let limits = PlanLimits { max_activations: 1, features_enabled: vec![] };
        let gate = derive_execution_gate(GateEvaluationInput {
            signature_valid: true,
            hardware_match: true,
            heartbeat_status: HeartbeatStatus::Fresh,
            tier: LicenseTier::Explorer,
            activations: 2,
            plan_limits: &limits,
        });
        assert_eq!(gate.verdict, GateVerdict::UpgradeRequired);
    }

    #[test]
    fn derive_execution_gate_allows_when_activations_equal_the_limit() {
        let limits = PlanLimits { max_activations: 3, features_enabled: vec![] };
        let gate = derive_execution_gate(GateEvaluationInput {
            signature_valid: true,
            hardware_match: true,
            heartbeat_status: HeartbeatStatus::Fresh,
            tier: LicenseTier::Sovereign,
            activations: 3,
            plan_limits: &limits,
        });
        assert_eq!(gate.verdict, GateVerdict::Allow, "activaciones == límite no debe exigir upgrade");
    }

    // ── CRITERIO #8 (Orden §5): guardarraíl ADR-0093 -- sin secretos ────────

    /// CRITERIO DE CIERRE (guardarraíl ADR-0093): el JSON serializado de
    /// `ExecutionGate` contiene EXACTAMENTE estas cinco claves -- ninguna
    /// credencial de bróker, IP de servidor live, ni clave de firma.
    #[test]
    fn execution_gate_json_never_leaks_secret_fields() {
        let gate = ExecutionGate {
            verdict: GateVerdict::Allow,
            suppress_work_telemetry: true,
            tier: LicenseTier::Sovereign,
            activations: 1,
            reason: "licencia válida dentro de los límites del plan".to_string(),
        };

        let json = serde_json::to_value(&gate).expect("ExecutionGate debe serializar a JSON");
        let object = json.as_object().expect("el JSON de ExecutionGate debe ser un objeto");

        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();

        assert_eq!(
            keys,
            vec!["activations", "reason", "suppress_work_telemetry", "tier", "verdict"],
            "ExecutionGate solo puede exponer estas cinco claves -- ninguna credencial de bróker, \
             IP de servidor live, ni clave de firma (ADR-0093)"
        );

        let json_string = json.to_string();
        for forbidden in ["password", "api_key", "api-key", "broker_secret", "private_key", "signing_key", "192.168.", "10.0.0."] {
            assert!(
                !json_string.to_lowercase().contains(forbidden),
                "el JSON de ExecutionGate no debe contener '{forbidden}'"
            );
        }
    }

    // ── CRITERIO #9 (Orden §5): hot-path sin I/O de red (inspección/estructura) ──

    /// CRITERIO DE CIERRE (ADR-0039, inspección estructural): `derive_execution_gate`
    /// es una función SÍNCRONA (no `async`) que no recibe ningún handle de
    /// red ni de base de datos -- solo puede operar sobre los valores ya
    /// evaluados en `GateEvaluationInput`. Esta prueba, al poder llamarse
    /// dentro de un `#[test]` normal (sin runtime async, sin pool, sin
    /// mock de red), es en sí misma la demostración de que el método no
    /// depende de I/O: si dependiera de I/O de red, no compilaría ni
    /// correría aquí.
    #[test]
    fn derive_execution_gate_has_no_network_io_dependency() {
        let gate = derive_execution_gate(GateEvaluationInput {
            signature_valid: true,
            hardware_match: true,
            heartbeat_status: HeartbeatStatus::Fresh,
            tier: LicenseTier::Explorer,
            activations: 1,
            plan_limits: &permissive_plan_limits(),
        });
        assert_eq!(gate.verdict, GateVerdict::Allow);
    }

    // ── Tier: round-trip de representación en texto ─────────────────────────

    #[test]
    fn license_tier_round_trips_through_its_string_representation() {
        for tier in [LicenseTier::Sovereign, LicenseTier::Explorer] {
            assert_eq!(LicenseTier::from_str_value(tier.as_str()), Some(tier));
        }
        assert_eq!(LicenseTier::from_str_value("UNKNOWN"), None);
    }

    // ── Hash de auditoría encadenado ─────────────────────────────────────────

    #[test]
    fn compute_license_audit_hash_is_deterministic() {
        let hash_a = compute_license_audit_hash(
            "id-1", 1_000, 1, None, "owner-1", "node-1", LicenseTier::Sovereign, 2_000, "ACTIVE", "sig-hex-1",
        );
        let hash_b = compute_license_audit_hash(
            "id-1", 1_000, 1, None, "owner-1", "node-1", LicenseTier::Sovereign, 2_000, "ACTIVE", "sig-hex-1",
        );
        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn compute_license_audit_hash_changes_when_compliance_status_changes() {
        let active = compute_license_audit_hash(
            "id-1", 2_000, 2, Some("prev"), "owner-1", "node-1", LicenseTier::Sovereign, 2_000, "ACTIVE", "sig-hex-1",
        );
        let expired = compute_license_audit_hash(
            "id-1", 2_000, 2, Some("prev"), "owner-1", "node-1", LicenseTier::Sovereign, 2_000, "EXPIRED", "sig-hex-1",
        );
        assert_ne!(active, expired);
    }

    #[test]
    fn compute_license_audit_hash_changes_when_signature_hash_changes() {
        let first = compute_license_audit_hash(
            "id-1", 2_000, 2, Some("prev"), "owner-1", "node-1", LicenseTier::Sovereign, 2_000, "ACTIVE", "sig-hex-1",
        );
        let resigned = compute_license_audit_hash(
            "id-1", 2_000, 2, Some("prev"), "owner-1", "node-1", LicenseTier::Sovereign, 2_000, "ACTIVE", "sig-hex-2",
        );
        assert_ne!(first, resigned, "re-firmar la licencia debe cambiar el hash de auditoría");
    }

    // ── heartbeat_status_to_compliance_status_id ─────────────────────────────

    #[test]
    fn heartbeat_status_maps_to_expected_compliance_status_id() {
        assert_eq!(heartbeat_status_to_compliance_status_id(HeartbeatStatus::Fresh), "ACTIVE");
        assert_eq!(heartbeat_status_to_compliance_status_id(HeartbeatStatus::RecheckWindow), "ACTIVE");
        assert_eq!(heartbeat_status_to_compliance_status_id(HeartbeatStatus::WithinGrace), "GRACE");
        assert_eq!(heartbeat_status_to_compliance_status_id(HeartbeatStatus::Expired), "EXPIRED");
    }
}
