//! Journal-Eintragstypen und Zustandsdefinitionen (SPEC §11.6, Anhang F.9, ER-29).

use serde::{Deserialize, Serialize};

/// Zustände des Löschabläufs (SPEC 11.2, ER-29).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum SagaState {
    /// Löschauftrag eingegangen.
    Requested,
    /// Gate gesetzt, Schlüsselvernichtung bevorsteht.
    Gated,
    /// Schlüssel vernichtet.
    KeyDestroyed,
    /// Purge-Phase läuft.
    Purging,
    /// Verifikations-Phase läuft.
    Verifying,
    /// Löschung vollständig abgeschlossen.
    Completed,
    /// Verzögerung (temporär, mit Grund).
    Pending,
    /// Endgültig verweigert (terminal).
    Refused,
}

/// Ein Journal-Eintrag (SPEC §11.6, ER-29).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Lösch-ID.
    pub erasure_id: [u8; 16],
    /// Saga-Zustand.
    pub state: SagaState,
    /// Zeitstempel (Mikrosekunden UTC).
    pub timestamp: u64,
    /// Commitment-Hash (BLAKE3-keyed).
    pub commit_hash: [u8; 32],
}
