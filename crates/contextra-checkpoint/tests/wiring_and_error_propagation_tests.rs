#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_checkpoint::{
    wiring::{CheckpointWiring, CheckpointWiringDeps},
    CheckpointGuard, InstanceOrphanRegistry, PersistentCheckpointStore, PinGuard, StateCheckpoint,
};
use contextra_ports::{BoxFuture, Clock, ManualClock, StorageEngine, StorageStats};
use contextra_types::{ContextraError, Result, TxId};
use parking_lot::Mutex;
use std::sync::Arc;

struct FaultyStorage {
    fail_unpin: Mutex<bool>,
}

impl StorageEngine for FaultyStorage {
    fn get<'a>(&'a self, _key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { Ok(None) })
    }
    fn get_at_seq<'a>(
        &'a self,
        _key: &'a [u8],
        _seq: u64,
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { Ok(None) })
    }
    fn put<'a>(
        &'a self,
        _tx_id: TxId,
        _key: &'a [u8],
        _value: &'a [u8],
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn delete<'a>(&'a self, _tx_id: TxId, _key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn commit<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn rollback<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
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
    fn pin_checkpoint<'a>(&'a self, _seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn unpin_checkpoint<'a>(&'a self, _seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            if *self.fail_unpin.lock() {
                Err(ContextraError::Storage(
                    "Simulated unpin I/O failure".into(),
                ))
            } else {
                Ok(())
            }
        })
    }
    fn scan_prefix<'a>(
        &'a self,
        _prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { Ok(Vec::new()) })
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

#[tokio::test]
async fn test_pin_guard_unpin_failure_retains_orphan_pin() {
    let clock: Arc<dyn Clock> = Arc::new(ManualClock::new(100_000));
    let storage = Arc::new(FaultyStorage {
        fail_unpin: Mutex::new(true),
    });
    let orphan_reg = Arc::new(InstanceOrphanRegistry::new("").with_clock(clock.clone()));

    let pin_guard = PinGuard::pin_with_clock(storage, 888, orphan_reg.clone(), clock)
        .await
        .expect("pin_with_clock");

    let unpin_res = pin_guard.unpin().await;
    assert!(
        unpin_res.is_err(),
        "unpin must return error when storage unpin fails"
    );

    let orphans = orphan_reg.get_orphan_pins();
    assert_eq!(
        orphans.len(),
        1,
        "Sequence pin must NOT be lost on unpin failure; registered as orphan for recovery"
    );
    assert_eq!(orphans[0].seq_no, 888);
}

#[tokio::test]
async fn test_wiring_build_creates_valid_instance() {
    let clock: Arc<dyn Clock> = Arc::new(ManualClock::new(1_000_000));
    let storage = Arc::new(FaultyStorage {
        fail_unpin: Mutex::new(false),
    });
    let store = PersistentCheckpointStore::open(storage, "wiring_test")
        .await
        .expect("open store")
        .with_clock(clock.clone());

    let orphan_reg = store.orphan_registry().clone();

    let deps = CheckpointWiringDeps {
        store: Arc::new(store),
        orphan_registry: orphan_reg,
        clock,
    };

    let wiring = CheckpointWiring::build(deps).expect("CheckpointWiring::build should succeed");
    let recovered = wiring
        .recover_orphaned_pins()
        .await
        .expect("recover_orphaned_pins should succeed");
    assert!(recovered.is_empty());
}

#[tokio::test]
async fn test_new_with_clock_constructor() {
    let clock: Arc<dyn Clock> = Arc::new(ManualClock::new(500_000_000));
    let storage = Arc::new(FaultyStorage {
        fail_unpin: Mutex::new(false),
    });
    let cp = StateCheckpoint {
        tx_id: TxId::new(10),
        timestamp_ms: 500,
        namespace: Some("test_ns".to_string()),
    };

    let guard = CheckpointGuard::new_with_clock(cp, storage, "test_ns", clock.clone());
    assert_eq!(guard.clock().now_unix_nanos(), 500_000_000);
}

#[tokio::test]
async fn test_inv_checkpoint_determinism_1_same_seed_and_clock() {
    let clock1 = Arc::new(ManualClock::new(10_000_000));
    let storage1 = Arc::new(FaultyStorage {
        fail_unpin: Mutex::new(false),
    });
    let cp1 = StateCheckpoint {
        tx_id: TxId::new(42),
        timestamp_ms: 10,
        namespace: Some("det".to_string()),
    };
    let guard1 = CheckpointGuard::new_with_clock(cp1, storage1, "det", clock1);

    let clock2 = Arc::new(ManualClock::new(10_000_000));
    let storage2 = Arc::new(FaultyStorage {
        fail_unpin: Mutex::new(false),
    });
    let cp2 = StateCheckpoint {
        tx_id: TxId::new(42),
        timestamp_ms: 10,
        namespace: Some("det".to_string()),
    };
    let guard2 = CheckpointGuard::new_with_clock(cp2, storage2, "det", clock2);

    assert_eq!(guard1.checkpoint().unwrap(), guard2.checkpoint().unwrap());
    assert_eq!(
        guard1.clock().now_unix_nanos(),
        guard2.clock().now_unix_nanos()
    );
}
