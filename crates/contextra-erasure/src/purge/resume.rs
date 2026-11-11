//! Wiederanlauf- und Fortschritts-API für den Purge-Lebenszyklus (P9, WP-2, WP-3, SPEC §11.2, §11.4).
//!
//! Stellt getypte Methoden zur Verfügung, um den Fortschritt eines Purge-Laufes
//! nach einem Crash wiederherzustellen, zu validieren und schrittweise voranzutreiben.

// Herkunft: 4b98cf91b:crates/contextra-store/src/lsm/purge/resume.rs:24

use super::phase::{PurgePhase, PurgePhaseError};
use serde::{Deserialize, Serialize};

/// Persistierbarer Fortschritts-Zustand eines Purge-Laufes für Wiederanlauf (P9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurgeProgressTracker {
    /// Eindeutige Kennung des Purge-Vorgangs.
    pub purge_id: [u8; 16],
    /// Aktuell erreichte und erfolgreich verifizierte Phase.
    pub current_phase: PurgePhase,
    /// Anzahl der Wiederholungs- bzw. Wiederanlaufversuche.
    pub attempts: u32,
    /// Letzte Fehlerbeschreibung (falls vorheriger Versuch fehlschlug).
    pub last_phase_error: Option<PurgePhaseError>,
}

impl PurgeProgressTracker {
    /// Initialisiert einen neuen Fortschrittstracker im Zustand `Admitted`.
    pub fn new(purge_id: [u8; 16]) -> Self {
        Self {
            purge_id,
            current_phase: PurgePhase::Admitted,
            attempts: 1,
            last_phase_error: None,
        }
    }

    /// Stellt einen Tracker aus persistierten Rohdaten wieder her (P9 Resume).
    ///
    /// Validiert den Phasencode fail-closed.
    pub fn recover(
        purge_id: [u8; 16],
        phase_code: u8,
        attempts: u32,
    ) -> Result<Self, PurgePhaseError> {
        let current_phase = PurgePhase::from_code(phase_code)?;
        Ok(Self {
            purge_id,
            current_phase,
            attempts: attempts.saturating_add(1),
            last_phase_error: None,
        })
    }

    /// Gibt die aktuell erreichte Phase zurück.
    pub fn current_phase(&self) -> PurgePhase {
        self.current_phase
    }

    /// Ermittelt die als nächstes auszuführende Phase.
    ///
    /// Liefert `None`, falls der Purge bereits vollständig im Zustand `Released` abgeschlossen ist.
    pub fn next_phase(&self) -> Option<PurgePhase> {
        self.current_phase.next_phase()
    }

    /// Prüft, ob der Purge bereits terminal beendet ist.
    pub fn is_completed(&self) -> bool {
        self.current_phase.is_terminal()
    }

    /// Schließt die aktuelle Phase erfolgreich ab und wechselt auf die Folgephase.
    ///
    /// Schlägt fail-closed fehl, wenn keine Folgephase existiert oder der Übergang unzulässig ist.
    pub fn advance_to_next(&mut self) -> Result<PurgePhase, PurgePhaseError> {
        let next = self.next_phase().ok_or(PurgePhaseError::InvalidTransition {
            from: self.current_phase,
            to: self.current_phase,
        })?;

        self.current_phase.transition_to(next)?;
        self.last_phase_error = None;
        Ok(self.current_phase)
    }

    /// Zeichnet einen Phasenfehler auf.
    pub fn record_failure(&mut self, err: PurgePhaseError) {
        self.last_phase_error = Some(err);
    }
}
