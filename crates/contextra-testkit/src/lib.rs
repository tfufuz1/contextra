// FILE-CONTEXT
// STAND: 2026-10-18
// ZWECK: Deterministic test utilities, ManualClock, InMemoryStorageEngine, FaultVfs, ReferenceModel, Erasure Harness, Deletion Oracle, DST, SimFS, SeededRng, and Wiring.
// INVARIANTEN: No unsafe code allowed in testkit (#![forbid(unsafe_code)]).

#![forbid(unsafe_code)]

pub mod deletion_oracle;
pub mod dst;
pub mod erasure_harness;
pub mod fault_vfs;
pub mod in_memory_store;
pub mod manual_clock;
pub mod reference_model;
pub mod seeded_rng;
pub mod sim_fs;
pub mod wiring;

pub use deletion_oracle::{raw_sweep, OrcFieldMarker, OrcHit, OrcMarkerSet, OrcSweepReport};
pub use dst::{EntropyRng, SeededRng};
#[cfg(feature = "fault-injection")]
pub use erasure_harness::FailpointHelper;
pub use erasure_harness::{KeyState, ReferenceStore, SweepHit, SweepReport};
#[cfg(not(loom))]
pub use erasure_harness::{TenantMatrixContext, TenantMatrixMode, TenantMatrixRunner};
pub use fault_vfs::{FaultConfig, FaultVfs};
pub use in_memory_store::InMemoryStorageEngine;
pub use manual_clock::ManualClock;
pub use reference_model::{RefOp, ReferenceModel};
pub use seeded_rng::SeededRng as TestkitSeededRng;
pub use wiring::{TestkitWiring, TestkitWiringDeps};
