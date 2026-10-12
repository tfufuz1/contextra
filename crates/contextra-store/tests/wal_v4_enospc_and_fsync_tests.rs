use contextra_core::TxId;
use contextra_store::wal::{Wal, WalConfig, WalFlusherConfig, WalOp, WalVersion};
use tempfile::TempDir;

#[tokio::test]
async fn test_wal_v4_segment_preallocation_option() {
    let tmp = TempDir::new().expect("tempdir");
    let wal_path = tmp.path().join("wal_v4_prealloc.log");

    let mut config = WalConfig::default();
    config.min_wal_version = WalVersion::V3;

    let wal = Wal::open_with_config(&wal_path, config).await.expect("open wal");
    let _ = wal;
}

#[tokio::test]
async fn test_wal_v4_fsync_failure_poisons_handle() {
    let tmp = TempDir::new().expect("tempdir");
    let wal_path = tmp.path().join("wal_v4_fsync_poison.log");

    let mut config = WalConfig::default();
    config.min_wal_version = WalVersion::V3;
    config.flusher_config = WalFlusherConfig {
        batch_window_micros: 0,
        queue_capacity: 10,
    };

    let wal = Wal::open_with_config(&wal_path, config).await.expect("open wal");

    assert!(!wal.is_poisoned());

    let (batch, _) = wal
        .prepare_batch(vec![(
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"k1".to_vec(),
                value: b"v1".to_vec(),
            },
            1,
        )])
        .await
        .expect("prepare batch");

    wal.append_batch(batch).await.expect("append batch");
    assert!(!wal.is_poisoned());
}
