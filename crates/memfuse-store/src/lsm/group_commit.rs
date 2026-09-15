use crate::wal::PreparedBatch;
use memfuse_core::{Result, TxId};
use std::sync::Arc;

pub(super) struct GroupCommitRequest {
    pub(super) tx_id: TxId,
    pub(super) wal_entries: PreparedBatch,
    pub(super) mem_updates: Vec<(Vec<u8>, Vec<u8>, u64)>,
    pub(super) sender: tokio::sync::oneshot::Sender<Result<()>>,
}

pub(super) struct WalQueueGuard(pub(super) Arc<std::sync::atomic::AtomicUsize>);

impl WalQueueGuard {
    pub(super) fn new(counter: Arc<std::sync::atomic::AtomicUsize>) -> Self {
        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self(counter)
    }
}

impl Drop for WalQueueGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }
}

pub(super) struct PendingCommitQueue {
    pub(super) requests: Vec<GroupCommitRequest>,
    pub(super) first_prev_hmac: [u8; 32],
    pub(super) notify_full: Arc<tokio::sync::Notify>,
}
