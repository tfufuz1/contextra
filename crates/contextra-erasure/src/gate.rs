//! Gate-Subsystem (ErasedSet, GateLog, ReadGate).

pub mod erased_set;
pub mod log;

pub use erased_set::{ErasedSet, GateKind, SharedReadGate};
pub use log::{GateLog, GateLogEntry};
