use crate::lsm::config::DurabilityMode;
use crate::wal::{PreparedBatch, Wal, WalOp};
use contextra_core::{Result, TxId};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub(super) async fn execute_group_commit_append(
    wal: &Wal,
    durability_mode: DurabilityMode,
    all_wal_entries: PreparedBatch,
    truncate_guard: tokio::sync::MutexGuard<'_, ()>,
    committed_flag: &AtomicBool,
) -> Result<()> {
    match durability_mode {
        DurabilityMode::Full | DurabilityMode::WalNoHmac => {
            let start_offset = wal.size();
            let start_hmac = wal.last_hmac_snapshot().await;
            let append_res = wal
                .append_batch_locked(all_wal_entries, &truncate_guard)
                .await;
            if append_res.is_err() {
                if let Err(trunc_err) = wal.truncate(start_offset, start_hmac).await {
                    tracing::error!(
                        "Failed to truncate WAL after failed group append_batch: {trunc_err}"
                    );
                }
                if wal.is_poisoned() {
                    match wal.recover_from_poison().await {
                        Ok(()) => tracing::info!("WAL successfully recovered from poison state"),
                        Err(rec_err) => {
                            tracing::error!("Failed to recover WAL from poison state: {rec_err}")
                        }
                    }
                }
                drop(truncate_guard);
            } else {
                committed_flag.store(true, Ordering::Release);
                drop(truncate_guard);
            }
            append_res
        }
        DurabilityMode::MemoryOnly => {
            committed_flag.store(true, Ordering::Release);
            drop(truncate_guard);
            Ok(())
        }
    }
}

pub(super) struct GroupCommitRequest {
    pub(super) tx_id: TxId,
    pub(super) wal_ops: Vec<(WalOp, u64)>,
    pub(super) mem_updates: Vec<(Vec<u8>, Vec<u8>, u64)>,
    pub(super) sender: tokio::sync::oneshot::Sender<Result<()>>,
}

pub(super) struct WalQueueGuard(pub(super) Arc<std::sync::atomic::AtomicUsize>);

impl WalQueueGuard {
    pub(super) fn new(counter: Arc<std::sync::atomic::AtomicUsize>) -> Self {
        counter.fetch_add(1, Ordering::Relaxed);
        Self(counter)
    }
}

impl Drop for WalQueueGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

pub(super) struct PendingCommitQueue {
    pub(super) requests: Vec<GroupCommitRequest>,
    pub(super) notify_full: Arc<tokio::sync::Notify>,
    pub(super) committed_flag: Arc<AtomicBool>,
}
