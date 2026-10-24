#![cfg(feature = "fault-injection")]

use contextra_core::TxId;
use contextra_store::wal::{Wal, WalOp, FAIL_APPEND_AFTER_PARTIAL_BYTES, FAIL_APPEND_PARTIAL_ONCE};
use tempfile::tempdir;

static TEST_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[tokio::test]
async fn test_wal_torn_write_causes_poisoning_and_blocks_further_appends() {
    let _guard = TEST_MUTEX.lock().await;
    let dir = tempdir().unwrap();
    let wal_path = dir.path().join("test_torn_write.wal");

    let wal = Wal::open(&wal_path).await.unwrap();
    assert!(!wal.is_poisoned());

    // Create a valid entry first
    let op1 = WalOp::Put {
        tx_id: TxId::new(100),
        key: b"key1".to_vec(),
        value: b"val1".to_vec(),
    };
    let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await.unwrap();
    wal.append_batch(batch1).await.unwrap();

    let initial_size = wal.size();
    assert!(initial_size > 0);

    // Set fault injection for a partial write
    FAIL_APPEND_AFTER_PARTIAL_BYTES.store(10, std::sync::atomic::Ordering::SeqCst);
    FAIL_APPEND_PARTIAL_ONCE.store(true, std::sync::atomic::Ordering::SeqCst);

    let op2 = WalOp::Put {
        tx_id: TxId::new(101),
        key: b"key2".to_vec(),
        value: b"val2_long_enough_to_be_more_than_10_bytes".to_vec(),
    };
    let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await.unwrap();

    // Append should fail due to fault injection
    let res = wal.append_batch(batch2).await;
    assert!(res.is_err(), "Append should fail under fault injection");

    // (i) wal.is_poisoned() is true
    assert!(
        wal.is_poisoned(),
        "WAL must be poisoned after partial write failure"
    );

    let phys_size_after_fail = std::fs::metadata(&wal_path).unwrap().len();
    assert!(
        phys_size_after_fail > initial_size,
        "Physical file should contain partial bytes from torn write"
    );

    // (iii) A second append attempt fails immediately with Err, without changing physical file size
    let op3 = WalOp::Put {
        tx_id: TxId::new(102),
        key: b"key3".to_vec(),
        value: b"val3".to_vec(),
    };
    let (batch3, _) = wal.prepare_batch(vec![(op3, 3)]).await.unwrap();

    let res2 = wal.append_batch(batch3).await;
    assert!(res2.is_err(), "Subsequent append must fail while poisoned");

    let err_msg = res2.unwrap_err().to_string();
    assert!(
        err_msg.contains("poisoned"),
        "Error message should mention poisoned handle: {err_msg}"
    );

    let phys_size_after_second_attempt = std::fs::metadata(&wal_path).unwrap().len();
    assert_eq!(
        phys_size_after_fail, phys_size_after_second_attempt,
        "Physical file size must not change during rejected append on poisoned handle"
    );
}

#[tokio::test]
async fn test_wal_recover_from_poison_restores_handle_and_preserves_valid_entries() {
    let _guard = TEST_MUTEX.lock().await;
    let dir = tempdir().unwrap();
    let wal_path = dir.path().join("test_recovery.wal");

    let wal = Wal::open(&wal_path).await.unwrap();

    let op1 = WalOp::Put {
        tx_id: TxId::new(200),
        key: b"valid_key_1".to_vec(),
        value: b"valid_val_1".to_vec(),
    };
    let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await.unwrap();
    wal.append_batch(batch1).await.unwrap();

    // Trigger torn write
    FAIL_APPEND_AFTER_PARTIAL_BYTES.store(15, std::sync::atomic::Ordering::SeqCst);
    FAIL_APPEND_PARTIAL_ONCE.store(true, std::sync::atomic::Ordering::SeqCst);

    let op_torn = WalOp::Put {
        tx_id: TxId::new(201),
        key: b"torn_key".to_vec(),
        value: b"torn_value_payload_data".to_vec(),
    };
    let (batch_torn, _) = wal.prepare_batch(vec![(op_torn, 2)]).await.unwrap();

    let _ = wal.append_batch(batch_torn).await;
    assert!(wal.is_poisoned());

    // Call recover_from_poison()
    wal.recover_from_poison().await.unwrap();

    // (i) wal.is_poisoned() == false
    assert!(
        !wal.is_poisoned(),
        "WAL handle should no longer be poisoned after recovery"
    );

    // (ii) Regular append is successful again
    let op3 = WalOp::Put {
        tx_id: TxId::new(202),
        key: b"valid_key_2".to_vec(),
        value: b"valid_val_2".to_vec(),
    };
    let (batch3, _) = wal.prepare_batch(vec![(op3, 2)]).await.unwrap();

    wal.append_batch(batch3).await.unwrap();

    // (iii) Offline replay finds ALL entries confirmed as Ok before poison + after recovery
    let replayed = wal.replay().await.unwrap();
    assert_eq!(
        replayed.len(),
        2,
        "Offline replay should recover exactly 2 valid entries"
    );
    assert_eq!(replayed[0].1.tx_id().inner(), 200);
    assert_eq!(replayed[1].1.tx_id().inner(), 202);
}
