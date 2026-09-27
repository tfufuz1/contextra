use crate::wal::WalOp;
use contextra_core::TxId;
use std::time::Duration;

/// Maximum latency permitted for a synchronous `WalObserver::on_commit` invocation.
/// Exceeding this limit triggers observer deregistration and a `tracing::warn!` event
/// to enforce the fail-open durability invariant.
pub const DEFAULT_MAX_OBSERVER_LATENCY: Duration = Duration::from_millis(1);

/// Identifies the origin of a committed transaction batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteOrigin {
    /// Regular user-facing transaction write.
    UserWrite,
    /// Internal cognition memory consolidation write.
    CognitionConsolidation,
    /// Background compaction rewrite.
    CompactionRewrite,
}

/// Lightweight zero-copy reference to a staged WAL operation during commit.
#[derive(Debug, Clone, PartialEq)]
pub struct WalEntryRef<'a> {
    pub op: &'a WalOp,
    pub seq_no: u64,
}

impl<'a> WalEntryRef<'a> {
    pub fn tx_id(&self) -> TxId {
        self.op.tx_id()
    }
}

/// Represents a batch of WAL entries committed under a single transaction.
#[derive(Debug, Clone, PartialEq)]
pub struct CommittedBatch<'a> {
    pub entries: Vec<WalEntryRef<'a>>,
    pub origin: WriteOrigin,
}

/// Synchronous observer interface for WAL commits (In-Process CDC / Event Stream).
///
/// Implementations MUST NOT block execution longer than `DEFAULT_MAX_OBSERVER_LATENCY`.
/// Violations result in immediate observer deregistration (fail-open) to prevent
/// stalling the write path.
pub trait WalObserver: Send + Sync {
    fn on_commit(&self, batch: &CommittedBatch<'_>, seq_no: u64, tx_id: TxId);
}
