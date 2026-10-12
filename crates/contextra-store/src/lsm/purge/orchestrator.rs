// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Orchestriert den Purge-Ablauf als crashsicheren Zustandsautomaten (Spezifikation C5.1/C5.6).
// INVARIANTEN: Atomic Intent Persistence nach jeder Phase, RAII-FenceGuard, error propagation, kein let _ =.

use super::dispose::execute_dispose;
use super::failpoint::{OrxFailpointSet, OrxPurgeError};
use super::intent::OrxPurgeIntent;
use super::plan::PurgePhase;
use crate::compaction::purge_filter::{FreeSpaceProbe, PurgeBlocked, PurgeReport, PurgeScope};
use crate::lsm::engine::LsmStorage;
use crate::lsm::fence::ErasureFence;
use crate::purge_failpoint;
use contextra_core::StorageEngine;
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;

/// RAII guard ensuring ErasureFence is released on drop or error without unlinking intent.
pub struct FenceGuard<'a> {
    fence: &'a ErasureFence,
    released: bool,
}

impl<'a> FenceGuard<'a> {
    /// Acquires the scope fence and returns an RAII guard.
    pub fn acquire(fence: &'a ErasureFence) -> Self {
        fence.acquire_scope_fence();
        Self {
            fence,
            released: false,
        }
    }

    /// Releases the scope fence manually.
    pub fn release(&mut self) {
        if !self.released {
            self.fence.release_scope_fence();
            self.released = true;
        }
    }
}

impl<'a> Drop for FenceGuard<'a> {
    fn drop(&mut self) {
        self.release();
    }
}

/// Callbacks and external hooks for PurgeOrchestrator stages.
pub struct OrxHooks<'h> {
    /// Purge compaction execution callback.
    pub purge: Box<
        dyn Fn(
                PurgeScope,
                u64,
            ) -> std::pin::Pin<
                Box<
                    dyn std::future::Future<
                            Output = std::result::Result<PurgeReport, OrxPurgeError>,
                        > + Send
                        + 'h,
                >,
            > + Send
            + Sync
            + 'h,
    >,
    /// Obsolete files callback.
    pub obsolete_files: Box<dyn Fn() -> Vec<PathBuf> + Send + Sync + 'h>,
    /// WAL retirement callback.
    pub wal_retire: Box<dyn Fn() -> std::result::Result<(), OrxPurgeError> + Send + Sync + 'h>,
    /// Deletion proof witness callback.
    pub witness: Box<dyn Fn() -> std::result::Result<(), OrxPurgeError> + Send + Sync + 'h>,
    /// Failpoints active during test/simulation execution.
    pub failpoints: Option<OrxFailpointSet>,
    /// Secure delete flag.
    pub secure_delete: bool,
}

impl Default for OrxHooks<'static> {
    fn default() -> Self {
        Self {
            purge: Box::new(|_scope, _as_of_tx| {
                Box::pin(async {
                    Ok(PurgeReport {
                        records_purged: 0,
                        bytes_reclaimed: 0,
                        sstables_replaced: 0,
                    })
                })
            }),
            obsolete_files: Box::new(|| Vec::new()),
            wal_retire: Box::new(|| Ok(())),
            witness: Box::new(|| Ok(())),
            failpoints: None,
            secure_delete: false,
        }
    }
}

impl OrxHooks<'static> {
    /// Creates default `OrxHooks`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets custom failpoint set.
    pub fn with_failpoints(mut self, failpoints: OrxFailpointSet) -> Self {
        self.failpoints = Some(failpoints);
        self
    }
}

/// Orchestrates the crash-consistent execution of a multi-phase purge pipeline.
pub struct PurgeOrchestrator<'a> {
    storage: &'a LsmStorage,
    intent_file: PathBuf,
    journal_path: PathBuf,
}

impl<'a> PurgeOrchestrator<'a> {
    /// Creates a new `PurgeOrchestrator` instance for `storage`.
    pub fn new(storage: &'a LsmStorage) -> Self {
        let intent_file = storage.config.path.join("purge.intent");
        let journal_path = storage.config.path.join("pending_delete.journal");
        Self {
            storage,
            intent_file,
            journal_path,
        }
    }

    /// Legacy API method for backwards compatibility with existing callers.
    pub async fn execute_purge(
        &self,
        scope: PurgeScope,
        as_of_tx: u64,
        space_probe: Option<&'a (dyn FreeSpaceProbe + 'static)>,
        wal_callback: Option<Box<dyn FnOnce() -> contextra_core::Result<()> + Send>>,
    ) -> std::result::Result<PurgeReport, PurgeBlocked> {
        let storage = self.storage;
        let wal_cb_cell = Arc::new(Mutex::new(wal_callback));

        let hooks = OrxHooks {
            purge: Box::new(move |s, tx| {
                Box::pin(async move {
                    storage
                        .purge_keys(s, tx, space_probe)
                        .await
                        .map_err(OrxPurgeError::Blocked)
                })
            }),
            obsolete_files: Box::new(|| Vec::new()),
            wal_retire: Box::new(move || {
                if let Some(cb) = wal_cb_cell.lock().take() {
                    cb().map_err(|e| OrxPurgeError::Io(e.to_string()))?;
                }
                Ok(())
            }),
            witness: Box::new(|| Ok(())),
            failpoints: None,
            secure_delete: false,
        };

        self.execute_purge_with_hooks(scope, as_of_tx, &hooks)
            .await
            .map_err(|err| match err {
                OrxPurgeError::Blocked(blocked) => blocked,
                _ => PurgeBlocked::UnclassifiedFiles { count: 1 },
            })
    }

    /// Executes the purge pipeline using `hooks`.
    pub async fn execute_purge_with_hooks<'h>(
        &self,
        scope: PurgeScope,
        as_of_tx: u64,
        hooks: &OrxHooks<'h>,
    ) -> std::result::Result<PurgeReport, OrxPurgeError> {
        let purge_id = uuid::Uuid::new_v4().into_bytes();
        let scope_bytes = serde_json::to_vec(&format!("{:?}", scope))
            .map_err(|e| OrxPurgeError::Io(e.to_string()))?;
        let scope_hash = blake3::hash(&scope_bytes).into();

        let intent = OrxPurgeIntent::new(
            purge_id,
            scope_hash,
            1,
            None,
            None,
            as_of_tx,
            PurgePhase::Admitted,
        );

        self.execute_from(intent, scope, hooks).await
    }

    /// Resumes or executes a purge operation from `intent`.
    pub async fn execute_from<'h>(
        &self,
        mut intent: OrxPurgeIntent,
        scope: PurgeScope,
        hooks: &OrxHooks<'h>,
    ) -> std::result::Result<PurgeReport, OrxPurgeError> {
        let mut report = PurgeReport {
            records_purged: 0,
            bytes_reclaimed: 0,
            sstables_replaced: 0,
        };

        // Phase: Admitted
        if intent.phase.is_at_least(PurgePhase::Admitted) && !intent.phase.is_at_least(PurgePhase::Flush) {
            intent.phase = PurgePhase::Admitted;
            intent.save(&self.intent_file).map_err(|e| OrxPurgeError::Io(e.to_string()))?;
            purge_failpoint!(&hooks.failpoints, "after_intent");

            let _fence_guard = FenceGuard::acquire(&self.storage.erasure_fence);
            purge_failpoint!(&hooks.failpoints, "after_fence");

            intent.phase = PurgePhase::Flush;
            intent.save(&self.intent_file).map_err(|e| OrxPurgeError::Io(e.to_string()))?;
        }

        // Acquire Fence Guard if starting/resuming from Flush or beyond
        let mut fence_guard = FenceGuard::acquire(&self.storage.erasure_fence);

        // Phase: Flush
        if intent.phase == PurgePhase::Flush || intent.phase == PurgePhase::P3Flush {
            <LsmStorage as StorageEngine>::flush(self.storage)
                .await
                .map_err(|e| OrxPurgeError::Io(e.to_string()))?;
            purge_failpoint!(&hooks.failpoints, "after_flush");

            intent.phase = PurgePhase::PurgeCompaction;
            intent.save(&self.intent_file).map_err(|e| OrxPurgeError::Io(e.to_string()))?;
        }

        // Phase: PurgeCompaction
        if intent.phase == PurgePhase::PurgeCompaction || intent.phase == PurgePhase::P4PurgeCompaction {
            purge_failpoint!(&hooks.failpoints, "mid_compaction_output");
            report = (hooks.purge)(scope.clone(), intent.as_of_tx).await?;
            purge_failpoint!(&hooks.failpoints, "after_compaction_before_manifest");

            intent.phase = PurgePhase::Dispose;
            intent.save(&self.intent_file).map_err(|e| OrxPurgeError::Io(e.to_string()))?;
        }

        // Phase: Dispose
        if intent.phase == PurgePhase::Dispose || intent.phase == PurgePhase::P5Dispose {
            purge_failpoint!(&hooks.failpoints, "after_manifest_before_unlink");
            let obs_files = (hooks.obsolete_files)();
            purge_failpoint!(&hooks.failpoints, "after_journal_record");
            purge_failpoint!(&hooks.failpoints, "mid_unlink");

            execute_dispose(self.journal_path.clone(), obs_files, hooks.secure_delete)
                .await
                .map_err(|e| OrxPurgeError::Io(e.to_string()))?;

            intent.phase = PurgePhase::WalRetire;
            intent.save(&self.intent_file).map_err(|e| OrxPurgeError::Io(e.to_string()))?;
        }

        // Phase: WalRetire
        if intent.phase == PurgePhase::WalRetire || intent.phase == PurgePhase::P6WalCallback {
            purge_failpoint!(&hooks.failpoints, "after_unlink_before_wal");
            (hooks.wal_retire)()?;
            purge_failpoint!(&hooks.failpoints, "after_wal_retire");

            intent.phase = PurgePhase::Witness;
            intent.save(&self.intent_file).map_err(|e| OrxPurgeError::Io(e.to_string()))?;
        }

        // Phase: Witness
        if intent.phase == PurgePhase::Witness {
            purge_failpoint!(&hooks.failpoints, "after_proof_mint");
            (hooks.witness)()?;
            purge_failpoint!(&hooks.failpoints, "after_witness");

            intent.phase = PurgePhase::Finalize;
            intent.save(&self.intent_file).map_err(|e| OrxPurgeError::Io(e.to_string()))?;
        }

        // Phase: Finalize
        if intent.phase == PurgePhase::Finalize {
            purge_failpoint!(&hooks.failpoints, "after_finalize");
            purge_failpoint!(&hooks.failpoints, "after_chain_before_ack");

            intent.phase = PurgePhase::Release;
            intent.save(&self.intent_file).map_err(|e| OrxPurgeError::Io(e.to_string()))?;
        }

        // Phase: Release
        if intent.phase == PurgePhase::Release || intent.phase == PurgePhase::P7Complete {
            fence_guard.release();
            if self.intent_file.exists() {
                contextra_durable_fs::durable_remove(&self.intent_file)
                    .map_err(|e| OrxPurgeError::Io(e.to_string()))?;
            }
        }

        Ok(report)
    }
}
