// FILE-CONTEXT
// STAND: 2026-09-19T20:12:00Z (SESSION: 01c5be8b)
// ZWECK: Deterministic Clock Mock for unit and integration testing without real sleep.
// INVARIANTEN: Monotonic time progression, thread-safe via AtomicU64 nanoseconds.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use contextra_ports::Clock;
use contextra_types::DeterministicTimestamp;

/// A thread-safe, manually advanceable clock for deterministic tests.
///
/// Enforces monotonicity as a type invariant: `set_nanos` will never move
/// the clock timestamp backwards. To explicitly rewind time in tests, call `rewind_for_test`.
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
    ///
    /// Monotonicity invariant: `set_nanos` will never move the clock backwards.
    /// If `nanos` is less than the current timestamp, the timestamp remains unchanged.
    pub fn set_nanos(&self, nanos: u64) {
        self.nanos.fetch_max(nanos, Ordering::SeqCst);
    }

    /// Explicitly rewinds the clock timestamp for test scenarios (e.g. timeout / TTL tests).
    pub fn rewind_for_test(&self, nanos: u64) {
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
    fn now(&self) -> DeterministicTimestamp {
        let wall_ms = self.now_nanos() / 1_000_000;
        DeterministicTimestamp::new(wall_ms, self.now_nanos())
    }

    fn monotonic_ticks(&self) -> u64 {
        self.now_nanos()
    }

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
        let clock = ManualClock::new(10);
        clock.set_nanos(42);
        assert_eq!(clock.now_nanos(), 42);
        assert_eq!(
            clock.now_system_time(),
            UNIX_EPOCH + Duration::from_nanos(42)
        );
    }

    #[test]
    fn test_manual_clock_set_nanos_monotonic() {
        let clock = ManualClock::new(100);
        // Attempting to set nanos backwards must be ignored by set_nanos
        clock.set_nanos(50);
        assert_eq!(clock.now_nanos(), 100);

        // Setting forwards works
        clock.set_nanos(200);
        assert_eq!(clock.now_nanos(), 200);
    }

    #[test]
    fn test_manual_clock_rewind_for_test() {
        let clock = ManualClock::new(100);
        clock.rewind_for_test(30);
        assert_eq!(clock.now_nanos(), 30);
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

    #[test]
    fn test_manual_clock_now_and_monotonic_ticks() {
        let clock = ManualClock::new(2_000_000); // 2ms = 2_000_000 nanos
        let dyn_clock: &dyn Clock = &clock;

        // Repeated reads without advancing deliver identical values
        let ts1 = dyn_clock.now();
        let ts2 = dyn_clock.now();
        let ticks1 = dyn_clock.monotonic_ticks();
        let ticks2 = dyn_clock.monotonic_ticks();

        assert_eq!(ts1, ts2);
        assert_eq!(ts1.wall_ms, 2);
        assert_eq!(ts1.mono_ctr, 2_000_000);
        assert_eq!(ticks1, ticks2);
        assert_eq!(ticks1, 2_000_000);

        // Advancing clock increases now and monotonic_ticks by exact expected amount
        let advance_duration = Duration::from_millis(5); // 5_000_000 nanos
        clock.advance(advance_duration);

        let ts3 = dyn_clock.now();
        let ticks3 = dyn_clock.monotonic_ticks();

        assert_eq!(ts3.wall_ms, 7);
        assert_eq!(ts3.mono_ctr, 7_000_000);
        assert_eq!(ticks3, 7_000_000);
        assert_eq!(ticks3 - ticks1, 5_000_000);
    }
}
