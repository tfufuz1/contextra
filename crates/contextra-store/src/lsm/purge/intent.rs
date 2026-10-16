// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Persistenter Purge-Intent für Crash-Konsistency (Spezifikation C5.1/C5.6).
// INVARIANTEN: Atomare Persistenz via atomic_replace mit CRC32C-Schutz, Korruptionserkennung propagiert Err.

pub use contextra_ports::PurgeState;
use super::plan::PurgePhase;
use crate::compaction::purge_filter::PurgeScope;
use contextra_durable_fs::atomic_replace;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;

const INTENT_MAGIC: &[u8; 4] = b"ORXI";
const INTENT_VERSION: u8 = 1;

/// Durable intent representing an active or pending purge operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrxPurgeIntent {
    /// Unique identifier for this purge operation.
    pub purge_id: [u8; 16],
    /// Optional ledger sequence number reference for scope reconstruction.
    pub ledger_seq: Option<u64>,
    /// Optional cryptographic proof ID reference for scope reconstruction.
    pub proof_id: Option<[u8; 32]>,
    /// BLAKE3 hash of the purge scope (keys or prefixes). Key bytes are never persisted.
    pub scope_hash: [u8; 32],
    /// Cutoff transaction ID for MVCC visibility during purge compaction.
    pub as_of_tx: u64,
    /// Current phase of the purge state machine.
    pub phase: PurgePhase,
    /// Execution attempt counter.
    pub attempt: u32,
}

impl OrxPurgeIntent {
    /// Creates a new `OrxPurgeIntent` for a given scope and cutoff transaction.
    pub fn new(
        purge_id: [u8; 16],
        scope: &PurgeScope,
        as_of_tx: u64,
        ledger_seq: Option<u64>,
        proof_id: Option<[u8; 32]>,
        initial_phase: PurgePhase,
    ) -> Self {
        let scope_hash = Self::compute_scope_hash(scope);
        Self {
            purge_id,
            ledger_seq,
            proof_id,
            scope_hash,
            as_of_tx,
            phase: initial_phase,
            attempt: 1,
        }
    }

    /// Computes BLAKE3 hash over scope content (keys or prefixes).
    pub fn compute_scope_hash(scope: &PurgeScope) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        match scope {
            PurgeScope::Keys(keys) => {
                hasher.update(b"KEYS:");
                for key in keys {
                    hasher.update(&(key.len() as u64).to_le_bytes());
                    hasher.update(key);
                }
            }
            PurgeScope::Prefixes(prefixes) => {
                hasher.update(b"PREFIXES:");
                for prefix in prefixes {
                    hasher.update(&(prefix.len() as u64).to_le_bytes());
                    hasher.update(prefix);
                }
            }
        }
        *hasher.finalize().as_bytes()
    }

    /// Serializes intent to compact binary bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(128);
        buf.extend_from_slice(&self.purge_id);

        match self.ledger_seq {
            Some(seq) => {
                buf.push(1);
                buf.extend_from_slice(&seq.to_le_bytes());
            }
            None => buf.push(0),
        }

        match self.proof_id {
            Some(pid) => {
                buf.push(1);
                buf.extend_from_slice(&pid);
            }
            None => buf.push(0),
        }

        buf.extend_from_slice(&self.scope_hash);
        buf.extend_from_slice(&self.as_of_tx.to_le_bytes());
        buf.push(self.phase.to_u8());
        buf.extend_from_slice(&self.attempt.to_le_bytes());
        buf
    }

    /// Deserializes intent from compact binary bytes.
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 16 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "intent binary too short for purge_id",
            ));
        }

        let mut purge_id = [0u8; 16];
        purge_id.copy_from_slice(&bytes[..16]);
        bytes = &bytes[16..];

        if bytes.is_empty() {
            return Err(Error::new(ErrorKind::InvalidData, "missing ledger_seq tag"));
        }
        let ledger_seq = if bytes[0] == 1 {
            bytes = &bytes[1..];
            if bytes.len() < 8 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "truncated ledger_seq value",
                ));
            }
            let seq = u64::from_le_bytes(
                bytes[..8]
                    .try_into()
                    .map_err(|_| Error::new(ErrorKind::InvalidData, "ledger_seq parse error"))?,
            );
            bytes = &bytes[8..];
            Some(seq)
        } else if bytes[0] == 0 {
            bytes = &bytes[1..];
            None
        } else {
            return Err(Error::new(ErrorKind::InvalidData, "invalid ledger_seq tag"));
        };

        if bytes.is_empty() {
            return Err(Error::new(ErrorKind::InvalidData, "missing proof_id tag"));
        }
        let proof_id = if bytes[0] == 1 {
            bytes = &bytes[1..];
            if bytes.len() < 32 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "truncated proof_id value",
                ));
            }
            let mut pid = [0u8; 32];
            pid.copy_from_slice(&bytes[..32]);
            bytes = &bytes[32..];
            Some(pid)
        } else if bytes[0] == 0 {
            bytes = &bytes[1..];
            None
        } else {
            return Err(Error::new(ErrorKind::InvalidData, "invalid proof_id tag"));
        };

        if bytes.len() < 32 + 8 + 1 + 4 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "intent binary truncated for remaining fields",
            ));
        }

        let mut scope_hash = [0u8; 32];
        scope_hash.copy_from_slice(&bytes[..32]);
        bytes = &bytes[32..];

        let as_of_tx = u64::from_le_bytes(
            bytes[..8]
                .try_into()
                .map_err(|_| Error::new(ErrorKind::InvalidData, "as_of_tx parse error"))?,
        );
        bytes = &bytes[8..];

        let phase = PurgePhase::from_u8(bytes[0])
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "invalid PurgePhase code"))?;
        bytes = &bytes[1..];

        let attempt = u32::from_le_bytes(
            bytes[..4]
                .try_into()
                .map_err(|_| Error::new(ErrorKind::InvalidData, "attempt parse error"))?,
        );

        Ok(Self {
            purge_id,
            ledger_seq,
            proof_id,
            scope_hash,
            as_of_tx,
            phase,
            attempt,
        })
    }

    /// Persists this intent to `path` via `atomic_replace` with CRC32C frame.
    /// Ensures parent directory exists and is synced after replace.
    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let payload = self.to_bytes();
        let checksum = crc32fast::hash(&payload);

        let mut buffer = Vec::with_capacity(4 + 1 + 4 + 4 + payload.len());
        buffer.extend_from_slice(INTENT_MAGIC);
        buffer.push(INTENT_VERSION);
        buffer.extend_from_slice(&checksum.to_le_bytes());
        buffer.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        buffer.extend_from_slice(&payload);

        atomic_replace(path, &buffer)?;

        if let Some(parent) = path.parent() {
            if let Ok(dir_file) = fs::File::open(parent) {
                let _ = dir_file.sync_all();
            }
        }
        Ok(())
    }

    /// Reads an intent from `path`.
    ///
    /// Returns `Ok(None)` if the file does not exist.
    /// Returns `Err(...)` if the file exists but is corrupted, empty, or CRC fails.
    pub fn read_from_file(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let bytes = fs::read(path)?;
        if bytes.len() < 13 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "purge intent header too short",
            ));
        }

        if &bytes[0..4] != INTENT_MAGIC {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "invalid purge intent magic header",
            ));
        }

        if bytes[4] != INTENT_VERSION {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("unsupported purge intent version: {}", bytes[4]),
            ));
        }

        let expected_checksum = u32::from_le_bytes(
            bytes[5..9]
                .try_into()
                .map_err(|_| Error::new(ErrorKind::InvalidData, "checksum parse error"))?,
        );

        let payload_len = u32::from_le_bytes(
            bytes[9..13]
                .try_into()
                .map_err(|_| Error::new(ErrorKind::InvalidData, "payload len parse error"))?,
        ) as usize;

        if bytes.len() < 13 + payload_len {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "purge intent file truncated",
            ));
        }

        let payload = &bytes[13..13 + payload_len];
        let actual_checksum = crc32fast::hash(payload);

        if actual_checksum != expected_checksum {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "purge intent CRC32C checksum mismatch",
            ));
        }

        let intent = Self::from_bytes(payload)?;
        Ok(Some(intent))
    }

    /// Reads all intent files located in `storage_dir/purge/` or fallback `storage_dir/purge.intent`.
    pub fn read_all_intents(storage_dir: &Path) -> Result<Vec<Self>> {
        let mut intents = Vec::new();

        // 1. Check legacy single intent file at `storage_dir/purge.intent`
        let legacy_file = storage_dir.join("purge.intent");
        if let Some(intent) = Self::read_from_file(&legacy_file)? {
            intents.push(intent);
        }

        // 2. Scan `storage_dir/purge/` directory for `<purge_id>.intent` files
        let purge_dir = storage_dir.join("purge");
        if purge_dir.is_dir() {
            for entry in fs::read_dir(&purge_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("intent") {
                    if let Some(intent) = Self::read_from_file(&path)? {
                        intents.push(intent);
                    }
                }
            }
        }

        Ok(intents)
    }
}
