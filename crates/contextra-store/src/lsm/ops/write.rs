use super::super::config::DurabilityMode;
use super::super::engine::LsmStorage;
use super::super::group_commit::{
    execute_group_commit_append, GroupCommitRequest, PendingCommitQueue, WalQueueGuard,
};
use super::super::observer::WriteOrigin;
use super::super::validate::{derive_doc_id, validate_key, validate_value};
use super::super::{WalOp, MAX_BATCH_SIZE, MAX_GROUP_COMMIT_BATCH_SIZE};
use contextra_core::{ContextraError, IndexOp, Result, StorageEngine, TxId, TOMBSTONE_BIT};
use contextra_mvcc::SsiValidator;
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
    storage.apply_backpressure().await;
    if !storage.budget.has_memory_capacity() {
        return Err(ContextraError::Storage(
            "Memory budget exceeded (95%) — delete staging rejected".into(),
        ));
    }
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
    storage.apply_backpressure().await;
    if !storage.budget.has_memory_capacity() {
        return Err(ContextraError::Storage(
            "Memory budget exceeded (95%) — delete staging rejected".into(),
        ));
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

/// Commits staged transaction operations to disk and memory.
///
/// # Lock Hierarchy Invariant
/// To prevent deadlocks and ensure strict durability & ordering:
/// 1. `commit_mutex` (`tokio::sync::Mutex`): Serializes transaction seq_no allocation and group batch preparation.
/// 2. `pending_commit_queue` (`tokio::sync::Mutex`): Synchronizes follower request queuing for group commit.
/// 3. `truncate_lock` (`tokio::sync::Mutex`): Serializes physical WAL append, flush, and truncation.
/// 4. `state` (`tokio::sync::RwLock`): Read guard protects `MemTable` and `immutable_memtables` vector structure.
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

    // LOCK ORDER 1: commit_mutex
    let _commit_lock = storage.commit_mutex.lock().await;

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
                return Err(e);
            }
        }
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

    for (key, _, seq_no) in &mem_updates {
        let raw_seq = seq_no & !TOMBSTONE_BIT;
        storage.ssi_validator.record_commit_key(key, raw_seq);
    }

    let first_seq = mem_updates
        .first()
        .map(|(_, _, seq)| seq & !TOMBSTONE_BIT)
        .unwrap_or(0);

    let wal = storage.wal.read().await.clone();

    // If group commit window is disabled (0 micros), perform immediate single commit
    if storage.config.group_commit_window_micros == 0 {
        let (wal_entries, _prev_hmac) = wal.prepare_batch(wal_ops).await?;
        let entries_for_observer = if storage.has_observers() {
            wal_entries.entries().to_vec()
        } else {
            Vec::new()
        };
        let durability_mode = storage.config.durability_mode;

        let append_handle = tokio::spawn(async move {
            match durability_mode {
                DurabilityMode::Full | DurabilityMode::WalNoHmac => {
                    let truncate_guard = wal.truncate_lock.lock().await;
                    let start_offset = wal.size();
                    let start_hmac = wal.last_hmac_snapshot().await;
                    let append_res = wal.append_batch_locked(wal_entries, &truncate_guard).await;
                    if append_res.is_err() {
                        drop(truncate_guard);
                        if let Err(trunc_err) = wal.truncate(start_offset, start_hmac).await {
                            tracing::error!("Failed to truncate WAL after failed single append_batch: {trunc_err}");
                        }
                    }
                    append_res
                }
                DurabilityMode::MemoryOnly => Ok(()),
            }
        });

        let single_append_res = match append_handle.await {
            Ok(res) => res,
            Err(join_err) => Err(ContextraError::Internal(format!(
                "WAL append task panicked: {join_err}"
            ))),
        };

        if let Err(e) = single_append_res {
            if first_seq > 0 {
                storage.ssi_validator.forget_from(first_seq);
            }
            storage.cleanup_intent_locks_for_tx(tx_id);
            return Err(ContextraError::Storage(format!(
                "Commit failed (at WAL append), WAL anchor rollback executed: {e}"
            )));
        }

        let is_durable = storage.config.durability_mode != DurabilityMode::MemoryOnly;
        storage.notify_commit_observers(
            &entries_for_observer,
            tx_id,
            WriteOrigin::UserWrite,
            is_durable,
        );

        // LOCK ORDER 4: state (read guard)
        // Lock Strength: Read guard on LsmState is sufficient because MemTable handles its own
        // thread-safe internal locking via parking_lot::RwLock, while LsmState only protects structural swaps.
        let state = storage.state.read().await;
        storage.apply_mem_updates(&state.memtable, &mem_updates, tx_id);
        storage.advance_visibility(tx_id);

        let should_flush = state.memtable.size() > storage.config.memtable_size_limit;
        drop(state);
        if should_flush {
            storage.request_flush();
        }

        storage.cleanup_intent_locks_for_tx(tx_id);
        return Ok(());
    }

    // --- PHASE 2b: Group Commit Coordination ---
    // LOCK ORDER 2: pending_commit_queue
    let mut queue_guard = storage.pending_commit_queue.lock().await;

    if let Some(ref mut queue) = *queue_guard {
        // --- FOLLOWER PATH ---
        let _wal_queue_guard = WalQueueGuard::new(Arc::clone(&storage.wal_queue_depth));
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        let committed_flag = Arc::clone(&queue.committed_flag);
        let req = GroupCommitRequest {
            tx_id,
            wal_ops,
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

        let res = match tokio::time::timeout(storage.config.tx_timeout, &mut rx).await {
            Ok(Ok(res)) => res,
            Ok(Err(_)) => {
                if committed_flag.load(Ordering::Acquire) {
                    Ok(())
                } else {
                    if first_seq > 0 {
                        storage.ssi_validator.forget_from(first_seq);
                    }
                    storage.cleanup_intent_locks_for_tx(tx_id);
                    Err(ContextraError::Storage(
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
                        storage.ssi_validator.forget_from(first_seq);
                    }
                    storage.cleanup_intent_locks_for_tx(tx_id);
                    Err(ContextraError::CommitTimeout {
                        tx_id: tx_id.inner(),
                    })
                } else {
                    drop(queue_guard);
                    if committed_flag.load(Ordering::Acquire) {
                        Ok(())
                    } else {
                        Err(ContextraError::CommitTimeout {
                            tx_id: tx_id.inner(),
                        })
                    }
                }
            }
        };
        storage.cleanup_intent_locks_for_tx(tx_id);
        res
    } else {
        // --- LEADER PATH ---
        let leader_tx_id = tx_id;
        let leader_wal_ops = wal_ops;
        let leader_mem_updates = mem_updates;

        let notify_full = Arc::new(tokio::sync::Notify::new());
        let committed_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
        *queue_guard = Some(PendingCommitQueue {
            requests: Vec::new(),
            notify_full: notify_full.clone(),
            committed_flag: committed_flag.clone(),
        });
        drop(queue_guard);
        drop(_commit_lock);

        struct LeaderCancelGuard<'a> {
            storage: &'a LsmStorage,
            leader_tx_id: TxId,
            leader_first_seq: u64,
            active: bool,
        }

        impl<'a> Drop for LeaderCancelGuard<'a> {
            fn drop(&mut self) {
                if self.active {
                    if let Ok(mut queue_guard) = self.storage.pending_commit_queue.try_lock() {
                        if let Some(pending_queue) = queue_guard.take() {
                            for r in pending_queue.requests {
                                let r_first_seq = r
                                    .mem_updates
                                    .first()
                                    .map(|(_, _, seq)| seq & !TOMBSTONE_BIT)
                                    .unwrap_or(0);
                                if r_first_seq > 0 {
                                    self.storage.ssi_validator.forget_from(r_first_seq);
                                }
                                self.storage.cleanup_intent_locks_for_tx(r.tx_id);
                                let _ = r.sender.send(Err(ContextraError::Storage(
                                    "Leader cancelled group commit".into(),
                                )));
                            }
                        }
                    }
                    if self.leader_first_seq > 0 {
                        self.storage
                            .ssi_validator
                            .forget_from(self.leader_first_seq);
                    }
                    self.storage.cleanup_intent_locks_for_tx(self.leader_tx_id);
                }
            }
        }

        let mut leader_guard = LeaderCancelGuard {
            storage,
            leader_tx_id,
            leader_first_seq: first_seq,
            active: true,
        };

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_micros(storage.config.group_commit_window_micros)) => {},
            _ = notify_full.notified() => {},
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

        leader_guard.active = false;

        let mut all_wal_ops = leader_wal_ops;
        for r in pending_queue.requests.iter() {
            all_wal_ops.extend(r.wal_ops.clone());
        }

        let (all_wal_entries, _prev_hmac) = match wal.prepare_batch(all_wal_ops).await {
            Ok(res) => res,
            Err(e) => {
                if first_seq > 0 {
                    storage.ssi_validator.forget_from(first_seq);
                }
                for r in &pending_queue.requests {
                    let r_first_seq = r
                        .mem_updates
                        .first()
                        .map(|(_, _, seq)| seq & !TOMBSTONE_BIT)
                        .unwrap_or(0);
                    if r_first_seq > 0 {
                        storage.ssi_validator.forget_from(r_first_seq);
                    }
                }
                let mut batch_txs = vec![leader_tx_id];
                batch_txs.extend(pending_queue.requests.iter().map(|r| r.tx_id));
                storage.cleanup_intent_locks_for_txs(&batch_txs);

                let err_msg = format!("Group commit batch preparation failed: {e}");
                for r in pending_queue.requests {
                    let _ = r.sender.send(Err(ContextraError::Storage(err_msg.clone())));
                }
                return Err(e);
            }
        };

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

        drop(_commit_lock);

        let wal_clone = wal.clone();
        let durability_mode = storage.config.durability_mode;
        let batch_for_append = all_wal_entries.clone();
        let append_handle = tokio::spawn(async move {
            let truncate_guard = wal_clone.truncate_lock.lock().await;
            execute_group_commit_append(
                &wal_clone,
                durability_mode,
                batch_for_append,
                truncate_guard,
            )
            .await
        });

        let append_res = match append_handle.await {
            Ok(res) => res,
            Err(join_err) => Err(ContextraError::Internal(format!(
                "Group commit append task panicked: {join_err}"
            ))),
        };

        if append_res.is_ok() {
            committed_flag.store(true, Ordering::Release);
        }

        if let Err(e) = append_res {
            if first_seq > 0 {
                storage.ssi_validator.forget_from(first_seq);
            }
            for r in &pending_queue.requests {
                let r_first_seq = r
                    .mem_updates
                    .first()
                    .map(|(_, _, seq)| seq & !TOMBSTONE_BIT)
                    .unwrap_or(0);
                if r_first_seq > 0 {
                    storage.ssi_validator.forget_from(r_first_seq);
                }
            }

            let err_msg = format!("Commit failed (at WAL append), WAL anchor rollback executed: WAL append failed: {e}");

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
        let mut all_updates: Vec<MemUpdateBatch> =
            Vec::with_capacity(1 + pending_queue.requests.len());
        all_updates.push((leader_tx_id, &leader_mem_updates));
        for r in &pending_queue.requests {
            all_updates.push((r.tx_id, &r.mem_updates));
        }

        let _commit_lock = storage.commit_mutex.lock().await;
        // LOCK ORDER 4: state (read guard)
        // Lock Strength: Read guard on LsmState is sufficient because MemTable handles its own
        // thread-safe internal locking via parking_lot::RwLock, while LsmState only protects structural swaps.
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

        Ok(())
    }
}
