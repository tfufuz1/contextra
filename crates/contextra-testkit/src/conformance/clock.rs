//! # Konformitätsprüfsuite für den `Clock`-Port (`conformance/clock`)
//!
//! Dieses Modul testet die vertraglichen Zusicherungen (Contract Invariants)
//! von Implementierungen des [`contextra_ports::Clock`]-Ports.
//!
//! ## Getestete Verträge
//! 1. **Monotonie:** [`contextra_ports::Clock::monotonic_nanos`] darf niemals rückwärts laufen ($t_2 \ge t_1$).
//! 2. **Gültiger Unix-Zeitstempel:** [`contextra_ports::Clock::now_unix_nanos`] darf nicht 0 sein (verhindert uninitialisierte Mock-Zustände).

use super::failure::{ConformanceFailure, ConformanceResult};
use contextra_ports::Clock;

// ANCHOR[contextra-testkit:CONF-CLOCK-SUITE-001]
/// Führt die vollständige Konformitätsprüfung für eine [`Clock`]-Implementierung durch.
///
/// **Ablauf:**
/// 1. Ruft zweimal hintereinander `monotonic_nanos()` ab.
/// 2. Prüft, dass der zweite Wert größer oder gleich dem ersten ist.
/// 3. Fragt `now_unix_nanos()` ab und stellt sicher, dass der Wert ungleich 0 ist.
///
/// **Rückgabe:**
/// - `Ok(())` bei Vertragskonformität.
/// - `Err(ConformanceFailure)` mit Testfall und Fehlerursache bei Verletzung.
pub fn run_clock_suite<C: Clock>(clock: &C) -> ConformanceResult {
    let suite = "Clock";

    let t1 = clock.monotonic_nanos();
    let t2 = clock.monotonic_nanos();
    if t2 < t1 {
        return Err(ConformanceFailure::new(
            suite,
            "monotonic_non_decreasing",
            format!("monotonic clock decreased from {t1} to {t2}"),
        ));
    }

    let unix = clock.now_unix_nanos();
    if unix == 0 {
        return Err(ConformanceFailure::new(
            suite,
            "valid_unix_nanos",
            "now_unix_nanos returned 0",
        ));
    }

    Ok(())
}
