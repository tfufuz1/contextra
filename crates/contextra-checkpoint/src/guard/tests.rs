use super::*;
use contextra_ports::{BoxFuture, StorageEngine, StorageStats};
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};

struct MockStorage {
    data: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
    pinned: Mutex<HashSet<u64>>,
    rolled_back_tx: Mutex<Vec<TxId>>,
}

impl MockStorage {
    fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
            pinned: Mutex::new(HashSet::new()),
            rolled_back_tx: Mutex::new(Vec::new()),
        }
    }
}

impl StorageEngine for MockStorage {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { Ok(self.data.lock().get(key).cloned().map(bytes::Bytes::from)) })
    }
    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        _seq: u64,
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { self.get(key).await })
    }
    fn put<'a>(
        &'a self,
        _tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.data.lock().insert(key.to_vec(), value.to_vec());
            Ok(())
        })
    }
    fn delete<'a>(&'a self, _tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.data.lock().remove(key);
            Ok(())
        })
    }
    fn commit<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn rollback<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.rolled_back_tx.lock().push(tx_id);
            Ok(())
        })
    }
    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move { Ok(0) })
    }
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(TxId::new(0)) })
    }
    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        Box::pin(async move {
            Ok(StorageStats {
                num_segments: 0,
                total_size_bytes: 0,
                memtable_size_bytes: 0,
            })
        })
    }
    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.pinned.lock().insert(seq_no);
            Ok(())
        })
    }
    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.pinned.lock().remove(&seq_no);
            Ok(())
        })
    }
    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let data = self.data.lock();
            Ok(data
                .iter()
                .filter(|(k, _)| k.starts_with(prefix))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect())
        })
    }
    fn scan<'a>(
        &'a self,
        _s: std::ops::Bound<&'a [u8]>,
        _e: std::ops::Bound<&'a [u8]>,
        _: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { Ok(Vec::new()) })
    }
}

#[test]
fn test_orphan_registry_persists_across_drop() {
    let registry = Arc::new(InstanceOrphanRegistry::new(""));
    let storage = Arc::new(MockStorage::new());
    let seq_no = 12345;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime for test");

    let guard = rt.block_on(async {
        PinGuard::pin(storage.clone(), seq_no, registry.clone())
            .await
            .unwrap()
    });

    // Drop guard without runtime or explicit unpin/defuse
    drop(guard);

    // Verify orphan ID appears in registry
    let orphans = registry.get_orphan_pins();
    assert_eq!(
        orphans.len(),
        1,
        "Orphan sequence number 12345 must appear in registry upon PinGuard drop"
    );
    assert_eq!(orphans[0].seq_no, seq_no);
}

#[test]
fn test_rollback_blocking_in_sync_context() {
    let storage = Arc::new(MockStorage::new());
    let cp = StateCheckpoint {
        tx_id: TxId::new(909),
        timestamp_ms: 1000,
        namespace: Some("test".to_string()),
    };
    let guard = CheckpointGuard::new(cp, storage.clone(), "test");
    let res = guard.rollback_blocking();
    assert!(
        res.is_ok(),
        "rollback_blocking must succeed in sync context"
    );

    let rolled_back = storage.rolled_back_tx.lock().clone();
    assert_eq!(rolled_back, vec![TxId::new(909)]);
}

#[tokio::test]
async fn test_rollback_blocking_in_async_context_returns_error() {
    let storage = Arc::new(MockStorage::new());
    let cp = StateCheckpoint {
        tx_id: TxId::new(1010),
        timestamp_ms: 1000,
        namespace: Some("test".to_string()),
    };
    let guard = CheckpointGuard::new(cp, storage, "test");

    // Timeout safety net to guarantee no hanging/deadlock
    let res = tokio::time::timeout(std::time::Duration::from_secs(2), async move {
        guard.rollback_blocking()
    })
    .await
    .expect("rollback_blocking in async context timed out - possible deadlock!");

    assert!(
        res.is_err(),
        "rollback_blocking called from async context must return error immediately to prevent deadlock"
    );
    if let Err(ContextraError::Internal(msg)) = res {
        assert!(msg.contains("active async Tokio runtime context"));
        assert!(msg.contains("rollback().await"));
    } else {
        panic!(
            "Expected ContextraError::Internal error message instructing to use rollback().await"
        );
    }
}

#[tokio::test]
async fn test_checkpoint_guard_for_agent_step() {
    let storage = Arc::new(MockStorage::new());
    let guard = CheckpointGuard::for_agent_step(storage.clone(), TxId::new(55))
        .await
        .unwrap();

    let cp = guard.checkpoint().unwrap();
    assert_eq!(cp.tx_id, TxId::new(55));
    assert!(cp.timestamp_ms > 0);

    let committed = guard.commit().unwrap();
    assert_eq!(committed.tx_id, TxId::new(55));
}

#[allow(non_snake_case)]
#[tokio::test]
async fn checkpoint_guard_CASE_uncommitted_guard_holds_state() {
    let storage = Arc::new(MockStorage::new());
    let cp = StateCheckpoint {
        tx_id: TxId::new(500),
        timestamp_ms: 1000,
        namespace: Some("test".to_string()),
    };
    let guard = CheckpointGuard::new(cp, storage, "test");
    let cp_ref = guard.checkpoint().expect("// expect #[cfg(test)]").clone();
    assert_eq!(cp_ref.tx_id, TxId::new(500));

    // Commit takes ownership of self and consumes the state checkpoint
    let committed_cp = guard.commit().expect("// expect #[cfg(test)]");
    assert_eq!(committed_cp.tx_id, TxId::new(500));
}

#[allow(non_snake_case)]
#[tokio::test]
async fn checkpoint_guard_CASE_commit_moves_ownership() {
    let storage = Arc::new(MockStorage::new());
    let guard = CheckpointGuard::new(
        StateCheckpoint {
            tx_id: TxId::new(777),
            timestamp_ms: 12345,
            namespace: Some("test".to_string()),
        },
        storage,
        "test",
    );

    assert!(guard.checkpoint().is_ok());

    let cp = guard.commit().expect("// expect #[cfg(test)]");
    assert_eq!(cp.tx_id, TxId::new(777));
    assert_eq!(cp.timestamp_ms, 12345);
}

#[allow(non_snake_case)]
#[tokio::test]
async fn checkpoint_guard_CASE_rollback_consumed_returns_err() {
    let dummy_storage = Arc::new(MockStorage::new());
    let consumed_guard = CheckpointGuard::<MockStorage> {
        checkpoint: None,
        storage: dummy_storage,
        namespace: "test".to_string(),
        orphan_registry: Arc::new(InstanceOrphanRegistry::new("")),
        skipped_rollbacks: Arc::new(AtomicU64::new(0)),
        clock: Arc::new(SystemClock::new()),
    };
    assert!(matches!(
        consumed_guard.checkpoint(),
        Err(ContextraError::Internal(_))
    ));

    let consumed_guard2 = CheckpointGuard::<MockStorage> {
        checkpoint: None,
        storage: Arc::new(MockStorage::new()),
        namespace: "test".to_string(),
        orphan_registry: Arc::new(InstanceOrphanRegistry::new("")),
        skipped_rollbacks: Arc::new(AtomicU64::new(0)),
        clock: Arc::new(SystemClock::new()),
    };
    assert!(matches!(
        consumed_guard2.commit(),
        Err(ContextraError::Internal(_))
    ));

    let consumed_guard3 = CheckpointGuard::<MockStorage> {
        checkpoint: None,
        storage: Arc::new(MockStorage::new()),
        namespace: "test".to_string(),
        orphan_registry: Arc::new(InstanceOrphanRegistry::new("")),
        skipped_rollbacks: Arc::new(AtomicU64::new(0)),
        clock: Arc::new(SystemClock::new()),
    };
    let res = consumed_guard3.rollback().await;
    assert!(matches!(res, Err(ContextraError::Internal(_))));
}
