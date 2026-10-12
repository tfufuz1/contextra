//! Ed25519 signer and key rotation attestations for deletion proofs (ADR-12, C7).

use crate::error::{CryptoError, Result};
use ed25519_dalek::{Signature, Signer as DalekSigner, SigningKey, Verifier, VerifyingKey};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use zeroize::Zeroizing;

/// Trait defining proof signing capability and key identification.
pub trait ProofSigner: Send + Sync {
    /// Signs a message byte slice and returns a 64-byte Ed25519 signature.
    fn sign(&self, msg: &[u8]) -> Result<[u8; 64]>;

    /// Returns the BLAKE3 digest (32 bytes) of the public verification key.
    fn key_id(&self) -> [u8; 32];

    /// Returns the 32-byte public verification key.
    fn public_key(&self) -> [u8; 32];
}

/// In-memory Ed25519 signer wrapping key material in zeroizing memory.
pub struct ProofKeyPairSigner {
    signing_key: Zeroizing<[u8; 32]>,
    public_key: [u8; 32],
    key_id: [u8; 32],
}

impl ProofKeyPairSigner {
    /// Creates a signer from a 32-byte secret seed.
    pub fn from_secret_bytes(secret: [u8; 32]) -> Self {
        let dalek_sk = SigningKey::from_bytes(&secret);
        let public_key = dalek_sk.verifying_key().to_bytes();
        let key_id = *blake3::hash(&public_key).as_bytes();
        Self {
            signing_key: Zeroizing::new(secret),
            public_key,
            key_id,
        }
    }

    /// Generates a new random keypair signer using CSPRNG.
    pub fn generate(rng: &mut impl rand::RngCore) -> Self {
        let mut secret = [0u8; 32];
        rng.fill_bytes(&mut secret);
        Self::from_secret_bytes(secret)
    }

    /// Persists this keypair to a file with strict 0600 permissions and durable fsync.
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let parent = path
            .parent()
            .ok_or_else(|| CryptoError::Crypto("Invalid key file path".to_string()))?;

        #[cfg(unix)]
        use std::os::unix::fs::OpenOptionsExt;

        let mut options = OpenOptions::new();
        options.write(true).create_new(true);

        #[cfg(unix)]
        options.mode(0o600);

        let mut file = options
            .open(path)
            .map_err(|e| CryptoError::Crypto(format!("Failed to create key file O_EXCL: {e}")))?;

        file.write_all(self.signing_key.as_ref())
            .map_err(|e| CryptoError::Crypto(format!("Failed to write key material: {e}")))?;

        file.sync_all()
            .map_err(|e| CryptoError::Crypto(format!("Failed to fsync key file: {e}")))?;

        contextra_durable_fs::sync_dir(parent)
            .map_err(|e| CryptoError::Crypto(format!("Failed to fsync key directory: {e}")))?;

        Ok(())
    }

    /// Loads a keypair from a file, verifying strict 0600 permissions.
    pub fn load_from_file(path: &Path) -> Result<Self> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(path).map_err(|e| {
                CryptoError::Crypto(format!("Failed to read key file metadata: {e}"))
            })?;
            let mode = metadata.permissions().mode() & 0o777;
            if mode != 0o600 {
                return Err(CryptoError::Crypto(format!(
                    "Key file permissions too permissive (0{:o}); required 0600 fail-closed",
                    mode
                )));
            }
        }

        let raw_bytes = std::fs::read(path)
            .map_err(|e| CryptoError::Crypto(format!("Failed to read key file: {e}")))?;

        if raw_bytes.len() != 32 {
            return Err(CryptoError::Crypto(format!(
                "Invalid key file length: expected 32 bytes, got {}",
                raw_bytes.len()
            )));
        }

        let mut secret = [0u8; 32];
        secret.copy_from_slice(&raw_bytes);
        Ok(Self::from_secret_bytes(secret))
    }

    /// Attests the rotation to a new signer's `key_id`, signed by this signer.
    pub fn attest_rotation(&self, new_key_id: &[u8; 32]) -> Result<ProofRotationAttestation> {
        let domain_prefix = b"contextra/key-rotation/v1\0";
        let mut msg = Vec::with_capacity(domain_prefix.len() + 32 + 32);
        msg.extend_from_slice(domain_prefix);
        msg.extend_from_slice(&self.key_id);
        msg.extend_from_slice(new_key_id);

        let sig = self.sign(&msg)?;
        Ok(ProofRotationAttestation {
            old_key_id: self.key_id,
            new_key_id: *new_key_id,
            signature: sig,
        })
    }
}

impl ProofSigner for ProofKeyPairSigner {
    fn sign(&self, msg: &[u8]) -> Result<[u8; 64]> {
        let dalek_sk = SigningKey::from_bytes(&self.signing_key);
        let sig = dalek_sk.sign(msg);
        Ok(sig.to_bytes())
    }

    fn key_id(&self) -> [u8; 32] {
        self.key_id
    }

    fn public_key(&self) -> [u8; 32] {
        self.public_key
    }
}

/// Attestation confirming key rotation from `old_key_id` to `new_key_id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofRotationAttestation {
    /// Old signer key ID (BLAKE3 hash of old public key).
    pub old_key_id: [u8; 32],
    /// New signer key ID (BLAKE3 hash of new public key).
    pub new_key_id: [u8; 32],
    /// Ed25519 signature by old key certifying new key ID.
    pub signature: [u8; 64],
}

impl ProofRotationAttestation {
    /// Verifies that the rotation attestation was signed by the old public key.
    pub fn verify(&self, old_public_key: &[u8; 32]) -> Result<()> {
        let expected_old_key_id = *blake3::hash(old_public_key).as_bytes();
        if self.old_key_id != expected_old_key_id {
            return Err(CryptoError::Crypto(
                "Rotation attestation old_key_id mismatch".to_string(),
            ));
        }

        let vk = VerifyingKey::from_bytes(old_public_key)
            .map_err(|e| CryptoError::Crypto(format!("Invalid verifying key: {e}")))?;

        let domain_prefix = b"contextra/key-rotation/v1\0";
        let mut msg = Vec::with_capacity(domain_prefix.len() + 32 + 32);
        msg.extend_from_slice(domain_prefix);
        msg.extend_from_slice(&self.old_key_id);
        msg.extend_from_slice(&self.new_key_id);

        let sig = Signature::from_bytes(&self.signature);
        vk.verify(&msg, &sig).map_err(|e| {
            CryptoError::Crypto(format!("Invalid rotation attestation signature: {e}"))
        })?;

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;
    use tempfile::NamedTempFile;

    #[test]
    fn test_signer_save_and_load_file_roundtrip() {
        let signer = ProofKeyPairSigner::generate(&mut OsRng);
        let tmp = NamedTempFile::new().expect("tmpfile failed");
        let path = tmp.path().to_path_buf();
        // Remove tmp file so save_to_file can create with O_EXCL
        std::fs::remove_file(&path).ok();

        signer.save_to_file(&path).expect("save_to_file failed");

        let loaded = ProofKeyPairSigner::load_from_file(&path).expect("load_from_file failed");

        assert_eq!(signer.public_key(), loaded.public_key());
        assert_eq!(signer.key_id(), loaded.key_id());

        let msg = b"test message to sign";
        let sig1 = signer.sign(msg).expect("sign failed");
        let sig2 = loaded.sign(msg).expect("sign failed");
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_signer_wrong_permissions_fails_load() {
        let signer = ProofKeyPairSigner::generate(&mut OsRng);
        let tmp = NamedTempFile::new().expect("tmpfile failed");
        let path = tmp.path().to_path_buf();
        std::fs::remove_file(&path).ok();

        signer.save_to_file(&path).expect("save_to_file failed");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
                .expect("chmod failed");

            let res = ProofKeyPairSigner::load_from_file(&path);
            assert!(res.is_err(), "load_from_file MUST fail on 0644 permissions");
        }
    }

    #[test]
    fn test_signer_rotation_attestation_verifiable() {
        let signer1 = ProofKeyPairSigner::generate(&mut OsRng);
        let signer2 = ProofKeyPairSigner::generate(&mut OsRng);

        let attestation = signer1
            .attest_rotation(&signer2.key_id())
            .expect("attest_rotation failed");

        assert!(attestation.verify(&signer1.public_key()).is_ok());

        // Wrong public key fails verification
        assert!(attestation.verify(&signer2.public_key()).is_err());
    }
}
