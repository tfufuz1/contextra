//! Key-granular multi-key locking module (`kv_locks.rs`).
//!
//! Provides `KvKeyLocks` for fine-grained key locking across $2^N$ shards with canonical
//! lock acquisition ordering (`acquire_multi_sorted`) to prevent multi-key deadlocks.

use std::sync::{RwLock, RwLockWriteGuard};
use thiserror::Error;

/// Error types for key locking operations.
#[derive(Debug, Error)]
pub enum LockError {
    /// Returned when an underlying shard lock is poisoned.
    #[error("lock poisoned")]
    Poisoned,
    /// Future-proof variant for timeout-capable lock acquisition wrappers.
    /// Currently unused as standard `RwLock` is blocking without timeout.
    #[error("lock acquisition timed out")]
    Timeout,
}

/// Key-granular multi-key lock manager using $2^N$ sharded `RwLock` primitives.
pub struct KvKeyLocks {
    shards: Vec<RwLock<()>>,
    shard_mask: u64,
}

/// RAII guard holding a write lock for a single key shard.
pub struct KeyGuard<'a> {
    _guard: RwLockWriteGuard<'a, ()>,
}

/// RAII guard holding write locks across multiple key shards in ascending shard order.
pub struct MultiKeyGuard<'a> {
    _guards: Vec<RwLockWriteGuard<'a, ()>>,
}

impl KvKeyLocks {
    /// Constructs a new `KvKeyLocks` instance with $2^{\text{shard\_count\_pow2}}$ shards.
    pub fn new(shard_count_pow2: u32) -> Self {
        let pow2 = shard_count_pow2.min(16);
        let num_shards = 1usize << pow2;
        let shard_mask = (num_shards as u64) - 1;
        let shards = (0..num_shards).map(|_| RwLock::new(())).collect();
        Self { shards, shard_mask }
    }

    /// Acquires a write lock on the shard corresponding to `key_hash`.
    pub fn acquire(&self, key_hash: u64) -> KeyGuard<'_> {
        let shard_idx = (key_hash & self.shard_mask) as usize;
        let _guard = match self.shards[shard_idx].write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        KeyGuard { _guard }
    }

    /// Acquires write locks for multiple key hashes strictly in ascending shard index order.
    ///
    /// Duplicate shard indices resulting from key collisions or identical hashes are automatically
    /// deduplicated to prevent self-deadlock.
    pub fn acquire_multi_sorted(
        &self,
        sorted_key_hashes: &[u64],
    ) -> Result<MultiKeyGuard<'_>, LockError> {
        if sorted_key_hashes.is_empty() {
            return Ok(MultiKeyGuard {
                _guards: Vec::new(),
            });
        }

        let mut shard_indices: Vec<usize> = sorted_key_hashes
            .iter()
            .map(|&hash| (hash & self.shard_mask) as usize)
            .collect();

        shard_indices.sort_unstable();
        shard_indices.dedup();

        let mut _guards = Vec::with_capacity(shard_indices.len());
        for idx in shard_indices {
            let guard = self.shards[idx].write().map_err(|_| LockError::Poisoned)?;
            _guards.push(guard);
        }

        Ok(MultiKeyGuard { _guards })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_single_key_blocking() {
        let locks = Arc::new(KvKeyLocks::new(4));
        let key_hash = 42u64;

        let guard = locks.acquire(key_hash);

        let (tx, rx) = mpsc::channel();
        let locks_clone = Arc::clone(&locks);

        thread::spawn(move || {
            let shard_idx = (key_hash & locks_clone.shard_mask) as usize;
            let try_res = locks_clone.shards[shard_idx].try_write();
            let is_blocked = try_res.is_err();
            tx.send(is_blocked).unwrap();
        });

        let is_blocked = rx.recv().unwrap();
        assert!(
            is_blocked,
            "Second acquire on same key must be blocked while guard is held"
        );

        drop(guard);

        let shard_idx = (key_hash & locks.shard_mask) as usize;
        assert!(
            locks.shards[shard_idx].try_write().is_ok(),
            "After dropping guard, shard lock should be available"
        );
    }

    #[test]
    fn test_multi_sorted_canonical_order() {
        let locks = KvKeyLocks::new(4);
        // Pass key hashes in non-sorted order
        let keys_a = vec![100u64, 5u64, 25u64];
        let keys_b = vec![25u64, 5u64, 100u64];

        let guard_a = locks.acquire_multi_sorted(&keys_a).unwrap();
        assert_eq!(guard_a._guards.len(), 3);
        drop(guard_a);

        let guard_b = locks.acquire_multi_sorted(&keys_b).unwrap();
        assert_eq!(guard_b._guards.len(), 3);
        drop(guard_b);
    }

    #[test]
    fn test_multi_sorted_dedup() {
        let locks = KvKeyLocks::new(4);
        // Duplicate key hashes that map to the same shard
        let keys_with_dups = vec![42u64, 42u64, 10u64, 42u64, 10u64];

        let guard = locks.acquire_multi_sorted(&keys_with_dups).unwrap();
        // Since 42 and 10 map to distinct or same shards, deduplicated count must be <= 2
        assert!(guard._guards.len() <= 2);
    }
}
