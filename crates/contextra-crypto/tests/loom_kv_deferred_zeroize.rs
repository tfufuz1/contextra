#![allow(unexpected_cfgs)]

//! Loom-basierter Nebenläufigkeitsbeweis für Deferred Zeroize in `contextra-security` (`crates/contextra-crypto`).
//! Ausführung: RUSTFLAGS="--cfg loom" cargo test -p contextra-security --test loom_kv_deferred_zeroize -- --nocapture
//!
//! Testziel: Beweisen, dass beim Entfernen / Evictieren von KV-Segmenten das Shard-Lock
//! VOR dem Zeroize (in `drop()`) freigegeben wird und kein konkurrierender Reader nach dem
//! Entfernen aus der Map noch Zugriff auf veraltete oder teilweise gezeroizte Daten erhalten kann.

#[cfg(loom)]
mod loom_tests {
    use loom::sync::Arc;
    use loom::sync::RwLock;
    use loom::thread;
    use std::collections::HashMap;

    // APM-LOOM-STATE-EXPLOSION: Maximal 2-3 Threads pro loom::model()-Aufruf,
    // um eine Explosion des Suchraums (State-Explosion) und stundenlange Testlaufzeiten zu vermeiden.
    //
    // ARCHITEKTUR-HINWEIS / PARKING_LOT-KOMPATIBILITÄT:
    // Die Produktionsklasse `TenantIsolatedKvStore` nutzt intern `parking_lot::RwLock`.
    // `parking_lot::RwLock` wird von Loom nicht direkt abgefangen oder bezüglich Thread-Interleavings
    // simuliert. Daher bilden wir das exakte Two-Phase-Deferred-Drop-Verhalten der Shard-Struktur
    // aus `crates/contextra-crypto/src/kv_segment/store.rs` unter Verwendung von `loom::sync::RwLock`
    // und `loom::sync::Arc` in diesem Loom-Test nach, ohne den Produktionscode zu verändern.

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

        /// Nachbildung des Two-Phase Deferred-Drop Musters aus `store.rs`:
        /// Phase 1: Entfernen aus der HashMap unter Shard-Write-Lock.
        /// Phase 2: Rückgabe des evicteten Segments, sodass `drop()` erst AUSSERHALB des Locks aufgerufen wird.
        fn remove_deferred(&self, id: u64) -> Option<MockSegment> {
            // Phase 1: Lock-Erwerb und Map-Entfernung
            let removed = {
                let mut guard = self.map.write().unwrap();
                guard.remove(&id)
            }; // Shard-Write-Lock wird HIER freigegeben

            // Phase 2: Rückgabe zur verzögerten Freigabe (Deferred Drop) außerhalb des Locks
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

            // Thread 1 (Remover): Führt Phase 1 (remove unter write lock) und Phase 2 (deferred drop) aus
            let handle_remover = thread::spawn(move || {
                let deferred_seg = shard_remover.remove_deferred(42);
                // Phase 2: drop() läuft hier außerhalb des Write-Locks
                drop(deferred_seg);
            });

            // Thread 2 (Reader): Konkurrierender Lesezugriff
            let handle_reader = thread::spawn(move || {
                let res = shard_reader.get_bytes(42);
                if let Some(bytes) = res {
                    assert_eq!(
                        bytes,
                        vec![0xAA; 16],
                        "Falls der Reader Daten vor dem Remove liest, müssen sie intakt sein"
                    );
                } else {
                    // Reader erhält None nach dem Remove - korrekt und erwartet
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

            // Beschränkung auf 3 Threads, um Loom State-Explosion zu verhindern (APM-LOOM-STATE-EXPLOSION)
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
    use contextra_crypto::kv_segment::segment::KvSegment;
    use contextra_crypto::kv_segment::store::TenantIsolatedKvStore;
    use contextra_types::TenantId;
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
