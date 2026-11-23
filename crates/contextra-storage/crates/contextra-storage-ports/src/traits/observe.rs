//! Metriken und Beobachter (E-44). Keine Inhalte; Labels sind nur endliche Enumwerte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricSample {
    /// Monotone count. Resets only when the process restarts.
    Counter(u64),
    /// Current value (unsigned; ratios use basis points 0..=10_000).
    Gauge(u64),
    /// One histogram observation in the metric's documented base unit.
    Observe(u64),
}

pub trait Metrics: Send + Sync {
    /// Must be nonblocking and must discard unknown names or invalid labels.
    /// Names and label keys are static; label values come from a closed vocabulary.
    fn record(&self, name: &'static str, sample: MetricSample, labels: &[(&'static str, &'static str)]);
}

pub trait Observer: Send + Sync {
    fn event(&self, name: &'static str, labels: &[(&'static str, &'static str)]);
}
