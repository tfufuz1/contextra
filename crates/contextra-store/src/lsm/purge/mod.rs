// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Export-Modul für Purge-Pipeline.

pub mod backend;
pub mod dispose;
pub mod failpoint;
pub mod intent;
pub mod lsm_backend_impl;
pub mod orchestrator;
pub mod plan;
pub mod resume;
pub mod scope_file;
pub mod wiring;

pub use backend::{MockBackend, PurgeBackend};
pub use dispose::OrxDisposeReport;
pub use failpoint::{CANONICAL_FAILPOINTS, OrxFailpointSet, OrxPurgeError};
pub use intent::OrxPurgeIntent;
pub use orchestrator::{HookCallback, OrxHooks, PurgeOrchestrator, ScopeReconstructFn};
pub use plan::PurgePhase;
pub use scope_file::ScopeFile;
pub use resume::{
    resume_pending, resume_pending_for_backend, resume_pending_for_storage,
    resume_pending_for_storage_with_policy, resume_pending_with_policy,
};
pub use wiring::PurgeWiring;
