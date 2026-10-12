// FILE-CONTEXT
// STAND: 2026-10-11 (T-2026-9103)
// ZWECK: Datenstrukturen und Framing-/Krypto-Logik für den Erasure-Ledger.
// INVARIANTEN: Zero-Panic, Fail-Closed, BLAKE3-Keyed MAC Chaining, CRC32 Checksumming.

use contextra_core::{ContextraError, Result};

pub const ERASED_LEDGER_DOMAIN: &[u8] = b"contextra/erasure-ledger/v1";
pub const MAX_LEDGER_PAYLOAD_SIZE: usize = 1024;

/// Cryptographic 32-byte ledger key derived from master secret.
#[derive(Clone, PartialEq, Eq)]
pub struct LedgerKey([u8; 32]);

impl std::fmt::Debug for LedgerKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LedgerKey")
            .field("bytes", &"[REDACTED]")
            .finish()
    }
}

impl LedgerKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn derive_from_master_secret(master_secret: &[u8; 32]) -> Self {
        let derived = blake3::derive_key("contextra/erasure-ledger/v1", master_secret);
        Self(derived)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Drop for LedgerKey {
    fn drop(&mut self) {
        self.0.fill(0);
        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
    }
}

/// Computes the keyed BLAKE3 pseudonym hash for a key or prefix under a storage scope.
pub fn compute_keyed_hash(key: &LedgerKey, storage_scope: &[u8], bytes: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_keyed(key.as_bytes());
    hasher.update(ERASED_LEDGER_DOMAIN);
    hasher.update(storage_scope);
    hasher.update(bytes);
    *hasher.finalize().as_bytes()
}

/// Kind of entry recorded in the erasure ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerEntryKind {
    Key,
    Prefix { prefix_len: u32 },
}

/// An entry in the erasure ledger log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntry {
    pub ledger_seq: u64,
    pub kind: LedgerEntryKind,
    pub keyed_hash: [u8; 32],
    pub erased_at_tx: u64,
    pub proof_id: [u8; 16],
}

impl LedgerEntry {
    /// Serializes the entry payload into binary bytes.
    pub fn to_payload_bytes(&self) -> Result<Vec<u8>> {
        let mut payload = Vec::with_capacity(70);
        payload.extend_from_slice(&self.ledger_seq.to_be_bytes());
        match self.kind {
            LedgerEntryKind::Key => {
                payload.push(1u8);
                payload.extend_from_slice(&self.keyed_hash);
            }
            LedgerEntryKind::Prefix { prefix_len } => {
                payload.push(2u8);
                payload.extend_from_slice(&self.keyed_hash);
                payload.extend_from_slice(&prefix_len.to_be_bytes());
            }
        }
        payload.extend_from_slice(&self.erased_at_tx.to_be_bytes());
        payload.extend_from_slice(&self.proof_id);
        Ok(payload)
    }

    /// Deserializes the entry payload from binary bytes.
    pub fn from_payload_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 8 + 1 + 32 + 8 + 16 {
            return Err(ContextraError::Serialization(
                "ErasureLedger payload too short".into(),
            ));
        }

        let ledger_seq = u64::from_be_bytes(
            bytes[0..8]
                .try_into()
                .map_err(|_| ContextraError::Serialization("Invalid ledger_seq".into()))?,
        );
        let kind_byte = bytes[8];

        let (kind, keyed_hash, offset) = match kind_byte {
            1 => {
                if bytes.len() < 8 + 1 + 32 + 8 + 16 {
                    return Err(ContextraError::Serialization(
                        "ErasureLedger key payload too short".into(),
                    ));
                }
                let mut kh = [0u8; 32];
                kh.copy_from_slice(&bytes[9..41]);
                (LedgerEntryKind::Key, kh, 41)
            }
            2 => {
                if bytes.len() < 8 + 1 + 32 + 4 + 8 + 16 {
                    return Err(ContextraError::Serialization(
                        "ErasureLedger prefix payload too short".into(),
                    ));
                }
                let mut kh = [0u8; 32];
                kh.copy_from_slice(&bytes[9..41]);
                let prefix_len = u32::from_be_bytes(
                    bytes[41..45]
                        .try_into()
                        .map_err(|_| ContextraError::Serialization("Invalid prefix_len".into()))?,
                );
                (LedgerEntryKind::Prefix { prefix_len }, kh, 45)
            }
            _ => {
                return Err(ContextraError::Serialization(format!(
                    "Unknown ErasureLedger kind: {}",
                    kind_byte
                )))
            }
        };

        let erased_at_tx_offset = offset;
        if bytes.len() < erased_at_tx_offset + 8 + 16 {
            return Err(ContextraError::Serialization(
                "ErasureLedger erased_at_tx or proof_id missing".into(),
            ));
        }

        let erased_at_tx = u64::from_be_bytes(
            bytes[erased_at_tx_offset..erased_at_tx_offset + 8]
                .try_into()
                .map_err(|_| ContextraError::Serialization("Invalid erased_at_tx".into()))?,
        );

        let proof_id_offset = erased_at_tx_offset + 8;
        let mut proof_id = [0u8; 16];
        proof_id.copy_from_slice(&bytes[proof_id_offset..proof_id_offset + 16]);

        Ok(Self {
            ledger_seq,
            kind,
            keyed_hash,
            erased_at_tx,
            proof_id,
        })
    }
}

/// Encapsulates a complete framed record on disk:
/// `len(u32 BE) | payload | crc32fast(u32 BE) | chain_mac(32)`
pub fn serialize_frame(
    key: &LedgerKey,
    prev_mac: &[u8; 32],
    payload: &[u8],
) -> Result<(Vec<u8>, [u8; 32])> {
    #[allow(clippy::cast_possible_truncation)]
    let len = payload.len() as u32;
    let crc = crc32fast::hash(payload);

    let mut hasher = blake3::Hasher::new_keyed(key.as_bytes());
    hasher.update(prev_mac);
    hasher.update(payload);
    let chain_mac = *hasher.finalize().as_bytes();

    let mut frame = Vec::with_capacity(4 + payload.len() + 4 + 32);
    frame.extend_from_slice(&len.to_be_bytes());
    frame.extend_from_slice(payload);
    frame.extend_from_slice(&crc.to_be_bytes());
    frame.extend_from_slice(&chain_mac);

    Ok((frame, chain_mac))
}

/// Result of parsing a frame from disk at a given offset.
pub enum FrameReadResult {
    Ok {
        entry: LedgerEntry,
        chain_mac: [u8; 32],
        frame_len: usize,
    },
    TruncatedTail {
        bytes_read: usize,
    },
    Corrupt {
        reason: String,
    },
}

/// Reads and validates a single frame from `bytes` starting at offset `0`.
pub fn parse_frame(key: &LedgerKey, prev_mac: &[u8; 32], bytes: &[u8]) -> FrameReadResult {
    if bytes.len() < 4 {
        return FrameReadResult::TruncatedTail {
            bytes_read: bytes.len(),
        };
    }

    let len = u32::from_be_bytes(bytes[0..4].try_into().unwrap_or([0; 4])) as usize;
    let total_frame_len = 4 + len + 4 + 32;

    if bytes.len() < total_frame_len {
        return FrameReadResult::TruncatedTail {
            bytes_read: bytes.len(),
        };
    }

    if len > MAX_LEDGER_PAYLOAD_SIZE {
        return FrameReadResult::Corrupt {
            reason: format!(
                "Frame payload length {} exceeds max allowed {}",
                len, MAX_LEDGER_PAYLOAD_SIZE
            ),
        };
    }

    let payload = &bytes[4..4 + len];
    let crc_bytes = &bytes[4 + len..4 + len + 4];
    let stored_crc = u32::from_be_bytes(crc_bytes.try_into().unwrap_or([0; 4]));
    let computed_crc = crc32fast::hash(payload);

    if stored_crc != computed_crc {
        return FrameReadResult::Corrupt {
            reason: format!(
                "CRC mismatch: stored={:#010x}, computed={:#010x}",
                stored_crc, computed_crc
            ),
        };
    }

    let stored_mac_bytes = &bytes[4 + len + 4..total_frame_len];
    let mut stored_mac = [0u8; 32];
    stored_mac.copy_from_slice(stored_mac_bytes);

    let mut hasher = blake3::Hasher::new_keyed(key.as_bytes());
    hasher.update(prev_mac);
    hasher.update(payload);
    let expected_mac = *hasher.finalize().as_bytes();

    if stored_mac != expected_mac {
        return FrameReadResult::Corrupt {
            reason: "BLAKE3 chain MAC mismatch".into(),
        };
    }

    match LedgerEntry::from_payload_bytes(payload) {
        Ok(entry) => FrameReadResult::Ok {
            entry,
            chain_mac: stored_mac,
            frame_len: total_frame_len,
        },
        Err(e) => FrameReadResult::Corrupt {
            reason: format!("Payload deserialization failed: {}", e),
        },
    }
}
