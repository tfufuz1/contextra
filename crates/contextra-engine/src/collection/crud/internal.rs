use crate::collection::Collection;
use contextra_core::{Result, StorageEngine, VectorIndex};

pub(crate) fn validate_doc_id(id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(contextra_core::ContextraError::invalid_input(
            "Document ID cannot be empty",
        ));
    }
    if id.len() > 256 {
        return Err(contextra_core::ContextraError::invalid_input(
            "Document ID exceeds maximum length of 256 bytes",
        ));
    }
    if id.contains('\0') {
        return Err(contextra_core::ContextraError::invalid_input(
            "Document ID cannot contain null bytes",
        ));
    }
    Ok(())
}

pub(crate) fn validate_embedding(embedding: &[f32]) -> Result<()> {
    if embedding.is_empty() {
        return Err(contextra_core::ContextraError::invalid_input(
            "Embedding vector cannot be empty",
        ));
    }
    if embedding.iter().all(|&x| x == 0.0) {
        return Err(contextra_core::ContextraError::invalid_input(
            "Zero vector embeddings are not allowed in regular Collection insertion. Use put_kv for non-vector entries.",
        ));
    }
    if embedding.iter().any(|&x| !x.is_finite()) {
        return Err(contextra_core::ContextraError::invalid_input(
            "Embedding vector contains NaN or Infinite values",
        ));
    }
    Ok(())
}

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Applies configured backpressure delay if system pressure level is Critical (INV-PRESSURE-1).
    pub(crate) async fn apply_insert_backpressure(&self) {
        let delay_ms = self.config.read().backpressure_delay_ms;
        if let Some(delay_ms) = delay_ms {
            let rx_opt = self.pressure_rx.read().clone();
            if let Some(ref rx) = rx_opt {
                if rx.borrow().pressure_level == contextra_store::PressureLevel::Critical {
                    let backpressure_delay = std::time::Duration::from_millis(delay_ms);
                    tracing::warn!(
                        "Insert backpressure active (Critical pressure level); delaying {}ms",
                        backpressure_delay.as_millis()
                    );
                    tokio::time::sleep(backpressure_delay).await;
                }
            }
        }
    }

    /// Locks keys in deterministic sorted order to prevent deadlocks across batch operations.
    /// Deduplicates by lock shard index to prevent self-deadlock when multiple keys map to the same shard.
    pub(crate) async fn lock_keys_sorted<'a>(
        &'a self,
        keys: impl IntoIterator<Item = &'a str>,
    ) -> Vec<tokio::sync::MutexGuard<'a, ()>> {
        let mut shard_indices: Vec<usize> = keys
            .into_iter()
            .map(|k| self.kv_locks.shard_idx(k))
            .collect();
        shard_indices.sort_unstable();
        shard_indices.dedup();
        let mut guards = Vec::with_capacity(shard_indices.len());
        for idx in shard_indices {
            guards.push(self.kv_locks.lock_shard(idx).await);
        }
        guards
    }
}
