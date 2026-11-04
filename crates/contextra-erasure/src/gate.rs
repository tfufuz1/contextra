//! Gate-Subsystem (ErasedSet, GateLog, ReadGate, Batch-Commit).

pub mod batch;
pub mod erased_set;
pub mod log;
pub mod read_gate;

pub use erased_set::ErasedSet;
pub use log::{GateLog, GateLogEntry};
pub use read_gate::GateGuard;
