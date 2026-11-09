//! Clock implementation for HNSW vector index ensuring determinism (B11).

use std::sync::atomic::{AtomicU64, Ordering};
use contextra_ports::Clock;

/// Deterministic monotonic counter clock for HNSW vector index.
///
/// Implements [`Clock`] without any reference to system wall-clock or non-deterministic OS time sources.
#[derive(Debug)]
pub struct MonotonicCounterClock {
    nanos: AtomicU64,
}

impl MonotonicCounterClock {
    /// Creates a new [`MonotonicCounterClock`] starting at the given nanosecond timestamp.
    pub fn new(initial_nanos: u64) -> Self {
        Self {
            nanos: AtomicU64::new(initial_nanos),
        }
    }

    /// Advances the clock by `delta_nanos`.
    pub fn advance_nanos(&self, delta_nanos: u64) {
        self.nanos.fetch_add(delta_nanos, Ordering::SeqCst);
    }
}

impl Default for MonotonicCounterClock {
    fn default() -> Self {
        Self::new(1_000_000_000)
    }
}

impl Clock for MonotonicCounterClock {
    fn now_unix_nanos(&self) -> u64 {
        self.nanos.load(Ordering::SeqCst)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.nanos.load(Ordering::SeqCst)
    }
}
