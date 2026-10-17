//! Ed25519 signer and key rotation attestations for deletion proofs (ADR-12, C7).

use crate::error::{CryptoError, Result};
use ed25519_dalek::{Signature, Signer as DalekSigner, SigningKey, VerifyingKey};
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::RwLock;
use zeroize::Zeroizing;

/// Domain separation tag for key rotation attestations and statements.
pub const KEY_ROTATION_DOMAIN_V1: &[u8] = b"contextra/key-rotation/v1\0";

/// Trait for validating key file ownership.
pub trait FileOwnershipCheck: Send + Sync {
    /// Verifies if the file's ownership (file UID) is acceptable.
    fn verify_ownership(&self, file_uid: u32) -> Result<()>;
}

/// Default strict file ownership check: checks file UID against current user/process UID (or root UID 0).
#[derive(Debug, Clone, Copy, Default)]
pub struct StrictFileOwnershipCheck;

impl FileOwnershipCheck for StrictFileOwnershipCheck {
    fn verify_ownership(&self, file_uid: u32) -> Result<()> {
        #[cfg(unix)]
        {
            let current_uid = std::env::var("USER_ID")
                .or_else(|_| std::env::var("UID"))
                .ok()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(1000);
            if file_uid != current_uid && file_uid != 0 {
                return Err(CryptoError::Crypto(format!(
                    "Key file ownership mismatch: owned by UID {}, running as UID {}",
                    file_uid, current_uid
                )));
            }
        }
        Ok(())
    }
}

/// Fake/permissive file ownership check for testing or container environments with fake UIDs.
#[derive(Debug, Clone, Copy, Default)]
pub struct FakeFileOwnershipCheck;

impl FileOwnershipCheck for FakeFileOwnershipCheck {
    fn verify_ownership(&self, _file_uid: u32) -> Result<()> {
        Ok(())
    }
}

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

impl std::fmt::Debug for ProofKeyPairSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProofKeyPairSigner")
            .field("signing_key", &"***REDACTED***")
            .field("public_key", &self.public_key)
            .field("key_id", &self.key_id)
            .finish()
    }
}

impl ProofKeyPairSigner {
    /// Creates a signer from a 32-byte secret seed.
    pub fn from_secret_bytes(secret: [u8; 32]) -> Self {
        let secret_zero = Zeroizing::new(secret);
        let dalek_sk = SigningKey::from_bytes(&*secret_zero);
        let public_key = dalek_sk.verifying_key().to_bytes();
        let key_id = *blake3::hash(&public_key).as_bytes();
        Self {
            signing_key: secret_zero,
            public_key,
            key_id,
        }
    }

    /// Generates a new random keypair signer using CSPRNG.
    pub fn generate(rng: &mut impl rand::RngCore) -> Self {
        let mut secret = Zeroizing::new([0u8; 32]);
        rng.fill_bytes(secret.as_mut());
        Self::from_secret_bytes(*secret)
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

    /// Loads a keypair from a file using default strict file ownership check ([`StrictFileOwnershipCheck`]).
    pub fn load_from_file(path: &Path) -> Result<Self> {
        Self::load_from_file_with_ownership_check(path, &StrictFileOwnershipCheck)
    }

    /// Loads a keypair from a file, verifying strict 0600 permissions, rejecting symlinks, and executing `ownership_check`.
    pub fn load_from_file_with_ownership_check(
        path: &Path,
        ownership_check: &dyn FileOwnershipCheck,
    ) -> Result<Self> {
        let symlink_meta = std::fs::symlink_metadata(path)
            .map_err(|e| CryptoError::Crypto(format!("Failed to read key file metadata: {e}")))?;

        if symlink_meta.file_type().is_symlink() {
            return Err(CryptoError::Crypto(
                "Refusing to load key file: path is a symbolic link".to_string(),
            ));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            use std::os::unix::fs::PermissionsExt;

            let mode = symlink_meta.permissions().mode() & 0o777;
            if mode != 0o600 {
                return Err(CryptoError::Crypto(format!(
                    "Key file permissions too permissive (0{:o}); required 0600 fail-closed",
                    mode
                )));
            }

            let file_uid = symlink_meta.uid();
            ownership_check.verify_ownership(file_uid)?;
        }

        let raw_bytes = std::fs::read(path)
            .map_err(|e| CryptoError::Crypto(format!("Failed to read key file: {e}")))?;

        if raw_bytes.len() != 32 {
            return Err(CryptoError::Crypto(format!(
                "Invalid key file length: expected 32 bytes, got {}",
                raw_bytes.len()
            )));
        }

        let mut secret = Zeroizing::new([0u8; 32]);
        secret.copy_from_slice(&raw_bytes);
        Ok(Self::from_secret_bytes(*secret))
    }

    /// Verifies a chain of key rotation attestations from `trusted_root_pubkey` down to the active key_id.
    pub fn verify_rotation_chain(
        attestations: &[ProofRotationAttestation],
        trusted_root_pubkey: &[u8; 32],
    ) -> Result<[u8; 32]> {
        if attestations.is_empty() {
            let root_key_id = *blake3::hash(trusted_root_pubkey).as_bytes();
            return Ok(root_key_id);
        }

        let mut current_key_id = *blake3::hash(trusted_root_pubkey).as_bytes();

        for att in attestations {
            if att.old_key_id != current_key_id {
                return Err(CryptoError::Crypto(format!(
                    "Rotation chain broken: expected old_key_id {:?}, got {:?}",
                    current_key_id, att.old_key_id
                )));
            }

            att.verify(trusted_root_pubkey)?;
            current_key_id = att.new_key_id;
        }

        Ok(current_key_id)
    }

    /// Attests the rotation to a new signer's `key_id`, signed by this signer.
    pub fn attest_rotation(&self, new_key_id: &[u8; 32]) -> Result<ProofRotationAttestation> {
        let domain_prefix = KEY_ROTATION_DOMAIN_V1;
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

    /// Generates a signed statement marking this key as non-issuing (retired/rotated) as of a timestamp.
    pub fn create_rotation_statement(
        &self,
        rotated_at_unix_ms: u64,
    ) -> Result<KeyRotationStatement> {
        let mut payload = Vec::with_capacity(KEY_ROTATION_DOMAIN_V1.len() + 32 + 8);
        payload.extend_from_slice(KEY_ROTATION_DOMAIN_V1);
        payload.extend_from_slice(&self.key_id);
        payload.extend_from_slice(&rotated_at_unix_ms.to_le_bytes());

        let signature = self.sign(&payload)?;
        Ok(KeyRotationStatement {
            key_id: self.key_id,
            public_key: self.public_key,
            rotated_at_unix_ms,
            signature,
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

/// Signed statement declaring that a key is rotated out and no longer issuing proofs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRotationStatement {
    /// Key ID (BLAKE3 hash of public key) of the retired key.
    pub key_id: [u8; 32],
    /// Public verification key of the retired key.
    pub public_key: [u8; 32],
    /// Unix timestamp in milliseconds after which the key is non-issuing.
    pub rotated_at_unix_ms: u64,
    /// Ed25519 signature by the retired key certifying its rotation.
    pub signature: [u8; 64],
}

impl KeyRotationStatement {
    /// Verifies the rotation statement signature against its public key.
    pub fn verify(&self) -> Result<()> {
        let expected_key_id = *blake3::hash(&self.public_key).as_bytes();
        if self.key_id != expected_key_id {
            return Err(CryptoError::Crypto(
                "Rotation statement key_id mismatch with public key".to_string(),
            ));
        }

        let vk = VerifyingKey::from_bytes(&self.public_key).map_err(|e| {
            CryptoError::Crypto(format!("Invalid public key in rotation statement: {e}"))
        })?;

        let mut payload = Vec::with_capacity(KEY_ROTATION_DOMAIN_V1.len() + 32 + 8);
        payload.extend_from_slice(KEY_ROTATION_DOMAIN_V1);
        payload.extend_from_slice(&self.key_id);
        payload.extend_from_slice(&self.rotated_at_unix_ms.to_le_bytes());

        let sig = Signature::from_bytes(&self.signature);
        vk.verify_strict(&payload, &sig).map_err(|e| {
            CryptoError::Crypto(format!("Invalid rotation statement signature: {e}"))
        })?;

        Ok(())
    }

    /// Checks if a given timestamp is at or after the rotation timestamp.
    pub fn is_expired_at(&self, issued_at_unix_ms: u64) -> bool {
        issued_at_unix_ms >= self.rotated_at_unix_ms
    }
}

/// Registry of revoked or compromised key IDs whose proofs must be rejected.
#[derive(Debug, Default)]
pub struct UntrustedKeyList {
    untrusted_key_ids: RwLock<HashSet<[u8; 32]>>,
}

impl UntrustedKeyList {
    /// Creates a new empty untrusted key list.
    pub fn new() -> Self {
        Self {
            untrusted_key_ids: RwLock::new(HashSet::new()),
        }
    }

    /// Marks a key ID as untrusted.
    pub fn revoke_key_id(&self, key_id: [u8; 32]) -> Result<()> {
        let mut guard = self.untrusted_key_ids.write().map_err(|_| {
            CryptoError::Crypto("Lock poison error in UntrustedKeyList".to_string())
        })?;
        guard.insert(key_id);
        Ok(())
    }

    /// Marks a key as untrusted using its public key.
    pub fn revoke_public_key(&self, public_key: &[u8; 32]) -> Result<()> {
        let key_id = *blake3::hash(public_key).as_bytes();
        self.revoke_key_id(key_id)
    }

    /// Checks if a key ID is untrusted (fail-closed).
    pub fn is_untrusted(&self, key_id: &[u8; 32]) -> bool {
        self.untrusted_key_ids
            .read()
            .map(|guard| guard.contains(key_id))
            .unwrap_or(true)
    }

    /// Verifies that a key ID is trusted; returns an error if untrusted or lock is poisoned.
    pub fn verify_trusted(&self, key_id: &[u8; 32]) -> Result<()> {
        if self.is_untrusted(key_id) {
            Err(CryptoError::Crypto(format!(
                "Key ID {:?} is marked untrusted in UntrustedKeyList",
                key_id
            )))
        } else {
            Ok(())
        }
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

        let domain_prefix = KEY_ROTATION_DOMAIN_V1;
        let mut msg = Vec::with_capacity(domain_prefix.len() + 32 + 32);
        msg.extend_from_slice(domain_prefix);
        msg.extend_from_slice(&self.old_key_id);
        msg.extend_from_slice(&self.new_key_id);

        let sig = Signature::from_bytes(&self.signature);
        vk.verify_strict(&msg, &sig).map_err(|e| {
            CryptoError::Crypto(format!("Invalid rotation attestation signature: {e}"))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;
    use tempfile::NamedTempFile;

    #[test]
    fn test_signer_save_and_load_file_roundtrip(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let signer = ProofKeyPairSigner::generate(&mut OsRng);
        let tmp = NamedTempFile::new()?;
        let path = tmp.path().to_path_buf();
        // Remove tmp file so save_to_file can create with O_EXCL
        std::fs::remove_file(&path).ok();

        signer.save_to_file(&path)?;

        let loaded = ProofKeyPairSigner::load_from_file_with_ownership_check(
            &path,
            &FakeFileOwnershipCheck,
        )?;

        assert_eq!(signer.public_key(), loaded.public_key());
        assert_eq!(signer.key_id(), loaded.key_id());

        let msg = b"test message to sign";
        let sig1 = signer.sign(msg)?;
        let sig2 = loaded.sign(msg)?;
        assert_eq!(sig1, sig2);
        Ok(())
    }

    #[test]
    fn test_signer_wrong_permissions_fails_load(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let signer = ProofKeyPairSigner::generate(&mut OsRng);
        let tmp = NamedTempFile::new()?;
        let path = tmp.path().to_path_buf();
        std::fs::remove_file(&path).ok();

        signer.save_to_file(&path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))?;

            let res = ProofKeyPairSigner::load_from_file(&path);
            assert!(res.is_err(), "load_from_file MUST fail on 0644 permissions");
        }
        Ok(())
    }

    #[test]
    fn test_signer_rotation_attestation_verifiable(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let signer1 = ProofKeyPairSigner::generate(&mut OsRng);
        let signer2 = ProofKeyPairSigner::generate(&mut OsRng);

        let attestation = signer1.attest_rotation(&signer2.key_id())?;

        assert!(attestation.verify(&signer1.public_key()).is_ok());

        // Wrong public key fails verification
        assert!(attestation.verify(&signer2.public_key()).is_err());
        Ok(())
    }

    #[test]
    fn test_rotation_statement_verifies_and_expires(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let signer = ProofKeyPairSigner::generate(&mut OsRng);
        let statement = signer.create_rotation_statement(1000)?;

        assert!(statement.verify().is_ok());
        assert!(!statement.is_expired_at(999));
        assert!(statement.is_expired_at(1000));
        assert!(statement.is_expired_at(1001));
        Ok(())
    }

    #[test]
    fn test_untrusted_key_list() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let signer = ProofKeyPairSigner::generate(&mut OsRng);
        let list = UntrustedKeyList::new();

        assert!(list.verify_trusted(&signer.key_id()).is_ok());

        list.revoke_key_id(signer.key_id())?;
        assert!(list.is_untrusted(&signer.key_id()));
        assert!(list.verify_trusted(&signer.key_id()).is_err());
        Ok(())
    }
}
