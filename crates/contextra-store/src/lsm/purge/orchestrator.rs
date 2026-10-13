// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Orchestriert den Purge-Ablauf über Phasen P1..P8 (Intent, Flush, Compaction, Dispose, Witness, Release).

use super::dispose::dispose_obsolete_files;
use super::failpoint::{OrxFailpointSet, OrxPurgeError};
use super::intent::OrxPurgeIntent;
use super::plan::PurgePhase;
use crate::compaction::purge_filter::{FreeSpaceProbe, PurgeBlocked, PurgeReport, PurgeScope};
use crate::lsm::engine::LsmStorage;
use contextra_durable_fs::durable_remove;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Hooks callback function type.
pub type HookCallback = Box<dyn Fn() -> std::io::Result<()> + Send + Sync>;

/// Scope reconstruction callback function type.
pub type ScopeReconstructFn =
    Box<dyn Fn(Option<u64>, Option<[u8; 32]>) -> Result<PurgeScope, OrxPurgeError> + Send + Sync>;

/// Configuration and callbacks for purge lifecycle steps.
#[derive(Default)]
pub struct OrxHooks {
    /// Callback executed during WAL retirement (Phase 6).
    pub wal_retire: Option<HookCallback>,
    /// Callback executed during witness generation (Phase 7).
    pub witness: Option<HookCallback>,
    /// Callback to reconstruct PurgeScope from intent ledger sequence or proof ID.
    pub reconstruct_scope: Option<ScopeReconstructFn>,
    /// Callback to flush memtable.
    pub flush_memtable: Option<HookCallback>,
    /// Callback to acquire erasure fence.
    pub fence_acquire: Option<HookCallback>,
    /// Callback to release erasure fence.
    pub fence_release: Option<HookCallback>,
}

impl std::fmt::Debug for OrxHooks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrxHooks")
            .field("wal_retire", &self.wal_retire.is_some())
            .field("witness", &self.witness.is_some())
            .field("reconstruct_scope", &self.reconstruct_scope.is_some())
            .field("flush_memtable", &self.flush_memtable.is_some())
            .field("fence_acquire", &self.fence_acquire.is_some())
            .field("fence_release", &self.fence_release.is_some())
            .finish()
    }
}

/// Orchestrates the execution of a multi-phase purge pipeline.
pub struct PurgeOrchestrator<'a> {
    storage: Option<&'a LsmStorage>,
    storage_dir: PathBuf,
    intent_file: PathBuf,
    failpoints: Option<OrxFailpointSet>,
}

impl<'a> PurgeOrchestrator<'a> {
    /// Creates a new `PurgeOrchestrator` instance for `storage`.
    pub fn new(storage: &'a LsmStorage) -> Self {
        let storage_dir = storage.config.path.clone();
        let intent_file = storage_dir.join("purge.intent");
        Self {
            storage: Some(storage),
            storage_dir,
            intent_file,
            failpoints: None,
        }
    }

    /// Creates a standalone `PurgeOrchestrator` without active storage.
    pub fn new_standalone(storage_dir: &Path) -> Self {
        let storage_dir = storage_dir.to_path_buf();
        let intent_file = storage_dir.join("purge.intent");
        Self {
            storage: None,
            storage_dir,
            intent_file,
            failpoints: None,
        }
    }

    /// Configures active failpoints for testing.
    pub fn with_failpoints(mut self, failpoints: OrxFailpointSet) -> Self {
        self.failpoints = Some(failpoints);
        self
    }

    /// Executes complete purge pipeline.
    pub async fn execute_purge(
        &self,
        scope: PurgeScope,
        as_of_tx: u64,
        space_probe: Option<&dyn FreeSpaceProbe>,
        wal_callback: Option<Box<dyn FnOnce() -> Result<(), PurgeBlocked> + Send>>,
    ) -> std::result::Result<PurgeReport, PurgeBlocked> {
        let mut hooks = OrxHooks::default();
        if let Some(cb) = wal_callback {
            let cb_cell = Arc::new(Mutex::new(Some(cb)));
            hooks.wal_retire = Some(Box::new(move || {
                if let Ok(mut lock) = cb_cell.lock() {
                    if let Some(f) = lock.take() {
                        return f().map_err(|e| {
                            std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", e))
                        });
                    }
                }
                Ok(())
            }));
        }

        self.execute_purge_internal(scope, as_of_tx, space_probe, &hooks, false)
            .await
            .map_err(Into::into)
    }

    /// Internal purge pipeline execution with failpoint checks.
    pub async fn execute_purge_internal(
        &self,
        scope: PurgeScope,
        as_of_tx: u64,
        space_probe: Option<&dyn FreeSpaceProbe>,
        hooks: &OrxHooks,
        secure_delete: bool,
    ) -> Result<PurgeReport, OrxPurgeError> {
        self.check_failpoint("p0_init")?;

        // Generate deterministic purge_id from scope hash and cutoff tx
        let scope_hash = OrxPurgeIntent::compute_scope_hash(&scope);
        let mut hasher = blake3::Hasher::new();
        hasher.update(&scope_hash);
        hasher.update(&as_of_tx.to_le_bytes());
        let mut purge_id = [0u8; 16];
        purge_id.copy_from_slice(&hasher.finalize().as_bytes()[..16]);

        let intent = OrxPurgeIntent::new(
            purge_id,
            &scope,
            as_of_tx,
            None,
            None,
            PurgePhase::P1Intent,
        );

        // P1: Record intent durably
        intent.write_to_file(&self.intent_file)?;
        self.check_failpoint("after_intent")?;

        self.execute_from_intent(intent, scope, hooks, secure_delete, space_probe)
            .await
    }

    /// Resumes or completes purge pipeline from saved intent.
    pub async fn execute_from_intent(
        &self,
        mut intent: OrxPurgeIntent,
        scope: PurgeScope,
        hooks: &OrxHooks,
        secure_delete: bool,
        space_probe: Option<&dyn FreeSpaceProbe>,
    ) -> Result<PurgeReport, OrxPurgeError> {
        // P2: Acquire Fence
        if intent.phase <= PurgePhase::P2FenceAcquired {
            if let Some(storage) = self.storage {
                storage.erasure_fence.acquire_scope_fence();
            }
            if let Some(ref f) = hooks.fence_acquire {
                f()?;
            }
            intent.phase = PurgePhase::P2FenceAcquired;
            intent.write_to_file(&self.intent_file)?;
            self.check_failpoint("after_fence")?;
        }

        // P3: Flush Memtable
        if intent.phase <= PurgePhase::Flush {
            if let Some(ref f) = hooks.flush_memtable {
                f()?;
            }
            intent.phase = PurgePhase::P3Flush;
            intent.write_to_file(&self.intent_file)?;
            self.check_failpoint("after_flush")?;
        }

        // P4: Execute Purge Compaction
        let report = if intent.phase <= PurgePhase::PurgeCompaction {
            intent.phase = PurgePhase::P4PurgeCompaction;
            intent.write_to_file(&self.intent_file)?;
            self.check_failpoint("mid_compaction_output")?;

            let rep = if let Some(storage) = self.storage {
                match storage.purge_keys(scope, intent.as_of_tx, space_probe).await {
                    Ok(rep) => rep,
                    Err(blocked) => {
                        storage.erasure_fence.release_scope_fence();
                        return Err(OrxPurgeError::Blocked(blocked));
                    }
                }
            } else {
                PurgeReport {
                    records_purged: 0,
                    bytes_reclaimed: 0,
                    sstables_replaced: 0,
                }
            };

            self.check_failpoint("after_compaction_before_manifest")?;
            rep
        } else {
            PurgeReport {
                records_purged: 0,
                bytes_reclaimed: 0,
                sstables_replaced: 0,
            }
        };

        // P5: Dispose obsolete files
        if intent.phase <= PurgePhase::Dispose {
            intent.phase = PurgePhase::P5Dispose;
            intent.write_to_file(&self.intent_file)?;
            self.check_failpoint("after_manifest_before_unlink")?;

            let _disp_rep =
                dispose_obsolete_files(&self.storage_dir, Vec::new(), secure_delete).await?;

            self.check_failpoint("mid_dispose_journal")?;
            self.check_failpoint("after_unlink")?;
        }

        // P6: WAL Retire
        if intent.phase <= PurgePhase::WalRetire {
            intent.phase = PurgePhase::P6WalCallback;
            intent.write_to_file(&self.intent_file)?;
            if let Some(ref cb) = hooks.wal_retire {
                cb()?;
            }
            self.check_failpoint("after_wal_callback")?;
        }

        // P7: Witness & Finalize
        if intent.phase <= PurgePhase::Finalize {
            intent.phase = PurgePhase::Witness;
            intent.write_to_file(&self.intent_file)?;
            if let Some(ref cb) = hooks.witness {
                cb()?;
            }
            self.check_failpoint("after_witness")?;

            intent.phase = PurgePhase::Finalize;
            intent.write_to_file(&self.intent_file)?;
            self.check_failpoint("after_finalize")?;
        }

        // P8: Release Erasure Fence & Cleanup
        if intent.phase <= PurgePhase::Release {
            if let Some(storage) = self.storage {
                storage.erasure_fence.release_scope_fence();
            }
            if let Some(ref f) = hooks.fence_release {
                f()?;
            }
            intent.phase = PurgePhase::Release;
            intent.write_to_file(&self.intent_file)?;
            self.check_failpoint("after_release")?;

            if self.intent_file.exists() {
                durable_remove(&self.intent_file)?;
            }
            self.check_failpoint("after_intent_cleanup")?;
        }

        Ok(report)
    }

    fn check_failpoint(&self, name: &str) -> Result<(), OrxPurgeError> {
        if let Some(ref set) = self.failpoints {
            if set.is_active(name) {
                return Err(OrxPurgeError::FailpointTriggered(name.to_string()));
            }
        }
        if std::env::var(format!("PURGE_FAILPOINT_{}", name)).is_ok() {
            return Err(OrxPurgeError::FailpointTriggered(name.to_string()));
        }
        Ok(())
    }
}
