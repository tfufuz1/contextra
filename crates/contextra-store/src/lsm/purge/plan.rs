//! # Purge Phase Lifecycle and Crash Consistency Model
//!
//! | Phase | Idempotence Strategy | Persisted State / On-Disk Artefact |
//! |---|---|---|
//! | `Admitted` | Read intent or record new intent | `purge.intent` recorded with scope ledger ref |
//! | `Flush` | Re-flush immutable memtables (no-op if empty) | SSTables flushed to disk, WAL updated |
//! | `PurgeCompaction` | Re-run compaction or skip if SSTable manifest already updated | New SSTables written, Manifest committed |
//! | `Dispose` | Process pending delete journal (skip missing files) | `pending_delete.journal` updated/cleared |
//! | `WalRetire` | Callback re-execution or WAL segment marker check | WAL segment retired / tombstoned |
//! | `Witness` | Issue/verify deletion proof witness | `LayerCleanupProof` or witness record |
//! | `Finalize` | Confirm purge transaction commit | Manifest/Ledger final state recorded |
//! | `Release` | Erasure Fence release & Intent file unlinked | `purge.intent` removed |

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PurgePhase {
    /// Phase 1: Request admitted, intent file initialized with scope ledger reference.
    Admitted,
    /// Phase 2: Active memtables flushed to SSTables.
    Flush,
    /// Phase 3: Purge compaction executed and committed to manifest.
    PurgeCompaction,
    /// Phase 4: Obsolete files recorded to journal and durably deleted.
    Dispose,
    /// Phase 5: WAL callback executed to retire obsolete WAL records.
    WalRetire,
    /// Phase 6: Deletion proof witness generated/recorded.
    Witness,
    /// Phase 7: Purge operation finalized.
    Finalize,
    /// Phase 8: Erasure fence released and intent file removed.
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

impl PurgePhase {
    /// Returns true if this phase is at or past `other`.
    #[must_use]
    pub fn is_at_least(self, other: Self) -> bool {
        self.order() >= other.order()
    }

    fn order(self) -> u8 {
        match self {
            PurgePhase::Admitted | PurgePhase::P0Init | PurgePhase::P1Intent => 1,
            PurgePhase::P2FenceAcquired => 2,
            PurgePhase::Flush | PurgePhase::P3Flush => 3,
            PurgePhase::PurgeCompaction | PurgePhase::P4PurgeCompaction => 4,
            PurgePhase::Dispose | PurgePhase::P5Dispose => 5,
            PurgePhase::WalRetire | PurgePhase::P6WalCallback => 6,
            PurgePhase::Witness => 7,
            PurgePhase::Finalize => 8,
            PurgePhase::Release | PurgePhase::P7Complete => 9,
        }
    }
}
