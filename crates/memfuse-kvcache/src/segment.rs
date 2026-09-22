// FILE-CONTEXT
// ZWECK: KvSegment mit Zeroize-Garantie, Tier-2 AEAD-Verschlüsselung & Crypto-Shredding.
// STAND: TS:2026-09-15T00:00:00Z

use memfuse_core::{MemFuseError, TenantId};
use memfuse_crypto::CryptoKey;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::eviction_worker::EvictionWorker;
use super::radix::KvBlockGuard;

#[cfg(feature = "kv-encryption")]
use memfuse_crypto::{CryptoError, EncryptedKvLayer, KvSegmentCipher, ModelFingerprint};

/// Aktuelle Version der KV-Segment-Schlüsselableitung.
/// Erhöhe diesen Wert, wenn sich der HKDF-Info-String oder der Salt-Aufbau ändert.
pub const CURRENT_KV_KEY_DERIVATION_VERSION: u8 = 1;

/// Shred-fähiger Schlüssel für Tier-2 Segmentdateien (Crypto-Shredding).
/// Durch Aufruf von `shred()` wird der Schlüssel im Speicher gezeroized und gelöscht.
/// Ausgelagerte Segmentdateien auf Disk werden dadurch mathematisch dauerhaft unlesbar.
#[derive(Clone)]
pub struct ShreddableSegmentKey {
    key_manager: Arc<RwLock<Option<CryptoKey>>>,
}

impl ShreddableSegmentKey {
    /// Erstellt einen neuen shred-fähigen Schlüssel aus Passphrase und Salt.
    pub fn try_new(passphrase: &str, salt: &[u8]) -> Result<Self, MemFuseError> {
        let km = CryptoKey::try_new(passphrase, salt)
            .map_err(|e| MemFuseError::Crypto(e.to_string()))?;
        Ok(Self {
            key_manager: Arc::new(RwLock::new(Some(km))),
        })
    }

    /// Erstellt einen neuen shred-fähigen Schlüssel mit zufälligem Salt.
    pub fn try_new_random(passphrase: &str) -> Result<Self, MemFuseError> {
        let (km, _) = CryptoKey::try_new_random_salt(passphrase)
            .map_err(|e| MemFuseError::Crypto(e.to_string()))?;
        Ok(Self {
            key_manager: Arc::new(RwLock::new(Some(km))),
        })
    }

    /// Verschlüsselt Daten mit AES-256-GCM-SIV und automatischem Nonce.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12]), MemFuseError> {
        let guard = self.key_manager.read();
        let km = guard.as_ref().ok_or_else(|| {
            MemFuseError::Crypto("Segment key has been shredded: decryption impossible".to_string())
        })?;
        km.encrypt_auto_nonce(plaintext)
            .map_err(|e| MemFuseError::Crypto(e.to_string()))
    }

    /// Entschlüsselt Daten mit AES-256-GCM-SIV.
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, MemFuseError> {
        let guard = self.key_manager.read();
        let km = guard.as_ref().ok_or_else(|| {
            MemFuseError::Crypto("Segment key has been shredded: decryption impossible".to_string())
        })?;
        km.decrypt_auto_nonce(ciphertext, nonce)
            .map_err(|e| MemFuseError::Crypto(e.to_string()))
    }

    /// Crypto-Shredding: Vernichtet den Schlüssel im Speicher.
    /// Alle mit diesem Schlüssel verschlüsselten Daten werden dauerhaft unlesbar.
    pub fn shred(&self) {
        if let Some(mut km) = self.key_manager.write().take() {
            km.emergency_wipe();
        }
    }

    /// Prüft, ob der Schlüssel ge-shredded wurde.
    pub fn is_shredded(&self) -> bool {
        self.key_manager.read().is_none()
    }
}

impl std::fmt::Debug for ShreddableSegmentKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShreddableSegmentKey")
            .field("shredded", &self.is_shredded())
            .finish()
    }
}

/// Verschlüsselte Tier-2 Segmentdatei (ausgelagerter KV-Block auf Disk).
pub struct Tier2EncryptedSegment {
    pub tenant_id: TenantId,
    pub segment_id: u64,
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
    pub key: ShreddableSegmentKey,
}

impl Tier2EncryptedSegment {
    /// Verschlüsselt Klartext-Tensor-Bytes und erstellt ein neues Tier-2 Segment.
    pub fn new(
        tenant_id: TenantId,
        segment_id: u64,
        plaintext: &[u8],
        key: ShreddableSegmentKey,
    ) -> Result<Self, MemFuseError> {
        let (ciphertext, nonce) = key.encrypt(plaintext)?;
        Ok(Self {
            tenant_id,
            segment_id,
            ciphertext,
            nonce,
            key,
        })
    }

    /// Liest und entschlüsselt das Segment. Scheitert sofort, wenn der Key ge-shredded wurde.
    pub fn read_and_decrypt(&self) -> Result<Vec<u8>, MemFuseError> {
        self.key.decrypt(&self.ciphertext, &self.nonce)
    }
}

/// Encrypted layer representation stored inside a KvSegment when encryption is active.
#[cfg(feature = "kv-encryption")]
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct EncryptedSegmentPayload {
    /// Encrypted KV layer container from memfuse-crypto. Zeroized on drop.
    pub layer: EncryptedKvLayer,
}

/// Ein KV-Cache-Segment. P9-Pflicht: Zeroize-on-Drop, nie unverschlüsselt
/// auf persistentem/auslagerbarem Speicher.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct KvSegment {
    #[zeroize(skip)] // Metadaten, keine sensiblen Tensor-Daten
    pub tenant_id: TenantId,
    #[zeroize(skip)]
    pub segment_id: u64,
    /// Versionsnummer der HKDF-Schlüsselableitung.
    #[zeroize(skip)]
    pub key_derivation_version: u8,
    #[zeroize(skip)]
    pub encrypted: bool,
    #[zeroize(skip)]
    #[cfg(feature = "kv-encryption")]
    pub model_fingerprint: Option<ModelFingerprint>,
    #[zeroize(skip)]
    pub rope_offset: Option<usize>,
    #[zeroize(skip)]
    pub active_refs: Arc<AtomicUsize>,
    /// Rohe Tensor-Bytes (Klartext oder Ciphertext). WIRD gezeroized beim Drop.
    data: Vec<u8>,
    #[cfg(feature = "kv-encryption")]
    encrypted_payload: Option<EncryptedSegmentPayload>,
}

impl KvSegment {
    /// Erstellt ein neues Klartext-KV-Cache-Segment (Default / Zero-Config, P12-konform).
    pub fn new(tenant_id: TenantId, segment_id: u64, data: Vec<u8>) -> Self {
        Self {
            tenant_id,
            segment_id,
            key_derivation_version: CURRENT_KV_KEY_DERIVATION_VERSION,
            encrypted: false,
            #[cfg(feature = "kv-encryption")]
            model_fingerprint: None,
            rope_offset: None,
            active_refs: Arc::new(AtomicUsize::new(0)),
            data,
            #[cfg(feature = "kv-encryption")]
            encrypted_payload: None,
        }
    }

    /// Erstellt ein neues Klartext-KV-Cache-Segment mit optionalen Metadaten.
    pub fn new_with_metadata(
        tenant_id: TenantId,
        segment_id: u64,
        data: Vec<u8>,
        #[cfg(feature = "kv-encryption")] model_fingerprint: Option<ModelFingerprint>,
        rope_offset: Option<usize>,
    ) -> Self {
        Self {
            tenant_id,
            segment_id,
            key_derivation_version: CURRENT_KV_KEY_DERIVATION_VERSION,
            encrypted: false,
            #[cfg(feature = "kv-encryption")]
            model_fingerprint,
            rope_offset,
            active_refs: Arc::new(AtomicUsize::new(0)),
            data,
            #[cfg(feature = "kv-encryption")]
            encrypted_payload: None,
        }
    }

    /// Erstellt ein verschlüsseltes KV-Cache-Segment aus einem Klartext-Tensor.
    #[cfg(feature = "kv-encryption")]
    pub fn new_encrypted(
        cipher: &KvSegmentCipher,
        tenant_id: TenantId,
        segment_id: u64,
        model_fingerprint: ModelFingerprint,
        rope_offset: Option<usize>,
        plaintext: &[u8],
    ) -> Result<Self, CryptoError> {
        let encrypted_layer = cipher.encrypt_with_version(
            tenant_id,
            segment_id,
            CURRENT_KV_KEY_DERIVATION_VERSION,
            model_fingerprint.clone(),
            plaintext,
        )?;
        let ciphertext_copy = encrypted_layer.ciphertext.clone();

        Ok(Self {
            tenant_id,
            segment_id,
            key_derivation_version: CURRENT_KV_KEY_DERIVATION_VERSION,
            encrypted: true,
            model_fingerprint: Some(model_fingerprint),
            rope_offset,
            active_refs: Arc::new(AtomicUsize::new(0)),
            data: ciphertext_copy,
            encrypted_payload: Some(EncryptedSegmentPayload {
                layer: encrypted_layer,
            }),
        })
    }

    /// Gibt die aktuelle Anzahl aktiver Referenzen zurück.
    pub fn active_refs(&self) -> usize {
        self.active_refs.load(Ordering::SeqCst)
    }

    /// Erstellt einen `KvBlockGuard` für diesen Block und inkrementiert den Referenzzähler.
    pub fn acquire_guard(&self, worker: Option<Arc<EvictionWorker>>) -> KvBlockGuard {
        KvBlockGuard::new(
            self.segment_id,
            self.tenant_id,
            Arc::clone(&self.active_refs),
            worker,
        )
    }

    /// Entschlüsselt die Daten des Segments, falls es verschlüsselt ist.
    #[cfg(feature = "kv-encryption")]
    pub fn decrypt_data(&self, cipher: &KvSegmentCipher) -> Result<Vec<u8>, CryptoError> {
        if !self.encrypted {
            return Ok(self.data.clone());
        }

        if self.key_derivation_version > CURRENT_KV_KEY_DERIVATION_VERSION {
            return Err(CryptoError::Crypto(format!(
                "Unsupported key derivation version: {}",
                self.key_derivation_version
            )));
        }

        if let Some(payload) = &self.encrypted_payload {
            cipher.decrypt_with_version(
                &payload.layer,
                self.segment_id,
                self.key_derivation_version,
            )
        } else if self.model_fingerprint.is_some() {
            Err(CryptoError::Crypto(
                "Missing encrypted payload nonce for encrypted segment decryption".into(),
            ))
        } else {
            Err(CryptoError::Crypto(
                "Missing model fingerprint for encrypted segment decryption".into(),
            ))
        }
    }

    /// Serialisiert das Segment bzw. dessen verschlüsselte Layer für Tier-2-LSM-Spill.
    pub fn to_spill_bytes(&self) -> Vec<u8> {
        #[cfg(feature = "kv-encryption")]
        if let Some(payload) = &self.encrypted_payload {
            if let Ok(bytes) = bincode::serialize(&payload.layer) {
                return bytes;
            }
        }
        self.data.clone()
    }

    /// Read-Only-Zugriff. Kein Klartext-Export nach außen ohne expliziten Call.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Länge der Tensor-Bytes in Bytes.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Prüft ob das Segment leer ist.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl std::fmt::Debug for KvSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KvSegment")
            .field("tenant_id", &self.tenant_id)
            .field("segment_id", &self.segment_id)
            .field("data_len", &self.data.len())
            .field("active_refs", &self.active_refs())
            .field("data", &"*** REDACTED ***")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::ManuallyDrop;

    #[test]
    #[allow(unsafe_code)]
    fn test_kv_segment_zeroize_on_drop() {
        let tenant = TenantId::try_new(1).unwrap();
        let data = vec![0xAAu8; 1024];
        let mut segment = ManuallyDrop::new(KvSegment::new(tenant, 1, data));
        let ptr = segment.as_bytes().as_ptr();
        let len = segment.len();

        unsafe {
            let slice = std::slice::from_raw_parts(ptr, len);
            assert_eq!(slice, &[0xAAu8; 1024]);
        }

        Zeroize::zeroize(&mut *segment);

        unsafe {
            let cleared_slice = std::slice::from_raw_parts(ptr, len);
            assert_eq!(
                cleared_slice, &[0x00u8; 1024],
                "KvSegment data MUST be zeroed after zeroize"
            );
        }
    }

    #[test]
    fn test_shreddable_key_aead_roundtrip_and_shredding() {
        let tenant = TenantId::try_new(10).unwrap();
        let key = ShreddableSegmentKey::try_new_random("tenant-secret-passphrase").unwrap();
        assert!(!key.is_shredded());

        let plaintext = b"Sensibler Tensor-Inhalt fuer KV-Cache Tier-2";
        let tier2 = Tier2EncryptedSegment::new(tenant, 1001, plaintext, key.clone()).unwrap();

        let decrypted = tier2.read_and_decrypt().unwrap();
        assert_eq!(decrypted, plaintext);

        // Perform crypto-shredding
        key.shred();
        assert!(key.is_shredded());

        // Decryption MUST fail immediately after shredding
        let res = tier2.read_and_decrypt();
        assert!(res.is_err());
        let err = res.err().unwrap().to_string();
        assert!(err.contains("shredded"));
    }

    #[test]
    fn test_kv_segment_metadata_and_debug() {
        let tenant = TenantId::try_new(42).unwrap();
        let data = vec![1, 2, 3, 4, 5];

        #[cfg(feature = "kv-encryption")]
        let segment = KvSegment::new_with_metadata(tenant, 100, data, None, Some(128));
        #[cfg(not(feature = "kv-encryption"))]
        let segment = KvSegment::new_with_metadata(tenant, 100, data, Some(128));

        assert_eq!(segment.tenant_id, tenant);
        assert_eq!(segment.segment_id, 100);
        assert_eq!(segment.rope_offset, Some(128));
        assert_eq!(segment.len(), 5);
        assert!(!segment.is_empty());
        assert_eq!(segment.active_refs(), 0);

        let debug_str = format!("{:?}", segment);
        assert!(debug_str.contains("*** REDACTED ***"));
        assert!(debug_str.contains("tenant_id"));
        assert!(debug_str.contains("segment_id"));

        let empty_segment = KvSegment::new(tenant, 101, vec![]);
        assert!(empty_segment.is_empty());
        assert_eq!(empty_segment.len(), 0);
    }

    #[test]
    fn test_kv_segment_version_1_derivation() {
        let tenant = TenantId::try_new(101).unwrap();
        let segment = KvSegment::new(tenant, 1, vec![1, 2, 3]);
        assert_eq!(
            segment.key_derivation_version,
            CURRENT_KV_KEY_DERIVATION_VERSION
        );
        assert_eq!(segment.key_derivation_version, 1);
    }

    #[test]
    #[cfg(feature = "kv-encryption")]
    fn test_kv_segment_v0_legacy_backward_compatibility() {
        let km = CryptoKey::try_new("passphrase-123456", b"salt-123456").unwrap();
        let cipher = KvSegmentCipher::new(km);
        let tenant = TenantId::try_new(101).unwrap();
        let fp = ModelFingerprint::new([0x11u8; 32], "test-model", "Q4_K_M");
        let plaintext = b"legacy version 0 plaintext payload";

        let encrypted_layer = cipher
            .encrypt_with_version(tenant, 1, 0, fp.clone(), plaintext)
            .unwrap();
        let ciphertext_copy = encrypted_layer.ciphertext.clone();

        let segment = KvSegment {
            tenant_id: tenant,
            segment_id: 1,
            key_derivation_version: 0,
            encrypted: true,
            model_fingerprint: Some(fp),
            rope_offset: None,
            active_refs: Arc::new(AtomicUsize::new(0)),
            data: ciphertext_copy,
            encrypted_payload: Some(EncryptedSegmentPayload {
                layer: encrypted_layer,
            }),
        };

        let decrypted = segment.decrypt_data(&cipher).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    #[cfg(feature = "kv-encryption")]
    fn test_kv_segment_v1_vs_v0_key_separation() {
        let km = CryptoKey::try_new("passphrase-123456", b"salt-123456").unwrap();
        let cipher = KvSegmentCipher::new(km);
        let tenant = TenantId::try_new(101).unwrap();
        let fp = ModelFingerprint::new([0x11u8; 32], "test-model", "Q4_K_M");
        let plaintext = b"version 1 plaintext payload";

        let mut segment =
            KvSegment::new_encrypted(&cipher, tenant, 1, fp, None, plaintext).unwrap();

        let decrypted = segment.decrypt_data(&cipher).unwrap();
        assert_eq!(decrypted, plaintext);

        segment.key_derivation_version = 0;
        let res = segment.decrypt_data(&cipher);
        assert!(
            res.is_err(),
            "Decryption of v1 ciphertext with v0 key derivation MUST fail"
        );
    }

    #[test]
    #[cfg(feature = "kv-encryption")]
    fn test_kv_segment_unsupported_version_error() {
        let km = CryptoKey::try_new("passphrase-123456", b"salt-123456").unwrap();
        let cipher = KvSegmentCipher::new(km);
        let tenant = TenantId::try_new(101).unwrap();
        let fp = ModelFingerprint::new([0x11u8; 32], "test-model", "Q4_K_M");
        let plaintext = b"unsupported version test payload";

        let mut segment =
            KvSegment::new_encrypted(&cipher, tenant, 1, fp, None, plaintext).unwrap();
        segment.key_derivation_version = 99;

        let res = segment.decrypt_data(&cipher);
        assert!(res.is_err());
        let err_msg = res.err().unwrap().to_string();
        assert!(err_msg.contains("Unsupported key derivation version: 99"));
    }
}
