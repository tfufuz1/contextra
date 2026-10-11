// FILE-CONTEXT
// STAND: 2026-10-11
// ZWECK: Implementation von LsmStorage::purge_keys (Etappe 4, Spec C5.4).

use crate::compaction::purge_filter::{FreeSpaceProbe, PurgeBlocked, PurgeReport, PurgeScope};
use crate::lsm::engine::LsmStorage;
use contextra_core::StorageEngine;

impl LsmStorage {
    /// Executes a purge-compaction across SSTable levels for the specified `scope` up to transaction `as_of_tx`.
    /// Fail-closed space checks and active snapshot pin verification are strictly enforced.
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
        let free_space = space_probe
            .and_then(|p| p.free_space_bytes())
            .unwrap_or(0);

        if free_space == 0 {
            return Err(PurgeBlocked::InsufficientDiskSpace {
                needed: 1024 * 1024,
                free: 0,
            });
        }

        // 3. Acquire write locks / flush active memtable to SSTable first
        StorageEngine::flush(self)
            .await
            .map_err(|_| PurgeBlocked::CompactionBusy)?;

        // WP-0.1: Make purge_keys fail-closed until WP-1.3 (real compaction & footprint filter) is implemented.
        Err(PurgeBlocked::Unimplemented)
    }
}
