//! Abstraktion für Block- und Segmentverschlüsselung.

use crate::error::Result;
use crate::segment::SegmentKey;

/// Trait zur Segment-Verschlüsselung.
pub trait SegmentCipher: Send + Sync {
    /// Verschlüsselt Klartextbytes mit dem angegebenen Schlüssel.
    fn encrypt_segment(&self, key: &SegmentKey, plaintext: &[u8]) -> Result<Vec<u8>>;

    /// Entschlüsselt Geheimtextbytes mit dem angegebenen Schlüssel.
    fn decrypt_segment(&self, key: &SegmentKey, ciphertext: &[u8]) -> Result<Vec<u8>>;
}

/// No-Op Cipher für Klartext-Caches und Tests.
#[derive(Debug, Default, Clone, Copy)]
pub struct PlaintextSegmentCipher;

impl SegmentCipher for PlaintextSegmentCipher {
    fn encrypt_segment(&self, _key: &SegmentKey, plaintext: &[u8]) -> Result<Vec<u8>> {
        Ok(plaintext.to_vec())
    }

    fn decrypt_segment(&self, _key: &SegmentKey, ciphertext: &[u8]) -> Result<Vec<u8>> {
        Ok(ciphertext.to_vec())
    }
}
