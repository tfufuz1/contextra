// FILE-CONTEXT
// STAND: 2026-10-11
// ZWECK: Implementation von LsmStorage::purge_keys (Etappe 4, Spec C5.4).

use crate::compaction::purge_filter::{FreeSpaceProbe, PurgeBlocked, PurgeReport, PurgeScope};
use crate::lsm::engine::LsmStorage;
use contextra_core::StorageEngine;

impl LsmStorage {
    /// Führt Vorprüfungen (Active-Snapshot-Pin, Disk-Space, Memtable-Flush) durch
    /// und bricht bewusst mit `Err(PurgeBlocked::Unimplemented)` ab, bis die reale
    /// Compaction-basierte Purge-Implementierung (WP-1.3) bereitsteht.
    pub async fn purge_keys(
        &self,
        _scope: PurgeScope,
        as_of_tx: u64,
        space_probe: Option<&dyn FreeSpaceProbe>,
    ) -> std::result::Result<PurgeReport, PurgeBlocked> {
        // 1. Active snapshot pin check
        let min_active_seq = self.snapshot_registry.min_active_seqno();
        if min_active_seq <= as_of_tx {
            return Err(PurgeBlocked::ActiveSnapshot {
                min_seq: min_active_seq,
            });
        }

        // 2. Space probe check
        let free_space = space_probe.and_then(|p| p.free_space_bytes()).unwrap_or(0);

        // 3. Acquire write locks / flush active memtable to SSTable first
        StorageEngine::flush(self)
            .await
            .map_err(|e| PurgeBlocked::FlushFailed(e.to_string()))?;

        // 4. Re-select candidates after flush to include any newly created SSTable
        let sstables_guard = self.sstables.read().await;
        let candidates = select_purge_candidates(&sstables_guard, &scope, space_probe)?;
        drop(sstables_guard);

        if candidates.is_empty() {
            return Ok(PurgeReport::default());
        }

        // 5. Generate output path and run Purge Compaction
        let output_path = self
            .compaction_engine
            .generate_sst_path(&self.config.path)
            .map_err(|_| PurgeBlocked::CompactionBusy)?;

        let report = self
            .compaction_engine
            .purge_compact_sstables(&candidates, &output_path, &scope, as_of_tx, None)
            .await
            .map_err(|_| PurgeBlocked::CompactionBusy)?;

        // 6. In-memory state swap and MANIFEST atomic commit under write lock
        let mut sstables_guard = self.sstables.write().await;

        let is_valid_window = candidates
            .iter()
            .all(|cand| sstables_guard.iter().any(|s| Arc::ptr_eq(s, cand)));

        if !is_valid_window {
            if !report.files_written.is_empty() {
                let p = output_path.clone();
                let _ =
                    tokio::task::spawn_blocking(move || contextra_durable_fs::durable_remove(&p))
                        .await;
            }
            return Err(PurgeBlocked::CompactionBusy);
        }

        // 3. Acquire write locks / flush active memtable to SSTable first
        StorageEngine::flush(self)
            .await
            .map_err(|_| PurgeBlocked::CompactionBusy)?;

        // WP-0.1: Make purge_keys fail-closed until WP-1.3 (real compaction & footprint filter) is implemented.
        Err(PurgeBlocked::Unimplemented)
    }
}
