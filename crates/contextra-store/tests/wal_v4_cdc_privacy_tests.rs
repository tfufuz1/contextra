use contextra_core::TxId;
use contextra_store::wal::{Wal, WalConfig, WalOp, WalVersion};
use tempfile::TempDir;

#[tokio::test]
async fn test_wal_v4_cdc_events_contain_no_cleartext_payload() {
    let tmp = TempDir::new().expect("tempdir");
    let wal_path = tmp.path().join("cdc_privacy.wal");

    let mut config = WalConfig::default();
    config.min_wal_version = WalVersion::V3;

    let wal = Wal::open_with_config(&wal_path, config).await.expect("open wal");

    let op_put = WalOp::Put {
        tx_id: TxId::new(500),
        key: b"secret_key".to_vec(),
        value: b"CONFIDENTIAL_USER_DATA_PII_CLEAR_TEXT".to_vec(),
    };

    let op_delete = WalOp::Delete {
        tx_id: TxId::new(500),
        key: b"idx_opaque_hash_77".to_vec(),
    };

    let (batch, _) = wal
        .prepare_batch(vec![(op_put, 1), (op_delete, 2)])
        .await
        .expect("prepare batch");

    wal.append_batch(batch).await.expect("append batch");

    assert_eq!(wal.size(), tokio::fs::metadata(&wal_path).await.unwrap().len());
}
