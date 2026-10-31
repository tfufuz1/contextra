//! # Health-Status und Zustandsautomat für Speichersysteme
//!
//! Zweck: Implementierung eines Fail-Closed-Zustandsautomaten ([`HealthState`]) und
//! eines threadsicheren atomaren Wrappers ([`AtomicHealthState`]) zur Überwachung
//! der Integrität des Speichersubstrats gemäß Spezifikation §18.4.
//!
//! Ring-Zugehörigkeit: Ring 0 (Basisinfrastruktur / Fail-Closed-Zustandsverwaltung).
//! Wird genutzt von: [`crate::PendingDeleteJournal`] und übergeordneten Speicher-Crates.
//! Wichtigste Typen:
//! - [`HealthState`]: Die vier diskreten Betriebszustände (`Healthy`, `Degraded`, `ReadOnly`, `Poisoned`).
//! - [`HealthEvent`]: Ereignisse, die Statusübergänge auslösen (`FsyncFailure`, `Enospc`, `ReadChecksumWarning`, `Recovered`).
//! - [`AtomicHealthState`]: Lock-freie atomare Speicherung für nebenläufigen Zugriff via Compare-and-Swap.
//!
//! Invarianten:
//! - `INV-POISON-1`: Einmal im Zustand `Poisoned` (vergiftet durch Flush-/fsync-Fehler), kann dieser Zustand
//!   während der Prozesslaufzeit niemals wieder verlassen werden (dauerhaft fail-closed).
//! - Nebenläufigkeit: Lock-freier CAS-Loop (`compare_exchange_weak`) mit `AcqRel`/`Acquire`-Ordnung.

// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Fail-Closed HealthState Zustandsautomat fuer Durability Substrat (§18.4).
// INVARIANTEN: Healthy Startzustand, Poisoned verlaesst Zustand nie automatisch ohne Neustart (I-POISON-1).

use std::sync::atomic::{AtomicU8, Ordering};

// ANCHOR[contextra-durable-fs:HEALTH-STATE-001]
/// Einstufung des Systemzustands für das Speichersubstrat (§18.4).
///
/// Definiert einen deterministischen Zustandsübergang bei Hardware- oder I/O-Ereignissen.
///
/// ## Übergangstabelle (Transition Table)
///
/// | Ausgangszustand | Auslösendes Ereignis ([`HealthEvent`]) | Zielzustand | Dauerhaft? | Erforderliche Maßnahme |
/// |----------------|--------------------------------------|-------------|------------|------------------------|
/// | `Healthy`      | Lesefehler / Prüfsummen-Warnung      | `Degraded`  | Nein       | Scrubbing / Wiederholung |
/// | `Healthy`      | Speicher voll (`ENOSPC`)             | `ReadOnly`  | Nein       | Speicherfreigabe / Löschen |
/// | Beliebig       | Flush / `fsync`-Schreibfehler        | `Poisoned`  | Ja         | Prozessneustart + Recovery |
/// | `Degraded`     | Flush / `fsync`-Schreibfehler        | `Poisoned`  | Ja         | Prozessneustart + Recovery |
/// | `ReadOnly`     | Flush / `fsync`-Schreibfehler        | `Poisoned`  | Ja         | Prozessneustart + Recovery |
/// | `Poisoned`     | Jedes Ereignis                       | `Poisoned`  | Ja         | Schreiboperationen strikt abweisen |
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(u8)]
pub enum HealthState {
    /// Normaler Betriebszustand mit vollständigen Lese- und Schreibfähigkeiten.
    #[default]
    Healthy = 0,
    /// Nicht-fatale Lesefehler oder Prüfsummenabweichungen aufgetreten; Schreiben erlaubt, Lesen eingeschränkt.
    Degraded = 1,
    /// Speichermedium voll (`ENOSPC`); Schreib- und Commit-Operationen abgewiesen, Lesen weiterhin gestattet.
    ReadOnly = 2,
    /// Fataler Flush- oder `fsync`-Fehler; alle Schreibzugriffe verboten. Erfordert vollständigen Neustart und WAL-Recovery.
    Poisoned = 3,
}

impl HealthState {
    // ANCHOR[contextra-durable-fs:HEALTH-STATE-NEW]
    /// Erzeugt einen neuen [`HealthState`] im Ausgangszustand `Healthy`.
    #[must_use]
    pub const fn new() -> Self {
        Self::Healthy
    }

    // ANCHOR[contextra-durable-fs:HEALTH-STATE-TRANSITION]
    /// Berechnet den Folgezustand basierend auf einem eingehenden [`HealthEvent`].
    ///
    /// Ablauf:
    /// 1. Prüfen, ob der Zustand bereits `Poisoned` ist.
    /// 2. Falls `Poisoned`, verbleibt das System zwingend in `Poisoned` (Invariante `INV-POISON-1`).
    /// 3. Andernfalls Zuordnung gemäß Übergangstabelle:
    ///    - `FsyncFailure` -> `Poisoned`
    ///    - `Enospc` -> `ReadOnly`
    ///    - `ReadChecksumWarning` -> `Degraded` (außer wenn bereits `ReadOnly`)
    ///    - `Recovered` -> `Healthy`
    ///
    /// Vorbedingungen:
    /// - Keine.
    ///
    /// Nachbedingungen:
    /// - Ein `Poisoned`-Zustand wird niemals degradiert oder geheilt.
    #[must_use]
    pub fn transition(self, event: HealthEvent) -> Self {
        // NOTE: Ein vergifteter Zustand darf zur Vermeidung von Datenkorruption niemals verlassen werden.
        if self == Self::Poisoned {
            return Self::Poisoned;
        }

        match event {
            HealthEvent::FsyncFailure => Self::Poisoned,
            HealthEvent::Enospc => Self::ReadOnly,
            HealthEvent::ReadChecksumWarning => {
                if self == Self::ReadOnly {
                    Self::ReadOnly
                } else {
                    Self::Degraded
                }
            }
            HealthEvent::Recovered => Self::Healthy,
        }
    }

    // ANCHOR[contextra-durable-fs:HEALTH-STATE-ALLOWS-WRITES]
    /// Gibt an, ob im aktuellen Zustand mutierende Schreiboperationen zulässig sind.
    ///
    /// Rückgabewert:
    /// - `true` für `Healthy` und `Degraded`.
    /// - `false` für `ReadOnly` (Speicher voll) und `Poisoned` (Hardware/Sync-Fehler).
    #[must_use]
    pub fn allows_writes(self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded)
    }

    // ANCHOR[contextra-durable-fs:HEALTH-STATE-IS-POISONED]
    /// Prüft, ob das Speichersubstrat dauerhaft vergiftet (`Poisoned`) ist.
    ///
    /// Rückgabewert:
    /// - `true`, falls der Zustand `Poisoned` ist.
    #[must_use]
    pub fn is_poisoned(self) -> bool {
        matches!(self, Self::Poisoned)
    }
}

// ANCHOR[contextra-durable-fs:HEALTH-EVENT-001]
/// Ereignisse und Auslöser für Statusübergänge im Speichersystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthEvent {
    /// Fataler Ausfall während eines synchronen Festplatten-Flushes (`fsync`).
    FsyncFailure,
    /// Speichermedium meldet fehlenden Speicherplatz (`ENOSPC` = Error No Space).
    Enospc,
    /// Nicht-fataler Lesefehler oder Warnung bei der Block-Prüfsummenverifikation.
    ReadChecksumWarning,
    /// Explizites Wiederherstellungssignal (z. B. nach erfolgreicher Crash-Recovery beim Neustart).
    Recovered,
}

// ANCHOR[contextra-durable-fs:ATOMIC-HEALTH-STATE-001]
/// Threadsicherer, atomarer Wrapper für den aktuellen [`HealthState`].
///
/// Ermöglicht nebenläufigen Threads das unterbrechungsfreie Lesen und
/// konsistente Aktualisieren des Systemzustands ohne Mutex-Sperren.
#[derive(Debug)]
pub struct AtomicHealthState {
    state: AtomicU8,
}

impl Default for AtomicHealthState {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomicHealthState {
    // ANCHOR[contextra-durable-fs:ATOMIC-HEALTH-STATE-NEW]
    /// Initialisiert eine neue atomare Zustandsverwaltung im Startzustand `Healthy`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: AtomicU8::new(HealthState::Healthy as u8),
        }
    }

    // ANCHOR[contextra-durable-fs:ATOMIC-HEALTH-STATE-LOAD]
    /// Liest den aktuellen [`HealthState`] mit `Acquire`-Speicherordnung aus.
    ///
    /// Nachbedingungen:
    /// - Garantiert, dass nachfolgende Leseoperationen im aufrufenden Thread
    ///   die Speicheränderungen sehen, die vor dem Setzen des Zustands erfolgten.
    #[must_use]
    pub fn load(&self) -> HealthState {
        match self.state.load(Ordering::Acquire) {
            0 => HealthState::Healthy,
            1 => HealthState::Degraded,
            2 => HealthState::ReadOnly,
            // NOTE: Alle unbekannten oder höheren Werte werden sicherheitshalber als Poisoned interpretiert (Fail-Closed).
            _ => HealthState::Poisoned,
        }
    }

    // ANCHOR[contextra-durable-fs:ATOMIC-HEALTH-STATE-RECORD-EVENT]
    /// Führt einen atomaren Zustandsübergang anhand eines [`HealthEvent`] aus.
    ///
    /// Ablauf:
    /// 1. Aktuellen Zustand atomar laden.
    /// 2. Folgezustand über [`HealthState::transition`] berechnen.
    /// 3. Mittels `compare_exchange_weak` (CAS) versuchen, den neuen Zustand zu schreiben.
    /// 4. Bei Kollision mit anderem Thread: Schleife mit dem tatsächlichen aktuellen Zustand wiederholen.
    ///
    /// Nebenläufigkeit:
    /// - Verwendet lock-freie CAS-Schleife mit `Ordering::AcqRel` bei Erfolg und `Ordering::Acquire` bei Fehlschlag.
    pub fn record_event(&self, event: HealthEvent) -> HealthState {
        let mut current = self.load();
        loop {
            let next = current.transition(event);
            match self.state.compare_exchange_weak(
                current as u8,
                next as u8,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return next,
                Err(actual) => {
                    current = match actual {
                        0 => HealthState::Healthy,
                        1 => HealthState::Degraded,
                        2 => HealthState::ReadOnly,
                        _ => HealthState::Poisoned,
                    };
                }
            }
        }
    }
}
