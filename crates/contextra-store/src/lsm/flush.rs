//! # MemTable-Flush-Mechanismus (Ring 1)
//!
//! Schreibt volle MemTables als neue L0-SSTables asynchron auf Platte.

use super::*;
use contextra_types::Result;

impl LsmStorage {
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
