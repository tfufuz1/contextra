//! # Commit-Pipeline & Transaktionsabschluss (Ring 1)
//!
//! // FILE-CONTEXT
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
use contextra_mvcc::IndexOp;
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

    // 1. ReadSet VOR drain_kv
    let read_set = storage.tx_buffer.get_read_set(tx_id);

    // 2. Transaktions-Operationen aus TxBuffer entnehmen
    let ops = storage.tx_buffer.drain_kv(tx_id);
    if ops.is_empty() {
        storage.cleanup_intent_locks_for_tx(tx_id);
        return Ok(());
    }

    // 3. SSI ReadSet Validierung unter commit_mutex
    if let Some(ref rs) = read_set {
        if !rs.is_empty() {
            if let Err(e) = storage.ssi_validator.validate(tx_id, rs) {
                storage.cleanup_intent_locks_for_tx(tx_id);
                return Err(e.into());
            }
        }
    }

    let (wal_ops, mem_updates, first_seq) = prepare_commit_ops(storage, tx_id, &ops)?;

    let wal = storage.wal.read().await.clone();

    if storage.config.group_commit_window_micros == 0 {
        return single_commit_path(storage, tx_id, wal, wal_ops, mem_updates, first_seq).await;
    }

    let queue_guard = storage.pending_commit_queue.lock().await;

    let params = GroupCommitParams {
        tx_id,
        wal_ops,
        mem_updates,
        first_seq,
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
    ops: &[IndexOp<(Vec<u8>, Vec<u8>)>],
) -> Result<(Vec<(WalOp, u64)>, Vec<(Vec<u8>, Vec<u8>, u64)>, u64)> {
    let mut wal_ops = Vec::with_capacity(ops.len() + 1);
    let mut mem_updates = Vec::with_capacity(ops.len());

    for op in ops {
        let seq_no = storage.next_seq_no.fetch_add(1, Ordering::SeqCst);
        match op {
            IndexOp::Insert { doc_id: _, data } => {
                let (key, value) = data;
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
            IndexOp::Delete { doc_id: _, data } => {
                if let Some((key, _)) = data {
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
            _ => {
                storage.cleanup_intent_locks_for_tx(tx_id);
                return Err(StoreError::InvalidInput(
                    "Unsupported operation type staged in LSM commit".to_string(),
                ));
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

    for (key, _, seq_no) in &mem_updates {
        let raw_seq = seq_no & !TOMBSTONE_BIT;
        if let Some(ref seq_val) = storage.sequence_log_ssi_validator {
            seq_val.record_commit_key(key, raw_seq);
        }
    }

    let first_seq = mem_updates
        .first()
        .map(|(_, _, seq)| seq & !TOMBSTONE_BIT)
        .unwrap_or(0);

    Ok((wal_ops, mem_updates, first_seq))
}
