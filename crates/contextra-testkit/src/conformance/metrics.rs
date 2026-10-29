use super::failure::{ConformanceFailure, ConformanceResult};
use contextra_ports::MetricsSink;

/// Runs the complete conformance test suite for a [`MetricsSink`] implementation.
///
/// # Contract Requirements Tested
/// 1. `record_counter`, `record_gauge`, and `record_histogram` accept metrics without panicking or failing.
pub fn run_metrics_sink_suite<M: MetricsSink>(sink: &M) -> ConformanceResult {
    let suite = "MetricsSink";

    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        sink.record_counter("test_counter", 1, &[("layer", "conformance")]);
        sink.record_counter("test_counter", 5, &[("layer", "conformance")]);
        sink.record_gauge("test_gauge", 42.0, &[("env", "test")]);
        sink.record_histogram("test_latency", 0.005, &[("op", "put")]);
    }));

    if res.is_err() {
        return Err(ConformanceFailure::new(
            suite,
            "record_panic",
            "MetricsSink panicked during metric recording",
        ));
    }

    Ok(())
}
