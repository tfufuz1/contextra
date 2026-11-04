//! Persistentes Append-Log des Löschjournals journal.log (SPEC §11.6, Anhang F.9, ER-29).

use super::record::JournalEntry;
use crate::saga::{EraseSaga, SagaState};
use contextra_types::ids::ErasureId;
use std::collections::BTreeMap;

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

    /// Rekonstruiert den Zustand aller noch nicht abgeschlossenen Sagas aus dem Journal (SPEC §11.2, §11.6).
    ///
    /// Jede nicht-vollendete Löschung (nicht `Completed` oder `Refused`) wird zurückgegeben,
    /// damit sie beim nächsten Schritt fortgesetzt werden kann.
    pub fn recover_open_sagas(&self) -> Vec<EraseSaga> {
        let mut latest_state: BTreeMap<[u8; 16], (SagaState, u64)> = BTreeMap::new();
        for e in &self.entries {
            latest_state.insert(e.erasure_id, (e.state, e.timestamp));
        }

        let mut recovered = Vec::new();
        for (id_bytes, (state, _ts)) in latest_state {
            if !state.is_terminal() {
                recovered.push(EraseSaga::with_state(ErasureId::new(id_bytes), state));
            }
        }
        recovered
    }
}

impl Default for AppendLog {
    fn default() -> Self {
        Self::new()
    }
}
