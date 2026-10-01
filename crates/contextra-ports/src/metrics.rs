//! Metrics reporting port trait definitions.
//!
//! Observability Mandate (v17 Teil 11 & Teil 2.2):
//! Mandatory measurement points across the system include:
//! - Commit latency (`lsm_commit_duration_seconds`, `lsm_commit_total`) - Covered by Prompt 13 (contextra-store)
//! - Backpressure level (`lsm_backpressure_level`) - Covered by Prompt 13 (contextra-store)
//! - Search latency per signal - Covered by Prompt 14
//! - Rebuild duration (HNSW) - Covered by Prompt 15
//! - Checkpoint duration & DLQ depth - Covered by Prompt 25

/// Metrics sink port trait for recording system counters, gauges, and histograms.
///
/// Implemented by observability backends to collect operational metrics from domain
/// components without creating direct dependencies on specific metrics libraries.
pub trait MetricsSink: Send + Sync + 'static {
    /// Records a counter metric increment.
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]);

    /// Records a gauge metric absolute value.
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);

    /// Records a histogram observation value.
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]);
}

/// A no-op implementation of [`MetricsSink`] that silently discards all metrics events.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopMetricsSink;

impl MetricsSink for NoopMetricsSink {
    fn record_counter(&self, _name: &str, _value: u64, _labels: &[(&str, &str)]) {}

    fn record_gauge(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {}

    fn record_histogram(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {}
}

/// A recorded metric event captured by [`TestMetricsSink`].
#[derive(Debug, Clone, PartialEq)]
pub enum MetricEvent {
    /// Counter increment event.
    Counter {
        /// Metric name.
        name: String,
        /// Incremented value.
        value: u64,
        /// Metric key-value labels.
        labels: Vec<(String, String)>,
    },
    /// Gauge setting event.
    Gauge {
        /// Metric name.
        name: String,
        /// Absolute gauge value.
        value: f64,
        /// Metric key-value labels.
        labels: Vec<(String, String)>,
    },
    /// Histogram observation event.
    Histogram {
        /// Metric name.
        name: String,
        /// Observed value.
        value: f64,
        /// Metric key-value labels.
        labels: Vec<(String, String)>,
    },
}

/// An in-memory [`MetricsSink`] implementation for testing that records all emitted metric events.
#[derive(Debug, Default, Clone)]
pub struct TestMetricsSink {
    events: std::sync::Arc<std::sync::Mutex<Vec<MetricEvent>>>,
}

impl TestMetricsSink {
    /// Creates a new empty [`TestMetricsSink`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a snapshot copy of all recorded metric events.
    pub fn events(&self) -> Vec<MetricEvent> {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Clears all recorded metric events.
    pub fn clear(&self) {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
    }
}

impl MetricsSink for TestMetricsSink {
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]) {
        let labels_vec = labels
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(MetricEvent::Counter {
                name: name.to_string(),
                value,
                labels: labels_vec,
            });
    }

    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let labels_vec = labels
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(MetricEvent::Gauge {
                name: name.to_string(),
                value,
                labels: labels_vec,
            });
    }

    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let labels_vec = labels
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(MetricEvent::Histogram {
                name: name.to_string(),
                value,
                labels: labels_vec,
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    #[derive(Default)]
    struct MockMetricsSink {
        counter_sum: Arc<AtomicU64>,
    }

    impl MetricsSink for MockMetricsSink {
        fn record_counter(&self, _name: &str, value: u64, _labels: &[(&str, &str)]) {
            self.counter_sum.fetch_add(value, Ordering::SeqCst);
        }

        fn record_gauge(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {}

        fn record_histogram(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {}
    }

    #[test]
    fn test_noop_metrics_sink() {
        let sink = NoopMetricsSink;
        sink.record_counter("test_counter", 10, &[("layer", "ring0")]);
        sink.record_gauge("test_gauge", 42.0, &[]);
        sink.record_histogram("test_hist", 1.23, &[]);
    }

    #[test]
    fn test_test_metrics_sink() {
        let sink = TestMetricsSink::new();
        sink.record_counter("c1", 5, &[("env", "test")]);
        sink.record_gauge("g1", 1.5, &[]);
        sink.record_histogram("h1", 0.02, &[("op", "commit")]);

        let events = sink.events();
        assert_eq!(events.len(), 3);
        assert_eq!(
            events[0],
            MetricEvent::Counter {
                name: "c1".to_string(),
                value: 5,
                labels: vec![("env".to_string(), "test".to_string())],
            }
        );
        assert_eq!(
            events[1],
            MetricEvent::Gauge {
                name: "g1".to_string(),
                value: 1.5,
                labels: vec![],
            }
        );
        assert_eq!(
            events[2],
            MetricEvent::Histogram {
                name: "h1".to_string(),
                value: 0.02,
                labels: vec![("op".to_string(), "commit".to_string())],
            }
        );

        sink.clear();
        assert!(sink.events().is_empty());
    }

    #[test]
    fn test_mock_metrics_sink() {
        let sink = MockMetricsSink::default();
        let counter_sum = sink.counter_sum.clone();

        sink.record_counter("queries", 5, &[]);
        sink.record_counter("queries", 15, &[]);

        assert_eq!(counter_sum.load(Ordering::SeqCst), 20);
    }
}
