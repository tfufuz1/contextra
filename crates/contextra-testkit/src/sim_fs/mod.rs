// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Simulator File System (SimFs) module exports.
// INVARIANTEN: Zero unsafe, fully deterministic.

pub mod fs;
pub mod vfs_mirror;

pub use fs::{SimFs, SimFsConfig};
pub use vfs_mirror::{SimOpenOptions, SimVfs, SimVfsFile};
