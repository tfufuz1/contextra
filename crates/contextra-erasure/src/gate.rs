//! Gate-Subsystem (ErasedSet, GateLog, ReadGate, Batch-Commit, Fence).

pub mod batch;
pub mod erased_set;
pub mod fence;
pub mod log;
pub mod read_gate;

pub use erased_set::ErasedSet;
pub use fence::{
    FenceGate, FenceGateError, FenceRange, FenceToken, InMemoryFenceGate, InMemoryFenceGate as MemoryFenceGate, WriterGuard,
};
pub use log::{GateLog, GateLogEntry};
pub use read_gate::GateGuard;
