//! Local port traits decoupled from the removed `contextra-ports` crate.

use std::time::{SystemTime, UNIX_EPOCH};

/// Monotonic / wall-clock time source.
pub trait Clock: Send + Sync {
    /// Current timestamp in milliseconds since UNIX epoch.
    fn now_ms(&self) -> u64;
}

/// Default system clock implementation.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl SystemClock {
    /// Creates a new `SystemClock`.
    #[inline]
    pub const fn new() -> Self {
        Self
    }
}

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
            .unwrap_or(0)
    }
}

/// Metrics sink trait for MVCC observability.
pub trait MetricsSink: Send + Sync {
    /// Records a counter increment.
    fn increment_counter(&self, name: &str, value: u64);
    /// Records a gauge value.
    fn record_gauge(&self, name: &str, value: f64);
}
