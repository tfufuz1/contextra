//! Clock port trait for deterministic time operations.

use contextra_types::DeterministicTimestamp;
use std::sync::Arc;

// TODO(WP-0 / PR-1 Schritt 4 / B0-d): Doppelte Definition von `SystemClock` bereinigen.
// `SystemClock` existiert aktuell in beiden Crates (`contextra-ports` und `contextra-sys`).
// Sobald `contextra-sys` den `Clock`-Trait via optionales Feature `ports` implementiert,
// sollen alle 63 Consumer-Dateien auf `contextra_sys::SystemClock` umgestellt werden
// und diese Definition hier als Re-export bzw. deprecated markiert werden.
/// Production system clock implementation backed by [`std::time::SystemTime`] and [`std::time::Instant`].
#[derive(Debug, Clone, Copy)]
pub struct SystemClock {
    start_instant: std::time::Instant,
}

impl SystemClock {
    /// Creates a new [`SystemClock`] with current instant as monotonic baseline.
    // DETERMINISM-EXCEPTION: SystemClock port implementation in contextra-ports
    #[allow(clippy::disallowed_methods)]
    pub fn new() -> Self {
        Self {
            start_instant: std::time::Instant::now(),
        }
    }

    /// Returns the current wall-clock time as nanoseconds since the Unix epoch.
    pub fn now_unix_nanos(&self) -> u64 {
        Clock::now_unix_nanos(self)
    }

    /// Returns nanoseconds elapsed since monotonic baseline reference point.
    pub fn monotonic_nanos(&self) -> u64 {
        Clock::monotonic_nanos(self)
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    // DETERMINISM-EXCEPTION: SystemClock port implementation in contextra-ports
    #[allow(clippy::disallowed_methods)]
    fn now(&self) -> DeterministicTimestamp {
        let wall_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
            .unwrap_or(0);
        let mono_ctr = u64::try_from(
            std::time::Instant::now()
                .saturating_duration_since(self.start_instant)
                .as_nanos(),
        )
        .unwrap_or(u64::MAX);
        DeterministicTimestamp::new(wall_ms, mono_ctr)
    }

    // DETERMINISM-EXCEPTION: SystemClock port implementation in contextra-ports
    #[allow(clippy::disallowed_methods)]
    fn monotonic_ticks(&self) -> u64 {
        u64::try_from(
            std::time::Instant::now()
                .saturating_duration_since(self.start_instant)
                .as_nanos(),
        )
        .unwrap_or(u64::MAX)
    }

    // DETERMINISM-EXCEPTION: SystemClock port implementation in contextra-ports
    #[allow(clippy::disallowed_methods)]
    fn now_unix_nanos(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
            .unwrap_or(0)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.monotonic_ticks()
    }
}

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
