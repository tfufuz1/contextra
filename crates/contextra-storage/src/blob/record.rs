//! # Blob-Record-Definitionen
//!
//! SPEC §9.6: Header (48 B), Records, Hole-Records, Footer (20 B) und BlobRef (28 B).

pub use crate::blob::blob_ref::{BlobFooter, BlobRef, BLOB_FOOTER_LEN, BLOB_REF_LEN};
use crate::codec::crc32c::crc32c;
use crate::codec::varint::{decode_u64, encode_u64};
use std::fmt;

/// Magic-Bytes für Blob-Header und Footer ("CTXB").
pub const BLOB_MAGIC: &[u8; 4] = b"CTXB";

/// Feste Länge des Blob-Datei-Headers (48 Bytes).
pub const BLOB_HEADER_LEN: usize = 48;

/// Record-Kind für regulären Nutzdaten-Körper.
pub const RECORD_KIND_BODY: u8 = 0x01;

/// Record-Kind für ein Hole (INV-BLOB-3).
pub const RECORD_KIND_HOLE: u8 = 0x02;

/// Fehler beim Kodieren oder Dekodieren von Blob-Records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordError {
    /// Puffer zu kurz für geforderte Struktur.
    UnexpectedEof,
    /// Ungültige Magic-Bytes.
    BadMagic,
    /// Nicht unterstützte Formatversion.
    UnsupportedVersion(u16),
    /// Prüfsumme stimmt nicht überein.
    ChecksumMismatch {
        /// Erwartete Prüfsumme.
        expected: u32,
        /// Tatsächlich gelesene Prüfsumme.
        actual: u32,
    },
    /// Ungültiges Record-Format.
    InvalidFormat(&'static str),
    /// Schlüssel zu lang.
    KeyTooLarge,
    /// Ganzzahl-Überlauf.
    Overflow,
}

impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "Unerwartetes Dateiende im Blob-Record"),
            Self::BadMagic => write!(f, "Ungültige Blob-Magic-Bytes"),
            Self::UnsupportedVersion(v) => write!(f, "Nicht unterstützte Blob-Version: {v}"),
            Self::ChecksumMismatch { expected, actual } => {
                write!(f, "Blob-Prüfsummenfehler: erwartet {expected:#x}, gefunden {actual:#x}")
            }
            Self::InvalidFormat(msg) => write!(f, "Ungültiges Blob-Format: {msg}"),
            Self::KeyTooLarge => write!(f, "Schlüssel im Blob-Record überschreitet Limit"),
            Self::Overflow => write!(f, "Arithmetischer Überlauf bei Blob-Längenberechnung"),
        }
    }
}

impl std::error::Error for RecordError {}

/// Header einer Blob-Datei (48 Bytes nach SPEC §9.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobHeader {
    /// Formatversion (aktuell 1).
    pub version: u16,
    /// Flags (aktuell 0).
    pub flags: u16,
    /// Eindeutige Blob-Nummer.
    pub blob_no: u64,
    /// Generationsnummer (beginnend bei 1).
    pub generation: u32,
    /// Space-Kennung.
    pub space: u16,
    /// Reservierter Bereich (22 Bytes Nullen).
    pub reserved: [u8; 22],
}

impl BlobHeader {
    /// Erstellt einen neuen Header mit Versionsnummer 1.
    pub fn new(blob_no: u64, generation: u32, space: u16) -> Self {
        Self {
            version: 1,
            flags: 0,
            blob_no,
            generation,
            space,
            reserved: [0u8; 22],
        }
    }

    /// Kodiert den Header in ein 48-Byte-Array.
    pub fn encode(&self) -> [u8; BLOB_HEADER_LEN] {
        let mut buf = [0u8; BLOB_HEADER_LEN];
        let (header_data, crc_slot) = buf.split_at_mut(44);

        let (magic_slot, rest) = header_data.split_at_mut(4);
        magic_slot.copy_from_slice(BLOB_MAGIC);

        let (ver_slot, rest) = rest.split_at_mut(2);
        ver_slot.copy_from_slice(&self.version.to_le_bytes());

        let (flags_slot, rest) = rest.split_at_mut(2);
        flags_slot.copy_from_slice(&self.flags.to_le_bytes());

        let (no_slot, rest) = rest.split_at_mut(8);
        no_slot.copy_from_slice(&self.blob_no.to_le_bytes());

        let (gen_slot, rest) = rest.split_at_mut(4);
        gen_slot.copy_from_slice(&self.generation.to_le_bytes());

        let (space_slot, rest) = rest.split_at_mut(2);
        space_slot.copy_from_slice(&self.space.to_le_bytes());

        let (res_slot, _) = rest.split_at_mut(22);
        res_slot.copy_from_slice(&self.reserved);

        let chk = crc32c(0, header_data);
        crc_slot.copy_from_slice(&chk.to_le_bytes());

        buf
    }

    /// Dekodiert einen 48-Byte-Header und verifiziert Magic und CRC32C.
    pub fn decode(buf: &[u8]) -> Result<Self, RecordError> {
        if buf.len() < BLOB_HEADER_LEN {
            return Err(RecordError::UnexpectedEof);
        }
        let (slice, _) = buf.split_at(BLOB_HEADER_LEN);

        let (magic_slot, rest) = slice.split_at(4);
        if magic_slot != BLOB_MAGIC {
            return Err(RecordError::BadMagic);
        }

        let (ver_slot, rest) = rest.split_at(2);
        let mut ver_b = [0u8; 2];
        ver_b.copy_from_slice(ver_slot);
        let version = u16::from_le_bytes(ver_b);
        if version != 1 {
            return Err(RecordError::UnsupportedVersion(version));
        }

        let (flags_slot, rest) = rest.split_at(2);
        let mut flags_b = [0u8; 2];
        flags_b.copy_from_slice(flags_slot);
        let flags = u16::from_le_bytes(flags_b);

        let (no_slot, rest) = rest.split_at(8);
        let mut no_b = [0u8; 8];
        no_b.copy_from_slice(no_slot);
        let blob_no = u64::from_le_bytes(no_b);

        let (gen_slot, rest) = rest.split_at(4);
        let mut gen_b = [0u8; 4];
        gen_b.copy_from_slice(gen_slot);
        let generation = u32::from_le_bytes(gen_b);

        let (space_slot, rest) = rest.split_at(2);
        let mut space_b = [0u8; 2];
        space_b.copy_from_slice(space_slot);
        let space = u16::from_le_bytes(space_b);

        let (res_slot, crc_slot) = rest.split_at(22);
        let mut reserved = [0u8; 22];
        reserved.copy_from_slice(res_slot);

        let mut crc_b = [0u8; 4];
        crc_b.copy_from_slice(crc_slot);
        let expected_chk = u32::from_le_bytes(crc_b);

        let (header_data, _) = slice.split_at(44);
        let actual_chk = crc32c(0, header_data);
        if actual_chk != expected_chk {
            return Err(RecordError::ChecksumMismatch {
                expected: expected_chk,
                actual: actual_chk,
            });
        }

        Ok(Self {
            version,
            flags,
            blob_no,
            generation,
            space,
            reserved,
        })
    }
}

/// Inhalt eines Blob-Records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobRecord {
    /// Art des Records (1 = Body).
    pub kind: u8,
    /// Verschlüsselungsdomäne des Körpers.
    pub domain: u32,
    /// Commit-Stempel des Changes.
    pub stamp: u64,
    /// Benutzerschlüssel.
    pub key: Vec<u8>,
    /// Rohwert des Wertkörpers.
    pub body: Vec<u8>,
}

impl BlobRecord {
    /// Erstellt einen neuen Körper-Record.
    pub fn new_body(domain: u32, stamp: u64, key: Vec<u8>, body: Vec<u8>) -> Self {
        Self {
            kind: RECORD_KIND_BODY,
            domain,
            stamp,
            key,
            body,
        }
    }

    /// Kodiert den Inhalt (content) nach SPEC §9.6:
    /// `kind u8 | domain u32 | stamp u64 | klen varint | key | body`
    pub fn encode_content(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.kind);
        buf.extend_from_slice(&self.domain.to_le_bytes());
        buf.extend_from_slice(&self.stamp.to_le_bytes());

        let mut v_buf = [0u8; 10];
        let klen_u64 = u64::try_from(self.key.len()).unwrap_or(0);
        let v_len = encode_u64(klen_u64, &mut v_buf);
        let (v_slice, _) = v_buf.split_at(v_len);
        buf.extend_from_slice(v_slice);

        buf.extend_from_slice(&self.key);
        buf.extend_from_slice(&self.body);
        buf
    }

    /// Dekodiert den Inhalt (content) nach SPEC §9.6.
    pub fn decode_content(buf: &[u8]) -> Result<Self, RecordError> {
        if buf.len() < 13 {
            return Err(RecordError::UnexpectedEof);
        }
        let (head, mut rest) = buf.split_at(13);

        let (kind_slot, head_rest) = head.split_at(1);
        let kind = kind_slot.first().copied().unwrap_or(0);

        let (dom_slot, stamp_slot) = head_rest.split_at(4);
        let mut dom_b = [0u8; 4];
        dom_b.copy_from_slice(dom_slot);
        let domain = u32::from_le_bytes(dom_b);

        let mut stamp_b = [0u8; 8];
        stamp_b.copy_from_slice(stamp_slot);
        let stamp = u64::from_le_bytes(stamp_b);

        let (klen_u64, consumed) =
            decode_u64(rest).map_err(|_| RecordError::InvalidFormat("Ungültiger klen varint"))?;
        let (_, r) = rest.split_at(consumed);
        rest = r;

        let klen = usize::try_from(klen_u64).map_err(|_| RecordError::Overflow)?;
        if rest.len() < klen {
            return Err(RecordError::UnexpectedEof);
        }

        let (key_bytes, body_bytes) = rest.split_at(klen);

        Ok(Self {
            kind,
            domain,
            stamp,
            key: key_bytes.to_vec(),
            body: body_bytes.to_vec(),
        })
    }
}
