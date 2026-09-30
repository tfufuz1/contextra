#![allow(unexpected_cfgs)]

use ahash::RandomState;

const KV_LOCK_SHARDS: usize = 16;

// INV-HASHER-SEEDS: niemals randomisieren
const SEED_A: u64 = 0x9E37_79B9_7F4A_7C15;
const SEED_B: u64 = 0xBF58_476D_1CE4_E5B9;
const SEED_C: u64 = 0x94D0_49BB_1331_11EB;
const SEED_D: u64 = 0x2545_F491_4F6C_DD1D;

/// Pure, synchronous helper that maps string keys to deduplicated, strictly ascending shard indices.
///
/// # Invariants
/// - Output vector is strictly ascending (`out[i] < out[i+1]`).
/// - Output vector contains no duplicates (`dedup`).
/// - Vector length is bounded by `shard_count` (`len <= shard_count`).
/// - Result is invariant under permutations of input keys.
pub fn sorted_unique_shards<'a>(
    keys: impl IntoIterator<Item = &'a str>,
    shard_count: usize,
    hasher: &RandomState,
) -> Vec<usize> {
    if shard_count == 0 {
        return Vec::new();
    }
    let mut indices: Vec<usize> = keys
        .into_iter()
        .map(|k| (hasher.hash_one(k) as usize) % shard_count)
        .collect();
    indices.sort_unstable();
    indices.dedup();
    indices
}

#[cfg(not(loom))]
use tokio::sync::{Mutex, MutexGuard};

#[cfg(loom)]
use loom::sync::{Mutex, MutexGuard};

/// RAII guard holding a lock for a key shard.
pub(crate) struct KeyGuard<'a> {
    _guard: MutexGuard<'a, ()>,
}

pub(crate) struct KvKeyLocks {
    shards: [Mutex<()>; KV_LOCK_SHARDS],
    hasher: RandomState,
}

impl KvKeyLocks {
    pub fn new() -> Self {
        // INV-HASHER-SEEDS: niemals randomisieren
        let hasher = RandomState::with_seeds(SEED_A, SEED_B, SEED_C, SEED_D);
        Self {
            shards: Default::default(),
            hasher,
        }
    }

    pub fn shard_idx(&self, key: &str) -> usize {
        (self.hasher.hash_one(key) as usize) % KV_LOCK_SHARDS
    }

    pub async fn lock_shard<'a>(&'a self, idx: usize) -> KeyGuard<'a> {
        #[cfg(not(loom))]
        let guard = self.shards[idx].lock().await;

        #[cfg(loom)]
        let guard = self.shards[idx].lock().unwrap();

        KeyGuard { _guard: guard }
    }

    pub async fn lock_for<'a>(&'a self, key: &str) -> KeyGuard<'a> {
        let idx = self.shard_idx(key);
        self.lock_shard(idx).await
    }

    #[cfg(not(loom))]
    pub fn try_lock_shard<'a>(&'a self, idx: usize) -> Option<KeyGuard<'a>> {
        self.shards[idx]
            .try_lock()
            .ok()
            .map(|guard| KeyGuard { _guard: guard })
    }

    #[cfg(not(loom))]
    pub fn try_lock_for<'a>(&'a self, key: &str) -> Option<KeyGuard<'a>> {
        let idx = self.shard_idx(key);
        self.try_lock_shard(idx)
    }

    /// Locks multiple shards in ascending shard index order to prevent deadlocks across operations.
    /// Deduplicates by lock shard index to prevent self-deadlock when multiple keys map to the same shard.
    pub async fn lock_many_sorted<'a>(
        &'a self,
        keys: impl IntoIterator<Item = &'a str>,
    ) -> Vec<KeyGuard<'a>> {
        let shard_indices = sorted_unique_shards(keys, KV_LOCK_SHARDS, &self.hasher);
        let mut guards = Vec::with_capacity(shard_indices.len());
        for idx in shard_indices {
            guards.push(self.lock_shard(idx).await);
        }
        guards
    }
}

impl Default for KvKeyLocks {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_sorted_unique_shards_properties(
            keys in prop::collection::vec(".*", 0..50),
            shard_count in 1usize..64,
        ) {
            let hasher = RandomState::with_seeds(SEED_A, SEED_B, SEED_C, SEED_D);
            let key_refs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();

            let shards = sorted_unique_shards(key_refs.clone(), shard_count, &hasher);

            // 1. Bound check
            prop_assert!(shards.len() <= shard_count);

            // 2. Strict ascending order and uniqueness
            for window in shards.windows(2) {
                prop_assert!(window[0] < window[1]);
            }

            // 3. Permutation independence
            let mut permuted_keys = key_refs.clone();
            permuted_keys.reverse();
            let permuted_shards = sorted_unique_shards(permuted_keys, shard_count, &hasher);
            prop_assert_eq!(shards, permuted_shards);
        }
    }
}

#[cfg(loom)]
pub mod loom_tests {
    use super::*;
    use loom::sync::Arc;
    use loom::thread;

    #[test]
    fn test_loom_multi_key_lock_deadlock_freedom() {
        loom::model(|| {
            let locks = Arc::new(KvKeyLocks::new());

            let locks_t1 = Arc::clone(&locks);
            let t1 = thread::spawn(move || {
                let mut rt = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap();
                rt.block_on(async {
                    let _guards = locks_t1.lock_many_sorted(&["key_alpha", "key_beta"]).await;
                });
            });

            let locks_t2 = Arc::clone(&locks);
            let t2 = thread::spawn(move || {
                let mut rt = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap();
                rt.block_on(async {
                    let _guards = locks_t2.lock_many_sorted(&["key_beta", "key_alpha"]).await;
                });
            });

            t1.join().expect("thread 1 completes");
            t2.join().expect("thread 2 completes");
        });
    }
}
