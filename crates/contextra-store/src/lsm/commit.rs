use super::observer::WriteOrigin;
use super::*;
use crate::wal::WalEntry;
use contextra_core::{TxId, TOMBSTONE_BIT};
use contextra_mvcc::tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES;

impl LsmStorage {
    /// Notifies registered observers of committed WAL entries for a transaction.
    pub(super) fn notify_commit_observers(
        &self,
        entries: &[WalEntry],
        tx_id: TxId,
        origin: WriteOrigin,
        durable: bool,
    ) {
        let seq_no = entries.last().map(|e| e.seq_no).unwrap_or(0);
        let ctx = CommitContext {
            tx_id,
            origin,
            durable,
        };
        self.observer_registry
            .notify_with_context(entries, seq_no, ctx);
    }

    /// Clears intent locks matching a predicate, gracefully recovering from poisoned locks.
    pub fn cleanup_intent_locks_where<F>(&self, mut predicate: F)
    where
        F: FnMut(TxId) -> bool,
    {
        let mut locks = self.intent_locks.lock().unwrap_or_else(|e| e.into_inner());
        locks.retain(|_, locked_tx| !predicate(*locked_tx));
    }

    pub fn clear_intent_locks_for_tx(&self, tx_id: TxId) {
        self.cleanup_intent_locks_where(|t| t == tx_id);
    }

    /// Removes all intent lock registrations associated with transactions strictly newer than target_tx.
    pub fn clear_intent_locks_above_tx(&self, target_tx: TxId) {
        self.cleanup_intent_locks_where(|t| t > target_tx);
    }

    pub(super) fn cleanup_intent_locks_for_tx(&self, tx_id: TxId) {
        self.cleanup_intent_locks_where(|t| t == tx_id);
    }

    pub(super) fn cleanup_intent_locks_for_txs(&self, tx_ids: &[TxId]) {
        self.cleanup_intent_locks_where(|t| tx_ids.contains(&t));
    }

    /// Suspends execution briefly if memory usage exceeds 80% to apply backpressure.
    pub(super) async fn apply_backpressure(&self) {
        if self.budget.memory_used()
            >= (self.config.max_ram_mb as f64 * 1024.0 * 1024.0 * 0.80) as u64
        {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    }

    /// Advances the MVCC visible transaction horizon (`last_committed_tx`) atomically.
    /// Internal transactions (`>= TxId::INTERNAL_BASE`) are ignored as they are never MVCC-visible.
    /// `tx_id == 0` is ignored with a warning to prevent MVCC blackout.
    #[inline]
    pub(super) fn advance_visibility(&self, tx_id: TxId) {
        if tx_id.inner() == 0 {
            tracing::debug!(
                "LsmStorage::commit tx=0 called — ignoring visibility update to prevent blackout"
            );
            return;
        }
        if tx_id.inner() < TxId::INTERNAL_BASE {
            let mut current = self.last_committed_tx.load(Ordering::Acquire);
            while tx_id.inner() > current {
                match self.last_committed_tx.compare_exchange_weak(
                    current,
                    tx_id.inner(),
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(actual) => current = actual,
                }
            }
        }
    }

    /// Applies memory updates to the provided `MemTable` and updates budget tracking.
    ///
    /// # Lock Invariants
    /// The caller holds a read guard (`state.read()`) on `LsmState`. This is intentional and symmetrical across
    /// single-commit and group-commit paths:
    /// - `state.read()` prevents `MemTable` pointer replacement during concurrent `flush()` operations while allowing
    ///   concurrent readers.
    /// - `MemTable` internally uses `parking_lot::RwLock` for thread-safe concurrent mutations during `put()`.
    /// - `state.write()` is strictly reserved for atomic `MemTable` and `WAL` rotation in `flush()`.
    ///
    /// INVARIANTE: LOCK-REIHENFOLGE: commit_mutex → state.read → MemTable-RwLock.
    pub(super) fn apply_mem_updates(
        &self,
        memtable: &MemTable,
        mem_updates: &[(Vec<u8>, Vec<u8>, u64)],
        tx_id: TxId,
    ) {
        let mut max_seq = 0u64;
        for (key, value, seq) in mem_updates {
            let raw_seq = seq & !TOMBSTONE_BIT;
            if raw_seq > max_seq {
                max_seq = raw_seq;
            }
            let entry_size = key.len() + value.len() + STAGING_ENTRY_OVERHEAD_BYTES;
            if let Err(e) = self.budget.consume_memory(entry_size as u64) {
                self.budget_tracking_drift_bytes
                    .fetch_add(entry_size as u64, std::sync::atomic::Ordering::Relaxed);
                tracing::warn!(
                    drift_bytes = entry_size,
                    total_drift_bytes = self
                        .budget_tracking_drift_bytes
                        .load(std::sync::atomic::Ordering::Relaxed),
                    "Memory budget tracking warning during commit: {e}"
                );
            }
            memtable.put(
                Bytes::from(key.clone()),
                Bytes::from(value.clone()),
                *seq,
                tx_id.inner(),
            );
            self.compaction_engine.record_write_op(*seq);
        }
        if max_seq > 0 {
            let mut current = self.last_applied_seq.load(Ordering::Acquire);
            while max_seq > current {
                match self.last_applied_seq.compare_exchange_weak(
                    current,
                    max_seq,
                    Ordering::Release,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(actual) => current = actual,
                }
            }
        }
    }
}
