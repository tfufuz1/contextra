// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Sovereign Subsystem Wiring for contextra facade (Ring 4).
// INVARIANTEN: #![forbid(unsafe_code)]; KmsWiring -> DeletionProofWiring -> PrivacyWiring. Fail-closed without valid key.

#![forbid(unsafe_code)]

#[cfg(feature = "sovereign")]
use std::path::PathBuf;
#[cfg(feature = "sovereign")]
use std::sync::Arc;

#[cfg(feature = "sovereign")]
use contextra_types::ContextraError;
#[cfg(feature = "sovereign")]
use contextra_crypto::deletion_proof::signer::ProofKeyPairSigner;
#[cfg(feature = "sovereign")]
use contextra_crypto::deletion_proof::wiring::{DeletionProofWiring, DeletionProofWiringDeps};
#[cfg(feature = "sovereign")]
use contextra_crypto::ed25519_dalek;
#[cfg(feature = "sovereign")]
use contextra_crypto::kdf::KdfHeader;
#[cfg(feature = "sovereign")]
use contextra_crypto::kv_shredding::KeyRegistry;
#[cfg(feature = "sovereign")]
use contextra_crypto::revocation_log::RevocationLog;
#[cfg(feature = "sovereign")]
use contextra_crypto::wiring::{KmsWiring, KmsWiringDeps};
#[cfg(feature = "sovereign")]
use contextra_ports::clock::Clock;
#[cfg(feature = "sovereign")]
use contextra_privacy::wiring::{PrivacyWiring, PrivacyWiringConfig};

/// Bundles configured Sovereign subsystem instances (KMS, DeletionProof, Privacy).
#[cfg(feature = "sovereign")]
pub struct SovereignWiringBundle {
    pub kms: Arc<KmsWiring>,
    pub deletion_proof: Arc<DeletionProofWiring>,
    pub privacy: Arc<PrivacyWiring>,
}

#[cfg(feature = "sovereign")]
impl SovereignWiringBundle {
    /// Builds `SovereignWiringBundle` in fixed order: `KmsWiring` -> `DeletionProofWiring` -> `PrivacyWiring`.
    ///
    /// Fails closed if encryption passphrase is missing or key derivation fails.
    pub fn build(
        storage_path: &PathBuf,
        passphrase: Option<&str>,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, ContextraError> {
        let passphrase_str = passphrase.ok_or_else(|| {
            ContextraError::PolicyViolation(
                "Sovereign feature requires an explicit encryption_passphrase (fail-closed)".to_string(),
            )
        })?;

        let kdf_header = KdfHeader::generate_default()
            .map_err(|e| ContextraError::PolicyViolation(e.to_string()))?;

        let mut rng = rand::thread_rng();
        let signing_key = ed25519_dalek::SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();

        let revocation_log = Arc::new(RevocationLog::new_in_memory(
            clock.clone(),
            Some(signing_key.clone()),
            verifying_key,
        ));

        let registry = Arc::new(KeyRegistry::new());

        let kms = KmsWiring::build(KmsWiringDeps {
            passphrase: passphrase_str.to_string(),
            kdf_header,
            registry,
            revocation_log: revocation_log.clone(),
            clock: clock.clone(),
            wrapped_deks: None,
            registry_log: None,
        })
        .map_err(|e| ContextraError::PolicyViolation(e.to_string()))?;

        let signer = Arc::new(ProofKeyPairSigner::generate(&mut rng));

        let path_str = storage_path.to_string_lossy();
        let path_bytes = path_str.as_bytes();
        let mut db_instance_id = [0u8; 16];
        for (i, &b) in path_bytes.iter().enumerate() {
            db_instance_id[i % 16] ^= b;
        }

        let deletion_proof = DeletionProofWiring::build(DeletionProofWiringDeps {
            storage_dir: storage_path.clone(),
            signer,
            db_instance_id,
            clock: clock.clone(),
            revocation_log: Some(revocation_log),
            revocation_signing_key: Some(signing_key),
            revocation_verifying_key: verifying_key,
            has_existing_keys: false,
        })?;

        let privacy = PrivacyWiring::build(PrivacyWiringConfig::default())
            .map_err(|e| ContextraError::PolicyViolation(e.to_string()))?;

        Ok(Self {
            kms: Arc::new(kms),
            deletion_proof: Arc::new(deletion_proof),
            privacy: Arc::new(privacy),
        })
    }
}
