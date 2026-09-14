use super::*;
use memfuse_core::TxId;

impl LsmStorage {
    pub fn clear_intent_locks_for_tx(&self, tx_id: TxId) {
        if let Ok(mut locks) = self.intent_locks.lock() {
            locks.retain(|_, locked_tx| *locked_tx != tx_id);
        }
    }

    /// Removes all intent lock registrations associated with transactions strictly newer than target_tx.
    pub fn clear_intent_locks_above_tx(&self, target_tx: TxId) {
        if let Ok(mut locks) = self.intent_locks.lock() {
            locks.retain(|_, locked_tx| *locked_tx <= target_tx);
        }
    }

    /// Forces a flush (to be used by PersistentCheckpointStore or tests).

    pub(super) fn cleanup_intent_locks_for_tx(&self, tx_id: TxId) {
        let mut locks = self.intent_locks.lock().unwrap_or_else(|e| e.into_inner());
        locks.retain(|_, v| *v != tx_id);
    }

    pub(super) fn cleanup_intent_locks_for_txs(&self, tx_ids: &[TxId]) {
        let mut locks = self.intent_locks.lock().unwrap_or_else(|e| e.into_inner());
        locks.retain(|_, v| !tx_ids.contains(v));
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
            if tx_id.inner() == 0 {
                tracing::warn!(
                    "LsmStorage::commit tx=0 called — ignoring visibility update to prevent blackout"
                );
            }
        }
    }

    /// Applies memory updates to the provided `MemTable` and updates budget tracking.
    ///
    /// # Lock Invariants
    /// The caller must hold appropriate lock access (read or write guard on `LsmState`)
    /// guarding the `MemTable`. `MemTable` internally uses `parking_lot::RwLock` for safe
    /// concurrent mutations.
    pub(super) fn apply_mem_updates(
        &self,
        memtable: &MemTable,
        mem_updates: &[(Vec<u8>, Vec<u8>, u64)],
        tx_id: TxId,
    ) {
        for (key, value, seq) in mem_updates {
            let entry_size = key.len() + value.len() + 8;
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
        }
    }
}
