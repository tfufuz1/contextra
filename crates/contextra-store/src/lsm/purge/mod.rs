// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Export-Modul für Purge-Pipeline.

pub mod dispose;
pub mod failpoint;
pub mod intent;
pub mod orchestrator;
pub mod plan;
pub mod resume;
pub mod wiring;

pub use dispose::OrxDisposeReport;
pub use failpoint::{OrxFailpointSet, OrxPurgeError, CANONICAL_FAILPOINTS};
pub use intent::OrxPurgeIntent;
pub use orchestrator::{HookCallback, OrxHooks, PurgeOrchestrator, ScopeReconstructFn};
pub use plan::PurgePhase;
pub use resume::resume_pending;
pub use wiring::PurgeWiring;
