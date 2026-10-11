// FILE-CONTEXT
// ZWECK: Dedicated AEAD encryption and key isolation for KV-cache segments (contextra-crypto).
// INVARIANTEN: Key derivation per (tenant_id, model_fingerprint) tuple via KeyManager HKDF-Expand.
// NICHT-OFFENSICHTLICH: OsRng generates fresh 12-byte nonces per encrypt call. AES-256-GCM-SIV provides misuse-resistance.
// STAND: TS:2026-09-08T00:00:00Z

//! KV-Cache Segment Encryption Module (contextra-crypto).
//!
//! # Migration Strategy (Zwei-Schritt-Migrationsstrategie)
//! - **Schritt 1 (Dieses Modul):** Erstellung des eigenständigen Krypto-Moduls `KvSegmentCipher`
//!   mit kryptographischer Tenant-Isolation und Modell-Versionierungs-Trennung in `contextra-crypto`.
//! - **Schritt 2:** Verdrahtung von `KvSegmentCipher` und `EncryptedKvLayer`
//!   in `contextra-kvcache` und `contextra-infer-candle` (kv_bridge) zur Erweiterung der `KvSegment`-Struktur.
//!
//! # Nonce-Sicherheit & Nonce-Misuse-Resistance (RFC 8452)
//! Das Modul verwendet AES-256-GCM-SIV mit per-call `OsRng` generierten 12-Byte-Nonces.
//! Die Nonce-Misuse-Resistance von AES-256-GCM-SIV schützt vor Authentifizierungs-Schlüssel-Leaks
//! bei versehentlicher Nonce-Wiederverwendung. Sie dient als kryptographisches Sicherheitsnetz,
//! ersetzt jedoch **nicht** die Notwendigkeit, für jede Verschlüsselungsoperation frische,
//! kryptographisch zufällige Nonces über `OsRng` zu erzeugen.

#![forbid(unsafe_code)]

use crate::crypto::KeyManager;
use crate::error::{CryptoError, Result};
use contextra_types::TenantId;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

pub use contextra_types::ModelFingerprint;

/// Current serialized format version for [`EncryptedKvLayer`].
/// Increment when the serialized layout changes in a breaking way.
pub const CURRENT_KV_FORMAT_VERSION: u8 = 2;

/// Container for an encrypted KV-cache segment layer.
///
/// Plaintext data and nonce are zeroized on drop. Public metadata (`tenant_id`, `model_fingerprint`, `format_version`)
/// is explicitly skipped during zeroization (`#[zeroize(skip)]`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
pub struct EncryptedKvLayer {
    /// Serialized format version. Must equal [`CURRENT_KV_FORMAT_VERSION`] on decrypt.
    #[zeroize(skip)]
    pub format_version: u8,
    /// AES-256-GCM-SIV ciphertext containing encrypted KV tensor payload and 16-byte auth tag.
    pub ciphertext: Vec<u8>,
    /// 12-byte initialization vector (nonce) generated via `OsRng`.
    pub nonce: [u8; 12],
    /// Tenant identifier bound to this layer.
    #[zeroize(skip)]
    pub tenant_id: TenantId,
    /// Model fingerprint bound to this layer.
    #[zeroize(skip)]
    pub model_fingerprint: ModelFingerprint,
}

/// Abstract cipher trait for KV block encryption and seal/open operations.
pub trait KvCipher: Send + Sync {
    /// Encrypts (seals) plaintext byte payload into authenticated ciphertext bytes.
    fn seal(&self, plaintext: &[u8]) -> contextra_types::Result<Vec<u8>>;
    /// Decrypts (opens) authenticated ciphertext bytes back to plaintext byte payload.
    fn open(&self, ciphertext: &[u8]) -> contextra_types::Result<Vec<u8>>;
}

impl KvCipher for KeyManager {
    fn seal(&self, plaintext: &[u8]) -> contextra_types::Result<Vec<u8>> {
        let (ct, nonce) = self
            .encrypt_auto_nonce(plaintext)
            .map_err(|e| contextra_types::ContextraError::Crypto(e.to_string()))?;
        let mut out = Vec::with_capacity(12 + ct.len());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ct);
        Ok(out)
    }

    fn open(&self, ciphertext: &[u8]) -> contextra_types::Result<Vec<u8>> {
        if ciphertext.len() < 12 {
            return Err(contextra_types::ContextraError::Crypto(
                "Ciphertext too short for nonce payload".into(),
            ));
        }
        let nonce: &[u8; 12] =
            ciphertext[..12]
                .try_into()
                .map_err(|e: std::array::TryFromSliceError| {
                    contextra_types::ContextraError::ParseError(e.to_string())
                })?;
        let ct = &ciphertext[12..];
        self.decrypt_auto_nonce(ct, nonce)
            .map_err(|e| contextra_types::ContextraError::Crypto(e.to_string()))
    }
}

use crate::kv_shredding::KeyRegistry;
use crate::revocation_log::RevocationLog;
use std::sync::Arc;

/// High-level cipher engine for KV-cache segment encryption and decryption.
/// Consolidates onto `KeyRegistry` envelope crypto-shredding as the single shredding path.
pub struct KvSegmentCipher {
    key_manager: KeyManager,
    registry: KeyRegistry,
}

impl KvSegmentCipher {
    /// Creates a new `KvSegmentCipher` wrapping the workspace master `KeyManager` and mandatory `RevocationLog`.
    pub fn new(key_manager: KeyManager, log: Arc<RevocationLog>) -> Self {
        let registry = KeyRegistry::new().with_revocation_log(log);
        Self {
            key_manager,
            registry,
        }
    }

    /// Creates an ephemeral in-memory `KvSegmentCipher` without persistent disk storage (primarily for tests and temporary caches).
    pub fn ephemeral(key_manager: KeyManager) -> Self {
        Self {
            key_manager,
            registry: KeyRegistry::new(),
        }
    }

    /// Attaches or overrides the `RevocationLog` on the underlying `KeyRegistry`.
    pub fn with_revocation_log(mut self, log: Arc<RevocationLog>) -> Self {
        self.registry = self.registry.with_revocation_log(log);
        self
    }

    /// Returns a reference to the consolidated `KeyRegistry` for crypto-shredding key management.
    pub fn registry(&self) -> &KeyRegistry {
        &self.registry
    }

    /// Encrypts KV-cache plaintext payload for a given `(tenant_id, model_fingerprint)` pair.
    ///
    /// Derives an isolated sub-key via HKDF-SHA256 through `KeyManager::derive_kv_key()`
    /// to cryptographically enforce tenant isolation and model quantization boundaries.
    pub fn encrypt(
        &self,
        tenant_id: TenantId,
        model_fingerprint: ModelFingerprint,
        plaintext: &[u8],
    ) -> Result<EncryptedKvLayer> {
        let sub_km = self
            .key_manager
            .derive_kv_key(tenant_id, &model_fingerprint)?;
        let (ciphertext, nonce) = sub_km.encrypt_auto_nonce(plaintext)?;

        Ok(EncryptedKvLayer {
            format_version: CURRENT_KV_FORMAT_VERSION,
            ciphertext,
            nonce,
            tenant_id,
            model_fingerprint,
        })
    }

    /// Decrypts an `EncryptedKvLayer` back to its original plaintext.
    ///
    /// Checks that `encrypted.format_version == CURRENT_KV_FORMAT_VERSION`.
    /// Derives the exact sub-key for `(encrypted.tenant_id, encrypted.model_fingerprint)`.
    /// Fails with a format version error or authentication error if the layer or key
    /// was tampered with.
    pub fn decrypt(&self, encrypted: &EncryptedKvLayer) -> Result<Vec<u8>> {
        if encrypted.format_version != CURRENT_KV_FORMAT_VERSION {
            return Err(CryptoError::KvFormatVersionMismatch {
                expected: CURRENT_KV_FORMAT_VERSION,
                found: encrypted.format_version,
            });
        }

        let sub_km = self
            .key_manager
            .derive_kv_key(encrypted.tenant_id, &encrypted.model_fingerprint)?;
        sub_km.decrypt_auto_nonce(&encrypted.ciphertext, &encrypted.nonce)
    }

    /// Encrypts KV-cache plaintext payload using consolidated `KeyRegistry` envelope crypto-shredding.
    pub fn encrypt_with_version(
        &self,
        tenant_id: TenantId,
        segment_id: u64,
        _version: u8,
        model_fingerprint: ModelFingerprint,
        plaintext: &[u8],
    ) -> Result<EncryptedKvLayer> {
        let (ciphertext, nonce) =
            self.registry
                .encrypt_with_group(&self.key_manager, segment_id, plaintext)?;

        Ok(EncryptedKvLayer {
            format_version: CURRENT_KV_FORMAT_VERSION,
            ciphertext,
            nonce,
            tenant_id,
            model_fingerprint,
        })
    }

    /// Decrypts an `EncryptedKvLayer` using consolidated `KeyRegistry` envelope crypto-shredding.
    pub fn decrypt_with_version(
        &self,
        encrypted: &EncryptedKvLayer,
        segment_id: u64,
        _version: u8,
    ) -> Result<Vec<u8>> {
        if encrypted.format_version != CURRENT_KV_FORMAT_VERSION {
            return Err(CryptoError::KvFormatVersionMismatch {
                expected: CURRENT_KV_FORMAT_VERSION,
                found: encrypted.format_version,
            });
        }

        self.registry
            .decrypt_with_group(segment_id, &encrypted.ciphertext, &encrypted.nonce)
    }
}

impl KvCipher for KvSegmentCipher {
    fn seal(&self, plaintext: &[u8]) -> contextra_types::Result<Vec<u8>> {
        self.key_manager.seal(plaintext)
    }

    fn open(&self, ciphertext: &[u8]) -> contextra_types::Result<Vec<u8>> {
        self.key_manager.open(ciphertext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_fingerprint(id: &str) -> ModelFingerprint {
        ModelFingerprint::new([0x42u8; 32], id, "Q4_K_M")
    }

    #[test]
    fn test_kv_segment_cipher_encrypt_decrypt_roundtrip() {
        let master_km = KeyManager::try_new("master-passphrase", b"master-salt").unwrap();
        let cipher = KvSegmentCipher::ephemeral(master_km);

        let tenant_id = TenantId::try_new(101).unwrap();
        let fp = dummy_fingerprint("model-v1");
        let plaintext = b"KV tensor keys and values payload data";

        let encrypted = cipher.encrypt(tenant_id, fp.clone(), plaintext).unwrap();
        assert_eq!(encrypted.tenant_id, tenant_id);
        assert_eq!(encrypted.model_fingerprint, fp);

        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_tenant_id_fails_decryption() {
        let master_km = KeyManager::try_new("master-passphrase", b"master-salt").unwrap();
        let cipher = KvSegmentCipher::ephemeral(master_km);

        let tenant_a = TenantId::try_new(101).unwrap();
        let tenant_b = TenantId::try_new(202).unwrap();
        let fp = dummy_fingerprint("model-v1");
        let plaintext = b"tenant A confidential payload";

        let mut encrypted = cipher.encrypt(tenant_a, fp, plaintext).unwrap();

        // Attempt decryption with tampered tenant_id in layer header
        encrypted.tenant_id = tenant_b;
        let res = cipher.decrypt(&encrypted);
        assert!(
            res.is_err(),
            "Decryption with mismatching tenant_id MUST fail authentication"
        );
    }

    #[test]
    fn test_wrong_model_fingerprint_fails_decryption() {
        let master_km = KeyManager::try_new("master-passphrase", b"master-salt").unwrap();
        let cipher = KvSegmentCipher::ephemeral(master_km);

        let tenant = TenantId::try_new(101).unwrap();
        let fp_q4 = ModelFingerprint::new([0x11u8; 32], "llama-3", "Q4_K_M");
        let fp_q8 = ModelFingerprint::new([0x11u8; 32], "llama-3", "Q8_0");
        let plaintext = b"Q4 model KV cache data";

        let mut encrypted = cipher.encrypt(tenant, fp_q4, plaintext).unwrap();

        // Attempt decryption with different quantization tier
        encrypted.model_fingerprint = fp_q8;
        let res = cipher.decrypt(&encrypted);
        assert!(
            res.is_err(),
            "Decryption with mismatching model_fingerprint MUST fail authentication"
        );
    }

    #[test]
    fn test_nonce_freshness_identical_plaintext() {
        let master_km = KeyManager::try_new("master-passphrase", b"master-salt").unwrap();
        let cipher = KvSegmentCipher::ephemeral(master_km);

        let tenant = TenantId::try_new(101).unwrap();
        let fp = dummy_fingerprint("model-v1");
        let plaintext = b"identical plaintext payload";

        let enc1 = cipher.encrypt(tenant, fp.clone(), plaintext).unwrap();
        let enc2 = cipher.encrypt(tenant, fp, plaintext).unwrap();

        assert_ne!(
            enc1.nonce, enc2.nonce,
            "Two encrypt calls for identical plaintext MUST produce distinct nonces"
        );
        assert_ne!(
            enc1.ciphertext, enc2.ciphertext,
            "Two encrypt calls for identical plaintext MUST produce distinct ciphertexts"
        );
    }

    #[test]
    fn test_invalid_format_version_returns_version_mismatch_error() {
        let master_km = KeyManager::try_new("master-passphrase", b"master-salt").unwrap();
        let cipher = KvSegmentCipher::ephemeral(master_km);

        let tenant_id = TenantId::try_new(101).unwrap();
        let fp = dummy_fingerprint("model-v1");
        let plaintext = b"KV tensor payload data";

        let mut encrypted = cipher.encrypt(tenant_id, fp, plaintext).unwrap();
        assert_eq!(encrypted.format_version, CURRENT_KV_FORMAT_VERSION);

        // Tamper with format_version (e.g., set to old version 1 or future version 99)
        encrypted.format_version = 1;
        let res = cipher.decrypt(&encrypted);

        match res {
            Err(CryptoError::KvFormatVersionMismatch { expected, found }) => {
                assert_eq!(expected, CURRENT_KV_FORMAT_VERSION);
                assert_eq!(found, 1);
            }
            res => panic!("Expected KvFormatVersionMismatch error, got: {:?}", res),
        }
    }

    #[test]
    fn test_format_version_mismatch_returns_specific_error() {
        let master_km = KeyManager::try_new("master-passphrase", b"master-salt").unwrap();
        let cipher = KvSegmentCipher::ephemeral(master_km);
        let tenant = TenantId::try_new(1).unwrap();
        let fp = dummy_fingerprint("model-v1");
        let plaintext = b"test payload";

        let mut layer = cipher.encrypt(tenant, fp, plaintext).unwrap();
        // Tamper with format_version to simulate a v1-format segment
        layer.format_version = 1;

        let err = cipher.decrypt(&layer).unwrap_err();
        assert!(
            matches!(
                err,
                CryptoError::KvFormatVersionMismatch {
                    expected: 2,
                    found: 1
                }
            ),
            "Expected KvFormatVersionMismatch, got: {err:?}"
        );
    }

    #[test]
    fn test_kv_cipher_revocation_survives_registry_reconstruction() {
        use contextra_ports::SystemClock;
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let log_path = temp_dir.path().join("kv_revocation.log");

        let sk = SigningKey::generate(&mut OsRng);
        let vk = sk.verifying_key();
        let clock = Arc::new(SystemClock::new());

        let log = RevocationLog::open_or_create(&log_path, clock.clone(), Some(sk), vk)
            .expect("Failed to create persistent revocation log");
        let log_arc = Arc::new(log);

        let master_km1 = KeyManager::try_new("master-passphrase", b"master-salt").unwrap();
        let cipher1 = KvSegmentCipher::new(master_km1, log_arc.clone());

        let tenant_id = TenantId::try_new(101).unwrap();
        let fp = dummy_fingerprint("model-v1");
        let group_id = 888;
        let plaintext = b"Sensitive persistent group payload";

        let encrypted = cipher1
            .encrypt_with_version(tenant_id, group_id, 1, fp.clone(), plaintext)
            .unwrap();

        // Revoke group_id on cipher1
        let revoked = cipher1
            .registry()
            .revoke_group(group_id)
            .expect("revoke_group failed");
        assert!(revoked, "Group 888 must be revoked on cipher1");
        assert!(cipher1.registry().is_group_revoked(group_id));

        // Drop cipher1 and log_arc simulating process shutdown / registry destruction
        drop(cipher1);
        drop(log_arc);

        // Re-open persistent revocation log and construct a new KvSegmentCipher with new KeyRegistry
        let reopened_log = RevocationLog::open_or_create(&log_path, clock, None, vk)
            .expect("Failed to reopen persistent revocation log");
        let master_km2 = KeyManager::try_new("master-passphrase", b"master-salt").unwrap();
        let cipher2 = KvSegmentCipher::new(master_km2, Arc::new(reopened_log));

        // Assert group 888 is still revoked in the reconstructed cipher's registry
        assert!(
            cipher2.registry().is_group_revoked(group_id),
            "Revocation MUST survive registry reconstruction on persisted log"
        );

        // Decryption of encrypted layer on new reconstructed cipher MUST fail due to key revocation
        let decrypt_res = cipher2.decrypt_with_version(&encrypted, group_id, 1);
        assert!(
            decrypt_res.is_err(),
            "Decryption on reconstructed cipher MUST fail for revoked group"
        );
    }

    #[test]
    fn test_single_shredding_path_consolidation_and_revocation() {
        // PROOF OF CONSOLIDATION: KvSegmentCipher routes segment encryption through
        // KeyRegistry envelope crypto-shredding (the single consolidated shredding path).
        let master_km = KeyManager::try_new("master-passphrase", b"master-salt").unwrap();
        let cipher = KvSegmentCipher::ephemeral(master_km);
        let tenant_id = TenantId::try_new(101).unwrap();
        let fp = dummy_fingerprint("model-v1");
        let segment_id = 42;
        let plaintext = b"Segment tensor data payload";

        let encrypted = cipher
            .encrypt_with_version(tenant_id, segment_id, 1, fp.clone(), plaintext)
            .unwrap();

        let decrypted = cipher
            .decrypt_with_version(&encrypted, segment_id, 1)
            .unwrap();
        assert_eq!(decrypted, plaintext);

        // Revoke sub-key in the consolidated KeyRegistry
        let revoked = cipher.registry().revoke_subkey(segment_id).unwrap();
        assert!(revoked, "Sub-key revocation in KeyRegistry must succeed");

        // Decryption now fails because the single shredding path has been revoked
        let res = cipher.decrypt_with_version(&encrypted, segment_id, 1);
        assert!(
            res.is_err(),
            "Decryption of segment after KeyRegistry revocation MUST fail"
        );
    }
}
