//! F.5 SSTable Parser (`kind = 2`).
//!
//! SPEC Anhang F.5:
//! - Kopf nach F.1 (`kind = 2`) -> 64 Bytes
//! - Datenblöcke (nominal 32 KiB Klartext) mit Einträgen:
//!   `klen(u16) || key || vlen(u32) || value`
//!   - value = Envelope v2 (F.6) oder Tombstone (`vlen = 0`)
//!   - aufsteigend nach key sortiert
//! - Indexblock (`key_lo`, Offset, Länge je Datenblock)
//! - Bloomblock (Schlüsselhash über tenant||coll||cluster||doc_tag Präfix, 10 Bit/Schlüssel)
//! - Statistikblock (`min_seq`, `max_seq`, Zähler)
//! - Footer (fest 64 B, unverschlüsselt außer Tag):
//!   `index_off(u64) || bloom_off(u64) || stats_off(u64) || block_count(u64) || footer_tag(16) || magic(8)`
//!   - magic(8) = "CTXSST\0\1"

use crate::error::LabError;
use crate::format::file_header::{FileHeader, FileKind, FILE_HEADER_SIZE};

/// Feste Footer-Größe für SSTables (64 Byte).
pub const SSTABLE_FOOTER_SIZE: usize = 64;

/// Magische Bytes für SSTable Footer.
pub const SSTABLE_FOOTER_MAGIC: &[u8; 8] = &[0x43, 0x54, 0x58, 0x53, 0x53, 0x54, 0x00, 0x01];

/// Strukturierter SSTable Footer (64 Byte).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsTableFooter {
    /// Dateioffset des Indexblocks.
    pub index_off: u64,
    /// Dateioffset des Bloomfilters.
    pub bloom_off: u64,
    /// Dateioffset des Statistikblocks.
    pub stats_off: u64,
    /// Anzahl Datenblöcke.
    pub block_count: u64,
    /// AEAD/Integritäts-Tag des Footers.
    pub footer_tag: [u8; 16],
}

impl SsTableFooter {
    /// Parst die 64 Byte des Footers.
    pub fn parse(bytes: &[u8]) -> Result<Self, LabError> {
        if bytes.len() < SSTABLE_FOOTER_SIZE {
            return Err(LabError::UnexpectedEof {
                expected: SSTABLE_FOOTER_SIZE,
                actual: bytes.len(),
            });
        }

        let magic = bytes.get(56..64).ok_or(LabError::InvalidMagic)?;
        if magic != SSTABLE_FOOTER_MAGIC {
            return Err(LabError::InvalidMagic);
        }

        let index_off = u64::from_be_bytes(
            bytes
                .get(0..8)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("index_off fehlt"))?,
        );
        let bloom_off = u64::from_be_bytes(
            bytes
                .get(8..16)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("bloom_off fehlt"))?,
        );
        let stats_off = u64::from_be_bytes(
            bytes
                .get(16..24)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("stats_off fehlt"))?,
        );
        let block_count = u64::from_be_bytes(
            bytes
                .get(24..32)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("block_count fehlt"))?,
        );

        let footer_tag: [u8; 16] = bytes
            .get(32..48)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("footer_tag fehlt"))?;

        let reserved = bytes
            .get(48..56)
            .ok_or(LabError::Corrupted("Reserved fehlt"))?;
        if reserved.iter().any(|&b| b != 0) {
            return Err(LabError::Corrupted("Footer Reserved ist nicht 0"));
        }

        Ok(Self {
            index_off,
            bloom_off,
            stats_off,
            block_count,
            footer_tag,
        })
    }
}

/// Eintrag in einem SSTable Datenblock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsTableEntry {
    /// Schlüssel.
    pub key: Vec<u8>,
    /// Wert oder leer bei Tombstone.
    pub value: Vec<u8>,
    /// True, wenn Tombstone (vlen == 0).
    pub is_tombstone: bool,
}

impl SsTableEntry {
    /// Parst sequentiell Einträge aus einem Klartext-Datenblock.
    pub fn parse_entries(mut slice: &[u8]) -> Result<Vec<Self>, LabError> {
        let mut entries = Vec::new();
        let mut prev_key: Option<Vec<u8>> = None;

        while !slice.is_empty() {
            if slice.len() < 2 {
                return Err(LabError::UnexpectedEof {
                    expected: 2,
                    actual: slice.len(),
                });
            }

            let klen = u16::from_be_bytes(
                slice
                    .get(0..2)
                    .and_then(|s| s.try_into().ok())
                    .ok_or(LabError::Corrupted("klen fehlt"))?,
            ) as usize;

            let after_klen = slice.get(2..).ok_or(LabError::Corrupted("Slice fehlt"))?;
            if after_klen.len() < klen + 4 {
                return Err(LabError::UnexpectedEof {
                    expected: klen + 4,
                    actual: after_klen.len(),
                });
            }

            let key = after_klen
                .get(0..klen)
                .ok_or(LabError::Corrupted("key fehlt"))?
                .to_vec();

            if let Some(ref pk) = prev_key {
                if &key <= pk {
                    return Err(LabError::Corrupted(
                        "SSTable Einträge nicht strikt aufsteigend sortiert",
                    ));
                }
            }

            let vlen = u32::from_be_bytes(
                after_klen
                    .get(klen..klen + 4)
                    .and_then(|s| s.try_into().ok())
                    .ok_or(LabError::Corrupted("vlen fehlt"))?,
            ) as usize;

            let after_vlen = after_klen
                .get(klen + 4..)
                .ok_or(LabError::Corrupted("Slice fehlt"))?;
            if after_vlen.len() < vlen {
                return Err(LabError::UnexpectedEof {
                    expected: vlen,
                    actual: after_vlen.len(),
                });
            }

            let value = after_vlen
                .get(0..vlen)
                .ok_or(LabError::Corrupted("value fehlt"))?
                .to_vec();

            let is_tombstone = vlen == 0;
            prev_key = Some(key.clone());
            slice = after_vlen.get(vlen..).unwrap_or(&[]);

            entries.push(Self {
                key,
                value,
                is_tombstone,
            });
        }

        Ok(entries)
    }
}

/// Strukturierte Repräsentation einer SSTable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsTableFile {
    /// F.1 Dateikopf.
    pub header: FileHeader,
    /// Footer.
    pub footer: SsTableFooter,
}

impl SsTableFile {
    /// Parst eine SSTable-Datei und ihren Footer.
    pub fn parse(bytes: &[u8]) -> Result<Self, LabError> {
        let min_size = FILE_HEADER_SIZE + SSTABLE_FOOTER_SIZE;
        if bytes.len() < min_size {
            return Err(LabError::UnexpectedEof {
                expected: min_size,
                actual: bytes.len(),
            });
        }

        let header = FileHeader::parse(bytes)?;
        if header.kind != FileKind::SsTable {
            return Err(LabError::UnsupportedKind(header.kind as u8));
        }

        let footer_bytes = bytes
            .get(bytes.len() - SSTABLE_FOOTER_SIZE..)
            .ok_or(LabError::Corrupted("Footer fehlt"))?;
        let footer = SsTableFooter::parse(footer_bytes)?;

        if footer.index_off < FILE_HEADER_SIZE as u64
            || footer.bloom_off < footer.index_off
            || footer.stats_off < footer.bloom_off
            || footer.stats_off > (bytes.len() - SSTABLE_FOOTER_SIZE) as u64
        {
            return Err(LabError::Corrupted(
                "Inkonsistente Block-Offsets im SSTable Footer",
            ));
        }

        Ok(Self { header, footer })
    }
}
