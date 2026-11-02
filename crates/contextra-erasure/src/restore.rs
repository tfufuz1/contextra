//! Restore-Modul (SPEC §11.8).
//!
//! Restore-Guard und Replay-Logik für den Fall eines Backup-Restores.
pub mod guard;
pub mod replay;

pub use guard::{GuardOutcome, RestoreGuard};
pub use replay::{replay_erasures, ReplayOutcome};
