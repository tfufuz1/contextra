// FILE-CONTEXT
// STAND: 2026-09-19T20:12:00Z (SESSION: 01c5be8b)
// ZWECK: Deterministic Clock Mock for unit and integration testing without real sleep.
// INVARIANTEN: Monotonic time progression, thread-safe via AtomicU64 nanoseconds.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use contextra_ports::Clock;

/// A thread-safe, manually advanceable clock for deterministic tests.
#[derive(Debug)]
pub struct ManualClock {
    nanos: AtomicU64,
}

impl Default for ManualClock {
    fn default() -> Self {
        Self::new(1_000_000_000) // Start at 1s past epoch by default
    }
}

impl ManualClock {
    /// Creates a new `ManualClock` initialized to `initial_nanos` since UNIX_EPOCH.
    pub fn new(initial_nanos: u64) -> Self {
        Self {
            nanos: AtomicU64::new(initial_nanos),
        }
    }

    /// Advances the clock by the given duration.
    pub fn advance(&self, duration: Duration) {
        let delta = duration.as_nanos() as u64;
        self.nanos.fetch_add(delta, Ordering::SeqCst);
    }

    /// Sets the clock to an absolute timestamp in nanoseconds since UNIX_EPOCH.
    pub fn set_nanos(&self, nanos: u64) {
        self.nanos.store(nanos, Ordering::SeqCst);
    }

    /// Returns the current timestamp in nanoseconds since UNIX_EPOCH.
    pub fn now_nanos(&self) -> u64 {
        self.nanos.load(Ordering::SeqCst)
    }

    /// Returns the current timestamp as seconds (floating point).
    pub fn now_secs_f64(&self) -> f64 {
        self.now_nanos() as f64 / 1_000_000_000.0
    }

    /// Returns the current timestamp as a `SystemTime`.
    pub fn now_system_time(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_nanos(self.now_nanos())
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_manual_clock_advance() {
        let clock = ManualClock::new(100);
        assert_eq!(clock.now_nanos(), 100);

        clock.advance(Duration::from_nanos(50));
        assert_eq!(clock.now_nanos(), 150);

        clock.advance(Duration::from_secs(1));
        assert_eq!(clock.now_nanos(), 1_000_000_150);
    }

    #[test]
    fn test_manual_clock_set() {
        let clock = ManualClock::default();
        clock.set_nanos(42);
        assert_eq!(clock.now_nanos(), 42);
        assert_eq!(
            clock.now_system_time(),
            UNIX_EPOCH + Duration::from_nanos(42)
        );
    }

    #[test]
    fn test_manual_clock_as_dyn_clock() {
        let clock = ManualClock::new(1_000);
        let dyn_clock: &dyn Clock = &clock;

        assert_eq!(dyn_clock.now_unix_nanos(), 1_000);
        assert_eq!(dyn_clock.monotonic_nanos(), 1_000);

        clock.advance(Duration::from_nanos(500));

        assert_eq!(dyn_clock.now_unix_nanos(), 1_500);
        assert_eq!(dyn_clock.monotonic_nanos(), 1_500);
    }
}
