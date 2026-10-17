//! Deletion capabilities descriptor for redb storage engine adapter.

use serde::{Deserialize, Serialize};

/// Deletion stage achievable by a storage engine backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DeletionStage {
    /// S0: Logical deletion / tombstone written.
    S0,
    /// S1: Key removed from index / free-listed (bytes remain in database file free-list pages until reused).
    S1,
    /// S2: Physical purge / zeroing / file rewrite.
    S2,
}

/// Capability descriptor stating deletion semantics for redb backend.
///
/// # Honest Deletion Semantics
/// In redb, deleting an entry (`table.remove()`) releases the containing B-tree pages
/// to redb's internal free-list. The raw bytes of deleted keys/values remain in the `.redb` file
/// until overwritten by future page allocations or vacuum operations.
/// Therefore, physical disk scrubbing/zeroing is **not** available, and forget receipts on
/// this backend can reach at most stage **S1** (Logical free-listing) unless crypto-shredding is used.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedbDeletionCapabilities {
    /// Indicates whether physical purge (secure disk scrubbing/overwriting) is supported.
    /// For redb, this is always `false`.
    pub physical_purge_supported: bool,
    /// The maximum deletion stage achievable by this storage backend without crypto-shredding.
    /// For redb, this is [`DeletionStage::S1`].
    pub max_deletion_stage: DeletionStage,
    /// Explanation of backend deletion behavior.
    pub explanation: &'static str,
}

impl Default for RedbDeletionCapabilities {
    fn default() -> Self {
        Self {
            physical_purge_supported: false,
            max_deletion_stage: DeletionStage::S1,
            explanation: "Redb retains deleted key/value bytes in B-tree free-list pages until reused by subsequent allocations; physical disk zeroing/purge is not supported.",
        }
    }
}
