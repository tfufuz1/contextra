#![allow(unexpected_cfgs)]

//! Loom concurrency model test for `SnapshotRegistry` in `contextra-mvcc`.
//! Execution: `RUSTFLAGS="--cfg loom" cargo test -p contextra-mvcc --test loom_snapshot_registry -- --nocapture`

#[cfg(loom)]
mod loom_tests {
    use contextra_mvcc::sync::Arc;
    use contextra_mvcc::SnapshotRegistry;
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
    fn test_loom_snapshot_registry_concurrent_acquire_release() {
        loom_builder().check(|| {
            let registry = Arc::new(SnapshotRegistry::new());

            let r1 = Arc::clone(&registry);
            let r2 = Arc::clone(&registry);

            let t1 = thread::spawn(move || {
                let lease = SnapshotRegistry::acquire(&r1, || 100);
                let min = r1.min_active_seqno();
                assert!(min <= 100);
                drop(lease);
            });

            let t2 = thread::spawn(move || {
                let lease = SnapshotRegistry::acquire(&r2, || 50);
                let min = r2.min_active_seqno();
                assert!(min <= 50);
                drop(lease);
            });

            t1.join().unwrap();
            t2.join().unwrap();

            assert_eq!(registry.min_active_seqno(), u64::MAX);
        });
    }

    #[test]
    fn test_loom_snapshot_registry_interleaved_pin_unpin() {
        loom_builder().check(|| {
            let registry = Arc::new(SnapshotRegistry::new());

            let r1 = Arc::clone(&registry);
            let r2 = Arc::clone(&registry);

            let t1 = thread::spawn(move || {
                r1.pin(200);
                let min = r1.min_active_seqno();
                assert!(min <= 200);
                r1.unpin(200);
            });

            let t2 = thread::spawn(move || {
                r2.pin(150);
                let min = r2.min_active_seqno();
                assert!(min <= 150);
                r2.unpin(150);
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
            let seq_base = u64::try_from(i + 1).unwrap_or(0) * 100;

            handles.push(thread::spawn(move || {
                let mut count = 0;
                while run.load(Ordering::Relaxed) && count < 1000 {
                    let seq = seq_base + (count % 10);
                    let lease = reg.acquire(|| seq);
                    let min = reg.min_active_seqno();
                    assert!(min <= seq);
                    drop(lease);
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
