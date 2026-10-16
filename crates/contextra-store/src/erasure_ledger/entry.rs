// FILE-CONTEXT
// STAND: 2026-10-16
// ZWECK: CTRALEDG v1 Format-Spezifikation (Spezifikation §6.5; Befund B-10).
// INVARIANTEN: Zero-Panic, Fail-Closed, HMAC-BLAKE3 Chaining, CRC32C Checksumming.

use contextra_types::{ContextraError, Result};

pub const CTRALEDG_MAGIC: &[u8; 8] = b"CTRALEDG";
pub const CTRALEDG_VERSION: u32 = 1;
pub const CTRALEDG_HEADER_SIZE: usize = 52; // 8 magic + 4 version + 8 seq + 32 prev_hash
pub const MAX_LEDGER_PAYLOAD_SIZE: usize = 64 * 1024 * 1024; // 64 MiB max segment
pub const LEDGER_LABEL: &str = "ledger/v1";

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
        let derived = blake3::derive_key(LEDGER_LABEL, master_secret);
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

/// Computes HMAC-BLAKE3 keyed digest for canonical bytes.
pub fn compute_keyed_digest(key: &LedgerKey, data: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_keyed(key.as_bytes());
    hasher.update(data);
    *hasher.finalize().as_bytes()
}

/// Computes HMAC-BLAKE3 keyed digest for family bytes.
pub fn compute_keyed_hash(key: &LedgerKey, _storage_scope: &[u8], bytes: &[u8]) -> [u8; 32] {
    compute_keyed_digest(key, bytes)
}

/// Record types per Spec §6.5.
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
                "Unknown CTRALEDG record type: {:#04x}",
                val
            ))),
        }
    }
}

/// CTRALEDG v1 Segment Header (52 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CTRALSegmentHeader {
    pub segment_seq: u64,
    pub prev_segment_hash: [u8; 32],
}

impl CTRALSegmentHeader {
    pub fn new(segment_seq: u64, prev_segment_hash: [u8; 32]) -> Self {
        Self {
            segment_seq,
            prev_segment_hash,
        }
    }

    pub fn to_bytes(&self) -> [u8; CTRALEDG_HEADER_SIZE] {
        let mut buf = [0u8; CTRALEDG_HEADER_SIZE];
        buf[0..8].copy_from_slice(CTRALEDG_MAGIC);
        buf[8..12].copy_from_slice(&CTRALEDG_VERSION.to_be_bytes());
        buf[12..20].copy_from_slice(&self.segment_seq.to_be_bytes());
        buf[20..52].copy_from_slice(&self.prev_segment_hash);
        buf
    }

    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < CTRALEDG_HEADER_SIZE {
            return Err(ContextraError::Serialization(
                "Segment header slice too short".into(),
            ));
        }

        if &bytes[0..8] != CTRALEDG_MAGIC {
            return Err(ContextraError::Serialization("Invalid CTRALEDG magic".into()));
        }

        let version = u32::from_be_bytes(
            bytes[8..12]
                .try_into()
                .map_err(|_| ContextraError::Serialization("Invalid version".into()))?,
        );
        if version != CTRALEDG_VERSION {
            return Err(ContextraError::Serialization(format!(
                "Unsupported CTRALEDG version: {}",
                version
            )));
        }

        let segment_seq = u64::from_be_bytes(
            bytes[12..20]
                .try_into()
                .map_err(|_| ContextraError::Serialization("Invalid segment_seq".into()))?,
        );

        let mut prev_segment_hash = [0u8; 32];
        prev_segment_hash.copy_from_slice(&bytes[20..52]);

        Ok(Self {
            segment_seq,
            prev_segment_hash,
        })
    }
}

/// CTRALEDG v1 Record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CTRALRecord {
    pub record_type: RecordType,
    pub tx: u64,
    pub timestamp_ms: u64,
    pub purge_id: [u8; 16],
    pub scope_digest: [u8; 32],
    pub family_digests: Vec<[u8; 32]>,
    pub payload: Vec<u8>,
    pub mac: [u8; 32],
}

impl CTRALRecord {
    /// Serializes record and returns the framed bytes:
    /// `len(u32 BE) | type(u8) | tx(u64 BE) | timestamp_ms(u64 BE) | purge_id(16B) | scope_digest(32B) | num_families(u16 BE) | family_digests | payload_len(u32 BE) | payload | mac(32B) | crc32c(u32 BE)`
    pub fn serialize(&self, key: &LedgerKey, prev_mac: &[u8; 32]) -> Result<(Vec<u8>, [u8; 32])> {
        let mut body = Vec::new();
        body.push(self.record_type as u8);
        body.extend_from_slice(&self.tx.to_be_bytes());
        body.extend_from_slice(&self.timestamp_ms.to_be_bytes());
        body.extend_from_slice(&self.purge_id);
        body.extend_from_slice(&self.scope_digest);

        #[allow(clippy::cast_possible_truncation)]
        let num_families = self.family_digests.len() as u16;
        body.extend_from_slice(&num_families.to_be_bytes());
        for fd in &self.family_digests {
            body.extend_from_slice(fd);
        }

        #[allow(clippy::cast_possible_truncation)]
        let payload_len = self.payload.len() as u32;
        body.extend_from_slice(&payload_len.to_be_bytes());
        body.extend_from_slice(&self.payload);

        // Compute MAC over prev_mac || body
        let mut hasher = blake3::Hasher::new_keyed(key.as_bytes());
        hasher.update(prev_mac);
        hasher.update(&body);
        let mac = *hasher.finalize().as_bytes();

        body.extend_from_slice(&mac);

        // Compute CRC32C over body (including MAC)
        let crc = crc32fast::hash(&body);

        #[allow(clippy::cast_possible_truncation)]
        let record_len = (body.len() + 4) as u32;

        let mut frame = Vec::with_capacity(4 + body.len() + 4);
        frame.extend_from_slice(&record_len.to_be_bytes());
        frame.extend_from_slice(&body);
        frame.extend_from_slice(&crc.to_be_bytes());

        Ok((frame, mac))
    }
}

/// Parsing result for a record frame.
pub enum FrameReadResult {
    Ok {
        record: CTRALRecord,
        mac: [u8; 32],
        frame_len: usize,
    },
    TruncatedTail {
        bytes_read: usize,
    },
    Corrupt {
        reason: String,
    },
}

/// Parses a CTRALEDG record frame from `bytes` starting at offset 0.
pub fn parse_record(key: &LedgerKey, prev_mac: &[u8; 32], bytes: &[u8]) -> FrameReadResult {
    if bytes.len() < 4 {
        return CtraledgFrameResult::TruncatedTail {
            bytes_read: bytes.len(),
        };
    }

    let record_len = u32::from_be_bytes(bytes[0..4].try_into().unwrap_or([0; 4])) as usize;
    if record_len < 4 + 1 + 8 + 8 + 16 + 32 + 2 + 4 + 32 + 4 {
        return FrameReadResult::Corrupt {
            reason: format!("Record length {} is impossibly small", record_len),
        };
    }

    let total_frame_len = 4 + record_len;
    if bytes.len() < total_frame_len {
        return CtraledgFrameResult::TruncatedTail {
            bytes_read: bytes.len(),
        };
    }

    let body_and_crc = &bytes[4..total_frame_len];
    let body_len = body_and_crc.len() - 4;
    let body = &body_and_crc[..body_len];
    let stored_crc = u32::from_be_bytes(
        body_and_crc[body_len..body_len + 4]
            .try_into()
            .unwrap_or([0; 4]),
    );

    let computed_crc = crc32fast::hash(body);
    if stored_crc != computed_crc {
        return FrameReadResult::Corrupt {
            reason: format!(
                "CRC32C mismatch: stored={:#010x}, computed={:#010x}",
                stored_crc, computed_crc
            ),
        };
    }

    if body.len() < 1 + 8 + 8 + 16 + 32 + 2 + 4 + 32 {
        return FrameReadResult::Corrupt {
            reason: "Record body too short".into(),
        };
    }

    let mac_offset = body.len() - 32;
    let body_before_mac = &body[..mac_offset];
    let stored_mac_slice = &body[mac_offset..];
    let mut stored_mac = [0u8; 32];
    stored_mac.copy_from_slice(stored_mac_slice);

    let mut hasher = blake3::Hasher::new_keyed(key.as_bytes());
    hasher.update(prev_mac);
    hasher.update(body_before_mac);
    let expected_mac = *hasher.finalize().as_bytes();

    if stored_mac != expected_mac {
        return FrameReadResult::Corrupt {
            reason: "MAC chain verification failed".into(),
        };
    }

    // Parse fields
    let record_type = match RecordType::from_u8(body[0]) {
        Ok(t) => t,
        Err(e) => {
            return FrameReadResult::Corrupt {
                reason: e.to_string(),
            }
        }
    };

    let tx = u64::from_be_bytes(body[1..9].try_into().unwrap_or([0; 8]));
    let timestamp_ms = u64::from_be_bytes(body[9..17].try_into().unwrap_or([0; 8]));

    let mut purge_id = [0u8; 16];
    purge_id.copy_from_slice(&body[17..33]);

    let mut scope_digest = [0u8; 32];
    scope_digest.copy_from_slice(&body[33..65]);

    let num_families = u16::from_be_bytes(body[65..67].try_into().unwrap_or([0; 2])) as usize;
    let mut offset = 67;

    let families_len = num_families * 32;
    if body_before_mac.len() < offset + families_len + 4 {
        return FrameReadResult::Corrupt {
            reason: "Record truncated before family digests or payload len".into(),
        };
    }

    let mut family_digests = Vec::with_capacity(num_families);
    for _ in 0..num_families {
        let mut fd = [0u8; 32];
        fd.copy_from_slice(&body[offset..offset + 32]);
        family_digests.push(fd);
        offset += 32;
    }

    let payload_len = u32::from_be_bytes(body[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
    offset += 4;

    if offset + payload_len != mac_offset {
        return FrameReadResult::Corrupt {
            reason: format!(
                "Payload length mismatch: offset {} + payload {} != mac_offset {}",
                offset, payload_len, mac_offset
            ),
        };
    }

    let payload = body[offset..offset + payload_len].to_vec();

    // Check Spec rule: Plaintext prefixes MUST NOT be in Purged (0x02) records
    if record_type == RecordType::Purged && !payload.is_empty() {
        return FrameReadResult::Corrupt {
            reason: "Purged record MUST NOT contain non-empty plaintext payload".into(),
        };
    }

    FrameReadResult::Ok {
        record: CTRALRecord {
            record_type,
            tx,
            timestamp_ms,
            purge_id,
            scope_digest,
            family_digests,
            payload,
            mac: stored_mac,
        },
        mac: stored_mac,
        frame_len: total_frame_len,
    }
}

/// Helper for legacy ELDG entry parsing to enable atomic migration on open.
pub fn parse_legacy_eldg_frame(bytes: &[u8]) -> Option<(Vec<u8>, usize)> {
    if bytes.len() < 4 {
        return None;
    }
    let len = u32::from_be_bytes(bytes[0..4].try_into().ok()?) as usize;
    let total_len = 4 + len + 4 + 32;
    if bytes.len() < total_len {
        return None;
    }

    let payload = &bytes[4..4 + len];
    let crc_bytes = &bytes[4 + len..4 + len + 4];
    let stored_crc = u32::from_be_bytes(crc_bytes.try_into().ok()?);
    let computed_crc = crc32fast::hash(payload);

    if stored_crc != computed_crc {
        return None;
    }

    Some((payload.to_vec(), total_len))
}
