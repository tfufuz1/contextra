// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Export-Modul für Purge-Pipeline.

pub mod dispose;
pub mod failpoint;
pub mod intent;
pub mod orchestrator;
pub mod plan;
pub mod resume;

pub use dispose::OrxDisposeReport;
pub use failpoint::{
    OrxFailpointSet, OrxPurgeError, ALL_PURGE_FAILPOINTS, ENGINE_PURGE_FAILPOINTS,
    STORE_PURGE_FAILPOINTS,
};
pub use intent::OrxPurgeIntent;
pub use orchestrator::{OrxHooks, PurgeOrchestrator};
pub use plan::PurgePhase;
pub use resume::{resume_pending, OrxResumeReport};
