//! Controllable manual clock implementation for deterministic testing.

use crate::Clock;
use std::sync::atomic::{AtomicU64, Ordering};

// =========================================================================
// SECTION: ManualClock Implementation
// ANCHOR[id=manual-clock-root,epic=clock,seq=9]
// NOTE: Steuerbare Test-Uhr zur exakten Determinismusprüfung und Zeitsimulation.
// LINK docs/crates/contextra-env.md §2.1
// =========================================================================

/// Addiert `delta` sättigend (bei `u64::MAX` gedeckelt) auf einen atomaren Zähler.
///
/// Ein Überlauf mit Wrapping würde die Uhr rückwärts laufen lassen und damit
/// `INV-ENV-TIME-2` (Monotonie) verletzen.
#[inline]
fn saturating_add(counter: &AtomicU64, delta: u64) {
    // NOTE: `fetch_update` mit `Some(..)` schlägt nie fehl; das Ergebnis wird daher verworfen.
    let _ = counter.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
        Some(current.saturating_add(delta))
    });
}

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
    pub const fn new(unix_nanos: u64, mono_nanos: u64) -> Self {
        Self {
            unix_nanos: AtomicU64::new(unix_nanos),
            mono_nanos: AtomicU64::new(mono_nanos),
        }
    }

    /// Advances the clock by the given delta in nanoseconds.
    ///
    /// Erhöht sowohl Unix- als auch Monotonie-Nanosekunden um `delta` (sättigend).
    pub fn advance_nanos(&self, delta: u64) {
        saturating_add(&self.unix_nanos, delta);
        saturating_add(&self.mono_nanos, delta);
    }

    /// Advances the clock by the given number of milliseconds (sättigend).
    ///
    /// Entspricht Spec §2.1 (`advance_millis`); Millisekunden werden exakt in Nanosekunden umgerechnet.
    pub fn advance_millis(&self, ms: u64) {
        self.advance_nanos(ms.saturating_mul(1_000_000));
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

// =========================================================================
// SECTION: Unit Tests
// ANCHOR[id=manual-clock-tests,epic=clock,seq=10]
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_default_initial_values() {
        let clock = ManualClock::new(10, 20);
        assert_eq!(clock.now_unix_nanos(), 10);
        assert_eq!(clock.monotonic_nanos(), 20);

        let default = ManualClock::default();
        assert_eq!(default.now_unix_nanos(), 1_000_000_000);
        assert_eq!(default.monotonic_nanos(), 1_000_000_000);
    }

    #[test]
    fn test_advance_nanos_moves_both_clocks() {
        let clock = ManualClock::new(100, 200);
        clock.advance_nanos(50);
        assert_eq!(clock.now_unix_nanos(), 150);
        assert_eq!(clock.monotonic_nanos(), 250);
    }

    #[test]
    fn test_advance_millis_converts_to_nanos() {
        let clock = ManualClock::new(0, 0);
        clock.advance_millis(3);
        assert_eq!(clock.now_unix_nanos(), 3_000_000);
        assert_eq!(clock.monotonic_nanos(), 3_000_000);
    }

    #[test]
    fn test_setters_are_independent() {
        let clock = ManualClock::new(1, 2);
        clock.set_unix_nanos(500);
        assert_eq!(clock.now_unix_nanos(), 500);
        assert_eq!(clock.monotonic_nanos(), 2);

        clock.set_monotonic_nanos(900);
        assert_eq!(clock.now_unix_nanos(), 500);
        assert_eq!(clock.monotonic_nanos(), 900);
    }

    #[test]
    fn test_advance_saturates_instead_of_wrapping() {
        // NOTE: Verifiziert INV-ENV-TIME-2: Kein Rücklauf durch Überlauf.
        let clock = ManualClock::new(u64::MAX - 5, u64::MAX - 5);
        let before = clock.monotonic_nanos();
        clock.advance_nanos(100);
        assert_eq!(clock.now_unix_nanos(), u64::MAX);
        assert_eq!(clock.monotonic_nanos(), u64::MAX);
        assert!(clock.monotonic_nanos() >= before);

        clock.advance_millis(u64::MAX);
        assert_eq!(clock.now_unix_nanos(), u64::MAX);
        assert_eq!(clock.monotonic_nanos(), u64::MAX);
    }

    #[test]
    fn test_manual_clock_is_object_safe() {
        let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(ManualClock::new(7, 7));
        assert_eq!(clock.now_unix_nanos(), 7);
    }
}

// !SECTION: Unit Tests
