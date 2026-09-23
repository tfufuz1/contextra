//! Metrics reporting port trait definitions.

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
    fn test_mock_metrics_sink() {
        let sink = MockMetricsSink::default();
        let counter_sum = sink.counter_sum.clone();

        sink.record_counter("queries", 5, &[]);
        sink.record_counter("queries", 15, &[]);

        assert_eq!(counter_sum.load(Ordering::SeqCst), 20);
    }
}
