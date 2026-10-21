// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Orchestriert den Purge-Ablauf über Phasen 1..9 (Admitted..Released) über PurgeBackend.

use super::backend::PurgeBackend;
use super::dispose::dispose_obsolete_files;
use super::failpoint::{OrxFailpointSet, OrxPurgeError};
use super::intent::OrxPurgeIntent;
use super::plan::PurgePhase;
use crate::compaction::purge_filter::{FreeSpaceProbe, PurgeReport, PurgeScope};
use contextra_durable_fs::durable_remove;
use contextra_ports::erasure::{PurgeError, PurgeId};
use std::path::{Path, PathBuf};

/// Detailed outcome of a purge pipeline execution run.
#[derive(Debug)]
pub enum PurgeOutcome {
    /// Pipeline completed all phases successfully.
    Completed(PurgeReport),
    /// Pipeline completed all phases, but obsolete file disposal encountered refusals.
    CompletedWithRefusals {
        report: PurgeReport,
        refusals: usize,
    },
    /// Pipeline failed at `phase` with error.
    Failed {
        phase: PurgePhase,
        error: PurgeError,
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
            PurgeOutcome::Completed(report) | PurgeOutcome::CompletedWithRefusals { report, .. } => Ok(report),
            PurgeOutcome::Failed { error, .. } => Err(OrxPurgeError::from(error)),
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

/// Orchestrates the execution of a multi-phase purge pipeline bound to `B: PurgeBackend`.
pub struct PurgeOrchestrator<'a, B: PurgeBackend> {
    backend: &'a B,
    storage_dir: PathBuf,
    intent_file: PathBuf,
    failpoints: Option<OrxFailpointSet>,
}

impl<'a, B: PurgeBackend> PurgeOrchestrator<'a, B> {
    /// Creates a new `PurgeOrchestrator` instance for `backend` and `storage_dir`.
    pub fn new(backend: &'a B, storage_dir: &Path) -> Self {
        let storage_dir = storage_dir.to_path_buf();
        let intent_file = storage_dir.join("purge.intent");
        Self {
            backend,
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
                phase: PurgePhase::Admitted,
                error: e.into(),
            };
        }

        let scope_hash = OrxPurgeIntent::compute_scope_hash(&scope);
        let mut hasher = blake3::Hasher::new();
        hasher.update(&scope_hash);
        hasher.update(&as_of_tx.to_le_bytes());
        let mut purge_id_bytes = [0u8; 16];
        purge_id_bytes.copy_from_slice(&hasher.finalize().as_bytes()[..16]);

        let mut intent =
            OrxPurgeIntent::new(purge_id_bytes, &scope, as_of_tx, None, None, PurgePhase::Admitted);

        if let Err(e) = self.record_phase(&mut intent, PurgePhase::Admitted) {
            return PurgeOutcome::Failed {
                phase: PurgePhase::Admitted,
                error: e.into(),
            };
        }

        if let Err(e) = self.check_failpoint("after_intent") {
            return PurgeOutcome::Failed {
                phase: PurgePhase::Admitted,
                error: e.into(),
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
        let mut fence_guard = None;
        let mut fence_acquired =
            intent.phase >= PurgePhase::Fenced && intent.phase < PurgePhase::Released;

        let run_pipeline = async {
            // Phase 2: Fenced
            if intent.phase <= PurgePhase::Fenced {
                let guard = self
                    .backend
                    .acquire_fence(&scope, PurgeId::new(intent.purge_id))
                    .map_err(|_e| OrxPurgeError::Blocked(crate::compaction::purge_filter::PurgeBlocked::CompactionBusy))?;
                fence_guard = Some(guard);

                if let Some(ref f) = hooks.fence_acquire {
                    f().map_err(OrxPurgeError::Io)?;
                }
                fence_acquired = true;
                self.record_phase(&mut intent, PurgePhase::Fenced)?;
                self.check_failpoint("after_fence")?;
            }

            // Phase 3: Flushed
            if intent.phase <= PurgePhase::Flushed {
                self.backend.flush_memtable().await.map_err(OrxPurgeError::from)?;
                if let Some(ref f) = hooks.flush_memtable {
                    f().map_err(OrxPurgeError::Io)?;
                }
                self.record_phase(&mut intent, PurgePhase::Flushed)?;
                self.check_failpoint("after_flush")?;
            }

            // Phase 4: Rewriting
            let mut report = PurgeReport {
                records_purged: 0,
                bytes_reclaimed: 0,
                sstables_replaced: 0,
                obsolete_files: Vec::new(),
                residual_risks: Vec::new(),
            };

            if intent.phase <= PurgePhase::Rewriting {
                self.record_phase(&mut intent, PurgePhase::Rewriting)?;
                self.check_failpoint("mid_compaction_output")?;

                match self
                    .backend
                    .purge_keys(scope.clone(), intent.as_of_tx, space_probe)
                    .await
                {
                    Ok(rep) => report = rep,
                    Err(err) => {
                        return Err(OrxPurgeError::from(err));
                    }
                }

                self.check_failpoint("after_compaction_before_manifest")?;
            }

            // Phase 5: Disposing
            let mut refusals = 0;
            if intent.phase <= PurgePhase::Disposing {
                self.record_phase(&mut intent, PurgePhase::Disposing)?;
                self.check_failpoint("after_manifest_before_unlink")?;

                let disp_rep =
                    dispose_obsolete_files(&self.storage_dir, report.obsolete_files.clone(), secure_delete).await?;
                refusals = disp_rep.refusals.len();

                self.check_failpoint("mid_dispose_journal")?;
                self.check_failpoint("after_unlink")?;
            }

            // Phase 6: WalRetiring
            if intent.phase <= PurgePhase::WalRetiring {
                self.record_phase(&mut intent, PurgePhase::WalRetiring)?;
                self.backend.retire_wal().await.map_err(OrxPurgeError::from)?;
                if let Some(ref cb) = hooks.wal_retire {
                    cb().map_err(OrxPurgeError::Io)?;
                }
                self.check_failpoint("after_wal_callback")?;
            }

            // Phase 7 & 8: Witnessing & Finalizing
            if intent.phase <= PurgePhase::Finalizing {
                self.record_phase(&mut intent, PurgePhase::Witnessing)?;
                if let Some(ref cb) = hooks.witness {
                    cb().map_err(OrxPurgeError::Io)?;
                }
                self.check_failpoint("after_witness")?;

                self.record_phase(&mut intent, PurgePhase::Finalizing)?;
                self.check_failpoint("after_finalize")?;
            }

            // Verification scan before Phase 9 Released
            if self.backend.scan_residue(&scope).await.map_err(OrxPurgeError::from)? {
                return Err(OrxPurgeError::from(PurgeError::CorruptedIntent(
                    "Residue detected during verification scan before release".into(),
                )));
            }

            // Phase 9: Released
            if intent.phase <= PurgePhase::Released {
                fence_guard.take();
                if let Some(ref f) = hooks.fence_release {
                    f().map_err(OrxPurgeError::Io)?;
                }
                fence_acquired = false;

                self.record_phase(&mut intent, PurgePhase::Released)?;
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
                    error: err.into(),
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
