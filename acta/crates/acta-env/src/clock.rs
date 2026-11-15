//! Clock port trait and system time implementation.

use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

// =========================================================================
// SECTION: Clock Port Trait
// ANCHOR[id=clock-root,epic=clock,seq=2]
// NOTE: Primäre Zeiterfassungsabstraktion zur Entkopplung von Systemzeit und Tests.
// LINK crates/acta-env/SPEC.md#41-clock
// =========================================================================

/// Port trait for time operations, enabling deterministic testing by decoupling
/// system wall-clock and monotonic time sources.
///
/// In production, [`SystemClock`] provides access to the real OS clocks.
/// In unit tests or simulations, deterministic doubles like `ManualClock` can be substituted.
pub trait Clock: Send + Sync + 'static {
    /// Returns the current wall-clock time as nanoseconds since the Unix epoch.
    ///
    /// Returns `0` if system time is before the epoch instead of panicking.
    ///
    /// # Invarianten
    /// - `INV-ENV-TIME-1`: Garantiert panikfrei; liefert bei Zeiten vor Unix-Epoche `0`.
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei, liefert bei Zeit vor Unix-Epoche `0`.
    /// # Crash
    /// Reiner Statusabruf ohne persistente Mutation.
    fn now_unix_nanos(&self) -> u64;

    /// Returns nanoseconds elapsed since a monotonic baseline reference point.
    ///
    /// # Invarianten
    /// - `INV-ENV-TIME-2`: Garantiert panikfrei und monoton nicht fallend innerhalb des Prozesses.
    ///
    /// # Fehler
    /// Keine; garantiert panikfrei und monoton nicht fallend.
    /// # Crash
    /// Reiner Statusabruf ohne persistente Mutation.
    fn monotonic_nanos(&self) -> u64;
}

// !SECTION: Clock Port Trait

// =========================================================================
// SECTION: SystemClock Adapter Implementation
// ANCHOR[id=clock-system,epic=clock,seq=3]
// NOTE: Einzige autorisierte Stelle für direkte Systemzeit-Aufrufe (OS System/Instant).
// =========================================================================

/// Standard production implementation of [`Clock`] backed by [`SystemTime`] and [`Instant`].
#[derive(Debug, Clone, Copy)]
pub struct SystemClock {
    /// Monotoner Referenzpunkt zur Initialisierungszeit der Instanz.
    start_instant: Instant,
}

impl SystemClock {
    /// Creates a new [`SystemClock`] using the current [`Instant`] as the monotonic baseline.
    ///
    /// # Didaktischer Hinweis
    /// Die Initialisierung von `start_instant` via `Instant::now()` setzt einen unveränderlichen
    /// Nullpunkt für die Berechnung von relativen Nanosekunden im Prozessleben.
    #[allow(clippy::disallowed_methods)]
    pub fn new() -> Self {
        Self {
            // ANCHOR[id=time-call-1,epic=clock,seq=4]
            // DIRECT TIME CALL (1/3): Authorized baseline monotonic instant for SystemClock.
            start_instant: Instant::now(),
        }
    }
}

impl Default for SystemClock {
    /// Standardkonstruktor delegiert direkt an `SystemClock::new()`.
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    /// Ermittelt die aktuelle Unix-Epochenzeit in Nanosekunden.
    ///
    /// Bei Zeitständen vor `UNIX_EPOCH` (z. B. fehlerhafte Systemuhr) wird fail-closed `0` zurückgegeben.
    #[allow(clippy::disallowed_methods)]
    fn now_unix_nanos(&self) -> u64 {
        // ANCHOR[id=time-call-2,epic=clock,seq=5]
        // DIRECT TIME CALL (2/3): Authorized OS wall-clock access encapsulated within SystemClock adapter.
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
            .unwrap_or(0)
    }

    /// Ermittelt die verstrichenen Nanosekunden seit `start_instant`.
    ///
    /// Verwendet `saturating_duration_since`, um arithmetische Überläufe und Unterläufe auszuschließen.
    #[allow(clippy::disallowed_methods)]
    fn monotonic_nanos(&self) -> u64 {
        // ANCHOR[id=time-call-3,epic=clock,seq=6]
        // DIRECT TIME CALL (3/3): Authorized OS monotonic clock access encapsulated within SystemClock adapter.
        let nanos = Instant::now()
            .saturating_duration_since(self.start_instant)
            .as_nanos();
        u64::try_from(nanos).unwrap_or(u64::MAX)
    }
}

// !SECTION: SystemClock Adapter Implementation

// =========================================================================
// SECTION: Blanket Implementation for Arc<T>
// ANCHOR[id=clock-arc,epic=clock,seq=7]
// NOTE: Erlaubt direkte Nutzung von `Arc<dyn Clock>` oder `Arc<ClockImpl>` ohne Wrapper.
// =========================================================================

impl<T: Clock + ?Sized> Clock for Arc<T> {
    #[inline]
    fn now_unix_nanos(&self) -> u64 {
        (**self).now_unix_nanos()
    }

    #[inline]
    fn monotonic_nanos(&self) -> u64 {
        (**self).monotonic_nanos()
    }
}

// !SECTION: Blanket Implementation for Arc<T>

// =========================================================================
// SECTION: Unit Tests
// ANCHOR[id=clock-tests,epic=clock,seq=8]
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Statische Typprüfung, ob `&dyn Clock` dynamisch dispatchbar ist (Object Safety).
    fn _assert_dyn_clock(_: Option<&dyn Clock>) {}

    #[test]
    fn test_system_clock_monotonic_non_decreasing() {
        // NOTE: Verifiziert INV-ENV-TIME-2 (Monotonie über Zeitintervall)
        let clock = SystemClock::new();
        let t1 = clock.monotonic_nanos();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let t2 = clock.monotonic_nanos();
        assert!(
            t2 >= t1,
            "Monotonic clock must be non-decreasing: t2={t2} < t1={t1}"
        );
    }

    #[test]
    fn test_system_clock_now_unix_nanos() {
        // NOTE: Verifiziert INV-ENV-TIME-1 (Gültige Epochenzeit > 0)
        let clock = SystemClock::default();
        let nanos = clock.now_unix_nanos();
        assert!(nanos > 0, "Unix nanos must be > 0 in modern systems");
    }

    #[test]
    fn test_arc_dyn_clock() {
        // NOTE: Verifiziert Object Safety und Arc-Deref-Delegation
        let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());
        _assert_dyn_clock(Some(clock.as_ref()));
        assert!(clock.now_unix_nanos() > 0);
    }
}

// !SECTION: Unit Tests
