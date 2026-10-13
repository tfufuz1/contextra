use super::*;
use contextra_core::Result;

impl LsmStorage {
    /// Flushes the active MemTable to disk as an SSTable.
    pub async fn flush(&self) -> Result<()> {
        if self.is_read_only {
            return Err(ContextraError::Storage(format!(
                "Storage engine is in ReadOnly mode due to erasure ledger status ({:?})",
                self.ledger_state
            )));
        }
        StorageEngine::flush(self).await
    }

    // TODO(wiring): Facade-Anbindung in contextra/src/builder.rs folgt in separatem Task
    pub async fn force_flush(&self) -> Result<()> {
        self.flush().await
    }

    /// Evaluates whether compaction should run and performs it if needed.
    /// Uses the persistent CompactionEngine instance to maintain a stable counter
    /// across calls, preventing SSTable filename collisions (audit H-3).
    #[doc(hidden)]
    pub async fn maybe_compact(&self) -> Result<bool> {
        self.compaction_engine
            .maybe_compact(&self.sstables, &self.config.path)
            .await
    }
}
