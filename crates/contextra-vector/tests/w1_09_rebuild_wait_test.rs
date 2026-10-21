use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use contextra_vector::hnsw::{HnswConfigBuilder, HnswIndex};
use contextra_types::DistanceMetric;

#[test]
fn rebuild_wait_does_not_block_current_thread_runtime() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let config = HnswConfigBuilder::new(4)
            .m(16)
            .ef_construction(200)
            .distance_metric(DistanceMetric::Cosine)
            .build()
            .unwrap();
        let index = HnswIndex::try_new(config).unwrap();

        // Simulate rebuild active
        index.inner_core().hot.rebuilding.store(true, Ordering::SeqCst);

        let timer_progressed = Arc::new(AtomicBool::new(false));
        let timer_flag = timer_progressed.clone();

        // Spawn a background timer task on the single-threaded runtime
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            timer_flag.store(true, Ordering::SeqCst);
        });

        // Spawn wait_for_rebuild_with_timeout
        let index_arc = index.inner_core_arc();
        let wait_handle = tokio::spawn(async move {
            index_arc
                .wait_for_rebuild_with_timeout(Duration::from_millis(100))
                .await
        });

        // Sleep for 30ms so timer task should have fired IF the runtime thread was not blocked by std::thread::sleep
        tokio::time::sleep(Duration::from_millis(30)).await;

        let timer_done = timer_progressed.load(Ordering::SeqCst);

        // Clean up
        index.inner_core().hot.rebuilding.store(false, Ordering::SeqCst);
        let _ = wait_handle.await;

        assert!(
            timer_done,
            "Timer task must progress on single-threaded runtime during wait_for_rebuild"
        );
    });
}
