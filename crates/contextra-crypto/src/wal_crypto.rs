// FILE-CONTEXT
// ZWECK: Transparent encrypted WAL chunk provider and constant-time HMAC-SHA256 integrity verifier.
// INVARIANTEN: EncryptedWal prepends 12-byte nonce to ciphertext. IntegrityVerifier checks sequence HMAC chain in constant time.
// NICHT-OFFENSICHTLICH: Subtle constant-time byte comparisons prevent timing attacks. All operations lock-free & I/O-free.
// HOTSPOTS: [35-180]
// STAND: TS:2026-08-31T21:13:05Z (SESSION: 8427f167)

//! Encryption at Rest layer for LSM/WAL components (WP-3.2)
//!
//! Secures blocks of data transparently via ChaCha20Poly1305 / AES-GCM-SIV.

#![forbid(unsafe_code)]

use std::sync::Arc;
use zeroize::{Zeroize, Zeroizing};

use crate::crypto::KeyManager;
use crate::error::{CryptoError, Result};

/// Key Management Strategy interface for data key derivation, key wrapping/unwrapping, and key rotation.
pub trait KmsProvider: Send + Sync {
    /// Retrieves the Data Encryption Key (DEK) zeroized on drop.
    fn get_key(&self) -> Result<Zeroizing<Vec<u8>>>;

    /// Returns the active key identifier or version alias.
    fn active_key_id(&self) -> &str;

    /// Derives a raw data key bound to the specified `key_id`.
    fn derive_data_key(&self, key_id: &[u8]) -> Result<Zeroizing<Vec<u8>>>;

    /// Wraps (encrypts) plaintext data or a raw key using the key bound to `key_id`.
    fn wrap_key(&self, key_id: &[u8], plaintext: &[u8]) -> Result<Vec<u8>>;

    /// Unwraps (decrypts) wrapped ciphertext data or key using the key bound to `key_id`.
    fn unwrap_key(&self, key_id: &[u8], wrapped: &[u8]) -> Result<Zeroizing<Vec<u8>>>;

    /// Rotates the active master key or key version and returns the new active key ID.
    fn rotate_key(&mut self) -> Result<String>;
}

/// Standard local implementation of [`KmsProvider`] wrapping [`KeyManager`].
///
/// Provides thread-safe, zeroized key derivation, wrapping, unwrapping, and rotation.
pub struct LocalKeyManagerKms {
    key_manager: KeyManager,
    key_id: String,
    rotation_counter: u64,
}

impl LocalKeyManagerKms {
    /// Creates a new `LocalKeyManagerKms` wrapping an existing `KeyManager`.
    pub fn new(key_manager: KeyManager, key_id: impl Into<String>) -> Self {
        Self {
            key_manager,
            key_id: key_id.into(),
            rotation_counter: 0,
        }
    }

    /// Accesses the underlying `KeyManager`.
    pub fn key_manager(&self) -> &KeyManager {
        &self.key_manager
    }
}

impl KmsProvider for LocalKeyManagerKms {
    fn get_key(&self) -> Result<Zeroizing<Vec<u8>>> {
        Ok(self.key_manager.export_dek())
    }

    fn active_key_id(&self) -> &str {
        &self.key_id
    }

    fn derive_data_key(&self, key_id: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
        if key_id.is_empty() {
            return Err(CryptoError::InvalidInput(
                "key_id cannot be empty for data key derivation".to_string(),
            ));
        }
        let sub_km = self.key_manager.derive_file_key(key_id)?;
        Ok(sub_km.export_dek())
    }

    fn wrap_key(&self, key_id: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
        let (ciphertext, nonce) = if key_id.is_empty() {
            self.key_manager.encrypt_auto_nonce(plaintext)?
        } else {
            let sub_km = self.key_manager.derive_file_key(key_id)?;
            sub_km.encrypt_auto_nonce(plaintext)?
        };
        let mut out = Vec::with_capacity(12 + ciphertext.len());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    fn unwrap_key(&self, key_id: &[u8], wrapped: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
        if wrapped.len() < AES_GCM_SIV_NONCE_LEN {
            return Err(CryptoError::InvalidInput(
                "Wrapped key payload too short for 12-byte nonce".into(),
            ));
        }
        let (nonce_bytes, ciphertext) = wrapped.split_at(AES_GCM_SIV_NONCE_LEN);
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(nonce_bytes);

        let plaintext = if key_id.is_empty() {
            self.key_manager.decrypt_auto_nonce(ciphertext, &nonce)?
        } else {
            let sub_km = self.key_manager.derive_file_key(key_id)?;
            sub_km.decrypt_auto_nonce(ciphertext, &nonce)?
        };
        Ok(Zeroizing::new(plaintext))
    }

    fn rotate_key(&mut self) -> Result<String> {
        self.rotation_counter += 1;
        let rotation_info = format!("rotation-epoch-{}", self.rotation_counter);
        self.key_manager = self.key_manager.derive_file_key(rotation_info.as_bytes())?;
        let base_id = self
            .key_id
            .split("-v")
            .next()
            .unwrap_or(&self.key_id)
            .to_string();
        self.key_id = format!("{}-v{}", base_id, self.rotation_counter);
        Ok(self.key_id.clone())
    }
}

#[cfg(feature = "cloud-egress-guard")]
/// Egress guard for inspecting and sanitizing encrypted payload blocks prior to cloud egress.
#[derive(Debug, Clone)]
pub struct CloudEgressGuard {
    max_egress_bytes: usize,
}

#[cfg(feature = "cloud-egress-guard")]
impl Default for CloudEgressGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "cloud-egress-guard")]
impl CloudEgressGuard {
    /// Creates a new `CloudEgressGuard` with a default maximum payload limit of 10 MB.
    pub fn new() -> Self {
        Self {
            max_egress_bytes: 10 * 1024 * 1024,
        }
    }

    /// Creates a custom `CloudEgressGuard` with a specified maximum payload size.
    pub fn with_max_bytes(max_egress_bytes: usize) -> Self {
        Self { max_egress_bytes }
    }

    /// Validates that an egress payload meets size and structural integrity bounds.
    pub fn validate_egress_payload(&self, payload: &[u8]) -> Result<()> {
        if payload.is_empty() {
            return Err(CryptoError::InvalidInput(
                "Egress payload cannot be empty".to_string(),
            ));
        }
        if payload.len() > self.max_egress_bytes {
            return Err(CryptoError::InvalidInput(format!(
                "Egress payload length {} exceeds maximum allowed limit of {} bytes",
                payload.len(),
                self.max_egress_bytes
            )));
        }
        Ok(())
    }
}

/// Encrypted WAL chunk provider that handles transparent encryption/decryption of WAL payloads via KMS.
///
/// # Invariants
/// - Delegates encryption and key management to a [`KmsProvider`].
/// - Prepends a unique 12-byte random nonce to every encrypted chunk output.
///
/// # Usage
/// Use when writing or reading encrypted WAL logs at rest. Call `encrypt_chunk` to wrap
/// plain payload bytes into an encrypted chunk with prepended nonce, and `decrypt_chunk`
/// to recover the original payload.
///
/// # Errors
/// Emits `CryptoError::Crypto` if encryption/decryption fails, key derivation fails,
/// or if chunk length is less than 12 bytes during decryption.
pub struct EncryptedWal {
    kms: Arc<dyn KmsProvider>,
}

const MAX_CHUNK_SIZE: usize = 100 * 1024 * 1024; // 100 MB
const AES_GCM_SIV_NONCE_LEN: usize = 12;
const AES_GCM_SIV_TAG_LEN: usize = 16;
const MAX_ENCRYPTED_CHUNK_SIZE: usize =
    MAX_CHUNK_SIZE + AES_GCM_SIV_NONCE_LEN + AES_GCM_SIV_TAG_LEN;

impl EncryptedWal {
    /// Creates a new EncryptedWal with per-file key derivation via LocalKeyManagerKms to prevent nonce-reuse.
    /// `file_id` (e.g., filename) is used to derive a unique sub-key for this stream.
    pub fn new(key_manager: KeyManager, file_id: &[u8]) -> Result<Self> {
        if file_id.is_empty() {
            return Err(CryptoError::InvalidInput(
                "file_id cannot be empty".to_string(),
            ));
        }
        if file_id.len() > 10_000 {
            return Err(CryptoError::InvalidInput(format!(
                "file_id length {} exceeds maximum allowed bound of 10000 bytes",
                file_id.len()
            )));
        }
        let sub_km = key_manager.derive_file_key(file_id)?;
        let key_id_str = String::from_utf8_lossy(file_id).to_string();
        let kms = Arc::new(LocalKeyManagerKms::new(sub_km, key_id_str));
        Ok(Self { kms })
    }

    /// Creates a new EncryptedWal using an arbitrary KmsProvider.
    pub fn with_kms(kms: Arc<dyn KmsProvider>) -> Self {
        Self { kms }
    }

    /// Returns a reference to the inner KMS provider.
    pub fn kms(&self) -> &Arc<dyn KmsProvider> {
        &self.kms
    }

    /// Wraps the internal WAL chunk in AES-256-GCM-SIV stream.
    /// Prepends the 12-byte nonce to the encrypted ciphertext.
    pub fn encrypt_chunk(&self, payload: &[u8]) -> Result<Vec<u8>> {
        if payload.len() > MAX_CHUNK_SIZE {
            return Err(CryptoError::InvalidInput(format!(
                "Payload size {} exceeds maximum permitted limit of {} bytes",
                payload.len(),
                MAX_CHUNK_SIZE
            )));
        }
        self.kms.wrap_key(b"", payload)
    }

    /// Decrypts the WAL chunk by extracting the prepended 12-byte nonce from the data.
    pub fn decrypt_chunk(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < AES_GCM_SIV_NONCE_LEN {
            return Err(CryptoError::InvalidInput(
                "Encrypted WAL chunk too short for 12-byte nonce".into(),
            ));
        }
        if data.len() > MAX_ENCRYPTED_CHUNK_SIZE {
            return Err(CryptoError::InvalidInput(format!(
                "Encrypted chunk size {} exceeds maximum permitted limit of {} bytes",
                data.len(),
                MAX_ENCRYPTED_CHUNK_SIZE
            )));
        }
        let decrypted_zeroizing = self.kms.unwrap_key(b"", data)?;
        Ok(decrypted_zeroizing.to_vec())
    }
}

use hmac::{Hmac, Mac};
use sha2::Sha256;

/// Stateful wrapper around HMAC-SHA256 initialized with WAL domain separation.
///
/// # Invariants
/// - Initialized with `b"contextra-wal-v1"` domain separation string as the first updated block,
///   preventing cross-context HMAC collisions if the integrity key is reused elsewhere.
///
/// # Usage
/// Use to compute deterministic 32-byte HMAC-SHA256 checksums over WAL entry fields and
/// previous chain links during WAL appends or verification.
///
/// # Errors
/// Emits `CryptoError::Crypto` if key initialization fails.
pub struct WalHmac {
    mac: Hmac<Sha256>,
}

impl WalHmac {
    pub fn new(integrity_key: &[u8]) -> Result<Self> {
        if integrity_key.is_empty() {
            return Err(CryptoError::InvalidInput("Key cannot be empty".to_string()));
        }
        if integrity_key.len() > 10_000 {
            return Err(CryptoError::InvalidInput(format!(
                "integrity_key length {} exceeds maximum allowed bound of 10000 bytes",
                integrity_key.len()
            )));
        }
        let mut mac = Hmac::<Sha256>::new_from_slice(integrity_key)
            .map_err(|e| CryptoError::Crypto(format!("HMAC key error: {}", e)))?;
        mac.update(b"contextra-wal-v1");
        Ok(Self { mac })
    }

    pub fn update(&mut self, data: &[u8]) {
        self.mac.update(data);
    }

    pub fn finalize(self) -> [u8; 32] {
        self.mac.finalize().into_bytes().into()
    }
}

/// Immutable snapshot of a WAL entry used for cryptographic integrity verification.
///
/// # Invariants
/// - Captures entry fields (`seq_no`, `op_type`, `key`, `value`, `checksum`, `prev_hmac`)
///   required to recompute the entry's HMAC-SHA256 checksum and verify hash-chain continuity.
///
/// # Usage
/// Passed to `IntegrityVerifier::verify_and_update()` during WAL replay or recovery.
#[derive(Debug, Clone, Zeroize)]
#[zeroize(drop)]
pub struct WalEntrySnapshot {
    pub tx_id: u64,
    pub seq_no: u64,
    pub op_type: u8, // 0: Put, 1: Delete, 2: TxEnd
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub checksum: [u8; 32],
    pub prev_hmac: [u8; 32],
}

/// Stateful verifier for WAL HMAC-SHA256 hash chains.
///
/// # Invariants
/// - Maintains running `last_hmac` state across entries to enforce sequential hash-chain continuity.
/// - Performs constant-time comparison (`subtle::ConstantTimeEq`) of checksums and `prev_hmac`
///   to prevent timing side-channel attacks.
/// - Zeroizes key material on drop to prevent secret key leakage from memory.
/// - Any HMAC discrepancy or broken chain link immediately halts verification.
///
/// # Usage
/// Instantiate with the WAL's integrity key at the start of replay and call `verify_and_update()`
/// sequentially for each deserialized WAL entry.
///
/// # Errors
/// Emits `CryptoError::WalCorruption` immediately if `checksum` or `prev_hmac` does not match,
/// or `CryptoError::Crypto` if HMAC initialization fails.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct IntegrityVerifier {
    last_hmac: [u8; 32],
    integrity_key: Zeroizing<Vec<u8>>,
    last_seq_no: Option<u64>,
}

impl IntegrityVerifier {
    pub fn new(integrity_key: &[u8]) -> Self {
        Self {
            last_hmac: [0u8; 32],
            integrity_key: Zeroizing::new(integrity_key.to_vec()),
            last_seq_no: None,
        }
    }

    /// Übernimmt einen bereits bekannten `last_hmac`-Kettenzustand, z. B. beim
    /// Wechsel von einem Normal- auf einen Legacy-Key-Verifier MITTEN in einem
    /// Batch, damit die HMAC-Chain-Kontinuität über den Verifier-Wechsel hinweg
    /// gewahrt bleibt (verhindert einen Split-Brain-Zustand, bei dem ein frisch
    /// initialisierter Verifier fälschlich `prev_hmac == [0u8;32]` als gültigen
    /// Kettenanfang akzeptiert, obwohl bereits vorherige Einträge verifiziert wurden).
    pub fn set_last_hmac(&mut self, hmac: [u8; 32]) {
        self.last_hmac = hmac;
    }

    /// Gibt den aktuellen `last_hmac`-Kettenzustand zurück (für Verifier-Handoff).
    pub fn last_hmac_snapshot(&self) -> [u8; 32] {
        self.last_hmac
    }

    /// Übernimmt eine bereits bekannte `last_seq_no` für den Verifier-Handoff.
    pub fn set_last_seq_no(&mut self, seq_no: Option<u64>) {
        self.last_seq_no = seq_no;
    }

    /// Gibt die aktuell bekannte `last_seq_no` zurück (für Verifier-Handoff).
    pub fn last_seq_no_snapshot(&self) -> Option<u64> {
        self.last_seq_no
    }

    /// Verifies a V3 entry (with tx_id and length prefixes) and updates the chain state.
    pub fn verify_and_update_v3(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()> {
        if entry.op_type > 2 {
            return Err(CryptoError::wal_corruption(
                offset,
                format!("Unsupported op_type {}", entry.op_type),
            ));
        }

        if let Some(last_seq) = self.last_seq_no {
            if entry.seq_no <= last_seq {
                return Err(CryptoError::wal_corruption(
                    offset,
                    format!(
                        "Duplicate or non-monotonic sequence number {} (last: {})",
                        entry.seq_no, last_seq
                    ),
                ));
            }
            if entry.seq_no > last_seq + 1 {
                return Err(CryptoError::wal_corruption(
                    offset,
                    format!(
                        "Sequence gap detected: expected {}, got {}",
                        last_seq + 1,
                        entry.seq_no
                    ),
                ));
            }
        }

        let mut mac = WalHmac::new(&self.integrity_key)?;
        mac.update(&self.last_hmac);
        mac.update(&entry.seq_no.to_le_bytes());

        let tx_id_bytes = entry.tx_id.to_le_bytes();
        mac.update(&tx_id_bytes);

        if entry.op_type == 0 {
            // Put
            mac.update(&[0u8]);
            mac.update(&(entry.key.len() as u32).to_le_bytes());
            mac.update(&entry.key);
            mac.update(&(entry.value.len() as u32).to_le_bytes());
            mac.update(&entry.value);
        } else if entry.op_type == 1 {
            // Delete
            mac.update(&[1u8]);
            mac.update(&(entry.key.len() as u32).to_le_bytes());
            mac.update(&entry.key);
        } else if entry.op_type == 2 {
            // TxEnd
            mac.update(&[2u8]);
            let committed = if entry.value.first().copied().unwrap_or(0) != 0 {
                1u8
            } else {
                0u8
            };
            mac.update(&[committed]);
        }

        use subtle::ConstantTimeEq;
        let computed = mac.finalize();
        if computed.ct_eq(&entry.checksum).unwrap_u8() == 0
            || entry.prev_hmac.ct_eq(&self.last_hmac).unwrap_u8() == 0
        {
            return Err(CryptoError::wal_corruption(
                offset,
                format!("HMAC mismatch for seq {}", entry.seq_no),
            ));
        }

        self.last_hmac = computed;
        self.last_seq_no = Some(entry.seq_no);
        Ok(())
    }

    /// Verifies a V2 entry (legacy without tx_id and without length prefixes) and updates the chain state.
    ///
    /// # Sicherheitshinweis & Kollisionslücke (Architektur-Audit F2 / Spec §22.2b)
    /// - Dieser Pfad verwendet KEINE längenpräfixierte Feldkodierung. Für `op_type == 0` (Put) werden `entry.key` und
    ///   `entry.value` ohne Längenpräfix direkt hintereinander in den HMAC eingespeist. Bei variabler Key/Value-Aufteilung
    ///   (z. B. `key = b"ab"`, `value = b"c"` vs. `key = b"a"`, `value = b"bc"`) ist dieser Pfad kollisionsanfällig.
    /// - Dieser Pfad darf NUR zum Lesen/Reproduzieren bereits vor Einführung von `WalVersion::V3` geschriebener
    ///   Bestandsdateien verwendet werden und darf NIEMALS für neu geschriebene WAL-Segmente aufgerufen werden.
    /// - Referenz: Spec §22.2b und Architektur-Audit Befund F2 (2026-09-26).
    #[deprecated(
        note = "Nur für Legacy-Replay von WalVersion::V2-Bestandsdateien. Nicht für neue WAL-Segmente verwenden — siehe Doc-Kommentar. Audit-Referenz: F2."
    )]
    pub fn verify_and_update_v2(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()> {
        if entry.op_type > 2 {
            return Err(CryptoError::wal_corruption(
                offset,
                format!("Unsupported op_type {}", entry.op_type),
            ));
        }

        if let Some(last_seq) = self.last_seq_no {
            if entry.seq_no <= last_seq {
                return Err(CryptoError::wal_corruption(
                    offset,
                    format!(
                        "Duplicate or non-monotonic sequence number {} (last: {})",
                        entry.seq_no, last_seq
                    ),
                ));
            }
            if entry.seq_no > last_seq + 1 {
                return Err(CryptoError::wal_corruption(
                    offset,
                    format!(
                        "Sequence gap detected: expected {}, got {}",
                        last_seq + 1,
                        entry.seq_no
                    ),
                ));
            }
        }

        let mut mac = WalHmac::new(&self.integrity_key)?;
        mac.update(&self.last_hmac);
        mac.update(&entry.seq_no.to_le_bytes());

        mac.update(&[entry.op_type]);
        if entry.op_type == 0 {
            // Put
            mac.update(&entry.key);
            mac.update(&entry.value);
        } else if entry.op_type == 1 {
            // Delete
            mac.update(&entry.key);
        } else if entry.op_type == 2 {
            // TxEnd
            let committed = if entry.value.first().copied().unwrap_or(0) != 0 {
                1u8
            } else {
                0u8
            };
            mac.update(&[committed]);
        }

        use subtle::ConstantTimeEq;
        let computed = mac.finalize();
        if computed.ct_eq(&entry.checksum).unwrap_u8() == 0
            || entry.prev_hmac.ct_eq(&self.last_hmac).unwrap_u8() == 0
        {
            return Err(CryptoError::wal_corruption(
                offset,
                format!("HMAC mismatch for seq {}", entry.seq_no),
            ));
        }

        self.last_hmac = computed;
        self.last_seq_no = Some(entry.seq_no);
        Ok(())
    }

    /// Updates the chain state for legacy V1 entries without HMAC verification.
    pub fn skip_hmac_verify_legacy(&mut self, entry: &WalEntrySnapshot) {
        self.last_hmac = entry.checksum;
        self.last_seq_no = Some(entry.seq_no);
    }

    /// Default verification delegating to V3 verification.
    pub fn verify_and_update(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()> {
        self.verify_and_update_v3(entry, offset)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    // INTENT: IntegrityVerifier lifecycle and HMAC chain validation verified.
    #[test]
    fn test_wal_hmac_basic() {
        let key = b"test-key-32-bytes-long-----------";
        let mut hmac = WalHmac::new(key).unwrap(); // unwrap
        hmac.update(b"data");
        let result = hmac.finalize();
        assert_ne!(result, [0u8; 32]);
    }

    #[test]
    fn test_integrity_verifier_chain() {
        let key = b"test-key-32-bytes-long-----------";
        let mut verifier = IntegrityVerifier::new(key);

        // entry 1
        let e1 = create_entry(key, [0u8; 32], 100, 0, b"k1", b"v1");
        let checksum1 = e1.checksum;

        verifier.verify_and_update(&e1, 100).expect("e1 valid"); // expect

        // entry 2
        let e2 = create_entry(key, checksum1, 101, 1, b"k1", b"");
        let checksum2 = e2.checksum;

        verifier.verify_and_update(&e2, 200).expect("e2 valid"); // expect

        // entry 3 (corrupt)
        let e3 = WalEntrySnapshot {
            tx_id: 3,
            seq_no: 102,
            op_type: 1,
            key: b"k1".to_vec(),
            value: Vec::new(),
            checksum: [0u8; 32],
            prev_hmac: checksum2,
        };
        let err = verifier.verify_and_update(&e3, 300).unwrap_err();
        if let CryptoError::WalCorruption { offset, .. } = err {
            assert_eq!(offset, 300);
        } else {
            panic!("Expected WalCorruption with offset");
        }
    }

    #[tokio::test]
    async fn test_encrypted_wal_roundtrip() -> Result<()> {
        let km = crate::crypto::KeyManager::try_new("test-pass", b"salt1")?;
        let wal = EncryptedWal::new(km, b"test-wal.log")?;
        let data = b"wal-entry-data-to-encrypt";
        let encrypted = wal.encrypt_chunk(data)?;
        assert_ne!(encrypted.as_slice(), data);

        let decrypted = wal.decrypt_chunk(&encrypted)?;
        assert_eq!(decrypted.as_slice(), data);

        Ok(())
    }

    #[tokio::test]
    async fn test_encrypted_wal_1mb_roundtrip() -> Result<()> {
        let km = crate::crypto::KeyManager::try_new("test-pass-1mb", b"salt1234")?;
        let wal = EncryptedWal::new(km, b"test-wal-1mb.log")?;

        // 1MB payload
        let size = 1024 * 1024;
        let mut payload = vec![0u8; size];
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte = (i % 251) as u8;
        }

        // Encrypt (prepends nonce)
        let encrypted = wal.encrypt_chunk(&payload)?;
        assert!(encrypted.len() >= 12 + size);
        assert_ne!(&encrypted[12..], &payload[..]);

        // Simulate serialization/deserialization over bytes
        let serialized_bytes = encrypted.clone();
        let deserialized_bytes = serialized_bytes.as_slice();

        // Decrypt
        let decrypted = wal.decrypt_chunk(deserialized_bytes)?;
        assert_eq!(decrypted.len(), payload.len());
        assert_eq!(decrypted, payload);

        Ok(())
    }

    fn create_entry(
        key: &[u8],
        prev_hmac: [u8; 32],
        seq_no: u64,
        op_type: u8,
        k: &[u8],
        v: &[u8],
    ) -> WalEntrySnapshot {
        let tx_id = seq_no;
        let mut hmac = WalHmac::new(key).expect("hmac init"); // expect
        hmac.update(&prev_hmac);
        hmac.update(&seq_no.to_le_bytes());
        hmac.update(&tx_id.to_le_bytes());
        if op_type == 0 {
            hmac.update(&[0u8]);
            hmac.update(&(k.len() as u32).to_le_bytes());
            hmac.update(k);
            hmac.update(&(v.len() as u32).to_le_bytes());
            hmac.update(v);
        } else {
            hmac.update(&[1u8]);
            hmac.update(&(k.len() as u32).to_le_bytes());
            hmac.update(k);
        }
        let checksum = hmac.finalize();
        WalEntrySnapshot {
            tx_id,
            seq_no,
            op_type,
            key: k.to_vec(),
            value: v.to_vec(),
            checksum,
            prev_hmac,
        }
    }

    #[test]
    fn test_hmac_tamper_detection() {
        let key = b"integrity-key-32-bytes-long-----";
        let e1 = create_entry(key, [0u8; 32], 1, 0, b"key1", b"val1");
        let mut tampered_e1 = e1.clone();
        tampered_e1.value = b"tampered".to_vec();

        let mut verifier = IntegrityVerifier::new(key);
        assert!(verifier.verify_and_update(&tampered_e1, 10).is_err());
    }

    #[test]
    fn test_single_bit_checksum_corruption() {
        let key = b"integrity-key-32-bytes-long-----";
        let entry = create_entry(key, [0u8; 32], 1, 0, b"key1", b"val1");
        let mut corrupted_entry = entry.clone();
        corrupted_entry.checksum[0] ^= 0x01; // Flip a single bit in checksum

        let mut verifier = IntegrityVerifier::new(key);
        let err = verifier
            .verify_and_update(&corrupted_entry, 100)
            .unwrap_err();
        if let CryptoError::WalCorruption { offset, reason, .. } = err {
            assert_eq!(offset, 100);
            assert!(reason.contains("HMAC mismatch"));
        } else {
            panic!("Expected WalCorruption error");
        }
    }

    #[test]
    fn hmac_chain_detects_tampered_entry() {
        let key = b"integrity-key-32-bytes-long-----";
        let e1 = create_entry(key, [0u8; 32], 1, 0, b"key1", b"val1");
        let e2 = create_entry(key, e1.checksum, 2, 0, b"key2", b"val2");

        // Tamper with entry 2's payload value
        let mut tampered_e2 = e2.clone();
        tampered_e2.value = b"tampered_val2".to_vec();

        let e3 = create_entry(key, e2.checksum, 3, 0, b"key3", b"val3");

        let mut verifier = IntegrityVerifier::new(key);
        verifier.verify_and_update(&e1, 10).expect("e1 valid"); // expect

        // Verification of tampered entry 2 must fail
        assert!(verifier.verify_and_update(&tampered_e2, 20).is_err());

        // Verification of entry 3 with original e2's prev_hmac must fail because verifier chain was not updated with tampered e2
        assert!(verifier.verify_and_update(&e3, 30).is_err());
    }

    #[test]
    fn hmac_chain_detects_deleted_entry() {
        let key = b"integrity-key-32-bytes-long-----";
        let e1 = create_entry(key, [0u8; 32], 1, 0, b"key1", b"val1");
        let e2 = create_entry(key, e1.checksum, 2, 0, b"key2", b"val2");
        let e3 = create_entry(key, e2.checksum, 3, 0, b"key3", b"val3");

        let mut verifier = IntegrityVerifier::new(key);
        verifier.verify_and_update(&e1, 10).expect("e1 valid"); // expect

        // Skip entry 2 (removed entry) and attempt to verify entry 3 directly
        let err = verifier.verify_and_update(&e3, 30).unwrap_err();
        if let CryptoError::WalCorruption { offset, reason, .. } = err {
            assert_eq!(offset, 30);
            assert!(reason.contains("Sequence gap") || reason.contains("HMAC mismatch"));
        } else {
            panic!("Expected WalCorruption error");
        }
    }

    #[test]
    fn test_encrypted_wal_new_empty_id() {
        let km = KeyManager::try_new("test-pass", b"salt1").expect("km"); // expect
        let res = EncryptedWal::new(km, b"");
        assert!(matches!(res, Err(CryptoError::InvalidInput(_))));
    }

    #[test]
    fn test_wal_hmac_oversized_key() {
        let oversized_key = vec![0x55u8; 10_001];
        let res = WalHmac::new(&oversized_key);
        assert!(matches!(res, Err(CryptoError::InvalidInput(_))));
    }

    #[test]
    fn test_encrypt_chunk_max_boundary() {
        let km = KeyManager::try_new("test-pass", b"salt1").expect("km"); // expect
        let wal = EncryptedWal::new(km, b"wal.log").expect("wal"); // expect
        let payload = vec![0xABu8; MAX_CHUNK_SIZE]; // Exactly 100 MB
        let encrypted = wal
            .encrypt_chunk(&payload)
            .expect("100MB payload encryption");
        assert_eq!(encrypted.len(), 12 + MAX_CHUNK_SIZE + 16);

        let decrypted = wal
            .decrypt_chunk(&encrypted)
            .expect("100MB payload decryption");
        assert_eq!(decrypted.len(), MAX_CHUNK_SIZE);
    }

    #[test]
    fn test_encrypt_chunk_oversized() {
        let km = KeyManager::try_new("test-pass", b"salt1").expect("km"); // expect
        let wal = EncryptedWal::new(km, b"wal.log").expect("wal"); // expect
        let payload = vec![0u8; MAX_CHUNK_SIZE + 1];
        let res = wal.encrypt_chunk(&payload);
        assert!(matches!(res, Err(CryptoError::InvalidInput(_))));
    }

    #[test]
    fn test_decrypt_chunk_too_short() {
        let km = KeyManager::try_new("test-pass", b"salt1").expect("km"); // expect
        let wal = EncryptedWal::new(km, b"wal.log").expect("wal"); // expect
        let res = wal.decrypt_chunk(&[0u8; 11]);
        assert!(matches!(res, Err(CryptoError::InvalidInput(_))));
    }

    #[test]
    fn test_decrypt_chunk_oversized() {
        let km = KeyManager::try_new("test-pass", b"salt1").expect("km"); // expect
        let wal = EncryptedWal::new(km, b"wal.log").expect("wal"); // expect
        let data = vec![0u8; MAX_ENCRYPTED_CHUNK_SIZE + 1];
        let res = wal.decrypt_chunk(&data);
        assert!(matches!(res, Err(CryptoError::InvalidInput(_))));
    }

    #[test]
    fn test_wal_hmac_empty_key() {
        let res = WalHmac::new(b"");
        assert!(matches!(res, Err(CryptoError::InvalidInput(_))));
    }

    #[test]
    fn test_encrypted_wal_max_file_id_boundary() {
        let km = KeyManager::try_new("test-pass", b"salt1").expect("km"); // expect
        let max_id = vec![0x33u8; 10_000];
        let res = EncryptedWal::new(km, &max_id);
        assert!(
            res.is_ok(),
            "file_id of exactly 10,000 bytes MUST be accepted"
        );
    }

    #[test]
    fn test_encrypted_wal_oversized_file_id() {
        let km = KeyManager::try_new("test-pass", b"salt1").expect("km"); // expect
        let oversized_id = vec![0x33u8; 10_001];
        let res = EncryptedWal::new(km, &oversized_id);
        assert!(matches!(res, Err(CryptoError::InvalidInput(_))));
    }

    #[test]
    fn test_wal_hmac_max_key_boundary() {
        let max_key = vec![0x55u8; 10_000];
        let res = WalHmac::new(&max_key);
        assert!(
            res.is_ok(),
            "integrity_key of exactly 10,000 bytes MUST be accepted"
        );
    }

    #[test]
    fn test_encrypted_wal_empty_payload_roundtrip() {
        let km = KeyManager::try_new("test-pass", b"salt1").expect("km"); // expect
        let wal = EncryptedWal::new(km, b"empty_payload.log").expect("wal"); // expect
        let encrypted = wal.encrypt_chunk(b"").expect("encrypt empty"); // expect
        assert_eq!(encrypted.len(), 12 + 16); // 12-byte nonce + 16-byte AES-GCM-SIV tag

        let decrypted = wal.decrypt_chunk(&encrypted).expect("decrypt empty"); // expect
        assert_eq!(decrypted, b"");
    }

    #[test]
    fn test_encrypted_wal_modified_nonce_fails() {
        let km = KeyManager::try_new("test-pass", b"salt1").expect("km"); // expect
        let wal = EncryptedWal::new(km, b"wal.log").expect("wal"); // expect
        let mut encrypted = wal.encrypt_chunk(b"payload").expect("encrypt"); // expect
        encrypted[0] ^= 0xFF; // Corrupt first byte of nonce

        let res = wal.decrypt_chunk(&encrypted);
        assert!(matches!(res, Err(CryptoError::Crypto(_))));
    }

    #[test]
    fn test_verify_and_update_v3_length_prefix_prevents_partition_collision() {
        let key = b"integrity-key-32-bytes-v3-coll-";

        // Entry 1: key = "ab", value = "c"
        let mut mac1 = WalHmac::new(key).unwrap();
        mac1.update(&[0u8; 32]); // prev_hmac
        mac1.update(&1u64.to_le_bytes()); // seq_no
        mac1.update(&1u64.to_le_bytes()); // tx_id
        mac1.update(&[0u8]); // Put op
        mac1.update(&(2u32).to_le_bytes()); // key_len = 2
        mac1.update(b"ab");
        mac1.update(&(1u32).to_le_bytes()); // val_len = 1
        mac1.update(b"c");
        let checksum1 = mac1.finalize();

        let entry1 = WalEntrySnapshot {
            tx_id: 1,
            seq_no: 1,
            op_type: 0,
            key: b"ab".to_vec(),
            value: b"c".to_vec(),
            checksum: checksum1,
            prev_hmac: [0u8; 32],
        };

        // Entry 2: key = "a", value = "bc"
        let mut mac2 = WalHmac::new(key).unwrap();
        mac2.update(&[0u8; 32]); // prev_hmac
        mac2.update(&1u64.to_le_bytes()); // seq_no
        mac2.update(&1u64.to_le_bytes()); // tx_id
        mac2.update(&[0u8]); // Put op
        mac2.update(&(1u32).to_le_bytes()); // key_len = 1
        mac2.update(b"a");
        mac2.update(&(2u32).to_le_bytes()); // val_len = 2
        mac2.update(b"bc");
        let checksum2 = mac2.finalize();

        let entry2 = WalEntrySnapshot {
            tx_id: 1,
            seq_no: 1,
            op_type: 0,
            key: b"a".to_vec(),
            value: b"bc".to_vec(),
            checksum: checksum2,
            prev_hmac: [0u8; 32],
        };

        // Checksums MUST be different
        assert_ne!(
            checksum1, checksum2,
            "Length prefixes in V3 MUST guarantee distinct HMAC checksums for [\"ab\", \"c\"] vs [\"a\", \"bc\"]"
        );

        // Verifier 1 accepts entry 1
        let mut verifier1 = IntegrityVerifier::new(key);
        assert!(verifier1.verify_and_update_v3(&entry1, 0).is_ok());

        // Verifier 2 fails if entry 1's checksum is used with entry 2's key/value split
        let mut verifier2 = IntegrityVerifier::new(key);
        let mut cross_entry = entry2.clone();
        cross_entry.checksum = checksum1;
        assert!(verifier2.verify_and_update_v3(&cross_entry, 0).is_err());
    }

    #[test]
    #[allow(deprecated)]
    fn test_integrity_verifier_v2_roundtrip_and_tamper() {
        let key = b"integrity-key-32-bytes-v2------";
        let mut verifier = IntegrityVerifier::new(key);

        // Compute V2 checksum independently
        let mut hmac = WalHmac::new(key).expect("hmac"); // expect
        hmac.update(&[0u8; 32]); // prev_hmac
        hmac.update(&1u64.to_le_bytes()); // seq_no
        hmac.update(&[0u8]); // op_type = Put
        hmac.update(b"v2_key");
        hmac.update(b"v2_val");
        let checksum = hmac.finalize();

        let v2_entry = WalEntrySnapshot {
            tx_id: 1,
            seq_no: 1,
            op_type: 0,
            key: b"v2_key".to_vec(),
            value: b"v2_val".to_vec(),
            checksum,
            prev_hmac: [0u8; 32],
        };

        // Verification should succeed
        assert!(verifier.verify_and_update_v2(&v2_entry, 10).is_ok());

        // Verification of tampered entry should fail with WalCorruption
        let mut tampered_v2 = v2_entry.clone();
        tampered_v2.value = b"v2_tampered".to_vec();
        assert!(matches!(
            verifier.verify_and_update_v2(&tampered_v2, 20),
            Err(CryptoError::WalCorruption { .. })
        ));
    }

    #[test]
    fn test_integrity_verifier_skip_hmac_verify_legacy() {
        let key = b"integrity-key-32-bytes-legacy--";
        let mut verifier = IntegrityVerifier::new(key);

        let legacy_entry = WalEntrySnapshot {
            tx_id: 10,
            seq_no: 10,
            op_type: 0,
            key: b"legacy_key".to_vec(),
            value: b"legacy_val".to_vec(),
            checksum: [0xAA; 32],
            prev_hmac: [0u8; 32],
        };

        verifier.skip_hmac_verify_legacy(&legacy_entry);
        assert_eq!(verifier.last_hmac, [0xAA; 32]);
    }

    #[test]
    fn test_wal_hmac_anti_mirroring_reference_check() {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let key = b"integrity-key-32-bytes-check---";
        let payload = b"critical-wal-data-payload";

        let mut wal_hmac = WalHmac::new(key).expect("WalHmac new"); // expect
        wal_hmac.update(payload);
        let actual_checksum = wal_hmac.finalize();

        // Independent external reference calculation using hmac::Hmac<Sha256>
        let mut ref_hmac = Hmac::<Sha256>::new_from_slice(key).expect("hmac ref init"); // expect
        ref_hmac.update(b"contextra-wal-v1"); // Domain separation string
        ref_hmac.update(payload);
        let expected_checksum: [u8; 32] = ref_hmac.finalize().into_bytes().into();

        assert_eq!(
            actual_checksum, expected_checksum,
            "WalHmac checksum MUST match independent HMAC-SHA256 reference calculation!"
        );
    }

    #[test]
    fn test_integrity_verifier_handoff_via_snapshot() {
        let key = b"integrity-key-32-bytes-handoff-";
        let mut verifier_a = IntegrityVerifier::new(key);

        // Process entry 1 with verifier A
        let e1 = create_entry(key, [0u8; 32], 1, 0, b"key1", b"val1");
        verifier_a.verify_and_update(&e1, 10).expect("e1 valid");

        // Take snapshot of chain state
        let snapshot = verifier_a.last_hmac_snapshot();
        assert_eq!(snapshot, e1.checksum);

        // Initialize verifier B and transfer snapshot state
        let mut verifier_b = IntegrityVerifier::new(key);
        verifier_b.set_last_hmac(snapshot);
        assert_eq!(verifier_b.last_hmac_snapshot(), snapshot);

        // Process entry 2 with verifier B
        let e2 = create_entry(key, e1.checksum, 2, 0, b"key2", b"val2");
        verifier_b
            .verify_and_update(&e2, 20)
            .expect("e2 valid with verifier B after handoff");
    }

    #[test]
    fn test_kms_provider_wrap_unwrap_roundtrip() -> Result<()> {
        let km = KeyManager::try_new("kms-passphrase-test", b"kms-salt-12345")?;
        let kms = LocalKeyManagerKms::new(km, "master-key-001");

        assert_eq!(kms.active_key_id(), "master-key-001");
        let dek = kms.get_key()?;
        assert_eq!(dek.len(), 32);

        let data_key = kms.derive_data_key(b"data-stream-42")?;
        assert_eq!(data_key.len(), 32);

        let plaintext = b"secret-payload-for-kms-wrapping";
        let wrapped = kms.wrap_key(b"data-stream-42", plaintext)?;
        assert_ne!(wrapped.as_slice(), plaintext);

        let unwrapped = kms.unwrap_key(b"data-stream-42", &wrapped)?;
        assert_eq!(unwrapped.as_slice(), plaintext);

        Ok(())
    }

    #[test]
    fn test_kms_provider_wrong_key_id_fails_unwrap() -> Result<()> {
        let km = KeyManager::try_new("kms-passphrase-test", b"kms-salt-12345")?;
        let kms = LocalKeyManagerKms::new(km, "master-key-001");

        let plaintext = b"sensitive-data-block";
        let wrapped = kms.wrap_key(b"key-stream-A", plaintext)?;

        // Unwrapping with wrong key_id MUST fail
        let res = kms.unwrap_key(b"key-stream-B", &wrapped);
        assert!(res.is_err(), "Unwrapping with wrong key_id MUST fail");

        Ok(())
    }

    #[test]
    fn test_kms_provider_key_rotation() -> Result<()> {
        let km = KeyManager::try_new("kms-passphrase-test", b"kms-salt-12345")?;
        let mut kms = LocalKeyManagerKms::new(km, "master-key-001");

        let key1 = kms.get_key()?.to_vec();
        assert_eq!(kms.active_key_id(), "master-key-001");

        let new_id = kms.rotate_key()?;
        assert_eq!(new_id, "master-key-001-v1");
        assert_eq!(kms.active_key_id(), "master-key-001-v1");

        let key2 = kms.get_key()?.to_vec();
        assert_ne!(key1, key2, "Rotated KMS key MUST differ from previous key");

        Ok(())
    }

    #[test]
    fn test_kms_provider_tamper_detection() -> Result<()> {
        let km = KeyManager::try_new("kms-passphrase-test", b"kms-salt-12345")?;
        let kms = LocalKeyManagerKms::new(km, "master-key-001");

        let plaintext = b"payload-to-be-tampered";
        let mut wrapped = kms.wrap_key(b"stream-1", plaintext)?;

        // Flip a bit in ciphertext
        let len = wrapped.len();
        wrapped[len - 1] ^= 0xFF;

        let res = kms.unwrap_key(b"stream-1", &wrapped);
        assert!(
            res.is_err(),
            "Tampered ciphertext MUST fail AEAD decryption"
        );

        Ok(())
    }

    #[test]
    fn test_encrypted_wal_with_custom_kms_provider() -> Result<()> {
        let km = KeyManager::try_new("wal-kms-pass", b"wal-kms-salt")?;
        let kms = Arc::new(LocalKeyManagerKms::new(km, "custom-kms-wal"));

        let wal = EncryptedWal::with_kms(Arc::clone(&kms) as Arc<dyn KmsProvider>);
        assert_eq!(wal.kms().active_key_id(), "custom-kms-wal");

        let payload = b"encrypted-wal-via-kms-provider";
        let chunk = wal.encrypt_chunk(payload)?;
        let decrypted = wal.decrypt_chunk(&chunk)?;

        assert_eq!(decrypted.as_slice(), payload);
        Ok(())
    }

    #[test]
    #[cfg(feature = "cloud-egress-guard")]
    fn test_cloud_egress_guard_validation() -> Result<()> {
        let guard = CloudEgressGuard::with_max_bytes(1024);

        assert!(guard.validate_egress_payload(b"valid payload").is_ok());

        let empty_res = guard.validate_egress_payload(b"");
        assert!(matches!(empty_res, Err(CryptoError::InvalidInput(_))));

        let oversized = vec![0u8; 1025];
        let oversized_res = guard.validate_egress_payload(&oversized);
        assert!(matches!(oversized_res, Err(CryptoError::InvalidInput(_))));

        Ok(())
    }
}
