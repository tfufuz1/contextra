//! Journal für Saga-Zustände (SPEC §11.6, ER-29, F.9).

pub use crate::saga::SagaState;
use serde::{Deserialize, Serialize};

/// Ein Eintrag im Löschjournal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Löschungs-ID
    pub erasure_id: [u8; 16],
    /// Erreichter Saga-Zustand
    pub state: SagaState,
    /// Zeitstempel in Mikrosekunden
    pub timestamp: u64,
    /// Commitment-Hash
    pub commit_hash: [u8; 32],
}

/// In-Memory-Abstraktion des Journals (Appendlog nach F.9).
#[derive(Debug, Default)]
pub struct ErasureJournal {
    entries: Vec<JournalEntry>,
}

impl ErasureJournal {
    /// Erstellt ein neues leeres Journal.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Fügt einen neuen Eintrag an.
    pub fn append(&mut self, entry: JournalEntry) {
        self.entries.push(entry);
    }

    /// Gibt alle Einträge zurück.
    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }
}
