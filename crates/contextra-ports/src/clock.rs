//! Clock port trait and system time implementation.

use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Port trait for time operations, enabling deterministic testing by decoupling
/// system wall-clock and monotonic time sources.
pub trait Clock: Send + Sync + 'static {
    /// Returns the current wall-clock time as nanoseconds since the Unix epoch.
    ///
    /// Returns `0` if system time is before the epoch instead of panicking.
    fn now_unix_nanos(&self) -> u64;

    /// Returns nanoseconds elapsed since a monotonic baseline reference point.
    fn monotonic_nanos(&self) -> u64;
}

/// Standard production implementation of [`Clock`] backed by [`SystemTime`] and [`Instant`].
#[derive(Debug, Clone, Copy)]
pub struct SystemClock {
    start_instant: Instant,
}

impl SystemClock {
    /// Creates a new [`SystemClock`] using the current [`Instant`] as the monotonic baseline.
    pub fn new() -> Self {
        Self {
            start_instant: Instant::now(),
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
            .saturating_duration_since(self.start_instant)
            .as_nanos() as u64
    }
}

impl<T: Clock + ?Sized> Clock for Arc<T> {
    fn now_unix_nanos(&self) -> u64 {
        (**self).now_unix_nanos()
    }

    fn monotonic_nanos(&self) -> u64 {
        (**self).monotonic_nanos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _assert_dyn_clock(_: Option<&dyn Clock>) {}

    #[test]
    fn test_system_clock_monotonic_non_decreasing() {
        let clock = SystemClock::new();
        let t1 = clock.monotonic_nanos();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let t2 = clock.monotonic_nanos();
        assert!(t2 >= t1);
    }

    #[test]
    fn test_system_clock_now_unix_nanos() {
        let clock = SystemClock::default();
        let nanos = clock.now_unix_nanos();
        assert!(nanos > 0);
    }

    #[test]
    fn test_arc_dyn_clock() {
        let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());
        _assert_dyn_clock(Some(clock.as_ref()));
        assert!(clock.now_unix_nanos() > 0);
    }
}
