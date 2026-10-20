//! Execution: `RUSTFLAGS="--cfg loom" cargo test -p contextra-kvcache --test loom_offset_counters -- --nocapture`

#[cfg(loom)]
mod loom_tests {
    use loom::sync::atomic::{AtomicUsize, Ordering};
    use loom::sync::Arc;
    use loom::thread;

    fn loom_builder() -> loom::model::Builder {
        let mut builder = loom::model::Builder::new();
        builder.max_branches = 10000;
        builder
    }

    #[test]
    fn test_loom_round_robin_global_and_shard_offsets() {
        loom_builder().check(|| {
            let global_shard_offset = Arc::new(AtomicUsize::new(0));
            let shard_count = 4;

            let eviction_round_offsets = Arc::new(
                (0..shard_count)
                    .map(|_| AtomicUsize::new(0))
                    .collect::<Vec<_>>(),
            );

            let g_offset1 = Arc::clone(&global_shard_offset);
            let e_offsets1 = Arc::clone(&eviction_round_offsets);
            let t1 = thread::spawn(move || {
                let start_shard = g_offset1.fetch_add(1, Ordering::Relaxed) % shard_count;
                let tenant_count = 3;
                let tenant_offset =
                    e_offsets1[start_shard].fetch_add(1, Ordering::Relaxed) % tenant_count;
                (start_shard, tenant_offset)
            });

            let g_offset2 = Arc::clone(&global_shard_offset);
            let e_offsets2 = Arc::clone(&eviction_round_offsets);
            let t2 = thread::spawn(move || {
                let start_shard = g_offset2.fetch_add(1, Ordering::Relaxed) % shard_count;
                let tenant_count = 3;
                let tenant_offset =
                    e_offsets2[start_shard].fetch_add(1, Ordering::Relaxed) % tenant_count;
                (start_shard, tenant_offset)
            });

            let res1 = t1.join().unwrap();
            let res2 = t2.join().unwrap();

            assert!(res1.0 < shard_count);
            assert!(res2.0 < shard_count);
            assert!(res1.1 < 3);
            assert!(res2.1 < 3);

            let total_global_fetches = global_shard_offset.load(Ordering::Relaxed);
            assert_eq!(total_global_fetches, 2);
        });
    }
}

#[cfg(not(loom))]
#[test]
fn test_loom_placeholder_when_disabled() {
    // When loom is disabled, this standard test confirms offset behavior in normal execution mode.
    use std::sync::atomic::{AtomicUsize, Ordering};

    let offset = AtomicUsize::new(0);
    let val1 = offset.fetch_add(1, Ordering::Relaxed);
    let val2 = offset.fetch_add(1, Ordering::Relaxed);
    assert_eq!(val1, 0);
    assert_eq!(val2, 1);
}
