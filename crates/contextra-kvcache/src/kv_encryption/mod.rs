//! Krypto-Integration für `contextra-kvcache` über `contextra-crypto`.

use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

pub use contextra_crypto::{
    CryptoError, CryptoKey, EncryptedKvLayer, KvCipher, KvSegmentCipher, ModelFingerprint,
};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{KvCacheError, Result};
#[cfg(feature = "exp-kivi")]
use crate::quantize_kivi::{
    compress_bytes, decompress_bytes, kivi_dequantize, kivi_quantize, KiviQuantizeConfig,
    KvTensorView,
};
use crate::segment::{
    KvSegment, KvSegmentContent, SegmentAead, ShreddableSegmentKey,
    CURRENT_KV_KEY_DERIVATION_VERSION,
};
use crate::store::TenantIsolatedKvStore;
use crate::types::TenantId;

/// Verschlüsselte Layer-Repräsentation innerhalb eines `KvSegment`, wenn Feature `kv-encryption` aktiv ist.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct EncryptedSegmentPayload {
    /// Verschlüsselter KV-Layer-Container aus `contextra-crypto`. Wird beim Drop gezeroized.
    pub layer: EncryptedKvLayer,
    /// Gespeicherte Schlüsselableitungsversion zum Zeitpunkt der Verschlüsselung.
    #[zeroize(skip)]
    pub key_derivation_version: u8,
    /// Gebundene segment_id zur Authenticated Additional Data (AAD) Verifikation.
    #[zeroize(skip)]
    pub bound_segment_id: Option<u64>,
}

/// Wrapper um `contextra_crypto::CryptoKey`, der das lokale `SegmentAead`-Trait implementiert.
pub struct CryptoKeyAead(pub CryptoKey);

impl SegmentAead for CryptoKeyAead {
    fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        self.0
            .encrypt_auto_nonce(plaintext)
            .map_err(|e| KvCacheError::CryptoError(e.to_string()))
    }

    fn decrypt(&self, ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>> {
        self.0
            .decrypt_auto_nonce(ciphertext, nonce)
            .map_err(|e| KvCacheError::CryptoError(e.to_string()))
    }

    fn wipe(&mut self) {
        self.0.emergency_wipe();
    }
}

impl ShreddableSegmentKey {
    /// Erstellt einen neuen shred-fähigen Schlüssel aus Passphrase und Salt.
    pub fn try_new(passphrase: &str, salt: &[u8]) -> Result<Self> {
        let km = CryptoKey::try_new(passphrase, salt)
            .map_err(|e| KvCacheError::CryptoError(e.to_string()))?;
        Ok(Self::from_aead(Box::new(CryptoKeyAead(km))))
    }

    /// Erstellt einen neuen shred-fähigen Schlüssel mit zufällig erzeugtem Salt.
    pub fn try_new_random(passphrase: &str) -> Result<Self> {
        let (km, _) = CryptoKey::try_new_random_salt(passphrase)
            .map_err(|e| KvCacheError::CryptoError(e.to_string()))?;
        Ok(Self::from_aead(Box::new(CryptoKeyAead(km))))
    }
}

impl KvSegment {
    /// Erstellt ein verschlüsseltes KV-Cache-Segment aus einem Klartext-Tensor.
    pub fn new_encrypted(
        cipher: &KvSegmentCipher,
        tenant_id: impl Into<TenantId>,
        segment_id: u64,
        model_fingerprint: ModelFingerprint,
        rope_offset: Option<usize>,
        plaintext: &[u8],
    ) -> std::result::Result<Self, CryptoError> {
        let tenant_id = tenant_id.into();
        let encrypted_layer = cipher.encrypt_with_version(
            bincode::deserialize(&tenant_id.0.to_le_bytes()).unwrap_or_else(|_| panic!("Valid tenant u64")),
            segment_id,
            CURRENT_KV_KEY_DERIVATION_VERSION,
            model_fingerprint.clone(),
            plaintext,
        )?;
        let ciphertext_copy = encrypted_layer.ciphertext.clone();
        let content_bytes = bytes::Bytes::copy_from_slice(&ciphertext_copy);

        Ok(Self {
            tenant_id,
            segment_id,
            key_derivation_version: CURRENT_KV_KEY_DERIVATION_VERSION,
            encrypted: true,
            model_fingerprint: Some(model_fingerprint),
            rope_offset,
            active_refs: Arc::new(AtomicUsize::new(0)),
            data: ciphertext_copy,
            content: KvSegmentContent::Raw(content_bytes),
            encrypted_payload: Some(EncryptedSegmentPayload {
                layer: encrypted_layer,
                key_derivation_version: CURRENT_KV_KEY_DERIVATION_VERSION,
                bound_segment_id: Some(segment_id),
            }),
        })
    }

    /// Entschlüsselt die Daten des Segments, falls es verschlüsselt ist.
    pub fn decrypt_data(
        &self,
        cipher: &KvSegmentCipher,
    ) -> std::result::Result<Vec<u8>, CryptoError> {
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
            if payload.key_derivation_version != self.key_derivation_version {
                return Err(CryptoError::Crypto(format!(
                    "Key derivation version mismatch: segment key_derivation_version is {}, payload key_derivation_version is {}",
                    self.key_derivation_version, payload.key_derivation_version
                )));
            }
            if let Some(expected_seg_id) = payload.bound_segment_id {
                if expected_seg_id != self.segment_id {
                    return Err(CryptoError::Crypto(format!(
                        "AAD verification failed: segment_id mismatch (expected {}, got {})",
                        expected_seg_id, self.segment_id
                    )));
                }
            }
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

    /// Schreibt ein quantisiertes und verschlüsseltes Segment (Reihenfolge: Quantisierung -> Komprimierung -> AEAD-Verschlüsselung).
    #[cfg(feature = "exp-kivi")]
    pub fn write_quantized(
        &mut self,
        raw_kv: &KvTensorView,
        config: KiviQuantizeConfig,
        cipher: &dyn KvCipher,
    ) -> Result<()> {
        // 1. Quantization FIRST
        let quantized = kivi_quantize(raw_kv, config)?;
        let serialized_quant = bincode::serialize(&quantized)
            .map_err(|e| KvCacheError::Serialization(e.to_string()))?;

        // 2. Compression
        let compressed = compress_bytes(&serialized_quant);

        // 3. AEAD Encryption SECOND
        let encrypted = cipher
            .seal(&compressed)
            .map_err(|e| KvCacheError::CryptoError(e.to_string()))?;

        self.data = encrypted;
        self.encrypted = true;
        self.content = KvSegmentContent::KiviQuantized(quantized);
        Ok(())
    }

    /// Liest ein quantisiertes und verschlüsseltes Segment (Reihenfolge: AEAD-Entschlüsselung -> Dekomprimierung -> Dequantisierung).
    #[cfg(feature = "exp-kivi")]
    pub fn read_dequantized(&self, cipher: &dyn KvCipher) -> Result<KvTensorView> {
        // 1. AEAD Decryption FIRST
        let decrypted = cipher
            .open(&self.data)
            .map_err(|e| KvCacheError::CryptoError(e.to_string()))?;

        // 2. Decompression
        let decompressed = decompress_bytes(&decrypted)?;

        // 3. Dequantization SECOND
        match &self.content {
            KvSegmentContent::KiviQuantized(block) => kivi_dequantize(&decompressed, &block.meta),
            KvSegmentContent::Raw(_) => KvTensorView::from_bytes(&decompressed),
        }
    }
}

impl TenantIsolatedKvStore {
    /// Verschlüsselt einen Klartext-Tensor und fügt ein verschlüsseltes Segment ein.
    pub fn insert_encrypted_segment(
        &self,
        cipher: &KvSegmentCipher,
        tenant: impl Into<TenantId>,
        segment_id: u64,
        model_fingerprint: ModelFingerprint,
        rope_offset: Option<usize>,
        plaintext: &[u8],
    ) -> std::result::Result<(), CryptoError> {
        let tenant = tenant.into();
        let segment = KvSegment::new_encrypted(
            cipher,
            tenant,
            segment_id,
            model_fingerprint,
            rope_offset,
            plaintext,
        )?;
        self.insert_segment(tenant, segment);
        Ok(())
    }

    /// Liest ein Segment und entschlüsselt es falls nötig.
    pub fn get_decrypted_segment(
        &self,
        cipher: &KvSegmentCipher,
        tenant: impl Into<TenantId>,
        segment_id: u64,
    ) -> std::result::Result<Option<Vec<u8>>, CryptoError> {
        let tenant = tenant.into();
        let idx = (tenant.inner() as usize) & (self.shard_count() - 1);
        let mut shard = self.shards()[idx].lock.write();
        if let Some(state) = shard.get_mut(&tenant) {
            if let Some(seg) = state.get_segment_ref_mut(segment_id) {
                let decrypted = seg.decrypt_data(cipher)?;
                return Ok(Some(decrypted));
            }
        }
        Ok(None)
    }
}

/// Konvertiert einen lokalen `TenantId` in den von `contextra-crypto` erwarteten Typ.
pub fn to_crypto_tenant<T: serde::de::DeserializeOwned>(tenant: TenantId) -> T {
    bincode::deserialize(&tenant.0.to_le_bytes()).unwrap_or_else(|_| panic!("Valid tenant ID"))
}
