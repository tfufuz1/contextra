//! # Health-Status und Zustandsautomat für Speichersysteme
//!
//! Zweck: Implementierung eines Fail-Closed-Zustandsautomaten ([`HealthState`]) und
//! eines threadsicheren atomaren Wrappers ([`AtomicHealthState`]) zur Überwachung
//! der Integrität des Speichersubstrats gemäß Spezifikation §18.4.
//!
//! Invarianten:
//! - `INV-FSYNC-1` / `INV-POISON-1`: Schlägt ein Flush/fsync fehl, wechselt das System
//!   unwiderruflich in den Zustand `Poisoned`. Ein Verlassen des Zustands ohne Neustart
//!   ist strikt verboten (Fail-Closed).
//! - Nebenläufigkeit: Lock-freier CAS-Loop (`compare_exchange_weak`) mit `AcqRel`/`Acquire`-Ordnung.

use std::sync::atomic::{AtomicU8, Ordering};

/// Einstufung des Systemzustands für das Speichersubstrat (§18.4).
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
    /// Fataler Flush- oder `fsync`-Fehler; alle Schreibzugriffe dauerhaft abgewiesen (`INV-FSYNC-1`).
    Poisoned = 3,
}

impl HealthState {
    /// Erzeugt einen neuen [`HealthState`] im Ausgangszustand `Healthy`.
    #[must_use]
    pub const fn new() -> Self {
        Self::Healthy
    }

    /// Berechnet den Folgezustand basierend auf einem eingehenden [`HealthEvent`].
    #[must_use]
    pub fn transition(self, event: HealthEvent) -> Self {
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

    /// Gibt an, ob im aktuellen Zustand mutierende Schreiboperationen zulässig sind.
    #[must_use]
    pub fn allows_writes(self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded)
    }

    /// Prüft, ob das Speichersubstrat dauerhaft vergiftet (`Poisoned`) ist.
    #[must_use]
    pub fn is_poisoned(self) -> bool {
        matches!(self, Self::Poisoned)
    }
}

/// Ereignisse und Auslöser für Statusübergänge im Speichersystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthEvent {
    /// Fataler Ausfall während eines synchronen Festplatten-Flushes (`fsync`).
    FsyncFailure,
    /// Speichermedium meldet fehlenden Speicherplatz (`ENOSPC`).
    Enospc,
    /// Nicht-fataler Lesefehler oder Warnung bei der Block-Prüfsummenverifikation.
    ReadChecksumWarning,
    /// Explizites Wiederherstellungssignal (z. B. nach erfolgreicher Crash-Recovery beim Neustart).
    Recovered,
}

/// Threadsicherer, atomarer Wrapper für den aktuellen [`HealthState`].
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
    /// Initialisiert eine neue atomare Zustandsverwaltung im Startzustand `Healthy`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: AtomicU8::new(HealthState::Healthy as u8),
        }
    }

    /// Liest den aktuellen [`HealthState`] mit `Acquire`-Speicherordnung aus.
    #[must_use]
    pub fn load(&self) -> HealthState {
        match self.state.load(Ordering::Acquire) {
            0 => HealthState::Healthy,
            1 => HealthState::Degraded,
            2 => HealthState::ReadOnly,
            _ => HealthState::Poisoned,
        }
    }

    /// Führt einen atomaren Zustandsübergang anhand eines [`HealthEvent`] aus.
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
