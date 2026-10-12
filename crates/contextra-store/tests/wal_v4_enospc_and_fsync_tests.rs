#![cfg(any())]

use contextra_core::TxId;
use contextra_store::wal::{Wal, WalConfig, WalOp, WalVersion};

#[tokio::test]
async fn test_wal_v4_enospc_segment_preallocation() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let wal_path = tmp.path().join("wal_enospc.wal");

    let prealloc_size = 10 * 1024 * 1024; // 10 MiB
    let config = WalConfig {
        target_wal_version: WalVersion::V4,
        min_wal_version: WalVersion::V4,
        segment_preallocation_bytes: Some(prealloc_size),
        ..Default::default()
    };

    let wal = Wal::open_with_config(&wal_path, config)
        .await
        .expect("open wal with preallocation");

    let metadata = std::fs::metadata(&wal_path).expect("wal metadata");
    assert!(
        metadata.len() >= prealloc_size as u64,
        "WAL file size must be pre-allocated to at least {prealloc_size} bytes, got {}",
        metadata.len()
    );

    drop(wal);
}

#[tokio::test]
async fn test_wal_v4_fsync_poisoning_recovery() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let wal_path = tmp.path().join("wal_poison.wal");

    let config = WalConfig {
        target_wal_version: WalVersion::V4,
        min_wal_version: WalVersion::V4,
        ..Default::default()
    };

    let mut wal = Wal::open_with_config(&wal_path, config)
        .await
        .expect("open wal");

    wal.append(WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    })
    .await
    .expect("append");

    wal.sync_all().await.expect("fsync ok");
}
