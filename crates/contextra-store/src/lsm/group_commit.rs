use crate::lsm::config::DurabilityMode;
use crate::wal::{PreparedBatch, Wal};
use contextra_core::{Result, TxId};
use std::sync::Arc;

pub(super) async fn execute_group_commit_append(
    wal: &Wal,
    durability_mode: DurabilityMode,
    all_wal_entries: PreparedBatch,
    truncate_guard: &tokio::sync::MutexGuard<'_, ()>,
    prev_hmac_snapshot: [u8; 32],
) -> Result<()> {
    match durability_mode {
        DurabilityMode::Full => {
            wal.append_batch_locked(all_wal_entries, truncate_guard)
                .await
        }
        DurabilityMode::WalNoHmac => {
            let res = wal
                .append_batch_locked(all_wal_entries, truncate_guard)
                .await;
            if res.is_ok() {
                let _ = wal.restore_last_hmac(prev_hmac_snapshot).await;
            }
            res
        }
        DurabilityMode::MemoryOnly => Ok(()),
    }
}

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
