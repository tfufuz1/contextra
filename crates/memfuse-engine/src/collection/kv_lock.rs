use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tokio::sync::Mutex;

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

    pub fn shard_idx(&self, key: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % KV_LOCK_SHARDS
    }

    pub async fn lock_shard<'a>(&'a self, idx: usize) -> tokio::sync::MutexGuard<'a, ()> {
        self.shards[idx].lock().await
    }

    pub async fn lock_for<'a>(&'a self, key: &str) -> tokio::sync::MutexGuard<'a, ()> {
        let idx = self.shard_idx(key);
        self.lock_shard(idx).await
    }
}
