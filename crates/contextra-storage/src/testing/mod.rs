//! # Testing Modul
//!
//! Testwerkzeuge, deterministische Test-Dateisysteme (`MemFs`, `FaultFs`),
//! Test-Hooks und Konformitätssuiten für `contextra-storage`.

pub mod conformance;
pub mod fault_fs;
pub mod hooks;
pub mod mem_fs;

pub use fault_fs::{CrashMode, FaultConfig, FaultFs, XorShift64};
pub use hooks::{HookPoint, TestHooks};
pub use mem_fs::MemFs;
