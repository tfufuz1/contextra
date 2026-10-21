// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: PurgePhase Lifecycle Enum und Phasenkontrolle (Spezifikation C5.1).

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PurgePhase {
    /// Phase 0: Pipeline initialized.
    Init,
    /// Phase 1: Request admitted, intent file initialized with scope ledger reference.
    Admitted,
    /// Phase 2: Erasure fence acquired on scope.
    FenceAcquired,
    /// Phase 3: Active memtables flushed to SSTables.
    Flush,
    /// Phase 4: Purge compaction executed and committed to manifest.
    PurgeCompaction,
    /// Phase 5: Obsolete files recorded to journal and durably deleted.
    Dispose,
    /// Phase 6: WAL callback executed to retire obsolete WAL records.
    WalRetire,
    /// Phase 7: Deletion proof witness generated/recorded.
    Witness,
    /// Phase 8: Purge operation finalized.
    Finalize,
    /// Phase 9: Erasure fence released and intent file removed.
    Release,

    // Legacy phase enum variants preserved for backwards compatibility with existing serialized state.
    P0Init,
    P1Intent,
    P2FenceAcquired,
    P3Flush,
    P4PurgeCompaction,
    P5Dispose,
    P6WalCallback,
    P7Complete,
}

impl PartialOrd for PurgePhase {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PurgePhase {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order().cmp(&other.order())
    }
}

impl PurgePhase {
    /// Returns true if this phase is at or past `other`.
    #[must_use]
    pub fn is_at_least(self, other: Self) -> bool {
        self.order() >= other.order()
    }

    fn order(self) -> u8 {
        match self {
            PurgePhase::Init | PurgePhase::P0Init => 0,
            PurgePhase::Admitted | PurgePhase::P1Intent => 1,
            PurgePhase::FenceAcquired | PurgePhase::P2FenceAcquired => 2,
            PurgePhase::Flush | PurgePhase::P3Flush => 3,
            PurgePhase::PurgeCompaction | PurgePhase::P4PurgeCompaction => 4,
            PurgePhase::Dispose | PurgePhase::P5Dispose => 5,
            PurgePhase::WalRetire | PurgePhase::P6WalCallback => 6,
            PurgePhase::Witness => 7,
            PurgePhase::Finalize => 8,
            PurgePhase::Release | PurgePhase::P7Complete => 9,
        }
    }

    /// Converts phase to u8 numeric code for binary encoding.
    #[must_use]
    pub fn to_u8(self) -> u8 {
        self.order()
    }

    /// Parses phase from u8 numeric code.
    #[must_use]
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::Init),
            1 => Some(Self::Admitted),
            2 => Some(Self::FenceAcquired),
            3 => Some(Self::Flush),
            4 => Some(Self::PurgeCompaction),
            5 => Some(Self::Dispose),
            6 => Some(Self::WalRetire),
            7 => Some(Self::Witness),
            8 => Some(Self::Finalize),
            9 => Some(Self::Release),
            _ => None,
        }
    }
}
