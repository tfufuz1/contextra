use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::sync::Arc;
use tempfile::TempDir;

async fn test_commit_read_lock_concurrency_mode(window_micros: u64) {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 64 * 1024 * 1024,
        max_ram_mb: 512,
        group_commit_window_micros: window_micros,
        ..Default::default()
    };

    let storage = Arc::new(LsmStorage::new(config).await.expect("create storage"));

    // Prepare a key/value batch
    let num_items = 50;
    for i in 1..=num_items {
        let tx = TxId::new(i);
        let key = format!("k_{:04}", i).into_bytes();
        let val = format!("v_{:04}", i).into_bytes();
        storage.put(tx, &key, &val).await.expect("put");
    }

    // Spawn concurrent reader tasks while commits occur
    let mut join_set = tokio::task::JoinSet::new();

    // Reader task constantly reading storage while commits run
    let storage_reader = Arc::clone(&storage);
    join_set.spawn(async move {
        for _ in 0..100 {
            let read_res = storage_reader.get(b"k_0001").await;
            assert!(read_res.is_ok(), "Concurrent read failed");
            tokio::task::yield_now().await;
        }
    });

    // Writer/Committer task
    let storage_writer = Arc::clone(&storage);
    join_set.spawn(async move {
        for i in 1..=num_items {
            let tx = TxId::new(i);
            storage_writer.commit(tx).await.expect("commit");
        }
    });

    while let Some(res) = join_set.join_next().await {
        res.expect("task panicked");
    }

    // Verify all keys are present after commits
    for i in 1..=num_items {
        let key = format!("k_{:04}", i).into_bytes();
        let val = format!("v_{:04}", i).into_bytes();
        let res = storage.get(&key).await.expect("get").expect("key exists");
        assert_eq!(res, val);
    }
}

#[tokio::test]
async fn test_commit_read_lock_concurrency_single_commit() {
    test_commit_read_lock_concurrency_mode(0).await;
}

#[tokio::test]
async fn test_commit_read_lock_concurrency_group_commit() {
    test_commit_read_lock_concurrency_mode(500).await;
}
