// ZWECK: Integrationstest für Messung der Blocking-Thread-Pool-Auslastung (blocking_util) und MetricsSink-Emission.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_ports::MetricsSink;
use contextra_store::system_pressure::{SystemPressureMonitor, BLOCKING_UTIL_ELEVATED};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Default)]
struct TestMetricsSink {
    blocking_util_gauges: Arc<RwLock<Vec<f64>>>,
    backpressure_levels: Arc<RwLock<Vec<(f64, String)>>>,
}

impl MetricsSink for TestMetricsSink {
    fn record_counter(&self, _name: &str, _value: u64, _labels: &[(&str, &str)]) {}

    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        if name == "lsm_blocking_pool_utilization" {
            self.blocking_util_gauges.write().push(value);
        } else if name == "lsm_backpressure_level" {
            let level_label = labels
                .iter()
                .find(|(k, _)| *k == "level")
                .map(|(_, v)| v.to_string())
                .unwrap_or_default();
            self.backpressure_levels.write().push((value, level_label));
        }
    }

    fn record_histogram(&self, _name: &str, _value_seconds: f64, _labels: &[(&str, &str)]) {}
}

#[tokio::test]
async fn test_blocking_util_increases_under_pool_load() {
    let monitor = SystemPressureMonitor::new(Duration::from_millis(20));
    let mut rx = monitor.pressure_rx.clone();
    let cancellation = CancellationToken::new();
    let cancel_token = cancellation.clone();

    let test_sink = Arc::new(TestMetricsSink::default());
    let gauges_ref = Arc::clone(&test_sink.blocking_util_gauges);
    let sink_container: Arc<RwLock<Arc<dyn MetricsSink>>> = Arc::new(RwLock::new(test_sink));

    let handle = tokio::spawn(async move {
        monitor
            .run_with_metrics_sink(cancel_token, || 0, || 10, 10, Some(sink_container))
            .await;
    });

    // High WAL queue depth alone (e.g. 50) without blocking pool load should keep blocking_util low
    tokio::time::sleep(Duration::from_millis(60)).await;
    let initial_util = rx.borrow().blocking_util;
    assert!(
        initial_util < BLOCKING_UTIL_ELEVATED,
        "Initial blocking_util without blocking load should be low, got {}",
        initial_util
    );

    // Now saturate blocking pool with long-running spawn_blocking tasks
    let mut blocking_handles = Vec::new();
    for _ in 0..512 {
        blocking_handles.push(tokio::task::spawn_blocking(|| {
            std::thread::sleep(Duration::from_millis(150));
        }));
    }

    // Wait for monitor to detect pool saturation and update blocking_util
    let mut high_util_detected = false;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        if rx.changed().await.is_ok() {
            let p = rx.borrow().clone();
            if p.blocking_util > BLOCKING_UTIL_ELEVATED {
                high_util_detected = true;
                break;
            }
        }
    }

    assert!(
        high_util_detected,
        "blocking_util should increase above BLOCKING_UTIL_ELEVATED under blocking pool load"
    );

    cancellation.cancel();
    let _ = handle.await;
    for h in blocking_handles {
        let _ = h.await;
    }

    // Verify MetricsSink received gauge updates
    let recorded_gauges = gauges_ref.read().clone();
    assert!(
        !recorded_gauges.is_empty(),
        "MetricsSink must record lsm_blocking_pool_utilization gauges"
    );
    assert!(
        recorded_gauges.iter().any(|&v| v > 0.5),
        "At least one gauge sample should reflect high blocking pool utilization"
    );
}
