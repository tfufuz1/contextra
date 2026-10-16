use contextra_engine::transaction::DbTransaction;
use contextra_engine::{Contextra, ContextraConfig};
use contextra_ports::StorageEngine;
use std::time::Duration;

#[tokio::test]
async fn test_db_transaction_raii_drop_cleans_up_staged_writes() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let mut config = ContextraConfig::default();
    config.group_commit_window_micros = 0;

    let engine = Contextra::open_with_config(temp_dir.path(), config)
        .await
        .expect("open engine");
    let col = engine.collection("test_col").await.expect("get collection");

    let tx_id_1 = engine.allocate_tx().expect("allocate tx_id_1");
    let key = col.namespaced_key(b"test_raii_key", 0);
    let storage = col.storage();

    // 1. Create a DbTransaction wrapper for tx_id_1
    {
        let db_tx = DbTransaction::new(col.as_ref().clone(), tx_id_1);
        let written = storage
            .put_if_absent(db_tx.tx_id, &key, b"value_1")
            .await
            .expect("put_if_absent db_tx");
        assert!(written, "First write via DbTransaction should succeed");

        // db_tx is dropped here WITHOUT calling commit!
    }

    // Give background drop task a brief moment to run rollback
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 2. Second writer tx_id_2 attempts write on the same key
    let tx_id_2 = engine.allocate_tx().expect("allocate tx_id_2");
    let written_2 = storage
        .put_if_absent(tx_id_2, &key, b"value_2")
        .await
        .expect("put_if_absent tx_id_2 after DbTransaction drop");

    // Should succeed because DbTransaction RAII drop guard automatically rolled back tx_id_1!
    assert!(
        written_2,
        "Second write must succeed after DbTransaction RAII drop rolled back abandoned tx_id_1"
    );
}

#[tokio::test]
async fn test_raw_tx_id_manual_rollback_cleans_up_staged_writes() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let mut config = ContextraConfig::default();
    config.group_commit_window_micros = 0;

    let engine = Contextra::open_with_config(temp_dir.path(), config)
        .await
        .expect("open engine");
    let col = engine.collection("test_col").await.expect("get collection");

    let tx_id_1 = engine.allocate_tx().expect("allocate tx_id_1");
    let key = col.namespaced_key(b"test_raw_tx_key", 0);
    let storage = col.storage();

    let written = storage
        .put_if_absent(tx_id_1, &key, b"value_1")
        .await
        .expect("put_if_absent tx_id_1");
    assert!(written, "First write should succeed");

    // Abandon raw primitive tx_id_1 (Copy type, no RAII guard)
    let _ = tx_id_1;

    // Second write fails while tx_id_1 is staged
    let tx_id_2 = engine.allocate_tx().expect("allocate tx_id_2");
    let written_2_before = storage
        .put_if_absent(tx_id_2, &key, b"value_2")
        .await
        .expect("put_if_absent tx_id_2 before rollback");

    assert!(
        !written_2_before,
        "Second write fails while key is staged by un-rolled-back raw tx_id_1"
    );

    // Rollback raw tx_id_1
    storage.rollback(tx_id_1).await.expect("rollback tx_id_1");

    // Second write succeeds after explicit rollback
    let tx_id_3 = engine.allocate_tx().expect("allocate tx_id_3");
    let written_3_after = storage
        .put_if_absent(tx_id_3, &key, b"value_3")
        .await
        .expect("put_if_absent tx_id_3 after rollback");

    assert!(
        written_3_after,
        "Third write succeeds after raw tx_id_1 is explicitly rolled back"
    );
}
