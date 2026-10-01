use contextra_core::{StorageEngine, TxId};
use contextra_ports::MetricsSink;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;

#[derive(Default)]
struct SpyMetricsSink {
    commit_counter_success: AtomicU64,
    commit_counter_failure: AtomicU64,
    histogram_observations_success: AtomicU64,
    histogram_observations_failure: AtomicU64,
    histogram_sum_secs_nanos: AtomicU64,
}

impl MetricsSink for SpyMetricsSink {
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]) {
        if name == "lsm_commit_total" {
            let status = labels
                .iter()
                .find(|(k, _)| *k == "status")
                .map(|(_, v)| *v)
                .unwrap_or_default();
            if status == "success" {
                self.commit_counter_success.fetch_add(value, Ordering::SeqCst);
            } else if status == "failure" {
                self.commit_counter_failure.fetch_add(value, Ordering::SeqCst);
            }
        }
    }

    fn record_gauge(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {}

    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        if name == "lsm_commit_duration_seconds" {
            let status = labels
                .iter()
                .find(|(k, _)| *k == "status")
                .map(|(_, v)| *v)
                .unwrap_or_default();
            if status == "success" {
                self.histogram_observations_success
                    .fetch_add(1, Ordering::SeqCst);
                let nanos = (value * 1_000_000_000.0) as u64;
                self.histogram_sum_secs_nanos
                    .fetch_add(nanos, Ordering::SeqCst);
            } else if status == "failure" {
                self.histogram_observations_failure
                    .fetch_add(1, Ordering::SeqCst);
            }
        }
    }
}

#[tokio::test]
async fn test_metrics_sink_records_commit_events() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let mut config = LsmConfig::default();
    config.path = temp_dir.path().to_path_buf();
    config.group_commit_window_micros = 0; // Immediate single commits

    let storage = LsmStorage::new(config).await.expect("create storage");
    #[cfg(feature = "fault-injection")]
    storage.restore_wal_file_handle_for_test().await;

    let spy_sink = Arc::new(SpyMetricsSink::default());
    storage.set_metrics_sink(spy_sink.clone());

    const N: u64 = 15;

    let start_time = Instant::now();
    for i in 1..=N {
        let tx_id = TxId::new(i);
        let key = format!("key_{i}").into_bytes();
        let value = format!("value_{i}").into_bytes();

        storage.put(tx_id, &key, &value).await.expect("put key");
        storage.commit(tx_id).await.expect("commit tx");
    }
    let total_elapsed = start_time.elapsed();

    // Verify exactly N commits resulted in N counter increments and N duration histogram observations
    let success_count = spy_sink.commit_counter_success.load(Ordering::SeqCst);
    let failure_count = spy_sink.commit_counter_failure.load(Ordering::SeqCst);
    let hist_success_count = spy_sink
        .histogram_observations_success
        .load(Ordering::SeqCst);
    let hist_failure_count = spy_sink
        .histogram_observations_failure
        .load(Ordering::SeqCst);
    let recorded_nanos_sum = spy_sink.histogram_sum_secs_nanos.load(Ordering::SeqCst);

    println!(
        "Testergebnis: N={} Commits -> success_counter={}, failure_counter={}, hist_observations={}, recorded_nanos_sum={}ns, total_wall_clock={:?}",
        N, success_count, failure_count, hist_success_count, recorded_nanos_sum, total_elapsed
    );

    assert_eq!(success_count, N, "Counter must record exactly N successful commits");
    assert_eq!(failure_count, 0, "No commit failures expected");
    assert_eq!(
        hist_success_count, N,
        "Histogram must observe exactly N duration events"
    );
    assert_eq!(hist_failure_count, 0, "No failure histogram events expected");

    // Plausibility check: duration must be > 0 and sum of reported durations <= total wall clock time + small tolerance
    assert!(recorded_nanos_sum > 0, "Recorded commit duration must be non-zero");
    let total_elapsed_nanos = total_elapsed.as_nanos() as u64;
    assert!(
        recorded_nanos_sum <= total_elapsed_nanos + 1_000_000,
        "Sum of reported commit durations ({recorded_nanos_sum} ns) should not exceed wall time ({total_elapsed_nanos} ns)"
    );
}

#[cfg(feature = "fault-injection")]
#[tokio::test]
async fn test_metrics_sink_records_commit_failures() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let mut config = LsmConfig::default();
    config.path = temp_dir.path().to_path_buf();
    config.group_commit_window_micros = 0;

    let storage = LsmStorage::new(config).await.expect("create storage");
    let spy_sink = Arc::new(SpyMetricsSink::default());
    storage.set_metrics_sink(spy_sink.clone());

    let tx_id = TxId::new(100);
    storage
        .put(tx_id, b"fail_key", b"fail_value")
        .await
        .expect("stage put");
    storage.simulate_wal_append_failure_for_tx_for_test(100).await;

    let commit_res = storage.commit(tx_id).await;
    assert!(commit_res.is_err(), "Commit should fail due to simulated WAL failure");

    storage.restore_wal_file_handle_for_test().await;

    let success_count = spy_sink.commit_counter_success.load(Ordering::SeqCst);
    let failure_count = spy_sink.commit_counter_failure.load(Ordering::SeqCst);
    let hist_failure_count = spy_sink
        .histogram_observations_failure
        .load(Ordering::SeqCst);

    assert_eq!(success_count, 0, "No successful commits expected");
    assert_eq!(failure_count, 1, "Counter must record 1 commit failure");
    assert_eq!(hist_failure_count, 1, "Histogram must record 1 duration observation for failure");
}
