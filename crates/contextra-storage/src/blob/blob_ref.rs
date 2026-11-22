//! # BlobRef und Footer
//!
//! Definitionen für BlobRef (28 B) und BlobFooter (20 B) nach SPEC §9.6.

use crate::blob::record::{BLOB_MAGIC, RecordError};
use crate::codec::crc32c::crc32c;

/// Feste Länge des Blob-Datei-Footers (20 Bytes).
pub const BLOB_FOOTER_LEN: usize = 20;

/// Feste Länge einer BlobRef (28 Bytes).
pub const BLOB_REF_LEN: usize = 28;

/// Footer einer Blob-Datei (20 Bytes nach SPEC §9.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobFooter {
    /// Anzahl der Records im Blob.
    pub record_count: u32,
    /// Gesamtbytes vor dem Footer (bzw. Offset des Footers).
    pub bytes: u64,
}

impl BlobFooter {
    /// Erstellt einen neuen Footer.
    pub fn new(record_count: u32, bytes: u64) -> Self {
        Self { record_count, bytes }
    }

    /// Kodiert den Footer in ein 20-Byte-Array.
    pub fn encode(&self) -> [u8; BLOB_FOOTER_LEN] {
        let mut buf = [0u8; BLOB_FOOTER_LEN];
        let (data, crc_slot) = buf.split_at_mut(16);

        let (magic_slot, rest) = data.split_at_mut(4);
        magic_slot.copy_from_slice(BLOB_MAGIC);

        let (cnt_slot, bytes_slot) = rest.split_at_mut(4);
        cnt_slot.copy_from_slice(&self.record_count.to_le_bytes());
        bytes_slot.copy_from_slice(&self.bytes.to_le_bytes());

        let chk = crc32c(0, data);
        crc_slot.copy_from_slice(&chk.to_le_bytes());

        buf
    }

    /// Dekodiert und verifiziert einen 20-Byte-Footer.
    pub fn decode(buf: &[u8]) -> Result<Self, RecordError> {
        if buf.len() < BLOB_FOOTER_LEN {
            return Err(RecordError::UnexpectedEof);
        }
        let (slice, _) = buf.split_at(BLOB_FOOTER_LEN);

        let (magic_slot, rest) = slice.split_at(4);
        if magic_slot != BLOB_MAGIC {
            return Err(RecordError::BadMagic);
        }

        let (cnt_slot, rest) = rest.split_at(4);
        let mut cnt_b = [0u8; 4];
        cnt_b.copy_from_slice(cnt_slot);
        let record_count = u32::from_le_bytes(cnt_b);

        let (bytes_slot, crc_slot) = rest.split_at(8);
        let mut bytes_b = [0u8; 8];
        bytes_b.copy_from_slice(bytes_slot);
        let bytes = u64::from_le_bytes(bytes_b);

        let mut crc_b = [0u8; 4];
        crc_b.copy_from_slice(crc_slot);
        let expected_chk = u32::from_le_bytes(crc_b);

        let (data, _) = slice.split_at(16);
        let actual_chk = crc32c(0, data);
        if actual_chk != expected_chk {
            return Err(RecordError::ChecksumMismatch {
                expected: expected_chk,
                actual: actual_chk,
            });
        }

        Ok(Self { record_count, bytes })
    }
}

/// Referenz auf einen Blob-Record (28 Bytes nach SPEC §9.5, Golden A.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobRef {
    /// Blob-Dateinummer.
    pub blob_no: u64,
    /// Dateioffset des Records.
    pub offset: u64,
    /// Länge des Record-Inhalts (rlen).
    pub len: u32,
    /// CRC32C-Prüfsumme des Inhalts.
    pub crc32c: u32,
    /// Kryptografische Domäne.
    pub domain: u32,
}

impl BlobRef {
    /// Erstellt eine geschredderte Referenz (blob_no = 0) nach SPEC §12.1.
    pub const fn shredded(domain: u32) -> Self {
        Self {
            blob_no: 0,
            offset: 0,
            len: 0,
            crc32c: 0,
            domain,
        }
    }

    /// Prüft, ob der Körper vernichtet ist (blob_no = 0).
    pub const fn is_shredded(&self) -> bool {
        self.blob_no == 0
    }

    /// Kodiert die BlobRef in 28 Bytes (SPEC §9.5, Golden A.6).
    pub fn encode(&self) -> [u8; BLOB_REF_LEN] {
        let mut buf = [0u8; BLOB_REF_LEN];
        let (no_slot, rest) = buf.split_at_mut(8);
        no_slot.copy_from_slice(&self.blob_no.to_le_bytes());

        let (off_slot, rest) = rest.split_at_mut(8);
        off_slot.copy_from_slice(&self.offset.to_le_bytes());

        let (len_slot, rest) = rest.split_at_mut(4);
        len_slot.copy_from_slice(&self.len.to_le_bytes());

        let (crc_slot, dom_slot) = rest.split_at_mut(4);
        crc_slot.copy_from_slice(&self.crc32c.to_le_bytes());
        dom_slot.copy_from_slice(&self.domain.to_le_bytes());

        buf
    }

    /// Dekodiert eine BlobRef aus 28 Bytes.
    pub fn decode(buf: &[u8]) -> Result<Self, RecordError> {
        if buf.len() < BLOB_REF_LEN {
            return Err(RecordError::UnexpectedEof);
        }
        let (slice, _) = buf.split_at(BLOB_REF_LEN);

        let (no_slot, rest) = slice.split_at(8);
        let mut no_b = [0u8; 8];
        no_b.copy_from_slice(no_slot);
        let blob_no = u64::from_le_bytes(no_b);

        let (off_slot, rest) = rest.split_at(8);
        let mut off_b = [0u8; 8];
        off_b.copy_from_slice(off_slot);
        let offset = u64::from_le_bytes(off_b);

        let (len_slot, rest) = rest.split_at(4);
        let mut len_b = [0u8; 4];
        len_b.copy_from_slice(len_slot);
        let len = u32::from_le_bytes(len_b);

        let (crc_slot, dom_slot) = rest.split_at(4);
        let mut crc_b = [0u8; 4];
        crc_b.copy_from_slice(crc_slot);
        let crc = u32::from_le_bytes(crc_b);

        let mut dom_b = [0u8; 4];
        dom_b.copy_from_slice(dom_slot);
        let domain = u32::from_le_bytes(dom_b);

        Ok(Self {
            blob_no,
            offset,
            len,
            crc32c: crc,
            domain,
        })
    }
}
