//! # BlobRef: Verweise auf ausgelagerte Körper in Blob-Dateien
//!
//! Layout (28 B, SPEC §9.5, Anhang A.6):
//! `blob_no u64 | offset u64 | len u32 | crc32c u32 | domain u32`
//! `blob_no = 0` bedeutet vernichteter Körper.
//! Keine Integer-Casts (M-02 = 0).

use super::error::RunError;

/// Referenz auf einen ausgelagerten Körper in einer Blob-Datei.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlobRef {
    /// Logische Blob-Nummer (nicht Dateinummer). 0 = vernichtet.
    pub blob_no: u64,
    /// Byte-Offset des Records in der Blob-Datei.
    pub offset: u64,
    /// Logische Körperlänge vor Versiegelung.
    pub len: u32,
    /// CRC32C über die gespeicherten Bytes.
    pub crc32c: u32,
    /// Domänenkennung für kryptografisches Löschen.
    pub domain: u32,
}

impl BlobRef {
    /// Feste Länge der binären Repräsentation (28 B).
    pub const LEN: usize = 28;

    /// Erstellt eine neue Blob-Referenz.
    pub const fn new(blob_no: u64, offset: u64, len: u32, crc32c: u32, domain: u32) -> Self {
        Self {
            blob_no,
            offset,
            len,
            crc32c,
            domain,
        }
    }

    /// Markiert einen vernichteten Körper für die angegebene Domäne (`blob_no = 0`).
    pub const fn shredded(domain: u32) -> Self {
        Self {
            blob_no: 0,
            offset: 0,
            len: 0,
            crc32c: 0,
            domain,
        }
    }

    /// Prüft, ob der Körper vernichtet wurde (`blob_no == 0`).
    pub const fn is_shredded(&self) -> bool {
        self.blob_no == 0
    }

    /// Kodiert die Blob-Referenz in 28 Bytes little-endian.
    pub fn encode(&self) -> [u8; Self::LEN] {
        let mut buf = [0u8; Self::LEN];
        let blob_no_bytes = self.blob_no.to_le_bytes();
        let offset_bytes = self.offset.to_le_bytes();
        let len_bytes = self.len.to_le_bytes();
        let crc_bytes = self.crc32c.to_le_bytes();
        let domain_bytes = self.domain.to_le_bytes();

        if let Some(dst) = buf.get_mut(0..8) {
            dst.copy_from_slice(&blob_no_bytes);
        }
        if let Some(dst) = buf.get_mut(8..16) {
            dst.copy_from_slice(&offset_bytes);
        }
        if let Some(dst) = buf.get_mut(16..20) {
            dst.copy_from_slice(&len_bytes);
        }
        if let Some(dst) = buf.get_mut(20..24) {
            dst.copy_from_slice(&crc_bytes);
        }
        if let Some(dst) = buf.get_mut(24..28) {
            dst.copy_from_slice(&domain_bytes);
        }

        buf
    }

    /// Dekodiert eine Blob-Referenz aus einem Byte-Ausschnitt.
    pub fn decode(bytes: &[u8]) -> Result<Self, RunError> {
        let slice = bytes
            .get(0..Self::LEN)
            .ok_or_else(|| RunError::Corrupt("BlobRef zu kurz (< 28 B)".into()))?;

        let mut b_blob_no = [0u8; 8];
        let mut b_offset = [0u8; 8];
        let mut b_len = [0u8; 4];
        let mut b_crc = [0u8; 4];
        let mut b_domain = [0u8; 4];

        if let Some(s) = slice.get(0..8) {
            b_blob_no.copy_from_slice(s);
        }
        if let Some(s) = slice.get(8..16) {
            b_offset.copy_from_slice(s);
        }
        if let Some(s) = slice.get(16..20) {
            b_len.copy_from_slice(s);
        }
        if let Some(s) = slice.get(20..24) {
            b_crc.copy_from_slice(s);
        }
        if let Some(s) = slice.get(24..28) {
            b_domain.copy_from_slice(s);
        }

        Ok(Self {
            blob_no: u64::from_le_bytes(b_blob_no),
            offset: u64::from_le_bytes(b_offset),
            len: u32::from_le_bytes(b_len),
            crc32c: u32::from_le_bytes(b_crc),
            domain: u32::from_le_bytes(b_domain),
        })
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_a6_blobref_vectors() {
        // Vektor aus SPEC Anhang A.6:
        // blob_no = 0x71, offset = 4096, len = 1000, crc32c = 0xDEADBEEF, domain = 42
        let b = BlobRef::new(0x71, 4096, 1000, 0xDEADBEEF, 42);
        let encoded = b.encode();
        let expected = [
            0x71, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // blob_no
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // offset = 4096
            0xe8, 0x03, 0x00, 0x00,                         // len = 1000
            0xef, 0xbe, 0xad, 0xde,                         // crc32c = 0xDEADBEEF
            0x2a, 0x00, 0x00, 0x00,                         // domain = 42
        ];
        assert_eq!(encoded, expected);

        let decoded = BlobRef::decode(&encoded).unwrap_or(BlobRef::shredded(0));
        assert_eq!(decoded, b);
        assert!(!decoded.is_shredded());

        let shredded = BlobRef::shredded(42);
        assert!(shredded.is_shredded());
        assert_eq!(shredded.domain, 42);
    }
}
