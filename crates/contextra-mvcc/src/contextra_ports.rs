//! Local port traits decoupled from the removed `contextra-ports` crate.

use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Monotonic / wall-clock time source.
pub trait Clock: Send + Sync {
    /// Current wall-clock timestamp in nanoseconds since UNIX epoch.
    fn now_unix_nanos(&self) -> u64;
    /// Monotonic timestamp in nanoseconds.
    fn monotonic_nanos(&self) -> u64;
}

/// Default system clock implementation.
#[derive(Debug, Clone, Copy)]
pub struct SystemClock {
    _private: (),
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemClock {
    /// Creates a new `SystemClock`.
    #[inline]
    pub const fn new() -> Self {
        Self { _private: () }
    }
}

impl Clock for SystemClock {
    fn now_unix_nanos(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
            .unwrap_or(0)
    }

    fn monotonic_nanos(&self) -> u64 {
        static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
        let start = START.get_or_init(Instant::now);
        u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }
}

/// Metrics sink trait for MVCC observability.
pub trait MetricsSink: Send + Sync {
    /// Records a counter increment.
    fn increment_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]);
    /// Records a gauge value.
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);
}

/// Deterministic manual clock for unit tests.
#[derive(Debug, Default)]
pub struct ManualClock {
    nanos: std::sync::atomic::AtomicU64,
}

impl ManualClock {
    /// Creates a new `ManualClock` initialized to `initial_nanos`.
    pub fn new(initial_nanos: u64) -> Self {
        Self {
            nanos: std::sync::atomic::AtomicU64::new(initial_nanos),
        }
    }

    /// Returns the current timestamp in nanoseconds.
    pub fn now_nanos(&self) -> u64 {
        self.nanos.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Advances the clock by `delta`.
    pub fn advance(&self, delta: std::time::Duration) {
        let delta_nanos = u64::try_from(delta.as_nanos()).unwrap_or(u64::MAX);
        self.nanos
            .fetch_add(delta_nanos, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now_unix_nanos(&self) -> u64 {
        self.nanos.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.nanos.load(std::sync::atomic::Ordering::SeqCst)
    }
}
