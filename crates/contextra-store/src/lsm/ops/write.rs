use super::super::engine::LsmStorage;
use super::super::group_commit::{GroupCommitRequest, PendingCommitQueue, WalQueueGuard};
use super::super::guard::CommitGuard;
use super::super::validate::{derive_doc_id, validate_key, validate_value};
use super::super::{WalOp, MAX_BATCH_SIZE, MAX_GROUP_COMMIT_BATCH_SIZE};
use contextra_core::{ContextraError, IndexOp, Result, StorageEngine, TxId, TOMBSTONE_BIT};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

pub(super) async fn put(storage: &LsmStorage, tx_id: TxId, key: &[u8], value: &[u8]) -> Result<()> {
    validate_key(key)?;
    validate_value(value)?;
    storage.apply_backpressure().await;
    if !storage.budget.has_memory_capacity() {
        return Err(ContextraError::Storage(
            "Memory budget exceeded (95%)".into(),
        ));
    }
    let doc_id = derive_doc_id(key);

    storage.tx_buffer.stage_kv(
        tx_id,
        IndexOp::Insert {
            doc_id,
            data: (key.to_vec(), value.to_vec()),
        },
    )?;
    Ok(())
}

pub(super) async fn put_if_absent(
    storage: &LsmStorage,
    tx_id: TxId,
    key: &[u8],
    value: &[u8],
) -> Result<bool> {
    validate_key(key)?;
    validate_value(value)?;
    storage.apply_backpressure().await;
    if !storage.budget.has_memory_capacity() {
        return Err(ContextraError::Storage(
            "Memory budget exceeded (95%)".into(),
        ));
    }

    // 1. Staged write check across all active transactions
    if let Some(is_insert) = storage.tx_buffer.staged_status(key) {
        if is_insert {
            return Ok(false);
        }
    }
    if storage.tx_buffer.staged_status(key) == Some(true)
        || matches!(storage.tx_buffer.staged_status(key), Some(true))
    {
        return Ok(false);
    }

    // 2. Intent Lock check and registration
    {
        let mut locks = storage
            .intent_locks
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(&existing_tx) = locks.get(key) {
            if existing_tx != tx_id {
                return Ok(false);
            }
        } else {
            locks.insert(key.to_vec(), tx_id);
        }
    }

    // 3. Query committed state
    let current_max_seq = storage.next_seq_no.load(Ordering::Acquire);
    let is_present = match storage.get_at_seq(key, current_max_seq).await {
        Ok(opt) => opt.is_some(),
        Err(e) => {
            let mut locks = storage
                .intent_locks
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if locks.get(key) == Some(&tx_id) {
                locks.remove(key);
            }
            return Err(e);
        }
    };

    if is_present {
        let mut locks = storage
            .intent_locks
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if locks.get(key) == Some(&tx_id) {
            locks.remove(key);
        }
        return Ok(false);
    }

    // 4. Stage operation in tx_buffer
    let doc_id = derive_doc_id(key);

    if let Err(e) = storage.tx_buffer.stage_kv(
        tx_id,
        IndexOp::Insert {
            doc_id,
            data: (key.to_vec(), value.to_vec()),
        },
    ) {
        let mut locks = storage
            .intent_locks
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if locks.get(key) == Some(&tx_id) {
            locks.remove(key);
        }
        return Err(e);
    }

    Ok(true)
}

pub(super) async fn delete(storage: &LsmStorage, tx_id: TxId, key: &[u8]) -> Result<()> {
    validate_key(key)?;
    let doc_id = derive_doc_id(key);

    storage.tx_buffer.stage_kv(
        tx_id,
        IndexOp::Delete {
            doc_id,
            data: Some((key.to_vec(), Vec::new())),
        },
    )?;
    Ok(())
}

pub(super) async fn delete_many(
    storage: &LsmStorage,
    tx_id: TxId,
    keys: Vec<Vec<u8>>,
) -> Result<u64> {
    if keys.len() > MAX_BATCH_SIZE {
        return Err(ContextraError::InvalidInput(format!(
            "Batch size ({} items) exceeds limit of {} items",
            keys.len(),
            MAX_BATCH_SIZE
        )));
    }
    for key in &keys {
        validate_key(key)?;
    }
    let count = keys.len() as u64;
    if count == 0 {
        return Ok(0);
    }

    let ops: Vec<IndexOp<(Vec<u8>, Vec<u8>)>> = keys
        .into_iter()
        .map(|key| {
            let doc_id = derive_doc_id(&key);
            IndexOp::Delete {
                doc_id,
                data: Some((key, Vec::new())),
            }
        })
        .collect();

    storage.tx_buffer.stage_many(tx_id, ops)?;
    Ok(count)
}

pub(super) async fn delete_prefix(storage: &LsmStorage, tx_id: TxId, prefix: &[u8]) -> Result<u64> {
    let matching_keys: Vec<Vec<u8>> = storage
        .scan_prefix(prefix)
        .await?
        .into_iter()
        .map(|(key, _)| key)
        .collect();
    storage.delete_many(tx_id, matching_keys).await
}

pub(super) async fn commit(storage: &LsmStorage, tx_id: TxId) -> Result<()> {
    storage.apply_backpressure().await;
    if !storage.budget.has_memory_capacity() {
        return Err(ContextraError::Storage(
            "Memory budget exceeded (95%)".into(),
        ));
    }

    struct IntentLockGuard<'a>(&'a LsmStorage, TxId);
    impl<'a> Drop for IntentLockGuard<'a> {
        fn drop(&mut self) {
            self.0.clear_intent_locks_for_tx(self.1);
        }
    }
    let _intent_guard = IntentLockGuard(storage, tx_id);

    // ANCHOR[ALG-FIX:D6-001] STATUS:DONE (TS:2026-06-01T00:00:00Z) — Snapshot-Inversion bei parallel commit (INV-MVCC-1)
    // FIX: Commit-Mutex serialisiert fetch_add + wal.prepare_batch.
    let _commit_lock = storage.commit_mutex.lock().await;

    let ops = storage.tx_buffer.drain_kv(tx_id);
    if ops.is_empty() {
        storage.cleanup_intent_locks_for_tx(tx_id);
        return Ok(());
    }

    let mut wal_ops = Vec::with_capacity(ops.len() + 1);
    let mut mem_updates = Vec::with_capacity(ops.len());
    let mut last_seq = 0u64;

    for op in &ops {
        let seq_no = storage.next_seq_no.fetch_add(1, Ordering::SeqCst);
        last_seq = seq_no;
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
                return Err(ContextraError::InvalidInput(
                    "Unsupported operation type staged in LSM commit".to_string(),
                ));
            }
        }
    }

    wal_ops.push((
        WalOp::TxEnd {
            tx_id,
            committed: true,
        },
        last_seq,
    ));

    // --- PHASE 2: Prepare WAL entries under commit_mutex ---
    let wal = storage.wal.read().await.clone();
    let (wal_entries, prev_hmac_snapshot) = wal.prepare_batch(wal_ops).await?;

    // If group commit window is disabled (0 micros), perform immediate single commit
    if storage.config.group_commit_window_micros == 0 {
        if let Err(e) = wal.append_batch(wal_entries).await {
            let _ = wal.restore_last_hmac(prev_hmac_snapshot).await;
            let last_tx = TxId::new(storage.last_committed_tx.load(Ordering::Acquire));
            let commit_guard = CommitGuard {
                _lock: &_commit_lock,
            };
            if let Err(rollback_err) = storage.rollback_to_tx_locked(last_tx, &commit_guard).await {
                tracing::error!(
                    "Failed to execute rollback_to_tx_locked after failed WAL append: {}",
                    rollback_err
                );
            }
            storage.cleanup_intent_locks_for_tx(tx_id);
            return Err(ContextraError::Storage(format!(
                "Commit failed (at WAL append), WAL rollback executed: {}",
                e
            )));
        }

        let state = storage.state.write().await;
        storage.advance_visibility(tx_id);
        storage.apply_mem_updates(&state.memtable, &mem_updates, tx_id);

        let should_flush = state.memtable.size() > storage.config.memtable_size_limit;
        drop(state);
        if should_flush {
            storage.flush().await?;
        }

        storage.cleanup_intent_locks_for_tx(tx_id);
        return Ok(());
    }

    // --- PHASE 2b: Group Commit Coordination ---
    let mut queue_guard = storage.pending_commit_queue.lock().await;

    if let Some(ref mut queue) = *queue_guard {
        let _wal_queue_guard = WalQueueGuard::new(Arc::clone(&storage.wal_queue_depth));
        let (tx, rx) = tokio::sync::oneshot::channel();
        let req = GroupCommitRequest {
            tx_id,
            wal_entries,
            mem_updates,
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
        drop(_commit_lock);

        if let Some(notify) = notify_full {
            notify.notify_one();
        }

        let res = match tokio::time::timeout(storage.config.tx_timeout, rx).await {
            Ok(Ok(res)) => res,
            Ok(Err(_)) => Err(ContextraError::Internal(
                "Group commit leader dropped without sending result".to_string(),
            )),
            Err(_) => {
                let mut queue_guard = storage.pending_commit_queue.lock().await;
                *queue_guard = None;
                drop(queue_guard);
                Err(ContextraError::CommitTimeout {
                    tx_id: tx_id.inner(),
                })
            }
        };
        storage.cleanup_intent_locks_for_tx(tx_id);
        res
    } else {
        let leader_tx_id = tx_id;
        let leader_wal_entries = wal_entries;
        let leader_mem_updates = mem_updates;

        let notify_full = Arc::new(tokio::sync::Notify::new());
        *queue_guard = Some(PendingCommitQueue {
            requests: Vec::new(),
            first_prev_hmac: prev_hmac_snapshot,
            notify_full: notify_full.clone(),
        });
        drop(queue_guard);
        drop(_commit_lock);

        tokio::task::yield_now().await;
        let has_followers = {
            let q = storage.pending_commit_queue.lock().await;
            q.as_ref().is_some_and(|q| !q.requests.is_empty())
        };

        if has_followers {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_micros(storage.config.group_commit_window_micros)) => {},
                _ = notify_full.notified() => {},
            }
        }

        let _commit_lock = storage.commit_mutex.lock().await;
        let mut queue_guard = storage.pending_commit_queue.lock().await;
        let pending_queue = match queue_guard.take() {
            Some(q) => q,
            None => {
                tracing::error!("Group commit leader: pending_commit_queue unexpectedly missing.");
                return Err(ContextraError::Internal(
                    "Group commit queue invariant violated".into(),
                ));
            }
        };
        drop(queue_guard);

        let wal = storage.wal.read().await.clone();
        let mut all_wal_entries = leader_wal_entries;
        for r in pending_queue.requests.iter() {
            all_wal_entries.extend(r.wal_entries.clone());
        }

        let truncate_guard = wal.truncate_lock.lock().await;
        drop(_commit_lock);

        let append_res = wal
            .append_batch_locked(all_wal_entries, &truncate_guard)
            .await;
        drop(truncate_guard);

        if let Err(e) = append_res {
            let _commit_lock = storage.commit_mutex.lock().await;
            let _ = wal.restore_last_hmac(pending_queue.first_prev_hmac).await;
            let last_tx = TxId::new(storage.last_committed_tx.load(Ordering::Acquire));
            let commit_guard = CommitGuard {
                _lock: &_commit_lock,
            };
            let rollback_res = storage.rollback_to_tx_locked(last_tx, &commit_guard).await;

            let err_msg = if let Err(ref rollback_err) = rollback_res {
                tracing::error!(
                    "Failed to execute rollback_to_tx_locked after failed group WAL append: {}",
                    rollback_err
                );
                format!("Fatal double-fault: WAL append failed ({e}) and subsequent rollback failed: {rollback_err}")
            } else {
                format!("Commit failed (at WAL append), WAL rollback executed: {e}")
            };

            let mut batch_txs = vec![leader_tx_id];
            batch_txs.extend(pending_queue.requests.iter().map(|r| r.tx_id));
            storage.cleanup_intent_locks_for_txs(&batch_txs);

            for r in pending_queue.requests {
                if r.sender
                    .send(Err(ContextraError::Storage(err_msg.clone())))
                    .is_err()
                {
                    tracing::warn!(follower_tx = ?r.tx_id, "Follower dropped receiver during group commit failure notification");
                }
            }
            return Err(ContextraError::Storage(err_msg));
        }

        type MemUpdateBatch<'a> = (TxId, &'a [(Vec<u8>, Vec<u8>, u64)]);
        let mut all_updates: Vec<MemUpdateBatch> =
            Vec::with_capacity(1 + pending_queue.requests.len());
        all_updates.push((leader_tx_id, &leader_mem_updates));
        for r in &pending_queue.requests {
            all_updates.push((r.tx_id, &r.mem_updates));
        }

        let _commit_lock = storage.commit_mutex.lock().await;
        let state = storage.state.read().await;
        for (req_tx_id, mem_updates) in all_updates {
            storage.advance_visibility(req_tx_id);
            storage.apply_mem_updates(&state.memtable, mem_updates, req_tx_id);
        }

        let needs_flush = state.memtable.size() > storage.config.memtable_size_limit;
        drop(state);
        if needs_flush {
            if let Err(flush_err) = storage.flush().await {
                tracing::error!("Flush failed after group commit: {}", flush_err);
            }
        }

        let mut batch_txs = vec![leader_tx_id];
        batch_txs.extend(pending_queue.requests.iter().map(|r| r.tx_id));
        storage.cleanup_intent_locks_for_txs(&batch_txs);

        for r in pending_queue.requests {
            if r.sender.send(Ok(())).is_err() {
                tracing::warn!(follower_tx = ?r.tx_id, "Follower dropped receiver before group commit notification");
            }
        }

        Ok(())
    }
}
