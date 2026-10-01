use contextra_core::{StorageEngine, TxId};
use contextra_ports::{MetricEvent, MetricsSink, TestMetricsSink};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_store::system_pressure::{SystemPressureMonitor, WAL_QUEUE_CRITICAL_THRESHOLD, WAL_QUEUE_ELEVATED_THRESHOLD};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio_util::sync::CancellationToken;

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
async fn test_metrics_sink_records_commit_events_with_test_metrics_sink() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let mut config = LsmConfig::default();
    config.path = temp_dir.path().to_path_buf();
    config.group_commit_window_micros = 0; // Immediate single commits

    let storage = LsmStorage::new(config).await.expect("create storage");
    #[cfg(feature = "fault-injection")]
    storage.restore_wal_file_handle_for_test().await;

    let test_sink = Arc::new(TestMetricsSink::new());
    storage.set_metrics_sink(test_sink.clone());

    let tx_id = TxId::new(1);
    storage.put(tx_id, b"key1", b"val1").await.expect("put key");
    storage.commit(tx_id).await.expect("commit tx");

    let events = test_sink.events();

    let commit_counter_events: Vec<&MetricEvent> = events
        .iter()
        .filter(|e| matches!(e, MetricEvent::Counter { name, .. } if name == "lsm_commit_total"))
        .collect();

    let commit_histogram_events: Vec<&MetricEvent> = events
        .iter()
        .filter(|e| matches!(e, MetricEvent::Histogram { name, .. } if name == "lsm_commit_duration_seconds"))
        .collect();

    assert_eq!(
        commit_counter_events.len(), 1,
        "Exactly 1 lsm_commit_total counter event must be recorded for a single commit"
    );
    assert_eq!(
        commit_histogram_events.len(), 1,
        "Exactly 1 lsm_commit_duration_seconds histogram event must be recorded for a single commit"
    );

    assert_eq!(
        commit_counter_events[0],
        &MetricEvent::Counter {
            name: "lsm_commit_total".to_string(),
            value: 1,
            labels: vec![("status".to_string(), "success".to_string())],
        }
    );

    if let MetricEvent::Histogram { name, value, labels } = commit_histogram_events[0] {
        assert_eq!(name, "lsm_commit_duration_seconds");
        assert!(*value > 0.0, "Recorded commit duration must be > 0.0 seconds");
        assert_eq!(labels, &vec![("status".to_string(), "success".to_string())]);
    } else {
        panic!("Expected Histogram event");
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

#[tokio::test]
async fn test_metrics_sink_backpressure_level_transitions() {
    let test_sink = Arc::new(TestMetricsSink::new());
    let sink_container = Arc::new(parking_lot::RwLock::new(
        test_sink.clone() as Arc<dyn MetricsSink>
    ));

    let monitor = SystemPressureMonitor::new(Duration::from_millis(10));
    let cancellation = CancellationToken::new();
    let cancel_token = cancellation.clone();

    let wal_depth = Arc::new(AtomicUsize::new(0));
    let wal_depth_clone = Arc::clone(&wal_depth);

    let handle = tokio::spawn(async move {
        monitor
            .run_with_metrics_sink(
                cancel_token,
                move || wal_depth_clone.load(Ordering::Relaxed),
                || 10,
                10,
                Some(sink_container),
            )
            .await;
    });

    // Wait briefly for initial Normal gauge emission (0.0)
    tokio::time::sleep(Duration::from_millis(30)).await;

    // Simulate transition to Elevated
    wal_depth.store(WAL_QUEUE_ELEVATED_THRESHOLD + 10, Ordering::Relaxed);
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Simulate transition to Critical
    wal_depth.store(WAL_QUEUE_CRITICAL_THRESHOLD + 10, Ordering::Relaxed);
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Simulate transition back to Normal
    wal_depth.store(0, Ordering::Relaxed);
    tokio::time::sleep(Duration::from_millis(50)).await;

    cancellation.cancel();
    let _ = handle.await;

    let events = test_sink.events();
    let gauge_events: Vec<&MetricEvent> = events
        .iter()
        .filter(|e| matches!(e, MetricEvent::Gauge { name, .. } if name == "lsm_backpressure_level"))
        .collect();

    assert!(
        gauge_events.len() >= 4,
        "Expected at least 4 backpressure gauge events (Initial Normal, Elevated, Critical, Normal), got {}",
        gauge_events.len()
    );

    // Verify presence of Elevated (1.0) and Critical (2.0) gauge values
    let values: Vec<f64> = gauge_events
        .iter()
        .map(|e| match e {
            MetricEvent::Gauge { value, .. } => *value,
            _ => unreachable!(),
        })
        .collect();

    assert!(
        values.contains(&1.0),
        "Gauge events must contain Elevated value 1.0"
    );
    assert!(
        values.contains(&2.0),
        "Gauge events must contain Critical value 2.0"
    );
    assert!(
        values.contains(&0.0),
        "Gauge events must contain Normal value 0.0"
    );
}
