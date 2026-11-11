//! # PurgePhase: Physische Bereinigungs-Zustandsmaschine (WP-2 / SPEC §11.4).
//!
//! Kapselt die Phasen des physischen Purge mit expliziter `can_transition_to`-Prüfung
//! und Fail-Closed-Verhalten bei unzulässigen Übergängen.

use crate::error::ErasureError;
use serde::{Deserialize, Serialize};

/// Phasen des physischen Purge-Prozesses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PurgePhase {
    /// Initialer Zustand vor Beginn der Bereinigung.
    Init,
    /// Lese-/Schreib-Fence und ReadGate sind aktiv gesetzt.
    Gated,
    /// Krypto-Shredding: DEK/SubKey im KeyStore vernichtet.
    KeysDestroyed,
    /// Physischer Speicher und Segmente (SSTables, WAL, Cache) bereinigt.
    StoragePurged,
    /// Unabhängige Restmengen-Messung und Scanner-Zeugen verifiziert.
    Verified,
    /// Purge erfolgreich abgeschlossen und im Ledger verankert.
    Complete,
    /// Bereinigung fehlgeschlagen; Fail-Closed Zustand.
    Failed,
}

impl PurgePhase {
    /// Prüft, ob ein Übergang von `self` zu `target` nach dem Protokoll zulässig ist.
    pub fn can_transition_to(&self, target: PurgePhase) -> bool {
        match (self, target) {
            // Aus Failed gibt es keine regulären Übergänge mehr (Terminalzustand)
            (PurgePhase::Failed, _) => false,
            // Aus Complete gibt es keine weiteren Übergänge (Terminalzustand)
            (PurgePhase::Complete, _) => false,

            // Zu Failed kann aus jeder nicht-terminalen Phase gewechselt werden
            (_, PurgePhase::Failed) => true,

            // Reguläre Vorwärts-Pipeline:
            (PurgePhase::Init, PurgePhase::Gated) => true,
            (PurgePhase::Gated, PurgePhase::KeysDestroyed) => true,
            (PurgePhase::KeysDestroyed, PurgePhase::StoragePurged) => true,
            (PurgePhase::StoragePurged, PurgePhase::Verified) => true,
            (PurgePhase::Verified, PurgePhase::Complete) => true,

            // Idempotente Übergänge auf sich selbst sind erlaubt
            (a, b) if *a == b => true,

            // Alle anderen Übergänge sind unzulässig
            _ => false,
        }
    }

    /// Führt den Phasenübergang durch oder liefert einen Fehler der Fehlertaxonomie.
    pub fn transition_to(&mut self, target: PurgePhase) -> Result<(), ErasureError> {
        if self.can_transition_to(target) {
            *self = target;
            Ok(())
        } else {
            Err(ErasureError::PurgePhaseTransition {
                from: *self,
                to: target,
            })
        }
    }

    /// Gibt an, ob der Purge erfolgreich abgeschlossen ist.
    pub fn is_complete(&self) -> bool {
        matches!(self, PurgePhase::Complete)
    }

    /// Gibt an, ob sich der Purge in einem Fehlerzustand befindet.
    pub fn is_failed(&self) -> bool {
        matches!(self, PurgePhase::Failed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purge_phase_valid_flow() {
        let mut phase = PurgePhase::Init;
        assert!(phase.can_transition_to(PurgePhase::Gated));
        phase.transition_to(PurgePhase::Gated).unwrap();

        assert!(phase.can_transition_to(PurgePhase::KeysDestroyed));
        phase.transition_to(PurgePhase::KeysDestroyed).unwrap();

        assert!(phase.can_transition_to(PurgePhase::StoragePurged));
        phase.transition_to(PurgePhase::StoragePurged).unwrap();

        assert!(phase.can_transition_to(PurgePhase::Verified));
        phase.transition_to(PurgePhase::Verified).unwrap();

        assert!(phase.can_transition_to(PurgePhase::Complete));
        phase.transition_to(PurgePhase::Complete).unwrap();

        assert!(phase.is_complete());
        // Complete ist terminal
        assert!(!phase.can_transition_to(PurgePhase::Init));
        assert!(!phase.can_transition_to(PurgePhase::Failed));
    }

    #[test]
    fn test_purge_phase_fail_closed() {
        let mut phase = PurgePhase::Gated;
        assert!(phase.can_transition_to(PurgePhase::Failed));
        phase.transition_to(PurgePhase::Failed).unwrap();
        assert!(phase.is_failed());

        // Aus Failed kein Übergang mehr
        assert!(!phase.can_transition_to(PurgePhase::Init));
        assert!(!phase.can_transition_to(PurgePhase::Complete));
        assert!(phase.transition_to(PurgePhase::Complete).is_err());
    }

    #[test]
    fn test_purge_phase_illegal_skip() {
        let mut phase = PurgePhase::Init;
        // Überspringen von Phasen ist verboten
        assert!(!phase.can_transition_to(PurgePhase::StoragePurged));
        assert!(!phase.can_transition_to(PurgePhase::Complete));
        assert!(phase.transition_to(PurgePhase::Complete).is_err());
    }
}
