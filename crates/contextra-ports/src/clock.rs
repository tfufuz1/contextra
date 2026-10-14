//! Clock port trait and system/manual clock implementations.

use contextra_types::DeterministicTimestamp;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Port trait for time operations, enabling deterministic testing by decoupling
/// system wall-clock and monotonic time sources.
///
/// Returns [`DeterministicTimestamp`] (`wall_ms` + `mono_ctr`).
/// Duration measurements are differences of monotonic ticks returned by [`Clock::monotonic_ticks`].
pub trait Clock: Send + Sync + 'static {
    /// Returns the current [`DeterministicTimestamp`] containing wall-clock time in milliseconds
    /// and a monotonic counter value.
    fn now(&self) -> DeterministicTimestamp;

    /// Returns monotonic ticks (e.g. nanoseconds or discrete ticks) for duration measurements.
    fn monotonic_ticks(&self) -> u64;

    /// Returns the current wall-clock time as nanoseconds since the Unix epoch.
    fn now_unix_nanos(&self) -> u64 {
        self.now().wall_ms.saturating_mul(1_000_000)
    }

    /// Returns nanoseconds elapsed since a monotonic baseline reference point.
    fn monotonic_nanos(&self) -> u64 {
        self.monotonic_ticks()
    }
}

impl<T: Clock + ?Sized> Clock for Arc<T> {
    fn now(&self) -> DeterministicTimestamp {
        (**self).now()
    }

    fn monotonic_ticks(&self) -> u64 {
        (**self).monotonic_ticks()
    }

    fn now_unix_nanos(&self) -> u64 {
        (**self).now_unix_nanos()
    }

    fn monotonic_nanos(&self) -> u64 {
        (**self).monotonic_nanos()
    }
}

/// Standard production implementation of [`Clock`] backed by [`SystemTime`] and [`Instant`].
#[derive(Debug, Clone, Copy)]
pub struct SystemClock {
    start_instant: Instant,
}

impl SystemClock {
    /// Creates a new [`SystemClock`] using the current [`Instant`] as the monotonic baseline.
    #[allow(clippy::disallowed_methods)]
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
    #[allow(clippy::disallowed_methods, clippy::cast_possible_truncation)]
    fn now(&self) -> DeterministicTimestamp {
        let wall_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
            .unwrap_or(0);

        let mono_ctr = u64::try_from(
            Instant::now()
                .saturating_duration_since(self.start_instant)
                .as_nanos(),
        )
        .unwrap_or(u64::MAX);

        DeterministicTimestamp::new(wall_ms, mono_ctr)
    }

    #[allow(clippy::disallowed_methods, clippy::cast_possible_truncation)]
    fn monotonic_ticks(&self) -> u64 {
        u64::try_from(
            Instant::now()
                .saturating_duration_since(self.start_instant)
                .as_nanos(),
        )
        .unwrap_or(u64::MAX)
    }
}

/// A thread-safe, manually advanceable clock implementation for deterministic testing.
#[derive(Debug)]
pub struct ManualClock {
    nanos: AtomicU64,
}

impl ManualClock {
    /// Creates a new [`ManualClock`] initialized to `initial_nanos`.
    pub fn new(initial_nanos: u64) -> Self {
        Self {
            nanos: AtomicU64::new(initial_nanos),
        }
    }

    /// Advances the clock by the specified [`Duration`](std::time::Duration).
    pub fn advance(&self, duration: std::time::Duration) {
        let delta: u64 = duration.as_nanos().try_into().unwrap_or(u64::MAX);
        self.nanos.fetch_add(delta, Ordering::SeqCst);
    }

    /// Sets the clock nanoseconds directly to `nanos`.
    pub fn set_nanos(&self, nanos: u64) {
        self.nanos.store(nanos, Ordering::SeqCst);
    }

    /// Returns the current clock nanoseconds.
    pub fn now_nanos(&self) -> u64 {
        self.nanos.load(Ordering::SeqCst)
    }
}

impl Default for ManualClock {
    fn default() -> Self {
        Self::new(1_000_000_000)
    }
}

impl Clock for ManualClock {
    fn now(&self) -> DeterministicTimestamp {
        let n = self.now_nanos();
        DeterministicTimestamp::new(n / 1_000_000, n % 1_000_000)
    }

    fn monotonic_ticks(&self) -> u64 {
        self.now_nanos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_dyn_clock(_: Option<&dyn Clock>) {}

    #[test]
    fn test_system_clock() {
        let clock = SystemClock::new();
        let ts = clock.now();
        assert!(ts.wall_ms > 0);
        let t1 = clock.monotonic_ticks();
        let t2 = clock.monotonic_ticks();
        assert!(t2 >= t1);
    }

    #[test]
    fn test_manual_clock() {
        let clock = ManualClock::new(100_000_000);
        let ts = clock.now();
        assert_eq!(ts.wall_ms, 100);
        clock.advance(std::time::Duration::from_millis(50));
        assert_eq!(clock.now().wall_ms, 150);
    }

    #[test]
    fn test_arc_dyn_clock() {
        let clock: Arc<dyn Clock> = Arc::new(ManualClock::new(1_000_000_000));
        assert_dyn_clock(Some(clock.as_ref()));
        assert_eq!(clock.now().wall_ms, 1000);
    }
}
