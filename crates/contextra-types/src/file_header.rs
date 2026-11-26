//! Common file header (SPEC Anhang B) and the CRC-32C it is protected with.
//!
//! Layout, little-endian, 41 bytes:
//!
//! | offset | size | field |
//! |---|---|---|
//! | 0  | 4  | magic |
//! | 4  | 2  | format_version |
//! | 6  | 2  | compat_flags |
//! | 8  | 4  | incompat_flags |
//! | 12 | 1  | file_kind |
//! | 13 | 16 | file_id |
//! | 29 | 8  | created_hlc |
//! | 37 | 4  | header_crc32c (over bytes 0..37) |
//!
//! Readers must reject unknown incompatible flags. Unknown compatible flags
//! may be ignored.

use crate::hlc::Hlc;
use core::fmt;

/// Encoded header length in bytes.
pub const FILE_HEADER_LEN: usize = 41;
/// Number of bytes covered by the checksum.
pub const FILE_HEADER_CHECKED_LEN: usize = 37;
/// Incompatible flags understood by this build. Any other bit makes a file unreadable.
pub const KNOWN_INCOMPAT_FLAGS: u32 = 0;

/// File kind, identified by its 4-byte magic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileMagic {
    /// Write-ahead log.
    Wal,
    /// Manifest log.
    Manifest,
    /// Truth pack (`.cxp`).
    Segment,
    /// Index file.
    Index,
    /// Blob file.
    Blob,
    /// Sidecar file.
    Sidecar,
}

impl FileMagic {
    /// The 4-byte magic.
    pub const fn bytes(self) -> [u8; 4] {
        match self {
            Self::Wal => *b"CWAL",
            Self::Manifest => *b"CMLG",
            Self::Segment => *b"CSEG",
            Self::Index => *b"CIDX",
            Self::Blob => *b"CBLB",
            Self::Sidecar => *b"CSID",
        }
    }

    /// Identifies a magic. `None` for unknown magic.
    pub fn from_bytes(magic: [u8; 4]) -> Option<FileMagic> {
        [
            Self::Wal,
            Self::Manifest,
            Self::Segment,
            Self::Index,
            Self::Blob,
            Self::Sidecar,
        ]
        .into_iter()
        .find(|m| m.bytes() == magic)
    }
}

/// Decoded common file header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileHeader {
    /// File kind.
    pub magic: FileMagic,
    /// Format version of this file kind.
    pub format_version: u16,
    /// Compatible feature flags.
    pub compat_flags: u16,
    /// Incompatible feature flags. Must be a subset of [`KNOWN_INCOMPAT_FLAGS`] to be read.
    pub incompat_flags: u32,
    /// Subtype byte, interpreted per file kind.
    pub file_kind: u8,
    /// Random file identifier.
    pub file_id: u128,
    /// Creation time as HLC.
    pub created_hlc: Hlc,
}

/// Errors from [`FileHeader::decode`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileHeaderError {
    /// Input is not exactly [`FILE_HEADER_LEN`] bytes.
    Length(usize),
    /// Magic does not match any known file kind.
    UnknownMagic([u8; 4]),
    /// Stored checksum does not match the header bytes.
    Checksum {
        /// Checksum stored in the file.
        stored: u32,
        /// Checksum computed over the header.
        computed: u32,
    },
    /// Incompatible flags that this build does not understand.
    UnknownIncompatFlags(u32),
}

impl fmt::Display for FileHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length(n) => write!(f, "header has {n} bytes, expected {FILE_HEADER_LEN}"),
            Self::UnknownMagic(m) => write!(f, "unknown file magic {m:?}"),
            Self::Checksum { stored, computed } => {
                write!(
                    f,
                    "header checksum mismatch: stored {stored:#010x}, computed {computed:#010x}"
                )
            }
            Self::UnknownIncompatFlags(bits) => {
                write!(f, "unknown incompatible flags {bits:#010x}")
            }
        }
    }
}

impl std::error::Error for FileHeaderError {}

impl FileHeader {
    /// Encodes the header and appends its CRC-32C.
    pub fn encode(&self) -> [u8; FILE_HEADER_LEN] {
        let mut out = [0u8; FILE_HEADER_LEN];
        out[0..4].copy_from_slice(&self.magic.bytes());
        out[4..6].copy_from_slice(&self.format_version.to_le_bytes());
        out[6..8].copy_from_slice(&self.compat_flags.to_le_bytes());
        out[8..12].copy_from_slice(&self.incompat_flags.to_le_bytes());
        out[12] = self.file_kind;
        out[13..29].copy_from_slice(&self.file_id.to_le_bytes());
        out[29..37].copy_from_slice(&self.created_hlc.to_le_bytes());
        let crc = crc32c(&out[..FILE_HEADER_CHECKED_LEN]);
        out[37..41].copy_from_slice(&crc.to_le_bytes());
        out
    }

    /// Decodes and validates a header.
    ///
    /// Checks length, magic, checksum and incompatible flags, in this order.
    pub fn decode(bytes: &[u8]) -> Result<FileHeader, FileHeaderError> {
        let arr: &[u8; FILE_HEADER_LEN] = bytes
            .try_into()
            .map_err(|_| FileHeaderError::Length(bytes.len()))?;

        let mut magic = [0u8; 4];
        magic.copy_from_slice(&arr[0..4]);
        let magic = FileMagic::from_bytes(magic).ok_or(FileHeaderError::UnknownMagic(magic))?;

        let stored = u32::from_le_bytes([arr[37], arr[38], arr[39], arr[40]]);
        let computed = crc32c(&arr[..FILE_HEADER_CHECKED_LEN]);
        if stored != computed {
            return Err(FileHeaderError::Checksum { stored, computed });
        }

        let incompat_flags = u32::from_le_bytes([arr[8], arr[9], arr[10], arr[11]]);
        if incompat_flags & !KNOWN_INCOMPAT_FLAGS != 0 {
            return Err(FileHeaderError::UnknownIncompatFlags(incompat_flags));
        }

        let mut file_id = [0u8; 16];
        file_id.copy_from_slice(&arr[13..29]);
        let mut hlc = [0u8; 8];
        hlc.copy_from_slice(&arr[29..37]);

        Ok(FileHeader {
            magic,
            format_version: u16::from_le_bytes([arr[4], arr[5]]),
            compat_flags: u16::from_le_bytes([arr[6], arr[7]]),
            incompat_flags,
            file_kind: arr[12],
            file_id: u128::from_le_bytes(file_id),
            created_hlc: Hlc::from_le_bytes(hlc),
        })
    }
}

/// CRC-32C (Castagnoli, reflected polynomial `0x82F63B78`).
///
/// Bitwise implementation without tables, so it adds no dependency. Header
/// checks are not on a hot path.
pub fn crc32c(data: &[u8]) -> u32 {
    let mut crc: u32 = !0;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0x82F6_3B78 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> FileHeader {
        FileHeader {
            magic: FileMagic::Wal,
            format_version: 1,
            compat_flags: 0,
            incompat_flags: 0,
            file_kind: 2,
            file_id: 0x0123_4567_89AB_CDEF_0011_2233_4455_6677,
            created_hlc: Hlc::from_parts(1_700_000_000_000, 9).unwrap_or(Hlc::ZERO),
        }
    }

    #[test]
    fn crc32c_standard_check_value() {
        // Well-known check value of CRC-32C for "123456789".
        assert_eq!(crc32c(b"123456789"), 0xE306_9283);
        assert_eq!(crc32c(b""), 0);
    }

    #[test]
    fn header_roundtrip() {
        let h = sample();
        let bytes = h.encode();
        assert_eq!(bytes.len(), FILE_HEADER_LEN);
        assert_eq!(FileHeader::decode(&bytes), Ok(h));
    }

    #[test]
    fn every_magic_roundtrips() {
        for m in [
            FileMagic::Wal,
            FileMagic::Manifest,
            FileMagic::Segment,
            FileMagic::Index,
            FileMagic::Blob,
            FileMagic::Sidecar,
        ] {
            assert_eq!(FileMagic::from_bytes(m.bytes()), Some(m));
        }
        assert_eq!(FileMagic::from_bytes(*b"XXXX"), None);
    }

    #[test]
    fn corrupted_byte_is_detected_by_checksum() {
        let mut bytes = sample().encode();
        bytes[20] ^= 0x01;
        assert!(matches!(
            FileHeader::decode(&bytes),
            Err(FileHeaderError::Checksum { .. })
        ));
    }

    #[test]
    fn unknown_incompat_flag_is_rejected_after_valid_checksum() {
        let mut h = sample();
        h.incompat_flags = 1 << 4;
        let bytes = h.encode();
        assert_eq!(
            FileHeader::decode(&bytes),
            Err(FileHeaderError::UnknownIncompatFlags(1 << 4))
        );
    }

    #[test]
    fn wrong_length_and_unknown_magic() {
        assert_eq!(
            FileHeader::decode(&[0u8; 10]),
            Err(FileHeaderError::Length(10))
        );
        let mut bytes = sample().encode();
        bytes[0..4].copy_from_slice(b"ZZZZ");
        assert!(matches!(
            FileHeader::decode(&bytes),
            Err(FileHeaderError::UnknownMagic(_))
        ));
    }
}
