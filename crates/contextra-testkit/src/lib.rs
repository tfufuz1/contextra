// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Deterministic test utilities, ManualClock, InMemoryStorageEngine, FaultVfs, SimFs, and ReferenceModel.
// INVARIANTEN: No unsafe code allowed in testkit (#![forbid(unsafe_code)]).

#![forbid(unsafe_code)]

pub mod deletion_oracle;
pub mod dst;
pub mod fault_vfs;
pub mod in_memory_store;
pub mod manual_clock;
pub mod reference_model;
pub mod sim_fs;

pub use deletion_oracle::{raw_sweep, OrcFieldMarker, OrcHit, OrcMarkerSet, OrcSweepReport};
pub use fault_vfs::{FaultConfig, FaultVfs};
pub use in_memory_store::InMemoryStorageEngine;
pub use manual_clock::ManualClock;
pub use reference_model::{RefOp, ReferenceModel};
pub use sim_fs::{SimFs, SimFsConfig, SimOpenOptions, SimVfs, SimVfsFile};
