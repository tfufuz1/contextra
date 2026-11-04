//! Metrics reporting port trait definitions and in-memory implementation.
//!
//! Observability Mandate (v17 Teil 11 & Teil 2.2):
//! Mandatory measurement points across the system include:
//! - Commit latency (`lsm_commit_duration_seconds`, `lsm_commit_total`)
//! - Backpressure level (`lsm_backpressure_level`)
//! - Search latency per signal
//! - Rebuild duration (HNSW)
//! - Checkpoint duration & DLQ depth

use std::collections::BTreeMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// Metrics sink port trait for recording system counters, gauges, and histograms.
///
/// Implemented by observability backends to collect operational metrics from domain
/// components without creating direct dependencies on specific metrics libraries.
pub trait MetricsSink: Send + Sync + 'static {
    /// Records a counter metric increment.
    /// # Fehler
    /// Keine; Aufzeichnung erfolgt non-blocking.
    ///
    /// # Crash
    /// Flüchtige Metrikerfassung im RAM.
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]);

    /// Records a gauge metric absolute value.
    /// # Fehler
    /// Keine; atomare Aktualisierung.
    ///
    /// # Crash
    /// Flüchtige Metrikerfassung im RAM.
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);

    /// Records a histogram observation value.
    /// # Fehler
    /// Keine; Histogramm-Update.
    ///
    /// # Crash
    /// Flüchtige Metrikerfassung im RAM.
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

/// Static enumeration of all authorized metric names in Contextra (SPEC §14.5, PO-09, D-64).
///
/// In accordance with D-64 and PO-09, metric names are strictly static enums and metric
/// labels NEVER contain user content, IDs, document tags, or tenant names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MetricName {
    /// Latency until erasure reach Gated state (`ctx_erase_gated_seconds`).
    EraseGatedSeconds,
    /// Latency of durable fsync / flush operations (`ctx_fsync_seconds`).
    FsyncSeconds,
    /// Count of open/active erasure operations (`ctx_open_erasures_count`).
    OpenErasuresCount,
    /// Age in seconds of the oldest active erasure (`ctx_oldest_open_age_seconds`).
    OldestOpenAgeSeconds,
    /// Count of SLA violations during erasure operations (`ctx_sla_violations_total`).
    SlaViolationsTotal,
    /// Ratio of tombstones to active records in storage/indexes (`ctx_tombstone_ratio`).
    TombstoneRatio,
    /// Free storage bytes on the primary volume (`ctx_free_bytes`).
    FreeBytes,
    /// Maximum time spent waiting on the purge mutual exclusion barrier (`ctx_max_purge_barrier_wait_seconds`).
    MaxPurgeBarrierWaitSeconds,
    /// Commit total counter.
    CommitTotal,
    /// Backpressure level gauge.
    BackpressureLevel,
    /// Thread pool utilization gauge.
    ThreadPoolUtilization,
    /// Search latency histogram.
    SearchLatencySeconds,
}

impl MetricName {
    /// Returns the canonical Prometheus/OpenTelemetry compatible string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::EraseGatedSeconds => "ctx_erase_gated_seconds",
            Self::FsyncSeconds => "ctx_fsync_seconds",
            Self::OpenErasuresCount => "ctx_open_erasures_count",
            Self::OldestOpenAgeSeconds => "ctx_oldest_open_age_seconds",
            Self::SlaViolationsTotal => "ctx_sla_violations_total",
            Self::TombstoneRatio => "ctx_tombstone_ratio",
            Self::FreeBytes => "ctx_free_bytes",
            Self::MaxPurgeBarrierWaitSeconds => "ctx_max_purge_barrier_wait_seconds",
            Self::CommitTotal => "ctx_commit_total",
            Self::BackpressureLevel => "ctx_backpressure_level",
            Self::ThreadPoolUtilization => "ctx_thread_pool_utilization",
            Self::SearchLatencySeconds => "ctx_search_latency_seconds",
        }
    }
}

impl fmt::Display for MetricName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Static, privacy-preserving metrics port (SPEC §14.5, PO-09, D-64).
///
/// Follows PO-09: Accepts exclusively static [`MetricName`] variants and forbids dynamic ID labels.
pub trait Metrics: Send + Sync + 'static {
    /// Records a counter metric increment.
    ///
    /// # Fehler
    /// Keine; unfehlbare Zähleraktualisierung im RAM.
    ///
    /// # Crash
    /// Reiner In-Memory-Metrikzustand; keine I/O- oder Festplattenmutation.
    fn record_counter(&self, metric: MetricName, value: u64);

    /// Records a gauge metric absolute value.
    ///
    /// # Fehler
    /// Keine; unfehlbare Pegelstandsmessung im RAM.
    ///
    /// # Crash
    /// Reiner In-Memory-Metrikzustand; keine I/O- oder Festplattenmutation.
    fn record_gauge(&self, metric: MetricName, value: f64);

    /// Records a histogram observation value in seconds or units.
    ///
    /// # Fehler
    /// Keine; unfehlbare Beobachtungsregistrierung im RAM.
    ///
    /// # Crash
    /// Reiner In-Memory-Metrikzustand; keine I/O- oder Festplattenmutation.
    fn record_histogram(&self, metric: MetricName, value: f64);
}

impl<T: Metrics + ?Sized> Metrics for Arc<T> {
    fn record_counter(&self, metric: MetricName, value: u64) {
        (**self).record_counter(metric, value);
    }

    fn record_gauge(&self, metric: MetricName, value: f64) {
        (**self).record_gauge(metric, value);
    }

    fn record_histogram(&self, metric: MetricName, value: f64) {
        (**self).record_histogram(metric, value);
    }
}

/// Adapter implementing [`Metrics`] over any existing [`MetricsSink`].
#[derive(Debug, Clone)]
pub struct MetricsSinkAdapter<S> {
    sink: S,
}

impl<S> MetricsSinkAdapter<S> {
    /// Creates a new adapter wrapping `sink`.
    pub fn new(sink: S) -> Self {
        Self { sink }
    }

    /// Returns a reference to the inner sink.
    pub fn inner(&self) -> &S {
        &self.sink
    }
}

impl<S: MetricsSink> Metrics for MetricsSinkAdapter<S> {
    fn record_counter(&self, metric: MetricName, value: u64) {
        self.sink.record_counter(metric.as_str(), value, &[]);
    }

    fn record_gauge(&self, metric: MetricName, value: f64) {
        self.sink.record_gauge(metric.as_str(), value, &[]);
    }

    fn record_histogram(&self, metric: MetricName, value: f64) {
        self.sink.record_histogram(metric.as_str(), value, &[]);
    }
}

/// Key identifying a metric by name and sorted key-value labels.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MetricKey {
    /// Metric identifier (e.g. `lsm_commit_total`).
    pub name: String,
    /// Sorted label key-value pairs.
    pub labels: Vec<(String, String)>,
}

impl MetricKey {
    /// Creates a new normalized [`MetricKey`].
    pub fn new(name: impl Into<String>, labels: &[(&str, &str)]) -> Self {
        let mut sorted_labels: Vec<(String, String)> = labels
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        sorted_labels.sort();
        Self {
            name: name.into(),
            labels: sorted_labels,
        }
    }
}

/// Snapshot of a recorded histogram metric.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HistogramSnapshot {
    /// Sum of all observed values.
    pub sum: f64,
    /// Total count of observations.
    pub count: u64,
    /// Minimum observed value (None if empty).
    pub min: Option<f64>,
    /// Maximum observed value (None if empty).
    pub max: Option<f64>,
    /// Pre-configured standard latency/value bucket counts.
    /// Buckets: <= 0.001, <= 0.005, <= 0.01, <= 0.05, <= 0.1, <= 0.5, <= 1.0, <= 5.0, <= 10.0, > 10.0
    pub buckets: BTreeMap<String, u64>,
}

/// In-memory snapshot of all metrics collected by [`InMemoryMetricsSink`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MetricSnapshot {
    /// Counter metrics (accumulated values).
    pub counters: BTreeMap<MetricKey, u64>,
    /// Gauge metrics (last recorded f64 values).
    pub gauges: BTreeMap<MetricKey, f64>,
    /// Histogram metrics.
    pub histograms: BTreeMap<MetricKey, HistogramSnapshot>,
}

#[derive(Debug, Default)]
struct HistogramState {
    sum: f64,
    count: u64,
    min: Option<f64>,
    max: Option<f64>,
    b_1ms: u64,
    b_5ms: u64,
    b_10ms: u64,
    b_50ms: u64,
    b_100ms: u64,
    b_500ms: u64,
    b_1s: u64,
    b_5s: u64,
    b_10s: u64,
    b_inf: u64,
}

impl HistogramState {
    fn observe(&mut self, val: f64) {
        self.sum += val;
        self.count += 1;
        self.min = Some(self.min.map_or(val, |m| m.min(val)));
        self.max = Some(self.max.map_or(val, |m| m.max(val)));

        if val <= 0.001 {
            self.b_1ms += 1;
        } else if val <= 0.005 {
            self.b_5ms += 1;
        } else if val <= 0.010 {
            self.b_10ms += 1;
        } else if val <= 0.050 {
            self.b_50ms += 1;
        } else if val <= 0.100 {
            self.b_100ms += 1;
        } else if val <= 0.500 {
            self.b_500ms += 1;
        } else if val <= 1.000 {
            self.b_1s += 1;
        } else if val <= 5.000 {
            self.b_5s += 1;
        } else if val <= 10.000 {
            self.b_10s += 1;
        } else {
            self.b_inf += 1;
        }
    }

    fn snapshot(&self) -> HistogramSnapshot {
        let mut buckets = BTreeMap::new();
        buckets.insert("<= 0.001s".to_string(), self.b_1ms);
        buckets.insert("<= 0.005s".to_string(), self.b_5ms);
        buckets.insert("<= 0.010s".to_string(), self.b_10ms);
        buckets.insert("<= 0.050s".to_string(), self.b_50ms);
        buckets.insert("<= 0.100s".to_string(), self.b_100ms);
        buckets.insert("<= 0.500s".to_string(), self.b_500ms);
        buckets.insert("<= 1.000s".to_string(), self.b_1s);
        buckets.insert("<= 5.000s".to_string(), self.b_5s);
        buckets.insert("<= 10.000s".to_string(), self.b_10s);
        buckets.insert("> 10.000s".to_string(), self.b_inf);

        HistogramSnapshot {
            sum: self.sum,
            count: self.count,
            min: self.min,
            max: self.max,
            buckets,
        }
    }
}

/// Production-ready local thread-safe implementation of [`MetricsSink`].
///
/// Stores metrics strictly in-process with no external network exports (privacy-preserving).
#[derive(Debug, Default)]
pub struct InMemoryMetricsSink {
    counters: RwLock<BTreeMap<MetricKey, Arc<AtomicU64>>>,
    gauges: RwLock<BTreeMap<MetricKey, f64>>,
    histograms: RwLock<BTreeMap<MetricKey, HistogramState>>,
}

impl InMemoryMetricsSink {
    /// Creates a new empty [`InMemoryMetricsSink`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Captures a point-in-time [`MetricSnapshot`] of all recorded counters, gauges, and histograms.
    pub fn snapshot(&self) -> MetricSnapshot {
        let counters_guard = match self.counters.read() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        let counters = counters_guard
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect();

        let gauges_guard = match self.gauges.read() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        let gauges = gauges_guard.clone();

        let histograms_guard = match self.histograms.read() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        let histograms = histograms_guard
            .iter()
            .map(|(k, v)| (k.clone(), v.snapshot()))
            .collect();

        MetricSnapshot {
            counters,
            gauges,
            histograms,
        }
    }
}

impl MetricsSink for InMemoryMetricsSink {
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]) {
        let key = MetricKey::new(name, labels);
        {
            let guard = match self.counters.read() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            if let Some(counter) = guard.get(&key) {
                counter.fetch_add(value, Ordering::Relaxed);
                return;
            }
        }

        let mut guard = match self.counters.write() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        let counter = guard
            .entry(key)
            .or_insert_with(|| Arc::new(AtomicU64::new(0)));
        counter.fetch_add(value, Ordering::Relaxed);
    }

    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let key = MetricKey::new(name, labels);
        let mut guard = match self.gauges.write() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.insert(key, value);
    }

    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let key = MetricKey::new(name, labels);
        let mut guard = match self.histograms.write() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.entry(key).or_default().observe(value);
    }
}

impl<T: MetricsSink + ?Sized> MetricsSink for Arc<T> {
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]) {
        (**self).record_counter(name, value, labels);
    }

    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        (**self).record_gauge(name, value, labels);
    }

    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        (**self).record_histogram(name, value, labels);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SPEC: T-ARCH-6
    #[test]
    fn test_static_metrics_adapter() {
        let sink = Arc::new(InMemoryMetricsSink::new());
        let adapter = MetricsSinkAdapter::new(sink.clone());

        adapter.record_counter(MetricName::CommitTotal, 42);
        adapter.record_gauge(MetricName::BackpressureLevel, 1.5);
        adapter.record_histogram(MetricName::FsyncSeconds, 0.003);

        let snap = sink.snapshot();
        let counter_key = MetricKey::new(MetricName::CommitTotal.as_str(), &[]);
        let gauge_key = MetricKey::new(MetricName::BackpressureLevel.as_str(), &[]);
        let hist_key = MetricKey::new(MetricName::FsyncSeconds.as_str(), &[]);

        assert_eq!(snap.counters.get(&counter_key), Some(&42));
        assert_eq!(snap.gauges.get(&gauge_key), Some(&1.5));
        let hist_snap = snap.histograms.get(&hist_key).expect("hist key");
        assert_eq!(hist_snap.count, 1);
        assert_eq!(hist_snap.min, Some(0.003));
    }

    /// SPEC: T-ARCH-6
    #[test]
    fn test_noop_metrics_sink() {
        let sink = NoopMetricsSink;
        sink.record_counter("test_counter", 10, &[("layer", "ring0")]);
        sink.record_gauge("test_gauge", 42.0, &[]);
        sink.record_histogram("test_hist", 1.23, &[]);
    }

    /// SPEC: T-ARCH-6
    #[test]
    fn test_in_memory_metrics_sink_counters_and_gauges() {
        let sink = InMemoryMetricsSink::new();
        sink.record_counter("commit_count", 1, &[("tenant", "t1")]);
        sink.record_counter("commit_count", 2, &[("tenant", "t1")]);

        sink.record_gauge("backpressure", 0.75, &[("layer", "lsm")]);

        let snap = sink.snapshot();
        let counter_key = MetricKey::new("commit_count", &[("tenant", "t1")]);
        let gauge_key = MetricKey::new("backpressure", &[("layer", "lsm")]);

        assert_eq!(snap.counters.get(&counter_key), Some(&3));
        assert_eq!(snap.gauges.get(&gauge_key), Some(&0.75));
    }

    /// SPEC: T-ARCH-6
    #[test]
    fn test_in_memory_metrics_sink_histograms() {
        let sink = InMemoryMetricsSink::new();
        let key_labels = [("signal", "hnsw")];
        sink.record_histogram("search_latency", 0.002, &key_labels);
        sink.record_histogram("search_latency", 0.020, &key_labels);

        let snap = sink.snapshot();
        let hist_key = MetricKey::new("search_latency", &key_labels);
        let hist_snap = snap
            .histograms
            .get(&hist_key)
            .expect("histogram key present");

        assert_eq!(hist_snap.count, 2);
        assert!((hist_snap.sum - 0.022).abs() < 1e-6);
        assert_eq!(hist_snap.min, Some(0.002));
        assert_eq!(hist_snap.max, Some(0.020));
        assert_eq!(hist_snap.buckets.get("<= 0.005s"), Some(&1));
        assert_eq!(hist_snap.buckets.get("<= 0.050s"), Some(&1));
    }
}
