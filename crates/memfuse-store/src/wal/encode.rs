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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::wal::Wal;
    use memfuse_crypto::crypto::KeyManager;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::fs;

#[test]
fn test_wal_entry_serialization_roundtrip() {
    let op = WalOp::Put {
        tx_id: TxId::new(42),
        key: b"key".to_vec(),
        value: b"value".to_vec(),
    };
    let dummy_key = b"test-integrity-key-32-bytes-long!";
    let entry = WalEntry::try_new(op, 100, dummy_key, [0u8; 32]).expect("try_new"); // expect
    let bytes = entry.to_bytes().expect("serialization failed"); // expect

    // 4 (len) + 4 (crc) + 8 (seq) + 32 (hmac) + 32 (prev) + 1 (op) + 8 (tx) + 4 (klen) + 3 (k) + 4 (vlen) + 5 (v) = 105
    assert_eq!(bytes.len(), 105);
    let total_payload_size = u32::from_le_bytes(bytes[0..4].try_into().expect("valid slice")); // expect
    assert_eq!(total_payload_size, 101); // 4 (crc) + 97 (payload)
}


#[test]
fn test_wal_entry_crc_corruption_detected() {
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"key".to_vec(),
        value: b"value".to_vec(),
    };
    let dummy_key = b"test-integrity-key-32-bytes-long!";
    let entry = WalEntry::try_new(op, 1, dummy_key, [0u8; 32]).expect("try_new"); // expect

    let mut bytes = entry.to_bytes().expect("serialization failed"); // expect

    // Let's corrupt the payload which is after the length prefix(4) and CRC(4)
    if bytes.len() > 10 {
        bytes[10] ^= 0xFF;
    }

    // Check using from_bytes (skipping the length prefix at the start)
    let result = WalEntry::from_bytes(&bytes[4..]);
    assert!(result.is_err(), "Corruption must be detected by CRC check");
    let err = result.unwrap_err();
    assert!(format!("{}", err).contains("CRC mismatch"));
}


#[test]
fn test_wal_entry_header_fuzzing() {
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k".to_vec(),
        value: b"v".to_vec(),
    };
    let integrity_key = b"test-integrity-key-32-bytes-long!";
    let entry = WalEntry::try_new(op, 12345, integrity_key, [0u8; 32]).expect("try_new"); // expect

    let original_bytes = entry.to_bytes().expect("serialization failed"); // expect

    // Systematisch jedes Bit der ersten 12 Bytes flippen
    for byte_idx in 0..12 {
        for bit_idx in 0..8 {
            let mut corrupted_bytes = original_bytes.clone();
            corrupted_bytes[byte_idx] ^= 1 << bit_idx;

            // Testverhalten unterscheidet sich je nach Position
            if byte_idx < 4 {
                // Length prefix corrupted.
                // Das wird normalerweise von Wal::replay abgefangen,
                // aber from_bytes kriegt hier nur den Teil ab Index 4.
                // Wenn wir bytes[0..4] flippen, ändert das für from_bytes(&bytes[4..]) nichts.
                let result = WalEntry::from_bytes(&corrupted_bytes[4..]);
                assert!(
                    result.is_ok(),
                    "Flipping bytes[0..4] should not affect from_bytes(bytes[4..])"
                );
            } else {
                // CRC (4-7) oder SeqNo (8-11) korrumpiert.
                // Das MUSS von from_bytes erkannt werden.
                let result = WalEntry::from_bytes(&corrupted_bytes[4..]);
                assert!(
                    result.is_err(),
                    "Corruption at byte {}, bit {} was NOT detected! result: {:?}",
                    byte_idx,
                    bit_idx,
                    result
                );
            }
        }
    }
}


#[test]
fn test_wal_entry_crc_roundtrip() {
    let op = WalOp::Put {
        tx_id: TxId::new(42),
        key: b"test_key".to_vec(),
        value: b"test_value".to_vec(),
    };
    let dummy_key = b"test-integrity-key-32-bytes-long!";
    let entry = WalEntry::try_new(op, 100, dummy_key, [0u8; 32]).expect("try_new"); // expect

    let bytes = entry.to_bytes().expect("serialization failed"); // expect
    let decoded = WalEntry::from_bytes(&bytes[4..]).expect("Roundtrip must work"); // expect

    assert_eq!(decoded.seq_no, 100);
    if let WalOp::Put { key, value, .. } = decoded.op {
        assert_eq!(key, b"test_key");
        assert_eq!(value, b"test_value");
    } else {
        panic!("Wrong op type");
    }
}


#[tokio::test]
async fn test_batch_encryption_single_nonce_layout() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("single_nonce_test.wal");

    let km = Arc::new(
        KeyManager::try_new("test_passphrase", b"salt123456789012345678901234567890").expect("km"), // expect
    );
    let wal = Wal::open_with_key_manager(&wal_path, Some(km))
        .await
        .expect("open wal"); // expect

    let ops = vec![
        (
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"k1".to_vec(),
                value: b"v1".to_vec(),
            },
            100,
        ),
        (
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"k2".to_vec(),
                value: b"v2".to_vec(),
            },
            101,
        ),
        (
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"k3".to_vec(),
                value: b"v3".to_vec(),
            },
            102,
        ),
    ];

    let (batch, _) = wal.prepare_batch(ops).await.expect("prepare batch"); // expect
    assert_eq!(batch.len(), 3);

    wal.append_batch(batch).await.expect("append batch"); // expect

    let file_bytes = fs::read(&wal_path).await.expect("read wal file"); // expect

    // Layout:
    // Offset 0..4: WAL_V3_HEADER (b"MFW3")
    // Offset 4..8: batch chunk_len (u32 LE)
    // Offset 8..20: single 12-byte nonce
    // Offset 20..: AES-GCM-SIV ciphertext
    assert_eq!(&file_bytes[0..4], &WAL_V3_HEADER);
    let chunk_len = u32::from_le_bytes(file_bytes[4..8].try_into().unwrap()) as usize; // unwrap
    assert_eq!(file_bytes.len(), 4 + 4 + chunk_len);

    // Verify there is exactly one batch chunk header (12-byte nonce) in the file for N=3 entries
    let nonce_bytes = &file_bytes[8..20];
    assert_eq!(nonce_bytes.len(), 12);
}


#[test]
fn test_wal_op_from_bytes_oversized_key_val() {
    // Construct payload with key_len > 1MB
    let mut payload = vec![0u8; 90];
    // op_type = 0 (Put) at index 72
    payload[72] = 0;
    // tx_id = 1
    payload[73..81].copy_from_slice(&1u64.to_le_bytes());
    // key_len = 2 MB
    payload[81..85].copy_from_slice(&(2 * 1024 * 1024u32).to_le_bytes());

    let crc = crc32fast::hash(&payload);
    let mut data = vec![0u8; 4];
    data[0..4].copy_from_slice(&crc.to_le_bytes());
    data.extend_from_slice(&payload);

    let res = WalEntry::from_bytes(&data);
    assert!(res.is_err());
    if let Err(MemFuseError::Serialization(msg)) = res {
        assert!(msg.contains("key_len exceeds 1 MiB limit"));
    } else {
        panic!("Expected Serialization error for key_len limit");
    }
}


#[test]
fn test_wal_entry_from_bytes_invalid_cases() {
    // 1. Too short data (< 94 bytes)
    let short_data = vec![0u8; 50];
    let res = WalEntry::from_bytes(&short_data);
    assert!(matches!(res, Err(MemFuseError::Serialization(_))));

    // 2. Invalid WalOp tag (e.g., tag = 255)
    let mut payload = vec![0u8; 90];
    payload[72] = 255; // Invalid tag
    let crc = crc32fast::hash(&payload);
    let mut data = vec![0u8; 4];
    data[0..4].copy_from_slice(&crc.to_le_bytes());
    data.extend_from_slice(&payload);

    let res_op = WalEntry::from_bytes(&data);
    assert!(matches!(res_op, Err(MemFuseError::Serialization(_))));
}

}
