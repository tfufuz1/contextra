//! Controllable manual clock implementation for deterministic testing.

use crate::Clock;
use std::sync::atomic::{AtomicU64, Ordering};

// =========================================================================
// SECTION: ManualClock Implementation
// ANCHOR[id=manual-clock-root,epic=clock,seq=9]
// NOTE: Steuerbare Test-Uhr zur exakten Determinismusprüfung und Zeitsimulation.
// LINK crates/contextra-env/SPEC.md#41-clock
// =========================================================================

/// Controllable clock implementation for testing determinism and time regression.
///
/// Permits precise manual stepping of unix and monotonic nanoseconds via atomic operations.
#[derive(Debug)]
pub struct ManualClock {
    /// Aktueller simulierter Unix-Zeitstempel in Nanosekunden.
    unix_nanos: AtomicU64,
    /// Aktueller simulierter monotoner Zeitstempel in Nanosekunden.
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
    ///
    /// Erhöht sowohl Unix- als auch Monotonie-Nanosekunden um `delta`.
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
    /// Initialisiert auf 1.000.000.000 ns (1 Sekunde nach Unix-Epoche).
    fn default() -> Self {
        Self::new(1_000_000_000, 1_000_000_000)
    }
}

impl Clock for ManualClock {
    /// Liest den atomaren Unix-Zeitstempel sequentiell konsistent aus.
    fn now_unix_nanos(&self) -> u64 {
        self.unix_nanos.load(Ordering::SeqCst)
    }

    /// Liest den atomaren monotonen Zeitstempel sequentiell konsistent aus.
    fn monotonic_nanos(&self) -> u64 {
        self.mono_nanos.load(Ordering::SeqCst)
    }
}

// !SECTION: ManualClock Implementation
