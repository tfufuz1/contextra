// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: Deterministic test utilities, ManualClock, InMemoryStorageEngine, FaultVfs, ReferenceModel, and Erasure Harness.
// INVARIANTEN: No unsafe code allowed in testkit (#![forbid(unsafe_code)]).

#![forbid(unsafe_code)]

pub mod erasure_harness;
pub mod fault_vfs;
pub mod in_memory_store;
pub mod manual_clock;
pub mod reference_model;

#[cfg(feature = "fault-injection")]
pub use erasure_harness::FailpointHelper;
pub use erasure_harness::{
    KeyState, ReferenceStore, SweepHit, SweepReport, TenantMatrixContext, TenantMatrixMode,
    TenantMatrixRunner,
};
pub use fault_vfs::{FaultConfig, FaultVfs};
pub use in_memory_store::InMemoryStorageEngine;
pub use manual_clock::ManualClock;
pub use reference_model::{RefOp, ReferenceModel};
