// FILE-CONTEXT
// STAND: 2026-10-08 (T-2026-9104)
// ZWECK: Eigenständiger, robuster Parser für SSTables im Witness-Lesepfad.
//
// ETAPPE 0 DOKUMENTATION — ON-DISK-FORMAT & ETAPPE 0 BEFUNDE (AP-08, AP-09):
//
// 1. ON-DISK FORMAT DER SSTABLE (MFSX / Legacy):
//    - Magic:
//      - Format v1..v4: `SSTABLE_MAGIC_MFSX` = 0x5853_464D (MFSX in Little Endian: 'M','F','S','X').
//      - Format v0: Trailer-Länge 52, Magic `SSTABLE_MAGIC_MFSX` an Offset -4.
//      - Legacy-Format: Magic `SSTABLE_MAGIC_LEGACY` = 0x5853_5453 ('S','T','S','X') an Offset -4.
//    - Footer / Trailer (am Dateiende geschrieben):
//      - v1..v4 (54 Bytes):
//        `[min_tx_id: u64 LE]` (0..8)
//        `[max_tx_id: u64 LE]` (8..16)
//        `[min_seq: u64 LE]` (16..24)
//        `[max_seq: u64 LE]` (24..32)
//        `[bloom_offset: u64 LE]` (32..40)
//        `[index_offset: u64 LE]` (40..48)
//        `[format_version: u16 LE]` (48..50) (z. B. 3 oder 4)
//        `[magic: u32 LE]` (50..54) (SSTABLE_MAGIC_MFSX)
//      - Min/Max Key im Footer (AP-08 Befund):
//        Der Footer speichert `min_seq`/`max_seq` sowie `min_tx_id`/`max_tx_id`, speichert jedoch NICHT
//        direkt `first_key` oder `last_key`! Stattdessen wird `last_key` aus dem letzten Index-Eintrag
//        abgeleitet und `first_key` durch Lesen des ersten Eintrags im ersten Block (Offset 0) bestimmt.
//    - Datenblöcke (Offset 0 bis `index_offset`):
//      - Verschlüsselung: Wenn KeyManager/Passphrase übergeben wird, ist dem Block-Payload eine 12-Byte-Nonce
//        vorangestellt (`[nonce: 12B][encrypted_payload]`). Ohne Schlüssel meldet der Reader `NotWitnessable`.
//      - CRC32: 4-Byte CRC32 LE dem Block-Payload vorangestellt: `[crc32: u32 LE][block_data]`.
//      - Einträge (KV-Layout):
//        `[key_len: u16 LE]` (2B)
//        `[key_bytes]` (key_len Bytes)
//        `[seq_no: u64 LE]` (8B; Bit 63 = TOMBSTONE_BIT)
//        `[tx_id: u64 LE]` (8B)
//        `[val_len: u32 LE]` (4B)
//        `[val_bytes]` (val_len Bytes)
//      - Block-Trailer:
//        - Format v4: `[bloom_bytes...][offsets: u32 LE * n][bloom_len: u32 LE][num_offsets: u32 LE]`
//        - Format v3: `[fixed_bloom: 32B][offsets: u32 LE * n][num_offsets: u32 LE]`
//        - Format v0..v2: `[fixed_bloom: 8B][offsets: u16 LE * n][num_offsets: u16 LE]`
//    - Index-Sektion (Offset `index_offset` bis `bloom_offset`):
//      - optionales 4-Byte-CRC32-Präfix `[crc32: u32 LE]`
//      - Sequenz von Index-Einträgen: `[key_len: u16 LE][last_key_bytes][block_offset: u64 LE]`
//    - Bloom-Filter-Sektion (Offset `bloom_offset` bis Trailer):
//      - optionales 4-Byte-CRC32-Präfix `[crc32: u32 LE]` + Bloom-Filter-Nutzlast.
//    - Tombstones & delete_prefix:
//      - Tombstone: Bit 63 in `seq_no` gesetzt (`seq_no & TOMBSTONE_BIT != 0`).
//
// 2. KLASSENTABELLE DER INVENTUR (DATEINAMEN UND VERZEICHNISSE):
//    - Live: Validierte `.sst`-Datei, die im aktuellen Manifest aufgeführt ist.
//    - ObsoletePending: `.sst`-Datei, die in `pending_delete_files` gelistet ist.
//    - Orphan: `.sst`-Datei im Datenverzeichnis, die NICHT im Manifest existiert.
//    - Temp: `.tmp`-Dateien, die bei temporären Schreibvorgängen oder Compaction entstehen.
//    - OldManifest: Historische/Sicherungs-Manifeste (`MANIFEST.old`, `MANIFEST.bak`, `MANIFEST.tmp.*`).
//    - Unknown: Unbekannte/nicht-zuordnungsbare Datei im Datenverzeichnis -> `UnclassifiedFile` (fail-closed).
//
// 3. MULTI-VERSIONIERUNG & SCHLÜSSEL-RETENTION (AP-09 BEFUND):
//    - SSTables enthalten mehrere Versionen desselben Schlüssels, sortiert nach Key ASC, `raw_seq` DESC.
//    - Der WitnessReader & Scanner durchmustern ALLE Versionen (`seq <= as_of_tx`), um auch durch Tombstones
//      oder neuere Werte überschattete historische Schlüssel/Werte (z. B. Injektionsmatrix Zeile 5) nachweislich zu finden.

use super::witness_types::{WitnessEntry, WitnessReadError, WitnessSstMetadata};
use crate::wal::KeyManager;
use bytes::Bytes;
use std::fmt;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const WITNESS_SSTABLE_MAGIC_MFSX: u32 = 0x5853_464D;
pub const WITNESS_SSTABLE_MAGIC_LEGACY: u32 = 0x5853_5453;

/// Independent, clean-room WitnessReader parser for SSTable validation and offline audit.
pub struct WitnessReader {
    path: PathBuf,
    file_size: u64,
    file: File,
    metadata: WitnessSstMetadata,
    index: Vec<(Bytes, u64)>,
    index_offset: u64,
    bloom_offset: u64,
    has_crc: bool,
    format_version: u16,
    key_manager: Option<Arc<KeyManager>>,
    is_encrypted: bool,
}

impl fmt::Debug for WitnessReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WitnessReader")
            .field("path", &self.path)
            .field("file_size", &self.file_size)
            .field("metadata", &self.metadata)
            .field("index_len", &self.index.len())
            .field("index_offset", &self.index_offset)
            .field("format_version", &self.format_version)
            .field("is_encrypted", &self.is_encrypted)
            .finish()
    }
}

impl WitnessReader {
    /// Opens and parses an SSTable file independently without production reader code.
    pub fn open(
        path: impl AsRef<Path>,
        key_manager: Option<Arc<KeyManager>>,
    ) -> Result<Self, WitnessReadError> {
        let path_buf = path.as_ref().to_path_buf();
        let derived_km = if let Some(ref km) = key_manager {
            let file_id = path_buf
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default();
            match km.derive_file_key(file_id.as_bytes()) {
                Ok(k) => Some(Arc::new(k)),
                Err(e) => {
                    return Err(WitnessReadError::CorruptFile(format!(
                        "Key derivation failed: {e}"
                    )))
                }
            }
        } else {
            None
        };

        let mut file = File::open(&path_buf)
            .map_err(|e| WitnessReadError::Io(format!("Failed to open SSTable: {e}")))?;

        let file_size = file
            .metadata()
            .map_err(|e| WitnessReadError::Io(format!("Failed to read metadata: {e}")))?
            .len();

        if file_size < 12 {
            return Err(WitnessReadError::CorruptFile(
                "SSTable file too small (< 12 bytes)".into(),
            ));
        }

        // Read trailer
        let read_trailer_len = usize::try_from(54u64.min(file_size)).unwrap_or(54);
        let mut trailer_buf = vec![0u8; read_trailer_len];
        let trailer_seek_pos = file_size.saturating_sub(read_trailer_len as u64);
        file.seek(SeekFrom::Start(trailer_seek_pos))
            .map_err(|e| WitnessReadError::Io(format!("Seek trailer failed: {e}")))?;
        file.read_exact(&mut trailer_buf)
            .map_err(|e| WitnessReadError::Io(format!("Read trailer failed: {e}")))?;

        let trailer_len = trailer_buf.len();
        if trailer_len < 12 {
            return Err(WitnessReadError::CorruptFile("Trailer too short".into()));
        }

        let mut format_version = 0u16;
        let mut is_mfsx = false;

        if trailer_len >= 54 {
            if let Some(slice) = trailer_buf.get(50..54) {
                let magic = u32::from_le_bytes(slice.try_into().unwrap_or([0; 4]));
                if magic == WITNESS_SSTABLE_MAGIC_MFSX {
                    if let Some(ver_slice) = trailer_buf.get(48..50) {
                        format_version = u16::from_le_bytes(ver_slice.try_into().unwrap_or([0; 2]));
                        is_mfsx = true;
                    }
                }
            }
        }

        if !is_mfsx && trailer_len >= 52 {
            if let Some(slice) = trailer_buf.get(48..52) {
                let magic = u32::from_le_bytes(slice.try_into().unwrap_or([0; 4]));
                if magic == WITNESS_SSTABLE_MAGIC_MFSX {
                    is_mfsx = true;
                    format_version = 0;
                }
            }
        }

        if !is_mfsx && trailer_len >= 54 {
            if let Some(slice) = trailer_buf.get(50..54) {
                let magic = u32::from_le_bytes(slice.try_into().unwrap_or([0; 4]));
                if magic == WITNESS_SSTABLE_MAGIC_LEGACY {
                    return Err(WitnessReadError::CorruptFile(
                        "Legacy SSTable format detected (not supported in witness mode)".into(),
                    ));
                }
            }
        }

        if !is_mfsx {
            return Err(WitnessReadError::CorruptFile(
                "Invalid or unrecognized SSTable magic header".into(),
            ));
        }

        if format_version > 4 {
            return Err(WitnessReadError::CorruptFile(format!(
                "Unsupported SSTable format version {format_version}"
            )));
        }

        let base = if format_version >= 1 { 0 } else { 2 };
        let min_tx_id = u64::from_le_bytes(
            trailer_buf
                .get(base..base + 8)
                .ok_or_else(|| {
                    WitnessReadError::CorruptFile("Invalid min_tx_id in trailer".into())
                })?
                .try_into()
                .unwrap_or([0; 8]),
        );
        let max_tx_id = u64::from_le_bytes(
            trailer_buf
                .get(base + 8..base + 16)
                .ok_or_else(|| {
                    WitnessReadError::CorruptFile("Invalid max_tx_id in trailer".into())
                })?
                .try_into()
                .unwrap_or([0; 8]),
        );
        let min_seq = u64::from_le_bytes(
            trailer_buf
                .get(base + 16..base + 24)
                .ok_or_else(|| WitnessReadError::CorruptFile("Invalid min_seq in trailer".into()))?
                .try_into()
                .unwrap_or([0; 8]),
        );
        let max_seq = u64::from_le_bytes(
            trailer_buf
                .get(base + 24..base + 32)
                .ok_or_else(|| WitnessReadError::CorruptFile("Invalid max_seq in trailer".into()))?
                .try_into()
                .unwrap_or([0; 8]),
        );
        let bloom_offset = u64::from_le_bytes(
            trailer_buf
                .get(base + 32..base + 40)
                .ok_or_else(|| {
                    WitnessReadError::CorruptFile("Invalid bloom_offset in trailer".into())
                })?
                .try_into()
                .unwrap_or([0; 8]),
        );
        let index_offset = u64::from_le_bytes(
            trailer_buf
                .get(base + 40..base + 48)
                .ok_or_else(|| {
                    WitnessReadError::CorruptFile("Invalid index_offset in trailer".into())
                })?
                .try_into()
                .unwrap_or([0; 8]),
        );

        let trailer_offset = file_size.saturating_sub(if format_version >= 1 { 54 } else { 52 });

        if index_offset > trailer_offset
            || (bloom_offset > 0 && (bloom_offset < index_offset || bloom_offset > trailer_offset))
        {
            return Err(WitnessReadError::CorruptFile(
                "Invalid section offsets in trailer".into(),
            ));
        }

        // Read index section
        let index_end = if bloom_offset > 0 {
            bloom_offset
        } else {
            trailer_offset
        };
        let index_len = usize::try_from(index_end.saturating_sub(index_offset)).map_err(|_| {
            WitnessReadError::CorruptFile("Index length exceeds platform bounds".into())
        })?;

        let mut index_raw = vec![0u8; index_len];
        file.seek(SeekFrom::Start(index_offset))
            .map_err(|e| WitnessReadError::Io(e.to_string()))?;
        file.read_exact(&mut index_raw)
            .map_err(|e| WitnessReadError::Io(e.to_string()))?;

        let index_payload = if index_raw.len() >= 4 {
            let stored_crc = u32::from_le_bytes(index_raw[0..4].try_into().unwrap_or([0; 4]));
            let payload = &index_raw[4..];
            if crc32fast::hash(payload) != stored_crc {
                return Err(WitnessReadError::CorruptFile("Index CRC mismatch".into()));
            }
            payload
        } else {
            &index_raw[..]
        };

        let mut index = Vec::new();
        let mut pos = 0;
        let payload_len = index_payload.len();

        while pos + 10 <= payload_len {
            let key_len = usize::from(u16::from_le_bytes(
                index_payload[pos..pos + 2].try_into().unwrap_or([0; 2]),
            ));
            pos += 2;

            if pos + key_len + 8 > payload_len {
                return Err(WitnessReadError::CorruptFile("Index key truncated".into()));
            }

            let key_bytes = Bytes::copy_from_slice(&index_payload[pos..pos + key_len]);
            pos += key_len;

            let block_offset =
                u64::from_le_bytes(index_payload[pos..pos + 8].try_into().unwrap_or([0; 8]));
            pos += 8;

            if block_offset > index_offset {
                return Err(WitnessReadError::CorruptFile(
                    "Block offset in index exceeds index_offset".into(),
                ));
            }

            index.push((key_bytes, block_offset));
        }

        if pos != payload_len {
            return Err(WitnessReadError::CorruptFile(
                "Index trailing unparsed bytes".into(),
            ));
        }

        let last_key = index.last().map(|(k, _)| k.clone()).unwrap_or_default();

        let mut reader = Self {
            path: path_buf,
            file_size,
            file,
            metadata: WitnessSstMetadata {
                first_key: Bytes::new(),
                last_key,
                file_size,
                min_tx_id,
                max_tx_id,
                min_seq,
                max_seq,
                format_version,
            },
            index,
            index_offset,
            bloom_offset,
            has_crc: true,
            format_version,
            key_manager: derived_km,
            is_encrypted: false,
        };

        // Try reading first key to determine encryption or complete metadata
        if let Err(err) = reader.read_first_key() {
            if matches!(err, WitnessReadError::NotWitnessable(_)) {
                reader.is_encrypted = true;
            } else if reader.key_manager.is_none() {
                // If no key manager and first block failed CRC / parse, file is encrypted or corrupt.
                // Fail-closed to NotWitnessable if encrypted without key.
                return Err(WitnessReadError::NotWitnessable(
                    "Encrypted SSTable cannot be witnessed without decryption key".into(),
                ));
            } else {
                return Err(err);
            }
        }

        Ok(reader)
    }

    pub fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    pub fn bloom_offset(&self) -> u64 {
        self.bloom_offset
    }

    pub fn index_offset(&self) -> u64 {
        self.index_offset
    }

    pub fn metadata(&self) -> &WitnessSstMetadata {
        &self.metadata
    }

    pub fn index(&self) -> &[(Bytes, u64)] {
        &self.index
    }

    pub fn format_version(&self) -> u16 {
        self.format_version
    }

    /// Helper to read and decrypt a raw block payload at offset.
    fn read_block_raw(
        &mut self,
        offset: u64,
        next_offset: u64,
    ) -> Result<Vec<u8>, WitnessReadError> {
        let block_len = usize::try_from(next_offset.saturating_sub(offset))
            .map_err(|_| WitnessReadError::CorruptFile("Block length overflow".into()))?;

        if block_len > 16 * 1024 * 1024 {
            return Err(WitnessReadError::CorruptFile(
                "Block length exceeds 16MB limit".into(),
            ));
        }

        let mut buf = vec![0u8; block_len];
        self.file
            .seek(SeekFrom::Start(offset))
            .map_err(|e| WitnessReadError::Io(e.to_string()))?;
        self.file
            .read_exact(&mut buf)
            .map_err(|e| WitnessReadError::Io(e.to_string()))?;

        let block_data = if let Some(km) = &self.key_manager {
            if buf.len() < 12 {
                return Err(WitnessReadError::CorruptFile(
                    "Encrypted block missing nonce".into(),
                ));
            }
            let mut nonce = [0u8; 12];
            nonce.copy_from_slice(&buf[0..12]);
            match km.decrypt_auto_nonce(&buf[12..], &nonce) {
                Ok(dec) => dec,
                Err(e) => {
                    return Err(WitnessReadError::CorruptFile(format!(
                        "Decryption failed: {e}"
                    )))
                }
            }
        } else {
            buf
        };

        if self.has_crc {
            if block_data.len() < 4 {
                return Err(WitnessReadError::CorruptFile(
                    "Block too small for CRC".into(),
                ));
            }
            let stored_crc = u32::from_le_bytes(block_data[0..4].try_into().unwrap_or([0; 4]));
            let payload = &block_data[4..];
            if crc32fast::hash(payload) != stored_crc {
                if self.key_manager.is_none() {
                    return Err(WitnessReadError::NotWitnessable(
                        "SSTable block CRC mismatch (file is encrypted or missing key)".into(),
                    ));
                }
                return Err(WitnessReadError::CorruptFile("Block CRC mismatch".into()));
            }
            Ok(payload.to_vec())
        } else {
            Ok(block_data)
        }
    }

    /// Reads first key from first block (offset 0 / index[0].offset) to complete AP-08 metadata.
    pub fn read_first_key(&mut self) -> Result<Bytes, WitnessReadError> {
        if self.index.is_empty() {
            return Ok(Bytes::new());
        }
        let first_block_offset = self.index[0].1;
        let next_offset = if self.index.len() > 1 {
            self.index[1].1
        } else {
            self.index_offset
        };

        let block_bytes = self.read_block_raw(first_block_offset, next_offset)?;
        if block_bytes.len() < 2 {
            return Err(WitnessReadError::CorruptFile(
                "First block too short for key header".into(),
            ));
        }
        let k_len = usize::from(u16::from_le_bytes(
            block_bytes[0..2].try_into().unwrap_or([0; 2]),
        ));
        if block_bytes.len() < 2 + k_len {
            return Err(WitnessReadError::CorruptFile(
                "First key truncated in block".into(),
            ));
        }
        let first_key = Bytes::copy_from_slice(&block_bytes[2..2 + k_len]);
        self.metadata.first_key = first_key.clone();
        Ok(first_key)
    }

    /// Iterates over all KV entries across all blocks in sorted key order.
    pub fn read_all_entries(&mut self) -> Result<Vec<WitnessEntry>, WitnessReadError> {
        if self.is_encrypted && self.key_manager.is_none() {
            return Err(WitnessReadError::NotWitnessable(
                "Cannot read entries from encrypted SSTable without decryption key".into(),
            ));
        }

        let mut entries = Vec::new();
        if self.index.is_empty() {
            return Ok(entries);
        }

        let is_v3 = self.format_version >= 3;

        for idx in 0..self.index.len() {
            let offset = self.index[idx].1;
            let next_offset = if idx + 1 < self.index.len() {
                self.index[idx + 1].1
            } else {
                self.index_offset
            };

            let block_bytes = self.read_block_raw(offset, next_offset)?;
            let block_entries = parse_block_entries(&block_bytes, self.format_version, is_v3)?;
            entries.extend(block_entries);
        }

        Ok(entries)
    }
}

/// Helper to parse trailer and entries out of an uncompressed/decrypted block payload.
fn parse_block_entries(
    block_data: &[u8],
    format_version: u16,
    is_v3: bool,
) -> Result<Vec<WitnessEntry>, WitnessReadError> {
    let n = block_data.len();
    let (num_offsets, offsets_start) = if format_version >= 4 {
        if n < 8 {
            return Err(WitnessReadError::CorruptFile(
                "Block trailer too small (< 8 bytes)".into(),
            ));
        }
        let num_offsets = usize::try_from(u32::from_le_bytes(
            block_data[n - 4..n].try_into().unwrap_or([0; 4]),
        ))
        .unwrap_or(0);
        let bloom_len = usize::try_from(u32::from_le_bytes(
            block_data[n - 8..n - 4].try_into().unwrap_or([0; 4]),
        ))
        .unwrap_or(0);
        let offsets_len = num_offsets.checked_mul(4).ok_or_else(|| {
            WitnessReadError::CorruptFile("Overflow in offsets length calculation".into())
        })?;
        let total_trailer = 8 + bloom_len + offsets_len;
        if n < total_trailer {
            return Err(WitnessReadError::CorruptFile(
                "Block trailer bounds error".into(),
            ));
        }
        let offsets_start = n - 8 - offsets_len;
        (num_offsets, offsets_start)
    } else if format_version == 3 {
        if n < 36 {
            return Err(WitnessReadError::CorruptFile(
                "Block trailer too small (< 36 bytes)".into(),
            ));
        }
        let num_offsets = usize::try_from(u32::from_le_bytes(
            block_data[n - 4..n].try_into().unwrap_or([0; 4]),
        ))
        .unwrap_or(0);
        let offsets_len = num_offsets.checked_mul(4).ok_or_else(|| {
            WitnessReadError::CorruptFile("Overflow in offsets length calculation".into())
        })?;
        if n < 36 + offsets_len {
            return Err(WitnessReadError::CorruptFile(
                "Block trailer bounds error".into(),
            ));
        }
        let offsets_start = n - 4 - offsets_len;
        (num_offsets, offsets_start)
    } else {
        if n < 10 {
            return Err(WitnessReadError::CorruptFile(
                "Block trailer too small (< 10 bytes)".into(),
            ));
        }
        let num_offsets = usize::from(u16::from_le_bytes(
            block_data[n - 2..n].try_into().unwrap_or([0; 2]),
        ));
        let offsets_len = num_offsets.checked_mul(2).ok_or_else(|| {
            WitnessReadError::CorruptFile("Overflow in offsets length calculation".into())
        })?;
        if n < 10 + offsets_len {
            return Err(WitnessReadError::CorruptFile(
                "Block trailer bounds error".into(),
            ));
        }
        let offsets_start = n - 2 - offsets_len;
        (num_offsets, offsets_start)
    };

    let mut entries = Vec::with_capacity(num_offsets);

    for i in 0..num_offsets {
        let entry_off = if is_v3 || format_version >= 4 {
            let p = offsets_start + i * 4;
            if p + 4 > block_data.len() {
                return Err(WitnessReadError::CorruptFile(
                    "Offset array out of bounds".into(),
                ));
            }
            usize::try_from(u32::from_le_bytes(
                block_data[p..p + 4].try_into().unwrap_or([0; 4]),
            ))
            .unwrap_or(0)
        } else {
            let p = offsets_start + i * 2;
            if p + 2 > block_data.len() {
                return Err(WitnessReadError::CorruptFile(
                    "Offset array out of bounds".into(),
                ));
            }
            usize::from(u16::from_le_bytes(
                block_data[p..p + 2].try_into().unwrap_or([0; 2]),
            ))
        };

        if entry_off + 2 > block_data.len() {
            return Err(WitnessReadError::CorruptFile(
                "Entry offset out of bounds".into(),
            ));
        }

        let key_len = usize::from(u16::from_le_bytes(
            block_data[entry_off..entry_off + 2]
                .try_into()
                .unwrap_or([0; 2]),
        ));
        let pos_key = entry_off + 2;
        let pos_seq = pos_key + key_len;
        let pos_tx = pos_seq + 8;
        let pos_vlen = pos_tx + 8;
        let pos_val = pos_vlen + 4;

        if pos_val > block_data.len() {
            return Err(WitnessReadError::CorruptFile(
                "Entry metadata out of bounds".into(),
            ));
        }

        let key_bytes = Bytes::copy_from_slice(&block_data[pos_key..pos_key + key_len]);
        let seq_no = u64::from_le_bytes(
            block_data[pos_seq..pos_seq + 8]
                .try_into()
                .unwrap_or([0; 8]),
        );
        let tx_id = u64::from_le_bytes(block_data[pos_tx..pos_tx + 8].try_into().unwrap_or([0; 8]));
        let val_len = usize::try_from(u32::from_le_bytes(
            block_data[pos_vlen..pos_vlen + 4]
                .try_into()
                .unwrap_or([0; 4]),
        ))
        .unwrap_or(0);

        let end_val = pos_val + val_len;
        if end_val > offsets_start {
            return Err(WitnessReadError::CorruptFile(
                "Value extends into trailer section".into(),
            ));
        }

        let val_bytes = Bytes::copy_from_slice(&block_data[pos_val..end_val]);
        let is_tombstone = (seq_no & contextra_types::TOMBSTONE_BIT) != 0;

        entries.push(WitnessEntry {
            key: key_bytes,
            value: val_bytes,
            seq_no,
            tx_id,
            is_tombstone,
        });
    }

    Ok(entries)
}
