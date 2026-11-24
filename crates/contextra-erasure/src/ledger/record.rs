//! CTRALEDG v1 Framing, Serialisierung und Deserialisierung (SPEC §11.8, Anhang F.9, ER-30).
//! Herkunft: e67ea4e55:crates/contextra-store/src/erasure_ledger/entry.rs:16

use super::error::LedgerError;

/// CTRALEDG Magic (8 Bytes): "CTRALEDG"
pub const CTRALEDG_MAGIC: &[u8; 8] = b"CTRALEDG";

/// Aktuelle Version des Ledger-Formats
pub const CTRALEDG_VERSION: u32 = 1;

/// Dateikopf (Header) eines CTRALEDG v1 Ledgers (16 Bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerFileHeader {
    /// Magic-Bytes
    pub magic: [u8; 8],
    /// Versionsnummer
    pub version: u32,
    /// Reservierte Flags
    pub flags: u32,
}

impl LedgerFileHeader {
    /// Größe des Headers in Bytes.
    pub const SIZE: usize = 16;

    /// Erstellt einen Standard-Header.
    pub fn new() -> Self {
        Self {
            magic: *CTRALEDG_MAGIC,
            version: CTRALEDG_VERSION,
            flags: 0,
        }
    }

    /// Serialisiert den Header in 16 Bytes.
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        if let Some(dst) = buf.get_mut(0..8) {
            dst.copy_from_slice(&self.magic);
        }
        if let Some(dst) = buf.get_mut(8..12) {
            dst.copy_from_slice(&self.version.to_le_bytes());
        }
        if let Some(dst) = buf.get_mut(12..16) {
            dst.copy_from_slice(&self.flags.to_le_bytes());
        }
        buf
    }

    /// Parst und validiert den Header aus einem Byte-Slice.
    pub fn parse(buf: &[u8]) -> Result<Self, LedgerError> {
        if buf.len() < Self::SIZE {
            return Err(LedgerError::Corrupt(format!(
                "Header too short: expected {} bytes, got {}",
                Self::SIZE,
                buf.len()
            )));
        }
        let magic_slice = buf.get(0..8).ok_or_else(|| {
            LedgerError::Corrupt("Out of bounds while reading header magic".to_string())
        })?;
        let mut magic = [0u8; 8];
        magic.copy_from_slice(magic_slice);
        if &magic != CTRALEDG_MAGIC {
            return Err(LedgerError::Corrupt(format!(
                "Invalid ledger magic: {:?}",
                magic
            )));
        }

        let ver_slice = buf.get(8..12).ok_or_else(|| {
            LedgerError::Corrupt("Out of bounds while reading header version".to_string())
        })?;
        let mut ver_bytes = [0u8; 4];
        ver_bytes.copy_from_slice(ver_slice);
        let version = u32::from_le_bytes(ver_bytes);
        if version != CTRALEDG_VERSION {
            return Err(LedgerError::UnsupportedVersion(version));
        }

        let flags_slice = buf.get(12..16).ok_or_else(|| {
            LedgerError::Corrupt("Out of bounds while reading header flags".to_string())
        })?;
        let mut flags_bytes = [0u8; 4];
        flags_bytes.copy_from_slice(flags_slice);
        let flags = u32::from_le_bytes(flags_bytes);

        Ok(Self {
            magic,
            version,
            flags,
        })
    }
}

impl Default for LedgerFileHeader {
    fn default() -> Self {
        Self::new()
    }
}

/// Typ eines Ledger-Records im CTRALEDG-Format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RecordType {
    /// Intent vor Beginn
    Intent = 0x01,
    /// Completed / Purged nach Ausführung
    Completed = 0x02,
    /// KeyDestroyed
    KeyDestroyed = 0x03,
}

impl RecordType {
    /// Parst den Record-Typ aus einem u8.
    pub fn from_u8(v: u8) -> Result<Self, LedgerError> {
        match v {
            0x01 => Ok(RecordType::Intent),
            0x02 => Ok(RecordType::Completed),
            0x03 => Ok(RecordType::KeyDestroyed),
            other => Err(LedgerError::InvalidRecord(format!(
                "Unknown record type byte: {:#04x}",
                other
            ))),
        }
    }
}

/// Strukturierter Record im CTRALEDG-Format (Framing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FramedRecord {
    /// Typ des Records
    pub record_type: RecordType,
    /// Sequenznummer (gated_seq)
    pub seq: u64,
    /// Zeitstempel (completed_at bzw. creation in Mikrosekunden)
    pub timestamp: u64,
    /// Kryptografisches Commitment
    pub commitment: [u8; 32],
    /// Bereichs-Tags (opak)
    pub range_tags: Vec<u8>,
    /// Vorheriger Ketten-Hash
    pub prev_hash: [u8; 32],
    /// Aktueller Ketten-Hash: BLAKE3(prev_hash ‖ commitment ‖ seq_le)
    pub chain_hash: [u8; 32],
    /// Integritäts-MAC: BLAKE3-keyed(K_ledger, chain_hash)[0..16]
    pub ledger_mac: [u8; 16],
}

impl FramedRecord {
    /// Fixer Anteil der Frame-Länge (ohne range_tags und CRC):
    /// record_type(1) + seq(8) + timestamp(8) + commitment(32) + range_tags_len(4)
    /// + prev_hash(32) + chain_hash(32) + ledger_mac(16) = 133 Bytes.
    ///
    /// Zzgl. CRC32C (4 Bytes) = 137 Bytes minimal.
    pub const FIXED_HEADER_LEN: usize = 1 + 8 + 8 + 32 + 4 + 32 + 32 + 16;
    /// Minimale Framelänge inklusive CRC32C.
    pub const MIN_FRAME_LEN: usize = Self::FIXED_HEADER_LEN + 4;

    /// Serialisiert den Record inklusive CRC32C Prüfsumme.
    pub fn serialize(&self) -> Vec<u8> {
        let range_len = u32::try_from(self.range_tags.len()).unwrap_or(u32::MAX);
        let mut buf = Vec::with_capacity(Self::MIN_FRAME_LEN.saturating_add(self.range_tags.len()));

        buf.push(self.record_type as u8);
        buf.extend_from_slice(&self.seq.to_le_bytes());
        buf.extend_from_slice(&self.timestamp.to_le_bytes());
        buf.extend_from_slice(&self.commitment);
        buf.extend_from_slice(&range_len.to_le_bytes());
        buf.extend_from_slice(&self.range_tags);
        buf.extend_from_slice(&self.prev_hash);
        buf.extend_from_slice(&self.chain_hash);
        buf.extend_from_slice(&self.ledger_mac);

        // CRC32 über alle bisherigen Bytes des Records
        let crc = crc32_checksum(&buf);
        buf.extend_from_slice(&crc.to_le_bytes());
        buf
    }

    /// Parst einen Record aus einem Slice. Gibt den geparsten Record und die gelesene Bytezahl zurück.
    pub fn parse(buf: &[u8]) -> Result<(Self, usize), LedgerError> {
        if buf.len() < Self::MIN_FRAME_LEN {
            return Err(LedgerError::Corrupt(format!(
                "Record slice too short: {} bytes",
                buf.len()
            )));
        }

        let first_byte = *buf
            .first()
            .ok_or_else(|| LedgerError::Corrupt("Empty buffer reading record type".to_string()))?;
        let record_type = RecordType::from_u8(first_byte)?;

        let seq_slice = buf
            .get(1..9)
            .ok_or_else(|| LedgerError::Corrupt("Out of bounds reading seq".to_string()))?;
        let mut seq_bytes = [0u8; 8];
        seq_bytes.copy_from_slice(seq_slice);
        let seq = u64::from_le_bytes(seq_bytes);

        let ts_slice = buf
            .get(9..17)
            .ok_or_else(|| LedgerError::Corrupt("Out of bounds reading timestamp".to_string()))?;
        let mut ts_bytes = [0u8; 8];
        ts_bytes.copy_from_slice(ts_slice);
        let timestamp = u64::from_le_bytes(ts_bytes);

        let com_slice = buf
            .get(17..49)
            .ok_or_else(|| LedgerError::Corrupt("Out of bounds reading commitment".to_string()))?;
        let mut commitment = [0u8; 32];
        commitment.copy_from_slice(com_slice);

        let rlen_slice = buf
            .get(49..53)
            .ok_or_else(|| LedgerError::Corrupt("Out of bounds reading range len".to_string()))?;
        let mut rlen_bytes = [0u8; 4];
        rlen_bytes.copy_from_slice(rlen_slice);
        let rlen = usize::try_from(u32::from_le_bytes(rlen_bytes)).unwrap_or(usize::MAX);

        let needed = match Self::MIN_FRAME_LEN.checked_add(rlen) {
            Some(n) => n,
            None => {
                return Err(LedgerError::Corrupt(
                    "Overflow calculating record frame size".to_string(),
                ))
            }
        };

        if buf.len() < needed {
            return Err(LedgerError::Corrupt(format!(
                "Truncated record: needs {} bytes, got {}",
                needed,
                buf.len()
            )));
        }

        let range_slice = buf.get(53..53 + rlen).ok_or_else(|| {
            LedgerError::Corrupt("Out of bounds reading range tags".to_string())
        })?;
        let range_tags = range_slice.to_vec();
        let mut offset = 53 + rlen;

        let prev_slice = buf.get(offset..offset + 32).ok_or_else(|| {
            LedgerError::Corrupt("Out of bounds reading prev_hash".to_string())
        })?;
        let mut prev_hash = [0u8; 32];
        prev_hash.copy_from_slice(prev_slice);
        offset += 32;

        let chain_slice = buf.get(offset..offset + 32).ok_or_else(|| {
            LedgerError::Corrupt("Out of bounds reading chain_hash".to_string())
        })?;
        let mut chain_hash = [0u8; 32];
        chain_hash.copy_from_slice(chain_slice);
        offset += 32;

        let mac_slice = buf.get(offset..offset + 16).ok_or_else(|| {
            LedgerError::Corrupt("Out of bounds reading ledger_mac".to_string())
        })?;
        let mut ledger_mac = [0u8; 16];
        ledger_mac.copy_from_slice(mac_slice);
        offset += 16;

        let payload_for_crc = buf
            .get(0..offset)
            .ok_or_else(|| LedgerError::Corrupt("Out of bounds reading crc payload".to_string()))?;

        let crc_slice = buf
            .get(offset..offset + 4)
            .ok_or_else(|| LedgerError::Corrupt("Out of bounds reading crc bytes".to_string()))?;
        let mut crc_bytes = [0u8; 4];
        crc_bytes.copy_from_slice(crc_slice);
        let expected_crc = u32::from_le_bytes(crc_bytes);
        offset += 4;

        let actual_crc = crc32_checksum(payload_for_crc);
        if actual_crc != expected_crc {
            return Err(LedgerError::Corrupt(format!(
                "CRC mismatch in record at seq {}: expected {:#010x}, calculated {:#010x}",
                seq, expected_crc, actual_crc
            )));
        }

        Ok((
            Self {
                record_type,
                seq,
                timestamp,
                commitment,
                range_tags,
                prev_hash,
                chain_hash,
                ledger_mac,
            },
            offset,
        ))
    }
}

/// Einfache standardkonforme CRC32-Berechnung (ISO 3309 / ITU-T V.42 / IEEE 802.3).
fn crc32_checksum(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = if (crc & 1) != 0 {
                0xEDB8_8320u32
            } else {
                0u32
            };
            crc = (crc >> 1) ^ mask;
        }
    }
    !crc
}
