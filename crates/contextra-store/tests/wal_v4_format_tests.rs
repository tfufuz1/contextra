use contextra_core::TxId;
use contextra_store::wal::{
    Wal, WalConfig, WalOp, WalVersion, WAL_V3_HEADER,
};
use tempfile::TempDir;

#[tokio::test]
async fn test_wal_v4_golden_bytes_header_and_ops() {
    let tmp = TempDir::new().expect("tempdir");
    let wal_path = tmp.path().join("wal_v4_golden.log");

    let mut config = WalConfig::default();
    config.min_wal_version = WalVersion::V3;

    let wal = Wal::open_with_config(&wal_path, config).await.expect("open wal");

    let op_put = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"idx_test_123".to_vec(),
        value: b"CREATE_BTREE".to_vec(),
    };

    let op_end = WalOp::TxEnd {
        tx_id: TxId::new(10),
        committed: true,
    };

    let (batch, _) = wal
        .prepare_batch(vec![(op_put, 1), (op_end, 2)])
        .await
        .expect("prepare batch");

    wal.append_batch(batch).await.expect("append batch");

    let raw_bytes = tokio::fs::read(&wal_path).await.expect("read wal file");

    assert!(
        raw_bytes.len() >= 4,
        "WAL file must be at least 4 bytes header"
    );
    assert_eq!(
        &raw_bytes[0..4],
        WAL_V3_HEADER,
        "WAL file header must match MFW3 magic"
    );
}

#[tokio::test]
async fn test_wal_v4_open_index_intents_replay() {
    let tmp = TempDir::new().expect("tempdir");
    let wal_path = tmp.path().join("wal_v4_intents.log");

    let mut config = WalConfig::default();
    config.min_wal_version = WalVersion::V3;

    let wal = Wal::open_with_config(&wal_path, config).await.expect("open wal");

    let op_put1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"idx_a".to_vec(),
        value: b"BUILD".to_vec(),
    };

    let op_put2 = WalOp::Put {
        tx_id: TxId::new(2),
        key: b"idx_b".to_vec(),
        value: b"BUILD".to_vec(),
    };

    let op_end1 = WalOp::TxEnd {
        tx_id: TxId::new(1),
        committed: true,
    };

    let (batch, _) = wal
        .prepare_batch(vec![(op_put1, 10), (op_put2, 11), (op_end1, 12)])
        .await
        .expect("prepare batch");

    wal.append_batch(batch).await.expect("append batch");
    assert_eq!(wal.size(), tokio::fs::metadata(&wal_path).await.unwrap().len());
}

#[tokio::test]
async fn test_wal_v4_downgrade_rejected() {
    let tmp = TempDir::new().expect("tempdir");
    let wal_path = tmp.path().join("wal_v4_downgrade.log");

    tokio::fs::write(&wal_path, b"MFW4_SOME_V4_CONTENT_HERE")
        .await
        .expect("write mock v4 wal");

    let mut config = WalConfig::default();
    config.min_wal_version = WalVersion::V3;

    let res = Wal::open_with_config(&wal_path, config).await;
    assert!(
        res.is_err(),
        "Opening V4 WAL with min_wal_version V3 without V4 support should be handled"
    );
}
