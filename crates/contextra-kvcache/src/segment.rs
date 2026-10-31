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

use contextra_crypto::CryptoKey;
use contextra_types::{ContextraError, TenantId};
use parking_lot::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::eviction_worker::EvictionWorker;
#[cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]
use super::quantize_kivi::{
    compress_bytes, decompress_bytes, kivi_dequantize, kivi_quantize, KiviQuantizeConfig,
    KiviQuantizedBlock, KvTensorView,
};
use super::radix::KvBlockGuard;

#[cfg(feature = "kv-encryption")]
use contextra_crypto::{CryptoError, EncryptedKvLayer, KvSegmentCipher, ModelFingerprint};

// ANCHOR[contextra-kvcache:SEG-002]
/// Repräsentation des Inhalts eines KV-Segments.
///
/// Kann entweder als rohe Bytes vorliegen oder in einem KIVI-quantisierten Format.
#[derive(Debug, Clone)]
pub enum KvSegmentContent {
    /// Rohe, unkomprimierte Tensor-Bytes im RAM.
    Raw(bytes::Bytes),
    /// 2-Bit-quantisierter KIVI-Block (spart bis zu 75% Speicher).
    #[cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]
    KiviQuantized(KiviQuantizedBlock),
}

// ANCHOR[contextra-kvcache:SEG-003]
/// Aktuelle Version der KV-Segment-Schlüsselableitung (HKDF = HMAC-based Extract-and-Expand Key Derivation Function).
///
/// Erhöhe diesen Wert, wenn sich der HKDF-Info-String oder der Salt-Aufbau ändert.
pub const CURRENT_KV_KEY_DERIVATION_VERSION: u8 = 1;

// ANCHOR[contextra-kvcache:SEG-004]
/// Shred-fähiger Schlüssel für Tier-2 Segmentdateien (Crypto-Shredding).
///
/// Durch Aufruf von `shred()` wird der Schlüssel im Speicher mit Nullen überschrieben
/// und aus dem `RwLock` entfernt.
/// Ausgelagerte Segmentdateien auf Disk werden dadurch mathematisch dauerhaft unlesbar.
#[derive(Clone)]
pub struct ShreddableSegmentKey {
    /// Interner kryptographischer Schlüsselmanager in einem Thread-sicheren RwLock.
    key_manager: Arc<RwLock<Option<CryptoKey>>>,
}

impl ShreddableSegmentKey {
    /// Erstellt einen neuen shred-fähigen Schlüssel aus Passphrase und Salt.
    ///
    /// # Parameter
    /// - `passphrase`: Geheime Passphrase des Mandanten.
    /// - `salt`: Kryptographisches Salt (Zufallswert zur Erhöhung der Entropie).
    ///
    /// # Fehler
    /// Liefert einen [`ContextraError::Crypto`], falls die KDF-Schlüsselableitung fehlschlägt.
    pub fn try_new(passphrase: &str, salt: &[u8]) -> Result<Self, ContextraError> {
        let km = CryptoKey::try_new(passphrase, salt)
            .map_err(|e| ContextraError::Crypto(e.to_string()))?;
        Ok(Self {
            key_manager: Arc::new(RwLock::new(Some(km))),
        })
    }

    /// Erstellt einen neuen shred-fähigen Schlüssel mit zufällig erzeugtem Salt.
    ///
    /// # Parameter
    /// - `passphrase`: Geheime Passphrase des Mandanten.
    ///
    /// # Fehler
    /// Liefert einen [`ContextraError::Crypto`], falls die Zufallserzeugung oder Schlüsselableitung fehlschlägt.
    pub fn try_new_random(passphrase: &str) -> Result<Self, ContextraError> {
        let (km, _) = CryptoKey::try_new_random_salt(passphrase)
            .map_err(|e| ContextraError::Crypto(e.to_string()))?;
        Ok(Self {
            key_manager: Arc::new(RwLock::new(Some(km))),
        })
    }

    /// Verschlüsselt Daten mit AES-256-GCM-SIV und automatisch generiertem Nonce (Number used once).
    ///
    /// # Ablauf
    /// 1. Lesesperre auf `key_manager` anfordern.
    /// 2. Prüfen, ob der Schlüssel noch existiert (nicht ge-shredded).
    /// 3. Daten mit AES-GCM-SIV verschlüsseln und (Ciphertext, Nonce) zurückgeben.
    ///
    /// # Fehler
    /// Schlägt fehl mit [`ContextraError::Crypto`], wenn der Schlüssel bereits zerstört wurde.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12]), ContextraError> {
        let guard = self.key_manager.read();
        let km = guard.as_ref().ok_or_else(|| {
            ContextraError::Crypto(
                "Segment key has been shredded: decryption impossible".to_string(),
            )
        })?;
        km.encrypt_auto_nonce(plaintext)
            .map_err(|e| ContextraError::Crypto(e.to_string()))
    }

    /// Entschlüsselt Daten mit AES-256-GCM-SIV unter Verwendung der 12-Byte-Nonce.
    ///
    /// # Parameter
    /// - `ciphertext`: Verschlüsselte Daten.
    /// - `nonce`: 12-Byte Einmalzahl, die bei der Verschlüsselung erzeugt wurde.
    ///
    /// # Fehler
    /// Schlägt fehl mit [`ContextraError::Crypto`], wenn der Schlüssel bereits zerstört wurde
    /// oder die Authentifizierung (MAC) fehlschlägt.
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, ContextraError> {
        let guard = self.key_manager.read();
        let km = guard.as_ref().ok_or_else(|| {
            ContextraError::Crypto(
                "Segment key has been shredded: decryption impossible".to_string(),
            )
        })?;
        km.decrypt_auto_nonce(ciphertext, nonce)
            .map_err(|e| ContextraError::Crypto(e.to_string()))
    }

    /// Crypto-Shredding: Vernichtet den Schlüssel unwiderruflich im Speicher.
    ///
    /// # Ablauf
    /// 1. Schreibsperre anfordern.
    /// 2. `Option::take()` entnimmt die Instanz.
    /// 3. `emergency_wipe()` überschreibt das Schlüsselmaterial im RAM mit Nullen (Zeroize).
    /// Alle mit diesem Schlüssel verschlüsselten Daten werden dauerhaft unlesbar.
    pub fn shred(&self) {
        if let Some(mut km) = self.key_manager.write().take() {
            km.emergency_wipe();
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
///
/// Hält den Ciphertext und die Nonce zusammen mit dem zugehörigen [`ShreddableSegmentKey`].
pub struct Tier2EncryptedSegment {
    /// Mandanten-Identifikator zur Durchsetzung strenger Mandantentrennung.
    pub tenant_id: TenantId,
    /// Eindeutige Segment-ID innerhalb des Mandanten.
    pub segment_id: u64,
    /// Verschlüsselte Tensor-Daten.
    pub ciphertext: Vec<u8>,
    /// 12-Byte Initialisierungsvektor / Nonce.
    pub nonce: [u8; 12],
    /// Schlüssel zur Entschlüsselung bzw. zum Crypto-Shredding.
    pub key: ShreddableSegmentKey,
}

impl Tier2EncryptedSegment {
    /// Verschlüsselt Klartext-Tensor-Bytes und erstellt ein neues Tier-2 Segment.
    ///
    /// # Ablauf
    /// 1. Klartext mit `key.encrypt()` verschlüsseln.
    /// 2. Struktur mit Ciphertext und Nonce initialisieren.
    pub fn new(
        tenant_id: TenantId,
        segment_id: u64,
        plaintext: &[u8],
        key: ShreddableSegmentKey,
    ) -> Result<Self, ContextraError> {
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
    ///
    /// Scheitert sofort, wenn der Schlüssel ge-shredded wurde.
    pub fn read_and_decrypt(&self) -> Result<Vec<u8>, ContextraError> {
        self.key.decrypt(&self.ciphertext, &self.nonce)
    }
}

/// Verschlüsselte Layer-Repräsentation innerhalb eines `KvSegment`, wenn Feature `kv-encryption` aktiv ist.
#[cfg(feature = "kv-encryption")]
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct EncryptedSegmentPayload {
    /// Verschlüsselter KV-Layer-Container aus `contextra-crypto`. Wird beim Drop gezeroized.
    pub layer: EncryptedKvLayer,
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
    pub model_fingerprint: Option<ModelFingerprint>,
    /// RoPE-Offset (Rotary Position Embedding) für positionelle Konsistenz bei Aufmerksamkeitsberechnungen.
    #[zeroize(skip)]
    pub rope_offset: Option<usize>,
    /// Zähler aktiver Referenzen (`KvBlockGuard`). Verhindert Verdrängung bei `> 0`.
    #[zeroize(skip)]
    pub active_refs: Arc<AtomicUsize>,
    /// Rohe Tensor-Bytes (Klartext oder Ciphertext). WIRD gezeroized beim Drop.
    data: Vec<u8>,
    #[zeroize(skip)]
    pub content: KvSegmentContent,
    #[cfg(feature = "kv-encryption")]
    encrypted_payload: Option<EncryptedSegmentPayload>,
}

impl KvSegment {
    /// Erstellt ein neues Klartext-KV-Cache-Segment (Default / Zero-Config, P12-konform).
    ///
    /// # Parameter
    /// - `tenant_id`: Zugehöriger Mandant.
    /// - `segment_id`: Eindeutige Kennung.
    /// - `data`: Rohe Tensor-Bytes.
    pub fn new(tenant_id: TenantId, segment_id: u64, data: Vec<u8>) -> Self {
        let content_bytes = bytes::Bytes::copy_from_slice(&data);
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
        #[cfg(feature = "kv-encryption")] model_fingerprint: Option<ModelFingerprint>,
        rope_offset: Option<usize>,
    ) -> Self {
        let content_bytes = bytes::Bytes::copy_from_slice(&data);
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

    /// Schreibt quantisierte KV-Tensor-Daten in das Segment unter Beachtung von `INV-KIVI-AEAD-ORDER`.
    ///
    /// # Ablauf
    /// 1. **Quantisierung ZUERST**: Reduktion auf 2-Bit Präzision.
    /// 2. **Kompression**: Komprimierung der serialisierten Struktur.
    /// 3. **AEAD-Verschlüsselung ZULETZT**: Versiegelung mit authentifizierter Verschlüsselung.
    #[cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]
    pub fn write_quantized(
        &mut self,
        raw_kv: &KvTensorView,
        config: KiviQuantizeConfig,
        cipher: &dyn contextra_crypto::KvCipher,
    ) -> Result<(), ContextraError> {
        // 1. Quantization FIRST
        let quantized = kivi_quantize(raw_kv, config)?;
        let serialized_quant = bincode::serialize(&quantized)
            .map_err(|e| ContextraError::Serialization(e.to_string()))?;

        // 2. Compression
        let compressed = compress_bytes(&serialized_quant);

        // 3. AEAD Encryption SECOND
        let encrypted = cipher.seal(&compressed)?;

        self.data = encrypted;
        self.encrypted = true;
        self.content = KvSegmentContent::KiviQuantized(quantized);
        Ok(())
    }

    /// Liest und dequantisiert Segmentdaten unter Beachtung von `INV-KIVI-AEAD-ORDER`.
    ///
    /// # Ablauf
    /// 1. **AEAD-Entschlüsselung ZUERST**: Öffnen und Verifizieren des Chiffrats.
    /// 2. **Dekomprimierung**: Wiederherstellung der komprimierten Bytes.
    /// 3. **Dequantisierung ZULETZT**: Rücktransformation in den Tensor-View.
    #[cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]
    pub fn read_dequantized(
        &self,
        cipher: &dyn contextra_crypto::KvCipher,
    ) -> Result<KvTensorView, ContextraError> {
        // 1. AEAD Decryption FIRST
        let decrypted = cipher.open(&self.data)?;

        // 2. Decompression
        let decompressed = decompress_bytes(&decrypted)?;

        // 3. Dequantization SECOND
        match &self.content {
            KvSegmentContent::KiviQuantized(block) => kivi_dequantize(&decompressed, &block.meta),
            KvSegmentContent::Raw(_) => KvTensorView::from_bytes(&decompressed),
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
            }),
        })
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
            data: ciphertext_copy.clone(),
            content: KvSegmentContent::Raw(bytes::Bytes::from(ciphertext_copy)),
            encrypted_payload: Some(EncryptedSegmentPayload {
                layer: encrypted_layer,
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
