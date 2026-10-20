//! Clock port trait for deterministic time operations.

use contextra_types::DeterministicTimestamp;
use std::sync::Arc;

/// Port trait for time operations, enabling deterministic testing by decoupling
/// system wall-clock and monotonic time sources.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TestClock {
        wall_ms: AtomicU64,
        ticks: AtomicU64,
    }

    impl Clock for TestClock {
        fn now(&self) -> DeterministicTimestamp {
            let ms = self.wall_ms.load(Ordering::Relaxed);
            let ctr = self.ticks.fetch_add(1, Ordering::Relaxed);
            DeterministicTimestamp::new(ms, ctr)
        }

        fn monotonic_ticks(&self) -> u64 {
            self.ticks.load(Ordering::Relaxed)
        }
    }

    fn assert_dyn_clock(_: Option<&dyn Clock>) {}

    #[test]
    fn test_clock_object_safety() {
        let clock = Arc::new(TestClock {
            wall_ms: AtomicU64::new(1000),
            ticks: AtomicU64::new(10),
        });

        assert_dyn_clock(Some(clock.as_ref()));

        let dyn_clock: Arc<dyn Clock> = clock;
        let ts = dyn_clock.now();
        assert_eq!(ts.wall_ms, 1000);
        assert_eq!(ts.mono_ctr, 10);
        assert_eq!(dyn_clock.now_unix_nanos(), 1_000_000_000);

        let t1 = dyn_clock.monotonic_ticks();
        let t2 = dyn_clock.monotonic_ticks();
        assert!(t2 >= t1);
        let duration_diff = t2 - t1;
        assert_eq!(duration_diff, 0);
    }
}
