//! # Epochenverwaltung und Klon-Divergenzschutz
//!
//! SPEC §15.2, SD-63, §18 (INV-EPOCH-1):
//! Schlüsselepoch-Zähler für beschreibbare Klone und Checkpoints.
//! Stellt sicher, dass Original und Klon unter gleicher ID und Nonce niemals
//! mit demselben Schlüssel verschiedene Daten versiegeln (INV-EPOCH-1).

use std::fmt;

/// Repräsentiert die Epoche einer Schlüsselableitung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyEpoch(pub u32);

impl KeyEpoch {
    /// Die initiale Epoche bei Neuerstellung einer Instanz.
    pub const INITIAL: Self = Self(0);

    /// Erstellt eine Epoche mit dem angegebenen Zähler.
    pub const fn new(epoch: u32) -> Self {
        Self(epoch)
    }

    /// Liefert den numerischen Epochenwert.
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Erhöht die Epoche um 1 (z. B. beim Anlegen eines beschreibbaren Klons).
    // INV-EPOCH-1: Beschreibbare Klone erhalten stets key_epoch + 1.
    pub fn next_clone_epoch(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl fmt::Display for KeyEpoch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KeyEpoch({})", self.0)
    }
}

/// Prüft und verwaltet die Gültigkeit von Epochen bei der Schlüsselableitung.
#[derive(Debug, Clone)]
pub struct EpochTracker {
    current_epoch: KeyEpoch,
}

impl EpochTracker {
    /// Initialisiert den Tracker mit der Startepoche.
    pub fn new(epoch: KeyEpoch) -> Self {
        Self {
            current_epoch: epoch,
        }
    }

    /// Liefert die aktuelle Epoche für neue Schreibvorgänge.
    pub fn current_epoch(&self) -> KeyEpoch {
        self.current_epoch
    }

    /// Führt einen Epochensprung für einen Klon durch.
    // INV-EPOCH-1: Neuer Klon erhält strikt inkrementierte Epoche.
    pub fn advance_for_clone(&mut self) -> KeyEpoch {
        self.current_epoch = self.current_epoch.next_clone_epoch();
        self.current_epoch
    }

    /// Prüft, ob eine gelesene Dateiepoche gültig ist (darf nicht in der Zukunft liegen).
    pub fn is_valid_read_epoch(&self, epoch: KeyEpoch) -> bool {
        epoch <= self.current_epoch
    }

    /// Prüft, ob eine Epoche für neue Dateien zulässig ist (muss der aktuellen Epoche entsprechen).
    pub fn is_valid_write_epoch(&self, epoch: KeyEpoch) -> bool {
        epoch == self.current_epoch
    }
}
