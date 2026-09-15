use memfuse_core::{MemFuseError, Result, TxId};
use memfuse_crypto::wal_crypto::WalHmac;

use super::MAX_WAL_ENTRY_SIZE;

#[derive(Debug, Clone, PartialEq)]
pub enum WalOp {
    /// Inserts or updates a key-value pair.
    Put {
        tx_id: TxId,
        key: Vec<u8>,
        value: Vec<u8>,
    },
    /// Deletes a key.
    Delete { tx_id: TxId, key: Vec<u8> },
}

impl WalOp {
    pub fn tx_id(&self) -> TxId {
        match self {
            WalOp::Put { tx_id, .. } => *tx_id,
            WalOp::Delete { tx_id, .. } => *tx_id,
        }
    }
}

/// Magic header for V2 batch-encrypted WAL files (`b"MFW2"`).
pub const WAL_V2_HEADER: [u8; 4] = *b"MFW2";

/// Magic header for V3 WAL files (`b"MFW3"`).
pub const WAL_V3_HEADER: [u8; 4] = *b"MFW3";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WalVersion {
    V1, // Legacy: kein HMAC
    V2, // Current: HMAC ohne tx_id
    V3, // New: HMAC mit tx_id
}

/// A batch of prepared WAL entries bound to a specific HMAC chain.
///
/// `PreparedBatch` has a private inner field and cannot be constructed outside `wal.rs`.
/// It can only be created by calling [`Wal::prepare_batch`].
#[derive(Debug, Clone)]
pub struct PreparedBatch(pub(crate) Vec<WalEntry>);

impl PreparedBatch {
    /// Returns `true` if the batch contains no entries.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of entries in the batch.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns a slice of the inner WAL entries.
    pub fn entries(&self) -> &[WalEntry] {
        &self.0
    }

    /// Consumes the `PreparedBatch` and returns the inner vector of entries.
    pub fn into_inner(self) -> Vec<WalEntry> {
        self.0
    }

    /// Extends this prepared batch with entries from another prepared batch.
    pub(crate) fn extend(&mut self, other: PreparedBatch) {
        self.0.extend(other.0);
    }
}

/// A single entry in the Write-Ahead Log.
#[derive(Debug, Clone, PartialEq)]
pub struct WalEntry {
    /// The operation performed.
    pub op: WalOp,
    /// Sequence number assigned to the operation.
    pub seq_no: u64,
    /// HMAC of the current entry (includes previous HMAC).
    pub checksum: [u8; 32],
    /// HMAC of the previous entry (the chain link).
    pub prev_hmac: [u8; 32],
}

impl WalEntry {
    pub fn tx_id(&self) -> TxId {
        self.op.tx_id()
    }
}

impl WalEntry {
    /// Creates a new WAL entry with HMAC-SHA256 checksum and chaining.
    pub fn try_new(
        op: WalOp,
        seq_no: u64,
        integrity_key: &[u8],
        prev_hmac: [u8; 32],
    ) -> Result<Self> {
        let checksum = Self::compute_checksum_v3(&op, seq_no, integrity_key, prev_hmac)?;
        Ok(Self {
            op,
            seq_no,
            checksum,
            prev_hmac,
        })
    }

    /// Computes V3 checksum (includes tx_id and length prefixes for key/value).
    pub fn compute_checksum_v3(
        op: &WalOp,
        seq_no: u64,
        integrity_key: &[u8],
        prev_hmac: [u8; 32],
    ) -> Result<[u8; 32]> {
        let mut mac = WalHmac::new(integrity_key)?;

        // Hash Chaining: binding to the previous entry
        mac.update(&prev_hmac);
        mac.update(&seq_no.to_le_bytes());

        // tx_id MUST come before op_type
        let tx_id_bytes = op.tx_id().inner().to_le_bytes();
        mac.update(&tx_id_bytes);

        match op {
            WalOp::Put { key, value, .. } => {
                mac.update(&[0u8]); // op type
                mac.update(&(key.len() as u32).to_le_bytes());
                mac.update(key);
                mac.update(&(value.len() as u32).to_le_bytes());
                mac.update(value);
            }
            WalOp::Delete { key, .. } => {
                mac.update(&[1u8]); // op type
                mac.update(&(key.len() as u32).to_le_bytes());
                mac.update(key);
            }
        }
        Ok(mac.finalize())
    }

    /// Legacy V2 checksum calculation (without tx_id and length-prefixes in HMAC).
    pub fn compute_checksum_v2(
        op: &WalOp,
        seq_no: u64,
        integrity_key: &[u8],
        prev_hmac: [u8; 32],
    ) -> Result<[u8; 32]> {
        let mut mac = WalHmac::new(integrity_key)?;

        mac.update(&prev_hmac);
        mac.update(&seq_no.to_le_bytes());
        match op {
            WalOp::Put { key, value, .. } => {
                mac.update(&[0u8]);
                mac.update(key);
                mac.update(value);
            }
            WalOp::Delete { key, .. } => {
                mac.update(&[1u8]);
                mac.update(key);
            }
        }
        Ok(mac.finalize())
    }

    pub fn compute_checksum(
        op: &WalOp,
        seq_no: u64,
        integrity_key: &[u8],
        prev_hmac: [u8; 32],
    ) -> Result<[u8; 32]> {
        Self::compute_checksum_v3(op, seq_no, integrity_key, prev_hmac)
    }

    /// Serializes the entry to bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let op_size = match &self.op {
            WalOp::Put { key, value, .. } => 1 + 8 + 4 + key.len() + 4 + value.len(),
            WalOp::Delete { key, .. } => 1 + 8 + 4 + key.len(),
        };

        // payload = seq_no(8) + checksum(32) + prev_hmac(32) + op
        let payload_size = 8 + 32 + 32 + op_size;
        // total_payload = CRC32(4) + payload
        let total_payload_size = 4 + payload_size;
        // total_size = length_prefix(4) + total_payload
        let total_size = 4 + total_payload_size;

        let mut buf = Vec::with_capacity(total_size);

        // 1. Length Prefix
        if total_payload_size > MAX_WAL_ENTRY_SIZE as usize {
            return Err(MemFuseError::Serialization(format!(
                "WAL entry too large: {} bytes (max {})",
                total_payload_size, MAX_WAL_ENTRY_SIZE
            )));
        }
        buf.extend_from_slice(&(total_payload_size as u32).to_le_bytes());

        // 2. CRC32 Placeholder (we'll fill this at the end)
        let crc_offset = buf.len();
        buf.extend_from_slice(&[0u8; 4]);

        // 3. Payload
        let payload_start = buf.len();
        buf.extend_from_slice(&self.seq_no.to_le_bytes());
        buf.extend_from_slice(&self.checksum);
        buf.extend_from_slice(&self.prev_hmac);

        match &self.op {
            WalOp::Put { tx_id, key, value } => {
                buf.push(0u8);
                buf.extend_from_slice(&tx_id.inner().to_le_bytes());
                buf.extend_from_slice(&(key.len() as u32).to_le_bytes());
                buf.extend_from_slice(key);
                buf.extend_from_slice(&(value.len() as u32).to_le_bytes());
                buf.extend_from_slice(value);
            }
            WalOp::Delete { tx_id, key } => {
                buf.push(1u8);
                buf.extend_from_slice(&tx_id.inner().to_le_bytes());
                buf.extend_from_slice(&(key.len() as u32).to_le_bytes());
                buf.extend_from_slice(key);
            }
        }

        // 4. Compute CRC32 over payload and fill placeholder
        let crc = crc32fast::hash(&buf[payload_start..]);
        buf[crc_offset..crc_offset + 4].copy_from_slice(&crc.to_le_bytes());

        Ok(buf)
    }

    /// Deserializes a WAL entry from bytes, verifying CRC32.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 4 {
            return Err(MemFuseError::Serialization(
                "WAL entry too short for CRC header".into(),
            ));
        }

        let stored_crc = u32::from_le_bytes(
            data[0..4]
                .try_into()
                .map_err(|_| MemFuseError::Serialization("Invalid CRC format".into()))?,
        );
        let payload = &data[4..];
        let computed_crc = crc32fast::hash(payload);

        if stored_crc != computed_crc {
            // FIND-STO-001: Explicitly return a message that includes "CRC mismatch"
            // so replay can map it to WalCorruption.
            return Err(MemFuseError::Serialization(format!(
                "CRC mismatch: stored={:#010x}, computed={:#010x}",
                stored_crc, computed_crc
            )));
        }

        if payload.len() < 73 {
            // 8(seq) + 32(checksum) + 32(prev_hmac) + 1(op_type)
            return Err(MemFuseError::Serialization("WAL payload too short".into()));
        }

        let seq_no = u64::from_le_bytes(
            payload[0..8]
                .try_into()
                .map_err(|_| MemFuseError::Serialization("Invalid seq_no format".into()))?,
        );
        let checksum: [u8; 32] = payload[8..40]
            .try_into()
            .map_err(|_| MemFuseError::Serialization("Invalid checksum format".into()))?;
        let prev_hmac: [u8; 32] = payload[40..72]
            .try_into()
            .map_err(|_| MemFuseError::Serialization("Invalid prev_hmac format".into()))?;
        let op_type = payload[72];
        let remaining = &payload[73..];

        let op =
            match op_type {
                0 => {
                    // Put
                    if remaining.len() < 12 {
                        return Err(MemFuseError::Serialization("Put op too short".into()));
                    }
                    let tx_id = TxId::new(u64::from_le_bytes(remaining[0..8].try_into().map_err(
                        |_| MemFuseError::Serialization("Invalid tx_id format".into()),
                    )?));
                    let key_len = u32::from_le_bytes(remaining[8..12].try_into().map_err(|_| {
                        MemFuseError::Serialization("Invalid key_len format".into())
                    })?) as usize;
                    if key_len > 1024 * 1024 {
                        return Err(MemFuseError::Serialization(
                            "key_len exceeds 1 MiB limit".into(),
                        ));
                    }
                    if remaining.len() < 12 + key_len + 4 {
                        return Err(MemFuseError::Serialization(
                            "Put op missing key/val_len".into(),
                        ));
                    }
                    let key = remaining[12..12 + key_len].to_vec();
                    let val_start = 12 + key_len;
                    let val_len =
                        u32::from_le_bytes(remaining[val_start..val_start + 4].try_into().map_err(
                            |_| MemFuseError::Serialization("Invalid val_len format".into()),
                        )?) as usize;
                    if val_len > 128 * 1024 * 1024 {
                        return Err(MemFuseError::Serialization(
                            "val_len exceeds 128 MiB limit".into(),
                        ));
                    }
                    if remaining.len() < val_start + 4 + val_len {
                        return Err(MemFuseError::Serialization(
                            "Put op missing value data".into(),
                        ));
                    }
                    let value = remaining[val_start + 4..val_start + 4 + val_len].to_vec();
                    WalOp::Put { tx_id, key, value }
                }
                1 => {
                    // Delete
                    if remaining.len() < 12 {
                        return Err(MemFuseError::Serialization("Delete op too short".into()));
                    }
                    let tx_id = TxId::new(u64::from_le_bytes(remaining[0..8].try_into().map_err(
                        |_| MemFuseError::Serialization("Invalid tx_id format".into()),
                    )?));
                    let key_len = u32::from_le_bytes(remaining[8..12].try_into().map_err(|_| {
                        MemFuseError::Serialization("Invalid key_len format".into())
                    })?) as usize;
                    if key_len > 1024 * 1024 {
                        return Err(MemFuseError::Serialization(
                            "key_len exceeds 1 MiB limit".into(),
                        ));
                    }
                    if remaining.len() < 12 + key_len {
                        return Err(MemFuseError::Serialization(
                            "Delete op missing key data".into(),
                        ));
                    }
                    let key = remaining[12..12 + key_len].to_vec();
                    WalOp::Delete { tx_id, key }
                }
                _ => {
                    return Err(MemFuseError::Serialization(format!(
                        "Unknown WAL op type: {}",
                        op_type
                    )))
                }
            };

        Ok(Self {
            op,
            seq_no,
            checksum,
            prev_hmac,
        })
    }
}
