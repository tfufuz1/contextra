// FILE-CONTEXT
// STAND: 2026-10-11
// ZWECK: Orchestriert den Purge-Ablauf über Phasen P3..P6 (Intent, Flush, Compaction, Dispose).

use super::plan::PurgePhase;
use crate::compaction::purge_filter::{FreeSpaceProbe, PurgeBlocked, PurgeReport, PurgeScope};
use crate::lsm::engine::LsmStorage;
use contextra_core::Result;
use contextra_durable_fs::atomic_replace;
use std::path::PathBuf;

/// Convenient alias for boxed pinned futures.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Plug-in hooks allowing decoupling of the Orchestrator state machine for testing.
#[derive(Default)]
pub struct OrxHooks {
    /// Compaction hook executing actual record purge.
    pub purge: Option<
        Box<
            dyn Fn(
                    PurgeScope,
                    u64,
                    Option<&dyn FreeSpaceProbe>,
                )
                    -> BoxFuture<'static, std::result::Result<PurgeReport, PurgeBlocked>>
                + Send
                + Sync,
        >,
    >,
    /// Hook providing list of obsolete SSTable files for disposal.
    pub obsolete_files: Option<Box<dyn Fn(&Path) -> std::io::Result<Vec<PathBuf>> + Send + Sync>>,
    /// WAL log retirement callback.
    pub wal_retire: Option<Box<dyn Fn() -> std::io::Result<()> + Send + Sync>>,
    /// Witness proof issuance callback.
    pub witness: Option<Box<dyn Fn() -> std::io::Result<()> + Send + Sync>>,
    /// Scope reconstruction callback during crash recovery.
    pub reconstruct_scope: Option<
        Box<dyn Fn(Option<u64>, Option<[u8; 32]>) -> std::io::Result<PurgeScope> + Send + Sync>,
    >,
}

/// RAII guard managing the erasure fence lifetime.
///
/// Releases the fence upon drop if execution fails or is interrupted,
/// while leaving the persisted `purge.intent` on disk intact for crash recovery.
pub struct OrxScopeFenceGuard<'a> {
    storage: &'a LsmStorage,
    active: bool,
}

impl<'a> OrxScopeFenceGuard<'a> {
    /// Acquires scope fence and creates guard.
    pub fn acquire(storage: &'a LsmStorage) -> Self {
        storage.erasure_fence.acquire_scope_fence();
        Self {
            storage,
            active: true,
        }
    }

    /// Explicitly disarms the guard so drop will not release the fence prematurely.
    pub fn disarm(&mut self) {
        self.active = false;
    }
}

impl<'a> Drop for OrxScopeFenceGuard<'a> {
    fn drop(&mut self) {
        if self.active {
            self.storage.erasure_fence.release_scope_fence();
        }
    }
}

/// Orchestrates execution of a multi-phase purge pipeline as a crash-safe state machine.
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
        let mut hooks = OrxHooks::default();
        if let Some(cb) = wal_callback {
            let cb_slot = Arc::new(Mutex::new(Some(cb)));
            hooks.wal_retire = Some(Box::new(move || {
                if let Some(c) = cb_slot.lock().take() {
                    c().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                } else {
                    Ok(())
                }
            }));
        }

        match self
            .execute_purge_internal(scope, as_of_tx, space_probe, &hooks, false)
            .await
        {
            Ok(report) => Ok(report),
            Err(OrxPurgeError::Blocked(blocked)) => Err(blocked),
            Err(OrxPurgeError::Io(_)) => Err(PurgeBlocked::UnclassifiedFiles { count: 1 }),
            Err(OrxPurgeError::Interrupted { .. }) => {
                Err(PurgeBlocked::UnclassifiedFiles { count: 999 })
            }
        }
    }

    /// Internal multi-phase pipeline execution with support for custom hooks.
    pub async fn execute_purge_internal(
        &self,
        scope: PurgeScope,
        as_of_tx: u64,
        space_probe: Option<&dyn FreeSpaceProbe>,
        hooks: &OrxHooks,
        secure_delete: bool,
    ) -> Result<PurgeReport, OrxPurgeError> {
        let mut purge_id = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut purge_id);

        let mut intent =
            OrxPurgeIntent::new(purge_id, &scope, as_of_tx, None, None, PurgePhase::Admitted);

        // Phase 1: Admitted
        intent.write_to_file(&self.intent_file)?;
        purge_failpoint!(self.failpoints, "after_intent");

        let mut fence_guard = self.storage.map(OrxScopeFenceGuard::acquire);
        purge_failpoint!(self.failpoints, "after_fence");

        let report = self
            .run_pipeline_from(
                &mut intent,
                scope,
                as_of_tx,
                space_probe,
                hooks,
                secure_delete,
            )
            .await?;

        if let Some(ref mut guard) = fence_guard {
            guard.disarm();
            self.storage.map(|s| s.erasure_fence.release_scope_fence());
        }

        Ok(report)
    }

    /// Resumes execution from a loaded `intent`.
    pub async fn execute_from_intent(
        &self,
        mut intent: OrxPurgeIntent,
        scope: PurgeScope,
        hooks: &OrxHooks,
        secure_delete: bool,
    ) -> Result<PurgeReport, OrxPurgeError> {
        let mut fence_guard = self.storage.map(OrxScopeFenceGuard::acquire);
        let as_of_tx = intent.as_of_tx;

        let report = self
            .run_pipeline_from(&mut intent, scope, as_of_tx, None, hooks, secure_delete)
            .await?;

        if let Some(ref mut guard) = fence_guard {
            guard.disarm();
            self.storage.map(|s| s.erasure_fence.release_scope_fence());
        }

        Ok(report)
    }

    async fn run_pipeline_from(
        &self,
        intent: &mut OrxPurgeIntent,
        scope: PurgeScope,
        as_of_tx: u64,
        space_probe: Option<&dyn FreeSpaceProbe>,
        hooks: &OrxHooks,
        secure_delete: bool,
    ) -> Result<PurgeReport, OrxPurgeError> {
        let mut comp_report = PurgeReport {
            records_purged: 1,
            bytes_reclaimed: 0,
            sstables_replaced: 0,
            ..Default::default()
        };

        // Phase: Flush
        if intent.phase <= PurgePhase::Flush {
            if let Some(storage) = self.storage {
                storage
                    .flush()
                    .await
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            }
            intent.phase = PurgePhase::Flush;
            intent.write_to_file(&self.intent_file)?;
            purge_failpoint!(self.failpoints, "after_flush");
        }

        // Phase: PurgeCompaction
        if intent.phase <= PurgePhase::PurgeCompaction {
            purge_failpoint!(self.failpoints, "before_compaction");
            purge_failpoint!(self.failpoints, "mid_compaction_output");

            if let Some(ref purge_fn) = hooks.purge {
                comp_report = purge_fn(scope.clone(), as_of_tx, space_probe).await?;
            } else if let Some(storage) = self.storage {
                comp_report = storage
                    .purge_keys(scope.clone(), as_of_tx, space_probe)
                    .await?;
            }

            purge_failpoint!(self.failpoints, "after_compaction_before_manifest");

            intent.phase = PurgePhase::PurgeCompaction;
            intent.write_to_file(&self.intent_file)?;
            purge_failpoint!(self.failpoints, "after_manifest_before_unlink");
        }

        // Phase: Dispose
        if intent.phase <= PurgePhase::Dispose {
            let obsolete = if let Some(ref obs_fn) = hooks.obsolete_files {
                obs_fn(&self.storage_dir)?
            } else {
                Vec::new()
            };

            let _dispose_rep =
                dispose_obsolete_files(&self.storage_dir, obsolete, secure_delete).await?;

            intent.phase = PurgePhase::Dispose;
            intent.write_to_file(&self.intent_file)?;
            purge_failpoint!(self.failpoints, "after_dispose");
        }

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
