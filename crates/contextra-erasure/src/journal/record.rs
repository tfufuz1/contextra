//! Journal-Eintragstypen und Zustandsdefinitionen (SPEC §11.6, Anhang F.9, ER-29).

use serde::{Deserialize, Serialize};

pub use crate::saga::SagaState;

/// Ein Journal-Eintrag (SPEC §11.6, ER-29).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
