use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_put_if_absent_cancellation_releases_intent_lock() {
    let dir = tempdir().expect("tempdir creation failed");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(
        LsmStorage::new(config)
            .await
            .expect("LsmStorage::new failed"),
    );

    let key = b"cancel_key";
    let val1 = b"val1";
    let val2 = b"val2";
    let tx1 = TxId::new(100);
    let tx2 = TxId::new(101);

    // Task A: We call put_if_absent in a task and abort the task immediately
    let storage_clone = Arc::clone(&storage);
    let handle = tokio::spawn(async move {
        let _ = storage_clone.put_if_absent(tx1, key, val1).await;
    });

    // Abort the task (drops the Future during or around execution)
    handle.abort();
    let _ = handle.await;

    // Transaction tx2 calls put_if_absent on the same key and must succeed because tx1's lock was dropped by RAII guard
    let put_res = storage
        .put_if_absent(tx2, key, val2)
        .await
        .expect("tx2 put_if_absent failed");
    assert!(
        put_res,
        "tx2 put_if_absent should succeed after tx1 put_if_absent task cancellation"
    );

    // Commit tx2 and verify get(key) returns val2
    storage.commit(tx2).await.expect("commit tx2 failed");
    let get_res = storage.get(key).await.expect("get failed");
    assert_eq!(get_res, Some(bytes::Bytes::from_static(val2)));
}

#[tokio::test]
async fn test_put_if_absent_success_holds_lock_until_commit_or_rollback() {
    let dir = tempdir().expect("tempdir creation failed");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(
        LsmStorage::new(config)
            .await
            .expect("LsmStorage::new failed"),
    );

    let key = b"hold_key";
    let tx1 = TxId::new(200);
    let tx2 = TxId::new(201);

    // 1. tx1 successfully calls put_if_absent
    let res1 = storage
        .put_if_absent(tx1, key, b"v1")
        .await
        .expect("tx1 put_if_absent failed");
    assert!(res1, "tx1 put_if_absent should return true");

    // 2. While tx1 is uncommitted, tx2 put_if_absent on key returns Ok(false)
    let res2 = storage
        .put_if_absent(tx2, key, b"v2")
        .await
        .expect("tx2 put_if_absent failed");
    assert!(
        !res2,
        "tx2 put_if_absent must return false while tx1 holds intent lock / staged status"
    );

    // 3. Rollback tx1, freeing the lock and clearing staged status
    storage.rollback(tx1).await.expect("tx1 rollback failed");

    // 4. Now tx2 put_if_absent on key succeeds
    let res3 = storage
        .put_if_absent(tx2, key, b"v2")
        .await
        .expect("tx2 put_if_absent failed after tx1 rollback");
    assert!(
        res3,
        "tx2 put_if_absent should return true after tx1 rollback releases lock"
    );

    storage.commit(tx2).await.expect("tx2 commit failed");
    let get_res = storage.get(key).await.expect("get key failed");
    assert_eq!(get_res, Some(bytes::Bytes::from_static(b"v2")));
}
