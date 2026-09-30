//! Loom-basierter Nebenläufigkeitsbeweis für multi-key locking in `contextra-store`.
//! Ausführung: RUSTFLAGS="--cfg loom" cargo test -p contextra-store --test loom_multi_key_lock --release
//!
//! Beweist, dass `acquire_multi_sorted` mit überlappenden, unterschiedlich geordnet übergebenen
//! Schlüsselmengen zwei nebenläufige Aufrufe ohne Deadlock terminiert (kanonische Lock-Ordnung).

#![allow(unexpected_cfgs)]

#[cfg(loom)]
mod loom_tests {
    use loom::sync::Arc;
    use loom::sync::Mutex;
    use loom::thread;

    struct LoomKvKeyLocks {
        shards: Vec<Mutex<()>>,
        shard_mask: u64,
    }

    impl LoomKvKeyLocks {
        fn new(shard_count_pow2: u32) -> Self {
            let pow2 = shard_count_pow2.min(16);
            let num_shards = 1usize << pow2;
            let shard_mask = (num_shards as u64) - 1;
            let shards = (0..num_shards).map(|_| Mutex::new(())).collect();
            Self { shards, shard_mask }
        }

        fn acquire_multi_sorted(&self, key_hashes: &[u64]) -> Vec<loom::sync::MutexGuard<'_, ()>> {
            let mut shard_indices: Vec<usize> = key_hashes
                .iter()
                .map(|&hash| (hash & self.shard_mask) as usize)
                .collect();

            shard_indices.sort_unstable();
            shard_indices.dedup();

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
            let locks = Arc::new(LoomKvKeyLocks::new(2)); // 4 shards

            let locks_t1 = Arc::clone(&locks);
            let t1 = thread::spawn(move || {
                // Thread 1 receives keys in order [10, 1]
                let _guards = locks_t1.acquire_multi_sorted(&[10, 1]);
            });

            let locks_t2 = Arc::clone(&locks);
            let t2 = thread::spawn(move || {
                // Thread 2 receives overlapping keys in order [1, 10]
                let _guards = locks_t2.acquire_multi_sorted(&[1, 10]);
            });

            t1.join().expect("thread 1 completes");
            t2.join().expect("thread 2 completes");
        });
    }
}

#[cfg(not(loom))]
#[test]
fn test_multi_key_lock_deadlock_freedom_non_loom() {
    use std::sync::{Arc, Mutex, MutexGuard};
    use std::thread;

    struct StandaloneKvKeyLocks {
        shards: Vec<Mutex<()>>,
        shard_mask: u64,
    }

    impl StandaloneKvKeyLocks {
        fn new(shard_count_pow2: u32) -> Self {
            let pow2 = shard_count_pow2.min(16);
            let num_shards = 1usize << pow2;
            let shard_mask = (num_shards as u64) - 1;
            let shards = (0..num_shards).map(|_| Mutex::new(())).collect();
            Self { shards, shard_mask }
        }

        fn acquire_multi_sorted(&self, key_hashes: &[u64]) -> Vec<MutexGuard<'_, ()>> {
            let mut shard_indices: Vec<usize> = key_hashes
                .iter()
                .map(|&hash| (hash & self.shard_mask) as usize)
                .collect();

            shard_indices.sort_unstable();
            shard_indices.dedup();

            let mut guards = Vec::with_capacity(shard_indices.len());
            for idx in shard_indices {
                guards.push(self.shards[idx].lock().unwrap());
            }
            guards
        }
    }

    let locks = Arc::new(StandaloneKvKeyLocks::new(2));

    let locks_t1 = Arc::clone(&locks);
    let t1 = thread::spawn(move || {
        let _guard = locks_t1.acquire_multi_sorted(&[10, 1]);
    });

    let locks_t2 = Arc::clone(&locks);
    let t2 = thread::spawn(move || {
        let _guard = locks_t2.acquire_multi_sorted(&[1, 10]);
    });

    t1.join().unwrap();
    t2.join().unwrap();
}
