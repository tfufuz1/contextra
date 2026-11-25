//! Metric names constants and safe recording helpers for contextra-engine operational observability.
//!
//! Complies with Spezifikation v17 Teil 2.2 & 11.

/// Metric name for engine commit latency histogram observations.
pub const COMMIT_LATENCY: &str = "commit.latency";

/// Metric name for index rebuild duration histogram observations.
pub const REBUILD_DURATION: &str = "rebuild.duration";

/// Metric name for system backpressure level gauge readings (0.0 = Normal, 1.0 = Elevated, 2.0 = Critical).
pub const BACKPRESSURE_LEVEL: &str = "backpressure.level";

/// Metric name for checkpoint operation duration histogram observations.
pub const CHECKPOINT_DURATION: &str = "checkpoint.duration";

/// Reserved metric name for vector signal search latency observations.
pub const SEARCH_LATENCY_VECTOR: &str = "search.latency.vector";

/// Reserved metric name for text signal search latency observations.
pub const SEARCH_LATENCY_TEXT: &str = "search.latency.text";

/// Reserved metric name for graph signal search latency observations.
pub const SEARCH_LATENCY_GRAPH: &str = "search.latency.graph";

/// Helper to record a histogram metric best-effort without allowing panics to propagate.
pub fn record_histogram_safe(
    sink: &dyn contextra_ports::MetricsSink,
    name: &str,
    value: f64,
    labels: &[(&str, &str)],
) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        sink.record_histogram(name, value, labels);
    }));
}

/// Helper to record a gauge metric best-effort without allowing panics to propagate.
pub fn record_gauge_safe(
    sink: &dyn contextra_ports::MetricsSink,
    name: &str,
    value: f64,
    labels: &[(&str, &str)],
) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        sink.record_gauge(name, value, labels);
    }));
}

/// Helper to record a counter metric best-effort without allowing panics to propagate.
pub fn record_counter_safe(
    sink: &dyn contextra_ports::MetricsSink,
    name: &str,
    value: u64,
    labels: &[(&str, &str)],
) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        sink.record_counter(name, value, labels);
    }));
}
