//! # SSTable-Format: Konstanten, Header & Trailer
//!
//! Zweck: Zentralisiert die Format-Konstanten, Magic-Bytes, Trailer-Größen
//! und Versionsprüfung für SSTables (Format V4 MFSX und Legacy V3 MFST).
//!
//! Invarianten:
//! - Magic `SSTABLE_MAGIC_MFSX` = `0x5853_464D` (b"MFSX" little-endian)
//! - Magic `SSTABLE_MAGIC_LEGACY` = `0x4D46_5354` (b"MFST" little-endian)
//! - Bit 63 (`TOMBSTONE_BIT`) wird vor `max_seq`-Vergleichen maskiert (INV-STORE-4).
//! - Byte-Identität persistierter Formate bleibt strikt gewahrt.

use crate::error::{StoreError, StoreResult};

/// Magic-Bytes für V4 SSTable: `MFSX` in Hex (0x5853_464D).
pub const SSTABLE_MAGIC_MFSX: u32 = 0x5853_464D;

/// Magic-Bytes für Legacy SSTable: `MFST` in Hex (0x4D46_5354).
pub const SSTABLE_MAGIC_LEGACY: u32 = 0x4D46_5354;

/// Standard-Blockgröße für SSTable Data Blocks (4 KiB).
pub const BLOCK_SIZE: usize = 4096;

/// Aktuelle unterstützte SSTable-Formatversion (V4).
pub const CURRENT_SSTABLE_FORMAT_VERSION: u16 = 4;

/// Trailer-Länge für MFSX Format Version >= 1 (54 Bytes).
pub const MFSX_TRAILER_LEN_V1: usize = 54;

/// Trailer-Länge für MFSX Format Version 0 (52 Bytes).
pub const MFSX_TRAILER_LEN_V0: usize = 52;

/// Trailer-Länge für Legacy MFST (54 Bytes).
pub const LEGACY_TRAILER_LEN: usize = 54;

/// Strukturierte Repräsentation des SSTable-Footers / Trailers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SstableFooter {
    /// Format-Version (z. B. 4 für MFSX v4, 0/1 für frühere MFSX, 3 für Legacy).
    pub format_version: u16,
    /// Gibt an, ob es sich um das moderne MFSX-Format handelt (sonst Legacy MFST).
    pub is_mfsx: bool,
    /// Byte-Offset des Bloom-Filters in der SSTable (0 falls kein Bloom-Filter vorhanden).
    pub bloom_offset: u64,
    /// Byte-Offset des Block-Index in der SSTable.
    pub index_offset: u64,
    /// Kleinste enthaltene Transaktions-ID.
    pub min_tx_id: u64,
    /// Größte enthaltene Transaktions-ID.
    pub max_tx_id: u64,
    /// Kleinste enthaltene Sequenznummer.
    pub min_seq: u64,
    /// Größte enthaltene Sequenznummer (maskiert von Tombstone-Bits).
    pub max_seq: u64,
    /// Trailer-Länge in Bytes.
    pub trailer_len: usize,
}

impl SstableFooter {
    /// Parst und validiert den SSTable-Trailer aus den letzten Bytes der Datei.
    pub fn parse(trailer_data: &[u8], path_display: &str) -> StoreResult<Self> {
        let trailer_len = trailer_data.len();
        if trailer_len < 12 {
            return Err(StoreError::Storage("Invalid trailer".into()));
        }

        let mut format_version = 0u16;
        let mut is_mfsx = false;

        if trailer_len >= MFSX_TRAILER_LEN_V1 {
            if let Some(slice) = trailer_data.get(50..54) {
                let magic_v1 = u32::from_le_bytes(
                    slice
                        .try_into()
                        .map_err(|_| StoreError::Storage("Invalid trailer".into()))?,
                );
                if magic_v1 == SSTABLE_MAGIC_MFSX {
                    if let Some(ver_slice) = trailer_data.get(48..50) {
                        format_version = u16::from_le_bytes(
                            ver_slice
                                .try_into()
                                .map_err(|_| StoreError::Storage("Invalid trailer".into()))?,
                        );
                        is_mfsx = true;
                    }
                }
            }
        }

        if !is_mfsx && trailer_len >= MFSX_TRAILER_LEN_V0 {
            if let Some(slice) = trailer_data.get(48..52) {
                let magic_v0 = u32::from_le_bytes(
                    slice
                        .try_into()
                        .map_err(|_| StoreError::Storage("Invalid trailer".into()))?,
                );
                if magic_v0 == SSTABLE_MAGIC_MFSX {
                    is_mfsx = true;
                    format_version = 0;
                }
            }
        }

        if is_mfsx && format_version > CURRENT_SSTABLE_FORMAT_VERSION {
            return Err(StoreError::Storage(format!(
                "Unsupported SSTable format version: {}",
                format_version
            )));
        }

        let mut bloom_offset = 0;
        let mut min_tx_id = 0;
        let mut max_tx_id = 0;
        let min_seq;
        let max_seq;

        let index_offset = if is_mfsx {
            let base = if format_version >= 1 { 0 } else { 2 };
            min_tx_id = u64::from_le_bytes(
                trailer_data
                    .get(base..base + 8)
                    .ok_or_else(|| StoreError::ParseError("Invalid min_tx_id".into()))?
                    .try_into()
                    .map_err(|_| StoreError::ParseError("Invalid min_tx_id".into()))?,
            );
            max_tx_id = u64::from_le_bytes(
                trailer_data
                    .get(base + 8..base + 16)
                    .ok_or_else(|| StoreError::ParseError("Invalid max_tx_id".into()))?
                    .try_into()
                    .map_err(|_| StoreError::ParseError("Invalid max_tx_id".into()))?,
            );
            min_seq = u64::from_le_bytes(
                trailer_data
                    .get(base + 16..base + 24)
                    .ok_or_else(|| StoreError::ParseError("Invalid min_seq".into()))?
                    .try_into()
                    .map_err(|_| StoreError::ParseError("Invalid min_seq".into()))?,
            );
            max_seq = u64::from_le_bytes(
                trailer_data
                    .get(base + 24..base + 32)
                    .ok_or_else(|| StoreError::ParseError("Invalid max_seq".into()))?
                    .try_into()
                    .map_err(|_| StoreError::ParseError("Invalid max_seq".into()))?,
            );
            bloom_offset = u64::from_le_bytes(
                trailer_data
                    .get(base + 32..base + 40)
                    .ok_or_else(|| StoreError::ParseError("Invalid bloom_offset".into()))?
                    .try_into()
                    .map_err(|_| StoreError::ParseError("Invalid bloom_offset".into()))?,
            );
            u64::from_le_bytes(
                trailer_data
                    .get(base + 40..base + 48)
                    .ok_or_else(|| StoreError::ParseError("Invalid index_offset".into()))?
                    .try_into()
                    .map_err(|_| StoreError::ParseError("Invalid index_offset".into()))?,
            )
        } else if trailer_len >= LEGACY_TRAILER_LEN {
            let magic_legacy = u32::from_le_bytes(
                trailer_data[50..54]
                    .try_into()
                    .map_err(|_| StoreError::checksum_mismatch(path_display, 0))?,
            );
            if magic_legacy == SSTABLE_MAGIC_LEGACY {
                format_version = 3;
                min_seq = 0;
                max_seq = 0;
                u64::from_le_bytes(
                    trailer_data[42..50]
                        .try_into()
                        .map_err(|_| StoreError::ParseError("Invalid index offset".into()))?,
                )
            } else {
                return Err(StoreError::Storage("Invalid SSTable magic number".into()));
            }
        } else {
            return Err(StoreError::Storage("Invalid SSTable trailer length".into()));
        };

        let trailer_actual_len = if is_mfsx {
            if format_version >= 1 {
                MFSX_TRAILER_LEN_V1
            } else {
                MFSX_TRAILER_LEN_V0
            }
        } else {
            LEGACY_TRAILER_LEN
        };

        Ok(Self {
            format_version,
            is_mfsx,
            bloom_offset,
            index_offset,
            min_tx_id,
            max_tx_id,
            min_seq,
            max_seq,
            trailer_len: trailer_actual_len,
        })
    }
}
