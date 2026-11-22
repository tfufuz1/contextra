//! # Segmentkopf (48 Byte)
//!
//! SPEC §9.4: Header (48 B):
//! `"CTXL" | version u16 | flags u16 | segment_id u64 | first_stamp u64 | chain_seed u32 (Entropy)`
//! `| prealloc_len u64 | reserved u64 | crc32c u32`

use crate::codec::crc32c::crc32c;
use crate::redo::error::RedoError;

/// Feste Länge des Segmentkopfes in Bytes.
pub const HEADER_SIZE: usize = 48;

/// Magische Kennung für Redo-Segmente ("CTXL").
pub const MAGIC: [u8; 4] = *b"CTXL";

/// Aktuelle Formatversion des Segmentkopfes.
pub const FORMAT_VERSION: u16 = 1;

/// 48-Byte Segmentkopf eines Redo-Segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedoHeader {
    /// Formatversion (1).
    pub version: u16,
    /// Header-Flags (derzeit 0).
    pub flags: u16,
    /// Eindeutige Segment-ID.
    pub segment_id: u64,
    /// Erster Commit-Stamp in diesem Segment (oder 0 falls leer).
    pub first_stamp: u64,
    /// Zufälliger Kettenstartwert für Plain-Prüfsummenkette (SPEC §9.4, §10.5).
    pub chain_seed: u32,
    /// Vorallokierte Dateilänge in Bytes.
    pub prealloc_len: u64,
    /// Reservierter Bereich (0).
    pub reserved: u64,
    /// CRC32C über die vorherigen 44 Bytes mit Seed 0.
    pub crc32c: u32,
}

impl RedoHeader {
    /// Erstellt einen neuen Segmentkopf und berechnet die Prüfsumme.
    pub fn new(
        segment_id: u64,
        first_stamp: u64,
        chain_seed: u32,
        prealloc_len: u64,
    ) -> Self {
        let mut h = Self {
            version: FORMAT_VERSION,
            flags: 0,
            segment_id,
            first_stamp,
            chain_seed,
            prealloc_len,
            reserved: 0,
            crc32c: 0,
        };
        let mut buf = [0u8; 48];
        h.encode_prefix(&mut buf);
        let chk = crc32c(0, match buf.get(..44) {
            Some(slice) => slice,
            None => &[],
        });
        h.crc32c = chk;
        h
    }

    fn encode_prefix(&self, dst: &mut [u8; 48]) {
        if let Some(target) = dst.get_mut(0..4) {
            target.copy_from_slice(&MAGIC);
        }
        if let Some(target) = dst.get_mut(4..6) {
            target.copy_from_slice(&self.version.to_le_bytes());
        }
        if let Some(target) = dst.get_mut(6..8) {
            target.copy_from_slice(&self.flags.to_le_bytes());
        }
        if let Some(target) = dst.get_mut(8..16) {
            target.copy_from_slice(&self.segment_id.to_le_bytes());
        }
        if let Some(target) = dst.get_mut(16..24) {
            target.copy_from_slice(&self.first_stamp.to_le_bytes());
        }
        if let Some(target) = dst.get_mut(24..28) {
            target.copy_from_slice(&self.chain_seed.to_le_bytes());
        }
        if let Some(target) = dst.get_mut(28..36) {
            target.copy_from_slice(&self.prealloc_len.to_le_bytes());
        }
        if let Some(target) = dst.get_mut(36..44) {
            target.copy_from_slice(&self.reserved.to_le_bytes());
        }
    }

    /// Kodiert den vollständigen 48-Byte-Kopf inklusive CRC32C.
    pub fn encode(&self, dst: &mut [u8; 48]) {
        self.encode_prefix(dst);
        if let Some(target) = dst.get_mut(44..48) {
            target.copy_from_slice(&self.crc32c.to_le_bytes());
        }
    }

    /// Liefert den Header als 48-Byte-Array.
    pub fn to_bytes(&self) -> [u8; 48] {
        let mut buf = [0u8; 48];
        self.encode(&mut buf);
        buf
    }

    /// Dekodiert und verifiziert einen 48-Byte-Segmentkopf.
    pub fn decode(src: &[u8; 48]) -> Result<Self, RedoError> {
        let magic = match src.get(0..4) {
            Some(m) => m,
            None => return Err(RedoError::Corrupt("Segmentkopf zu kurz".into())),
        };
        if magic != MAGIC {
            return Err(RedoError::Corrupt("Ungültige magische Segmentkennung".into()));
        }

        let prefix = match src.get(0..44) {
            Some(p) => p,
            None => return Err(RedoError::Corrupt("Segmentkopf-Präfix fehlt".into())),
        };
        let expected_crc = crc32c(0, prefix);

        let crc_bytes: [u8; 4] = match src.get(44..48).and_then(|s| s.try_into().ok()) {
            Some(b) => b,
            None => return Err(RedoError::Corrupt("Segmentkopf-CRC fehlt".into())),
        };
        let stored_crc = u32::from_le_bytes(crc_bytes);

        if stored_crc != expected_crc {
            return Err(RedoError::Corrupt("Segmentkopf-Prüfsumme fehlerhaft".into()));
        }

        let version = u16::from_le_bytes(
            match src.get(4..6).and_then(|s| s.try_into().ok()) {
                Some(b) => b,
                None => [0, 0],
            },
        );
        if version != FORMAT_VERSION {
            return Err(RedoError::Unsupported(format!(
                "Nicht unterstützte Segmentversion: {version}"
            )));
        }

        let flags = u16::from_le_bytes(
            match src.get(6..8).and_then(|s| s.try_into().ok()) {
                Some(b) => b,
                None => [0, 0],
            },
        );
        let segment_id = u64::from_le_bytes(
            match src.get(8..16).and_then(|s| s.try_into().ok()) {
                Some(b) => b,
                None => [0; 8],
            },
        );
        let first_stamp = u64::from_le_bytes(
            match src.get(16..24).and_then(|s| s.try_into().ok()) {
                Some(b) => b,
                None => [0; 8],
            },
        );
        let chain_seed = u32::from_le_bytes(
            match src.get(24..28).and_then(|s| s.try_into().ok()) {
                Some(b) => b,
                None => [0; 4],
            },
        );
        let prealloc_len = u64::from_le_bytes(
            match src.get(28..36).and_then(|s| s.try_into().ok()) {
                Some(b) => b,
                None => [0; 8],
            },
        );
        let reserved = u64::from_le_bytes(
            match src.get(36..44).and_then(|s| s.try_into().ok()) {
                Some(b) => b,
                None => [0; 8],
            },
        );

        Ok(Self {
            version,
            flags,
            segment_id,
            first_stamp,
            chain_seed,
            prealloc_len,
            reserved,
            crc32c: stored_crc,
        })
    }
}
