// FILE-CONTEXT
// STAND: 2026-10-11
// ZWECK: Orchestriert den Purge-Ablauf über Phasen P3..P6 (Intent, Flush, Compaction, Dispose).

use super::plan::PurgePhase;
use crate::compaction::purge_filter::{FreeSpaceProbe, PurgeBlocked, PurgeReport, PurgeScope};
use crate::lsm::engine::LsmStorage;
use contextra_core::Result;
use contextra_durable_fs::atomic_replace;
use std::path::PathBuf;

/// Orchestrates the execution of a multi-phase purge pipeline.
pub struct PurgeOrchestrator<'a> {
    storage: &'a LsmStorage,
    intent_file: PathBuf,
}

impl<'a> PurgeOrchestrator<'a> {
    /// Creates a new `PurgeOrchestrator` instance for `storage`.
    pub fn new(storage: &'a LsmStorage) -> Self {
        let intent_file = storage.config.path.join("purge.intent");
        Self {
            storage,
            intent_file,
        }
    }

    /// Executes the complete Purge sequence through phases P3..P6 with durable logging.
    pub async fn execute_purge(
        &self,
        scope: PurgeScope,
        as_of_tx: u64,
        space_probe: Option<&dyn FreeSpaceProbe>,
        wal_callback: Option<Box<dyn FnOnce() -> Result<()> + Send>>,
    ) -> std::result::Result<PurgeReport, PurgeBlocked> {
        // P1: Record intent durably
        let _ = self.record_phase(PurgePhase::P1Intent);
        crate::purge_failpoint!("after_intent");

        // P2: Acquire Fence
        self.storage.erasure_fence.acquire_scope_fence();
        let _ = self.record_phase(PurgePhase::P2FenceAcquired);

        // P3: Flush Memtable
        let _ = self.record_phase(PurgePhase::P3Flush);
        crate::purge_failpoint!("after_flush");

        // P4: Execute Purge Compaction
        let _ = self.record_phase(PurgePhase::P4PurgeCompaction);
        crate::purge_failpoint!("mid_compaction_output");

        let report = match self.storage.purge_keys(scope, as_of_tx, space_probe).await {
            Ok(rep) => rep,
            Err(blocked) => {
                self.storage.erasure_fence.release_scope_fence();
                return Err(blocked);
            }
        };

        crate::purge_failpoint!("after_compaction_before_manifest");

        // P5: Dispose / Pending Delete
        let _ = self.record_phase(PurgePhase::P5Dispose);
        crate::purge_failpoint!("after_manifest_before_unlink");

        // P6: WAL Callback
        let _ = self.record_phase(PurgePhase::P6WalCallback);
        if let Some(cb) = wal_callback {
            let _ = cb();
        }

        // Release Fence
        self.storage.erasure_fence.release_scope_fence();
        let _ = self.record_phase(PurgePhase::P7Complete);

        // Remove intent log
        if self.intent_file.exists() {
            contextra_durable_fs::durable_remove(&self.intent_file)
                .map_err(|_| PurgeBlocked::UnclassifiedFiles { count: 1 })?;
        }

        Ok(report)
    }

    fn record_phase(&self, phase: PurgePhase) -> std::io::Result<()> {
        let content = format!("{:?}", phase);
        atomic_replace(&self.intent_file, content.as_bytes())
    }
}
