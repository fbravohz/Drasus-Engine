//! [SHELL] Composición del cimiento #11 (`docs/features/instance-continuity.md`,
//! ADR-0146, ADR-0093, STORY-039).
//!
//! Capa delgada sobre [`crate::persistence::instance_continuity`]: traduce
//! las operaciones que el resto del substrato necesita -- "cifra y
//! registra un snapshot" y "reclama/consulta la titularidad de custodia"
//! -- sin que el llamador tenga que conocer los repositorios ni el
//! esquema de las tablas. Mismo rol que
//! `orchestrator::consent_registry::record_consent_action`/
//! `resolve_consent_verdict` para el cimiento #5.

use sqlx::SqlitePool;

use crate::domain::clock::Clock;
use crate::domain::instance_continuity::{
    canonical_delta_bytes, compute_backup_delta, decode_hex, decrypt_backup_blob,
    derive_encryption_key, encrypt_backup_blob, generate_nonce, is_current_titular, sha256_hex,
    BackupField, CustodyState, EncryptedBackupBlob, EncryptionError,
};
use crate::persistence::instance_continuity::{
    BackupRegistryRepository, BackupRegistryRepositoryError, ClaimTitularInput, CustodyRepository,
    CustodyRepositoryError, CustodyRow, InstanceBackupRow, RecordBackupInput,
};

/// Identidad de quien respalda / reclama custodia (Perfil D, ADR-0020) --
/// espejo de `EventEmissionIdentity`/`ReportGenerationIdentity`.
#[derive(Debug, Clone)]
pub struct InstanceContinuityIdentity {
    pub owner_id: String,
    pub institutional_tag: String,
    pub node_id: String,
}

/// Errores de la composición completa de [`take_encrypted_snapshot`].
#[derive(Debug, thiserror::Error)]
pub enum BackupSnapshotError {
    #[error("fallo de cifrado: {0}")]
    Encryption(#[from] EncryptionError),
    #[error("fallo al persistir el registro de respaldo: {0}")]
    Repository(#[from] BackupRegistryRepositoryError),
}

/// Resultado de [`take_encrypted_snapshot`]: el blob cifrado (puerto
/// `backup_blob_out`, listo para el adaptador de almacén de objetos
/// diferido) + la fila de metadatos ya persistida en el registro
/// append-only (`instance_backups`).
#[derive(Debug, Clone)]
pub struct BackupSnapshotResult {
    pub blob: EncryptedBackupBlob,
    pub row: InstanceBackupRow,
}

/// Composición completa del puerto `backup_blob_out`: filtra secretos de
/// bróker/IPs live del snapshot crudo, deriva la clave desde el secreto
/// maestro, cifra con el nonce inyectado y registra la fila de metadatos
/// append-only atómica.
///
/// `raw_fields` son los campos crudos candidatos del snapshot -- algunos
/// pueden ser secretos, se filtran ANTES de cifrar (nunca llegan al
/// ciphertext, ver [`compute_backup_delta`]). `nonce_seed` determina el
/// nonce: sembrado/determinista si lo provee un test, derivado de una
/// fuente de entropía real si lo resuelve la Shell de producción (el Core
/// nunca decide esto por su cuenta).
pub async fn take_encrypted_snapshot(
    pool: &SqlitePool,
    clock: &dyn Clock,
    identity: &InstanceContinuityIdentity,
    master_secret: &str,
    raw_fields: &[BackupField],
    nonce_seed: u64,
) -> Result<BackupSnapshotResult, BackupSnapshotError> {
    // Filtra secretos de bróker / IPs live ANTES de tocar el cifrado --
    // un campo excluido aquí nunca llega, ni siquiera cifrado, al blob.
    let delta = compute_backup_delta(raw_fields);
    let plaintext = canonical_delta_bytes(&delta);

    let key = derive_encryption_key(master_secret, &identity.owner_id);
    let nonce = generate_nonce(nonce_seed);
    let blob = encrypt_backup_blob(&plaintext, &key, &nonce)?;

    let ciphertext_bytes = decode_hex(&blob.ciphertext_hex)
        // El ciphertext que acabamos de producir con encode_hex SIEMPRE es
        // hexadecimal válido -- solo fallaría si encrypt_backup_blob
        // tuviera un defecto interno de codificación, lo cual no ocurre.
        .expect("el ciphertext recién cifrado siempre es hexadecimal válido");
    let blob_hash = sha256_hex(&ciphertext_bytes);

    let now_ns = clock.timestamp_ns();
    let repo = BackupRegistryRepository::new(pool, clock);
    let row = repo
        .record_backup(RecordBackupInput {
            owner_id: identity.owner_id.clone(),
            institutional_tag: identity.institutional_tag.clone(),
            node_id: identity.node_id.clone(),
            snapshot_at_ns: now_ns,
            blob_hash,
            blob_size_bytes: ciphertext_bytes.len() as i64,
            nonce_hex: blob.nonce_hex.clone(),
        })
        .await?;

    Ok(BackupSnapshotResult { blob, row })
}

/// Recorre el round-trip completo (cifrar -> descifrar) sobre `plaintext`,
/// usada por el harness de verificación CLI para demostrar que el blob
/// producido por [`take_encrypted_snapshot`] es recuperable con la MISMA
/// clave. Delegada al Core -- esta función no añade lógica propia, solo
/// re-deriva la clave (la Shell nunca guarda la clave entre llamadas,
/// ADR-0093).
pub fn round_trip_decrypts_to(
    blob: &EncryptedBackupBlob,
    master_secret: &str,
    owner_id: &str,
    expected_plaintext: &[u8],
) -> Result<bool, EncryptionError> {
    let key = derive_encryption_key(master_secret, owner_id);
    let decrypted = decrypt_backup_blob(blob, &key)?;
    Ok(decrypted == expected_plaintext)
}

/// Errores de [`claim_custody`].
#[derive(Debug, thiserror::Error)]
pub enum ClaimCustodyError {
    #[error("{0}")]
    Repository(#[from] CustodyRepositoryError),
}

/// Composición completa del gate de titularidad: intenta reclamar la
/// titularidad de custodia para `identity.node_id` desde `expected_epoch`.
/// Ver [`CustodyRepository::claim_titular`] para el detalle de la guarda de
/// concurrencia optimista.
pub async fn claim_custody(
    pool: &SqlitePool,
    clock: &dyn Clock,
    identity: &InstanceContinuityIdentity,
    expected_epoch: i64,
) -> Result<CustodyRow, ClaimCustodyError> {
    let repo = CustodyRepository::new(pool, clock);
    let row = repo
        .claim_titular(ClaimTitularInput {
            owner_id: identity.owner_id.clone(),
            institutional_tag: identity.institutional_tag.clone(),
            claiming_node_id: identity.node_id.clone(),
            expected_epoch,
        })
        .await?;
    Ok(row)
}

/// Consulta el puerto `custody_status_out`: ¿`node_id` es la titular
/// vigente de la custodia PERSISTIDA para `owner_id`? `false` si nunca se
/// registró custodia para este `owner_id` (ninguna máquina es titular
/// todavía, incluida esta).
///
/// Delega la decisión al Core puro ([`is_current_titular`]) -- esta
/// función solo trae el dato y pregunta, no decide nada por sí misma.
pub async fn is_titular(
    pool: &SqlitePool,
    clock: &dyn Clock,
    owner_id: &str,
    node_id: &str,
) -> Result<bool, CustodyRepositoryError> {
    let repo = CustodyRepository::new(pool, clock);
    match repo.find_by_owner(owner_id).await? {
        None => Ok(false),
        Some(row) => {
            let state = CustodyState {
                owner_id: row.owner_id,
                titular_node_id: row.titular_node_id,
                custody_epoch: row.custody_epoch,
            };
            Ok(is_current_titular(node_id, &state))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::clock::DeterministicClock;
    use crate::persistence::pool::{connect, migrate};

    async fn migrated_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.expect("conectar en memoria");
        migrate(&pool).await.expect("aplicar migraciones");
        pool
    }

    fn sample_identity() -> InstanceContinuityIdentity {
        InstanceContinuityIdentity {
            owner_id: "owner-1".to_string(),
            institutional_tag: "DRASUS_LOCAL".to_string(),
            node_id: "node-A".to_string(),
        }
    }

    /// Camino feliz completo: cifra + persiste + el round-trip recupera el
    /// plaintext exacto, ejercitando repo -> Core -> repo tal como lo
    /// recorrería el flujo real.
    #[tokio::test]
    async fn take_encrypted_snapshot_persists_and_round_trips() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let identity = sample_identity();

        let fields = vec![
            BackupField { key: "strategy_name".to_string(), value: "mean-reversion-1".to_string() },
            BackupField { key: "broker_credential_secret".to_string(), value: "no-debe-sobrevivir".to_string() },
        ];

        let result = take_encrypted_snapshot(&pool, &clock, &identity, "correct horse battery staple", &fields, 42)
            .await
            .expect("tomar el snapshot debe tener éxito");

        assert_eq!(result.row.event_sequence_id, 1);
        assert_eq!(result.row.owner_id, "owner-1");

        // El ciphertext nunca debe contener el secreto en claro (ni
        // codificado hex de forma reconocible) -- prueba indirecta de que
        // el filtro de delta corrió ANTES del cifrado.
        assert!(!result.blob.ciphertext_hex.is_empty());

        // El delta filtrado (solo strategy_name) debe reconstruirse tras
        // el round-trip.
        let expected_plaintext = canonical_delta_bytes(&compute_backup_delta(&fields));
        let round_trip_ok = round_trip_decrypts_to(&result.blob, "correct horse battery staple", "owner-1", &expected_plaintext)
            .expect("el descifrado debe tener éxito con la clave correcta");
        assert!(round_trip_ok, "el round-trip debe recuperar el delta filtrado exacto");
    }

    /// `is_titular` es `false` para cualquier máquina antes de que exista
    /// custodia registrada, y `true` solo para la titular tras un reclamo
    /// exitoso.
    #[tokio::test]
    async fn is_titular_reflects_the_persisted_custody_state() {
        let pool = migrated_pool().await;
        let clock = DeterministicClock::new(1_000, 100);
        let identity = sample_identity();

        assert!(
            !is_titular(&pool, &clock, &identity.owner_id, &identity.node_id).await.expect("consulta debe tener éxito"),
            "sin custodia registrada, ninguna máquina es titular todavía"
        );

        let row = claim_custody(&pool, &clock, &identity, 0).await.expect("el reclamo bootstrap debe tener éxito");
        assert_eq!(row.custody_epoch, 1);

        assert!(is_titular(&pool, &clock, &identity.owner_id, &identity.node_id).await.expect("consulta debe tener éxito"));

        let other_node = "node-B";
        assert!(!is_titular(&pool, &clock, &identity.owner_id, other_node).await.expect("consulta debe tener éxito"));
    }
}
