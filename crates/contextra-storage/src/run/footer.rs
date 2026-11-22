//! # Footer: 28-Byte Trailer eines Runs
//!
//! Format (SPEC §9.5):
//! `"CTXR" | version u16 | flags u16 | directory_offset u64 | directory_len u32 | crc32c u32 | "CTXR"`
//! Keine Integer-Casts (M-02 = 0).

use super::error::RunError;
use crate::codec::crc32c::crc32c;

/// Magic-Bytes für Run-Dateien: `b"CTXR"`.
pub const RUN_MAGIC: &[u8; 4] = b"CTXR";

/// Unterstützte Run-Formatversion (Version 1).
pub const RUN_VERSION_1: u16 = 1;

/// Strukturierte Repräsentation des 28-Byte-Footers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Footer {
    /// Formatversion (aktuell 1).
    pub version: u16,
    /// Format-Flags.
    pub flags: u16,
    /// Byte-Offset des Directory-Blocks.
    pub directory_offset: u64,
    /// Länge des Directory-Blocks in Bytes.
    pub directory_len: u32,
    /// CRC32C-Prüfsumme über die ersten 20 Bytes des Footers.
    pub crc32c: u32,
}

impl Footer {
    /// Feste Länge des Footers auf dem Datenträger (28 B).
    pub const LEN: usize = 28;

    /// Erstellt einen neuen Footer für angegebene Directory-Lage.
    pub fn new(directory_offset: u64, directory_len: u32) -> Self {
        let mut f = Self {
            version: RUN_VERSION_1,
            flags: 0,
            directory_offset,
            directory_len,
            crc32c: 0,
        };
        let mut buf20 = [0u8; 20];
        f.write_first_20(&mut buf20);
        f.crc32c = crc32c(0, &buf20);
        f
    }

    fn write_first_20(&self, dst: &mut [u8; 20]) {
        if let Some(s) = dst.get_mut(0..4) {
            s.copy_from_slice(RUN_MAGIC);
        }
        let ver = self.version.to_le_bytes();
        if let Some(s) = dst.get_mut(4..6) {
            s.copy_from_slice(&ver);
        }
        let fl = self.flags.to_le_bytes();
        if let Some(s) = dst.get_mut(6..8) {
            s.copy_from_slice(&fl);
        }
        let off = self.directory_offset.to_le_bytes();
        if let Some(s) = dst.get_mut(8..16) {
            s.copy_from_slice(&off);
        }
        let len = self.directory_len.to_le_bytes();
        if let Some(s) = dst.get_mut(16..20) {
            s.copy_from_slice(&len);
        }
    }

    /// Kodiert den Footer in exakt 28 Bytes.
    pub fn encode(&self) -> [u8; Self::LEN] {
        let mut buf = [0u8; Self::LEN];
        let mut first20 = [0u8; 20];
        self.write_first_20(&mut first20);

        if let Some(s) = buf.get_mut(0..20) {
            s.copy_from_slice(&first20);
        }
        let chk = self.crc32c.to_le_bytes();
        if let Some(s) = buf.get_mut(20..24) {
            s.copy_from_slice(&chk);
        }
        if let Some(s) = buf.get_mut(24..28) {
            s.copy_from_slice(RUN_MAGIC);
        }
        buf
    }

    /// Dekodiert und validiert den Footer aus den letzten Bytes der Datei.
    pub fn decode(bytes: &[u8]) -> Result<Self, RunError> {
        let slice = bytes
            .get(0..Self::LEN)
            .ok_or_else(|| RunError::Corrupt("Footer zu kurz (< 28 B)".into()))?;

        // Magic 1 prüfen
        let m1 = slice
            .get(0..4)
            .ok_or_else(|| RunError::Corrupt("Magic 1 fehlt".into()))?;
        if m1 != RUN_MAGIC {
            return Err(RunError::Corrupt("Ungültiges Start-Magic in Footer".into()));
        }

        // Magic 2 prüfen
        let m2 = slice
            .get(24..28)
            .ok_or_else(|| RunError::Corrupt("Magic 2 fehlt".into()))?;
        if m2 != RUN_MAGIC {
            return Err(RunError::Corrupt("Ungültiges End-Magic in Footer".into()));
        }

        // CRC32C prüfen
        let first20 = slice
            .get(0..20)
            .ok_or_else(|| RunError::Corrupt("Header-Ausschnitt fehlt".into()))?;
        let expected_crc = crc32c(0, first20);

        let mut crc_bytes = [0u8; 4];
        if let Some(s) = slice.get(20..24) {
            crc_bytes.copy_from_slice(s);
        }
        let stored_crc = u32::from_le_bytes(crc_bytes);
        if stored_crc != expected_crc {
            return Err(RunError::Corrupt("Footer-Prüfsumme CRC32C ungültig".into()));
        }

        let mut ver_bytes = [0u8; 2];
        let mut fl_bytes = [0u8; 2];
        let mut off_bytes = [0u8; 8];
        let mut len_bytes = [0u8; 4];

        if let Some(s) = slice.get(4..6) {
            ver_bytes.copy_from_slice(s);
        }
        if let Some(s) = slice.get(6..8) {
            fl_bytes.copy_from_slice(s);
        }
        if let Some(s) = slice.get(8..16) {
            off_bytes.copy_from_slice(s);
        }
        if let Some(s) = slice.get(16..20) {
            len_bytes.copy_from_slice(s);
        }

        let version = u16::from_le_bytes(ver_bytes);
        let flags = u16::from_le_bytes(fl_bytes);
        let directory_offset = u64::from_le_bytes(off_bytes);
        let directory_len = u32::from_le_bytes(len_bytes);

        if version != RUN_VERSION_1 {
            return Err(RunError::Unsupported(format!(
                "Run-Version {version} nicht unterstützt (erwartet {RUN_VERSION_1})"
            )));
        }

        Ok(Self {
            version,
            flags,
            directory_offset,
            directory_len,
            crc32c: stored_crc,
        })
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn test_footer_roundtrip() {
        let f = Footer::new(12345, 678);
        assert_eq!(f.version, 1);
        assert_eq!(f.directory_offset, 12345);
        assert_eq!(f.directory_len, 678);

        let enc = f.encode();
        assert_eq!(enc.len(), 28);
        assert_eq!(&enc[0..4], b"CTXR");
        assert_eq!(&enc[24..28], b"CTXR");

        let dec = Footer::decode(&enc).unwrap_or(Footer::new(0, 0));
        assert_eq!(dec, f);
    }

    #[test]
    fn test_footer_corrupt_crc() {
        let f = Footer::new(100, 50);
        let mut enc = f.encode();
        if let Some(b) = enc.get_mut(10) {
            *b ^= 0xFF;
        }
        assert!(Footer::decode(&enc).is_err());
    }

    #[test]
    fn test_footer_corrupt_magic() {
        let f = Footer::new(100, 50);
        let mut enc = f.encode();
        if let Some(b) = enc.get_mut(0) {
            *b = b'X';
        }
        assert!(Footer::decode(&enc).is_err());
    }
}
