// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Consolidated PurgePhase Lifecycle Enum und Phasenkontrolle (Spezifikation C5.1, WP-2).

use crate::compaction::purge_filter::PurgeError;

/// Purge operation phase state machine (§C5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PurgePhase {
    /// Phase 1: Request admitted, intent file initialized with scope ledger reference.
    Admitted = 1,
    /// Phase 2: Erasure fence acquired on scope.
    Fenced = 2,
    /// Phase 3: Active memtables flushed to SSTables.
    Flushed = 3,
    /// Phase 4: Purge compaction executed and committed to manifest.
    Rewriting = 4,
    /// Phase 5: Obsolete files recorded to journal and durably deleted.
    Disposing = 5,
    /// Phase 6: WAL callback executed to retire obsolete WAL records.
    WalRetiring = 6,
    /// Phase 7: Deletion proof witness generated/recorded.
    Witnessing = 7,
    /// Phase 8: Purge operation finalized.
    Finalizing = 8,
    /// Phase 9: Erasure fence released and intent file removed.
    Released = 9,
}

impl PurgePhase {
    /// Converts phase to u8 numeric code for binary encoding.
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Returns `true` if `self` phase is strictly before `target` phase (i.e. `self < target`).
    #[must_use]
    pub fn must_run(self, target: Self) -> bool {
        self < target
    }
}

impl TryFrom<u8> for PurgePhase {
    type Error = PurgeError;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        match code {
            1 => Ok(Self::Admitted),
            2 => Ok(Self::Fenced),
            3 => Ok(Self::Flushed),
            4 => Ok(Self::Rewriting),
            5 => Ok(Self::Disposing),
            6 => Ok(Self::WalRetiring),
            7 => Ok(Self::Witnessing),
            8 => Ok(Self::Finalizing),
            9 => Ok(Self::Released),
            0 => Err(PurgeError::Intent("Phase code 0 is invalid (never persisted)".to_string())),
            c => Err(PurgeError::Intent(format!("Invalid or legacy PurgePhase code: {c}"))),
        }
    }
}
