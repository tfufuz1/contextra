use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use tempfile::TempDir;

#[tokio::test]
async fn test_memory_budget_accounting_leak() {
    let temp_dir = TempDir::new().unwrap();
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        max_ram_mb: 64,
        ..Default::default()
    };

    let storage = LsmStorage::new(config).await.unwrap();

    let initial_used = storage.budget_for_test().memory_used();

    // Insert 100 entries via put_batch + commit
    for i in 0..100 {
        let key = format!("key_{:04}", i).into_bytes();
        let val = vec![0u8; 1024]; // 1KB value
        let tx_id = TxId::new(i + 1);
        storage
            .put_batch(tx_id, &[(key, val)])
            .await
            .expect("put_batch failed");
        storage.commit(tx_id).await.expect("commit failed");
    }

    let used_after_commits = storage.budget_for_test().memory_used();
    println!("Initial budget used: {initial_used}");
    println!("Budget used after commits: {used_after_commits}");
    assert!(
        used_after_commits > initial_used,
        "Memory budget used should increase after commits"
    );

    // Force flush
    storage.force_flush().await.expect("flush failed");

    let used_after_flush = storage.budget_for_test().memory_used();
    println!("Budget used after flush: {used_after_flush}");

    // After flush of all memtables, budget used should return to initial_used (or 0)
    assert_eq!(
        used_after_flush, initial_used,
        "Memory budget should be fully released after flushing all memtables"
    );
}

#[tokio::test]
async fn test_memory_budget_accounting_rollback_leak() {
    let temp_dir = TempDir::new().unwrap();
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        max_ram_mb: 64,
        ..Default::default()
    };

    let storage = LsmStorage::new(config).await.unwrap();

    let _initial_used = storage.budget_for_test().memory_used();

    // Insert entries for TxId(10) and TxId(20)
    for i in 0..50 {
        let key = format!("tx10_key_{:04}", i).into_bytes();
        let val = vec![0u8; 1024];
        let tx_id = TxId::new(10 + i % 2); // 10 or 11
        storage
            .put_batch(tx_id, &[(key, val)])
            .await
            .expect("put_batch failed");
        storage.commit(tx_id).await.expect("commit failed");
    }

    for i in 0..50 {
        let key = format!("tx20_key_{:04}", i).into_bytes();
        let val = vec![0u8; 1024];
        let tx_id = TxId::new(20 + i); // 20..70
        storage
            .put_batch(tx_id, &[(key, val)])
            .await
            .expect("put_batch failed");
        storage.commit(tx_id).await.expect("commit failed");
    }

    let used_before_rollback = storage.budget_for_test().memory_used();

    // Rollback to TxId(11)
    storage
        .rollback_to_tx(TxId::new(11))
        .await
        .expect("rollback failed");

    let used_after_rollback = storage.budget_for_test().memory_used();
    println!("Budget used before rollback: {used_before_rollback}");
    println!("Budget used after rollback: {used_after_rollback}");

    // Budget used after rollback should decrease proportionally to rolled back entries
    assert!(
        used_after_rollback < used_before_rollback,
        "Memory budget should decrease after transaction rollback"
    );
}
