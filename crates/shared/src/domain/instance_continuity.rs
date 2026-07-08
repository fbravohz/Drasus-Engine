//! [CORE] Lógica pura de Instance Continuity (`docs/features/instance-continuity.md`,
//! ADR-0146 -- cimiento #11 rector, ADR-0093, ADR-0143, ADR-0002, STORY-039).
//!
//! Sin I/O, sin reloj de sistema, sin aleatoriedad sin semilla
//! (ADR-0002/0004). Cinco piezas de lógica pura:
//! - [`derive_encryption_key`]: deriva la clave de cifrado AES-256 desde el
//!   secreto maestro del usuario, vía un KDF estándar (Argon2id). La clave
//!   y el secreto maestro NUNCA se persisten ni salen de esta función hacia
//!   ningún tipo de puerto.
//! - [`generate_nonce`]: el nonce de AES-GCM, generado con un RNG SEMBRADO
//!   e inyectado (`seed: u64`) -- mismo patrón que el ruido gaussiano de
//!   `data_aggregation::apply_differential_privacy` (#9). Determinista en
//!   tests, alimentado con una semilla realmente aleatoria en producción
//!   (la Shell decide de dónde sale esa semilla -- el Core nunca lee
//!   entropía del sistema).
//! - [`encrypt_backup_blob`] / [`decrypt_backup_blob`]: cifrado/descifrado
//!   autenticado AES-256-GCM. El tag de autenticación de GCM detecta
//!   CUALQUIER manipulación del ciphertext -- alterar un solo byte hace
//!   fallar el descifrado con un error tipado, nunca devuelve basura
//!   silenciosa.
//! - [`compute_backup_delta`]: filtra del snapshot crudo cualquier campo
//!   que sea un secreto de bróker o una IP de servidor live -- las MISMAS
//!   clases de secreto que se excluyen de la telemetría (ADR-0093). Este
//!   filtro corre ANTES de que el contenido llegue al cifrado.
//! - [`decide_custody_claim`] / [`is_current_titular`]: el gate de
//!   titularidad exclusiva -- concurrencia optimista a nivel de INSTANCIA
//!   COMPLETA (`custody_epoch`), no a nivel de una fila de negocio
//!   cualquiera. Puro y determinista dado el estado actual.
//!
//! Más los tipos de puerto público (ADR-0137): [`EncryptedBackupBlob`]
//! (`backup_blob_out`) y [`CustodyStatusOut`] (`custody_status_out`).

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use argon2::Argon2;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;
use sha2::{Digest, Sha256};

// ── Utilidades de hexadecimal (mismo patrón que el resto del substrato) ─────

/// Codifica bytes crudos a su representación hexadecimal en minúsculas.
pub(crate) fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Motivo por el que una cadena hexadecimal no pudo decodificarse a bytes.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HexDecodeError {
    #[error("la cadena hexadecimal tiene longitud impar")]
    OddLength,
    #[error("la cadena contiene un carácter que no es hexadecimal válido")]
    InvalidDigit,
}

/// Decodifica una cadena hexadecimal (minúsculas o mayúsculas) a bytes.
/// Inversa de [`encode_hex`].
pub(crate) fn decode_hex(hex_str: &str) -> Result<Vec<u8>, HexDecodeError> {
    if !hex_str.len().is_multiple_of(2) {
        return Err(HexDecodeError::OddLength);
    }
    (0..hex_str.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_str[i..i + 2], 16).map_err(|_| HexDecodeError::InvalidDigit))
        .collect()
}

/// SHA-256 hex de un buffer de bytes -- usado para el `blob_hash` que
/// persiste el registro de respaldos (nunca el ciphertext en sí).
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    encode_hex(&hasher.finalize())
}

// ── Derivación de clave (KDF, ADR-0093) ─────────────────────────────────────

/// Deriva la clave de cifrado AES-256 (32 bytes) desde el **secreto
/// maestro** del usuario, usando Argon2id (KDF estándar resistente a
/// fuerza bruta sobre una frase de paso de baja entropía humana).
///
/// El *salt* es el SHA-256 de `owner_id` -- determinista (la MISMA cuenta
/// siempre deriva la MISMA clave desde el MISMO secreto maestro, condición
/// necesaria para que otra máquina activada pueda descifrar el mismo blob)
/// y **NO secreto** (`owner_id` ya es un identificador público de cuenta,
/// igual que en `AccountIdentity`) -- no hace falta persistirlo por
/// separado.
///
/// **Esta función NUNCA persiste ni expone la clave devuelta** -- quien la
/// llama es responsable de usarla solo en memoria, dentro de
/// [`encrypt_backup_blob`]/[`decrypt_backup_blob`], y de descartarla
/// después (ADR-0093: "la clave y el secreto maestro NUNCA salen de la
/// máquina del usuario").
pub fn derive_encryption_key(master_secret: &str, owner_id: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(owner_id.as_bytes());
    let salt = hasher.finalize();

    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(master_secret.as_bytes(), &salt, &mut key)
        // Argon2 solo falla si el salt o la salida violan sus límites
        // documentados (salt demasiado corto, clave de salida demasiado
        // larga). El salt aquí SIEMPRE mide 32 bytes (SHA-256) y la clave
        // de salida SIEMPRE mide 32 bytes (AES-256) -- ambos tamaños fijos
        // están siempre dentro de los límites de Argon2, así que esto
        // nunca falla en la práctica.
        .expect("salt de 32 bytes y clave de salida de 32 bytes siempre son válidos para Argon2");
    key
}

// ── Nonce de AES-GCM (RNG sembrado e inyectado, ADR-0002) ───────────────────

/// Genera el nonce de 12 bytes (96 bits) que exige AES-GCM, con un RNG
/// **sembrado** (`seed: u64`) -- mismo patrón que
/// [`super::data_aggregation::apply_differential_privacy`] (RNG sembrado
/// para el ruido de privacidad diferencial).
///
/// ## Por qué el nonce se siembra (y nunca `rand::thread_rng()`)
///
/// Un Core FCIS es puro: mismo input -> mismo output, bit a bit. Si esta
/// función leyera entropía del sistema, la MISMA llamada con los MISMOS
/// argumentos produciría un nonce distinto en cada ejecución -- imposible
/// de probar y de auditar. Sembrando con `seed_from_u64(seed)`, la MISMA
/// semilla produce SIEMPRE el MISMO nonce. En **producción**, la Shell es
/// quien decide de dónde sale la semilla (una fuente de entropía real,
/// nunca del Core) -- este módulo solo consume la semilla ya resuelta.
///
/// ## El nonce NUNCA se reutiliza con la misma clave
///
/// AES-GCM es catastróficamente inseguro si el MISMO par (clave, nonce) se
/// usa dos veces para cifrar mensajes distintos -- filtra información que
/// permite recuperar la clave de autenticación. Por eso la Shell de
/// producción SIEMPRE deriva una semilla fresca (de una fuente de entropía
/// real) en cada snapshot -- nunca reutiliza la semilla del snapshot
/// anterior. El nonce en sí no es secreto (se persiste junto al blob,
/// [`EncryptedBackupBlob::nonce_hex`]) -- lo que nunca se repite es el PAR
/// (clave, nonce).
pub fn generate_nonce(seed: u64) -> [u8; 12] {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut nonce = [0u8; 12];
    rng.fill(&mut nonce);
    nonce
}

// ── Cifrado/descifrado autenticado AES-256-GCM (ADR-0093) ───────────────────

/// Motivo por el que el cifrado/descifrado del blob de respaldo falló.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EncryptionError {
    /// La autenticación GCM falló: el ciphertext o el tag fueron
    /// manipulados, o la clave/nonce usados para descifrar no son los que
    /// cifraron el blob originalmente. AES-GCM es cifrado AUTENTICADO --
    /// nunca devuelve un plaintext "aproximado" o corrupto en silencio,
    /// siempre falla limpio ante cualquier alteración.
    #[error("la autenticación del blob cifrado falló -- el contenido fue manipulado, o la clave/nonce no coinciden")]
    AuthenticationFailed,
    /// El campo `nonce_hex`/`ciphertext_hex` del blob no es hexadecimal
    /// válido (blob corrupto o mal formado, nunca debería ocurrir con un
    /// blob que salió de [`encrypt_backup_blob`]).
    #[error("el blob de respaldo está corrupto: {0}")]
    MalformedBlob(#[from] HexDecodeError),
}

/// El blob de respaldo cifrado -- el tipo de puerto `backup_blob_out`
/// (ADR-0137, catálogo, enmienda ADR-0146): snapshot cifrado listo para
/// subir al almacén de objetos del proveedor (adaptador diferido).
///
/// **Guardarraíl ADR-0093 (estructural):** este struct SOLO expone el
/// ciphertext (que YA incluye el tag de autenticación GCM, indistinguible
/// de ruido para quien no tiene la clave) y el nonce (no secreto, se
/// necesita para descifrar). NUNCA la clave ni el secreto maestro -- el
/// test [`tests::encrypted_backup_blob_json_never_leaks_key_or_secret`]
/// fija la lista exacta de claves permitidas en el JSON serializado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EncryptedBackupBlob {
    /// El ciphertext (con el tag GCM anexado), en hexadecimal.
    pub ciphertext_hex: String,
    /// El nonce usado para cifrar, en hexadecimal -- NO es secreto (se
    /// necesita junto con la clave para descifrar).
    pub nonce_hex: String,
}

/// Cifra `plaintext` con AES-256-GCM usando `key` (derivada por
/// [`derive_encryption_key`]) y `nonce` (generado por [`generate_nonce`] o
/// una fuente de entropía real en producción).
///
/// Determinista dado sus tres argumentos (ADR-0002): el MISMO plaintext +
/// la MISMA clave + el MISMO nonce producen SIEMPRE el MISMO ciphertext --
/// verificable en tests con un nonce sembrado.
pub fn encrypt_backup_blob(
    plaintext: &[u8],
    key: &[u8; 32],
    nonce: &[u8; 12],
) -> Result<EncryptedBackupBlob, EncryptionError> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce_ga = Nonce::from_slice(nonce);

    // `aead::Error` es opaco a propósito (RustCrypto no expone el motivo
    // exacto del fallo para no filtrar información al atacante) -- se
    // traduce al error tipado del Core.
    let ciphertext = cipher
        .encrypt(nonce_ga, plaintext)
        .map_err(|_| EncryptionError::AuthenticationFailed)?;

    Ok(EncryptedBackupBlob {
        ciphertext_hex: encode_hex(&ciphertext),
        nonce_hex: encode_hex(nonce),
    })
}

/// Descifra `blob` con `key` (la MISMA clave que lo cifró). El nonce viaja
/// DENTRO del blob (`nonce_hex`) -- no es secreto, así que no hace falta
/// pasarlo por separado.
///
/// **Criterio de cierre (regla obligatoria #1, ADR-0093):** si el
/// ciphertext o el tag GCM fueron alterados (aunque sea un solo byte), o
/// si `key` no es la clave correcta, esta función devuelve
/// `Err(EncryptionError::AuthenticationFailed)` -- NUNCA un plaintext
/// corrupto o parcial. La autenticación GCM se verifica ANTES de exponer
/// cualquier byte del plaintext.
pub fn decrypt_backup_blob(blob: &EncryptedBackupBlob, key: &[u8; 32]) -> Result<Vec<u8>, EncryptionError> {
    let ciphertext = decode_hex(&blob.ciphertext_hex)?;
    let nonce_bytes = decode_hex(&blob.nonce_hex)?;

    // Un nonce de longitud distinta a 12 bytes no puede ser el nonce que
    // AES-GCM exige -- blob corrupto, se trata como fallo de autenticación
    // (nunca se intenta descifrar con un nonce de tamaño incorrecto).
    if nonce_bytes.len() != 12 {
        return Err(EncryptionError::AuthenticationFailed);
    }

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext.as_slice())
        .map_err(|_| EncryptionError::AuthenticationFailed)
}

// ── Delta a respaldar: excluye secretos de bróker / IPs live (ADR-0093) ────

/// Un campo candidato del snapshot a respaldar -- clave lógica + valor en
/// texto. La cáscara recolecta estos campos desde el estado local (tablas,
/// configuración); este módulo NUNCA los lee de disco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupField {
    pub key: String,
    pub value: String,
}

/// Subcadenas de clave que marcan un campo como secreto de bróker o IP de
/// servidor live -- las MISMAS clases de secreto que ADR-0093 excluye de
/// la telemetría de trabajo. La comparación es *case-insensitive*.
const EXCLUDED_BACKUP_KEY_SUBSTRINGS: [&str; 6] = [
    "broker_credential",
    "broker_password",
    "investor_password",
    "broker_api_key",
    "live_server_ip",
    "live_ip",
];

/// Decide si un campo, por su CLAVE, pertenece a una de las clases de
/// secreto excluidas del respaldo.
fn is_excluded_from_backup(key: &str) -> bool {
    let lower = key.to_lowercase();
    EXCLUDED_BACKUP_KEY_SUBSTRINGS.iter().any(|pattern| lower.contains(pattern))
}

/// Calcula el delta a respaldar: los campos de `fields` que NO son
/// secretos de bróker ni IPs de servidor live (regla obligatoria #3,
/// ADR-0093). El resultado es lo único que puede llegar a
/// [`encrypt_backup_blob`] -- un campo excluido aquí NUNCA llega, ni
/// siquiera cifrado, al blob de respaldo.
///
/// Preserva el orden de `fields` (no reordena) -- determinista dado el
/// mismo input.
pub fn compute_backup_delta(fields: &[BackupField]) -> Vec<BackupField> {
    fields.iter().filter(|field| !is_excluded_from_backup(&field.key)).cloned().collect()
}

/// Serializa el delta ya filtrado a un buffer canónico de bytes, listo
/// para cifrar. Ordena por clave ANTES de concatenar (determinismo:
/// [`compute_backup_delta`] preserva el orden de entrada, pero dos
/// llamadores que recolectan los mismos campos en orden distinto deben
/// producir el MISMO buffer). El separador `\u{1F}` (Unit Separator ASCII)
/// es el mismo patrón que `audit_log::canonical_bytes`.
pub fn canonical_delta_bytes(fields: &[BackupField]) -> Vec<u8> {
    let mut sorted: Vec<&BackupField> = fields.iter().collect();
    sorted.sort_by(|a, b| a.key.cmp(&b.key));

    const SEP: char = '\u{1F}';
    let mut buffer = String::new();
    for field in sorted {
        buffer.push_str(&field.key);
        buffer.push('=');
        buffer.push_str(&field.value);
        buffer.push(SEP);
    }
    buffer.into_bytes()
}

// ── Gate de titularidad exclusiva (custody_epoch, ADR-0146) ────────────────

/// El estado de custodia vigente de una cuenta: qué máquina es la titular
/// escritora y en qué epoch está (concurrencia optimista a nivel de
/// INSTANCIA COMPLETA, no de una fila de negocio cualquiera).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustodyState {
    pub owner_id: String,
    pub titular_node_id: String,
    pub custody_epoch: i64,
}

/// El reclamo de titularidad partió de un `custody_epoch` que ya no es el
/// vigente -- otra máquina reclamó primero. La máquina que recibe este
/// error queda BLOQUEADA: NUNCA escribe la cadena de auditoría en paralelo
/// con la titular real (regla obligatoria #4, ADR-0146).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CustodyClaimError {
    #[error(
        "conflicto de custodia: el dueño '{owner_id}' ya no está en el epoch {expected_epoch} \
         -- otra máquina reclamó la titularidad primero"
    )]
    CustodyConflict { owner_id: String, expected_epoch: i64 },
}

/// Decide si un reclamo de titularidad es válido, dado el estado ACTUAL y
/// el epoch que el reclamante CREE vigente (`expected_epoch`).
///
/// Puro y determinista (ADR-0002): si `expected_epoch` coincide con
/// `current.custody_epoch`, el reclamo gana y produce el estado siguiente
/// (`titular_node_id` = el reclamante, `custody_epoch` + 1). Si NO
/// coincide -- otra máquina ya avanzó el epoch -- devuelve
/// [`CustodyClaimError::CustodyConflict`] SIN producir ningún estado
/// nuevo. Esta función NO toca I/O: la Shell (`persistence::instance_continuity`)
/// es quien aplica la MISMA guarda contra la fila real en SQLite (`UPDATE
/// ... WHERE custody_epoch = ?`), para que dos escritores concurrentes de
/// verdad nunca ganen ambos.
pub fn decide_custody_claim(
    current: &CustodyState,
    claiming_node_id: &str,
    expected_epoch: i64,
) -> Result<CustodyState, CustodyClaimError> {
    if current.custody_epoch != expected_epoch {
        return Err(CustodyClaimError::CustodyConflict {
            owner_id: current.owner_id.clone(),
            expected_epoch,
        });
    }

    Ok(CustodyState {
        owner_id: current.owner_id.clone(),
        titular_node_id: claiming_node_id.to_string(),
        custody_epoch: current.custody_epoch + 1,
    })
}

/// Verifica si `node_id` es la máquina titular vigente según `state`.
///
/// Pura y determinista: no consulta nada, solo compara el `node_id` dado
/// contra `state.titular_node_id` (regla obligatoria #4, ADR-0146: "`true`
/// solo para el `node_id` titular vigente").
pub fn is_current_titular(node_id: &str, state: &CustodyState) -> bool {
    state.titular_node_id == node_id
}

/// El tipo de puerto `custody_status_out` (ADR-0137, catálogo, enmienda
/// ADR-0146): ¿esta máquina es la titular vigente de la cadena de
/// auditoría de la cuenta? Consumido por el arranque de la app y por
/// `licensing-system` (#2, downstream -- este cimiento NO importa
/// `licensing-system`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CustodyStatusOut {
    pub owner_id: String,
    pub node_id: String,
    pub is_titular: bool,
    pub custody_epoch: i64,
}

// ── Hash de auditoría encadenado (ambas tablas, ADR-0141) ───────────────────

/// Calcula el `audit_hash` SHA-256 (hex) de una fila de `instance_backups`,
/// encadenado a la fila anterior en la cadena GLOBAL (APPEND-ONLY,
/// `event_sequence_id`) -- mismo patrón que
/// `enriched_domain_events::compute_event_audit_hash`.
#[allow(clippy::too_many_arguments)]
pub fn compute_backup_audit_hash(
    id: &str,
    created_at_ns: i64,
    event_sequence_id: i64,
    previous_audit_hash: &str,
    owner_id: &str,
    institutional_tag: &str,
    node_id: &str,
    snapshot_at_ns: i64,
    blob_hash: &str,
    blob_size_bytes: i64,
    nonce_hex: &str,
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
    push(&snapshot_at_ns.to_string());
    push(blob_hash);
    push(&blob_size_bytes.to_string());
    push(nonce_hex);

    sha256_hex(buffer.as_bytes())
}

/// Calcula el `audit_hash` SHA-256 (hex) de una VERSIÓN de fila de
/// `custody_state`, encadenado al `audit_hash` de la versión anterior de
/// esa MISMA fila (o [`crate::domain::audit_log::GENESIS_PREVIOUS_HASH`]
/// si es la versión génesis, `custody_epoch == 1`) -- mismo patrón que
/// `central_identity::compute_account_audit_hash` (cadena POR CUENTA, no
/// global).
pub fn compute_custody_audit_hash(
    id: &str,
    updated_at_ns: i64,
    custody_epoch: i64,
    previous_audit_hash: Option<&str>,
    owner_id: &str,
    institutional_tag: &str,
    titular_node_id: &str,
) -> String {
    const SEP: char = '\u{1F}';

    let mut buffer = String::new();
    let mut push = |field: &str| {
        buffer.push_str(field);
        buffer.push(SEP);
    };

    push(id);
    push(&updated_at_ns.to_string());
    push(&custody_epoch.to_string());
    push(previous_audit_hash.unwrap_or(crate::domain::audit_log::GENESIS_PREVIOUS_HASH));
    push(owner_id);
    push(institutional_tag);
    push(titular_node_id);

    sha256_hex(buffer.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Criterio #2 (Orden §5): round-trip de cifrado + autenticación GCM ──

    /// CRITERIO DE CIERRE: `encrypt_backup_blob` -> `decrypt_backup_blob`
    /// con la MISMA clave recupera el plaintext EXACTO.
    #[test]
    fn round_trip_recovers_the_exact_plaintext() {
        let key = derive_encryption_key("correct horse battery staple", "owner-1");
        let nonce = generate_nonce(42);
        let plaintext = b"snapshot-bytes-del-usuario";

        let blob = encrypt_backup_blob(plaintext, &key, &nonce).expect("cifrar debe tener éxito");
        let decrypted = decrypt_backup_blob(&blob, &key).expect("descifrar con la clave correcta debe tener éxito");

        assert_eq!(decrypted, plaintext, "el round-trip debe recuperar el plaintext exacto");
    }

    /// CRITERIO DE CIERRE (regla obligatoria #1, ADR-0093): alterar UN SOLO
    /// byte del ciphertext hace FALLAR el descifrado -- la autenticación
    /// GCM detecta la manipulación, nunca devuelve basura.
    #[test]
    fn tampering_a_single_byte_of_the_ciphertext_fails_authentication() {
        let key = derive_encryption_key("correct horse battery staple", "owner-1");
        let nonce = generate_nonce(42);
        let plaintext = b"snapshot-bytes-del-usuario";

        let blob = encrypt_backup_blob(plaintext, &key, &nonce).expect("cifrar debe tener éxito");

        // Altera el primer byte del ciphertext hexadecimal (cambia un
        // nibble hex por otro distinto).
        let mut tampered_hex = blob.ciphertext_hex.clone();
        let first_char = tampered_hex.chars().next().expect("el ciphertext no debe estar vacío");
        let replacement = if first_char == 'a' { 'b' } else { 'a' };
        tampered_hex.replace_range(0..1, &replacement.to_string());

        let tampered_blob = EncryptedBackupBlob { ciphertext_hex: tampered_hex, nonce_hex: blob.nonce_hex };

        let result = decrypt_backup_blob(&tampered_blob, &key);
        assert_eq!(
            result,
            Err(EncryptionError::AuthenticationFailed),
            "un ciphertext manipulado debe fallar la autenticación, nunca devolver basura silenciosa"
        );
    }

    /// Descifrar con una clave INCORRECTA también debe fallar la
    /// autenticación (no solo la manipulación directa del ciphertext).
    #[test]
    fn decrypting_with_the_wrong_key_fails_authentication() {
        let key = derive_encryption_key("correct horse battery staple", "owner-1");
        let wrong_key = derive_encryption_key("una frase completamente distinta", "owner-1");
        let nonce = generate_nonce(42);

        let blob = encrypt_backup_blob(b"contenido", &key, &nonce).expect("cifrar debe tener éxito");

        assert_eq!(decrypt_backup_blob(&blob, &wrong_key), Err(EncryptionError::AuthenticationFailed));
    }

    // ── Criterio #3 (Orden §5): nonce sembrado, determinista, no reutilizado ─

    /// CRITERIO DE CIERRE: la MISMA semilla produce SIEMPRE el MISMO
    /// nonce -- reproducibilidad (patrón #9).
    #[test]
    fn generate_nonce_is_deterministic_for_the_same_seed() {
        assert_eq!(generate_nonce(42), generate_nonce(42));
    }

    /// Semillas DISTINTAS producen nonces distintos -- confirma que la
    /// semilla realmente participa (no es un parámetro decorativo).
    #[test]
    fn generate_nonce_differs_across_different_seeds() {
        assert_ne!(generate_nonce(1), generate_nonce(2));
    }

    /// CRITERIO DE CIERRE: mismo plaintext + misma clave + mismo nonce
    /// sembrado -> mismo ciphertext (reproducible); nonces distintos ->
    /// ciphertexts distintos.
    #[test]
    fn same_seed_same_ciphertext_different_seed_different_ciphertext() {
        let key = derive_encryption_key("secreto-maestro", "owner-1");
        let plaintext = b"contenido-identico";

        let nonce_a = generate_nonce(7);
        let nonce_b = generate_nonce(7);
        let nonce_c = generate_nonce(8);

        let blob_a = encrypt_backup_blob(plaintext, &key, &nonce_a).expect("cifrar A");
        let blob_b = encrypt_backup_blob(plaintext, &key, &nonce_b).expect("cifrar B");
        let blob_c = encrypt_backup_blob(plaintext, &key, &nonce_c).expect("cifrar C");

        assert_eq!(blob_a, blob_b, "mismo plaintext + misma clave + mismo nonce sembrado -> mismo ciphertext");
        assert_ne!(blob_a.ciphertext_hex, blob_c.ciphertext_hex, "un nonce distinto debe producir un ciphertext distinto");
    }

    // ── Criterio #4 (Orden §5, ADR-0093): guardarraíl estructural ───────────

    /// CRITERIO DE CIERRE (guardarraíl ADR-0093): el JSON serializado de
    /// `EncryptedBackupBlob` contiene EXACTAMENTE estas dos claves -- ni la
    /// clave de cifrado, ni el secreto maestro, ni ningún otro campo.
    #[test]
    fn encrypted_backup_blob_json_never_leaks_key_or_secret() {
        let key = derive_encryption_key("correct horse battery staple", "owner-1");
        let nonce = generate_nonce(1);
        let blob = encrypt_backup_blob(b"contenido", &key, &nonce).expect("cifrar debe tener éxito");

        let json = serde_json::to_value(&blob).expect("EncryptedBackupBlob debe serializar a JSON");
        let object = json.as_object().expect("debe ser un objeto JSON");
        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();

        assert_eq!(
            keys,
            vec!["ciphertext_hex", "nonce_hex"],
            "EncryptedBackupBlob solo puede exponer estas dos claves -- nunca la clave de cifrado ni el secreto maestro"
        );

        let json_string = json.to_string();
        for forbidden in ["correct horse battery staple", "master_secret", "encryption_key", "password"] {
            assert!(!json_string.to_lowercase().contains(&forbidden.to_lowercase()), "el JSON no debe contener '{forbidden}'");
        }
    }

    /// CRITERIO DE CIERRE (guardarraíl ADR-0093): el JSON de
    /// `CustodyStatusOut` tampoco porta secretos ni claves.
    #[test]
    fn custody_status_out_json_never_leaks_secrets() {
        let status = CustodyStatusOut {
            owner_id: "owner-1".to_string(),
            node_id: "node-A".to_string(),
            is_titular: true,
            custody_epoch: 3,
        };
        let json = serde_json::to_value(&status).expect("CustodyStatusOut debe serializar a JSON");
        let object = json.as_object().expect("debe ser un objeto JSON");
        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, vec!["custody_epoch", "is_titular", "node_id", "owner_id"]);
    }

    // ── Criterio #5 (Orden §5): el delta excluye secretos ───────────────────

    /// CRITERIO DE CIERRE: campos cuya clave marca un secreto de bróker o
    /// una IP de servidor live se EXCLUYEN del delta; campos normales se
    /// conservan.
    #[test]
    fn compute_backup_delta_excludes_broker_secrets_and_live_ips() {
        let fields = vec![
            BackupField { key: "account_balance_e8".to_string(), value: "150000000000".to_string() },
            BackupField { key: "broker_credential_secret".to_string(), value: "top-secret".to_string() },
            BackupField { key: "investor_password".to_string(), value: "hunter2".to_string() },
            BackupField { key: "live_server_ip".to_string(), value: "203.0.113.5".to_string() },
            BackupField { key: "strategy_name".to_string(), value: "mean-reversion-1".to_string() },
        ];

        let delta = compute_backup_delta(&fields);
        let kept_keys: Vec<&str> = delta.iter().map(|f| f.key.as_str()).collect();

        assert_eq!(kept_keys, vec!["account_balance_e8", "strategy_name"], "solo deben sobrevivir los campos no-secretos");

        let delta_bytes = canonical_delta_bytes(&delta);
        let delta_text = String::from_utf8(delta_bytes).expect("el delta canónico debe ser UTF-8 válido");
        for forbidden in ["top-secret", "hunter2", "203.0.113.5"] {
            assert!(!delta_text.contains(forbidden), "el delta canónico no debe contener el secreto '{forbidden}'");
        }
    }

    /// El delta canónico es determinista: el mismo conjunto de campos, en
    /// distinto orden de entrada, produce el MISMO buffer (se ordena por
    /// clave antes de concatenar).
    #[test]
    fn canonical_delta_bytes_is_order_independent() {
        let a = vec![
            BackupField { key: "zeta".to_string(), value: "1".to_string() },
            BackupField { key: "alpha".to_string(), value: "2".to_string() },
        ];
        let b = vec![
            BackupField { key: "alpha".to_string(), value: "2".to_string() },
            BackupField { key: "zeta".to_string(), value: "1".to_string() },
        ];

        assert_eq!(canonical_delta_bytes(&a), canonical_delta_bytes(&b));
    }

    // ── Criterio #6 (Orden §5): gate de titularidad exclusiva ───────────────

    /// CRITERIO DE CIERRE: un reclamo desde el epoch VIGENTE gana --
    /// produce el estado siguiente con el reclamante como titular y el
    /// epoch avanzado en +1.
    #[test]
    fn decide_custody_claim_succeeds_from_the_current_epoch() {
        let current = CustodyState { owner_id: "owner-1".to_string(), titular_node_id: "node-A".to_string(), custody_epoch: 3 };

        let next = decide_custody_claim(&current, "node-B", 3).expect("el reclamo desde el epoch vigente debe ganar");

        assert_eq!(next.titular_node_id, "node-B");
        assert_eq!(next.custody_epoch, 4);
    }

    /// CRITERIO DE CIERRE: un reclamo desde un epoch VENCIDO (distinto del
    /// vigente) devuelve `CustodyConflict` -- NUNCA produce un estado
    /// nuevo, NUNCA dos máquinas quedan tituales a la vez.
    #[test]
    fn decide_custody_claim_rejects_a_stale_epoch() {
        let current = CustodyState { owner_id: "owner-1".to_string(), titular_node_id: "node-A".to_string(), custody_epoch: 3 };

        let result = decide_custody_claim(&current, "node-B", 2);

        assert_eq!(result, Err(CustodyClaimError::CustodyConflict { owner_id: "owner-1".to_string(), expected_epoch: 2 }));
    }

    /// `is_current_titular` es `true` SOLO para el `node_id` titular
    /// vigente -- falso para cualquier otra máquina.
    #[test]
    fn is_current_titular_is_true_only_for_the_vigent_titular() {
        let state = CustodyState { owner_id: "owner-1".to_string(), titular_node_id: "node-A".to_string(), custody_epoch: 5 };

        assert!(is_current_titular("node-A", &state));
        assert!(!is_current_titular("node-B", &state));
    }

    // ── Utilidades hex ───────────────────────────────────────────────────────

    #[test]
    fn encode_hex_and_decode_hex_round_trip() {
        let bytes = vec![0u8, 1, 255, 16, 128];
        assert_eq!(decode_hex(&encode_hex(&bytes)).expect("decode debe tener éxito"), bytes);
    }

    #[test]
    fn decode_hex_rejects_odd_length() {
        assert_eq!(decode_hex("abc"), Err(HexDecodeError::OddLength));
    }

    #[test]
    fn decode_hex_rejects_invalid_digit() {
        assert_eq!(decode_hex("zz"), Err(HexDecodeError::InvalidDigit));
    }

    // ── Hash de auditoría encadenado ─────────────────────────────────────────

    #[test]
    fn compute_backup_audit_hash_is_deterministic() {
        let a = compute_backup_audit_hash("id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1", 900, "blobhash", 128, "noncehex");
        let b = compute_backup_audit_hash("id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1", 900, "blobhash", 128, "noncehex");
        assert_eq!(a, b);
    }

    #[test]
    fn compute_backup_audit_hash_changes_when_blob_hash_changes() {
        let a = compute_backup_audit_hash("id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1", 900, "blobhash-a", 128, "noncehex");
        let b = compute_backup_audit_hash("id-1", 1_000, 1, "GENESIS", "owner-1", "DRASUS_LOCAL", "node-1", 900, "blobhash-b", 128, "noncehex");
        assert_ne!(a, b);
    }

    #[test]
    fn compute_custody_audit_hash_is_deterministic() {
        let a = compute_custody_audit_hash("id-1", 1_000, 1, None, "owner-1", "DRASUS_LOCAL", "node-A");
        let b = compute_custody_audit_hash("id-1", 1_000, 1, None, "owner-1", "DRASUS_LOCAL", "node-A");
        assert_eq!(a, b);
    }

    #[test]
    fn compute_custody_audit_hash_changes_when_titular_changes() {
        let a = compute_custody_audit_hash("id-1", 2_000, 2, Some("prev-hash"), "owner-1", "DRASUS_LOCAL", "node-A");
        let b = compute_custody_audit_hash("id-1", 2_000, 2, Some("prev-hash"), "owner-1", "DRASUS_LOCAL", "node-B");
        assert_ne!(a, b);
    }
}
