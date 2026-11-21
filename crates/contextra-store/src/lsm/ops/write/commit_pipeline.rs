//! # Commit-Pipeline & Transaktionsabschluss (Ring 1)
//!
//! // STAND: 2026-11-11
//! // ZWECK: Orchestrierung des Commit-Ablaufs: Vorbereitung, Validierung und Pfadentscheidung
//! // INVARIANTEN: ADR-0XX-P0A (Lock-Handoff), INV-LOCK-ORDER-1, INV-STORE-1
//! // HOTSPOTS: commit_mutex, wal.truncate_lock, state.read()

use super::super::super::engine::LsmStorage;
use super::super::super::{WalOp};
use super::guards::IntentLockGuard;
use super::group::{follower_commit_path, leader_commit_path, GroupCommitParams};
use super::single::single_commit_path;
use crate::error::{StoreError, StoreResult as Result};
use contextra_ids::{TxId, TOMBSTONE_BIT};
use crate::staging::StagedOp;
use std::sync::atomic::Ordering;

/// Commits staged transaction operations to disk and memory.
pub(crate) async fn commit(storage: &LsmStorage, tx_id: TxId) -> Result<()> {
    let clock = storage.clock();
    let start_nanos = clock.monotonic_nanos();
    let res = commit_internal(storage, tx_id).await;
    let end_nanos = clock.monotonic_nanos();
    let elapsed_secs = (end_nanos.saturating_sub(start_nanos)) as f64 / 1_000_000_000.0;
    let status_label = if res.is_ok() { "success" } else { "failure" };

    let sink = storage.metrics_sink();
    sink.record_histogram(
        "lsm_commit_duration_seconds",
        elapsed_secs,
        &[("status", status_label)],
    );
    sink.record_counter("lsm_commit_total", 1, &[("status", status_label)]);

    res
}

async fn commit_internal(storage: &LsmStorage, tx_id: TxId) -> Result<()> {
    storage.apply_backpressure().await;
    if !storage.budget.has_memory_capacity() {
        return Err(crate::error::StoreError::Storage(
            "Memory budget exceeded (95%)".into(),
        ));
    }

    let _intent_guard = IntentLockGuard(storage, tx_id);

    // LOCK ORDER (SPEC §3.5, INV-LOCK-ORDER-1, see lsm::lock_order::RANK_COMMIT_MUTEX):
    let commit_lock = storage.commit_mutex.lock().await;

    // 2. Transaktions-Operationen aus TxBuffer entnehmen
    let ops = storage.tx_staging.drain(tx_id);
    if ops.is_empty() {
        storage.cleanup_intent_locks_for_tx(tx_id);
        return Ok(());
    }

    let (wal_ops, mem_updates) = prepare_commit_ops(storage, tx_id, &ops)?;

    let wal = storage.wal.read().await.clone();

    if storage.config.group_commit_window_micros == 0 {
        return single_commit_path(storage, tx_id, wal, wal_ops, mem_updates).await;
    }

    let queue_guard = storage.pending_commit_queue.lock().await;

    let params = GroupCommitParams {
        tx_id,
        wal_ops,
        mem_updates,
    };

    if queue_guard.is_some() {
        follower_commit_path(storage, params, queue_guard, commit_lock).await
    } else {
        leader_commit_path(storage, wal, params, queue_guard, commit_lock).await
    }
}

fn prepare_commit_ops(
    storage: &LsmStorage,
    tx_id: TxId,
    ops: &[StagedOp],
) -> Result<(Vec<(WalOp, u64)>, Vec<(Vec<u8>, Vec<u8>, u64)>)> {
    let mut wal_ops = Vec::with_capacity(ops.len() + 1);
    let mut mem_updates = Vec::with_capacity(ops.len());

    for op in ops {
        let seq_no = storage.next_seq_no.fetch_add(1, Ordering::SeqCst);
        match op {
            StagedOp::Put { key, value } => {
                wal_ops.push((
                    WalOp::Put {
                        tx_id,
                        key: key.clone(),
                        value: value.clone(),
                    },
                    seq_no,
                ));
                mem_updates.push((key.clone(), value.clone(), seq_no));
            }
            StagedOp::Delete { key } => {
                wal_ops.push((
                    WalOp::Delete {
                        tx_id,
                        key: key.clone(),
                    },
                    seq_no,
                ));
                mem_updates.push((key.clone(), Vec::new(), seq_no | TOMBSTONE_BIT));
            }
        }
    }

    let tx_end_seq_no = storage.next_seq_no.fetch_add(1, Ordering::SeqCst);
    wal_ops.push((
        WalOp::TxEnd {
            tx_id,
            committed: true,
        },
        tx_end_seq_no,
    ));



    Ok((wal_ops, mem_updates))
}
