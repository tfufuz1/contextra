//! Read snapshot and snapshot reader port traits (§7.2).

#![forbid(unsafe_code)]

use contextra_types::{Result, TxId};
use serde::{Deserialize, Serialize};

/// Unique identifier for a read snapshot (§7.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SnapshotId(pub u64);

impl SnapshotId {
    /// Creates a new `SnapshotId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the raw numeric value.
    pub const fn inner(self) -> u64 {
        self.0
    }
}

/// Read snapshot token for isolation across search and storage queries (§7.2).
///
/// # Invariants
/// - Does not derive `Copy` (explicit cloning required to track snapshot reference counts).
/// - Contains no wall-clock `Instant` or system time (age measured via Clock port ticks).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadSnapshot {
    /// Snapshot identifier.
    pub id: SnapshotId,
    /// Maximum committed sequence / transaction number visible in this snapshot.
    pub max_seq: TxId,
}

impl ReadSnapshot {
    /// Creates a new `ReadSnapshot`.
    pub fn new(id: SnapshotId, max_seq: TxId) -> Self {
        Self { id, max_seq }
    }

    /// Convenience constructor for sequence-bound snapshots.
    pub fn from_seq(seq_no: u64) -> Self {
        Self {
            id: SnapshotId::new(seq_no),
            max_seq: TxId::try_new(seq_no).unwrap_or(TxId(seq_no)),
        }
    }

    /// Returns numeric sequence value for this snapshot.
    pub fn seq(&self) -> u64 {
        self.max_seq.0
    }

    /// Returns numeric sequence value for this snapshot (alias for `seq`).
    pub fn max_seq(&self) -> u64 {
        self.max_seq.0
    }
}

/// Trait implemented by storage engines or snapshot providers to supply consistent point-in-time snapshots (§7.2).
pub trait SnapshotReader: Send + Sync {
    /// Acquires a new [`ReadSnapshot`] pinned at the current high-water transaction ID.
    fn snapshot(&self) -> Result<ReadSnapshot>;

    /// Reads value for `key` isolated at `snap`.
    fn read_at(&self, snap: &ReadSnapshot, key: &[u8]) -> Result<Option<Vec<u8>>>;
}
