//! Controllable manual clock implementation for deterministic testing.

use crate::Clock;
use std::sync::atomic::{AtomicU64, Ordering};

/// Controllable clock implementation for testing determinism and time regression.
#[derive(Debug)]
pub struct ManualClock {
    unix_nanos: AtomicU64,
    mono_nanos: AtomicU64,
}

impl ManualClock {
    /// Creates a new `ManualClock` initialized at the given unix and monotonic timestamps.
    pub fn new(unix_nanos: u64, mono_nanos: u64) -> Self {
        Self {
            unix_nanos: AtomicU64::new(unix_nanos),
            mono_nanos: AtomicU64::new(mono_nanos),
        }
    }

    /// Advances the clock by the given delta in nanoseconds.
    pub fn advance_nanos(&self, delta: u64) {
        self.unix_nanos.fetch_add(delta, Ordering::SeqCst);
        self.mono_nanos.fetch_add(delta, Ordering::SeqCst);
    }

    /// Sets the unix timestamp to an absolute value in nanoseconds.
    pub fn set_unix_nanos(&self, nanos: u64) {
        self.unix_nanos.store(nanos, Ordering::SeqCst);
    }

    /// Sets the monotonic timestamp to an absolute value in nanoseconds.
    pub fn set_monotonic_nanos(&self, nanos: u64) {
        self.mono_nanos.store(nanos, Ordering::SeqCst);
    }
}

impl Default for ManualClock {
    fn default() -> Self {
        Self::new(1_000_000_000, 1_000_000_000)
    }
}

impl Clock for ManualClock {
    fn now_unix_nanos(&self) -> u64 {
        self.unix_nanos.load(Ordering::SeqCst)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.mono_nanos.load(Ordering::SeqCst)
    }
}
