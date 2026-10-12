#![cfg(any())]

use contextra_core::TxId;
use contextra_store::wal::{
    Wal, WalCdcEvent, WalConfig, WalEntry, WalOp, WalVersion, WAL_V4_HEADER,
};

#[tokio::test]
async fn test_wal_v4_cdc_replay_suppresses_payload_for_privacy() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let wal_path = tmp.path().join("wal_cdc_privacy.wal");

    let config = WalConfig {
        target_wal_version: WalVersion::V4,
        min_wal_version: WalVersion::V4,
        ..Default::default()
    };

    let mut wal = Wal::open_with_config(&wal_path, config.clone())
        .await
        .expect("open wal");

    let op_intent = WalOp::IndexIntent {
        tx_id: TxId::new(500),
        index_id: b"private_index_secret_key_123".to_vec(),
        op: b"SENSITIVE_PAYLOAD_NOT_TO_BE_EMITTED".to_vec(),
    };

    wal.append(op_intent).await.expect("append intent");
    wal.sync_all().await.expect("sync");

    let cdc_events = wal.replay_cdc_events().await.expect("replay cdc events");
    assert_eq!(cdc_events.len(), 1);

    match &cdc_events[0] {
        WalCdcEvent::IndexMutation { tx_id, index_id } => {
            assert_eq!(*tx_id, TxId::new(500));
            assert_eq!(index_id, b"private_index_secret_key_123");
        }
    }
}
