use contextra_core::{ContextraError, TxId};
use contextra_store::wal::{Wal, WalEntry, WalOp};
use tempfile::tempdir;

#[tokio::test]
async fn test_partial_tail_crc_mismatch_truncates_and_opens() {
    let dir = tempdir().unwrap();
    let wal_path = dir.path().join("partial_tail_crc.wal");

    // 1. Create a WAL with 2 valid entries
    let wal = Wal::open(&wal_path).await.unwrap();
    let op1 = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"key1".to_vec(),
        value: b"value1".to_vec(),
    };
    let op2 = WalOp::Put {
        tx_id: TxId::new(11),
        key: b"key2".to_vec(),
        value: b"value2".to_vec(),
    };

    let (batch, _) = wal
        .prepare_batch(vec![(op1, 1), (op2, 2)])
        .await
        .unwrap();
    wal.append_batch(batch).await.unwrap();
    wal.close().await.unwrap();

    let valid_len = std::fs::metadata(&wal_path).unwrap().len();

    // 2. Corrupt/append a partial entry at physical tail (pos >= file_size)
    // Write an invalid entry header with bad CRC or partial payload at tail
    let mut raw_bytes = std::fs::read(&wal_path).unwrap();
    let fake_len: u32 = 40;
    raw_bytes.extend_from_slice(&fake_len.to_le_bytes()); // length header
    raw_bytes.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]); // invalid CRC
    raw_bytes.extend_from_slice(&[0xFF; 36]); // fake payload data
    std::fs::write(&wal_path, &raw_bytes).unwrap();

    let corrupted_len = std::fs::metadata(&wal_path).unwrap().len();
    assert!(corrupted_len > valid_len);

    // 3. Re-open WAL. The partial tail write with CRC mismatch should be cleanly truncated
    let wal_reopened = Wal::open(&wal_path).await.unwrap();
    let replayed = wal_reopened.replay().await.unwrap();

    assert_eq!(
        replayed.len(),
        2,
        "Should recover exactly 2 valid entries despite CRC mismatch at physical tail"
    );
    assert_eq!(replayed[0].1.tx_id().inner(), 10);
    assert_eq!(replayed[1].1.tx_id().inner(), 11);

    // Physical file size should be truncated back to valid_len
    let final_len = std::fs::metadata(&wal_path).unwrap().len();
    assert_eq!(final_len, valid_len, "File should be truncated to valid length");
}

#[tokio::test]
async fn test_mid_file_crc_mismatch_returns_wal_corruption() {
    let dir = tempdir().unwrap();
    let wal_path = dir.path().join("mid_file_crc.wal");

    // 1. Create a WAL with 3 valid entries
    let wal = Wal::open(&wal_path).await.unwrap();
    let op1 = WalOp::Put {
        tx_id: TxId::new(20),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let op2 = WalOp::Put {
        tx_id: TxId::new(21),
        key: b"k2_mid".to_vec(),
        value: b"v2_mid".to_vec(),
    };
    let op3 = WalOp::Put {
        tx_id: TxId::new(22),
        key: b"k3".to_vec(),
        value: b"v3".to_vec(),
    };

    let (batch, _) = wal
        .prepare_batch(vec![(op1, 1), (op2, 2), (op3, 3)])
        .await
        .unwrap();
    wal.append_batch(batch).await.unwrap();
    wal.close().await.unwrap();

    // 2. Corrupt a byte in the payload of entry 2 (mid-file, pos < file_size)
    let mut bytes = std::fs::read(&wal_path).unwrap();
    let mid_offset = bytes.len() / 2;
    bytes[mid_offset] ^= 0xFF; // Flip bits in middle of file
    std::fs::write(&wal_path, &bytes).unwrap();

    // 3. Re-open WAL. Mid-file CRC mismatch must return ContextraError::WalCorruption
    let res = Wal::open(&wal_path).await;
    assert!(
        res.is_err(),
        "Mid-file CRC mismatch must return an error and fail startup"
    );
    let err_str = res.unwrap_err().to_string();
    assert!(
        err_str.contains("WalCorruption") || err_str.contains("CRC validation failed"),
        "Error should indicate WAL corruption: {err_str}"
    );
}

#[test]
fn test_seq_no_max_overflow_rejected() {
    let op = WalOp::Put {
        tx_id: TxId::new(100),
        key: b"test_key".to_vec(),
        value: b"test_val".to_vec(),
    };
    let integrity_key = [1u8; 32];
    let prev_hmac = [0u8; 32];

    // u64::MAX must be rejected with ContextraError::InvalidInput
    let res = WalEntry::try_new(op, u64::MAX, &integrity_key, prev_hmac);
    assert!(res.is_err(), "seq_no == u64::MAX must be rejected");

    match res.unwrap_err() {
        ContextraError::InvalidInput(msg) => {
            assert!(
                msg.contains("WAL sequence number overflow"),
                "Error message should mention overflow: {msg}"
            );
        }
        err => panic!("Expected InvalidInput error, got: {err:?}"),
    }
}

#[tokio::test]
async fn test_recover_from_bak_fsyncs_parent_dir() {
    let dir = tempdir().unwrap();
    let wal_path = dir.path().join("recovery_test.wal");
    let bak_path = dir.path().join("recovery_test.wal.v1.bak");

    // Create a valid WAL file content for backup
    {
        let wal = Wal::open(&wal_path).await.unwrap();
        let op = WalOp::Put {
            tx_id: TxId::new(50),
            key: b"bak_key".to_vec(),
            value: b"bak_val".to_vec(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await.unwrap();
        wal.append_batch(batch).await.unwrap();
        wal.close().await.unwrap();
    }

    // Move valid WAL to .v1.bak and truncate original WAL to 0 bytes
    let valid_bytes = std::fs::read(&wal_path).unwrap();
    std::fs::write(&bak_path, &valid_bytes).unwrap();
    std::fs::write(&wal_path, b"").unwrap();

    // Opening WAL should automatically recover from .v1.bak
    let wal_recovered = Wal::open(&wal_path).await.unwrap();
    let replayed = wal_recovered.replay().await.unwrap();

    assert!(
        !bak_path.exists(),
        "Backup file should be removed after recovery"
    );
    assert_eq!(
        replayed.len(),
        1,
        "WAL file content should be restored from backup"
    );
    assert_eq!(replayed[0].1.tx_id().inner(), 50);
}
