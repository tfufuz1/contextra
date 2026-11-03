//! Gate-Modul (SPEC §11.3).
pub mod erased_set;
pub mod read_gate;
pub mod log;
pub mod batch;

pub use erased_set::ErasedSet;
pub use read_gate::GateGuard;
pub use log::{GateLog, GateLogEntry};
