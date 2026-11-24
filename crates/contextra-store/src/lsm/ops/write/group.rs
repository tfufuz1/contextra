//! # Group-Commit Pfad (Ring 1)
//!
//! // FILE-CONTEXT
//! // STAND: 2026-11-11
//! // ZWECK: Leader- und Follower-Pfade fuer Gruppen-Commits
//! // INVARIANTEN: ADR-0XX-P0A (Lock-Handoff), INV-LOCK-ORDER-1

use super::super::super::config::DurabilityMode;
use super::super::super::engine::LsmStorage;
use super::super::super::group_commit::{
    execute_group_commit_append, GroupCommitRequest, PendingCommitQueue, WalQueueGuard,
};
use super::super::super::observer::WriteOrigin;
use super::super::super::{Wal, WalOp, MAX_GROUP_COMMIT_BATCH_SIZE};
use super::guards::LeaderCancelGuard;
use crate::error::{StoreError, StoreResult as Result};
use contextra_ids::{TxId, TOMBSTONE_BIT};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

pub(super) struct GroupCommitParams {
    pub(super) tx_id: TxId,
    pub(super) wal_ops: Vec<(WalOp, u64)>,
    pub(super) mem_updates: Vec<(Vec<u8>, Vec<u8>, u64)>,
    pub(super) first_seq: u64,
}

pub(super) async fn follower_commit_path(
    storage: &LsmStorage,
    params: GroupCommitParams,
    mut queue_guard: tokio::sync::MutexGuard<'_, Option<PendingCommitQueue>>,
    commit_lock: tokio::sync::MutexGuard<'_, ()>,
) -> Result<()> {
    let queue = match queue_guard.as_mut() {
        Some(q) => q,
        None => {
            return Err(StoreError::Internal(
                "Group commit queue missing".into(),
            ));
        }
    };
    let tx_id = params.tx_id;
    let first_seq = params.first_seq;
    let _wal_queue_guard = WalQueueGuard::new(Arc::clone(&storage.wal_queue_depth));
    let (tx, mut rx) = tokio::sync::oneshot::channel();
    let committed_flag = Arc::clone(&queue.committed_flag);
    let req = GroupCommitRequest {
        tx_id: params.tx_id,
        wal_ops: params.wal_ops,
        mem_updates: params.mem_updates,
        sender: tx,
    };
    queue.requests.push(req);
    let is_full = queue.requests.len() >= MAX_GROUP_COMMIT_BATCH_SIZE;
    let notify_full = if is_full {
        Some(queue.notify_full.clone())
    } else {
        None
    };
    drop(queue_guard);
    drop(commit_lock);

    if let Some(notify) = notify_full {
        notify.notify_one();
    }

    let res = match tokio::time::timeout(storage.config.tx_timeout, &mut rx).await {
        Ok(Ok(res)) => res,
        Ok(Err(_)) => {
            if committed_flag.load(Ordering::Acquire) {
                Ok(())
            } else {
                if first_seq > 0 {
                    if let Some(ref seq_val) = storage.sequence_log_ssi_validator {
                        seq_val.forget_from(first_seq);
                    }
                }
                storage.cleanup_intent_locks_for_tx(tx_id);
                Err(crate::error::StoreError::Storage(
                    "Commit failed at WAL append after leader cancellation".into(),
                ))
            }
        }
        Err(_) => {
            let mut queue_guard = storage.pending_commit_queue.lock().await;
            let queue_active = queue_guard.is_some();
            let taken_by_leader = queue_guard
                .as_ref()
                .is_some_and(|q| !q.requests.iter().any(|r| r.tx_id == tx_id));

            if queue_active && !taken_by_leader {
                if let Some(ref mut queue) = *queue_guard {
                    queue.requests.retain(|r| r.tx_id != tx_id);
                }
                drop(queue_guard);
                if first_seq > 0 {
                    if let Some(ref seq_val) = storage.sequence_log_ssi_validator {
                        seq_val.forget_from(first_seq);
                    }
                }
                storage.cleanup_intent_locks_for_tx(tx_id);
                Err(crate::error::StoreError::CommitTimeout {
                    tx_id: tx_id.inner(),
                })
            } else {
                drop(queue_guard);
                if !committed_flag.load(Ordering::Acquire) {
                    if first_seq > 0 {
                        if let Some(ref seq_val) = storage.sequence_log_ssi_validator {
                            seq_val.forget_from(first_seq);
                        }
                    }
                    storage.cleanup_intent_locks_for_tx(tx_id);
                    Err(crate::error::StoreError::CommitTimeout {
                        tx_id: tx_id.inner(),
                    })
                } else {
                    match rx.await {
                        Ok(res) => res,
                        Err(_) => Ok(()),
                    }
                }
            }
        }
    };
    storage.cleanup_intent_locks_for_tx(tx_id);
    res
}

pub(super) async fn leader_commit_path(
    storage: &LsmStorage,
    wal: Arc<Wal>,
    params: GroupCommitParams,
    mut queue_guard: tokio::sync::MutexGuard<'_, Option<PendingCommitQueue>>,
    commit_lock: tokio::sync::MutexGuard<'_, ()>,
) -> Result<()> {
    let leader_tx_id = params.tx_id;
    let leader_wal_ops = params.wal_ops;
    let leader_mem_updates = params.mem_updates;
    let first_seq = params.first_seq;

    let notify_full = Arc::new(tokio::sync::Notify::new());
    let committed_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    *queue_guard = Some(PendingCommitQueue {
        requests: Vec::new(),
        notify_full: notify_full.clone(),
        committed_flag: committed_flag.clone(),
    });
    drop(queue_guard);

    let mut leader_guard = LeaderCancelGuard {
        storage,
        wal: &wal,
        leader_tx_id,
        leader_first_seq: first_seq,
        pending_queue: None,
        prev_hmac: None,
        committed: false,
    };

    drop(commit_lock);

    tokio::select! {
        _ = tokio::time::sleep(Duration::from_micros(storage.config.group_commit_window_micros)) => {},
        _ = notify_full.notified() => {},
    }

    let commit_lock = storage.commit_mutex.lock().await;

    let mut queue_guard = storage.pending_commit_queue.lock().await;
    let pending_queue = match queue_guard.take() {
        Some(q) => q,
        None => {
            tracing::error!("Group commit leader: pending_commit_queue unexpectedly missing.");
            return Err(StoreError::Internal(
                "Group commit queue invariant violated".into(),
            ));
        }
    };
    drop(queue_guard);

    leader_guard.pending_queue = Some(pending_queue);

    let mut all_wal_ops = leader_wal_ops;
    if let Some(ref pq) = leader_guard.pending_queue {
        for r in pq.requests.iter() {
            all_wal_ops.extend(r.wal_ops.clone());
        }
    }

    let (all_wal_entries, prev_hmac) = match wal.prepare_batch(all_wal_ops).await {
        Ok(res) => res,
        Err(e) => {
            return Err(e);
        }
    };
    leader_guard.prev_hmac = Some(prev_hmac);

    let leader_entries_count = leader_mem_updates.len() + 1;
    let leader_entries_for_observer = if storage.has_observers() {
        all_wal_entries
            .entries()
            .get(..leader_entries_count)
            .map(|s| s.to_vec())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let durability_mode = storage.config.durability_mode;
    let batch_for_append = all_wal_entries.clone();
    let truncate_guard = wal.truncate_lock.lock().await;
    let append_res = execute_group_commit_append(
        &wal,
        durability_mode,
        batch_for_append,
        truncate_guard,
        &committed_flag,
    )
    .await;

    if durability_mode == DurabilityMode::MemoryOnly {
        let _ = wal.rollback_hmac(prev_hmac).await;
    }

    if let Err(e) = append_res {
        let err_msg = format!(
            "Commit failed (at WAL append), WAL anchor rollback executed: WAL append failed: {e}"
        );
        if let Some(pending_queue) = leader_guard.pending_queue.take() {
            for r in pending_queue.requests {
                let r_first_seq = r
                    .mem_updates
                    .first()
                    .map(|(_, _, seq)| seq & !TOMBSTONE_BIT)
                    .unwrap_or(0);
                if r_first_seq > 0 {
                    if let Some(ref seq_val) = storage.sequence_log_ssi_validator {
                        seq_val.forget_from(r_first_seq);
                    }
                }
                storage.cleanup_intent_locks_for_tx(r.tx_id);
                let _ = r.sender.send(Err(crate::error::StoreError::Storage(err_msg.clone())));
            }
        }
        return Err(crate::error::StoreError::Storage(err_msg));
    }

    leader_guard.committed = true;
    let pending_queue = match leader_guard.pending_queue.take() {
        Some(pq) => pq,
        None => {
            return Err(StoreError::Internal(
                "Group commit pending_queue unexpectedly missing after durability".into(),
            ));
        }
    };

    let is_durable = storage.config.durability_mode != DurabilityMode::MemoryOnly;
    storage.notify_commit_observers(
        &leader_entries_for_observer,
        leader_tx_id,
        WriteOrigin::UserWrite,
        is_durable,
    );

    let mut current_idx = leader_entries_count;
    for r in &pending_queue.requests {
        let follower_entries_count = r.mem_updates.len() + 1;
        if storage.has_observers() {
            if let Some(entries) = all_wal_entries
                .entries()
                .get(current_idx..current_idx + follower_entries_count)
            {
                storage.notify_commit_observers(
                    entries,
                    r.tx_id,
                    WriteOrigin::UserWrite,
                    is_durable,
                );
            }
        }
        current_idx += follower_entries_count;
    }

    type MemUpdateBatch<'a> = (TxId, &'a [(Vec<u8>, Vec<u8>, u64)]);
    let mut all_updates: Vec<MemUpdateBatch> = Vec::with_capacity(1 + pending_queue.requests.len());
    all_updates.push((leader_tx_id, &leader_mem_updates));
    for r in &pending_queue.requests {
        all_updates.push((r.tx_id, &r.mem_updates));
    }

    let state = storage.state.read().await;
    for (req_tx_id, mem_updates) in all_updates {
        storage.apply_mem_updates(&state.memtable, mem_updates, req_tx_id);
        storage.advance_visibility(req_tx_id);
    }

    let needs_flush = state.memtable.size() > storage.config.memtable_size_limit;
    drop(state);
    if needs_flush {
        storage.request_flush();
    }

    let mut batch_txs = vec![leader_tx_id];
    batch_txs.extend(pending_queue.requests.iter().map(|r| r.tx_id));
    storage.cleanup_intent_locks_for_txs(&batch_txs);

    for r in pending_queue.requests {
        if r.sender.send(Ok(())).is_err() {
            tracing::warn!(follower_tx = ?r.tx_id, "Follower dropped receiver before group commit notification");
        }
    }

    drop(commit_lock);

    Ok(())
}
