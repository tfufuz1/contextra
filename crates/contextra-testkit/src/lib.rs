// FILE-CONTEXT
// STAND: 2026-10-25
// ZWECK: Deterministic test utilities, ManualClock, InMemoryStorageEngine, FaultVfs, ReferenceModel, and Mocks.
// INVARIANTEN: No unsafe code allowed in testkit (#![forbid(unsafe_code)]).

#![forbid(unsafe_code)]

pub mod fault_vfs;
pub mod in_memory_store;
pub mod manual_clock;
pub mod mocks;
pub mod reference_model;

pub use fault_vfs::{FaultConfig, FaultVfs};
pub use in_memory_store::InMemoryStorageEngine;
pub use manual_clock::ManualClock;
pub use mocks::*;
pub use reference_model::{RefOp, ReferenceModel};
