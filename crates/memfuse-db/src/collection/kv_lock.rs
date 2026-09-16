use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tokio::sync::{Mutex, MutexGuard};

const KV_LOCK_SHARDS: usize = 16;

pub(crate) struct KvKeyLocks {
    shards: [Mutex<()>; KV_LOCK_SHARDS],
}

impl KvKeyLocks {
    pub const fn new() -> Self {
        Self {
            shards: [
                Mutex::const_new(()),
                Mutex::const_new(()),
                Mutex::const_new(()),
                Mutex::const_new(()),
                Mutex::const_new(()),
                Mutex::const_new(()),
                Mutex::const_new(()),
                Mutex::const_new(()),
                Mutex::const_new(()),
                Mutex::const_new(()),
                Mutex::const_new(()),
                Mutex::const_new(()),
                Mutex::const_new(()),
                Mutex::const_new(()),
                Mutex::const_new(()),
                Mutex::const_new(()),
            ],
        }
    }

    pub(crate) fn shard_idx(key: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % KV_LOCK_SHARDS
    }

    pub async fn lock_for<'a>(&'a self, key: &str) -> MutexGuard<'a, ()> {
        let idx = Self::shard_idx(key);
        self.shards[idx].lock().await
    }

    pub async fn lock_for_keys<'a>(
        &'a self,
        keys: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Vec<MutexGuard<'a, ()>> {
        let mut shard_indices: Vec<usize> =
            keys.into_iter().map(|k| Self::shard_idx(k.as_ref())).collect();
        shard_indices.sort_unstable();
        shard_indices.dedup();

        let mut guards = Vec::with_capacity(shard_indices.len());
        for idx in shard_indices {
            guards.push(self.shards[idx].lock().await);
        }
        guards
    }

    pub async fn lock_all<'a>(&'a self) -> Vec<MutexGuard<'a, ()>> {
        let mut guards = Vec::with_capacity(KV_LOCK_SHARDS);
        for idx in 0..KV_LOCK_SHARDS {
            guards.push(self.shards[idx].lock().await);
        }
        guards
    }
}
