// FILE-CONTEXT
// ZWECK: KvSegment mit Zeroize-Garantie (ZeroizeOnDrop, nie unverschlüsselt auf Disk).
// STAND: TS:2026-09-08T00:00:00Z (SESSION: a413a598)

use memfuse_core::TenantId;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[cfg(feature = "kv-encryption")]
use crate::{EncryptedKvLayer, KvSegmentCipher, ModelFingerprint};

/// Aktuelle Version der KV-Segment-Schlüsselableitung.
/// Erhöhe diesen Wert, wenn sich der HKDF-Info-String oder der Salt-Aufbau ändert.
pub const CURRENT_KV_KEY_DERIVATION_VERSION: u8 = 1;

/// Encrypted layer representation stored inside a KvSegment when encryption is active.
#[cfg(feature = "kv-encryption")]
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct EncryptedSegmentPayload {
    /// Encrypted KV layer container from memfuse-crypto. Zeroized on drop.
    pub layer: EncryptedKvLayer,
}

/// Ein KV-Cache-Segment. P9-Pflicht: Zeroize-on-Drop, nie unverschlüsselt
/// auf persistentem/auslagerbarem Speicher.
///
/// Gemäß Gesamtspezifikation v7.0 §7.3 enthält ein Segment:
/// - `tenant_id`: Mandanten-Identifikator
/// - `segment_id`: Segment-Identifikator
/// - `model_fingerprint`: Optionaler Modell-Fingerprint zur Schlüsselableitung
/// - `rope_offset`: Optionaler RoPE-Positions-Offset (bereits vorbereitet; falls memfuse-mcp
///   diesen Wert noch nicht bereitstellt, ist `None` als offener Folgepunkt dokumentiert)
/// - `encrypted`: Kennzeichen, ob die Tensor-Bytes verschlüsselt vorliegen
/// - `data`: Rohe Tensor-Bytes oder Ciphertext-Bytes. WIRD gezeroized beim Drop.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct KvSegment {
    #[zeroize(skip)] // Metadaten, keine sensiblen Tensor-Daten
    pub tenant_id: TenantId,
    #[zeroize(skip)]
    pub segment_id: u64,
    /// Versionsnummer der HKDF-Schlüsselableitung.
    /// 0 = Legacy (vor Versionierung, kein Versionsfeld in HKDF-Info)
    /// 1 = Aktuell (HKDF-Info enthält "v1" als explizites Byte)
    #[zeroize(skip)]
    pub key_derivation_version: u8,
    #[zeroize(skip)]
    pub encrypted: bool,
    #[zeroize(skip)]
    #[cfg(feature = "kv-encryption")]
    pub model_fingerprint: Option<ModelFingerprint>,
    #[zeroize(skip)]
    pub rope_offset: Option<usize>,
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
    ) -> Result<Self, crate::CryptoError> {
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
            data: ciphertext_copy,
            encrypted_payload: Some(EncryptedSegmentPayload {
                layer: encrypted_layer,
            }),
        })
    }

    /// Entschlüsselt die Daten des Segments, falls es verschlüsselt ist.
    /// Gibt bei Klartext-Segmenten direkt einen Klon der `data`-Bytes zurück.
    #[cfg(feature = "kv-encryption")]
    pub fn decrypt_data(&self, cipher: &KvSegmentCipher) -> Result<Vec<u8>, crate::CryptoError> {
        if !self.encrypted {
            return Ok(self.data.clone());
        }

        if self.key_derivation_version > CURRENT_KV_KEY_DERIVATION_VERSION {
            return Err(crate::CryptoError::Crypto(format!(
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
            // AI-TAG[CRYPTO][MAJOR][RESOLVED] Fail fast on missing encrypted payload/nonce instead of dummy zero nonce (ID: AGT-CRYPTO-fae9dd56) (TS: 2026-09-09T13:17:00Z) (SESSION: a413a598)
            Err(crate::CryptoError::Crypto(
                "Missing encrypted payload nonce for encrypted segment decryption".into(),
            ))
        } else {
            Err(crate::CryptoError::Crypto(
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
            .field("data", &"*** REDACTED ***")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::ManuallyDrop;

    #[test]
    fn test_kv_segment_zeroize_on_drop() {
        let tenant = TenantId::try_new(1).unwrap();
        let data = vec![0xAAu8; 1024];
        let mut segment = ManuallyDrop::new(KvSegment::new(tenant, 1, data));
        let ptr = segment.as_bytes().as_ptr();
        let len = segment.len();

        // Precondition: check that memory contains original non-zero tensor bytes
        // SAFETY: `segment` is alive in ManuallyDrop wrapper and `ptr` points directly to its buffer.
        unsafe {
            let slice = std::slice::from_raw_parts(ptr, len);
            assert_eq!(slice, &[0xAAu8; 1024]);
        }

        // Action: Explicitly invoke zeroize without deallocating/dropping stack frame memory
        Zeroize::zeroize(&mut *segment);

        // Postcondition: Check that memory was zeroed in place without UAF
        // SAFETY: `segment` memory buffer is still allocated within ManuallyDrop wrapper in this frame.
        unsafe {
            let cleared_slice = std::slice::from_raw_parts(ptr, len);
            assert_eq!(
                cleared_slice, &[0x00u8; 1024],
                "KvSegment data MUST be zeroed after zeroize"
            );
        }
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
        let km = crate::crypto::KeyManager::try_new("passphrase-123456", b"salt-123456").unwrap();
        let cipher = KvSegmentCipher::new(km);
        let tenant = TenantId::try_new(101).unwrap();
        let fp = ModelFingerprint::new([0x11u8; 32], "test-model", "Q4_K_M");
        let plaintext = b"legacy version 0 plaintext payload";

        // Encrypt specifically with version 0
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
            data: ciphertext_copy,
            encrypted_payload: Some(EncryptedSegmentPayload {
                layer: encrypted_layer,
            }),
        };

        // Version 0 segment should successfully decrypt with version 0 HKDF info
        let decrypted = segment.decrypt_data(&cipher).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    #[cfg(feature = "kv-encryption")]
    fn test_kv_segment_v1_vs_v0_key_separation() {
        let km = crate::crypto::KeyManager::try_new("passphrase-123456", b"salt-123456").unwrap();
        let cipher = KvSegmentCipher::new(km);
        let tenant = TenantId::try_new(101).unwrap();
        let fp = ModelFingerprint::new([0x11u8; 32], "test-model", "Q4_K_M");
        let plaintext = b"version 1 plaintext payload";

        // Create new_encrypted segment (defaults to version 1)
        let mut segment =
            KvSegment::new_encrypted(&cipher, tenant, 1, fp, None, plaintext).unwrap();

        // Roundtrip with version 1 MUST succeed
        let decrypted = segment.decrypt_data(&cipher).unwrap();
        assert_eq!(decrypted, plaintext);

        // Tampering version to 0 MUST fail decryption because key_v0 != key_v1
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
        let km = crate::crypto::KeyManager::try_new("passphrase-123456", b"salt-123456").unwrap();
        let cipher = KvSegmentCipher::new(km);
        let tenant = TenantId::try_new(101).unwrap();
        let fp = ModelFingerprint::new([0x11u8; 32], "test-model", "Q4_K_M");
        let plaintext = b"unsupported version test payload";

        let mut segment =
            KvSegment::new_encrypted(&cipher, tenant, 1, fp, None, plaintext).unwrap();
        // Set an unknown future version
        segment.key_derivation_version = 99;

        let res = segment.decrypt_data(&cipher);
        assert!(res.is_err());
        let err_msg = res.err().unwrap().to_string();
        assert!(err_msg.contains("Unsupported key derivation version: 99"));
    }
}
