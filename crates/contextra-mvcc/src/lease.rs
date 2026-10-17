//! RAII `SnapshotLease` guard for active snapshot read isolation.

use contextra_types::ContextraError;

use crate::snapshot::SnapshotRegistry;
use crate::sync::Arc;

/// Move-only RAII lease guard for an active snapshot.
///
/// Automatically deregisters the snapshot sequence number from the [`SnapshotRegistry`]
/// when dropped.
#[derive(Debug)]
pub struct SnapshotLease {
    pub(crate) registry: Arc<SnapshotRegistry>,
    pub(crate) seq_no: u64,
    pub(crate) created_at_ticks: Option<u64>,
}

impl SnapshotLease {
    /// Creates a new `SnapshotLease`.
    pub fn new(registry: Arc<SnapshotRegistry>, seq_no: u64, created_at_ticks: Option<u64>) -> Self {
        Self {
            registry,
            seq_no,
            created_at_ticks,
        }
    }

    /// Returns the sequence number pinned by this snapshot lease.
    #[inline]
    pub fn seq_no(&self) -> u64 {
        self.seq_no
    }

    /// Returns the monotonic creation timestamp in ticks (nanoseconds), if available.
    #[inline]
    pub fn created_at_ticks(&self) -> Option<u64> {
        self.created_at_ticks
    }

    /// Validates whether the lease is still active and unexpired relative to `now_ticks`.
    ///
    /// # Errors
    /// Returns `Err(ContextraError::SnapshotExpired)` if the age of the lease exceeds `max_snapshot_age`.
    pub fn check_valid_at(&self, now_ticks: u64) -> Result<(), ContextraError> {
        self.registry.check_lease_valid(self, now_ticks)
    }
}

impl Drop for SnapshotLease {
    fn drop(&mut self) {
        self.registry.release_at(self.seq_no, self.created_at_ticks);
    }
}
