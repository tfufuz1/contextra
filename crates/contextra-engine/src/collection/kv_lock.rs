use std::sync::{RwLock, RwLockWriteGuard};

const KV_LOCK_SHARDS: usize = 16;

// INV-HASHER-SEEDS: niemals randomisieren
const SEED_A: u64 = 0x9E37_79B9_7F4A_7C15;
const SEED_B: u64 = 0xBF58_476D_1CE4_E5B9;
const SEED_C: u64 = 0x94D0_49BB_1331_11EB;
const SEED_D: u64 = 0x2545_F491_4F6C_DD1D;

/// RAII guard holding a write lock for a key shard.
pub(crate) struct KeyGuard<'a> {
    _guard: RwLockWriteGuard<'a, ()>,
}

pub(crate) struct KvKeyLocks {
    shards: [RwLock<()>; KV_LOCK_SHARDS],
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

    pub fn lock_shard<'a>(&'a self, idx: usize) -> KeyGuard<'a> {
        let guard = match self.shards[idx].write() {
            Ok(g) => g,
            Err(poisoned) => {
                tracing::error!(
                    shard_idx = idx,
                    "KvKeyLocks shard RwLock was poisoned due to a thread panic; recovering write lock via into_inner()"
                );
                poisoned.into_inner()
            }
        };
        KeyGuard { _guard: guard }
    }

    pub async fn lock_for<'a>(&'a self, key: &str) -> KeyGuard<'a> {
        let idx = self.shard_idx(key);
        self.lock_shard(idx)
    }

    #[allow(dead_code)]
    pub fn try_lock_shard<'a>(&'a self, idx: usize) -> Option<KeyGuard<'a>> {
        match self.shards[idx].try_write() {
            Ok(g) => Some(KeyGuard { _guard: g }),
            Err(std::sync::TryLockError::Poisoned(poisoned)) => {
                tracing::error!(
                    shard_idx = idx,
                    "KvKeyLocks shard RwLock was poisoned due to a thread panic; recovering write lock via into_inner()"
                );
                Some(KeyGuard {
                    _guard: poisoned.into_inner(),
                })
            }
            Err(std::sync::TryLockError::WouldBlock) => None,
        }
    }

    #[allow(dead_code)]
    pub fn try_lock_for<'a>(&'a self, key: &str) -> Option<KeyGuard<'a>> {
        let idx = self.shard_idx(key);
        self.try_lock_shard(idx)
    }

    #[allow(dead_code)]
    pub fn is_shard_poisoned(&self, idx: usize) -> bool {
        self.shards[idx].is_poisoned()
    }
}

impl Default for KvKeyLocks {
    fn default() -> Self {
        Self::new()
    }
}
