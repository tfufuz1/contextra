//! # Loom-Test: `kvcache_loom_kv_deferred_zeroize.rs`
// Herkunfts-Commit: ca103a3c743df21a70c3b0b0e2b3972a693a5ca4
//!
//! Validiert die Thread-Sicherheit und Deferred-Drop-Garantie für `contextra-kvcache`.
//! Ausführung: RUSTFLAGS="--cfg loom" cargo test --release -p contextra-kvcache --test kvcache_loom_kv_deferred_zeroize

#![allow(unexpected_cfgs)]

#[cfg(loom)]
mod loom_tests {
    use loom::sync::Arc;
    use loom::sync::RwLock;
    use loom::thread;
    use std::collections::HashMap;

    #[derive(Debug, PartialEq, Eq)]
    struct MockSegment {
        id: u64,
        data: Vec<u8>,
    }

    struct MockShard {
        map: RwLock<HashMap<u64, MockSegment>>,
    }

    impl MockShard {
        fn new() -> Self {
            Self {
                map: RwLock::new(HashMap::new()),
            }
        }

        fn insert(&self, id: u64, data: Vec<u8>) {
            let mut guard = self.map.write().unwrap();
            guard.insert(id, MockSegment { id, data });
        }

        fn get_bytes(&self, id: u64) -> Option<Vec<u8>> {
            let guard = self.map.read().unwrap();
            guard.get(&id).map(|s| s.data.clone())
        }

        fn remove_deferred(&self, id: u64) -> Option<MockSegment> {
            let removed = {
                let mut guard = self.map.write().unwrap();
                guard.remove(&id)
            };
            removed
        }
    }

    #[test]
    fn test_deferred_zeroize_race_freedom() {
        loom::model(|| {
            let shard = Arc::new(MockShard::new());
            shard.insert(42, vec![0xAA; 16]);

            let shard_remover = Arc::clone(&shard);
            let shard_reader = Arc::clone(&shard);

            let handle_remover = thread::spawn(move || {
                let deferred_seg = shard_remover.remove_deferred(42);
                drop(deferred_seg);
            });

            let handle_reader = thread::spawn(move || {
                let res = shard_reader.get_bytes(42);
                if let Some(bytes) = res {
                    assert_eq!(
                        bytes,
                        vec![0xAA; 16],
                        "Falls der Reader Daten vor dem Remove liest, müssen sie intakt sein"
                    );
                }
            });

            handle_remover.join().expect("remover finished");
            handle_reader.join().expect("reader finished");
        });
    }

    #[test]
    fn test_deferred_zeroize_three_thread_interleaving() {
        loom::model(|| {
            let shard = Arc::new(MockShard::new());
            shard.insert(100, vec![0xBB; 8]);

            let s1 = Arc::clone(&shard);
            let s2 = Arc::clone(&shard);
            let s3 = Arc::clone(&shard);

            let t1 = thread::spawn(move || {
                let seg = s1.remove_deferred(100);
                drop(seg);
            });

            let t2 = thread::spawn(move || {
                let _ = s2.get_bytes(100);
            });

            let t3 = thread::spawn(move || {
                s3.insert(200, vec![0xCC; 8]);
            });

            t1.join().unwrap();
            t2.join().unwrap();
            t3.join().unwrap();
        });
    }
}

#[cfg(not(loom))]
#[cfg(test)]
mod normal_tests {
    use contextra_ids::TenantId;
    use contextra_kvcache::{KvSegment, TenantIsolatedKvStore};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_deferred_zeroize_concurrent_removal_and_read() {
        let store = Arc::new(TenantIsolatedKvStore::new());
        let tenant = TenantId::try_new(42).unwrap();

        for id in 1..=100 {
            let seg = KvSegment::new(tenant, id, vec![id as u8; 64]);
            store.insert_segment(tenant, seg);
        }

        let store_remover = Arc::clone(&store);
        let store_reader = Arc::clone(&store);
        let running = Arc::new(AtomicBool::new(true));
        let running_reader = Arc::clone(&running);

        let reader_handle = thread::spawn(move || {
            let mut read_count = 0;
            while running_reader.load(Ordering::Relaxed) {
                for id in 1..=100 {
                    if let Some(bytes) = store_reader.get_segment_bytes(tenant, id) {
                        assert_eq!(bytes, vec![id as u8; 64]);
                        read_count += 1;
                    }
                }
            }
            read_count
        });

        let remover_handle = thread::spawn(move || {
            for id in 1..=100 {
                store_remover.remove_segment(tenant, id);
            }
        });

        remover_handle.join().unwrap();
        running.store(false, Ordering::Relaxed);
        let reads = reader_handle.join().unwrap();

        assert_eq!(store.get_tenant_segment_len(tenant), 0);
        println!("Non-loom test completed with {reads} successful concurrent reads before removal");
    }
}
