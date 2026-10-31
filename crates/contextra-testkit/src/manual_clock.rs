//! # Deterministischer Clock-Mock (`manual_clock`)
//!
//! Dieses Modul stellt mit [`ManualClock`] eine thread-sichere Uhr bereit,
//! deren Zeitstempel in Tests rein manuell und deterministisch gesteuert wird.
//!
//! ## Ring & Architektur-Einordnung
//! - **Ring:** Werkzeug / Tooling
//! - **Port:** Implementiert den Port-Trait [`contextra_ports::Clock`].
//! - **Regel 3 (SPEC §3.2):** Zeit darf in der Antigravity-Architektur niemals direkt
//!   über das Betriebssystem (`std::time::SystemTime::now()` oder `Instant::now()`)
//!   abgefragt werden, sondern ausschließlich über den `Clock`-Port. [`ManualClock`]
//!   ist die Referenzimplementierung für Tests, um Timeouts und Zeitabläufe ohne
//!   echte Wartezeiten (`thread::sleep`) zu prüfen.
//!
//! ## Thread-Sicherheit & Nebenläufigkeit
//! - Verwendet atomare 64-Bit-Ganzzahlen ([`std::sync::atomic::AtomicU64`]) mit `Ordering::SeqCst`
//!   für maximale Konsistenz über Thread-Grenzen hinweg.
//! - Mehrere Threads können die Uhr parallel abfragen oder vorstellen, ohne dass
//!   Sperren (Locks) benötigt werden.

// FILE-CONTEXT
// STAND: 2026-09-19T20:12:00Z (SESSION: 01c5be8b)
// ZWECK: Deterministic Clock Mock for unit and integration testing without real sleep.
// INVARIANTEN: Monotonic time progression, thread-safe via AtomicU64 nanoseconds.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use contextra_ports::Clock;

// ANCHOR[contextra-testkit:MANUAL-CLOCK-STRUCT-001]
/// Thread-sichere, manuell vorstellbare Uhr für reproduzierbare Tests.
///
/// **Zweck:** Ermöglicht deterministische zeitabhängige Tests ohne Abhängigkeit
/// von der Systemuhr und ohne unzuverlässige `sleep`-Aufrufe.
///
/// **Invarianten:**
/// - Der interne Zähler (`nanos`) speichert die Nanosekunden seit dem Unix-Epochenstart.
/// - Operationen sind atomar und `SeqCst`-geordnet (sequenziell konsistent).
#[derive(Debug)]
pub struct ManualClock {
    /// Nanosekunden seit UNIX_EPOCH (1970-01-01 00:00:00 UTC).
    nanos: AtomicU64,
}

impl Default for ManualClock {
    /// Erzeugt eine Uhr, die standardmäßig bei genau 1 Sekunde (1.000.000.000 ns)
    /// nach UNIX_EPOCH startet, um Nullwert-Sonderfälle in Tests zu vermeiden.
    fn default() -> Self {
        Self::new(1_000_000_000) // Start at 1s past epoch by default
    }
}

impl ManualClock {
    // ANCHOR[contextra-testkit:MANUAL-CLOCK-NEW-001]
    /// Erstellt eine neue `ManualClock` mit einem gegebenen Nanosekundenwert.
    ///
    /// **Parameter:**
    /// - `initial_nanos`: Startzeit in Nanosekunden seit UNIX_EPOCH.
    ///
    /// **Rückgabe:** Neue Instanz von [`ManualClock`].
    pub fn new(initial_nanos: u64) -> Self {
        Self {
            nanos: AtomicU64::new(initial_nanos),
        }
    }

    // ANCHOR[contextra-testkit:MANUAL-CLOCK-ADVANCE-001]
    /// Stellt die Uhr um eine bestimmte Dauer vor.
    ///
    /// **Ablauf:**
    /// 1. Konvertiert die [`Duration`] in Nanosekunden (`u128` zu `u64`).
    /// 2. Bei Überlauf wird auf `u64::MAX` abgesichert (Sättigung).
    /// 3. Addiert das Delta atomar mittels `fetch_add` mit `SeqCst`-Ordnung.
    ///
    /// **Nebenwirkungen:**
    /// Erhöht den von allen Threads gesehenen Zeitstempel sofort.
    pub fn advance(&self, duration: Duration) {
        let delta = u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX);
        self.nanos.fetch_add(delta, Ordering::SeqCst);
    }

    // ANCHOR[contextra-testkit:MANUAL-CLOCK-SET-001]
    /// Setzt die Uhr absolut auf einen bestimmten Nanosekunden-Zeitstempel.
    ///
    /// **Hinweis:** Kann die Zeit auch rückwärts setzen; sollte mit Bedacht
    /// genutzt werden, um Monotonie-Invarianten in darauf aufbauenden Systemen
    /// nicht zu verletzen.
    pub fn set_nanos(&self, nanos: u64) {
        self.nanos.store(nanos, Ordering::SeqCst);
    }

    // ANCHOR[contextra-testkit:MANUAL-CLOCK-NOW-001]
    /// Liest den aktuellen Zeitstempel in Nanosekunden seit UNIX_EPOCH aus.
    pub fn now_nanos(&self) -> u64 {
        self.nanos.load(Ordering::SeqCst)
    }

    // ANCHOR[contextra-testkit:MANUAL-CLOCK-NOW-SECS-001]
    /// Gibt den aktuellen Zeitstempel als Fließkommazahl in Sekunden zurück.
    pub fn now_secs_f64(&self) -> f64 {
        self.now_nanos() as f64 / 1_000_000_000.0
    }

    // ANCHOR[contextra-testkit:MANUAL-CLOCK-SYSTEM-TIME-001]
    /// Konvertiert den aktuellen Zustand in einen Standard-Rust-[`SystemTime`]-Zeitpunkt.
    pub fn now_system_time(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_nanos(self.now_nanos())
    }
}

// ANCHOR[contextra-testkit:MANUAL-CLOCK-TRAIT-IMPL-001]
impl Clock for ManualClock {
    /// Liefert die Unix-Nanosekunden gemäß `Clock`-Port-Spezifikation.
    fn now_unix_nanos(&self) -> u64 {
        self.now_nanos()
    }

    /// Liefert eine monotone Nanosekunden-Zählung (entspricht hier `now_nanos`).
    fn monotonic_nanos(&self) -> u64 {
        self.now_nanos()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // ANCHOR[contextra-testkit:TEST-CLOCK-ADVANCE-001]
    // Prüft das relative Vorstellen der Uhr um verschiedene Zeiteinheiten (ns, s).
    #[test]
    fn test_manual_clock_advance() {
        let clock = ManualClock::new(100);
        assert_eq!(clock.now_nanos(), 100);

        clock.advance(Duration::from_nanos(50));
        assert_eq!(clock.now_nanos(), 150);

        clock.advance(Duration::from_secs(1));
        assert_eq!(clock.now_nanos(), 1_000_000_150);
    }

    // ANCHOR[contextra-testkit:TEST-CLOCK-SET-001]
    // Prüft das absolute Setzen der Uhr und die Umrechnung in SystemTime.
    #[test]
    fn test_manual_clock_set() {
        let clock = ManualClock::default();
        clock.set_nanos(42);
        assert_eq!(clock.now_nanos(), 42);
        assert_eq!(
            clock.now_system_time(),
            UNIX_EPOCH + Duration::from_nanos(42)
        );
    }

    // ANCHOR[contextra-testkit:TEST-CLOCK-DYN-001]
    // Prüft die Nutzung von ManualClock über ein Trait-Objekt `&dyn Clock`.
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
}
