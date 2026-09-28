#![allow(unexpected_cfgs)]

//! Loom concurrency model test for `SnapshotRegistry` in `contextra-mvcc`.
//! Execution: `RUSTFLAGS="--cfg loom" cargo test -p contextra-mvcc --test loom_snapshot_registry -- --nocapture`

#[cfg(loom)]
mod loom_tests {
    use loom::sync::Arc;
    use loom::sync::Mutex;
    use loom::thread;
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicU64, Ordering};

    // APM-LOOM-STATE-EXPLOSION: Limit to 2-3 threads per loom::model execution.
    // ARCHITECTURE NOTE / PARKING_LOT COMPATIBILITY:
    // Production SnapshotRegistry uses parking_lot::Mutex which Loom does not intercept directly.
    // As documented in Loom test patterns across the repository (e.g. loom_kv_deferred_zeroize.rs),
    // we mirror the state transitions of SnapshotRegistry using loom::sync::Mutex and loom::sync::Arc
    // to model thread interleaving and atomic minimum state transitions under Loom.
    struct MockSnapshotRegistry {
        active: Mutex<BTreeMap<u64, usize>>,
        min_active_seqno: AtomicU64,
    }

    impl MockSnapshotRegistry {
        fn new() -> Self {
            Self {
                active: Mutex::new(BTreeMap::new()),
                min_active_seqno: AtomicU64::new(u64::MAX),
            }
        }

        fn register(&self, seq_no: u64) {
            let mut active = self.active.lock().unwrap();
            *active.entry(seq_no).or_default() += 1;
            self.update_min(&active);
        }

        fn release(&self, seq_no: u64) {
            let mut active = self.active.lock().unwrap();
            if let Some(count) = active.get_mut(&seq_no) {
                *count -= 1;
                if *count == 0 {
                    active.remove(&seq_no);
                }
            }
            self.update_min(&active);
        }

        fn min_active_seqno(&self) -> u64 {
            self.min_active_seqno.load(Ordering::Acquire)
        }

        fn update_min(&self, active: &BTreeMap<u64, usize>) {
            let min = active.keys().next().copied().unwrap_or(u64::MAX);
            self.min_active_seqno.store(min, Ordering::Release);
        }
    }

    #[test]
    fn test_loom_snapshot_registry_concurrent_register_release() {
        loom::model(|| {
            let registry = Arc::new(MockSnapshotRegistry::new());

            let r1 = Arc::clone(&registry);
            let r2 = Arc::clone(&registry);

            let t1 = thread::spawn(move || {
                r1.register(100);
                let min = r1.min_active_seqno();
                assert!(min <= 100);
                r1.release(100);
            });

            let t2 = thread::spawn(move || {
                r2.register(50);
                let min = r2.min_active_seqno();
                assert!(min <= 50);
                r2.release(50);
            });

            t1.join().unwrap();
            t2.join().unwrap();

            assert_eq!(registry.min_active_seqno(), u64::MAX);
        });
    }

    #[test]
    fn test_loom_snapshot_registry_interleaved_pin_unpin() {
        loom::model(|| {
            let registry = Arc::new(MockSnapshotRegistry::new());

            let r1 = Arc::clone(&registry);
            let r2 = Arc::clone(&registry);

            let t1 = thread::spawn(move || {
                r1.register(200);
                let min = r1.min_active_seqno();
                assert!(min <= 200);
                r1.release(200);
            });

            let t2 = thread::spawn(move || {
                r2.register(150);
                let min = r2.min_active_seqno();
                assert!(min <= 150);
                r2.release(150);
            });

            t1.join().unwrap();
            t2.join().unwrap();

            assert_eq!(registry.min_active_seqno(), u64::MAX);
        });
    }
}

#[cfg(not(loom))]
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod normal_tests {
    use contextra_mvcc::SnapshotRegistry;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_concurrent_snapshot_registry_stress() {
        let registry = Arc::new(SnapshotRegistry::new());
        let running = Arc::new(AtomicBool::new(true));

        let num_threads = 4;
        let mut handles = Vec::new();

        for i in 0..num_threads {
            let reg = Arc::clone(&registry);
            let run = Arc::clone(&running);
            let seq_base = (i + 1) as u64 * 100;

            handles.push(thread::spawn(move || {
                let mut count = 0;
                while run.load(Ordering::Relaxed) && count < 1000 {
                    let g = reg.register(seq_base + (count % 10));
                    let min = reg.min_active_seqno();
                    assert!(min <= seq_base + (count % 10));
                    drop(g);
                    count += 1;
                }
            }));
        }

        for h in handles {
            h.join().expect("thread panicked"); // #[cfg(test)] // expect
        }

        assert_eq!(registry.min_active_seqno(), u64::MAX);
    }
}
