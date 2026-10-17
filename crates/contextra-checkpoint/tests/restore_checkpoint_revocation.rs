// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Integrationstest für Checkpoint-Restore mit Anti-Resurrection CpyRestoreFilter.
// INVARIANTEN: Zero-Panic, Fail-Closed, Anti-Resurrection nach Checkpoint-Restore.

use contextra_checkpoint::{CpyRestoreFilter, NoOpRestoreFilter, PersistentCheckpointStore};
use contextra_ports::StorageEngine;
use contextra_testkit::InMemoryStorageEngine;
use contextra_types::TxId;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

#[derive(Default)]
struct MockRevocationFilter {
    revoked_keys: RwLock<HashSet<Vec<u8>>>,
}

impl MockRevocationFilter {
    fn revoke(&self, key: &[u8]) {
        self.revoked_keys
            .write()
            .expect("lock")
            .insert(key.to_vec());
    }
}

impl CpyRestoreFilter for MockRevocationFilter {
    fn drop_record(&self, key: &[u8], _tx: u64) -> bool {
        self.revoked_keys.read().expect("lock").contains(key)
    }
}

#[tokio::test]
async fn test_checkpoint_restore_revocation_filter() {
    let storage = Arc::new(InMemoryStorageEngine::new());
    let store = PersistentCheckpointStore::open(storage.clone(), "test_ns")
        .await
        .expect("open store");

    let filter = Arc::new(MockRevocationFilter::default());
    let store = store.with_restore_filter(filter.clone());

    // 1. Write initial data at tx = 10
    #[allow(deprecated)]
    let tx10 = TxId::new(10);
    storage
        .put(tx10, b"key_normal", b"val_normal")
        .await
        .expect("put");
    storage
        .put(tx10, b"key_erased", b"val_erased")
        .await
        .expect("put");
    storage.commit(tx10).await.expect("commit");

    // 2. Create checkpoint CP1 at tx = 10
    let cp1 = store
        .create_checkpoint("CP1", "col_1", 1, tx10, serde_json::json!({}))
        .await
        .expect("create checkpoint");
    assert_eq!(cp1.tx_id, tx10);

    // 3. Delete key_erased at tx = 20 and record in revocation filter
    #[allow(deprecated)]
    let tx20 = TxId::new(20);
    storage.delete(tx20, b"key_erased").await.expect("delete");
    filter.revoke(b"key_erased");
    storage.commit(tx20).await.expect("commit");

    // 4. Restore CP1 -> key_erased MUST NOT be resurrected!
    let restored_meta = store
        .restore_checkpoint("CP1")
        .await
        .expect("restore checkpoint");
    assert_eq!(restored_meta.tx_id, tx10);

    let val_normal = storage.get(b"key_normal").await.expect("get key_normal");
    assert_eq!(val_normal.as_deref(), Some(&b"val_normal"[..]));

    let val_erased = storage.get(b"key_erased").await.expect("get key_erased");
    assert_eq!(
        val_erased, None,
        "Erased key must not be resurrected during restore!"
    );
}

#[tokio::test]
async fn test_checkpoint_restore_without_filter_fails_closed_when_history_exists() {
    let storage = Arc::new(InMemoryStorageEngine::new());
    let store = PersistentCheckpointStore::open(storage.clone(), "test_ns_fail_closed")
        .await
        .expect("open store");

    // 1. Write initial data at tx = 10 & create checkpoint
    #[allow(deprecated)]
    let tx10 = TxId::new(10);
    storage.put(tx10, b"k1", b"v1").await.expect("put");
    storage.commit(tx10).await.expect("commit");

    store
        .create_checkpoint("CP1", "col_1", 1, tx10, serde_json::json!({}))
        .await
        .expect("create checkpoint");

    // 2. Perform subsequent transaction tx = 20
    #[allow(deprecated)]
    let tx20 = TxId::new(20);
    storage.put(tx20, b"k2", b"v2").await.expect("put");
    storage.commit(tx20).await.expect("commit");

    // 3. Attempt restore without filter -> MUST fail closed!
    let err = store.restore_checkpoint("CP1").await.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("Fail-closed restore barrier"),
        "Unexpected error message: {err_msg}"
    );

    // 4. Set explicit NoOpRestoreFilter -> restore succeeds
    let store_noop = store.with_restore_filter(Arc::new(NoOpRestoreFilter));
    let restored = store_noop
        .restore_checkpoint("CP1")
        .await
        .expect("restore with noop filter");
    assert_eq!(restored.tx_id, tx10);
}
