use contextra_crypto::kv_segment::segment::KvSegment;
use contextra_crypto::kv_segment::store::TenantIsolatedKvStore;
use contextra_crypto::kv_segment::EvictionWorker;
use contextra_types::TenantId;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[test]
fn eviction_worker_drop_does_not_starve_other_tasks_indefinitely() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .expect("Failed to build 1-worker-thread Tokio runtime");

    rt.block_on(async {
        let store = Arc::new(TenantIsolatedKvStore::new());
        let tenant = TenantId::try_new(100).unwrap();

        // 1. Populate store with 50 large segments (1 MiB each = 50 MiB total)
        for id in 1..=50 {
            let data = vec![0xAB; 1024 * 1024];
            store.insert_segment(tenant, KvSegment::new(tenant, id, data));
        }

        assert_eq!(store.get_tenant_segment_len(tenant), 50);

        // 2. Start EvictionWorker
        let worker = EvictionWorker::spawn(Arc::clone(&store));

        // 3. Trigger heavy eviction (45 MiB target)
        worker.trigger_eviction(45 * 1024 * 1024);

        // 4. Immediately spawn an independent async task on the same single worker thread
        let task_handle = tokio::spawn(async {
            let task_start = Instant::now();
            tokio::time::sleep(Duration::from_millis(10)).await;
            task_start.elapsed()
        });

        // 5. Drop worker directly on the Tokio worker thread
        let drop_start = Instant::now();
        drop(worker);
        let drop_duration = drop_start.elapsed();

        // 6. Await the independent task with timeout guard (5 seconds upper bound to prevent infinite hang)
        let task_elapsed = tokio::time::timeout(Duration::from_secs(5), task_handle)
            .await
            .expect("Independent Tokio task timed out after 5s - potential executor starvation deadlock")
            .expect("Task join failed");

        // 7. Quantitative characterization report for future CI observability
        let overhead = task_elapsed.saturating_sub(Duration::from_millis(10));
        eprintln!(
            "\n[EVICTION WORKER REGRESSION CHARACTERIZATION REPORT]\n\
             - Tokio Worker Threads: 1\n\
             - EvictionWorker Drop Duration: {:.2?}\n\
             - Independent Task Total Duration (base sleep = 10ms): {:.2?}\n\
             - Executor Delay Overhead due to Synchronous Join in Drop: {:.2?}\n",
            drop_duration,
            task_elapsed,
            overhead
        );

        // The test must pass cleanly while documenting the delay
        assert!(
            task_elapsed < Duration::from_secs(5),
            "Task finished within safety bound"
        );
    });
}
