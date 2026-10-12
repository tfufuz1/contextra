use contextra_core::TxId;
use contextra_store::wal::{
    Wal, WalCdcEvent, WalConfig, WalEntry, WalOp, WalVersion, WAL_V4_HEADER,
};
use std::path::Path;
use tempfile::TempDir;

#[tokio::test]
async fn test_wal_v4_golden_bytes_header_and_ops() {
    let tmp = TempDir::new().expect("tempdir");
    let wal_path = tmp.path().join("wal_v4_golden.log");

    let config = WalConfig {
        target_wal_version: WalVersion::V4,
        min_wal_version: WalVersion::V4,
        ..Default::default()
    };

    let wal = Wal::open_with_config(&wal_path, config).await.expect("open wal");

    let op_intent = WalOp::IndexIntent {
        tx_id: TxId::new(10),
        index_id: b"idx_test_123".to_vec(),
        op: b"CREATE_BTREE".to_vec(),
    };

    let op_applied = WalOp::IndexApplied {
        tx_id: TxId::new(10),
        index_id: b"idx_test_123".to_vec(),
        seq: 42,
    };

    let (batch, _) = wal
        .prepare_batch(vec![(op_intent, 1), (op_applied, 2)])
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
        WAL_V4_HEADER,
        "WAL file header must match MFW4 magic"
    );
}

#[tokio::test]
async fn test_wal_v4_open_index_intents_replay() {
    let tmp = TempDir::new().expect("tempdir");
    let wal_path = tmp.path().join("wal_v4_intents.log");

    let config = WalConfig {
        target_wal_version: WalVersion::V4,
        min_wal_version: WalVersion::V4,
        ..Default::default()
    };

    let wal = Wal::open_with_config(&wal_path, config).await.expect("open wal");

    let op_intent1 = WalOp::IndexIntent {
        tx_id: TxId::new(1),
        index_id: b"idx_a".to_vec(),
        op: b"BUILD".to_vec(),
    };

    let op_intent2 = WalOp::IndexIntent {
        tx_id: TxId::new(2),
        index_id: b"idx_b".to_vec(),
        op: b"BUILD".to_vec(),
    };

    let op_applied1 = WalOp::IndexApplied {
        tx_id: TxId::new(1),
        index_id: b"idx_a".to_vec(),
        seq: 10,
    };

    let (batch, _) = wal
        .prepare_batch(vec![(op_intent1, 10), (op_intent2, 11), (op_applied1, 12)])
        .await
        .expect("prepare batch");

    wal.append_batch(batch).await.expect("append batch");

    let open_intents = wal.replay_open_index_intents().await.expect("replay intents");

    assert_eq!(
        open_intents.len(),
        1,
        "Only unacknowledged idx_b intent should remain"
    );
    assert_eq!(open_intents[0].0, 11);

    if let WalOp::IndexIntent { index_id, .. } = &open_intents[0].1.op {
        assert_eq!(index_id, b"idx_b");
    } else {
        panic!("Expected IndexIntent op variant");
    }
}

#[tokio::test]
async fn test_wal_v4_downgrade_rejected() {
    let tmp = TempDir::new().expect("tempdir");
    let wal_path = tmp.path().join("wal_v4_downgrade.log");

    tokio::fs::write(&wal_path, b"MFW4_SOME_V4_CONTENT_HERE")
        .await
        .expect("write mock v4 wal");

    let config = WalConfig {
        target_wal_version: WalVersion::V3,
        min_wal_version: WalVersion::V3,
        ..Default::default()
    };

    let res = Wal::open_with_config(&wal_path, config).await;
    assert!(
        res.is_err(),
        "Opening V4 WAL with min_wal_version V3 without V4 support should be handled"
    );
}
