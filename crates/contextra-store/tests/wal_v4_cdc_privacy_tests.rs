#![cfg(feature = "wal-v4-wip")]

use contextra_store::wal::{Wal, WalConfig, WalOp, WalVersion};
use contextra_types::TxId;
use tempfile::TempDir;

#[tokio::test]
async fn test_wal_v4_cdc_events_contain_no_cleartext_payload() {
    let tmp = TempDir::new().expect("tempdir");
    let wal_path = tmp.path().join("cdc_privacy.wal");

    let config = WalConfig {
        target_wal_version: WalVersion::V4,
        min_wal_version: WalVersion::V4,
        ..Default::default()
    };

    let wal = Wal::open_with_config(&wal_path, config)
        .await
        .expect("open wal");

    let op_put = WalOp::Put {
        tx_id: TxId::new(500),
        key: b"secret_key".to_vec(),
        value: b"CONFIDENTIAL_USER_DATA_PII_CLEAR_TEXT".to_vec(),
    };

    let op_intent = WalOp::IndexIntent {
        tx_id: TxId::new(500),
        index_id: b"idx_opaque_hash_77".to_vec(),
        op: b"SECRET_EMBEDDING_VECTOR_DATA_FLOATS".to_vec(),
    };

    let (batch, _) = wal
        .prepare_batch(vec![(op_put, 1), (op_intent, 2)])
        .await
        .expect("prepare batch");

    wal.append_batch(batch).await.expect("append batch");

    let cdc_events = wal.replay_cdc_events().await.expect("replay cdc events");

    assert_eq!(cdc_events.len(), 2);

    for event in &cdc_events {
        let opaque_str = String::from_utf8_lossy(&event.opaque_id);
        assert!(
            !opaque_str.contains("CONFIDENTIAL"),
            "CDC event must not contain confidential user payload"
        );
        assert!(
            !opaque_str.contains("PII"),
            "CDC event must not contain PII payload"
        );
        assert!(
            !opaque_str.contains("SECRET_EMBEDDING"),
            "CDC event must not contain vector embedding cleartext"
        );
    }
}
