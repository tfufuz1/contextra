// FILE-CONTEXT
// STAND: 2026-10-16
// ZWECK: CTRALEDG v1 Anti-Resurrection Ledger Formate, Framing und Krypto-Logik (Spezifikation §6.5).
// INVARIANTEN: Zero-Panic, Fail-Closed, BLAKE3-Keyed MAC Chaining, CRC32 Checksumming.

use contextra_types::{ContextraError, Result};

pub const ERASED_LEDGER_DOMAIN: &[u8] = b"contextra/erasure-ledger/v1";
pub const CTRALEDG_MAGIC: &[u8; 8] = b"CTRALEDG";
pub const CTRALEDG_VERSION: u32 = 1;
pub const MAX_LEDGER_PAYLOAD_SIZE: usize = 64 * 1024 * 1024; // 64 MiB max segment size

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
        let derived = blake3::derive_key("ledger/v1", master_secret);
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

/// Computes HMAC-BLAKE3 digest of arbitrary canonical bytes using ledger_key.
pub fn compute_digest(key: &LedgerKey, canonical_bytes: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_keyed(key.as_bytes());
    hasher.update(canonical_bytes);
    *hasher.finalize().as_bytes()
}

/// Record type codes according to Spezifikation §6.5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RecordType {
    Intent = 0x01,
    Purged = 0x02,
    EpochMarker = 0x03,
    AdoptMarker = 0x04,
}

impl RecordType {
    pub fn from_u8(val: u8) -> Result<Self> {
        match val {
            0x01 => Ok(RecordType::Intent),
            0x02 => Ok(RecordType::Purged),
            0x03 => Ok(RecordType::EpochMarker),
            0x04 => Ok(RecordType::AdoptMarker),
            _ => Err(ContextraError::Serialization(format!(
                "Invalid CTRALEDG RecordType: {:#04x}",
                val
            ))),
        }
    }
}

/// CTRALEDG Segment Header (52 bytes)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtraledgSegmentHeader {
    pub magic: [u8; 8],
    pub version: u32,
    pub segment_num: u64,
    pub prev_segment_hash: [u8; 32],
}

impl CtraledgSegmentHeader {
    pub const SIZE: usize = 8 + 4 + 8 + 32;

    pub fn new(segment_num: u64, prev_segment_hash: [u8; 32]) -> Self {
        Self {
            magic: *CTRALEDG_MAGIC,
            version: CTRALEDG_VERSION,
            segment_num,
            prev_segment_hash,
        }
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        buf[0..8].copy_from_slice(&self.magic);
        buf[8..12].copy_from_slice(&self.version.to_le_bytes());
        buf[12..20].copy_from_slice(&self.segment_num.to_le_bytes());
        buf[20..52].copy_from_slice(&self.prev_segment_hash);
        buf
    }

    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(ContextraError::Serialization(
                "CTRALEDG header too short".into(),
            ));
        }
        if &bytes[0..8] != CTRALEDG_MAGIC {
            return Err(ContextraError::Storage(format!(
                "Invalid CTRALEDG magic header: {:?}",
                &bytes[0..8]
            )));
        }
        let version = u32::from_le_bytes(bytes[8..12].try_into().unwrap_or([0; 4]));
        if version > CTRALEDG_VERSION {
            return Err(ContextraError::Storage(format!(
                "Unsupported CTRALEDG version: {}",
                version
            )));
        }
        let segment_num = u64::from_le_bytes(bytes[12..20].try_into().unwrap_or([0; 8]));
        let mut prev_segment_hash = [0u8; 32];
        prev_segment_hash.copy_from_slice(&bytes[20..52]);

        Ok(Self {
            magic: *CTRALEDG_MAGIC,
            version,
            segment_num,
            prev_segment_hash,
        })
    }
}

/// CTRALEDG v1 Record
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtraledgRecord {
    pub record_type: RecordType,
    pub tx: u64,
    pub timestamp: u64,
    pub purge_id: [u8; 16],
    pub scope_digest: [u8; 32],
    pub family_digests: Vec<[u8; 32]>,
    pub payload: Vec<u8>,
    pub mac: [u8; 32],
    pub crc32c: u32,
}

impl CtraledgRecord {
    /// Computes MAC over `prev_mac` + canonical record content (excluding mac and crc32c).
    pub fn compute_mac(
        key: &LedgerKey,
        prev_mac: &[u8; 32],
        record_type: RecordType,
        tx: u64,
        timestamp: u64,
        purge_id: &[u8; 16],
        scope_digest: &[u8; 32],
        family_digests: &[[u8; 32]],
        payload: &[u8],
    ) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new_keyed(key.as_bytes());
        hasher.update(prev_mac);
        hasher.update(&[record_type as u8]);
        hasher.update(&tx.to_le_bytes());
        hasher.update(&timestamp.to_le_bytes());
        hasher.update(purge_id);
        hasher.update(scope_digest);
        hasher.update(&(family_digests.len() as u16).to_le_bytes());
        for fd in family_digests {
            hasher.update(fd);
        }
        hasher.update(&(payload.len() as u32).to_le_bytes());
        hasher.update(payload);
        *hasher.finalize().as_bytes()
    }

    /// Serializes CTRALEDG record into binary frame:
    /// `len(u32 LE) | record_type(u8) | tx(u64 LE) | timestamp(u64 LE) | purge_id(16) | scope_digest(32) | family_count(u16 LE) | family_digests | payload_len(u32 LE) | payload | mac(32) | crc32c(u32 LE)`
    pub fn serialize_frame(
        key: &LedgerKey,
        prev_mac: &[u8; 32],
        record_type: RecordType,
        tx: u64,
        timestamp: u64,
        purge_id: [u8; 16],
        scope_digest: [u8; 32],
        family_digests: Vec<[u8; 32]>,
        payload: Vec<u8>,
    ) -> Result<(Vec<u8>, [u8; 32])> {
        let mac = Self::compute_mac(
            key,
            prev_mac,
            record_type,
            tx,
            timestamp,
            &purge_id,
            &scope_digest,
            &family_digests,
            &payload,
        );

        let mut body = Vec::new();
        body.push(record_type as u8);
        body.extend_from_slice(&tx.to_le_bytes());
        body.extend_from_slice(&timestamp.to_le_bytes());
        body.extend_from_slice(&purge_id);
        body.extend_from_slice(&scope_digest);
        body.extend_from_slice(&(family_digests.len() as u16).to_le_bytes());
        for fd in &family_digests {
            body.extend_from_slice(fd);
        }
        body.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        body.extend_from_slice(&payload);
        body.extend_from_slice(&mac);

        let crc32c = crc32fast::hash(&body);

        let body_len = (body.len() + 4) as u32; // body + crc32c

        let mut frame = Vec::with_capacity(4 + body.len() + 4);
        frame.extend_from_slice(&body_len.to_le_bytes());
        frame.extend_from_slice(&body);
        frame.extend_from_slice(&crc32c.to_le_bytes());

        Ok((frame, mac))
    }
}

/// Kind of entry recorded in the erasure ledger (compatibility wrapper).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerEntryKind {
    Key,
    Prefix { prefix_len: u32 },
}

/// Legacy / high-level entry representation in erasure ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntry {
    pub ledger_seq: u64,
    pub kind: LedgerEntryKind,
    pub keyed_hash: [u8; 32],
    pub erased_at_tx: u64,
    pub proof_id: [u8; 16],
}

impl LedgerEntry {
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

/// Result of parsing a CTRALEDG frame from disk.
pub enum CtraledgFrameResult {
    Ok {
        record: CtraledgRecord,
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

/// Reads and validates a single CTRALEDG frame from `bytes` starting at offset `0`.
pub fn parse_ctraledg_frame(
    key: &LedgerKey,
    prev_mac: &[u8; 32],
    bytes: &[u8],
) -> CtraledgFrameResult {
    if bytes.len() < 4 {
        return CtraledgFrameResult::TruncatedTail {
            bytes_read: bytes.len(),
        };
    }

    let body_len = u32::from_le_bytes(bytes[0..4].try_into().unwrap_or([0; 4])) as usize;
    let total_frame_len = 4 + body_len;

    if bytes.len() < total_frame_len {
        return CtraledgFrameResult::TruncatedTail {
            bytes_read: bytes.len(),
        };
    }

    let body_and_crc = &bytes[4..total_frame_len];
    if body_and_crc.len() < 4 {
        return CtraledgFrameResult::TruncatedTail {
            bytes_read: bytes.len(),
        };
    }

    let body = &body_and_crc[..body_and_crc.len() - 4];
    let stored_crc = u32::from_le_bytes(
        body_and_crc[body_and_crc.len() - 4..]
            .try_into()
            .unwrap_or([0; 4]),
    );
    let computed_crc = crc32fast::hash(body);

    if stored_crc != computed_crc {
        return CtraledgFrameResult::Corrupt {
            reason: format!(
                "CRC32 mismatch: stored={:#010x}, computed={:#010x}",
                stored_crc, computed_crc
            ),
        };
    }

    // Parse record fields from body:
    // record_type(1) | tx(8) | timestamp(8) | purge_id(16) | scope_digest(32) | family_count(2) | family_digests(32*N) | payload_len(4) | payload | mac(32)
    let min_body_size = 1 + 8 + 8 + 16 + 32 + 2 + 4 + 32;
    if body.len() < min_body_size {
        return CtraledgFrameResult::Corrupt {
            reason: "CTRALEDG record body too short".into(),
        };
    }

    let record_type = match RecordType::from_u8(body[0]) {
        Ok(rt) => rt,
        Err(e) => return CtraledgFrameResult::Corrupt { reason: e.to_string() },
    };

    let tx = u64::from_le_bytes(body[1..9].try_into().unwrap_or([0; 8]));
    let timestamp = u64::from_le_bytes(body[9..17].try_into().unwrap_or([0; 8]));
    let mut purge_id = [0u8; 16];
    purge_id.copy_from_slice(&body[17..33]);

    let mut scope_digest = [0u8; 32];
    scope_digest.copy_from_slice(&body[33..65]);

    let family_count = u16::from_le_bytes(body[65..67].try_into().unwrap_or([0; 2])) as usize;
    let mut offset = 67;

    let mut family_digests = Vec::with_capacity(family_count);
    for _ in 0..family_count {
        if body.len() < offset + 32 {
            return CtraledgFrameResult::Corrupt {
                reason: "CTRALEDG family digest truncated".into(),
            };
        }
        let mut fd = [0u8; 32];
        fd.copy_from_slice(&body[offset..offset + 32]);
        family_digests.push(fd);
        offset += 32;
    }

    if body.len() < offset + 4 + 32 {
        return CtraledgFrameResult::Corrupt {
            reason: "CTRALEDG payload len truncated".into(),
        };
    }

    let payload_len = u32::from_le_bytes(body[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
    offset += 4;

    if body.len() < offset + payload_len + 32 {
        return CtraledgFrameResult::Corrupt {
            reason: "CTRALEDG payload truncated".into(),
        };
    }

    let payload = body[offset..offset + payload_len].to_vec();
    offset += payload_len;

    if body.len() != offset + 32 {
        return CtraledgFrameResult::Corrupt {
            reason: "CTRALEDG extra trailing bytes in record body".into(),
        };
    }

    let mut stored_mac = [0u8; 32];
    stored_mac.copy_from_slice(&body[offset..offset + 32]);

    let expected_mac = CtraledgRecord::compute_mac(
        key,
        prev_mac,
        record_type,
        tx,
        timestamp,
        &purge_id,
        &scope_digest,
        &family_digests,
        &payload,
    );

    if stored_mac != expected_mac {
        return CtraledgFrameResult::Corrupt {
            reason: "BLAKE3 MAC mismatch in CTRALEDG record".into(),
        };
    }

    let record = CtraledgRecord {
        record_type,
        tx,
        timestamp,
        purge_id,
        scope_digest,
        family_digests,
        payload,
        mac: stored_mac,
        crc32c: stored_crc,
    };

    CtraledgFrameResult::Ok {
        record,
        chain_mac: stored_mac,
        frame_len: total_frame_len,
    }
}

// Backward-compatible parse_frame & serialize_frame definitions for tests & transition
pub fn serialize_frame(
    key: &LedgerKey,
    prev_mac: &[u8; 32],
    payload: &[u8],
) -> Result<(Vec<u8>, [u8; 32])> {
    let entry = LedgerEntry::from_payload_bytes(payload)?;
    CtraledgRecord::serialize_frame(
        key,
        prev_mac,
        RecordType::Intent,
        entry.erased_at_tx,
        0,
        entry.proof_id,
        entry.keyed_hash,
        Vec::new(),
        payload.to_vec(),
    )
}

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

pub fn parse_frame(key: &LedgerKey, prev_mac: &[u8; 32], bytes: &[u8]) -> FrameReadResult {
    match parse_ctraledg_frame(key, prev_mac, bytes) {
        CtraledgFrameResult::Ok {
            record,
            chain_mac,
            frame_len,
        } => {
            let entry = if !record.payload.is_empty() {
                match LedgerEntry::from_payload_bytes(&record.payload) {
                    Ok(e) => e,
                    Err(_) => LedgerEntry {
                        ledger_seq: 1,
                        kind: LedgerEntryKind::Key,
                        keyed_hash: record.scope_digest,
                        erased_at_tx: record.tx,
                        proof_id: record.purge_id,
                    },
                }
            } else {
                LedgerEntry {
                    ledger_seq: 1,
                    kind: LedgerEntryKind::Key,
                    keyed_hash: record.scope_digest,
                    erased_at_tx: record.tx,
                    proof_id: record.purge_id,
                }
            };
            FrameReadResult::Ok {
                entry,
                chain_mac,
                frame_len,
            }
        }
        CtraledgFrameResult::TruncatedTail { bytes_read } => {
            FrameReadResult::TruncatedTail { bytes_read }
        }
        CtraledgFrameResult::Corrupt { reason } => FrameReadResult::Corrupt { reason },
    }
}
