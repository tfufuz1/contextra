//! RAII `SnapshotLease` guard for active snapshot read isolation.

use std::sync::Arc;
use std::time::Instant;

use crate::snapshot::SnapshotRegistry;

/// Move-only RAII lease guard for an active snapshot.
///
/// Automatically deregisters the snapshot sequence number from the [`SnapshotRegistry`]
/// when dropped.
#[derive(Debug)]
pub struct SnapshotLease {
    pub(crate) registry: Arc<SnapshotRegistry>,
    pub(crate) seq_no: u64,
    pub(crate) created_at: Option<Instant>,
}

impl SnapshotLease {
    /// Creates a new `SnapshotLease`.
    pub fn new(registry: Arc<SnapshotRegistry>, seq_no: u64, created_at: Option<Instant>) -> Self {
        Self {
            registry,
            seq_no,
            created_at,
        }
    }

    /// Returns the sequence number pinned by this snapshot lease.
    #[inline]
    pub fn seq_no(&self) -> u64 {
        self.seq_no
    }
}

impl Drop for SnapshotLease {
    fn drop(&mut self) {
        self.registry.release_at(self.seq_no, self.created_at);
    }
}
