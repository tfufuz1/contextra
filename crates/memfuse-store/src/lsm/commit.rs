use super::*;
use memfuse_core::TxId;

impl LsmStorage {
    pub fn clear_intent_locks_for_tx(&self, tx_id: TxId) {
        if let Ok(mut locks) = self.intent_locks.lock() {
            locks.retain(|_, locked_tx| *locked_tx != tx_id);
        }
    }

    /// Removes all intent lock registrations associated with transactions strictly newer than target_tx.
    pub fn clear_intent_locks_above_tx(&self, target_tx: TxId) {
        if let Ok(mut locks) = self.intent_locks.lock() {
            locks.retain(|_, locked_tx| *locked_tx <= target_tx);
        }
    }

    /// Forces a flush (to be used by PersistentCheckpointStore or tests).

    pub(super) fn cleanup_intent_locks_for_tx(&self, tx_id: TxId) {
        let mut locks = self.intent_locks.lock().unwrap_or_else(|e| e.into_inner());
        locks.retain(|_, v| *v != tx_id);
    }

    pub(super) fn cleanup_intent_locks_for_txs(&self, tx_ids: &[TxId]) {
        let mut locks = self.intent_locks.lock().unwrap_or_else(|e| e.into_inner());
        locks.retain(|_, v| !tx_ids.contains(v));
    }

    /// Suspends execution briefly if memory usage exceeds 80% to apply backpressure.
    pub(super) async fn apply_backpressure(&self) {
        if self.budget.memory_used()
            >= (self.config.max_ram_mb as f64 * 1024.0 * 1024.0 * 0.80) as u64
        {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    }

    /// Advances the MVCC visible transaction horizon (`last_committed_tx`) atomically.
    /// Internal transactions (`>= TxId::INTERNAL_BASE`) are ignored as they are never MVCC-visible.
    /// `tx_id == 0` is ignored with a warning to prevent MVCC blackout.
    #[inline]
    pub(super) fn advance_visibility(&self, tx_id: TxId) {
        if tx_id.inner() < TxId::INTERNAL_BASE {
            let mut current = self.last_committed_tx.load(Ordering::Acquire);
            while tx_id.inner() > current {
                match self.last_committed_tx.compare_exchange_weak(
                    current,
                    tx_id.inner(),
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(actual) => current = actual,
                }
            }
            if tx_id.inner() == 0 {
                tracing::warn!(
                    "LsmStorage::commit tx=0 called — ignoring visibility update to prevent blackout"
                );
            }
        }
    }

    /// Applies memory updates to the provided `MemTable` and updates budget tracking.
    ///
    /// # Lock Invariants
    /// The caller must hold appropriate lock access (read or write guard on `LsmState`)
    /// guarding the `MemTable`. `MemTable` internally uses `parking_lot::RwLock` for safe
    /// concurrent mutations.
    pub(super) fn apply_mem_updates(
        &self,
        memtable: &MemTable,
        mem_updates: &[(Vec<u8>, Vec<u8>, u64)],
        tx_id: TxId,
    ) {
        for (key, value, seq) in mem_updates {
            let entry_size = key.len() + value.len() + 8;
            if let Err(e) = self.budget.consume_memory(entry_size as u64) {
                self.budget_tracking_drift_bytes
                    .fetch_add(entry_size as u64, std::sync::atomic::Ordering::Relaxed);
                tracing::warn!(
                    drift_bytes = entry_size,
                    total_drift_bytes = self
                        .budget_tracking_drift_bytes
                        .load(std::sync::atomic::Ordering::Relaxed),
                    "Memory budget tracking warning during commit: {e}"
                );
            }
            memtable.put(
                Bytes::from(key.clone()),
                Bytes::from(value.clone()),
                *seq,
                tx_id.inner(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn test_storage() -> (LsmStorage, TempDir) {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 1024 * 1024,
            max_ram_mb: 64,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            ..Default::default()
        };
        let storage = LsmStorage::new(config).await.expect("create storage");
        (storage, tmp)
    }

    #[tokio::test]
    async fn test_put_get_roundtrip() {
        let (storage, _tmp) = test_storage().await;
        let tx = TxId::new(1);

        storage.put(tx, b"hello", b"world").await.expect("put");
        storage.commit(tx).await.expect("commit");

        let val = storage.get(b"hello").await.expect("get");
        assert_eq!(val, Some(b"world".to_vec()));
    }

    #[tokio::test]
    async fn test_delete() {
        let (storage, _tmp) = test_storage().await;
        let tx1 = TxId::new(1);

        storage.put(tx1, b"key", b"val").await.expect("put");
        storage.commit(tx1).await.expect("commit");

        let tx2 = TxId::new(2);
        storage.delete(tx2, b"key").await.expect("delete");
        storage.commit(tx2).await.expect("commit");

        let val = storage.get(b"key").await.expect("get");
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn test_delete_prefix_removes_all_matching_keys() {
        let (storage, _tmp) = test_storage().await;
        let tx1 = TxId::new(1);

        storage.put(tx1, b"test:1", b"val1").await.unwrap();
        storage.put(tx1, b"test:2", b"val2").await.unwrap();
        storage.put(tx1, b"test:3", b"val3").await.unwrap();
        storage.put(tx1, b"other:1", b"val4").await.unwrap();
        storage.commit(tx1).await.unwrap();

        let tx2 = TxId::new(2);
        let deleted = storage.delete_prefix(tx2, b"test:").await.unwrap();
        assert_eq!(deleted, 3);
        storage.commit(tx2).await.unwrap();

        assert_eq!(storage.get(b"test:1").await.unwrap(), None);
        assert_eq!(storage.get(b"test:2").await.unwrap(), None);
        assert_eq!(storage.get(b"test:3").await.unwrap(), None);
        assert_eq!(
            storage.get(b"other:1").await.unwrap(),
            Some(b"val4".to_vec())
        );
    }

    #[tokio::test]
    async fn test_lsm_storage_delete_many_uses_single_batch() {
        let (storage, _tmp) = test_storage().await;
        let tx1 = TxId::new(1);

        let keys_to_delete: Vec<Vec<u8>> = (0..50)
            .map(|i| format!("batch_key_{i}").into_bytes())
            .collect();

        for key in &keys_to_delete {
            storage.put(tx1, key, b"value").await.unwrap();
        }
        storage.commit(tx1).await.unwrap();

        let tx2 = TxId::new(2);
        let count = storage
            .delete_many(tx2, keys_to_delete.clone())
            .await
            .unwrap();
        assert_eq!(count, 50);

        let staged_ops = storage.tx_buffer.get_ops(tx2).expect("ops staged");
        assert_eq!(staged_ops.len(), 50);

        storage.commit(tx2).await.unwrap();
        for key in &keys_to_delete {
            assert_eq!(storage.get(key).await.unwrap(), None);
        }
    }

    #[tokio::test]
    async fn test_rollback() {
        let (storage, _tmp) = test_storage().await;
        let tx = TxId::new(1);

        storage.put(tx, b"key", b"val").await.expect("put");
        storage.rollback(tx).await.expect("rollback");

        let val = storage.get(b"key").await.expect("get");
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn test_overwrite() {
        let (storage, _tmp) = test_storage().await;

        let tx1 = TxId::new(1);
        storage.put(tx1, b"key", b"val1").await.expect("put1");
        storage.commit(tx1).await.expect("commit1");

        let tx2 = TxId::new(2);
        storage.put(tx2, b"key", b"val2").await.expect("put2");
        storage.commit(tx2).await.expect("commit2");

        let val = storage.get(b"key").await.expect("get");
        assert_eq!(val, Some(b"val2".to_vec()));
    }

    #[tokio::test]
    async fn test_input_boundary_guards() {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            ..Default::default()
        };
        let storage = LsmStorage::new(config).await.expect("create storage");
        let tx = TxId::new(1);

        assert!(matches!(
            storage.put(tx, b"", b"val").await,
            Err(MemFuseError::InvalidInput(_))
        ));
        assert!(matches!(
            storage.delete(tx, b"").await,
            Err(MemFuseError::InvalidInput(_))
        ));
        assert!(matches!(
            storage.get(b"").await,
            Err(MemFuseError::InvalidInput(_))
        ));
        assert!(matches!(
            storage.get_at_seq(b"", 10).await,
            Err(MemFuseError::InvalidInput(_))
        ));

        let huge_key = vec![b'a'; MAX_KEY_SIZE + 1];
        assert!(matches!(
            storage.put(tx, &huge_key, b"val").await,
            Err(MemFuseError::InvalidInput(_))
        ));
        assert!(matches!(
            storage.delete(tx, &huge_key).await,
            Err(MemFuseError::InvalidInput(_))
        ));
        assert!(matches!(
            storage.get(&huge_key).await,
            Err(MemFuseError::InvalidInput(_))
        ));

        let too_many_keys = vec![b"key".to_vec(); MAX_BATCH_SIZE + 1];
        assert!(matches!(
            storage.delete_many(tx, too_many_keys).await,
            Err(MemFuseError::InvalidInput(_))
        ));

        let huge_val = vec![b'v'; MAX_VALUE_SIZE + 1];
        assert!(matches!(
            storage.put(tx, b"valid_key", &huge_val).await,
            Err(MemFuseError::InvalidInput(_))
        ));
    }

    #[tokio::test]
    async fn test_lsm_commit_append_failure_restores_hmac() {
        let (storage, _tmp) = test_storage().await;

        let tx1 = TxId::new(1);
        storage.put(tx1, b"k1", b"v1").await.unwrap();
        storage.commit(tx1).await.unwrap();

        let wal = storage.wal.read().await;
        let hmac_before = wal.last_hmac_snapshot().await;

        drop(wal);
        storage.simulate_wal_append_failure_for_test().await;

        let tx2 = TxId::new(2);
        storage.put(tx2, b"k2", b"v2").await.unwrap();
        let commit_res = storage.commit(tx2).await;
        assert!(commit_res.is_err(), "Commit must fail when WAL write fails");

        let wal = storage.wal.read().await;
        let hmac_after = wal.last_hmac_snapshot().await;
        assert_eq!(
            hmac_after, hmac_before,
            "last_hmac must be restored to pre-commit state after commit failure"
        );
    }

    #[tokio::test]
    async fn test_commit_tracks_budget_drift_on_consume_memory_failure() {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 1024 * 1024,
            max_ram_mb: 1,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            ..Default::default()
        };
        let storage = LsmStorage::new(config).await.expect("create storage");

        assert_eq!(storage.budget_tracking_drift_bytes(), 0);

        let key = b"drift_key";
        let value = vec![b'v'; 60000];
        let expected_entry_size = (key.len() + value.len() + 8) as u64;

        let tx = TxId::new(1);
        storage.put(tx, key, &value).await.expect("put succeeds");

        storage
            .budget
            .consume_memory(990_000)
            .expect("fill budget to 990,000");

        let commit_res = storage.commit(tx).await;
        assert!(
            commit_res.is_ok(),
            "commit must succeed even when consume_memory fails"
        );

        assert_eq!(
            storage.budget_tracking_drift_bytes(),
            expected_entry_size,
            "budget drift metric must equal entry_size after consume_memory failure"
        );
    }
}
