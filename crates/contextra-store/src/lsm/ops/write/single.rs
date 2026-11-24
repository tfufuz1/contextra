//! # Single-Commit Pfad (Ring 1)
//!
//! // FILE-CONTEXT
//! // STAND: 2026-11-11
//! // ZWECK: Direkte, serielle Commit-Ausfuehrung ohne Group-Commit-Batching
//! // INVARIANTEN: ADR-0XX-P0A, INV-STORE-1, INV-LOCK-ORDER-1

use super::super::super::config::DurabilityMode;
use super::super::super::engine::LsmStorage;
use super::super::super::group_commit::execute_group_commit_append;
use super::super::super::observer::WriteOrigin;
use super::super::super::{Wal, WalOp};
use super::guards::SingleCommitGuard;
use crate::error::StoreResult as Result;
use contextra_ids::TxId;
use std::sync::Arc;

pub(super) async fn single_commit_path(
    storage: &LsmStorage,
    tx_id: TxId,
    wal: Arc<Wal>,
    wal_ops: Vec<(WalOp, u64)>,
    mem_updates: Vec<(Vec<u8>, Vec<u8>, u64)>,
    first_seq: u64,
) -> Result<()> {
    let (wal_entries, prev_hmac) = wal.prepare_batch(wal_ops).await?;
    let entries_for_observer = if storage.has_observers() {
        wal_entries.entries().to_vec()
    } else {
        Vec::new()
    };
    let durability_mode = storage.config.durability_mode;

    let mut guard = SingleCommitGuard {
        storage,
        wal: &wal,
        tx_id,
        first_seq,
        prev_hmac,
        committed: false,
    };

    let single_append_res = match durability_mode {
        DurabilityMode::Full | DurabilityMode::WalNoHmac => {
            let truncate_guard = wal.truncate_lock.lock().await;
            let start_offset = wal.size();
            let append_res = wal.append_batch_locked(wal_entries, &truncate_guard).await;
            drop(truncate_guard);
            if append_res.is_err() {
                if let Err(trunc_err) = wal.truncate(start_offset, prev_hmac).await {
                    tracing::error!(
                        "Failed to truncate WAL after failed single append_batch: {trunc_err}"
                    );
                }
                if wal.is_poisoned() {
                    match wal.recover_from_poison().await {
                        Ok(()) => {
                            tracing::info!("WAL successfully recovered from poison state")
                        }
                        Err(rec_err) => {
                            tracing::error!("Failed to recover WAL from poison state: {rec_err}")
                        }
                    }
                }
            }
            append_res
        }
        DurabilityMode::MemoryOnly => {
            let _ = wal.rollback_hmac(prev_hmac).await;
            Ok(())
        }
    };

    if let Err(e) = single_append_res {
        return Err(crate::error::StoreError::Storage(format!(
            "Commit failed (at WAL append), WAL anchor rollback executed: {e}"
        )));
    }

    guard.committed = true;

    let is_durable = storage.config.durability_mode != DurabilityMode::MemoryOnly;
    storage.notify_commit_observers(
        &entries_for_observer,
        tx_id,
        WriteOrigin::UserWrite,
        is_durable,
    );

    let state = storage.state.read().await;
    storage.apply_mem_updates(&state.memtable, &mem_updates, tx_id);
    storage.advance_visibility(tx_id);

    let should_flush = state.memtable.size() > storage.config.memtable_size_limit;
    drop(state);
    if should_flush {
        storage.request_flush();
    }

    storage.cleanup_intent_locks_for_tx(tx_id);
    Ok(())
}
