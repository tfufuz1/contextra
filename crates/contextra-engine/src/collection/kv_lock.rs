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

    pub fn try_lock_shard<'a>(&'a self, idx: usize) -> Option<KeyGuard<'a>> {
        self.shards[idx].try_lock().ok().map(|guard| KeyGuard { _guard: guard })
    }

    pub async fn lock_for<'a>(&'a self, key: &str) -> KeyGuard<'a> {
        let idx = self.shard_idx(key);
        self.lock_shard(idx).await
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
        let mut shard_indices: Vec<usize> = keys.into_iter().map(|k| self.shard_idx(k)).collect();
        shard_indices.sort_unstable();
        shard_indices.dedup();

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
