#![cfg(any())]

use contextra_core::TxId;
use contextra_store::wal::{
    Wal, WalCdcEvent, WalConfig, WalEntry, WalOp, WalVersion, WAL_V4_HEADER,
};
use std::path::Path;

#[tokio::test]
async fn test_wal_v4_golden_bytes_header_and_checksum() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let wal_path = tmp.path().join("wal_v4_golden.wal");

    let config = WalConfig {
        target_wal_version: WalVersion::V4,
        min_wal_version: WalVersion::V4,
        ..Default::default()
    };

    let mut wal = Wal::open_with_config(&wal_path, config.clone())
        .await
        .expect("open wal");

    let op_intent = WalOp::IndexIntent {
        tx_id: TxId::new(10),
        index_id: b"hnsw_vector_idx_1".to_vec(),
        op: b"CREATE_BUILD".to_vec(),
    };

    let op_applied = WalOp::IndexApplied {
        tx_id: TxId::new(10),
        index_id: b"hnsw_vector_idx_1".to_vec(),
        seq: 42,
    };

    wal.append(op_intent).await.expect("append intent");
    wal.append(op_applied).await.expect("append applied");
    wal.sync_all().await.expect("sync");

    let raw_bytes = std::fs::read(&wal_path).expect("read wal file");
    assert!(
        raw_bytes.len() >= 4,
        "WAL file must contain at least 4 bytes for magic header"
    );
    assert_eq!(
        &raw_bytes[0..4],
        &WAL_V4_HEADER,
        "WAL file magic header must match MFW4"
    );
}

#[tokio::test]
async fn test_wal_v4_replay_open_index_intents() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let wal_path = tmp.path().join("wal_v4_intents.wal");

    let config = WalConfig {
        target_wal_version: WalVersion::V4,
        min_wal_version: WalVersion::V4,
        ..Default::default()
    };

    let mut wal = Wal::open_with_config(&wal_path, config.clone())
        .await
        .expect("open wal");

    let op_intent1 = WalOp::IndexIntent {
        tx_id: TxId::new(100),
        index_id: b"idx_a".to_vec(),
        op: b"INTENT_BUILD".to_vec(),
    };
    let op_intent2 = WalOp::IndexIntent {
        tx_id: TxId::new(101),
        index_id: b"idx_b".to_vec(),
        op: b"INTENT_BUILD".to_vec(),
    };
    let op_applied1 = WalOp::IndexApplied {
        tx_id: TxId::new(100),
        index_id: b"idx_a".to_vec(),
        seq: 1,
    };

    wal.append(op_intent1).await.expect("append intent 1");
    wal.append(op_intent2).await.expect("append intent 2");
    wal.append(op_applied1).await.expect("append applied 1");
    wal.sync_all().await.expect("sync");

    let open_intents = wal.replay_open_index_intents().await.expect("replay intents");
    assert_eq!(
        open_intents.len(),
        1,
        "Only idx_b should remain open (idx_a was applied)"
    );
    assert_eq!(open_intents[0].0, TxId::new(101));

    if let WalOp::IndexIntent { index_id, .. } = &open_intents[0].1.op {
        assert_eq!(index_id, b"idx_b");
    } else {
        panic!("Expected IndexIntent op variant");
    }
}

#[tokio::test]
async fn test_wal_v4_backward_compatibility_v3() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let wal_path = tmp.path().join("wal_v3_compat.wal");

    let config_v3 = WalConfig {
        target_wal_version: WalVersion::V3,
        ..Default::default()
    };

    let mut wal = Wal::open_with_config(&wal_path, config_v3)
        .await
        .expect("open wal v3");

    wal.append(WalOp::Put {
        tx_id: TxId::new(1),
        key: b"compat_k".to_vec(),
        value: b"compat_v".to_vec(),
    })
    .await
    .expect("append");

    wal.sync_all().await.expect("sync");
    drop(wal);

    let config_v4_reader = WalConfig::default();
    let replayed = Wal::open_with_config(&wal_path, config_v4_reader)
        .await
        .expect("reopen with v4 reader");

    let (max_seq, last_hmac) = replayed.verify_and_scan().await.expect("scan");
    assert_eq!(max_seq, 1);
    assert_ne!(last_hmac, [0u8; 32]);
}
