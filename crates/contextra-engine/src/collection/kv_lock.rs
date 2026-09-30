use tokio::sync::{Mutex, MutexGuard};

const KV_LOCK_SHARDS: usize = 16;

// INV-HASHER-SEEDS: niemals randomisieren
const SEED_A: u64 = 0x9E37_79B9_7F4A_7C15;
const SEED_B: u64 = 0xBF58_476D_1CE4_E5B9;
const SEED_C: u64 = 0x94D0_49BB_1331_11EB;
const SEED_D: u64 = 0x2545_F491_4F6C_DD1D;

/// RAII guard holding a lock for a key shard.
pub(crate) struct KeyGuard<'a> {
    _guard: MutexGuard<'a, ()>,
}

pub(crate) struct KvKeyLocks {
    shards: [Mutex<()>; KV_LOCK_SHARDS],
    hasher: ahash::RandomState,
}

impl KvKeyLocks {
    pub fn new() -> Self {
        // INV-HASHER-SEEDS: niemals randomisieren
        let hasher = ahash::RandomState::with_seeds(SEED_A, SEED_B, SEED_C, SEED_D);
        Self {
            shards: Default::default(),
            hasher,
        }
    }

    pub fn shard_idx(&self, key: &str) -> usize {
        (self.hasher.hash_one(key) as usize) % KV_LOCK_SHARDS
    }

    pub async fn lock_shard<'a>(&'a self, idx: usize) -> KeyGuard<'a> {
        let guard = self.shards[idx].lock().await;
        KeyGuard { _guard: guard }
    }

    #[cfg(test)]
    pub fn try_lock_shard<'a>(&'a self, idx: usize) -> Option<KeyGuard<'a>> {
        self.shards[idx].try_lock().ok().map(|guard| KeyGuard { _guard: guard })
    }

    pub async fn lock_for<'a>(&'a self, key: &str) -> KeyGuard<'a> {
        let idx = self.shard_idx(key);
        self.lock_shard(idx).await
    }

    pub fn try_lock_shard<'a>(&'a self, idx: usize) -> Option<KeyGuard<'a>> {
        self.shards[idx].try_lock().ok().map(|guard| KeyGuard { _guard: guard })
    }

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

/// Pure, synchronous function to map string keys to unique shard indices in ascending order.
///
/// Deduplicates shard indices to prevent self-deadlock when multiple keys map to the same shard
/// and sorts indices in ascending order to prevent cross-operation deadlocks.
pub fn sorted_unique_shards<'a>(
    keys: impl IntoIterator<Item = &'a str>,
    shard_count: usize,
    hasher: &ahash::RandomState,
) -> Vec<usize> {
    if shard_count == 0 {
        return Vec::new();
    }
    let mut shard_indices: Vec<usize> = keys
        .into_iter()
        .map(|k| (hasher.hash_one(k) as usize) % shard_count)
        .collect();
    shard_indices.sort_unstable();
    shard_indices.dedup();
    shard_indices
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
            keys in prop::collection::vec(".*", 0..100),
            shard_count in 1..64usize,
        ) {
            let hasher = ahash::RandomState::with_seeds(SEED_A, SEED_B, SEED_C, SEED_D);
            let str_keys: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();

            let shards = sorted_unique_shards(str_keys.iter().copied(), shard_count, &hasher);

            // 1. Length <= shard_count
            prop_assert!(shards.len() <= shard_count);

            // 2. Strictly ascending (and thus no duplicates)
            for window in shards.windows(2) {
                prop_assert!(window[0] < window[1]);
            }

            // 3. Order independent: permuting input keys yields identical output
            let mut permuted_keys = str_keys.clone();
            permuted_keys.reverse();
            let permuted_shards = sorted_unique_shards(permuted_keys.iter().copied(), shard_count, &hasher);
            prop_assert_eq!(shards, permuted_shards);
        }
    }
}

#[allow(unexpected_cfgs)]
#[cfg(loom)]
mod loom_tests {
    use super::*;
    use loom::sync::Arc;
    use loom::sync::Mutex;
    use loom::thread;

    struct LoomKvKeyLocks {
        shards: Vec<Mutex<()>>,
        shard_count: usize,
        hasher: ahash::RandomState,
    }

    impl LoomKvKeyLocks {
        fn new(shard_count: usize) -> Self {
            let hasher = ahash::RandomState::with_seeds(SEED_A, SEED_B, SEED_C, SEED_D);
            let shards = (0..shard_count).map(|_| Mutex::new(())).collect();
            Self {
                shards,
                shard_count,
                hasher,
            }
        }

        fn acquire_multi_sorted(&self, keys: &[&str]) -> Vec<loom::sync::MutexGuard<'_, ()>> {
            let shard_indices =
                sorted_unique_shards(keys.iter().copied(), self.shard_count, &self.hasher);
            let mut guards = Vec::with_capacity(shard_indices.len());
            for idx in shard_indices {
                guards.push(self.shards[idx].lock().unwrap());
            }
            guards
        }
    }

    #[test]
    fn test_loom_multi_key_lock_deadlock_freedom() {
        loom::model(|| {
            let locks = Arc::new(LoomKvKeyLocks::new(4));

            let locks_t1 = Arc::clone(&locks);
            let t1 = thread::spawn(move || {
                let _guards = locks_t1.acquire_multi_sorted(&["key_alpha", "key_beta"]);
            });

            let locks_t2 = Arc::clone(&locks);
            let t2 = thread::spawn(move || {
                let _guards = locks_t2.acquire_multi_sorted(&["key_beta", "key_alpha"]);
            });

            t1.join().expect("thread 1 completes");
            t2.join().expect("thread 2 completes");
        });
    }
}

#[cfg(not(loom))]
#[cfg(test)]
mod non_loom_tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_multi_key_lock_deadlock_freedom_non_loom() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let locks = Arc::new(KvKeyLocks::new());

            let locks_t1 = Arc::clone(&locks);
            let t1 = tokio::spawn(async move {
                let _guards = locks_t1.lock_many_sorted(["key_alpha", "key_beta"]).await;
            });

            let locks_t2 = Arc::clone(&locks);
            let t2 = tokio::spawn(async move {
                let _guards = locks_t2.lock_many_sorted(["key_beta", "key_alpha"]).await;
            });

            t1.await.unwrap();
            t2.await.unwrap();
        });
    }
}
