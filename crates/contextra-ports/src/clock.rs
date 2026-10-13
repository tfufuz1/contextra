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
    #[allow(clippy::cast_possible_truncation)]
    fn now_unix_nanos(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
            .unwrap_or(0)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn monotonic_nanos(&self) -> u64 {
        u64::try_from(
            Instant::now()
                .saturating_duration_since(self.start_instant)
                .as_nanos(),
        )
        .unwrap_or(u64::MAX)
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

/// A thread-safe, manually advanceable clock implementation for deterministic testing.
///
/// Uses an [`AtomicU64`](std::sync::atomic::AtomicU64) to track nanoseconds elapsed since the Unix epoch
/// or a deterministic monotonic baseline.
#[derive(Debug)]
pub struct ManualClock {
    nanos: std::sync::atomic::AtomicU64,
}

impl ManualClock {
    /// Creates a new [`ManualClock`] initialized to `initial_nanos`.
    pub fn new(initial_nanos: u64) -> Self {
        Self {
            nanos: std::sync::atomic::AtomicU64::new(initial_nanos),
        }
    }

    /// Advances the clock by the specified [`Duration`](std::time::Duration).
    pub fn advance(&self, duration: std::time::Duration) {
        let delta: u64 = duration.as_nanos().try_into().unwrap_or(u64::MAX);
        self.nanos
            .fetch_add(delta, std::sync::atomic::Ordering::SeqCst);
    }

    /// Sets the clock nanoseconds directly to `nanos`.
    pub fn set_nanos(&self, nanos: u64) {
        self.nanos.store(nanos, std::sync::atomic::Ordering::SeqCst);
    }

    /// Returns the current clock nanoseconds.
    pub fn now_nanos(&self) -> u64 {
        self.nanos.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Default for ManualClock {
    fn default() -> Self {
        Self::new(1_000_000_000)
    }
}

impl Clock for ManualClock {
    fn now_unix_nanos(&self) -> u64 {
        self.now_nanos()
    }

    fn monotonic_nanos(&self) -> u64 {
        self.now_nanos()
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

    #[test]
    fn test_manual_clock_advance_and_set() {
        let clock = ManualClock::new(100);
        assert_eq!(clock.now_unix_nanos(), 100);

        clock.advance(std::time::Duration::from_nanos(50));
        assert_eq!(clock.now_unix_nanos(), 150);
        assert_eq!(clock.monotonic_nanos(), 150);

        clock.set_nanos(500);
        assert_eq!(clock.now_nanos(), 500);

        let default_clock = ManualClock::default();
        assert_eq!(default_clock.now_nanos(), 1_000_000_000);
    }
}
