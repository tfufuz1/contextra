// FILE-CONTEXT
// STAND: 2026-10-11 (T-2026-9103)
// ZWECK: ErasureLedger Modul-Einstieg und Implementierung der Append-Only Persistenz und Decision Filter.
// INVARIANTEN: Zero-Panic, Fail-Closed, INV-DURABILITY-RING (fsync_parent_dir + sync_all).

pub mod coverage;
pub mod entry;

use contextra_core::{ContextraError, Result};
pub use coverage::{LedgerCoverage, LedgerGap};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

pub use entry::{
    compute_keyed_hash, parse_frame, serialize_frame, FrameReadResult, LedgerEntry,
    LedgerEntryKind, LedgerKey, ERASED_LEDGER_DOMAIN,
};

pub const LEDGER_FILENAME: &str = "erasure.ledger";
pub const HEAD_FILENAME: &str = "ledger.head";
pub const INITIAL_CHAIN_MAC: [u8; 32] = [0u8; 32];

/// Decision returned when evaluating whether a key version at transaction `tx` should be kept or dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerDecision {
    Keep,
    Drop,
}

/// Status returned when opening the erasure ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerOpenStatus {
    Fresh,
    Ok,
    RollbackSuspected { reason: String },
    Corrupt { reason: String },
}

/// In-memory representation and file handle of the ErasureLedger.
pub struct ErasureLedger {
    dir: PathBuf,
    key: LedgerKey,
    last_chain_mac: [u8; 32],
    entries: Vec<LedgerEntry>,
    seen_proof_ids: HashSet<[u8; 16]>,
    distinct_prefix_lens: HashSet<u32>,
    db_instance_id: [u8; 16],
}

impl std::fmt::Debug for ErasureLedger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErasureLedger")
            .field("dir", &self.dir)
            .field("last_chain_mac", &self.last_chain_mac)
            .field("entries_count", &self.entries.len())
            .finish()
    }
}

impl ErasureLedger {
    /// Opens the erasure ledger or returns fail-closed status if rollback/corruption is suspected.
    pub async fn open_or_fail_closed(
        dir: impl AsRef<Path>,
        key: LedgerKey,
        db_has_data: bool,
        expected_head_mac: Option<[u8; 32]>,
    ) -> Result<(Option<Self>, LedgerOpenStatus)> {
        let dir = dir.as_ref().to_path_buf();
        let ledger_path = dir.join(LEDGER_FILENAME);
        let head_path = dir.join(HEAD_FILENAME);

        let ledger_exists = tokio::fs::try_exists(&ledger_path).await.unwrap_or(false);
        let head_exists = tokio::fs::try_exists(&head_path).await.unwrap_or(false);

        if !ledger_exists {
            if db_has_data {
                return Ok((
                    None,
                    LedgerOpenStatus::RollbackSuspected {
                        reason: "Database contains data but erasure.ledger is missing".into(),
                    },
                ));
            }

            // Fresh database: create new ledger
            let mut db_instance_id = [0u8; 16];
            rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut db_instance_id);

            let ledger = Self {
                dir: dir.clone(),
                key,
                last_chain_mac: INITIAL_CHAIN_MAC,
                entries: Vec::new(),
                seen_proof_ids: HashSet::new(),
                distinct_prefix_lens: HashSet::new(),
                db_instance_id,
            };

            ledger.write_head_marker().await?;

            return Ok((Some(ledger), LedgerOpenStatus::Fresh));
        }

        // Read and parse ledger contents
        let file_bytes = tokio::fs::read(&ledger_path).await.map_err(|e| {
            ContextraError::Storage(format!("Failed to read erasure.ledger: {}", e))
        })?;

        let mut offset = 0;
        let mut prev_mac = INITIAL_CHAIN_MAC;
        let mut entries = Vec::new();
        let mut seen_proof_ids = HashSet::new();
        let mut distinct_prefix_lens = HashSet::new();
        let mut torn_tail_detected = false;
        let mut verified_len = 0;

        while offset < file_bytes.len() {
            let res = parse_frame(&key, &prev_mac, &file_bytes[offset..]);
            match res {
                FrameReadResult::Ok {
                    entry,
                    chain_mac,
                    frame_len,
                } => {
                    if seen_proof_ids.contains(&entry.proof_id) {
                        // Deduplicated entry
                    } else {
                        seen_proof_ids.insert(entry.proof_id);
                        if let LedgerEntryKind::Prefix { prefix_len } = entry.kind {
                            distinct_prefix_lens.insert(prefix_len);
                        }
                        entries.push(entry);
                    }
                    prev_mac = chain_mac;
                    offset += frame_len;
                    verified_len = offset;
                }
                FrameReadResult::TruncatedTail { bytes_read: _ } => {
                    tracing::warn!(
                        "ErasureLedger tail truncation detected at offset {}",
                        verified_len
                    );
                    torn_tail_detected = true;
                    break;
                }
                FrameReadResult::Corrupt { reason } => {
                    return Ok((
                        None,
                        LedgerOpenStatus::Corrupt {
                            reason: format!("Corruption at offset {}: {}", offset, reason),
                        },
                    ));
                }
            }
        }

        // If torn tail was detected, truncate file to last verified position
        if torn_tail_detected && verified_len < file_bytes.len() {
            let file = OpenOptions::new()
                .write(true)
                .open(&ledger_path)
                .await
                .map_err(|e| ContextraError::Storage(e.to_string()))?;
            file.set_len(verified_len as u64)
                .await
                .map_err(|e| ContextraError::Storage(e.to_string()))?;
            file.sync_all()
                .await
                .map_err(|e| ContextraError::Storage(e.to_string()))?;
            crate::util::fsync_parent_dir(&ledger_path).await?;
        }

        // Check head marker consistency
        let (stored_db_id, stored_head_mac) = if head_exists {
            Self::read_head_marker(&head_path).await?
        } else {
            ([0u8; 16], INITIAL_CHAIN_MAC)
        };

        if head_exists && stored_head_mac != prev_mac {
            return Ok((
                None,
                LedgerOpenStatus::RollbackSuspected {
                    reason: "Head marker MAC does not match ledger chain MAC".into(),
                },
            ));
        }

        if let Some(exp_mac) = expected_head_mac {
            if exp_mac != prev_mac {
                return Ok((
                    None,
                    LedgerOpenStatus::RollbackSuspected {
                        reason: "Expected head MAC mismatch".into(),
                    },
                ));
            }
        }

        let db_instance_id = if head_exists {
            stored_db_id
        } else {
            let mut id = [0u8; 16];
            rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut id);
            id
        };

        let ledger = Self {
            dir,
            key,
            last_chain_mac: prev_mac,
            entries,
            seen_proof_ids,
            distinct_prefix_lens,
            db_instance_id,
        };

        if !head_exists {
            ledger.write_head_marker().await?;
        }

        Ok((Some(ledger), LedgerOpenStatus::Ok))
    }

    /// Appends entries to the erasure ledger with atomic deduplication and fsync.
    pub async fn append(&mut self, entries: &[LedgerEntry]) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }

        let mut to_write = Vec::new();
        for entry in entries {
            if self.seen_proof_ids.contains(&entry.proof_id) {
                continue;
            }
            to_write.push(entry.clone());
        }

        if to_write.is_empty() {
            return Ok(());
        }

        let ledger_path = self.dir.join(LEDGER_FILENAME);

        let mut buffer = Vec::new();
        let mut current_mac = self.last_chain_mac;

        for entry in &to_write {
            let payload = entry.to_payload_bytes()?;
            let (frame_bytes, new_mac) = serialize_frame(&self.key, &current_mac, &payload)?;
            buffer.extend_from_slice(&frame_bytes);
            current_mac = new_mac;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&ledger_path)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to open ledger: {}", e)))?;

        file.write_all(&buffer).await.map_err(|e| {
            ContextraError::Storage(format!("ErasureLedger write_all failed: {}", e))
        })?;
        file.sync_all().await.map_err(|e| {
            ContextraError::Storage(format!("ErasureLedger sync_all failed: {}", e))
        })?;

        crate::util::fsync_parent_dir(&ledger_path).await?;

        // Update state
        self.last_chain_mac = current_mac;
        for entry in to_write {
            self.seen_proof_ids.insert(entry.proof_id);
            if let LedgerEntryKind::Prefix { prefix_len } = entry.kind {
                self.distinct_prefix_lens.insert(prefix_len);
            }
            self.entries.push(entry);
        }

        self.write_head_marker().await?;

        Ok(())
    }

    /// Decides whether a key version at transaction `tx` should be kept or dropped.
    ///
    /// NOTE: This method performs cryptographic hashing for key and distinct prefix lengths.
    /// It MUST NOT be invoked on the latency-sensitive read hot-path (only during WAL replay,
    /// compaction, recovery, or tombstone audits).
    pub fn decide(&self, storage_scope: &[u8], key: &[u8], tx: u64) -> LedgerDecision {
        let key_hash = compute_keyed_hash(&self.key, storage_scope, key);

        // Compute prefix hashes once per distinct prefix length present in the ledger
        let mut prefix_hashes = std::collections::HashMap::new();
        for &len in &self.distinct_prefix_lens {
            let usize_len = len as usize;
            if key.len() >= usize_len {
                let prefix_bytes = &key[..usize_len];
                let prefix_hash = compute_keyed_hash(&self.key, storage_scope, prefix_bytes);
                prefix_hashes.insert(len, prefix_hash);
            }
        }

        for entry in &self.entries {
            match entry.kind {
                LedgerEntryKind::Key => {
                    if entry.keyed_hash == key_hash && tx <= entry.erased_at_tx {
                        return LedgerDecision::Drop;
                    }
                }
                LedgerEntryKind::Prefix { prefix_len } => {
                    if let Some(prefix_hash) = prefix_hashes.get(&prefix_len) {
                        if entry.keyed_hash == *prefix_hash && tx <= entry.erased_at_tx {
                            return LedgerDecision::Drop;
                        }
                    }
                }
            }
        }

        LedgerDecision::Keep
    }

    /// Bulk decide for multiple keys.
    pub fn decide_batch(
        &self,
        storage_scope: &[u8],
        items: &[(&[u8], u64)],
    ) -> Vec<LedgerDecision> {
        items
            .iter()
            .map(|(key, tx)| self.decide(storage_scope, key, *tx))
            .collect()
    }

    pub fn last_chain_mac(&self) -> &[u8; 32] {
        &self.last_chain_mac
    }

    pub fn db_instance_id(&self) -> &[u8; 16] {
        &self.db_instance_id
    }

    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }

    async fn write_head_marker(&self) -> Result<()> {
        let head_path = self.dir.join(HEAD_FILENAME);
        let mut marker_bytes = Vec::with_capacity(48);
        marker_bytes.extend_from_slice(b"ELDG"); // Magic
        marker_bytes.extend_from_slice(&self.db_instance_id);
        marker_bytes.extend_from_slice(&self.last_chain_mac);

        contextra_durable_fs::atomic_replace(&head_path, &marker_bytes)?;
        Ok(())
    }

    async fn read_head_marker(path: &Path) -> Result<([u8; 16], [u8; 32])> {
        let bytes = tokio::fs::read(path)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to read head marker: {}", e)))?;

        if bytes.len() != 4 + 16 + 32 || &bytes[0..4] != b"ELDG" {
            return Err(ContextraError::Storage("Corrupt head marker".into()));
        }

        let mut db_id = [0u8; 16];
        db_id.copy_from_slice(&bytes[4..20]);

        let mut head_mac = [0u8; 32];
        head_mac.copy_from_slice(&bytes[20..52]);

        Ok((db_id, head_mac))
    }
}
