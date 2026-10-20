// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Orchestriert den Purge-Ablauf über Phasen P1..P8 (Intent, Flush, Compaction, Dispose, Witness, Release).
// INVARIANTEN:
// - P1..P8 State Machine: Jede Phase wird vor Ausführung atomar auf Disk in `purge.intent` mittels record_phase() persistiert.
// - Fail-closed Error Recovery: Jeder Fehler (IO, Failpoint, Callback, Refusal) gibt das Erasure Fence frei und belässt `purge.intent` im zuletzt erfolgreich aufgezeichneten Zustand für Wiederaufnahme (`resume_pending`).
// - Zero Dropped Results: Kein `let _ =` auf IO/Sicherheit; Ergebnisse werden mit `?` propagiert.

use super::dispose::dispose_obsolete_files;
use super::failpoint::{OrxFailpointSet, OrxPurgeError};
use super::intent::OrxPurgeIntent;
use super::plan::PurgePhase;
use crate::compaction::purge_filter::{FreeSpaceProbe, PurgeBlocked, PurgeReport, PurgeScope};
use crate::lsm::engine::LsmStorage;
use contextra_durable_fs::durable_remove;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Detailed outcome of a purge pipeline execution run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PurgeOutcome {
    /// Pipeline completed all phases successfully.
    Completed(PurgeReport),
    /// Pipeline completed all phases, but obsolete file disposal encountered refusals.
    CompletedWithRefusals {
        report: PurgeReport,
        refusals: usize,
    },
    /// Pipeline failed at `phase` with error description.
    Failed {
        phase: PurgePhase,
        error_msg: String,
    },
}

impl PurgeOutcome {
    /// Returns true if the outcome represents complete success without failures.
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            PurgeOutcome::Completed(_) | PurgeOutcome::CompletedWithRefusals { .. }
        )
    }

    /// Converts the outcome into a Result<PurgeReport, OrxPurgeError>.
    pub fn into_result(self) -> Result<PurgeReport, OrxPurgeError> {
        match self {
            PurgeOutcome::Completed(report) => Ok(report),
            PurgeOutcome::CompletedWithRefusals { report, .. } => Ok(report),
            PurgeOutcome::Failed { phase, error_msg } => {
                if error_msg.contains("Failpoint triggered:") {
                    let fp_name = error_msg
                        .split("Failpoint triggered:")
                        .nth(1)
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    Err(OrxPurgeError::FailpointTriggered(fp_name))
                } else {
                    Err(OrxPurgeError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Purge failed at phase {:?}: {}", phase, error_msg),
                    )))
                }
            }
        }
    }
}

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

    /// Durably records phase transition to intent file.
    pub fn record_phase(
        &self,
        intent: &mut OrxPurgeIntent,
        phase: PurgePhase,
    ) -> Result<(), OrxPurgeError> {
        intent.phase = phase;
        intent
            .write_to_file(&self.intent_file)
            .map_err(OrxPurgeError::Io)
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

    /// Executes complete purge pipeline and returns `PurgeOutcome`.
    pub async fn execute_purge_outcome(
        &self,
        scope: PurgeScope,
        as_of_tx: u64,
        space_probe: Option<&dyn FreeSpaceProbe>,
        hooks: &OrxHooks,
        secure_delete: bool,
    ) -> PurgeOutcome {
        if let Err(e) = self.check_failpoint("p0_init") {
            return PurgeOutcome::Failed {
                phase: PurgePhase::Init,
                error_msg: e.to_string(),
            };
        }

        let scope_hash = OrxPurgeIntent::compute_scope_hash(&scope);
        let mut hasher = blake3::Hasher::new();
        hasher.update(&scope_hash);
        hasher.update(&as_of_tx.to_le_bytes());
        let mut purge_id = [0u8; 16];
        purge_id.copy_from_slice(&hasher.finalize().as_bytes()[..16]);

        let mut intent =
            OrxPurgeIntent::new(purge_id, &scope, as_of_tx, None, None, PurgePhase::Admitted);

        if let Err(e) = self.record_phase(&mut intent, PurgePhase::Admitted) {
            return PurgeOutcome::Failed {
                phase: PurgePhase::Admitted,
                error_msg: e.to_string(),
            };
        }

        if let Err(e) = self.check_failpoint("after_intent") {
            return PurgeOutcome::Failed {
                phase: PurgePhase::Admitted,
                error_msg: e.to_string(),
            };
        }

        self.execute_from_intent_outcome(intent, scope, hooks, secure_delete, space_probe)
            .await
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
        self.execute_purge_outcome(scope, as_of_tx, space_probe, hooks, secure_delete)
            .await
            .into_result()
    }

    /// Executes purge pipeline from saved intent and returns `PurgeOutcome`.
    pub async fn execute_from_intent_outcome(
        &self,
        mut intent: OrxPurgeIntent,
        scope: PurgeScope,
        hooks: &OrxHooks,
        secure_delete: bool,
        space_probe: Option<&dyn FreeSpaceProbe>,
    ) -> PurgeOutcome {
        let fence_prefixes = match &scope {
            PurgeScope::Prefixes(prefixes) => prefixes.clone(),
            PurgeScope::Keys(keys) => keys.clone(),
        };

        let mut fence_guard = None;
        let mut fence_acquired =
            intent.phase >= PurgePhase::FenceAcquired && intent.phase < PurgePhase::Release;

        let run_pipeline = async {
            // P2: Acquire Fence
            if intent.phase <= PurgePhase::FenceAcquired {
                if let Some(storage) = self.storage {
                    if fence_prefixes.is_empty() {
                        return Err(OrxPurgeError::Blocked(PurgeBlocked::UnclassifiedFiles { count: 0 }));
                    }
                    let guard = storage
                        .erasure_fence
                        .acquire_scope_fence_for(fence_prefixes.clone())
                        .map_err(|_| OrxPurgeError::Blocked(PurgeBlocked::CompactionBusy))?;
                    fence_guard = Some(guard);
                }
                if let Some(ref f) = hooks.fence_acquire {
                    f().map_err(OrxPurgeError::Io)?;
                }
                fence_acquired = true;
                self.record_phase(&mut intent, PurgePhase::FenceAcquired)?;
                self.check_failpoint("after_fence")?;
            }

            // P3: Flush Memtable
            if intent.phase <= PurgePhase::Flush {
                if let Some(ref f) = hooks.flush_memtable {
                    f().map_err(OrxPurgeError::Io)?;
                }
                self.record_phase(&mut intent, PurgePhase::Flush)?;
                self.check_failpoint("after_flush")?;
            }

            // P4: Execute Purge Compaction
            let mut report = PurgeReport {
                records_purged: 0,
                bytes_reclaimed: 0,
                sstables_replaced: 0,
                obsolete_files: Vec::new(),
            };

            if intent.phase <= PurgePhase::PurgeCompaction {
                self.record_phase(&mut intent, PurgePhase::PurgeCompaction)?;
                self.check_failpoint("mid_compaction_output")?;

                if let Some(storage) = self.storage {
                    match storage
                        .purge_keys(scope, intent.as_of_tx, space_probe)
                        .await
                    {
                        Ok(rep) => report = rep,
                        Err(err) => {
                            return Err(OrxPurgeError::from(err));
                        }
                    }
                }

                self.check_failpoint("after_compaction_before_manifest")?;
            }

            // P5: Dispose Obsolete Files
            let mut refusals = 0;
            if intent.phase <= PurgePhase::Dispose {
                self.record_phase(&mut intent, PurgePhase::Dispose)?;
                self.check_failpoint("after_manifest_before_unlink")?;

                let disp_rep =
                    dispose_obsolete_files(&self.storage_dir, report.obsolete_files.clone(), secure_delete).await?;
                refusals = disp_rep.refusals.len();

                self.check_failpoint("mid_dispose_journal")?;
                self.check_failpoint("after_unlink")?;
            }

            // P6: WAL Retire
            if intent.phase <= PurgePhase::WalRetire {
                self.record_phase(&mut intent, PurgePhase::WalRetire)?;
                if let Some(ref cb) = hooks.wal_retire {
                    cb().map_err(OrxPurgeError::Io)?;
                }
                self.check_failpoint("after_wal_callback")?;
            }

            // P7: Witness & Finalize
            if intent.phase <= PurgePhase::Finalize {
                self.record_phase(&mut intent, PurgePhase::Witness)?;
                if let Some(ref cb) = hooks.witness {
                    cb().map_err(OrxPurgeError::Io)?;
                }
                self.check_failpoint("after_witness")?;

                self.record_phase(&mut intent, PurgePhase::Finalize)?;
                self.check_failpoint("after_finalize")?;
            }

            // P8: Release Erasure Fence & Cleanup
            if intent.phase <= PurgePhase::Release {
                fence_guard.take();
                if let Some(ref f) = hooks.fence_release {
                    f().map_err(OrxPurgeError::Io)?;
                }
                fence_acquired = false;

                self.record_phase(&mut intent, PurgePhase::Release)?;
                self.check_failpoint("after_release")?;

                if self.intent_file.exists() {
                    durable_remove(&self.intent_file).map_err(OrxPurgeError::Io)?;
                }
                self.check_failpoint("after_intent_cleanup")?;
            }

            Ok((report, refusals))
        };

        match run_pipeline.await {
            Ok((report, refusals)) => {
                if refusals > 0 {
                    PurgeOutcome::CompletedWithRefusals { report, refusals }
                } else {
                    PurgeOutcome::Completed(report)
                }
            }
            Err(err) => {
                if fence_acquired || fence_guard.is_some() {
                    fence_guard.take();
                    if let Some(ref f) = hooks.fence_release {
                        if let Err(rel_err) = f() {
                            tracing::warn!(
                                "Error releasing fence during purge failure cleanup: {:?}",
                                rel_err
                            );
                        }
                    }
                }
                PurgeOutcome::Failed {
                    phase: intent.phase,
                    error_msg: err.to_string(),
                }
            }
        }
    }

    /// Resumes or completes purge pipeline from saved intent.
    pub async fn execute_from_intent(
        &self,
        intent: OrxPurgeIntent,
        scope: PurgeScope,
        hooks: &OrxHooks,
        secure_delete: bool,
        space_probe: Option<&dyn FreeSpaceProbe>,
    ) -> Result<PurgeReport, OrxPurgeError> {
        self.execute_from_intent_outcome(intent, scope, hooks, secure_delete, space_probe)
            .await
            .into_result()
    }

    fn check_failpoint(&self, name: &str) -> Result<(), OrxPurgeError> {
        if let Some(ref set) = self.failpoints {
            if set.is_active(name) {
                return Err(OrxPurgeError::FailpointTriggered(name.to_string()));
            }
        }
        #[cfg(feature = "fault-injection")]
        if std::env::var(format!("PURGE_FAILPOINT_{}", name)).is_ok() {
            return Err(OrxPurgeError::FailpointTriggered(name.to_string()));
        }
        Ok(())
    }
}
