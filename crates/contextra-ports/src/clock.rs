//! Clock port trait definition and system clock implementation.

use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Abstract clock trait providing time measurements for non-determinism injection.
pub trait Clock: Send + Sync + 'static {
    /// Returns the current wall-clock time in nanoseconds since UNIX_EPOCH.
    ///
    /// If system time is before UNIX_EPOCH, returns 0.
    fn now_unix_nanos(&self) -> u64;

    /// Returns a monotonic time value in nanoseconds relative to an arbitrary anchor.
    ///
    /// Monotonic time is guaranteed not to decrease between successive calls on the same clock.
    fn monotonic_nanos(&self) -> u64;
}

impl<T: Clock + ?Sized> Clock for Arc<T> {
    fn now_unix_nanos(&self) -> u64 {
        (**self).now_unix_nanos()
    }

    fn monotonic_nanos(&self) -> u64 {
        (**self).monotonic_nanos()
    }
}

/// Production implementation of [`Clock`] backed by [`SystemTime`] and [`Instant`].
#[derive(Debug, Clone)]
pub struct SystemClock {
    anchor_instant: Instant,
}

impl SystemClock {
    /// Creates a new [`SystemClock`] anchored to the current [`Instant`].
    pub fn new() -> Self {
        Self {
            anchor_instant: Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn now_unix_nanos(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
    }

    fn monotonic_nanos(&self) -> u64 {
        Instant::now()
            .saturating_duration_since(self.anchor_instant)
            .as_nanos() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_clock_monotonic_non_decreasing() {
        let clock = SystemClock::new();
        let t1 = clock.monotonic_nanos();
        let t2 = clock.monotonic_nanos();
        assert!(t2 >= t1);
    }

    #[test]
    fn test_system_clock_unix_nanos_positive() {
        let clock = SystemClock::default();
        let nanos = clock.now_unix_nanos();
        assert!(nanos > 0);
    }

    #[test]
    fn test_arc_dyn_clock_compiles() {
        let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());
        let _ = clock.now_unix_nanos();
        let _ = clock.monotonic_nanos();
    }
}
