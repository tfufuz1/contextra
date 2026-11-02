//! Ledger und Audit-Sink außerhalb des Datenverzeichnisses (SPEC §11.8, ER-30).

use contextra_ports::ledger::{LedgerEntry, LedgerSink, Timestamp};
use parking_lot::RwLock;
use std::sync::Arc;

/// In-Memory-Implementierung von `LedgerSink`.
#[derive(Debug, Default, Clone)]
pub struct InMemoryLedger {
    entries: Arc<RwLock<Vec<LedgerEntry>>>,
}

impl InMemoryLedger {
    /// Erstellt ein neues leeres In-Memory-Ledger.
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl LedgerSink for InMemoryLedger {
    fn append(&self, e: &LedgerEntry) -> Result<(), String> {
        let mut entries = self.entries.write();
        entries.push(e.clone());
        Ok(())
    }

    fn read_all(&self) -> Result<Vec<LedgerEntry>, String> {
        let entries = self.entries.read();
        Ok(entries.clone())
    }

    fn prune(&self, older_than: Timestamp) -> Result<usize, String> {
        let mut entries = self.entries.write();
        let initial_len = entries.len();
        entries.retain(|e| e.completed_at >= older_than);
        Ok(initial_len.saturating_sub(entries.len()))
    }
}
