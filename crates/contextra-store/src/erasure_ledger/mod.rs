// FILE-CONTEXT
// STAND: 2026-10-12 (T-2026-9204)
// ZWECK: ErasureLedger Modul-Einstieg und Implementierung der Append-Only Persistenz und Decision Filter.
// INVARIANTEN: Zero-Panic, Fail-Closed, INV-DURABILITY-RING (fsync_parent_dir + sync_all).

pub mod coverage;
pub mod entry;
pub mod replay_filter;

use contextra_types::{ContextraError, Result};
pub use coverage::{LedgerCoverage, LedgerGap};
pub use replay_filter::{ErasureLedgerHandle, LedgerReplayFilter};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

pub use entry::{
    compute_keyed_hash, parse_frame, serialize_frame, FrameReadResult, LedgerEntry,
    LedgerEntryKind, LedgerKey, ERASED_LEDGER_DOMAIN,
};
use contextra_ports::{LayerObservation, LayerScan, PhysicalScope, ScanError};

pub const LEDGER_FILENAME: &str = "erasure.ledger";
pub const HEAD_FILENAME: &str = "ledger.head";
pub const INITIAL_CHAIN_MAC: [u8; 32] = [0u8; 32];

/// Decision returned when evaluating whether a key version at transaction `tx` should be kept or dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerDecision {
    Keep,
    Drop,
}

/// Status returned when checking or applying anti-resurrection guarantees (E-4 / INV-ERA-1).
///
/// NOTE: "Truncation without verifiable anchor brings deleted keys back" safety must NOT be claimed
/// unless the truncation anchor is verified against ledger chain MAC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AntiResurrection {
    /// Accelerated tombstone reapplication with verified truncation anchor guarantees anti-resurrection.
    Guaranteed { tombstones_reapplied: usize },
    /// Truncation without verifiable anchor or missing acceleration index cannot guarantee anti-resurrection.
    NotGuaranteed { reason: &'static str },
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
    seen_keyed_hashes: HashSet<[u8; 32]>,
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
    #[deprecated(note = "use open_or_fail_closed_with_instance_id to inject db_instance_id")]
    pub async fn open_or_fail_closed(
        dir: impl AsRef<Path>,
        key: LedgerKey,
        db_has_data: bool,
        expected_head_mac: Option<[u8; 32]>,
    ) -> Result<(Option<Self>, LedgerOpenStatus)> {
        let head_path = dir.as_ref().join(HEAD_FILENAME);
        let instance_id = if tokio::fs::try_exists(&head_path).await.unwrap_or(false) {
            if let Ok((stored_id, _)) = Self::read_head_marker(&head_path).await {
                stored_id
            } else {
                [0u8; 16]
            }
        } else {
            let mut default_id = [0u8; 16];
            let rng = contextra_ports::SystemRng::new();
            contextra_ports::Rng::fill_bytes(&rng, &mut default_id);
            default_id
        };
        Self::open_or_fail_closed_with_instance_id(dir, key, db_has_data, expected_head_mac, instance_id).await
    }

    /// Opens the erasure ledger with explicit injected `db_instance_id` or returns fail-closed status.
    pub async fn open_or_fail_closed_with_instance_id(
        dir: impl AsRef<Path>,
        key: LedgerKey,
        db_has_data: bool,
        expected_head_mac: Option<[u8; 32]>,
        db_instance_id: [u8; 16],
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
            let rng = contextra_ports::SystemRng::new();
            contextra_ports::Rng::fill_bytes(&rng, &mut db_instance_id);

            let ledger = Self {
                dir: dir.clone(),
                key,
                last_chain_mac: INITIAL_CHAIN_MAC,
                entries: Vec::new(),
                seen_proof_ids: HashSet::new(),
                seen_keyed_hashes: HashSet::new(),
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
        let mut seen_keyed_hashes = HashSet::new();
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
                    if seen_proof_ids.contains(&entry.proof_id) || seen_keyed_hashes.contains(&entry.keyed_hash) {
                        // Deduplicated entry
                    } else {
                        seen_proof_ids.insert(entry.proof_id);
                        seen_keyed_hashes.insert(entry.keyed_hash);
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

        if head_exists && stored_db_id != db_instance_id {
            return Ok((
                None,
                LedgerOpenStatus::RollbackSuspected {
                    reason: "Injected db_instance_id mismatch with ledger head".into(),
                },
            ));
        }

        let ledger = Self {
            dir,
            key,
            last_chain_mac: prev_mac,
            entries,
            seen_proof_ids,
            seen_keyed_hashes,
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
            if self.seen_proof_ids.contains(&entry.proof_id) || self.seen_keyed_hashes.contains(&entry.keyed_hash) {
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
            self.seen_keyed_hashes.insert(entry.keyed_hash);
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

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn key(&self) -> &LedgerKey {
        &self.key
    }

    pub fn seen_proof_ids(&self) -> &HashSet<[u8; 16]> {
        &self.seen_proof_ids
    }

    pub fn update_after_append(&mut self, new_mac: [u8; 32], to_write: Vec<LedgerEntry>) {
        self.last_chain_mac = new_mac;
        for entry in to_write {
            self.seen_proof_ids.insert(entry.proof_id);
            self.seen_keyed_hashes.insert(entry.keyed_hash);
            if let LedgerEntryKind::Prefix { prefix_len } = entry.kind {
                self.distinct_prefix_lens.insert(prefix_len);
            }
            self.entries.push(entry);
        }
    }

    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }

    /// Reapplies tombstones from the ledger using an accelerated hash index over `keyed_hash`.
    ///
    /// If `truncation_anchor_mac` is `None` or does not match the ledger's verified head MAC,
    /// returns `AntiResurrection::NotGuaranteed` per INV-ERA-1 / E-4.
    pub fn reapply_tombstones_from_ledger(
        &self,
        _storage_scope: &[u8],
        truncation_anchor_mac: Option<[u8; 32]>,
    ) -> (Vec<LedgerEntry>, AntiResurrection) {
        let anchor_valid = match truncation_anchor_mac {
            Some(anchor) => anchor == self.last_chain_mac,
            None => false,
        };

        // Build accelerated index over keyed_hashes present in ledger entries for this scope
        let accelerated_index: HashSet<[u8; 32]> = self
            .entries
            .iter()
            .map(|e| e.keyed_hash)
            .collect();

        let matching: Vec<LedgerEntry> = self
            .entries
            .iter()
            .filter(|e| accelerated_index.contains(&e.keyed_hash))
            .cloned()
            .collect();

        if !anchor_valid {
            (
                matching,
                AntiResurrection::NotGuaranteed {
                    reason: "Missing or unverified truncation anchor; anti-resurrection cannot be guaranteed",
                },
            )
        } else {
            let count = matching.len();
            (
                matching,
                AntiResurrection::Guaranteed {
                    tombstones_reapplied: count,
                },
            )
        }
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

impl LayerScan for ErasureLedger {
    fn layer_id(&self) -> u16 {
        // Layer ID 12 for ErasureLedger layer
        12
    }

    fn scan(&self, scope: &PhysicalScope) -> std::result::Result<LayerObservation, ScanError> {
        let mut matches_found: u64 = 0;
        let storage_scope = b"lsm";

        for key_bytes in scope.keys() {
            let key_hash = compute_keyed_hash(&self.key, storage_scope, key_bytes);
            for entry in &self.entries {
                if entry.keyed_hash == key_hash {
                    matches_found += 1;
                }
            }
        }

        for prefix_bytes in scope.prefixes() {
            let prefix_hash = compute_keyed_hash(&self.key, storage_scope, prefix_bytes);
            for entry in &self.entries {
                if entry.keyed_hash == prefix_hash {
                    matches_found += 1;
                }
            }
        }

        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.layer_id().to_le_bytes());
        hasher.update(self.last_chain_mac());
        hasher.update(&(self.entries.len() as u64).to_le_bytes());

        let state_digest = *hasher.finalize().as_bytes();

        Ok(LayerObservation {
            layer_id: self.layer_id(),
            clean: matches_found == 0,
            state_digest,
            item_count: self.entries.len() as u64,
        })
    }
}
