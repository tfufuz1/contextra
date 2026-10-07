#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mvcc::SequenceLogSsiValidator;
use contextra_mvcc::SnapshotRegistry;
use contextra_ports::metrics::{MetricEvent, TestMetricsSink};
use contextra_ports::MetricsSink;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[test]
fn test_ssi_metrics_observability_threshold_and_coarsening() {
    let test_sink = Arc::new(TestMetricsSink::new());
    let sink_container: Arc<RwLock<Arc<dyn MetricsSink>>> =
        Arc::new(RwLock::new(test_sink.clone() as Arc<dyn MetricsSink>));

    let max_keys = 20;
    let validator =
        SequenceLogSsiValidator::new_with_bounds(max_keys).with_metrics_sink(sink_container);

    // Commit 17 distinct keys to exceed the 80% threshold (17 / 20 = 85%)
    for i in 1..=17 {
        let key = format!("user_key_{i:02}");
        let seq = u64::try_from(i).unwrap_or(0);
        validator.record_commit_key(key.as_bytes(), seq);
    }

    let events = test_sink.events();
    assert!(!events.is_empty(), "Metric events must be recorded");

    // Verify utilization ratio gauge
    let has_utilization_gauge = events.iter().any(|e| match e {
        MetricEvent::Gauge { name, value, .. } => {
            name == "mvcc_ssi_commit_register_utilization_ratio" && *value >= 0.80
        }
        _ => false,
    });
    assert!(
        has_utilization_gauge,
        "Expected mvcc_ssi_commit_register_utilization_ratio gauge event with ratio >= 0.80, got: {:?}",
        events
    );

    // Verify coarsened buckets total gauge
    let has_coarsened_gauge = events.iter().any(|e| match e {
        MetricEvent::Gauge { name, value, .. } => {
            name == "mvcc_ssi_coarsened_buckets_total" && *value >= 1.0
        }
        _ => false,
    });
    assert!(
        has_coarsened_gauge,
        "Expected mvcc_ssi_coarsened_buckets_total gauge event with value >= 1.0, got: {:?}",
        events
    );
}

#[test]
fn test_ssi_metrics_observability_long_pin_duration() {
    let test_sink = Arc::new(TestMetricsSink::new());
    let sink_container: Arc<RwLock<Arc<dyn MetricsSink>>> =
        Arc::new(RwLock::new(test_sink.clone() as Arc<dyn MetricsSink>));

    let registry = Arc::new(SnapshotRegistry::new());
    let now = Instant::now();
    let past_350s = now.checked_sub(Duration::from_secs(350)).unwrap_or(now);
    let _lease = registry.acquire_at(|| 42, past_350s);

    let max_keys = 20;
    let validator = SequenceLogSsiValidator::new_with_bounds(max_keys)
        .with_snapshot_registry(registry)
        .with_metrics_sink(sink_container);

    // Commit 17 distinct keys to exceed threshold
    for i in 1..=17 {
        let key = format!("data_key_{i:02}");
        let seq = u64::try_from(i).unwrap_or(0);
        validator.record_commit_key(key.as_bytes(), seq);
    }

    let events = test_sink.events();

    let has_pin_duration_gauge = events.iter().any(|e| match e {
        MetricEvent::Gauge { name, value, .. } => {
            name == "mvcc_ssi_longest_pin_duration_seconds" && *value >= 350.0
        }
        _ => false,
    });
    assert!(
        has_pin_duration_gauge,
        "Expected mvcc_ssi_longest_pin_duration_seconds gauge event >= 350.0, got: {:?}",
        events
    );
}

#[test]
fn test_ssi_metrics_observability_hot_swap_consistency() {
    let sink1 = Arc::new(TestMetricsSink::new());
    let sink_container: Arc<RwLock<Arc<dyn MetricsSink>>> =
        Arc::new(RwLock::new(sink1.clone() as Arc<dyn MetricsSink>));

    let max_keys = 20;
    let validator = SequenceLogSsiValidator::new_with_bounds(max_keys)
        .with_metrics_sink(sink_container.clone());

    // Trigger metrics on initial sink
    validator.record_commit_key(b"key_init", 1);
    let events_sink1_before = sink1.events();
    assert_eq!(
        events_sink1_before.len(),
        1,
        "sink1 should receive initial metric call"
    );

    // Swap the inner MetricsSink inside the RwLock container (simulating set_metrics_sink())
    let sink2 = Arc::new(TestMetricsSink::new());
    *sink_container.write() = sink2.clone() as Arc<dyn MetricsSink>;

    // Trigger metrics again after hot-swap
    validator.record_commit_key(b"key_after_swap", 2);

    // Verify sink1 received no new events after swap
    let events_sink1_after = sink1.events();
    assert_eq!(
        events_sink1_after.len(),
        1,
        "sink1 should not receive events after hot-swap"
    );

    // Verify sink2 received the new event
    let events_sink2 = sink2.events();
    assert_eq!(
        events_sink2.len(),
        1,
        "sink2 must receive metric events recorded after hot-swap"
    );
    assert!(matches!(
        &events_sink2[0],
        MetricEvent::Gauge { name, .. } if name == "mvcc_ssi_commit_register_utilization_ratio"
    ));
}
