// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Persistenter Purge Intent (C5.1 / C5.6).
// INVARIANTEN: CRC32C-Rahmen über JSON-Payload, atomic_replace, fsync File + Verzeichnis,
//              Scope wird NIEMALS im Footprint/Intent im Klartext persistiert, nur ledger_seq/proof_id + Ref-Hash.

use super::plan::PurgePhase;
use contextra_durable_fs::{atomic_replace, sync_dir};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Durably persisted intent for an in-flight Purge operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrxPurgeIntent {
    /// Unique 16-byte ID for this purge attempt/session.
    pub purge_id: [u8; 16],
    /// BLAKE3/SHA256 digest (32 bytes) of the PurgeScope (never raw keys).
    pub scope_hash: [u8; 32],
    /// Number of items/prefixes in the scope.
    pub scope_len: usize,
    /// Optional ledger sequence number to reconstruct scope from ledger.
    pub ledger_seq: Option<u64>,
    /// Optional proof ID associated with deletion proof.
    pub proof_id: Option<String>,
    /// Cutoff MVCC sequence number.
    pub as_of_tx: u64,
    /// Current execution phase.
    pub phase: PurgePhase,
    /// Attempt counter for retries.
    pub attempt: u32,
}

impl OrxPurgeIntent {
    /// Creates a new `OrxPurgeIntent`.
    pub fn new(
        purge_id: [u8; 16],
        scope_hash: [u8; 32],
        scope_len: usize,
        ledger_seq: Option<u64>,
        proof_id: Option<String>,
        as_of_tx: u64,
        phase: PurgePhase,
    ) -> Self {
        Self {
            purge_id,
            scope_hash,
            scope_len,
            ledger_seq,
            proof_id,
            as_of_tx,
            phase,
            attempt: 1,
        }
    }

    /// Persists `self` to `path` with a CRC32C frame via `atomic_replace` and parent directory `sync_dir`.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json_bytes = serde_json::to_vec(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let crc = crc32fast::hash(&json_bytes);

        let mut frame = Vec::with_capacity(4 + json_bytes.len());
        frame.extend_from_slice(&crc.to_le_bytes());
        frame.extend_from_slice(&json_bytes);

        atomic_replace(path, &frame)?;

        if let Some(parent) = path.parent() {
            sync_dir(parent)?;
        }

        Ok(())
    }

    /// Loads `OrxPurgeIntent` from `path`, verifying the CRC32C frame.
    /// Returns `Err` on missing file, CRC mismatch, or deserialization failure.
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read(path)?;
        if content.len() < 4 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Intent file too short for CRC header",
            ));
        }

        let stored_crc = u32::from_le_bytes(
            content[0..4]
                .try_into()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid CRC bytes"))?,
        );

        let payload = &content[4..];
        let computed_crc = crc32fast::hash(payload);

        if stored_crc != computed_crc {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Intent CRC mismatch: stored={:#x}, computed={:#x}",
                    stored_crc, computed_crc
                ),
            ));
        }

        let intent: Self = serde_json::from_slice(payload)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(intent)
    }
}
