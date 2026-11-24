//! Restore-Modul (SPEC §11.8).
//!
//! Restore-Guard und Replay-Logik für den Fall eines Backup-Restores.

pub mod guard;
pub mod replay;

pub use guard::{FileRestoreChecker, GuardOutcome};
pub use replay::{replay_erasures, ReplayOutcome};

use crate::types_local::Seq;

/// Restore-Guard prüft Sequenznummern gegen den Ledger-Floor.
#[derive(Debug, Clone, Copy)]
pub struct RestoreGuard {
    ledger_floor: Seq,
}

impl RestoreGuard {
    /// Erstellt einen neuen RestoreGuard mit einem Ledger-Floor.
    pub fn new(ledger_floor: Seq) -> Self {
        Self { ledger_floor }
    }

    /// Prüft, ob eine Sequenznummer kleiner oder gleich dem Floor ist (Replay / Altbestand).
    pub fn is_replay(&self, seq: Seq) -> bool {
        seq.as_u64() <= self.ledger_floor.as_u64()
    }
}
