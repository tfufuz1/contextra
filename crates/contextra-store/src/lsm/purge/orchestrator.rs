// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Orchestriert den Purge-Ablauf über Phasen P1..P9 (Intent, Fence, Flush, Compaction, Dispose, WalRetire, Witness, Finalize, Release).
// INVARIANTEN:
// - P1..P9 State Machine: Jede Phase wird vor Ausführung atomar auf Disk in `purge.intent` mittels record_phase() persistiert.
// - Fail-closed Error Recovery: Jeder Fehler gibt das Erasure Fence frei und belässt `purge.intent` im zuletzt erfolgreich aufgezeichneten Zustand für Wiederaufnahme (`resume_pending`).
// - Zero Dropped Results: Kein `let _ =` auf IO/Sicherheit; Ergebnisse werden mit `?` propagiert.

use super::dispose::dispose_obsolete_files;
use super::failpoint::OrxFailpointSet;
use super::intent::OrxPurgeIntent;
use super::plan::PurgePhase;
use crate::compaction::purge_filter::{FreeSpaceProbe, PurgeBlocked, PurgeError, PurgeReport, PurgeScope};
use crate::lsm::engine::LsmStorage;
use contextra_durable_fs::durable_remove;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Detailed outcome of a purge pipeline execution run.
#[derive(Debug, Error)]
pub enum PurgeOutcome {
    /// Pipeline completed all phases successfully.
    #[error("Purge completed: {0:?}")]
    Completed(PurgeReport),
    /// Pipeline completed all phases, but obsolete file disposal encountered refusals.
    #[error("Purge completed with {refusals} disposal refusals: {report:?}")]
    CompletedWithRefusals {
        report: PurgeReport,
        refusals: usize,
    },
    /// Pipeline failed at `phase` with typed `PurgeError`.
    #[error("Purge failed at phase {phase:?}: {error}")]
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

    /// Converts the outcome into a Result<PurgeReport, PurgeError>.
    pub fn into_result(self) -> Result<PurgeReport, PurgeError> {
        match self {
            PurgeOutcome::Completed(report) => Ok(report),
            PurgeOutcome::CompletedWithRefusals { report, .. } => Ok(report),
            PurgeOutcome::Failed { error, .. } => Err(error),
        }
    }
}

/// Hooks callback function type.
pub type HookCallback = Box<dyn Fn() -> std::io::Result<()> + Send + Sync>;

/// Scope reconstruction callback function type.
pub type ScopeReconstructFn =
    Box<dyn Fn(Option<u64>, Option<[u8; 32]>) -> Result<PurgeScope, PurgeError> + Send + Sync>;

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

    // TODO(WP-4 / PR-13 / N1): Befund N1 beheben (Resume ist No-op bei `storage: None`):
    // `new_standalone` initialisiert `storage: None`. Wird dieser Orchestrator für Recovery/Resume
    // (`resume_pending_purge` in `resume.rs`) aufgerufen, wird Phase P4 (`PurgeCompaction`)
    // stillschweigend übersprungen, da `self.storage` `None` ist. Das Intent-File wird abgeräumt,
    // ohne dass die Schlüssel jemals per Compaction gelöscht wurden.
    // Implementierungsaufgabe:
    // Standalone-Resume muss entweder einen echten `LsmStorage`-Handle erhalten oder eine deterministische
    // Offline-Compaction durchführen, damit anstehende Purges nach Absturz vollständig ausgeführt werden.
    /// Creates a standalone `PurgeOrchestrator` without active storage.
    ///
    /// Running compaction (Phase P4) with standalone orchestrator will fail-closed with an error
    /// to ensure `purge.intent` is preserved until active storage is attached.
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
    ) -> Result<(), PurgeError> {
        intent.phase = phase;
        intent
            .write_to_file(&self.intent_file)
            .map_err(|e| PurgeError::Write {
                file: self.intent_file.to_string_lossy().to_string(),
                source: e.to_string(),
            })
    }

    /// Executes complete purge pipeline.
    pub async fn execute_purge(
        &self,
        scope: PurgeScope,
        as_of_tx: u64,
        space_probe: Option<&dyn FreeSpaceProbe>,
        wal_callback: Option<Box<dyn FnOnce() -> Result<(), PurgeBlocked> + Send>>,
    ) -> std::result::Result<PurgeReport, PurgeError> {
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
                error: e,
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
                error: e,
            };
        }

        if let Err(e) = self.check_failpoint("after_intent") {
            return PurgeOutcome::Failed {
                phase: PurgePhase::Admitted,
                error: e,
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
    ) -> Result<PurgeReport, PurgeError> {
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
            intent.phase >= PurgePhase::Fenced && intent.phase < PurgePhase::Released;

        if fence_acquired {
            if let Some(storage) = self.storage {
                if !fence_prefixes.is_empty() {
                    if let Ok(guard) = storage.erasure_fence.acquire_scope_fence_for(fence_prefixes.clone()) {
                        fence_guard = Some(guard);
                    }
                }
            }
        }

        let run_pipeline = async {
            // Phase 2: Fenced
            if intent.phase.must_run(PurgePhase::Fenced) {
                if let Some(storage) = self.storage {
                    if !fence_prefixes.is_empty() {
                        let guard = storage
                            .erasure_fence
                            .acquire_scope_fence_for(fence_prefixes.clone())
                            .map_err(|_| PurgeError::Blocked(PurgeBlocked::CompactionBusy))?;
                        fence_guard = Some(guard);
                    }
                }
                if let Some(ref f) = hooks.fence_acquire {
                    f().map_err(|e| PurgeError::Fence(e.to_string()))?;
                }
                fence_acquired = true;
                self.record_phase(&mut intent, PurgePhase::Fenced)?;
                self.check_failpoint("after_fence")?;
            }

            // Phase 3: Flushed
            if intent.phase.must_run(PurgePhase::Flushed) {
                if let Some(ref f) = hooks.flush_memtable {
                    f().map_err(|e| PurgeError::Write {
                        file: "flush_memtable".to_string(),
                        source: e.to_string(),
                    })?;
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
            };

            if intent.phase.must_run(PurgePhase::Rewriting) {
                self.check_failpoint("mid_compaction_output")?;

                // TODO(WP-4 / PR-13 / N1): Bei `self.storage == None` wird `purge_keys` komplett übersprungen!
                // Für Standalone-Recovery muss hier entweder `storage` bereitgestellt oder ein Fehler
                // zurückgegeben werden, damit kein Intent fälschlicherweise als erfolgreich gelöscht markiert wird.
                if let Some(storage) = self.storage {
                    report = storage
                        .purge_keys(scope, intent.as_of_tx, space_probe)
                        .await?;
                    self.record_phase(&mut intent, PurgePhase::Rewriting)?;
                } else {
                    return Err(PurgeError::Write {
                        file: self.intent_file.to_string_lossy().to_string(),
                        source: "Storage handle required for PurgeCompaction (P4)".to_string(),
                    });
                }

                self.check_failpoint("after_compaction_before_manifest")?;
            }

            // Phase 5: Disposing
            let mut refusals = 0;
            if intent.phase.must_run(PurgePhase::Disposing) {
                self.record_phase(&mut intent, PurgePhase::Disposing)?;
                self.check_failpoint("after_manifest_before_unlink")?;

                let disp_rep = dispose_obsolete_files(
                    &self.storage_dir,
                    report.obsolete_files.clone(),
                    secure_delete,
                )
                .await?;
                refusals = disp_rep.refusals.len();

                self.check_failpoint("mid_dispose_journal")?;
                self.check_failpoint("after_unlink")?;
            }

            // Phase 6: WalRetiring
            if intent.phase.must_run(PurgePhase::WalRetiring) {
                self.record_phase(&mut intent, PurgePhase::WalRetiring)?;
                if let Some(ref cb) = hooks.wal_retire {
                    cb().map_err(|e| PurgeError::Write {
                        file: "wal_retire".to_string(),
                        source: e.to_string(),
                    })?;
                }
                self.check_failpoint("after_wal_callback")?;
            }

            // Phase 7: Witnessing & Phase 8: Finalizing
            if intent.phase.must_run(PurgePhase::Witnessing) {
                self.record_phase(&mut intent, PurgePhase::Witnessing)?;
                if let Some(ref cb) = hooks.witness {
                    cb().map_err(|e| PurgeError::Write {
                        file: "witness".to_string(),
                        source: e.to_string(),
                    })?;
                }
                self.check_failpoint("after_witness")?;
            }

            if intent.phase.must_run(PurgePhase::Finalizing) {
                self.record_phase(&mut intent, PurgePhase::Finalizing)?;
                self.check_failpoint("after_finalize")?;
            }

            // Phase 9: Released
            if intent.phase.must_run(PurgePhase::Released) {
                fence_guard.take();
                if let Some(ref f) = hooks.fence_release {
                    f().map_err(|e| PurgeError::Fence(e.to_string()))?;
                }
                fence_acquired = false;

                self.record_phase(&mut intent, PurgePhase::Released)?;
                self.check_failpoint("after_release")?;

                if self.intent_file.exists() {
                    durable_remove(&self.intent_file).map_err(|e| PurgeError::Write {
                        file: self.intent_file.to_string_lossy().to_string(),
                        source: e.to_string(),
                    })?;
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
                    error: err,
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
    ) -> Result<PurgeReport, PurgeError> {
        self.execute_from_intent_outcome(intent, scope, hooks, secure_delete, space_probe)
            .await
            .into_result()
    }

    fn check_failpoint(&self, name: &'static str) -> Result<(), PurgeError> {
        if let Some(ref set) = self.failpoints {
            if set.is_active(name) {
                #[cfg(any(test, feature = "fault-injection"))]
                return Err(PurgeError::Failpoint(name));
            }
        }
        #[cfg(any(test, feature = "fault-injection"))]
        if std::env::var(format!("PURGE_FAILPOINT_{}", name)).is_ok() {
            return Err(PurgeError::Failpoint(name));
        }
        Ok(())
    }
}
