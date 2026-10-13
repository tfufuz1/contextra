// FILE-CONTEXT
// STAND: 2026-10-12 (T-2026-9205)
// ZWECK: Implementation von LsmStorage::purge_keys (Purge-Compaction, Spec C5.4).

use crate::compaction::purge_filter::{FreeSpaceProbe, PurgeBlocked, PurgeReport, PurgeScope};
use crate::compaction::purge_select::select_purge_candidates;
use crate::lsm::engine::LsmStorage;
use crate::sstable::SstableReader;
use contextra_core::{StorageEngine, TOMBSTONE_BIT};
use std::sync::Arc;

impl LsmStorage {
    /// Führt Vorprüfungen (Active-Snapshot-Pin, Disk-Space, Memtable-Flush) und die
    /// echte Compaction-basierte Purge-Implementierung (Spec C5.4) durch.
    pub async fn purge_keys(
        &self,
        scope: PurgeScope,
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

        // 2. Space probe check & candidate selection on currently live SSTables
        let sstables_guard = self.sstables.read().await;
        let _initial_candidates = select_purge_candidates(&sstables_guard, &scope, space_probe)?;
        drop(sstables_guard);

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

        let manifest = &self.manifest;
        if let Some(written_path) = report.files_written.first() {
            let new_reader = SstableReader::open_with_key_manager(
                written_path,
                Arc::clone(&self.block_cache),
                self.key_manager.clone(),
            )
            .await
            .map_err(|_| PurgeBlocked::CompactionBusy)?;

            manifest
                .append(&crate::manifest::ManifestEntry::Replace {
                    removed: report.obsolete_files.clone(),
                    added: written_path.clone(),
                    added_max_tx: new_reader.metadata().max_tx_id,
                    rank: 0,
                })
                .await
                .map_err(|_| PurgeBlocked::CompactionBusy)?;

            sstables_guard.retain(|sst| !candidates.iter().any(|cand| Arc::ptr_eq(cand, sst)));
            sstables_guard.push(Arc::new(new_reader));
            sstables_guard.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT);
        } else {
            let entries: Vec<crate::manifest::ManifestEntry> = report
                .obsolete_files
                .iter()
                .map(|p| crate::manifest::ManifestEntry::Remove { path: p.clone() })
                .collect();

            manifest
                .append_batch(&entries)
                .await
                .map_err(|_| PurgeBlocked::CompactionBusy)?;

            sstables_guard.retain(|sst| !candidates.iter().any(|cand| Arc::ptr_eq(cand, sst)));
        }

        drop(sstables_guard);

        Ok(report)
    }
}
