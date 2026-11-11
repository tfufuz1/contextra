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
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

// =========================================================================
// SECTION: MetricsSink Port Trait
// ANCHOR[id=metrics-root,epic=metrics,seq=1]
// NOTE: Primäre Schnittstelle für Telemetrie und Performance-Monitoring.
// LINK crates/contextra-env/SPEC.md#44-metricssink
// =========================================================================

/// Metrics sink port trait for recording system counters, gauges, and histograms.
///
/// Implemented by observability backends to collect operational metrics from domain
/// components without creating direct dependencies on specific metrics libraries.
pub trait MetricsSink: Send + Sync + 'static {
    /// Records a counter metric increment.
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei und nicht-blockierend.
    /// # Crash
    /// Reiner flüchtiger RAM-Zähler; keine I/O oder persistente Mutation.
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]);

    /// Records a gauge metric absolute value.
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei und nicht-blockierend.
    /// # Crash
    /// Reiner flüchtiger RAM-Messwert; keine I/O oder persistente Mutation.
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);

    /// Records a histogram observation value.
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei und nicht-blockierend.
    /// # Crash
    /// Reiner flüchtiger RAM-Histogrammwert; keine I/O oder persistente Mutation.
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]);
}

// !SECTION: MetricsSink Port Trait

// =========================================================================
// SECTION: NoopMetricsSink Null-Object Adapter
// ANCHOR[id=metrics-noop,epic=metrics,seq=2]
// NOTE: Discard-Implementierung ohne Overhead für Produktion ohne Metriken oder Tests.
// =========================================================================

/// A no-op implementation of [`MetricsSink`] that silently discards all metrics events.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopMetricsSink;

impl MetricsSink for NoopMetricsSink {
    #[inline]
    fn record_counter(&self, _name: &str, _value: u64, _labels: &[(&str, &str)]) {}

    #[inline]
    fn record_gauge(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {}

    #[inline]
    fn record_histogram(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {}
}

// !SECTION: NoopMetricsSink Null-Object Adapter

// =========================================================================
// SECTION: MetricKey Domain Model
// ANCHOR[id=metrics-key,epic=metrics,seq=3]
// NOTE: Eindeutiger Identifikator bestehend aus Name und normalisierten Labels.
// =========================================================================

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
    ///
    /// Sortiert Labels deterministisch lexikografisch, um Hash- und BTreeMap-Konsistenz zu wahren.
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

// !SECTION: MetricKey Domain Model

// =========================================================================
// SECTION: Snapshots & Internal State
// ANCHOR[id=metrics-snapshot,epic=metrics,seq=4]
// NOTE: Unveränderliche Momentaufnahmen der Metriken für Tests und Assertionen.
// =========================================================================

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

/// Interner Akkumulator für Histogrammbeobachtungen.
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
    /// Aktualisiert Summe, Anzahl, Min/Max und inkrementiert das passende Bucket.
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

    /// Erzeugt einen lesbaren Snapshot mit beschrifteten Bucket-Grenzen.
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

// !SECTION: Snapshots & Internal State

// =========================================================================
// SECTION: InMemoryMetricsSink Implementation
// ANCHOR[id=metrics-inmemory,epic=metrics,seq=5]
// NOTE: Lokale threadsichere Senke ohne externe Netzwerk-Exports (Privacy-Preserving).
// =========================================================================

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
        // NOTE: Vergiftete Locks werden via `into_inner()` panikfrei entschärft
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
        // Fast-Path: Existierenden Counter unter Read-Lock atomar inkrementieren
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

        // Slow-Path: Write-Lock nur zur Erzeugung des neuen AtomicU64-Eintrags
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

// !SECTION: InMemoryMetricsSink Implementation

// =========================================================================
// SECTION: Blanket Implementation for Arc<T>
// ANCHOR[id=metrics-arc,epic=metrics,seq=6]
// NOTE: Erlaubt direkte Nutzung von `Arc<dyn MetricsSink>` ohne Wrapper.
// =========================================================================

impl<T: MetricsSink + ?Sized> MetricsSink for Arc<T> {
    #[inline]
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]) {
        (**self).record_counter(name, value, labels);
    }

    #[inline]
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        (**self).record_gauge(name, value, labels);
    }

    #[inline]
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        (**self).record_histogram(name, value, labels);
    }
}

// !SECTION: Blanket Implementation for Arc<T>

// =========================================================================
// SECTION: Unit Tests
// ANCHOR[id=metrics-tests,epic=metrics,seq=7]
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_metrics_sink() {
        // NOTE: Verifiziert, dass NoopMetricsSink panikfrei ohne Seiteneffekte operiert
        let sink = NoopMetricsSink;
        sink.record_counter("test_counter", 10, &[("layer", "ring0")]);
        sink.record_gauge("test_gauge", 42.0, &[]);
        sink.record_histogram("test_hist", 1.23, &[]);
    }

    #[test]
    fn test_in_memory_metrics_sink_counters_and_gauges() {
        // NOTE: Verifiziert Akkumulation von Zählern und Überschreiben von Gauges
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

    #[test]
    fn test_in_memory_metrics_sink_histograms() {
        // NOTE: Verifiziert Verteilung auf Latenz-Buckets und Minimum/Maximum-Berechnung
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

// !SECTION: Unit Tests
