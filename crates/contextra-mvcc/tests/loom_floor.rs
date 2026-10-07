#![allow(unexpected_cfgs)]

//! Loom concurrency model test for concurrent `acquire()` and GC floor reading in `contextra-mvcc`.
//! Execution: `RUSTFLAGS="--cfg loom" cargo test -p contextra-mvcc --test loom_floor -- --nocapture`

#[cfg(loom)]
mod loom_floor_tests {
    use loom::sync::Arc;
    use loom::sync::Mutex;
    use loom::thread;
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicU64, Ordering};

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

        fn acquire(&self, read_seq: impl FnOnce() -> u64) -> u64 {
            let mut active = self.active.lock().unwrap();
            let seq = read_seq();
            *active.entry(seq).or_default() += 1;
            let min = active.keys().next().copied().unwrap_or(u64::MAX);
            self.min_active_seqno.store(min, Ordering::Release);
            seq
        }

        fn release(&self, seq: u64) {
            let mut active = self.active.lock().unwrap();
            if let Some(count) = active.get_mut(&seq) {
                *count -= 1;
                if *count == 0 {
                    active.remove(&seq);
                }
            }
            let min = active.keys().next().copied().unwrap_or(u64::MAX);
            self.min_active_seqno.store(min, Ordering::Release);
        }

        fn floor(&self, last_applied: &AtomicU64) -> u64 {
            let reg_min = self.min_active_seqno.load(Ordering::Acquire);
            let la = last_applied.load(Ordering::Acquire);
            reg_min.min(la)
        }
    }

    #[test]
    fn test_loom_concurrent_acquire_and_gc_floor() {
        loom::model(|| {
            let registry = Arc::new(MockSnapshotRegistry::new());
            let last_applied = Arc::new(AtomicU64::new(100));

            let reg1 = Arc::clone(&registry);
            let la1 = Arc::clone(&last_applied);

            let reg2 = Arc::clone(&registry);
            let la2 = Arc::clone(&last_applied);

            // Thread 1: Concurrent reader acquiring snapshot lease at last_applied
            let t1 = thread::spawn(move || {
                let seq = reg1.acquire(|| la1.load(Ordering::Acquire));
                let floor = reg1.floor(&la1);
                // INVARIANT: floor must never exceed an active reader's acquired sequence
                assert!(floor <= seq);
                reg1.release(seq);
            });

            // Thread 2: Concurrent compaction worker reading GC floor or advancing sequence
            let t2 = thread::spawn(move || {
                let floor = reg2.floor(&la2);
                assert!(floor <= 100);
            });

            t1.join().unwrap();
            t2.join().unwrap();
        });
    }
}

#[cfg(not(loom))]
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod normal_floor_tests {
    use contextra_mvcc::snapshot::{GcFloor, SnapshotRegistry};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_concurrent_acquire_and_gc_floor_stress() {
        let registry = Arc::new(SnapshotRegistry::new());
        let last_applied = Arc::new(AtomicU64::new(100));
        let floor_calc = Arc::new(GcFloor::<Vec<u8>>::new(
            registry.clone(),
            None,
            last_applied.clone(),
        ));

        let num_threads = 4;
        let mut handles = Vec::new();

        for _i in 0..num_threads {
            let reg = registry.clone();
            let la = last_applied.clone();
            let fc = floor_calc.clone();

            handles.push(thread::spawn(move || {
                for count in 0..500 {
                    let seq_read = la.load(Ordering::Acquire);
                    let lease = reg.acquire(|| seq_read);
                    let current_floor = fc.floor();
                    assert!(
                        current_floor <= lease.seq_no(),
                        "GC floor {current_floor} exceeded reader seq {}",
                        lease.seq_no()
                    );
                    drop(lease);

                    if count % 10 == 0 {
                        la.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }));
        }

        for h in handles {
            h.join().expect("thread panicked");
        }
    }
}
