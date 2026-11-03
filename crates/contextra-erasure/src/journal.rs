//! Journal-Modul (SPEC §11.6).
pub mod appendlog;
pub mod record;
pub mod reduce;
pub mod epoch;

pub use record::{JournalEntry, SagaState};
pub use appendlog::AppendLog;
pub use epoch::Epoch;

/// In-Memory-Journal für Löschvorgänge (Fassade über `AppendLog`).
pub struct ErasureJournal {
    log: AppendLog,
}

impl ErasureJournal {
    /// Erstellt ein leeres Journal.
    pub fn new() -> Self {
        Self {
            log: AppendLog::new(),
        }
    }

    /// Fügt einen Eintrag hinzu.
    pub fn append(&mut self, entry: JournalEntry) {
        self.log.append(entry);
    }

    /// Liefert alle Einträge.
    pub fn entries(&self) -> &[JournalEntry] {
        self.log.entries()
    }
}

impl Default for ErasureJournal {
    fn default() -> Self {
        Self::new()
    }
}
