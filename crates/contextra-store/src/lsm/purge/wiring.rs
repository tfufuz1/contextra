// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Composition Root Wiring für Purge-Orchestrator (Vertrag §7).

use super::dispose::{DisposePolicy, OrxDisposeReport};
use super::failpoint::{OrxFailpointSet, OrxPurgeError};
use super::orchestrator::{OrxHooks, PurgeOrchestrator};
use super::resume::resume_pending_with_policy;
use crate::compaction::purge_filter::PurgeReport;
use contextra_core::Result;
use contextra_ports::Clock;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Dependencies for PurgeWiring composition root.
pub struct PurgeWiringDeps {
    /// Target database storage directory.
    pub storage_dir: PathBuf,
    /// Lifecycle hooks and callbacks.
    pub hooks: OrxHooks,
    /// Deterministic clock port.
    pub clock: Option<Arc<dyn Clock>>,
    /// Active failpoint set for crash testing.
    pub failpoints: Option<OrxFailpointSet>,
    /// Explicit disposal policy for file scrubbing.
    pub dispose_policy: DisposePolicy,
}

impl std::fmt::Debug for PurgeWiringDeps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PurgeWiringDeps")
            .field("storage_dir", &self.storage_dir)
            .field("hooks", &self.hooks)
            .field("clock", &self.clock.is_some())
            .field("failpoints", &self.failpoints)
            .field("dispose_policy", &self.dispose_policy)
            .finish()
    }
}

/// Wiring facade composition root for LSM Purge subsystem.
pub struct PurgeWiring {
    storage_dir: PathBuf,
    hooks: OrxHooks,
    clock: Option<Arc<dyn Clock>>,
    failpoints: Option<OrxFailpointSet>,
    dispose_policy: DisposePolicy,
}

impl PurgeWiring {
    /// Constructs a new `PurgeWiring` instance with explicit dependencies.
    pub fn build(deps: PurgeWiringDeps) -> Result<Self> {
        Ok(Self {
            storage_dir: deps.storage_dir,
            hooks: deps.hooks,
            clock: deps.clock,
            failpoints: deps.failpoints,
            dispose_policy: deps.dispose_policy,
        })
    }

    /// Convenience builder from storage path with default hooks and policy.
    pub fn build_default(storage_dir: impl AsRef<Path>) -> Result<Self> {
        Self::build(PurgeWiringDeps {
            storage_dir: storage_dir.as_ref().to_path_buf(),
            hooks: OrxHooks::default(),
            clock: None,
            failpoints: None,
            dispose_policy: DisposePolicy::RequireScrub,
        })
    }

    /// Resumes any pending or interrupted purge operation present in the storage directory.
    pub async fn resume_pending(&self) -> std::result::Result<Option<(PurgeReport, OrxDisposeReport)>, OrxPurgeError> {
        resume_pending_with_policy(
            &self.storage_dir,
            &self.hooks,
            self.failpoints.as_ref(),
            self.dispose_policy,
        )
        .await
    }

    /// Constructs a `PurgeOrchestrator` bound to the storage directory and failpoint settings.
    pub fn orchestrator_standalone(&self) -> PurgeOrchestrator<'static> {
        let mut orch = PurgeOrchestrator::new_standalone(&self.storage_dir);
        if let Some(ref fp) = self.failpoints {
            orch = orch.with_failpoints(fp.clone());
        }
        orch
    }

    /// Returns references to hooks.
    pub fn hooks(&self) -> &OrxHooks {
        &self.hooks
    }

    /// Returns the storage directory path.
    pub fn storage_dir(&self) -> &Path {
        &self.storage_dir
    }

    /// Returns the optional Clock reference.
    pub fn clock(&self) -> Option<&Arc<dyn Clock>> {
        self.clock.as_ref()
    }
}
