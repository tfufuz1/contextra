//! # Konformitätsprüfsuite für den `MetricsSink`-Port (`conformance/metrics`)
//!
//! Dieses Modul testet die vertraglichen Zusicherungen für Telemetrie- und
//! Metriksenken des [`contextra_ports::MetricsSink`]-Ports.
//!
//! ## Getestete Verträge
//! 1. **Robustheit / Panic-Freiheit (Regel 2):** Aufrufe von `record_counter`, `record_gauge`
//!    und `record_histogram` müssen fehlerfrei und ohne Panics durchlaufen, selbst
//!    bei mehrfacher Aufzeichnung mit Labels.

use super::failure::{ConformanceFailure, ConformanceResult};
use contextra_ports::MetricsSink;

// ANCHOR[contextra-testkit:CONF-METRICS-SUITE-001]
/// Führt die vollständige Konformitätsprüfung für eine [`MetricsSink`]-Implementierung durch.
///
/// **Ablauf:**
/// 1. Ruft in einem geschützten Block (`catch_unwind`) Zähler-, Pegel- und Histogramm-Aufzeichnungen auf.
/// 2. Fängt etwaige Panics ab und meldet diese strukturiert als [`ConformanceFailure`].
///
/// **Rückgabe:** `Ok(())` bei Erfolg, `Err(ConformanceFailure)` bei Panic.
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
