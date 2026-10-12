#![allow(unexpected_cfgs)]

//! Loom concurrency model test for concurrent `acquire()` and GC floor reading in `contextra-mvcc`.
//! Execution: `RUSTFLAGS="--cfg loom" cargo test -p contextra-mvcc --test loom_floor -- --nocapture`

#[cfg(loom)]
mod loom_floor_tests {
    use contextra_mvcc::sync::atomic::{AtomicU64, Ordering};
    use contextra_mvcc::sync::Arc;
    use contextra_mvcc::{GcFloor, SnapshotRegistry};
    use loom::thread;

    fn loom_builder() -> loom::model::Builder {
        let mut builder = loom::model::Builder::new();
        let max_preemptions = std::env::var("LOOM_MAX_PREEMPTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2);
        builder.preemption_bound = Some(max_preemptions);
        builder
    }

    #[test]
    fn test_loom_concurrent_acquire_and_gc_floor() {
        loom_builder().check(|| {
            let registry = Arc::new(SnapshotRegistry::new());
            let last_applied = Arc::new(AtomicU64::new(100));
            let gc_floor = Arc::new(GcFloor::<Vec<u8>>::new(
                Arc::clone(&registry),
                None,
                Arc::clone(&last_applied),
            ));

            let reg1 = Arc::clone(&registry);
            let la1 = Arc::clone(&last_applied);
            let gf1 = Arc::clone(&gc_floor);

            let la2 = Arc::clone(&last_applied);
            let gf2 = Arc::clone(&gc_floor);

            // Thread 1: Concurrent reader acquiring snapshot lease at last_applied
            let t1 = thread::spawn(move || {
                let lease = SnapshotRegistry::acquire(&reg1, || la1.load(Ordering::Acquire));
                let floor = gf1.floor();
                // INVARIANT: floor must never exceed an active reader's acquired sequence
                assert!(floor <= lease.seq_no());
                drop(lease);
            });

            // Thread 2: Concurrent compaction worker reading GC floor or advancing sequence
            let t2 = thread::spawn(move || {
                let floor = gf2.floor();
                assert!(floor <= la2.load(Ordering::Acquire));
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
    use contextra_mvcc::{GcFloor, SnapshotRegistry};
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
