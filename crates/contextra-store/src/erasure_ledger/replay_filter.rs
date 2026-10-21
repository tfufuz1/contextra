//! FILE-CONTEXT
//! STAND: 2026-10-12 (T-2026-9204)
//! ZWECK: Thread-sicherer Handle für ErasureLedger und Implementierung von WalReplayFilter und CompactionRecordFilter.
//! INVARIANTEN: Zero-Panic, Fail-Closed, non-blocking sync decision evaluate für Hotpath/Replay.

use super::{
    fsync_ledger_dir, serialize_frame, AntiResurrection, ErasureLedger, LedgerDecision,
    LedgerEntry, HEAD_FILENAME, LEDGER_FILENAME,
};
use crate::compaction::purge_filter::CompactionRecordFilter;
use crate::wal::replay_filter::WalReplayFilter;
use contextra_types::{ContextraError, Result};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

/// Thread-safe handle wrapping [`ErasureLedger`] with sync read and async append locks.
#[derive(Debug)]
pub struct ErasureLedgerHandle {
    ledger: parking_lot::RwLock<ErasureLedger>,
    append_mutex: tokio::sync::Mutex<()>,
}

impl ErasureLedgerHandle {
    /// Constructs a new [`ErasureLedgerHandle`] wrapping the given ledger.
    pub fn new(ledger: ErasureLedger) -> Self {
        Self {
            ledger: parking_lot::RwLock::new(ledger),
            append_mutex: tokio::sync::Mutex::new(()),
        }
    }

    /// Appends entries to the erasure ledger with atomic fsync durability.
    ///
    /// Async I/O is serialized via `append_mutex`. No synchronous lock is held
    /// across `.await` points, ensuring worker threads calling `decide` are never blocked.
    pub async fn append(&self, entries: &[LedgerEntry]) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }

        let _guard = self.append_mutex.lock().await;

        let (dir, key, last_mac, db_instance_id, to_write) = {
            let guard = self.ledger.read();
            let mut batch_seen_proof_ids = std::collections::HashSet::new();
            let mut batch_seen_keyed_hashes = std::collections::HashSet::new();
            let mut to_write = Vec::new();
            for entry in entries {
                if guard.seen_proof_ids().contains(&entry.proof_id)
                    || guard.seen_keyed_hashes().contains(&entry.keyed_hash)
                    || batch_seen_proof_ids.contains(&entry.proof_id)
                    || batch_seen_keyed_hashes.contains(&entry.keyed_hash)
                {
                    continue;
                }
                batch_seen_proof_ids.insert(entry.proof_id);
                batch_seen_keyed_hashes.insert(entry.keyed_hash);
                to_write.push(entry.clone());
            }
            (
                guard.dir().to_path_buf(),
                guard.key().clone(),
                *guard.last_chain_mac(),
                *guard.db_instance_id(),
                to_write,
            )
        };

        if to_write.is_empty() {
            return Ok(());
        }

        let ledger_path = dir.join(LEDGER_FILENAME);
        let initial_len = match tokio::fs::metadata(&ledger_path).await {
            Ok(meta) => meta.len(),
            Err(_) => 0,
        };

        let mut buffer = Vec::new();
        let mut current_mac = last_mac;

        for entry in &to_write {
            let payload = entry.to_payload_bytes()?;
            let (frame_bytes, new_mac) = serialize_frame(&key, &current_mac, &payload)?;
            buffer.extend_from_slice(&frame_bytes);
            current_mac = new_mac;
        }

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&ledger_path)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to open ledger: {}", e)))?;

        if let Err(write_err) = file.write_all(&buffer).await {
            let _ = file.set_len(initial_len).await;
            return Err(ContextraError::Storage(format!("ErasureLedger write_all failed: {}", write_err)));
        }

        if let Err(sync_err) = file.sync_all().await {
            let _ = file.set_len(initial_len).await;
            return Err(ContextraError::Storage(format!("ErasureLedger sync_all failed: {}", sync_err)));
        }

        if let Err(fsync_err) = fsync_ledger_dir(&ledger_path).await {
            let _ = file.set_len(initial_len).await;
            let _ = file.sync_all().await;
            return Err(fsync_err);
        }

        // Write head marker atomically
        let head_path = dir.join(HEAD_FILENAME);
        let mut marker_bytes = Vec::with_capacity(48);
        marker_bytes.extend_from_slice(b"ELDG");
        marker_bytes.extend_from_slice(&db_instance_id);
        marker_bytes.extend_from_slice(&current_mac);
        contextra_durable_fs::atomic_replace(&head_path, &marker_bytes)?;

        // Update in-RAM state under sync write lock (held for microseconds, no .await inside)
        {
            let mut guard = self.ledger.write();
            guard.update_after_append(current_mac, to_write);
        }

        Ok(())
    }

    /// Decides whether a key version at transaction `tx` should be kept or dropped.
    pub fn decide(&self, storage_scope: &[u8], key: &[u8], tx: u64) -> LedgerDecision {
        self.ledger.read().decide(storage_scope, key, tx)
    }

    /// Returns the database instance ID associated with this ledger.
    pub fn db_instance_id(&self) -> [u8; 16] {
        *self.ledger.read().db_instance_id()
    }

    /// Returns the last chain MAC of the ledger log.
    pub fn last_chain_mac(&self) -> [u8; 32] {
        *self.ledger.read().last_chain_mac()
    }

    /// Returns a copy of all current ledger entries.
    pub fn entries(&self) -> Vec<LedgerEntry> {
        self.ledger.read().entries().to_vec()
    }

    /// Returns `true` if `keyed_hash` has been recorded in the erasure ledger.
    pub fn is_erased(&self, keyed_hash: &[u8; 32]) -> bool {
        self.ledger.read().is_erased(keyed_hash)
    }

    /// Returns `true` if a record with `keyed_hash` was erased at or after transaction `tx`.
    pub fn is_erased_at_tx(&self, keyed_hash: &[u8; 32], tx: u64) -> bool {
        self.ledger.read().is_erased_at_tx(keyed_hash, tx)
    }

    /// Enumerates all ledger entries recorded at or after transaction `min_tx`.
    pub fn entries_since_tx(&self, min_tx: u64) -> Vec<LedgerEntry> {
        self.ledger.read().entries_since_tx(min_tx)
    }

    /// Reapplies tombstones from the ledger with accelerated index.
    pub fn reapply_tombstones_from_ledger(
        &self,
        storage_scope: &[u8],
        truncation_anchor_mac: Option<[u8; 32]>,
    ) -> (Vec<LedgerEntry>, AntiResurrection) {
        self.ledger.read().reapply_tombstones_from_ledger(storage_scope, truncation_anchor_mac)
    }
}

/// Filter implementing both [`WalReplayFilter`] and [`CompactionRecordFilter`] via [`ErasureLedgerHandle`].
#[derive(Clone, Debug)]
pub struct LedgerReplayFilter {
    handle: Arc<ErasureLedgerHandle>,
}

impl LedgerReplayFilter {
    /// Constructs a new [`LedgerReplayFilter`] backed by the provided ledger handle.
    pub fn new(handle: Arc<ErasureLedgerHandle>) -> Self {
        Self { handle }
    }
}

impl WalReplayFilter for LedgerReplayFilter {
    fn drop_record(&self, key: &[u8], tx: u64) -> bool {
        self.handle.decide(b"lsm", key, tx) == LedgerDecision::Drop
    }
}

impl CompactionRecordFilter for LedgerReplayFilter {
    fn drop_record(&self, key: &[u8], tx: u64) -> bool {
        self.handle.decide(b"lsm", key, tx) == LedgerDecision::Drop
    }
}
