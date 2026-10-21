// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Kanonische PurgePhase nach WP-2 / Spezifikation (u8-Codes 1..=9).

use contextra_ports::erasure::PurgeError;
use serde::{Deserialize, Serialize};

/// Multi-phase purge lifecycle state machine (codes 1..=9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum PurgePhase {
    /// Request admitted, intent and scope files written.
    Admitted = 1,
    /// Erasure fence acquired on scope.
    Fenced = 2,
    /// Active memtables flushed.
    Flushed = 3,
    /// Purge rewrite executed and committed.
    Rewriting = 4,
    /// Obsolete files disposed.
    Disposing = 5,
    /// WAL retired.
    WalRetiring = 6,
    /// Witness generated and recorded.
    Witnessing = 7,
    /// Purge operation finalized.
    Finalizing = 8,
    /// Fence released and intent removed.
    Released = 9,
}

impl PurgePhase {
    /// Returns true if this phase must execute step `phase`.
    #[must_use]
    pub fn must_run(self, phase: PurgePhase) -> bool {
        (self as u8) <= (phase as u8)
    }

    /// Converts phase to numeric u8 code.
    #[must_use]
    pub fn code(self) -> u8 {
        self as u8
    }

    /// Backwards compatibility alias for code().
    #[must_use]
    pub fn to_u8(self) -> u8 {
        self.code()
    }

    /// Backwards compatibility method.
    #[must_use]
    pub fn from_u8(val: u8) -> Option<Self> {
        Self::try_from(val).ok()
    }
}

impl TryFrom<u8> for PurgePhase {
    type Error = PurgeError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            1 => Ok(PurgePhase::Admitted),
            2 => Ok(PurgePhase::Fenced),
            3 => Ok(PurgePhase::Flushed),
            4 => Ok(PurgePhase::Rewriting),
            5 => Ok(PurgePhase::Disposing),
            6 => Ok(PurgePhase::WalRetiring),
            7 => Ok(PurgePhase::Witnessing),
            8 => Ok(PurgePhase::Finalizing),
            9 => Ok(PurgePhase::Released),
            _ => Err(PurgeError::CorruptedIntent(format!(
                "Invalid PurgePhase code: {}",
                val
            ))),
        }
    }
}
