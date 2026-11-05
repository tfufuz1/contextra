//! Journal für Saga-Zustände (SPEC §11.6, ER-29, F.9).

pub mod appendlog;
pub mod epoch;
pub mod record;
pub mod reduce;

pub use crate::saga::SagaState;
pub use appendlog::{AppendLog, DurableJournal};
pub use epoch::{compute_commitment, Epoch, EpochKeys};
pub use record::{JournalEntry, JournalRecord, RecordCode, TargetType};
pub use reduce::reduce;

use crate::error::ErasureError;
use crate::saga::EraseSaga;
use contextra_ports::vfs::Vfs;
use contextra_types::ids::ErasureId;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

/// In-Memory- und Disk-Abstraktion des Journals (Appendlog nach F.9).
#[derive(Debug, Default)]
pub struct ErasureJournal {
    entries: Vec<JournalEntry>,
}

impl ErasureJournal {
    /// Erstellt ein neues leeres In-Memory-Journal.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Öffnet ein dauerhaftes Journal auf Vfs.
    pub fn open_durable(
        vfs: Arc<dyn Vfs>,
        path: &Path,
        derived_key: &[u8; 32],
    ) -> Result<DurableJournal, ErasureError> {
        DurableJournal::open_or_create(vfs, path, derived_key)
    }

    /// Fügt einen neuen Eintrag an.
    pub fn append(&mut self, entry: JournalEntry) {
        self.entries.push(entry);
    }

    /// Gibt alle Einträge zurück.
    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
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
