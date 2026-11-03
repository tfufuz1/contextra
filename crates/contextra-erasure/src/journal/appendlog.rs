//! Persistentes Append-Log des Löschjournals journal.log (SPEC §11.6, Anhang F.9, ER-29).

use super::record::JournalEntry;

/// In-Memory-Journal-Implementierung (vollständige Disk-Implementierung in WP-4.2).
pub struct AppendLog {
    entries: Vec<JournalEntry>,
}

impl AppendLog {
    /// Erstellt ein leeres Log.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Fügt einen Eintrag hinzu.
    pub fn append(&mut self, entry: JournalEntry) {
        self.entries.push(entry);
    }

    /// Liefert alle Einträge.
    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }

    /// Anzahl der Einträge.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Prüft ob das Log leer ist.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for AppendLog {
    fn default() -> Self {
        Self::new()
    }
}
