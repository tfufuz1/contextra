use contextra_ports::StorageEngine;
use contextra_store_redb::RedbStorageEngine;
use contextra_types::TxId;
use std::sync::Arc;
use tokio::task;

#[tokio::test]
async fn test_concurrent_commits_strict_sequence_ordering() {
    let temp = tempfile::tempdir().expect("tempdir");
    let db_path = temp.path().join("concurrent_test.redb");

    let engine = RedbStorageEngine::open(&db_path).expect("open engine");
    let engine = Arc::new(engine);

    let num_tasks = 10;
    let mut handles = Vec::new();

    for t in 0..num_tasks {
        let engine = Arc::clone(&engine);
        handles.push(tokio::spawn(async move {
            let tx_id = TxId(100 + t as u64);
            let key = format!("key_{:02}", t);
            let val = format!("val_{:02}", t);

            engine
                .put(tx_id, key.as_bytes(), val.as_bytes())
                .await
                .expect("put");

            engine.commit(tx_id).await.expect("commit");
        }));
    }

    for h in handles {
        h.await.expect("task finished");
    }

    let last_seq = engine.last_seq_no().await.expect("last_seq_no");
    assert_eq!(
        last_seq, num_tasks as u64,
        "all committed transactions must increment sequence number contiguously"
    );

    // Reopen and check persistence & exact final sequence
    drop(engine);
    let reopened = RedbStorageEngine::open(&db_path).expect("reopen engine");
    let reopened_seq = reopened.last_seq_no().await.expect("reopened seq");
    assert_eq!(reopened_seq, num_tasks as u64);

    let items = reopened.scan_prefix(b"").await.expect("scan");
    assert_eq!(items.len(), num_tasks);
}

#[tokio::test]
async fn test_staged_write_abort_and_drop_isolation() {
    let temp = tempfile::tempdir().expect("tempdir");
    let db_path = temp.path().join("abort_test.redb");

    let engine = RedbStorageEngine::open(&db_path).expect("open engine");

    let tx_committing = TxId(1);
    let tx_aborted = TxId(2);

    engine
        .put(tx_committing, b"committed_k", b"committed_v")
        .await
        .expect("put 1");
    engine.commit(tx_committing).await.expect("commit 1");

    // Stage uncommitted writes for tx_aborted
    engine
        .put(tx_aborted, b"staged_uncommitted_k", b"staged_v")
        .await
        .expect("put 2");

    // Rollback explicitly
    engine.rollback(tx_aborted).await.expect("rollback");

    let val_committed = engine.get(b"committed_k").await.expect("get committed");
    assert_eq!(val_committed.as_deref(), Some(&b"committed_v"[..]));

    let val_aborted = engine
        .get(b"staged_uncommitted_k")
        .await
        .expect("get aborted");
    assert_eq!(
        val_aborted, None,
        "uncommitted staged ops must not be visible after rollback"
    );
}

#[tokio::test]
async fn test_concurrent_readers_during_active_writes() {
    let temp = tempfile::tempdir().expect("tempdir");
    let db_path = temp.path().join("mvcc_readers.redb");

    let engine = RedbStorageEngine::open(&db_path).expect("open engine");
    let engine = Arc::new(engine);

    // Seed key k1 with v1 at TxId(1)
    let tx1 = TxId(1);
    engine.put(tx1, b"k1", b"v1").await.expect("put initial");
    engine.commit(tx1).await.expect("commit initial");

    let seq_initial = engine.last_seq_no().await.expect("seq initial");

    // Spawn reader task reading as_of_seq(seq_initial)
    let engine_reader = Arc::clone(&engine);
    let reader_handle = task::spawn(async move {
        for _ in 0..50 {
            let val = engine_reader
                .get_at_seq(b"k1", seq_initial)
                .await
                .expect("get_at_seq");
            assert_eq!(
                val.as_deref(),
                Some(&b"v1"[..]),
                "reader must see snapshot at seq_initial despite concurrent updates"
            );
            tokio::task::yield_now().await;
        }
    });

    // Writer task performing updates
    let engine_writer = Arc::clone(&engine);
    let writer_handle = task::spawn(async move {
        for i in 2..=10 {
            let tx = TxId(i);
            let val = format!("v_{}", i);
            engine_writer
                .put(tx, b"k1", val.as_bytes())
                .await
                .expect("put update");
            engine_writer.commit(tx).await.expect("commit update");
            tokio::task::yield_now().await;
        }
    });

    reader_handle.await.expect("reader ok");
    writer_handle.await.expect("writer ok");

    let final_val = engine.get(b"k1").await.expect("get latest");
    assert_eq!(final_val.as_deref(), Some(&b"v_10"[..]));
}

#[tokio::test]
async fn test_rollback_to_tx_and_reopen_recovery() {
    let temp = tempfile::tempdir().expect("tempdir");
    let db_path = temp.path().join("rollback_to_tx.redb");

    let engine = RedbStorageEngine::open(&db_path).expect("open engine");

    // Commit transactions 1 through 5
    for i in 1..=5 {
        let tx = TxId(i);
        let k = format!("k_{}", i);
        let v = format!("v_{}", i);
        engine
            .put(tx, k.as_bytes(), v.as_bytes())
            .await
            .expect("put");
        engine.commit(tx).await.expect("commit");
    }

    assert_eq!(engine.last_tx_id().await.expect("last tx"), TxId(5));
    assert_eq!(engine.last_seq_no().await.expect("last seq"), 5);

    // Rollback to TxId(3)
    engine
        .rollback_to_tx(TxId(3))
        .await
        .expect("rollback to tx 3");

    assert_eq!(
        engine.last_tx_id().await.expect("last tx post rollback"),
        TxId(3)
    );
    assert_eq!(
        engine.last_seq_no().await.expect("last seq post rollback"),
        3
    );

    assert!(engine.get(b"k_3").await.expect("get k_3").is_some());
    assert!(engine.get(b"k_4").await.expect("get k_4").is_none());
    assert!(engine.get(b"k_5").await.expect("get k_5").is_none());

    // Reopen and ensure state matches rollback target
    drop(engine);
    let reopened = RedbStorageEngine::open(&db_path).expect("reopen");
    assert_eq!(
        reopened.last_tx_id().await.expect("reopened last tx"),
        TxId(3)
    );
    assert_eq!(reopened.last_seq_no().await.expect("reopened last seq"), 3);
    assert!(reopened
        .get(b"k_3")
        .await
        .expect("reopened get k_3")
        .is_some());
    assert!(reopened
        .get(b"k_4")
        .await
        .expect("reopened get k_4")
        .is_none());
}
