//! # KV-Cache-Segment und Verschlüsselung
//!
//! ANCHOR[contextra-kvcache:SEG-001]
//!
//! Dieses Modul definiert die Kern-Datenstrukturen für KV-Segmente im RAM und auf Sekundärspeichern.
//!
//! ## Was ist ein KV-Segment?
//! Ein `KvSegment` speichert die konkreten Tensor-Rohdaten (oder quantisierten Blöcke) einer
//! Token-Sequenz. Ein Segment gehört immer genau einem Mandanten (`TenantId`) und besitzt eine
//! eindeutige `segment_id`.
//!
//! ## Sicherheits- und Speichergarantien
//! - **P9 (Zeroize-on-Drop)**:
//!   Sensible Tensor-Bytes implementieren `ZeroizeOnDrop`. Wenn ein Segment aus dem Speicher
//!   freigegeben wird, werden die Speicherbereiche sofort mit Nullen überschrieben (Zeroization).
//! - **Crypto-Shredding ([`ShreddableSegmentKey`])**:
//!   Segmente, die im Rahmen des Tiering (Tier-2) auf persistente Medien ausgelagert werden,
//!   werden mit AES-256-GCM-SIV verschlüsselt. Durch Vernichten des Schlüssels im RAM (`shred()`)
//!   werden die ausgelagerten Segmente mathematisch unlesbar, ohne dass die Festplatte physisch
//!   gelöscht werden muss.
//! - **Referenzzählung ([`KvBlockGuard`])**:
//!   Aktive Inferenz-Prozesse halten einen Guard. Ein Segment mit `active_refs > 0` darf unter
//!   keinen Umständen verdrängt werden.

use crate::error::{KvCacheError, Result};
use crate::types::TenantId;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::eviction_worker::EvictionWorker;
#[cfg(feature = "exp-kivi")]
use super::quantize_kivi::{
    compress_bytes, decompress_bytes, kivi_dequantize, kivi_quantize, KiviQuantizeConfig,
    KiviQuantizedBlock, KvTensorView,
};
use super::radix::KvBlockGuard;

#[cfg(feature = "kv-encryption")]
use crate::kv_encryption::EncryptedSegmentPayload;

// ANCHOR[contextra-kvcache:SEG-002]
/// Repräsentation des Inhalts eines KV-Segments.
///
/// Kann entweder als rohe Bytes vorliegen oder in einem KIVI-quantisierten Format.
#[derive(Debug, Clone)]
pub enum KvSegmentContent {
    /// Rohe, unkomprimierte Tensor-Bytes im RAM.
    Raw(bytes::Bytes),
    /// 2-Bit-quantisierter KIVI-Block (spart bis zu 75% Speicher).
    #[cfg(feature = "exp-kivi")]
    KiviQuantized(KiviQuantizedBlock),
}

// ANCHOR[contextra-kvcache:SEG-003]
/// Aktuelle Version der KV-Segment-Schlüsselableitung (HKDF = HMAC-based Extract-and-Expand Key Derivation Function).
///
/// Erhöhe diesen Wert, wenn sich der HKDF-Info-String oder der Salt-Aufbau ändert.
pub const CURRENT_KV_KEY_DERIVATION_VERSION: u8 = 1;

// ANCHOR[contextra-kvcache:SEG-AEAD-001]
/// Lokales Trait für AEAD-Segmentverschlüsselung (Authenticated Encryption with Associated Data).
pub trait SegmentAead: Send + Sync {
    /// Verschlüsselt Daten mit AEAD und liefert (Ciphertext, 12-Byte Nonce).
    fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12])>;
    /// Entschlüsselt Daten mit AEAD unter Verwendung der 12-Byte Nonce.
    fn decrypt(&self, ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>>;
    /// Überschreibt das Schlüsselmaterial sicher im Speicher.
    fn wipe(&mut self);
}

// ANCHOR[contextra-kvcache:SEG-004]
/// Shred-fähiger Schlüssel für Tier-2 Segmentdateien (Crypto-Shredding).
///
/// Durch Aufruf von `shred()` wird der Schlüssel im Speicher mit Nullen überschrieben
/// und aus dem `RwLock` entfernt.
/// Ausgelagerte Segmentdateien auf Disk werden dadurch mathematisch dauerhaft unlesbar.
#[derive(Clone)]
pub struct ShreddableSegmentKey {
    key_manager: Arc<RwLock<Option<Box<dyn SegmentAead>>>>,
}

impl ShreddableSegmentKey {
    /// Erzeugt einen neuen `ShreddableSegmentKey` aus einer dynamischen `SegmentAead`-Implementierung.
    pub fn from_aead(aead: Box<dyn SegmentAead>) -> Self {
        Self {
            key_manager: Arc::new(RwLock::new(Some(aead))),
        }
    }

    /// Erzeugt einen leeren oder bereits ge-shreddeten Schlüssel.
    pub fn shredded() -> Self {
        Self {
            key_manager: Arc::new(RwLock::new(None)),
        }
    }

    /// Verschlüsselt Daten mit AEAD und automatisch generiertem Nonce.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        let guard = self.key_manager.read();
        let km = guard.as_ref().ok_or_else(|| {
            KvCacheError::CryptoError(
                "Segment key has been shredded: decryption impossible".to_string(),
            )
        })?;
        km.encrypt(plaintext)
    }

    /// Entschlüsselt Daten mit AEAD unter Verwendung der 12-Byte-Nonce.
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>> {
        let guard = self.key_manager.read();
        let km = guard.as_ref().ok_or_else(|| {
            KvCacheError::CryptoError(
                "Segment key has been shredded: decryption impossible".to_string(),
            )
        })?;
        km.decrypt(ciphertext, nonce)
    }

    /// Crypto-Shredding: Überschreibt den Schlüssel im Speicher mit Nullen.
    pub fn shred(&self) {
        if let Some(mut km) = self.key_manager.write().take() {
            km.wipe();
        }
    }

    /// Prüft, ob der Schlüssel bereits ge-shredded (vernichtet) wurde.
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


// ANCHOR[contextra-kvcache:SEG-005]
/// Verschlüsselte Tier-2 Segmentdatei (ausgelagerter KV-Block auf persistentem Speicher).
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
    ) -> Result<Self> {
        let (ciphertext, nonce) = key.encrypt(plaintext)?;
        Ok(Self {
            tenant_id,
            segment_id,
            ciphertext,
            nonce,
            key,
        })
    }

    /// Liest und entschlüsselt das Segment.
    pub fn read_and_decrypt(&self) -> Result<Vec<u8>> {
        self.key.decrypt(&self.ciphertext, &self.nonce)
    }
}


// ANCHOR[contextra-kvcache:SEG-006]
/// Ein KV-Cache-Segment im Hauptspeicher.
///
/// Erfüllt die P9-Sicherheitsanforderung: Zeroize-on-Drop für vertrauliche Daten.
/// Darf niemals unverschlüsselt auf Sekundärspeicher (Tier-2) geschrieben werden.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct KvSegment {
    #[zeroize(skip)] // Metadaten, keine sensiblen Tensor-Daten
    pub tenant_id: TenantId,
    #[zeroize(skip)]
    pub segment_id: u64,
    /// Versionsnummer der HKDF-Schlüsselableitung.
    #[zeroize(skip)]
    pub key_derivation_version: u8,
    /// Gibt an, ob der Inhalt im RAM verschlüsselt vorliegt.
    #[zeroize(skip)]
    pub encrypted: bool,
    #[zeroize(skip)]
    #[cfg(feature = "kv-encryption")]
    pub model_fingerprint: Option<crate::kv_encryption::ModelFingerprint>,
    /// RoPE-Offset (Rotary Position Embedding) für positionelle Konsistenz bei Aufmerksamkeitsberechnungen.
    #[zeroize(skip)]
    pub rope_offset: Option<usize>,
    /// Zähler aktiver Referenzen (`KvBlockGuard`). Verhindert Verdrängung bei `> 0`.
    #[zeroize(skip)]
    pub active_refs: Arc<AtomicUsize>,
    /// Rohe Tensor-Bytes (Klartext oder Ciphertext). WIRD gezeroized beim Drop.
    pub(crate) data: Vec<u8>,
    #[zeroize(skip)]
    pub content: KvSegmentContent,
    #[cfg(feature = "kv-encryption")]
    pub(crate) encrypted_payload: Option<EncryptedSegmentPayload>,
}

impl KvSegment {
    /// Erstellt ein neues Klartext-KV-Cache-Segment (Default / Zero-Config, P12-konform).
    ///
    /// # Parameter
    /// - `tenant_id`: Zugehöriger Mandant.
    /// - `segment_id`: Eindeutige Kennung.
    /// - `data`: Rohe Tensor-Bytes.
    pub fn new(tenant_id: impl Into<TenantId>, segment_id: u64, data: Vec<u8>) -> Self {
        let tenant_id = tenant_id.into(); let content_bytes = bytes::Bytes::copy_from_slice(&data);
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
            content: KvSegmentContent::Raw(content_bytes),
            #[cfg(feature = "kv-encryption")]
            encrypted_payload: None,
        }
    }

    /// Erstellt ein neues Klartext-KV-Cache-Segment mit optionalen Metadaten.
    ///
    /// Ermöglicht die Übergabe eines RoPE-Offsets und eines Modell-Fingerprints.
    pub fn new_with_metadata(
        tenant_id: TenantId,
        segment_id: u64,
        data: Vec<u8>,
        #[cfg(feature = "kv-encryption")] model_fingerprint: Option<crate::kv_encryption::ModelFingerprint>,
        rope_offset: Option<usize>,
    ) -> Self {
        let tenant_id = tenant_id.into(); let content_bytes = bytes::Bytes::copy_from_slice(&data);
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
            content: KvSegmentContent::Raw(content_bytes),
            #[cfg(feature = "kv-encryption")]
            encrypted_payload: None,
        }
    }


    /// Gibt die aktuelle Anzahl aktiver Referenzen (`KvBlockGuard`) zurück.
    pub fn active_refs(&self) -> usize {
        self.active_refs.load(Ordering::SeqCst)
    }

    /// Erstellt einen [`KvBlockGuard`] für diesen Block und inkrementiert den Referenzzähler atomar.
    pub fn acquire_guard(&self, worker: Option<Arc<EvictionWorker>>) -> KvBlockGuard {
        KvBlockGuard::new(
            self.segment_id,
            self.tenant_id,
            Arc::clone(&self.active_refs),
            worker,
        )
    }


    /// Serialisiert das Segment bzw. dessen verschlüsselte Layer für Tier-2-LSM-Spill (Auslagerung).
    pub fn to_spill_bytes(&self) -> Vec<u8> {
        #[cfg(feature = "kv-encryption")]
        if let Some(payload) = &self.encrypted_payload {
            if let Ok(bytes) = bincode::serialize(&payload.layer) {
                return bytes;
            }
        }
        self.data.clone()
    }

    /// Read-Only-Zugriff auf die Rohdaten.
    ///
    /// Kein unverschlüsselter Export nach außen ohne explizite Autorisierung.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Länge der Tensor-Bytes in Bytes.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Prüft, ob das Segment leer ist (0 Bytes).
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
    #[cfg(feature = "kv-encryption")]
    use crate::kv_encryption::{CryptoKey, KvSegmentCipher, ModelFingerprint};

    // Prüft, dass Daten im KvSegment beim Drop oder expliziten Zeroize genullt werden.
    #[test]
    fn test_kv_segment_zeroize_on_drop() {
        let tenant = TenantId::try_new(1).unwrap();
        let data = vec![0xAAu8; 1024];
        let mut segment = KvSegment::new(tenant, 1, data);

        assert_eq!(segment.as_bytes(), &[0xAAu8; 1024]);

        segment.data.as_mut_slice().zeroize();

        assert_eq!(
            segment.as_bytes(),
            &[0x00u8; 1024],
            "KvSegment data MUST be zeroed after zeroize"
        );
    }

    #[cfg(feature = "kv-encryption")]
    // Prüft den Lebenszyklus des ShreddableSegmentKey: Verschlüsselung, Entschlüsselung und Unlesbarkeit nach Shredding.
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

    // Prüft Metadatenfelder und Schwärzung sensibler Daten im Debug-Format.
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

    // Prüft, dass neu erstellte Segmente standardmäßig Version 1 der Schlüsselableitung nutzen.
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

    // Prüft die Rückwärtskompatibilität für verschlüsselte Segmente mit Version 0.
    #[test]
    #[cfg(feature = "kv-encryption")]
    fn test_kv_segment_v0_legacy_backward_compatibility() {
        let km = CryptoKey::try_new("passphrase-123456", b"salt-123456").unwrap();
        let cipher = KvSegmentCipher::new(km);
        let tenant = TenantId::try_new(101).unwrap();
        let fp = ModelFingerprint::new([0x11u8; 32], "test-model", "Q4_K_M");
        let plaintext = b"legacy version 0 plaintext payload";

        let crypto_tenant = crate::kv_encryption::to_crypto_tenant(tenant);
        let encrypted_layer = cipher
            .encrypt_with_version(crypto_tenant, 1, 0, fp.clone(), plaintext)
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
            data: ciphertext_copy.clone(),
            content: KvSegmentContent::Raw(bytes::Bytes::from(ciphertext_copy)),
            encrypted_payload: Some(EncryptedSegmentPayload {
                layer: encrypted_layer,
                key_derivation_version: 0,
                bound_segment_id: Some(1),
            }),
        };

        let decrypted = segment.decrypt_data(&cipher).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    // Prüft, dass Schlüsselableitung von v1 und v0 strikt getrennt sind und nicht kompatibel sind.
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

    // Prüft, dass nicht unterstützte Schlüsselableitungsversionen kontrolliert mit einem Fehler abgewiesen werden.
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

/// Neutraler Segmentschlüssel (Klasse A Baustein).
pub type SegmentKey = ShreddableSegmentKey;

/// Configuration for the bounded segment pool.
#[derive(Debug, Clone, Copy)]
pub struct SegmentPoolConfig {
    /// Size of each fixed block in bytes.
    pub block_size: usize,
    /// Maximum number of blocks the pool can allocate.
    pub max_blocks: usize,
}

/// Generational handle to an allocated segment in the pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SegmentHandle {
    slot_index: usize,
    generation: u64,
}

impl SegmentHandle {
    /// Returns the slot index of this handle.
    pub const fn slot_index(&self) -> usize {
        self.slot_index
    }

    /// Returns the generation counter of this handle.
    pub const fn generation(&self) -> u64 {
        self.generation
    }
}

struct SegmentSlot {
    generation: u64,
    allocated: bool,
    namespace: u64,
    data: Vec<u8>,
    actual_len: usize,
}

/// Bounded segment pool providing generational safety, use-after-free & double-free prevention,
/// zeroization on deallocation, and fragmentation metrics tracking.
pub struct SegmentPool {
    config: SegmentPoolConfig,
    slots: parking_lot::RwLock<Vec<SegmentSlot>>,
}

impl SegmentPool {
    /// Creates a new `SegmentPool` with the given configuration.
    pub fn new(config: SegmentPoolConfig) -> Self {
        let mut slots = Vec::with_capacity(config.max_blocks);
        for _ in 0..config.max_blocks {
            slots.push(SegmentSlot {
                generation: 0,
                allocated: false,
                namespace: 0,
                data: vec![0u8; config.block_size],
                actual_len: 0,
            });
        }
        Self {
            config,
            slots: parking_lot::RwLock::new(slots),
        }
    }

    /// Allocates a segment block within the given namespace, storing the payload data.
    pub fn allocate(&self, namespace: u64, data: &[u8]) -> Result<SegmentHandle> {
        if data.len() > self.config.block_size {
            return Err(crate::error::KvCacheError::BudgetExceeded(
                "Payload exceeds pool block_size".into(),
            ));
        }

        let mut slots = self.slots.write();
        for (idx, slot) in slots.iter_mut().enumerate() {
            if !slot.allocated {
                slot.allocated = true;
                slot.namespace = namespace;
                slot.actual_len = data.len();
                slot.data[..data.len()].copy_from_slice(data);
                return Ok(SegmentHandle {
                    slot_index: idx,
                    generation: slot.generation,
                });
            }
        }

        Err(crate::error::KvCacheError::BudgetExceeded(
            "SegmentPool capacity exceeded".into(),
        ))
    }

    /// Deallocates the segment slot indicated by handle, zeroizing payload memory and bumping generation.
    pub fn deallocate(&self, namespace: u64, handle: SegmentHandle) -> Result<()> {
        let mut slots = self.slots.write();
        if handle.slot_index >= slots.len() {
            return Err(crate::error::KvCacheError::SegmentNotFound(
                handle.slot_index as u64,
            ));
        }
        let slot = &mut slots[handle.slot_index];
        if !slot.allocated || slot.generation != handle.generation || slot.namespace != namespace {
            return Err(crate::error::KvCacheError::InvalidSegment(
                "Invalid segment handle, double-free or generational mismatch".into(),
            ));
        }

        slot.data.as_mut_slice().zeroize();
        slot.allocated = false;
        slot.actual_len = 0;
        slot.generation = slot.generation.wrapping_add(1);
        Ok(())
    }

    /// Reads payload data from an active handle.
    pub fn read(&self, namespace: u64, handle: SegmentHandle) -> Result<Vec<u8>> {
        let slots = self.slots.read();
        if handle.slot_index >= slots.len() {
            return Err(crate::error::KvCacheError::SegmentNotFound(
                handle.slot_index as u64,
            ));
        }
        let slot = &slots[handle.slot_index];
        if !slot.allocated || slot.generation != handle.generation || slot.namespace != namespace {
            return Err(crate::error::KvCacheError::InvalidSegment(
                "Invalid handle or use-after-free".into(),
            ));
        }
        Ok(slot.data[..slot.actual_len].to_vec())
    }

    /// Returns the total count of currently allocated blocks.
    pub fn allocated_blocks(&self) -> usize {
        self.slots.read().iter().filter(|s| s.allocated).count()
    }

    /// Returns the sum of actual bytes stored across allocated blocks.
    pub fn allocated_bytes(&self) -> usize {
        self.slots
            .read()
            .iter()
            .filter(|s| s.allocated)
            .map(|s| s.actual_len)
            .sum()
    }

    /// Returns the fragmentation ratio: 1.0 - (allocated_bytes / capacity_of_allocated_blocks).
    /// If 0 blocks are allocated, returns 0.0.
    pub fn fragmentation_ratio(&self) -> f64 {
        let blocks = self.allocated_blocks();
        if blocks == 0 {
            return 0.0;
        }
        let allocated_cap = (blocks * self.config.block_size) as f64;
        let used_bytes = self.allocated_bytes() as f64;
        (allocated_cap - used_bytes) / allocated_cap
    }
}
